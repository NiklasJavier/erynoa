//! JWT Validator
//!
//! Validiert ZITADEL JWT Tokens

use anyhow::{Context, Result};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};

use super::claims::Claims;
use super::jwks::JwksCache;
use crate::config::AuthSettings;

/// JWT Validator für ZITADEL Tokens
pub struct JwtValidator {
    /// JWKS Cache
    jwks: JwksCache,
    /// Validation Settings
    validation: Validation,
    /// Erlaubte Issuer
    #[allow(dead_code)]
    issuer: String,
}

impl JwtValidator {
    /// Erstellt einen neuen Validator
    pub async fn new(settings: &AuthSettings) -> Result<Self> {
        let internal_issuer = settings.internal_issuer.as_deref();
        let jwks = JwksCache::new(&settings.issuer, internal_issuer, settings.jwks_cache_duration).await?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&settings.issuer]);
        validation.set_audience(&settings.audiences);
        validation.validate_exp = true;
        validation.validate_nbf = true;

        Ok(Self {
            jwks,
            validation,
            issuer: settings.issuer.clone(),
        })
    }

    /// Validiert einen JWT Token und gibt die Claims zurück
    pub async fn validate(&self, token: &str) -> Result<Claims> {
        // Header dekodieren um Key ID zu bekommen
        let header = decode_header(token).context("Invalid token header")?;

        let kid = header.kid.context("Token missing 'kid' in header")?;

        // Key aus Cache holen
        let jwk = self
            .jwks
            .get_key(&kid)
            .await?
            .context(format!("Unknown key ID: {}", kid))?;

        // Decoding Key erstellen
        let decoding_key = DecodingKey::from_jwk(&jwk).context("Invalid JWK")?;

        // Token validieren und dekodieren
        let token_data = decode::<Claims>(token, &decoding_key, &self.validation)
            .context("Token validation failed")?;

        let mut claims = token_data.claims;
        
        // Rollen extrahieren
        claims.extract_roles();

        Ok(claims)
    }

    /// Prüft ob der Validator noch funktioniert (für Health Checks)
    pub async fn is_healthy(&self) -> bool {
        self.jwks.get_key("any").await.is_ok()
    }
}

#[cfg(test)]
mod tests {
    // Tests kommen hier hin
    // Für echte Tests brauchen wir einen Mock ZITADEL Server
}
