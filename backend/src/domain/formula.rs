//! # Weltformel Types
//!
//! Komponenten der Weltformel V2.0 gemäß Axiome Κ15a-d.
//!
//! ## Axiom-Referenz
//!
//! - **Κ15a (Trust-gedämpfte Surprisal)**: `𝒮(s) = ‖𝕎(s)‖² · ℐ(s)`
//! - **Κ15b (Weltformel)**: `𝔼 = Σ 𝔸(s) · σ⃗( ‖𝕎(s)‖_w · ln|ℂ(s)| · 𝒮(s) ) · Ĥ(s) · w(s,t)`
//! - **Κ15c (Sigmoid)**: `σ⃗(x) = 1 / (1 + e^(-x))`
//! - **Κ15d (Approximation)**: Count-Min Sketch für ℐ

use crate::domain::{ContextType, DID, TrustVector6D};
use serde::{Deserialize, Serialize};

/// Aktivitäts-Präsenz 𝔸(s) ∈ [0,1]
///
/// ```text
///         |{e ∈ ℂ(s) : age(e) < τ}|
/// 𝔸(s) = ─────────────────────────────
///        |{e ∈ ℂ(s) : age(e) < τ}| + κ
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Activity {
    /// Anzahl Events im Zeitfenster
    pub recent_events: u64,
    /// Zeitfenster τ in Tagen
    pub tau_days: u64,
    /// Aktivitäts-Schwelle κ
    pub kappa: u64,
}

impl Activity {
    /// Standard-Parameter: τ=90d, κ=10
    pub fn new(recent_events: u64) -> Self {
        Self {
            recent_events,
            tau_days: 90,
            kappa: 10,
        }
    }

    /// Mobile-Parameter: τ=30d, κ=10
    pub fn mobile(recent_events: u64) -> Self {
        Self {
            recent_events,
            tau_days: 30,
            kappa: 10,
        }
    }

    /// Berechne 𝔸(s)
    pub fn value(&self) -> f64 {
        let n = self.recent_events as f64;
        let k = self.kappa as f64;
        n / (n + k)
    }
}

impl Default for Activity {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Shannon-Surprisal ℐ(s) und Trust-gedämpfte Surprisal 𝒮(s)
///
/// ```text
/// ℐ(e|s) = −log₂ P(e | ℂ(s))
/// 𝒮(s) = ‖𝕎(s)‖² · ℐ(s)    (Anti-Hype)
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Surprisal {
    /// Raw Shannon-Surprisal in bits
    pub raw_surprisal: f64,
    /// Trust-Norm ‖𝕎‖
    pub trust_norm: f64,
}

impl Surprisal {
    /// Erstelle aus Frequenz
    pub fn from_frequency(frequency: f64, total: f64) -> Self {
        let probability = (frequency + 1.0) / (total + 1.0); // Laplace smoothing
        let raw = -probability.log2();

        Self {
            raw_surprisal: raw,
            trust_norm: 0.5, // Default, wird später gesetzt
        }
    }

    /// Setze Trust-Norm
    pub fn with_trust_norm(mut self, norm: f64) -> Self {
        self.trust_norm = norm;
        self
    }

    /// Κ15a: Trust-gedämpfte Surprisal 𝒮 = ‖𝕎‖² · ℐ
    pub fn dampened(&self) -> f64 {
        self.trust_norm.powi(2) * self.raw_surprisal
    }

    /// Dämpfungs-Faktor
    pub fn dampening_factor(&self) -> f64 {
        self.trust_norm.powi(2)
    }
}

impl Default for Surprisal {
    fn default() -> Self {
        Self {
            raw_surprisal: 1.0,
            trust_norm: 0.5,
        }
    }
}

/// Human-Alignment Factor Ĥ(s)
///
/// ```text
/// Ĥ(s) ∈ {1.0, 1.2, 1.5}
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HumanFactor {
    /// Nicht verifiziert oder KI-Agent
    NotVerified,
    /// Basis Human-Attestation
    BasicAttestation,
    /// Volle Human-Attestation
    FullAttestation,
}

impl HumanFactor {
    /// Numerischer Wert
    pub fn value(&self) -> f64 {
        match self {
            HumanFactor::NotVerified => 1.0,
            HumanFactor::BasicAttestation => 1.2,
            HumanFactor::FullAttestation => 1.5,
        }
    }

    /// Aus DID-Typ ableiten
    pub fn from_did(did: &DID, has_attestation: bool, full_attestation: bool) -> Self {
        if !did.is_human_capable() {
            return Self::NotVerified;
        }

        if full_attestation {
            Self::FullAttestation
        } else if has_attestation {
            Self::BasicAttestation
        } else {
            Self::NotVerified
        }
    }
}

impl Default for HumanFactor {
    fn default() -> Self {
        Self::NotVerified
    }
}

/// Vollständiger Beitrag zur Weltformel
///
/// ```text
/// 𝔼(s) = 𝔸(s) · σ⃗( ‖𝕎(s)‖_w · ln|ℂ(s)| · 𝒮(s) ) · Ĥ(s) · w(s,t)
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFormulaContribution {
    /// Subject DID
    pub subject: DID,

    /// Aktivität 𝔸(s)
    pub activity: Activity,

    /// Trust-Vektor 𝕎(s)
    pub trust: TrustVector6D,

    /// Größe der kausalen Geschichte |ℂ(s)|
    pub causal_history_size: u64,

    /// Surprisal 𝒮(s)
    pub surprisal: Surprisal,

    /// Human-Factor Ĥ(s)
    pub human_factor: HumanFactor,

    /// Temporale Gewichtung w(s,t)
    pub temporal_weight: f64,

    /// Kontext für Trust-Gewichtung
    pub context: ContextType,
}

impl WorldFormulaContribution {
    /// Erstelle neue Contribution
    pub fn new(subject: DID) -> Self {
        Self {
            subject,
            activity: Activity::default(),
            trust: TrustVector6D::default(),
            causal_history_size: 1,
            surprisal: Surprisal::default(),
            human_factor: HumanFactor::default(),
            temporal_weight: 1.0,
            context: ContextType::Default,
        }
    }

    /// Κ15c: Sigmoid-Funktion
    fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }

    /// Berechne den Beitrag 𝔼(s)
    pub fn compute(&self) -> f64 {
        // 𝔸(s)
        let a = self.activity.value();

        // ‖𝕎(s)‖_w
        let weights = self.context.weights();
        let trust_norm = self.trust.weighted_norm(&weights);

        // ln|ℂ(s)|
        let log_history = (self.causal_history_size.max(1) as f64).ln();

        // 𝒮(s) = ‖𝕎‖² · ℐ
        let dampened_surprisal = self.surprisal.with_trust_norm(trust_norm).dampened();

        // σ⃗(...)
        let inner = trust_norm * log_history * dampened_surprisal;
        let sigmoid = Self::sigmoid(inner);

        // Ĥ(s)
        let h = self.human_factor.value();

        // w(s,t)
        let w = self.temporal_weight;

        // Kombination
        a * sigmoid * h * w
    }

    /// Builder-Pattern: Mit Aktivität
    pub fn with_activity(mut self, activity: Activity) -> Self {
        self.activity = activity;
        self
    }

    /// Builder-Pattern: Mit Trust
    pub fn with_trust(mut self, trust: TrustVector6D) -> Self {
        self.trust = trust;
        self
    }

    /// Builder-Pattern: Mit Surprisal
    pub fn with_surprisal(mut self, surprisal: Surprisal) -> Self {
        self.surprisal = surprisal;
        self
    }

    /// Builder-Pattern: Mit Human-Factor
    pub fn with_human_factor(mut self, human_factor: HumanFactor) -> Self {
        self.human_factor = human_factor;
        self
    }

    /// Builder-Pattern: Mit kausaler Geschichte
    pub fn with_causal_history(mut self, size: u64) -> Self {
        self.causal_history_size = size;
        self
    }

    /// Builder-Pattern: Mit Kontext
    pub fn with_context(mut self, context: ContextType) -> Self {
        self.context = context;
        self
    }
}

/// Globaler Weltformel-Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFormulaStatus {
    /// Gesamtwert 𝔼
    pub total_e: f64,

    /// Änderung in den letzten 24h
    pub delta_24h: f64,

    /// Anzahl Entitäten
    pub entity_count: u64,

    /// Durchschnittliche Aktivität
    pub avg_activity: f64,

    /// Durchschnittliche Trust-Norm
    pub avg_trust_norm: f64,

    /// Anteil Human-verifizierter Entitäten
    pub human_verified_ratio: f64,

    /// Realm (optional, für Realm-spezifische Berechnung)
    pub realm_id: Option<String>,

    /// Zeitstempel
    pub computed_at: chrono::DateTime<chrono::Utc>,
}

impl Default for WorldFormulaStatus {
    fn default() -> Self {
        Self {
            total_e: 0.0,
            delta_24h: 0.0,
            entity_count: 0,
            avg_activity: 0.0,
            avg_trust_norm: 0.0,
            human_verified_ratio: 0.0,
            realm_id: None,
            computed_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity() {
        // Mit 10 Events und κ=10: 𝔸 = 10/(10+10) = 0.5
        let activity = Activity::new(10);
        assert!((activity.value() - 0.5).abs() < 0.001);

        // Mit 90 Events: 𝔸 = 90/(90+10) = 0.9
        let high_activity = Activity::new(90);
        assert!((high_activity.value() - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_surprisal_dampening() {
        let surprisal = Surprisal {
            raw_surprisal: 5.0,
            trust_norm: 0.9,
        };

        // 𝒮 = 0.9² × 5.0 = 0.81 × 5.0 = 4.05
        assert!((surprisal.dampened() - 4.05).abs() < 0.001);

        // Niedriger Trust → stärkere Dämpfung
        let low_trust = Surprisal {
            raw_surprisal: 5.0,
            trust_norm: 0.3,
        };

        // 𝒮 = 0.3² × 5.0 = 0.09 × 5.0 = 0.45
        assert!((low_trust.dampened() - 0.45).abs() < 0.001);
    }

    #[test]
    fn test_human_factor() {
        assert!((HumanFactor::NotVerified.value() - 1.0).abs() < 0.001);
        assert!((HumanFactor::BasicAttestation.value() - 1.2).abs() < 0.001);
        assert!((HumanFactor::FullAttestation.value() - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_world_formula_contribution() {
        let did = DID::new_self("alice");
        let contribution = WorldFormulaContribution::new(did)
            .with_activity(Activity::new(50))
            .with_trust(TrustVector6D::new(0.8, 0.9, 0.7, 0.6, 0.5, 0.9))
            .with_causal_history(1000)
            .with_human_factor(HumanFactor::BasicAttestation);

        let value = contribution.compute();

        // Value sollte positiv und sinnvoll sein
        assert!(value > 0.0);
        assert!(value < 10.0); // Sanity check
    }

    #[test]
    fn test_human_bonus_impact() {
        let did = DID::new_self("alice");

        let without_human = WorldFormulaContribution::new(did.clone())
            .with_activity(Activity::new(50))
            .with_trust(TrustVector6D::default())
            .with_causal_history(100)
            .with_human_factor(HumanFactor::NotVerified)
            .compute();

        let with_human = WorldFormulaContribution::new(did)
            .with_activity(Activity::new(50))
            .with_trust(TrustVector6D::default())
            .with_causal_history(100)
            .with_human_factor(HumanFactor::FullAttestation)
            .compute();

        // Human-verifiziert sollte 50% mehr sein
        assert!((with_human / without_human - 1.5).abs() < 0.001);
    }
}
