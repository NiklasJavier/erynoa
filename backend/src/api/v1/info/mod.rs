//! Info API
//!
//! Public configuration endpoint for control (Connect-RPC only)

mod handlers;

#[cfg(feature = "connect")]
pub use handlers::get_info_handler;
