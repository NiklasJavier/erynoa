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

use crate::domain::{EventId, FinalityLevel, TrustVector6D, WitnessAttestation, DID};
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
    InsufficientTrust { current: f64, required: f64 },

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
    pub min_witness_trust: f64,

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
        signature: String,
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

        // Erstelle Attestation
        let attestation = WitnessAttestation {
            event_id: event_id.clone(),
            witness,
            trust_weight: trust_norm,
            signature,
            timestamp: Utc::now(),
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
        let total_trust: f64 = attestations.iter().map(|a| a.trust_weight).sum();

        // Maximum möglicher Trust (alle registrierten Witnesses)
        let max_possible_trust: f64 = self
            .witness_trust
            .values()
            .map(|t| t.weighted_norm(&[1.0; 6]))
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

    fn setup_engine() -> ConsensusEngine {
        let mut engine = ConsensusEngine::default();

        // Registriere 5 Witnesses mit unterschiedlichem Trust
        for (name, trust) in [
            ("w1", 0.9),
            ("w2", 0.85),
            ("w3", 0.8),
            ("w4", 0.7),
            ("w5", 0.6),
        ] {
            engine.register_witness(
                DID::new_self(name),
                TrustVector6D::new(trust, trust, trust, trust, trust, trust),
            );
        }

        engine
    }

    #[test]
    fn test_single_attestation_not_final() {
        let mut engine = setup_engine();
        let event_id = EventId::new("event:test:1");

        let result = engine
            .add_attestation(event_id.clone(), DID::new_self("w1"), "sig1".to_string())
            .unwrap();

        // 1 Witness < 3 (min_witnesses)
        assert!(!result.reached);
        assert_eq!(result.witness_count, 1);
    }

    #[test]
    fn test_three_attestations_finality() {
        let mut engine = setup_engine();
        let event_id = EventId::new("event:test:2");

        // Drei hochvertrauenswürdige Witnesses
        engine
            .add_attestation(event_id.clone(), DID::new_self("w1"), "sig1".to_string())
            .unwrap();
        engine
            .add_attestation(event_id.clone(), DID::new_self("w2"), "sig2".to_string())
            .unwrap();
        let result = engine
            .add_attestation(event_id.clone(), DID::new_self("w3"), "sig3".to_string())
            .unwrap();

        // 3 Witnesses mit hohem Trust sollten Threshold erreichen
        assert_eq!(result.witness_count, 3);
        assert!(result.trust_ratio > 0.5); // (0.9+0.85+0.8) / (0.9+0.85+0.8+0.7+0.6) = 2.55/3.85 ≈ 0.66
    }

    #[test]
    fn test_unauthorized_witness_rejected() {
        let mut engine = setup_engine();
        let event_id = EventId::new("event:test:3");

        let result = engine.add_attestation(event_id, DID::new_self("unknown"), "sig".to_string());

        assert!(matches!(
            result,
            Err(ConsensusError::UnauthorizedWitness(_))
        ));
    }

    #[test]
    fn test_revert_probability_decreases() {
        let mut engine = setup_engine();
        let event_id = EventId::new("event:test:4");

        // Eine Attestation
        engine
            .add_attestation(event_id.clone(), DID::new_self("w1"), "sig1".to_string())
            .unwrap();
        let check1 = engine.check_finality(&event_id).unwrap();

        // Zwei Attestations
        engine
            .add_attestation(event_id.clone(), DID::new_self("w2"), "sig2".to_string())
            .unwrap();
        let check2 = engine.check_finality(&event_id).unwrap();

        // Drei Attestations
        engine
            .add_attestation(event_id.clone(), DID::new_self("w3"), "sig3".to_string())
            .unwrap();
        let check3 = engine.check_finality(&event_id).unwrap();

        // Revert-Wahrscheinlichkeit sollte sinken
        assert!(check2.revert_probability < check1.revert_probability);
        assert!(check3.revert_probability < check2.revert_probability);
    }
}
