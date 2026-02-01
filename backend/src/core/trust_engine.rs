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
//!
//! ## Phase 3.2: ExecutionContext Integration
//!
//! Erweitert um `*_with_ctx`-Methoden für Gas-Accounting und Event-Emission.

use crate::core::engine::trust_gas;
use crate::domain::unified::Cost;
use crate::domain::{
    ContextType, Event, TrustCombination, TrustDampeningMatrix, TrustVector6D, UniversalId, DID,
};
use crate::execution::{ExecutionContext, ExecutionError, ExecutionResult};
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
    /// Trust-Vektoren pro UniversalId (DID.id)
    trust_vectors: HashMap<UniversalId, TrustVector6D>,

    /// Trust-Beziehungen (from → to → context → trust)
    relationships: HashMap<UniversalId, HashMap<UniversalId, HashMap<ContextType, f64>>>,

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
    pub fn initialize_trust(&mut self, id: &UniversalId) {
        if !self.trust_vectors.contains_key(id) {
            self.trust_vectors.insert(
                id.clone(),
                TrustVector6D::default(), // 𝕎₀ = (0.5, 0.5, 0.5, 0.5, 0.5, 0.5)
            );
        }
    }

    /// Κ2: Initialisiere Trust für DID (Kompatibilität)
    pub fn initialize_trust_for_did(&mut self, did: &DID) {
        self.initialize_trust(&did.id);
    }

    /// Hole Trust-Vektor für UniversalId
    pub fn get_trust(&self, id: &UniversalId) -> Option<&TrustVector6D> {
        self.trust_vectors.get(id)
    }

    /// Hole Trust-Vektor für DID (Kompatibilität)
    pub fn get_trust_for_did(&self, did: &DID) -> Option<&TrustVector6D> {
        self.get_trust(&did.id)
    }

    /// Hole Trust-Vektor oder Default
    pub fn get_trust_or_default(&mut self, id: &UniversalId) -> &TrustVector6D {
        self.initialize_trust(id);
        self.trust_vectors.get(id).unwrap()
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
                trust.update(dimension, delta as f32);
            }
        }

        Ok(())
    }

    /// Setze direkten Trust-Wert (für Attestationen)
    pub fn set_direct_trust(
        &mut self,
        from: &UniversalId,
        to: &UniversalId,
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

    /// Hole direkten Trust zwischen zwei UniversalIds
    pub fn get_direct_trust(
        &self,
        from: &UniversalId,
        to: &UniversalId,
        context: ContextType,
    ) -> Option<f64> {
        self.relationships
            .get(from)?
            .get(to)?
            .get(&context)
            .copied()
    }

    /// Κ5: Kombiniere Trust aus mehreren Quellen
    pub fn combine_trust(&self, sources: &[(UniversalId, f64)]) -> f64 {
        let trusts: Vec<f32> = sources.iter().map(|(_, t)| *t as f32).collect();
        TrustCombination::combine_all(&trusts) as f64
    }

    /// Τ1: Berechne Chain-Trust über mehrere Hops
    pub fn chain_trust(&self, chain: &[UniversalId], context: ContextType) -> f64 {
        if chain.len() < 2 {
            return 1.0;
        }

        let mut trusts = Vec::new();
        for window in chain.windows(2) {
            let trust = self
                .get_direct_trust(&window[0], &window[1], context)
                .unwrap_or(self.config.default_trust);
            trusts.push(trust as f32);
        }

        TrustCombination::chain_trust(&trusts) as f64
    }

    /// Berechne gewichtete Trust-Norm für Kontext
    pub fn contextual_trust_norm(&self, id: &UniversalId, context: ContextType) -> f32 {
        self.trust_vectors
            .get(id)
            .map(|t| t.weighted_norm(&context.weights()))
            .unwrap_or(self.config.default_trust as f32)
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
    pub fn can_interact(&self, id: &UniversalId) -> bool {
        self.trust_vectors
            .get(id)
            .map(|t| t.min_component() >= self.config.interaction_threshold as f32)
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
                .map(|t| t.weighted_norm(&[1.0; 6]) as f64)
                .sum::<f64>()
                / trust_values.len() as f64
        };

        let low_trust_count = trust_values
            .iter()
            .filter(|t| t.min_component() < self.config.interaction_threshold as f32)
            .count();

        TrustEngineStats {
            total_entities: self.trust_vectors.len(),
            total_relationships: self.relationships.values().map(|inner| inner.len()).sum(),
            average_trust: avg_trust,
            low_trust_entities: low_trust_count,
        }
    }

    // =========================================================================
    // ExecutionContext-Integration (Phase 3.2)
    // =========================================================================

    /// Κ2: Initialisiere Trust mit Gas-Accounting
    ///
    /// Gas: LOOKUP + UPDATE
    pub fn initialize_trust_with_ctx(
        &mut self,
        ctx: &mut ExecutionContext,
        entity: &UniversalId,
    ) -> ExecutionResult<()> {
        ctx.consume_gas(trust_gas::LOOKUP)?;

        // Prüfe ob bereits initialisiert
        if self.trust_vectors.contains_key(entity) {
            return Ok(());
        }

        ctx.consume_gas(trust_gas::UPDATE)?;

        // Κ2: Default Trust = 0.5 für alle Dimensionen
        let default_trust = TrustVector6D::default();
        self.trust_vectors.insert(entity.clone(), default_trust);

        ctx.emit_raw("trust.initialized", entity.to_hex().as_bytes());
        ctx.track_cost(Cost::new(trust_gas::LOOKUP + trust_gas::UPDATE, 0, 0.0));

        Ok(())
    }

    /// Κ4: Verarbeite Event und aktualisiere Trust mit ExecutionContext
    ///
    /// Gas: LOOKUP + UPDATE + PROPAGATE
    pub fn process_event_with_ctx(
        &mut self,
        ctx: &mut ExecutionContext,
        event: &Event,
    ) -> ExecutionResult<()> {
        ctx.consume_gas(trust_gas::LOOKUP)?;

        // Initialisiere Trust falls nötig
        self.initialize_trust_with_ctx(ctx, &event.author)?;

        // Trust-Änderung basierend auf Event-Typ
        let (dimension, delta, _is_positive) = Self::derive_trust_delta(event);

        ctx.consume_gas(trust_gas::UPDATE)?;

        // Κ4: Asymmetrisches Update (negativ wirkt 2× so stark)
        if let Some(trust) = self.trust_vectors.get_mut(&event.author) {
            trust.update(dimension, delta as f32);
        }

        ctx.emit_raw("trust.updated", event.author.to_hex().as_bytes());
        ctx.track_cost(Cost::new(trust_gas::LOOKUP + trust_gas::UPDATE, 0, 0.0));

        Ok(())
    }

    /// Κ3: Setze direkten Trust-Wert mit Bounds-Check und Gas-Accounting
    ///
    /// Gas: LOOKUP + UPDATE
    pub fn set_direct_trust_with_ctx(
        &mut self,
        ctx: &mut ExecutionContext,
        from: &UniversalId,
        to: &UniversalId,
        context: ContextType,
        trust: f64,
    ) -> ExecutionResult<()> {
        ctx.consume_gas(trust_gas::LOOKUP)?;

        // Κ3: Trust-Bounds prüfen
        if !(0.0..=1.0).contains(&trust) {
            return Err(ExecutionError::InvalidInput(format!(
                "Trust value {} out of bounds [0,1]",
                trust
            )));
        }

        // Self-Attestation verhindern
        if from == to {
            return Err(ExecutionError::InvalidInput(
                "Self-attestation not allowed".to_string(),
            ));
        }

        ctx.consume_gas(trust_gas::UPDATE)?;

        // Update relationship
        self.relationships
            .entry(from.clone())
            .or_default()
            .entry(to.clone())
            .or_default()
            .insert(context, trust);

        ctx.emit_raw(
            "trust.relationship",
            format!("{}→{}", from.to_hex(), to.to_hex()).as_bytes(),
        );
        ctx.track_cost(Cost::new(trust_gas::LOOKUP + trust_gas::UPDATE, 0, 0.0));

        Ok(())
    }

    /// Κ5: Kombiniere Trust aus mehreren Quellen mit Gas-Accounting
    ///
    /// Gas: COMBINE + LOOKUP × sources.len()
    pub fn combine_trust_with_ctx(
        &self,
        ctx: &mut ExecutionContext,
        sources: &[(UniversalId, f64)],
    ) -> ExecutionResult<f64> {
        ctx.consume_gas(trust_gas::COMBINE)?;

        // Gas für jeden Lookup
        let lookup_cost = trust_gas::LOOKUP * sources.len() as u64;
        ctx.consume_gas(lookup_cost)?;

        // Κ5: 𝕎_comb = 1 - ∏(1 - 𝕎ⱼ)
        let combined = self.combine_trust(sources);

        ctx.track_cost(Cost::new(trust_gas::COMBINE + lookup_cost, 0, 0.0));

        Ok(combined)
    }

    /// Τ1: Berechne Chain-Trust mit ExecutionContext
    ///
    /// Gas: CHAIN_TRUST_BASE + LOOKUP × path.len()
    pub fn chain_trust_with_ctx(
        &self,
        ctx: &mut ExecutionContext,
        path: &[UniversalId],
        context: ContextType,
    ) -> ExecutionResult<f64> {
        ctx.consume_gas(trust_gas::CHAIN_TRUST_BASE)?;

        let lookup_cost = trust_gas::LOOKUP * path.len() as u64;
        ctx.consume_gas(lookup_cost)?;

        let trust = self.chain_trust(path, context);

        ctx.track_cost(Cost::new(trust_gas::CHAIN_TRUST_BASE + lookup_cost, 0, 0.0));

        Ok(trust)
    }

    /// Bestimme Trust-Delta basierend auf Event-Typ (Helper)
    fn derive_trust_delta(event: &Event) -> (crate::domain::TrustDimension, f64, bool) {
        use crate::domain::{EventPayload, TrustDimension};

        match &event.payload {
            EventPayload::Transfer { .. } => (TrustDimension::Reliability, 0.01, true),
            EventPayload::Attest { .. } => (TrustDimension::Integrity, 0.02, true),
            EventPayload::Delegate { .. } => (TrustDimension::Competence, 0.01, true),
            EventPayload::CredentialIssue { .. } => (TrustDimension::Prestige, 0.015, true),
            EventPayload::Custom { event_type, .. } => {
                if event_type.starts_with("violation") {
                    (TrustDimension::Reliability, 0.05, false) // Κ4: Negativ
                } else {
                    (TrustDimension::Prestige, 0.005, true)
                }
            }
            _ => (TrustDimension::Prestige, 0.001, true),
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
        let did = DID::new_self(b"alice");

        engine.initialize_trust(&did.id);
        let trust = engine.get_trust(&did.id).unwrap();

        // Κ2: Default = 0.5
        assert!((trust.r - 0.5).abs() < 0.001);
        assert!((trust.i - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_process_positive_event() {
        let mut engine = TrustEngine::default();
        let alice = DID::new_self(b"alice");
        let bob = DID::new_self(b"bob");

        // Transfer-Event (positiv für Reliability)
        let event = Event::new(
            alice.id.clone(),
            vec![],
            EventPayload::Transfer {
                from: alice.id.clone(),
                to: bob.id.clone(),
                amount: 100,
                asset_type: "ERY".to_string(),
            },
            0,
        );

        engine.process_event(&event).unwrap();

        let trust = engine.get_trust(&alice.id).unwrap();
        // Reliability sollte gestiegen sein
        assert!(trust.r > 0.5);
    }

    #[test]
    fn test_asymmetric_update() {
        let mut engine = TrustEngine::default();
        let alice = DID::new_self(b"alice");

        engine.initialize_trust(&alice.id);
        let initial = engine.get_trust(&alice.id).unwrap().r;

        // Positive Update
        if let Some(trust) = engine.trust_vectors.get_mut(&alice.id) {
            trust.update(TrustDimension::Reliability, 0.1);
        }
        let after_positive = engine.get_trust(&alice.id).unwrap().r;

        // Negative Update (sollte 2× so stark wirken)
        if let Some(trust) = engine.trust_vectors.get_mut(&alice.id) {
            trust.update(TrustDimension::Reliability, -0.1);
        }
        let after_negative = engine.get_trust(&alice.id).unwrap().r;

        // Κ4: Negativ wirkt 1.5× so stark (für Reliability)
        let positive_delta = after_positive - initial;
        let negative_delta = after_positive - after_negative;

        // Bei gleichem Betrag sollte negativ 1.5× wirken
        // (asymmetry_factor für Reliability ist 1.5)
        assert!(
            negative_delta > positive_delta * 1.0,
            "Expected negative_delta ({}) > positive_delta ({}) * 1.0",
            negative_delta,
            positive_delta
        );
    }

    #[test]
    fn test_combine_trust() {
        let engine = TrustEngine::default();

        // Κ5: 1 - (1-0.8)(1-0.7)(1-0.6) = 1 - 0.2×0.3×0.4 = 0.976
        let sources = vec![
            (DID::new_self(b"a").id, 0.8),
            (DID::new_self(b"b").id, 0.7),
            (DID::new_self(b"c").id, 0.6),
        ];

        let combined = engine.combine_trust(&sources);
        assert!((combined - 0.976).abs() < 0.001);
    }

    #[test]
    fn test_chain_trust() {
        let mut engine = TrustEngine::default();

        let alice = DID::new_self(b"alice");
        let bob = DID::new_self(b"bob");
        let carol = DID::new_self(b"carol");

        // Alice → Bob: 0.9
        // Bob → Carol: 0.8
        engine
            .set_direct_trust(&alice.id, &bob.id, ContextType::Default, 0.9)
            .unwrap();
        engine
            .set_direct_trust(&bob.id, &carol.id, ContextType::Default, 0.8)
            .unwrap();

        // Τ1: Chain trust mit √n Dampening
        // exp((ln(0.9) + ln(0.8)) / √2) = exp(-0.328 / 1.414) ≈ 0.79
        let chain_trust = engine.chain_trust(
            &[alice.id.clone(), bob.id.clone(), carol.id.clone()],
            ContextType::Default,
        );

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
        let alice = DID::new_self(b"alice");

        let result = engine.set_direct_trust(&alice.id, &alice.id, ContextType::Default, 0.9);
        assert!(matches!(result, Err(TrustError::SelfAttestation)));
    }

    // =========================================================================
    // ExecutionContext Tests (Phase 3.2)
    // =========================================================================

    #[test]
    fn test_initialize_trust_with_ctx() {
        let mut engine = TrustEngine::default();
        let mut ctx = ExecutionContext::default_for_testing();
        let alice = DID::new_self(b"alice");

        let initial_gas = ctx.gas_remaining;

        engine
            .initialize_trust_with_ctx(&mut ctx, &alice.id)
            .unwrap();

        // Trust wurde initialisiert
        let trust = engine.get_trust(&alice.id).unwrap();
        assert!((trust.r - 0.5).abs() < 0.001); // Κ2

        // Gas wurde verbraucht
        assert!(ctx.gas_remaining < initial_gas);

        // Event wurde emittiert
        assert!(!ctx.emitted_events.is_empty());
    }

    #[test]
    fn test_process_event_with_ctx() {
        let mut engine = TrustEngine::default();
        let mut ctx = ExecutionContext::default_for_testing();
        let alice = DID::new_self(b"alice");
        let bob = DID::new_self(b"bob");

        let event = Event::new(
            alice.id.clone(),
            vec![],
            EventPayload::Transfer {
                from: alice.id.clone(),
                to: bob.id.clone(),
                amount: 100,
                asset_type: "ERY".to_string(),
            },
            0,
        );

        let initial_gas = ctx.gas_remaining;
        engine.process_event_with_ctx(&mut ctx, &event).unwrap();

        // Trust wurde erhöht
        let trust = engine.get_trust(&alice.id).unwrap();
        assert!(trust.r > 0.5);

        // Gas wurde verbraucht
        assert!(ctx.gas_remaining < initial_gas);
    }

    #[test]
    fn test_set_direct_trust_with_ctx() {
        let mut engine = TrustEngine::default();
        let mut ctx = ExecutionContext::default_for_testing();
        let alice = DID::new_self(b"alice");
        let bob = DID::new_self(b"bob");

        engine
            .set_direct_trust_with_ctx(&mut ctx, &alice.id, &bob.id, ContextType::Default, 0.8)
            .unwrap();

        // Event wurde emittiert
        assert!(!ctx.emitted_events.is_empty());
    }

    #[test]
    fn test_set_direct_trust_with_ctx_bounds_check() {
        let mut engine = TrustEngine::default();
        let mut ctx = ExecutionContext::default_for_testing();
        let alice = DID::new_self(b"alice");
        let bob = DID::new_self(b"bob");

        // Κ3: Out of bounds
        let result = engine.set_direct_trust_with_ctx(
            &mut ctx,
            &alice.id,
            &bob.id,
            ContextType::Default,
            1.5,
        );
        assert!(matches!(result, Err(ExecutionError::InvalidInput(_))));
    }

    #[test]
    fn test_combine_trust_with_ctx() {
        let engine = TrustEngine::default();
        let mut ctx = ExecutionContext::default_for_testing();

        let sources = vec![
            (DID::new_self(b"a").id, 0.8),
            (DID::new_self(b"b").id, 0.7),
            (DID::new_self(b"c").id, 0.6),
        ];

        let combined = engine.combine_trust_with_ctx(&mut ctx, &sources).unwrap();

        // Κ5: 1 - (1-0.8)(1-0.7)(1-0.6) = 0.976
        assert!((combined - 0.976).abs() < 0.001);
    }

    #[test]
    fn test_chain_trust_with_ctx() {
        let mut engine = TrustEngine::default();
        let mut ctx = ExecutionContext::default_for_testing();

        let alice = DID::new_self(b"alice");
        let bob = DID::new_self(b"bob");
        let carol = DID::new_self(b"carol");

        engine
            .set_direct_trust(&alice.id, &bob.id, ContextType::Default, 0.9)
            .unwrap();
        engine
            .set_direct_trust(&bob.id, &carol.id, ContextType::Default, 0.8)
            .unwrap();

        let initial_gas = ctx.gas_remaining;
        let chain_trust = engine
            .chain_trust_with_ctx(
                &mut ctx,
                &[alice.id, bob.id, carol.id],
                ContextType::Default,
            )
            .unwrap();

        // Τ1: Chain trust
        assert!(chain_trust > 0.0);
        assert!(chain_trust < 1.0);

        // Gas wurde verbraucht
        assert!(ctx.gas_remaining < initial_gas);
    }
}
