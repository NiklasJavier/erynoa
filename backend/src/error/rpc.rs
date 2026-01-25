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
