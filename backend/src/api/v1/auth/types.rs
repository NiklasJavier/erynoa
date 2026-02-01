//! Passkey Authentication Types
//!
//! Request and Response types for WebAuthn/Passkey endpoints.

use serde::{Deserialize, Serialize};

/// Challenge Response für WebAuthn Operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    /// Base64URL encoded Challenge (32 random bytes)
    pub challenge: String,
    /// Optional Challenge ID for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge_id: Option<String>,
    /// Expiration timestamp (Unix seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
}

/// Passkey Registration Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyRegistrationRequest {
    /// Credential ID (Base64URL encoded)
    pub credential_id: String,
    /// Public Key (Base64URL encoded)
    pub public_key: String,
    /// COSE Algorithm ID (-8 for Ed25519, -7 for ES256)
    pub algorithm: i32,
    /// Generated DID
    pub did: String,
    /// DID Namespace
    pub namespace: String,
    /// Optional display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Authenticator transports
    #[serde(default)]
    pub transports: Vec<String>,
}

/// Passkey Registration Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyRegistrationResponse {
    /// Whether registration was successful
    pub success: bool,
    /// Registered DID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did: Option<String>,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Passkey Verification Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyVerificationRequest {
    /// Credential ID (Base64URL encoded)
    pub credential_id: String,
    /// Signature (Base64URL encoded)
    pub signature: String,
    /// Authenticator Data (Base64URL encoded)
    pub authenticator_data: String,
    /// Client Data JSON (Base64URL encoded)
    pub client_data_json: String,
}

/// Passkey Verification Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyVerificationResponse {
    /// Whether verification was successful
    pub success: bool,
    /// Verified DID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did: Option<String>,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Stored Passkey Credential (for backend persistence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPasskeyCredential {
    /// Credential ID (Base64URL encoded)
    pub credential_id: String,
    /// Public Key (Hex encoded for compatibility with existing system)
    pub public_key_hex: String,
    /// COSE Algorithm ID
    pub algorithm: i32,
    /// Associated DID
    pub did: String,
    /// DID Namespace
    pub namespace: String,
    /// Display Name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Authenticator transports
    #[serde(default)]
    pub transports: Vec<String>,
    /// Sign counter (for replay protection)
    pub sign_count: u32,
    /// Creation timestamp
    pub created_at: i64,
    /// Last authentication timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<i64>,
}

/// COSE Algorithm Constants
pub mod cose {
    /// Ed25519 (EdDSA with Ed25519 curve)
    pub const ED25519: i32 = -8;
    /// ES256 (ECDSA with P-256 and SHA-256)
    pub const ES256: i32 = -7;
    /// RS256 (RSASSA-PKCS1-v1_5 with SHA-256)
    pub const RS256: i32 = -257;
}

/// Validates that the algorithm is supported
pub fn is_supported_algorithm(alg: i32) -> bool {
    matches!(alg, cose::ED25519 | cose::ES256)
}

/// Returns algorithm name for display
pub fn algorithm_name(alg: i32) -> &'static str {
    match alg {
        cose::ED25519 => "Ed25519",
        cose::ES256 => "ES256",
        cose::RS256 => "RS256",
        _ => "Unknown",
    }
}
