//! Connect-RPC Info Service Implementation

use axum::extract::State;
use axum::http::request::Parts;
use axum_connect::parts::RpcFromRequestParts;
use async_trait::async_trait;

use crate::config::version::VERSION;
use crate::server::AppState;
use crate::api::middleware::FrontendOrigin;
use crate::gen::erynoa::v1::{
    GetInfoRequest, GetInfoResponse, AuthConfig, UrlConfig, FeatureFlags,
};

/// Extract FrontendOrigin from request extensions
#[async_trait]
impl<M, S> RpcFromRequestParts<M, S> for FrontendOrigin
where
    M: prost::Message,
    S: Send + Sync,
{
    type Rejection = axum_connect::error::RpcError;

    async fn rpc_from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<FrontendOrigin>()
            .cloned()
            .ok_or_else(|| {
                axum_connect::error::RpcError::new(
                    axum_connect::error::RpcErrorCode::Internal,
                    "Failed to get frontend origin from request".to_string(),
                )
            })
    }
}

/// Get Info Handler
/// 
/// Returns application configuration including OIDC client ID.
/// Determines the correct client ID based on the request origin/referer header.
pub async fn get_info_handler(
    state: State<AppState>,
    frontend: FrontendOrigin,
    _request: GetInfoRequest,
) -> GetInfoResponse {
    let client_id = frontend.client_id;
    
    GetInfoResponse {
        version: VERSION.to_string(),
        environment: state.config.application.environment.as_str().to_string(),
        auth: Some(AuthConfig {
            issuer: state.config.auth.issuer.clone(),
            client_id,
        }),
        urls: Some(UrlConfig {
            console: state.config.application.console_url.clone(),
            platform: state.config.application.platform_url.clone(),
            docs: state.config.application.docs_url.clone(),
            api: state.config.application.api_url.clone(),
        }),
        features: Some(FeatureFlags {
            registration: state.config.features.registration,
            social_login: state.config.features.social_login,
        }),
    }
}
