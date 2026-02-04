//! # TCP Fallback Transport
//!
//! Fallback-Transport für Szenarien, in denen QUIC nicht verfügbar ist:
//! - Firewalls, die UDP blockieren
//! - Strenge NAT-Umgebungen
//! - Legacy-Infrastruktur
//!
//! ## Verwendung
//!
//! TCP wird automatisch als Fallback verwendet, wenn QUIC fehlschlägt.
//! Der HybridTransport koordiniert die Auswahl.
//!
//! ## Einschränkungen gegenüber QUIC
//!
//! - Kein 0-RTT (minimum 1 RTT für TLS)
//! - Head-of-Line Blocking bei Multiplexing
//! - Keine Connection-Migration
//!
//! ## Axiom-Referenz
//!
//! - **RL24**: TCP als Fallback wenn QUIC blockiert

use crate::domain::UniversalId;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Default TCP Port
pub const DEFAULT_TCP_PORT: u16 = 4434;

/// Connection-Timeout für TCP
pub const DEFAULT_TCP_TIMEOUT_MS: u64 = 10_000;

/// Keep-Alive für TCP
pub const DEFAULT_TCP_KEEPALIVE_MS: u64 = 30_000;

// ============================================================================
// TCP FALLBACK CONFIG
// ============================================================================

/// TCP-Fallback-Konfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpFallbackConfig {
    /// Bind-Adresse für TCP Server
    pub bind_addr: String,

    /// Connection-Timeout in Millisekunden
    pub connection_timeout_ms: u64,

    /// Read/Write-Timeout in Millisekunden
    pub io_timeout_ms: u64,

    /// Keep-Alive Interval in Millisekunden
    pub keep_alive_ms: u64,

    /// Maximum concurrent Connections
    pub max_connections: u32,

    /// TLS aktivieren
    pub enable_tls: bool,

    /// Nodelay (Nagle-Algorithmus deaktivieren)
    pub nodelay: bool,

    /// Buffer-Größe (Bytes)
    pub buffer_size: usize,
}

impl Default for TcpFallbackConfig {
    fn default() -> Self {
        Self {
            bind_addr: format!("0.0.0.0:{}", DEFAULT_TCP_PORT),
            connection_timeout_ms: DEFAULT_TCP_TIMEOUT_MS,
            io_timeout_ms: 30_000,
            keep_alive_ms: DEFAULT_TCP_KEEPALIVE_MS,
            max_connections: 100,
            enable_tls: true,
            nodelay: true, // Wichtig für Latenz
            buffer_size: 65536, // 64 KB
        }
    }
}

impl TcpFallbackConfig {
    /// Config für lokale Entwicklung
    pub fn development() -> Self {
        Self {
            bind_addr: "127.0.0.1:0".to_string(),
            enable_tls: false, // Einfacher für Dev
            ..Default::default()
        }
    }
}

// ============================================================================
// TCP FALLBACK TRANSPORT
// ============================================================================

/// TCP-Fallback-Transport
///
/// Wird verwendet wenn QUIC nicht verfügbar ist.
pub struct TcpFallbackTransport {
    /// Konfiguration
    config: TcpFallbackConfig,

    /// Aktive Connections
    connections: Arc<RwLock<HashMap<String, TcpConnectionState>>>,

    /// Metriken
    metrics: Arc<RwLock<TcpMetrics>>,
}

/// TCP Connection State
#[derive(Debug, Clone)]
/// TCP-Connection-State (v0.4.0: Mit UniversalId)
pub struct TcpConnectionState {
    /// Remote-Adresse
    pub remote_addr: SocketAddr,
    /// Verbunden seit
    pub connected_at: Instant,
    /// Letzte Aktivität
    pub last_activity: Instant,
    /// Gesendete Bytes
    pub bytes_sent: u64,
    /// Empfangene Bytes
    pub bytes_received: u64,
    /// Ist TLS aktiv
    pub is_tls: bool,
    /// UniversalId des Peers (v0.4.0)
    pub universal_id: Option<UniversalId>,
    /// Identität verifiziert (v0.4.0)
    pub identity_verified: bool,
}

/// TCP-Transport-Metriken
#[derive(Debug, Clone, Default)]
pub struct TcpMetrics {
    /// Gesamte Connections (lifetime)
    pub total_connections: u64,
    /// Aktive Connections
    pub active_connections: u64,
    /// TLS Connections
    pub tls_connections: u64,
    /// Fehlgeschlagene Connections
    pub failed_connections: u64,
    /// Gesendete Bytes
    pub bytes_sent: u64,
    /// Empfangene Bytes
    pub bytes_received: u64,
}

impl TcpFallbackTransport {
    /// Erstelle neuen TCP-Fallback-Transport
    pub fn new(config: TcpFallbackConfig) -> Self {
        Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(TcpMetrics::default())),
        }
    }

    /// Hole Config-Referenz
    pub fn config(&self) -> &TcpFallbackConfig {
        &self.config
    }

    /// Hole aktuelle Metriken
    pub fn metrics(&self) -> TcpMetrics {
        self.metrics.read().clone()
    }

    /// Anzahl aktiver Connections
    pub fn active_connections(&self) -> usize {
        self.connections.read().len()
    }

    /// Registriere neue Connection
    ///
    /// Tracking für Statistiken und Idle-Detection.
    pub fn register_connection(
        &self,
        peer_id: &str,
        remote_addr: SocketAddr,
        is_tls: bool,
        universal_id: Option<UniversalId>,
    ) -> TcpConnectionState {
        let now = Instant::now();
        let state = TcpConnectionState {
            remote_addr,
            connected_at: now,
            last_activity: now,
            bytes_sent: 0,
            bytes_received: 0,
            is_tls,
            universal_id,
            identity_verified: false,
        };

        // Update Metrics
        {
            let mut metrics = self.metrics.write();
            metrics.total_connections += 1;
            metrics.active_connections += 1;
            if is_tls {
                metrics.tls_connections += 1;
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
    pub fn find_connection_by_universal_id(&self, universal_id: &UniversalId) -> Option<(String, TcpConnectionState)> {
        self.connections
            .read()
            .iter()
            .find(|(_, conn)| conn.universal_id.as_ref() == Some(universal_id))
            .map(|(peer_id, conn)| (peer_id.clone(), conn.clone()))
    }

    /// Prüfe ob Connection für Peer existiert
    pub fn has_connection(&self, peer_id: &str) -> bool {
        self.connections.read().contains_key(peer_id)
    }

    /// Hole Connection-State
    pub fn get_connection(&self, peer_id: &str) -> Option<TcpConnectionState> {
        self.connections.read().get(peer_id).cloned()
    }

    /// Aktualisiere Connection-Statistiken
    pub fn update_stats(&self, peer_id: &str, bytes_sent: u64, bytes_received: u64) {
        if let Some(conn) = self.connections.write().get_mut(peer_id) {
            conn.bytes_sent += bytes_sent;
            conn.bytes_received += bytes_received;
            conn.last_activity = Instant::now();
        }

        let mut metrics = self.metrics.write();
        metrics.bytes_sent += bytes_sent;
        metrics.bytes_received += bytes_received;
    }

    /// Bereinige idle Connections
    pub fn cleanup_idle(&self, timeout: Duration) {
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

    /// Registriere fehlgeschlagene Connection
    pub fn register_failure(&self) {
        self.metrics.write().failed_connections += 1;
    }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/// TCP-Transport-Fehler
#[derive(Debug, thiserror::Error)]
pub enum TcpError {
    #[error("Connection failed: {reason}")]
    ConnectionFailed { reason: String },

    #[error("Timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("TLS handshake failed: {reason}")]
    TlsHandshakeFailed { reason: String },

    #[error("IO error: {reason}")]
    IoError { reason: String },

    #[error("Too many connections: {current}/{max}")]
    TooManyConnections { current: u32, max: u32 },
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
        let config = TcpFallbackConfig::default();
        assert!(config.enable_tls);
        assert!(config.nodelay);
        assert_eq!(config.max_connections, 100);
    }

    #[test]
    fn test_transport_creation() {
        let transport = TcpFallbackTransport::new(TcpFallbackConfig::default());
        assert_eq!(transport.active_connections(), 0);
    }

    #[test]
    fn test_connection_lifecycle() {
        let transport = TcpFallbackTransport::new(TcpFallbackConfig::default());

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 4434);
        transport.register_connection("peer-1", addr, true, None);

        assert!(transport.has_connection("peer-1"));
        assert_eq!(transport.active_connections(), 1);

        transport.remove_connection("peer-1");
        assert!(!transport.has_connection("peer-1"));
        assert_eq!(transport.active_connections(), 0);
    }

    #[test]
    fn test_stats_update() {
        let transport = TcpFallbackTransport::new(TcpFallbackConfig::default());

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 4434);
        transport.register_connection("peer-1", addr, false, None);

        transport.update_stats("peer-1", 500, 1000);

        let conn = transport.get_connection("peer-1").unwrap();
        assert_eq!(conn.bytes_sent, 500);
        assert_eq!(conn.bytes_received, 1000);
    }
}
