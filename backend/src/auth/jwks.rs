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
    /// Host-Header für interne Anfragen (falls ZITADEL)
    host_header: Option<String>,
}

impl JwksCache {
    /// Erstellt einen neuen JWKS Cache
    /// 
    /// * `issuer` - Die externe Issuer-URL (für Token-Validierung)
    /// * `internal_issuer` - Optionale interne URL (für Docker-Netzwerk JWKS-Fetch)
    /// * `cache_duration_secs` - Cache-Dauer in Sekunden
    pub async fn new(issuer: &str, internal_issuer: Option<&str>, cache_duration_secs: u64) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .context("Failed to create HTTP client")?;

        // Verwende interne URL für JWKS-Fetch falls vorhanden, sonst externe
        let fetch_issuer = internal_issuer.unwrap_or(issuer);
        
        // Extrahiere Host-Header aus externer URL (für ZITADEL Domain-Validierung)
        let host_header = if internal_issuer.is_some() {
            url::Url::parse(issuer).ok().and_then(|u| {
                let host = u.host_str()?.to_string();
                let port = u.port();
                Some(if let Some(p) = port {
                    format!("{}:{}", host, p)
                } else {
                    host
                })
            })
        } else {
            None
        };
        
        // Fetch JWKS URL from OpenID Configuration
        let openid_config_url = format!("{}/.well-known/openid-configuration", fetch_issuer.trim_end_matches('/'));
        tracing::debug!("Fetching OpenID config from: {}, host_header: {:?}", openid_config_url, host_header);
        
        let mut request = client.get(&openid_config_url);
        if let Some(ref host) = host_header {
            request = request.header("Host", host);
        }
        
        let openid_config: serde_json::Value = request
            .send()
            .await
            .context("Failed to fetch OpenID configuration")?
            .json()
            .await
            .context("Failed to parse OpenID configuration")?;

        let mut jwks_url = openid_config["jwks_uri"]
            .as_str()
            .context("jwks_uri not found in OpenID configuration")?
            .to_string();
        
        // Wenn interne URL verwendet wird, ersetze den Host in der JWKS-URL
        if let Some(internal) = internal_issuer {
            if let (Ok(internal_url), Ok(mut parsed_jwks)) = (
                url::Url::parse(internal),
                url::Url::parse(&jwks_url)
            ) {
                if let Some(host) = internal_url.host_str() {
                    let _ = parsed_jwks.set_host(Some(host));
                    if let Some(port) = internal_url.port() {
                        let _ = parsed_jwks.set_port(Some(port));
                    }
                    jwks_url = parsed_jwks.to_string();
                    tracing::debug!("Using internal JWKS URL: {}", jwks_url);
                }
            }
        }

        let cache = Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            jwks_url,
            client,
            cache_duration: Duration::from_secs(cache_duration_secs),
            last_refresh: Arc::new(RwLock::new(Instant::now() - Duration::from_secs(cache_duration_secs + 1))),
            host_header,
        };

        // Initial Load
        cache.refresh().await?;

        Ok(cache)
    }

    /// Holt einen Key nach ID (refresht bei Bedarf)
    pub async fn get_key(&self, kid: &str) -> Result<Option<Jwk>> {
        // Prüfe ob Refresh nötig - scope the lock guard
        let needs_refresh = {
            let last = self.last_refresh.read();
            last.elapsed() > self.cache_duration
        };
        
        if needs_refresh {
            self.refresh().await?;
        }

        let keys = self.keys.read();
        Ok(keys.get(kid).cloned())
    }

    /// Refresht die Keys von ZITADEL
    async fn refresh(&self) -> Result<()> {
        tracing::debug!("Refreshing JWKS from {}", self.jwks_url);

        let mut request = self.client.get(&self.jwks_url);
        if let Some(ref host) = self.host_header {
            request = request.header("Host", host);
        }
        
        let response = request
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
