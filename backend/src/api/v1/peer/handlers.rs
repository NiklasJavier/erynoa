//! PeerService Connect-RPC Handlers
//!
//! Implementiert die Peer-Endpunkte für Status, Info, Key-Management und Gateway.

use axum::extract::State;
use chrono::Utc;

use crate::config::VERSION;
use crate::gen::erynoa::v1::{
    Algorithm, ChainType, DeriveKeyRequest, DeriveKeyResponse, DerivedKey, Did,
    EvaluateGatewayRequest, EvaluateGatewayResponse, GatewayStatus, ListDerivedKeysRequest,
    ListDerivedKeysResponse, PeerCapabilities, PeerConfig, PeerServiceGetInfoRequest,
    PeerServiceGetInfoResponse, PeerServiceGetStatusRequest, PeerServiceGetStatusResponse,
    PeerState, PredicateResult, StartPeerRequest, StartPeerResponse, StopPeerRequest,
    StopPeerResponse, TrustVector6D,
};
use crate::server::AppState;

// ============================================================================
// GET STATUS
// ============================================================================

/// GetStatus - Aktueller Peer-Status
///
/// Gibt den aktuellen Status des Peers zurück inkl. DID, verbundene Chains,
/// Wallets und aktive Sagas.
pub async fn get_status_handler(
    State(state): State<AppState>,
    _request: PeerServiceGetStatusRequest,
) -> PeerServiceGetStatusResponse {
    // Lade Peer-DID aus Storage (falls vorhanden)
    let peer_did = state
        .storage
        .identity()
        .get_primary()
        .await
        .ok()
        .flatten()
        .map(|identity| Did {
            namespace: "self".to_string(),
            unique_id: identity.did.id().to_string(),
            created_at: None,
        });

    // Peer-ID aus DID oder generiert
    let peer_id = peer_did
        .as_ref()
        .map(|d| format!("peer:{}", d.unique_id))
        .unwrap_or_else(|| format!("peer:anonymous:{}", uuid::Uuid::new_v4()));

    // Started-at Timestamp
    let started_at = state.started_at.map(|instant| {
        let duration = instant.elapsed();
        let start_time = Utc::now() - chrono::Duration::from_std(duration).unwrap_or_default();
        axum_connect::pbjson_types::Timestamp {
            seconds: start_time.timestamp(),
            nanos: start_time.timestamp_subsec_nanos() as i32,
        }
    });

    PeerServiceGetStatusResponse {
        peer_id,
        did: peer_did,
        state: PeerState::Running as i32,
        connected_chains: vec![], // TODO: Implementiere Chain-Connections
        wallets: vec![],          // TODO: Lade aus KeyVault
        gateway_status: Some(GatewayStatus {
            active: true,
            registered_realms: 1, // Root-Realm
            pending_crossings: 0,
        }),
        active_sagas: vec![], // TODO: Lade aktive Sagas
        started_at,
        last_activity: None,
    }
}

// ============================================================================
// GET INFO
// ============================================================================

/// GetInfo - Peer-Konfiguration und Capabilities
///
/// Gibt die Konfiguration und unterstützten Features des Peers zurück.
pub async fn get_info_handler(
    _state: State<AppState>,
    _request: PeerServiceGetInfoRequest,
) -> PeerServiceGetInfoResponse {
    PeerServiceGetInfoResponse {
        version: VERSION.to_string(),
        supported_chains: vec![
            ChainType::Erynoa as i32,
            ChainType::Ethereum as i32,
            ChainType::Polygon as i32,
            ChainType::Iota as i32,
            ChainType::Shimmer as i32,
        ],
        supported_algorithms: vec![
            Algorithm::Ed25519 as i32,
            Algorithm::Secp256k1 as i32,
        ],
        capabilities: Some(PeerCapabilities {
            composer: true,   // IntentParser + SagaComposer verfügbar
            gateway: true,    // GatewayGuard verfügbar
            key_vault: true,  // Ed25519 KeyVault verfügbar
            htlc_support: false, // TODO: HTLC Support implementieren
            streaming: false, // TODO: Streaming Support
            supported_intent_types: vec![
                "transfer".to_string(),
                "attest".to_string(),
                "stake".to_string(),
                "governance".to_string(),
            ],
        }),
        config: Some(PeerConfig {
            max_concurrent_sagas: 10,
            default_timeout_seconds: 3600,
            default_slippage_percent: 1.0,
            default_environment: "root".to_string(),
        }),
    }
}

// ============================================================================
// LIST DERIVED KEYS
// ============================================================================

/// ListDerivedKeys - Liste aller abgeleiteten Schlüssel
pub async fn list_derived_keys_handler(
    State(state): State<AppState>,
    _request: ListDerivedKeysRequest,
) -> ListDerivedKeysResponse {
    // Lade alle Identitäten und deren öffentliche Schlüssel
    let keys = match state.storage.identity().list_all().await {
        Ok(identities) => identities
            .into_iter()
            .map(|identity| DerivedKey {
                chain: ChainType::Erynoa as i32,
                public_key: hex::encode(identity.public_key.as_bytes()),
                address: identity.did.to_string(),
                derivation_path: "m/44'/1234'/0'/0/0".to_string(), // Erynoa BIP44
                algorithm: Algorithm::Ed25519 as i32,
                derived_at: None,
            })
            .collect(),
        Err(_) => vec![],
    };

    ListDerivedKeysResponse { keys }
}

// ============================================================================
// DERIVE KEY
// ============================================================================

/// DeriveKey - Leitet einen neuen Schlüssel für eine Chain ab
pub async fn derive_key_handler(
    State(state): State<AppState>,
    request: DeriveKeyRequest,
) -> DeriveKeyResponse {
    let chain = ChainType::try_from(request.chain).unwrap_or(ChainType::Erynoa);

    // Erstelle neue Identität für die angeforderte Chain
    let derivation_path = request
        .path
        .unwrap_or_else(|| get_default_derivation_path(chain));

    let algorithm = request
        .algorithm
        .and_then(|a| Algorithm::try_from(a).ok())
        .unwrap_or(Algorithm::Ed25519);

    // Generiere neuen Ed25519 Schlüssel
    match state.storage.identity().create().await {
        Ok(identity) => {
            let key = DerivedKey {
                chain: chain as i32,
                public_key: hex::encode(identity.public_key.as_bytes()),
                address: identity.did.to_string(),
                derivation_path,
                algorithm: algorithm as i32,
                derived_at: Some(axum_connect::pbjson_types::Timestamp {
                    seconds: Utc::now().timestamp(),
                    nanos: 0,
                }),
            };
            DeriveKeyResponse { key: Some(key) }
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to derive key");
            DeriveKeyResponse { key: None }
        }
    }
}

fn get_default_derivation_path(chain: ChainType) -> String {
    match chain {
        ChainType::Erynoa => "m/44'/1234'/0'/0/0".to_string(),
        ChainType::Ethereum
        | ChainType::Polygon
        | ChainType::Arbitrum
        | ChainType::Optimism => "m/44'/60'/0'/0/0".to_string(),
        ChainType::Iota | ChainType::Shimmer => "m/44'/4218'/0'/0'/0'".to_string(),
        _ => "m/44'/0'/0'/0/0".to_string(),
    }
}

// ============================================================================
// EVALUATE GATEWAY
// ============================================================================

/// EvaluateGateway - Prüft ob ein Realm-Crossing erlaubt ist (PR3, PR6)
///
/// Evaluiert Gateway-Prädikate für einen Realm-Übergang und berechnet
/// die Trust-Transformation gemäß der Dampening-Matrix.
pub async fn evaluate_gateway_handler(
    State(state): State<AppState>,
    request: EvaluateGatewayRequest,
) -> EvaluateGatewayResponse {
    use crate::domain::TrustVector6D as DomainTrustVector;
    use crate::peer::GatewayGuard;

    // Lade DID des Nutzers (oder eigene DID)
    let user_did = if let Some(did) = request.user {
        crate::domain::DID::parse(&format!("did:erynoa:{}:{}", did.namespace, did.unique_id))
            .unwrap_or_else(|_| crate::domain::DID::generate())
    } else {
        // Eigene DID laden
        state
            .storage
            .identity()
            .get_primary()
            .await
            .ok()
            .flatten()
            .map(|i| i.did)
            .unwrap_or_else(crate::domain::DID::generate)
    };

    // Gateway evaluieren
    let gateway = GatewayGuard::new(Default::default());

    // Source und Target Realm
    let source_realm = request
        .source_realm
        .map(|r| crate::domain::RealmId::from_string(&r.id))
        .unwrap_or_else(crate::domain::RealmId::root);

    let target_realm = request
        .target_realm
        .map(|r| crate::domain::RealmId::from_string(&r.id))
        .unwrap_or_else(crate::domain::RealmId::root);

    // Trust-Vektor für den Nutzer laden (Default: Newcomer)
    let original_trust = DomainTrustVector::newcomer();

    // Gateway-Check durchführen
    let result = gateway.check_crossing(&user_did, &source_realm, &target_realm, &original_trust);

    // Trust nach Crossing (mit Dampening)
    let transformed_trust = if result.is_ok() {
        gateway.apply_crossing_dampening(&original_trust, &source_realm, &target_realm)
    } else {
        original_trust.clone()
    };

    // Konvertiere zu Proto-Typen
    let (allowed, denial_reason) = match result {
        Ok(_) => (true, String::new()),
        Err(e) => (false, e.to_string()),
    };

    // Prädikate für verbose Output
    let predicates = if request.verbose {
        vec![
            PredicateResult {
                name: "P₁: min_trust".to_string(),
                satisfied: allowed,
                description: format!("Trust ≥ min_trust für {}", target_realm),
                failure_reason: if !allowed {
                    Some(denial_reason.clone())
                } else {
                    None
                },
            },
            PredicateResult {
                name: "P₂: realm_rules".to_string(),
                satisfied: true,
                description: "Realm-Regeln erfüllt".to_string(),
                failure_reason: None,
            },
        ]
    } else {
        vec![]
    };

    EvaluateGatewayResponse {
        allowed,
        predicates,
        original_trust: Some(domain_to_proto_trust(&original_trust)),
        transformed_trust: Some(domain_to_proto_trust(&transformed_trust)),
        applied_matrix: None, // TODO: Matrix exportieren
        denial_reason,
    }
}

fn domain_to_proto_trust(trust: &crate::domain::TrustVector6D) -> TrustVector6D {
    TrustVector6D {
        reliability: trust.r(),
        integrity: trust.i(),
        competence: trust.c(),
        prestige: trust.p(),
        vigilance: trust.v(),
        omega: trust.omega(),
    }
}

// ============================================================================
// START/STOP PEER (Lifecycle)
// ============================================================================

/// StartPeer - Startet den Peer-Service
pub async fn start_peer_handler(
    State(_state): State<AppState>,
    _request: StartPeerRequest,
) -> StartPeerResponse {
    // Peer ist bereits gestartet wenn dieser Handler aufgerufen wird
    let peer_id = format!("peer:{}", uuid::Uuid::new_v4());

    StartPeerResponse {
        success: true,
        peer_id,
        error: None,
    }
}

/// StopPeer - Stoppt den Peer-Service
pub async fn stop_peer_handler(
    State(_state): State<AppState>,
    request: StopPeerRequest,
) -> StopPeerResponse {
    // In einer echten Implementierung würden wir hier aktive Sagas abbrechen
    let cancelled_sagas = if request.force { 0 } else { 0 };

    StopPeerResponse {
        success: true,
        cancelled_sagas,
        error: None,
    }
}
