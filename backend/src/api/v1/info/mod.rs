//! Info API
//!
//! Public configuration endpoint for frontend

mod handler;
mod models;
mod routes;
#[cfg(feature = "connect")]
mod connect;

pub use routes::create_info_routes;
#[cfg(feature = "connect")]
pub use connect::get_info_handler;
