//! Static File Serving
//!
//! Dient statische Frontend-Dateien (Console, Docs, Platform) unter
//! den entsprechenden Pfaden: /console, /docs, /platform
//!
//! Die Frontends werden als SPA (Single Page Applications) mit fallback
//! auf index.html behandelt.

use axum::Router;
use std::path::PathBuf;
use tower_http::services::{ServeDir, ServeFile};

/// Konfiguration für Static File Serving
#[derive(Debug, Clone)]
pub struct StaticConfig {
    /// Basis-Verzeichnis für statische Dateien
    pub static_dir: PathBuf,
    /// Cache-Control max-age für immutable assets (hashed files)
    pub immutable_max_age: u32,
    /// Cache-Control max-age für index.html (muss revalidiert werden)
    pub html_max_age: u32,
}

impl Default for StaticConfig {
    fn default() -> Self {
        Self {
            static_dir: PathBuf::from("./static"),
            immutable_max_age: 31536000, // 1 Jahr für hashed assets
            html_max_age: 0,             // Immer revalidieren für index.html
        }
    }
}

impl StaticConfig {
    /// Erstellt Config aus einem Pfad
    pub fn new(static_dir: impl Into<PathBuf>) -> Self {
        Self {
            static_dir: static_dir.into(),
            ..Default::default()
        }
    }

    /// Prüft ob das Static-Verzeichnis existiert
    pub fn is_available(&self) -> bool {
        self.static_dir.exists()
    }

    /// Pfad zu einem Frontend
    pub fn frontend_path(&self, name: &str) -> PathBuf {
        self.static_dir.join(name)
    }
}

/// Erstellt den Router für statische Frontend-Dateien
///
/// Struktur:
/// - /console/* → static/console/
/// - /docs/* → static/docs/
/// - /platform/* → static/platform/
///
/// Jedes Frontend ist eine SPA mit fallback auf index.html
pub fn create_static_router(config: &StaticConfig) -> Router {
    let mut router = Router::new();

    // Prüfe ob Static-Verzeichnis existiert
    if !config.is_available() {
        tracing::warn!(
            path = %config.static_dir.display(),
            "Static directory not found - frontend routes disabled"
        );
        return router;
    }

    // Console Frontend
    let console_path = config.frontend_path("console");
    if console_path.exists() {
        let console_index = console_path.join("index.html");
        let service =
            ServeDir::new(&console_path).not_found_service(ServeFile::new(&console_index));

        router = router.nest_service("/console", service);
        tracing::info!(path = %console_path.display(), "✅ Console frontend mounted at /console");
    } else {
        tracing::debug!(path = %console_path.display(), "Console frontend not found");
    }

    // Docs Frontend
    let docs_path = config.frontend_path("docs");
    if docs_path.exists() {
        let docs_index = docs_path.join("index.html");
        let service = ServeDir::new(&docs_path).not_found_service(ServeFile::new(&docs_index));

        router = router.nest_service("/docs", service);
        tracing::info!(path = %docs_path.display(), "✅ Docs frontend mounted at /docs");
    } else {
        tracing::debug!(path = %docs_path.display(), "Docs frontend not found");
    }

    // Platform Frontend
    let platform_path = config.frontend_path("platform");
    if platform_path.exists() {
        let platform_index = platform_path.join("index.html");
        let service =
            ServeDir::new(&platform_path).not_found_service(ServeFile::new(&platform_index));

        router = router.nest_service("/platform", service);
        tracing::info!(path = %platform_path.display(), "✅ Platform frontend mounted at /platform");
    } else {
        tracing::debug!(path = %platform_path.display(), "Platform frontend not found");
    }

    router
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_config_default() {
        let config = StaticConfig::default();
        assert_eq!(config.immutable_max_age, 31536000);
        assert_eq!(config.html_max_age, 0);
    }

    #[test]
    fn test_frontend_path() {
        let config = StaticConfig::new("/app/static");
        assert_eq!(
            config.frontend_path("console"),
            PathBuf::from("/app/static/console")
        );
        assert_eq!(
            config.frontend_path("docs"),
            PathBuf::from("/app/static/docs")
        );
    }
}
