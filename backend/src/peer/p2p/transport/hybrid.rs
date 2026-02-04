//! # Hybrid Transport Manager
//!
//! Koordiniert die Auswahl zwischen QUIC und TCP basierend auf
//! Netzwerkbedingungen und Peer-Fähigkeiten.
//!
//! ## Auswahllogik
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                   TRANSPORT SELECTION                           │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │  1. Versuche QUIC (0-RTT wenn Token verfügbar)                 │
//! │     │                                                           │
//! │     ├─ Erfolg → QUIC verwenden                                  │
//! │     │                                                           │
//! │     └─ Timeout/Fehler nach 2s → TCP Fallback                   │
//! │        │                                                        │
//! │        ├─ Erfolg → TCP verwenden                                │
//! │        │                                                        │
//! │        └─ Fehler → Connection-Fehler                            │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Performance-Tracking
//!
//! Der Manager trackt die Erfolgsraten beider Protokolle und
//! passt die Timeouts entsprechend an.
//!
//! ## StateEvent-Integration (v0.4.0)
//!
//! Der HybridTransport emittiert nun StateEvents für Transport-Operationen:
//!
//! - `NetworkMetricUpdate(BytesSent/BytesReceived)` bei Datenübertragung
//! - `NetworkMetricUpdate(LatencyAvg)` bei RTT-Updates
//! - `NetworkMetricUpdate(ConnectedPeers)` bei Connection-Änderungen

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::core::state::{NetworkMetric, StateEvent, StateEventEmitter, NoOpEmitter};
use super::quic::{QuicConfig, QuicTransport};
use super::tcp_fallback::{TcpFallbackConfig, TcpFallbackTransport};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Timeout für QUIC bevor Fallback zu TCP
const QUIC_FALLBACK_TIMEOUT_MS: u64 = 2000;

/// Minimum erfolgreiche QUIC Connections vor Fallback-Disable
const MIN_QUIC_SUCCESS_FOR_DISABLE_FALLBACK: u32 = 10;

// ============================================================================
// TRANSPORT MODE
// ============================================================================

/// Aktiver Transport-Modus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportMode {
    /// QUIC aktiv (bevorzugt)
    Quic,
    /// TCP aktiv (Fallback)
    Tcp,
    /// Hybrid: Versucht QUIC, fällt zu TCP zurück
    Hybrid,
    /// Automatisch: Wählt basierend auf Erfolgsraten
    Auto,
}

impl Default for TransportMode {
    fn default() -> Self {
        Self::Hybrid
    }
}

// ============================================================================
// TRANSPORT METRICS
// ============================================================================

/// Kombinierte Transport-Metriken
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransportMetrics {
    /// QUIC-spezifische Metriken
    pub quic: QuicMetricsSummary,
    /// TCP-spezifische Metriken
    pub tcp: TcpMetricsSummary,
    /// Aktueller Modus
    pub current_mode: TransportMode,
    /// Fallback-Events
    pub fallback_events: u64,
    /// Durchschnittliche Connection-Zeit (ms)
    pub avg_connect_time_ms: u32,
}

/// QUIC-Metriken-Zusammenfassung
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuicMetricsSummary {
    pub total_connections: u64,
    pub active_connections: u64,
    pub zero_rtt_ratio: f64,
    pub success_rate: f64,
    pub avg_rtt_ms: u32,
}

/// TCP-Metriken-Zusammenfassung
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TcpMetricsSummary {
    pub total_connections: u64,
    pub active_connections: u64,
    pub success_rate: f64,
}

// ============================================================================
// HYBRID TRANSPORT
// ============================================================================

/// Hybrid Transport Manager
///
/// Koordiniert QUIC und TCP Transport mit automatischem Fallback.
/// Emittiert StateEvents für Transport-Metriken (v0.4.0).
pub struct HybridTransport {
    /// QUIC Transport
    quic: Arc<QuicTransport>,
    /// TCP Fallback Transport
    tcp: Arc<TcpFallbackTransport>,
    /// Aktueller Modus
    mode: RwLock<TransportMode>,
    /// Fallback-Timeout
    fallback_timeout: Duration,
    /// Erfolgsstatistiken
    stats: RwLock<HybridStats>,

    // ========================================================================
    // StateEvent-Integration (v0.4.0)
    // ========================================================================
    /// StateEvent-Emitter für Integration mit UnifiedState
    state_event_emitter: Arc<dyn StateEventEmitter>,
    /// Bytes gesendet (Gesamtzähler)
    total_bytes_sent: AtomicU64,
    /// Bytes empfangen (Gesamtzähler)
    total_bytes_received: AtomicU64,
    /// Letzte RTT (ms)
    last_rtt_ms: AtomicU64,
}

/// Hybrid-Transport-Statistiken
#[derive(Debug, Clone, Default)]
struct HybridStats {
    /// QUIC-Verbindungsversuche
    quic_attempts: u64,
    /// QUIC-Erfolge
    quic_successes: u64,
    /// TCP-Verbindungsversuche
    tcp_attempts: u64,
    /// TCP-Erfolge
    tcp_successes: u64,
    /// Fallback-Events
    fallback_events: u64,
    /// Letzte Modus-Änderung
    last_mode_change: Option<Instant>,
}

impl HybridTransport {
    /// Erstelle neuen Hybrid Transport
    pub fn new(quic_config: QuicConfig, tcp_config: TcpFallbackConfig) -> Self {
        Self {
            quic: Arc::new(QuicTransport::new(quic_config)),
            tcp: Arc::new(TcpFallbackTransport::new(tcp_config)),
            mode: RwLock::new(TransportMode::Hybrid),
            fallback_timeout: Duration::from_millis(QUIC_FALLBACK_TIMEOUT_MS),
            stats: RwLock::new(HybridStats::default()),
            state_event_emitter: Arc::new(NoOpEmitter),
            total_bytes_sent: AtomicU64::new(0),
            total_bytes_received: AtomicU64::new(0),
            last_rtt_ms: AtomicU64::new(0),
        }
    }

    /// Erstelle mit Standard-Konfiguration
    pub fn default_config() -> Self {
        Self::new(QuicConfig::default(), TcpFallbackConfig::default())
    }

    /// Erstelle mit StateEventEmitter (v0.4.0)
    pub fn new_with_emitter(
        quic_config: QuicConfig,
        tcp_config: TcpFallbackConfig,
        emitter: Arc<dyn StateEventEmitter>,
    ) -> Self {
        let mut transport = Self::new(quic_config, tcp_config);
        transport.state_event_emitter = emitter;
        transport
    }

    /// Setze StateEventEmitter nachträglich
    pub fn set_state_event_emitter(&mut self, emitter: Arc<dyn StateEventEmitter>) {
        self.state_event_emitter = emitter;
    }

    /// Record bytes sent and emit StateEvent (v0.4.0)
    pub fn record_bytes_sent(&self, bytes: u64) {
        let new_total = self.total_bytes_sent.fetch_add(bytes, Ordering::Relaxed) + bytes;

        self.state_event_emitter.emit(StateEvent::NetworkMetricUpdate {
            metric: NetworkMetric::BytesSent,
            value: new_total,
            delta: bytes as i64,
        });
    }

    /// Record bytes received and emit StateEvent (v0.4.0)
    pub fn record_bytes_received(&self, bytes: u64) {
        let new_total = self.total_bytes_received.fetch_add(bytes, Ordering::Relaxed) + bytes;

        self.state_event_emitter.emit(StateEvent::NetworkMetricUpdate {
            metric: NetworkMetric::BytesReceived,
            value: new_total,
            delta: bytes as i64,
        });
    }

    /// Record RTT update and emit StateEvent (v0.4.0)
    pub fn record_rtt_update(&self, rtt_ms: u64) {
        let old_rtt = self.last_rtt_ms.swap(rtt_ms, Ordering::Relaxed);

        self.state_event_emitter.emit(StateEvent::NetworkMetricUpdate {
            metric: NetworkMetric::LatencyAvg,
            value: rtt_ms,
            delta: rtt_ms as i64 - old_rtt as i64,
        });
    }

    /// Record connection change and emit StateEvent (v0.4.0)
    pub fn record_connection_change(&self, connected: bool, peer_id: &str, transport_type: &str) {
        let delta = if connected { 1 } else { -1 };

        // Hole aktuelle Anzahl von Metriken
        let metrics = self.metrics();
        let total_active = metrics.quic.active_connections + metrics.tcp.active_connections;

        self.state_event_emitter.emit(StateEvent::NetworkMetricUpdate {
            metric: NetworkMetric::ConnectedPeers,
            value: total_active,
            delta,
        });

        // Logging für Transport-Typ
        tracing::debug!(
            peer_id = peer_id,
            transport = transport_type,
            connected = connected,
            total_active = total_active,
            "Transport connection change"
        );
    }

    /// Emit transport fallback event (v0.4.0)
    fn emit_fallback_event(&self, from: &str, to: &str, reason: &str) {
        tracing::info!(
            from = from,
            to = to,
            reason = reason,
            "Transport fallback triggered"
        );

        // Increment fallback counter
        let stats = self.stats.read();
        let fallback_events = stats.fallback_events;
        drop(stats);

        // Emit StateEvent für Fallback
        self.state_event_emitter.emit(StateEvent::NetworkMetricUpdate {
            metric: NetworkMetric::TransportFallbacks,
            value: fallback_events + 1,
            delta: 1,
        });
    }

    /// Hole aktuellen Transport-Modus
    pub fn mode(&self) -> TransportMode {
        *self.mode.read()
    }

    /// Setze Transport-Modus
    pub fn set_mode(&self, mode: TransportMode) {
        let mut current = self.mode.write();
        *current = mode;
        self.stats.write().last_mode_change = Some(Instant::now());
    }

    /// Hole QUIC-Transport (für direkte Verwendung)
    pub fn quic(&self) -> &Arc<QuicTransport> {
        &self.quic
    }

    /// Hole TCP-Transport (für direkte Verwendung)
    pub fn tcp(&self) -> &Arc<TcpFallbackTransport> {
        &self.tcp
    }

    /// Hole kombinierte Metriken
    pub fn metrics(&self) -> TransportMetrics {
        let stats = self.stats.read();
        let quic_metrics = self.quic.metrics();
        let tcp_metrics = self.tcp.metrics();

        let quic_success_rate = if stats.quic_attempts > 0 {
            stats.quic_successes as f64 / stats.quic_attempts as f64
        } else {
            0.0
        };

        let tcp_success_rate = if stats.tcp_attempts > 0 {
            stats.tcp_successes as f64 / stats.tcp_attempts as f64
        } else {
            0.0
        };

        let zero_rtt_ratio = if quic_metrics.total_connections > 0 {
            quic_metrics.zero_rtt_connections as f64 / quic_metrics.total_connections as f64
        } else {
            0.0
        };

        TransportMetrics {
            quic: QuicMetricsSummary {
                total_connections: quic_metrics.total_connections,
                active_connections: quic_metrics.active_connections,
                zero_rtt_ratio,
                success_rate: quic_success_rate,
                avg_rtt_ms: quic_metrics.avg_rtt_ms,
            },
            tcp: TcpMetricsSummary {
                total_connections: tcp_metrics.total_connections,
                active_connections: tcp_metrics.active_connections,
                success_rate: tcp_success_rate,
            },
            current_mode: *self.mode.read(),
            fallback_events: stats.fallback_events,
            avg_connect_time_ms: quic_metrics.avg_rtt_ms, // Approximation
        }
    }

    /// Empfehle Transport-Protokoll für Peer
    ///
    /// Basierend auf:
    /// - Aktuellem Modus
    /// - Verfügbarkeit von 0-RTT Token
    /// - Historischen Erfolgsraten
    pub fn recommend_protocol(&self, peer_id: &str) -> TransportProtocol {
        let mode = *self.mode.read();

        match mode {
            TransportMode::Quic => TransportProtocol::Quic,
            TransportMode::Tcp => TransportProtocol::Tcp,
            TransportMode::Hybrid | TransportMode::Auto => {
                // Wenn 0-RTT Token verfügbar → QUIC bevorzugen
                if self.quic.has_0rtt_token(peer_id) {
                    return TransportProtocol::QuicZeroRtt;
                }

                // Prüfe Erfolgsraten
                let stats = self.stats.read();
                let quic_rate = if stats.quic_attempts >= 5 {
                    stats.quic_successes as f64 / stats.quic_attempts as f64
                } else {
                    0.8 // Optimistisch wenn wenige Daten
                };

                if quic_rate >= 0.5 {
                    TransportProtocol::Quic
                } else {
                    TransportProtocol::TcpPreferred
                }
            }
        }
    }

    /// Registriere erfolgreichen QUIC-Verbindungsaufbau
    pub fn record_quic_success(&self, _peer_id: &str, _is_0rtt: bool) {
        let mut stats = self.stats.write();
        stats.quic_attempts += 1;
        stats.quic_successes += 1;

        // Auto-Modus: Wenn genug QUIC-Erfolge, zu QUIC wechseln
        if *self.mode.read() == TransportMode::Auto
            && stats.quic_successes >= MIN_QUIC_SUCCESS_FOR_DISABLE_FALLBACK as u64
        {
            drop(stats);
            self.set_mode(TransportMode::Quic);
        }
    }

    /// Registriere fehlgeschlagenen QUIC-Verbindungsaufbau
    pub fn record_quic_failure(&self) {
        let mut stats = self.stats.write();
        stats.quic_attempts += 1;
    }

    /// Registriere Fallback zu TCP
    pub fn record_fallback(&self) {
        let mut stats = self.stats.write();
        stats.fallback_events += 1;
    }

    /// Registriere erfolgreichen TCP-Verbindungsaufbau
    pub fn record_tcp_success(&self) {
        let mut stats = self.stats.write();
        stats.tcp_attempts += 1;
        stats.tcp_successes += 1;
    }

    /// Registriere fehlgeschlagenen TCP-Verbindungsaufbau
    pub fn record_tcp_failure(&self) {
        let mut stats = self.stats.write();
        stats.tcp_attempts += 1;
    }

    /// Gesamtzahl aktiver Connections
    pub fn total_active_connections(&self) -> usize {
        self.quic.active_connections() + self.tcp.active_connections()
    }

    /// Bereinige idle Connections auf beiden Transports
    pub fn cleanup_idle(&self) {
        self.quic.cleanup_idle_connections();
        self.tcp.cleanup_idle(self.fallback_timeout * 2);
    }
}

/// Empfohlenes Transport-Protokoll
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportProtocol {
    /// QUIC ohne 0-RTT
    Quic,
    /// QUIC mit 0-RTT (Token verfügbar)
    QuicZeroRtt,
    /// TCP (kein QUIC)
    Tcp,
    /// TCP bevorzugt (schlechte QUIC-Erfolgsrate)
    TcpPreferred,
}

impl TransportProtocol {
    /// Ist QUIC-basiert?
    pub fn is_quic(&self) -> bool {
        matches!(self, Self::Quic | Self::QuicZeroRtt)
    }

    /// Ist TCP-basiert?
    pub fn is_tcp(&self) -> bool {
        matches!(self, Self::Tcp | Self::TcpPreferred)
    }

    /// Ist 0-RTT möglich?
    pub fn is_zero_rtt(&self) -> bool {
        matches!(self, Self::QuicZeroRtt)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_transport_creation() {
        let transport = HybridTransport::default_config();
        assert_eq!(transport.mode(), TransportMode::Hybrid);
        assert_eq!(transport.total_active_connections(), 0);
    }

    #[test]
    fn test_mode_switching() {
        let transport = HybridTransport::default_config();

        transport.set_mode(TransportMode::Quic);
        assert_eq!(transport.mode(), TransportMode::Quic);

        transport.set_mode(TransportMode::Tcp);
        assert_eq!(transport.mode(), TransportMode::Tcp);
    }

    #[test]
    fn test_protocol_recommendation() {
        let transport = HybridTransport::default_config();

        // Ohne 0-RTT Token sollte QUIC empfohlen werden (optimistisch)
        let rec = transport.recommend_protocol("unknown-peer");
        assert!(rec.is_quic());
    }

    #[test]
    fn test_stats_recording() {
        let transport = HybridTransport::default_config();

        transport.record_quic_success("peer-1", false);
        transport.record_quic_success("peer-2", true);
        transport.record_quic_failure();
        transport.record_tcp_success();

        let metrics = transport.metrics();
        assert_eq!(metrics.quic.total_connections, 0); // Keine echten Connections
        assert!(metrics.quic.success_rate > 0.0); // Stats wurden aufgezeichnet
    }

    #[test]
    fn test_fallback_recording() {
        let transport = HybridTransport::default_config();

        transport.record_fallback();
        transport.record_fallback();

        let metrics = transport.metrics();
        assert_eq!(metrics.fallback_events, 2);
    }

    #[test]
    fn test_transport_protocol_checks() {
        assert!(TransportProtocol::Quic.is_quic());
        assert!(TransportProtocol::QuicZeroRtt.is_quic());
        assert!(!TransportProtocol::Tcp.is_quic());

        assert!(TransportProtocol::Tcp.is_tcp());
        assert!(TransportProtocol::TcpPreferred.is_tcp());
        assert!(!TransportProtocol::Quic.is_tcp());

        assert!(TransportProtocol::QuicZeroRtt.is_zero_rtt());
        assert!(!TransportProtocol::Quic.is_zero_rtt());
    }
}
