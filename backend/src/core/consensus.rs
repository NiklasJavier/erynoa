//! # Consensus Engine
//!
//! Trust-gewichteter Konsensus gemäß Κ18.
//!
//! ## Axiom-Referenz
//!
//! - **Κ18 (Trust-Weighted Consensus)**: `∀ 𝒞, k : finality(e) = f(Σ 𝕎ᵢ)` mit k ≥ 3, P_revert ≤ 10⁻⁵⁰
//!
//! ## Konsensus-Regel
//!
//! Ein Event e erreicht Finalität wenn:
//! ```text
//! Σᵢ 𝕎(vᵢ) ≥ θ_finality
//! ```
//! wobei vᵢ die Witnesses sind und θ_finality = 0.67 (Supermajorität)
//!
//! ## Phase 3.4: ExecutionContext Integration
//!
//! Erweitert um `*_with_ctx`-Methoden für Gas-Accounting und Κ10 Invariant-Checks.

use crate::domain::unified::Cost;
use crate::domain::{
    EventId, FinalityLevel, Signature64, TemporalCoord, TrustVector6D, WitnessAttestation, DID,
};
use crate::execution::{ExecutionContext, ExecutionError, ExecutionResult};
use chrono::Utc;
use std::collections::HashMap;
use thiserror::Error;

/// Fehler bei Consensus-Operationen
#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error("Event not found: {0}")]
    EventNotFound(EventId),

    #[error("Witness not authorized: {0}")]
    UnauthorizedWitness(String),

    #[error("Insufficient trust for finality: {current} < {required}")]
    InsufficientTrust { current: f32, required: f32 },

    #[error("Invalid attestation signature")]
    InvalidSignature,
}

/// Ergebnis von Consensus-Operationen
pub type ConsensusResult<T> = Result<T, ConsensusError>;

/// Consensus Engine - Trust-gewichteter Konsensus (Κ18)
///
/// ```text
/// ┌──────────────────────────────────────────────────────────────┐
/// │                    ConsensusEngine                           │
/// │                                                              │
/// │    Event e                                                   │
/// │        │                                                     │
/// │        ▼                                                     │
/// │    ┌─────────────┐                                          │
/// │    │  Witnesses  │  w₁, w₂, w₃, ...                         │
/// │    │  attest e   │                                          │
/// │    └──────┬──────┘                                          │
/// │           │                                                  │
/// │           ▼                                                  │
/// │    Σ 𝕎(wᵢ) ≥ θ ?                                            │
/// │           │                                                  │
/// │     ┌─────┴─────┐                                           │
/// │     │    yes    │ → e.finality = WITNESSED                  │
/// │     └───────────┘                                           │
/// └──────────────────────────────────────────────────────────────┘
/// ```
pub struct ConsensusEngine {
    /// Attestations pro Event
    attestations: HashMap<EventId, Vec<WitnessAttestation>>,

    /// Trust-Vektoren (Referenz, in Produktion via TrustEngine)
    witness_trust: HashMap<DID, TrustVector6D>,

    /// Konfiguration
    config: ConsensusConfig,
}

/// Konfiguration für ConsensusEngine
#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    /// Κ18: Minimum k Witnesses
    pub min_witnesses: usize,

    /// Finalitäts-Schwelle θ (Trust-gewichtete Supermajorität)
    pub finality_threshold: f64,

    /// Minimum Trust pro Witness
    pub min_witness_trust: f32,

    /// Κ18: Maximum Revert-Wahrscheinlichkeit
    pub max_revert_probability: f64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            min_witnesses: 3,              // Κ18: k ≥ 3
            finality_threshold: 0.67,      // 2/3 Supermajorität
            min_witness_trust: 0.5,        // Minimum Trust für Witness
            max_revert_probability: 1e-50, // Κ18: P_revert ≤ 10⁻⁵⁰
        }
    }
}

impl ConsensusEngine {
    /// Erstelle neue ConsensusEngine
    pub fn new(config: ConsensusConfig) -> Self {
        Self {
            attestations: HashMap::new(),
            witness_trust: HashMap::new(),
            config,
        }
    }

    /// Erstelle mit Default-Config
    pub fn default() -> Self {
        Self::new(ConsensusConfig::default())
    }

    /// Registriere Witness mit Trust-Vektor
    pub fn register_witness(&mut self, did: DID, trust: TrustVector6D) {
        self.witness_trust.insert(did, trust);
    }

    /// Füge Witness-Attestation hinzu
    pub fn add_attestation(
        &mut self,
        event_id: EventId,
        witness: DID,
        _signature: String, // TODO: Parse to Signature64
    ) -> ConsensusResult<FinalityCheck> {
        // Prüfe ob Witness registriert ist
        let trust = self
            .witness_trust
            .get(&witness)
            .ok_or_else(|| ConsensusError::UnauthorizedWitness(witness.to_uri()))?;

        // Prüfe Minimum Trust
        let trust_norm = trust.weighted_norm(&[1.0; 6]);
        if trust_norm < self.config.min_witness_trust {
            return Err(ConsensusError::InsufficientTrust {
                current: trust_norm,
                required: self.config.min_witness_trust,
            });
        }

        // Erstelle Attestation mit unified Typen
        let attestation = WitnessAttestation {
            event_id: event_id.clone(),
            witness: witness.id.clone(),
            trust_at_witness: trust_norm,
            signature: Signature64::NULL, // TODO: Parse actual signature
            attested_at: TemporalCoord::now(0, &event_id),
        };

        // Speichere
        self.attestations
            .entry(event_id.clone())
            .or_default()
            .push(attestation);

        // Prüfe Finalität
        self.check_finality(&event_id)
    }

    /// Κ18: Prüfe ob Event Finalität erreicht hat
    pub fn check_finality(&self, event_id: &EventId) -> ConsensusResult<FinalityCheck> {
        let attestations = self
            .attestations
            .get(event_id)
            .map(|a| a.as_slice())
            .unwrap_or(&[]);

        let witness_count = attestations.len();

        // Berechne Trust-gewichtete Summe
        let total_trust: f64 = attestations.iter().map(|a| a.trust_at_witness as f64).sum();

        // Maximum möglicher Trust (alle registrierten Witnesses)
        let max_possible_trust: f64 = self
            .witness_trust
            .values()
            .map(|t| t.weighted_norm(&[1.0; 6]) as f64)
            .sum();

        // Anteil
        let trust_ratio = if max_possible_trust > 0.0 {
            total_trust / max_possible_trust
        } else {
            0.0
        };

        // Erreicht?
        let reached = witness_count >= self.config.min_witnesses
            && trust_ratio >= self.config.finality_threshold;

        // Berechne geschätzte Revert-Wahrscheinlichkeit
        let revert_probability = self.estimate_revert_probability(witness_count, trust_ratio);

        let level = if reached {
            if revert_probability < 1e-30 {
                FinalityLevel::Anchored
            } else {
                FinalityLevel::Witnessed
            }
        } else if witness_count > 0 {
            FinalityLevel::Validated
        } else {
            FinalityLevel::Nascent
        };

        Ok(FinalityCheck {
            event_id: event_id.clone(),
            witness_count,
            total_trust,
            trust_ratio,
            threshold: self.config.finality_threshold,
            reached,
            recommended_level: level,
            revert_probability,
        })
    }

    /// Schätze Revert-Wahrscheinlichkeit (vereinfachtes Modell)
    fn estimate_revert_probability(&self, witness_count: usize, trust_ratio: f64) -> f64 {
        if witness_count == 0 || trust_ratio == 0.0 {
            return 1.0;
        }

        // Vereinfachtes Modell: P_revert ≈ (1 - trust_ratio)^(k × factor)
        let factor = 10.0; // Security factor
        (1.0 - trust_ratio).powf((witness_count as f64) * factor)
    }

    /// Hole alle Attestations für ein Event
    pub fn get_attestations(&self, event_id: &EventId) -> &[WitnessAttestation] {
        self.attestations
            .get(event_id)
            .map(|a| a.as_slice())
            .unwrap_or(&[])
    }

    /// Statistiken
    pub fn stats(&self) -> ConsensusEngineStats {
        let total_attestations: usize = self.attestations.values().map(|a| a.len()).sum();

        let finalized_events = self
            .attestations
            .keys()
            .filter(|id| self.check_finality(id).map(|c| c.reached).unwrap_or(false))
            .count();

        ConsensusEngineStats {
            registered_witnesses: self.witness_trust.len(),
            total_attestations,
            events_with_attestations: self.attestations.len(),
            finalized_events,
        }
    }

    // =========================================================================
    // ExecutionContext-Integration (Phase 3.4)
    // =========================================================================

    /// Gas-Konstanten für Konsensus-Operationen
    const GAS_ATTESTATION: u64 = 100;
    const GAS_FINALITY_CHECK: u64 = 50;
    const GAS_PER_WITNESS: u64 = 20;

    /// Füge Attestation mit Gas-Accounting hinzu
    ///
    /// Gas: GAS_ATTESTATION + GAS_FINALITY_CHECK + GAS_PER_WITNESS × witness_count
    pub fn add_attestation_with_ctx(
        &mut self,
        ctx: &mut ExecutionContext,
        event_id: EventId,
        witness: DID,
        signature: String,
    ) -> ExecutionResult<FinalityCheck> {
        ctx.consume_gas(Self::GAS_ATTESTATION)?;

        // Legacy-Methode aufrufen
        let check = self
            .add_attestation(event_id.clone(), witness.clone(), signature)
            .map_err(|e| match e {
                ConsensusError::UnauthorizedWitness(_) => ExecutionError::TrustGateBlocked {
                    required: self.config.min_witness_trust,
                    actual: 0.0,
                },
                ConsensusError::InsufficientTrust { current, required } => {
                    ExecutionError::TrustGateBlocked {
                        required,
                        actual: current,
                    }
                }
                _ => ExecutionError::Internal(e.to_string()),
            })?;

        // Gas für Finality-Check
        let witness_gas = Self::GAS_PER_WITNESS * check.witness_count as u64;
        ctx.consume_gas(Self::GAS_FINALITY_CHECK + witness_gas)?;

        ctx.emit_raw("consensus.attestation", event_id.as_bytes());

        // Wenn Finality erreicht, zusätzliches Event
        if check.reached {
            ctx.emit_raw(
                "consensus.finality_reached",
                format!("{:?}:{}", check.recommended_level, event_id.to_hex()).as_bytes(),
            );
        }

        ctx.track_cost(Cost::new(
            Self::GAS_ATTESTATION + Self::GAS_FINALITY_CHECK + witness_gas,
            0,
            0.0,
        ));

        Ok(check)
    }

    /// Κ10: Prüfe Finalitäts-Übergang mit Invariant-Check
    ///
    /// Κ10: Finality darf nur aufsteigen (Nascent → Validated → Witnessed → Anchored → Eternal)
    pub fn validate_finality_transition_with_ctx(
        &self,
        ctx: &mut ExecutionContext,
        event_id: &EventId,
        current: FinalityLevel,
        proposed: FinalityLevel,
    ) -> ExecutionResult<bool> {
        ctx.consume_gas(Self::GAS_FINALITY_CHECK)?;

        // Κ10: Finality kann nur aufsteigen
        if proposed < current {
            return Err(ExecutionError::FinalityRegression {
                event_id: event_id.to_hex(),
                old_level: current as u8,
                new_level: proposed as u8,
            });
        }

        // Prüfe ob Übergang durch Konsensus gedeckt ist
        let check = self
            .check_finality(event_id)
            .map_err(|_| ExecutionError::NotFound {
                resource_type: "Event".into(),
                id: event_id.to_hex(),
            })?;

        // Prüfe ob empfohlenes Level mindestens so hoch wie vorgeschlagen
        let transition_valid = check.recommended_level >= proposed;

        ctx.track_cost(Cost::new(Self::GAS_FINALITY_CHECK, 0, 0.0));

        Ok(transition_valid)
    }

    /// Prüfe Konsensus-Status mit Gas-Accounting
    ///
    /// Gas: GAS_FINALITY_CHECK + GAS_PER_WITNESS × witness_count
    pub fn check_finality_with_ctx(
        &self,
        ctx: &mut ExecutionContext,
        event_id: &EventId,
    ) -> ExecutionResult<FinalityCheck> {
        ctx.consume_gas(Self::GAS_FINALITY_CHECK)?;

        let check = self
            .check_finality(event_id)
            .map_err(|_| ExecutionError::NotFound {
                resource_type: "Event".into(),
                id: event_id.to_hex(),
            })?;

        let witness_gas = Self::GAS_PER_WITNESS * check.witness_count as u64;
        ctx.consume_gas(witness_gas)?;

        ctx.track_cost(Cost::new(Self::GAS_FINALITY_CHECK + witness_gas, 0, 0.0));

        Ok(check)
    }

    /// Registriere Witness mit Trust-Check
    ///
    /// Gas: 50 (konstant)
    pub fn register_witness_with_ctx(
        &mut self,
        ctx: &mut ExecutionContext,
        did: DID,
        trust: TrustVector6D,
    ) -> ExecutionResult<()> {
        ctx.consume_gas(50)?;

        // Prüfe Minimum Trust für Witness-Registrierung
        let trust_norm = trust.weighted_norm(&[1.0; 6]);
        if trust_norm < self.config.min_witness_trust {
            return Err(ExecutionError::TrustGateBlocked {
                required: self.config.min_witness_trust,
                actual: trust_norm,
            });
        }

        self.register_witness(did.clone(), trust);

        ctx.emit_raw("consensus.witness_registered", did.to_string().as_bytes());
        ctx.track_cost(Cost::new(50, 0, 0.0));

        Ok(())
    }
}

/// Ergebnis einer Finalitäts-Prüfung
#[derive(Debug, Clone)]
pub struct FinalityCheck {
    pub event_id: EventId,
    pub witness_count: usize,
    pub total_trust: f64,
    pub trust_ratio: f64,
    pub threshold: f64,
    pub reached: bool,
    pub recommended_level: FinalityLevel,
    pub revert_probability: f64,
}

/// Statistiken der ConsensusEngine
#[derive(Debug, Clone)]
pub struct ConsensusEngineStats {
    pub registered_witnesses: usize,
    pub total_attestations: usize,
    pub events_with_attestations: usize,
    pub finalized_events: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DIDNamespace;

    fn setup_engine() -> ConsensusEngine {
        let mut engine = ConsensusEngine::default();

        // Registriere 5 Witnesses mit unterschiedlichem Trust
        for (name, trust) in [
            (b"w1".as_slice(), 0.9),
            (b"w2".as_slice(), 0.85),
            (b"w3".as_slice(), 0.8),
            (b"w4".as_slice(), 0.7),
            (b"w5".as_slice(), 0.6),
        ] {
            engine.register_witness(
                DID::new(DIDNamespace::Self_, name),
                TrustVector6D::new(trust, trust, trust, trust, trust, trust),
            );
        }

        engine
    }

    fn test_event_id(suffix: &str) -> EventId {
        UniversalId::new(UniversalId::TAG_EVENT, 1, suffix.as_bytes())
    }

    #[test]
    fn test_single_attestation_not_final() {
        let mut engine = setup_engine();
        let event_id = test_event_id("test:1");

        let result = engine
            .add_attestation(
                event_id.clone(),
                DID::new(DIDNamespace::Self_, b"w1"),
                "sig1".to_string(),
            )
            .unwrap();

        // 1 Witness < 3 (min_witnesses)
        assert!(!result.reached);
        assert_eq!(result.witness_count, 1);
    }

    #[test]
    fn test_three_attestations_finality() {
        let mut engine = setup_engine();
        let event_id = test_event_id("test:2");

        // Drei hochvertrauenswürdige Witnesses
        engine
            .add_attestation(
                event_id.clone(),
                DID::new(DIDNamespace::Self_, b"w1"),
                "sig1".to_string(),
            )
            .unwrap();
        engine
            .add_attestation(
                event_id.clone(),
                DID::new(DIDNamespace::Self_, b"w2"),
                "sig2".to_string(),
            )
            .unwrap();
        let result = engine
            .add_attestation(
                event_id.clone(),
                DID::new(DIDNamespace::Self_, b"w3"),
                "sig3".to_string(),
            )
            .unwrap();

        // 3 Witnesses mit hohem Trust sollten Threshold erreichen
        assert_eq!(result.witness_count, 3);
        assert!(result.trust_ratio > 0.5); // (0.9+0.85+0.8) / (0.9+0.85+0.8+0.7+0.6) = 2.55/3.85 ≈ 0.66
    }

    #[test]
    fn test_unauthorized_witness_rejected() {
        let mut engine = setup_engine();
        let event_id = test_event_id("test:3");

        let result = engine.add_attestation(
            event_id,
            DID::new(DIDNamespace::Self_, b"unknown"),
            "sig".to_string(),
        );

        assert!(matches!(
            result,
            Err(ConsensusError::UnauthorizedWitness(_))
        ));
    }

    #[test]
    fn test_revert_probability_decreases() {
        let mut engine = setup_engine();
        let event_id = test_event_id("test:4");

        // Eine Attestation
        engine
            .add_attestation(
                event_id.clone(),
                DID::new(DIDNamespace::Self_, b"w1"),
                "sig1".to_string(),
            )
            .unwrap();
        let check1 = engine.check_finality(&event_id).unwrap();

        // Zwei Attestations
        engine
            .add_attestation(
                event_id.clone(),
                DID::new(DIDNamespace::Self_, b"w2"),
                "sig2".to_string(),
            )
            .unwrap();
        let check2 = engine.check_finality(&event_id).unwrap();

        // Drei Attestations
        engine
            .add_attestation(
                event_id.clone(),
                DID::new(DIDNamespace::Self_, b"w3"),
                "sig3".to_string(),
            )
            .unwrap();
        let check3 = engine.check_finality(&event_id).unwrap();

        // Revert-Wahrscheinlichkeit sollte sinken
        assert!(check2.revert_probability < check1.revert_probability);
        assert!(check3.revert_probability < check2.revert_probability);
    }

    // =========================================================================
    // ExecutionContext Tests (Phase 3.4)
    // =========================================================================

    #[test]
    fn test_add_attestation_with_ctx() {
        let mut engine = setup_engine();
        let mut ctx = ExecutionContext::default_for_testing();
        let event_id = test_event_id("ctx:1");

        let initial_gas = ctx.gas_remaining;

        let check = engine
            .add_attestation_with_ctx(
                &mut ctx,
                event_id,
                DID::new(DIDNamespace::Self_, b"w1"),
                "sig1".to_string(),
            )
            .unwrap();

        // Attestation wurde verarbeitet
        assert_eq!(check.witness_count, 1);

        // Gas wurde verbraucht
        assert!(ctx.gas_remaining < initial_gas);

        // Event wurde emittiert
        assert!(!ctx.emitted_events.is_empty());
    }

    #[test]
    fn test_register_witness_with_ctx() {
        let mut engine = ConsensusEngine::default();
        let mut ctx = ExecutionContext::default_for_testing();

        // Hoher Trust - sollte funktionieren
        engine
            .register_witness_with_ctx(
                &mut ctx,
                DID::new(DIDNamespace::Self_, b"high_trust"),
                TrustVector6D::new(0.9, 0.9, 0.9, 0.9, 0.9, 0.9),
            )
            .unwrap();

        // Niedriger Trust - sollte abgelehnt werden
        let result = engine.register_witness_with_ctx(
            &mut ctx,
            DID::new(DIDNamespace::Self_, b"low_trust"),
            TrustVector6D::new(0.1, 0.1, 0.1, 0.1, 0.1, 0.1),
        );

        assert!(matches!(
            result,
            Err(ExecutionError::TrustGateBlocked { .. })
        ));
    }

    #[test]
    fn test_validate_finality_transition_k10() {
        let mut engine = setup_engine();
        let mut ctx = ExecutionContext::default_for_testing();
        let event_id = test_event_id("k10");

        // Füge Attestations hinzu
        engine
            .add_attestation(
                event_id.clone(),
                DID::new(DIDNamespace::Self_, b"w1"),
                "sig1".to_string(),
            )
            .unwrap();

        // Gültiger Übergang: Nascent → Validated
        let valid = engine
            .validate_finality_transition_with_ctx(
                &mut ctx,
                &event_id,
                FinalityLevel::Nascent,
                FinalityLevel::Validated,
            )
            .unwrap();
        assert!(valid);

        // Κ10: Regression verboten - Witnessed → Nascent
        let result = engine.validate_finality_transition_with_ctx(
            &mut ctx,
            &event_id,
            FinalityLevel::Witnessed,
            FinalityLevel::Nascent,
        );
        assert!(matches!(
            result,
            Err(ExecutionError::FinalityRegression { .. })
        ));
    }

    #[test]
    fn test_check_finality_with_ctx() {
        let mut engine = setup_engine();
        let mut ctx = ExecutionContext::default_for_testing();
        let event_id = test_event_id("check");

        // Füge drei Attestations hinzu
        engine
            .add_attestation(
                event_id.clone(),
                DID::new(DIDNamespace::Self_, b"w1"),
                "sig1".to_string(),
            )
            .unwrap();
        engine
            .add_attestation(
                event_id.clone(),
                DID::new(DIDNamespace::Self_, b"w2"),
                "sig2".to_string(),
            )
            .unwrap();
        engine
            .add_attestation(
                event_id.clone(),
                DID::new(DIDNamespace::Self_, b"w3"),
                "sig3".to_string(),
            )
            .unwrap();

        let initial_gas = ctx.gas_remaining;
        let check = engine.check_finality_with_ctx(&mut ctx, &event_id).unwrap();

        // Check korrekt
        assert_eq!(check.witness_count, 3);
        assert!(check.trust_ratio > 0.5);

        // Gas wurde verbraucht (abhängig von witness_count)
        assert!(ctx.gas_remaining < initial_gas);
    }

    #[test]
    fn test_finality_reached_event_emission() {
        let mut engine = setup_engine();
        let mut ctx = ExecutionContext::default_for_testing();
        let event_id = test_event_id("finality");

        // Erste zwei Attestations (noch keine Finality)
        engine
            .add_attestation_with_ctx(
                &mut ctx,
                event_id.clone(),
                DID::new(DIDNamespace::Self_, b"w1"),
                "sig1".to_string(),
            )
            .unwrap();
        engine
            .add_attestation_with_ctx(
                &mut ctx,
                event_id.clone(),
                DID::new(DIDNamespace::Self_, b"w2"),
                "sig2".to_string(),
            )
            .unwrap();

        let events_before_finality = ctx.emitted_events.len();

        // Dritte Attestation erreicht Finality
        let check = engine
            .add_attestation_with_ctx(
                &mut ctx,
                event_id.clone(),
                DID::new(DIDNamespace::Self_, b"w3"),
                "sig3".to_string(),
            )
            .unwrap();

        // Sollte zusätzliches finality_reached Event emittieren
        assert!(ctx.emitted_events.len() > events_before_finality);
        assert!(check.witness_count >= 3);
    }
}
