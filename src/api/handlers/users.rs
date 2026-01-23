//! User Handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::Claims;
use crate::error::{ApiError, Result};
use crate::server::AppState;

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 { 1 }
fn default_page_size() -> u32 { 20 }

#[derive(Debug, Serialize)]
pub struct ListUsersResponse {
    pub users: Vec<UserResponse>,
    pub total: u32,
    pub page: u32,
}

/// GET /me - Current user info
pub async fn get_current_user(claims: Claims) -> Json<UserResponse> {
    Json(UserResponse {
        id: claims.sub,
        email: claims.email,
        name: claims.name,
        roles: claims.roles,
    })
}

/// GET /users/:id
pub async fn get_user(
    Path(user_id): Path<Uuid>,
    claims: Claims,
    State(_state): State<AppState>,
) -> Result<Json<UserResponse>> {
    let is_self = claims.sub == user_id.to_string();
    let is_admin = claims.has_role("admin");
    
    if !is_self && !is_admin {
        return Err(ApiError::Forbidden);
    }

    // TODO: Load from database
    Err(ApiError::NotFound(format!("User {user_id} not found")))
}

/// GET /users - List users (admin only)
pub async fn list_users(
    Query(query): Query<ListUsersQuery>,
    claims: Claims,
    State(_state): State<AppState>,
) -> Result<Json<ListUsersResponse>> {
    if !claims.has_role("admin") {
        return Err(ApiError::Forbidden);
    }

    // TODO: Load from database with pagination
    tracing::debug!("Listing users: page={}, page_size={}", query.page, query.page_size);
    
    Ok(Json(ListUsersResponse {
        users: vec![],
        total: 0,
        page: query.page,
    }))
}
