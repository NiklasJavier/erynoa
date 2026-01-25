//! Users API
//!
//! User management endpoints (Connect-RPC only)

mod handlers;

#[cfg(feature = "connect")]
pub use handlers::{list_users_handler, get_user_handler, get_current_user_handler};
