//! Error Handling Module
//!
//! Zentrale Fehlerbehandlung mit anyhow und thiserror

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

/// Result-Typ für die API
pub type Result<T> = std::result::Result<T, ApiError>;

/// Haupt-Fehlertyp für die API
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    // ─────────────────────────────────────────────────────────────────────────
    // Authentication & Authorization
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: insufficient permissions")]
    Forbidden,

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    // ─────────────────────────────────────────────────────────────────────────
    // Validation
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    // ─────────────────────────────────────────────────────────────────────────
    // Resources
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Resource already exists: {0}")]
    Conflict(String),

    // ─────────────────────────────────────────────────────────────────────────
    // Database
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    // ─────────────────────────────────────────────────────────────────────────
    // Cache
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Cache error")]
    Cache(#[from] fred::error::RedisError),

    // ─────────────────────────────────────────────────────────────────────────
    // Internal
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl ApiError {
    /// HTTP Status Code für den Fehler
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized(_) | Self::InvalidToken(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Validation(_) | Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Database(_) | Self::Cache(_) | Self::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// Fehler-Code für die API-Response
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::Forbidden => "FORBIDDEN",
            Self::InvalidToken(_) => "INVALID_TOKEN",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Cache(_) => "CACHE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
        }
    }
}

/// API Error Response Format
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
}

#[derive(Serialize)]
pub struct ErrorDetails {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Log den Fehler (für interne Fehler mit voller Info)
        match &self {
            ApiError::Database(e) => {
                tracing::error!(error = ?e, "Database error occurred");
            }
            ApiError::Cache(e) => {
                tracing::error!(error = ?e, "Cache error occurred");
            }
            ApiError::Internal(e) => {
                tracing::error!(error = ?e, "Internal error occurred");
            }
            _ => {
                tracing::warn!(error = %self, "Client error occurred");
            }
        }

        let status = self.status_code();
        let error_code = self.error_code();

        // Für Production: Interne Fehler nicht im Detail exponieren
        let message = match &self {
            ApiError::Database(_) | ApiError::Cache(_) | ApiError::Internal(_) => {
                "An internal error occurred. Please try again later.".to_string()
            }
            _ => self.to_string(),
        };

        let body = ErrorResponse {
            error: ErrorDetails {
                code: error_code,
                message,
                details: None,
            },
        };

        (status, Json(body)).into_response()
    }
}

/// Extension Trait für einfacheres Error-Handling
pub trait ResultExt<T> {
    fn context_api(self, msg: &str) -> Result<T>;
}

impl<T, E: fmt::Display> ResultExt<T> for std::result::Result<T, E> {
    fn context_api(self, msg: &str) -> Result<T> {
        self.map_err(|e| ApiError::Internal(anyhow::anyhow!("{}: {}", msg, e)))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Connect-RPC Error Conversion
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "connect")]
mod rpc {
    //! RPC Error Conversion Utilities
    //!
    //! Provides utilities for converting ApiError to RpcError for Connect-RPC handlers

    use axum_connect::error::{RpcError, RpcErrorCode, RpcIntoError};
    use crate::error::ApiError;

    /// Extension trait for converting ApiError to RpcError
    pub trait ApiErrorToRpc {
        /// Convert ApiError to RpcError
        fn to_rpc_error(self) -> RpcError;
    }

    impl ApiErrorToRpc for ApiError {
        fn to_rpc_error(self) -> RpcError {
            let (code, message) = match self {
                ApiError::Unauthorized(msg) => (
                    RpcErrorCode::Unauthenticated,
                    format!("Unauthorized: {}", msg),
                ),
                ApiError::Forbidden => (
                    RpcErrorCode::PermissionDenied,
                    "Forbidden: insufficient permissions".to_string(),
                ),
                ApiError::InvalidToken(msg) => (
                    RpcErrorCode::Unauthenticated,
                    format!("Invalid token: {}", msg),
                ),
                ApiError::Validation(msg) | ApiError::BadRequest(msg) => (
                    RpcErrorCode::InvalidArgument,
                    format!("Validation error: {}", msg),
                ),
                ApiError::NotFound(msg) => (
                    RpcErrorCode::NotFound,
                    format!("Resource not found: {}", msg),
                ),
                ApiError::Conflict(msg) => (
                    RpcErrorCode::AlreadyExists,
                    format!("Resource already exists: {}", msg),
                ),
                ApiError::Database(_) | ApiError::Cache(_) | ApiError::Internal(_) => (
                    RpcErrorCode::Internal,
                    "An internal error occurred. Please try again later.".to_string(),
                ),
                ApiError::ServiceUnavailable(msg) => (
                    RpcErrorCode::Unavailable,
                    format!("Service unavailable: {}", msg),
                ),
            };

            (code, message).rpc_into_error()
        }
    }

    /// Helper function to convert Result<T, ApiError> to Result<T, RpcError>
    pub fn map_api_error<T>(result: Result<T, ApiError>) -> Result<T, RpcError> {
        result.map_err(|e| e.to_rpc_error())
    }
}

#[cfg(feature = "connect")]
pub use rpc::{ApiErrorToRpc, map_api_error};
