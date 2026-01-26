//! Connect-RPC Info Service Implementation

use axum::extract::State;
use axum::http::request::Parts;
use axum_connect::parts::RpcFromRequestParts;
use async_trait::async_trait;

use crate::config::version::VERSION;
use crate::server::AppState;
use crate::gen::erynoa::v1::{
    GetInfoRequest, GetInfoResponse, AuthConfig, UrlConfig, FeatureFlags,
};

/// Frontend identifier extracted from request headers
struct FrontendOrigin {
    client_id: String,
}

#[async_trait]
impl<M, S> RpcFromRequestParts<M, S> for FrontendOrigin
where
    M: prost::Message,
    S: Send + Sync,
{
    type Rejection = axum_connect::error::RpcError;

    async fn rpc_from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let app_state = state.downcast_ref::<AppState>()
            .ok_or_else(|| axum_connect::error::RpcError::internal("Failed to get app state"))?;
        
        // Determine client ID based on origin/referer headers
        let client_id = if let Some(origin) = parts.headers.get("origin") {
            if let Ok(origin_str) = origin.to_str() {
                if origin_str.contains("/platform") || origin_str.contains(":3001/platform") {
                    app_state.config.auth.platform_client_id.clone()
                } else if origin_str.contains("/docs") || origin_str.contains(":3001/docs") {
                    app_state.config.auth.docs_client_id.clone()
                } else if origin_str.contains("/console") || origin_str.contains(":3001/console") {
                    app_state.config.auth.console_client_id.clone()
                } else {
                    app_state.config.auth.console_client_id.clone()
                }
            } else {
                app_state.config.auth.console_client_id.clone()
            }
        } else if let Some(referer) = parts.headers.get("referer") {
            if let Ok(referer_str) = referer.to_str() {
                if referer_str.contains("/platform") {
                    app_state.config.auth.platform_client_id.clone()
                } else if referer_str.contains("/docs") {
                    app_state.config.auth.docs_client_id.clone()
                } else if referer_str.contains("/console") {
                    app_state.config.auth.console_client_id.clone()
                } else {
                    app_state.config.auth.console_client_id.clone()
                }
            } else {
                app_state.config.auth.console_client_id.clone()
            }
        } else {
            app_state.config.auth.console_client_id.clone()
        };
        
        Ok(FrontendOrigin { client_id })
    }
}

/// Get Info Handler
/// 
/// Determines the correct client ID based on the request origin/referer header
/// to return the appropriate OIDC client ID for each frontend.
/// 
/// Note: State must come before the request parameter for RpcHandlerUnary
pub async fn get_info_handler(
    state: State<AppState>,
    frontend: FrontendOrigin,
    _request: GetInfoRequest,
) -> GetInfoResponse {
    GetInfoResponse {
        version: VERSION.to_string(),
        environment: state.config.application.environment.as_str().to_string(),
        auth: Some(AuthConfig {
            issuer: state.config.auth.issuer.clone(),
            client_id: frontend.client_id,
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
