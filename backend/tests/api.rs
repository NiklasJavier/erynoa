//! Integration Tests

use godstack_api::config::Settings;
use godstack_api::server::Server;

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_check_works() {
        let app = TestApp::spawn().await;
        let res = app.get("/api/v1/health").await;
        
        assert!(res.status().is_success());
        let body: serde_json::Value = res.json().await.unwrap();
        assert_eq!(body["status"], "healthy");
    }
}
