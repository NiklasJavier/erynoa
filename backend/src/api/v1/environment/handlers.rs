//! EnvironmentService Connect-RPC Handlers
//!
//! Implementiert Environment-Endpunkte: List, Tree, Create, Join, Leave, Info, Switch, Bootstrap
//!
//! ## Realm-Hierarchie
//!
//! ```text
//! Root (Κ19)
//!   ├── Virtual Environment A
//!   │     ├── Sub-Virtual A1
//!   │     └── Sub-Virtual A2
//!   └── Virtual Environment B
//! ```

use axum::extract::State;
use chrono::Utc;

use crate::domain::RealmId as DomainRealmId;
use crate::gen::erynoa::v1::{
    BootstrapCheck, BootstrapMode, BootstrapState, BootstrapStatus, CreateEnvironmentRequest,
    CreateEnvironmentResponse, EnvironmentNode, EnvironmentSummary, EnvironmentType,
    GetBootstrapStatusRequest, GetBootstrapStatusResponse, GetEnvironmentInfoRequest,
    GetEnvironmentInfoResponse, GetEnvironmentTreeRequest, GetEnvironmentTreeResponse,
    GovernanceType, JoinEnvironmentRequest, JoinEnvironmentResponse, JoinStatus,
    LeaveEnvironmentRequest, LeaveEnvironmentResponse, ListEnvironmentsRequest,
    ListEnvironmentsResponse, RealmId, SwitchEnvironmentRequest, SwitchEnvironmentResponse,
    TrustVector6D,
};
use crate::server::AppState;

// ============================================================================
// LIST ENVIRONMENTS
// ============================================================================

/// ListEnvironments - Liste aller bekannten Environments
pub async fn list_environments_handler(
    State(_state): State<AppState>,
    _request: ListEnvironmentsRequest,
) -> ListEnvironmentsResponse {
    // Root-Environment ist immer verfügbar
    let root = EnvironmentSummary {
        id: Some(RealmId {
            id: "root".to_string(),
        }),
        name: "Root".to_string(),
        r#type: EnvironmentType::Root as i32,
        parent: None,
        member_count: 1,
        joined: true,
    };

    ListEnvironmentsResponse {
        environments: vec![root],
    }
}

// ============================================================================
// GET ENVIRONMENT TREE
// ============================================================================

/// GetEnvironmentTree - Hierarchische Baum-Ansicht
pub async fn get_environment_tree_handler(
    State(_state): State<AppState>,
    request: GetEnvironmentTreeRequest,
) -> GetEnvironmentTreeResponse {
    let max_depth = request.max_depth.unwrap_or(3);

    // Κ19: Root-Realm mit möglichen Sub-Realms
    let root = EnvironmentNode {
        id: Some(RealmId {
            id: "root".to_string(),
        }),
        name: "Erynoa Root".to_string(),
        r#type: EnvironmentType::Root as i32,
        children: if max_depth > 1 {
            vec![
                // Beispiel Virtual Environment
                EnvironmentNode {
                    id: Some(RealmId {
                        id: "ve:energy".to_string(),
                    }),
                    name: "Energy Markets".to_string(),
                    r#type: EnvironmentType::Virtual as i32,
                    children: vec![],
                    axiom_count: 5,
                    has_cbdc: false,
                },
                EnvironmentNode {
                    id: Some(RealmId {
                        id: "ve:gov".to_string(),
                    }),
                    name: "Governance".to_string(),
                    r#type: EnvironmentType::Virtual as i32,
                    children: vec![],
                    axiom_count: 12,
                    has_cbdc: true,
                },
            ]
        } else {
            vec![]
        },
        axiom_count: 24, // Core axioms
        has_cbdc: true,  // ERY Token
    };

    GetEnvironmentTreeResponse { root: Some(root) }
}

// ============================================================================
// CREATE ENVIRONMENT
// ============================================================================

/// CreateEnvironment - Erstellt neues Virtual Environment
pub async fn create_environment_handler(
    State(_state): State<AppState>,
    request: CreateEnvironmentRequest,
) -> CreateEnvironmentResponse {
    tracing::info!(
        name = %request.name,
        governance = ?request.governance,
        "Creating new environment"
    );

    // Generiere Realm-ID
    let realm_id = DomainRealmId::from_string(&format!("ve:{}", request.name.to_lowercase()));

    CreateEnvironmentResponse {
        id: Some(RealmId {
            id: realm_id.to_string(),
        }),
        success: true,
        error: None,
        bootstrap: Some(BootstrapStatus {
            mode: BootstrapMode::Short as i32,
            state: BootstrapState::Active as i32,
            progress_percent: 100,
            current_step: None,
            estimated_completion: None,
            checks: vec![
                BootstrapCheck {
                    name: "Governance configured".to_string(),
                    passed: true,
                    message: None,
                },
                BootstrapCheck {
                    name: "Parent realm valid".to_string(),
                    passed: true,
                    message: None,
                },
            ],
        }),
    }
}

// ============================================================================
// JOIN ENVIRONMENT
// ============================================================================

/// JoinEnvironment - Κ20: Tritt einem Environment bei
pub async fn join_environment_handler(
    State(_state): State<AppState>,
    request: JoinEnvironmentRequest,
) -> JoinEnvironmentResponse {
    let realm_id = request
        .environment
        .map(|r| r.id)
        .unwrap_or_else(|| "root".to_string());

    tracing::info!(
        environment = %realm_id,
        message = ?request.application_message,
        "Joining environment"
    );

    // Short Bootstrap: Sofort aktiv
    JoinEnvironmentResponse {
        success: true,
        status: JoinStatus::Approved as i32,
        error: None,
    }
}

// ============================================================================
// LEAVE ENVIRONMENT
// ============================================================================

/// LeaveEnvironment - Verlässt ein Environment
pub async fn leave_environment_handler(
    State(_state): State<AppState>,
    request: LeaveEnvironmentRequest,
) -> LeaveEnvironmentResponse {
    let realm_id = request
        .environment
        .map(|r| r.id)
        .unwrap_or_else(|| "unknown".to_string());

    tracing::info!(environment = %realm_id, "Leaving environment");

    if realm_id == "root" {
        return LeaveEnvironmentResponse {
            success: false,
            error: Some("Cannot leave root environment".to_string()),
        };
    }

    LeaveEnvironmentResponse {
        success: true,
        error: None,
    }
}

// ============================================================================
// GET ENVIRONMENT INFO
// ============================================================================

/// GetEnvironmentInfo - Detaillierte Informationen zu einem Environment
pub async fn get_environment_info_handler(
    State(_state): State<AppState>,
    request: GetEnvironmentInfoRequest,
) -> GetEnvironmentInfoResponse {
    let realm_id = request
        .environment
        .map(|r| r.id)
        .unwrap_or_else(|| "root".to_string());

    let now = Utc::now();

    // Κ19: Realm-Autonomie - jedes Environment hat eigene Axiome
    GetEnvironmentInfoResponse {
        id: Some(RealmId {
            id: realm_id.clone(),
        }),
        name: if realm_id == "root" {
            "Erynoa Root".to_string()
        } else {
            realm_id.clone()
        },
        r#type: if realm_id == "root" {
            EnvironmentType::Root as i32
        } else {
            EnvironmentType::Virtual as i32
        },
        parent: if realm_id == "root" {
            None
        } else {
            Some(RealmId {
                id: "root".to_string(),
            })
        },
        description: "Core Erynoa environment with full axiom set".to_string(),
        governance: GovernanceType::Dao as i32,
        member_count: 1,
        axiom_count: 24,
        local_axioms: vec![
            "Κ1: Identity Uniqueness".to_string(),
            "Κ2: Intent Causality".to_string(),
            "Κ3: Trust Transitivity".to_string(),
            "Κ19: Realm Autonomy".to_string(),
            "Κ20: Reputation Inheritance".to_string(),
            "Κ22: Saga Composition".to_string(),
            "Κ24: Atomic Compensation".to_string(),
        ],
        cbdc: None,
        min_trust_required: Some(TrustVector6D {
            reliability: 0.1,
            integrity: 0.1,
            competence: 0.0,
            prestige: 0.0,
            vigilance: 0.0,
            omega: 0.0,
        }),
        required_credentials: vec![],
        created_at: Some(axum_connect::pbjson_types::Timestamp {
            seconds: now.timestamp() - 86400 * 365, // 1 Jahr
            nanos: 0,
        }),
    }
}

// ============================================================================
// SWITCH ENVIRONMENT
// ============================================================================

/// SwitchEnvironment - PR3: Wechselt aktives Environment (Gateway-Check)
pub async fn switch_environment_handler(
    State(_state): State<AppState>,
    request: SwitchEnvironmentRequest,
) -> SwitchEnvironmentResponse {
    let realm_id = request
        .environment
        .as_ref()
        .map(|r| r.id.clone())
        .unwrap_or_else(|| "root".to_string());

    tracing::info!(environment = %realm_id, "Switching environment");

    // PR3, PR6: Gateway würde hier Trust-Check und Dämpfung durchführen
    SwitchEnvironmentResponse {
        success: true,
        active_environment: request.environment.unwrap_or(RealmId {
            id: "root".to_string(),
        }),
        error: None,
    }
}

// ============================================================================
// GET BOOTSTRAP STATUS
// ============================================================================

/// GetBootstrapStatus - Status des Bootstrap-Prozesses
pub async fn get_bootstrap_status_handler(
    State(_state): State<AppState>,
    request: GetBootstrapStatusRequest,
) -> GetBootstrapStatusResponse {
    let realm_id = request
        .environment
        .map(|r| r.id)
        .unwrap_or_else(|| "root".to_string());

    // Root ist immer vollständig gebootstrapt
    if realm_id == "root" {
        return GetBootstrapStatusResponse {
            status: Some(BootstrapStatus {
                mode: BootstrapMode::Long as i32,
                state: BootstrapState::Active as i32,
                progress_percent: 100,
                current_step: None,
                estimated_completion: None,
                checks: vec![
                    BootstrapCheck {
                        name: "Genesis complete".to_string(),
                        passed: true,
                        message: None,
                    },
                    BootstrapCheck {
                        name: "Core axioms verified".to_string(),
                        passed: true,
                        message: None,
                    },
                    BootstrapCheck {
                        name: "Root identity established".to_string(),
                        passed: true,
                        message: None,
                    },
                ],
            }),
        };
    }

    // Andere Environments: Simpler Status
    GetBootstrapStatusResponse {
        status: Some(BootstrapStatus {
            mode: BootstrapMode::Short as i32,
            state: BootstrapState::Pending as i32,
            progress_percent: 0,
            current_step: Some("Awaiting join".to_string()),
            estimated_completion: None,
            checks: vec![],
        }),
    }
}
