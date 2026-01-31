//! Request Logging Middleware
//!
//! Loggt alle Requests mit Method, Path, Status und Duration

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use std::time::Instant;
use tracing::info;

/// Loggt Requests mit Timing-Informationen
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let start = Instant::now();

    let response = next.run(request).await;
    let duration = start.elapsed();
    let status = response.status();

    // Log-Level basierend auf Status Code
    if status.is_server_error() {
        tracing::error!(
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = duration.as_millis(),
            "Request failed"
        );
    } else if status.is_client_error() {
        // Log 404 errors at debug level (deprecated endpoints, etc.)
        // Other client errors (400, 401, 403) are still warnings
        if status == StatusCode::NOT_FOUND {
            tracing::debug!(
                method = %method,
                path = %path,
                status = %status.as_u16(),
                duration_ms = duration.as_millis(),
                "Not found (deprecated endpoint or invalid path)"
            );
        } else {
            tracing::warn!(
                method = %method,
                path = %path,
                status = %status.as_u16(),
                duration_ms = duration.as_millis(),
                "Client error"
            );
        }
    } else {
        // Health checks und häufige Endpoints auf DEBUG-Level loggen
        if path.contains("HealthService") || path.contains("InfoService") {
            tracing::debug!(
                method = %method,
                path = %path,
                status = %status.as_u16(),
                duration_ms = duration.as_millis(),
                "Request completed"
            );
        } else {
            info!(
                method = %method,
                path = %path,
                status = %status.as_u16(),
                duration_ms = duration.as_millis(),
                "Request completed"
            );
        }
    }

    response
}
