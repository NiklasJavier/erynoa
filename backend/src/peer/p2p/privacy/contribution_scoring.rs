//! # Contribution-Scoring (V2.6)
//!
//! Kumulatives Score-System mit exponentiellem Decay für DC3+-Integration.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                   CONTRIBUTION-SCORING SYSTEM                               │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐           │
//! │  │  RAW-SCORES      │  │  DECAY-ENGINE    │  │  AGGREGATION     │           │
//! │  │  ─────────────   │  │  ─────────────   │  │  ─────────────   │           │
//! │  │  Storage         │  │  Exponential     │  │  Weighted-Sum    │           │
//! │  │  Bandwidth       │→ │  Time-Based      │→ │  Normalization   │           │
//! │  │  Compute         │  │  Configurable    │  │  Clamping        │           │
//! │  │  PoUW            │  │                  │  │                  │           │
//! │  └──────────────────┘  └──────────────────┘  └──────────────────┘           │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Decay-Formel
//!
//! ```text
//! score(t) = base_score × e^(-γ × age_days)
//!
//! γ = ln(2) / half_life_days
//!
//! Defaults:
//! - Storage: half_life = 30 Tage
//! - Bandwidth: half_life = 14 Tage
//! - Compute: half_life = 7 Tage
//! - PoUW: half_life = 7 Tage
//! ```

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Storage-Score Decay Half-Life (30 Tage)
pub const STORAGE_DECAY_HALF_LIFE_DAYS: f64 = 30.0;

/// Bandwidth-Score Decay Half-Life (14 Tage)
pub const BANDWIDTH_DECAY_HALF_LIFE_DAYS: f64 = 14.0;

/// Compute-Score Decay Half-Life (7 Tage)
pub const COMPUTE_DECAY_HALF_LIFE_DAYS: f64 = 7.0;

/// PoUW-Score Decay Half-Life (7 Tage)
pub const POUW_DECAY_HALF_LIFE_DAYS: f64 = 7.0;

/// Maximum Score-History Einträge
pub const MAX_SCORE_HISTORY: usize = 1000;

/// Minimum Score für Tracking
pub const MIN_CONTRIBUTION_SCORE: f64 = 0.001;

// ============================================================================
// EXPONENTIAL DECAY CALCULATOR
// ============================================================================

/// Exponentieller Decay-Calculator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExponentialDecayCalculator {
    /// Half-Life in Tagen
    pub half_life_days: f64,
    /// Berechneter Decay-Koeffizient γ
    gamma: f64,
}

impl ExponentialDecayCalculator {
    /// Erstelle neuen Calculator
    pub fn new(half_life_days: f64) -> Self {
        let gamma = (2.0_f64).ln() / half_life_days;
        Self {
            half_life_days,
            gamma,
        }
    }

    /// Berechne Decay-Faktor für Alter in Tagen
    pub fn decay_factor(&self, age_days: f64) -> f64 {
        (-self.gamma * age_days).exp()
    }

    /// Berechne aktuellen Wert mit Decay
    pub fn apply_decay(&self, base_value: f64, age_days: f64) -> f64 {
        base_value * self.decay_factor(age_days)
    }

    /// Berechne Alter in Tagen seit Timestamp
    pub fn age_days_since(timestamp: u64) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now > timestamp {
            (now - timestamp) as f64 / 86400.0
        } else {
            0.0
        }
    }

    /// Standardwerte für verschiedene Score-Typen
    pub fn for_storage() -> Self {
        Self::new(STORAGE_DECAY_HALF_LIFE_DAYS)
    }

    pub fn for_bandwidth() -> Self {
        Self::new(BANDWIDTH_DECAY_HALF_LIFE_DAYS)
    }

    pub fn for_compute() -> Self {
        Self::new(COMPUTE_DECAY_HALF_LIFE_DAYS)
    }

    pub fn for_pouw() -> Self {
        Self::new(POUW_DECAY_HALF_LIFE_DAYS)
    }
}

impl Default for ExponentialDecayCalculator {
    fn default() -> Self {
        Self::new(14.0) // 14 Tage Standard
    }
}

// ============================================================================
// SCORE ENTRY
// ============================================================================

/// Einzelner Score-Eintrag mit Timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreEntry {
    /// Base-Score (ohne Decay)
    pub base_score: f64,
    /// Timestamp der Erstellung
    pub timestamp: u64,
    /// Kategorie
    pub category: ScoreCategory,
    /// Optionale Beschreibung
    pub description: Option<String>,
}

impl ScoreEntry {
    /// Erstelle neuen Eintrag
    pub fn new(base_score: f64, category: ScoreCategory, description: Option<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            base_score,
            timestamp: now,
            category,
            description,
        }
    }

    /// Alter in Tagen
    pub fn age_days(&self) -> f64 {
        ExponentialDecayCalculator::age_days_since(self.timestamp)
    }

    /// Aktueller Score mit Decay
    pub fn current_score(&self, decay: &ExponentialDecayCalculator) -> f64 {
        decay.apply_decay(self.base_score, self.age_days())
    }
}

/// Score-Kategorie
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScoreCategory {
    /// Storage-Beitrag
    Storage,
    /// Bandwidth-Beitrag
    Bandwidth,
    /// Compute-Beitrag
    Compute,
    /// Proof-of-Useful-Work
    ProofOfUsefulWork,
    /// DC3-Challenge
    Dc3Challenge,
    /// Uptime-Bonus
    Uptime,
}

impl ScoreCategory {
    /// Gewicht für Aggregation
    pub fn weight(&self) -> f64 {
        match self {
            Self::Storage => 0.20,
            Self::Bandwidth => 0.25,
            Self::Compute => 0.20,
            Self::ProofOfUsefulWork => 0.20,
            Self::Dc3Challenge => 0.10,
            Self::Uptime => 0.05,
        }
    }

    /// Passender Decay-Calculator
    pub fn decay_calculator(&self) -> ExponentialDecayCalculator {
        match self {
            Self::Storage => ExponentialDecayCalculator::for_storage(),
            Self::Bandwidth => ExponentialDecayCalculator::for_bandwidth(),
            Self::Compute | Self::ProofOfUsefulWork | Self::Dc3Challenge => {
                ExponentialDecayCalculator::for_compute()
            }
            Self::Uptime => ExponentialDecayCalculator::new(60.0), // Längerer Decay
        }
    }
}

// ============================================================================
// CUMULATIVE CONTRIBUTION SCORE
// ============================================================================

/// Kumulativer Contribution-Score (ersetzt Token-Stake)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CumulativeContributionScore {
    /// Score-Historie
    entries: VecDeque<ScoreEntry>,
    /// Cached aggregierte Scores pro Kategorie
    category_scores: [f64; 6],
    /// Letzter Update-Timestamp
    last_update: u64,
    /// Gesamt-Score (cached)
    cached_total: f64,
    /// Cache-Valid-Until
    cache_valid_until: u64,
}

impl CumulativeContributionScore {
    /// Erstelle neuen Score
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            entries: VecDeque::new(),
            category_scores: [0.0; 6],
            last_update: now,
            cached_total: 0.0,
            cache_valid_until: now,
        }
    }

    /// Füge Score-Eintrag hinzu
    pub fn add_entry(&mut self, entry: ScoreEntry) {
        // Limit History
        while self.entries.len() >= MAX_SCORE_HISTORY {
            self.entries.pop_front();
        }

        self.entries.push_back(entry);
        self.invalidate_cache();
    }

    /// Füge Score für Kategorie hinzu
    pub fn add_score(&mut self, category: ScoreCategory, score: f64, description: Option<String>) {
        if score >= MIN_CONTRIBUTION_SCORE {
            let entry = ScoreEntry::new(score, category, description);
            self.add_entry(entry);
        }
    }

    /// Berechne aktuellen Gesamt-Score
    pub fn total_score(&mut self) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Cache prüfen
        if now < self.cache_valid_until {
            return self.cached_total;
        }

        // Neu berechnen
        self.recalculate();
        self.cached_total
    }

    /// Hole Score für spezifische Kategorie
    pub fn category_score(&mut self, category: ScoreCategory) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now >= self.cache_valid_until {
            self.recalculate();
        }

        self.category_scores[category as usize]
    }

    /// Neuberechnung aller Scores
    fn recalculate(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Reset
        self.category_scores = [0.0; 6];

        // Summiere Scores pro Kategorie mit Decay
        for entry in &self.entries {
            let decay = entry.category.decay_calculator();
            let current = entry.current_score(&decay);

            if current >= MIN_CONTRIBUTION_SCORE {
                self.category_scores[entry.category as usize] += current;
            }
        }

        // Gewichtete Summe
        self.cached_total = self
            .category_scores
            .iter()
            .zip([
                ScoreCategory::Storage,
                ScoreCategory::Bandwidth,
                ScoreCategory::Compute,
                ScoreCategory::ProofOfUsefulWork,
                ScoreCategory::Dc3Challenge,
                ScoreCategory::Uptime,
            ])
            .map(|(&score, cat)| (score.min(1.0)) * cat.weight())
            .sum();

        // Cache für 5 Minuten
        self.cache_valid_until = now + 300;
        self.last_update = now;
    }

    /// Cache invalidieren
    fn invalidate_cache(&mut self) {
        self.cache_valid_until = 0;
    }

    /// Cleanup alte Einträge (Score < MIN_CONTRIBUTION_SCORE)
    pub fn cleanup_expired(&mut self) {
        let min_score = MIN_CONTRIBUTION_SCORE;

        self.entries.retain(|entry| {
            let decay = entry.category.decay_calculator();
            entry.current_score(&decay) >= min_score
        });

        self.invalidate_cache();
    }

    /// Anzahl aktiver Einträge
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Ist Score ausreichend für Apprentice-Eligibility?
    pub fn meets_apprentice_threshold(&mut self) -> bool {
        self.total_score() >= 0.3
    }

    /// Ist Score ausreichend für Full-Relay-Eligibility?
    pub fn meets_full_relay_threshold(&mut self) -> bool {
        self.total_score() >= 0.6
    }

    /// Geschätzte Zeit bis Threshold erreicht (Tage)
    pub fn estimated_days_to_threshold(&mut self, target: f64) -> Option<f64> {
        let current = self.total_score();

        if current >= target {
            return Some(0.0);
        }

        // Berechne durchschnittliche tägliche Score-Zunahme
        let recent_entries: Vec<_> = self
            .entries
            .iter()
            .filter(|e| e.age_days() < 7.0)
            .collect();

        if recent_entries.is_empty() {
            return None;
        }

        let daily_avg: f64 = recent_entries.iter().map(|e| e.base_score).sum::<f64>() / 7.0;

        if daily_avg <= 0.0 {
            return None;
        }

        let deficit = target - current;
        Some(deficit / daily_avg)
    }

    /// Export als Zusammenfassung
    pub fn summary(&mut self) -> ContributionSummary {
        self.recalculate();

        ContributionSummary {
            total_score: self.cached_total,
            storage_score: self.category_scores[ScoreCategory::Storage as usize],
            bandwidth_score: self.category_scores[ScoreCategory::Bandwidth as usize],
            compute_score: self.category_scores[ScoreCategory::Compute as usize],
            pouw_score: self.category_scores[ScoreCategory::ProofOfUsefulWork as usize],
            dc3_score: self.category_scores[ScoreCategory::Dc3Challenge as usize],
            uptime_score: self.category_scores[ScoreCategory::Uptime as usize],
            entry_count: self.entries.len(),
            last_update: self.last_update,
        }
    }
}

impl Default for CumulativeContributionScore {
    fn default() -> Self {
        Self::new()
    }
}

/// Contribution-Zusammenfassung
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionSummary {
    pub total_score: f64,
    pub storage_score: f64,
    pub bandwidth_score: f64,
    pub compute_score: f64,
    pub pouw_score: f64,
    pub dc3_score: f64,
    pub uptime_score: f64,
    pub entry_count: usize,
    pub last_update: u64,
}

// ============================================================================
// SCORE AGGREGATOR (für mehrere Peers)
// ============================================================================

/// Aggregator für Netzwerk-weite Score-Statistiken
#[derive(Debug, Clone, Default)]
pub struct ScoreAggregator {
    /// Scores pro Peer
    scores: std::collections::HashMap<libp2p::PeerId, CumulativeContributionScore>,
}

impl ScoreAggregator {
    /// Erstelle neuen Aggregator
    pub fn new() -> Self {
        Self {
            scores: std::collections::HashMap::new(),
        }
    }

    /// Hole oder erstelle Score für Peer
    pub fn get_or_create(&mut self, peer: libp2p::PeerId) -> &mut CumulativeContributionScore {
        self.scores
            .entry(peer)
            .or_insert_with(CumulativeContributionScore::new)
    }

    /// Hole Score für Peer (nur lesen)
    pub fn get(&self, peer: &libp2p::PeerId) -> Option<&CumulativeContributionScore> {
        self.scores.get(peer)
    }

    /// Anzahl Peers
    pub fn peer_count(&self) -> usize {
        self.scores.len()
    }

    /// Durchschnittlicher Score
    pub fn average_score(&mut self) -> f64 {
        if self.scores.is_empty() {
            return 0.0;
        }

        let total: f64 = self.scores.values_mut().map(|s| s.total_score()).sum();
        total / self.scores.len() as f64
    }

    /// Median-Score
    pub fn median_score(&mut self) -> f64 {
        if self.scores.is_empty() {
            return 0.0;
        }

        let mut scores: Vec<f64> = self.scores.values_mut().map(|s| s.total_score()).collect();
        scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mid = scores.len() / 2;
        if scores.len() % 2 == 0 {
            (scores[mid - 1] + scores[mid]) / 2.0
        } else {
            scores[mid]
        }
    }

    /// Top-N Peers nach Score
    pub fn top_peers(&mut self, n: usize) -> Vec<(libp2p::PeerId, f64)> {
        let mut peer_scores: Vec<_> = self
            .scores
            .iter_mut()
            .map(|(p, s)| (*p, s.total_score()))
            .collect();

        peer_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        peer_scores.into_iter().take(n).collect()
    }

    /// Cleanup alle abgelaufenen Einträge
    pub fn cleanup_all(&mut self) {
        for score in self.scores.values_mut() {
            score.cleanup_expired();
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decay_calculator_creation() {
        let calc = ExponentialDecayCalculator::new(14.0);

        assert_eq!(calc.half_life_days, 14.0);
        assert!((calc.gamma - 0.0495).abs() < 0.001);
    }

    #[test]
    fn test_decay_factor_at_half_life() {
        let calc = ExponentialDecayCalculator::new(14.0);

        let factor = calc.decay_factor(14.0);

        // Nach einer Half-Life sollte der Faktor ~0.5 sein
        assert!((factor - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_decay_factor_at_zero() {
        let calc = ExponentialDecayCalculator::new(14.0);

        let factor = calc.decay_factor(0.0);

        assert!((factor - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_apply_decay() {
        let calc = ExponentialDecayCalculator::new(14.0);

        let value = calc.apply_decay(1.0, 14.0);

        assert!((value - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_score_category_weights() {
        let total: f64 = [
            ScoreCategory::Storage,
            ScoreCategory::Bandwidth,
            ScoreCategory::Compute,
            ScoreCategory::ProofOfUsefulWork,
            ScoreCategory::Dc3Challenge,
            ScoreCategory::Uptime,
        ]
        .iter()
        .map(|c| c.weight())
        .sum();

        assert!((total - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_score_entry_creation() {
        let entry = ScoreEntry::new(0.5, ScoreCategory::Storage, Some("Test".to_string()));

        assert_eq!(entry.base_score, 0.5);
        assert_eq!(entry.category, ScoreCategory::Storage);
        assert!(entry.age_days() < 1.0);
    }

    #[test]
    fn test_cumulative_score_add() {
        let mut score = CumulativeContributionScore::new();

        score.add_score(ScoreCategory::Storage, 0.5, None);
        score.add_score(ScoreCategory::Bandwidth, 0.3, None);

        assert_eq!(score.entry_count(), 2);
    }

    #[test]
    fn test_cumulative_score_total() {
        let mut score = CumulativeContributionScore::new();

        score.add_score(ScoreCategory::Storage, 1.0, None);
        score.add_score(ScoreCategory::Bandwidth, 1.0, None);
        score.add_score(ScoreCategory::Compute, 1.0, None);
        score.add_score(ScoreCategory::ProofOfUsefulWork, 1.0, None);
        score.add_score(ScoreCategory::Dc3Challenge, 1.0, None);
        score.add_score(ScoreCategory::Uptime, 1.0, None);

        let total = score.total_score();

        // Alle Kategorien maxed out sollte ~1.0 ergeben
        assert!((total - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_cumulative_score_thresholds() {
        let mut score = CumulativeContributionScore::new();

        assert!(!score.meets_apprentice_threshold());
        assert!(!score.meets_full_relay_threshold());

        // Füge genug Score hinzu
        score.add_score(ScoreCategory::Storage, 1.0, None);
        score.add_score(ScoreCategory::Bandwidth, 1.0, None);

        assert!(score.meets_apprentice_threshold());
    }

    #[test]
    fn test_cumulative_score_category_specific() {
        let mut score = CumulativeContributionScore::new();

        score.add_score(ScoreCategory::Storage, 0.5, None);
        score.add_score(ScoreCategory::Bandwidth, 0.3, None);

        let storage = score.category_score(ScoreCategory::Storage);
        let bandwidth = score.category_score(ScoreCategory::Bandwidth);

        assert!((storage - 0.5).abs() < 0.01);
        assert!((bandwidth - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_cumulative_score_summary() {
        let mut score = CumulativeContributionScore::new();

        score.add_score(ScoreCategory::Storage, 0.5, None);
        score.add_score(ScoreCategory::Bandwidth, 0.3, None);

        let summary = score.summary();

        assert_eq!(summary.entry_count, 2);
        assert!(summary.storage_score > 0.0);
        assert!(summary.bandwidth_score > 0.0);
    }

    #[test]
    fn test_score_aggregator() {
        let mut aggregator = ScoreAggregator::new();
        let peer1 = libp2p::PeerId::random();
        let peer2 = libp2p::PeerId::random();

        aggregator
            .get_or_create(peer1)
            .add_score(ScoreCategory::Storage, 1.0, None);
        aggregator
            .get_or_create(peer2)
            .add_score(ScoreCategory::Storage, 0.5, None);

        assert_eq!(aggregator.peer_count(), 2);
        assert!(aggregator.average_score() > 0.0);
    }

    #[test]
    fn test_score_aggregator_top_peers() {
        let mut aggregator = ScoreAggregator::new();

        for i in 0..5 {
            let peer = libp2p::PeerId::random();
            aggregator
                .get_or_create(peer)
                .add_score(ScoreCategory::Storage, (i as f64 + 1.0) * 0.2, None);
        }

        let top = aggregator.top_peers(3);

        assert_eq!(top.len(), 3);
        // Top sollte höchsten Score haben
        assert!(top[0].1 >= top[1].1);
        assert!(top[1].1 >= top[2].1);
    }

    #[test]
    fn test_minimum_score_filter() {
        let mut score = CumulativeContributionScore::new();

        // Score unter Minimum
        score.add_score(ScoreCategory::Storage, 0.0001, None);

        assert_eq!(score.entry_count(), 0);

        // Score über Minimum
        score.add_score(ScoreCategory::Storage, 0.01, None);

        assert_eq!(score.entry_count(), 1);
    }

    #[test]
    fn test_decay_calculator_presets() {
        let storage = ExponentialDecayCalculator::for_storage();
        let bandwidth = ExponentialDecayCalculator::for_bandwidth();
        let compute = ExponentialDecayCalculator::for_compute();

        assert_eq!(storage.half_life_days, STORAGE_DECAY_HALF_LIFE_DAYS);
        assert_eq!(bandwidth.half_life_days, BANDWIDTH_DECAY_HALF_LIFE_DAYS);
        assert_eq!(compute.half_life_days, COMPUTE_DECAY_HALF_LIFE_DAYS);
    }
}
