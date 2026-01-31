//! # Diversity Monitor
//!
//! Überwacht System-Diversität gemäß Κ20.
//!
//! ## Axiom-Referenz
//!
//! - **Κ20 (Diversitäts-Erhaltung)**: `H(TypeDistribution) ≥ θ_diversity`
//!
//! ## Shannon-Entropie
//!
//! ```text
//! H(X) = -Σ p(x) · log₂(p(x))
//! ```
//!
//! Höhere Entropie = mehr Diversität

use crate::domain::DID;
use std::collections::HashMap;
use thiserror::Error;

/// Fehler bei Diversity-Operationen
#[derive(Debug, Error)]
pub enum DiversityError {
    #[error("Diversity threshold violated: {current} < {required}")]
    ThresholdViolated { current: f64, required: f64 },

    #[error("Monoculture detected: {category} dominates with {percentage}%")]
    MonocultureDetected { category: String, percentage: f64 },
}

/// Ergebnis von Diversity-Operationen
pub type DiversityResult<T> = Result<T, DiversityError>;

/// Diversity Monitor (Κ20)
///
/// Überwacht verschiedene Diversitäts-Dimensionen:
/// - **Typ-Diversität**: Verteilung von DID-Typen
/// - **Geografische Diversität**: Verteilung über Regionen
/// - **Aktivitäts-Diversität**: Verteilung von Event-Typen
pub struct DiversityMonitor {
    /// Zähler pro Kategorie pro Dimension
    dimensions: HashMap<String, HashMap<String, u64>>,

    /// Konfiguration
    config: DiversityConfig,
}

/// Konfiguration für DiversityMonitor
#[derive(Debug, Clone)]
pub struct DiversityConfig {
    /// Κ20: Minimum Entropie-Schwelle
    pub min_entropy: f64,

    /// Maximum Anteil einer einzelnen Kategorie
    pub max_single_category: f64,

    /// Alarm wenn Entropie unter diesem Wert fällt
    pub alarm_entropy_threshold: f64,
}

impl Default for DiversityConfig {
    fn default() -> Self {
        Self {
            min_entropy: 2.0,           // ~4 gleichverteilte Kategorien
            max_single_category: 0.5,   // Keine Kategorie > 50%
            alarm_entropy_threshold: 1.5,
        }
    }
}

impl DiversityMonitor {
    /// Erstelle neuen DiversityMonitor
    pub fn new(config: DiversityConfig) -> Self {
        Self {
            dimensions: HashMap::new(),
            config,
        }
    }

    /// Erstelle mit Default-Config
    pub fn default() -> Self {
        Self::new(DiversityConfig::default())
    }

    /// Registriere Beobachtung in einer Dimension
    pub fn observe(&mut self, dimension: &str, category: &str) {
        *self.dimensions
            .entry(dimension.to_string())
            .or_default()
            .entry(category.to_string())
            .or_default() += 1;
    }

    /// Registriere DID (für Typ-Diversität)
    pub fn observe_did(&mut self, did: &DID) {
        let category = format!("{:?}", did.namespace);
        self.observe("did_type", &category);
    }

    /// Κ20: Berechne Shannon-Entropie für eine Dimension
    ///
    /// ```text
    /// H(X) = -Σ p(x) · log₂(p(x))
    /// ```
    pub fn entropy(&self, dimension: &str) -> f64 {
        let counts = match self.dimensions.get(dimension) {
            Some(c) => c,
            None => return 0.0,
        };

        let total: u64 = counts.values().sum();
        if total == 0 {
            return 0.0;
        }

        let mut entropy = 0.0;
        for &count in counts.values() {
            if count > 0 {
                let p = count as f64 / total as f64;
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    /// Berechne Maximum mögliche Entropie (log₂(n))
    pub fn max_entropy(&self, dimension: &str) -> f64 {
        let counts = match self.dimensions.get(dimension) {
            Some(c) => c,
            None => return 0.0,
        };

        let n = counts.len() as f64;
        if n <= 1.0 {
            return 0.0;
        }

        n.log2()
    }

    /// Berechne normalisierte Entropie (0 bis 1)
    pub fn normalized_entropy(&self, dimension: &str) -> f64 {
        let h = self.entropy(dimension);
        let h_max = self.max_entropy(dimension);

        if h_max == 0.0 {
            return 0.0;
        }

        h / h_max
    }

    /// Prüfe Diversitäts-Schwelle für Dimension
    pub fn check_diversity(&self, dimension: &str) -> DiversityResult<()> {
        let entropy = self.entropy(dimension);

        if entropy < self.config.min_entropy {
            return Err(DiversityError::ThresholdViolated {
                current: entropy,
                required: self.config.min_entropy,
            });
        }

        Ok(())
    }

    /// Prüfe auf Monokultur (eine Kategorie dominiert)
    pub fn check_monoculture(&self, dimension: &str) -> DiversityResult<()> {
        let counts = match self.dimensions.get(dimension) {
            Some(c) => c,
            None => return Ok(()),
        };

        let total: u64 = counts.values().sum();
        if total == 0 {
            return Ok(());
        }

        for (category, &count) in counts {
            let percentage = count as f64 / total as f64;
            if percentage > self.config.max_single_category {
                return Err(DiversityError::MonocultureDetected {
                    category: category.clone(),
                    percentage: percentage * 100.0,
                });
            }
        }

        Ok(())
    }

    /// Hole Top-Kategorien für eine Dimension
    pub fn top_categories(&self, dimension: &str, n: usize) -> Vec<(String, u64, f64)> {
        let counts = match self.dimensions.get(dimension) {
            Some(c) => c,
            None => return vec![],
        };

        let total: u64 = counts.values().sum();
        if total == 0 {
            return vec![];
        }

        let mut sorted: Vec<_> = counts.iter()
            .map(|(k, &v)| (k.clone(), v, v as f64 / total as f64 * 100.0))
            .collect();

        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(n);
        sorted
    }

    /// Statistiken für alle Dimensionen
    pub fn stats(&self) -> DiversityStats {
        let dimension_stats: HashMap<String, DimensionStats> = self.dimensions.keys()
            .map(|dim| {
                let entropy = self.entropy(dim);
                let normalized = self.normalized_entropy(dim);
                let category_count = self.dimensions.get(dim)
                    .map(|c| c.len())
                    .unwrap_or(0);
                let total_observations = self.dimensions.get(dim)
                    .map(|c| c.values().sum::<u64>())
                    .unwrap_or(0);

                (dim.clone(), DimensionStats {
                    entropy,
                    normalized_entropy: normalized,
                    category_count,
                    total_observations,
                })
            })
            .collect();

        let avg_entropy = if dimension_stats.is_empty() {
            0.0
        } else {
            dimension_stats.values()
                .map(|s| s.entropy)
                .sum::<f64>() / dimension_stats.len() as f64
        };

        DiversityStats {
            dimensions: dimension_stats,
            average_entropy: avg_entropy,
        }
    }
}

/// Statistiken pro Dimension
#[derive(Debug, Clone)]
pub struct DimensionStats {
    pub entropy: f64,
    pub normalized_entropy: f64,
    pub category_count: usize,
    pub total_observations: u64,
}

/// Gesamtstatistiken
#[derive(Debug, Clone)]
pub struct DiversityStats {
    pub dimensions: HashMap<String, DimensionStats>,
    pub average_entropy: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_uniform_distribution() {
        let mut dm = DiversityMonitor::default();

        // 4 Kategorien mit je 25 Beobachtungen
        for _ in 0..25 {
            dm.observe("test", "A");
            dm.observe("test", "B");
            dm.observe("test", "C");
            dm.observe("test", "D");
        }

        // H = log₂(4) = 2.0
        let entropy = dm.entropy("test");
        assert!((entropy - 2.0).abs() < 0.01);

        // Normalisierte Entropie sollte 1.0 sein
        let normalized = dm.normalized_entropy("test");
        assert!((normalized - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_entropy_skewed_distribution() {
        let mut dm = DiversityMonitor::default();

        // Eine Kategorie dominiert
        for _ in 0..90 {
            dm.observe("test", "dominant");
        }
        for _ in 0..10 {
            dm.observe("test", "minor");
        }

        // Entropie sollte niedrig sein
        let entropy = dm.entropy("test");
        assert!(entropy < 1.0);
    }

    #[test]
    fn test_monoculture_detection() {
        let mut dm = DiversityMonitor::new(DiversityConfig {
            max_single_category: 0.5,
            ..Default::default()
        });

        // Eine Kategorie > 50%
        for _ in 0..60 {
            dm.observe("test", "dominant");
        }
        for _ in 0..40 {
            dm.observe("test", "other");
        }

        let result = dm.check_monoculture("test");
        assert!(matches!(result, Err(DiversityError::MonocultureDetected { .. })));
    }

    #[test]
    fn test_did_observation() {
        let mut dm = DiversityMonitor::default();

        // Verschiedene DID-Typen
        dm.observe_did(&DID::new_self("alice"));
        dm.observe_did(&DID::new_self("bob"));
        dm.observe_did(&DID::new_guild("guild1"));
        dm.observe_did(&DID::new_guild("guild2"));

        let stats = dm.stats();
        let did_stats = stats.dimensions.get("did_type").unwrap();

        assert_eq!(did_stats.total_observations, 4);
        assert!(did_stats.category_count >= 2);
    }
}
