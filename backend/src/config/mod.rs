//! Configuration Module
//!
//! Lädt Konfiguration aus Environment und Config-Dateien
//!
//! ## Dezentrale Architektur
//!
//! Erynoa verwendet ausschließlich dezentrale Komponenten:
//! - **Storage**: Fjall (embedded KV-Store)
//! - **Identität**: DID-basierte Auth (Ed25519)
//! - **Cache**: In-Memory (kein externer Redis)

pub mod constants;
pub mod version;

pub use version::{DESCRIPTION, NAME, VERSION};

use serde::{Deserialize, Deserializer};
use std::convert::{TryFrom, TryInto};

/// Hauptkonfiguration der Anwendung
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    /// Dezentraler Storage (Fjall)
    #[serde(default)]
    pub storage: StorageSettings,
    /// Feature Flags für die Anwendung
    #[serde(default)]
    pub features: FeatureFlags,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
    pub environment: Environment,
    /// Basis-URL für Console (CORS)
    pub console_url: String,
    /// Basis-URL für Platform (CORS)
    pub platform_url: String,
    /// Basis-URL für Docs (CORS)
    pub docs_url: String,
    /// Öffentliche API-URL
    pub api_url: String,
}

/// Dezentraler Storage (Fjall embedded KV-Store)
#[derive(Debug, Clone, Deserialize)]
pub struct StorageSettings {
    /// Datenverzeichnis für Fjall (Standard: ./data)
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
    /// Max Content-Größe in Bytes (Standard: 100 MB)
    #[serde(default = "default_max_content_size")]
    pub max_content_size: u64,
}

impl Default for StorageSettings {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            max_content_size: default_max_content_size(),
        }
    }
}

fn default_data_dir() -> String {
    "./data".to_string()
}

fn default_max_content_size() -> u64 {
    104_857_600 // 100 MB
}

/// Feature Flags für die Anwendung
#[derive(Debug, Clone, Deserialize)]
pub struct FeatureFlags {
    /// Benutzer-Registrierung aktiviert
    #[serde(default = "default_true")]
    pub registration: bool,
    /// Social Login aktiviert
    #[serde(default = "default_false")]
    pub social_login: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            registration: true,
            social_login: false,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

#[allow(dead_code)]
fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct StringOrVec;

    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or a sequence of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect())
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(item) = seq.next_element::<String>()? {
                vec.push(item);
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_any(StringOrVec)
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
