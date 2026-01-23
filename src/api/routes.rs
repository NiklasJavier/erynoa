//! API Routes

use crate::server::AppState;
use axum::{
    http::{header, HeaderValue, Method},
    routing::get,
    Router,
};
use std::time::Duration;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use super::handlers;

pub fn create_router(state: AppState) -> Router {
    let cors = build_cors(&state);

    // All routes - auth is handled via Claims extractor
    let api = Router::new()
        // Public
        .route("/health", get(handlers::health::health_check))
        .route("/ready", get(handlers::health::readiness_check))
        .route("/info", get(handlers::info::get_info))
        // Protected (Claims extractor validates JWT)
        .route("/me", get(handlers::users::get_current_user))
        .route("/users", get(handlers::users::list_users))
        .route("/users/:id", get(handlers::users::get_user));

    Router::new()
        .nest("/api/v1", api)
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(CompressionLayer::new())
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &axum::http::Request<_>| {
                    let id = req
                        .headers()
                        .get("x-request-id")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("-");
                    tracing::info_span!("http", method = %req.method(), uri = %req.uri(), id)
                })
                .on_response(|res: &axum::http::Response<_>, latency: Duration, _: &tracing::Span| {
                    tracing::info!(status = %res.status(), ms = latency.as_millis(), "response");
                }),
        )
        .with_state(state)
}

fn build_cors(state: &AppState) -> CorsLayer {
    if state.config.application.environment.is_production() {
        let origin = state.config.application.frontend_url.clone();
        CorsLayer::new()
            .allow_origin(origin.parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
            .allow_credentials(true)
            .max_age(Duration::from_secs(86400))
    } else {
        CorsLayer::very_permissive()
    }
}
