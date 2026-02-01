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
    // Storage (Fjall) - Decentralized
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Storage error: {0}")]
    Storage(String),

    // ─────────────────────────────────────────────────────────────────────────
    // Internal
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    // ─────────────────────────────────────────────────────────────────────────
    // Rate Limiting (Mana System)
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Rate limited: insufficient mana")]
    RateLimited {
        /// Zeit bis genug Mana regeneriert ist
        retry_after: std::time::Duration,
    },

    // ─────────────────────────────────────────────────────────────────────────
    // Domain Errors (für ?-Operator Propagation)
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Event error: {0}")]
    Event(String),

    #[error("Trust error: {0}")]
    Trust(String),

    #[error("Consensus error: {0}")]
    Consensus(String),

    #[error("Gateway error: {0}")]
    Gateway(String),

    #[error("Intent parse error: {0}")]
    IntentParse(String),

    #[error("Saga composition error: {0}")]
    SagaComposition(String),

    #[error("DID error: {0}")]
    DIDError(String),

    #[error("Protection error: {0}")]
    Protection(String),

    // ─────────────────────────────────────────────────────────────────────────
    // State & Capabilities
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Operation not supported: {0}")]
    NotSupported(String),
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
            Self::Storage(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            // Domain Errors → 400 Bad Request (client-side fixable)
            Self::Event(_) | Self::Trust(_) | Self::Consensus(_) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Self::Gateway(_) => StatusCode::FORBIDDEN,
            Self::IntentParse(_) | Self::SagaComposition(_) => StatusCode::BAD_REQUEST,
            Self::DIDError(_) => StatusCode::BAD_REQUEST,
            Self::Protection(_) => StatusCode::FORBIDDEN,
            Self::InvalidState(_) => StatusCode::CONFLICT,
            Self::NotSupported(_) => StatusCode::NOT_IMPLEMENTED,
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
            Self::Storage(_) => "STORAGE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            Self::RateLimited { .. } => "RATE_LIMITED",
            // Domain Errors
            Self::Event(_) => "EVENT_ERROR",
            Self::Trust(_) => "TRUST_ERROR",
            Self::Consensus(_) => "CONSENSUS_ERROR",
            Self::Gateway(_) => "GATEWAY_ERROR",
            Self::IntentParse(_) => "INTENT_PARSE_ERROR",
            Self::SagaComposition(_) => "SAGA_COMPOSITION_ERROR",
            Self::DIDError(_) => "DID_ERROR",
            Self::Protection(_) => "PROTECTION_ERROR",
            Self::InvalidState(_) => "INVALID_STATE",
            Self::NotSupported(_) => "NOT_SUPPORTED",
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
            ApiError::Internal(e) => {
                tracing::error!(error = ?e, "Internal error occurred");
            }
            ApiError::Storage(e) => {
                tracing::error!(error = %e, "Storage error occurred");
            }
            _ => {
                tracing::warn!(error = %self, "Client error occurred");
            }
        }

        let status = self.status_code();
        let error_code = self.error_code();

        // Für Production: Interne Fehler nicht im Detail exponieren
        let message = match &self {
            ApiError::Storage(_) | ApiError::Internal(_) => {
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

        // Für RateLimited: Retry-After Header hinzufügen
        if let ApiError::RateLimited { retry_after } = &self {
            return (
                status,
                [(
                    axum::http::header::RETRY_AFTER,
                    retry_after.as_secs().to_string(),
                )],
                Json(body),
            )
                .into_response();
        }

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
// From-Implementierungen für Domain-Errors
// Ermöglicht nahtlose Verwendung des ?-Operators
// ─────────────────────────────────────────────────────────────────────────────

impl From<crate::core::event_engine::EventError> for ApiError {
    fn from(err: crate::core::event_engine::EventError) -> Self {
        ApiError::Event(err.to_string())
    }
}

impl From<crate::core::trust_engine::TrustError> for ApiError {
    fn from(err: crate::core::trust_engine::TrustError) -> Self {
        ApiError::Trust(err.to_string())
    }
}

impl From<crate::core::consensus::ConsensusError> for ApiError {
    fn from(err: crate::core::consensus::ConsensusError) -> Self {
        ApiError::Consensus(err.to_string())
    }
}

impl From<crate::peer::gateway::GatewayError> for ApiError {
    fn from(err: crate::peer::gateway::GatewayError) -> Self {
        ApiError::Gateway(err.to_string())
    }
}

impl From<crate::peer::intent_parser::ParseError> for ApiError {
    fn from(err: crate::peer::intent_parser::ParseError) -> Self {
        ApiError::IntentParse(err.to_string())
    }
}

impl From<crate::peer::saga_composer::CompositionError> for ApiError {
    fn from(err: crate::peer::saga_composer::CompositionError) -> Self {
        ApiError::SagaComposition(err.to_string())
    }
}

impl From<crate::domain::IdentityError> for ApiError {
    fn from(err: crate::domain::IdentityError) -> Self {
        ApiError::DIDError(err.to_string())
    }
}

impl From<crate::protection::diversity::DiversityError> for ApiError {
    fn from(err: crate::protection::diversity::DiversityError) -> Self {
        ApiError::Protection(err.to_string())
    }
}

impl From<crate::protection::anti_calcification::CalcificationError> for ApiError {
    fn from(err: crate::protection::anti_calcification::CalcificationError) -> Self {
        ApiError::Protection(err.to_string())
    }
}

impl From<crate::protection::quadratic::GovernanceError> for ApiError {
    fn from(err: crate::protection::quadratic::GovernanceError) -> Self {
        ApiError::Protection(err.to_string())
    }
}

impl From<crate::protection::anomaly::AnomalyError> for ApiError {
    fn from(err: crate::protection::anomaly::AnomalyError) -> Self {
        ApiError::Protection(err.to_string())
    }
}

impl From<fjall::Error> for ApiError {
    fn from(err: fjall::Error) -> Self {
        ApiError::Storage(err.to_string())
    }
}
