//! Connect-RPC User Service Implementation

use axum::extract::State;
use uuid::Uuid;
use axum_connect::pbjson_types::Timestamp;

use crate::auth::Claims;
use crate::server::AppState;
use crate::gen::godstack::v1::{
    ListUsersRequest, ListUsersResponse, GetUserRequest, GetUserResponse,
    User as ProtoUser,
};

/// List Users Handler
/// 
/// Note: Claims must be extracted via middleware before this handler
pub async fn list_users_handler(
    _claims: Claims,
    state: State<AppState>,
    request: ListUsersRequest,
) -> ListUsersResponse {
    // Check admin role (should be done in middleware, but double-check here)
    if !_claims.has_role("admin") {
        return ListUsersResponse {
            users: vec![],
            next_page_token: String::new(),
            total_count: 0,
        };
    }

    let page_size = if request.page_size == 0 { 20 } else { request.page_size } as i64;
    let page_token = if request.page_token.is_empty() { "0" } else { &request.page_token };
    let offset: i64 = page_token.parse().unwrap_or(0);

    // Fetch users and total count in parallel
    let (db_users_result, total_count_result) = tokio::join!(
        crate::db::User::list(&state.db, page_size, offset),
        crate::db::User::count(&state.db)
    );

    let users = match db_users_result {
        Ok(db_users) => db_users
            .into_iter()
            .map(|user| {
                // Convert chrono DateTime to protobuf Timestamp
                let created_at = Some(Timestamp {
                    seconds: user.created_at.timestamp(),
                    nanos: user.created_at.timestamp_subsec_nanos() as i32,
                });
                let updated_at = Some(Timestamp {
                    seconds: user.updated_at.timestamp(),
                    nanos: user.updated_at.timestamp_subsec_nanos() as i32,
                });

                // Clone email once to avoid move errors
                let email = user.email.clone().unwrap_or_default();

                ProtoUser {
                    id: user.id.to_string(),
                    email: email.clone(),
                    // Name is not stored in database, would need to fetch from ZITADEL userinfo endpoint
                    // For now, use email as fallback or empty string
                    name: email,
                    role: user.internal_role,
                    created_at,
                    updated_at,
                }
            })
            .collect(),
        Err(_) => vec![],
    };

    let total_count = total_count_result.unwrap_or(0) as i32;

    let next_page_token = if users.len() == page_size as usize {
        (offset + page_size).to_string()
    } else {
        String::new()
    };

    ListUsersResponse {
        users,
        next_page_token,
        total_count,
    }
}

/// Get User Handler
pub async fn get_user_handler(
    claims: Claims,
    state: State<AppState>,
    request: GetUserRequest,
) -> GetUserResponse {
    // Validate user ID format
    let user_id = match Uuid::parse_str(&request.id) {
        Ok(id) => id,
        Err(_) => return GetUserResponse { user: None },
    };

    let is_self = claims.sub == request.id;
    let is_admin = claims.has_role("admin");

    if !is_self && !is_admin {
        return GetUserResponse { user: None };
    }

    // Load from database
    match crate::db::User::find_by_id(&state.db, user_id).await {
        Ok(Some(user)) => {
            let created_at = Some(Timestamp {
                seconds: user.created_at.timestamp(),
                nanos: user.created_at.timestamp_subsec_nanos() as i32,
            });
            let updated_at = Some(Timestamp {
                seconds: user.updated_at.timestamp(),
                nanos: user.updated_at.timestamp_subsec_nanos() as i32,
            });

            // Clone email once to avoid move errors
            let email = user.email.clone().unwrap_or_default();

            GetUserResponse {
                user: Some(ProtoUser {
                    id: user.id.to_string(),
                    email: email.clone(),
                    // Name is not stored in database, would need to fetch from ZITADEL userinfo endpoint
                    // For now, use email as fallback
                    name: email,
                    role: user.internal_role,
                    created_at,
                    updated_at,
                }),
            }
        }
        Ok(None) => GetUserResponse { user: None },
        Err(_) => GetUserResponse { user: None },
    }
}
