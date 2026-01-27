//! Authentication Middleware
//!
//! Validiert JWT Tokens aus Authorization Header und fügt Claims zu Request Extensions hinzu
//!
//! Note: Diese Middleware wird als Layer auf spezifische Routen angewendet.
//! Für public routes wird sie nicht verwendet.

use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};

// use crate::auth::Claims; // Wird aktuell nicht verwendet (auth_middleware ist deaktiviert)
use crate::error::ApiError;
use crate::server::AppState;

/// JWT Authentication Middleware
///
/// Extrahiert Bearer Token aus Authorization Header, validiert es
/// und fügt Claims zu Request Extensions hinzu.
///
/// Wenn kein Token vorhanden ist, wird der Request weitergeleitet
/// (für public routes). Die Route-Handler können dann Claims als
/// optionalen Parameter verwenden.
///
/// Diese Middleware sollte nur auf protected routes angewendet werden.
#[allow(dead_code)]
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Prüfe ob Authorization Header vorhanden ist
    if let Some(auth_header) = request.headers().get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // Token validieren
                if let Some(ref validator) = state.jwt_validator {
                    match validator.validate(token).await {
                        Ok(claims) => {
                            // Claims zu Extensions hinzufügen
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
                    // JWT Validator nicht verfügbar - Auth deaktiviert
                    tracing::debug!("JWT validator not available, skipping auth");
                }
            }
        }
    }

    Ok(next.run(request).await)
}
