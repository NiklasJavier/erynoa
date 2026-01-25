//! Request Logging Middleware
//!
//! Loggt alle Requests mit Method, Path, Status und Duration

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
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
        tracing::warn!(
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = duration.as_millis(),
            "Client error"
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

    response
}
