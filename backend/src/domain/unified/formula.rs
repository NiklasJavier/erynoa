//! # Unified Data Model – World Formula
//!
//! Komponenten der Weltformel V2.0 gemäß Axiome Κ15a-d.
//!
//! ## Axiom-Referenz
//!
//! - **Κ15a (Trust-gedämpfte Surprisal)**: `𝒮(s) = ‖𝕎(s)‖² · ℐ(s)`
//! - **Κ15b (Weltformel)**: `𝔼 = Σ 𝔸(s) · σ⃗( ‖𝕎(s)‖_w · ln|ℂ(s)| · 𝒮(s) ) · Ĥ(s) · w(s,t)`
//! - **Κ15c (Sigmoid)**: `σ⃗(x) = 1 / (1 + e^(-x))`
//! - **Κ15d (Approximation)**: Count-Min Sketch für ℐ
//!
//! ## Migration von domain/formula.rs
//!
//! - Surprisal als echtes struct mit TemporalCoord
//! - Integration mit Cost-Algebra

use super::cost::Cost;
use super::identity::DIDNamespace;
use super::primitives::{TemporalCoord, UniversalId};
use super::trust::TrustVector6D;
use serde::{Deserialize, Serialize};

// ============================================================================
// Activity 𝔸(s)
// ============================================================================

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
    /// Zeitfenster τ in Sekunden
    pub tau_seconds: u64,
    /// Aktivitäts-Schwelle κ
    pub kappa: u64,
    /// Zeitpunkt der Berechnung
    pub computed_at: TemporalCoord,
}

impl Activity {
    /// Standard-Parameter: τ=90d, κ=10
    pub fn new(recent_events: u64, lamport: u32) -> Self {
        let id = UniversalId::NULL;
        Self {
            recent_events,
            tau_seconds: 90 * 24 * 3600, // 90 Tage
            kappa: 10,
            computed_at: TemporalCoord::now(lamport, &id),
        }
    }

    /// Mobile-Parameter: τ=30d, κ=10
    pub fn mobile(recent_events: u64, lamport: u32) -> Self {
        let id = UniversalId::NULL;
        Self {
            recent_events,
            tau_seconds: 30 * 24 * 3600, // 30 Tage
            kappa: 10,
            computed_at: TemporalCoord::now(lamport, &id),
        }
    }

    /// Berechne 𝔸(s) ∈ [0, 1)
    pub fn value(&self) -> f64 {
        let n = self.recent_events as f64;
        let k = self.kappa as f64;
        n / (n + k)
    }

    /// Update mit neuen Events
    pub fn update(&mut self, additional_events: u64, lamport: u32) {
        self.recent_events = self.recent_events.saturating_add(additional_events);
        self.computed_at = TemporalCoord::now(lamport, &UniversalId::NULL);
    }
}

impl Default for Activity {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

// ============================================================================
// Surprisal ℐ(s) und 𝒮(s)
// ============================================================================

/// Shannon-Surprisal ℐ(s) und Trust-gedämpfte Surprisal 𝒮(s) (Κ15a)
///
/// ```text
/// ℐ(e|s) = −log₂ P(e | ℂ(s))
/// 𝒮(s) = ‖𝕎(s)‖² · ℐ(s)    (Anti-Hype)
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Surprisal {
    /// Raw Shannon-Surprisal in bits
    pub raw_bits: f64,
    /// Trust-Norm ‖𝕎‖ zum Zeitpunkt der Berechnung
    pub trust_norm: f32,
    /// Event-Identifikator (für Audit)
    pub event_id: Option<UniversalId>,
    /// Zeitpunkt der Berechnung
    pub computed_at: TemporalCoord,
}

impl Surprisal {
    /// Erstelle aus Frequenz (mit Laplace-Smoothing)
    pub fn from_frequency(frequency: u64, total: u64, lamport: u32) -> Self {
        let probability = (frequency as f64 + 1.0) / (total as f64 + 1.0);
        let raw = -probability.log2();

        let id = UniversalId::NULL;
        Self {
            raw_bits: raw,
            trust_norm: 0.5, // Default
            event_id: None,
            computed_at: TemporalCoord::now(lamport, &id),
        }
    }

    /// Erstelle aus Trust-Vektor
    pub fn from_trust(trust: &TrustVector6D, frequency: u64, total: u64, lamport: u32) -> Self {
        let probability = (frequency as f64 + 1.0) / (total as f64 + 1.0);
        let raw = -probability.log2();

        let id = UniversalId::NULL;
        Self {
            raw_bits: raw,
            trust_norm: trust.weighted_norm(&TrustVector6D::default_weights()),
            event_id: None,
            computed_at: TemporalCoord::now(lamport, &id),
        }
    }

    /// Setze Event-ID (für Audit-Trail)
    pub fn with_event(mut self, event_id: UniversalId) -> Self {
        self.event_id = Some(event_id);
        self
    }

    /// Κ15a: Trust-gedämpfte Surprisal 𝒮 = ‖𝕎‖² · ℐ
    pub fn dampened(&self) -> f64 {
        (self.trust_norm as f64).powi(2) * self.raw_bits
    }

    /// Dämpfungs-Faktor ‖𝕎‖²
    pub fn dampening_factor(&self) -> f64 {
        (self.trust_norm as f64).powi(2)
    }

    /// Raw-Wert in bits
    pub fn raw(&self) -> f64 {
        self.raw_bits
    }
}

impl Default for Surprisal {
    fn default() -> Self {
        Self {
            raw_bits: 1.0,
            trust_norm: 0.5,
            event_id: None,
            computed_at: TemporalCoord::default(),
        }
    }
}

// ============================================================================
// HumanFactor Ĥ(s)
// ============================================================================

/// Human-Alignment Factor Ĥ(s) (Κ15b)
///
/// ```text
/// Ĥ(s) ∈ {1.0, 1.2, 1.5}
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum HumanFactor {
    /// Nicht verifiziert oder KI-Agent (1.0)
    NotVerified = 0,
    /// Basis Human-Attestation (1.2)
    BasicAttestation = 1,
    /// Volle Human-Attestation (1.5)
    FullAttestation = 2,
}

impl HumanFactor {
    /// Numerischer Wert
    pub fn value(&self) -> f64 {
        match self {
            Self::NotVerified => 1.0,
            Self::BasicAttestation => 1.2,
            Self::FullAttestation => 1.5,
        }
    }

    /// Aus DID-Namespace ableiten
    pub fn from_namespace(namespace: DIDNamespace, attestation_level: AttestationLevel) -> Self {
        if !namespace.is_human_capable() {
            return Self::NotVerified;
        }

        match attestation_level {
            AttestationLevel::None => Self::NotVerified,
            AttestationLevel::Basic => Self::BasicAttestation,
            AttestationLevel::Full => Self::FullAttestation,
        }
    }

    /// Bonus gegenüber NotVerified
    pub fn bonus(&self) -> f64 {
        self.value() - 1.0
    }
}

impl Default for HumanFactor {
    fn default() -> Self {
        Self::NotVerified
    }
}

/// Attestation-Level für Human-Factor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttestationLevel {
    /// Keine Attestation
    None,
    /// Basis-Attestation (z.B. E-Mail verifiziert)
    Basic,
    /// Volle Attestation (z.B. KYC)
    Full,
}

impl Default for AttestationLevel {
    fn default() -> Self {
        Self::None
    }
}

// ============================================================================
// TemporalWeight w(s,t)
// ============================================================================

/// Temporaler Gewichtungsfaktor w(s,t) (Κ15b)
///
/// ```text
/// w(s,t) = 1 / (1 + λ · Δt)
/// ```
/// wobei λ der Decay-Faktor und Δt die Zeit seit letzter Aktivität ist.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TemporalWeight {
    /// Decay-Faktor λ (typisch 0.01 pro Tag)
    pub lambda: f64,
    /// Maximales Alter in Sekunden (danach w=0)
    pub max_age_seconds: u64,
}

impl TemporalWeight {
    /// Standard: λ=0.01/Tag, max 365 Tage
    pub fn standard() -> Self {
        Self {
            lambda: 0.01 / (24.0 * 3600.0), // Pro Sekunde
            max_age_seconds: 365 * 24 * 3600,
        }
    }

    /// Schneller Decay (für volatile Inhalte)
    pub fn fast() -> Self {
        Self {
            lambda: 0.1 / (24.0 * 3600.0),
            max_age_seconds: 90 * 24 * 3600,
        }
    }

    /// Berechne Gewicht für gegebene Zeitdifferenz
    pub fn weight(&self, delta_seconds: u64) -> f64 {
        if delta_seconds > self.max_age_seconds {
            return 0.0;
        }
        1.0 / (1.0 + self.lambda * delta_seconds as f64)
    }

    /// Berechne aus Lamport-Differenz (approximiert)
    pub fn weight_from_lamport(&self, lamport_diff: u32, avg_seconds_per_lamport: f64) -> f64 {
        let delta_seconds = (lamport_diff as f64 * avg_seconds_per_lamport) as u64;
        self.weight(delta_seconds)
    }
}

impl Default for TemporalWeight {
    fn default() -> Self {
        Self::standard()
    }
}

// ============================================================================
// WorldFormulaContribution
// ============================================================================

/// Beitrag eines Subjects zur Weltformel (Κ15b)
///
/// ```text
/// contribution(s) = 𝔸(s) · σ⃗( ‖𝕎(s)‖_w · ln|ℂ(s)| · 𝒮(s) ) · Ĥ(s) · w(s,t)
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFormulaContribution {
    /// Subject-ID
    pub subject: UniversalId,
    /// Aktivitäts-Faktor 𝔸(s)
    pub activity: Activity,
    /// Trust-Vektor 𝕎(s) - Legacy-Feld für Kompatibilität
    #[serde(default)]
    pub trust: TrustVector6D,
    /// Trust-Norm ‖𝕎(s)‖_w (berechnet aus trust)
    pub trust_norm: f32,
    /// Kausale Konnektivität |ℂ(s)|
    pub causal_connectivity: u64,
    /// Surprisal 𝒮(s)
    pub surprisal: Surprisal,
    /// Human-Factor Ĥ(s)
    pub human_factor: HumanFactor,
    /// Temporaler Gewichtsfaktor w(s,t)
    pub temporal_weight: f64,
    /// Kontext für Trust-Gewichtung - Legacy-Feld für Kompatibilität
    #[serde(default)]
    pub context: super::trust::ContextType,
    /// Berechneter Beitrag
    pub contribution: f64,
    /// Kosten der Berechnung
    pub computation_cost: Cost,
    /// Zeitpunkt der Berechnung
    pub computed_at: TemporalCoord,
}

impl WorldFormulaContribution {
    /// Berechne neuen Beitrag (statische Factory-Methode)
    pub fn compute_full(
        subject: UniversalId,
        activity: Activity,
        trust: &TrustVector6D,
        causal_connectivity: u64,
        surprisal: Surprisal,
        human_factor: HumanFactor,
        temporal_weight: f64,
        lamport: u32,
    ) -> Self {
        let context = super::trust::ContextType::Default;
        let trust_norm = trust.weighted_norm(&TrustVector6D::default_weights());

        // Κ15b: Inner term
        let ln_connectivity = (causal_connectivity.max(1) as f64).ln();
        let inner = (trust_norm as f64) * ln_connectivity * surprisal.dampened();

        // Κ15c: Sigmoid
        let sigmoid = 1.0 / (1.0 + (-inner).exp());

        // Final contribution
        let contribution = activity.value() * sigmoid * human_factor.value() * temporal_weight;

        // Kosten: O(1) für diese Berechnung
        let computation_cost = Cost::new(10, 5, 0.001);

        let id = UniversalId::NULL;
        Self {
            subject,
            activity,
            trust: trust.clone(),
            trust_norm,
            causal_connectivity,
            surprisal,
            human_factor,
            temporal_weight,
            context,
            contribution,
            computation_cost,
            computed_at: TemporalCoord::now(lamport, &id),
        }
    }

    // ========================================================================
    // Modern API
    // ========================================================================

    /// Sigmoid-Funktion σ⃗(x) (Κ15c)
    pub fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }

    /// Aktualisiere mit neuem Trust-Vektor
    pub fn update_trust(&mut self, trust: &TrustVector6D, lamport: u32) {
        let new_contribution = Self::compute_full(
            self.subject,
            self.activity,
            trust,
            self.causal_connectivity,
            self.surprisal,
            self.human_factor,
            self.temporal_weight,
            lamport,
        );
        *self = new_contribution;
    }
}

// ============================================================================
// SurprisalComponents (für detaillierte Analyse)
// ============================================================================

/// Komponenten der Surprisal-Berechnung (Κ15d)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurprisalComponents {
    /// Geschätzte Frequenz (aus Count-Min Sketch)
    pub estimated_frequency: u64,
    /// Totale Beobachtungen
    pub total_observations: u64,
    /// Hash-Funktionen für Count-Min
    pub hash_count: u8,
    /// Sketch-Breite
    pub sketch_width: u32,
    /// Konfidenz der Schätzung
    pub confidence: f64,
}

impl SurprisalComponents {
    /// Erstelle aus Count-Min Sketch Parametern
    pub fn from_sketch(
        estimated_frequency: u64,
        total_observations: u64,
        hash_count: u8,
        sketch_width: u32,
    ) -> Self {
        // Konfidenz basiert auf Sketch-Qualität
        let confidence = 1.0 - (1.0 / sketch_width as f64).powi(hash_count as i32);

        Self {
            estimated_frequency,
            total_observations,
            hash_count,
            sketch_width,
            confidence,
        }
    }

    /// Berechne Surprisal aus Komponenten
    pub fn to_surprisal(&self, lamport: u32) -> Surprisal {
        Surprisal::from_frequency(self.estimated_frequency, self.total_observations, lamport)
    }
}

// ============================================================================
// WorldFormulaStatus – Globaler Weltformel-Status
// ============================================================================

/// Globaler Weltformel-Status (Κ15b)
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
    pub realm_id: Option<super::realm::RealmId>,

    /// Zeitpunkt der Berechnung
    pub computed_at: TemporalCoord,
}

impl WorldFormulaStatus {
    /// Erstelle neuen Status
    pub fn new(lamport: u32) -> Self {
        Self {
            total_e: 0.0,
            delta_24h: 0.0,
            entity_count: 0,
            avg_activity: 0.0,
            avg_trust_norm: 0.0,
            human_verified_ratio: 0.0,
            realm_id: None,
            computed_at: TemporalCoord::now(lamport, &UniversalId::NULL),
        }
    }

    /// Mit Realm-ID
    pub fn for_realm(realm_id: super::realm::RealmId, lamport: u32) -> Self {
        let mut status = Self::new(lamport);
        status.realm_id = Some(realm_id);
        status
    }

    /// Ist das System "gesund" (𝔼 > Schwellwert)?
    pub fn is_healthy(&self, threshold: f64) -> bool {
        self.total_e > threshold
    }

    /// Wächst das System?
    pub fn is_growing(&self) -> bool {
        self.delta_24h > 0.0
    }
}

impl Default for WorldFormulaStatus {
    fn default() -> Self {
        Self::new(0)
    }
}

// ============================================================================
// WorldFormulaContribution – Builder-Pattern (Kompatibilität)
// ============================================================================

impl WorldFormulaContribution {
    /// Builder-Pattern: Erstelle mit Subject (Kompatibilität mit alter API - 1 Arg)
    ///
    /// Verwendet Lamport=0 als Default.
    pub fn from_subject(subject: UniversalId) -> Self {
        Self::new(subject, 0)
    }

    /// Builder-Pattern: Erstelle mit Subject und Lamport (neue API)
    pub fn new(subject: UniversalId, lamport: u32) -> Self {
        Self {
            subject,
            activity: Activity::default(),
            trust: TrustVector6D::default(),
            trust_norm: 0.5,
            causal_connectivity: 1,
            surprisal: Surprisal::default(),
            human_factor: HumanFactor::default(),
            temporal_weight: 1.0,
            context: super::trust::ContextType::Default,
            contribution: 0.0,
            computation_cost: Cost::ZERO,
            computed_at: TemporalCoord::now(lamport, &UniversalId::NULL),
        }
    }

    /// Builder: Mit Aktivität
    pub fn with_activity(mut self, activity: Activity) -> Self {
        self.activity = activity;
        self
    }

    /// Builder: Mit Trust-Vektor
    pub fn with_trust(mut self, trust: &TrustVector6D) -> Self {
        self.trust_norm = trust.weighted_norm(&TrustVector6D::default_weights());
        self
    }

    /// Builder: Mit Kontext (für gewichtete Trust-Norm)
    pub fn with_context(self, _context: super::trust::ContextType) -> Self {
        // Kontext beeinflusst die Gewichtung - hier speichern wir nur die Norm
        // Die eigentliche Gewichtung passiert bei with_trust()
        self
    }

    /// Builder: Mit Surprisal
    pub fn with_surprisal(mut self, surprisal: Surprisal) -> Self {
        self.surprisal = surprisal;
        self
    }

    /// Builder: Mit Human-Factor
    pub fn with_human_factor(mut self, human_factor: HumanFactor) -> Self {
        self.human_factor = human_factor;
        self
    }

    /// Builder: Mit kausaler Geschichte
    pub fn with_causal_history(mut self, size: u64) -> Self {
        self.causal_connectivity = size;
        self
    }

    /// Builder: Mit temporaler Gewichtung
    pub fn with_temporal_weight(mut self, weight: f64) -> Self {
        self.temporal_weight = weight;
        self
    }

    /// Berechne Beitrag (für Builder-Pattern)
    pub fn build(mut self) -> Self {
        // Κ15b: Inner term
        let ln_connectivity = (self.causal_connectivity.max(1) as f64).ln();
        let inner = (self.trust_norm as f64) * ln_connectivity * self.surprisal.dampened();

        // Κ15c: Sigmoid
        let sigmoid = 1.0 / (1.0 + (-inner).exp());

        // Final contribution
        self.contribution =
            self.activity.value() * sigmoid * self.human_factor.value() * self.temporal_weight;

        // Kosten: O(1)
        self.computation_cost = Cost::new(10, 5, 0.001);

        self
    }

    /// Berechne Beitrag und gib Wert zurück (Kompatibilität mit alter API)
    pub fn compute_value(&self) -> f64 {
        let ln_connectivity = (self.causal_connectivity.max(1) as f64).ln();
        let inner = (self.trust_norm as f64) * ln_connectivity * self.surprisal.dampened();
        let sigmoid = 1.0 / (1.0 + (-inner).exp());
        self.activity.value() * sigmoid * self.human_factor.value() * self.temporal_weight
    }

    /// Alias für compute_value() - alte API Kompatibilität
    /// Erwartet keine Argumente und berechnet den Beitrag
    #[inline]
    pub fn compute(&self) -> f64 {
        self.compute_value()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_value() {
        // n=0, k=10 → A = 0
        let a0 = Activity::new(0, 1);
        assert!((a0.value() - 0.0).abs() < 0.001);

        // n=10, k=10 → A = 0.5
        let a10 = Activity::new(10, 1);
        assert!((a10.value() - 0.5).abs() < 0.001);

        // n=100, k=10 → A ≈ 0.909
        let a100 = Activity::new(100, 1);
        assert!(a100.value() > 0.9);
    }

    #[test]
    fn test_surprisal_dampening() {
        let s = Surprisal {
            raw_bits: 4.0, // 4 bits
            trust_norm: 0.5,
            event_id: None,
            computed_at: TemporalCoord::default(),
        };

        // 𝒮 = 0.5² × 4.0 = 0.25 × 4.0 = 1.0
        assert!((s.dampened() - 1.0).abs() < 0.001);

        // Hoher Trust → weniger Dämpfung
        let s_high = Surprisal {
            raw_bits: 4.0,
            trust_norm: 1.0,
            event_id: None,
            computed_at: TemporalCoord::default(),
        };
        assert!((s_high.dampened() - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_human_factor() {
        assert!((HumanFactor::NotVerified.value() - 1.0).abs() < 0.001);
        assert!((HumanFactor::BasicAttestation.value() - 1.2).abs() < 0.001);
        assert!((HumanFactor::FullAttestation.value() - 1.5).abs() < 0.001);

        // Bonus
        assert!((HumanFactor::FullAttestation.bonus() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_temporal_weight() {
        let tw = TemporalWeight::standard();

        // Δt=0 → w=1.0
        assert!((tw.weight(0) - 1.0).abs() < 0.001);

        // Δt sehr groß → w→0
        assert!(tw.weight(365 * 24 * 3600 + 1) < 0.001);

        // Mittlerer Wert
        let w = tw.weight(30 * 24 * 3600); // 30 Tage
        assert!(w > 0.5 && w < 1.0);
    }

    #[test]
    fn test_sigmoid() {
        // σ(0) = 0.5
        assert!((WorldFormulaContribution::sigmoid(0.0) - 0.5).abs() < 0.001);

        // σ(-∞) → 0
        assert!(WorldFormulaContribution::sigmoid(-10.0) < 0.001);

        // σ(+∞) → 1
        assert!(WorldFormulaContribution::sigmoid(10.0) > 0.999);
    }

    #[test]
    fn test_world_formula_contribution() {
        let subject = UniversalId::new(UniversalId::TAG_DID, 1, b"alice");
        let activity = Activity::new(50, 1);
        let trust = TrustVector6D::default();
        let surprisal = Surprisal::from_frequency(5, 100, 1);
        let human_factor = HumanFactor::BasicAttestation;

        let contrib = WorldFormulaContribution::new(subject, 0)
            .with_activity(activity)
            .with_trust(&trust)
            .with_causal_history(100)
            .with_surprisal(surprisal)
            .with_human_factor(human_factor)
            .with_temporal_weight(0.9)
            .build();

        // Beitrag sollte positiv sein
        let value = contrib.compute();
        assert!(value > 0.0);
        // Und kleiner als 2 (wegen Sigmoid + Faktoren)
        assert!(value < 2.0);
    }

    #[test]
    fn test_surprisal_components() {
        let comp = SurprisalComponents::from_sketch(10, 1000, 5, 1024);

        assert_eq!(comp.estimated_frequency, 10);
        assert!(comp.confidence > 0.9);

        let surprisal = comp.to_surprisal(1);
        assert!(surprisal.raw_bits > 0.0);
    }
}
