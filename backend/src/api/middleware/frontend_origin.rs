//! Frontend Origin Detection Middleware
//!
//! Extrahiert die Frontend-Herkunft aus Origin/Referer Headers
//! und speichert sie in Request.extensions für Connect-RPC Handler

use axum::{
    extract::{Request, State},
    http::{HeaderMap, Uri},
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
    let uri = request.uri().clone();
    let client_id = determine_client_id_from_headers(headers, &uri, &state);
    
    // Store client_id in extensions for handlers to access
    request.extensions_mut().insert(FrontendOrigin {
        client_id,
    });
    
    next.run(request).await
}

/// Determine client ID from request headers and URI
fn determine_client_id_from_headers(headers: &HeaderMap, uri: &Uri, state: &AppState) -> String {
    // First: Check for X-Frontend-Origin header (set by frontend interceptor)
    if let Some(frontend_origin) = headers.get("x-frontend-origin") {
        if let Ok(frontend_str) = frontend_origin.to_str() {
            match frontend_str {
                "platform" => {
                    tracing::debug!("Detected Platform frontend from X-Frontend-Origin header");
                    return state.config.auth.platform_client_id.clone();
                }
                "docs" => {
                    tracing::debug!("Detected Docs frontend from X-Frontend-Origin header");
                    return state.config.auth.docs_client_id.clone();
                }
                "console" => {
                    tracing::debug!("Detected Console frontend from X-Frontend-Origin header");
                    return state.config.auth.console_client_id.clone();
                }
                _ => {
                    tracing::debug!("Unknown X-Frontend-Origin value: {}", frontend_str);
                }
            }
        }
    }
    
    // Try Origin header (most reliable for CORS requests)
    if let Some(origin) = headers.get("origin") {
        if let Ok(origin_str) = origin.to_str() {
            // Check for platform (must be first to avoid false matches)
            if origin_str.contains("/platform") || origin_str.contains(":5174") {
                tracing::debug!("Detected Platform frontend from Origin: {}", origin_str);
                return state.config.auth.platform_client_id.clone();
            }
            // Check for docs
            if origin_str.contains("/docs") || origin_str.contains(":5175") {
                tracing::debug!("Detected Docs frontend from Origin: {}", origin_str);
                return state.config.auth.docs_client_id.clone();
            }
            // Check for console
            if origin_str.contains("/console") || origin_str.contains(":5173") {
                tracing::debug!("Detected Console frontend from Origin: {}", origin_str);
                return state.config.auth.console_client_id.clone();
            }
            tracing::debug!("Origin header found but no frontend match: {}", origin_str);
        }
    }
    
    // Fallback to Referer header (for browser navigation)
    if let Some(referer) = headers.get("referer") {
        if let Ok(referer_str) = referer.to_str() {
            // Check for platform (must be first to avoid false matches)
            if referer_str.contains("/platform") {
                tracing::debug!("Detected Platform frontend from Referer: {}", referer_str);
                return state.config.auth.platform_client_id.clone();
            }
            // Check for docs
            if referer_str.contains("/docs") {
                tracing::debug!("Detected Docs frontend from Referer: {}", referer_str);
                return state.config.auth.docs_client_id.clone();
            }
            // Check for console
            if referer_str.contains("/console") {
                tracing::debug!("Detected Console frontend from Referer: {}", referer_str);
                return state.config.auth.console_client_id.clone();
            }
            tracing::debug!("Referer header found but no frontend match: {}", referer_str);
        }
    }
    
    // Last resort: Log all headers for debugging
    tracing::warn!(
        "No frontend identifier found, defaulting to console client ID. URI: {}, Headers: {:?}",
        uri,
        headers.keys().map(|k| k.as_str()).collect::<Vec<_>>()
    );
    
    // Default to console
    state.config.auth.console_client_id.clone()
}
