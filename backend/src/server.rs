//! Server module - Application startup and state management

use crate::api::create_router;
use crate::config::Settings;
use crate::local::DecentralizedStorage;
use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;

/// Shared application state for all handlers
#[derive(Clone)]
pub struct AppState {
    /// Dezentraler Storage (Fjall)
    pub storage: DecentralizedStorage,
    /// Anwendungskonfiguration
    pub config: Arc<Settings>,
    /// Startzeitpunkt für Uptime
    pub started_at: Option<Instant>,
}

impl AppState {
    /// Check if storage is reachable
    pub async fn health_check(&self) -> bool {
        self.storage.ping().await.is_ok()
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
        tracing::info!(
            env = %settings.application.environment.as_str(),
            "🏗️  Building server..."
        );

        // Dezentraler Storage (Fjall)
        let data_dir = &settings.storage.data_dir;
        let storage = DecentralizedStorage::open(data_dir)?;
        tracing::info!(path = %data_dir, "✅ Decentralized storage ready");

        let state = AppState {
            storage,
            config: Arc::new(settings.clone()),
            started_at: Some(Instant::now()),
        };

        let router = create_router(state);
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
