//! Database Module
//!
//! SQLx PostgreSQL/OrioleDB Integration

mod pool;
mod queries;

pub use pool::DatabasePool;
pub use queries::User;
