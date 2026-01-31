//! IntentService Connect-RPC Handlers
//!
//! Implementiert Intent-Endpunkte: SubmitIntent, ResolveIntent, Simulate, Status, List, Cancel
//!
//! ## Kern-Flow (PR1)
//!
//! ```text
//! SubmitIntent → IntentParser.parse() → SagaComposer.compose() → Saga
//! ```

use axum::extract::State;
use chrono::Utc;
use std::collections::HashMap;

use crate::domain::{Constraint, Goal, Intent as DomainIntent, RealmId, DID};
use crate::gen::erynoa::v1::{
    Budget, CancelIntentRequest, CancelIntentResponse, ChainType, CostEstimate, DependencyGraph,
    Did, GetIntentStatusRequest, GetIntentStatusResponse, Intent, IntentState, IntentSummary,
    ListIntentsRequest, ListIntentsResponse, ResolveIntentRequest, ResolveIntentResponse, SagaPlan,
    SagaStepPlan, SimulateIntentRequest, SimulateIntentResponse, SimulationResult, SimulationStep,
    SubmitIntentRequest, SubmitIntentResponse,
};
use crate::peer::{IntentParser, SagaComposer};
use crate::server::AppState;

// ============================================================================
// SUBMIT INTENT (Killer-Feature)
// ============================================================================

/// SubmitIntent - PR1: Intent → Saga Pipeline
///
/// Parst einen natürlichsprachlichen oder strukturierten Intent,
/// komponiert eine Saga und gibt einen geschätzten Plan zurück.
///
/// ## Beispiel-Intents
///
/// - "Kaufe 50 kWh Strom von Anbieter X"
/// - "Übertrage 100 USDC an did:erynoa:bob"
/// - "Stake 1000 ERY für 30 Tage"
pub async fn submit_intent_handler(
    State(state): State<AppState>,
    request: SubmitIntentRequest,
) -> SubmitIntentResponse {
    let parser = IntentParser::default();
    let composer = SagaComposer::default();

    // Lade eigene DID
    let author_did = state
        .storage
        .identity()
        .get_primary()
        .await
        .ok()
        .flatten()
        .map(|i| i.did)
        .unwrap_or_else(DID::generate);

    // Parse Goal aus Request
    let (goal, constraints) = parse_goal_from_request(&request);

    // Parse Intent
    let intent = match parser.parse_structured(author_did.clone(), goal.clone(), constraints) {
        Ok(intent) => intent,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to parse intent");
            return SubmitIntentResponse {
                intent_id: String::new(),
                state: IntentState::Failed as i32,
                estimated_plan: None,
                estimated_cost: None,
                required_approvals: vec![format!("Parse error: {}", e)],
            };
        }
    };

    // Κ22: Komponiere Saga
    let saga = match composer.compose(&intent) {
        Ok(saga) => saga,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to compose saga");
            return SubmitIntentResponse {
                intent_id: intent.id.to_string(),
                state: IntentState::Failed as i32,
                estimated_plan: None,
                estimated_cost: Some(CostEstimate {
                    amount: "0".to_string(),
                    asset: "ERY".to_string(),
                    fee_estimate: 0.0,
                    fee_asset: "ERY".to_string(),
                }),
                required_approvals: vec![format!("Composition error: {}", e)],
            };
        }
    };

    // Generiere Intent-ID
    let intent_id = saga.id.to_string();

    // Erstelle Plan aus Saga-Steps
    let estimated_plan = Some(SagaPlan {
        steps: saga
            .steps
            .iter()
            .enumerate()
            .map(|(i, step)| SagaStepPlan {
                step_number: i as i32,
                chain: ChainType::Erynoa as i32,
                action: step.name.clone(),
                description: step.description.clone(),
                cost: Some(CostEstimate {
                    amount: "0.001".to_string(),
                    asset: "ERY".to_string(),
                    fee_estimate: 0.001,
                    fee_asset: "ERY".to_string(),
                }),
                requires_approval: false,
            })
            .collect(),
        estimated_duration_seconds: (saga.steps.len() * 10) as i32,
        realm_crossings: vec![],
    });

    // Kosten schätzen
    let estimated_cost = Some(CostEstimate {
        amount: request
            .budget
            .as_ref()
            .map(|b| b.amount.clone())
            .unwrap_or_else(|| "0".to_string()),
        asset: request
            .budget
            .as_ref()
            .map(|b| b.asset.clone())
            .unwrap_or_else(|| "ERY".to_string()),
        fee_estimate: 0.01,
        fee_asset: "ERY".to_string(),
    });

    SubmitIntentResponse {
        intent_id,
        state: IntentState::Pending as i32,
        estimated_plan,
        estimated_cost,
        required_approvals: vec![],
    }
}

/// Parse Goal aus SubmitIntentRequest
fn parse_goal_from_request(request: &SubmitIntentRequest) -> (Goal, Vec<Constraint>) {
    let goal_text = request.goal.to_lowercase();
    let constraints = Vec::new();

    // Transfer-Patterns
    if goal_text.contains("transfer")
        || goal_text.contains("send")
        || goal_text.contains("pay")
        || goal_text.contains("übertrage")
    {
        // Versuche Betrag und Empfänger zu extrahieren
        let amount = request
            .budget
            .as_ref()
            .and_then(|b| b.amount.parse::<u64>().ok())
            .unwrap_or(0);

        let asset_type = request
            .budget
            .as_ref()
            .map(|b| b.asset.clone())
            .unwrap_or_else(|| "ERY".to_string());

        // Empfänger aus Metadata oder generisch
        let to_did = request
            .metadata
            .get("recipient")
            .map(|r| DID::parse(r).unwrap_or_else(|_| DID::generate()))
            .unwrap_or_else(DID::generate);

        return (
            Goal::Transfer {
                to: to_did,
                amount,
                asset_type,
            },
            constraints,
        );
    }

    // Attest-Patterns
    if goal_text.contains("attest")
        || goal_text.contains("verify")
        || goal_text.contains("certify")
        || goal_text.contains("bestätige")
    {
        let subject = request
            .metadata
            .get("subject")
            .map(|s| DID::parse(s).unwrap_or_else(|_| DID::generate()))
            .unwrap_or_else(DID::generate);

        return (
            Goal::Attest {
                subject,
                claim: request.goal.clone(),
            },
            constraints,
        );
    }

    // Delegate-Patterns
    if goal_text.contains("delegate")
        || goal_text.contains("authorize")
        || goal_text.contains("grant")
        || goal_text.contains("delegiere")
    {
        let to = request
            .metadata
            .get("delegate")
            .map(|d| DID::parse(d).unwrap_or_else(|_| DID::generate()))
            .unwrap_or_else(DID::generate);

        let capabilities: Vec<String> = request
            .metadata
            .get("capabilities")
            .map(|c| c.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|| vec!["*".to_string()]);

        let ttl_seconds = request.timeout_seconds.unwrap_or(3600) as u64;

        return (
            Goal::Delegate {
                to,
                capabilities,
                ttl_seconds,
            },
            constraints,
        );
    }

    // Query-Patterns
    if goal_text.contains("query")
        || goal_text.contains("find")
        || goal_text.contains("search")
        || goal_text.contains("suche")
    {
        return (
            Goal::Query {
                predicate: request.goal.clone(),
            },
            constraints,
        );
    }

    // Create-Patterns
    if goal_text.contains("create")
        || goal_text.contains("mint")
        || goal_text.contains("new")
        || goal_text.contains("erstelle")
    {
        return (
            Goal::Create {
                entity_type: "asset".to_string(),
                params: request.metadata.clone().into_iter().collect(),
            },
            constraints,
        );
    }

    // Fallback: Complex Goal
    (
        Goal::Complex {
            description: request.goal.clone(),
            parsed_goals: vec![],
        },
        constraints,
    )
}

// ============================================================================
// RESOLVE INTENT
// ============================================================================

/// ResolveIntent - Löst einen Intent in eine ausführbare Saga auf
pub async fn resolve_intent_handler(
    State(_state): State<AppState>,
    request: ResolveIntentRequest,
) -> ResolveIntentResponse {
    // In einer echten Implementierung würden wir den Intent aus dem Storage laden
    let saga_id = format!("saga:{}", uuid::Uuid::new_v4());

    ResolveIntentResponse {
        intent_id: request.intent_id,
        saga_id,
        plan: Some(SagaPlan {
            steps: vec![],
            estimated_duration_seconds: 30,
            realm_crossings: vec![],
        }),
        dependency_graph: Some(DependencyGraph {
            nodes: vec![],
            edges: vec![],
        }),
    }
}

// ============================================================================
// SIMULATE INTENT
// ============================================================================

/// SimulateIntent - Simuliert Intent-Ausführung ohne Ausführung
pub async fn simulate_intent_handler(
    State(_state): State<AppState>,
    request: SimulateIntentRequest,
) -> SimulateIntentResponse {
    SimulateIntentResponse {
        plan: Some(SagaPlan {
            steps: vec![],
            estimated_duration_seconds: 30,
            realm_crossings: vec![],
        }),
        steps: vec![
            SimulationStep {
                step_number: 0,
                action: "validate".to_string(),
                result: SimulationResult::Success as i32,
                state_before: None,
                state_after: None,
            },
            SimulationStep {
                step_number: 1,
                action: "execute".to_string(),
                result: SimulationResult::Success as i32,
                state_before: None,
                state_after: None,
            },
        ],
        total_cost: Some(CostEstimate {
            amount: "0.01".to_string(),
            asset: "ERY".to_string(),
            fee_estimate: 0.01,
            fee_asset: "ERY".to_string(),
        }),
        would_succeed: true,
        warnings: vec![],
    }
}

// ============================================================================
// GET INTENT STATUS
// ============================================================================

/// GetIntentStatus - Status eines bestehenden Intents
pub async fn get_intent_status_handler(
    State(_state): State<AppState>,
    request: GetIntentStatusRequest,
) -> GetIntentStatusResponse {
    // In einer echten Implementierung: Lade aus Intent-Storage
    let now = Utc::now();

    GetIntentStatusResponse {
        intent_id: request.intent_id.clone(),
        state: IntentState::Completed as i32,
        saga_id: Some(format!("saga:{}", uuid::Uuid::new_v4())),
        created_at: Some(axum_connect::pbjson_types::Timestamp {
            seconds: now.timestamp(),
            nanos: 0,
        }),
        updated_at: Some(axum_connect::pbjson_types::Timestamp {
            seconds: now.timestamp(),
            nanos: 0,
        }),
        error_message: None,
        intent: Some(Intent {
            id: request.intent_id,
            author: Some(Did {
                namespace: "self".to_string(),
                unique_id: "anonymous".to_string(),
                created_at: None,
            }),
            goal: "Query".to_string(),
            budget: None,
            target_realm: None,
            constraints: HashMap::new(),
            created_at: Some(axum_connect::pbjson_types::Timestamp {
                seconds: now.timestamp(),
                nanos: 0,
            }),
        }),
    }
}

// ============================================================================
// LIST INTENTS
// ============================================================================

/// ListIntents - Liste aller Intents (gefiltert)
pub async fn list_intents_handler(
    State(_state): State<AppState>,
    _request: ListIntentsRequest,
) -> ListIntentsResponse {
    // In einer echten Implementierung: Lade aus Intent-Storage
    ListIntentsResponse {
        intents: vec![],
        next_cursor: None,
        total_count: 0,
    }
}

// ============================================================================
// CANCEL INTENT
// ============================================================================

/// CancelIntent - Bricht einen Intent ab
pub async fn cancel_intent_handler(
    State(_state): State<AppState>,
    request: CancelIntentRequest,
) -> CancelIntentResponse {
    tracing::info!(
        intent_id = %request.intent_id,
        reason = ?request.reason,
        "Cancelling intent"
    );

    CancelIntentResponse {
        success: true,
        error: None,
    }
}
