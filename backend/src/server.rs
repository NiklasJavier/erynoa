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
//! │  └── config: Settings                                              │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

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

/// Shared application state for all handlers
///
/// Enthält hierarchisches State-Management:
/// - `unified_state`: Atomar Counter für alle Module
/// - `coordinator`: Invarianten-Checks, Health-Aggregation, enthält StateIntegrator
/// - `storage`: Fjall-basierter dezentraler Storage
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
        }
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
            "🏗️  Building server..."
        );

        // Dezentraler Storage (Fjall)
        let data_dir = &settings.storage.data_dir;
        let storage = DecentralizedStorage::open(data_dir)?;
        tracing::info!(path = %data_dir, "✅ Decentralized storage ready");

        // AppState mit Unified State Management
        let state = AppState::new(storage, settings.clone());
        tracing::info!("✅ Unified state management initialized");

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

        Ok(Self { listener, router })
    }

    /// Get the bound port
    pub fn port(&self) -> u16 {
        self.listener.local_addr().map(|a| a.port()).unwrap_or(0)
    }

    /// Run until shutdown signal
    pub async fn run(self) -> Result<(), std::io::Error> {
        axum::serve(
            self.listener,
            self.router
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
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
