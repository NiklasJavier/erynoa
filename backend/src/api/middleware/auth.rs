//! Authentication Middleware
//!
//! Unterstützt sowohl dezentrale DID-basierte Auth als auch Legacy JWT/OIDC.
//!
//! Note: Diese Middleware wird als Layer auf spezifische Routen angewendet.
//! Für public routes wird sie nicht verwendet.

use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};

use crate::error::ApiError;
use crate::server::AppState;

/// Authentication Middleware
///
/// Unterstützt zwei Modi:
/// 1. DID-basierte Auth: `Authorization: DID <did>:<signature>`
/// 2. Legacy JWT Auth: `Authorization: Bearer <token>` (wenn legacy-oidc aktiviert)
///
/// Diese Middleware sollte nur auf protected routes angewendet werden.
#[allow(dead_code)]
pub async fn auth_middleware(
    State(_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Prüfe ob Authorization Header vorhanden ist
    if let Some(auth_header) = request.headers().get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            // DID-basierte Auth
            if let Some(_did_auth) = auth_str.strip_prefix("DID ") {
                // TODO: DID Challenge-Response Authentifizierung implementieren
                // Format: DID <did>:<signature>
                // Die Signatur wird gegen eine gespeicherte Challenge verifiziert
                tracing::debug!("DID-based auth detected (not yet implemented)");
            }
            // Legacy JWT Auth
            #[cfg(feature = "legacy-oidc")]
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Some(ref validator) = state.jwt_validator {
                    match validator.validate(token).await {
                        Ok(claims) => {
                            request.extensions_mut().insert(claims);
                        }
                        Err(e) => {
                            return Err(ApiError::InvalidToken(format!(
                                "Token validation failed: {}",
                                e
                            )));
                        }
                    }
                } else {
                    tracing::debug!("JWT validator not available, skipping auth");
                }
            }
        }
    }

    Ok(next.run(request).await)
}
