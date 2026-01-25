//! Shared API Utilities
//!
//! Wiederverwendbare Komponenten für API-Features

mod pagination;

// Re-export pagination types
pub use pagination::{PaginationQuery, PaginatedResponse};
