//! # Unified Data Model – Konfigurierbare Parameter
//!
//! Zentrale Konfiguration für alle Weltformel-Parameter (Priorität 2).
//!
//! ## Parameter-Übersicht (Optimiert via Small-World Simulation)
//!
//! ### Trust-Parameter (Κ3-Κ5)
//!
//! | Parameter | Default | Bereich | Beschreibung | Axiom |
//! |-----------|---------|---------|--------------|-------|
//! | positive_delta | 0.045 | 0.04-0.05 | Basis-Lernrate für positive Updates | Κ4 |
//! | negative_multiplier | 2.2 | 2.0-2.5 | Negativity-Bias Faktor | Κ4 |
//! | hop_damping_factor | 0.85 | 0.82-0.88 | Chain-Trust Dämpfung (Τ1) | Κ8 |
//! | newcomer_trust | 0.1 | 0.05-0.15 | Newcomer Trust-Wert (Sybil) | Κ3 |
//! | default_trust | 0.5 | - | Default Trust-Wert | Κ3 |
//!
//! ### Protection-Parameter (Κ19 Anti-Calcification)
//!
//! | Parameter | Default | Bereich | Beschreibung |
//! |-----------|---------|---------|-------------|
//! | gini_threshold | 0.35 | 0.32-0.38 | Trigger für Power-Cap/Decay |
//! | decay_rate | 0.0008 | 0.0005-0.0012 | Anti-Calcification Decay |
//! | power_cap_exponent | 0.5 | - | sqrt-Scaling für Quadratic Voting |
//!
//! ### Realm-Parameter (Κ23-Κ24)
//!
//! | Parameter | Default | Bereich | Beschreibung |
//! |-----------|---------|---------|-------------|
//! | realm_crossing_penalty | 0.85 | 0.7-0.95 | Cross-Realm Trust Multiplier |
//!
//! ## Verwendung
//!
//! ```rust
//! use erynoa_api::domain::unified::WorldFormulaConfig;
//!
//! // Standard-Konfiguration
//! let config = WorldFormulaConfig::default();
//!
//! // Angepasste Konfiguration
//! let custom = WorldFormulaConfig::builder()
//!     .asymmetry_base(1.8)
//!     .activity_tau_days(60)
//!     .build();
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// WorldFormulaConfig – Hauptkonfiguration
// ============================================================================

/// Zentrale Konfiguration für Weltformel-Parameter
///
/// Alle Parameter mit Defaults gemäß IPS-01 §4 und
/// optimiert via Small-World Simulation (20% Malicious, Collusion+Badmouthing).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFormulaConfig {
    /// Trust-Parameter (Κ3-Κ5, Κ8)
    pub trust: TrustConfig,
    /// Aktivitäts-Parameter (Κ15b)
    pub activity: ActivityConfig,
    /// Temporale Parameter (Κ15b)
    pub temporal: TemporalConfig,
    /// Human-Factor Parameter (Κ15b)
    pub human_factor: HumanFactorConfig,
    /// Anti-Calcification Parameter (Κ19)
    pub protection: ProtectionConfig,
    /// Realm-Crossing Parameter (Κ23-Κ24)
    pub realm: RealmConfig,
}

impl WorldFormulaConfig {
    /// Builder starten
    pub fn builder() -> WorldFormulaConfigBuilder {
        WorldFormulaConfigBuilder::default()
    }

    /// Validiere Konfiguration
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        self.trust.validate()?;
        self.activity.validate()?;
        self.temporal.validate()?;
        self.human_factor.validate()?;
        self.protection.validate()?;
        self.realm.validate()?;
        Ok(())
    }
}

impl Default for WorldFormulaConfig {
    fn default() -> Self {
        Self {
            trust: TrustConfig::default(),
            activity: ActivityConfig::default(),
            temporal: TemporalConfig::default(),
            human_factor: HumanFactorConfig::default(),
            protection: ProtectionConfig::default(),
            realm: RealmConfig::default(),
        }
    }
}

// ============================================================================
// TrustConfig – Trust-Parameter (Κ2-Κ5)
// ============================================================================

/// Trust-Konfiguration gemäß Κ2-Κ5
///
/// Optimale Werte basieren auf Small-World Simulation mit 20% Angreifern:
/// - diff_mal_hon ≈ 0.004 (Angreifer nur +0.4% Vorteil)
/// - Gini ≈ 0.004 (nahezu perfekt egalitär)
/// - avg_trust ≈ 0.506 (stabil über Equilibrium)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustConfig {
    // ========================================================================
    // Learning Rate Parameter (Κ4)
    // ========================================================================
    /// Basis-Lernrate für positive Trust-Updates
    /// Κ4: Langsamer Aufbau verhindert Sybil/Collusion-Manipulation
    /// Optimal: 0.045 (Bereich: 0.04-0.05)
    /// - Zu niedrig (<0.04): Träge Konvergenz zu guten Equilibria
    /// - Zu hoch (>0.05): Angreifer profitieren stärker von Collusion
    pub positive_delta: f32,

    /// Negativity-Bias Multiplikator
    /// Κ4: negative_delta = positive_delta × negative_multiplier
    /// Optimal: 2.2 (Bereich: 2.0-2.5)
    /// - Minimiert diff_mal_hon ohne Over-Punishment
    /// - Badmouthing wird stark gedämpft
    pub negative_multiplier: f32,

    /// Asymmetrie-Faktor für Basis-Dimensionen (R,I,C,P)
    /// Κ4: Wird intern aus negative_multiplier abgeleitet
    /// Legacy-Feld für Kompatibilität
    pub asymmetry_base: f32,

    /// Asymmetrie-Faktor für kritische Dimensionen (V,Ω)
    /// Κ4: Höherer Faktor für sicherheitskritische Dimensionen
    pub asymmetry_critical: f32,

    // ========================================================================
    // Chain Trust Parameter (Κ8 / Τ1)
    // ========================================================================
    /// Hop-Damping-Faktor für Chain-Trust (Τ1)
    /// Κ8: trust_chain = trust_direct × (hop_damping ^ hops)
    /// Optimal: 0.85 (Bereich: 0.82-0.88)
    /// - Verhindert Propagation von Manipulation über lange Chains
    /// - Zu niedrig (<0.82): Indirekter Trust sickert zu weit (Sybil-Anfällig)
    /// - Zu hoch (>0.88): Nutzt kaum indirekte Info → langsame globale Konvergenz
    pub hop_damping_factor: f32,

    // ========================================================================
    // Trust Bounds (Κ3)
    // ========================================================================
    /// Newcomer Trust-Wert (Sybil-Schutz)
    /// Κ3: Neue Entitäten starten mit niedrigem Trust
    pub newcomer_trust: f32,

    /// Default Trust-Wert für etablierte Entitäten
    /// Κ3: Etablierte Entitäten haben mittleren Trust
    pub default_trust: f32,

    /// Minimaler Trust-Wert (untere Schranke)
    pub min_trust: f32,

    /// Maximaler Trust-Wert (obere Schranke)
    pub max_trust: f32,

    /// Gewichte pro Dimension [R, I, C, P, V, Ω]
    pub dimension_weights: [f32; 6],
}

impl TrustConfig {
    /// Validiere Trust-Konfiguration
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        // Learning Rate Validierung
        if self.positive_delta <= 0.0 || self.positive_delta > 0.1 {
            return Err(ConfigValidationError::InvalidLearningRate {
                name: "positive_delta",
                value: self.positive_delta,
                reason: "must be in (0.0, 0.1] for stable convergence",
            });
        }

        if self.negative_multiplier < 1.0 || self.negative_multiplier > 5.0 {
            return Err(ConfigValidationError::InvalidLearningRate {
                name: "negative_multiplier",
                value: self.negative_multiplier,
                reason: "must be in [1.0, 5.0] for balanced negativity bias",
            });
        }

        // Hop Damping Validierung
        if self.hop_damping_factor <= 0.0 || self.hop_damping_factor >= 1.0 {
            return Err(ConfigValidationError::InvalidDampingFactor {
                value: self.hop_damping_factor,
                reason: "must be in (0.0, 1.0) for proper chain decay",
            });
        }

        // Legacy Asymmetrie (für Kompatibilität)
        if self.asymmetry_base <= 1.0 {
            return Err(ConfigValidationError::InvalidAsymmetry {
                name: "asymmetry_base",
                value: self.asymmetry_base,
                reason: "must be > 1.0 for negative update amplification",
            });
        }

        if self.asymmetry_critical < self.asymmetry_base {
            return Err(ConfigValidationError::InvalidAsymmetry {
                name: "asymmetry_critical",
                value: self.asymmetry_critical,
                reason: "must be >= asymmetry_base",
            });
        }

        // Trust Bounds
        if self.newcomer_trust < 0.0 || self.newcomer_trust > 0.5 {
            return Err(ConfigValidationError::InvalidTrustBound {
                name: "newcomer_trust",
                value: self.newcomer_trust,
                min: 0.0,
                max: 0.5,
            });
        }

        if self.default_trust < self.newcomer_trust || self.default_trust > 1.0 {
            return Err(ConfigValidationError::InvalidTrustBound {
                name: "default_trust",
                value: self.default_trust,
                min: self.newcomer_trust,
                max: 1.0,
            });
        }

        // Gewichte müssen sich zu 1.0 summieren (mit Toleranz)
        let weight_sum: f32 = self.dimension_weights.iter().sum();
        if (weight_sum - 1.0).abs() > 0.01 {
            return Err(ConfigValidationError::InvalidWeights {
                sum: weight_sum,
                expected: 1.0,
            });
        }

        Ok(())
    }

    /// Effektive negative Lernrate (Κ4)
    /// negative_delta = positive_delta × negative_multiplier
    #[inline]
    pub fn negative_delta(&self) -> f32 {
        self.positive_delta * self.negative_multiplier
    }

    /// Chain-Trust für n Hops berechnen (Κ8)
    /// trust_chain = trust_direct × (hop_damping ^ hops)
    #[inline]
    pub fn chain_trust_factor(&self, hops: u32) -> f32 {
        self.hop_damping_factor.powi(hops as i32)
    }

    /// Asymmetrie-Faktor für eine Dimension (Κ4)
    pub fn asymmetry_for(&self, dim: super::trust::TrustDimension) -> f32 {
        use super::trust::TrustDimension;
        match dim {
            TrustDimension::Vigilance | TrustDimension::Omega => self.asymmetry_critical,
            _ => self.asymmetry_base,
        }
    }
}

impl Default for TrustConfig {
    fn default() -> Self {
        Self {
            // Optimale Werte aus Small-World Simulation (20% Malicious, Collusion+Badmouthing)
            // Resultat: diff_mal_hon ≈ 0.004, Gini ≈ 0.004, avg_trust ≈ 0.506
            positive_delta: 0.045,    // Optimal: 0.045 (Bereich: 0.04-0.05)
            negative_multiplier: 2.2, // Optimal: 2.2 (Bereich: 2.0-2.5)
            hop_damping_factor: 0.85, // Optimal: 0.85 (Bereich: 0.82-0.88)
            // Legacy-Felder (aus negative_multiplier abgeleitet)
            asymmetry_base: 2.2,      // = negative_multiplier
            asymmetry_critical: 2.75, // = negative_multiplier × 1.25 für V,Ω
            // Trust Bounds
            newcomer_trust: 0.1,
            default_trust: 0.5,
            min_trust: 0.0,
            max_trust: 1.0,
            dimension_weights: [0.167, 0.167, 0.167, 0.167, 0.166, 0.166],
        }
    }
}

// ============================================================================
// ActivityConfig – Aktivitäts-Parameter (Κ15b)
// ============================================================================

/// Aktivitäts-Konfiguration gemäß Κ15b
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityConfig {
    /// Zeitfenster τ für Aktivitätsberechnung in Tagen
    /// Default: 90 Tage
    pub tau_days: u32,

    /// Aktivitäts-Schwelle κ
    /// 𝔸(s) = n / (n + κ)
    /// Default: 10
    pub kappa: u32,

    /// Mobiles Zeitfenster (kürzerer Zyklus)
    /// Default: 30 Tage
    pub tau_days_mobile: u32,
}

impl ActivityConfig {
    /// τ in Sekunden
    pub fn tau_seconds(&self) -> u64 {
        self.tau_days as u64 * 24 * 3600
    }

    /// τ_mobile in Sekunden
    pub fn tau_seconds_mobile(&self) -> u64 {
        self.tau_days_mobile as u64 * 24 * 3600
    }

    /// Validiere Aktivitäts-Konfiguration
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if self.tau_days == 0 {
            return Err(ConfigValidationError::InvalidTau {
                value: self.tau_days,
                reason: "tau_days must be > 0",
            });
        }

        if self.kappa == 0 {
            return Err(ConfigValidationError::InvalidKappa {
                value: self.kappa,
                reason: "kappa must be > 0 to avoid division instability",
            });
        }

        if self.tau_days_mobile > self.tau_days {
            return Err(ConfigValidationError::InvalidTau {
                value: self.tau_days_mobile,
                reason: "tau_days_mobile should be <= tau_days",
            });
        }

        Ok(())
    }
}

impl Default for ActivityConfig {
    fn default() -> Self {
        Self {
            tau_days: 90,
            kappa: 10,
            tau_days_mobile: 30,
        }
    }
}

// ============================================================================
// TemporalConfig – Temporale Parameter (Κ15b)
// ============================================================================

/// Temporale Gewichtungs-Konfiguration gemäß Κ15b
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConfig {
    /// Decay-Faktor λ pro Tag
    /// w(s,t) = 1 / (1 + λ · Δt)
    /// Default: 0.01
    pub lambda_per_day: f64,

    /// Schneller Decay für volatile Inhalte
    /// Default: 0.1
    pub lambda_fast_per_day: f64,

    /// Maximales Alter in Tagen (danach w=0)
    /// Default: 365 Tage
    pub max_age_days: u32,

    /// Maximales Alter für schnellen Decay
    /// Default: 90 Tage
    pub max_age_fast_days: u32,
}

impl TemporalConfig {
    /// λ pro Sekunde
    pub fn lambda_per_second(&self) -> f64 {
        self.lambda_per_day / (24.0 * 3600.0)
    }

    /// λ_fast pro Sekunde
    pub fn lambda_fast_per_second(&self) -> f64 {
        self.lambda_fast_per_day / (24.0 * 3600.0)
    }

    /// Max-Age in Sekunden
    pub fn max_age_seconds(&self) -> u64 {
        self.max_age_days as u64 * 24 * 3600
    }

    /// Max-Age-Fast in Sekunden
    pub fn max_age_fast_seconds(&self) -> u64 {
        self.max_age_fast_days as u64 * 24 * 3600
    }

    /// Validiere temporale Konfiguration
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if self.lambda_per_day <= 0.0 || self.lambda_per_day > 1.0 {
            return Err(ConfigValidationError::InvalidLambda {
                value: self.lambda_per_day,
                reason: "lambda must be in (0, 1]",
            });
        }

        if self.lambda_fast_per_day < self.lambda_per_day {
            return Err(ConfigValidationError::InvalidLambda {
                value: self.lambda_fast_per_day,
                reason: "lambda_fast must be >= lambda",
            });
        }

        if self.max_age_days == 0 {
            return Err(ConfigValidationError::InvalidMaxAge {
                value: self.max_age_days,
                reason: "max_age_days must be > 0",
            });
        }

        Ok(())
    }
}

impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            lambda_per_day: 0.01,
            lambda_fast_per_day: 0.1,
            max_age_days: 365,
            max_age_fast_days: 90,
        }
    }
}

// ============================================================================
// HumanFactorConfig – Human-Factor Parameter (Κ15b)
// ============================================================================

/// Human-Factor Konfiguration gemäß Κ15b
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanFactorConfig {
    /// Faktor für nicht verifizierte Entitäten
    /// Default: 1.0
    pub not_verified: f64,

    /// Faktor für Basis-Attestation
    /// Default: 1.2
    pub basic_attestation: f64,

    /// Faktor für volle Attestation
    /// Default: 1.5
    pub full_attestation: f64,
}

impl HumanFactorConfig {
    /// Validiere Human-Factor Konfiguration
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if self.not_verified != 1.0 {
            return Err(ConfigValidationError::InvalidHumanFactor {
                name: "not_verified",
                value: self.not_verified,
                reason: "baseline must be 1.0",
            });
        }

        if self.basic_attestation <= self.not_verified {
            return Err(ConfigValidationError::InvalidHumanFactor {
                name: "basic_attestation",
                value: self.basic_attestation,
                reason: "must be > not_verified",
            });
        }

        if self.full_attestation <= self.basic_attestation {
            return Err(ConfigValidationError::InvalidHumanFactor {
                name: "full_attestation",
                value: self.full_attestation,
                reason: "must be > basic_attestation",
            });
        }

        Ok(())
    }
}

impl Default for HumanFactorConfig {
    fn default() -> Self {
        Self {
            not_verified: 1.0,
            basic_attestation: 1.2,
            full_attestation: 1.5,
        }
    }
}

// ============================================================================
// ProtectionConfig – Anti-Calcification Parameter (Κ19)
// ============================================================================

/// Anti-Calcification Konfiguration gemäß Κ19
///
/// Verhindert übermäßige Machtkonzentration und Trust-Ossifikation.
/// Optimale Werte aus Small-World Simulation validiert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionConfig {
    /// Gini-Schwellwert für Trigger von Power-Cap/Decay
    /// Κ19: Startet Korrekturmaßnahmen bei starker Ungleichheit
    /// Optimal: 0.35 (Bereich: 0.32-0.38)
    /// - In sparse Nets selten getriggert (Gini normal <0.01)
    /// - Bei dichten Nets/Whales aktiviert Protection
    /// - 0.35 entspricht realen sozialen Systemen (Lorenz-Kurve)
    pub gini_threshold: f64,

    /// Anti-Calcification Decay-Rate pro Step
    /// Κ19: Langsamer Decay löst Ossifikation auf
    /// Optimal: 0.0008 (Bereich: 0.0005-0.0012)
    /// - Sehr langsam verhindert Oscillation
    /// - Erhält legitime Realm-Cluster
    pub decay_rate: f64,

    /// Power-Cap Exponent (Quadratic Voting Inspiration)
    /// effective_power = total_power^exponent / count
    /// Optimal: 0.5 (sqrt-Scaling)
    /// - Linear (1.0) zu schwach gegen Sybil
    /// - Kubisch (0.33) zu hart gegen legitime Gruppen
    pub power_cap_exponent: f64,

    /// Minimal-Trust nach Decay (untere Schranke)
    /// Verhindert vollständigen Trust-Verlust
    pub min_trust_after_decay: f32,
}

impl ProtectionConfig {
    /// Validiere Protection-Konfiguration
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if self.gini_threshold <= 0.0 || self.gini_threshold >= 1.0 {
            return Err(ConfigValidationError::InvalidProtection {
                name: "gini_threshold",
                value: self.gini_threshold,
                reason: "must be in (0.0, 1.0)",
            });
        }

        if self.decay_rate <= 0.0 || self.decay_rate > 0.1 {
            return Err(ConfigValidationError::InvalidProtection {
                name: "decay_rate",
                value: self.decay_rate,
                reason: "must be in (0.0, 0.1] for stable decay",
            });
        }

        if self.power_cap_exponent <= 0.0 || self.power_cap_exponent > 1.0 {
            return Err(ConfigValidationError::InvalidProtection {
                name: "power_cap_exponent",
                value: self.power_cap_exponent,
                reason: "must be in (0.0, 1.0]",
            });
        }

        Ok(())
    }

    /// Berechne effektive Power mit Cap
    /// effective_power = total_power^exponent / sqrt(count)
    #[inline]
    pub fn capped_power(&self, total_power: f64, count: u32) -> f64 {
        if count == 0 {
            return 0.0;
        }
        total_power.powf(self.power_cap_exponent) / (count as f64).sqrt()
    }
}

impl Default for ProtectionConfig {
    fn default() -> Self {
        Self {
            // Optimale Werte aus Simulation
            gini_threshold: 0.35,    // Optimal: 0.35 (Bereich: 0.32-0.38)
            decay_rate: 0.0008,      // Optimal: 0.0008 (Bereich: 0.0005-0.0012)
            power_cap_exponent: 0.5, // sqrt-Scaling (Quadratic Voting)
            min_trust_after_decay: 0.05,
        }
    }
}

// ============================================================================
// RealmConfig – Realm-Crossing Parameter (Κ23-Κ24)
// ============================================================================

/// Realm-Konfiguration gemäß Κ23-Κ24
///
/// Steuert Cross-Realm-Interaktionen und Penalties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmConfig {
    /// Cross-Realm Trust Multiplikator (Κ24)
    /// cross_trust = local_trust × realm_crossing_penalty
    /// Optimal: 0.85 (Bereich: 0.7-0.95)
    /// - Erlaubt Kooperation zwischen Gruppen
    /// - Bestraft blinden Cross-Trust
    /// - Fördert diverse Realms ohne Isolation
    pub realm_crossing_penalty: f32,

    /// Maximale Realm-Tiefe (Κ1)
    /// Verhindert zu tiefe Hierarchien
    pub max_realm_depth: u32,

    /// Minimale Governance-Quorum (Κ22)
    /// Mindestanteil für Realm-Entscheidungen
    pub min_governance_quorum: f32,
}

impl RealmConfig {
    /// Validiere Realm-Konfiguration
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if self.realm_crossing_penalty <= 0.0 || self.realm_crossing_penalty > 1.0 {
            return Err(ConfigValidationError::InvalidRealm {
                name: "realm_crossing_penalty",
                value: self.realm_crossing_penalty,
                reason: "must be in (0.0, 1.0]",
            });
        }

        if self.max_realm_depth == 0 || self.max_realm_depth > 100 {
            return Err(ConfigValidationError::InvalidRealm {
                name: "max_realm_depth",
                value: self.max_realm_depth as f32,
                reason: "must be in [1, 100]",
            });
        }

        if self.min_governance_quorum <= 0.0 || self.min_governance_quorum > 1.0 {
            return Err(ConfigValidationError::InvalidRealm {
                name: "min_governance_quorum",
                value: self.min_governance_quorum,
                reason: "must be in (0.0, 1.0]",
            });
        }

        Ok(())
    }

    /// Berechne effektiven Cross-Realm Trust
    #[inline]
    pub fn cross_realm_trust(&self, local_trust: f32) -> f32 {
        local_trust * self.realm_crossing_penalty
    }
}

impl Default for RealmConfig {
    fn default() -> Self {
        Self {
            // Optimale Werte aus Simulation
            realm_crossing_penalty: 0.85, // Optimal: 0.85 (Bereich: 0.7-0.95)
            max_realm_depth: 10,
            min_governance_quorum: 0.51, // Einfache Mehrheit
        }
    }
}

// ============================================================================
// ConfigValidationError
// ============================================================================

/// Fehler bei der Konfigurationsvalidierung
#[derive(Debug, Clone)]
pub enum ConfigValidationError {
    InvalidAsymmetry {
        name: &'static str,
        value: f32,
        reason: &'static str,
    },
    InvalidLearningRate {
        name: &'static str,
        value: f32,
        reason: &'static str,
    },
    InvalidDampingFactor {
        value: f32,
        reason: &'static str,
    },
    InvalidTrustBound {
        name: &'static str,
        value: f32,
        min: f32,
        max: f32,
    },
    InvalidWeights {
        sum: f32,
        expected: f32,
    },
    InvalidTau {
        value: u32,
        reason: &'static str,
    },
    InvalidKappa {
        value: u32,
        reason: &'static str,
    },
    InvalidLambda {
        value: f64,
        reason: &'static str,
    },
    InvalidMaxAge {
        value: u32,
        reason: &'static str,
    },
    InvalidHumanFactor {
        name: &'static str,
        value: f64,
        reason: &'static str,
    },
    InvalidProtection {
        name: &'static str,
        value: f64,
        reason: &'static str,
    },
    InvalidRealm {
        name: &'static str,
        value: f32,
        reason: &'static str,
    },
}

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAsymmetry {
                name,
                value,
                reason,
            } => {
                write!(f, "Invalid {}: {} - {}", name, value, reason)
            }
            Self::InvalidLearningRate {
                name,
                value,
                reason,
            } => {
                write!(f, "Invalid learning rate {}: {} - {}", name, value, reason)
            }
            Self::InvalidDampingFactor { value, reason } => {
                write!(f, "Invalid damping factor: {} - {}", value, reason)
            }
            Self::InvalidTrustBound {
                name,
                value,
                min,
                max,
            } => {
                write!(f, "Invalid {}: {} not in [{}, {}]", name, value, min, max)
            }
            Self::InvalidWeights { sum, expected } => {
                write!(f, "Weights sum to {} but expected {}", sum, expected)
            }
            Self::InvalidTau { value, reason } => {
                write!(f, "Invalid tau: {} - {}", value, reason)
            }
            Self::InvalidKappa { value, reason } => {
                write!(f, "Invalid kappa: {} - {}", value, reason)
            }
            Self::InvalidLambda { value, reason } => {
                write!(f, "Invalid lambda: {} - {}", value, reason)
            }
            Self::InvalidMaxAge { value, reason } => {
                write!(f, "Invalid max_age: {} - {}", value, reason)
            }
            Self::InvalidHumanFactor {
                name,
                value,
                reason,
            } => {
                write!(f, "Invalid human factor {}: {} - {}", name, value, reason)
            }
            Self::InvalidProtection {
                name,
                value,
                reason,
            } => {
                write!(
                    f,
                    "Invalid protection param {}: {} - {}",
                    name, value, reason
                )
            }
            Self::InvalidRealm {
                name,
                value,
                reason,
            } => {
                write!(f, "Invalid realm param {}: {} - {}", name, value, reason)
            }
        }
    }
}

impl std::error::Error for ConfigValidationError {}

// ============================================================================
// Builder Pattern
// ============================================================================

/// Builder für WorldFormulaConfig
#[derive(Debug, Default)]
pub struct WorldFormulaConfigBuilder {
    trust: Option<TrustConfig>,
    activity: Option<ActivityConfig>,
    temporal: Option<TemporalConfig>,
    human_factor: Option<HumanFactorConfig>,
    protection: Option<ProtectionConfig>,
    realm: Option<RealmConfig>,
}

impl WorldFormulaConfigBuilder {
    /// Trust-Konfiguration setzen
    pub fn trust(mut self, trust: TrustConfig) -> Self {
        self.trust = Some(trust);
        self
    }

    // ========================================================================
    // Learning Rate Builder Methods (neue optimale Parameter)
    // ========================================================================

    /// Positive Lernrate setzen (Optimal: 0.045)
    pub fn positive_delta(mut self, value: f32) -> Self {
        let mut trust = self.trust.unwrap_or_default();
        trust.positive_delta = value;
        self.trust = Some(trust);
        self
    }

    /// Negative Multiplier setzen (Optimal: 2.2)
    pub fn negative_multiplier(mut self, value: f32) -> Self {
        let mut trust = self.trust.unwrap_or_default();
        trust.negative_multiplier = value;
        self.trust = Some(trust);
        self
    }

    /// Hop-Damping-Faktor setzen (Optimal: 0.85)
    pub fn hop_damping_factor(mut self, value: f32) -> Self {
        let mut trust = self.trust.unwrap_or_default();
        trust.hop_damping_factor = value;
        self.trust = Some(trust);
        self
    }

    // ========================================================================
    // Legacy Asymmetrie Builder Methods
    // ========================================================================

    /// Asymmetrie-Basis setzen
    pub fn asymmetry_base(mut self, value: f32) -> Self {
        let mut trust = self.trust.unwrap_or_default();
        trust.asymmetry_base = value;
        self.trust = Some(trust);
        self
    }

    /// Asymmetrie-Kritisch setzen
    pub fn asymmetry_critical(mut self, value: f32) -> Self {
        let mut trust = self.trust.unwrap_or_default();
        trust.asymmetry_critical = value;
        self.trust = Some(trust);
        self
    }

    /// Newcomer-Trust setzen
    pub fn newcomer_trust(mut self, value: f32) -> Self {
        let mut trust = self.trust.unwrap_or_default();
        trust.newcomer_trust = value;
        self.trust = Some(trust);
        self
    }

    // ========================================================================
    // Activity Builder Methods
    // ========================================================================

    /// Activity-Konfiguration setzen
    pub fn activity(mut self, activity: ActivityConfig) -> Self {
        self.activity = Some(activity);
        self
    }

    /// τ in Tagen setzen
    pub fn activity_tau_days(mut self, days: u32) -> Self {
        let mut activity = self.activity.unwrap_or_default();
        activity.tau_days = days;
        self.activity = Some(activity);
        self
    }

    /// κ setzen
    pub fn activity_kappa(mut self, kappa: u32) -> Self {
        let mut activity = self.activity.unwrap_or_default();
        activity.kappa = kappa;
        self.activity = Some(activity);
        self
    }

    // ========================================================================
    // Temporal Builder Methods
    // ========================================================================

    /// Temporale Konfiguration setzen
    pub fn temporal(mut self, temporal: TemporalConfig) -> Self {
        self.temporal = Some(temporal);
        self
    }

    /// λ pro Tag setzen
    pub fn lambda_per_day(mut self, lambda: f64) -> Self {
        let mut temporal = self.temporal.unwrap_or_default();
        temporal.lambda_per_day = lambda;
        self.temporal = Some(temporal);
        self
    }

    // ========================================================================
    // Human-Factor Builder Methods
    // ========================================================================

    /// Human-Factor Konfiguration setzen
    pub fn human_factor(mut self, hf: HumanFactorConfig) -> Self {
        self.human_factor = Some(hf);
        self
    }

    // ========================================================================
    // Protection Builder Methods (Κ19 Anti-Calcification)
    // ========================================================================

    /// Protection-Konfiguration setzen
    pub fn protection(mut self, protection: ProtectionConfig) -> Self {
        self.protection = Some(protection);
        self
    }

    /// Gini-Threshold setzen (Optimal: 0.35)
    pub fn gini_threshold(mut self, value: f64) -> Self {
        let mut protection = self.protection.unwrap_or_default();
        protection.gini_threshold = value;
        self.protection = Some(protection);
        self
    }

    /// Decay-Rate setzen (Optimal: 0.0008)
    pub fn decay_rate(mut self, value: f64) -> Self {
        let mut protection = self.protection.unwrap_or_default();
        protection.decay_rate = value;
        self.protection = Some(protection);
        self
    }

    // ========================================================================
    // Realm Builder Methods (Κ23-Κ24)
    // ========================================================================

    /// Realm-Konfiguration setzen
    pub fn realm(mut self, realm: RealmConfig) -> Self {
        self.realm = Some(realm);
        self
    }

    /// Realm-Crossing-Penalty setzen (Optimal: 0.85)
    pub fn realm_crossing_penalty(mut self, value: f32) -> Self {
        let mut realm = self.realm.unwrap_or_default();
        realm.realm_crossing_penalty = value;
        self.realm = Some(realm);
        self
    }

    // ========================================================================
    // Build Methods
    // ========================================================================

    /// Baue die Konfiguration
    pub fn build(self) -> WorldFormulaConfig {
        WorldFormulaConfig {
            trust: self.trust.unwrap_or_default(),
            activity: self.activity.unwrap_or_default(),
            temporal: self.temporal.unwrap_or_default(),
            human_factor: self.human_factor.unwrap_or_default(),
            protection: self.protection.unwrap_or_default(),
            realm: self.realm.unwrap_or_default(),
        }
    }

    /// Baue und validiere
    pub fn build_validated(self) -> Result<WorldFormulaConfig, ConfigValidationError> {
        let config = self.build();
        config.validate()?;
        Ok(config)
    }
}

// ============================================================================
// Global Config (Thread-Safe Singleton)
// ============================================================================

use std::sync::OnceLock;

static GLOBAL_CONFIG: OnceLock<WorldFormulaConfig> = OnceLock::new();

/// Globale Konfiguration initialisieren
///
/// Kann nur einmal aufgerufen werden. Nachfolgende Aufrufe werden ignoriert.
pub fn init_global_config(config: WorldFormulaConfig) {
    let _ = GLOBAL_CONFIG.set(config);
}

/// Globale Konfiguration abrufen
///
/// Gibt Default zurück, falls nicht initialisiert.
pub fn global_config() -> &'static WorldFormulaConfig {
    GLOBAL_CONFIG.get_or_init(WorldFormulaConfig::default)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = WorldFormulaConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_builder_creates_valid_config() {
        let config = WorldFormulaConfig::builder()
            .positive_delta(0.05)
            .negative_multiplier(2.5)
            .hop_damping_factor(0.88)
            .activity_tau_days(60)
            .lambda_per_day(0.02)
            .build();

        assert_eq!(config.trust.positive_delta, 0.05);
        assert_eq!(config.trust.negative_multiplier, 2.5);
        assert_eq!(config.trust.hop_damping_factor, 0.88);
        assert_eq!(config.activity.tau_days, 60);
        assert_eq!(config.temporal.lambda_per_day, 0.02);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_optimal_defaults() {
        // Überprüfe die optimalen Defaults aus der Simulation
        let config = TrustConfig::default();

        assert_eq!(config.positive_delta, 0.045); // Optimal: 0.045
        assert_eq!(config.negative_multiplier, 2.2); // Optimal: 2.2
        assert_eq!(config.hop_damping_factor, 0.85); // Optimal: 0.85

        // Effektive negative_delta = 0.045 × 2.2 = 0.099
        assert!((config.negative_delta() - 0.099).abs() < 0.001);

        // Chain-Trust Faktor für 3 Hops: 0.85^3 ≈ 0.614
        assert!((config.chain_trust_factor(3) - 0.614).abs() < 0.01);
    }

    #[test]
    fn test_protection_defaults() {
        let config = ProtectionConfig::default();

        assert_eq!(config.gini_threshold, 0.35); // Optimal: 0.35
        assert_eq!(config.decay_rate, 0.0008); // Optimal: 0.0008
        assert_eq!(config.power_cap_exponent, 0.5); // sqrt-Scaling
    }

    #[test]
    fn test_realm_defaults() {
        let config = RealmConfig::default();

        assert_eq!(config.realm_crossing_penalty, 0.85); // Optimal: 0.85
        assert_eq!(config.max_realm_depth, 10);
        assert_eq!(config.min_governance_quorum, 0.51);
    }

    #[test]
    fn test_invalid_learning_rate_rejected() {
        let mut config = WorldFormulaConfig::default();
        config.trust.positive_delta = 0.2; // > 0.1 ist ungültig

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_damping_factor_rejected() {
        let mut config = WorldFormulaConfig::default();
        config.trust.hop_damping_factor = 1.5; // > 1.0 ist ungültig

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_weights_rejected() {
        let mut config = WorldFormulaConfig::default();
        config.trust.dimension_weights = [0.1, 0.1, 0.1, 0.1, 0.1, 0.1]; // Sum = 0.6

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_trust_config_asymmetry_for() {
        use super::super::trust::TrustDimension;
        let config = TrustConfig::default();

        // Neue optimale asymmetry_base: 2.2
        assert_eq!(config.asymmetry_for(TrustDimension::Reliability), 2.2);
        // Neue optimale asymmetry_critical: 2.75
        assert_eq!(config.asymmetry_for(TrustDimension::Omega), 2.75);
    }

    #[test]
    fn test_temporal_conversions() {
        let config = TemporalConfig::default();

        assert_eq!(config.max_age_seconds(), 365 * 24 * 3600);
        assert!((config.lambda_per_second() - 0.01 / 86400.0).abs() < 1e-10);
    }

    #[test]
    fn test_activity_conversions() {
        let config = ActivityConfig::default();

        assert_eq!(config.tau_seconds(), 90 * 24 * 3600);
        assert_eq!(config.tau_seconds_mobile(), 30 * 24 * 3600);
    }

    #[test]
    fn test_global_config() {
        // Erstzugriff gibt Default zurück
        let config = global_config();
        assert_eq!(config.trust.positive_delta, 0.045);
    }

    #[test]
    fn test_capped_power() {
        let protection = ProtectionConfig::default();

        // sqrt(100) / sqrt(4) = 10 / 2 = 5
        let capped = protection.capped_power(100.0, 4);
        assert!((capped - 5.0).abs() < 0.01);

        // Count 0 gibt 0 zurück (Division durch 0 vermeiden)
        assert_eq!(protection.capped_power(100.0, 0), 0.0);
    }

    #[test]
    fn test_cross_realm_trust() {
        let realm = RealmConfig::default();

        // local_trust 0.8 × penalty 0.85 = 0.68
        let cross = realm.cross_realm_trust(0.8);
        assert!((cross - 0.68).abs() < 0.01);
    }

    #[test]
    fn test_build_validated_success() {
        let result = WorldFormulaConfig::builder()
            .positive_delta(0.045)
            .negative_multiplier(2.2)
            .build_validated();

        assert!(result.is_ok());
    }

    #[test]
    fn test_build_validated_failure() {
        let result = WorldFormulaConfig::builder()
            .positive_delta(-0.1) // Negativ ist ungültig
            .build_validated();

        assert!(result.is_err());
    }

    #[test]
    fn test_protection_builder() {
        let config = WorldFormulaConfig::builder()
            .gini_threshold(0.4)
            .decay_rate(0.001)
            .build();

        assert_eq!(config.protection.gini_threshold, 0.4);
        assert_eq!(config.protection.decay_rate, 0.001);
    }

    #[test]
    fn test_realm_builder() {
        let config = WorldFormulaConfig::builder()
            .realm_crossing_penalty(0.9)
            .build();

        assert_eq!(config.realm.realm_crossing_penalty, 0.9);
    }
}
