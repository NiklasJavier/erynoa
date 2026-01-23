//! Configuration Module
//!
//! Lädt Konfiguration aus Environment und Config-Dateien

use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};

/// Hauptkonfiguration der Anwendung
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub cache: CacheSettings,
    pub auth: AuthSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
    pub environment: Environment,
    /// Basis-URL für Frontend (CORS)
    pub frontend_url: String,
    /// Öffentliche API-URL
    pub api_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Secret<String>,
    pub database: String,
    /// Max Connections im Pool
    pub max_connections: u32,
    /// Min Connections im Pool (für Cold-Start Performance)
    pub min_connections: u32,
    /// Connection Timeout in Sekunden
    pub connect_timeout: u64,
    /// Idle Timeout in Sekunden
    pub idle_timeout: u64,
}

impl DatabaseSettings {
    /// Erstellt die vollständige Connection URL
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database
        ))
    }

    /// Connection URL ohne Datenbank (für DB-Erstellung)
    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheSettings {
    pub url: String,
    /// Max Connections im Pool
    pub pool_size: u32,
    /// Default TTL in Sekunden
    pub default_ttl: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthSettings {
    /// ZITADEL Issuer URL
    pub issuer: String,
    /// Client ID für das Backend (für Service-to-Service)
    pub client_id: String,
    /// JWKS Cache Duration in Sekunden
    pub jwks_cache_duration: u64,
    /// Erlaubte Audiences
    pub audiences: Vec<String>,
}

/// Umgebungs-Typen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Local,
    Development,
    Staging,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Development => "development",
            Environment::Staging => "staging",
            Environment::Production => "production",
        }
    }

    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "development" | "dev" => Ok(Self::Development),
            "staging" | "stage" => Ok(Self::Staging),
            "production" | "prod" => Ok(Self::Production),
            other => Err(format!(
                "'{}' is not a supported environment. Use 'local', 'development', 'staging' or 'production'",
                other
            )),
        }
    }
}

impl Settings {
    /// Lädt die Konfiguration aus Environment und Config-Dateien
    pub fn load() -> Result<Self, config::ConfigError> {
        let base_path = std::env::current_dir().expect("Failed to get current directory");
        let config_dir = base_path.join("config");

        // Ermittle die aktuelle Umgebung
        let environment: Environment = std::env::var("APP_ENVIRONMENT")
            .unwrap_or_else(|_| "local".into())
            .try_into()
            .expect("Failed to parse APP_ENVIRONMENT");

        let environment_filename = format!("{}.toml", environment.as_str());

        let settings = config::Config::builder()
            // Basis-Konfiguration
            .add_source(config::File::from(config_dir.join("base.toml")))
            // Umgebungs-spezifische Konfiguration
            .add_source(config::File::from(config_dir.join(environment_filename)).required(false))
            // Environment Variables (überschreiben alles)
            // Format: APP_APPLICATION__PORT=8080 -> application.port = 8080
            .add_source(
                config::Environment::with_prefix("APP")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;

        settings.try_deserialize::<Settings>()
    }
}
