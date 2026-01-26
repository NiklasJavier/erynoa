//! API Middleware
//!
//! Zentrale Middleware-Komponenten für Request-Verarbeitung

mod auth;
mod cors;
mod error_handler;
mod frontend_origin;
mod logging;

// auth_middleware is currently unused (auth is handled via Claims extractor)
// pub use auth::auth_middleware;
pub use cors::build_cors;
// error_handler is a placeholder for future global error handling
// pub use error_handler::error_handler;
pub use frontend_origin::{frontend_origin_middleware, FrontendOrigin};
pub use logging::logging_middleware;
