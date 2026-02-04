//! Phase 4: Ops & Recovery – Replay, Checkpoints (Debug/Admin)
//!
//! Endpoints für Replay von Events und manuelles Checkpoint.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::core::WrappedStateEvent;
use crate::server::AppState;

// ============================================================================
// Request / Response types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ReplayBody {
    pub from_sequence: u64,
    pub to_sequence: u64,
}

#[derive(Serialize)]
pub struct ReplayResponse {
    pub events_replayed: u64,
    pub from_sequence: u64,
    pub to_sequence: u64,
    pub errors: Vec<String>,
}

#[derive(Serialize)]
pub struct CheckpointResponse {
    pub checkpoint_id: String,
    pub sequence: u64,
    pub event_id: String,
}

// ============================================================================
// Replay
// ============================================================================

/// POST /api/v1/debug/replay – Replay events from_sequence..to_sequence
pub async fn debug_replay_handler(
    State(state): State<AppState>,
    Json(body): Json<ReplayBody>,
) -> impl IntoResponse {
    let from = body.from_sequence;
    let to = body.to_sequence;
    if from >= to {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_range",
                "message": "from_sequence must be less than to_sequence"
            })),
        )
            .into_response();
    }

    let events = state.unified_state.event_log.events_since(0);
    let to_replay: Vec<WrappedStateEvent> = events
        .into_iter()
        .filter(|e| e.sequence >= from && e.sequence < to)
        .collect();
    let count = to_replay.len() as u64;

    state.unified_state.replay_events(&to_replay);

    Json(ReplayResponse {
        events_replayed: count,
        from_sequence: from,
        to_sequence: to,
        errors: Vec::new(),
    })
    .into_response()
}

/// POST /api/v1/debug/replay/checkpoint – Replay from last checkpoint to current sequence
pub async fn debug_replay_checkpoint_handler(State(state): State<AppState>) -> impl IntoResponse {
    let snapshot = state.unified_state.event_log.snapshot();
    let last_checkpoint = snapshot.last_checkpoint_sequence;
    let current = snapshot.sequence;
    if last_checkpoint >= current {
        return Json(ReplayResponse {
            events_replayed: 0,
            from_sequence: last_checkpoint,
            to_sequence: current,
            errors: vec!["no events to replay".to_string()],
        })
        .into_response();
    }

    let events = state.unified_state.event_log.events_since(0);
    let to_replay: Vec<WrappedStateEvent> = events
        .into_iter()
        .filter(|e| e.sequence > last_checkpoint && e.sequence <= current)
        .collect();
    let count = to_replay.len() as u64;

    state.unified_state.replay_events(&to_replay);

    Json(ReplayResponse {
        events_replayed: count,
        from_sequence: last_checkpoint,
        to_sequence: current,
        errors: Vec::new(),
    })
    .into_response()
}

// ============================================================================
// Checkpoint
// ============================================================================

/// POST /api/v1/debug/checkpoint – Manually trigger checkpoint (state.create_checkpoint)
pub async fn debug_checkpoint_handler(State(state): State<AppState>) -> Json<CheckpointResponse> {
    let wrapped = state.unified_state.create_checkpoint();
    let checkpoint_id = format!("ckpt_{}", wrapped.sequence);
    Json(CheckpointResponse {
        checkpoint_id,
        sequence: wrapped.sequence,
        event_id: wrapped.id,
    })
}
