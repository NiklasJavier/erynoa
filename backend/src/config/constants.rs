//! Configuration Constants
//!
//! Zentrale Konstanten für Service-URLs und Ports
//! Dezentrale Architektur - keine externen Services (DB, Cache, Auth, S3)

/// Service URLs for local development
pub mod service_urls {
    /// Console URL
    pub const CONSOLE: &str = "http://localhost:5173";

    /// Platform URL
    pub const PLATFORM: &str = "http://localhost:5174";

    /// Docs URL
    pub const DOCS: &str = "http://localhost:5175";

    /// API URL
    pub const API: &str = "http://localhost:3000";
}

/// Service ports
pub mod ports {
    /// Console port
    pub const CONSOLE: u16 = 5173;

    /// Platform port
    pub const PLATFORM: u16 = 5174;

    /// Docs port
    pub const DOCS: u16 = 5175;

    /// API port
    pub const API: u16 = 3000;
}

/// Storage defaults
pub mod storage {
    /// Default data directory
    pub const DEFAULT_DATA_DIR: &str = "./data";

    /// Default max content size (100 MB)
    pub const DEFAULT_MAX_CONTENT_SIZE: u64 = 104_857_600;
}
