//! # QUIC Transport Layer (RL24)
//!
//! Hochperformanter QUIC-Transport mit 0-RTT Connection Setup
//! und nativer Stream-Multiplexierung.
//!
//! ## Features
//!
//! - **0-RTT Resumption**: Verbindungsaufbau in 0 Round-Trips (bei bekannten Peers)
//! - **Multi-Stream**: Parallele Streams ohne Head-of-Line Blocking
//! - **Connection Migration**: Nahtloser Wechsel bei IP-Änderung (Mobile)
//! - **Built-in Encryption**: TLS 1.3 integriert
//!
//! ## Performance-Ziele (V2.6)
//!
//! | Metrik               | Target        | Grund                    |
//! |----------------------|---------------|--------------------------|
//! | First-Message-Latency| < 30ms        | 0-RTT + lokale Verarbeitung |
//! | Circuit-Build-Time   | < 50ms        | 0-RTT für alle Hops      |
//! | Streams/Connection   | 100+          | Multi-Circuit Support    |
//!
//! ## Axiom-Referenzen
//!
//! - **RL24**: QUIC Transport (0-RTT Circuit-Setup)
//! - **RL17**: Saga-Latenz-Optimierung (schnelle Connection)
//!
//! ## Beispiel
//!
//! ```rust,ignore
//! use erynoa_api::peer::p2p::transport::quic::{QuicTransport, QuicConfig};
//!
//! let config = QuicConfig::default();
//! let transport = QuicTransport::new(config).await?;
//!
//! // 0-RTT Connection (bei bekanntem Peer)
//! let conn = transport.connect_0rtt(&peer_addr).await?;
//! let stream = conn.open_bi().await?;
//! ```

use crate::domain::UniversalId;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Default QUIC Port
pub const DEFAULT_QUIC_PORT: u16 = 4433;

/// Maximum Idle-Timeout für Connections
pub const DEFAULT_IDLE_TIMEOUT_MS: u64 = 30_000; // 30 Sekunden

/// Maximum Streams pro Connection
pub const DEFAULT_MAX_STREAMS: u32 = 100;

/// Keep-Alive Interval
pub const DEFAULT_KEEP_ALIVE_MS: u64 = 15_000; // 15 Sekunden

/// 0-RTT Token Expiry
pub const ZERO_RTT_TOKEN_EXPIRY_SECS: u64 = 86400; // 24 Stunden

// ============================================================================
// QUIC CONFIG
// ============================================================================

/// QUIC-Transport-Konfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuicConfig {
    /// Bind-Adresse für QUIC Server
    pub bind_addr: String,

    /// Idle-Timeout in Millisekunden
    pub idle_timeout_ms: u64,

    /// Maximum concurrent bi-directional streams
    pub max_bi_streams: u32,

    /// Maximum concurrent uni-directional streams
    pub max_uni_streams: u32,

    /// Keep-Alive Interval in Millisekunden
    pub keep_alive_ms: u64,

    /// 0-RTT aktivieren (für bekannte Peers)
    pub enable_0rtt: bool,

    /// Maximum Datagram-Größe (MTU)
    pub max_datagram_size: u16,

    /// Connection-Migration aktivieren (für Mobile)
    pub enable_migration: bool,

    /// TLS Certificate Path (optional, für Production)
    pub cert_path: Option<String>,

    /// TLS Key Path (optional, für Production)
    pub key_path: Option<String>,
}

impl Default for QuicConfig {
    fn default() -> Self {
        Self {
            bind_addr: format!("0.0.0.0:{}", DEFAULT_QUIC_PORT),
            idle_timeout_ms: DEFAULT_IDLE_TIMEOUT_MS,
            max_bi_streams: DEFAULT_MAX_STREAMS,
            max_uni_streams: 10,
            keep_alive_ms: DEFAULT_KEEP_ALIVE_MS,
            enable_0rtt: true,
            max_datagram_size: 1350, // Safe for most networks
            enable_migration: true,
            cert_path: None,
            key_path: None,
        }
    }
}

impl QuicConfig {
    /// Erstelle Config für Development (self-signed certs)
    pub fn development() -> Self {
        Self {
            bind_addr: "127.0.0.1:0".to_string(),
            enable_0rtt: true,
            enable_migration: false, // Nicht nötig in lokaler Entwicklung
            ..Default::default()
        }
    }

    /// Erstelle Config für Production
    pub fn production(cert_path: &str, key_path: &str) -> Self {
        Self {
            cert_path: Some(cert_path.to_string()),
            key_path: Some(key_path.to_string()),
            enable_0rtt: true,
            enable_migration: true,
            ..Default::default()
        }
    }

    /// Erstelle Config für Mobile
    pub fn mobile() -> Self {
        Self {
            idle_timeout_ms: 60_000,      // Längeres Timeout
            keep_alive_ms: 10_000,        // Häufigere Keep-Alives
            enable_migration: true,       // Wichtig für Mobile
            max_bi_streams: 50,           // Weniger Streams (Battery)
            max_datagram_size: 1200,      // Kleiner für Mobile Networks
            ..Default::default()
        }
    }
}

// ============================================================================
// QUIC TRANSPORT
// ============================================================================

/// QUIC Transport Layer
///
/// Verwaltet QUIC-Verbindungen mit 0-RTT Support und
/// automatischer Connection-Migration.
pub struct QuicTransport {
    /// Konfiguration
    config: QuicConfig,

    /// Aktive Connections (Peer-ID → Connection)
    connections: Arc<RwLock<HashMap<String, ConnectionState>>>,

    /// 0-RTT Token Cache (für Resumption)
    zero_rtt_cache: Arc<RwLock<HashMap<String, ZeroRttToken>>>,

    /// Metrics
    metrics: Arc<RwLock<TransportMetrics>>,

    /// Shutdown-Signal
    shutdown_tx: Option<mpsc::Sender<()>>,
}

/// Connection-State
#[derive(Debug, Clone)]
/// Connection-State für einen Peer (v0.4.0: Mit UniversalId)
pub struct ConnectionState {
    /// Remote-Adresse
    pub remote_addr: SocketAddr,
    /// Verbunden seit
    pub connected_at: Instant,
    /// Letzte Aktivität
    pub last_activity: Instant,
    /// Offene Streams
    pub open_streams: u32,
    /// Gesendete Bytes
    pub bytes_sent: u64,
    /// Empfangene Bytes
    pub bytes_received: u64,
    /// RTT in Millisekunden
    pub rtt_ms: u32,
    /// Ist 0-RTT Connection
    pub is_0rtt: bool,
    /// UniversalId des Peers (v0.4.0)
    pub universal_id: Option<UniversalId>,
    /// Identität verifiziert (v0.4.0)
    pub identity_verified: bool,
}

/// 0-RTT Resumption Token
#[derive(Debug, Clone)]
pub struct ZeroRttToken {
    /// Token-Daten
    pub token: Vec<u8>,
    /// Server-Name
    pub server_name: String,
    /// Erstellt am
    pub created_at: Instant,
    /// Ablaufzeit
    pub expires_at: Instant,
}

impl ZeroRttToken {
    /// Prüfe ob Token noch gültig ist
    pub fn is_valid(&self) -> bool {
        Instant::now() < self.expires_at
    }
}

/// Transport-Metriken
#[derive(Debug, Clone, Default)]
pub struct TransportMetrics {
    /// Gesamte Connections (lifetime)
    pub total_connections: u64,
    /// Aktive Connections
    pub active_connections: u64,
    /// 0-RTT Connections (erfolgreich)
    pub zero_rtt_connections: u64,
    /// 1-RTT Connections (Fallback)
    pub one_rtt_connections: u64,
    /// Fehlgeschlagene Connections
    pub failed_connections: u64,
    /// Gesendete Bytes (lifetime)
    pub bytes_sent: u64,
    /// Empfangene Bytes (lifetime)
    pub bytes_received: u64,
    /// Durchschnittliche RTT (ms)
    pub avg_rtt_ms: u32,
    /// Connection-Migrationen
    pub migrations: u64,
}

impl QuicTransport {
    /// Erstelle neuen QUIC Transport
    ///
    /// **Hinweis**: In Phase 1 wird nur die Struktur erstellt.
    /// Die tatsächliche QUIC-Implementation (quinn) wird in Phase 1b integriert.
    pub fn new(config: QuicConfig) -> Self {
        Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            zero_rtt_cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(TransportMetrics::default())),
            shutdown_tx: None,
        }
    }

    /// Hole Config-Referenz
    pub fn config(&self) -> &QuicConfig {
        &self.config
    }

    /// Hole aktuelle Metriken
    pub fn metrics(&self) -> TransportMetrics {
        self.metrics.read().clone()
    }

    /// Anzahl aktiver Connections
    pub fn active_connections(&self) -> usize {
        self.connections.read().len()
    }

    /// Prüfe ob 0-RTT für Peer verfügbar ist
    pub fn has_0rtt_token(&self, peer_id: &str) -> bool {
        self.zero_rtt_cache
            .read()
            .get(peer_id)
            .map(|t| t.is_valid())
            .unwrap_or(false)
    }

    /// Speichere 0-RTT Token für Peer
    pub fn store_0rtt_token(&self, peer_id: &str, token: Vec<u8>, server_name: &str) {
        let now = Instant::now();
        let token = ZeroRttToken {
            token,
            server_name: server_name.to_string(),
            created_at: now,
            expires_at: now + Duration::from_secs(ZERO_RTT_TOKEN_EXPIRY_SECS),
        };
        self.zero_rtt_cache
            .write()
            .insert(peer_id.to_string(), token);
    }

    /// Hole 0-RTT Token für Peer (wenn verfügbar und gültig)
    pub fn get_0rtt_token(&self, peer_id: &str) -> Option<Vec<u8>> {
        self.zero_rtt_cache.read().get(peer_id).and_then(|t| {
            if t.is_valid() {
                Some(t.token.clone())
            } else {
                None
            }
        })
    }

    /// Bereinige abgelaufene 0-RTT Tokens
    pub fn cleanup_expired_tokens(&self) {
        let mut cache = self.zero_rtt_cache.write();
        cache.retain(|_, token| token.is_valid());
    }

    /// Registriere neue Connection
    ///
    /// Tracking für Statistiken und Idle-Detection.
    pub fn register_connection(
        &self,
        peer_id: &str,
        remote_addr: SocketAddr,
        is_0rtt: bool,
        universal_id: Option<UniversalId>,
    ) -> ConnectionState {
        let now = Instant::now();
        let state = ConnectionState {
            remote_addr,
            connected_at: now,
            last_activity: now,
            open_streams: 0,
            bytes_sent: 0,
            bytes_received: 0,
            rtt_ms: 0,
            is_0rtt,
            universal_id,
            identity_verified: false,
        };

        // Update Metrics
        {
            let mut metrics = self.metrics.write();
            metrics.total_connections += 1;
            metrics.active_connections += 1;
            if is_0rtt {
                metrics.zero_rtt_connections += 1;
            } else {
                metrics.one_rtt_connections += 1;
            }
        }

        self.connections
            .write()
            .insert(peer_id.to_string(), state.clone());
        state
    }

    /// Entferne Connection
    pub fn remove_connection(&self, peer_id: &str) {
        if self.connections.write().remove(peer_id).is_some() {
            let mut metrics = self.metrics.write();
            if metrics.active_connections > 0 {
                metrics.active_connections -= 1;
            }
        }
    }

    /// Setze UniversalId für Connection nach Verifikation (v0.4.0)
    pub fn set_connection_identity(
        &self,
        peer_id: &str,
        universal_id: UniversalId,
        verified: bool,
    ) {
        if let Some(conn) = self.connections.write().get_mut(peer_id) {
            conn.universal_id = Some(universal_id);
            conn.identity_verified = verified;
        }
    }

    /// Finde Connection nach UniversalId (v0.4.0)
    pub fn find_connection_by_universal_id(&self, universal_id: &UniversalId) -> Option<(String, ConnectionState)> {
        self.connections
            .read()
            .iter()
            .find(|(_, conn)| conn.universal_id.as_ref() == Some(universal_id))
            .map(|(peer_id, conn)| (peer_id.clone(), conn.clone()))
    }

    /// Anzahl verifizierter Connections (v0.4.0)
    pub fn verified_connections_count(&self) -> usize {
        self.connections
            .read()
            .values()
            .filter(|conn| conn.identity_verified)
            .count()
    }

    /// Aktualisiere Connection-Statistiken
    pub fn update_connection_stats(
        &self,
        peer_id: &str,
        bytes_sent: u64,
        bytes_received: u64,
        rtt_ms: u32,
    ) {
        if let Some(conn) = self.connections.write().get_mut(peer_id) {
            conn.bytes_sent += bytes_sent;
            conn.bytes_received += bytes_received;
            conn.rtt_ms = rtt_ms;
            conn.last_activity = Instant::now();
        }

        // Global Metrics
        let mut metrics = self.metrics.write();
        metrics.bytes_sent += bytes_sent;
        metrics.bytes_received += bytes_received;
        // Einfache RTT-Mittelung (TODO: Exponential Smoothing)
        if rtt_ms > 0 && metrics.avg_rtt_ms > 0 {
            metrics.avg_rtt_ms = (metrics.avg_rtt_ms + rtt_ms) / 2;
        } else if rtt_ms > 0 {
            metrics.avg_rtt_ms = rtt_ms;
        }
    }

    /// Hole Connection-State für Peer
    pub fn get_connection(&self, peer_id: &str) -> Option<ConnectionState> {
        self.connections.read().get(peer_id).cloned()
    }

    /// Bereinige idle Connections
    pub fn cleanup_idle_connections(&self) {
        let timeout = Duration::from_millis(self.config.idle_timeout_ms);
        let now = Instant::now();

        let mut connections = self.connections.write();
        let mut metrics = self.metrics.write();

        connections.retain(|_, conn| {
            let is_active = now.duration_since(conn.last_activity) < timeout;
            if !is_active && metrics.active_connections > 0 {
                metrics.active_connections -= 1;
            }
            is_active
        });
    }

    /// Registriere Connection-Migration
    pub fn register_migration(&self, peer_id: &str, new_addr: SocketAddr) {
        if let Some(conn) = self.connections.write().get_mut(peer_id) {
            conn.remote_addr = new_addr;
            conn.last_activity = Instant::now();
        }

        self.metrics.write().migrations += 1;
    }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/// QUIC-Transport-Fehler
#[derive(Debug, thiserror::Error)]
pub enum QuicError {
    #[error("Connection failed: {reason}")]
    ConnectionFailed { reason: String },

    #[error("Stream error: {reason}")]
    StreamError { reason: String },

    #[error("Timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Connection closed by peer")]
    ConnectionClosed,

    #[error("Invalid configuration: {reason}")]
    InvalidConfig { reason: String },

    #[error("TLS error: {reason}")]
    TlsError { reason: String },

    #[error("Address parse error: {addr}")]
    AddressParseError { addr: String },

    #[error("0-RTT rejected by server")]
    ZeroRttRejected,

    #[error("Migration failed: {reason}")]
    MigrationFailed { reason: String },
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_config_defaults() {
        let config = QuicConfig::default();
        assert!(config.enable_0rtt);
        assert!(config.enable_migration);
        assert_eq!(config.max_bi_streams, DEFAULT_MAX_STREAMS);
    }

    #[test]
    fn test_config_development() {
        let config = QuicConfig::development();
        assert!(config.enable_0rtt);
        assert!(!config.enable_migration);
        assert!(config.bind_addr.starts_with("127.0.0.1"));
    }

    #[test]
    fn test_config_mobile() {
        let config = QuicConfig::mobile();
        assert!(config.enable_migration);
        assert_eq!(config.max_bi_streams, 50);
        assert_eq!(config.max_datagram_size, 1200);
    }

    #[test]
    fn test_transport_creation() {
        let transport = QuicTransport::new(QuicConfig::default());
        assert_eq!(transport.active_connections(), 0);
        assert!(!transport.has_0rtt_token("test-peer"));
    }

    #[test]
    fn test_zero_rtt_token_storage() {
        let transport = QuicTransport::new(QuicConfig::default());

        // Speichere Token
        transport.store_0rtt_token("peer-1", vec![1, 2, 3], "relay.example.com");

        // Token sollte verfügbar sein
        assert!(transport.has_0rtt_token("peer-1"));
        assert!(!transport.has_0rtt_token("peer-2"));

        // Token abrufen
        let token = transport.get_0rtt_token("peer-1");
        assert!(token.is_some());
        assert_eq!(token.unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_connection_registration() {
        let transport = QuicTransport::new(QuicConfig::default());

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 4433);
        let state = transport.register_connection("peer-1", addr, true, None);

        assert!(state.is_0rtt);
        assert_eq!(state.remote_addr, addr);
        assert_eq!(transport.active_connections(), 1);

        // Metriken prüfen
        let metrics = transport.metrics();
        assert_eq!(metrics.total_connections, 1);
        assert_eq!(metrics.active_connections, 1);
        assert_eq!(metrics.zero_rtt_connections, 1);
    }

    #[test]
    fn test_connection_removal() {
        let transport = QuicTransport::new(QuicConfig::default());

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 4433);
        transport.register_connection("peer-1", addr, false, None);
        assert_eq!(transport.active_connections(), 1);

        transport.remove_connection("peer-1");
        assert_eq!(transport.active_connections(), 0);

        let metrics = transport.metrics();
        assert_eq!(metrics.active_connections, 0);
        assert_eq!(metrics.one_rtt_connections, 1);
    }

    #[test]
    fn test_stats_update() {
        let transport = QuicTransport::new(QuicConfig::default());

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 4433);
        transport.register_connection("peer-1", addr, false, None);

        transport.update_connection_stats("peer-1", 1000, 2000, 50);

        let conn = transport.get_connection("peer-1").unwrap();
        assert_eq!(conn.bytes_sent, 1000);
        assert_eq!(conn.bytes_received, 2000);
        assert_eq!(conn.rtt_ms, 50);

        let metrics = transport.metrics();
        assert_eq!(metrics.bytes_sent, 1000);
        assert_eq!(metrics.bytes_received, 2000);
    }

    #[test]
    fn test_migration_registration() {
        let transport = QuicTransport::new(QuicConfig::default());

        let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 4433);
        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), 4433);

        transport.register_connection("peer-1", addr1, false, None);
        transport.register_migration("peer-1", addr2);

        let conn = transport.get_connection("peer-1").unwrap();
        assert_eq!(conn.remote_addr, addr2);

        let metrics = transport.metrics();
        assert_eq!(metrics.migrations, 1);
    }
}
