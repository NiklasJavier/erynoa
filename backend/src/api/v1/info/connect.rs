//! Connect-RPC Info Service Implementation

use axum::extract::State;

use crate::config::version::VERSION;
use crate::server::AppState;
use crate::gen::godstack::v1::{
    GetInfoRequest, GetInfoResponse, AuthConfig, UrlConfig, FeatureFlags,
};

/// Get Info Handler
/// 
/// Note: State must come before the request parameter for RpcHandlerUnary
pub async fn get_info_handler(
    state: State<AppState>,
    _request: GetInfoRequest,
) -> GetInfoResponse {
    GetInfoResponse {
        version: VERSION.to_string(),
        environment: state.config.application.environment.as_str().to_string(),
        auth: Some(AuthConfig {
            issuer: state.config.auth.issuer.clone(),
            client_id: state.config.auth.frontend_client_id.clone(),
        }),
        urls: Some(UrlConfig {
            frontend: state.config.application.frontend_url.clone(),
            api: state.config.application.api_url.clone(),
        }),
        features: Some(FeatureFlags {
            registration: false, // TODO: from config
            social_login: false, // TODO: from config
        }),
    }
}
