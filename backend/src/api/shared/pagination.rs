//! Pagination Utilities
//!
//! Helper für Pagination in List-Endpoints

use serde::{Deserialize, Serialize};

/// Pagination Query Parameters
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}

impl PaginationQuery {
    /// Konvertiert zu SQL LIMIT/OFFSET
    #[allow(dead_code)]
    pub fn to_sql(&self) -> (i64, i64) {
        let limit = self.page_size as i64;
        let offset = ((self.page - 1) * self.page_size) as i64;
        (limit, offset)
    }
}

/// Paginated Response
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub page_size: u32,
    pub total: Option<u64>,
    pub has_next: bool,
}

impl<T> PaginatedResponse<T> {
    #[allow(dead_code)]
    pub fn new(items: Vec<T>, page: u32, page_size: u32, total: Option<u64>) -> Self {
        let has_next = if let Some(total) = total {
            u64::from(page * page_size) < total
        } else {
            items.len() == page_size as usize
        };

        Self {
            items,
            page,
            page_size,
            total,
            has_next,
        }
    }
}
