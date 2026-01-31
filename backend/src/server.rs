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

// Legacy imports (optional)
#[cfg(feature = "legacy-oidc")]
use crate::auth::JwtValidator;
#[cfg(feature = "legacy-cache")]
use crate::cache::CachePool;
#[cfg(feature = "legacy-sql")]
use crate::db::DatabasePool;
#[cfg(feature = "legacy-s3")]
use crate::storage::StorageClient;

/// Shared application state for all handlers
#[derive(Clone)]
pub struct AppState {
    /// Dezentraler Storage (Standard)
    pub storage: DecentralizedStorage,
    /// Anwendungskonfiguration
    pub config: Arc<Settings>,
    /// Startzeitpunkt für Uptime
    pub started_at: Option<Instant>,

    // Legacy fields (optional)
    #[cfg(feature = "legacy-sql")]
    pub db: DatabasePool,
    #[cfg(feature = "legacy-cache")]
    pub cache: CachePool,
    #[cfg(feature = "legacy-s3")]
    pub s3_storage: Option<StorageClient>,
    #[cfg(feature = "legacy-oidc")]
    pub jwt_validator: Option<Arc<JwtValidator>>,
}

impl AppState {
    /// Check if all backends are reachable
    pub async fn health_check(&self) -> (bool, bool, bool) {
        let storage_ok = self.storage.ping().await.is_ok();

        #[cfg(feature = "legacy-sql")]
        let db_ok = self.db.ping().await.is_ok();
        #[cfg(not(feature = "legacy-sql"))]
        let db_ok = true;

        #[cfg(feature = "legacy-cache")]
        let cache_ok = self.cache.ping().await.is_ok();
        #[cfg(not(feature = "legacy-cache"))]
        let cache_ok = true;

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

        // Dezentraler Storage (Standard - immer verfügbar)
        let data_dir = settings.application.data_dir.clone().unwrap_or_else(|| "./data".to_string());
        let storage = DecentralizedStorage::open(&data_dir)?;
        tracing::info!(path = %data_dir, "✅ Decentralized storage ready");

        // Legacy: Database (optional)
        #[cfg(feature = "legacy-sql")]
        let db = {
            let db = DatabasePool::connect(&settings.database).await?;
            tracing::info!(host = %settings.database.host, "✅ Legacy database connected");
            db
        };

        // Legacy: Cache (optional)
        #[cfg(feature = "legacy-cache")]
        let cache = {
            let cache = CachePool::connect(&settings.cache).await?;
            tracing::info!(url = %settings.cache.url, "✅ Legacy cache connected");
            cache
        };

        // Legacy: JWT Validator (optional)
        #[cfg(feature = "legacy-oidc")]
        let jwt_validator = match JwtValidator::new(&settings.auth).await {
            Ok(jwt) => {
                tracing::info!(issuer = %settings.auth.issuer, "✅ JWT validator ready");
                Some(Arc::new(jwt))
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "⚠️  JWT validator disabled - Auth service not available"
                );
                None
            }
        };

        // Legacy: S3 Storage (optional)
        #[cfg(feature = "legacy-s3")]
        let s3_storage = match StorageClient::connect(&settings.storage).await {
            Ok(s) => {
                tracing::info!(
                    endpoint = %settings.storage.endpoint,
                    "✅ Legacy S3 storage connected"
                );
                Some(s)
            }
            Err(e) => {
                tracing::warn!(error = %e, "⚠️  Legacy S3 storage disabled");
                None
            }
        };

        let state = AppState {
            storage,
            config: Arc::new(settings.clone()),
            started_at: Some(Instant::now()),
            #[cfg(feature = "legacy-sql")]
            db,
            #[cfg(feature = "legacy-cache")]
            cache,
            #[cfg(feature = "legacy-s3")]
            s3_storage,
            #[cfg(feature = "legacy-oidc")]
            jwt_validator,
        };

        // Legacy: Run migrations in non-production
        #[cfg(feature = "legacy-sql")]
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
