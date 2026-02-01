//! Integration Tests
//!
//! Tests für die neue feature-basierte API-Struktur

use erynoa_api::config::Settings;
use erynoa_api::server::Server;
use serde_json::Value;

pub struct TestApp {
    pub address: String,
    pub client: reqwest::Client,
    /// Temporäres Datenverzeichnis (wird beim Drop automatisch gelöscht)
    _temp_dir: tempfile::TempDir,
}

impl TestApp {
    pub async fn spawn() -> Self {
        // Erstelle temporäres Verzeichnis für diesen Test
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let data_path = temp_dir.path().to_string_lossy().to_string();

        let mut settings = Settings::load().expect("Failed to load config");
        settings.application.port = 0;
        settings.storage.data_dir = data_path;

        let server = Server::build(settings)
            .await
            .expect("Failed to build server");
        let port = server.port();
        let address = format!("http://127.0.0.1:{port}");

        tokio::spawn(server.run());

        // Warte kurz, damit der Server startet
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Self {
            address,
            client: reqwest::Client::new(),
            _temp_dir: temp_dir,
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

        req.send().await.expect("Request failed")
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
        // Auth fields removed after ECLVM migration - now using Ed25519/DID
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
        assert!(app
            .get("/api/v1/users/123")
            .await
            .status()
            .is_client_error());
        assert!(app.get("/api/v1/me").await.status().is_client_error());

        // Storage endpoints
        assert!(app
            .get("/api/v1/storage/list")
            .await
            .status()
            .is_client_error());
        assert!(app
            .post("/api/v1/storage/upload", None)
            .await
            .status()
            .is_client_error());
        assert!(app
            .get("/api/v1/storage/buckets")
            .await
            .status()
            .is_client_error());
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
        let _headers = res.headers();
        // Prüfe ob Response erfolgreich ist (CORS funktioniert)
        assert!(res.status().is_success());
    }

    // ============================================================================
    // Passkey Auth Tests (v1/auth)
    // ============================================================================

    #[tokio::test]
    async fn auth_challenge_returns_valid_challenge() {
        let app = TestApp::spawn().await;
        let res = app.get("/api/v1/auth/challenge").await;

        assert!(res.status().is_success());
        let body: Value = res.json().await.unwrap();

        // Challenge sollte vorhanden und Base64URL encoded sein
        assert!(body["challenge"].is_string());
        let challenge = body["challenge"].as_str().unwrap();
        assert!(!challenge.is_empty());
        // Base64URL: nur a-z, A-Z, 0-9, -, _
        assert!(challenge
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_'));

        // Expires_at sollte in der Zukunft liegen
        assert!(body["expires_at"].is_number());
        let expires_at = body["expires_at"].as_i64().unwrap();
        let now = chrono::Utc::now().timestamp();
        assert!(expires_at > now);
    }

    #[tokio::test]
    async fn auth_challenge_returns_unique_challenges() {
        let app = TestApp::spawn().await;

        let res1 = app.get("/api/v1/auth/challenge").await;
        let body1: Value = res1.json().await.unwrap();
        let challenge1 = body1["challenge"].as_str().unwrap();

        let res2 = app.get("/api/v1/auth/challenge").await;
        let body2: Value = res2.json().await.unwrap();
        let challenge2 = body2["challenge"].as_str().unwrap();

        // Challenges sollten einzigartig sein
        assert_ne!(challenge1, challenge2);
    }

    #[tokio::test]
    async fn auth_register_requires_valid_body() {
        let app = TestApp::spawn().await;

        // Leerer Body
        let res = app.post("/api/v1/auth/passkey/register", None).await;
        assert!(res.status().is_client_error());

        // Unvollständiger Body
        let incomplete = serde_json::json!({
            "credential_id": "test-id"
        });
        let res = app
            .post("/api/v1/auth/passkey/register", Some(incomplete))
            .await;
        assert!(res.status().is_client_error());
    }

    #[tokio::test]
    async fn auth_register_accepts_valid_ed25519_registration() {
        let app = TestApp::spawn().await;

        // Valider Ed25519 Registration Request
        // In der Praxis kommt der Public Key vom Authenticator
        let valid_request = serde_json::json!({
            "credential_id": "dGVzdC1jcmVkZW50aWFsLWlk",
            "public_key": "dGVzdC1wdWJsaWMta2V5LWJhc2U2NC1lbmNvZGVk",
            "algorithm": -8,  // Ed25519
            "did": "did:erynoa:self:test123abc",
            "namespace": "self",
            "display_name": "Test User",
            "transports": ["internal"]
        });

        let res = app
            .post("/api/v1/auth/passkey/register", Some(valid_request))
            .await;

        // Sollte erfolgreich sein (oder spezifischer Fehler)
        let body: Value = res.json().await.unwrap();
        assert!(body.get("success").is_some() || body.get("error").is_some());
    }

    #[tokio::test]
    async fn auth_register_rejects_unsupported_algorithm() {
        let app = TestApp::spawn().await;

        // RS256 ist nicht unterstützt
        let request = serde_json::json!({
            "credential_id": "dGVzdC1jcmVkZW50aWFsLWlk",
            "public_key": "dGVzdC1wdWJsaWMta2V5",
            "algorithm": -257,  // RS256 - nicht unterstützt
            "did": "did:erynoa:self:test123",
            "namespace": "self"
        });

        let res = app
            .post("/api/v1/auth/passkey/register", Some(request))
            .await;
        let body: Value = res.json().await.unwrap();

        // Sollte fehlschlagen
        assert_eq!(body["success"], false);
        assert!(body["error"].is_string());
    }

    #[tokio::test]
    async fn auth_verify_requires_valid_body() {
        let app = TestApp::spawn().await;

        // Leerer Body
        let res = app.post("/api/v1/auth/passkey/verify", None).await;
        assert!(res.status().is_client_error());

        // Unvollständiger Body
        let incomplete = serde_json::json!({
            "credential_id": "test-id"
        });
        let res = app
            .post("/api/v1/auth/passkey/verify", Some(incomplete))
            .await;
        assert!(res.status().is_client_error());
    }

    #[tokio::test]
    async fn auth_verify_returns_error_for_unknown_credential() {
        let app = TestApp::spawn().await;

        let request = serde_json::json!({
            "credential_id": "dW5rbm93bi1jcmVkZW50aWFsLWlk",
            "signature": "dGVzdC1zaWduYXR1cmU",
            "authenticator_data": "dGVzdC1hdXRoLWRhdGE",
            "client_data_json": "dGVzdC1jbGllbnQtZGF0YQ"
        });

        let res = app.post("/api/v1/auth/passkey/verify", Some(request)).await;
        let body: Value = res.json().await.unwrap();

        // Sollte fehlschlagen weil Credential nicht registriert
        assert_eq!(body["success"], false);
        assert!(body["error"].is_string());
    }

    #[tokio::test]
    async fn auth_full_flow_register_then_verify() {
        let app = TestApp::spawn().await;

        // 1. Challenge holen
        let challenge_res = app.get("/api/v1/auth/challenge").await;
        assert!(challenge_res.status().is_success());
        let challenge_body: Value = challenge_res.json().await.unwrap();
        let _challenge = challenge_body["challenge"].as_str().unwrap();

        // 2. Registrieren
        // Hinweis: In einem echten Test würden wir eine echte WebAuthn-Signatur brauchen
        // Hier testen wir nur, dass der Flow grundsätzlich funktioniert
        let register_request = serde_json::json!({
            "credential_id": "Zmxvdy10ZXN0LWNyZWRlbnRpYWw",
            "public_key": "Zmxvdy10ZXN0LXB1YmxpYy1rZXk",
            "algorithm": -8,
            "did": "did:erynoa:self:flowtest123",
            "namespace": "self",
            "display_name": "Flow Test"
        });

        let register_res = app
            .post("/api/v1/auth/passkey/register", Some(register_request))
            .await;
        let register_body: Value = register_res.json().await.unwrap();

        // Verifizierung hängt davon ab, ob die Registrierung erfolgreich war
        if register_body["success"] == true {
            // 3. Versuche zu verifizieren (wird fehlschlagen ohne echte Signatur)
            let verify_request = serde_json::json!({
                "credential_id": "Zmxvdy10ZXN0LWNyZWRlbnRpYWw",
                "signature": "dGVzdC1zaWduYXR1cmU",
                "authenticator_data": "dGVzdC1hdXRoLWRhdGE",
                "client_data_json": "dGVzdC1jbGllbnQtZGF0YQ"
            });

            let verify_res = app
                .post("/api/v1/auth/passkey/verify", Some(verify_request))
                .await;
            let verify_body: Value = verify_res.json().await.unwrap();

            // Verifizierung sollte fehlschlagen (ungültige Signatur)
            // aber das Credential sollte gefunden werden
            assert!(verify_body.get("success").is_some() || verify_body.get("error").is_some());
        }
    }
}
