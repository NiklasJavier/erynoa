//! # Trust Engine
//!
//! Trust-Berechnung und -Aktualisierung gemäß Κ2-Κ5.
//!
//! ## Axiom-Referenz
//!
//! - **Κ2 (Trust-Default)**: Neue Entitäten starten mit `𝕎₀ = (0.5,…)`
//! - **Κ3 (Trust-Bounds)**: `0 ≤ 𝕎ᵢ ≤ 1`
//! - **Κ4 (Asymmetric Update)**: Vertrauen sinkt 2× schneller als es steigt
//! - **Κ5 (Probabilistic Combination)**: `𝕎_comb = 1 − ∏(1 − 𝕎ⱼ)`

use crate::domain::{
    ContextType, Event, TrustCombination, TrustDampeningMatrix, TrustVector6D, DID,
};
use std::collections::HashMap;
use thiserror::Error;

/// Fehler bei Trust-Operationen
#[derive(Debug, Error)]
pub enum TrustError {
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Invalid trust value: {0} (must be in [0,1])")]
    InvalidTrustValue(f64),

    #[error("Self-attestation not allowed")]
    SelfAttestation,
}

/// Ergebnis von Trust-Operationen
pub type TrustResult<T> = Result<T, TrustError>;

/// Trust Engine - berechnet und aktualisiert Trust-Vektoren (Κ2-Κ5)
///
/// ```text
/// ┌──────────────────────────────────────────────────────────────┐
/// │                     TrustEngine                              │
/// │                                                              │
/// │  ┌──────────┐    ┌──────────┐    ┌──────────────┐          │
/// │  │ Observe  │───▶│ Update   │───▶│ Propagate    │          │
/// │  │ (Events) │    │  (Κ4)    │    │ (Κ5, Τ1)    │          │
/// │  └──────────┘    └──────────┘    └──────────────┘          │
/// └──────────────────────────────────────────────────────────────┘
/// ```
pub struct TrustEngine {
    /// Trust-Vektoren pro DID
    trust_vectors: HashMap<DID, TrustVector6D>,

    /// Trust-Beziehungen (from → to → context → trust)
    relationships: HashMap<DID, HashMap<DID, HashMap<ContextType, f64>>>,

    /// Konfiguration
    config: TrustEngineConfig,
}

/// Konfiguration für TrustEngine
#[derive(Debug, Clone)]
pub struct TrustEngineConfig {
    /// Κ2: Default Trust-Wert
    pub default_trust: f64,

    /// Κ4: Positive Update Rate
    pub positive_rate: f64,

    /// Κ4: Negative Update Rate (2× positive)
    pub negative_rate: f64,

    /// Minimum Trust für Interaktion
    pub interaction_threshold: f64,
}

impl Default for TrustEngineConfig {
    fn default() -> Self {
        Self {
            default_trust: 0.5,
            positive_rate: 0.1,
            negative_rate: 0.2, // Κ4: 2× schneller
            interaction_threshold: 0.3,
        }
    }
}

impl TrustEngine {
    /// Erstelle neue TrustEngine
    pub fn new(config: TrustEngineConfig) -> Self {
        Self {
            trust_vectors: HashMap::new(),
            relationships: HashMap::new(),
            config,
        }
    }

    /// Erstelle mit Default-Config
    pub fn default() -> Self {
        Self::new(TrustEngineConfig::default())
    }

    /// Κ2: Initialisiere Trust für neue Entität
    pub fn initialize_trust(&mut self, did: &DID) {
        if !self.trust_vectors.contains_key(did) {
            self.trust_vectors.insert(
                did.clone(),
                TrustVector6D::default(), // 𝕎₀ = (0.5, 0.5, 0.5, 0.5, 0.5, 0.5)
            );
        }
    }

    /// Hole Trust-Vektor für DID
    pub fn get_trust(&self, did: &DID) -> Option<&TrustVector6D> {
        self.trust_vectors.get(did)
    }

    /// Hole Trust-Vektor oder Default
    pub fn get_trust_or_default(&mut self, did: &DID) -> &TrustVector6D {
        self.initialize_trust(did);
        self.trust_vectors.get(did).unwrap()
    }

    /// Κ4: Aktualisiere Trust basierend auf Event
    pub fn process_event(&mut self, event: &Event) -> TrustResult<()> {
        self.initialize_trust(&event.author);

        // Bestimme Trust-Dimension und Richtung
        if let Some(dimension) = event.primary_trust_dimension() {
            let delta = if event.is_negative_trust() {
                -self.config.negative_rate // Κ4: 2× schneller bei negativ
            } else {
                self.config.positive_rate
            };

            // Update Trust-Vektor
            if let Some(trust) = self.trust_vectors.get_mut(&event.author) {
                trust.update(dimension, delta.abs(), !event.is_negative_trust());
            }
        }

        Ok(())
    }

    /// Setze direkten Trust-Wert (für Attestationen)
    pub fn set_direct_trust(
        &mut self,
        from: &DID,
        to: &DID,
        context: ContextType,
        trust: f64,
    ) -> TrustResult<()> {
        // Κ3: Bounds-Check
        if !(0.0..=1.0).contains(&trust) {
            return Err(TrustError::InvalidTrustValue(trust));
        }

        // Self-Attestation verbieten
        if from == to {
            return Err(TrustError::SelfAttestation);
        }

        self.relationships
            .entry(from.clone())
            .or_default()
            .entry(to.clone())
            .or_default()
            .insert(context, trust);

        Ok(())
    }

    /// Hole direkten Trust zwischen zwei DIDs
    pub fn get_direct_trust(&self, from: &DID, to: &DID, context: ContextType) -> Option<f64> {
        self.relationships
            .get(from)?
            .get(to)?
            .get(&context)
            .copied()
    }

    /// Κ5: Kombiniere Trust aus mehreren Quellen
    pub fn combine_trust(&self, sources: &[(DID, f64)]) -> f64 {
        let trusts: Vec<f64> = sources.iter().map(|(_, t)| *t).collect();
        TrustCombination::combine_all(&trusts)
    }

    /// Τ1: Berechne Chain-Trust über mehrere Hops
    pub fn chain_trust(&self, chain: &[DID], context: ContextType) -> f64 {
        if chain.len() < 2 {
            return 1.0;
        }

        let mut trusts = Vec::new();
        for window in chain.windows(2) {
            let trust = self
                .get_direct_trust(&window[0], &window[1], context)
                .unwrap_or(self.config.default_trust);
            trusts.push(trust);
        }

        TrustCombination::chain_trust(&trusts)
    }

    /// Berechne gewichtete Trust-Norm für Kontext
    pub fn contextual_trust_norm(&self, did: &DID, context: ContextType) -> f64 {
        self.trust_vectors
            .get(did)
            .map(|t| t.weighted_norm(&context.weights()))
            .unwrap_or(self.config.default_trust)
    }

    /// Wende Realm-Crossing-Dampening an (Κ24)
    pub fn apply_realm_crossing(
        &self,
        trust: &TrustVector6D,
        _from_realm: &str,
        _to_realm: &str,
    ) -> TrustVector6D {
        // Verwende Standard-Dämpfungsfaktor 0.7 für Cross-Realm
        let matrix = TrustDampeningMatrix::generic_crossing(0.7);
        matrix.apply(trust)
    }

    /// Prüfe ob Trust ausreicht für Interaktion
    pub fn can_interact(&self, did: &DID) -> bool {
        self.trust_vectors
            .get(did)
            .map(|t| t.min_component() >= self.config.interaction_threshold)
            .unwrap_or(false)
    }

    /// Statistiken
    pub fn stats(&self) -> TrustEngineStats {
        let trust_values: Vec<_> = self.trust_vectors.values().collect();

        let avg_trust = if trust_values.is_empty() {
            0.0
        } else {
            trust_values
                .iter()
                .map(|t| t.weighted_norm(&[1.0; 6]))
                .sum::<f64>()
                / trust_values.len() as f64
        };

        let low_trust_count = trust_values
            .iter()
            .filter(|t| t.min_component() < self.config.interaction_threshold)
            .count();

        TrustEngineStats {
            total_entities: self.trust_vectors.len(),
            total_relationships: self.relationships.values().map(|inner| inner.len()).sum(),
            average_trust: avg_trust,
            low_trust_entities: low_trust_count,
        }
    }
}

/// Statistiken der TrustEngine
#[derive(Debug, Clone)]
pub struct TrustEngineStats {
    pub total_entities: usize,
    pub total_relationships: usize,
    pub average_trust: f64,
    pub low_trust_entities: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{EventPayload, TrustDimension};

    #[test]
    fn test_initialize_default_trust() {
        let mut engine = TrustEngine::default();
        let did = DID::new_self("alice");

        engine.initialize_trust(&did);
        let trust = engine.get_trust(&did).unwrap();

        // Κ2: Default = 0.5
        assert!((trust.r - 0.5).abs() < 0.001);
        assert!((trust.i - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_process_positive_event() {
        let mut engine = TrustEngine::default();
        let alice = DID::new_self("alice");

        // Transfer-Event (positiv für Reliability)
        let event = Event::new(
            alice.clone(),
            EventPayload::Transfer {
                from: alice.clone(),
                to: DID::new_self("bob"),
                amount: 100,
                asset_type: "ERY".to_string(),
            },
            vec![],
        );

        engine.process_event(&event).unwrap();

        let trust = engine.get_trust(&alice).unwrap();
        // Reliability sollte gestiegen sein
        assert!(trust.r > 0.5);
    }

    #[test]
    fn test_asymmetric_update() {
        let mut engine = TrustEngine::default();
        let alice = DID::new_self("alice");

        engine.initialize_trust(&alice);
        let initial = engine.get_trust(&alice).unwrap().r;

        // Positive Update
        if let Some(trust) = engine.trust_vectors.get_mut(&alice) {
            trust.update(TrustDimension::Reliability, 0.1, true);
        }
        let after_positive = engine.get_trust(&alice).unwrap().r;

        // Negative Update (sollte 2× so stark wirken)
        if let Some(trust) = engine.trust_vectors.get_mut(&alice) {
            trust.update(TrustDimension::Reliability, 0.1, false);
        }
        let after_negative = engine.get_trust(&alice).unwrap().r;

        // Κ4: Negativ wirkt 2× so stark
        let positive_delta = after_positive - initial;
        let negative_delta = after_positive - after_negative;

        // Bei gleichem Betrag sollte negativ 2× wirken
        // (Da wir asymmetry_factor im update haben)
        assert!(negative_delta > positive_delta * 1.5);
    }

    #[test]
    fn test_combine_trust() {
        let engine = TrustEngine::default();

        // Κ5: 1 - (1-0.8)(1-0.7)(1-0.6) = 1 - 0.2×0.3×0.4 = 0.976
        let sources = vec![
            (DID::new_self("a"), 0.8),
            (DID::new_self("b"), 0.7),
            (DID::new_self("c"), 0.6),
        ];

        let combined = engine.combine_trust(&sources);
        assert!((combined - 0.976).abs() < 0.001);
    }

    #[test]
    fn test_chain_trust() {
        let mut engine = TrustEngine::default();

        let alice = DID::new_self("alice");
        let bob = DID::new_self("bob");
        let carol = DID::new_self("carol");

        // Alice → Bob: 0.9
        // Bob → Carol: 0.8
        engine
            .set_direct_trust(&alice, &bob, ContextType::Default, 0.9)
            .unwrap();
        engine
            .set_direct_trust(&bob, &carol, ContextType::Default, 0.8)
            .unwrap();

        // Τ1: Chain trust mit √n Dampening
        // exp((ln(0.9) + ln(0.8)) / √2) = exp(-0.328 / 1.414) ≈ 0.79
        let chain_trust = engine.chain_trust(&[alice, bob, carol], ContextType::Default);

        // Sollte besser sein als einfaches Produkt (0.9 × 0.8 = 0.72)
        let simple_product = 0.9 * 0.8;
        assert!(
            chain_trust > simple_product,
            "Chain trust {} should be > simple product {}",
            chain_trust,
            simple_product
        );
        assert!(chain_trust > 0.0);
        assert!(chain_trust < 1.0);
    }

    #[test]
    fn test_self_attestation_rejected() {
        let mut engine = TrustEngine::default();
        let alice = DID::new_self("alice");

        let result = engine.set_direct_trust(&alice, &alice, ContextType::Default, 0.9);
        assert!(matches!(result, Err(TrustError::SelfAttestation)));
    }
}
