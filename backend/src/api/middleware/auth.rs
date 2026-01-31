//! Authentication Middleware
//!
//! Dezentrale DID-basierte Authentifizierung.

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
/// Unterstützt DID-basierte Auth: `Authorization: DID <did>:<signature>`
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
        }
    }

    Ok(next.run(request).await)
}

