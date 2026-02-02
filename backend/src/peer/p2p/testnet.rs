//! # Production-Grade Testnet Behaviour
//!
//! Vollständiges NetworkBehaviour für Docker-Testnets mit allen
//! Production-Features inklusive NAT-Traversal.
//!
//! ## Features
//!
//! - **Kademlia DHT**: Peer-Discovery und Record-Storage
//! - **Gossipsub**: PubSub für Realm-Topics mit Mesh-Networking
//! - **Request-Response**: Sync-Protokoll für Event-Synchronisation
//! - **Identify**: Peer-Identifikation und Protokoll-Negotiation
//! - **mDNS**: LAN-Discovery für lokale Peers
//! - **Ping**: Connection-Health und Keep-Alive
//! - **AutoNAT**: Automatische NAT-Typ-Erkennung
//! - **DCUTR**: Direct Connection Upgrade through Relay (Holepunching)
//! - **Relay-Client**: Circuit Relay für NAT-Traversal
//! - **Relay-Server**: Relay-Dienste für andere Peers bereitstellen
//! - **UPnP**: Automatisches Port-Mapping (wenn verfügbar)
//!
//! ## Sicherheitsfeatures
//!
//! - Noise-Protokoll für verschlüsselte Verbindungen
//! - Yamux für Multiplexing
//! - Signed Messages in Gossipsub
//! - Trust-basierte Relay-Auswahl (Κ19-konform)

use crate::peer::p2p::config::{P2PConfig, SyncConfig};
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
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::sync::broadcast;

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

/// Event vom Testnet-Swarm
#[derive(Debug, Clone)]
pub enum TestnetEvent {
    /// Neuer Peer verbunden
    PeerConnected { peer_id: PeerId },
    /// Peer getrennt
    PeerDisconnected { peer_id: PeerId },
    /// mDNS Peer entdeckt
    MdnsDiscovered {
        peer_id: PeerId,
        addresses: Vec<Multiaddr>,
    },
    /// mDNS Peer verloren
    MdnsExpired { peer_id: PeerId },
    /// Gossipsub-Nachricht empfangen
    GossipMessage {
        topic: TopicHash,
        data: Vec<u8>,
        source: Option<PeerId>,
    },
    /// Kademlia Bootstrap abgeschlossen
    KademliaBootstrapComplete,
    /// AutoNAT Status-Update
    AutoNatStatus { nat_status: String },
    /// Relay-Reservation erfolgreich
    RelayReservation { relay_peer: PeerId },
    /// DCUTR Holepunching erfolgreich
    DirectConnectionEstablished { peer_id: PeerId },
    /// UPnP Port-Mapping erfolgreich
    UpnpMapped { protocol: String, addr: Multiaddr },
}

// ============================================================================
// TESTNET SWARM
// ============================================================================

/// Production-Grade Testnet Swarm Runner
pub struct TestnetSwarm {
    peer_id: PeerId,
    swarm: Swarm<TestnetBehaviour>,
    event_tx: broadcast::Sender<TestnetEvent>,
}

impl TestnetSwarm {
    /// Erstelle neuen Production-Grade Testnet-Swarm
    ///
    /// Baut den vollständigen NAT-Traversal-Stack mit:
    /// - TCP + Relay Transport (kombiniert)
    /// - Noise-Verschlüsselung
    /// - Yamux-Multiplexing
    pub fn new(
        keypair: Keypair,
        config: &P2PConfig,
    ) -> Result<(Self, broadcast::Receiver<TestnetEvent>)> {
        let peer_id = PeerId::from(keypair.public());

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

        Ok((
            Self {
                peer_id,
                swarm,
                event_tx,
            },
            event_rx,
        ))
    }

    /// Peer ID
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// Event-Receiver erstellen
    pub fn event_receiver(&self) -> broadcast::Receiver<TestnetEvent> {
        self.event_tx.subscribe()
    }

    /// Swarm starten und Event-Loop ausführen
    pub async fn run(&mut self, config: &P2PConfig) -> Result<()> {
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

        // Subscribe zu Gossipsub-Topics
        let testnet_topic = gossipsub::IdentTopic::new("/erynoa/testnet/v1");
        if let Err(e) = self
            .swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&testnet_topic)
        {
            tracing::warn!(error = %e, "Failed to subscribe to testnet topic");
        } else {
            tracing::info!(topic = %testnet_topic, "📢 Subscribed to gossipsub topic");
        }

        tracing::info!(
            peer_id = %self.peer_id,
            "🚀 Production testnet swarm started with full NAT-Traversal stack"
        );

        // Event-Loop
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::Behaviour(event) => {
                    self.handle_behaviour_event(event);
                }
                SwarmEvent::ConnectionEstablished {
                    peer_id, endpoint, ..
                } => {
                    tracing::info!(
                        peer_id = %peer_id,
                        endpoint = ?endpoint,
                        "🔗 Connection established"
                    );
                    let _ = self.event_tx.send(TestnetEvent::PeerConnected { peer_id });
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
                }
                SwarmEvent::ExternalAddrConfirmed { address } => {
                    tracing::info!(addr = %address, "✅ External address confirmed");
                }
                SwarmEvent::ExternalAddrExpired { address } => {
                    tracing::debug!(addr = %address, "⏰ External address expired");
                }
                _ => {}
            }
        }
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
                tracing::debug!(
                    topic = %message.topic,
                    source = ?message.source,
                    propagation = %propagation_source,
                    "📨 Gossipsub message"
                );
                let _ = self.event_tx.send(TestnetEvent::GossipMessage {
                    topic: message.topic,
                    data: message.data,
                    source: message.source,
                });
            }
            gossipsub::Event::Subscribed { peer_id, topic } => {
                tracing::info!(peer_id = %peer_id, topic = %topic, "📣 Peer subscribed");
            }
            gossipsub::Event::Unsubscribed { peer_id, topic } => {
                tracing::debug!(peer_id = %peer_id, topic = %topic, "📣 Peer unsubscribed");
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
            kad::Event::RoutingUpdated { peer, .. } => {
                tracing::debug!(peer_id = %peer, "Kademlia routing updated");
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
                tracing::debug!(probe = ?probe, "AutoNAT inbound probe");
            }
            autonat::Event::OutboundProbe(probe) => {
                tracing::debug!(probe = ?probe, "AutoNAT outbound probe");
            }
            autonat::Event::StatusChanged { old, new } => {
                tracing::info!(
                    old_status = ?old,
                    new_status = ?new,
                    "🌐 AutoNAT status changed"
                );
                let _ = self.event_tx.send(TestnetEvent::AutoNatStatus {
                    nat_status: format!("{:?}", new),
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
                    "📡 Relay server: Circuit accepted"
                );
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
            }
            upnp::Event::NonRoutableGateway => {
                tracing::debug!("UPnP: Non-routable gateway");
            }
        }
    }

    fn handle_ping_event(&self, event: ping::Event) {
        match event.result {
            Ok(rtt) => {
                tracing::trace!(
                    peer_id = %event.peer,
                    rtt_ms = rtt.as_millis(),
                    "🏓 Ping"
                );
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
}

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
