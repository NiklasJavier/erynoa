//! API Middleware
//!
//! Zentrale Middleware-Komponenten für Request-Verarbeitung

mod auth;
mod cors;
mod frontend_origin;
mod logging;

// auth_middleware is currently unused (auth is handled via Claims extractor)
// pub use auth::auth_middleware;
pub use cors::build_cors;
pub use frontend_origin::frontend_origin_middleware;
// FrontendOrigin is re-exported for use in handlers
#[allow(unused_imports)]
pub use frontend_origin::FrontendOrigin;
pub use logging::logging_middleware;
