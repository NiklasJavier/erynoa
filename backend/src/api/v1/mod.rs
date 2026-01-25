//! API v1
//!
//! Version 1 der API mit feature-basierter Struktur

pub mod health;
pub mod info;
pub mod storage;
pub mod users;

#[cfg(feature = "connect")]
pub mod connect_routes;
