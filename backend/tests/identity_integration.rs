//! # Identity Integration Tests (Phase 8)
//!
//! Tests für Identity-System Integration mit UnifiedState, StateEvents und StateGraph.

use std::sync::atomic::Ordering;

use erynoa_api::core::identity_types::{IdentityMode, RealmRole, WalletAddress};
use erynoa_api::core::state::{
    StateComponent, StateEvent, StateGraph, UnifiedState, WrappedStateEvent,
};
use erynoa_api::domain::unified::identity::{Capability, DIDNamespace};
use erynoa_api::domain::unified::primitives::UniversalId;

// ============================================================================
// UNIFIED STATE IDENTITY INTEGRATION
// ============================================================================

mod unified_state_integration {
    use super::*;

    /// Test: IdentityState ist korrekt in UnifiedState integriert
    #[test]
    fn test_identity_state_in_unified_state() {
        let state = UnifiedState::new();

        // Identity sollte initialisiert aber nicht bootstrapped sein
        assert!(!state.identity.is_bootstrapped());
        assert_eq!(state.identity.current_mode(), IdentityMode::Interactive);

        // Bootstrap
        state.identity.bootstrap_test(&[1u8; 32]).unwrap();

        // Nach Bootstrap
        assert!(state.identity.is_bootstrapped());
        assert_eq!(state.identity.current_mode(), IdentityMode::Test);
    }

    /// Test: Identity-Snapshot wird korrekt in UnifiedSnapshot inkludiert
    #[test]
    fn test_unified_snapshot_includes_identity() {
        let state = UnifiedState::new();
        state.identity.bootstrap_test(&[1u8; 32]).unwrap();

        // Derive some sub-DIDs
        state.identity.derive_device_did(0).unwrap();
        state.identity.derive_agent_did(0).unwrap();

        let snapshot = state.snapshot();

        // Verify identity snapshot
        assert!(snapshot.identity.bootstrap_completed);
        assert_eq!(snapshot.identity.mode, IdentityMode::Test);
        assert!(snapshot.identity.root_did.is_some());
        assert_eq!(snapshot.identity.sub_dids_total, 2);
    }

    /// Test: Identity-Health beeinflusst UnifiedState-Health
    #[test]
    fn test_identity_health_affects_unified_health() {
        let state = UnifiedState::new();

        // Ohne Bootstrap: Identity hat Health 0
        let health_before = state.calculate_health();

        // Mit Bootstrap: Identity hat Health > 0
        state.identity.bootstrap_test(&[1u8; 32]).unwrap();
        let health_after = state.calculate_health();

        // Health sollte gestiegen sein (Identity trägt 10% bei)
        assert!(
            health_after > health_before,
            "Health should increase after bootstrap: {} -> {}",
            health_before,
            health_after
        );
    }
}

// ============================================================================
// STATE EVENT FLOW
// ============================================================================

mod state_event_flow {
    use super::*;

    /// Test: SubDIDDerived Event wird korrekt erstellt
    #[test]
    fn test_sub_did_derived_event_creation() {
        let state = UnifiedState::new();
        state.identity.bootstrap_test(&[1u8; 32]).unwrap();

        let root_did = state.identity.root_did_id().unwrap();
        let sub_did = UniversalId::new(UniversalId::TAG_DID, 1, b"test-agent");

        let event = StateEvent::SubDIDDerived {
            root_did,
            sub_did,
            namespace: DIDNamespace::Spirit,
            derivation_path: "m/44'/erynoa'/0'/agent/0".to_string(),
            purpose: "agent".to_string(),
            gas_used: 100,
            realm_id: None,
        };

        // Verify event properties
        assert_eq!(event.primary_component(), StateComponent::Identity);
        assert!(event.is_identity_event());
        assert!(event.estimated_size_bytes() > 0);

        // Verify involved identities
        let involved = event.involved_identities();
        assert!(involved.contains(&root_did));
        assert!(involved.contains(&sub_did));
    }

    /// Test: IdentityBootstrapped Event ist kritisch
    #[test]
    fn test_identity_bootstrapped_is_critical() {
        let root_did = UniversalId::new(UniversalId::TAG_DID, 1, b"root");

        let event = StateEvent::IdentityBootstrapped {
            root_did,
            namespace: DIDNamespace::Self_,
            mode: IdentityMode::Interactive,
            timestamp_ms: 1000,
        };

        assert!(event.is_critical());
    }

    /// Test: DelegationCreated Event Properties
    #[test]
    fn test_delegation_event_properties() {
        let state = UnifiedState::new();
        state.identity.bootstrap_test(&[1u8; 32]).unwrap();

        let root_did = state.identity.root_did_id().unwrap();
        let delegate = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");

        let event = StateEvent::DelegationCreated {
            delegator: root_did,
            delegate,
            trust_factor: 0.8,
            capabilities: vec!["read:*".to_string()],
            valid_until: None,
        };

        // Verify event properties
        assert_eq!(event.primary_component(), StateComponent::Identity);
        assert!(event.is_identity_event());

        // Verify involved identities
        let involved = event.involved_identities();
        assert!(involved.contains(&root_did));
        assert!(involved.contains(&delegate));
    }

    /// Test: RealmMembershipChanged Event hat Realm-Kontext
    #[test]
    fn test_realm_membership_event_context() {
        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"test-realm");
        let member_id = UniversalId::new(UniversalId::TAG_DID, 1, b"member");

        let event = StateEvent::RealmMembershipChanged {
            realm_id,
            member_id,
            action: "Joined".to_string(),
            new_role: Some("member".to_string()),
            realm_sub_did: None,
        };

        // Should have realm context
        assert_eq!(event.realm_context(), Some(&realm_id));

        // Should be identity event
        assert!(event.is_identity_event());

        // Should involve member_id
        let involved = event.involved_identities();
        assert!(involved.contains(&member_id));
    }

    /// Test: KeyRotated Event ist kritisch
    #[test]
    fn test_key_rotated_is_critical() {
        let did = UniversalId::new(UniversalId::TAG_DID, 1, b"user");
        let old_key = UniversalId::new(UniversalId::TAG_DID, 1, b"old-key");
        let new_key = UniversalId::new(UniversalId::TAG_DID, 1, b"new-key");

        let event = StateEvent::KeyRotated {
            did,
            old_key_id: old_key,
            new_key_id: new_key,
            reason: "Scheduled rotation".to_string(),
        };

        assert!(event.is_critical());
        assert!(event.is_identity_event());
    }

    /// Test: IdentityAnomalyDetected mit critical severity
    #[test]
    fn test_identity_anomaly_critical() {
        let did = UniversalId::new(UniversalId::TAG_DID, 1, b"user");

        let event = StateEvent::IdentityAnomalyDetected {
            did,
            anomaly_type: "suspicious_delegation_pattern".to_string(),
            severity: "critical".to_string(),
            details: "Multiple rapid delegations detected".to_string(),
        };

        assert!(event.is_critical());

        // Warning severity should not be critical
        let warning_event = StateEvent::IdentityAnomalyDetected {
            did,
            anomaly_type: "unusual_activity".to_string(),
            severity: "warning".to_string(),
            details: "Unusual login time".to_string(),
        };

        assert!(!warning_event.is_critical());
    }
}

// ============================================================================
// STATE GRAPH INTEGRATION
// ============================================================================

mod state_graph_integration {
    use super::*;
    use erynoa_api::core::state::StateRelation;

    /// Test: Identity-Komponenten sind im StateGraph
    #[test]
    fn test_identity_components_in_graph() {
        let graph = StateGraph::erynoa_graph();

        // Trust depends on Identity
        assert!(graph.edges.contains(&(
            StateComponent::Trust,
            StateRelation::DependsOn,
            StateComponent::Identity
        )));

        // Identity triggers Trust
        assert!(graph.edges.contains(&(
            StateComponent::Identity,
            StateRelation::Triggers,
            StateComponent::Trust
        )));
    }

    /// Test: Identity-Abhängigkeiten
    #[test]
    fn test_identity_dependencies() {
        let graph = StateGraph::erynoa_graph();

        let dependents = graph.dependents(StateComponent::Identity);

        // Multiple components should depend on Identity
        assert!(dependents.contains(&StateComponent::Trust));
        assert!(dependents.contains(&StateComponent::Realm));
        assert!(dependents.contains(&StateComponent::Controller));
    }

    /// Test: Identity-Trigger
    #[test]
    fn test_identity_triggers() {
        let graph = StateGraph::erynoa_graph();

        let triggered = graph.triggered_by(StateComponent::Identity);

        // Identity should trigger various components
        assert!(triggered.contains(&StateComponent::Trust));
        assert!(triggered.contains(&StateComponent::Realm));
        assert!(triggered.contains(&StateComponent::Event));
    }

    /// Test: KeyManagement im Graph
    #[test]
    fn test_key_management_in_graph() {
        let graph = StateGraph::erynoa_graph();

        // KeyManagement should have relationships
        let key_deps = graph.dependents(StateComponent::KeyManagement);
        assert!(!key_deps.is_empty());
    }
}

// ============================================================================
// CROSS-COMPONENT INTERACTIONS
// ============================================================================

mod cross_component_interactions {
    use super::*;

    /// Test: Identity → Trust Integration
    #[test]
    fn test_identity_trust_integration() {
        let state = UnifiedState::new();
        state.identity.bootstrap_test(&[1u8; 32]).unwrap();

        // Register identity in trust state
        let root_id = state.identity.root_did_id().unwrap();
        let result = state.core.trust.register_identity(root_id, 0.5);
        assert!(result.is_ok());

        // Verify trust is tracked
        assert_eq!(state.core.trust.get_trust(&root_id), Some(0.5));
    }

    /// Test: Identity → Realm Integration
    #[test]
    fn test_identity_realm_integration() {
        let state = UnifiedState::new();
        state.identity.bootstrap_test(&[1u8; 32]).unwrap();

        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"test-realm");

        // Join realm via identity
        state
            .identity
            .join_realm(realm_id, RealmRole::Member, Some(0.7))
            .unwrap();

        // Verify membership
        assert!(state.identity.is_realm_member(&realm_id));

        // Derive realm-specific DID
        let realm_did_result = state.identity.derive_realm_did(&realm_id);
        assert!(realm_did_result.is_ok());
    }

    /// Test: Identity → P2P SwarmState Integration
    #[test]
    fn test_identity_swarm_integration() {
        let state = UnifiedState::new();
        state.identity.bootstrap_test(&[1u8; 32]).unwrap();

        // Derive device DID for P2P
        state.identity.derive_device_did(0).unwrap();
        let device_id = state.identity.device_did_id().unwrap();

        // Set swarm peer identity
        state.p2p.swarm.set_peer_universal_id(device_id);

        // Verify
        assert_eq!(state.p2p.swarm.get_peer_universal_id(), Some(device_id));
    }

    /// Test: Complete Identity Lifecycle
    #[test]
    fn test_complete_identity_lifecycle() {
        let state = UnifiedState::new();

        // 1. Bootstrap (use test mode for simplicity)
        state.identity.bootstrap_test(&[1u8; 32]).unwrap();
        assert!(state.identity.is_bootstrapped());

        // 2. Derive device DID
        let device_id = state.identity.derive_device_did(0).unwrap();
        assert!(state.identity.device_did_id().is_some());

        // 3. Derive agent DID
        let _agent_id = state.identity.derive_agent_did(0).unwrap();

        // 4. Join realm
        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"my-realm");
        state
            .identity
            .join_realm(realm_id, RealmRole::Member, Some(0.7))
            .unwrap();

        // 5. Derive realm-specific DID
        let _realm_did = state.identity.derive_realm_did(&realm_id).unwrap();

        // 6. Add delegation
        let delegate_id = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");
        state
            .identity
            .add_delegation(
                delegate_id,
                0.8,
                vec![Capability::Read {
                    resource: "realm:*".to_string(),
                }],
                None,
            )
            .unwrap();

        // 7. Add wallet
        let root_id = state.identity.root_did_id().unwrap();
        let wallet = WalletAddress::new(
            "eip155:1",
            "0x1234567890123456789012345678901234567890",
            "m/44'/60'/0'/0/0",
            root_id,
        );
        state.identity.add_wallet_address(wallet).unwrap();

        // 8. Set P2P identity
        state.p2p.swarm.set_peer_universal_id(device_id);

        // 9. Register in trust state
        state.core.trust.register_identity(root_id, 0.5).unwrap();

        // Verify final state
        let snapshot = state.snapshot();
        assert!(snapshot.identity.bootstrap_completed);
        // device + agent + realm_sub_did (from join_realm) + explicit derive_realm_did = 4
        assert_eq!(snapshot.identity.sub_dids_total, 4);
        assert_eq!(snapshot.identity.realm_membership_count, 1);
        assert_eq!(snapshot.identity.active_delegations, 1);
        assert_eq!(snapshot.identity.addresses_total, 1);
        assert!(snapshot.p2p.swarm.peer_universal_id.is_some());
    }
}

// ============================================================================
// ERROR HANDLING
// ============================================================================

mod error_handling {
    use super::*;

    /// Test: Operations fail without bootstrap
    #[test]
    fn test_operations_fail_without_bootstrap() {
        let state = UnifiedState::new();

        // Derive should fail
        assert!(state.identity.derive_device_did(0).is_err());

        // Join realm should fail
        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"realm");
        assert!(state.identity.join_realm(realm_id, RealmRole::Member, None).is_err());

        // Add delegation should fail
        let delegate = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");
        assert!(state.identity.add_delegation(delegate, 0.8, vec![], None).is_err());
    }

    /// Test: Double bootstrap fails
    #[test]
    fn test_double_bootstrap_fails() {
        let state = UnifiedState::new();

        assert!(state.identity.bootstrap_test(&[1u8; 32]).is_ok());
        assert!(state.identity.bootstrap_test(&[2u8; 32]).is_err());
    }

    /// Test: Invalid trust factor fails
    #[test]
    fn test_invalid_trust_factor_fails() {
        let state = UnifiedState::new();
        state.identity.bootstrap_test(&[1u8; 32]).unwrap();

        let delegate = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");

        // > 1.0 should fail
        assert!(state.identity.add_delegation(delegate, 1.5, vec![], None).is_err());

        // <= 0 should fail
        assert!(state.identity.add_delegation(delegate, 0.0, vec![], None).is_err());
        assert!(state.identity.add_delegation(delegate, -0.1, vec![], None).is_err());
    }

    /// Test: Ephemeral mode restrictions
    #[test]
    fn test_ephemeral_mode_restrictions() {
        let state = UnifiedState::new();
        state.identity.bootstrap_ephemeral(&[1u8; 32]).unwrap();

        // Ephemeral should not allow realm membership
        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"realm");
        assert!(state.identity.join_realm(realm_id, RealmRole::Member, None).is_err());
    }
}
