//! Database Connection Pool
//!
//! High-Performance SQLx Pool für OrioleDB/PostgreSQL

use anyhow::{Context, Result};
use secrecy::ExposeSecret;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::{Pool, Postgres};
use std::time::Duration;

use crate::config::DatabaseSettings;

/// Database Connection Pool Wrapper
#[derive(Clone)]
pub struct DatabasePool {
    pool: Pool<Postgres>,
}

impl DatabasePool {
    /// Erstellt einen neuen Connection Pool
    pub async fn connect(settings: &DatabaseSettings) -> Result<Self> {
        let options = PgConnectOptions::new()
            .host(&settings.host)
            .port(settings.port)
            .username(&settings.username)
            .password(settings.password.expose_secret())
            .database(&settings.database)
            // SSL Mode (für Production)
            .ssl_mode(PgSslMode::Prefer)
            // Statement Cache
            .statement_cache_capacity(500);

        let pool = PgPoolOptions::new()
            // Pool Size
            .max_connections(settings.max_connections)
            .min_connections(settings.min_connections)
            // Timeouts
            .acquire_timeout(Duration::from_secs(settings.connect_timeout))
            .idle_timeout(Duration::from_secs(settings.idle_timeout))
            // Health Check
            .test_before_acquire(true)
            // Connect
            .connect_with(options)
            .await
            .context("Failed to connect to database")?;

        Ok(Self { pool })
    }

    /// Gibt eine Referenz zum Pool zurück
    pub fn inner(&self) -> &Pool<Postgres> {
        &self.pool
    }

    /// Health Check
    pub async fn ping(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .context("Database ping failed")?;
        Ok(())
    }

    /// Führt Migrationen aus
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .context("Failed to run migrations")?;
        Ok(())
    }
}

impl std::ops::Deref for DatabasePool {
    type Target = Pool<Postgres>;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}
