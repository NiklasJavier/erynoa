//! Frontend Origin Detection Middleware
//!
//! Extrahiert die Frontend-Herkunft aus Origin/Referer Headers
//! und speichert sie in Request.extensions für Connect-RPC Handler
//!
//! ## Dezentrale Architektur
//!
//! Mit DID-basierter Auth werden keine OIDC Client-IDs mehr benötigt.
//! Diese Middleware identifiziert nur noch das Frontend für Logging/Debugging.

use axum::{
    extract::Request,
    http::{HeaderMap, Uri},
    middleware::Next,
    response::Response,
};

/// Frontend identifier stored in request extensions
#[derive(Debug, Clone)]
pub struct FrontendOrigin {
    /// Which frontend made the request
    pub frontend: FrontendType,
}

/// Known frontend types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontendType {
    Console,
    Platform,
    Docs,
    Unknown,
}

impl FrontendType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FrontendType::Console => "console",
            FrontendType::Platform => "platform",
            FrontendType::Docs => "docs",
            FrontendType::Unknown => "unknown",
        }
    }
}

/// Middleware to extract frontend origin from headers
pub async fn frontend_origin_middleware(mut request: Request, next: Next) -> Response {
    let headers = request.headers();
    let uri = request.uri().clone();
    let frontend = determine_frontend_from_headers(headers, &uri);

    // Store frontend info in extensions for handlers to access
    request.extensions_mut().insert(FrontendOrigin { frontend });

    next.run(request).await
}

/// Determine frontend type from request headers and URI
fn determine_frontend_from_headers(headers: &HeaderMap, uri: &Uri) -> FrontendType {
    // First: Check for X-Frontend-Origin header (set by frontend interceptor)
    if let Some(frontend_origin) = headers.get("x-frontend-origin") {
        if let Ok(frontend_str) = frontend_origin.to_str() {
            match frontend_str {
                "platform" => {
                    tracing::debug!("Detected Platform frontend from X-Frontend-Origin header");
                    return FrontendType::Platform;
                }
                "docs" => {
                    tracing::debug!("Detected Docs frontend from X-Frontend-Origin header");
                    return FrontendType::Docs;
                }
                "console" => {
                    tracing::debug!("Detected Console frontend from X-Frontend-Origin header");
                    return FrontendType::Console;
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
            if origin_str.contains("/platform") || origin_str.contains(":5174") {
                tracing::debug!("Detected Platform frontend from Origin: {}", origin_str);
                return FrontendType::Platform;
            }
            if origin_str.contains("/docs") || origin_str.contains(":5175") {
                tracing::debug!("Detected Docs frontend from Origin: {}", origin_str);
                return FrontendType::Docs;
            }
            if origin_str.contains("/console") || origin_str.contains(":5173") {
                tracing::debug!("Detected Console frontend from Origin: {}", origin_str);
                return FrontendType::Console;
            }
        }
    }

    // Fallback to Referer header
    if let Some(referer) = headers.get("referer") {
        if let Ok(referer_str) = referer.to_str() {
            if referer_str.contains("/platform") {
                return FrontendType::Platform;
            }
            if referer_str.contains("/docs") {
                return FrontendType::Docs;
            }
            if referer_str.contains("/console") {
                return FrontendType::Console;
            }
        }
    }

    // System endpoints don't need frontend detection
    if uri.path().contains("HealthService") {
        tracing::debug!("System endpoint, frontend unknown: {}", uri);
    } else {
        tracing::debug!("No frontend identified for: {}", uri);
    }

    FrontendType::Unknown
}
