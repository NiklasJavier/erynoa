//! Frontend Origin Detection Middleware
//!
//! Extrahiert die Frontend-Herkunft aus Origin/Referer Headers
//! und speichert sie in Request.extensions für Connect-RPC Handler

use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};

use crate::server::AppState;

/// Frontend identifier stored in request extensions
#[derive(Debug, Clone)]
pub struct FrontendOrigin {
    pub client_id: String,
}

/// Middleware to extract frontend origin from headers and determine client_id
pub async fn frontend_origin_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let headers = request.headers();
    let client_id = determine_client_id_from_headers(headers, &state);
    
    // Store client_id in extensions for handlers to access
    request.extensions_mut().insert(FrontendOrigin {
        client_id,
    });
    
    next.run(request).await
}

/// Determine client ID from request headers
fn determine_client_id_from_headers(headers: &HeaderMap, state: &AppState) -> String {
    // Try Origin header first
    if let Some(origin) = headers.get("origin") {
        if let Ok(origin_str) = origin.to_str() {
            if origin_str.contains("/platform") || origin_str.contains(":3001/platform") {
                return state.config.auth.platform_client_id.clone();
            } else if origin_str.contains("/docs") || origin_str.contains(":3001/docs") {
                return state.config.auth.docs_client_id.clone();
            } else if origin_str.contains("/console") || origin_str.contains(":3001/console") {
                return state.config.auth.console_client_id.clone();
            }
        }
    }
    
    // Fallback to Referer header
    if let Some(referer) = headers.get("referer") {
        if let Ok(referer_str) = referer.to_str() {
            if referer_str.contains("/platform") {
                return state.config.auth.platform_client_id.clone();
            } else if referer_str.contains("/docs") {
                return state.config.auth.docs_client_id.clone();
            } else if referer_str.contains("/console") {
                return state.config.auth.console_client_id.clone();
            }
        }
    }
    
    // Default to console
    state.config.auth.console_client_id.clone()
}
