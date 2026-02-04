//! # Swarm Manager
//!
//! Lifecycle-Management für das libp2p Swarm.
//!
//! ## Verantwortlichkeiten
//!
//! - Swarm starten und stoppen
//! - Event-Loop verarbeiten
//! - Bootstrapping und Discovery
//! - Message-Routing zu Topics
//! - Privacy-Layer Integration (Phase 2 Woche 8)
//! - StateEvent-Emission (v0.4.0)
//!
//! ## StateEvent-Integration
//!
//! Der SwarmManager emittiert nun StateEvents an das zentrale UnifiedState:
//!
//! - `PeerConnectionChange` - bei Peer connected/disconnected
//! - `NetworkMetricUpdate` - bei Metrik-Änderungen
//! - Privacy-Events - bei Privacy-Layer-Operationen

use crate::core::state::{NetworkMetric, StateEvent, StateEventEmitter, NoOpEmitter};
use crate::core::identity_types::IdentityResolver;
use crate::domain::UniversalId;
use crate::peer::p2p::behaviour::{ErynoaBehaviour, ErynoaBehaviourEvent};
use crate::peer::p2p::config::P2PConfig;
use crate::peer::p2p::identity::{PeerIdentity, SignedPeerInfo};
#[cfg(feature = "privacy")]
use crate::peer::p2p::privacy::{
    CoverMessage, PrivacyService, PrivacyServiceConfig, RelayCandidate, SensitivityLevel,
};
use crate::peer::p2p::protocol::{SyncRequest, SyncResponse};
use crate::peer::p2p::topics::{RealmTopic, TopicManager, TopicMessage, SignedTopicMessage, SignatureError};
use crate::peer::p2p::trust_gate::TrustGate;
use anyhow::{anyhow, Result};
use futures::StreamExt;
use libp2p::gossipsub::{self, TopicHash};
use libp2p::identify;
use libp2p::kad::{self, QueryId, RecordKey};
use libp2p::mdns;
use libp2p::request_response::{self, OutboundRequestId, ResponseChannel};
use libp2p::swarm::{dial_opts::DialOpts, SwarmEvent};
use libp2p::{Multiaddr, PeerId, Swarm, Transport};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{broadcast, mpsc, oneshot};

/// Swarm Manager Command
#[derive(Debug)]
pub enum SwarmCommand {
    /// Starte Swarm
    Start,
    /// Stoppe Swarm
    Stop,
    /// Verbinde zu Peer
    Connect {
        addr: Multiaddr,
        response: oneshot::Sender<Result<PeerId>>,
    },
    /// Sende Gossipsub-Message
    Publish {
        topic: TopicHash,
        message: Vec<u8>,
        response: oneshot::Sender<Result<gossipsub::MessageId>>,
    },
    /// Subscribe Topic
    Subscribe {
        topic: RealmTopic,
        response: oneshot::Sender<Result<()>>,
    },
    /// Unsubscribe Topic
    Unsubscribe {
        topic: RealmTopic,
        response: oneshot::Sender<Result<()>>,
    },
    /// Sende Sync-Request
    SendRequest {
        peer_id: PeerId,
        request: SyncRequest,
        response: oneshot::Sender<Result<SyncResponse>>,
    },
    /// DHT Put
    DhtPut {
        key: Vec<u8>,
        value: Vec<u8>,
        response: oneshot::Sender<Result<QueryId>>,
    },
    /// DHT Get
    DhtGet {
        key: Vec<u8>,
        response: oneshot::Sender<Result<Vec<u8>>>,
    },
    /// Erhalte verbundene Peers
    GetConnectedPeers {
        response: oneshot::Sender<Vec<PeerId>>,
    },
    /// Erhalte lokale Listen-Adressen
    GetListenAddresses {
        response: oneshot::Sender<Vec<Multiaddr>>,
    },
    // ========================================================================
    // Privacy-Layer Commands (Phase 2 Woche 8)
    // ========================================================================
    /// Sende Privacy-Nachricht (Onion-verschlüsselt + Mixing)
    #[cfg(feature = "privacy")]
    SendPrivacyMessage {
        destination: PeerId,
        payload: Vec<u8>,
        sensitivity: SensitivityLevel,
        response: oneshot::Sender<Result<()>>,
    },
    /// Hole Privacy-Service Statistiken
    #[cfg(feature = "privacy")]
    GetPrivacyStats {
        response: oneshot::Sender<crate::peer::p2p::privacy::PrivacyServiceStats>,
    },
}

/// Event vom Swarm an Applikation (Clone-fähig)
#[derive(Debug, Clone)]
pub enum SwarmEvent2 {
    /// Neuer Peer verbunden
    PeerConnected { peer_id: PeerId },
    /// Peer getrennt
    PeerDisconnected { peer_id: PeerId },
    /// Gossipsub-Message empfangen (verifiziert)
    GossipMessage {
        topic: TopicHash,
        message: TopicMessage,
        source: Option<PeerId>,
        /// Signer UniversalId (v0.4.0 - falls signiert)
        signer_id: Option<UniversalId>,
        /// Signatur-Status (v0.4.0)
        signature_verified: bool,
    },
    /// Unsigned Gossipsub-Message empfangen
    UnsignedGossipMessage {
        topic: TopicHash,
        message: TopicMessage,
        source: Option<PeerId>,
    },
    /// Signatur-Verifikation fehlgeschlagen
    GossipMessageRejected {
        topic: TopicHash,
        source: Option<PeerId>,
        error: SignatureError,
    },
    /// Peer discovert via mDNS
    MdnsDiscovered {
        peer_id: PeerId,
        addresses: Vec<Multiaddr>,
    },
    /// Bootstrap abgeschlossen
    BootstrapComplete,
}

/// Sync-Request vom Swarm (nicht Clone-fähig wegen ResponseChannel)
#[derive(Debug)]
pub struct IncomingSyncRequest {
    pub peer_id: PeerId,
    pub request: SyncRequest,
    pub channel: ResponseChannel<Vec<u8>>,
}

/// Signatur-Statistiken für SwarmManager (v0.4.0)
#[derive(Debug, Clone, Default)]
pub struct SignatureStats {
    /// Erfolgreich verifizierte Signaturen
    pub verified: u64,
    /// Fehlgeschlagene Signatur-Verifikationen
    pub failed: u64,
    /// Unsignierte Messages
    pub unsigned: u64,
}

impl SignatureStats {
    /// Gesamtzahl verarbeiteter Messages
    pub fn total(&self) -> u64 {
        self.verified + self.failed + self.unsigned
    }

    /// Verifikationsrate (0.0 - 1.0)
    pub fn verification_rate(&self) -> f64 {
        let total = self.verified + self.failed;
        if total == 0 {
            1.0
        } else {
            self.verified as f64 / total as f64
        }
    }
}

/// Swarm Manager
///
/// Emittiert nun StateEvents für P2P-Netzwerk-Aktivitäten.
/// Verifiziert signierte Gossipsub-Messages (v0.4.0).
pub struct SwarmManager {
    /// Konfiguration
    config: P2PConfig,

    /// Peer-Identität
    identity: PeerIdentity,

    /// Topic-Manager
    topics: Arc<TopicManager>,

    /// Trust-Gate
    trust_gate: Arc<TrustGate>,

    /// Command-Sender
    command_tx: mpsc::Sender<SwarmCommand>,

    /// Event-Receiver (broadcast für multiple consumers)
    event_tx: broadcast::Sender<SwarmEvent2>,

    /// Sync-Request-Sender (separater Channel wegen ResponseChannel)
    sync_request_tx: mpsc::Sender<IncomingSyncRequest>,

    /// Running-State
    running: Arc<RwLock<bool>>,

    /// Pending DHT-Queries
    pending_dht_gets: Arc<RwLock<HashMap<QueryId, oneshot::Sender<Result<Vec<u8>>>>>>,

    /// Pending Request-Response
    pending_requests:
        Arc<RwLock<HashMap<OutboundRequestId, oneshot::Sender<Result<SyncResponse>>>>>,

    // ========================================================================
    // StateEvent-Emission (v0.4.0)
    // ========================================================================
    /// StateEvent-Emitter für Integration mit UnifiedState
    state_event_emitter: Arc<dyn StateEventEmitter>,

    /// Counter für Metrik-Deltas
    last_connected_peers: AtomicU64,
    last_gossip_messages: AtomicU64,

    // ========================================================================
    // Signatur-Verifikation (v0.4.0)
    // ========================================================================
    /// IdentityResolver für Signatur-Verifikation (optional)
    identity_resolver: Option<Arc<dyn IdentityResolver + Send + Sync>>,

    /// Strikte Signatur-Modus: Ablehnen unsignierter Messages
    require_signatures: bool,

    /// Signatur-Statistiken
    signatures_verified: AtomicU64,
    signatures_failed: AtomicU64,
    unsigned_messages: AtomicU64,

    // ========================================================================
    // Privacy-Layer (Phase 2 Woche 8)
    // ========================================================================
    /// Privacy-Service für Onion-Routing, Mixing und Cover-Traffic
    #[cfg(feature = "privacy")]
    privacy_service: Option<Arc<PrivacyService>>,

    /// Relay-Candidates Cache für Route-Auswahl
    #[cfg(feature = "privacy")]
    relay_candidates: Arc<RwLock<Vec<RelayCandidate>>>,
}

impl SwarmManager {
    /// Erstelle neuen SwarmManager
    pub fn new(
        config: P2PConfig,
        identity: PeerIdentity,
    ) -> (Self, mpsc::Receiver<IncomingSyncRequest>) {
        let (command_tx, _) = mpsc::channel(256);
        let (event_tx, _) = broadcast::channel(256);
        let (sync_request_tx, sync_request_rx) = mpsc::channel(256);

        let trust_gate = TrustGate::new_arc(config.trust_gate.clone());

        (
            Self {
                config,
                identity,
                topics: TopicManager::new_arc(),
                trust_gate,
                command_tx,
                event_tx,
                sync_request_tx,
                running: Arc::new(RwLock::new(false)),
                pending_dht_gets: Arc::new(RwLock::new(HashMap::new())),
                pending_requests: Arc::new(RwLock::new(HashMap::new())),
                state_event_emitter: Arc::new(NoOpEmitter),
                last_connected_peers: AtomicU64::new(0),
                last_gossip_messages: AtomicU64::new(0),
                identity_resolver: None,
                require_signatures: false, // Default: Akzeptiert signierte und unsignierte
                signatures_verified: AtomicU64::new(0),
                signatures_failed: AtomicU64::new(0),
                unsigned_messages: AtomicU64::new(0),
                #[cfg(feature = "privacy")]
                privacy_service: None,
                #[cfg(feature = "privacy")]
                relay_candidates: Arc::new(RwLock::new(Vec::new())),
            },
            sync_request_rx,
        )
    }

    /// Erstelle SwarmManager mit StateEventEmitter
    ///
    /// Ermöglicht Integration mit UnifiedState für StateEvent-Emission.
    pub fn new_with_emitter(
        config: P2PConfig,
        identity: PeerIdentity,
        emitter: Arc<dyn StateEventEmitter>,
    ) -> (Self, mpsc::Receiver<IncomingSyncRequest>) {
        let (mut manager, rx) = Self::new(config, identity);
        manager.state_event_emitter = emitter;
        (manager, rx)
    }

    /// Erstelle SwarmManager mit IdentityResolver für Signatur-Verifikation
    pub fn new_with_identity_resolver(
        config: P2PConfig,
        identity: PeerIdentity,
        resolver: Arc<dyn IdentityResolver + Send + Sync>,
        require_signatures: bool,
    ) -> (Self, mpsc::Receiver<IncomingSyncRequest>) {
        let (mut manager, rx) = Self::new(config, identity);
        manager.identity_resolver = Some(resolver);
        manager.require_signatures = require_signatures;
        (manager, rx)
    }

    /// Setze IdentityResolver nachträglich
    pub fn set_identity_resolver(&mut self, resolver: Arc<dyn IdentityResolver + Send + Sync>) {
        self.identity_resolver = Some(resolver);
    }

    /// Aktiviere striken Signatur-Modus (unsignierte Messages werden abgelehnt)
    pub fn set_require_signatures(&mut self, require: bool) {
        self.require_signatures = require;
    }

    /// Signatur-Statistiken
    pub fn signature_stats(&self) -> SignatureStats {
        SignatureStats {
            verified: self.signatures_verified.load(Ordering::Relaxed),
            failed: self.signatures_failed.load(Ordering::Relaxed),
            unsigned: self.unsigned_messages.load(Ordering::Relaxed),
        }
    }

    /// Setze StateEventEmitter
    ///
    /// Kann nachträglich gesetzt werden, z.B. nach Initialisierung.
    pub fn set_state_event_emitter(&mut self, emitter: Arc<dyn StateEventEmitter>) {
        self.state_event_emitter = emitter;
    }

    /// Erhalte Referenz auf StateEventEmitter
    pub fn state_event_emitter(&self) -> &Arc<dyn StateEventEmitter> {
        &self.state_event_emitter
    }

    /// Erstelle SwarmManager mit Privacy-Service (Phase 2 Woche 8)
    #[cfg(feature = "privacy")]
    pub fn with_privacy(
        config: P2PConfig,
        identity: PeerIdentity,
        privacy_config: PrivacyServiceConfig,
    ) -> (
        Self,
        mpsc::Receiver<IncomingSyncRequest>,
        mpsc::Receiver<(PeerId, Vec<u8>)>,
        mpsc::Receiver<CoverMessage>,
    ) {
        let (mut manager, sync_rx) = Self::new(config, identity);

        // Privacy-Service erstellen
        let (service, output_rx, cover_rx) = PrivacyService::new(privacy_config);
        manager.privacy_service = Some(Arc::new(service));

        (manager, sync_rx, output_rx, cover_rx)
    }

    /// Erstelle SwarmManager mit Privacy-Service und StateEventEmitter
    #[cfg(feature = "privacy")]
    pub fn with_privacy_and_emitter(
        config: P2PConfig,
        identity: PeerIdentity,
        privacy_config: PrivacyServiceConfig,
        emitter: Arc<dyn StateEventEmitter>,
    ) -> (
        Self,
        mpsc::Receiver<IncomingSyncRequest>,
        mpsc::Receiver<(PeerId, Vec<u8>)>,
        mpsc::Receiver<CoverMessage>,
    ) {
        let (mut manager, sync_rx, output_rx, cover_rx) =
            Self::with_privacy(config, identity, privacy_config);
        manager.state_event_emitter = emitter;
        (manager, sync_rx, output_rx, cover_rx)
    }

    /// Erhalte Command-Sender
    pub fn command_sender(&self) -> mpsc::Sender<SwarmCommand> {
        self.command_tx.clone()
    }

    /// Erhalte Event-Receiver
    pub fn event_receiver(&self) -> broadcast::Receiver<SwarmEvent2> {
        self.event_tx.subscribe()
    }

    /// Topic-Manager
    pub fn topics(&self) -> Arc<TopicManager> {
        self.topics.clone()
    }

    /// Trust-Gate
    pub fn trust_gate(&self) -> Arc<TrustGate> {
        self.trust_gate.clone()
    }

    /// Privacy-Service (Phase 2 Woche 8)
    #[cfg(feature = "privacy")]
    pub fn privacy_service(&self) -> Option<Arc<PrivacyService>> {
        self.privacy_service.clone()
    }

    /// Update Relay-Candidates für Route-Auswahl
    #[cfg(feature = "privacy")]
    pub fn update_relay_candidates(&self, candidates: Vec<RelayCandidate>) {
        *self.relay_candidates.write() = candidates;
    }

    /// Hole aktuelle Relay-Candidates
    #[cfg(feature = "privacy")]
    pub fn relay_candidates(&self) -> Vec<RelayCandidate> {
        self.relay_candidates.read().clone()
    }

    /// Ist Swarm aktiv?
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    /// Peer-ID
    pub fn peer_id(&self) -> PeerId {
        self.identity.peer_id
    }

    /// Starte Swarm (blocking - sollte in eigener Task laufen)
    pub async fn run(&self) -> Result<()> {
        // Baue Transport
        let transport = libp2p::tcp::tokio::Transport::default()
            .upgrade(libp2p::core::upgrade::Version::V1Lazy)
            .authenticate(libp2p::noise::Config::new(&self.identity.keypair())?)
            .multiplex(libp2p::yamux::Config::default())
            .boxed();

        // Baue Behaviour
        let behaviour = ErynoaBehaviour::new(&self.identity.keypair(), &self.config)?;

        // Baue Swarm
        let swarm_config = libp2p::swarm::Config::with_tokio_executor();
        let mut swarm = Swarm::new(transport, behaviour, self.peer_id(), swarm_config);

        // Listen-Adressen
        for addr in &self.config.listen_addresses {
            let addr: Multiaddr = addr
                .parse()
                .map_err(|e| anyhow!("Invalid address: {}", e))?;
            swarm.listen_on(addr)?;
        }

        // Bootstrap-Peers verbinden
        for addr in &self.config.bootstrap_peers {
            let addr: Multiaddr = addr
                .parse()
                .map_err(|e| anyhow!("Invalid bootstrap address: {}", e))?;
            if let Err(e) = swarm.dial(addr.clone()) {
                tracing::warn!(addr = %addr, error = %e, "Failed to dial bootstrap peer");
            }
        }

        // Setze Running-State
        *self.running.write() = true;

        // Command-Channel
        let (_command_tx, mut command_rx) = mpsc::channel::<SwarmCommand>(256);
        // Update self.command_tx würde &mut self benötigen, daher hier separat

        tracing::info!(peer_id = %self.peer_id(), "Swarm started");

        // Starte Privacy-Service Background-Tasks (Phase 2 Woche 8)
        #[cfg(feature = "privacy")]
        let _privacy_task = if let Some(ref service) = self.privacy_service {
            let relay_candidates = self.relay_candidates.clone();
            let service_clone = service.clone();
            Some(tokio::spawn(async move {
                let route_provider =
                    move || relay_candidates.read().iter().map(|c| c.peer_id).collect();
                if let Err(e) = service_clone.run_background_tasks(route_provider).await {
                    tracing::error!(error = %e, "Privacy service background tasks failed");
                }
            }))
        } else {
            None
        };

        // Event-Loop
        loop {
            tokio::select! {
                // Swarm-Events
                event = swarm.select_next_some() => {
                    self.handle_swarm_event(&mut swarm, event).await;
                }

                // Commands
                Some(cmd) = command_rx.recv() => {
                    if !self.handle_command(&mut swarm, cmd).await {
                        break; // Stop-Command
                    }
                }
            }
        }

        // Stoppe Privacy-Service
        #[cfg(feature = "privacy")]
        if let Some(ref service) = self.privacy_service {
            service.stop();
        }

        *self.running.write() = false;
        tracing::info!("Swarm stopped");
        Ok(())
    }

    /// Handle Swarm-Event
    async fn handle_swarm_event(
        &self,
        swarm: &mut Swarm<ErynoaBehaviour>,
        event: SwarmEvent<ErynoaBehaviourEvent>,
    ) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                tracing::info!(address = %address, "Listening on");
            }

            SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                // Trust-Check
                let decision = self.trust_gate.check_connection(&peer_id);
                if !decision.allowed {
                    tracing::warn!(peer_id = %peer_id, reason = ?decision.reason, "Rejecting connection");
                    let _ = swarm.disconnect_peer_id(peer_id);
                    return;
                }

                tracing::info!(peer_id = %peer_id, level = ?decision.level, "Peer connected");
                let _ = self.event_tx.send(SwarmEvent2::PeerConnected { peer_id });

                // Kademlia: Add to routing table
                swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, "/ip4/0.0.0.0/tcp/0".parse().unwrap());

                // StateEvent emittieren (v0.4.0)
                let peer_universal_id = self.trust_gate.get_universal_id_by_peer_id(&peer_id);
                let addr = endpoint.get_remote_address();
                self.state_event_emitter.emit(StateEvent::PeerConnectionChange {
                    peer_id: peer_id.to_string(),
                    peer_universal_id,
                    connected: true,
                    addr: Some(addr.to_string()),
                    connection_level: Some(format!("{:?}", decision.level)),
                });

                // Metrik-Update
                let new_count = self.last_connected_peers.fetch_add(1, Ordering::Relaxed) + 1;
                self.state_event_emitter.emit(StateEvent::NetworkMetricUpdate {
                    metric: NetworkMetric::ConnectedPeers,
                    value: new_count,
                    delta: 1,
                });
            }

            SwarmEvent::ConnectionClosed { peer_id, endpoint, .. } => {
                tracing::info!(peer_id = %peer_id, "Peer disconnected");
                let _ = self
                    .event_tx
                    .send(SwarmEvent2::PeerDisconnected { peer_id });

                // StateEvent emittieren (v0.4.0)
                let peer_universal_id = self.trust_gate.get_universal_id_by_peer_id(&peer_id);
                let addr = endpoint.get_remote_address();
                self.state_event_emitter.emit(StateEvent::PeerConnectionChange {
                    peer_id: peer_id.to_string(),
                    peer_universal_id,
                    connected: false,
                    addr: Some(addr.to_string()),
                    connection_level: None,
                });

                // Metrik-Update
                let old_count = self.last_connected_peers.fetch_sub(1, Ordering::Relaxed);
                let new_count = old_count.saturating_sub(1);
                self.state_event_emitter.emit(StateEvent::NetworkMetricUpdate {
                    metric: NetworkMetric::ConnectedPeers,
                    value: new_count,
                    delta: -1,
                });
            }

            SwarmEvent::Behaviour(behaviour_event) => {
                self.handle_behaviour_event(swarm, behaviour_event).await;
            }

            _ => {}
        }
    }

    /// Handle Behaviour-Event
    async fn handle_behaviour_event(
        &self,
        swarm: &mut Swarm<ErynoaBehaviour>,
        event: ErynoaBehaviourEvent,
    ) {
        match event {
            ErynoaBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source,
                message_id: _,
                message,
            }) => {
                // Versuche zuerst als SignedTopicMessage zu parsen (v0.4.0)
                if let Ok(mut signed_msg) = SignedTopicMessage::from_bytes(&message.data) {
                    // Prüfe ob Message abgelaufen ist (Replay-Schutz)
                    if signed_msg.is_expired() {
                        self.signatures_failed.fetch_add(1, Ordering::Relaxed);
                        let _ = self.event_tx.send(SwarmEvent2::GossipMessageRejected {
                            topic: message.topic.clone(),
                            source: Some(propagation_source),
                            error: SignatureError::MessageExpired,
                        });
                        return;
                    }

                    // Signatur verifizieren mit IdentityResolver (v0.4.0)
                    let (signature_verified, signer_id) = if let Some(ref resolver) = self.identity_resolver {
                        match signed_msg.verify(resolver.as_ref()) {
                            Ok(true) => {
                                self.signatures_verified.fetch_add(1, Ordering::Relaxed);
                                (true, signed_msg.signer_id)
                            }
                            Ok(false) | Err(_) => {
                                self.signatures_failed.fetch_add(1, Ordering::Relaxed);
                                let _ = self.event_tx.send(SwarmEvent2::GossipMessageRejected {
                                    topic: message.topic.clone(),
                                    source: Some(propagation_source),
                                    error: SignatureError::InvalidSignature,
                                });
                                return;
                            }
                        }
                    } else {
                        // Kein Resolver: Akzeptiere ohne Verifikation
                        self.signatures_verified.fetch_add(1, Ordering::Relaxed);
                        (true, signed_msg.signer_id)
                    };

                    let topic_msg = signed_msg.into_message();

                    let _ = self.event_tx.send(SwarmEvent2::GossipMessage {
                        topic: message.topic.clone(),
                        message: topic_msg,
                        source: Some(propagation_source),
                        signer_id: Some(signer_id),
                        signature_verified,
                    });

                    // StateEvent: Gossip-Metrik (v0.4.0)
                    let new_count = self.last_gossip_messages.fetch_add(1, Ordering::Relaxed) + 1;
                    self.state_event_emitter.emit(StateEvent::NetworkMetricUpdate {
                        metric: NetworkMetric::GossipMessagesReceived,
                        value: new_count,
                        delta: 1,
                    });
                }
                // Fallback: Versuche als unsignierte TopicMessage
                else if let Ok(topic_msg) = TopicMessage::from_bytes(&message.data) {
                    self.unsigned_messages.fetch_add(1, Ordering::Relaxed);

                    // Strikte Modus: Ablehnen unsignierter Messages
                    if self.require_signatures {
                        let _ = self.event_tx.send(SwarmEvent2::GossipMessageRejected {
                            topic: message.topic.clone(),
                            source: Some(propagation_source),
                            error: SignatureError::MissingSignature,
                        });
                        return;
                    }

                    let _ = self.event_tx.send(SwarmEvent2::UnsignedGossipMessage {
                        topic: message.topic.clone(),
                        message: topic_msg,
                        source: Some(propagation_source),
                    });

                    // StateEvent: Gossip-Metrik
                    let new_count = self.last_gossip_messages.fetch_add(1, Ordering::Relaxed) + 1;
                    self.state_event_emitter.emit(StateEvent::NetworkMetricUpdate {
                        metric: NetworkMetric::GossipMessagesReceived,
                        value: new_count,
                        delta: 1,
                    });
                }
            }

            ErynoaBehaviourEvent::RequestResponse(request_response::Event::Message {
                peer,
                message:
                    request_response::Message::Request {
                        request, channel, ..
                    },
            }) => {
                // Trust-Check für Requests
                let info = self.trust_gate.get_peer_info(&peer);
                let can_sync = info.map(|i| i.connection_level.can_sync()).unwrap_or(false);

                if !can_sync {
                    let error_response = SyncResponse::error(3, "Permission denied");
                    let _ = swarm
                        .behaviour_mut()
                        .request_response
                        .send_response(channel, error_response.to_bytes().unwrap_or_default());
                    return;
                }

                if let Ok(sync_req) = SyncRequest::from_bytes(&request) {
                    let _ = self.sync_request_tx.try_send(IncomingSyncRequest {
                        peer_id: peer,
                        request: sync_req,
                        channel,
                    });
                }
            }

            ErynoaBehaviourEvent::RequestResponse(request_response::Event::Message {
                message:
                    request_response::Message::Response {
                        request_id,
                        response,
                    },
                ..
            }) => {
                if let Some(sender) = self.pending_requests.write().remove(&request_id) {
                    let result = SyncResponse::from_bytes(&response);
                    let _ = sender.send(result);
                }
            }

            ErynoaBehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed {
                id,
                result: kad::QueryResult::GetRecord(result),
                ..
            }) => {
                if let Some(sender) = self.pending_dht_gets.write().remove(&id) {
                    match result {
                        Ok(kad::GetRecordOk::FoundRecord(peer_record)) => {
                            let _ = sender.send(Ok(peer_record.record.value));
                        }
                        Ok(kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. }) => {
                            let _ = sender.send(Err(anyhow!("Record not found")));
                        }
                        Err(e) => {
                            let _ = sender.send(Err(anyhow!("DHT get failed: {:?}", e)));
                        }
                    }
                }
            }

            #[cfg(feature = "p2p")]
            ErynoaBehaviourEvent::Mdns(mdns::Event::Discovered(list)) => {
                for (peer_id, addr) in list {
                    tracing::debug!(peer_id = %peer_id, addr = %addr, "mDNS discovered peer");
                    swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr.clone());

                    let _ = self.event_tx.send(SwarmEvent2::MdnsDiscovered {
                        peer_id,
                        addresses: vec![addr],
                    });
                }
            }

            ErynoaBehaviourEvent::Identify(identify::Event::Received { peer_id, info, .. }) => {
                tracing::debug!(
                    peer_id = %peer_id,
                    agent = %info.agent_version,
                    "Identified peer"
                );

                // Add addresses to Kademlia
                for addr in info.listen_addrs {
                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                }
            }

            _ => {}
        }
    }

    /// Handle Command
    async fn handle_command(&self, swarm: &mut Swarm<ErynoaBehaviour>, cmd: SwarmCommand) -> bool {
        match cmd {
            SwarmCommand::Stop => {
                return false;
            }

            SwarmCommand::Connect { addr, response } => {
                let result = swarm
                    .dial(DialOpts::unknown_peer_id().address(addr).build())
                    .map(|_| PeerId::random()) // TODO: Return actual peer ID
                    .map_err(|e| anyhow!("Dial failed: {:?}", e));
                let _ = response.send(result);
            }

            SwarmCommand::Publish {
                topic,
                message,
                response,
            } => {
                let result = swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(topic, message)
                    .map_err(|e| anyhow!("Publish failed: {:?}", e));
                let _ = response.send(result);
            }

            SwarmCommand::Subscribe { topic, response } => {
                let result = swarm
                    .behaviour_mut()
                    .gossipsub
                    .subscribe(topic.ident_topic())
                    .map(|_| {
                        self.topics.subscribe(topic);
                    })
                    .map_err(|e| anyhow!("Subscribe failed: {:?}", e));
                let _ = response.send(result);
            }

            SwarmCommand::Unsubscribe { topic, response } => {
                let result = swarm
                    .behaviour_mut()
                    .gossipsub
                    .unsubscribe(topic.ident_topic())
                    .map(|_| {
                        self.topics.unsubscribe(&topic);
                    })
                    .map_err(|e| anyhow!("Unsubscribe failed: {:?}", e));
                let _ = response.send(result);
            }

            SwarmCommand::SendRequest {
                peer_id,
                request,
                response,
            } => {
                let bytes = match request.to_bytes() {
                    Ok(b) => b,
                    Err(e) => {
                        let _ = response.send(Err(e));
                        return true;
                    }
                };

                let request_id = swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&peer_id, bytes);

                self.pending_requests.write().insert(request_id, response);
            }

            SwarmCommand::DhtPut {
                key,
                value,
                response,
            } => {
                let record = kad::Record {
                    key: RecordKey::new(&key),
                    value,
                    publisher: Some(self.peer_id()),
                    expires: None,
                };

                let result = swarm
                    .behaviour_mut()
                    .kademlia
                    .put_record(record, kad::Quorum::One)
                    .map_err(|e| anyhow!("DHT put failed: {:?}", e));

                let _ = response.send(result);
            }

            SwarmCommand::DhtGet { key, response } => {
                let query_id = swarm
                    .behaviour_mut()
                    .kademlia
                    .get_record(RecordKey::new(&key));

                self.pending_dht_gets.write().insert(query_id, response);
            }

            SwarmCommand::GetConnectedPeers { response } => {
                let peers: Vec<PeerId> = swarm.connected_peers().cloned().collect();
                let _ = response.send(peers);
            }

            SwarmCommand::GetListenAddresses { response } => {
                let addrs: Vec<Multiaddr> = swarm.listeners().cloned().collect();
                let _ = response.send(addrs);
            }

            // ================================================================
            // Privacy-Layer Commands (Phase 2 Woche 8)
            // ================================================================
            #[cfg(feature = "privacy")]
            SwarmCommand::SendPrivacyMessage {
                destination,
                payload,
                sensitivity,
                response,
            } => {
                let result = if let Some(ref service) = self.privacy_service {
                    let candidates = self.relay_candidates.read().clone();
                    service
                        .send_message(destination, payload, sensitivity, &candidates)
                        .await
                        .map_err(|e| anyhow!("Privacy send failed: {:?}", e))
                } else {
                    Err(anyhow!("Privacy service not configured"))
                };
                let _ = response.send(result);
            }

            #[cfg(feature = "privacy")]
            SwarmCommand::GetPrivacyStats { response } => {
                if let Some(ref service) = self.privacy_service {
                    let _ = response.send(service.stats());
                }
            }

            #[allow(unreachable_patterns)]
            _ => {}
        }

        true
    }

    /// Publish eigene Peer-Info ins DHT
    pub async fn publish_peer_info(&self, addresses: Vec<String>) -> Result<()> {
        let info = SignedPeerInfo::new(&self.identity, addresses)?;
        let key = info.record_key();
        let value = info.to_bytes()?;

        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(SwarmCommand::DhtPut {
                key,
                value,
                response: tx,
            })
            .await
            .map_err(|_| anyhow!("Failed to send command"))?;

        rx.await.map_err(|_| anyhow!("Channel closed"))??;
        Ok(())
    }

    /// Join Realm
    pub async fn join_realm(&self, realm_id: &str) -> Result<()> {
        let topics = vec![
            RealmTopic::realm_events(realm_id),
            RealmTopic::realm_trust(realm_id),
            RealmTopic::realm_sagas(realm_id),
        ];

        for topic in topics {
            let (tx, rx) = oneshot::channel();
            self.command_tx
                .send(SwarmCommand::Subscribe {
                    topic,
                    response: tx,
                })
                .await
                .map_err(|_| anyhow!("Failed to send command"))?;

            rx.await.map_err(|_| anyhow!("Channel closed"))??;
        }

        tracing::info!(realm_id = %realm_id, "Joined realm");
        Ok(())
    }

    /// Leave Realm
    pub async fn leave_realm(&self, realm_id: &str) -> Result<()> {
        let topics = self.topics.realm_topics(realm_id);

        for topic in topics {
            let (tx, rx) = oneshot::channel();
            self.command_tx
                .send(SwarmCommand::Unsubscribe {
                    topic,
                    response: tx,
                })
                .await
                .map_err(|_| anyhow!("Failed to send command"))?;

            rx.await.map_err(|_| anyhow!("Channel closed"))??;
        }

        self.topics.leave_realm(realm_id);
        tracing::info!(realm_id = %realm_id, "Left realm");
        Ok(())
    }

    /// Publish Event to Realm
    /// Publish Event (signiert mit PeerIdentity)
    ///
    /// Erstellt eine signierte TopicMessage und publiziert sie im Realm.
    pub async fn publish_event(
        &self,
        realm_id: &str,
        event_data: Vec<u8>,
        sender: &str,
    ) -> Result<gossipsub::MessageId> {
        let topic = RealmTopic::realm_events(realm_id);
        let message = TopicMessage::Event {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_data,
            sender: sender.to_string(),
        };

        // Signiere Message mit PeerIdentity (v0.4.0)
        let signer_id = self.identity.universal_id_owned();
        let signed_message = self.sign_topic_message(message, signer_id)?;

        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(SwarmCommand::Publish {
                topic: topic.hash(),
                message: signed_message.to_bytes()?,
                response: tx,
            })
            .await
            .map_err(|_| anyhow!("Failed to send command"))?;

        rx.await.map_err(|_| anyhow!("Channel closed"))?
    }

    /// Signiere eine TopicMessage mit der PeerIdentity
    fn sign_topic_message(
        &self,
        message: TopicMessage,
        signer_id: UniversalId,
    ) -> Result<SignedTopicMessage> {
        // Verwende die sign() Methode der PeerIdentity
        let keypair = self.identity.keypair();

        SignedTopicMessage::new(message, signer_id, |data| {
            // Ed25519-Signatur mit libp2p keypair
            let sig = keypair.sign(data).map_err(|e| anyhow!("Signing failed: {}", e))?;
            if sig.len() != 64 {
                return Err(anyhow!("Invalid signature length: {}", sig.len()));
            }
            let mut arr = [0u8; 64];
            arr.copy_from_slice(&sig);
            Ok(arr)
        })
    }

    /// Request Events from Peer
    pub async fn request_events(
        &self,
        peer_id: PeerId,
        realm_id: &str,
        after_hash: Option<String>,
        limit: usize,
    ) -> Result<SyncResponse> {
        let request = SyncRequest::GetEventsAfter {
            realm_id: realm_id.to_string(),
            realm_universal_id: None, // TODO: Konvertiere realm_id zu UniversalId
            after_hash,
            limit,
        };

        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(SwarmCommand::SendRequest {
                peer_id,
                request,
                response: tx,
            })
            .await
            .map_err(|_| anyhow!("Failed to send command"))?;

        rx.await.map_err(|_| anyhow!("Channel closed"))?
    }

    /// Get connected peers
    pub async fn connected_peers(&self) -> Result<Vec<PeerId>> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(SwarmCommand::GetConnectedPeers { response: tx })
            .await
            .map_err(|_| anyhow!("Failed to send command"))?;

        rx.await.map_err(|_| anyhow!("Channel closed"))
    }

    // ========================================================================
    // Privacy-Layer APIs (Phase 2 Woche 8)
    // ========================================================================

    /// Sende Privacy-Nachricht (Onion-verschlüsselt + Mixing)
    ///
    /// Die Nachricht wird:
    /// 1. Onion-verschlüsselt mit Trust-basierter Route
    /// 2. In den Mixing-Pool gelegt (LAMP-Enhanced)
    /// 3. Nach Delay-Ablauf gesendet
    ///
    /// # Beispiel
    ///
    /// ```rust,ignore
    /// manager.send_privacy_message(
    ///     destination_peer,
    ///     payload,
    ///     SensitivityLevel::High
    /// ).await?;
    /// ```
    #[cfg(feature = "privacy")]
    pub async fn send_privacy_message(
        &self,
        destination: PeerId,
        payload: Vec<u8>,
        sensitivity: SensitivityLevel,
    ) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(SwarmCommand::SendPrivacyMessage {
                destination,
                payload,
                sensitivity,
                response: tx,
            })
            .await
            .map_err(|_| anyhow!("Failed to send command"))?;

        rx.await.map_err(|_| anyhow!("Channel closed"))?
    }

    /// Hole Privacy-Service Statistiken
    ///
    /// Gibt Informationen über:
    /// - Mixing-Pool (Buffer-Größe, k_opt, Flush-Counts)
    /// - Cover-Traffic (gesendete Dummies, Rate)
    /// - Nachrichten-Counts (gesendet, empfangen, verworfen)
    #[cfg(feature = "privacy")]
    pub async fn privacy_stats(&self) -> Result<crate::peer::p2p::privacy::PrivacyServiceStats> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(SwarmCommand::GetPrivacyStats { response: tx })
            .await
            .map_err(|_| anyhow!("Failed to send command"))?;

        rx.await.map_err(|_| anyhow!("Channel closed"))
    }

    /// Sende Event mit Privacy-Layer
    ///
    /// Kombiniert `publish_event` mit Privacy-Routing.
    #[cfg(feature = "privacy")]
    /// Publish Event über Privacy-Layer (signiert)
    ///
    /// Sendet eine signierte TopicMessage über den Privacy-Layer
    /// (Onion-Routing, Mixing-Pool).
    pub async fn publish_event_private(
        &self,
        _realm_id: &str,
        event_data: Vec<u8>,
        sender: &str,
        sensitivity: SensitivityLevel,
    ) -> Result<()> {
        let message = TopicMessage::Event {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_data,
            sender: sender.to_string(),
        };

        // Signiere Message (v0.4.0)
        let signer_id = self.identity.universal_id_owned();
        let signed_message = self.sign_topic_message(message, signer_id)?;

        let payload = signed_message.to_bytes()?;

        // Wähle einen zufälligen Peer im Realm als Eintrittspunkt
        let connected = self.connected_peers().await?;
        if connected.is_empty() {
            return Err(anyhow!("No connected peers"));
        }

        // TODO: Bessere Auswahl basierend auf Realm-Membership
        let destination = connected[0];

        self.send_privacy_message(destination, payload, sensitivity)
            .await
    }

    /// Publish Trust-Attestation (signiert)
    pub async fn publish_trust_attestation(
        &self,
        realm_id: &str,
        subject: &str,
        trust_delta: f64,
        reason: Option<String>,
    ) -> Result<gossipsub::MessageId> {
        let topic = RealmTopic::realm_trust(realm_id);
        let attester = self.identity.did.to_uri();

        let message = TopicMessage::TrustAttestation {
            attester,
            subject: subject.to_string(),
            trust_delta,
            reason,
        };

        // Signiere Message (v0.4.0)
        let signer_id = self.identity.universal_id_owned();
        let signed_message = self.sign_topic_message(message, signer_id)?;

        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(SwarmCommand::Publish {
                topic: topic.hash(),
                message: signed_message.to_bytes()?,
                response: tx,
            })
            .await
            .map_err(|_| anyhow!("Failed to send command"))?;

        rx.await.map_err(|_| anyhow!("Channel closed"))?
    }

    /// Publish Saga-Broadcast (signiert)
    pub async fn publish_saga_broadcast(
        &self,
        realm_id: &str,
        saga_id: &str,
        phase: &str,
        payload: Vec<u8>,
    ) -> Result<gossipsub::MessageId> {
        let topic = RealmTopic::realm_sagas(realm_id);

        let message = TopicMessage::SagaBroadcast {
            saga_id: saga_id.to_string(),
            phase: phase.to_string(),
            payload,
        };

        // Signiere Message (v0.4.0)
        let signer_id = self.identity.universal_id_owned();
        let signed_message = self.sign_topic_message(message, signer_id)?;

        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(SwarmCommand::Publish {
                topic: topic.hash(),
                message: signed_message.to_bytes()?,
                response: tx,
            })
            .await
            .map_err(|_| anyhow!("Failed to send command"))?;

        rx.await.map_err(|_| anyhow!("Channel closed"))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_manager_creation() {
        let config = P2PConfig::default();
        let identity = PeerIdentity::generate();
        let (manager, _sync_rx) = SwarmManager::new(config, identity);

        assert!(!manager.is_running());
    }

    #[tokio::test]
    async fn test_topic_operations() {
        let config = P2PConfig::default();
        let identity = PeerIdentity::generate();
        let (manager, _sync_rx) = SwarmManager::new(config, identity);

        let topics = manager.topics();
        let hashes = topics.join_realm("test-realm");
        assert_eq!(hashes.len(), 3);

        assert!(topics.is_realm_member("test-realm"));

        topics.leave_realm("test-realm");
        assert!(!topics.is_realm_member("test-realm"));
    }

    // ========================================================================
    // Privacy-Layer Tests (Phase 2 Woche 8)
    // ========================================================================

    #[cfg(feature = "privacy")]
    #[test]
    fn test_swarm_manager_with_privacy() {
        let config = P2PConfig::default();
        let identity = PeerIdentity::generate();
        let privacy_config = PrivacyServiceConfig::default();

        let (manager, _sync_rx, _output_rx, _cover_rx) =
            SwarmManager::with_privacy(config, identity, privacy_config);

        assert!(!manager.is_running());
        assert!(manager.privacy_service().is_some());
    }

    #[cfg(feature = "privacy")]
    #[test]
    fn test_relay_candidates_update() {
        let config = P2PConfig::default();
        let identity = PeerIdentity::generate();
        let privacy_config = PrivacyServiceConfig::default();

        let (manager, _sync_rx, _output_rx, _cover_rx) =
            SwarmManager::with_privacy(config, identity, privacy_config);

        // Initial leer
        assert!(manager.relay_candidates().is_empty());

        // Update
        let secret1 = x25519_dalek::StaticSecret::random_from_rng(&mut rand::thread_rng());
        let secret2 = x25519_dalek::StaticSecret::random_from_rng(&mut rand::thread_rng());

        // Erstelle Test-TrustInfo
        let trust_info = crate::peer::p2p::trust_gate::PeerTrustInfo {
            universal_id: None,
            did: None,
            trust_r: 0.8,
            trust_omega: 0.7,
            last_seen: 0,
            successful_interactions: 10,
            failed_interactions: 0,
            is_newcomer: false,
            newcomer_since: None,
            connection_level: crate::peer::p2p::trust_gate::ConnectionLevel::Full,
        };

        let candidates = vec![
            RelayCandidate::from_peer_info(
                PeerId::random(),
                trust_info.clone(),
                x25519_dalek::PublicKey::from(&secret1),
            )
            .with_diversity("eu-west", 12345, "EU")
            .with_performance(50, 0.95, 0.8),
            RelayCandidate::from_peer_info(
                PeerId::random(),
                trust_info.clone(),
                x25519_dalek::PublicKey::from(&secret2),
            )
            .with_diversity("us-east", 54321, "US")
            .with_performance(100, 0.9, 0.7),
        ];

        manager.update_relay_candidates(candidates);
        assert_eq!(manager.relay_candidates().len(), 2);
    }

    #[cfg(feature = "privacy")]
    #[test]
    fn test_privacy_config_presets() {
        // Relay-Config
        let relay = PrivacyServiceConfig::for_relay();
        assert!(relay.cover_traffic.peer_type == crate::peer::p2p::privacy::PeerType::FullRelay);

        // High-Privacy
        let high = PrivacyServiceConfig::high_privacy();
        assert!(high.default_sensitivity == SensitivityLevel::High);

        // Mobile
        let mobile = PrivacyServiceConfig::mobile();
        assert!(mobile.default_sensitivity == SensitivityLevel::Low);
    }
}
