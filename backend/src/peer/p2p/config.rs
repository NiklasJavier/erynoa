//! # P2P-Konfiguration
//!
//! Konfigurationsstruktur für das libp2p-Netzwerk.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// P2P-Netzwerk-Konfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    /// TCP-Listen-Adressen
    pub listen_addresses: Vec<String>,

    /// Bootstrap-Peers (Multiaddrs)
    pub bootstrap_peers: Vec<String>,

    /// mDNS für LAN-Discovery aktivieren
    pub enable_mdns: bool,

    /// Kademlia-DHT-Konfiguration
    pub kademlia: KademliaConfig,

    /// Gossipsub-Konfiguration
    pub gossipsub: GossipsubConfig,

    /// Trust-Gate-Konfiguration
    pub trust_gate: TrustGateConfig,

    /// Sync-Protokoll-Konfiguration
    pub sync: SyncConfig,

    /// Connection-Limits
    pub connection_limits: ConnectionLimitsConfig,

    /// NAT-Traversal-Konfiguration (Priorität 3)
    pub nat: NatConfig,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            listen_addresses: vec![
                "/ip4/0.0.0.0/tcp/0".to_string(),
                "/ip6/::/tcp/0".to_string(),
            ],
            bootstrap_peers: vec![
                // Erynoa Foundation Bootstrap Nodes
                // "/ip4/51.159.23.74/tcp/4001/p2p/12D3KooW...".to_string(),
            ],
            enable_mdns: true,
            kademlia: KademliaConfig::default(),
            gossipsub: GossipsubConfig::default(),
            trust_gate: TrustGateConfig::default(),
            sync: SyncConfig::default(),
            connection_limits: ConnectionLimitsConfig::default(),
            nat: NatConfig::default(),
        }
    }
}

/// Kademlia-DHT-Konfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KademliaConfig {
    /// Record-Replication-Faktor
    pub replication_factor: usize,

    /// Query-Parallelität
    pub parallelism: usize,

    /// Record-TTL
    #[serde(with = "humantime_serde")]
    pub record_ttl: Duration,

    /// Provider-Record-Interval
    #[serde(with = "humantime_serde")]
    pub provider_interval: Duration,
}

impl Default for KademliaConfig {
    fn default() -> Self {
        Self {
            replication_factor: 20,
            parallelism: 3,
            record_ttl: Duration::from_secs(24 * 60 * 60), // 24h
            provider_interval: Duration::from_secs(12 * 60 * 60), // 12h
        }
    }
}

/// Gossipsub-Konfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipsubConfig {
    /// Heartbeat-Intervall
    #[serde(with = "humantime_serde")]
    pub heartbeat_interval: Duration,

    /// Mesh-Größe (D)
    pub mesh_n: usize,

    /// Minimum Mesh-Größe (D_lo)
    pub mesh_n_low: usize,

    /// Maximum Mesh-Größe (D_hi)
    pub mesh_n_high: usize,

    /// Gossip-Faktor
    pub gossip_factor: f64,

    /// History-Länge
    pub history_length: usize,

    /// History-Gossip
    pub history_gossip: usize,

    /// Flood-Publish aktivieren
    pub flood_publish: bool,

    /// Maximum Message-Größe (Bytes)
    pub max_transmit_size: usize,
}

impl Default for GossipsubConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(1),
            mesh_n: 6,
            mesh_n_low: 4,
            mesh_n_high: 12,
            gossip_factor: 0.25,
            history_length: 5,
            history_gossip: 3,
            flood_publish: true,
            max_transmit_size: 65536, // 64 KB
        }
    }
}

/// Trust-Gate-Konfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustGateConfig {
    /// Minimum Trust-R für eingehende Verbindungen
    pub min_incoming_trust_r: f64,

    /// Minimum Trust-Ω für Relay-Funktionen
    pub min_relay_trust_omega: f64,

    /// Trust-Check-Timeout
    #[serde(with = "humantime_serde")]
    pub trust_check_timeout: Duration,

    /// Automatisches Ablehnen von unbekannten Peers
    pub reject_unknown_peers: bool,

    /// Grace-Period für neue Peers (dürfen sich erstmal beweisen)
    #[serde(with = "humantime_serde")]
    pub newcomer_grace_period: Duration,
}

impl Default for TrustGateConfig {
    fn default() -> Self {
        Self {
            min_incoming_trust_r: 0.1,
            min_relay_trust_omega: 0.5,
            trust_check_timeout: Duration::from_secs(5),
            reject_unknown_peers: false,
            newcomer_grace_period: Duration::from_secs(60),
        }
    }
}

/// Sync-Protokoll-Konfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Maximum Events pro Sync-Request
    pub max_events_per_request: usize,

    /// Sync-Request-Timeout
    #[serde(with = "humantime_serde")]
    pub request_timeout: Duration,

    /// Concurrent Sync-Requests
    pub max_concurrent_requests: usize,

    /// Delta-Sync aktivieren (nur fehlende Events)
    pub delta_sync: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            max_events_per_request: 100,
            request_timeout: Duration::from_secs(30),
            max_concurrent_requests: 5,
            delta_sync: true,
        }
    }
}

/// Connection-Limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionLimitsConfig {
    /// Maximum eingehende Verbindungen
    pub max_incoming: u32,

    /// Maximum ausgehende Verbindungen
    pub max_outgoing: u32,

    /// Maximum Verbindungen pro Peer
    pub max_per_peer: u32,

    /// Idle-Timeout
    #[serde(with = "humantime_serde")]
    pub idle_timeout: Duration,
}

impl Default for ConnectionLimitsConfig {
    fn default() -> Self {
        Self {
            max_incoming: 100,
            max_outgoing: 50,
            max_per_peer: 2,
            idle_timeout: Duration::from_secs(60),
        }
    }
}

// ============================================================================
// NAT-Traversal-Konfiguration (Priorität 3)
// ============================================================================

/// NAT-Traversal-Konfiguration
///
/// Ermöglicht Verbindungen durch NATs mittels:
/// - **AutoNAT**: Automatische NAT-Erkennung
/// - **DCUTR**: Direct Connection Upgrade through Relay
/// - **Relay**: Circuit Relay als Fallback
/// - **UPnP**: Automatisches Port-Mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatConfig {
    /// AutoNAT aktivieren
    pub enable_autonat: bool,

    /// DCUTR (Holepunching) aktivieren
    pub enable_dcutr: bool,

    /// Als Relay-Server fungieren (für andere Peers)
    pub enable_relay_server: bool,

    /// Relay-Client aktivieren (Verbindung über Relays)
    pub enable_relay_client: bool,

    /// UPnP Port-Mapping aktivieren
    pub enable_upnp: bool,

    /// Bekannte Relay-Server (Multiaddrs)
    pub relay_servers: Vec<String>,

    /// AutoNAT Probe-Intervall
    #[serde(with = "humantime_serde")]
    pub autonat_probe_interval: Duration,

    /// Maximale Relay-Reservierungen (als Server)
    pub max_relay_reservations: u32,

    /// Relay-Reservierungs-TTL
    #[serde(with = "humantime_serde")]
    pub relay_reservation_ttl: Duration,

    /// Minimum Trust-R für Relay-Server (Κ19-konform)
    pub min_relay_server_trust: f32,
}

impl Default for NatConfig {
    fn default() -> Self {
        Self {
            enable_autonat: true,
            enable_dcutr: true,
            enable_relay_server: false, // Opt-in für Relay-Server
            enable_relay_client: true,
            enable_upnp: true,
            relay_servers: vec![
                // Erynoa Foundation Relay Nodes
                // "/ip4/51.159.23.74/tcp/4001/p2p/12D3KooW.../p2p-circuit".to_string(),
            ],
            autonat_probe_interval: Duration::from_secs(60),
            max_relay_reservations: 128,
            relay_reservation_ttl: Duration::from_secs(3600), // 1h
            min_relay_server_trust: 0.5,                      // Κ19: Nur vertrauenswürdige Relays
        }
    }
}

impl NatConfig {
    /// Prüfe ob NAT-Traversal komplett deaktiviert ist
    pub fn is_disabled(&self) -> bool {
        !self.enable_autonat && !self.enable_dcutr && !self.enable_relay_client && !self.enable_upnp
    }

    /// Prüfe ob Peer als Relay fungieren kann
    pub fn can_be_relay(&self) -> bool {
        self.enable_relay_server
    }
}

/// Humantime-Serde-Modul für Duration-Serialisierung
mod humantime_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = P2PConfig::default();
        assert!(config.enable_mdns);
        assert_eq!(config.gossipsub.mesh_n, 6);
        assert_eq!(config.kademlia.replication_factor, 20);
    }

    #[test]
    fn test_trust_gate_defaults() {
        let config = TrustGateConfig::default();
        assert_eq!(config.min_incoming_trust_r, 0.1);
        assert!(!config.reject_unknown_peers);
    }
}
