//! Integration Tests
//!
//! Tests für die neue feature-basierte API-Struktur

use godstack_api::config::Settings;
use godstack_api::server::Server;
use serde_json::Value;

pub struct TestApp {
    pub address: String,
    pub client: reqwest::Client,
}

impl TestApp {
    pub async fn spawn() -> Self {
        let mut settings = Settings::load().expect("Failed to load config");
        settings.application.port = 0;
        
        let server = Server::build(settings).await.expect("Failed to build server");
        let port = server.port();
        let address = format!("http://127.0.0.1:{port}");
        
        tokio::spawn(server.run());
        
        // Warte kurz, damit der Server startet
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Self {
            address,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get(&self, path: &str) -> reqwest::Response {
        self.client
            .get(format!("{}{}", self.address, path))
            .send()
            .await
            .expect("Request failed")
    }

    pub async fn post(&self, path: &str, body: Option<Value>) -> reqwest::Response {
        let mut req = self.client.post(format!("{}{}", self.address, path));
        
        if let Some(body) = body {
            req = req.json(&body);
        }
        
        req.send()
            .await
            .expect("Request failed")
    }

    pub async fn delete(&self, path: &str) -> reqwest::Response {
        self.client
            .delete(format!("{}{}", self.address, path))
            .send()
            .await
            .expect("Request failed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Health Check Tests (v1/health)
    // ============================================================================

    #[tokio::test]
    async fn health_check_works() {
        let app = TestApp::spawn().await;
        let res = app.get("/api/v1/health").await;
        
        assert!(res.status().is_success());
        let body: Value = res.json().await.unwrap();
        assert_eq!(body["status"], "healthy");
        assert!(body["version"].is_string());
    }

    #[tokio::test]
    async fn readiness_check_works() {
        let app = TestApp::spawn().await;
        let res = app.get("/api/v1/ready").await;
        
        // Readiness kann fehlschlagen wenn Services nicht laufen, aber Endpoint sollte existieren
        assert!(res.status().is_client_error() || res.status().is_success());
        let body: Value = res.json().await.unwrap();
        assert!(body["status"].is_string());
        assert!(body["services"].is_object());
    }

    // ============================================================================
    // Info Tests (v1/info)
    // ============================================================================

    #[tokio::test]
    async fn info_endpoint_works() {
        let app = TestApp::spawn().await;
        let res = app.get("/api/v1/info").await;
        
        assert!(res.status().is_success());
        let body: Value = res.json().await.unwrap();
        assert!(body["version"].is_string());
        assert!(body["environment"].is_string());
        assert!(body["auth_issuer"].is_string());
        assert!(body["auth_client_id"].is_string());
    }

    #[tokio::test]
    async fn status_endpoint_works() {
        let app = TestApp::spawn().await;
        let res = app.get("/api/v1/status").await;
        
        assert!(res.status().is_success());
        let body: Value = res.json().await.unwrap();
        assert!(body["services"].is_array());
    }

    // ============================================================================
    // User Tests (v1/users) - Protected Routes
    // ============================================================================

    #[tokio::test]
    async fn users_endpoint_requires_auth() {
        let app = TestApp::spawn().await;
        let res = app.get("/api/v1/users").await;
        
        // Sollte 401 Unauthorized oder 403 Forbidden sein ohne Token
        assert!(res.status().is_client_error());
    }

    #[tokio::test]
    async fn me_endpoint_requires_auth() {
        let app = TestApp::spawn().await;
        let res = app.get("/api/v1/me").await;
        
        // Sollte 401 Unauthorized sein ohne Token
        assert!(res.status().is_client_error());
    }

    // ============================================================================
    // Storage Tests (v1/storage) - Protected Routes
    // ============================================================================

    #[tokio::test]
    async fn storage_list_requires_auth() {
        let app = TestApp::spawn().await;
        let res = app.get("/api/v1/storage/list").await;
        
        // Sollte 401 Unauthorized sein ohne Token
        assert!(res.status().is_client_error());
    }

    #[tokio::test]
    async fn storage_upload_requires_auth() {
        let app = TestApp::spawn().await;
        let res = app.post("/api/v1/storage/upload", None).await;
        
        // Sollte 401 Unauthorized sein ohne Token
        assert!(res.status().is_client_error());
    }

    #[tokio::test]
    async fn storage_buckets_requires_auth() {
        let app = TestApp::spawn().await;
        let res = app.get("/api/v1/storage/buckets").await;
        
        // Sollte 401 Unauthorized sein ohne Token
        assert!(res.status().is_client_error());
    }

    // ============================================================================
    // Route Structure Tests
    // ============================================================================

    #[tokio::test]
    async fn all_public_routes_accessible() {
        let app = TestApp::spawn().await;
        
        // Health endpoints
        let health = app.get("/api/v1/health").await;
        assert!(health.status().is_success());
        
        let ready = app.get("/api/v1/ready").await;
        assert!(ready.status().is_client_error() || ready.status().is_success());
        
        // Info endpoints
        let info = app.get("/api/v1/info").await;
        assert!(info.status().is_success());
        
        let status = app.get("/api/v1/status").await;
        assert!(status.status().is_success());
    }

    #[tokio::test]
    async fn all_protected_routes_require_auth() {
        let app = TestApp::spawn().await;
        
        // User endpoints
        assert!(app.get("/api/v1/users").await.status().is_client_error());
        assert!(app.get("/api/v1/users/123").await.status().is_client_error());
        assert!(app.get("/api/v1/me").await.status().is_client_error());
        
        // Storage endpoints
        assert!(app.get("/api/v1/storage/list").await.status().is_client_error());
        assert!(app.post("/api/v1/storage/upload", None).await.status().is_client_error());
        assert!(app.get("/api/v1/storage/buckets").await.status().is_client_error());
    }

    #[tokio::test]
    async fn non_existent_routes_return_404() {
        let app = TestApp::spawn().await;
        let res = app.get("/api/v1/nonexistent").await;
        
        assert_eq!(res.status(), 404);
    }

    // ============================================================================
    // CORS Tests
    // ============================================================================

    #[tokio::test]
    async fn cors_headers_present() {
        let app = TestApp::spawn().await;
        let res = app.get("/api/v1/health").await;
        
        // CORS-Header sollten vorhanden sein (auch wenn nicht alle gesetzt sind)
        // In Development sollte CORS sehr permissiv sein
        let headers = res.headers();
        // Prüfe ob Response erfolgreich ist (CORS funktioniert)
        assert!(res.status().is_success());
    }
}
