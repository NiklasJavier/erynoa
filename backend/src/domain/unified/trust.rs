//! # Unified Data Model – Trust-Strukturen
//!
//! 6-dimensionaler Trust-Vektor gemäß Axiome Κ2-Κ5.
//!
//! ## Axiom-Referenz
//!
//! - **Κ2 (Trust-Funktor)**: `𝕋(g ∘ f) = 𝕋(f) ∘ 𝕋(g)` (Kontravariant)
//! - **Κ3 (Dimensionale Unabhängigkeit)**: `∂𝕎ᵢ/∂event ⊥ ∂𝕎ⱼ/∂event`
//! - **Κ4 (Asymmetrische Evolution)**: `Δ⁻(dim) = λ_asym · Δ⁺(dim)`
//! - **Κ5 (Probabilistische Kombination)**: `t₁ ⊕ t₂ = 1 - (1-t₁)(1-t₂)`

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

use super::primitives::{TemporalCoord, UniversalId};

// ============================================================================
// TrustDimension – Die 6 Trust-Dimensionen
// ============================================================================

/// Die 6 Trust-Dimensionen
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum TrustDimension {
    /// R - Reliability (Verhaltens-Historie)
    Reliability = 0,
    /// I - Integrity (Aussage-Konsistenz)
    Integrity = 1,
    /// C - Competence (Fähigkeits-Nachweis)
    Competence = 2,
    /// P - Prestige (Externe Attestation)
    Prestige = 3,
    /// V - Vigilance (Anomalie-Erkennung)
    Vigilance = 4,
    /// Ω - Omega (Axiom-Treue)
    Omega = 5,
}

impl TrustDimension {
    /// Alle Dimensionen
    pub const ALL: [Self; 6] = [
        Self::Reliability,
        Self::Integrity,
        Self::Competence,
        Self::Prestige,
        Self::Vigilance,
        Self::Omega,
    ];

    /// Asymmetrie-Faktor für diese Dimension (Κ4)
    #[inline]
    pub fn asymmetry_factor(&self) -> f32 {
        match self {
            Self::Reliability | Self::Integrity | Self::Competence | Self::Prestige => 1.5,
            Self::Vigilance | Self::Omega => 2.0,
        }
    }

    /// Index in Arrays
    #[inline]
    pub fn index(&self) -> usize {
        *self as usize
    }

    /// Von Index
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Reliability),
            1 => Some(Self::Integrity),
            2 => Some(Self::Competence),
            3 => Some(Self::Prestige),
            4 => Some(Self::Vigilance),
            5 => Some(Self::Omega),
            _ => None,
        }
    }

    /// Kurz-Symbol
    pub fn symbol(&self) -> char {
        match self {
            Self::Reliability => 'R',
            Self::Integrity => 'I',
            Self::Competence => 'C',
            Self::Prestige => 'P',
            Self::Vigilance => 'V',
            Self::Omega => 'Ω',
        }
    }
}

impl fmt::Display for TrustDimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

// ============================================================================
// TrustVector6D – Der 6-dimensionale Trust-Vektor
// ============================================================================

/// 6-dimensionaler Trust-Vektor 𝕎 ∈ [0,1]⁶
///
/// Kompaktes Layout: 24 Bytes (4 Bytes pro Dimension)
///
/// # Beispiel
///
/// ```rust
/// use erynoa_api::domain::unified::TrustVector6D;
///
/// let trust = TrustVector6D::default();
/// assert_eq!(trust.r, 0.5);
///
/// let newcomer = TrustVector6D::NEWCOMER;
/// assert_eq!(newcomer.r, 0.1);
/// ```
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(C, align(8))]
pub struct TrustVector6D {
    /// R - Reliability
    pub r: f32,
    /// I - Integrity
    pub i: f32,
    /// C - Competence
    pub c: f32,
    /// P - Prestige
    pub p: f32,
    /// V - Vigilance
    pub v: f32,
    /// Ω - Omega (Axiom-Treue)
    pub omega: f32,
}

impl TrustVector6D {
    /// Newcomer-Werte (Sybil-Schutz)
    pub const NEWCOMER: Self = Self {
        r: 0.1,
        i: 0.1,
        c: 0.1,
        p: 0.1,
        v: 0.1,
        omega: 0.1,
    };

    /// Default für etablierte Entitäten
    pub const DEFAULT: Self = Self {
        r: 0.5,
        i: 0.5,
        c: 0.5,
        p: 0.5,
        v: 0.5,
        omega: 0.5,
    };

    /// Null-Vektor
    pub const ZERO: Self = Self {
        r: 0.0,
        i: 0.0,
        c: 0.0,
        p: 0.0,
        v: 0.0,
        omega: 0.0,
    };

    /// Maximum (perfekt)
    pub const MAX: Self = Self {
        r: 1.0,
        i: 1.0,
        c: 1.0,
        p: 1.0,
        v: 1.0,
        omega: 1.0,
    };

    /// Erstelle neuen Trust-Vektor mit gegebenen Werten
    pub fn new(r: f32, i: f32, c: f32, p: f32, v: f32, omega: f32) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            i: i.clamp(0.0, 1.0),
            c: c.clamp(0.0, 1.0),
            p: p.clamp(0.0, 1.0),
            v: v.clamp(0.0, 1.0),
            omega: omega.clamp(0.0, 1.0),
        }
    }

    /// Von Array erstellen
    pub fn from_array(arr: [f32; 6]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3], arr[4], arr[5])
    }

    /// Als Array
    #[inline]
    pub fn to_array(&self) -> [f32; 6] {
        [self.r, self.i, self.c, self.p, self.v, self.omega]
    }

    /// Dimension nach Index
    #[inline]
    pub fn get(&self, dim: TrustDimension) -> f32 {
        match dim {
            TrustDimension::Reliability => self.r,
            TrustDimension::Integrity => self.i,
            TrustDimension::Competence => self.c,
            TrustDimension::Prestige => self.p,
            TrustDimension::Vigilance => self.v,
            TrustDimension::Omega => self.omega,
        }
    }

    /// Dimension setzen
    #[inline]
    pub fn set(&mut self, dim: TrustDimension, value: f32) {
        let value = value.clamp(0.0, 1.0);
        match dim {
            TrustDimension::Reliability => self.r = value,
            TrustDimension::Integrity => self.i = value,
            TrustDimension::Competence => self.c = value,
            TrustDimension::Prestige => self.p = value,
            TrustDimension::Vigilance => self.v = value,
            TrustDimension::Omega => self.omega = value,
        }
    }

    /// Gewichtete Norm (Κ3)
    #[inline]
    pub fn weighted_norm(&self, weights: &[f32; 6]) -> f32 {
        let arr = self.to_array();
        let mut sum = 0.0f32;
        for i in 0..6 {
            sum += weights[i] * arr[i] * arr[i];
        }
        sum.sqrt()
    }

    /// Euklidische Norm
    #[inline]
    pub fn euclidean_norm(&self) -> f32 {
        let arr = self.to_array();
        let sum: f32 = arr.iter().map(|x| x * x).sum();
        sum.sqrt()
    }

    /// Bayesian Update (Κ4: Asymmetrie)
    pub fn update(&mut self, dim: TrustDimension, delta: f32) {
        let current = self.get(dim);
        let asymmetry = dim.asymmetry_factor();

        let new_value = if delta < 0.0 {
            // Negative Updates stärker gewichtet (Κ4)
            (current + delta * asymmetry).clamp(0.0, 1.0)
        } else {
            (current + delta).clamp(0.0, 1.0)
        };

        self.set(dim, new_value);
    }

    /// Probabilistische Kombination (Κ5)
    pub fn combine(&self, other: &Self) -> Self {
        Self {
            r: 1.0 - (1.0 - self.r) * (1.0 - other.r),
            i: 1.0 - (1.0 - self.i) * (1.0 - other.i),
            c: 1.0 - (1.0 - self.c) * (1.0 - other.c),
            p: 1.0 - (1.0 - self.p) * (1.0 - other.p),
            v: 1.0 - (1.0 - self.v) * (1.0 - other.v),
            omega: 1.0 - (1.0 - self.omega) * (1.0 - other.omega),
        }
    }

    /// Interpolation zwischen zwei Vektoren
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            r: self.r + (other.r - self.r) * t,
            i: self.i + (other.i - self.i) * t,
            c: self.c + (other.c - self.c) * t,
            p: self.p + (other.p - self.p) * t,
            v: self.v + (other.v - self.v) * t,
            omega: self.omega + (other.omega - self.omega) * t,
        }
    }

    /// Minimum-Wert über alle Dimensionen
    pub fn min(&self) -> f32 {
        self.r
            .min(self.i)
            .min(self.c)
            .min(self.p)
            .min(self.v)
            .min(self.omega)
    }

    /// Maximum-Wert über alle Dimensionen
    pub fn max(&self) -> f32 {
        self.r
            .max(self.i)
            .max(self.c)
            .max(self.p)
            .max(self.v)
            .max(self.omega)
    }

    /// Durchschnitt
    pub fn mean(&self) -> f32 {
        (self.r + self.i + self.c + self.p + self.v + self.omega) / 6.0
    }
}

impl Default for TrustVector6D {
    /// Neutraler Startwert für ETABLIERTE Entitäten: 0.5
    /// Für neue Nutzer: `TrustVector6D::NEWCOMER` verwenden!
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl fmt::Debug for TrustVector6D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "𝕎[R={:.2}, I={:.2}, C={:.2}, P={:.2}, V={:.2}, Ω={:.2}]",
            self.r, self.i, self.c, self.p, self.v, self.omega
        )
    }
}

impl fmt::Display for TrustVector6D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({:.2},{:.2},{:.2},{:.2},{:.2},{:.2})",
            self.r, self.i, self.c, self.p, self.v, self.omega
        )
    }
}

// ============================================================================
// ContextType – Kontext für Trust-Gewichtung
// ============================================================================

/// Kontext-Typen für Trust-Gewichtung
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum ContextType {
    Default = 0x00,
    Finance = 0x01,
    Social = 0x02,
    Governance = 0x03,
    Technical = 0x04,
    Creative = 0x05,
}

impl ContextType {
    /// Standard-Gewichte pro Kontext
    pub fn default_weights(&self) -> [f32; 6] {
        match self {
            Self::Default => [0.17, 0.17, 0.17, 0.17, 0.16, 0.16],
            Self::Finance => [0.25, 0.25, 0.15, 0.15, 0.10, 0.10],
            Self::Social => [0.10, 0.15, 0.10, 0.30, 0.25, 0.10],
            Self::Governance => [0.15, 0.20, 0.10, 0.10, 0.10, 0.35],
            Self::Technical => [0.15, 0.15, 0.35, 0.10, 0.15, 0.10],
            Self::Creative => [0.10, 0.15, 0.25, 0.25, 0.15, 0.10],
        }
    }

    /// Berechne Trust-Norm für diesen Kontext
    pub fn compute_norm(&self, trust: &TrustVector6D) -> f32 {
        trust.weighted_norm(&self.default_weights())
    }
}

impl Default for ContextType {
    fn default() -> Self {
        Self::Default
    }
}

// ============================================================================
// TrustRecord – Vollständiger Trust-Datensatz
// ============================================================================

/// Vollständiger Trust-Record mit History
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustRecord {
    // === Identifikation ===
    pub subject_id: UniversalId,
    pub updated_at: TemporalCoord,

    // === Aktueller Vektor ===
    pub vector: TrustVector6D,

    // === Bayesian Posterior Confidence ===
    pub confidence: [f32; 6],
    pub sample_count: [u32; 6],

    // === Kontext-spezifische Overrides ===
    pub contexts: BTreeMap<ContextType, TrustVector6D>,

    // === History (komprimiert) ===
    pub history: TrustHistory,
}

impl TrustRecord {
    /// Erstelle neuen Record für Newcomer
    pub fn newcomer(subject_id: UniversalId, coord: TemporalCoord) -> Self {
        Self {
            subject_id,
            updated_at: coord,
            vector: TrustVector6D::NEWCOMER,
            confidence: [0.1; 6], // Geringe Konfidenz
            sample_count: [0; 6],
            contexts: BTreeMap::new(),
            history: TrustHistory::default(),
        }
    }

    /// Erstelle Record mit Default-Trust
    pub fn established(subject_id: UniversalId, coord: TemporalCoord) -> Self {
        Self {
            subject_id,
            updated_at: coord,
            vector: TrustVector6D::DEFAULT,
            confidence: [0.5; 6],
            sample_count: [10; 6], // Einige Samples angenommen
            contexts: BTreeMap::new(),
            history: TrustHistory::default(),
        }
    }

    /// Update Trust-Dimension
    pub fn update(
        &mut self,
        dim: TrustDimension,
        delta: f32,
        reason: TrustUpdateReason,
        coord: TemporalCoord,
    ) {
        let old_value = self.vector.get(dim);
        self.vector.update(dim, delta);
        let new_value = self.vector.get(dim);

        // Update Confidence (Bayesian)
        let idx = dim.index();
        self.sample_count[idx] += 1;
        self.confidence[idx] = (self.confidence[idx] * (self.sample_count[idx] - 1) as f32
            + (1.0 - (old_value - new_value).abs()))
            / self.sample_count[idx] as f32;

        // History Entry
        self.history.add_entry(TrustHistoryEntry {
            timestamp: coord,
            dimension: dim,
            old_value,
            new_value,
            delta,
            reason,
        });

        self.updated_at = coord;
    }

    /// Trust für spezifischen Kontext
    pub fn trust_for_context(&self, context: ContextType) -> TrustVector6D {
        self.contexts.get(&context).copied().unwrap_or(self.vector)
    }

    /// Gewichtete Norm für Kontext
    pub fn norm_for_context(&self, context: ContextType) -> f32 {
        let trust = self.trust_for_context(context);
        context.compute_norm(&trust)
    }
}

// ============================================================================
// TrustHistory – Komprimierte Historie
// ============================================================================

/// Trust-History mit Retention-Policy
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TrustHistory {
    /// Letzte Einträge (max 100)
    pub recent: Vec<TrustHistoryEntry>,
    /// Aggregierte Statistiken pro Tag
    pub daily_stats: Vec<DailyTrustStats>,
}

impl TrustHistory {
    const MAX_RECENT: usize = 100;

    /// Eintrag hinzufügen
    pub fn add_entry(&mut self, entry: TrustHistoryEntry) {
        self.recent.push(entry);

        // Trim wenn zu viele
        if self.recent.len() > Self::MAX_RECENT {
            // Aggregiere alte Einträge
            self.aggregate_old_entries();
        }
    }

    fn aggregate_old_entries(&mut self) {
        // Behalte nur die neuesten 50
        if self.recent.len() > 50 {
            let to_aggregate: Vec<_> = self.recent.drain(0..self.recent.len() - 50).collect();

            // Aggregiere zu DailyStats (vereinfacht)
            if !to_aggregate.is_empty() {
                let first = &to_aggregate[0];
                let last = to_aggregate.last().unwrap();

                let mut deltas = [0.0f32; 6];
                let mut counts = [0u32; 6];

                for entry in &to_aggregate {
                    let idx = entry.dimension.index();
                    deltas[idx] += entry.delta;
                    counts[idx] += 1;
                }

                self.daily_stats.push(DailyTrustStats {
                    period_start: first.timestamp,
                    period_end: last.timestamp,
                    net_deltas: deltas,
                    update_counts: counts,
                });
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustHistoryEntry {
    pub timestamp: TemporalCoord,
    pub dimension: TrustDimension,
    pub old_value: f32,
    pub new_value: f32,
    pub delta: f32,
    pub reason: TrustUpdateReason,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyTrustStats {
    pub period_start: TemporalCoord,
    pub period_end: TemporalCoord,
    pub net_deltas: [f32; 6],
    pub update_counts: [u32; 6],
}

/// Grund für Trust-Update
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TrustUpdateReason {
    DirectInteraction,
    AttestationReceived,
    DelegationCreated,
    DelegationRevoked,
    VouchReceived,
    VouchRevoked,
    PolicyViolation,
    PositiveContribution,
    AnomalyDetected,
    SystemAdjustment,
    DecayOverTime,
    Custom(String),
}

// ============================================================================
// Compile-Time Assertions
// ============================================================================

const _: () = {
    // TrustVector6D sollte 24 Bytes sein (6 × f32)
    assert!(std::mem::size_of::<TrustVector6D>() == 24);
    // Alignment für SIMD
    assert!(std::mem::align_of::<TrustVector6D>() >= 4);
};

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_vector_creation() {
        let trust = TrustVector6D::new(0.8, 0.7, 0.6, 0.5, 0.4, 0.3);

        assert_eq!(trust.r, 0.8);
        assert_eq!(trust.omega, 0.3);
    }

    #[test]
    fn test_trust_vector_clamping() {
        let trust = TrustVector6D::new(1.5, -0.5, 0.5, 0.5, 0.5, 0.5);

        assert_eq!(trust.r, 1.0);
        assert_eq!(trust.i, 0.0);
    }

    #[test]
    fn test_trust_vector_combine() {
        let t1 = TrustVector6D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5);
        let t2 = TrustVector6D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5);

        let combined = t1.combine(&t2);

        // 1 - (1-0.5)(1-0.5) = 1 - 0.25 = 0.75
        assert!((combined.r - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_trust_vector_update_asymmetry() {
        let mut trust = TrustVector6D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5);

        // Positive Update
        trust.update(TrustDimension::Reliability, 0.1);
        assert!((trust.r - 0.6).abs() < 0.001);

        // Negative Update (asymmetrisch: 1.5×)
        trust.update(TrustDimension::Reliability, -0.1);
        // 0.6 - 0.1 * 1.5 = 0.6 - 0.15 = 0.45
        assert!((trust.r - 0.45).abs() < 0.001);
    }

    #[test]
    fn test_trust_vector_weighted_norm() {
        let trust = TrustVector6D::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
        let weights = [0.17, 0.17, 0.17, 0.17, 0.16, 0.16];

        let norm = trust.weighted_norm(&weights);

        // sqrt(0.17 + 0.17 + 0.17 + 0.17 + 0.16 + 0.16) = sqrt(1.0) = 1.0
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_context_weights() {
        let trust = TrustVector6D::new(0.8, 0.8, 0.5, 0.5, 0.5, 0.5);

        // Finance gewichtet R und I höher
        let finance_norm = ContextType::Finance.compute_norm(&trust);
        let social_norm = ContextType::Social.compute_norm(&trust);

        // Finance sollte höher sein weil R und I hoch sind
        assert!(finance_norm > social_norm);
    }

    #[test]
    fn test_trust_record_update() {
        let subject = UniversalId::new(UniversalId::TAG_DID, 1, b"test");
        let coord = TemporalCoord::new(1000, 1, 1);

        let mut record = TrustRecord::newcomer(subject, coord);

        assert_eq!(record.vector.r, 0.1);

        record.update(
            TrustDimension::Reliability,
            0.1,
            TrustUpdateReason::PositiveContribution,
            TemporalCoord::new(1001, 2, 1),
        );

        assert!((record.vector.r - 0.2).abs() < 0.001);
        assert_eq!(record.sample_count[0], 1);
    }
}
