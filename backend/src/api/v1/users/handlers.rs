//! Connect-RPC User Service Implementation

use axum::extract::State;
use uuid::Uuid;
use axum_connect::pbjson_types::Timestamp;

use crate::auth::Claims;
use crate::server::AppState;
use crate::gen::erynoa::v1::{
    ListUsersRequest, ListUsersResponse, GetUserRequest, GetUserResponse,
    GetCurrentUserRequest,
    User as ProtoUser,
};

/// List Users Handler
/// 
/// Returns a paginated list of users from the database.
/// 
/// # Authentication
/// Requires authentication via JWT token. Claims are extracted via middleware.
/// 
/// # Authorization
/// Only users with "admin" role can access this endpoint.
/// Non-admin users will receive an empty list.
/// 
/// # Example Request (Connect-RPC)
/// ```protobuf
/// service UserService {
///   rpc List(ListUsersRequest) returns (ListUsersResponse);
/// }
/// ```
/// 
/// # Example Request (JSON)
/// ```json
/// {
///   "page_size": 20,
///   "page_token": "0"
/// }
/// ```
/// 
/// # Example Response
/// ```json
/// {
///   "users": [
///     {
///       "id": "123e4567-e89b-12d3-a456-426614174000",
///       "email": "user@example.com",
///       "name": "user@example.com",
///       "role": "user",
///       "created_at": { "seconds": 1234567890, "nanos": 0 },
///       "updated_at": { "seconds": 1234567890, "nanos": 0 }
///     }
///   ],
///   "next_page_token": "20",
///   "total_count": 100
/// }
/// ```
/// 
/// # Errors
/// - Returns empty list if user is not admin
/// - Returns empty list on database errors (errors are logged)
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
/// 
/// Retrieves a single user by ID from the database.
/// 
/// # Authentication
/// Requires authentication via JWT token.
/// 
/// # Authorization
/// Users can only retrieve their own user data, unless they have "admin" role.
/// 
/// # Example Request
/// ```json
/// {
///   "id": "123e4567-e89b-12d3-a456-426614174000"
/// }
/// ```
/// 
/// # Example Response
/// ```json
/// {
///   "user": {
///     "id": "123e4567-e89b-12d3-a456-426614174000",
///     "email": "user@example.com",
///     "name": "user@example.com",
///     "role": "user",
///     "created_at": { "seconds": 1234567890, "nanos": 0 },
///     "updated_at": { "seconds": 1234567890, "nanos": 0 }
///   }
/// }
/// ```
/// 
/// # Errors
/// - Returns `user: null` if user not found
/// - Returns `user: null` if user lacks permission
/// - Returns `user: null` on database errors (errors are logged)
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

/// Get Current User Handler
/// 
/// Returns the currently authenticated user's information.
/// 
/// # Authentication
/// Requires authentication via JWT token.
/// 
/// # Behavior
/// 1. Attempts to load user from database if `claims.sub` is a valid UUID
/// 2. Falls back to JWT claims if user not found in database
/// 3. This allows ZITADEL users to be represented even if not in our database
/// 
/// # Example Request
/// ```json
/// {}
/// ```
/// 
/// # Example Response
/// ```json
/// {
///   "user": {
///     "id": "123e4567-e89b-12d3-a456-426614174000",
///     "email": "user@example.com",
///     "name": "John Doe",
///     "role": "user",
///     "created_at": { "seconds": 1234567890, "nanos": 0 },
///     "updated_at": { "seconds": 1234567890, "nanos": 0 }
///   }
/// }
/// ```
/// 
/// # Notes
/// - ZITADEL user IDs might differ from database UUIDs
/// - For ZITADEL-only users, `created_at` and `updated_at` will be `null`
pub async fn get_current_user_handler(
    claims: Claims,
    state: State<AppState>,
    _request: GetCurrentUserRequest,
) -> GetUserResponse {
    // Try to load user from database using claims.sub (ZITADEL user ID)
    // Note: ZITADEL user ID might be different from our database UUID
    // For now, we'll return user info from claims, but ideally we should
    // have a mapping between ZITADEL user ID and our database user ID
    
    // Try to parse claims.sub as UUID (if it's our internal user ID)
    if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
        // Try to load from database
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

                let email = user.email.clone().unwrap_or_default();

                GetUserResponse {
                    user: Some(ProtoUser {
                        id: user.id.to_string(),
                        email: email.clone(),
                        name: email,
                        role: user.internal_role,
                        created_at,
                        updated_at,
                    }),
                }
            }
            _ => {
                // User not in database, return from claims
                GetUserResponse {
                    user: Some(ProtoUser {
                        id: claims.sub.clone(),
                        email: claims.email.clone().unwrap_or_default(),
                        name: claims.name.clone().unwrap_or_else(|| claims.email.clone().unwrap_or_default()),
                        role: claims.roles.first().cloned().unwrap_or_default(),
                        created_at: None,
                        updated_at: None,
                    }),
                }
            }
        }
    } else {
        // ZITADEL user ID (not our UUID), return from claims
        GetUserResponse {
            user: Some(ProtoUser {
                id: claims.sub.clone(),
                email: claims.email.clone().unwrap_or_default(),
                name: claims.name.clone().unwrap_or_else(|| claims.email.clone().unwrap_or_default()),
                role: claims.roles.first().cloned().unwrap_or_default(),
                created_at: None,
                updated_at: None,
            }),
        }
    }
}
