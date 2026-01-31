//! # World Formula Engine
//!
//! Berechnet die Weltformel V2.0 gemäß Κ15b-d.
//!
//! ## Axiom-Referenz
//!
//! - **Κ15b (Weltformel)**: `𝔼 = Σ 𝔸(s) · σ⃗( ‖𝕎(s)‖_w · ln|ℂ(s)| · 𝒮(s) ) · Ĥ(s) · w(s,t)`
//! - **Κ15c (Sigmoid)**: `σ⃗(x) = 1 / (1 + e^(-x))`
//! - **Κ15d (Approximation)**: Count-Min Sketch für ℐ

use crate::domain::{
    Activity, ContextType, DID, HumanFactor, Surprisal, TrustVector6D,
    WorldFormulaContribution, WorldFormulaStatus,
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// World Formula Engine - berechnet 𝔼 (Κ15b-d)
///
/// ```text
/// 𝔼 = Σ 𝔸(s) · σ⃗( ‖𝕎(s)‖_w · ln|ℂ(s)| · 𝒮(s) ) · Ĥ(s) · w(s,t)
///     \_____/   \_____________________________________________/  \___/  \____/
///     Activity          Sigmoid(Trust×History×Surprisal)        Human  Temporal
/// ```
pub struct WorldFormulaEngine {
    /// Contributions pro DID
    contributions: HashMap<DID, WorldFormulaContribution>,

    /// Letzte Berechnung
    last_computed: Option<WorldFormulaStatus>,

    /// Konfiguration
    config: WorldFormulaConfig,
}

/// Konfiguration für WorldFormulaEngine
#[derive(Debug, Clone)]
pub struct WorldFormulaConfig {
    /// Zeitfenster τ für Activity (in Tagen)
    pub activity_window_days: u64,

    /// Aktivitäts-Schwelle κ
    pub activity_threshold: u64,

    /// Temporal Decay Rate
    pub temporal_decay_rate: f64,

    /// Default-Kontext für Trust-Gewichtung
    pub default_context: ContextType,
}

impl Default for WorldFormulaConfig {
    fn default() -> Self {
        Self {
            activity_window_days: 90,
            activity_threshold: 10,
            temporal_decay_rate: 0.99,
            default_context: ContextType::Default,
        }
    }
}

impl WorldFormulaEngine {
    /// Erstelle neue WorldFormulaEngine
    pub fn new(config: WorldFormulaConfig) -> Self {
        Self {
            contributions: HashMap::new(),
            last_computed: None,
            config,
        }
    }

    /// Erstelle mit Default-Config
    pub fn default() -> Self {
        Self::new(WorldFormulaConfig::default())
    }

    /// Registriere oder aktualisiere Contribution für eine DID
    pub fn update_contribution(
        &mut self,
        did: DID,
        trust: TrustVector6D,
        recent_events: u64,
        causal_history_size: u64,
        surprisal: Surprisal,
        human_factor: HumanFactor,
    ) {
        let activity = Activity {
            recent_events,
            tau_days: self.config.activity_window_days,
            kappa: self.config.activity_threshold,
        };

        let contribution = WorldFormulaContribution::new(did.clone())
            .with_activity(activity)
            .with_trust(trust)
            .with_causal_history(causal_history_size)
            .with_surprisal(surprisal)
            .with_human_factor(human_factor)
            .with_context(self.config.default_context);

        self.contributions.insert(did, contribution);
    }

    /// Κ15b: Berechne globale Weltformel 𝔼
    pub fn compute_global(&mut self) -> WorldFormulaStatus {
        let now = Utc::now();
        let mut total_e = 0.0;
        let mut total_activity = 0.0;
        let mut total_trust_norm = 0.0;
        let mut human_verified = 0usize;

        for contribution in self.contributions.values() {
            total_e += contribution.compute();
            total_activity += contribution.activity.value();
            total_trust_norm += contribution.trust.weighted_norm(
                &contribution.context.weights(),
            );

            if contribution.human_factor != HumanFactor::NotVerified {
                human_verified += 1;
            }
        }

        let entity_count = self.contributions.len() as u64;
        let avg_activity = if entity_count > 0 {
            total_activity / entity_count as f64
        } else {
            0.0
        };
        let avg_trust_norm = if entity_count > 0 {
            total_trust_norm / entity_count as f64
        } else {
            0.0
        };
        let human_ratio = if entity_count > 0 {
            human_verified as f64 / entity_count as f64
        } else {
            0.0
        };

        let delta_24h = self.last_computed
            .as_ref()
            .map(|prev| total_e - prev.total_e)
            .unwrap_or(0.0);

        let status = WorldFormulaStatus {
            total_e,
            delta_24h,
            entity_count,
            avg_activity,
            avg_trust_norm,
            human_verified_ratio: human_ratio,
            realm_id: None,
            computed_at: now,
        };

        self.last_computed = Some(status.clone());
        status
    }

    /// Berechne Weltformel für ein spezifisches Realm
    pub fn compute_for_realm(&self, realm_id: &str) -> WorldFormulaStatus {
        let now = Utc::now();

        // Filter: nur DIDs die zum Realm gehören (TODO: Realm-Membership tracking)
        // Für jetzt: alle Contributions verwenden
        let realm_contributions: Vec<_> = self.contributions.values().collect();

        let mut total_e = 0.0;
        let mut total_activity = 0.0;
        let mut total_trust_norm = 0.0;
        let mut human_verified = 0usize;

        for contribution in &realm_contributions {
            total_e += contribution.compute();
            total_activity += contribution.activity.value();
            total_trust_norm += contribution.trust.weighted_norm(
                &contribution.context.weights(),
            );

            if contribution.human_factor != HumanFactor::NotVerified {
                human_verified += 1;
            }
        }

        let entity_count = realm_contributions.len() as u64;
        let avg_activity = if entity_count > 0 {
            total_activity / entity_count as f64
        } else {
            0.0
        };
        let avg_trust_norm = if entity_count > 0 {
            total_trust_norm / entity_count as f64
        } else {
            0.0
        };

        WorldFormulaStatus {
            total_e,
            delta_24h: 0.0, // Keine Historie für Realm-spezifische Berechnung
            entity_count,
            avg_activity,
            avg_trust_norm,
            human_verified_ratio: if entity_count > 0 {
                human_verified as f64 / entity_count as f64
            } else {
                0.0
            },
            realm_id: Some(realm_id.to_string()),
            computed_at: now,
        }
    }

    /// Hole Contribution für eine DID
    pub fn get_contribution(&self, did: &DID) -> Option<&WorldFormulaContribution> {
        self.contributions.get(did)
    }

    /// Berechne individuellen Beitrag
    pub fn compute_individual(&self, did: &DID) -> Option<f64> {
        self.contributions.get(did).map(|c| c.compute())
    }

    /// Top N Contributors
    pub fn top_contributors(&self, n: usize) -> Vec<(DID, f64)> {
        let mut sorted: Vec<_> = self.contributions
            .iter()
            .map(|(did, c)| (did.clone(), c.compute()))
            .collect();

        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.truncate(n);
        sorted
    }

    /// Temporal Weight w(s,t) - Decay über Zeit
    pub fn temporal_weight(&self, last_active: DateTime<Utc>) -> f64 {
        let now = Utc::now();
        let days_inactive = (now - last_active).num_days() as f64;

        // Exponentieller Decay
        self.config.temporal_decay_rate.powf(days_inactive)
    }

    /// Statistiken
    pub fn stats(&self) -> WorldFormulaEngineStats {
        let status = self.last_computed.clone().unwrap_or_default();

        WorldFormulaEngineStats {
            total_contributions: self.contributions.len(),
            total_e: status.total_e,
            avg_activity: status.avg_activity,
            avg_trust_norm: status.avg_trust_norm,
            human_verified_ratio: status.human_verified_ratio,
        }
    }
}

/// Statistiken der WorldFormulaEngine
#[derive(Debug, Clone)]
pub struct WorldFormulaEngineStats {
    pub total_contributions: usize,
    pub total_e: f64,
    pub avg_activity: f64,
    pub avg_trust_norm: f64,
    pub human_verified_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DID;
    use chrono::Duration;

    #[test]
    fn test_compute_global() {
        let mut engine = WorldFormulaEngine::default();

        // Alice: aktiv, hoher Trust, human-verified
        engine.update_contribution(
            DID::new_self("alice"),
            TrustVector6D::new(0.9, 0.9, 0.8, 0.7, 0.6, 0.9),
            50,
            1000,
            Surprisal::default(),
            HumanFactor::FullAttestation,
        );

        // Bob: weniger aktiv, niedrigerer Trust
        engine.update_contribution(
            DID::new_self("bob"),
            TrustVector6D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5),
            10,
            100,
            Surprisal::default(),
            HumanFactor::NotVerified,
        );

        let status = engine.compute_global();

        assert_eq!(status.entity_count, 2);
        assert!(status.total_e > 0.0);
        assert!(status.avg_activity > 0.0);
        assert!((status.human_verified_ratio - 0.5).abs() < 0.001); // 1 von 2
    }

    #[test]
    fn test_top_contributors() {
        let mut engine = WorldFormulaEngine::default();

        // Drei Contributors mit unterschiedlicher Aktivität/Trust
        engine.update_contribution(
            DID::new_self("high"),
            TrustVector6D::new(0.9, 0.9, 0.9, 0.9, 0.9, 0.9),
            100,
            5000,
            Surprisal::default(),
            HumanFactor::FullAttestation,
        );

        engine.update_contribution(
            DID::new_self("medium"),
            TrustVector6D::new(0.6, 0.6, 0.6, 0.6, 0.6, 0.6),
            30,
            500,
            Surprisal::default(),
            HumanFactor::BasicAttestation,
        );

        engine.update_contribution(
            DID::new_self("low"),
            TrustVector6D::new(0.3, 0.3, 0.3, 0.3, 0.3, 0.3),
            5,
            50,
            Surprisal::default(),
            HumanFactor::NotVerified,
        );

        let top = engine.top_contributors(2);

        assert_eq!(top.len(), 2);
        // "high" sollte erste Position haben
        assert!(top[0].0.to_uri().contains("high"));
        assert!(top[0].1 > top[1].1);
    }

    #[test]
    fn test_temporal_weight() {
        let engine = WorldFormulaEngine::default();

        // Gerade aktiv: weight ≈ 1.0
        let now = Utc::now();
        let weight_now = engine.temporal_weight(now);
        assert!(weight_now > 0.99);

        // 30 Tage inaktiv: weight = 0.99^30 ≈ 0.74
        let days_ago_30 = now - Duration::days(30);
        let weight_30 = engine.temporal_weight(days_ago_30);
        assert!(weight_30 < 0.8);
        assert!(weight_30 > 0.7);
    }
}
