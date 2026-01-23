//! JWKS (JSON Web Key Set) Management
//!
//! Lädt und cached die öffentlichen Schlüssel von ZITADEL

use anyhow::{Context, Result};
use jsonwebtoken::jwk::{JwkSet, Jwk};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// JWKS Cache mit automatischem Refresh
pub struct JwksCache {
    /// Die geladenen JWKs (Key ID -> JWK)
    keys: Arc<RwLock<HashMap<String, Jwk>>>,
    /// JWKS URL
    jwks_url: String,
    /// HTTP Client
    client: reqwest::Client,
    /// Cache Lebensdauer
    cache_duration: Duration,
    /// Letzter Refresh
    last_refresh: Arc<RwLock<Instant>>,
}

impl JwksCache {
    /// Erstellt einen neuen JWKS Cache
    pub async fn new(issuer: &str, cache_duration_secs: u64) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .context("Failed to create HTTP client")?;

        // Fetch JWKS URL from OpenID Configuration
        let openid_config_url = format!("{}/.well-known/openid-configuration", issuer.trim_end_matches('/'));
        let openid_config: serde_json::Value = client
            .get(&openid_config_url)
            .send()
            .await
            .context("Failed to fetch OpenID configuration")?
            .json()
            .await
            .context("Failed to parse OpenID configuration")?;

        let jwks_url = openid_config["jwks_uri"]
            .as_str()
            .context("jwks_uri not found in OpenID configuration")?
            .to_string();

        let cache = Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            jwks_url,
            client,
            cache_duration: Duration::from_secs(cache_duration_secs),
            last_refresh: Arc::new(RwLock::new(Instant::now() - Duration::from_secs(cache_duration_secs + 1))),
        };

        // Initial Load
        cache.refresh().await?;

        Ok(cache)
    }

    /// Holt einen Key nach ID (refresht bei Bedarf)
    pub async fn get_key(&self, kid: &str) -> Result<Option<Jwk>> {
        // Prüfe ob Refresh nötig
        {
            let last = self.last_refresh.read();
            if last.elapsed() > self.cache_duration {
                drop(last);
                self.refresh().await?;
            }
        }

        let keys = self.keys.read();
        Ok(keys.get(kid).cloned())
    }

    /// Refresht die Keys von ZITADEL
    async fn refresh(&self) -> Result<()> {
        tracing::debug!("Refreshing JWKS from {}", self.jwks_url);

        let response = self
            .client
            .get(&self.jwks_url)
            .send()
            .await
            .context("Failed to fetch JWKS")?;

        let jwks: JwkSet = response
            .json()
            .await
            .context("Failed to parse JWKS response")?;

        let mut keys = self.keys.write();
        keys.clear();

        for jwk in jwks.keys {
            if let Some(kid) = &jwk.common.key_id {
                keys.insert(kid.clone(), jwk);
            }
        }

        let mut last = self.last_refresh.write();
        *last = Instant::now();

        tracing::info!("JWKS refreshed, {} keys loaded", keys.len());

        Ok(())
    }

    /// Erzwingt einen Refresh (z.B. wenn ein Key nicht gefunden wurde)
    #[allow(dead_code)]
    pub async fn force_refresh(&self) -> Result<()> {
        self.refresh().await
    }
}
