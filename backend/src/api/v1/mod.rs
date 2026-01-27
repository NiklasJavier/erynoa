//! API v1
//!
//! Version 1 der API mit feature-basierter Struktur

#[cfg(feature = "connect")]
pub mod health;
#[cfg(feature = "connect")]
pub mod info;
#[cfg(feature = "connect")]
pub mod storage;
#[cfg(feature = "connect")]
pub mod users;

#[cfg(feature = "connect")]
pub mod connect_routes;
