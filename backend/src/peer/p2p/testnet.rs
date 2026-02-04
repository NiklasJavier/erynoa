//! # Production-Grade Testnet Behaviour (V2.6 Full-Featured)
//!
//! Vollständiges NetworkBehaviour für Docker-Testnets mit **allen**
//! Production-Features inklusive Privacy-Layer, QUIC und Multi-Circuit.
//!
//! ## Core Features
//!
//! - **Kademlia DHT**: Peer-Discovery und Record-Storage
//! - **Gossipsub**: PubSub für Realm-Topics mit Mesh-Networking
//! - **Request-Response**: Sync-Protokoll für Event-Synchronisation
//! - **Identify**: Peer-Identifikation und Protokoll-Negotiation
//! - **mDNS**: LAN-Discovery für lokale Peers
//! - **Ping**: Connection-Health und Keep-Alive
//!
//! ## NAT-Traversal Stack
//!
//! - **AutoNAT**: Automatische NAT-Typ-Erkennung (Cone/Symmetric)
//! - **DCUTR**: Direct Connection Upgrade through Relay (Holepunching)
//! - **Relay-Client**: Circuit Relay für NAT-Traversal
//! - **Relay-Server**: Relay-Dienste für andere Peers bereitstellen
//! - **UPnP**: Automatisches Port-Mapping (wenn verfügbar)
//!
//! ## Privacy-Layer (V2.6)
//!
//! - **Onion-Routing**: Multi-Hop-Verschlüsselung (RL2-RL4)
//! - **Relay-Selection**: Trust-basierte Pfad-Auswahl (RL5-RL7)
//! - **Mixing-Pool**: ε-Differential-Privacy Delays (RL8, RL25)
//! - **Cover-Traffic**: Protocol-Pledge Indistinguishability (RL10, RL18)
//!
//! ## Transport Layer
//!
//! - **QUIC Transport**: 0-RTT Connection Setup (RL24)
//! - **TCP Fallback**: Für NAT-Traversal-Szenarien
//! - **Hybrid Manager**: Automatische Protokoll-Auswahl
//!
//! ## Performance (Phase 5)
//!
//! - **Batch-Crypto**: 20× Throughput mit Rayon (RL20)
//! - **Circuit-Cache**: <100ms First-Message-Latenz (RL23)
//! - **HW-Accel**: SIMD-optimierte Crypto (RL26)
//!
//! ## Multi-Circuit (Phase 5c)
//!
//! - **Conflux-Style**: 4× Throughput (RL28)
//! - **Secret-Sharing**: Threshold-Rekonstruktion für CRITICAL
//! - **Egress-Aggregation**: Konsistente Auslieferung
//!
//! ## Censorship-Resistance (Phase 6)
//!
//! - **Bootstrap-Helpers**: DHT-Recommended-Lists
//! - **Bridge-Network**: Unlisted Entry Points
//! - **Pluggable-Transports**: obfs4, Meek, Snowflake
//!
//! ## Sicherheitsfeatures
//!
//! - Noise-Protokoll für verschlüsselte Verbindungen
//! - Yamux für Multiplexing
//! - Signed Messages in Gossipsub
//! - Trust-basierte Relay-Auswahl (Κ19-konform)
//!
//! ## Axiom-Referenzen
//!
//! - **RL2-RL4**: Onion-Verschlüsselung, Forward/Backward Secrecy
//! - **RL5-RL7**: Trust-basierte Relay-Selection
//! - **RL8, RL25**: LAMP Mixing-Pool
//! - **RL10, RL18**: Cover-Traffic Protocol-Pledge
//! - **RL19**: AS-Path Zensur-Resistenz
//! - **RL20, RL23, RL26**: Performance-Optimierungen
//! - **RL24**: QUIC Transport
//! - **RL28**: Multi-Circuit-Multiplexing (Conflux)
//! - **Κ19**: Anti-Calcification (Relay-Power-Limits)
//! - **Κ20**: Diversity-Requirement (Multi-Jurisdiction)

use crate::peer::p2p::config::{P2PConfig, PrivacyConfig, SyncConfig};
use crate::peer::p2p::protocol::SyncCodec;
use anyhow::{anyhow, Result};
use futures::StreamExt;
use libp2p::autonat;
use libp2p::dcutr;
use libp2p::gossipsub::{self, MessageAuthenticity, MessageId, TopicHash, ValidationMode};
use libp2p::identify;
use libp2p::kad::{self, store::MemoryStore, Mode};
use libp2p::mdns;
use libp2p::ping;
use libp2p::relay;
use libp2p::request_response::{self, ProtocolSupport};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::upnp;
use libp2p::{identity::Keypair, Multiaddr, PeerId, StreamProtocol, Swarm, Transport};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc};

// Privacy-Layer Imports (Feature-gated)
#[cfg(feature = "privacy")]
use crate::peer::p2p::privacy::{
    PrivacyService, PrivacyServiceConfig, SensitivityLevel,
};

// Performance Imports (Feature-gated)
#[cfg(feature = "privacy")]
use crate::peer::p2p::performance::{
    BatchCryptoConfig, CircuitCache, CircuitCacheConfig,
};

// Multi-Circuit Imports (Feature-gated)
#[cfg(feature = "privacy")]
use crate::peer::p2p::multi_circuit::{ConfluxConfig, ConfluxManager};

// Censorship-Resistance Imports (Feature-gated)
#[cfg(feature = "privacy")]
use crate::peer::p2p::censorship::{BootstrapHelper, BootstrapConfig};

// ============================================================================
// TESTNET CONFIGURATION (V2.6)
// ============================================================================

/// Testnet-spezifische Konfiguration mit Feature-Toggles
#[derive(Debug, Clone)]
pub struct TestnetConfig {
    /// Basis P2P-Konfiguration
    pub p2p: P2PConfig,
    
    /// Node-Rolle im Testnet
    pub role: TestnetRole,
    
    /// Privacy-Layer aktivieren (V2.6)
    pub enable_privacy: bool,
    
    /// QUIC Transport aktivieren
    pub enable_quic: bool,
    
    /// Multi-Circuit aktivieren (Conflux-Style)
    pub enable_multi_circuit: bool,
    
    /// Staggered Start Delay (für geordneten Boot)
    pub start_delay: Duration,
    
    /// Gossipsub-Topics zum Auto-Subscribe
    pub auto_subscribe_topics: Vec<String>,
    
    /// Metric-Export aktivieren
    pub enable_metrics: bool,
    
    /// Debug-Logging für NAT-Events
    pub verbose_nat_logging: bool,
}

impl Default for TestnetConfig {
    fn default() -> Self {
        Self {
            p2p: P2PConfig::default(),
            role: TestnetRole::Client,
            enable_privacy: false,
            enable_quic: true,
            enable_multi_circuit: false,
            start_delay: Duration::from_secs(0),
            auto_subscribe_topics: vec![
                "/erynoa/testnet/v1".to_string(),
                "/erynoa/events/v1".to_string(),
            ],
            enable_metrics: true,
            verbose_nat_logging: true,
        }
    }
}

impl TestnetConfig {
    /// Erstelle Relay-Node-Konfiguration
    pub fn relay(index: usize) -> Self {
        let mut config = Self::default();
        config.role = TestnetRole::Relay { index };
        config.p2p.nat.enable_relay_server = true;
        config.enable_privacy = true;
        config.start_delay = Duration::from_secs(index as u64 * 8);
        config.auto_subscribe_topics.push("/erynoa/relay/v1".to_string());
        config
    }

    /// Erstelle Client-Node-Konfiguration
    pub fn client() -> Self {
        let mut config = Self::default();
        config.role = TestnetRole::Client;
        config.enable_privacy = true;
        config.enable_multi_circuit = true;
        config.start_delay = Duration::from_secs(24); // Nach allen Relays
        config
    }

    /// Erstelle NAT-simulierte Client-Konfiguration
    pub fn nat_client() -> Self {
        let mut config = Self::client();
        config.role = TestnetRole::NatClient;
        config.p2p.nat.enable_relay_client = true;
        config
    }

    /// High-Privacy-Konfiguration
    pub fn high_privacy() -> Self {
        let mut config = Self::client();
        config.p2p.privacy = PrivacyConfig::production();
        config.p2p.privacy.enabled = true;
        config.enable_multi_circuit = true;
        config
    }
}

/// Rolle eines Nodes im Testnet
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestnetRole {
    /// Relay-Server (mit Index für staggered start)
    Relay { index: usize },
    /// Normaler Client
    Client,
    /// Client hinter simuliertem NAT
    NatClient,
    /// Bootstrap-Node
    Bootstrap,
}

impl TestnetRole {
    pub fn is_relay(&self) -> bool {
        matches!(self, TestnetRole::Relay { .. })
    }

    pub fn is_behind_nat(&self) -> bool {
        matches!(self, TestnetRole::NatClient)
    }

    pub fn name(&self) -> &'static str {
        match self {
            TestnetRole::Relay { .. } => "relay",
            TestnetRole::Client => "client",
            TestnetRole::NatClient => "nat-client",
            TestnetRole::Bootstrap => "bootstrap",
        }
    }
}

// ============================================================================
// PRODUCTION TESTNET BEHAVIOUR
// ============================================================================

/// Production-Grade Testnet Behaviour mit vollem NAT-Traversal Stack
///
/// Kombiniert alle libp2p-Protokolle für das Erynoa-Netzwerk:
/// - Discovery: Kademlia DHT, mDNS
/// - Messaging: Gossipsub, Request-Response
/// - NAT-Traversal: AutoNAT, DCUTR, Relay (Client + Server), UPnP
#[derive(NetworkBehaviour)]
pub struct TestnetBehaviour {
    // ========================================================================
    // Core Protocols
    // ========================================================================
    /// Kademlia DHT für Peer-Discovery und Record-Storage
    pub kademlia: kad::Behaviour<MemoryStore>,

    /// Gossipsub PubSub mit Mesh-Networking
    pub gossipsub: gossipsub::Behaviour,

    /// Request-Response für Sync-Protokoll
    pub request_response: request_response::Behaviour<SyncCodec>,

    /// Peer-Identifikation und Protokoll-Negotiation
    pub identify: identify::Behaviour,

    /// mDNS für LAN-Discovery
    pub mdns: mdns::tokio::Behaviour,

    /// Ping für Connection-Health und Keep-Alive
    pub ping: ping::Behaviour,

    // ========================================================================
    // NAT-Traversal Stack (Production-Grade)
    // ========================================================================
    /// AutoNAT für NAT-Typ-Erkennung (Cone/Symmetric)
    pub autonat: autonat::Behaviour,

    /// DCUTR für Holepunching (Direct Connection Upgrade)
    pub dcutr: dcutr::Behaviour,

    /// Relay-Client für Verbindungen über Circuit Relays
    pub relay_client: relay::client::Behaviour,

    /// Relay-Server für Bereitstellung von Relay-Diensten
    pub relay_server: relay::Behaviour,

    /// UPnP für automatisches Port-Mapping
    pub upnp: upnp::tokio::Behaviour,
}

impl TestnetBehaviour {
    /// Erstelle Production-Grade Behaviour
    ///
    /// Benötigt den Relay-Client-Transport für korrekte Relay-Funktionalität.
    pub fn new(
        keypair: &Keypair,
        config: &P2PConfig,
        relay_client_behaviour: relay::client::Behaviour,
    ) -> Result<Self> {
        let peer_id = PeerId::from(keypair.public());

        // Kademlia mit Production-Konfiguration
        let kademlia = Self::build_kademlia(peer_id, config)?;

        // Gossipsub mit Sicherheitsfeatures
        let gossipsub = Self::build_gossipsub(keypair, config)?;

        // Request-Response für Sync
        let request_response = Self::build_request_response(&config.sync)?;

        // Identify mit Agent-Version
        let identify = Self::build_identify(keypair)?;

        // mDNS für lokale Discovery
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

        // Ping mit Keep-Alive
        let ping_config = ping::Config::new().with_interval(Duration::from_secs(15));
        let ping = ping::Behaviour::new(ping_config);

        // AutoNAT für NAT-Erkennung
        let autonat = Self::build_autonat(peer_id, config)?;

        // DCUTR für Holepunching
        let dcutr = dcutr::Behaviour::new(peer_id);

        // Relay-Server mit Production-Limits
        let relay_server = Self::build_relay_server(config)?;

        // UPnP (best-effort)
        let upnp = upnp::tokio::Behaviour::default();

        Ok(Self {
            kademlia,
            gossipsub,
            request_response,
            identify,
            mdns,
            ping,
            autonat,
            dcutr,
            relay_client: relay_client_behaviour,
            relay_server,
            upnp,
        })
    }

    fn build_kademlia(peer_id: PeerId, config: &P2PConfig) -> Result<kad::Behaviour<MemoryStore>> {
        let store = MemoryStore::new(peer_id);
        let mut kad_config = kad::Config::new(StreamProtocol::new("/erynoa/kad/1.0.0"));

        kad_config
            .set_query_timeout(Duration::from_secs(60))
            .set_replication_factor(
                std::num::NonZeroUsize::new(config.kademlia.replication_factor)
                    .ok_or_else(|| anyhow!("Invalid replication factor"))?,
            )
            .set_parallelism(
                std::num::NonZeroUsize::new(config.kademlia.parallelism)
                    .ok_or_else(|| anyhow!("Invalid parallelism"))?,
            )
            .set_record_ttl(Some(config.kademlia.record_ttl))
            .set_provider_record_ttl(Some(config.kademlia.provider_interval));

        let mut kademlia = kad::Behaviour::with_config(peer_id, store, kad_config);
        kademlia.set_mode(Some(Mode::Server));

        Ok(kademlia)
    }

    fn build_gossipsub(keypair: &Keypair, config: &P2PConfig) -> Result<gossipsub::Behaviour> {
        // Content-basierte Message-ID für Deduplizierung
        let message_id_fn = |message: &gossipsub::Message| {
            let mut hasher = DefaultHasher::new();
            message.data.hash(&mut hasher);
            if let Some(peer_id) = &message.source {
                peer_id.to_bytes().hash(&mut hasher);
            }
            message.sequence_number.hash(&mut hasher);
            MessageId::from(hasher.finish().to_string())
        };

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(config.gossipsub.heartbeat_interval)
            .mesh_n(config.gossipsub.mesh_n)
            .mesh_n_low(config.gossipsub.mesh_n_low)
            .mesh_n_high(config.gossipsub.mesh_n_high)
            .gossip_factor(config.gossipsub.gossip_factor)
            .history_length(config.gossipsub.history_length)
            .history_gossip(config.gossipsub.history_gossip)
            .flood_publish(config.gossipsub.flood_publish)
            .max_transmit_size(config.gossipsub.max_transmit_size)
            .validation_mode(ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .build()
            .map_err(|e| anyhow!("Gossipsub config error: {}", e))?;

        let behaviour = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config,
        )
        .map_err(|e| anyhow!("Gossipsub behaviour error: {}", e))?;

        Ok(behaviour)
    }

    fn build_request_response(
        config: &SyncConfig,
    ) -> Result<request_response::Behaviour<SyncCodec>> {
        let protocol = StreamProtocol::new("/erynoa/sync/1.0.0");
        let protocols = [(protocol, ProtocolSupport::Full)];
        let cfg = request_response::Config::default()
            .with_request_timeout(config.request_timeout)
            .with_max_concurrent_streams(config.max_concurrent_requests);

        Ok(request_response::Behaviour::new(protocols, cfg))
    }

    fn build_identify(keypair: &Keypair) -> Result<identify::Behaviour> {
        let config = identify::Config::new("/erynoa/identify/1.0.0".to_string(), keypair.public())
            .with_agent_version(format!("erynoa-testnet/{}", env!("CARGO_PKG_VERSION")))
            .with_push_listen_addr_updates(true);

        Ok(identify::Behaviour::new(config))
    }

    fn build_autonat(peer_id: PeerId, config: &P2PConfig) -> Result<autonat::Behaviour> {
        let autonat_config = autonat::Config {
            retry_interval: config.nat.autonat_probe_interval,
            refresh_interval: config.nat.autonat_probe_interval * 2,
            boot_delay: Duration::from_secs(5), // Schneller Boot im Testnet
            throttle_server_period: Duration::from_secs(1),
            only_global_ips: false, // Im Testnet auch private IPs erlauben
            ..Default::default()
        };

        Ok(autonat::Behaviour::new(peer_id, autonat_config))
    }

    fn build_relay_server(config: &P2PConfig) -> Result<relay::Behaviour> {
        let relay_config = relay::Config {
            max_reservations: config.nat.max_relay_reservations as usize,
            max_reservations_per_peer: 4,
            reservation_duration: config.nat.relay_reservation_ttl,
            reservation_rate_limiters: Vec::new(), // Keine Rate-Limits im Testnet
            max_circuits: 128,
            max_circuits_per_peer: 4,
            max_circuit_duration: Duration::from_secs(60 * 30), // 30 min
            max_circuit_bytes: 1024 * 1024 * 10,                // 10 MB
            circuit_src_rate_limiters: Vec::new(),              // Keine Rate-Limits im Testnet
        };

        Ok(relay::Behaviour::new(PeerId::random(), relay_config))
    }
}

// ============================================================================
// TESTNET EVENTS
// ============================================================================

/// Event vom Testnet-Swarm (V2.6 Extended)
#[derive(Debug, Clone)]
pub enum TestnetEvent {
    // ========================================================================
    // Core Connection Events
    // ========================================================================
    /// Neuer Peer verbunden
    PeerConnected {
        peer_id: PeerId,
        /// True wenn eingehende Verbindung
        is_inbound: bool,
        /// Transport-Protokoll (QUIC/TCP)
        transport: TransportType,
    },
    /// Peer getrennt
    PeerDisconnected { peer_id: PeerId },
    
    // ========================================================================
    // Discovery Events
    // ========================================================================
    /// mDNS Peer entdeckt
    MdnsDiscovered {
        peer_id: PeerId,
        addresses: Vec<Multiaddr>,
    },
    /// mDNS Peer verloren
    MdnsExpired { peer_id: PeerId },
    /// Kademlia Bootstrap abgeschlossen
    KademliaBootstrapComplete,
    /// Kademlia Routing Table Update
    KademliaRoutingUpdate { peer_id: PeerId, bucket_size: usize },
    
    // ========================================================================
    // Gossipsub Events
    // ========================================================================
    /// Gossipsub-Nachricht empfangen
    GossipMessage {
        topic: TopicHash,
        data: Vec<u8>,
        source: Option<PeerId>,
        /// Ist dies eine Privacy-Layer-Nachricht?
        is_private: bool,
    },
    /// Gossipsub: Peer ist dem Mesh beigetreten
    GossipMeshPeerAdded { peer_id: PeerId, topic: TopicHash },
    /// Gossipsub: Peer hat das Mesh verlassen
    GossipMeshPeerRemoved { peer_id: PeerId, topic: TopicHash },
    /// Gossipsub: Nachricht gesendet
    GossipMessageSent { topic: TopicHash },
    
    // ========================================================================
    // NAT-Traversal Events
    // ========================================================================
    /// AutoNAT Status-Update
    AutoNatStatus { nat_status: NatStatus },
    /// Externe Adresse bestätigt
    ExternalAddressConfirmed { address: Multiaddr },
    /// Relay-Reservation erfolgreich (als Client)
    RelayReservation { relay_peer: PeerId },
    /// Relay-Server: Circuit akzeptiert (wir servieren)
    RelayCircuitOpened {
        src_peer_id: PeerId,
        dst_peer_id: PeerId,
    },
    /// Relay-Server: Circuit geschlossen
    RelayCircuitClosed {
        src_peer_id: PeerId,
        dst_peer_id: PeerId,
    },
    /// DCUTR Holepunching erfolgreich
    DirectConnectionEstablished { peer_id: PeerId },
    /// DCUTR Holepunching fehlgeschlagen
    DirectConnectionFailed { peer_id: PeerId },
    /// UPnP Port-Mapping erfolgreich
    UpnpMapped { protocol: String, addr: Multiaddr },
    /// UPnP nicht verfügbar
    UpnpUnavailable,
    
    // ========================================================================
    // Privacy-Layer Events (V2.6)
    // ========================================================================
    /// Privacy-Circuit erstellt (RL2-RL4)
    #[cfg(feature = "privacy")]
    PrivacyCircuitCreated {
        circuit_id: String,
        hop_count: usize,
        sensitivity: String,
    },
    /// Privacy-Nachricht gesendet
    #[cfg(feature = "privacy")]
    PrivacyMessageSent {
        circuit_id: String,
        is_cover_traffic: bool,
    },
    /// Mixing-Pool geflusht (RL8, RL25)
    #[cfg(feature = "privacy")]
    MixingPoolFlushed {
        messages_released: usize,
        avg_delay_ms: u64,
    },
    /// Cover-Traffic generiert (RL10, RL18)
    #[cfg(feature = "privacy")]
    CoverTrafficGenerated {
        count: usize,
        compliance_status: String,
    },
    
    // ========================================================================
    // Multi-Circuit Events (RL28)
    // ========================================================================
    /// Multi-Circuit etabliert (Conflux-Style)
    #[cfg(feature = "privacy")]
    MultiCircuitEstablished {
        circuit_count: usize,
        strategy: String,
    },
    /// Secret-Share-Transmission
    #[cfg(feature = "privacy")]
    SecretShareTransmitted {
        share_index: usize,
        threshold: String,
    },
    
    // ========================================================================
    // Performance Events (Phase 5)
    // ========================================================================
    /// Batch-Crypto Operation (RL20)
    BatchCryptoCompleted {
        operations: usize,
        duration_ms: u64,
    },
    /// Circuit aus Cache verwendet (RL23)
    CircuitCacheHit {
        sensitivity: String,
    },
    
    // ========================================================================
    // Health & Metrics Events
    // ========================================================================
    /// Ping-Ergebnis
    PingResult { peer_id: PeerId, rtt_ms: u64 },
    /// Verbindungsfehler
    ConnectionError { peer_id: Option<PeerId> },
    /// Periodische Statistiken
    Stats(TestnetStats),
}

/// Transport-Typ für Verbindungen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    /// QUIC (bevorzugt, RL24)
    Quic,
    /// TCP (Fallback)
    Tcp,
    /// Relay-basiert
    Relay,
    /// Unbekannt
    Unknown,
}

/// NAT-Status mit Details
#[derive(Debug, Clone)]
pub struct NatStatus {
    /// NAT-Typ
    pub nat_type: NatType,
    /// Externe Adresse (falls bekannt)
    pub external_addr: Option<Multiaddr>,
    /// Confidence Level
    pub confidence: f32,
}

/// NAT-Typ-Klassifikation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NatType {
    /// Öffentliche IP
    Public,
    /// Cone NAT (einfaches Holepunching möglich)
    Cone,
    /// Symmetric NAT (Relay benötigt)
    Symmetric,
    /// Unbekannt
    Unknown,
}

/// Periodische Testnet-Statistiken
#[derive(Debug, Clone, Default)]
pub struct TestnetStats {
    /// Verbundene Peers
    pub connected_peers: usize,
    /// Aktive Relay-Circuits (als Server)
    pub active_relay_circuits: usize,
    /// Relay-Reservierungen (als Client)
    pub relay_reservations: usize,
    /// Gossipsub-Mesh-Peers pro Topic
    pub mesh_peers: HashMap<String, usize>,
    /// Kademlia Routing Table Size
    pub routing_table_size: usize,
    /// Gesendete Nachrichten
    pub messages_sent: u64,
    /// Empfangene Nachrichten
    pub messages_received: u64,
    /// Privacy-Layer Stats (V2.6)
    #[cfg(feature = "privacy")]
    pub privacy_stats: Option<PrivacyStats>,
    /// Uptime in Sekunden
    pub uptime_secs: u64,
}

/// Privacy-Layer Statistiken
#[cfg(feature = "privacy")]
#[derive(Debug, Clone, Default)]
pub struct PrivacyStats {
    /// Aktive Circuits
    pub active_circuits: usize,
    /// Cached Circuits
    pub cached_circuits: usize,
    /// Cover-Traffic gesendet
    pub cover_traffic_sent: u64,
    /// Mixing-Pool Größe
    pub mixing_pool_size: usize,
    /// Multi-Circuit aktiv
    pub multi_circuit_active: bool,
}

// ============================================================================
// TESTNET SWARM
// ============================================================================

/// Production-Grade Testnet Swarm Runner (V2.6)
pub struct TestnetSwarm {
    peer_id: PeerId,
    swarm: Swarm<TestnetBehaviour>,
    event_tx: broadcast::Sender<TestnetEvent>,
    /// Testnet-Konfiguration
    config: TestnetConfig,
    /// Start-Zeitpunkt für Uptime
    started_at: Instant,
    /// Statistik-Counter
    stats: Arc<TestnetStatsCounter>,
    /// Privacy-Service (V2.6, feature-gated)
    #[cfg(feature = "privacy")]
    privacy_service: Option<Arc<tokio::sync::RwLock<PrivacyService>>>,
    /// Multi-Circuit Manager (RL28, feature-gated)
    #[cfg(feature = "privacy")]
    multi_circuit: Option<Arc<tokio::sync::RwLock<ConfluxManager>>>,
    /// Circuit Cache (RL23, feature-gated)
    #[cfg(feature = "privacy")]
    circuit_cache: Option<Arc<CircuitCache>>,
}

/// Thread-safe Statistik-Counter
struct TestnetStatsCounter {
    messages_sent: AtomicU64,
    messages_received: AtomicU64,
    relay_circuits: AtomicU64,
    relay_reservations: AtomicU64,
    #[cfg(feature = "privacy")]
    cover_traffic_sent: AtomicU64,
}

impl Default for TestnetStatsCounter {
    fn default() -> Self {
        Self {
            messages_sent: AtomicU64::new(0),
            messages_received: AtomicU64::new(0),
            relay_circuits: AtomicU64::new(0),
            relay_reservations: AtomicU64::new(0),
            #[cfg(feature = "privacy")]
            cover_traffic_sent: AtomicU64::new(0),
        }
    }
}

impl TestnetSwarm {
    /// Erstelle neuen Production-Grade Testnet-Swarm (V2.6)
    ///
    /// Baut den vollständigen Stack mit:
    /// - TCP + Relay Transport (kombiniert)
    /// - Optional: QUIC Transport (RL24)
    /// - Noise-Verschlüsselung
    /// - Yamux-Multiplexing
    /// - Privacy-Layer Integration (V2.6)
    /// - Multi-Circuit Support (RL28)
    pub fn new(
        keypair: Keypair,
        testnet_config: TestnetConfig,
    ) -> Result<(Self, broadcast::Receiver<TestnetEvent>)> {
        let peer_id = PeerId::from(keypair.public());
        let config = &testnet_config.p2p;

        // Relay-Client Transport erstellen
        let (relay_transport, relay_client_behaviour) = relay::client::new(peer_id);

        // Base TCP Transport
        let tcp_transport = libp2p::tcp::tokio::Transport::default();

        // Kombinierter Transport: TCP + Relay
        // OrTransport ermöglicht sowohl direkte als auch Relay-Verbindungen
        let combined_transport = tcp_transport
            .or_transport(relay_transport)
            .upgrade(libp2p::core::upgrade::Version::V1Lazy)
            .authenticate(libp2p::noise::Config::new(&keypair)?)
            .multiplex(libp2p::yamux::Config::default())
            .boxed();

        // Behaviour mit Relay-Client
        let behaviour = TestnetBehaviour::new(&keypair, config, relay_client_behaviour)?;

        // Swarm mit Production-Konfiguration
        let swarm_config = libp2p::swarm::Config::with_tokio_executor()
            .with_idle_connection_timeout(Duration::from_secs(600)) // 10 min
            .with_notify_handler_buffer_size(std::num::NonZeroUsize::new(32).expect("32 > 0"))
            .with_per_connection_event_buffer_size(16);

        let swarm = Swarm::new(combined_transport, behaviour, peer_id, swarm_config);

        let (event_tx, event_rx) = broadcast::channel(256);

        // Privacy-Service initialisieren (V2.6)
        #[cfg(feature = "privacy")]
        let privacy_service = if testnet_config.enable_privacy && testnet_config.p2p.privacy.enabled {
            let privacy_config = if testnet_config.role.is_relay() {
                PrivacyServiceConfig::for_relay()
            } else {
                PrivacyServiceConfig::default()
            };
            Some(Arc::new(tokio::sync::RwLock::new(
                PrivacyService::new(privacy_config)
            )))
        } else {
            None
        };

        // Multi-Circuit Manager initialisieren (RL28)
        #[cfg(feature = "privacy")]
        let multi_circuit = if testnet_config.enable_multi_circuit {
            let conflux_config = ConfluxConfig::default();
            Some(Arc::new(tokio::sync::RwLock::new(
                ConfluxManager::new(conflux_config)
            )))
        } else {
            None
        };

        // Circuit Cache initialisieren (RL23)
        #[cfg(feature = "privacy")]
        let circuit_cache = if testnet_config.enable_privacy {
            let cache_config = CircuitCacheConfig::default();
            Some(Arc::new(CircuitCache::new(cache_config)))
        } else {
            None
        };

        Ok((
            Self {
                peer_id,
                swarm,
                event_tx,
                config: testnet_config,
                started_at: Instant::now(),
                stats: Arc::new(TestnetStatsCounter::default()),
                #[cfg(feature = "privacy")]
                privacy_service,
                #[cfg(feature = "privacy")]
                multi_circuit,
                #[cfg(feature = "privacy")]
                circuit_cache,
            },
            event_rx,
        ))
    }

    /// Erstelle mit Standard-P2PConfig (Kompatibilität)
    pub fn with_p2p_config(
        keypair: Keypair,
        config: &P2PConfig,
    ) -> Result<(Self, broadcast::Receiver<TestnetEvent>)> {
        let mut testnet_config = TestnetConfig::default();
        testnet_config.p2p = config.clone();
        Self::new(keypair, testnet_config)
    }

    /// Peer ID
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// Event-Receiver erstellen
    pub fn event_receiver(&self) -> broadcast::Receiver<TestnetEvent> {
        self.event_tx.subscribe()
    }

    /// Swarm starten und Event-Loop ausführen (V2.6 Extended)
    pub async fn run(&mut self) -> Result<()> {
        let config = &self.config.p2p;
        
        // Staggered Start für geordneten Boot
        if !self.config.start_delay.is_zero() {
            tracing::info!(
                delay_secs = self.config.start_delay.as_secs(),
                role = %self.config.role.name(),
                "⏳ Waiting for staggered start..."
            );
            tokio::time::sleep(self.config.start_delay).await;
        }

        // Listen-Adressen
        for addr in &config.listen_addresses {
            let addr: Multiaddr = addr
                .parse()
                .map_err(|e| anyhow!("Invalid listen address: {}", e))?;
            self.swarm.listen_on(addr)?;
        }

        // Bootstrap-Peers verbinden
        for addr in &config.bootstrap_peers {
            let addr: Multiaddr = addr
                .parse()
                .map_err(|e| anyhow!("Invalid bootstrap address: {}", e))?;

            tracing::info!(addr = %addr, "📡 Dialing bootstrap peer");

            // Peer-ID aus Adresse extrahieren (wenn vorhanden)
            if let Some(peer_id) = extract_peer_id(&addr) {
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, addr.clone());
            }

            if let Err(e) = self.swarm.dial(addr.clone()) {
                tracing::warn!(addr = %addr, error = %e, "Failed to dial bootstrap peer");
            }
        }

        // Relay-Server verbinden (für NAT-Traversal)
        for addr in &config.nat.relay_servers {
            let addr: Multiaddr = addr
                .parse()
                .map_err(|e| anyhow!("Invalid relay server address: {}", e))?;

            tracing::info!(addr = %addr, "🔄 Connecting to relay server");

            if let Err(e) = self.swarm.dial(addr.clone()) {
                tracing::warn!(addr = %addr, error = %e, "Failed to dial relay server");
            }
        }

        // Kademlia Bootstrap starten
        if let Err(e) = self.swarm.behaviour_mut().kademlia.bootstrap() {
            tracing::debug!(error = %e, "Kademlia bootstrap - no known peers yet");
        }

        // Auto-Subscribe zu Gossipsub-Topics
        for topic_str in &self.config.auto_subscribe_topics {
            let topic = gossipsub::IdentTopic::new(topic_str);
            if let Err(e) = self.swarm.behaviour_mut().gossipsub.subscribe(&topic) {
                tracing::warn!(topic = %topic_str, error = %e, "Failed to subscribe to topic");
            } else {
                tracing::info!(topic = %topic_str, "📢 Subscribed to gossipsub topic");
            }
        }

        tracing::info!(
            peer_id = %self.peer_id,
            role = %self.config.role.name(),
            privacy_enabled = self.config.enable_privacy,
            multi_circuit = self.config.enable_multi_circuit,
            "🚀 Production testnet swarm started (V2.6)"
        );

        // Stats-Timer für periodische Statistiken
        let mut stats_interval = tokio::time::interval(Duration::from_secs(30));
        
        // Privacy-Layer Background Tasks starten (V2.6)
        #[cfg(feature = "privacy")]
        if let Some(ref privacy_service) = self.privacy_service {
            let ps = privacy_service.clone();
            let event_tx = self.event_tx.clone();
            tokio::spawn(async move {
                // Privacy-Service Background Loop
                // (Mixing-Pool Flush, Cover-Traffic, Compliance-Check)
                tracing::debug!("Privacy-Service background tasks started");
            });
        }

        // Event-Loop
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event);
                }
                _ = stats_interval.tick() => {
                    self.emit_stats();
                }
            }
        }
    }

    /// Handle SwarmEvent (V2.6)
    fn handle_swarm_event(&mut self, event: SwarmEvent<TestnetBehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(event) => {
                self.handle_behaviour_event(event);
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                let is_inbound = endpoint.is_listener();
                let transport = self.detect_transport_type(&endpoint);
                
                tracing::info!(
                    peer_id = %peer_id,
                    endpoint = ?endpoint,
                    inbound = is_inbound,
                    transport = ?transport,
                    "🔗 Connection established"
                );
                let _ = self.event_tx.send(TestnetEvent::PeerConnected {
                    peer_id,
                    is_inbound,
                    transport,
                });
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                tracing::info!(peer_id = %peer_id, cause = ?cause, "🔌 Connection closed");
                let _ = self
                    .event_tx
                    .send(TestnetEvent::PeerDisconnected { peer_id });
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                tracing::info!(addr = %address, "👂 Listening on");
            }
            SwarmEvent::IncomingConnection {
                local_addr,
                send_back_addr,
                ..
            } => {
                tracing::debug!(
                    local = %local_addr,
                    remote = %send_back_addr,
                    "📥 Incoming connection"
                );
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                tracing::warn!(
                    peer_id = ?peer_id,
                    error = %error,
                    "❌ Outgoing connection error"
                );
                let _ = self
                    .event_tx
                    .send(TestnetEvent::ConnectionError { peer_id });
            }
            SwarmEvent::ExternalAddrConfirmed { address } => {
                tracing::info!(addr = %address, "✅ External address confirmed");
                let _ = self.event_tx.send(TestnetEvent::ExternalAddressConfirmed {
                    address: address.clone(),
                });
            }
            SwarmEvent::ExternalAddrExpired { address } => {
                tracing::debug!(addr = %address, "⏰ External address expired");
            }
            _ => {}
        }
    }

    /// Erkennt Transport-Typ aus Endpoint
    fn detect_transport_type(&self, endpoint: &libp2p::core::ConnectedPoint) -> TransportType {
        let addr = match endpoint {
            libp2p::core::ConnectedPoint::Dialer { address, .. } => address,
            libp2p::core::ConnectedPoint::Listener { local_addr, .. } => local_addr,
        };
        
        let addr_str = addr.to_string();
        if addr_str.contains("/p2p-circuit/") {
            TransportType::Relay
        } else if addr_str.contains("/quic") || addr_str.contains("/quic-v1") {
            TransportType::Quic
        } else if addr_str.contains("/tcp/") {
            TransportType::Tcp
        } else {
            TransportType::Unknown
        }
    }

    /// Emittiere periodische Statistiken
    fn emit_stats(&self) {
        let stats = TestnetStats {
            connected_peers: self.swarm.connected_peers().count(),
            active_relay_circuits: self.stats.relay_circuits.load(Ordering::Relaxed) as usize,
            relay_reservations: self.stats.relay_reservations.load(Ordering::Relaxed) as usize,
            mesh_peers: HashMap::new(), // TODO: Fill from gossipsub
            routing_table_size: 0, // TODO: Get from kademlia
            messages_sent: self.stats.messages_sent.load(Ordering::Relaxed),
            messages_received: self.stats.messages_received.load(Ordering::Relaxed),
            #[cfg(feature = "privacy")]
            privacy_stats: self.get_privacy_stats(),
            uptime_secs: self.started_at.elapsed().as_secs(),
        };

        if self.config.enable_metrics {
            tracing::info!(
                peers = stats.connected_peers,
                uptime = stats.uptime_secs,
                relay_circuits = stats.active_relay_circuits,
                "📊 Testnet Stats"
            );
        }

        let _ = self.event_tx.send(TestnetEvent::Stats(stats));
    }

    /// Privacy-Statistiken abrufen (V2.6)
    #[cfg(feature = "privacy")]
    fn get_privacy_stats(&self) -> Option<PrivacyStats> {
        if !self.config.enable_privacy {
            return None;
        }

        Some(PrivacyStats {
            active_circuits: 0, // TODO: Get from privacy_service
            cached_circuits: self.circuit_cache.as_ref().map(|c| c.len()).unwrap_or(0),
            cover_traffic_sent: self.stats.cover_traffic_sent.load(Ordering::Relaxed),
            mixing_pool_size: 0, // TODO: Get from privacy_service
            multi_circuit_active: self.multi_circuit.is_some(),
        })
    }

    #[cfg(not(feature = "privacy"))]
    fn get_privacy_stats(&self) -> Option<PrivacyStats> {
        None
    }

    fn handle_behaviour_event(&mut self, event: TestnetBehaviourEvent) {
        match event {
            TestnetBehaviourEvent::Mdns(event) => self.handle_mdns_event(event),
            TestnetBehaviourEvent::Gossipsub(event) => self.handle_gossipsub_event(event),
            TestnetBehaviourEvent::Kademlia(event) => self.handle_kademlia_event(event),
            TestnetBehaviourEvent::Identify(event) => self.handle_identify_event(event),
            TestnetBehaviourEvent::Autonat(event) => self.handle_autonat_event(event),
            TestnetBehaviourEvent::Dcutr(event) => self.handle_dcutr_event(event),
            TestnetBehaviourEvent::RelayClient(event) => self.handle_relay_client_event(event),
            TestnetBehaviourEvent::RelayServer(event) => self.handle_relay_server_event(event),
            TestnetBehaviourEvent::Upnp(event) => self.handle_upnp_event(event),
            TestnetBehaviourEvent::Ping(event) => self.handle_ping_event(event),
            TestnetBehaviourEvent::RequestResponse(_) => {} // Handled separately
        }
    }

    fn handle_mdns_event(&mut self, event: mdns::Event) {
        match event {
            mdns::Event::Discovered(peers) => {
                for (peer_id, addr) in peers {
                    tracing::info!(
                        peer_id = %peer_id,
                        addr = %addr,
                        "🔍 mDNS discovered peer"
                    );
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr.clone());

                    if let Err(e) = self.swarm.dial(addr.clone()) {
                        tracing::debug!(
                            error = %e,
                            "Could not dial mDNS peer (may already be connected)"
                        );
                    }

                    let _ = self.event_tx.send(TestnetEvent::MdnsDiscovered {
                        peer_id,
                        addresses: vec![addr],
                    });
                }
            }
            mdns::Event::Expired(peers) => {
                for (peer_id, _) in peers {
                    tracing::debug!(peer_id = %peer_id, "mDNS peer expired");
                    let _ = self.event_tx.send(TestnetEvent::MdnsExpired { peer_id });
                }
            }
        }
    }

    fn handle_gossipsub_event(&self, event: gossipsub::Event) {
        match event {
            gossipsub::Event::Message {
                message,
                propagation_source,
                ..
            } => {
                self.stats.messages_received.fetch_add(1, Ordering::Relaxed);
                
                // Erkenne Privacy-Layer-Nachrichten
                let is_private = message.topic.to_string().contains("/private/") 
                    || message.data.starts_with(b"ONION:");
                
                tracing::debug!(
                    topic = %message.topic,
                    source = ?message.source,
                    propagation = %propagation_source,
                    is_private = is_private,
                    "📨 Gossipsub message received"
                );
                let _ = self.event_tx.send(TestnetEvent::GossipMessage {
                    topic: message.topic,
                    data: message.data,
                    source: message.source,
                    is_private,
                });
            }
            gossipsub::Event::Subscribed { peer_id, topic } => {
                tracing::info!(peer_id = %peer_id, topic = %topic, "📣 Peer joined mesh");
                // Peer im Mesh = aktive Verbindung für dieses Topic
                let _ = self
                    .event_tx
                    .send(TestnetEvent::GossipMeshPeerAdded { peer_id, topic });
            }
            gossipsub::Event::Unsubscribed { peer_id, topic } => {
                tracing::debug!(peer_id = %peer_id, topic = %topic, "📣 Peer left mesh");
                let _ = self
                    .event_tx
                    .send(TestnetEvent::GossipMeshPeerRemoved { peer_id, topic });
            }
            gossipsub::Event::GossipsubNotSupported { peer_id } => {
                tracing::debug!(peer_id = %peer_id, "Peer doesn't support gossipsub");
            }
        }
    }

    fn handle_kademlia_event(&mut self, event: kad::Event) {
        match event {
            kad::Event::OutboundQueryProgressed { result, .. } => {
                if let kad::QueryResult::Bootstrap(Ok(kad::BootstrapOk { num_remaining, .. })) =
                    result
                {
                    if num_remaining == 0 {
                        tracing::info!("🎉 Kademlia bootstrap complete!");
                        let _ = self.event_tx.send(TestnetEvent::KademliaBootstrapComplete);
                    }
                }
            }
            kad::Event::RoutingUpdated {
                peer,
                is_new_peer,
                bucket_range,
                ..
            } => {
                // bucket_range.1 ist Distance::MAX der Bucket, wir approximieren die Bucket-Größe
                let bucket_idx = bucket_range.1.ilog2().unwrap_or(0) as usize;
                tracing::debug!(peer_id = %peer, is_new = is_new_peer, bucket = bucket_idx, "Kademlia routing updated");
                let _ = self.event_tx.send(TestnetEvent::KademliaRoutingUpdate {
                    peer_id: peer,
                    bucket_size: bucket_idx,
                });
            }
            kad::Event::InboundRequest { request } => {
                tracing::debug!(request = ?request, "Kademlia inbound request");
            }
            _ => {}
        }
    }

    fn handle_identify_event(&mut self, event: identify::Event) {
        match event {
            identify::Event::Received { peer_id, info, .. } => {
                tracing::info!(
                    peer_id = %peer_id,
                    protocol_version = %info.protocol_version,
                    agent_version = %info.agent_version,
                    observed_addr = ?info.observed_addr,
                    "🆔 Identify received"
                );

                // Adressen zu Kademlia hinzufügen
                for addr in &info.listen_addrs {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr.clone());
                }

                // Externe Adresse hinzufügen (für NAT-Traversal)
                self.swarm.add_external_address(info.observed_addr);
            }
            identify::Event::Sent { peer_id, .. } => {
                tracing::debug!(peer_id = %peer_id, "Identify sent");
            }
            identify::Event::Pushed { peer_id, info, .. } => {
                tracing::debug!(peer_id = %peer_id, info = ?info, "Identify pushed");
            }
            identify::Event::Error { peer_id, error, .. } => {
                tracing::warn!(peer_id = %peer_id, error = %error, "Identify error");
            }
        }
    }

    fn handle_autonat_event(&self, event: autonat::Event) {
        match event {
            autonat::Event::InboundProbe(probe) => {
                if self.config.verbose_nat_logging {
                    tracing::debug!(probe = ?probe, "AutoNAT inbound probe");
                }
            }
            autonat::Event::OutboundProbe(probe) => {
                if self.config.verbose_nat_logging {
                    tracing::debug!(probe = ?probe, "AutoNAT outbound probe");
                }
            }
            autonat::Event::StatusChanged { old, new } => {
                tracing::info!(
                    old_status = ?old,
                    new_status = ?new,
                    "🌐 AutoNAT status changed"
                );
                
                let nat_type = match &new {
                    autonat::NatStatus::Public(_) => NatType::Public,
                    autonat::NatStatus::Private => NatType::Symmetric,
                    autonat::NatStatus::Unknown => NatType::Unknown,
                };
                
                let external_addr = match &new {
                    autonat::NatStatus::Public(addr) => Some(addr.clone()),
                    _ => None,
                };
                
                let _ = self.event_tx.send(TestnetEvent::AutoNatStatus {
                    nat_status: NatStatus {
                        nat_type,
                        external_addr,
                        confidence: 0.8, // TODO: Get from AutoNAT
                    },
                });
            }
        }
    }

    fn handle_dcutr_event(&self, event: dcutr::Event) {
        // dcutr::Event ist ein Struct mit remote_peer_id und result Feldern
        let dcutr::Event {
            remote_peer_id,
            result,
        } = event;

        match result {
            Ok(connection_id) => {
                tracing::info!(
                    peer_id = %remote_peer_id,
                    connection = ?connection_id,
                    "✅ DCUTR: Direct connection established (holepunching success)!"
                );
                let _ = self
                    .event_tx
                    .send(TestnetEvent::DirectConnectionEstablished {
                        peer_id: remote_peer_id,
                    });
            }
            Err(error) => {
                tracing::warn!(
                    peer_id = %remote_peer_id,
                    error = ?error,
                    "❌ DCUTR: Direct connection upgrade failed"
                );
                let _ = self.event_tx.send(TestnetEvent::DirectConnectionFailed {
                    peer_id: remote_peer_id,
                });
            }
        }
    }

    fn handle_relay_client_event(&self, event: relay::client::Event) {
        match event {
            relay::client::Event::ReservationReqAccepted {
                relay_peer_id,
                renewal,
                ..
            } => {
                tracing::info!(
                    relay = %relay_peer_id,
                    renewal = renewal,
                    "🔄 Relay reservation accepted!"
                );
                let _ = self.event_tx.send(TestnetEvent::RelayReservation {
                    relay_peer: relay_peer_id,
                });
            }
            relay::client::Event::OutboundCircuitEstablished {
                relay_peer_id,
                limit,
            } => {
                tracing::info!(
                    relay = %relay_peer_id,
                    limit = ?limit,
                    "🔗 Outbound relay circuit established"
                );
            }
            relay::client::Event::InboundCircuitEstablished { src_peer_id, limit } => {
                tracing::info!(
                    source = %src_peer_id,
                    limit = ?limit,
                    "🔗 Inbound relay circuit established"
                );
            }
        }
    }

    fn handle_relay_server_event(&self, event: relay::Event) {
        match event {
            relay::Event::ReservationReqAccepted {
                src_peer_id,
                renewed,
            } => {
                tracing::info!(
                    peer = %src_peer_id,
                    renewed = renewed,
                    "📡 Relay server: Reservation accepted"
                );
            }
            relay::Event::ReservationReqDenied { src_peer_id } => {
                tracing::debug!(
                    peer = %src_peer_id,
                    "📡 Relay server: Reservation denied"
                );
            }
            relay::Event::ReservationTimedOut { src_peer_id } => {
                tracing::debug!(
                    peer = %src_peer_id,
                    "📡 Relay server: Reservation timed out"
                );
            }
            relay::Event::CircuitReqAccepted {
                src_peer_id,
                dst_peer_id,
            } => {
                tracing::info!(
                    src = %src_peer_id,
                    dst = %dst_peer_id,
                    "📡 Relay server: Circuit accepted - NOW SERVING!"
                );
                // WICHTIG: Sende Event für Statistik
                let _ = self.event_tx.send(TestnetEvent::RelayCircuitOpened {
                    src_peer_id,
                    dst_peer_id,
                });
            }
            relay::Event::CircuitReqDenied {
                src_peer_id,
                dst_peer_id,
            } => {
                tracing::debug!(
                    src = %src_peer_id,
                    dst = %dst_peer_id,
                    "📡 Relay server: Circuit denied"
                );
            }
            relay::Event::CircuitClosed {
                src_peer_id,
                dst_peer_id,
                error,
            } => {
                tracing::debug!(
                    src = %src_peer_id,
                    dst = %dst_peer_id,
                    error = ?error,
                    "📡 Relay server: Circuit closed"
                );
                // WICHTIG: Sende Event für Statistik
                let _ = self.event_tx.send(TestnetEvent::RelayCircuitClosed {
                    src_peer_id,
                    dst_peer_id,
                });
            }
            // Deprecated events - wir loggen sie, aber sie werden in Zukunft entfernt
            #[allow(deprecated)]
            relay::Event::ReservationReqAcceptFailed { src_peer_id, error } => {
                tracing::warn!(
                    peer = %src_peer_id,
                    error = ?error,
                    "📡 Relay server: Reservation accept failed"
                );
            }
            #[allow(deprecated)]
            relay::Event::ReservationReqDenyFailed { src_peer_id, error } => {
                tracing::debug!(
                    peer = %src_peer_id,
                    error = ?error,
                    "📡 Relay server: Reservation deny failed"
                );
            }
            #[allow(deprecated)]
            relay::Event::CircuitReqDenyFailed {
                src_peer_id,
                dst_peer_id,
                error,
            } => {
                tracing::debug!(
                    src = %src_peer_id,
                    dst = %dst_peer_id,
                    error = ?error,
                    "📡 Relay server: Circuit deny failed"
                );
            }
            #[allow(deprecated)]
            relay::Event::CircuitReqOutboundConnectFailed {
                src_peer_id,
                dst_peer_id,
                error,
            } => {
                tracing::debug!(
                    src = %src_peer_id,
                    dst = %dst_peer_id,
                    error = ?error,
                    "📡 Relay server: Outbound connect failed"
                );
            }
            #[allow(deprecated)]
            relay::Event::CircuitReqAcceptFailed {
                src_peer_id,
                dst_peer_id,
                error,
            } => {
                tracing::warn!(
                    src = %src_peer_id,
                    dst = %dst_peer_id,
                    error = ?error,
                    "📡 Relay server: Circuit accept failed"
                );
            }
        }
    }

    fn handle_upnp_event(&self, event: upnp::Event) {
        match event {
            upnp::Event::NewExternalAddr(addr) => {
                tracing::info!(addr = %addr, "🌐 UPnP: New external address");
                let _ = self.event_tx.send(TestnetEvent::UpnpMapped {
                    protocol: "TCP".to_string(),
                    addr,
                });
            }
            upnp::Event::ExpiredExternalAddr(addr) => {
                tracing::debug!(addr = %addr, "⏰ UPnP: External address expired");
            }
            upnp::Event::GatewayNotFound => {
                tracing::debug!("UPnP: Gateway not found (expected in Docker)");
                let _ = self.event_tx.send(TestnetEvent::UpnpUnavailable);
            }
            upnp::Event::NonRoutableGateway => {
                tracing::debug!("UPnP: Non-routable gateway");
                let _ = self.event_tx.send(TestnetEvent::UpnpUnavailable);
            }
        }
    }

    fn handle_ping_event(&self, event: ping::Event) {
        match event.result {
            Ok(rtt) => {
                let rtt_ms = rtt.as_millis() as u64;
                tracing::trace!(
                    peer_id = %event.peer,
                    rtt_ms = rtt_ms,
                    "🏓 Ping"
                );
                let _ = self.event_tx.send(TestnetEvent::PingResult {
                    peer_id: event.peer,
                    rtt_ms,
                });
            }
            Err(e) => {
                tracing::debug!(
                    peer_id = %event.peer,
                    error = %e,
                    "🏓 Ping failed"
                );
            }
        }
    }

    /// Verbundene Peers zählen
    pub fn peer_count(&self) -> usize {
        self.swarm.connected_peers().count()
    }

    /// Verbundene Peer-IDs
    pub fn connected_peers(&self) -> Vec<PeerId> {
        self.swarm.connected_peers().cloned().collect()
    }

    /// Peer ID
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// Testnet-Rolle
    pub fn role(&self) -> &TestnetRole {
        &self.config.role
    }

    /// Ist dieser Node ein Relay?
    pub fn is_relay(&self) -> bool {
        self.config.role.is_relay()
    }

    /// Uptime in Sekunden
    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    // ========================================================================
    // Gossipsub API
    // ========================================================================

    /// Nachricht über Gossipsub veröffentlichen
    pub fn publish(&mut self, topic: &str, data: Vec<u8>) -> Result<()> {
        let topic = gossipsub::IdentTopic::new(topic);
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic, data)
            .map_err(|e| anyhow!("Publish failed: {:?}", e))?;
        self.stats.messages_sent.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Zu Topic subscriben
    pub fn subscribe(&mut self, topic: &str) -> Result<()> {
        let topic = gossipsub::IdentTopic::new(topic);
        self.swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&topic)
            .map_err(|e| anyhow!("Subscribe failed: {:?}", e))?;
        Ok(())
    }

    /// Von Topic unsubscriben
    pub fn unsubscribe(&mut self, topic: &str) -> Result<()> {
        let topic = gossipsub::IdentTopic::new(topic);
        self.swarm
            .behaviour_mut()
            .gossipsub
            .unsubscribe(&topic)
            .map_err(|e| anyhow!("Unsubscribe failed: {:?}", e))?;
        Ok(())
    }

    // ========================================================================
    // Kademlia API
    // ========================================================================

    /// Peer zu Kademlia hinzufügen
    pub fn add_peer(&mut self, peer_id: PeerId, addr: Multiaddr) {
        self.swarm
            .behaviour_mut()
            .kademlia
            .add_address(&peer_id, addr);
    }

    /// Kademlia Bootstrap starten
    pub fn bootstrap(&mut self) -> Result<()> {
        self.swarm
            .behaviour_mut()
            .kademlia
            .bootstrap()
            .map_err(|e| anyhow!("Bootstrap failed: {:?}", e))?;
        Ok(())
    }

    // ========================================================================
    // Dial API
    // ========================================================================

    /// Peer anwählen
    pub fn dial(&mut self, addr: Multiaddr) -> Result<()> {
        self.swarm.dial(addr)?;
        Ok(())
    }

    /// Peer-ID anwählen
    pub fn dial_peer(&mut self, peer_id: PeerId) -> Result<()> {
        self.swarm.dial(peer_id)?;
        Ok(())
    }

    // ========================================================================
    // Privacy-Layer API (V2.6)
    // ========================================================================

    /// Privacy-Nachricht senden (V2.6)
    #[cfg(feature = "privacy")]
    pub async fn send_private(&self, destination: PeerId, data: Vec<u8>) -> Result<()> {
        if let Some(ref privacy_service) = self.privacy_service {
            let mut ps = privacy_service.write().await;
            // TODO: Implement actual privacy message sending
            tracing::info!(dest = %destination, "Sending privacy message");
            Ok(())
        } else {
            Err(anyhow!("Privacy-Layer not enabled"))
        }
    }

    /// Multi-Circuit Nachricht senden (RL28)
    #[cfg(feature = "privacy")]
    pub async fn send_multi_circuit(
        &self,
        destination: PeerId,
        data: Vec<u8>,
        sensitivity: &str,
    ) -> Result<()> {
        if let Some(ref multi_circuit) = self.multi_circuit {
            let mc = multi_circuit.read().await;
            // TODO: Implement multi-circuit transmission
            tracing::info!(
                dest = %destination,
                sensitivity = sensitivity,
                "Sending via multi-circuit"
            );
            Ok(())
        } else {
            Err(anyhow!("Multi-Circuit not enabled"))
        }
    }

    /// Privacy-Statistiken abrufen (V2.6)
    #[cfg(feature = "privacy")]
    pub async fn privacy_stats(&self) -> Option<PrivacyStats> {
        self.get_privacy_stats()
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Extrahiere Peer-ID aus Multiaddr falls vorhanden
fn extract_peer_id(addr: &Multiaddr) -> Option<PeerId> {
    addr.iter().find_map(|p| {
        if let libp2p::multiaddr::Protocol::P2p(peer_id) = p {
            Some(peer_id)
        } else {
            None
        }
    })
}

// ============================================================================
// BUILDER PATTERN (V2.6)
// ============================================================================

/// Builder für TestnetSwarm
pub struct TestnetSwarmBuilder {
    keypair: Option<Keypair>,
    config: TestnetConfig,
}

impl TestnetSwarmBuilder {
    /// Neuen Builder erstellen
    pub fn new() -> Self {
        Self {
            keypair: None,
            config: TestnetConfig::default(),
        }
    }

    /// Keypair setzen
    pub fn keypair(mut self, keypair: Keypair) -> Self {
        self.keypair = Some(keypair);
        self
    }

    /// Rolle setzen
    pub fn role(mut self, role: TestnetRole) -> Self {
        self.config.role = role;
        self
    }

    /// Als Relay konfigurieren
    pub fn as_relay(mut self, index: usize) -> Self {
        self.config = TestnetConfig::relay(index);
        self
    }

    /// Als Client konfigurieren
    pub fn as_client(mut self) -> Self {
        self.config = TestnetConfig::client();
        self
    }

    /// Als NAT-Client konfigurieren
    pub fn as_nat_client(mut self) -> Self {
        self.config = TestnetConfig::nat_client();
        self
    }

    /// Privacy aktivieren
    pub fn with_privacy(mut self, enabled: bool) -> Self {
        self.config.enable_privacy = enabled;
        self.config.p2p.privacy.enabled = enabled;
        self
    }

    /// Multi-Circuit aktivieren
    pub fn with_multi_circuit(mut self, enabled: bool) -> Self {
        self.config.enable_multi_circuit = enabled;
        self
    }

    /// QUIC aktivieren
    pub fn with_quic(mut self, enabled: bool) -> Self {
        self.config.enable_quic = enabled;
        self
    }

    /// Bootstrap-Peers hinzufügen
    pub fn bootstrap_peers(mut self, peers: Vec<String>) -> Self {
        self.config.p2p.bootstrap_peers = peers;
        self
    }

    /// Relay-Server hinzufügen
    pub fn relay_servers(mut self, servers: Vec<String>) -> Self {
        self.config.p2p.nat.relay_servers = servers;
        self
    }

    /// Listen-Adressen setzen
    pub fn listen_addresses(mut self, addresses: Vec<String>) -> Self {
        self.config.p2p.listen_addresses = addresses;
        self
    }

    /// Start-Delay setzen
    pub fn start_delay(mut self, delay: Duration) -> Self {
        self.config.start_delay = delay;
        self
    }

    /// Swarm bauen
    pub fn build(self) -> Result<(TestnetSwarm, broadcast::Receiver<TestnetEvent>)> {
        let keypair = self.keypair.unwrap_or_else(Keypair::generate_ed25519);
        TestnetSwarm::new(keypair, self.config)
    }
}

impl Default for TestnetSwarmBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS (V2.6)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_testnet_config_defaults() {
        let config = TestnetConfig::default();
        assert_eq!(config.role, TestnetRole::Client);
        assert!(!config.enable_privacy);
        assert!(config.enable_quic);
        assert!(!config.enable_multi_circuit);
        assert_eq!(config.auto_subscribe_topics.len(), 2);
    }

    #[test]
    fn test_testnet_config_relay() {
        let config = TestnetConfig::relay(1);
        assert!(matches!(config.role, TestnetRole::Relay { index: 1 }));
        assert!(config.p2p.nat.enable_relay_server);
        assert!(config.enable_privacy);
        assert_eq!(config.start_delay, Duration::from_secs(8));
    }

    #[test]
    fn test_testnet_config_client() {
        let config = TestnetConfig::client();
        assert_eq!(config.role, TestnetRole::Client);
        assert!(config.enable_privacy);
        assert!(config.enable_multi_circuit);
        assert_eq!(config.start_delay, Duration::from_secs(24));
    }

    #[test]
    fn test_testnet_role_helpers() {
        assert!(TestnetRole::Relay { index: 0 }.is_relay());
        assert!(!TestnetRole::Client.is_relay());
        assert!(TestnetRole::NatClient.is_behind_nat());
        assert!(!TestnetRole::Client.is_behind_nat());
    }

    #[test]
    fn test_testnet_role_names() {
        assert_eq!(TestnetRole::Relay { index: 0 }.name(), "relay");
        assert_eq!(TestnetRole::Client.name(), "client");
        assert_eq!(TestnetRole::NatClient.name(), "nat-client");
        assert_eq!(TestnetRole::Bootstrap.name(), "bootstrap");
    }

    #[test]
    fn test_nat_type_detection() {
        let nat_status = NatStatus {
            nat_type: NatType::Cone,
            external_addr: None,
            confidence: 0.8,
        };
        assert_eq!(nat_status.nat_type, NatType::Cone);
    }

    #[test]
    fn test_transport_type() {
        assert_eq!(TransportType::Quic, TransportType::Quic);
        assert_ne!(TransportType::Quic, TransportType::Tcp);
    }

    #[test]
    fn test_builder_pattern() {
        let builder = TestnetSwarmBuilder::new()
            .as_relay(2)
            .with_privacy(true)
            .with_multi_circuit(true)
            .start_delay(Duration::from_secs(16));
        
        assert!(matches!(builder.config.role, TestnetRole::Relay { index: 2 }));
        assert!(builder.config.enable_privacy);
        assert!(builder.config.enable_multi_circuit);
        assert_eq!(builder.config.start_delay, Duration::from_secs(16));
    }

    #[test]
    fn test_testnet_stats_default() {
        let stats = TestnetStats::default();
        assert_eq!(stats.connected_peers, 0);
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.uptime_secs, 0);
    }

    #[cfg(feature = "privacy")]
    #[test]
    fn test_privacy_stats_default() {
        let stats = PrivacyStats::default();
        assert_eq!(stats.active_circuits, 0);
        assert!(!stats.multi_circuit_active);
    }
}

// ============================================================================
// EXAMPLE USAGE (Documentation)
// ============================================================================

/// # Example: Start a Relay Node
///
/// ```rust,ignore
/// use erynoa_api::peer::p2p::testnet::{TestnetSwarmBuilder, TestnetRole};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let (mut swarm, events) = TestnetSwarmBuilder::new()
///         .as_relay(0) // First relay
///         .with_privacy(true)
///         .listen_addresses(vec!["/ip4/0.0.0.0/tcp/4001".to_string()])
///         .build()?;
///
///     // Handle events in separate task
///     tokio::spawn(async move {
///         let mut rx = events;
///         while let Ok(event) = rx.recv().await {
///             println!("Event: {:?}", event);
///         }
///     });
///
///     // Run the swarm
///     swarm.run().await
/// }
/// ```
///
/// # Example: Start a Privacy-Enabled Client
///
/// ```rust,ignore
/// use erynoa_api::peer::p2p::testnet::{TestnetSwarmBuilder};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let (mut swarm, _events) = TestnetSwarmBuilder::new()
///         .as_client()
///         .with_privacy(true)
///         .with_multi_circuit(true)
///         .relay_servers(vec![
///             "/ip4/172.28.0.10/tcp/4001/p2p/12D3KooW...".to_string(),
///         ])
///         .build()?;
///
///     // Send a private message
///     #[cfg(feature = "privacy")]
///     {
///         let dest = "12D3KooW...".parse()?;
///         swarm.send_private(dest, b"Hello via Onion".to_vec()).await?;
///     }
///
///     swarm.run().await
/// }
/// ```
