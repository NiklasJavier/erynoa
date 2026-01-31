//! Connect-RPC Routes
//!
//! Routes für Connect-RPC/gRPC-Web Services
//!
//! Aktuell verfügbare Services:
//! - Health: Health checks und Readiness probes
//! - Info: System-Informationen
//! - Peer: Peer-Status, Key-Management, Gateway
//! - Intent: Intent-Submission und -Auflösung (Killer-Feature)
//! - Saga: Saga-Lifecycle und -Kompensation
//! - Environment: Realm-Management und -Navigation

use crate::server::AppState;
use axum::Router;

// Import generated service types
use crate::gen::erynoa::v1::{
    EnvironmentService, HealthService, InfoService, IntentService, PeerService, SagaService,
};

// Import handler functions
use crate::api::v1::environment::{
    create_environment_handler, get_bootstrap_status_handler, get_environment_info_handler,
    get_environment_tree_handler, join_environment_handler, leave_environment_handler,
    list_environments_handler, switch_environment_handler,
};
use crate::api::v1::health::{health_check_handler, ready_check_handler};
use crate::api::v1::info::get_info_handler;
use crate::api::v1::intent::{
    cancel_intent_handler, get_intent_status_handler, list_intents_handler, resolve_intent_handler,
    simulate_intent_handler, submit_intent_handler,
};
use crate::api::v1::peer::{
    derive_key_handler, evaluate_gateway_handler, get_info_handler as peer_get_info_handler,
    get_status_handler, list_derived_keys_handler, start_peer_handler, stop_peer_handler,
};
use crate::api::v1::saga::{
    cancel_saga_handler, execute_saga_handler, get_saga_history_handler, get_saga_status_handler,
    list_sagas_handler, rollback_saga_handler,
};

/// Erstellt Router für Connect-RPC Services
///
/// Connect-RPC Services werden unter /api/v1/connect/ bereitgestellt
/// z.B. /api/v1/connect/erynoa.v1.HealthService/Check
pub fn create_connect_routes(_state: AppState) -> Router<AppState> {
    // Start with empty router
    let router = Router::new();

    // =========================================================================
    // Health Service
    // =========================================================================
    let router = HealthService::check(health_check_handler)(router);
    let router = HealthService::ready(ready_check_handler)(router);

    // =========================================================================
    // Info Service
    // =========================================================================
    let router = InfoService::get_info(get_info_handler)(router);

    // =========================================================================
    // Peer Service
    // =========================================================================
    let router = PeerService::get_status(get_status_handler)(router);
    let router = PeerService::get_info(peer_get_info_handler)(router);
    let router = PeerService::list_derived_keys(list_derived_keys_handler)(router);
    let router = PeerService::derive_key(derive_key_handler)(router);
    let router = PeerService::evaluate_gateway(evaluate_gateway_handler)(router);
    let router = PeerService::start_peer(start_peer_handler)(router);
    let router = PeerService::stop_peer(stop_peer_handler)(router);

    // =========================================================================
    // Intent Service (PR1: Killer-Feature)
    // =========================================================================
    let router = IntentService::submit_intent(submit_intent_handler)(router);
    let router = IntentService::resolve_intent(resolve_intent_handler)(router);
    let router = IntentService::simulate_intent(simulate_intent_handler)(router);
    let router = IntentService::get_intent_status(get_intent_status_handler)(router);
    let router = IntentService::list_intents(list_intents_handler)(router);
    let router = IntentService::cancel_intent(cancel_intent_handler)(router);

    // =========================================================================
    // Saga Service (Κ22, Κ24)
    // =========================================================================
    let router = SagaService::list_sagas(list_sagas_handler)(router);
    let router = SagaService::get_saga_status(get_saga_status_handler)(router);
    let router = SagaService::execute_saga(execute_saga_handler)(router);
    let router = SagaService::cancel_saga(cancel_saga_handler)(router);
    let router = SagaService::rollback_saga(rollback_saga_handler)(router);
    let router = SagaService::get_saga_history(get_saga_history_handler)(router);

    // =========================================================================
    // Environment Service (Κ19, Κ20)
    // =========================================================================
    let router = EnvironmentService::list_environments(list_environments_handler)(router);
    let router = EnvironmentService::get_environment_tree(get_environment_tree_handler)(router);
    let router = EnvironmentService::create_environment(create_environment_handler)(router);
    let router = EnvironmentService::join_environment(join_environment_handler)(router);
    let router = EnvironmentService::leave_environment(leave_environment_handler)(router);
    let router = EnvironmentService::get_environment_info(get_environment_info_handler)(router);
    let router = EnvironmentService::switch_environment(switch_environment_handler)(router);
    let router = EnvironmentService::get_bootstrap_status(get_bootstrap_status_handler)(router);

    router
}
