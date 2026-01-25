//! Configuration Constants
//!
//! Zentrale Konstanten für Service-URLs und Ports
//! Harmonized with frontend/src/lib/service-urls.ts and docs/development/SERVICE_CONFIG.md

/// Service URLs for local development
/// These values should match backend/config/base.toml
pub mod service_urls {
    /// Frontend URL
    pub const FRONTEND: &str = "http://localhost:5173";
    
    /// API URL
    pub const API: &str = "http://localhost:3000";
    
    /// ZITADEL URL
    pub const ZITADEL: &str = "http://localhost:8080";
    
    /// ZITADEL Console URL
    pub const ZITADEL_CONSOLE: &str = "http://localhost:8080/ui/console";
    
    /// MinIO API URL
    pub const MINIO: &str = "http://localhost:9000";
    
    /// MinIO Console URL
    pub const MINIO_CONSOLE: &str = "http://localhost:9001";
    
    /// Database URL (for display purposes)
    pub const DATABASE: &str = "postgresql://localhost:5432";
    
    /// Cache URL (for display purposes)
    pub const CACHE: &str = "redis://localhost:6379";
}

/// Service ports
pub mod ports {
    /// Frontend port
    pub const FRONTEND: u16 = 5173;
    
    /// API port
    pub const API: u16 = 3000;
    
    /// ZITADEL port
    pub const ZITADEL: u16 = 8080;
    
    /// MinIO API port
    pub const MINIO_API: u16 = 9000;
    
    /// MinIO Console port
    pub const MINIO_CONSOLE: u16 = 9001;
    
    /// Database port
    pub const DATABASE: u16 = 5432;
    
    /// Cache port
    pub const CACHE: u16 = 6379;
}
