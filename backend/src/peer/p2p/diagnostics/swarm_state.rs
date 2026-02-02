//! # Live Swarm State - Echte Laufzeit-Daten vom Swarm
//!
//! Diese Datei enthält den `SwarmState` der echte Daten vom laufenden
//! libp2p Swarm sammelt und für Diagnostics bereitstellt.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                       SWARM EVENT LOOP                              │
//! │                                                                     │
//! │   SwarmEvent ──────────────────────────────────────────────────┐   │
//! │       │                                                        │   │
//! │       ▼                                                        ▼   │
//! │   ┌─────────┐    ┌─────────────┐    ┌─────────────────────────┐   │
//! │   │ Swarm   │───▶│ SwarmState  │◀───│ DiagnosticState         │   │
//! │   │ Actions │    │ (Arc<_>)    │    │ (merges SwarmState)     │   │
//! │   └─────────┘    └─────────────┘    └─────────────────────────┘   │
//! │                         │                      │                   │
//! │                         ▼                      ▼                   │
//! │                  ┌─────────────────────────────────────┐          │
//! │                  │         HTTP API / SSE              │          │
//! │                  │   Real metrics from live Swarm      │          │
//! │                  └─────────────────────────────────────┘          │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::RwLock;
use std::time::Instant;

// ============================================================================
// NAT STATUS
// ============================================================================

/// AutoNAT Status (echte Werte aus libp2p)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NatStatus {
    /// NAT-Status noch nicht ermittelt
    #[default]
    Unknown,
    /// Öffentlich erreichbar (kein NAT oder Port Forwarding)
    Public,
    /// Hinter NAT, nicht direkt erreichbar
    Private,
}

impl std::fmt::Display for NatStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "Unknown"),
            Self::Public => write!(f, "Public"),
            Self::Private => write!(f, "Private"),
        }
    }
}

// ============================================================================
// PEER INFO
// ============================================================================

/// Detaillierte Info über einen verbundenen Peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivePeerInfo {
    pub peer_id: String,
    pub connected_at: String,
    /// Letzte gemessene RTT in ms
    pub ping_rtt_ms: Option<u64>,
    /// Addresse über die wir verbunden sind
    pub address: Option<String>,
    /// Agent Version (aus Identify)
    pub agent_version: Option<String>,
    /// Protokoll-Version (aus Identify)
    pub protocol_version: Option<String>,
    /// Ist die Verbindung über Relay?
    pub is_relayed: bool,
    /// Ist in unserem Gossipsub Mesh?
    pub in_gossip_mesh: bool,
    /// Ist in Kademlia Routing Table?
    pub in_kademlia: bool,
}

// ============================================================================
// SWARM STATE
// ============================================================================

/// Live State vom laufenden Swarm
///
/// Thread-safe und für hohe Update-Frequenzen designed.
/// Wird direkt vom Swarm Event-Loop befüllt.
pub struct SwarmState {
    // ========================================================================
    // Identity
    // ========================================================================
    /// Eigene Peer-ID
    pub peer_id: String,

    // ========================================================================
    // NAT Status (von AutoNAT)
    // ========================================================================
    /// Aktueller NAT-Status
    nat_status: RwLock<NatStatus>,

    /// Bestätigte externe Adressen
    external_addresses: RwLock<Vec<String>>,

    // ========================================================================
    // Relay Status
    // ========================================================================
    /// Haben wir eine aktive Relay-Reservation?
    pub has_relay_reservation: AtomicBool,

    /// Anzahl aktiver Relay-Circuits (als Server)
    pub relay_circuits_serving: AtomicUsize,

    /// Relay-Peer über den wir erreichbar sind
    relay_peer: RwLock<Option<String>>,

    // ========================================================================
    // DCUTR Status
    // ========================================================================
    /// Erfolgreiche Holepunch-Verbindungen
    pub dcutr_successes: AtomicUsize,

    /// Fehlgeschlagene Holepunch-Versuche
    pub dcutr_failures: AtomicUsize,

    // ========================================================================
    // Discovery
    // ========================================================================
    /// Anzahl Peers in Kademlia Routing Table
    pub kademlia_routing_table_size: AtomicUsize,

    /// Bootstrap abgeschlossen?
    pub kademlia_bootstrap_complete: AtomicBool,

    /// Anzahl gespeicherter DHT Records
    pub dht_records_stored: AtomicUsize,

    /// mDNS aktiv?
    pub mdns_active: AtomicBool,

    /// Über mDNS entdeckte Peers
    pub mdns_discovered_count: AtomicUsize,

    // ========================================================================
    // Gossipsub
    // ========================================================================
    /// Peers in unserem Gossipsub Mesh
    pub gossip_mesh_size: AtomicUsize,

    /// Anzahl Topics auf die wir subscribed sind
    pub gossip_topics_subscribed: AtomicUsize,

    /// Empfangene Gossip-Messages
    pub gossip_messages_received: AtomicU64,

    /// Gesendete Gossip-Messages
    pub gossip_messages_sent: AtomicU64,

    // ========================================================================
    // Connection Stats
    // ========================================================================
    /// Aktuell verbundene Peers
    pub connected_peers_count: AtomicUsize,

    /// Verbindungsfehler
    pub connection_errors: AtomicUsize,

    /// Eingehende Verbindungen
    pub inbound_connections: AtomicUsize,

    /// Ausgehende Verbindungen
    pub outbound_connections: AtomicUsize,

    // ========================================================================
    // Ping Stats
    // ========================================================================
    /// Summe aller Ping RTTs (für Durchschnitt)
    ping_rtt_sum_us: AtomicU64,

    /// Anzahl Ping-Messungen
    ping_count: AtomicUsize,

    /// Minimale RTT in µs
    ping_min_us: AtomicU64,

    /// Maximale RTT in µs
    ping_max_us: AtomicU64,

    // ========================================================================
    // UPnP
    // ========================================================================
    /// UPnP Gateway gefunden?
    pub upnp_available: AtomicBool,

    /// Über UPnP gemappte Adresse
    upnp_external_addr: RwLock<Option<String>>,

    // ========================================================================
    // Peer Details
    // ========================================================================
    /// Detaillierte Infos pro Peer
    peers: RwLock<HashMap<String, LivePeerInfo>>,

    // ========================================================================
    // Timing
    // ========================================================================
    /// Start-Zeit des Swarms
    pub start_time: Instant,
}

impl SwarmState {
    /// Neuen SwarmState erstellen
    pub fn new(peer_id: impl Into<String>) -> Self {
        Self {
            peer_id: peer_id.into(),

            // NAT
            nat_status: RwLock::new(NatStatus::Unknown),
            external_addresses: RwLock::new(Vec::new()),

            // Relay
            has_relay_reservation: AtomicBool::new(false),
            relay_circuits_serving: AtomicUsize::new(0),
            relay_peer: RwLock::new(None),

            // DCUTR
            dcutr_successes: AtomicUsize::new(0),
            dcutr_failures: AtomicUsize::new(0),

            // Discovery
            kademlia_routing_table_size: AtomicUsize::new(0),
            kademlia_bootstrap_complete: AtomicBool::new(false),
            dht_records_stored: AtomicUsize::new(0),
            mdns_active: AtomicBool::new(false),
            mdns_discovered_count: AtomicUsize::new(0),

            // Gossipsub
            gossip_mesh_size: AtomicUsize::new(0),
            gossip_topics_subscribed: AtomicUsize::new(0),
            gossip_messages_received: AtomicU64::new(0),
            gossip_messages_sent: AtomicU64::new(0),

            // Connections
            connected_peers_count: AtomicUsize::new(0),
            connection_errors: AtomicUsize::new(0),
            inbound_connections: AtomicUsize::new(0),
            outbound_connections: AtomicUsize::new(0),

            // Ping
            ping_rtt_sum_us: AtomicU64::new(0),
            ping_count: AtomicUsize::new(0),
            ping_min_us: AtomicU64::new(u64::MAX),
            ping_max_us: AtomicU64::new(0),

            // UPnP
            upnp_available: AtomicBool::new(false),
            upnp_external_addr: RwLock::new(None),

            // Peers
            peers: RwLock::new(HashMap::new()),

            // Timing
            start_time: Instant::now(),
        }
    }

    // ========================================================================
    // NAT STATUS UPDATES
    // ========================================================================

    /// NAT-Status aktualisieren (von AutoNAT Event)
    pub fn set_nat_status(&self, status: NatStatus) {
        if let Ok(mut s) = self.nat_status.write() {
            *s = status;
        }
    }

    /// NAT-Status abfragen
    pub fn get_nat_status(&self) -> NatStatus {
        self.nat_status.read().map(|s| *s).unwrap_or_default()
    }

    /// Externe Adresse hinzufügen
    pub fn add_external_address(&self, addr: impl Into<String>) {
        if let Ok(mut addrs) = self.external_addresses.write() {
            let addr = addr.into();
            if !addrs.contains(&addr) {
                addrs.push(addr);
            }
        }
    }

    /// Externe Adressen abfragen
    pub fn get_external_addresses(&self) -> Vec<String> {
        self.external_addresses
            .read()
            .map(|a| a.clone())
            .unwrap_or_default()
    }

    // ========================================================================
    // RELAY UPDATES
    // ========================================================================

    /// Relay-Reservation erhalten
    pub fn relay_reservation_accepted(&self, relay_peer: impl Into<String>) {
        self.has_relay_reservation.store(true, Ordering::SeqCst);
        if let Ok(mut r) = self.relay_peer.write() {
            *r = Some(relay_peer.into());
        }
    }

    /// Relay-Circuit geöffnet (als Server)
    pub fn relay_circuit_opened(&self) {
        self.relay_circuits_serving.fetch_add(1, Ordering::SeqCst);
    }

    /// Relay-Circuit geschlossen (als Server)
    pub fn relay_circuit_closed(&self) {
        // Dekrement nur wenn > 0
        let _ = self
            .relay_circuits_serving
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                if v > 0 {
                    Some(v - 1)
                } else {
                    Some(0)
                }
            });
    }

    // ========================================================================
    // DCUTR UPDATES
    // ========================================================================

    /// Holepunch erfolgreich
    pub fn dcutr_success(&self) {
        self.dcutr_successes.fetch_add(1, Ordering::SeqCst);
    }

    /// Holepunch fehlgeschlagen
    pub fn dcutr_failure(&self) {
        self.dcutr_failures.fetch_add(1, Ordering::SeqCst);
    }

    /// DCUTR Erfolgsrate in Prozent
    pub fn dcutr_success_rate(&self) -> f64 {
        let successes = self.dcutr_successes.load(Ordering::Relaxed);
        let failures = self.dcutr_failures.load(Ordering::Relaxed);
        let total = successes + failures;
        if total == 0 {
            100.0 // Keine Versuche = OK
        } else {
            (successes as f64 / total as f64) * 100.0
        }
    }

    // ========================================================================
    // DISCOVERY UPDATES
    // ========================================================================

    /// Kademlia Bootstrap abgeschlossen
    pub fn kademlia_bootstrap_done(&self) {
        self.kademlia_bootstrap_complete
            .store(true, Ordering::SeqCst);
    }

    /// Kademlia Routing Table Update
    pub fn set_kademlia_routing_table_size(&self, size: usize) {
        self.kademlia_routing_table_size
            .store(size, Ordering::SeqCst);
    }

    /// mDNS Peer entdeckt
    pub fn mdns_peer_discovered(&self) {
        self.mdns_active.store(true, Ordering::SeqCst);
        self.mdns_discovered_count.fetch_add(1, Ordering::SeqCst);
    }

    // ========================================================================
    // GOSSIPSUB UPDATES
    // ========================================================================

    /// Gossipsub Mesh Size setzen
    pub fn set_gossip_mesh_size(&self, size: usize) {
        self.gossip_mesh_size.store(size, Ordering::SeqCst);
    }

    /// Gossip Message empfangen
    pub fn gossip_message_received(&self) {
        self.gossip_messages_received.fetch_add(1, Ordering::SeqCst);
    }

    /// Gossip Message gesendet
    pub fn gossip_message_sent(&self) {
        self.gossip_messages_sent.fetch_add(1, Ordering::SeqCst);
    }

    /// Topic subscribed
    pub fn gossip_topic_subscribed(&self) {
        self.gossip_topics_subscribed.fetch_add(1, Ordering::SeqCst);
    }

    // ========================================================================
    // CONNECTION UPDATES
    // ========================================================================

    /// Peer verbunden
    pub fn peer_connected(&self, peer_id: impl Into<String>, is_inbound: bool, is_relayed: bool) {
        let peer_str = peer_id.into();
        self.connected_peers_count.fetch_add(1, Ordering::SeqCst);

        if is_inbound {
            self.inbound_connections.fetch_add(1, Ordering::SeqCst);
        } else {
            self.outbound_connections.fetch_add(1, Ordering::SeqCst);
        }

        if let Ok(mut peers) = self.peers.write() {
            peers.insert(
                peer_str.clone(),
                LivePeerInfo {
                    peer_id: peer_str,
                    connected_at: chrono::Utc::now().to_rfc3339(),
                    ping_rtt_ms: None,
                    address: None,
                    agent_version: None,
                    protocol_version: None,
                    is_relayed,
                    in_gossip_mesh: false,
                    in_kademlia: false,
                },
            );
        }
    }

    /// Peer getrennt
    pub fn peer_disconnected(&self, peer_id: impl AsRef<str>) {
        // Dekrement nur wenn > 0
        let _ = self
            .connected_peers_count
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                if v > 0 {
                    Some(v - 1)
                } else {
                    Some(0)
                }
            });

        if let Ok(mut peers) = self.peers.write() {
            peers.remove(peer_id.as_ref());
        }
    }

    /// Verbindungsfehler
    pub fn connection_error(&self) {
        self.connection_errors.fetch_add(1, Ordering::SeqCst);
    }

    // ========================================================================
    // IDENTIFY UPDATES
    // ========================================================================

    /// Identify-Info für Peer aktualisieren
    pub fn update_peer_identify(
        &self,
        peer_id: impl AsRef<str>,
        agent_version: Option<String>,
        protocol_version: Option<String>,
    ) {
        if let Ok(mut peers) = self.peers.write() {
            if let Some(peer) = peers.get_mut(peer_id.as_ref()) {
                peer.agent_version = agent_version;
                peer.protocol_version = protocol_version;
            }
        }
    }

    // ========================================================================
    // PING UPDATES
    // ========================================================================

    /// Ping-RTT aufzeichnen
    pub fn record_ping(&self, peer_id: impl AsRef<str>, rtt: std::time::Duration) {
        let rtt_us = rtt.as_micros() as u64;

        self.ping_rtt_sum_us.fetch_add(rtt_us, Ordering::SeqCst);
        self.ping_count.fetch_add(1, Ordering::SeqCst);

        // Min/Max aktualisieren
        self.ping_min_us.fetch_min(rtt_us, Ordering::SeqCst);
        self.ping_max_us.fetch_max(rtt_us, Ordering::SeqCst);

        // Peer-spezifische RTT aktualisieren
        if let Ok(mut peers) = self.peers.write() {
            if let Some(peer) = peers.get_mut(peer_id.as_ref()) {
                peer.ping_rtt_ms = Some(rtt.as_millis() as u64);
            }
        }
    }

    /// Durchschnittliche Ping-RTT in ms
    pub fn avg_ping_ms(&self) -> f64 {
        let sum = self.ping_rtt_sum_us.load(Ordering::Relaxed);
        let count = self.ping_count.load(Ordering::Relaxed);
        if count == 0 {
            0.0
        } else {
            (sum as f64 / count as f64) / 1000.0 // µs → ms
        }
    }

    /// Minimale Ping-RTT in ms
    pub fn min_ping_ms(&self) -> f64 {
        let min = self.ping_min_us.load(Ordering::Relaxed);
        if min == u64::MAX {
            0.0
        } else {
            min as f64 / 1000.0
        }
    }

    /// Maximale Ping-RTT in ms
    pub fn max_ping_ms(&self) -> f64 {
        self.ping_max_us.load(Ordering::Relaxed) as f64 / 1000.0
    }

    // ========================================================================
    // UPNP UPDATES
    // ========================================================================

    /// UPnP External Address
    pub fn upnp_mapped(&self, addr: impl Into<String>) {
        self.upnp_available.store(true, Ordering::SeqCst);
        if let Ok(mut a) = self.upnp_external_addr.write() {
            *a = Some(addr.into());
        }
    }

    /// UPnP nicht verfügbar
    pub fn upnp_unavailable(&self) {
        self.upnp_available.store(false, Ordering::SeqCst);
    }

    // ========================================================================
    // SNAPSHOTS
    // ========================================================================

    /// Alle Peer-Infos abrufen
    pub fn get_peers(&self) -> Vec<LivePeerInfo> {
        self.peers
            .read()
            .map(|p| p.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Peer-IDs abrufen
    pub fn get_peer_ids(&self) -> Vec<String> {
        self.peers
            .read()
            .map(|p| p.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Uptime in Sekunden
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Snapshot aller Werte für Diagnostics
    pub fn snapshot(&self) -> SwarmSnapshot {
        SwarmSnapshot {
            peer_id: self.peer_id.clone(),
            uptime_secs: self.uptime_secs(),

            // NAT
            nat_status: self.get_nat_status(),
            external_addresses: self.get_external_addresses(),

            // Relay
            has_relay_reservation: self.has_relay_reservation.load(Ordering::Relaxed),
            relay_circuits_serving: self.relay_circuits_serving.load(Ordering::Relaxed),

            // DCUTR
            dcutr_successes: self.dcutr_successes.load(Ordering::Relaxed),
            dcutr_failures: self.dcutr_failures.load(Ordering::Relaxed),
            dcutr_success_rate: self.dcutr_success_rate(),

            // Discovery
            kademlia_routing_table_size: self.kademlia_routing_table_size.load(Ordering::Relaxed),
            kademlia_bootstrap_complete: self.kademlia_bootstrap_complete.load(Ordering::Relaxed),
            dht_records_stored: self.dht_records_stored.load(Ordering::Relaxed),
            mdns_active: self.mdns_active.load(Ordering::Relaxed),
            mdns_discovered_count: self.mdns_discovered_count.load(Ordering::Relaxed),

            // Gossipsub
            gossip_mesh_size: self.gossip_mesh_size.load(Ordering::Relaxed),
            gossip_topics_subscribed: self.gossip_topics_subscribed.load(Ordering::Relaxed),
            gossip_messages_received: self.gossip_messages_received.load(Ordering::Relaxed),
            gossip_messages_sent: self.gossip_messages_sent.load(Ordering::Relaxed),

            // Connections
            connected_peers_count: self.connected_peers_count.load(Ordering::Relaxed),
            connection_errors: self.connection_errors.load(Ordering::Relaxed),
            inbound_connections: self.inbound_connections.load(Ordering::Relaxed),
            outbound_connections: self.outbound_connections.load(Ordering::Relaxed),

            // Ping
            avg_ping_ms: self.avg_ping_ms(),
            min_ping_ms: self.min_ping_ms(),
            max_ping_ms: self.max_ping_ms(),

            // UPnP
            upnp_available: self.upnp_available.load(Ordering::Relaxed),

            // Peers
            peers: self.get_peers(),
        }
    }
}

// ============================================================================
// SWARM SNAPSHOT
// ============================================================================

/// Vollständiger Snapshot des Swarm-Zustands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmSnapshot {
    pub peer_id: String,
    pub uptime_secs: u64,

    // NAT
    pub nat_status: NatStatus,
    pub external_addresses: Vec<String>,

    // Relay
    pub has_relay_reservation: bool,
    pub relay_circuits_serving: usize,

    // DCUTR
    pub dcutr_successes: usize,
    pub dcutr_failures: usize,
    pub dcutr_success_rate: f64,

    // Discovery
    pub kademlia_routing_table_size: usize,
    pub kademlia_bootstrap_complete: bool,
    pub dht_records_stored: usize,
    pub mdns_active: bool,
    pub mdns_discovered_count: usize,

    // Gossipsub
    pub gossip_mesh_size: usize,
    pub gossip_topics_subscribed: usize,
    pub gossip_messages_received: u64,
    pub gossip_messages_sent: u64,

    // Connections
    pub connected_peers_count: usize,
    pub connection_errors: usize,
    pub inbound_connections: usize,
    pub outbound_connections: usize,

    // Ping
    pub avg_ping_ms: f64,
    pub min_ping_ms: f64,
    pub max_ping_ms: f64,

    // UPnP
    pub upnp_available: bool,

    // Peers
    pub peers: Vec<LivePeerInfo>,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_state_basics() {
        let state = SwarmState::new("test-peer-123");

        assert_eq!(state.get_nat_status(), NatStatus::Unknown);
        assert_eq!(state.connected_peers_count.load(Ordering::Relaxed), 0);

        state.set_nat_status(NatStatus::Public);
        assert_eq!(state.get_nat_status(), NatStatus::Public);

        state.peer_connected("peer-1", false, false);
        assert_eq!(state.connected_peers_count.load(Ordering::Relaxed), 1);

        state.peer_disconnected("peer-1");
        assert_eq!(state.connected_peers_count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_dcutr_rate() {
        let state = SwarmState::new("test-peer");

        // Keine Versuche = 100%
        assert_eq!(state.dcutr_success_rate(), 100.0);

        // 3 Erfolge, 1 Fehlschlag = 75%
        state.dcutr_success();
        state.dcutr_success();
        state.dcutr_success();
        state.dcutr_failure();

        assert_eq!(state.dcutr_success_rate(), 75.0);
    }

    #[test]
    fn test_ping_stats() {
        let state = SwarmState::new("test-peer");

        state.record_ping("peer-1", std::time::Duration::from_millis(10));
        state.record_ping("peer-2", std::time::Duration::from_millis(20));
        state.record_ping("peer-3", std::time::Duration::from_millis(30));

        assert_eq!(state.avg_ping_ms(), 20.0);
        assert_eq!(state.min_ping_ms(), 10.0);
        assert_eq!(state.max_ping_ms(), 30.0);
    }

    #[test]
    fn test_snapshot() {
        let state = SwarmState::new("test-peer");

        state.set_nat_status(NatStatus::Public);
        state.peer_connected("peer-1", true, false);
        state.kademlia_bootstrap_done();

        let snapshot = state.snapshot();

        assert_eq!(snapshot.nat_status, NatStatus::Public);
        assert_eq!(snapshot.connected_peers_count, 1);
        assert!(snapshot.kademlia_bootstrap_complete);
    }
}
