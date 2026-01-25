//! Server module - Application startup and state management

use crate::api::create_router;
use crate::auth::JwtValidator;
use crate::cache::CachePool;
use crate::config::Settings;
use crate::db::DatabasePool;
use crate::storage::StorageClient;
use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;

/// Shared application state for all handlers
#[derive(Clone)]
pub struct AppState {
    pub db: DatabasePool,
    pub cache: CachePool,
    pub storage: Option<StorageClient>,
    pub jwt_validator: Option<Arc<JwtValidator>>,
    pub config: Arc<Settings>,
    pub started_at: Option<Instant>,
}

impl AppState {
    /// Check if all backends are reachable
    pub async fn health_check(&self) -> (bool, bool, bool) {
        let db_ok = self.db.ping().await.is_ok();
        let cache_ok = self.cache.ping().await.is_ok();
        let storage_ok = match &self.storage {
            Some(s) => s.ping().await.is_ok(),
            None => true, // Wenn nicht konfiguriert, gilt als OK
        };
        (db_ok, cache_ok, storage_ok)
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

        // Database
        let db = DatabasePool::connect(&settings.database).await?;
        tracing::info!(host = %settings.database.host, "✅ Database connected");

        // Cache
        let cache = CachePool::connect(&settings.cache).await?;
        tracing::info!(url = %settings.cache.url, "✅ Cache connected");

        // JWT Validator (optional - kann fehlschlagen wenn ZITADEL nicht läuft)
        let jwt_validator = match JwtValidator::new(&settings.auth).await {
            Ok(jwt) => {
                tracing::info!(issuer = %settings.auth.issuer, "✅ JWT validator ready");
                Some(Arc::new(jwt))
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    issuer = %settings.auth.issuer,
                    "⚠️  JWT validator disabled - Auth service not available"
                );
                None
            }
        };

        // Storage (optional - kann fehlschlagen wenn MinIO nicht läuft)
        let storage = match StorageClient::connect(&settings.storage).await {
            Ok(s) => {
                tracing::info!(
                    endpoint = %settings.storage.endpoint,
                    bucket = %settings.storage.default_bucket,
                    "✅ Storage connected"
                );
                Some(s)
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    endpoint = %settings.storage.endpoint,
                    "⚠️  Storage disabled - MinIO not available"
                );
                None
            }
        };

        let state = AppState {
            db,
            cache,
            storage,
            jwt_validator,
            config: Arc::new(settings.clone()),
            started_at: Some(Instant::now()),
        };

        // Run migrations in non-production
        if !settings.application.environment.is_production() {
            if let Err(e) = state.db.migrate().await {
                tracing::warn!(error = %e, "Migration skipped");
            } else {
                tracing::info!("✅ Migrations applied");
            }
        }

        let router = create_router(state);
        let addr = format!("{}:{}", settings.application.host, settings.application.port);
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
            self.router.into_make_service_with_connect_info::<SocketAddr>(),
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
