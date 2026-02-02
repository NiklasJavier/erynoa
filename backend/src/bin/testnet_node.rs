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

    // P2P-Stack initialisieren
    #[cfg(feature = "p2p")]
    {
        use erynoa_api::peer::p2p::{P2PConfig, PeerIdentity, SwarmManager};

        // Peer-Identität generieren oder laden
        let identity_path = format!("{}/identity.key", args.data_dir);
        let identity = if std::path::Path::new(&identity_path).exists() {
            info!(path = %identity_path, "🔑 Loading existing identity");
            // TODO: Implementiere load/save für PeerIdentity
            // Für jetzt: immer neue generieren
            PeerIdentity::generate()
        } else {
            info!(path = %identity_path, "🔑 Generating new identity");
            PeerIdentity::generate()
        };

        info!(peer_id = %identity.peer_id, "🆔 Peer ID");

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
            "⚙️  P2P configuration"
        );

        // Swarm erstellen
        let (swarm_manager, mut sync_request_rx) = SwarmManager::new(config, identity);
        let swarm_manager = Arc::new(swarm_manager);

        // Event-Receiver für Peer-Tracking
        let mut event_rx = swarm_manager.event_receiver();

        // Peer-Tracking Task
        let _event_task = tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                match event {
                    erynoa_api::peer::p2p::SwarmEvent2::PeerConnected { peer_id } => {
                        let count = PEER_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
                        info!(peer_id = %peer_id, total_peers = count, "✅ Peer connected");
                    }
                    erynoa_api::peer::p2p::SwarmEvent2::PeerDisconnected { peer_id } => {
                        let count = PEER_COUNT.fetch_sub(1, Ordering::SeqCst) - 1;
                        info!(peer_id = %peer_id, total_peers = count, "👋 Peer disconnected");
                    }
                    erynoa_api::peer::p2p::SwarmEvent2::MdnsDiscovered { peer_id, addresses } => {
                        info!(peer_id = %peer_id, addresses = ?addresses, "🔍 mDNS discovered peer");
                    }
                    erynoa_api::peer::p2p::SwarmEvent2::BootstrapComplete => {
                        info!("🎉 Bootstrap complete!");
                    }
                    _ => {}
                }
            }
        });

        // Sync-Request Handler (für Tests)
        let _sync_task = tokio::spawn(async move {
            while let Some(req) = sync_request_rx.recv().await {
                info!(peer_id = %req.peer_id, "📨 Received sync request");
                // TODO: Handle sync requests
            }
        });

        // HTTP API Task
        let api_addr: SocketAddr = format!("0.0.0.0:{}", args.api_port).parse()?;
        let node_name = args.node_name.clone();
        let mode = args.mode.clone();
        let is_genesis = args.genesis_node;
        let api_task = tokio::spawn(async move {
            if let Err(e) = start_api_server(api_addr, node_name, mode, is_genesis).await {
                error!(error = %e, "API server error");
            }
        });

        info!(addr = %api_addr, "🌐 HTTP API server started");

        // Swarm starten
        let swarm_manager_clone = swarm_manager.clone();
        let swarm_task = tokio::spawn(async move {
            if let Err(e) = swarm_manager_clone.run().await {
                error!(error = %e, "Swarm error");
            }
        });

        info!("✅ P2P Swarm started");

        // Auf Shutdown warten
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Received Ctrl+C, shutting down...");
            }
            _ = swarm_task => {
                error!("Swarm task ended unexpectedly");
            }
            _ = api_task => {
                error!("API task ended unexpectedly");
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
) -> anyhow::Result<()> {
    use axum::{routing::get, Json, Router};
    use std::time::Instant;

    let start_time = Instant::now();
    let node_name_clone = node_name.clone();
    let mode_clone = mode.clone();

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route(
            "/status",
            get(move || async move {
                let status = serde_json::json!({
                    "node_name": node_name_clone.clone(),
                    "mode": mode_clone.clone(),
                    "is_genesis": is_genesis,
                    "peer_count": PEER_COUNT.load(Ordering::SeqCst),
                    "uptime_secs": start_time.elapsed().as_secs(),
                    "version": env!("CARGO_PKG_VERSION"),
                });
                Json(status)
            }),
        );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
