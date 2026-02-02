//! # Backend Integration Tests
//!
//! Umfassende Tests, die alle Backend-Komponenten zusammen prüfen:
//! - Domain Layer (Unified Data Model)
//! - Core Logic Layer (TrustEngine, ConsensusEngine, SurprisalCalculator)
//! - Execution Layer (ExecutionContext)
//! - Storage Layer (DecentralizedStorage)
//! - Protection Layer (AntiCalcification, Diversity, QuadraticGovernance)
//!
//! Diese Tests verifizieren, dass alle Komponenten als Gesamtsystem funktionieren.

// Domain Layer
use erynoa_api::domain::unified::*;

// Core Logic Layer
use erynoa_api::core::{ConsensusEngine, SurprisalCalculator, TrustEngine};

// Execution Layer
use erynoa_api::execution::ExecutionContext;

// Storage Layer
use erynoa_api::local::DecentralizedStorage;

// Protection Layer
use erynoa_api::protection::{AntiCalcification, DiversityMonitor, QuadraticGovernance};

// ============================================================================
// DOMAIN + CORE INTEGRATION
// ============================================================================

mod domain_core_integration {
    use super::*;

    /// Test: DID-Erstellung führt zu korrektem Trust-Vektor
    #[test]
    fn test_did_creation_with_trust_initialization() {
        // 1. Domain: DID erstellen
        let did = DID::new(DIDNamespace::Self_, b"integration-test-user");
        let _did_doc = DIDDocument::new(did.clone());

        // 2. Core: Trust-Engine initialisieren
        let mut trust_engine = TrustEngine::default();

        // 3. Trust für DID initialisieren
        trust_engine.initialize_trust_for_did(&did);

        // 4. Trust-Vektor abrufen
        let trust = trust_engine.get_trust_for_did(&did);
        assert!(trust.is_some());

        let trust = trust.unwrap();
        // Default Trust sollte 0.5 sein
        assert!((trust.r - 0.5).abs() < 0.01);
        assert!((trust.i - 0.5).abs() < 0.01);

        // 5. Newcomer-Trust vs Default-Trust
        let newcomer = TrustVector6D::NEWCOMER;
        let default = TrustVector6D::DEFAULT;

        assert!(newcomer.r < default.r);
        assert!(newcomer.i < default.i);
    }

    /// Test: Event-Erzeugung mit Surprisal-Berechnung
    #[test]
    fn test_event_creation_with_surprisal() {
        // 1. Domain: Event erstellen
        let actor = DID::new(DIDNamespace::Self_, b"event-actor");
        let actor_id = UniversalId::new(UniversalId::TAG_DID, 1, actor.id.as_bytes());

        let event = Event::new(
            actor_id.clone(),
            vec![],
            EventPayload::Custom {
                event_type: "test".to_string(),
                data: vec![1, 2, 3],
            },
            1,
        );

        // 2. Core: Surprisal berechnen
        let calc = SurprisalCalculator::new();
        let surprisal = calc.calculate_surprisal(&event);

        // Neues Event hat höheren Surprisal (niedrige Wahrscheinlichkeit)
        assert!(surprisal > 0.0);
    }

    /// Test: Delegation-Chain mit Trust-Decay
    #[test]
    fn test_delegation_chain_trust_decay() {
        // Root -> A -> B -> C (3 Hops)
        let root = DID::new(DIDNamespace::Self_, b"root");
        let a = DID::new(DIDNamespace::Self_, b"a");
        let b = DID::new(DIDNamespace::Self_, b"b");
        let c = DID::new(DIDNamespace::Self_, b"c");

        let root_id = UniversalId::new(UniversalId::TAG_DID, 1, root.id.as_bytes());
        let a_id = UniversalId::new(UniversalId::TAG_DID, 1, a.id.as_bytes());
        let b_id = UniversalId::new(UniversalId::TAG_DID, 1, b.id.as_bytes());
        let c_id = UniversalId::new(UniversalId::TAG_DID, 1, c.id.as_bytes());

        // Delegations mit 0.9 Trust-Faktor
        let del_a = Delegation::new(
            root_id.clone(),
            a_id.clone(),
            0.9,
            vec![Capability::Write {
                resource: "*".to_string(),
            }],
        );
        let del_b = Delegation::new(
            a_id.clone(),
            b_id.clone(),
            0.9,
            vec![Capability::Write {
                resource: "*".to_string(),
            }],
        );
        let del_c = Delegation::new(
            b_id.clone(),
            c_id.clone(),
            0.9,
            vec![Capability::Write {
                resource: "*".to_string(),
            }],
        );

        // Trust-Decay: 0.9 × 0.9 × 0.9 = 0.729
        let effective_trust = del_a.trust_factor * del_b.trust_factor * del_c.trust_factor;
        assert!((effective_trust - 0.729).abs() < 0.001);

        // Alle Delegations sollten gültig sein
        let now = TemporalCoord::now(1, &root_id);
        assert!(del_a.is_valid(&now));
        assert!(del_b.is_valid(&now));
        assert!(del_c.is_valid(&now));

        // Invariant-Check für Trust-Faktoren
        assert!(InvariantChecker::check_delegation_trust_factor(del_a.trust_factor).is_ok());
    }

    /// Test: Context-spezifische Trust-Gewichte
    #[test]
    fn test_context_specific_trust_weights() {
        let trust = TrustVector6D::new(0.9, 0.5, 0.9, 0.9, 0.9, 1.0);

        let default_weights = ContextType::Default.default_weights();
        let finance_weights = ContextType::Finance.default_weights();

        let default_norm = trust.weighted_norm(&default_weights);
        let finance_norm = trust.weighted_norm(&finance_weights);

        // Verschiedene Kontexte sollten verschiedene Normen geben
        assert!(default_norm > 0.0);
        assert!(finance_norm > 0.0);
    }
}

// ============================================================================
// EXECUTION LAYER INTEGRATION
// ============================================================================

mod execution_integration {
    use super::*;

    /// Test: ExecutionContext mit Gas-Verbrauch
    #[test]
    fn test_execution_context_gas_consumption() {
        let mut ctx = ExecutionContext::minimal();

        // Gas verbrauchen
        let result = ctx.consume_gas(100);
        assert!(result.is_ok());

        // Noch mehr verbrauchen
        let result2 = ctx.consume_gas(50);
        assert!(result2.is_ok());
    }

    /// Test: ExecutionContext Execute-Pattern
    #[test]
    fn test_execution_context_execute_pattern() {
        let mut ctx = ExecutionContext::default_for_testing();

        // Monadische Ausführung
        let result = ctx.execute(|ctx| {
            ctx.consume_gas(100)?;
            Ok(42)
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    /// Test: ExecutionContext mit Timeout
    #[test]
    fn test_execution_context_timeout_check() {
        let ctx = ExecutionContext::minimal();

        // Timeout sollte bei neuem Context nicht überschritten sein
        assert!(!ctx.is_timed_out());
    }

    /// Test: ExecutionContext Mana-Verbrauch
    #[test]
    fn test_execution_context_mana_consumption() {
        let mut ctx = ExecutionContext::default_for_testing();

        // Mana verbrauchen
        let result = ctx.consume_mana(100);
        assert!(result.is_ok());
    }
}

// ============================================================================
// STORAGE LAYER INTEGRATION
// ============================================================================

mod storage_integration {
    use super::*;

    /// Test: DecentralizedStorage erstellen und verwenden
    #[test]
    fn test_decentralized_storage_lifecycle() {
        let storage = DecentralizedStorage::open_temporary().expect("Failed to create storage");

        // Identity erstellen
        let identity = storage
            .identities
            .create_identity(DIDNamespace::Self_)
            .expect("Failed to create identity");

        assert!(!identity.did.to_string().is_empty());
        assert!(!identity.public_key.is_empty());

        // Identity abrufen
        let retrieved = storage
            .identities
            .get(&identity.did)
            .expect("Failed to get identity");

        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.did.to_string(), identity.did.to_string());
    }

    /// Test: ContentStore mit CAS
    #[test]
    fn test_content_store_cas() {
        let storage = DecentralizedStorage::open_temporary().expect("Failed to create storage");

        let content = b"Hello, Erynoa!";

        // Store content (returns content-addressed ID)
        let id = storage
            .content
            .put(content.to_vec(), "text/plain", None, vec![])
            .expect("Failed to store");

        // Retrieve by ID
        let retrieved = storage.content.get(&id).expect("Failed to get");
        assert!(retrieved.is_some());

        // Same content should produce same ID (content-addressing)
        let id2 = storage
            .content
            .put(content.to_vec(), "text/plain", None, vec![])
            .expect("Failed to store again");
        assert_eq!(id, id2);

        // Different content should produce different ID
        let different = b"Different content";
        let id3 = storage
            .content
            .put(different.to_vec(), "text/plain", None, vec![])
            .expect("Failed to store different");
        assert_ne!(id, id3);
    }

    /// Test: EventStore mit kausaler Ordnung
    #[test]
    fn test_event_store_causal_order() {
        let storage = DecentralizedStorage::open_temporary().expect("Failed to create storage");

        let actor = DID::new(DIDNamespace::Self_, b"event-store-actor");
        let actor_id = UniversalId::new(UniversalId::TAG_DID, 1, actor.id.as_bytes());

        // Event 1 erstellen
        let event1 = Event::new(
            actor_id.clone(),
            vec![],
            EventPayload::Custom {
                event_type: "test".to_string(),
                data: vec![1],
            },
            1,
        );

        storage
            .events
            .put(event1.clone())
            .expect("Failed to store event 1");

        // Event 2 mit Event 1 als Parent
        let event2 = Event::new(
            actor_id.clone(),
            vec![event1.id.clone()],
            EventPayload::Custom {
                event_type: "test".to_string(),
                data: vec![2],
            },
            2,
        );

        storage
            .events
            .put(event2.clone())
            .expect("Failed to store event 2");

        // Events abrufen
        let retrieved1 = storage
            .events
            .get(&event1.id)
            .expect("Failed to get event 1");
        let retrieved2 = storage
            .events
            .get(&event2.id)
            .expect("Failed to get event 2");

        assert!(retrieved1.is_some());
        assert!(retrieved2.is_some());

        // Kausale Ordnung prüfen
        let r2 = retrieved2.unwrap();
        assert!(r2.event.parents.contains(&event1.id));
    }

    /// Test: TrustStore mit Trust-Relationen
    #[test]
    fn test_trust_store_relations() {
        let storage = DecentralizedStorage::open_temporary().expect("Failed to create storage");

        let did_from = DID::new(DIDNamespace::Self_, b"trust-from");
        let did_to = DID::new(DIDNamespace::Self_, b"trust-to");
        let trust = TrustVector6D::new(0.7, 0.8, 0.6, 0.5, 0.9, 0.85);

        // Trust speichern
        storage
            .trust
            .put(did_from.clone(), did_to.clone(), trust)
            .expect("Failed to set trust");

        // Trust abrufen
        let retrieved = storage
            .trust
            .get(&did_from, &did_to)
            .expect("Failed to get trust");
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert!((retrieved.r - 0.7).abs() < 0.01);
        assert!((retrieved.i - 0.8).abs() < 0.01);
    }
}

// ============================================================================
// PROTECTION LAYER INTEGRATION
// ============================================================================

mod protection_integration {
    use super::*;

    /// Test: AntiCalcification erkennt Macht-Konzentration
    #[test]
    fn test_anti_calcification_detection() {
        let mut anti_calc = AntiCalcification::default();

        // Simuliere mehrere Akteure mit Power
        let actor1 = DID::new(DIDNamespace::Self_, b"actor1");
        let actor2 = DID::new(DIDNamespace::Self_, b"actor2");
        let actor3 = DID::new(DIDNamespace::Self_, b"actor3");

        anti_calc.set_power(actor1.clone(), 100.0);
        anti_calc.set_power(actor2.clone(), 50.0);
        anti_calc.set_power(actor3.clone(), 10.0);

        // Power-Cap berechnen
        let cap = anti_calc.calculate_power_cap();
        assert!(cap > 0.0);

        // Gini-Koeffizient berechnen
        let gini = anti_calc.gini_coefficient();
        assert!(gini >= 0.0 && gini <= 1.0);
    }

    /// Test: DiversityMonitor überwacht System-Diversität
    #[test]
    fn test_diversity_monitor() {
        let mut monitor = DiversityMonitor::default();

        // Verschiedene Kategorien beobachten
        monitor.observe("region", "EU");
        monitor.observe("region", "US");
        monitor.observe("region", "ASIA");
        monitor.observe("region", "EU"); // Wiederholung
        monitor.observe("client", "desktop");
        monitor.observe("client", "mobile");

        // Entropy für Region berechnen
        let entropy = monitor.entropy("region");
        assert!(entropy > 0.0);

        // Normalisierte Entropy
        let normalized = monitor.normalized_entropy("region");
        assert!(normalized >= 0.0 && normalized <= 1.0);
    }

    /// Test: QuadraticGovernance Voting
    #[test]
    fn test_quadratic_governance() {
        let mut governance = QuadraticGovernance::default();

        let voter1 = DID::new(DIDNamespace::Self_, b"voter1");
        let voter2 = DID::new(DIDNamespace::Self_, b"voter2");
        let proposer = DID::new(DIDNamespace::Self_, b"proposer");

        // Voters registrieren
        governance.register_voter(voter1.clone());
        governance.register_voter(voter2.clone());

        // Credits prüfen
        let credits1 = governance.get_credits(&voter1);
        assert!(credits1 > 0);

        // Proposal erstellen
        let proposal = governance.create_proposal(
            "prop-1".to_string(),
            "Test Proposal".to_string(),
            "A test proposal".to_string(),
            proposer.clone(),
        );

        assert_eq!(proposal.id, "prop-1");

        // Quadratische Kosten prüfen
        assert_eq!(QuadraticGovernance::vote_cost(1), 1);
        assert_eq!(QuadraticGovernance::vote_cost(2), 4);
        assert_eq!(QuadraticGovernance::vote_cost(3), 9);
    }
}

// ============================================================================
// CROSS-LAYER INTEGRATION
// ============================================================================

mod cross_layer_integration {
    use super::*;

    /// Test: Vollständiger User-Lifecycle über alle Layer
    #[test]
    fn test_complete_user_lifecycle() {
        // 1. Storage Layer: User erstellen
        let storage = DecentralizedStorage::open_temporary().expect("Failed to create storage");
        let identity = storage
            .identities
            .create_identity(DIDNamespace::Self_)
            .expect("Failed to create identity");

        // 2. Core Layer: Trust initialisieren
        let mut trust_engine = TrustEngine::default();
        trust_engine.initialize_trust_for_did(&identity.did);

        // 3. Protection Layer: Anti-Calcification tracking
        let mut anti_calc = AntiCalcification::default();
        anti_calc.set_power(identity.did.clone(), 1.0);

        // 4. Verifiziere Integration
        let stored_identity = storage.identities.get(&identity.did).unwrap();
        assert!(stored_identity.is_some());

        let trust = trust_engine.get_trust_for_did(&identity.did);
        assert!(trust.is_some());
    }

    /// Test: Event + Trust + Surprisal Integration
    #[test]
    fn test_event_trust_surprisal_integration() {
        // 1. Storage
        let storage = DecentralizedStorage::open_temporary().expect("Failed to create storage");

        // 2. User erstellen
        let identity = storage
            .identities
            .create_identity(DIDNamespace::Self_)
            .expect("Failed to create identity");

        let author_id = UniversalId::new(UniversalId::TAG_DID, 1, identity.did.id.as_bytes());

        // 3. Trust Engine
        let mut trust_engine = TrustEngine::default();
        trust_engine.initialize_trust_for_did(&identity.did);
        let trust = *trust_engine.get_trust_for_did(&identity.did).unwrap();

        // 4. Event erstellen
        let event = Event::new(
            author_id.clone(),
            vec![],
            EventPayload::Custom {
                event_type: "user.action".to_string(),
                data: b"test action".to_vec(),
            },
            1,
        );

        // 5. Event speichern
        storage
            .events
            .put(event.clone())
            .expect("Failed to store event");

        // 6. Surprisal berechnen
        let calc = SurprisalCalculator::new();
        let surprisal = calc.calculate_dampened_surprisal(&event, &trust);

        assert!(surprisal.raw_bits > 0.0);
        assert!(surprisal.trust_norm > 0.0);

        // 7. Verifiziere Event existiert
        let stored = storage.events.get(&event.id).unwrap();
        assert!(stored.is_some());
    }

    /// Test: Consensus + Trust Integration
    #[test]
    fn test_consensus_trust_integration() {
        // 1. Trust Engine mit mehreren Witnesses
        let mut trust_engine = TrustEngine::default();

        let witness1 = DID::new(DIDNamespace::Self_, b"witness1");
        let witness2 = DID::new(DIDNamespace::Self_, b"witness2");
        let witness3 = DID::new(DIDNamespace::Self_, b"witness3");

        trust_engine.initialize_trust_for_did(&witness1);
        trust_engine.initialize_trust_for_did(&witness2);
        trust_engine.initialize_trust_for_did(&witness3);

        // 2. Consensus Engine
        let mut consensus = ConsensusEngine::default();

        // 3. Witnesses mit Trust registrieren
        let trust1 = *trust_engine.get_trust_for_did(&witness1).unwrap();
        let trust2 = *trust_engine.get_trust_for_did(&witness2).unwrap();
        let trust3 = *trust_engine.get_trust_for_did(&witness3).unwrap();

        consensus.register_witness(witness1.clone(), trust1);
        consensus.register_witness(witness2.clone(), trust2);
        consensus.register_witness(witness3.clone(), trust3);

        // 4. Event für Attestation
        let event_id = event_id_from_content(b"consensus-event");

        // 5. Attestations hinzufügen
        let result1 = consensus.add_attestation(event_id.clone(), witness1, "sig1".to_string());
        let result2 = consensus.add_attestation(event_id.clone(), witness2, "sig2".to_string());
        let result3 = consensus.add_attestation(event_id.clone(), witness3, "sig3".to_string());

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());

        // 6. Finalität prüfen
        let finality = consensus.check_finality(&event_id);
        assert!(finality.is_ok());
    }
}

// ============================================================================
// END-TO-END WORKFLOW TESTS
// ============================================================================

mod end_to_end_workflows {
    use super::*;

    /// Test: Realm-Erstellung Workflow
    #[test]
    fn test_realm_creation_workflow() {
        // 1. Storage
        let storage = DecentralizedStorage::open_temporary().expect("Failed to create storage");

        // 2. Realm-Creator erstellen
        let creator_identity = storage
            .identities
            .create_identity(DIDNamespace::Self_)
            .expect("Failed to create creator");

        let creator_id =
            UniversalId::new(UniversalId::TAG_DID, 1, creator_identity.did.id.as_bytes());

        // 3. Realm-ID erstellen
        let realm_id = realm_id_from_name("test-realm");

        // 4. Realm existiert als Trait - wir testen die ID-Erstellung
        assert!(!realm_id.as_bytes().is_empty());

        // 6. Event für Realm-Erstellung
        let event = Event::new(
            creator_id.clone(),
            vec![],
            EventPayload::Custom {
                event_type: "realm.created".to_string(),
                data: realm_id.as_bytes().to_vec(),
            },
            1,
        );

        storage
            .events
            .put(event.clone())
            .expect("Failed to store realm event");

        // 7. Verifizieren
        let stored = storage.events.get(&event.id).unwrap();
        assert!(stored.is_some());
    }

    /// Test: Trust-basierter Zugriff
    #[test]
    fn test_trust_based_access_control() {
        // 1. Trust-Vektoren
        let newcomer_trust = TrustVector6D::NEWCOMER;
        let veteran_trust = TrustVector6D::DEFAULT;

        // 2. Gewichtete Normen berechnen
        let weights = [1.0_f32; 6];
        let newcomer_norm = newcomer_trust.weighted_norm(&weights);
        let veteran_norm = veteran_trust.weighted_norm(&weights);

        // 3. Veteran sollte höhere Norm haben
        assert!(veteran_norm > newcomer_norm);

        // 4. Trust-Gate Simulation
        let required_trust = 0.4;

        let newcomer_passes = (newcomer_norm as f64) >= required_trust;
        let veteran_passes = (veteran_norm as f64) >= required_trust;

        // Veteran should always pass if newcomer does
        assert!(!newcomer_passes || veteran_passes);
    }

    /// Test: Content + Event + Trust Vollständiger Flow
    #[test]
    fn test_complete_content_event_trust_flow() {
        // 1. Setup
        let storage = DecentralizedStorage::open_temporary().expect("Failed to create storage");

        let identity = storage
            .identities
            .create_identity(DIDNamespace::Self_)
            .expect("Failed to create identity");

        let author_id = UniversalId::new(UniversalId::TAG_DID, 1, identity.did.id.as_bytes());

        // 2. Trust initialisieren
        let mut trust_engine = TrustEngine::default();
        trust_engine.initialize_trust_for_did(&identity.did);

        // 3. Content erstellen
        let content = b"User-generated content for integration test";
        let content_id = storage
            .content
            .put(
                content.to_vec(),
                "text/plain",
                Some(identity.did.clone()),
                vec!["test".to_string()],
            )
            .expect("Failed to store content");

        // 4. Event für Content-Erstellung
        let event = Event::new(
            author_id.clone(),
            vec![],
            EventPayload::Custom {
                event_type: "content.created".to_string(),
                data: content_id.as_str().as_bytes().to_vec(),
            },
            1,
        );

        storage
            .events
            .put(event.clone())
            .expect("Failed to store event");

        // 5. Trust aktualisieren (positive action)
        let result = trust_engine.process_event(&event);
        assert!(result.is_ok());

        // 6. Verifiziere alles
        assert!(storage.content.get(&content_id).unwrap().is_some());
        assert!(storage.events.get(&event.id).unwrap().is_some());
    }
}

// ============================================================================
// STRESS TESTS
// ============================================================================

mod stress_tests {
    use super::*;

    /// Test: Viele gleichzeitige Events
    #[test]
    fn test_high_volume_events() {
        let storage = DecentralizedStorage::open_temporary().expect("Failed to create storage");

        let actor = DID::new(DIDNamespace::Self_, b"stress-test-actor");
        let actor_id = UniversalId::new(UniversalId::TAG_DID, 1, actor.id.as_bytes());

        // 500 Events erstellen
        let mut last_event_id = None;
        for i in 0..500 {
            let parents = last_event_id.iter().cloned().collect::<Vec<_>>();

            let event = Event::new(
                actor_id.clone(),
                parents,
                EventPayload::Custom {
                    event_type: "stress.test".to_string(),
                    data: vec![i as u8],
                },
                i,
            );

            storage
                .events
                .put(event.clone())
                .expect("Failed to store event");
            last_event_id = Some(event.id);
        }

        // Letztes Event sollte existieren
        assert!(last_event_id.is_some());
        let stored = storage.events.get(&last_event_id.unwrap()).unwrap();
        assert!(stored.is_some());
    }

    /// Test: Viele DIDs
    #[test]
    fn test_high_volume_identities() {
        let storage = DecentralizedStorage::open_temporary().expect("Failed to create storage");

        // 100 DIDs erstellen
        let mut dids = vec![];
        for _ in 0..100 {
            let identity = storage
                .identities
                .create_identity(DIDNamespace::Self_)
                .expect("Failed to create identity");
            dids.push(identity.did);
        }

        // Stichprobe prüfen
        let sample = &dids[50];
        let retrieved = storage.identities.get(sample).unwrap();
        assert!(retrieved.is_some());
    }
}

// ============================================================================
// INVARIANT TESTS
// ============================================================================

mod invariant_tests {
    use super::*;

    /// Test: Trust-Vektor Invarianten (Κ2-Κ5)
    #[test]
    fn test_trust_vector_invariants() {
        // Alle Dimensionen zwischen 0 und 1
        let trust = TrustVector6D::new(0.5, 0.6, 0.7, 0.8, 0.9, 1.0);

        assert!(trust.r >= 0.0 && trust.r <= 1.0);
        assert!(trust.i >= 0.0 && trust.i <= 1.0);
        assert!(trust.c >= 0.0 && trust.c <= 1.0);
        assert!(trust.p >= 0.0 && trust.p <= 1.0);
        assert!(trust.v >= 0.0 && trust.v <= 1.0);
        assert!(trust.omega >= 0.0 && trust.omega <= 1.0);

        // Gewichtete Norm ist nicht-negativ (kann > 1 sein je nach Gewichten)
        let weights = [1.0_f32; 6];
        let norm = trust.weighted_norm(&weights);
        assert!(norm >= 0.0);

        // Mit normalisierten Gewichten (Summe = 1)
        let normalized_weights = [1.0 / 6.0_f32; 6];
        let normalized_norm = trust.weighted_norm(&normalized_weights);
        assert!(normalized_norm >= 0.0);
    }

    /// Test: Delegation Invarianten (Κ8)
    #[test]
    fn test_delegation_invariants() {
        // Trust-Faktor muss zwischen 0 (exclusive) und 1 (inclusive) sein
        assert!(InvariantChecker::check_delegation_trust_factor(0.5).is_ok());
        assert!(InvariantChecker::check_delegation_trust_factor(0.0).is_err());
        assert!(InvariantChecker::check_delegation_trust_factor(1.5).is_err());
        assert!(InvariantChecker::check_delegation_trust_factor(1.0).is_ok());
    }

    /// Test: UniversalId Invarianten
    #[test]
    fn test_universal_id_invariants() {
        let id = UniversalId::new(UniversalId::TAG_DID, 1, b"test");

        // Tag sollte DID sein
        assert_eq!(id.type_tag(), UniversalId::TAG_DID);

        // Version sollte 1 sein
        assert_eq!(id.version(), 1);

        // ID sollte nicht leer sein
        assert!(!id.as_bytes().is_empty());
    }

    /// Test: Event-Kausalität Invarianten (Κ9)
    #[test]
    fn test_event_causality_invariants() {
        // Parent muss zeitlich vor Child sein
        let parent_coord = TemporalCoord::new(1000, 5, 42);
        let child_coord = TemporalCoord::new(1001, 6, 42);

        assert!(InvariantChecker::check_causal_order(&child_coord, &parent_coord).is_ok());
        assert!(InvariantChecker::check_causal_order(&parent_coord, &child_coord).is_err());
    }

    /// Test: Cost-Algebra Invarianten (Κ15)
    #[test]
    fn test_cost_algebra_invariants() {
        let c1 = Cost::new(10, 5, 0.1);
        let c2 = Cost::new(20, 10, 0.2);
        let c3 = Cost::new(30, 15, 0.15);

        // Assoziativität: (c1 ∘ c2) ∘ c3 = c1 ∘ (c2 ∘ c3)
        let left = c1.seq(c2).seq(c3);
        let right = c1.seq(c2.seq(c3));

        assert_eq!(left.gas, right.gas);
        assert_eq!(left.mana, right.mana);
        assert!((left.trust_risk - right.trust_risk).abs() < 0.0001);

        // Neutrales Element: c ∘ 0 = c
        let with_zero = c1.seq(Cost::ZERO);
        assert_eq!(with_zero.gas, c1.gas);

        // Invariant-Check
        assert!(InvariantChecker::check_cost_algebra(c1, c2, c3).is_ok());
    }
}

// ============================================================================
// ERROR CASES AND EDGE CASES
// ============================================================================

mod error_cases {
    use super::*;

    /// Test: Invalid Trust-Faktor (0.0) wird abgelehnt (Κ8)
    #[test]
    fn test_invalid_trust_factor_zero() {
        let result = InvariantChecker::check_delegation_trust_factor(0.0);
        assert!(result.is_err(), "Trust factor 0.0 should be rejected");
    }

    /// Test: Invalid Trust-Faktor (>1.0) wird abgelehnt (Κ8)
    #[test]
    fn test_invalid_trust_factor_above_one() {
        let result = InvariantChecker::check_delegation_trust_factor(1.5);
        assert!(result.is_err(), "Trust factor > 1.0 should be rejected");

        let result2 = InvariantChecker::check_delegation_trust_factor(100.0);
        assert!(result2.is_err(), "Trust factor 100.0 should be rejected");
    }

    /// Test: Negative Trust-Faktoren werden abgelehnt (Κ8)
    #[test]
    fn test_invalid_trust_factor_negative() {
        let result = InvariantChecker::check_delegation_trust_factor(-0.5);
        assert!(result.is_err(), "Negative trust factor should be rejected");
    }

    /// Test: Kausale Ordnung - gleichzeitige Events verletzen Kausalität (Κ9)
    #[test]
    fn test_causal_order_same_timestamp_violation() {
        let coord = TemporalCoord::new(1000, 5, 42);

        // Gleicher Timestamp als Parent ist ungültig
        let result = InvariantChecker::check_causal_order(&coord, &coord);
        assert!(result.is_err(), "Same timestamp should violate causality");
    }

    /// Test: Kausale Ordnung - Parent nach Event verletzt Kausalität (Κ9)
    #[test]
    fn test_causal_order_parent_after_event() {
        let event = TemporalCoord::new(1000, 5, 42);
        let parent_in_future = TemporalCoord::new(2000, 10, 42);

        let result = InvariantChecker::check_causal_order(&event, &parent_in_future);
        assert!(result.is_err(), "Parent in future should violate causality");
    }

    /// Test: Gas-Exhaustion in ExecutionContext
    #[test]
    fn test_gas_exhaustion_error() {
        let mut ctx = ExecutionContext::minimal(); // Nur 10_000 Gas

        // Versuche mehr Gas zu verbrauchen als verfügbar
        let result = ctx.consume_gas(100_000);

        assert!(result.is_err());
        match result {
            Err(erynoa_api::execution::ExecutionError::GasExhausted {
                required,
                available,
            }) => {
                assert_eq!(required, 100_000);
                assert_eq!(available, 10_000);
            }
            _ => panic!("Expected GasExhausted error"),
        }
    }

    /// Test: Mana-Exhaustion in ExecutionContext
    #[test]
    fn test_mana_exhaustion_error() {
        let mut ctx = ExecutionContext::minimal(); // Nur 1_000 Mana

        // Versuche mehr Mana zu verbrauchen als verfügbar
        let result = ctx.consume_mana(10_000);

        assert!(result.is_err());
        match result {
            Err(erynoa_api::execution::ExecutionError::ManaExhausted {
                required,
                available,
            }) => {
                assert_eq!(required, 10_000);
                assert_eq!(available, 1_000);
            }
            _ => panic!("Expected ManaExhausted error"),
        }
    }

    /// Test: Vollständige Gas-Erschöpfung blockiert weitere Operationen
    #[test]
    fn test_complete_gas_exhaustion_blocks_operations() {
        let mut ctx = ExecutionContext::minimal();

        // Verbrauche alles Gas
        ctx.consume_gas(10_000)
            .expect("Initial consumption should work");

        // Jede weitere Operation sollte fehlschlagen
        let result = ctx.consume_gas(1);
        assert!(result.is_err());

        // Execute sollte auch fehlschlagen
        let exec_result = ctx.execute(|_| Ok(42));
        assert!(exec_result.is_err());
    }

    /// Test: Trust-Vektor Clamping bei ungültigen Werten
    #[test]
    fn test_trust_vector_clamping() {
        // TrustVector6D::new clamped automatisch
        let trust = TrustVector6D::new(2.0, -0.5, 0.5, 0.5, 0.5, 0.5);

        // Werte sollten auf [0, 1] begrenzt sein
        assert!(trust.r >= 0.0 && trust.r <= 1.0);
        assert!(trust.i >= 0.0 && trust.i <= 1.0);
    }

    /// Test: Event ohne Parents (Genesis-Event) ist gültig
    #[test]
    fn test_genesis_event_no_parents() {
        let actor = DID::new(DIDNamespace::Self_, b"genesis-actor");
        let actor_id = UniversalId::new(UniversalId::TAG_DID, 1, actor.id.as_bytes());

        let genesis_event = Event::new(
            actor_id,
            vec![], // Keine Parents
            EventPayload::Custom {
                event_type: "genesis".to_string(),
                data: vec![],
            },
            0,
        );

        // Genesis-Event sollte valide sein
        assert!(genesis_event.parents.is_empty());
        assert!(!genesis_event.id.as_bytes().is_empty());
    }

    /// Test: Duplizierte Event-IDs (deterministisch)
    #[test]
    fn test_duplicate_event_ids_deterministic() {
        let actor = DID::new(DIDNamespace::Self_, b"dup-actor");
        let actor_id = UniversalId::new(UniversalId::TAG_DID, 1, actor.id.as_bytes());

        // Zwei identische Events sollten gleiche ID haben
        let event1 = Event::new(
            actor_id.clone(),
            vec![],
            EventPayload::Custom {
                event_type: "test".to_string(),
                data: vec![1, 2, 3],
            },
            1,
        );

        let event2 = Event::new(
            actor_id,
            vec![],
            EventPayload::Custom {
                event_type: "test".to_string(),
                data: vec![1, 2, 3],
            },
            1,
        );

        // Gleicher Content → gleiche ID (Content-Addressing)
        assert_eq!(event1.id, event2.id);
    }

    /// Test: Cost mit extremen Werten
    #[test]
    fn test_cost_extreme_values() {
        // Max-Werte
        let max_cost = Cost::new(u64::MAX, u64::MAX, 1.0);
        assert_eq!(max_cost.gas, u64::MAX);
        assert_eq!(max_cost.mana, u64::MAX);
        assert_eq!(max_cost.trust_risk, 1.0);

        // Zero-Cost
        let zero = Cost::ZERO;
        assert_eq!(zero.gas, 0);
        assert_eq!(zero.mana, 0);
        assert_eq!(zero.trust_risk, 0.0);

        // Seq mit Zero ist Identity
        let with_zero = max_cost.seq(zero);
        assert_eq!(with_zero.gas, u64::MAX);
    }

    /// Test: TemporalCoord Ordnung
    #[test]
    fn test_temporal_coord_ordering_edge_cases() {
        // Gleiche wall_time, unterschiedliche lamport
        let t1 = TemporalCoord::new(1000, 5, 1);
        let t2 = TemporalCoord::new(1000, 10, 1);

        // t2 ist "nach" t1 (höherer Lamport)
        assert!(t2 > t1);

        // Gleiche wall_time und lamport, unterschiedlicher node_hash
        let t3 = TemporalCoord::new(1000, 5, 2);
        let t4 = TemporalCoord::new(1000, 5, 1);

        // Sortierung nach node_hash als Tiebreaker
        assert_ne!(t3, t4);
    }
}

// ============================================================================
// EXTENDED CONSENSUS TESTS
// ============================================================================

mod extended_consensus_tests {
    use super::*;

    /// Test: Ungenügende Trust-Summe erreicht keine Finalität
    #[test]
    fn test_insufficient_trust_no_finality() {
        let mut trust_engine = TrustEngine::default();
        let mut consensus = ConsensusEngine::default();

        // Nur 1 Witness (braucht mindestens 3)
        let witness = DID::new(DIDNamespace::Self_, b"lone-witness");
        trust_engine.initialize_trust_for_did(&witness);
        let trust = *trust_engine.get_trust_for_did(&witness).unwrap();
        consensus.register_witness(witness.clone(), trust);

        // Event erstellen
        let event_id = event_id_from_content(b"insufficient-trust-event");

        // Attestation hinzufügen
        let _ = consensus.add_attestation(event_id.clone(), witness, "sig".to_string());

        // Finalität prüfen
        let check = consensus.check_finality(&event_id).unwrap();

        assert!(
            !check.reached,
            "Finality should not be reached with only 1 witness"
        );
        assert_eq!(check.witness_count, 1);
    }

    /// Test: Witness mit zu niedrigem Trust wird abgelehnt
    #[test]
    fn test_low_trust_witness_rejected() {
        let mut consensus = ConsensusEngine::default();

        let witness = DID::new(DIDNamespace::Self_, b"low-trust-witness");

        // Registriere mit sehr niedrigem Trust (unter Minimum 0.5)
        let low_trust = TrustVector6D::new(0.1, 0.1, 0.1, 0.1, 0.1, 0.1);
        consensus.register_witness(witness.clone(), low_trust);

        // Attestation sollte fehlschlagen
        let event_id = event_id_from_content(b"low-trust-event");
        let result = consensus.add_attestation(event_id, witness, "sig".to_string());

        assert!(result.is_err());
    }

    /// Test: Unregistrierter Witness wird abgelehnt
    #[test]
    fn test_unregistered_witness_rejected() {
        let consensus = ConsensusEngine::default();

        let unknown_witness = DID::new(DIDNamespace::Self_, b"unknown-witness");
        let event_id = event_id_from_content(b"unauthorized-event");

        // Cast to mutable for test (in real code this would be different)
        let mut consensus = consensus;
        let result = consensus.add_attestation(event_id, unknown_witness, "sig".to_string());

        assert!(result.is_err());
    }

    /// Test: Exakt k=3 Witnesses erreichen Finalität
    #[test]
    fn test_exact_threshold_witnesses() {
        let mut trust_engine = TrustEngine::default();
        let mut consensus = ConsensusEngine::default();

        // Exakt 3 Witnesses
        let witnesses: Vec<_> = (0..3)
            .map(|i| DID::new(DIDNamespace::Self_, format!("witness-{i}").as_bytes()))
            .collect();

        for w in &witnesses {
            trust_engine.initialize_trust_for_did(w);
            let trust = *trust_engine.get_trust_for_did(w).unwrap();
            consensus.register_witness(w.clone(), trust);
        }

        let event_id = event_id_from_content(b"threshold-event");

        // Alle 3 attestieren
        for w in &witnesses {
            let _ = consensus.add_attestation(event_id.clone(), w.clone(), "sig".to_string());
        }

        let check = consensus.check_finality(&event_id).unwrap();

        assert!(
            check.reached,
            "Finality should be reached with exactly 3 witnesses"
        );
        assert_eq!(check.witness_count, 3);
    }

    /// Test: Finalität mit unterschiedlichen Trust-Levels
    #[test]
    fn test_finality_with_varied_trust_levels() {
        let mut consensus = ConsensusEngine::default();

        // Witnesses mit unterschiedlichen Trust-Levels
        let w1 = DID::new(DIDNamespace::Self_, b"high-trust");
        let w2 = DID::new(DIDNamespace::Self_, b"medium-trust");
        let w3 = DID::new(DIDNamespace::Self_, b"low-but-valid");

        consensus.register_witness(w1.clone(), TrustVector6D::new(0.9, 0.9, 0.9, 0.9, 0.9, 0.9));
        consensus.register_witness(w2.clone(), TrustVector6D::new(0.7, 0.7, 0.7, 0.7, 0.7, 0.7));
        consensus.register_witness(w3.clone(), TrustVector6D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5));

        let event_id = event_id_from_content(b"varied-trust-event");

        let _ = consensus.add_attestation(event_id.clone(), w1, "sig1".to_string());
        let _ = consensus.add_attestation(event_id.clone(), w2, "sig2".to_string());
        let _ = consensus.add_attestation(event_id.clone(), w3, "sig3".to_string());

        let check = consensus.check_finality(&event_id).unwrap();

        assert!(check.reached);
        assert!(check.total_trust > 0.0);
        assert!(check.trust_ratio > 0.0);
    }

    /// Test: Revert-Wahrscheinlichkeit sinkt mit mehr Witnesses
    #[test]
    fn test_revert_probability_decreases() {
        let mut consensus = ConsensusEngine::default();

        // Viele hochvertrauenswürdige Witnesses
        let witnesses: Vec<_> = (0..10)
            .map(|i| DID::new(DIDNamespace::Self_, format!("trusted-{i}").as_bytes()))
            .collect();

        for w in &witnesses {
            consensus.register_witness(w.clone(), TrustVector6D::new(0.9, 0.9, 0.9, 0.9, 0.9, 0.9));
        }

        let event_id = event_id_from_content(b"revert-prob-event");

        // Attestiere nach und nach
        let mut last_revert_prob = 1.0;
        for w in &witnesses {
            let _ = consensus.add_attestation(event_id.clone(), w.clone(), "sig".to_string());

            let check = consensus.check_finality(&event_id).unwrap();

            // Revert-Wahrscheinlichkeit sollte sinken
            assert!(
                check.revert_probability <= last_revert_prob,
                "Revert probability should decrease: {} <= {}",
                check.revert_probability,
                last_revert_prob
            );
            last_revert_prob = check.revert_probability;
        }

        // Am Ende sollte sie sehr niedrig sein
        let final_check = consensus.check_finality(&event_id).unwrap();
        assert!(final_check.revert_probability < 1e-10);
    }

    /// Test: Stats werden korrekt berechnet
    #[test]
    fn test_consensus_stats() {
        let mut consensus = ConsensusEngine::default();

        // Registriere Witnesses
        for i in 0..5 {
            let w = DID::new(DIDNamespace::Self_, format!("stats-witness-{i}").as_bytes());
            consensus.register_witness(w, TrustVector6D::DEFAULT);
        }

        let stats = consensus.stats();
        assert_eq!(stats.registered_witnesses, 5);
        assert_eq!(stats.total_attestations, 0);
        assert_eq!(stats.events_with_attestations, 0);
    }
}

// ============================================================================
// STORAGE PERSISTENCE TESTS
// ============================================================================

mod storage_persistence_tests {
    use super::*;

    /// Test: In-Memory Storage behält Daten über Operationen hinweg
    #[test]
    fn test_storage_data_retention() {
        let storage = DecentralizedStorage::open_temporary().expect("Failed to create storage");

        // Erstelle mehrere Entitäten
        let identity1 = storage
            .identities
            .create_identity(DIDNamespace::Self_)
            .expect("Failed to create identity 1");
        let identity2 = storage
            .identities
            .create_identity(DIDNamespace::Guild)
            .expect("Failed to create identity 2");

        let content1 = b"Content 1";
        let content2 = b"Content 2";

        let cid1 = storage
            .content
            .put(content1.to_vec(), "text/plain", None, vec![])
            .expect("Failed to store content 1");
        let cid2 = storage
            .content
            .put(content2.to_vec(), "text/plain", None, vec![])
            .expect("Failed to store content 2");

        // Verifiziere alle Daten noch vorhanden
        assert!(storage.identities.get(&identity1.did).unwrap().is_some());
        assert!(storage.identities.get(&identity2.did).unwrap().is_some());
        assert!(storage.content.get(&cid1).unwrap().is_some());
        assert!(storage.content.get(&cid2).unwrap().is_some());
    }

    /// Test: Trust-Relationen bleiben erhalten
    #[test]
    fn test_trust_relations_retention() {
        let storage = DecentralizedStorage::open_temporary().expect("Failed to create storage");

        let from1 = DID::new(DIDNamespace::Self_, b"from1");
        let from2 = DID::new(DIDNamespace::Self_, b"from2");
        let to = DID::new(DIDNamespace::Self_, b"to");

        let trust1 = TrustVector6D::new(0.8, 0.8, 0.8, 0.8, 0.8, 0.8);
        let trust2 = TrustVector6D::new(0.6, 0.6, 0.6, 0.6, 0.6, 0.6);

        storage
            .trust
            .put(from1.clone(), to.clone(), trust1)
            .expect("Failed to set trust 1");
        storage
            .trust
            .put(from2.clone(), to.clone(), trust2)
            .expect("Failed to set trust 2");

        // Verifiziere beide Trust-Relationen
        let r1 = storage.trust.get(&from1, &to).unwrap().unwrap();
        let r2 = storage.trust.get(&from2, &to).unwrap().unwrap();

        assert!((r1.r - 0.8).abs() < 0.01);
        assert!((r2.r - 0.6).abs() < 0.01);
    }

    /// Test: Event-Kette bleibt konsistent
    #[test]
    fn test_event_chain_consistency() {
        let storage = DecentralizedStorage::open_temporary().expect("Failed to create storage");

        let actor = DID::new(DIDNamespace::Self_, b"chain-actor");
        let actor_id = UniversalId::new(UniversalId::TAG_DID, 1, actor.id.as_bytes());

        // Erstelle Event-Kette
        let mut events: Vec<Event> = vec![];

        for i in 0..10 {
            let parents = if events.is_empty() {
                vec![]
            } else {
                vec![events.last().unwrap().id.clone()]
            };

            let event = Event::new(
                actor_id.clone(),
                parents,
                EventPayload::Custom {
                    event_type: format!("chain-event-{i}"),
                    data: vec![i as u8],
                },
                i,
            );

            storage
                .events
                .put(event.clone())
                .expect("Failed to store event");
            events.push(event);
        }

        // Verifiziere Kette
        for (i, event) in events.iter().enumerate() {
            let stored = storage.events.get(&event.id).unwrap().unwrap();

            if i > 0 {
                assert!(!stored.event.parents.is_empty());
                assert_eq!(stored.event.parents[0], events[i - 1].id);
            }
        }
    }

    /// Test: Content-Deduplication funktioniert
    #[test]
    fn test_content_deduplication() {
        let storage = DecentralizedStorage::open_temporary().expect("Failed to create storage");

        let content = b"Duplicate content";

        // Store same content multiple times
        let id1 = storage
            .content
            .put(
                content.to_vec(),
                "text/plain",
                None,
                vec!["tag1".to_string()],
            )
            .expect("Store 1");
        let id2 = storage
            .content
            .put(
                content.to_vec(),
                "text/plain",
                None,
                vec!["tag2".to_string()],
            )
            .expect("Store 2");
        let id3 = storage
            .content
            .put(content.to_vec(), "text/plain", None, vec![])
            .expect("Store 3");

        // Alle sollten die gleiche Content-ID haben
        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
    }
}

// ============================================================================
// EXECUTION CONTEXT DEEP TESTS
// ============================================================================

mod execution_context_deep_tests {
    use super::*;

    /// Test: Execute mit Gas-Verbrauch tracking
    #[test]
    fn test_execute_with_gas_tracking() {
        let mut ctx = ExecutionContext::default_for_testing();
        let initial_gas = ctx.gas_remaining;

        let result = ctx.execute(|ctx| {
            ctx.consume_gas(100)?;
            ctx.consume_gas(50)?;
            Ok(42)
        });

        assert!(result.is_ok());
        assert_eq!(ctx.gas_remaining, initial_gas - 150);
        assert_eq!(ctx.accumulated_cost.gas, 150);
    }

    /// Test: Nested Execute mit Rollback-Simulation
    #[test]
    fn test_nested_execute_partial_failure() {
        let mut ctx = ExecutionContext::default_for_testing();

        // Erste Operation erfolgreich
        let r1 = ctx.execute(|ctx| {
            ctx.consume_gas(100)?;
            Ok(1)
        });
        assert!(r1.is_ok());

        let gas_after_first = ctx.gas_remaining;

        // Zweite Operation fehlschlägt
        let r2: erynoa_api::execution::ExecutionResult<()> = ctx.execute(|ctx| {
            ctx.consume_gas(100)?;
            // Simuliere Fehler
            Err(erynoa_api::execution::ExecutionError::PolicyViolation {
                policy: "test".to_string(),
                reason: "simulated failure".to_string(),
            })
        });
        assert!(r2.is_err());

        // Gas wurde trotzdem verbraucht (kein automatischer Rollback)
        assert!(ctx.gas_remaining < gas_after_first);
    }

    /// Test: Timeout-Check ist korrekt
    #[test]
    fn test_timeout_not_exceeded_initially() {
        let ctx = ExecutionContext::minimal();

        // Frisch erstellter Context sollte nicht getimed out sein
        assert!(!ctx.is_timed_out());
    }

    /// Test: Cost-Tracking akkumuliert korrekt
    #[test]
    fn test_cost_tracking_accumulation() {
        let mut ctx = ExecutionContext::default_for_testing();

        // Mehrere Costs tracken
        ctx.track_cost(Cost::new(10, 5, 0.1));
        ctx.track_cost(Cost::new(20, 10, 0.2));

        // Akkumulierte Kosten sollten addiert sein
        assert_eq!(ctx.accumulated_cost.gas, 30);
        assert_eq!(ctx.accumulated_cost.mana, 15);
    }

    /// Test: Metadata kann gesetzt und gelesen werden
    #[test]
    fn test_metadata_operations() {
        let mut ctx = ExecutionContext::default_for_testing();

        ctx.metadata
            .insert("key1".to_string(), "value1".to_string());
        ctx.metadata
            .insert("key2".to_string(), "value2".to_string());

        assert_eq!(ctx.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(ctx.metadata.get("key2"), Some(&"value2".to_string()));
        assert_eq!(ctx.metadata.get("nonexistent"), None);
    }

    /// Test: Gas-Erschöpfung während Execute
    #[test]
    fn test_gas_exhaustion_during_execute() {
        let mut ctx = ExecutionContext::minimal(); // 10_000 Gas

        let result = ctx.execute(|ctx| {
            // Verbrauche alles
            ctx.consume_gas(10_000)?;
            // Versuche noch mehr
            ctx.consume_gas(1)?;
            Ok(())
        });

        assert!(result.is_err());
    }

    /// Test: Mana und Gas kombiniert verbrauchen
    #[test]
    fn test_combined_resource_consumption() {
        let mut ctx = ExecutionContext::default_for_testing();

        let cost = Cost::new(500, 100, 0.05);
        let result = ctx.consume_cost(cost);

        assert!(result.is_ok());
        assert!(ctx.gas_remaining < ctx.gas_initial);
        assert!(ctx.mana_remaining < ctx.mana_initial);
        assert!(ctx.accumulated_cost.trust_risk > 0.0);
    }

    /// Test: Trust-Context ist korrekt gesetzt
    #[test]
    fn test_trust_context_access() {
        let ctx = ExecutionContext::default_for_testing();

        // Trust-Context sollte Executor-ID haben
        assert!(!ctx.trust_context.executor_id.as_bytes().is_empty());
    }

    /// Test: Lamport-Clock wird korrekt aktualisiert
    #[test]
    fn test_lamport_clock_updates() {
        let mut ctx = ExecutionContext::default_for_testing();

        let initial_lamport = ctx.state.lamport;
        ctx.state.tick();
        let after_advance = ctx.state.lamport;

        assert!(after_advance > initial_lamport);
    }

    /// Test: Execute-Sequenz verarbeitet mehrere Operationen
    #[test]
    fn test_execute_sequence() {
        let mut ctx = ExecutionContext::default_for_testing();

        let ops: Vec<
            Box<dyn FnOnce(&mut ExecutionContext) -> erynoa_api::execution::ExecutionResult<i32>>,
        > = vec![
            Box::new(|ctx| {
                ctx.consume_gas(10)?;
                Ok(1)
            }),
            Box::new(|ctx| {
                ctx.consume_gas(20)?;
                Ok(2)
            }),
            Box::new(|ctx| {
                ctx.consume_gas(30)?;
                Ok(3)
            }),
        ];

        let results = ctx.execute_seq(ops);
        assert!(results.is_ok());

        let values = results.unwrap();
        assert_eq!(values, vec![1, 2, 3]);
        assert_eq!(ctx.accumulated_cost.gas, 60);
    }
}
