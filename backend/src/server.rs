//! Server module - Application startup and state management
//!
//! ## State Hierarchy
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                         APPLICATION STATE                           │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  AppState                                                           │
//! │  ├── unified_state: SharedUnifiedState (Core + Execution + ...)    │
//! │  ├── coordinator: StateCoordinator (Health + Invariants)           │
//! │  ├── storage: DecentralizedStorage (Persistence)                   │
//! │  ├── p2p_handle: Option<P2PHandle> (P2P-Netzwerk)                  │
//! │  └── config: Settings                                              │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## P2P-Integration
//!
//! Wenn `features.p2p_enabled = true`, wird das P2P-Netzwerk automatisch gestartet:
//! - libp2p Swarm mit Gossipsub, Kademlia, mDNS
//! - NAT-Traversal (AutoNAT, DCUTR, Relay, UPnP)
//! - Trust-Gate für Peer-Filterung
//! - Event-Integration mit UnifiedState

use crate::api::{create_router, create_static_router, StaticConfig};
use crate::config::Settings;
use crate::core::{create_unified_state, SharedUnifiedState, StateCoordinator};
use crate::local::DecentralizedStorage;
use crate::peer::gateway::GatewayGuard;
use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;

// P2P-Imports (feature-gated)
#[cfg(feature = "p2p")]
use crate::peer::p2p::{P2PConfig, PeerIdentity, SwarmManager, SwarmEvent2};
#[cfg(feature = "p2p")]
use tokio::sync::mpsc;

/// P2P-Handle für Kommunikation mit dem Swarm
#[cfg(feature = "p2p")]
#[derive(Clone)]
pub struct P2PHandle {
    /// Command-Sender an SwarmManager
    pub command_tx: mpsc::Sender<crate::peer::p2p::SwarmCommand>,
    /// Peer-ID dieses Nodes
    pub peer_id: String,
    /// Node-Name
    pub node_name: String,
}

#[cfg(not(feature = "p2p"))]
#[derive(Clone)]
pub struct P2PHandle;

/// Shared application state for all handlers
///
/// Enthält hierarchisches State-Management:
/// - `unified_state`: Atomar Counter für alle Module
/// - `coordinator`: Invarianten-Checks, Health-Aggregation, enthält StateIntegrator
/// - `storage`: Fjall-basierter dezentraler Storage
/// - `p2p_handle`: Optional P2P-Netzwerk-Handle
#[derive(Clone)]
pub struct AppState {
    /// Unified State für alle Module (Thread-safe)
    pub unified_state: SharedUnifiedState,

    /// State Coordinator (Health, Invarianten; Integrator via coordinator.integrator())
    pub coordinator: Arc<StateCoordinator>,

    /// Dezentraler Storage (Fjall)
    pub storage: DecentralizedStorage,

    /// Anwendungskonfiguration
    pub config: Arc<Settings>,

    /// Startzeitpunkt für Uptime
    pub started_at: Option<Instant>,

    /// Optional: GatewayGuard für Crossing-Validierung (Phase 2)
    pub gateway: Option<Arc<GatewayGuard>>,

    /// Optional: P2P-Handle für Netzwerk-Kommunikation
    pub p2p_handle: Option<P2PHandle>,
}

impl AppState {
    /// Erstelle neuen AppState mit Unified State Management
    pub fn new(storage: DecentralizedStorage, config: Settings) -> Self {
        // Unified State erstellen
        let unified_state = create_unified_state();

        // Coordinator (enthält einen StateIntegrator für Observer-Pattern)
        let coordinator = Arc::new(StateCoordinator::new(unified_state.clone()));

        Self {
            unified_state,
            coordinator,
            storage,
            config: Arc::new(config),
            started_at: Some(Instant::now()),
            gateway: None,
            p2p_handle: None,
        }
    }

    /// Erstelle AppState mit P2P-Handle
    pub fn with_p2p(mut self, p2p_handle: P2PHandle) -> Self {
        self.p2p_handle = Some(p2p_handle);
        self
    }

    /// Ist P2P aktiv?
    pub fn is_p2p_active(&self) -> bool {
        self.p2p_handle.is_some()
    }

    /// Check if storage is reachable
    pub async fn health_check(&self) -> bool {
        self.storage.ping().await.is_ok()
    }

    /// Get system health report
    pub fn health_report(&self) -> crate::core::HealthReport {
        self.coordinator.aggregate_health()
    }

    /// Get unified state snapshot
    pub fn state_snapshot(&self) -> crate::core::UnifiedSnapshot {
        self.unified_state.snapshot()
    }

    /// State Integrator (Observer-Pattern). Clone ist günstig (Arc).
    pub fn integrator(&self) -> crate::core::StateIntegrator {
        self.coordinator.integrator().clone()
    }

    /// Get uptime in seconds
    pub fn uptime_secs(&self) -> u64 {
        self.started_at.map(|s| s.elapsed().as_secs()).unwrap_or(0)
    }
}

/// Application server
pub struct Server {
    listener: TcpListener,
    router: Router,
    /// P2P-Task-Handle (falls P2P aktiv)
    #[cfg(feature = "p2p")]
    p2p_task: Option<tokio::task::JoinHandle<()>>,
    #[cfg(not(feature = "p2p"))]
    #[allow(dead_code)]
    p2p_task: Option<()>,
}

impl Server {
    /// Build the application from settings
    pub async fn build(settings: Settings) -> Result<Self> {
        Self::build_with_static(settings, None).await
    }

    /// Build the application with optional static file serving
    pub async fn build_with_static(settings: Settings, static_dir: Option<&str>) -> Result<Self> {
        tracing::info!(
            env = %settings.application.environment.as_str(),
            p2p_enabled = settings.features.p2p_enabled,
            "🏗️  Building server..."
        );

        // Dezentraler Storage (Fjall)
        let data_dir = &settings.storage.data_dir;
        let storage = DecentralizedStorage::open(data_dir)?;
        tracing::info!(path = %data_dir, "✅ Decentralized storage ready");

        // AppState mit Unified State Management
        let mut state = AppState::new(storage, settings.clone());
        tracing::info!("✅ Unified state management initialized");

        // P2P initialisieren (falls aktiviert)
        #[cfg(feature = "p2p")]
        let p2p_task = if settings.features.p2p_enabled {
            let (p2p_handle, task) = Self::init_p2p(&settings, state.unified_state.clone()).await?;
            state = state.with_p2p(p2p_handle);
            Some(task)
        } else {
            tracing::info!("ℹ️  P2P disabled (set features.p2p_enabled = true to enable)");
            None
        };

        #[cfg(not(feature = "p2p"))]
        let p2p_task: Option<()> = {
            if settings.features.p2p_enabled {
                tracing::warn!("⚠️  P2P requested but not compiled (use --features p2p)");
            }
            None
        };

        // API Router
        let api_router = create_router(state);

        // Kombiniere API mit optionalem Static File Serving
        let router = if let Some(dir) = static_dir {
            let static_config = StaticConfig::new(dir);
            let static_router = create_static_router(&static_config);

            if static_config.is_available() {
                tracing::info!(
                    path = %dir,
                    "📁 Static file serving enabled"
                );
                // Static routes haben niedrigere Priorität als API
                api_router.merge(static_router)
            } else {
                tracing::warn!(
                    path = %dir,
                    "⚠️  Static directory not found - serving API only"
                );
                api_router
            }
        } else {
            api_router
        };

        let addr = format!(
            "{}:{}",
            settings.application.host, settings.application.port
        );
        let listener = TcpListener::bind(&addr).await?;

        tracing::info!(addr = %addr, "🚀 Server ready");

        Ok(Self { listener, router, p2p_task })
    }

    /// Initialize P2P network
    #[cfg(feature = "p2p")]
    async fn init_p2p(
        settings: &Settings,
        unified_state: SharedUnifiedState,
    ) -> Result<(P2PHandle, tokio::task::JoinHandle<()>)> {
        tracing::info!(
            node_name = %settings.p2p.node_name,
            port = settings.p2p.port,
            "🌐 Initializing P2P network..."
        );

        // PeerIdentity generieren (inklusive Ed25519 Keypair)
        let identity = PeerIdentity::generate();
        let peer_id = identity.peer_id;
        tracing::info!(peer_id = %peer_id, "🆔 Peer ID generated");

        // P2P-Konfiguration aus Settings erstellen
        let mut p2p_config = P2PConfig::default();
        
        // Listen-Adressen setzen
        if !settings.p2p.listen_addresses.is_empty() {
            p2p_config.listen_addresses = settings.p2p.listen_addresses.clone();
        }
        // Port setzen (überschreibt Default-Adressen)
        if settings.p2p.port != 0 {
            p2p_config.listen_addresses = vec![
                format!("/ip4/0.0.0.0/tcp/{}", settings.p2p.port),
                format!("/ip6/::/tcp/{}", settings.p2p.port),
            ];
        }

        // Bootstrap-Peers
        p2p_config.bootstrap_peers = settings.p2p.bootstrap_peers.clone();
        
        // Features
        p2p_config.enable_mdns = settings.p2p.enable_mdns;
        p2p_config.nat.enable_autonat = settings.p2p.enable_autonat;
        p2p_config.nat.enable_upnp = settings.p2p.enable_upnp;
        p2p_config.nat.enable_relay_server = settings.p2p.enable_relay_server;
        p2p_config.trust_gate.min_incoming_trust_r = settings.p2p.min_incoming_trust;

        // Privacy-Layer
        if settings.features.privacy_enabled {
            p2p_config.privacy.enabled = true;
            tracing::info!("🔐 Privacy-Layer enabled (Onion-Routing)");
        }

        tracing::info!(
            listen = ?p2p_config.listen_addresses,
            mdns = p2p_config.enable_mdns,
            bootstrap_peers = p2p_config.bootstrap_peers.len(),
            "⚙️  P2P configuration"
        );

        // SwarmManager erstellen
        let (manager, _sync_rx) = SwarmManager::new(p2p_config, identity);

        // Event-Receiver für StateEvent-Integration
        let event_rx = manager.event_receiver();

        // Command-Sender für Handle
        let command_tx = manager.command_sender();

        // Peer-ID als String für Handle
        let peer_id_string = peer_id.to_string();

        // P2P-Handle erstellen
        let p2p_handle = P2PHandle {
            command_tx,
            peer_id: peer_id_string.clone(),
            node_name: settings.p2p.node_name.clone(),
        };

        // P2P-Task spawnen
        let node_name = settings.p2p.node_name.clone();
        let p2p_task = tokio::spawn(async move {
            tracing::info!(node = %node_name, "🌐 P2P Swarm starting...");

            // Event-Handler Task
            let unified_state_events = unified_state.clone();
            let event_task = tokio::spawn(async move {
                Self::handle_p2p_events(event_rx, unified_state_events).await;
            });

            // Swarm-Manager Task
            if let Err(e) = manager.run().await {
                tracing::error!(error = %e, "P2P Swarm error");
            }

            event_task.abort();
            tracing::info!(node = %node_name, "🌐 P2P Swarm stopped");
        });

        tracing::info!(
            peer_id = %peer_id_string,
            "✅ P2P network initialized"
        );

        Ok((p2p_handle, p2p_task))
    }

    /// Handle P2P events and update UnifiedState
    #[cfg(feature = "p2p")]
    async fn handle_p2p_events(
        mut event_rx: tokio::sync::broadcast::Receiver<SwarmEvent2>,
        unified_state: SharedUnifiedState,
    ) {
        loop {
            match event_rx.recv().await {
                Ok(event) => {
                    match event {
                        SwarmEvent2::PeerConnected { peer_id } => {
                            unified_state.p2p.swarm.peer_connected(true);
                            tracing::debug!(peer_id = %peer_id, "Peer connected");
                        }
                        SwarmEvent2::PeerDisconnected { peer_id } => {
                            unified_state.p2p.swarm.peer_disconnected();
                            tracing::debug!(peer_id = %peer_id, "Peer disconnected");
                        }
                        SwarmEvent2::GossipMessage { topic, .. } => {
                            unified_state.p2p.gossip.message_received();
                            tracing::trace!(topic = %topic, "Gossip message received");
                        }
                        SwarmEvent2::MdnsDiscovered { peer_id, addresses } => {
                            tracing::debug!(
                                peer_id = %peer_id,
                                addresses = ?addresses,
                                "mDNS discovered peer"
                            );
                        }
                        SwarmEvent2::BootstrapComplete => {
                            if let Ok(mut b) = unified_state.p2p.kademlia.bootstrap_complete.write() {
                                *b = true;
                            }
                            tracing::info!("🎉 Kademlia bootstrap complete");
                        }
                        _ => {}
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    tracing::debug!("P2P event channel closed");
                    break;
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(lagged = n, "P2P event receiver lagged");
                }
            }
        }
    }

    /// Get the bound port
    pub fn port(&self) -> u16 {
        self.listener.local_addr().map(|a| a.port()).unwrap_or(0)
    }

    /// Run until shutdown signal
    pub async fn run(mut self) -> Result<(), std::io::Error> {
        // HTTP-Server Task
        let http_server = axum::serve(
            self.listener,
            self.router
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal());

        #[cfg(feature = "p2p")]
        {
            if let Some(p2p_task) = self.p2p_task.take() {
                // Beide Tasks parallel laufen lassen
                // Wir müssen p2p_task in einem Arc<Mutex<Option<...>>> wrappen oder anders lösen
                // Einfachste Lösung: HTTP-Server hat Priorität, P2P wird bei Shutdown abgebrochen
                let p2p_abort = p2p_task.abort_handle();
                
                tokio::select! {
                    result = http_server => {
                        tracing::info!("HTTP server stopped");
                        p2p_abort.abort();
                        result
                    }
                    result = p2p_task => {
                        match result {
                            Ok(()) => tracing::info!("P2P task stopped"),
                            Err(e) => tracing::error!(error = %e, "P2P task panicked"),
                        }
                        Ok(())
                    }
                }
            } else {
                http_server.await
            }
        }

        #[cfg(not(feature = "p2p"))]
        {
            http_server.await
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("Ctrl+C received"),
        _ = terminate => tracing::info!("SIGTERM received"),
    }

    tracing::info!("🛑 Shutting down gracefully...");
}
