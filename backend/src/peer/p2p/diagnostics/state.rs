//! Zentrale Diagnostic State - Thread-safe Zustand

use super::{
    ComponentStatus, DiagnosticEvent, EventBuffer, EventType, HealthStatus, MetricsCollector,
    NetworkMetrics,
};
use std::sync::Arc;

// ============================================================================
// DIAGNOSTIC STATE
// ============================================================================

/// Zentraler State für alle Diagnostics
///
/// Thread-safe und designed für hohe Update-Frequenzen.
pub struct DiagnosticState {
    /// Peer-ID dieses Nodes
    pub peer_id: String,

    /// Metriken-Collector (atomare Counter)
    pub metrics: Arc<MetricsCollector>,

    /// Event-Buffer (Ring-Buffer)
    pub events: Arc<EventBuffer>,

    /// Connected Peers (für Layer-Checks)
    pub connected_peers: std::sync::RwLock<Vec<PeerInfo>>,

    /// Start-Zeit für Uptime
    pub start_time: std::time::Instant,
}

/// Information über einen verbundenen Peer
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub connected_at: String,
    pub latency_ms: Option<u64>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connection_type: ConnectionType,
}

/// Verbindungstyp
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionType {
    Direct,
    Relayed,
    Holepunched,
    Unknown,
}

impl DiagnosticState {
    /// Neuen State erstellen
    pub fn new(peer_id: impl Into<String>) -> Self {
        let state = Self {
            peer_id: peer_id.into(),
            metrics: Arc::new(MetricsCollector::new()),
            events: Arc::new(EventBuffer::new(1000)),
            connected_peers: std::sync::RwLock::new(Vec::new()),
            start_time: std::time::Instant::now(),
        };

        // Start-Event loggen
        state
            .events
            .push(EventType::SwarmStarted, "Diagnostic system initialized");

        state
    }

    // ========================================================================
    // PEER MANAGEMENT
    // ========================================================================

    /// Peer als verbunden markieren
    pub fn peer_connected(&self, peer_id: impl Into<String>, conn_type: ConnectionType) {
        let peer_str = peer_id.into();

        if let Ok(mut peers) = self.connected_peers.write() {
            if !peers.iter().any(|p| p.peer_id == peer_str) {
                peers.push(PeerInfo {
                    peer_id: peer_str.clone(),
                    connected_at: chrono::Utc::now().to_rfc3339(),
                    latency_ms: None,
                    bytes_sent: 0,
                    bytes_received: 0,
                    connection_type: conn_type,
                });
            }
        }

        self.metrics.record_peer_connected();
        self.events
            .push_with_peer(EventType::PeerConnected, "Peer connected", peer_str);
    }

    /// Peer als getrennt markieren
    pub fn peer_disconnected(&self, peer_id: impl Into<String>) {
        let peer_str = peer_id.into();

        if let Ok(mut peers) = self.connected_peers.write() {
            peers.retain(|p| p.peer_id != peer_str);
        }

        self.metrics.record_peer_disconnected();
        self.events
            .push_with_peer(EventType::PeerDisconnected, "Peer disconnected", peer_str);
    }

    /// Peer entdeckt (noch nicht verbunden)
    pub fn peer_discovered(&self, peer_id: impl Into<String>) {
        self.metrics.record_peer_discovered();
        self.events.push_with_peer(
            EventType::PeerDiscovered,
            "New peer discovered",
            peer_id.into(),
        );
    }

    /// Verbindungsfehler
    pub fn connection_failed(&self, peer_id: impl Into<String>, reason: impl Into<String>) {
        self.metrics.record_connection_failed();
        self.events.push_with_details(
            EventType::ConnectionFailed,
            reason.into(),
            Some(peer_id.into()),
            None,
            None,
        );
    }

    /// Anzahl verbundener Peers
    pub fn peer_count(&self) -> usize {
        self.connected_peers.read().map(|p| p.len()).unwrap_or(0)
    }

    /// Liste verbundener Peer-IDs
    pub fn peer_ids(&self) -> Vec<String> {
        self.connected_peers
            .read()
            .map(|p| p.iter().map(|pi| pi.peer_id.clone()).collect())
            .unwrap_or_default()
    }

    // ========================================================================
    // PROTOCOL EVENTS
    // ========================================================================

    /// Gossip-Message empfangen
    pub fn gossip_received(&self, topic: impl Into<String>, from: Option<String>) {
        self.metrics.record_gossip_message();
        self.events.push_with_details(
            EventType::GossipReceived,
            "Gossip message received",
            from,
            Some(topic.into()),
            None,
        );
    }

    /// Gossip-Message gesendet
    pub fn gossip_sent(&self, topic: impl Into<String>) {
        self.metrics.record_message_sent();
        self.events.push_with_details(
            EventType::GossipSent,
            "Gossip message sent",
            None,
            Some(topic.into()),
            None,
        );
    }

    /// Request empfangen
    pub fn request_received(&self, protocol: impl Into<String>, from: impl Into<String>) {
        self.metrics.record_request_response();
        self.events.push_with_details(
            EventType::RequestReceived,
            protocol.into(),
            Some(from.into()),
            None,
            None,
        );
    }

    /// Kademlia Query
    pub fn kademlia_query(&self, query_type: impl Into<String>) {
        self.metrics.record_kademlia_query();
        self.events
            .push(EventType::KademliaQuery, format!("DHT query: {}", query_type.into()));
    }

    // ========================================================================
    // NAT EVENTS
    // ========================================================================

    /// AutoNAT Status Update
    pub fn autonat_status(&self, status: impl Into<String>) {
        self.events
            .push(EventType::AutoNatStatus, format!("NAT status: {}", status.into()));
    }

    /// DCUTR Versuch
    pub fn dcutr_attempt(&self, peer_id: impl Into<String>) {
        self.metrics.record_dcutr_attempt();
        self.events.push_with_peer(
            EventType::DcutrAttempt,
            "Holepunch attempt started",
            peer_id.into(),
        );
    }

    /// DCUTR Erfolg
    pub fn dcutr_success(&self, peer_id: impl Into<String>) {
        self.metrics.record_dcutr_success();
        self.events.push_with_peer(
            EventType::DcutrSuccess,
            "Direct connection established via holepunching",
            peer_id.into(),
        );
    }

    /// DCUTR Fehlgeschlagen
    pub fn dcutr_failed(&self, peer_id: impl Into<String>, reason: impl Into<String>) {
        self.events.push_with_details(
            EventType::DcutrFailed,
            reason.into(),
            Some(peer_id.into()),
            None,
            None,
        );
    }

    /// Relay-Reservation
    pub fn relay_reservation(&self, relay_peer: impl Into<String>) {
        self.metrics.record_relay_circuit_opened();
        self.events.push_with_peer(
            EventType::RelayReservation,
            "Relay reservation accepted",
            relay_peer.into(),
        );
    }

    /// UPnP Mapping
    pub fn upnp_mapped(&self, protocol: impl Into<String>, addr: impl Into<String>) {
        self.events.push(
            EventType::UpnpMapping,
            format!("Port mapped: {} -> {}", protocol.into(), addr.into()),
        );
    }

    // ========================================================================
    // PRIVACY EVENTS
    // ========================================================================

    /// Onion Circuit gebaut
    pub fn onion_circuit_built(&self, hops: usize) {
        self.metrics.record_onion_circuit();
        self.events
            .push(EventType::OnionCircuitBuilt, format!("{}-hop circuit built", hops));
    }

    /// Onion Circuit fehlgeschlagen
    pub fn onion_circuit_failed(&self, reason: impl Into<String>) {
        self.events
            .push(EventType::OnionCircuitFailed, reason.into());
    }

    /// Cover Traffic gesendet
    pub fn cover_traffic_sent(&self) {
        self.metrics.record_cover_traffic();
        // Nicht jedes Cover-Traffic-Event loggen (zu viel)
    }

    /// Message gemischt
    pub fn message_mixed(&self) {
        self.metrics.record_message_mixed();
        // Nicht jedes Mixing-Event loggen
    }

    // ========================================================================
    // TRAFFIC RECORDING
    // ========================================================================

    /// Bytes gesendet
    pub fn bytes_sent(&self, bytes: u64) {
        self.metrics.record_bytes_sent(bytes);
    }

    /// Bytes empfangen
    pub fn bytes_received(&self, bytes: u64) {
        self.metrics.record_bytes_received(bytes);
    }

    /// Latenz messen
    pub fn record_latency(&self, peer_id: &str, latency: std::time::Duration) {
        self.metrics.record_latency(latency);

        // Update peer info
        if let Ok(mut peers) = self.connected_peers.write() {
            if let Some(peer) = peers.iter_mut().find(|p| p.peer_id == peer_id) {
                peer.latency_ms = Some(latency.as_millis() as u64);
            }
        }
    }

    // ========================================================================
    // LOGGING
    // ========================================================================

    /// Info-Event loggen
    pub fn log_info(&self, message: impl Into<String>) {
        self.events.push(EventType::Info, message.into());
    }

    /// Warning-Event loggen
    pub fn log_warning(&self, message: impl Into<String>) {
        self.events.push(EventType::Warning, message.into());
    }

    /// Error-Event loggen
    pub fn log_error(&self, message: impl Into<String>) {
        self.events.push(EventType::Error, message.into());
    }

    // ========================================================================
    // SNAPSHOTS
    // ========================================================================

    /// Aktuelle Metriken abrufen
    pub fn get_metrics(&self) -> NetworkMetrics {
        self.metrics.snapshot()
    }

    /// Letzte Events abrufen
    pub fn get_recent_events(&self, count: usize) -> Vec<DiagnosticEvent> {
        self.events.get_recent(count)
    }

    /// Health-Status berechnen
    pub fn get_health_status(&self) -> HealthStatus {
        let peer_count = self.peer_count();
        let metrics = self.get_metrics();

        let (status, message) = if peer_count >= 3 {
            (ComponentStatus::Healthy, format!("Connected to {} peers", peer_count))
        } else if peer_count > 0 {
            (
                ComponentStatus::Degraded,
                format!("Only {} peer(s) - network degraded", peer_count),
            )
        } else {
            (
                ComponentStatus::Unavailable,
                "No peers connected - network isolated".to_string(),
            )
        };

        // Zusätzliche Checks
        let mut healthy_layers = 0u8;
        let total_layers = 8u8;

        // Simplified layer health check
        if peer_count > 0 {
            healthy_layers += 4; // Transport, Identity, Discovery, Application
        } else {
            healthy_layers += 2; // Transport, Identity always work
        }

        #[cfg(feature = "privacy")]
        {
            healthy_layers += 4; // Performance, Privacy, NAT, Censorship
        }

        #[cfg(not(feature = "privacy"))]
        {
            healthy_layers += 1; // Only NAT
        }

        let final_status = if metrics.failed_connections > metrics.connected_peers * 2 {
            ComponentStatus::Degraded
        } else {
            status
        };

        HealthStatus {
            status: final_status,
            healthy_layers,
            total_layers,
            message,
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_state() {
        let state = DiagnosticState::new("test-peer-123");

        state.peer_connected("peer-1", ConnectionType::Direct);
        state.peer_connected("peer-2", ConnectionType::Relayed);

        assert_eq!(state.peer_count(), 2);
        assert_eq!(state.peer_ids().len(), 2);

        state.peer_disconnected("peer-1");
        assert_eq!(state.peer_count(), 1);
    }

    #[test]
    fn test_health_status() {
        let state = DiagnosticState::new("test-peer");

        // Keine Peers = Unavailable
        let health = state.get_health_status();
        assert_eq!(health.status, ComponentStatus::Unavailable);

        // 1 Peer = Degraded
        state.peer_connected("peer-1", ConnectionType::Direct);
        let health = state.get_health_status();
        assert_eq!(health.status, ComponentStatus::Degraded);

        // 3 Peers = Healthy
        state.peer_connected("peer-2", ConnectionType::Direct);
        state.peer_connected("peer-3", ConnectionType::Direct);
        let health = state.get_health_status();
        assert_eq!(health.status, ComponentStatus::Healthy);
    }
}
