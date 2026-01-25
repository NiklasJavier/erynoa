//! Error Handler Middleware
//!
//! Konvertiert unhandled Errors in API-Responses
//!
//! Note: Axum's IntoResponse für ApiError wird bereits verwendet.
//! Diese Middleware kann für zusätzliche Error-Transformationen verwendet werden.

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

/// Error Handler Middleware
/// 
/// Aktuell wird Error-Handling über IntoResponse für ApiError gehandhabt.
/// Diese Middleware kann für zukünftige Error-Transformationen erweitert werden.
#[allow(dead_code)]
pub async fn error_handler(request: Request, next: Next) -> Response {
    next.run(request).await
}
