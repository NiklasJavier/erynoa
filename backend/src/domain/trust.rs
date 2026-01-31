//! # Trust Engine Types
//!
//! 6-dimensionaler Trust-Vektor 𝕎 gemäß Axiome Κ2-Κ5.
//!
//! ## Axiom-Referenz
//!
//! - **Κ2 (Trust-Funktor)**: `𝕋(g ∘ f) = 𝕋(f) ∘ 𝕋(g)` (Kontravariant)
//! - **Κ3 (Dimensionale Unabhängigkeit)**: `∂𝕎ᵢ/∂event ⊥ ∂𝕎ⱼ/∂event`
//! - **Κ4 (Asymmetrische Evolution)**: `Δ⁻(dim) = λ_asym · Δ⁺(dim)`
//! - **Κ5 (Probabilistische Kombination)**: `t₁ ⊕ t₂ = 1 - (1-t₁)(1-t₂)`

use serde::{Deserialize, Serialize};

/// Die 6 Trust-Dimensionen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrustDimension {
    /// R - Reliability (Verhaltens-Historie)
    Reliability,
    /// I - Integrity (Aussage-Konsistenz)
    Integrity,
    /// C - Competence (Fähigkeits-Nachweis)
    Competence,
    /// P - Prestige (Externe Attestation)
    Prestige,
    /// V - Vigilance (Anomalie-Erkennung)
    Vigilance,
    /// Ω - Omega (Axiom-Treue)
    Omega,
}

impl TrustDimension {
    /// Asymmetrie-Faktor für diese Dimension (Κ4)
    /// V und Ω haben höhere Asymmetrie
    pub fn asymmetry_factor(&self) -> f64 {
        match self {
            TrustDimension::Reliability => 1.5,
            TrustDimension::Integrity => 1.5,
            TrustDimension::Competence => 1.5,
            TrustDimension::Prestige => 1.5,
            TrustDimension::Vigilance => 2.0,
            TrustDimension::Omega => 2.0,
        }
    }

    /// Index in Arrays
    pub fn index(&self) -> usize {
        match self {
            TrustDimension::Reliability => 0,
            TrustDimension::Integrity => 1,
            TrustDimension::Competence => 2,
            TrustDimension::Prestige => 3,
            TrustDimension::Vigilance => 4,
            TrustDimension::Omega => 5,
        }
    }
}

/// 6-dimensionaler Trust-Vektor 𝕎 ∈ [0,1]⁶
///
/// # Beispiel
/// ```
/// use erynoa_api::domain::TrustVector6D;
///
/// let trust = TrustVector6D::default();
/// assert_eq!(trust.weighted_norm(&TrustVector6D::default_weights()), 0.5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TrustVector6D {
    /// R - Reliability
    pub r: f64,
    /// I - Integrity
    pub i: f64,
    /// C - Competence
    pub c: f64,
    /// P - Prestige
    pub p: f64,
    /// V - Vigilance
    pub v: f64,
    /// Ω - Omega (Axiom-Treue)
    pub omega: f64,
}

impl Default for TrustVector6D {
    /// Neutraler Startwert für ETABLIERTE Entitäten: 0.5
    /// Für neue Nutzer: `TrustVector6D::newcomer()` verwenden!
    fn default() -> Self {
        Self {
            r: 0.5,
            i: 0.5,
            c: 0.5,
            p: 0.5,
            v: 0.5,
            omega: 0.5,
        }
    }
}

impl TrustVector6D {
    /// Sybil-Schutz: Niedriger Startwert für NEUE Nutzer (0.1)
    ///
    /// Durch dampened_surprisal (trust²) hat ein Newcomer mit 0.1
    /// nur 0.01 (1%) des Einflusses eines etablierten Nutzers.
    ///
    /// Trust muss durch positive Interaktionen verdient werden.
    pub fn newcomer() -> Self {
        Self {
            r: 0.1,
            i: 0.1,
            c: 0.1,
            p: 0.1,
            v: 0.1,
            omega: 0.1,
        }
    }

    /// Vouched Newcomer: Bürge transferiert Teil seines Trusts
    ///
    /// Ein etablierter Nutzer kann für einen Newcomer bürgen und
    /// damit einen Teil seines Prestige-Scores "staken".
    /// Bei Fehlverhalten des Newcomers wird auch der Bürge bestraft.
    pub fn vouched(voucher_trust: &TrustVector6D, stake_ratio: f64) -> Self {
        let stake = stake_ratio.clamp(0.0, 0.3); // Max 30% Transfer
        Self {
            r: 0.1 + voucher_trust.r * stake * 0.5, // Reliability kann nicht gebürgt werden
            i: 0.1 + voucher_trust.i * stake * 0.5,
            c: 0.1,                           // Competence muss selbst bewiesen werden
            p: 0.1 + voucher_trust.p * stake, // Prestige kann übertragen werden
            v: 0.1,                           // Vigilance muss selbst bewiesen werden
            omega: 0.1 + voucher_trust.omega * stake * 0.3,
        }
    }

    /// Erstelle neuen Trust-Vektor mit gegebenen Werten
    pub fn new(r: f64, i: f64, c: f64, p: f64, v: f64, omega: f64) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            i: i.clamp(0.0, 1.0),
            c: c.clamp(0.0, 1.0),
            p: p.clamp(0.0, 1.0),
            v: v.clamp(0.0, 1.0),
            omega: omega.clamp(0.0, 1.0),
        }
    }

    /// Null-Vektor (für neue, unbekannte Entitäten)
    pub fn zero() -> Self {
        Self {
            r: 0.0,
            i: 0.0,
            c: 0.0,
            p: 0.0,
            v: 0.0,
            omega: 0.0,
        }
    }

    /// Maximaler Trust-Vektor (perfekt)
    pub fn max() -> Self {
        Self {
            r: 1.0,
            i: 1.0,
            c: 1.0,
            p: 1.0,
            v: 1.0,
            omega: 1.0,
        }
    }

    /// Default-Gewichte für Kontext "Default"
    pub fn default_weights() -> [f64; 6] {
        [0.17, 0.17, 0.17, 0.17, 0.16, 0.16]
    }

    /// Als Array
    pub fn to_array(&self) -> [f64; 6] {
        [self.r, self.i, self.c, self.p, self.v, self.omega]
    }

    /// Von Array erstellen
    pub fn from_array(arr: [f64; 6]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3], arr[4], arr[5])
    }

    /// Dimension nach Index
    pub fn get(&self, dim: TrustDimension) -> f64 {
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
    pub fn set(&mut self, dim: TrustDimension, value: f64) {
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

    /// Κ4: Asymmetrische Aktualisierung
    ///
    /// Bei positiven Events: `new = old + delta`
    /// Bei negativen Events: `new = old - λ × delta` (stärkere Strafe)
    pub fn update(&mut self, dim: TrustDimension, delta: f64, is_positive: bool) {
        let current = self.get(dim);
        let effective_delta = if is_positive {
            delta
        } else {
            delta * dim.asymmetry_factor()
        };

        let new_value = if is_positive {
            current + effective_delta
        } else {
            current - effective_delta
        };

        self.set(dim, new_value);
    }

    /// Euklidische Norm ‖𝕎‖
    pub fn norm(&self) -> f64 {
        let sum: f64 = self.to_array().iter().map(|x| x * x).sum();
        sum.sqrt()
    }

    /// Minimale Komponente des Trust-Vektors
    pub fn min_component(&self) -> f64 {
        self.to_array()
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min)
    }

    /// Maximale Komponente des Trust-Vektors
    pub fn max_component(&self) -> f64 {
        self.to_array()
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Κ15b: Gewichtete Norm ‖𝕎‖_w
    ///
    /// `‖𝕎‖_w = √(Σᵢ wᵢ · 𝕎ᵢ²) / √(Σᵢ wᵢ)`
    pub fn weighted_norm(&self, weights: &[f64; 6]) -> f64 {
        let values = self.to_array();
        let sum: f64 = values
            .iter()
            .zip(weights.iter())
            .map(|(v, w)| w * v * v)
            .sum();
        let weight_sum: f64 = weights.iter().sum();

        (sum / weight_sum).sqrt()
    }

    /// Multipliziert den Vektor mit einem Skalar (z.B. bei Delegation)
    pub fn scale(&self, factor: f64) -> Self {
        Self::new(
            self.r * factor,
            self.i * factor,
            self.c * factor,
            self.p * factor,
            self.v * factor,
            self.omega * factor,
        )
    }
}

/// Kontexttypen für Trust-Gewichtung
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextType {
    /// Standard/Default-Kontext
    Default,
    /// Finanztransaktionen
    Finance,
    /// Wissensaustausch
    Knowledge,
    /// Governance-Abstimmungen
    Governance,
    /// Energie-Handel
    Energy,
}

impl ContextType {
    /// Gewichte für diesen Kontext
    pub fn weights(&self) -> [f64; 6] {
        match self {
            ContextType::Default => [0.17, 0.17, 0.17, 0.17, 0.16, 0.16],
            ContextType::Finance => [0.30, 0.25, 0.15, 0.10, 0.15, 0.05],
            ContextType::Knowledge => [0.10, 0.30, 0.30, 0.15, 0.10, 0.05],
            ContextType::Governance => [0.15, 0.20, 0.15, 0.20, 0.10, 0.20],
            ContextType::Energy => [0.25, 0.20, 0.25, 0.15, 0.10, 0.05],
        }
    }
}

/// Κ5: Probabilistische Trust-Kombination
///
/// `t₁ ⊕ t₂ = 1 - (1-t₁)(1-t₂)`
///
/// Entspricht logischem "unabhängige Bestätigung ODER"
#[derive(Debug, Clone, Copy)]
pub struct TrustCombination;

impl TrustCombination {
    /// Κ5: Kombiniere zwei Trust-Werte
    pub fn combine(t1: f64, t2: f64) -> f64 {
        1.0 - (1.0 - t1) * (1.0 - t2)
    }

    /// Kombiniere mehrere Trust-Werte
    pub fn combine_all(values: &[f64]) -> f64 {
        values.iter().fold(0.0, |acc, &t| Self::combine(acc, t))
    }

    /// Τ1: Ketten-Trust mit √n Dämpfung
    ///
    /// `t_chain = exp(Σᵢ ln(tᵢ) / √n)`
    pub fn chain_trust(chain: &[f64]) -> f64 {
        if chain.is_empty() {
            return 0.0;
        }

        let n = chain.len() as f64;
        let log_sum: f64 = chain.iter().map(|t| t.max(1e-10).ln()).sum();

        (log_sum / n.sqrt()).exp()
    }
}

/// 6x6 Matrix für Trust-Dämpfung bei Realm-Crossings (Κ24)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustDampeningMatrix {
    /// 6x6 Matrix-Einträge
    pub data: [[f64; 6]; 6],
}

impl Default for TrustDampeningMatrix {
    /// Identitätsmatrix (keine Dämpfung)
    fn default() -> Self {
        Self {
            data: [
                [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

impl TrustDampeningMatrix {
    /// Erstelle Identitätsmatrix
    pub fn identity() -> Self {
        Self::default()
    }

    /// Erstelle Matrix für generischen Realm-Crossing
    /// Competence und Prestige werden stärker gedämpft
    pub fn generic_crossing(factor: f64) -> Self {
        Self {
            data: [
                [factor * 0.8, 0.0, 0.0, 0.0, 0.0, 0.0], // R: 80% erhalten
                [0.0, factor * 0.9, 0.0, 0.0, 0.0, 0.0], // I: 90% erhalten
                [0.0, 0.0, factor * 0.4, 0.0, 0.0, 0.0], // C: 40% erhalten
                [0.0, 0.0, 0.0, factor * 0.3, 0.0, 0.0], // P: 30% erhalten
                [0.0, 0.0, 0.0, 0.0, 1.0, 0.0],          // V: 100% (universal)
                [0.0, 0.0, 0.0, 0.0, 0.0, 1.0],          // Ω: 100% (universal)
            ],
        }
    }

    /// Κ24: 𝕎_target = M × 𝕎_source
    pub fn apply(&self, trust: &TrustVector6D) -> TrustVector6D {
        let source = trust.to_array();
        let mut result = [0.0; 6];

        for i in 0..6 {
            for j in 0..6 {
                result[i] += self.data[i][j] * source[j];
            }
        }

        TrustVector6D::from_array(result)
    }

    /// Prüft ob ‖M‖ ≤ 1 (Trust kann nicht steigen)
    pub fn is_valid(&self) -> bool {
        // Spektralnorm ≤ 1
        // Vereinfachte Prüfung: Alle Diagonalelemente ≤ 1
        self.data.iter().enumerate().all(|(i, row)| row[i] <= 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_vector_default() {
        let trust = TrustVector6D::default();
        assert_eq!(trust.r, 0.5);
        assert_eq!(trust.omega, 0.5);
    }

    #[test]
    fn test_trust_combination() {
        // Κ5: t₁ ⊕ t₂ = 1 - (1-t₁)(1-t₂)
        let combined = TrustCombination::combine(0.5, 0.5);
        assert!((combined - 0.75).abs() < 0.001);

        // Neutral element: t ⊕ 0 = t
        let neutral = TrustCombination::combine(0.7, 0.0);
        assert!((neutral - 0.7).abs() < 0.001);

        // Absorbing: t ⊕ 1 = 1
        let absorbing = TrustCombination::combine(0.3, 1.0);
        assert!((absorbing - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_chain_trust() {
        // Τ1: Ketten-Trust
        let chain = vec![0.8, 0.8, 0.8];
        let result = TrustCombination::chain_trust(&chain);

        // exp(3 × ln(0.8) / √3) = exp(-0.669 / 1.732) = exp(-0.386) ≈ 0.68
        // Die √n Dämpfung im Exponenten macht es besser als reine Multiplikation (0.8³=0.512)
        let simple_product = 0.8_f64.powi(3); // 0.512
        assert!(
            result > simple_product,
            "Chain trust {} should be > simple product {}",
            result,
            simple_product
        );
        assert!(result < 1.0);
    }

    #[test]
    fn test_weighted_norm() {
        let trust = TrustVector6D::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
        let weights = TrustVector6D::default_weights();
        let norm = trust.weighted_norm(&weights);
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_asymmetric_update() {
        let mut trust = TrustVector6D::default();

        // Positive update
        trust.update(TrustDimension::Reliability, 0.1, true);
        assert!((trust.r - 0.6).abs() < 0.001);

        // Negative update (stärker: × 1.5)
        trust.update(TrustDimension::Reliability, 0.1, false);
        assert!((trust.r - 0.45).abs() < 0.001); // 0.6 - 0.15 = 0.45
    }

    #[test]
    fn test_dampening_matrix() {
        let matrix = TrustDampeningMatrix::generic_crossing(1.0);
        let trust = TrustVector6D::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
        let dampened = matrix.apply(&trust);

        // V und Ω sollten unverändert sein
        assert!((dampened.v - 1.0).abs() < 0.001);
        assert!((dampened.omega - 1.0).abs() < 0.001);

        // C sollte auf 0.4 gedämpft sein
        assert!((dampened.c - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_newcomer_low_trust() {
        let newcomer = TrustVector6D::newcomer();

        // Newcomer startet mit 0.1 auf allen Dimensionen
        assert!((newcomer.r - 0.1).abs() < 0.001);
        assert!((newcomer.omega - 0.1).abs() < 0.001);

        // weighted_norm sollte ~0.1 sein
        let norm = newcomer.weighted_norm(&TrustVector6D::default_weights());
        assert!((norm - 0.1).abs() < 0.001);

        // Dampened influence (trust²) ist nur 1% eines etablierten Nutzers
        let established = TrustVector6D::default();
        let newcomer_influence = newcomer
            .weighted_norm(&TrustVector6D::default_weights())
            .powi(2);
        let established_influence = established
            .weighted_norm(&TrustVector6D::default_weights())
            .powi(2);

        assert!(
            newcomer_influence < established_influence * 0.05,
            "Newcomer influence {} should be < 5% of established {}",
            newcomer_influence,
            established_influence
        );
    }

    #[test]
    fn test_vouched_trust_transfer() {
        let voucher = TrustVector6D::new(0.8, 0.8, 0.8, 0.8, 0.8, 0.8);
        let vouched = TrustVector6D::vouched(&voucher, 0.2); // 20% stake

        // Prestige sollte am meisten profitieren
        assert!(
            vouched.p > vouched.c,
            "Prestige {} should be > Competence {}",
            vouched.p,
            vouched.c
        );

        // Competence und Vigilance bleiben bei 0.1 (müssen selbst bewiesen werden)
        assert!((vouched.c - 0.1).abs() < 0.001);
        assert!((vouched.v - 0.1).abs() < 0.001);

        // Vouched ist besser als reiner Newcomer
        let newcomer = TrustVector6D::newcomer();
        assert!(
            vouched.weighted_norm(&TrustVector6D::default_weights())
                > newcomer.weighted_norm(&TrustVector6D::default_weights())
        );

        // Aber immer noch deutlich unter Voucher
        assert!(
            vouched.weighted_norm(&TrustVector6D::default_weights())
                < voucher.weighted_norm(&TrustVector6D::default_weights()) * 0.5
        );
    }
}
