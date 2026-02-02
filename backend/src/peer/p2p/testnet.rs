//! # Testnet-spezifisches Behaviour
//!
//! Vereinfachtes NetworkBehaviour für Docker-Testnets ohne NAT-Traversal.
//!
//! Dieses Modul enthält ein reduziertes Behaviour ohne:
//! - AutoNAT
//! - DCUTR
//! - Relay (verursacht Panic wenn Transport nicht korrekt konfiguriert)
//! - UPnP
//!
//! Diese Features sind für Docker-Netzwerke mit direkter Konnektivität nicht nötig.

use crate::peer::p2p::config::{P2PConfig, SyncConfig};
use crate::peer::p2p::protocol::SyncCodec;
use anyhow::{anyhow, Result};
use futures::StreamExt;
use libp2p::gossipsub::{self, MessageAuthenticity, MessageId, TopicHash, ValidationMode};
use libp2p::identify;
use libp2p::kad::{self, store::MemoryStore, Mode};
use libp2p::mdns;
use libp2p::ping;
use libp2p::request_response::{self, ProtocolSupport};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{identity::Keypair, Multiaddr, PeerId, StreamProtocol, Swarm, Transport};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::sync::broadcast;

/// Testnet-spezifisches Behaviour (ohne NAT-Traversal)
#[derive(NetworkBehaviour)]
pub struct TestnetBehaviour {
    /// Kademlia DHT
    pub kademlia: kad::Behaviour<MemoryStore>,

    /// Gossipsub PubSub
    pub gossipsub: gossipsub::Behaviour,

    /// Request-Response für Sync
    pub request_response: request_response::Behaviour<SyncCodec>,

    /// Peer-Identifikation
    pub identify: identify::Behaviour,

    /// mDNS für Discovery im Docker-Netzwerk
    pub mdns: mdns::tokio::Behaviour,

    /// Ping für Connection-Health
    pub ping: ping::Behaviour,
}

impl TestnetBehaviour {
    /// Erstelle neues TestnetBehaviour
    pub fn new(keypair: &Keypair, config: &P2PConfig) -> Result<Self> {
        let peer_id = PeerId::from(keypair.public());

        // Kademlia mit Testnet-Konfiguration
        let kademlia = Self::build_kademlia(peer_id)?;

        // Gossipsub
        let gossipsub = Self::build_gossipsub(keypair)?;

        // Request-Response
        let request_response = Self::build_request_response(&config.sync)?;

        // Identify
        let identify = Self::build_identify(keypair)?;

        // mDNS - immer aktiviert im Testnet für Discovery
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

        // Ping - aktiviert Keep-Alive um Verbindungen offen zu halten
        let ping_config = ping::Config::new().with_interval(Duration::from_secs(15));
        let ping = ping::Behaviour::new(ping_config);

        Ok(Self {
            kademlia,
            gossipsub,
            request_response,
            identify,
            mdns,
            ping,
        })
    }

    fn build_kademlia(peer_id: PeerId) -> Result<kad::Behaviour<MemoryStore>> {
        let store = MemoryStore::new(peer_id);
        let mut config = kad::Config::new(StreamProtocol::new("/erynoa/kad/1.0.0"));
        config.set_query_timeout(Duration::from_secs(60));
        config.set_replication_factor(
            std::num::NonZeroUsize::new(3).ok_or_else(|| anyhow!("Invalid replication factor"))?,
        );

        let mut kademlia = kad::Behaviour::with_config(peer_id, store, config);
        kademlia.set_mode(Some(Mode::Server));

        Ok(kademlia)
    }

    fn build_gossipsub(keypair: &Keypair) -> Result<gossipsub::Behaviour> {
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
            .heartbeat_interval(Duration::from_secs(1))
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
        let cfg = request_response::Config::default().with_request_timeout(config.request_timeout);

        Ok(request_response::Behaviour::new(protocols, cfg))
    }

    fn build_identify(keypair: &Keypair) -> Result<identify::Behaviour> {
        let config = identify::Config::new("/erynoa/identify/1.0.0".to_string(), keypair.public())
            .with_agent_version(format!("erynoa-testnet/{}", env!("CARGO_PKG_VERSION")));

        Ok(identify::Behaviour::new(config))
    }
}

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
}

/// Testnet Swarm Runner
pub struct TestnetSwarm {
    peer_id: PeerId,
    swarm: Swarm<TestnetBehaviour>,
    event_tx: broadcast::Sender<TestnetEvent>,
}

impl TestnetSwarm {
    /// Erstelle neuen Testnet-Swarm
    pub fn new(
        keypair: Keypair,
        config: &P2PConfig,
    ) -> Result<(Self, broadcast::Receiver<TestnetEvent>)> {
        let peer_id = PeerId::from(keypair.public());

        // Transport: TCP + Noise + Yamux
        let transport = libp2p::tcp::tokio::Transport::default()
            .upgrade(libp2p::core::upgrade::Version::V1Lazy)
            .authenticate(libp2p::noise::Config::new(&keypair)?)
            .multiplex(libp2p::yamux::Config::default())
            .boxed();

        // Behaviour
        let behaviour = TestnetBehaviour::new(&keypair, config)?;

        // Swarm mit erhöhtem Idle-Timeout (10 Minuten)
        let swarm_config = libp2p::swarm::Config::with_tokio_executor()
            .with_idle_connection_timeout(Duration::from_secs(600));
        let swarm = Swarm::new(transport, behaviour, peer_id, swarm_config);

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

        // Kademlia Bootstrap starten
        if let Err(e) = self.swarm.behaviour_mut().kademlia.bootstrap() {
            tracing::debug!(error = %e, "Kademlia bootstrap - no known peers yet");
        }

        // Subscribe zu Gossipsub-Topics für aktive Verbindungen
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

        tracing::info!(peer_id = %self.peer_id, "🚀 Testnet swarm started");

        // Event-Loop
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::Behaviour(TestnetBehaviourEvent::Mdns(event)) => {
                    self.handle_mdns_event(event);
                }
                SwarmEvent::Behaviour(TestnetBehaviourEvent::Gossipsub(event)) => {
                    self.handle_gossipsub_event(event);
                }
                SwarmEvent::Behaviour(TestnetBehaviourEvent::Kademlia(event)) => {
                    self.handle_kademlia_event(event);
                }
                SwarmEvent::Behaviour(TestnetBehaviourEvent::Identify(event)) => {
                    self.handle_identify_event(event);
                }
                SwarmEvent::ConnectionEstablished {
                    peer_id, endpoint, ..
                } => {
                    tracing::info!(peer_id = %peer_id, endpoint = ?endpoint, "🔗 Connection established");
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
                    tracing::debug!(local = %local_addr, remote = %send_back_addr, "📥 Incoming connection");
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    tracing::warn!(peer_id = ?peer_id, error = %error, "❌ Outgoing connection error");
                }
                _ => {}
            }
        }
    }

    fn handle_mdns_event(&mut self, event: mdns::Event) {
        match event {
            mdns::Event::Discovered(peers) => {
                for (peer_id, addr) in peers {
                    tracing::info!(peer_id = %peer_id, addr = %addr, "🔍 mDNS discovered peer");
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr.clone());

                    // Versuche zu verbinden
                    if let Err(e) = self.swarm.dial(addr.clone()) {
                        tracing::debug!(error = %e, "Could not dial mDNS peer (may already be connected)");
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
        if let gossipsub::Event::Message { message, .. } = event {
            let _ = self.event_tx.send(TestnetEvent::GossipMessage {
                topic: message.topic,
                data: message.data,
                source: message.source,
            });
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
            _ => {}
        }
    }

    fn handle_identify_event(&mut self, event: identify::Event) {
        if let identify::Event::Received { peer_id, info, .. } = event {
            tracing::info!(
                peer_id = %peer_id,
                protocol_version = %info.protocol_version,
                agent_version = %info.agent_version,
                "🆔 Identify received"
            );

            // Adressen zu Kademlia hinzufügen
            for addr in info.listen_addrs {
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, addr);
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
