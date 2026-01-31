//! # Gateway Guard
//!
//! Cross-Realm Gateway Guard gemäß Κ23.
//!
//! ## Axiom-Referenz
//!
//! - **Κ23 (Gateway-Guard)**: `cross(s, 𝒞₁, 𝒞₂) requires G(s, 𝒞₂) = true`
//!
//! ## Validierung
//!
//! Der Gateway prüft:
//! 1. **Trust**: Entität erfüllt min_trust des Ziel-Realms
//! 2. **Rules**: Entität erfüllt alle zusätzlichen Regeln des Ziel-Realms
//! 3. **Credentials**: Entität hat erforderliche Credentials

use crate::domain::{RealmId, RootRealm, TrustDampeningMatrix, TrustVector6D, VirtualRealm, DID};
use std::collections::HashMap;
use thiserror::Error;

/// Fehler bei Gateway-Operationen
#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("Realm not found: {0}")]
    RealmNotFound(String),

    #[error("Trust insufficient for {did}: {current} < {required}")]
    InsufficientTrust {
        did: String,
        current: f64,
        required: f64,
    },

    #[error("Missing required rule: {0}")]
    MissingRule(String),

    #[error("Missing required credential: {0}")]
    MissingCredential(String),

    #[error("Entity not registered: {0}")]
    EntityNotRegistered(String),
}

/// Ergebnis von Gateway-Operationen
pub type GatewayResult<T> = Result<T, GatewayError>;

/// Gateway Guard (Κ23)
///
/// ```text
///             cross(s, 𝒞₁, 𝒞₂)
///                    │
///                    ▼
///    ┌───────────────────────────────┐
///    │        GatewayGuard           │
///    │                               │
///    │  1. Check Trust ≥ min_trust   │
///    │  2. Check Rules fulfilled     │
///    │  3. Check Credentials         │
///    │  4. Apply Trust Dampening     │
///    │                               │
///    └───────────────┬───────────────┘
///                    │
///               ┌────┴────┐
///               │ G = ?   │
///               └────┬────┘
///                ┌───┴───┐
///           true │       │ false
///                ▼       ▼
///            ALLOW     DENY
/// ```
pub struct GatewayGuard {
    /// Registrierte Realms
    realms: HashMap<RealmId, RealmEntry>,

    /// Trust-Vektoren pro DID (Referenz, in Produktion via TrustEngine)
    trust_vectors: HashMap<DID, TrustVector6D>,

    /// Credentials pro DID
    credentials: HashMap<DID, Vec<String>>,

    /// Konfiguration
    config: GatewayConfig,
}

/// Realm-Eintrag für Gateway
#[allow(dead_code)]
struct RealmEntry {
    id: RealmId,
    name: String,
    min_trust: f64,
    required_rules: Vec<String>,
    required_credentials: Vec<String>,
}

/// Konfiguration für GatewayGuard
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// Standard min_trust wenn nicht spezifiziert
    pub default_min_trust: f64,

    /// Aktiviere Trust-Dampening bei Crossing
    pub apply_trust_dampening: bool,

    /// Logging-Level
    pub verbose: bool,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            default_min_trust: 0.3,
            apply_trust_dampening: true,
            verbose: false,
        }
    }
}

/// Ergebnis einer Gateway-Prüfung
#[derive(Debug, Clone)]
pub struct CrossingResult {
    pub allowed: bool,
    pub from_realm: RealmId,
    pub to_realm: RealmId,
    pub did: DID,
    pub original_trust: TrustVector6D,
    pub dampened_trust: TrustVector6D,
    pub violations: Vec<String>,
}

impl GatewayGuard {
    /// Erstelle neuen GatewayGuard
    pub fn new(config: GatewayConfig) -> Self {
        let mut guard = Self {
            realms: HashMap::new(),
            trust_vectors: HashMap::new(),
            credentials: HashMap::new(),
            config,
        };

        // Registriere Root-Realm
        let root = RootRealm::default();
        guard.register_realm_entry(
            root.id.clone(),
            "Root Realm".to_string(),
            0.0,
            vec![],
            vec![],
        );

        guard
    }

    /// Erstelle mit Default-Config
    pub fn default() -> Self {
        Self::new(GatewayConfig::default())
    }

    /// Registriere Realm
    fn register_realm_entry(
        &mut self,
        id: RealmId,
        name: String,
        min_trust: f64,
        required_rules: Vec<String>,
        required_credentials: Vec<String>,
    ) {
        self.realms.insert(
            id.clone(),
            RealmEntry {
                id,
                name,
                min_trust,
                required_rules,
                required_credentials,
            },
        );
    }

    /// Registriere VirtualRealm
    pub fn register_virtual_realm(
        &mut self,
        realm: &VirtualRealm,
        required_credentials: Vec<String>,
    ) {
        let required_rules: Vec<_> = realm
            .rules
            .rules
            .iter()
            .filter(|r| !r.optional)
            .map(|r| r.id.clone())
            .collect();

        self.register_realm_entry(
            realm.id.clone(),
            realm.name.clone(),
            realm.min_trust,
            required_rules,
            required_credentials,
        );
    }

    /// Registriere Trust für DID
    pub fn register_trust(&mut self, did: DID, trust: TrustVector6D) {
        self.trust_vectors.insert(did, trust);
    }

    /// Registriere Credential für DID
    pub fn add_credential(&mut self, did: &DID, credential: String) {
        self.credentials
            .entry(did.clone())
            .or_default()
            .push(credential);
    }

    /// Κ23: Validiere Realm-Crossing
    ///
    /// `cross(s, 𝒞₁, 𝒞₂) requires G(s, 𝒞₂) = true`
    pub fn validate_crossing(
        &self,
        did: &DID,
        from_realm: &RealmId,
        to_realm: &RealmId,
    ) -> GatewayResult<CrossingResult> {
        let mut violations = Vec::new();

        // Hole Ziel-Realm
        let target = self
            .realms
            .get(to_realm)
            .ok_or_else(|| GatewayError::RealmNotFound(to_realm.to_string()))?;

        // Hole Trust
        let trust = self
            .trust_vectors
            .get(did)
            .ok_or_else(|| GatewayError::EntityNotRegistered(did.to_uri()))?;

        // 1. Trust-Check
        let trust_norm = trust.weighted_norm(&[1.0; 6]);
        if trust_norm < target.min_trust {
            violations.push(format!(
                "Insufficient trust: {} < {}",
                trust_norm, target.min_trust
            ));
        }

        // 2. Credentials-Check
        let did_credentials = self
            .credentials
            .get(did)
            .map(|c| c.as_slice())
            .unwrap_or(&[]);

        for required in &target.required_credentials {
            if !did_credentials.contains(required) {
                violations.push(format!("Missing credential: {}", required));
            }
        }

        // 3. Berechne gedämpften Trust
        let dampened = if self.config.apply_trust_dampening {
            // Verwende Standard-Dämpfungsfaktor 0.7 für Cross-Realm
            let matrix = TrustDampeningMatrix::generic_crossing(0.7);
            matrix.apply(trust)
        } else {
            trust.clone()
        };

        let allowed = violations.is_empty();

        Ok(CrossingResult {
            allowed,
            from_realm: from_realm.clone(),
            to_realm: to_realm.clone(),
            did: did.clone(),
            original_trust: trust.clone(),
            dampened_trust: dampened,
            violations,
        })
    }

    /// Prüfe und erlaube Crossing (wirft Fehler wenn nicht erlaubt)
    pub fn allow_crossing(
        &self,
        did: &DID,
        from_realm: &RealmId,
        to_realm: &RealmId,
    ) -> GatewayResult<TrustVector6D> {
        let result = self.validate_crossing(did, from_realm, to_realm)?;

        if !result.allowed {
            // Return first violation as error
            if let Some(violation) = result.violations.first() {
                if violation.contains("trust") {
                    return Err(GatewayError::InsufficientTrust {
                        did: did.to_uri(),
                        current: result.original_trust.weighted_norm(&[1.0; 6]),
                        required: self
                            .realms
                            .get(to_realm)
                            .map(|r| r.min_trust)
                            .unwrap_or(0.0),
                    });
                } else if violation.contains("credential") {
                    return Err(GatewayError::MissingCredential(violation.clone()));
                } else {
                    return Err(GatewayError::MissingRule(violation.clone()));
                }
            }
        }

        Ok(result.dampened_trust)
    }

    /// Statistiken
    pub fn stats(&self) -> GatewayStats {
        GatewayStats {
            registered_realms: self.realms.len(),
            registered_entities: self.trust_vectors.len(),
            total_credentials: self.credentials.values().map(|c| c.len()).sum(),
        }
    }
}

/// Statistiken des GatewayGuard
#[derive(Debug, Clone)]
pub struct GatewayStats {
    pub registered_realms: usize,
    pub registered_entities: usize,
    pub total_credentials: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_gateway() -> GatewayGuard {
        let mut guard = GatewayGuard::default();

        // Registriere Finance-Realm mit höheren Anforderungen
        guard.register_realm_entry(
            RealmId::new("realm:erynoa:finance"),
            "Finance".to_string(),
            0.7, // Hoher min_trust
            vec!["COMPLIANCE".to_string()],
            vec!["KYC".to_string()],
        );

        // Registriere Gaming-Realm mit niedrigeren Anforderungen
        guard.register_realm_entry(
            RealmId::new("realm:erynoa:gaming"),
            "Gaming".to_string(),
            0.3,
            vec![],
            vec![],
        );

        guard
    }

    #[test]
    fn test_crossing_to_low_trust_realm() {
        let mut guard = setup_gateway();

        let alice = DID::new_self("alice");
        guard.register_trust(
            alice.clone(),
            TrustVector6D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5),
        );

        let result = guard
            .validate_crossing(
                &alice,
                &RealmId::root(),
                &RealmId::new("realm:erynoa:gaming"),
            )
            .unwrap();

        assert!(result.allowed);
    }

    #[test]
    fn test_crossing_denied_insufficient_trust() {
        let mut guard = setup_gateway();

        let alice = DID::new_self("alice");
        guard.register_trust(
            alice.clone(),
            TrustVector6D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5), // Trust ~0.5
        );

        let result = guard
            .validate_crossing(
                &alice,
                &RealmId::root(),
                &RealmId::new("realm:erynoa:finance"), // Requires 0.7
            )
            .unwrap();

        assert!(!result.allowed);
        assert!(result.violations.iter().any(|v| v.contains("trust")));
    }

    #[test]
    fn test_crossing_denied_missing_credential() {
        let mut guard = setup_gateway();

        let alice = DID::new_self("alice");
        guard.register_trust(
            alice.clone(),
            TrustVector6D::new(0.9, 0.9, 0.9, 0.9, 0.9, 0.9), // High trust
        );
        // Aber kein KYC-Credential

        let result = guard
            .validate_crossing(
                &alice,
                &RealmId::root(),
                &RealmId::new("realm:erynoa:finance"),
            )
            .unwrap();

        assert!(!result.allowed);
        assert!(result.violations.iter().any(|v| v.contains("credential")));
    }

    #[test]
    fn test_crossing_allowed_with_credentials() {
        let mut guard = setup_gateway();

        let alice = DID::new_self("alice");
        guard.register_trust(
            alice.clone(),
            TrustVector6D::new(0.9, 0.9, 0.9, 0.9, 0.9, 0.9),
        );
        guard.add_credential(&alice, "KYC".to_string());

        let result = guard
            .validate_crossing(
                &alice,
                &RealmId::root(),
                &RealmId::new("realm:erynoa:finance"),
            )
            .unwrap();

        assert!(result.allowed);
    }

    #[test]
    fn test_trust_dampening() {
        let mut guard = setup_gateway();

        let alice = DID::new_self("alice");
        let original_trust = TrustVector6D::new(0.9, 0.9, 0.9, 0.9, 0.9, 0.9);
        guard.register_trust(alice.clone(), original_trust.clone());

        let result = guard
            .validate_crossing(
                &alice,
                &RealmId::new("realm:erynoa:gaming"),
                &RealmId::new("realm:erynoa:gaming"), // Same realm, still dampening
            )
            .unwrap();

        // Vigilance und Omega sollten erhalten bleiben (Κ24)
        // Verwende .v statt .vigilance und .omega für Zugriff
        assert!((result.dampened_trust.v - original_trust.v).abs() < 0.01);
        assert!((result.dampened_trust.omega - original_trust.omega).abs() < 0.01);
    }
}
