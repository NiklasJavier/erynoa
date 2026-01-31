//! # Anti-Calcification
//!
//! Verhindert Macht-Konzentration gemäß Κ19.
//!
//! ## Axiom-Referenz
//!
//! - **Κ19 (Macht-Verdünnung)**: `∀s : power(s) ≤ √(Σ power) / |S|^(1/4)`
//!
//! ## Mechanismen
//!
//! 1. **Power-Cap**: Individuelle Macht darf Schwelle nicht überschreiten
//! 2. **Decay**: Inaktive Macht verfällt über Zeit
//! 3. **Distribution**: Anreize für Macht-Verteilung

use crate::domain::DID;
use std::collections::HashMap;
use thiserror::Error;

/// Fehler bei Anti-Calcification
#[derive(Debug, Error)]
pub enum CalcificationError {
    #[error("Power cap exceeded for {did}: {current} > {max}")]
    PowerCapExceeded { did: String, current: f64, max: f64 },

    #[error("Calcification detected: top {count} entities hold {percentage}% of power")]
    CalcificationDetected { count: usize, percentage: f64 },
}

/// Ergebnis von Anti-Calcification Operationen
pub type CalcificationResult<T> = Result<T, CalcificationError>;

/// Anti-Calcification Monitor (Κ19)
///
/// ```text
///                    Κ19: power(s) ≤ √(Σ power) / |S|^(1/4)
///                                    │
///                    ┌───────────────┴───────────────┐
///                    │                               │
///                    ▼                               ▼
///            ┌─────────────┐                ┌─────────────┐
///            │  Power Cap  │                │   Decay     │
///            │  Check      │                │   Apply     │
///            └─────────────┘                └─────────────┘
/// ```
pub struct AntiCalcification {
    /// Power-Werte pro DID
    power_values: HashMap<DID, f64>,

    /// Konfiguration
    config: AntiCalcificationConfig,
}

/// Konfiguration für Anti-Calcification
#[derive(Debug, Clone)]
pub struct AntiCalcificationConfig {
    /// Κ19: Exponent für Entitäten-Anzahl (default: 0.25 = 1/4)
    pub entity_exponent: f64,

    /// Decay-Rate pro Tag
    pub decay_rate_per_day: f64,

    /// Alarm-Schwelle: wenn top n% mehr als x% der Macht halten
    pub alarm_top_percentage: f64,
    pub alarm_power_threshold: f64,
}

impl Default for AntiCalcificationConfig {
    fn default() -> Self {
        Self {
            entity_exponent: 0.25,
            decay_rate_per_day: 0.01,
            alarm_top_percentage: 0.01, // Top 1%
            alarm_power_threshold: 0.5, // 50% der Macht
        }
    }
}

impl AntiCalcification {
    /// Erstelle neuen Anti-Calcification Monitor
    pub fn new(config: AntiCalcificationConfig) -> Self {
        Self {
            power_values: HashMap::new(),
            config,
        }
    }

    /// Erstelle mit Default-Config
    pub fn default() -> Self {
        Self::new(AntiCalcificationConfig::default())
    }

    /// Setze Power-Wert für DID
    pub fn set_power(&mut self, did: DID, power: f64) {
        self.power_values.insert(did, power);
    }

    /// Κ19: Berechne maximale erlaubte Macht für eine Entität
    ///
    /// ```text
    /// max_power(s) = √(Σ power) / |S|^(1/4)
    /// ```
    pub fn calculate_power_cap(&self) -> f64 {
        let total_power: f64 = self.power_values.values().sum();
        let entity_count = self.power_values.len().max(1) as f64;

        total_power.sqrt() / entity_count.powf(self.config.entity_exponent)
    }

    /// Prüfe ob DID das Power-Cap überschreitet
    pub fn check_power_cap(&self, did: &DID) -> CalcificationResult<()> {
        let power = self.power_values.get(did).copied().unwrap_or(0.0);
        let cap = self.calculate_power_cap();

        if power > cap {
            return Err(CalcificationError::PowerCapExceeded {
                did: did.to_uri(),
                current: power,
                max: cap,
            });
        }

        Ok(())
    }

    /// Prüfe auf systemweite Calcification
    pub fn check_system_calcification(&self) -> CalcificationResult<()> {
        let total_power: f64 = self.power_values.values().sum();
        if total_power == 0.0 {
            return Ok(());
        }

        // Sortiere nach Power (absteigend)
        let mut sorted: Vec<_> = self.power_values.values().copied().collect();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        // Top n% der Entitäten
        let top_count =
            (self.power_values.len() as f64 * self.config.alarm_top_percentage).ceil() as usize;
        let top_count = top_count.max(1);

        let top_power: f64 = sorted.iter().take(top_count).sum();
        let top_percentage = top_power / total_power;

        if top_percentage > self.config.alarm_power_threshold {
            return Err(CalcificationError::CalcificationDetected {
                count: top_count,
                percentage: top_percentage * 100.0,
            });
        }

        Ok(())
    }

    /// Wende Decay auf alle Power-Werte an
    pub fn apply_decay(&mut self, days: f64) {
        let decay_factor = (1.0 - self.config.decay_rate_per_day).powf(days);

        for power in self.power_values.values_mut() {
            *power *= decay_factor;
        }
    }

    /// Berechne Gini-Koeffizient (Ungleichheitsmaß)
    pub fn gini_coefficient(&self) -> f64 {
        let mut values: Vec<f64> = self.power_values.values().copied().collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n = values.len() as f64;
        if n == 0.0 {
            return 0.0;
        }

        let sum: f64 = values.iter().sum();
        if sum == 0.0 {
            return 0.0;
        }

        let mut _cumsum = 0.0;
        let mut gini_sum = 0.0;

        for (i, &val) in values.iter().enumerate() {
            _cumsum += val;
            gini_sum += (2.0 * (i as f64 + 1.0) - n - 1.0) * val;
        }

        gini_sum / (n * sum)
    }

    /// Statistiken
    pub fn stats(&self) -> AntiCalcificationStats {
        let power_cap = self.calculate_power_cap();
        let entities_over_cap = self
            .power_values
            .iter()
            .filter(|(_, &p)| p > power_cap)
            .count();

        AntiCalcificationStats {
            total_entities: self.power_values.len(),
            total_power: self.power_values.values().sum(),
            power_cap,
            entities_over_cap,
            gini_coefficient: self.gini_coefficient(),
        }
    }
}

/// Statistiken des Anti-Calcification Monitors
#[derive(Debug, Clone)]
pub struct AntiCalcificationStats {
    pub total_entities: usize,
    pub total_power: f64,
    pub power_cap: f64,
    pub entities_over_cap: usize,
    pub gini_coefficient: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_cap_calculation() {
        let mut ac = AntiCalcification::default();

        // 100 Entitäten mit je 1.0 Power
        for i in 0..100 {
            ac.set_power(DID::new_self(&format!("user{}", i)), 1.0);
        }

        // √100 / 100^0.25 = 10 / 3.16 ≈ 3.16
        let cap = ac.calculate_power_cap();
        assert!(cap > 3.0 && cap < 3.5);
    }

    #[test]
    fn test_power_cap_violation() {
        let mut ac = AntiCalcification::default();

        // 10 normale User
        for i in 0..10 {
            ac.set_power(DID::new_self(&format!("user{}", i)), 1.0);
        }

        // 1 Whale mit viel Power
        let whale = DID::new_self("whale");
        ac.set_power(whale.clone(), 100.0);

        let result = ac.check_power_cap(&whale);
        assert!(matches!(
            result,
            Err(CalcificationError::PowerCapExceeded { .. })
        ));
    }

    #[test]
    fn test_calcification_detection() {
        let mut ac = AntiCalcification::new(AntiCalcificationConfig {
            alarm_top_percentage: 0.1,  // Top 10%
            alarm_power_threshold: 0.5, // 50%
            ..Default::default()
        });

        // 10 kleine User
        for i in 0..9 {
            ac.set_power(DID::new_self(&format!("small{}", i)), 1.0);
        }

        // 1 großer User hat mehr als 50%
        ac.set_power(DID::new_self("big"), 100.0);

        let result = ac.check_system_calcification();
        assert!(matches!(
            result,
            Err(CalcificationError::CalcificationDetected { .. })
        ));
    }

    #[test]
    fn test_decay() {
        let mut ac = AntiCalcification::new(AntiCalcificationConfig {
            decay_rate_per_day: 0.1, // 10% pro Tag
            ..Default::default()
        });

        let user_did = DID::new_self("user");
        ac.set_power(user_did.clone(), 100.0);

        // Verifiziere Initial-Wert
        assert_eq!(ac.power_values.get(&user_did).copied(), Some(100.0));

        // Nach 10 Tagen: 100 × (1 - 0.1)^10 = 100 × 0.9^10 ≈ 34.87
        ac.apply_decay(10.0);

        let power = ac
            .power_values
            .get(&user_did)
            .expect("Power should still exist after decay");
        assert!(
            *power > 30.0 && *power < 40.0,
            "Expected power between 30 and 40, got {}",
            power
        );
    }

    #[test]
    fn test_gini_coefficient() {
        let mut ac = AntiCalcification::default();

        // Perfekte Gleichheit
        for i in 0..10 {
            ac.set_power(DID::new_self(&format!("equal{}", i)), 10.0);
        }
        assert!(ac.gini_coefficient().abs() < 0.01);

        // Ungleichheit
        let mut ac2 = AntiCalcification::default();
        ac2.set_power(DID::new_self("rich"), 99.0);
        for i in 0..99 {
            ac2.set_power(DID::new_self(&format!("poor{}", i)), 0.01);
        }
        assert!(ac2.gini_coefficient() > 0.9);
    }
}
