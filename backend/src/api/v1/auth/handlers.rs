//! Passkey Authentication Handlers
//!
//! Axum handlers for WebAuthn Challenge, Registration, and Verification.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use rand::RngCore;

use super::types::{
    is_supported_algorithm, ChallengeResponse, PasskeyRegistrationRequest,
    PasskeyRegistrationResponse, PasskeyVerificationRequest, PasskeyVerificationResponse,
};
use crate::server::AppState;

/// Base64URL encode bytes
fn base64url_encode(bytes: &[u8]) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Base64URL decode string
fn base64url_decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    URL_SAFE_NO_PAD.decode(s)
}

/// Challenge validity in seconds (5 minutes)
const CHALLENGE_VALIDITY_SECS: i64 = 300;

// ============================================================================
// CHALLENGE ENDPOINT
// ============================================================================

/// GET /api/v1/auth/challenge
///
/// Generates a new cryptographically secure challenge for WebAuthn operations.
/// The challenge is a 32-byte random value, Base64URL encoded.
///
/// # Response
/// ```json
/// {
///     "challenge": "base64url-encoded-32-bytes",
///     "expires_at": 1706745600
/// }
/// ```
pub async fn get_challenge() -> impl IntoResponse {
    let mut challenge_bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut challenge_bytes);

    let challenge = base64url_encode(&challenge_bytes);
    let expires_at = chrono::Utc::now().timestamp() + CHALLENGE_VALIDITY_SECS;

    let response = ChallengeResponse {
        challenge,
        challenge_id: None, // Could add tracking if needed
        expires_at: Some(expires_at),
    };

    (StatusCode::OK, Json(response))
}

// ============================================================================
// REGISTRATION ENDPOINT
// ============================================================================

/// POST /api/v1/auth/passkey/register
///
/// Registers a new Passkey credential with the backend.
/// This stores the public key for future signature verification.
///
/// # Request Body
/// ```json
/// {
///     "credential_id": "base64url-credential-id",
///     "public_key": "base64url-public-key",
///     "algorithm": -8,
///     "did": "did:erynoa:self:abc123",
///     "namespace": "self",
///     "display_name": "My Passkey"
/// }
/// ```
///
/// # Response
/// ```json
/// {
///     "success": true,
///     "did": "did:erynoa:self:abc123"
/// }
/// ```
pub async fn register_passkey(
    State(state): State<AppState>,
    Json(request): Json<PasskeyRegistrationRequest>,
) -> impl IntoResponse {
    // Validate algorithm
    if !is_supported_algorithm(request.algorithm) {
        return (
            StatusCode::BAD_REQUEST,
            Json(PasskeyRegistrationResponse {
                success: false,
                did: None,
                error: Some(format!("Unsupported algorithm: {}", request.algorithm)),
            }),
        );
    }

    // Decode and validate public key
    let public_key_bytes = match base64url_decode(&request.public_key) {
        Ok(bytes) => bytes,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(PasskeyRegistrationResponse {
                    success: false,
                    did: None,
                    error: Some("Invalid public key encoding".to_string()),
                }),
            );
        }
    };

    // Validate Ed25519 key length (32 bytes)
    if request.algorithm == super::types::cose::ED25519 && public_key_bytes.len() != 32 {
        return (
            StatusCode::BAD_REQUEST,
            Json(PasskeyRegistrationResponse {
                success: false,
                did: None,
                error: Some(format!(
                    "Invalid Ed25519 public key length: {} (expected 32)",
                    public_key_bytes.len()
                )),
            }),
        );
    }

    // Convert to hex for storage (compatible with existing identity_store)
    let public_key_hex = hex::encode(&public_key_bytes);

    // Parse DID to validate format
    let did_result: Result<crate::domain::DID, _> = request.did.parse();
    if did_result.is_err() {
        return (
            StatusCode::BAD_REQUEST,
            Json(PasskeyRegistrationResponse {
                success: false,
                did: None,
                error: Some("Invalid DID format".to_string()),
            }),
        );
    }

    // Store the passkey credential
    // We use the existing identity_store with the passkey-specific metadata
    let stored = super::types::StoredPasskeyCredential {
        credential_id: request.credential_id.clone(),
        public_key_hex,
        algorithm: request.algorithm,
        did: request.did.clone(),
        namespace: request.namespace,
        display_name: request.display_name,
        transports: request.transports,
        sign_count: 0,
        created_at: chrono::Utc::now().timestamp(),
        last_used_at: None,
    };

    // Store in the passkey credentials store
    match state.storage.identities.store_passkey_credential(&stored) {
        Ok(_) => {
            tracing::info!(
                did = %request.did,
                credential_id = %request.credential_id,
                algorithm = request.algorithm,
                "Passkey registered successfully"
            );

            (
                StatusCode::OK,
                Json(PasskeyRegistrationResponse {
                    success: true,
                    did: Some(request.did),
                    error: None,
                }),
            )
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to store passkey credential");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PasskeyRegistrationResponse {
                    success: false,
                    did: None,
                    error: Some("Failed to store credential".to_string()),
                }),
            )
        }
    }
}

// ============================================================================
// VERIFICATION ENDPOINT
// ============================================================================

/// POST /api/v1/auth/passkey/verify
///
/// Verifies a WebAuthn assertion (authentication).
/// Validates the signature against the stored public key.
///
/// # Request Body
/// ```json
/// {
///     "credential_id": "base64url-credential-id",
///     "signature": "base64url-signature",
///     "authenticator_data": "base64url-auth-data",
///     "client_data_json": "base64url-client-data"
/// }
/// ```
pub async fn verify_passkey(
    State(state): State<AppState>,
    Json(request): Json<PasskeyVerificationRequest>,
) -> impl IntoResponse {
    // Retrieve stored credential
    let stored = match state
        .storage
        .identities
        .get_passkey_credential(&request.credential_id)
    {
        Ok(Some(cred)) => cred,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(PasskeyVerificationResponse {
                    success: false,
                    did: None,
                    error: Some("Credential not found".to_string()),
                }),
            );
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to retrieve credential");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PasskeyVerificationResponse {
                    success: false,
                    did: None,
                    error: Some("Internal error".to_string()),
                }),
            );
        }
    };

    // Decode signature
    let signature_bytes = match base64url_decode(&request.signature) {
        Ok(bytes) => bytes,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(PasskeyVerificationResponse {
                    success: false,
                    did: None,
                    error: Some("Invalid signature encoding".to_string()),
                }),
            );
        }
    };

    // Decode authenticator data and client data
    let auth_data = match base64url_decode(&request.authenticator_data) {
        Ok(bytes) => bytes,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(PasskeyVerificationResponse {
                    success: false,
                    did: None,
                    error: Some("Invalid authenticator data".to_string()),
                }),
            );
        }
    };

    let client_data_json = match base64url_decode(&request.client_data_json) {
        Ok(bytes) => bytes,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(PasskeyVerificationResponse {
                    success: false,
                    did: None,
                    error: Some("Invalid client data".to_string()),
                }),
            );
        }
    };

    // Compute client data hash (SHA-256)
    use sha2::{Digest, Sha256};
    let client_data_hash = Sha256::digest(&client_data_json);

    // Concatenate authenticator data + client data hash (this is what's signed)
    let mut signed_data = auth_data.clone();
    signed_data.extend_from_slice(&client_data_hash);

    // Verify signature based on algorithm
    let verification_result = match stored.algorithm {
        super::types::cose::ED25519 => {
            verify_ed25519_signature(&stored.public_key_hex, &signed_data, &signature_bytes)
        }
        _ => {
            // For now, only Ed25519 is fully implemented
            Err("Algorithm verification not implemented".to_string())
        }
    };

    match verification_result {
        Ok(true) => {
            // Update last used timestamp
            if let Err(e) = state
                .storage
                .identities
                .update_passkey_last_used(&request.credential_id)
            {
                tracing::warn!(error = %e, "Failed to update last used timestamp");
            }

            tracing::info!(
                did = %stored.did,
                credential_id = %request.credential_id,
                "Passkey verification successful"
            );

            (
                StatusCode::OK,
                Json(PasskeyVerificationResponse {
                    success: true,
                    did: Some(stored.did),
                    error: None,
                }),
            )
        }
        Ok(false) => {
            tracing::warn!(
                credential_id = %request.credential_id,
                "Passkey verification failed: invalid signature"
            );

            (
                StatusCode::UNAUTHORIZED,
                Json(PasskeyVerificationResponse {
                    success: false,
                    did: None,
                    error: Some("Invalid signature".to_string()),
                }),
            )
        }
        Err(e) => {
            tracing::error!(error = %e, "Passkey verification error");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PasskeyVerificationResponse {
                    success: false,
                    did: None,
                    error: Some(e),
                }),
            )
        }
    }
}

/// Verify an Ed25519 signature
fn verify_ed25519_signature(
    public_key_hex: &str,
    message: &[u8],
    signature: &[u8],
) -> Result<bool, String> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    // Decode public key from hex
    let public_key_bytes =
        hex::decode(public_key_hex).map_err(|e| format!("Invalid public key hex: {}", e))?;

    // Create verifying key
    let verifying_key = VerifyingKey::try_from(public_key_bytes.as_slice())
        .map_err(|e| format!("Invalid Ed25519 public key: {}", e))?;

    // Parse signature (64 bytes for Ed25519)
    if signature.len() != 64 {
        return Err(format!(
            "Invalid signature length: {} (expected 64)",
            signature.len()
        ));
    }

    let sig_bytes: [u8; 64] = signature
        .try_into()
        .map_err(|_| "Invalid signature length")?;
    let sig = Signature::from_bytes(&sig_bytes);

    // Verify
    Ok(verifying_key.verify(message, &sig).is_ok())
}

// ============================================================================
// ROUTER
// ============================================================================

use axum::{
    routing::{get, post},
    Router,
};

/// Creates the auth routes
pub fn create_auth_routes() -> Router<AppState> {
    Router::new()
        .route("/challenge", get(get_challenge))
        .route("/passkey/register", post(register_passkey))
        .route("/passkey/verify", post(verify_passkey))
}
