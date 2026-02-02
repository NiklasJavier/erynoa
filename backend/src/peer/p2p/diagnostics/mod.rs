//! # P2P Diagnostics Module - Real-Time Monitoring
//!
//! Umfassende Echtzeit-Diagnose-Tools für alle P2P-Schichten.
//!
//! ## Features
//!
//! - **Live-Metriken**: Bytes, Messages, Latenz, Peers
//! - **Event-Stream**: SSE für alle P2P-Events
//! - **Dashboard**: HTML-Interface für Browser
//! - **Layer-Diagnostics**: 8-Schichten-Analyse
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                    P2P DIAGNOSTICS PORTAL                           │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │                                                                     │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
//! │  │   METRICS    │  │   EVENTS     │  │   LAYERS     │              │
//! │  │   COLLECTOR  │  │   STREAM     │  │   CHECKER    │              │
//! │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘              │
//! │         │                 │                 │                       │
//! │  ┌──────┴─────────────────┴─────────────────┴──────┐               │
//! │  │              DIAGNOSTIC STATE                    │               │
//! │  │  • Thread-safe RwLock<DiagnosticState>          │               │
//! │  │  • Atomic counters for high-frequency updates   │               │
//! │  │  • Event buffer with ring-buffer semantics      │               │
//! │  └──────────────────────────────────────────────────┘               │
//! │                            │                                        │
//! │  ┌────────────────────────┴────────────────────────┐               │
//! │  │              API ENDPOINTS                       │               │
//! │  │  /diagnostics          - JSON Summary           │               │
//! │  │  /diagnostics/report   - ASCII Report           │               │
//! │  │  /diagnostics/stream   - SSE Live Updates       │               │
//! │  │  /diagnostics/metrics  - Detailed Metrics       │               │
//! │  │  /diagnostics/events   - Event Log              │               │
//! │  │  /diagnostics/dashboard - HTML Dashboard        │               │
//! │  └──────────────────────────────────────────────────┘               │
//! │                                                                     │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

mod dashboard;
mod events;
mod layers;
mod metrics;
mod state;
mod swarm_state;
mod types;

pub use dashboard::*;
pub use events::*;
pub use layers::*;
pub use metrics::*;
pub use state::*;
pub use swarm_state::*;
pub use types::*;

// ============================================================================
// CONVENIENCE RE-EXPORTS
// ============================================================================

/// Erstellt einen vollständig konfigurierten DiagnosticState
pub fn create_diagnostic_state(peer_id: String) -> std::sync::Arc<DiagnosticState> {
    std::sync::Arc::new(DiagnosticState::new(peer_id))
}

/// Erstellt Axum-Router für alle Diagnostic-Endpoints
#[cfg(feature = "p2p")]
pub fn diagnostic_routes(state: std::sync::Arc<DiagnosticState>) -> axum::Router {
    use axum::routing::get;

    axum::Router::new()
        // JSON API
        .route("/diagnostics", get(handlers::get_diagnostics))
        .route("/diagnostics/report", get(handlers::get_report))
        .route("/diagnostics/metrics", get(handlers::get_metrics))
        .route("/diagnostics/events", get(handlers::get_events))
        .route("/diagnostics/layers", get(handlers::get_layers))
        .route("/diagnostics/layers/:layer", get(handlers::get_layer))
        // Real-time
        .route("/diagnostics/stream", get(handlers::event_stream))
        // Dashboard
        .route("/diagnostics/dashboard", get(handlers::dashboard_html))
        .with_state(state)
}

// ============================================================================
// AXUM HANDLERS
// ============================================================================

#[cfg(feature = "p2p")]
pub mod handlers {
    use super::*;
    use axum::{
        extract::{Path, State},
        response::{
            sse::{Event, KeepAlive, Sse},
            Html, IntoResponse,
        },
        Json,
    };
    use futures::stream::Stream;
    use std::convert::Infallible;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio_stream::wrappers::IntervalStream;
    use tokio_stream::StreamExt;

    /// GET /diagnostics - JSON Summary
    pub async fn get_diagnostics(
        State(state): State<Arc<DiagnosticState>>,
    ) -> Json<P2PDiagnostics> {
        let runner = DiagnosticRunner::from_state(&state);
        let diagnostics = runner.run_all(Some(state.peer_id.clone())).await;
        Json(diagnostics)
    }

    /// GET /diagnostics/report - ASCII Report
    pub async fn get_report(State(state): State<Arc<DiagnosticState>>) -> String {
        let runner = DiagnosticRunner::from_state(&state);
        let diagnostics = runner.run_all(Some(state.peer_id.clone())).await;
        diagnostics.to_cli_report()
    }

    /// GET /diagnostics/metrics - Detailed Metrics
    pub async fn get_metrics(State(state): State<Arc<DiagnosticState>>) -> Json<NetworkMetrics> {
        Json(state.get_metrics())
    }

    /// GET /diagnostics/events - Event Log
    pub async fn get_events(
        State(state): State<Arc<DiagnosticState>>,
    ) -> Json<Vec<DiagnosticEvent>> {
        Json(state.get_recent_events(100))
    }

    /// GET /diagnostics/layers - All Layer Status
    pub async fn get_layers(
        State(state): State<Arc<DiagnosticState>>,
    ) -> Json<Vec<LayerDiagnostic>> {
        let runner = DiagnosticRunner::from_state(&state);
        let diagnostics = runner.run_all(Some(state.peer_id.clone())).await;
        Json(diagnostics.layers)
    }

    /// GET /diagnostics/layers/:layer - Single Layer
    pub async fn get_layer(
        State(state): State<Arc<DiagnosticState>>,
        Path(layer): Path<u8>,
    ) -> impl IntoResponse {
        let runner = DiagnosticRunner::from_state(&state);
        let diagnostics = runner.run_all(Some(state.peer_id.clone())).await;

        if let Some(l) = diagnostics.layers.iter().find(|l| l.layer_number == layer) {
            Json(l.clone()).into_response()
        } else {
            (axum::http::StatusCode::NOT_FOUND, "Layer not found").into_response()
        }
    }

    /// GET /diagnostics/stream - Server-Sent Events
    pub async fn event_stream(
        State(state): State<Arc<DiagnosticState>>,
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        let interval = tokio::time::interval(Duration::from_millis(500));
        let stream = IntervalStream::new(interval);

        let state_clone = state.clone();
        let sse_stream = stream.map(move |_| {
            let snapshot = StreamSnapshot {
                timestamp: chrono::Utc::now().to_rfc3339(),
                metrics: state_clone.get_metrics(),
                peer_count: state_clone
                    .metrics
                    .connected_peers
                    .load(std::sync::atomic::Ordering::Relaxed),
                recent_events: state_clone.get_recent_events(5),
                health: state_clone.get_health_status(),
            };

            let json = serde_json::to_string(&snapshot).unwrap_or_default();
            Ok(Event::default().data(json))
        });

        Sse::new(sse_stream).keep_alive(KeepAlive::default())
    }

    /// GET /diagnostics/dashboard - HTML Dashboard
    pub async fn dashboard_html(State(state): State<Arc<DiagnosticState>>) -> Html<String> {
        Html(generate_dashboard_html(&state.peer_id))
    }
}
