//! # Erynoa Testnet Node Binary
//!
//! Standalone P2P-Node für Multi-Node-Testnet.
//!
//! ## Verwendung
//!
//! ```bash
//! cargo run --features p2p --bin erynoa-testnet-node -- \
//!     --node-name relay1 \
//!     --p2p-port 4001 \
//!     --api-port 9000 \
//!     --mode relay
//! ```
//!
//! ## Environment-Variablen
//!
//! - `NODE_NAME`: Name des Nodes (Default: "node")
//! - `NODE_MODE`: Modus (relay|client, Default: "relay")
//! - `P2P_PORT`: libp2p Swarm Port (Default: 4001)
//! - `API_PORT`: HTTP API Port (Default: 9000)
//! - `BOOTSTRAP_PEERS`: Komma-separierte Multiaddrs
//! - `P2P_ENABLE_MDNS`: mDNS aktivieren (Default: true)
//! - `GENESIS_NODE`: Ob dieser Node der Genesis ist (Default: false)

use std::env;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;
use tracing::{error, info, warn, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// CLI-Argumente für Testnet-Node
struct Args {
    node_name: String,
    p2p_port: u16,
    api_port: u16,
    mode: String,
    bootstrap_peers: Vec<String>,
    enable_mdns: bool,
    genesis_node: bool,
    data_dir: String,
}

impl Args {
    fn parse() -> Self {
        let args: Vec<String> = env::args().collect();

        Self {
            node_name: Self::get_arg(&args, "--node-name")
                .or_else(|| env::var("NODE_NAME").ok())
                .unwrap_or_else(|| "node".to_string()),

            p2p_port: Self::get_arg(&args, "--p2p-port")
                .or_else(|| env::var("P2P_PORT").ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(4001),

            api_port: Self::get_arg(&args, "--api-port")
                .or_else(|| env::var("API_PORT").ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(9000),

            mode: Self::get_arg(&args, "--mode")
                .or_else(|| env::var("NODE_MODE").ok())
                .unwrap_or_else(|| "relay".to_string()),

            bootstrap_peers: Self::get_arg(&args, "--bootstrap-peers")
                .or_else(|| env::var("BOOTSTRAP_PEERS").ok())
                .map(|s| {
                    s.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_default(),

            enable_mdns: Self::get_arg(&args, "--enable-mdns")
                .or_else(|| env::var("P2P_ENABLE_MDNS").ok())
                .map(|s| s.to_lowercase() == "true" || s == "1")
                .unwrap_or(true),

            genesis_node: Self::get_arg(&args, "--genesis-node")
                .or_else(|| env::var("GENESIS_NODE").ok())
                .map(|s| s.to_lowercase() == "true" || s == "1")
                .unwrap_or(false),

            data_dir: Self::get_arg(&args, "--data-dir")
                .or_else(|| env::var("APP_STORAGE__DATA_DIR").ok())
                .unwrap_or_else(|| "./data".to_string()),
        }
    }

    fn get_arg(args: &[String], name: &str) -> Option<String> {
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            if arg == name {
                return iter.next().cloned();
            }
            if let Some(value) = arg.strip_prefix(&format!("{}=", name)) {
                return Some(value.to_string());
            }
        }
        None
    }
}

/// Peer-Counter für Status
static PEER_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Connected Peers Liste für API
type ConnectedPeers = Arc<RwLock<Vec<String>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logging initialisieren
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true))
        .with(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .init();

    let args = Args::parse();

    info!(
        node = %args.node_name,
        mode = %args.mode,
        p2p_port = args.p2p_port,
        api_port = args.api_port,
        genesis = args.genesis_node,
        "🌐 Starting Erynoa Testnet Node"
    );

    if !args.bootstrap_peers.is_empty() {
        info!(peers = ?args.bootstrap_peers, "📡 Bootstrap peers configured");
    } else if !args.genesis_node {
        warn!("⚠️  No bootstrap peers configured and not genesis node - relying on mDNS only");
    }

    // Storage-Verzeichnis erstellen
    std::fs::create_dir_all(&args.data_dir)?;
    info!(path = %args.data_dir, "💾 Storage directory ready");

    // Connected Peers Liste für API
    let connected_peers: ConnectedPeers = Arc::new(RwLock::new(Vec::new()));

    // P2P-Stack initialisieren
    #[cfg(feature = "p2p")]
    {
        use erynoa_api::peer::p2p::{NatStatus, P2PConfig, SwarmState, TestnetEvent, TestnetSwarm};
        use libp2p::identity::Keypair;

        // Keypair generieren
        let keypair = Keypair::generate_ed25519();
        let peer_id = libp2p::PeerId::from(keypair.public());

        info!(peer_id = %peer_id, "🆔 Peer ID");

        // P2P-Konfiguration erstellen
        let mut config = P2PConfig::default();

        // Listen-Adressen setzen
        config.listen_addresses = vec![
            format!("/ip4/0.0.0.0/tcp/{}", args.p2p_port),
            format!("/ip6/::/tcp/{}", args.p2p_port),
        ];

        // Bootstrap-Peers setzen
        config.bootstrap_peers = args.bootstrap_peers.clone();

        // mDNS konfigurieren
        config.enable_mdns = args.enable_mdns;

        info!(
            listen = ?config.listen_addresses,
            mdns = config.enable_mdns,
            bootstrap = ?config.bootstrap_peers,
            "⚙️  P2P configuration"
        );

        // TestnetSwarm erstellen
        let (mut swarm, event_rx) = TestnetSwarm::new(keypair, &config)?;

        info!(peer_id = %swarm.peer_id(), "✅ Testnet swarm created with full NAT-Traversal stack");

        // SwarmState für echte Diagnostics erstellen
        let swarm_state = Arc::new(SwarmState::new(peer_id.to_string()));

        // UnifiedState und Bridge für zentrales State-Management erstellen
        let unified_state = erynoa_api::core::create_unified_state();
        let integrator = erynoa_api::core::StateIntegrator::new(unified_state.clone());
        let bridge = erynoa_api::peer::p2p::UnifiedStateBridge::new(swarm_state.clone(), integrator);
        let bridge = Arc::new(bridge);

        // Event-Handler Task - befüllt SwarmState mit echten Daten
        let connected_peers_clone = connected_peers.clone();
        let swarm_state_clone = swarm_state.clone();
        let bridge_clone = bridge.clone();
        let event_task = tokio::spawn(async move {
            let mut event_rx = event_rx;
            while let Ok(event) = event_rx.recv().await {
                match event {
                    TestnetEvent::PeerConnected {
                        peer_id,
                        is_inbound,
                    } => {
                        let peer_str = peer_id.to_string();
                        let mut peers = connected_peers_clone.write().await;
                        if !peers.contains(&peer_str) {
                            peers.push(peer_str.clone());
                            let count = PEER_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
                            info!(peer_id = %peer_id, total_peers = count, inbound = is_inbound, "✅ Peer connected");
                            // SwarmState aktualisieren
                            swarm_state_clone.peer_connected(&peer_str, is_inbound, false);
                            // Bridge synchronisieren
                            bridge_clone.sync();
                        }
                    }
                    TestnetEvent::PeerDisconnected { peer_id } => {
                        let peer_str = peer_id.to_string();
                        let mut peers = connected_peers_clone.write().await;
                        if peers.contains(&peer_str) {
                            peers.retain(|p| p != &peer_str);
                            let count = PEER_COUNT.fetch_sub(1, Ordering::SeqCst).saturating_sub(1);
                            info!(peer_id = %peer_id, total_peers = count, "👋 Peer disconnected");
                            // SwarmState aktualisieren
                            swarm_state_clone.peer_disconnected(&peer_str);
                            // Bridge synchronisieren
                            bridge_clone.sync();
                        }
                    }
                    TestnetEvent::MdnsDiscovered { peer_id, addresses } => {
                        info!(peer_id = %peer_id, addresses = ?addresses, "🔍 mDNS discovered peer");
                        swarm_state_clone.mdns_peer_discovered();
                    }
                    TestnetEvent::MdnsExpired { peer_id } => {
                        info!(peer_id = %peer_id, "🔍 mDNS peer expired");
                    }
                    TestnetEvent::KademliaBootstrapComplete => {
                        info!("🎉 Kademlia bootstrap complete!");
                        swarm_state_clone.kademlia_bootstrap_done();
                        bridge_clone.sync();
                    }
                    TestnetEvent::KademliaRoutingUpdate {
                        peer_id,
                        bucket_size,
                    } => {
                        // Routing Table Size aktualisieren (approximativ)
                        let current = swarm_state_clone
                            .kademlia_routing_table_size
                            .load(Ordering::Relaxed);
                        if bucket_size > current {
                            swarm_state_clone.set_kademlia_routing_table_size(bucket_size);
                            bridge_clone.sync();
                        }
                        let _ = peer_id; // Unused but part of event
                    }
                    TestnetEvent::GossipMessage { topic, source, .. } => {
                        info!(topic = %topic, source = ?source, "📨 Gossip message received");
                        swarm_state_clone.gossip_message_received();
                        // Sync periodisch, nicht bei jeder Message
                    }
                    TestnetEvent::GossipMeshPeerAdded { peer_id, topic } => {
                        info!(peer_id = %peer_id, topic = %topic, "📣 Peer joined gossip mesh");
                        swarm_state_clone.gossip_mesh_peer_added();
                        bridge_clone.sync();
                    }
                    TestnetEvent::GossipMeshPeerRemoved { peer_id, topic } => {
                        info!(peer_id = %peer_id, topic = %topic, "📣 Peer left gossip mesh");
                        swarm_state_clone.gossip_mesh_peer_removed();
                        bridge_clone.sync();
                    }
                    TestnetEvent::GossipMessageSent { topic } => {
                        swarm_state_clone.gossip_message_sent();
                        let _ = topic;
                    }
                    // NAT-Traversal Events
                    TestnetEvent::AutoNatStatus { nat_status } => {
                        info!(status = %nat_status, "🌐 AutoNAT status changed");
                        // Parse NAT status
                        let status = if nat_status.contains("Public") {
                            NatStatus::Public
                        } else if nat_status.contains("Private") {
                            NatStatus::Private
                        } else {
                            NatStatus::Unknown
                        };
                        swarm_state_clone.set_nat_status(status);
                        bridge_clone.sync();
                    }
                    TestnetEvent::ExternalAddressConfirmed { address } => {
                        info!(addr = %address, "🌐 External address confirmed");
                        swarm_state_clone.add_external_address(address.to_string());
                    }
                    TestnetEvent::RelayReservation { relay_peer } => {
                        info!(relay = %relay_peer, "🔄 Relay reservation accepted (client)");
                        swarm_state_clone.relay_reservation_accepted(relay_peer.to_string());
                        bridge_clone.sync();
                    }
                    TestnetEvent::RelayCircuitOpened {
                        src_peer_id,
                        dst_peer_id,
                    } => {
                        info!(src = %src_peer_id, dst = %dst_peer_id, "📡 Relay: Now serving circuit!");
                        swarm_state_clone.relay_circuit_opened();
                        bridge_clone.sync();
                    }
                    TestnetEvent::RelayCircuitClosed {
                        src_peer_id,
                        dst_peer_id,
                    } => {
                        info!(src = %src_peer_id, dst = %dst_peer_id, "📡 Relay: Circuit closed");
                        swarm_state_clone.relay_circuit_closed();
                    }
                    TestnetEvent::DirectConnectionEstablished { peer_id } => {
                        info!(peer_id = %peer_id, "✅ DCUTR: Direct connection established via holepunching");
                        swarm_state_clone.dcutr_success();
                    }
                    TestnetEvent::DirectConnectionFailed { peer_id } => {
                        info!(peer_id = %peer_id, "❌ DCUTR: Holepunching failed");
                        swarm_state_clone.dcutr_failure();
                    }
                    TestnetEvent::UpnpMapped { protocol, addr } => {
                        info!(protocol = %protocol, addr = %addr, "🌐 UPnP: Port mapped");
                        swarm_state_clone.upnp_mapped(addr.to_string());
                    }
                    TestnetEvent::UpnpUnavailable => {
                        swarm_state_clone.upnp_unavailable();
                    }
                    TestnetEvent::PingResult { peer_id, rtt_ms } => {
                        // Ping-RTT aufzeichnen
                        swarm_state_clone.record_ping(
                            &peer_id.to_string(),
                            std::time::Duration::from_millis(rtt_ms),
                        );
                    }
                    TestnetEvent::ConnectionError { peer_id: _ } => {
                        swarm_state_clone.connection_error();
                    }
                }
            }
        });

        // HTTP API Task
        let api_addr: SocketAddr = format!("0.0.0.0:{}", args.api_port).parse()?;
        let node_name = args.node_name.clone();
        let mode = args.mode.clone();
        let is_genesis = args.genesis_node;
        let peer_id_string = peer_id.to_string();
        let connected_peers_api = connected_peers.clone();
        let swarm_state_api = swarm_state.clone();

        let api_task = tokio::spawn(async move {
            if let Err(e) = start_api_server(
                api_addr,
                node_name,
                mode,
                is_genesis,
                peer_id_string,
                connected_peers_api,
                swarm_state_api,
            )
            .await
            {
                error!(error = %e, "API server error");
            }
        });

        info!(addr = %api_addr, "🌐 HTTP API server started");

        // Swarm starten
        let config_clone = config.clone();
        let swarm_task = tokio::spawn(async move {
            if let Err(e) = swarm.run(&config_clone).await {
                error!(error = %e, "Swarm error");
            }
        });

        info!("✅ P2P Swarm started");

        // Auf Shutdown warten
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Received Ctrl+C, shutting down...");
            }
            result = swarm_task => {
                if let Err(e) = result {
                    error!(error = %e, "Swarm task error");
                }
            }
            _ = api_task => {
                error!("API task ended unexpectedly");
            }
            _ = event_task => {
                error!("Event task ended unexpectedly");
            }
        }

        info!("👋 Node shut down gracefully");
    }

    #[cfg(not(feature = "p2p"))]
    {
        error!("P2P feature not enabled! Compile with --features p2p");
        std::process::exit(1);
    }

    Ok(())
}

/// Startet einen minimalen HTTP-Server für Health-Checks und Status
async fn start_api_server(
    addr: SocketAddr,
    node_name: String,
    mode: String,
    is_genesis: bool,
    peer_id: String,
    connected_peers: ConnectedPeers,
    swarm_state: Arc<erynoa_api::peer::p2p::SwarmState>,
) -> anyhow::Result<()> {
    use axum::{
        extract::State,
        response::{
            sse::{Event, KeepAlive, Sse},
            Html, IntoResponse,
        },
        routing::get,
        Json, Router,
    };
    use erynoa_api::peer::p2p::{
        create_diagnostic_state, create_system_state, generate_dashboard_html,
        generate_system_layers, DiagnosticEvent, DiagnosticRunner, HealthStatus, NetworkMetrics,
        SwarmSnapshot, SystemSnapshot,
    };
    use futures::stream::Stream;
    use std::convert::Infallible;
    use std::time::{Duration, Instant};
    use tokio_stream::wrappers::IntervalStream;
    use tokio_stream::StreamExt;

    let start_time = Instant::now();

    // Diagnostic State erstellen (Legacy - wird noch für Events gebraucht)
    let diagnostic_state = create_diagnostic_state(peer_id.clone());

    // System State für Core/ECLVM/Local/Protection Metriken
    let system_state = create_system_state();

    // Snapshot-Typ für SSE - jetzt mit SwarmSnapshot und SystemSnapshot
    #[derive(serde::Serialize)]
    struct StreamSnapshot {
        timestamp: String,
        metrics: NetworkMetrics,
        peer_count: usize,
        recent_events: Vec<DiagnosticEvent>,
        health: HealthStatus,
        /// Echte Swarm-Daten (optional für backwards compatibility)
        #[serde(skip_serializing_if = "Option::is_none")]
        swarm: Option<SwarmSnapshot>,
        /// System-Module Daten (Core, ECLVM, Local, Protection)
        #[serde(skip_serializing_if = "Option::is_none")]
        system: Option<SystemSnapshot>,
    }

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route(
            "/status",
            get({
                let node_name = node_name.clone();
                let mode = mode.clone();
                let peer_id = peer_id.clone();
                let connected_peers = connected_peers.clone();
                let swarm_state = swarm_state.clone();
                move || {
                    let node_name = node_name.clone();
                    let mode = mode.clone();
                    let peer_id = peer_id.clone();
                    let connected_peers = connected_peers.clone();
                    let swarm_state = swarm_state.clone();
                    async move {
                        let peers = connected_peers.read().await.clone();
                        let snapshot = swarm_state.snapshot();
                        let status = serde_json::json!({
                            "node_name": node_name,
                            "mode": mode,
                            "peer_id": peer_id,
                            "is_genesis": is_genesis,
                            "peer_count": PEER_COUNT.load(Ordering::SeqCst),
                            "connected_peers": peers,
                            "uptime_secs": start_time.elapsed().as_secs(),
                            "version": env!("CARGO_PKG_VERSION"),
                            "nat_status": format!("{:?}", snapshot.nat_status),
                            "kademlia_peers": snapshot.kademlia_routing_table_size,
                            "gossip_mesh_size": snapshot.gossip_mesh_size,
                            "dcutr_success_rate": snapshot.dcutr_success_rate,
                            "avg_ping_ms": snapshot.avg_ping_ms,
                        });
                        Json(status)
                    }
                }
            }),
        )
        .route(
            "/peers",
            get({
                let swarm_state = swarm_state.clone();
                move || {
                    let swarm_state = swarm_state.clone();
                    async move {
                        let peers = swarm_state.get_peers();
                        Json(serde_json::json!({
                            "count": peers.len(),
                            "peers": peers
                        }))
                    }
                }
            }),
        )
        // =========================================
        // P2P DIAGNOSTICS ENDPOINTS (mit echten SwarmState Daten!)
        // =========================================
        // JSON Diagnostics - jetzt mit echten Swarm-Daten
        .route(
            "/diagnostics",
            get({
                let peer_id = peer_id.clone();
                let swarm_state = swarm_state.clone();
                move || {
                    let peer_id = peer_id.clone();
                    let swarm_state = swarm_state.clone();
                    async move {
                        // Verwende SwarmState für echte Laufzeit-Daten
                        let runner = DiagnosticRunner::from_swarm_state(&swarm_state);
                        let diagnostics = runner.run_all(Some(peer_id)).await;

                        Json(diagnostics)
                    }
                }
            }),
        )
        // CLI-friendly ASCII report
        .route(
            "/diagnostics/report",
            get({
                let peer_id = peer_id.clone();
                let swarm_state = swarm_state.clone();
                move || {
                    let peer_id = peer_id.clone();
                    let swarm_state = swarm_state.clone();
                    async move {
                        let runner = DiagnosticRunner::from_swarm_state(&swarm_state);
                        let diagnostics = runner.run_all(Some(peer_id)).await;

                        diagnostics.to_cli_report()
                    }
                }
            }),
        )
        // Detailed metrics - jetzt SwarmSnapshot
        .route(
            "/diagnostics/metrics",
            get({
                let swarm_state = swarm_state.clone();
                move || {
                    let swarm_state = swarm_state.clone();
                    async move { Json(swarm_state.snapshot()) }
                }
            }),
        )
        // Event log (noch vom diagnostic_state)
        .route(
            "/diagnostics/events",
            get({
                let diagnostic_state = diagnostic_state.clone();
                move || {
                    let diagnostic_state = diagnostic_state.clone();
                    async move { Json(diagnostic_state.get_recent_events(100)) }
                }
            }),
        )
        // Layer details - mit SwarmState
        .route(
            "/diagnostics/layers",
            get({
                let peer_id = peer_id.clone();
                let swarm_state = swarm_state.clone();
                move || {
                    let peer_id = peer_id.clone();
                    let swarm_state = swarm_state.clone();
                    async move {
                        let runner = DiagnosticRunner::from_swarm_state(&swarm_state);
                        let diagnostics = runner.run_all(Some(peer_id)).await;
                        Json(diagnostics.layers)
                    }
                }
            }),
        )
        // Server-Sent Events Stream (Real-Time!) - jetzt mit SwarmSnapshot UND SystemSnapshot
        .route(
            "/diagnostics/stream",
            get({
                let swarm_state = swarm_state.clone();
                let system_state = system_state.clone();
                let diagnostic_state = diagnostic_state.clone();
                move || {
                    let swarm_state = swarm_state.clone();
                    let system_state = system_state.clone();
                    let diagnostic_state = diagnostic_state.clone();
                    async move {
                        let interval = tokio::time::interval(Duration::from_millis(500));
                        let stream = IntervalStream::new(interval);

                        let sse_stream = stream.map(move |_| {
                            // Echte Swarm-Daten
                            let swarm_snapshot = swarm_state.snapshot();
                            // System-Module-Daten
                            let system_snapshot = system_state.snapshot();

                            let snapshot = StreamSnapshot {
                                timestamp: chrono::Utc::now().to_rfc3339(),
                                metrics: diagnostic_state.get_metrics(),
                                peer_count: swarm_snapshot.connected_peers_count,
                                recent_events: diagnostic_state.get_recent_events(5),
                                health: diagnostic_state.get_health_status(),
                                swarm: Some(swarm_snapshot),
                                system: Some(system_snapshot),
                            };

                            let json = serde_json::to_string(&snapshot).unwrap_or_default();
                            Ok::<_, Infallible>(Event::default().data(json))
                        });

                        Sse::new(sse_stream).keep_alive(KeepAlive::default())
                    }
                }
            }),
        )
        // HTML Dashboard
        .route(
            "/diagnostics/dashboard",
            get({
                let peer_id = peer_id.clone();
                move || {
                    let peer_id = peer_id.clone();
                    async move { Html(generate_dashboard_html(&peer_id)) }
                }
            }),
        );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
