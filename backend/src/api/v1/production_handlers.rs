//! Phase 2 & 3: Produktion Kern – Crossing, ECL, Trust, Identity, Realm, Governance, Controller, Intent, Saga
//!
//! Endpoints für Crossing/validate, ECL (stubs), Trust, Identity, Realm (Phase 2);
//! Governance (proposals, vote), Controller (check, permissions), Intent parse, Saga (compose, execute, stats) (Phase 3).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::core::{
    ControllerSnapshot, GatewaySnapshot, GovernanceSnapshot, RealmSnapshot, SagaComposerSnapshot,
    StateEvent, TrustReason,
};
use crate::core::state::IdentitySnapshot;
use crate::domain::{realm_id_from_name, DID};
use crate::server::AppState;

// ============================================================================
// Request / Response types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CrossingValidateBody {
    pub caller_did: String,
    pub from_realm: String,
    pub to_realm: String,
}

#[derive(Serialize)]
pub struct CrossingValidateResponse {
    pub allowed: bool,
    pub from_realm: String,
    pub to_realm: String,
    pub reason: Option<String>,
    pub trust_score: Option<f64>,
    pub violations: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TrustUpdateBody {
    pub entity_id: String,
    pub delta: f64,
    pub reason: TrustReason,
    #[serde(default)]
    pub from_realm: Option<String>,
}

#[derive(Serialize)]
pub struct TrustGetResponse {
    pub did: String,
    pub trust: Option<f64>,
}

#[derive(Serialize)]
pub struct TrustUpdateResponse {
    pub sequence: u64,
    pub component: String,
    pub event_id: String,
}

#[derive(Serialize)]
pub struct IdentityRootResponse {
    pub root_did: Option<String>,
    pub snapshot: IdentitySnapshot,
}

#[derive(Debug, Deserialize)]
pub struct RealmCreateBody {
    pub realm_id: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RealmMembersBody {
    pub identity_id: String,
    pub action: String, // "join" | "leave" | "invite" etc.
}

#[derive(Serialize)]
pub struct EclNotConfiguredResponse {
    pub error: &'static str,
    pub message: &'static str,
}

// Phase 3: Governance, Controller, Intent, Saga
#[derive(Debug, Deserialize)]
pub struct GovernanceProposalBody {
    pub proposal_id: String,
    pub realm_id: String,
    pub proposer_id: String,
    pub proposal_type: String,
    #[serde(default)]
    pub deadline_ms: Option<u128>,
}

#[derive(Debug, Deserialize)]
pub struct GovernanceVoteBody {
    pub voter_id: String,
    pub vote: bool,
    #[serde(default)]
    pub weight: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ControllerCheckBody {
    pub permission: String,
    pub resource: String,
    pub caller_did: String,
    pub realm_id: String,
}

#[derive(Serialize)]
pub struct ControllerCheckResponse {
    pub allowed: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IntentParseBody {
    pub text: Option<String>,
    pub goal: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SagaComposeBody {
    pub goal_type: Option<String>,
    pub constraints: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct SagaExecuteBody {
    pub saga_id: String,
    pub step: Option<usize>,
    pub total_steps: Option<usize>,
    pub cross_realm: Option<bool>,
    pub compensation_triggered: Option<bool>,
    pub realms: Option<Vec<String>>,
}

// ============================================================================
// Crossing & Gateway
// ============================================================================

/// POST /api/v1/crossing/validate – Validate realm crossing (GatewayGuard)
pub async fn crossing_validate_handler(
    State(state): State<AppState>,
    Json(body): Json<CrossingValidateBody>,
) -> impl IntoResponse {
    let Some(ref guard) = state.gateway else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "gateway_not_configured",
                "message": "GatewayGuard not wired; configure AppState.gateway to enable crossing validation"
            })),
        )
            .into_response();
    };

    let did = match DID::parse(&body.caller_did) {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid_did", "message": e.to_string() })),
            )
                .into_response();
        }
    };

    let from_realm = realm_id_from_name(&body.from_realm);
    let to_realm = realm_id_from_name(&body.to_realm);

    match guard.validate_crossing(&did, &from_realm, &to_realm) {
        Ok(result) => {
            let trust_score = Some(result.original_trust.weighted_norm(&[1.0_f32; 6]) as f64);
            Json(CrossingValidateResponse {
                allowed: result.allowed,
                from_realm: body.from_realm.clone(),
                to_realm: body.to_realm.clone(),
                reason: result.violations.first().cloned(),
                trust_score,
                violations: result.violations,
            })
            .into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "crossing_validation_failed", "message": e.to_string() })),
        )
            .into_response(),
    }
}

/// GET /api/v1/crossing/stats – GatewaySnapshot (crossings_total, allowed, denied, …)
pub async fn crossing_stats_handler(State(state): State<AppState>) -> Json<GatewaySnapshot> {
    let snapshot = state.unified_state.snapshot();
    Json(snapshot.peer.gateway)
}

// ============================================================================
// Trust
// ============================================================================

/// GET /api/v1/trust/:did – Trust value or TrustEntry for DID
pub async fn trust_get_handler(
    State(state): State<AppState>,
    Path(did_str): Path<String>,
) -> impl IntoResponse {
    let did = match DID::parse(&did_str) {
        Ok(d) => d,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid_did" })),
            )
                .into_response();
        }
    };

    let trust = state.unified_state.core.trust.get_trust(&did.id);
    Json(TrustGetResponse {
        did: did_str,
        trust,
    })
    .into_response()
}

/// POST /api/v1/trust/update – Emit TrustUpdate event via log_and_apply
pub async fn trust_update_handler(
    State(state): State<AppState>,
    Json(body): Json<TrustUpdateBody>,
) -> impl IntoResponse {
    let did = match DID::parse(&body.entity_id) {
        Ok(d) => d,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid_entity_id", "message": "entity_id must be a valid DID" })),
            )
                .into_response();
        }
    };

    let current = state.unified_state.core.trust.get_trust(&did.id).unwrap_or(0.0);
    let new_trust = (current + body.delta).clamp(0.0, 1.0);

    let event = StateEvent::TrustUpdate {
        entity_id: body.entity_id,
        delta: body.delta,
        reason: body.reason,
        from_realm: body.from_realm,
        triggered_events: 0,
        new_trust,
    };

    let wrapped = state.unified_state.log_and_apply(event, vec![]);
    Json(TrustUpdateResponse {
        sequence: wrapped.sequence,
        component: format!("{:?}", wrapped.component),
        event_id: wrapped.id,
    })
    .into_response()
}

// ============================================================================
// Identity
// ============================================================================

/// GET /api/v1/identity/root – Root DID and identity snapshot
pub async fn identity_root_handler(State(state): State<AppState>) -> Json<IdentityRootResponse> {
    let snapshot = state.unified_state.snapshot();
    Json(IdentityRootResponse {
        root_did: snapshot.identity.root_did.clone(),
        snapshot: snapshot.identity,
    })
}

/// GET /api/v1/identity/:did – Identity info for DID (from snapshot/state)
pub async fn identity_get_handler(
    State(state): State<AppState>,
    Path(did_str): Path<String>,
) -> impl IntoResponse {
    let snapshot = state.unified_state.snapshot();
    if let Some(ref root) = snapshot.identity.root_did {
        if root == &did_str {
            return Json(serde_json::json!({
                "did": did_str,
                "is_root": true,
                "snapshot": snapshot.identity
            }))
            .into_response();
        }
    }
    Json(serde_json::json!({
        "did": did_str,
        "is_root": false,
        "root_did": snapshot.identity.root_did
    }))
    .into_response()
}

// ============================================================================
// Realm
// ============================================================================

/// GET /api/v1/realms – Realm list (from snapshot.peer.realm)
pub async fn realms_list_handler(State(state): State<AppState>) -> Json<RealmSnapshot> {
    let snapshot = state.unified_state.snapshot();
    Json(snapshot.peer.realm)
}

/// GET /api/v1/realms/:realm_id – Single realm + rules, members, ECL info
pub async fn realm_get_handler(
    State(state): State<AppState>,
    Path(realm_id): Path<String>,
) -> impl IntoResponse {
    let snapshot = state.unified_state.snapshot();
    let realm_snap = snapshot.peer.realm.realms.get(&realm_id).cloned();
    match realm_snap {
        Some(rs) => Json(serde_json::json!({
            "realm_id": realm_id,
            "snapshot": rs,
            "total_realms": snapshot.peer.realm.total_realms
        }))
        .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "realm_not_found", "realm_id": realm_id })),
        )
            .into_response(),
    }
}

/// POST /api/v1/realms – Create realm (StateEvent::RealmLifecycle)
pub async fn realm_create_handler(
    State(state): State<AppState>,
    Json(body): Json<RealmCreateBody>,
) -> impl IntoResponse {
    use crate::domain::RealmAction;

    let event = StateEvent::RealmLifecycle {
        realm_id: body.realm_id.clone(),
        action: RealmAction::Created,
        config: None,
    };
    let wrapped = state.unified_state.log_and_apply(event, vec![]);
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "realm_id": body.realm_id,
            "sequence": wrapped.sequence,
            "event_id": wrapped.id
        })),
    )
        .into_response()
}

/// POST /api/v1/realms/:realm_id/members – Membership change (StateEvent::MembershipChange)
pub async fn realm_members_handler(
    State(state): State<AppState>,
    Path(realm_id): Path<String>,
    Json(body): Json<RealmMembersBody>,
) -> impl IntoResponse {
    use crate::domain::MembershipAction;

    let action = match body.action.to_lowercase().as_str() {
        "join" => MembershipAction::Joined,
        "leave" => MembershipAction::Left,
        "invite" => MembershipAction::Invited,
        "ban" => MembershipAction::Banned,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid_action", "action": body.action })),
            )
                .into_response();
        }
    };

    let event = StateEvent::MembershipChange {
        realm_id: realm_id.clone(),
        identity_id: body.identity_id.clone(),
        identity_universal_id: None,
        action,
        new_role: None,
        initiated_by: None,
        initiated_by_id: None,
    };
    let wrapped = state.unified_state.log_and_apply(event, vec![]);
    Json(serde_json::json!({
        "realm_id": realm_id,
        "identity_id": body.identity_id,
        "sequence": wrapped.sequence,
        "event_id": wrapped.id
    }))
    .into_response()
}

/// GET /api/v1/realms/:realm_id/ecl – RealmECLSnapshot for realm
pub async fn realm_ecl_handler(
    State(state): State<AppState>,
    Path(realm_id): Path<String>,
) -> impl IntoResponse {
    let snapshot = state.unified_state.snapshot();
    let realm_ecl = snapshot.eclvm.realm_ecl.get(&realm_id).cloned();
    match realm_ecl {
        Some(re) => Json(re).into_response(),
        None => Json(serde_json::json!({
            "realm_id": realm_id,
            "policies_executed": 0,
            "policies_passed": 0,
            "policies_denied": 0,
            "success_rate": 0.0,
            "gas_consumed": 0
        }))
        .into_response(),
    }
}

// ============================================================================
// ECL Entrypoints (stubs – return 501 until EclEntrypoints wired)
// ============================================================================

/// POST /api/v1/ecl/run – Run policy (stub)
pub async fn ecl_run_handler() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(EclNotConfiguredResponse {
            error: "ecl_not_configured",
            message: "ECL run_policy not wired; add EclEntrypoints to AppState to enable",
        }),
    )
}

/// POST /api/v1/ecl/api/:route_id – Run API handler (stub)
pub async fn ecl_api_handler(Path(_route_id): Path<String>) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(EclNotConfiguredResponse {
            error: "ecl_not_configured",
            message: "ECL run_api not wired; add EclEntrypoints to AppState to enable",
        }),
    )
}

/// POST /api/v1/ecl/ui/:component_id – Run UI handler (stub)
pub async fn ecl_ui_handler(Path(_component_id): Path<String>) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(EclNotConfiguredResponse {
            error: "ecl_not_configured",
            message: "ECL run_ui not wired; add EclEntrypoints to AppState to enable",
        }),
    )
}

/// POST /api/v1/ecl/controller/:permission_or_resource – Run controller (stub)
pub async fn ecl_controller_handler(Path(_key): Path<String>) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(EclNotConfiguredResponse {
            error: "ecl_not_configured",
            message: "ECL run_controller not wired; add EclEntrypoints to AppState to enable",
        }),
    )
}

// ============================================================================
// Phase 3: Governance & Controller
// ============================================================================

/// POST /api/v1/governance/proposals – Create proposal (StateEvent::ProposalCreated)
pub async fn governance_proposals_create_handler(
    State(state): State<AppState>,
    Json(body): Json<GovernanceProposalBody>,
) -> impl IntoResponse {
    let proposal_id = body.proposal_id.clone();
    let deadline_ms = body.deadline_ms.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
            + 86_400_000 // +24h default
    });
    let event = StateEvent::ProposalCreated {
        proposal_id: body.proposal_id,
        realm_id: body.realm_id,
        proposer_id: body.proposer_id,
        proposal_type: body.proposal_type,
        deadline_ms,
    };
    let wrapped = state.unified_state.log_and_apply(event, vec![]);
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "proposal_id": proposal_id,
            "sequence": wrapped.sequence,
            "event_id": wrapped.id
        })),
    )
        .into_response()
}

/// POST /api/v1/governance/proposals/:id/vote – Cast vote (StateEvent::VoteCast)
pub async fn governance_proposals_vote_handler(
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
    Json(body): Json<GovernanceVoteBody>,
) -> impl IntoResponse {
    let weight = body.weight.unwrap_or(1.0);
    let event = StateEvent::VoteCast {
        proposal_id,
        voter_id: body.voter_id,
        vote: body.vote,
        weight,
    };
    let wrapped = state.unified_state.log_and_apply(event, vec![]);
    Json(serde_json::json!({
        "sequence": wrapped.sequence,
        "event_id": wrapped.id
    }))
    .into_response()
}

/// GET /api/v1/governance/proposals – List proposals (GovernanceSnapshot)
pub async fn governance_proposals_list_handler(State(state): State<AppState>) -> Json<GovernanceSnapshot> {
    let snapshot = state.unified_state.snapshot();
    Json(snapshot.governance)
}

/// POST /api/v1/controller/check – AuthZ check (permission, resource, caller_did, realm_id)
pub async fn controller_check_handler(
    State(state): State<AppState>,
    Json(_body): Json<ControllerCheckBody>,
) -> impl IntoResponse {
    use std::sync::atomic::Ordering;
    // Record check in state; allow by default until ECL/Controller policy is wired
    state.unified_state.controller.authz_checks.fetch_add(1, Ordering::Relaxed);
    state.unified_state.controller.authz_allowed.fetch_add(1, Ordering::Relaxed);
    Json(ControllerCheckResponse {
        allowed: true,
        reason: None,
    })
    .into_response()
}

/// GET /api/v1/controller/permissions – Permissions for Realm/Caller (ControllerSnapshot)
pub async fn controller_permissions_handler(State(state): State<AppState>) -> Json<ControllerSnapshot> {
    let snapshot = state.unified_state.snapshot();
    Json(snapshot.controller)
}

// ============================================================================
// Phase 3: Intent & Saga
// ============================================================================

/// POST /api/v1/intent/parse – Parse intent (stub: returns minimal parsed intent)
pub async fn intent_parse_handler(Json(_body): Json<IntentParseBody>) -> impl IntoResponse {
    Json(serde_json::json!({
        "parsed": true,
        "intent_id": "stub",
        "goal_type": "unknown",
        "estimated_steps": 0,
        "message": "IntentParser not wired; stub response"
    }))
    .into_response()
}

/// POST /api/v1/saga/compose – Compose saga (stub: returns minimal composed saga)
pub async fn saga_compose_handler(Json(_body): Json<SagaComposeBody>) -> impl IntoResponse {
    Json(serde_json::json!({
        "composed": true,
        "saga_id": "stub",
        "steps": [],
        "gas_estimate": 0,
        "mana_estimate": 0,
        "message": "SagaComposer not wired; stub response"
    }))
    .into_response()
}

/// POST /api/v1/saga/execute – Execute saga step (StateEvent::SagaProgress)
pub async fn saga_execute_handler(
    State(state): State<AppState>,
    Json(body): Json<SagaExecuteBody>,
) -> impl IntoResponse {
    let saga_id = body.saga_id.clone();
    let step = body.step.unwrap_or(0);
    let total_steps = body.total_steps.unwrap_or(1);
    let cross_realm = body.cross_realm.unwrap_or(false);
    let compensation_triggered = body.compensation_triggered.unwrap_or(false);
    let realms = body.realms.unwrap_or_default();
    let event = StateEvent::SagaProgress {
        saga_id: body.saga_id,
        step,
        total_steps,
        cross_realm,
        compensation_triggered,
        realms,
    };
    let wrapped = state.unified_state.log_and_apply(event, vec![]);
    Json(serde_json::json!({
        "saga_id": saga_id,
        "sequence": wrapped.sequence,
        "event_id": wrapped.id
    }))
    .into_response()
}

/// GET /api/v1/saga/stats – SagaComposerSnapshot
pub async fn saga_stats_handler(State(state): State<AppState>) -> Json<SagaComposerSnapshot> {
    let snapshot = state.unified_state.snapshot();
    Json(snapshot.peer.saga)
}
