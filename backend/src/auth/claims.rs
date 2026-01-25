//! JWT Claims
//!
//! Strukturen für ZITADEL JWT Tokens

use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use serde::{Deserialize, Serialize};
use axum_connect::parts::RpcFromRequestParts;

use crate::error::ApiError;
#[cfg(feature = "connect")]
use crate::error::ApiErrorToRpc;

/// JWT Claims aus einem ZITADEL Token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (User ID in ZITADEL)
    pub sub: String,
    
    /// Issuer (ZITADEL URL)
    pub iss: String,
    
    /// Audience
    pub aud: OneOrMany<String>,
    
    /// Expiration Time (Unix Timestamp)
    pub exp: i64,
    
    /// Issued At (Unix Timestamp)
    pub iat: i64,
    
    /// Not Before (Unix Timestamp)
    #[serde(default)]
    pub nbf: Option<i64>,
    
    /// JWT ID
    #[serde(default)]
    pub jti: Option<String>,
    
    // ─────────────────────────────────────────────────────────────────────────
    // ZITADEL Spezifische Claims
    // ─────────────────────────────────────────────────────────────────────────
    
    /// Email Adresse
    #[serde(default)]
    pub email: Option<String>,
    
    /// Email verifiziert?
    #[serde(default)]
    pub email_verified: Option<bool>,
    
    /// Vollständiger Name
    #[serde(default)]
    pub name: Option<String>,
    
    /// Vorname
    #[serde(default)]
    pub given_name: Option<String>,
    
    /// Nachname
    #[serde(default)]
    pub family_name: Option<String>,
    
    /// Preferred Username
    #[serde(default)]
    pub preferred_username: Option<String>,
    
    /// Locale
    #[serde(default)]
    pub locale: Option<String>,
    
    // ─────────────────────────────────────────────────────────────────────────
    // Rollen & Berechtigungen (ZITADEL)
    // ─────────────────────────────────────────────────────────────────────────
    
    /// Rollen aus ZITADEL (urn:zitadel:iam:org:project:roles)
    #[serde(default, rename = "urn:zitadel:iam:org:project:roles")]
    pub zitadel_roles: Option<serde_json::Value>,
    
    /// Vereinfachte Rollen (von uns extrahiert)
    #[serde(skip)]
    pub roles: Vec<String>,
}

impl Claims {
    /// Prüft ob der User eine bestimmte Rolle hat
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
    
    /// Prüft ob der User eine der angegebenen Rollen hat
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }
    
    /// Extrahiert Rollen aus ZITADEL's komplexem Format
    pub fn extract_roles(&mut self) {
        if let Some(roles_value) = &self.zitadel_roles {
            if let Some(roles_obj) = roles_value.as_object() {
                self.roles = roles_obj.keys().cloned().collect();
            }
        }
    }
}

/// Hilfsstruct für aud Claim (kann String oder Array sein)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T: PartialEq> OneOrMany<T> {
    pub fn contains(&self, value: &T) -> bool {
        match self {
            OneOrMany::One(v) => v == value,
            OneOrMany::Many(vec) => vec.contains(value),
        }
    }
}

// Extractor für Claims aus Request Extensions
// Note: In axum 0.8, native async traits are supported, no async_trait needed
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or_else(|| ApiError::Unauthorized("Missing authentication".into()))
    }
}

// RpcFromRequestParts implementation for Connect-RPC
// This allows Claims to be extracted in Connect-RPC handlers
// Note: axum-connect uses async_trait for RpcFromRequestParts
use async_trait::async_trait;

#[async_trait]
impl<M, S> RpcFromRequestParts<M, S> for Claims
where
    M: prost::Message,
    S: Send + Sync,
{
    type Rejection = axum_connect::error::RpcError;

    async fn rpc_from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or_else(|| {
                // Use ApiErrorToRpc trait for consistent error conversion
                ApiError::Unauthorized("Missing authentication".into())
                    .to_rpc_error()
            })
    }
}
