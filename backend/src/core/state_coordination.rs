//! # State Coordination Layer
//!
//! Koordiniert State-Updates über mehrere Module hinweg.
//! Implementiert Transaktionen, Rollbacks und konsistente Snapshots.
//!
//! ## Features
//!
//! - **Atomic Multi-Module Updates**: State-Änderungen als Transaktion
//! - **Consistency Checks**: Invarianten-Prüfung nach Updates
//! - **Snapshot Isolation**: Konsistente Reads während Updates
//! - **Health Aggregation**: System-weite Health-Berechnung

use super::state::{SharedUnifiedState, UnifiedSnapshot};
use super::state_integration::StateIntegrator;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

// ============================================================================
// INVARIANT DEFINITIONS
// ============================================================================

/// System-Invarianten die geprüft werden
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Invariant {
    /// Trust-Asymmetry zwischen 1.5 und 3.0 (Κ4)
    TrustAsymmetry,
    /// Event-Finality < 5 Sekunden
    EventFinality,
    /// Consensus Success Rate > 90%
    ConsensusSuccessRate,
    /// Execution Success Rate > 95%
    ExecutionSuccessRate,
    /// Protection Health > 70%
    ProtectionHealth,
    /// Storage Growth Rate < 10MB/min
    StorageGrowthRate,
    /// Monokultur-Entropie > 0.7
    DiversityEntropy,
    /// World Formula 𝔼 > 0
    WorldFormulaPositive,
}

impl Invariant {
    /// Alle Invarianten
    pub fn all() -> &'static [Invariant] {
        &[
            Invariant::TrustAsymmetry,
            Invariant::EventFinality,
            Invariant::ConsensusSuccessRate,
            Invariant::ExecutionSuccessRate,
            Invariant::ProtectionHealth,
            Invariant::StorageGrowthRate,
            Invariant::DiversityEntropy,
            Invariant::WorldFormulaPositive,
        ]
    }

    /// Severity der Invariant-Verletzung
    pub fn severity(&self) -> InvariantSeverity {
        match self {
            Invariant::TrustAsymmetry => InvariantSeverity::Warning,
            Invariant::EventFinality => InvariantSeverity::Warning,
            Invariant::ConsensusSuccessRate => InvariantSeverity::Critical,
            Invariant::ExecutionSuccessRate => InvariantSeverity::Error,
            Invariant::ProtectionHealth => InvariantSeverity::Critical,
            Invariant::StorageGrowthRate => InvariantSeverity::Warning,
            Invariant::DiversityEntropy => InvariantSeverity::Warning,
            Invariant::WorldFormulaPositive => InvariantSeverity::Error,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum InvariantSeverity {
    Warning,
    Error,
    Critical,
}

/// Ergebnis einer Invariant-Prüfung
#[derive(Debug, Clone, Serialize)]
pub struct InvariantResult {
    pub invariant: Invariant,
    pub passed: bool,
    pub current_value: f64,
    pub threshold: f64,
    pub message: String,
}

// ============================================================================
// STATE COORDINATOR
// ============================================================================

/// State Coordinator - Koordiniert Cross-Module State-Updates
pub struct StateCoordinator {
    state: SharedUnifiedState,
    integrator: StateIntegrator,

    /// Snapshot-History für Trend-Analyse
    snapshot_history: std::sync::RwLock<Vec<(Instant, UnifiedSnapshot)>>,

    /// Letzte bekannte Storage-Size für Growth-Rate
    last_storage_check: std::sync::RwLock<(Instant, u64)>,
}

impl StateCoordinator {
    /// Erstelle neuen StateCoordinator
    pub fn new(state: SharedUnifiedState) -> Self {
        let integrator = StateIntegrator::new(state.clone());
        Self {
            state,
            integrator,
            snapshot_history: std::sync::RwLock::new(Vec::new()),
            last_storage_check: std::sync::RwLock::new((Instant::now(), 0)),
        }
    }

    /// Zugriff auf State
    pub fn state(&self) -> &SharedUnifiedState {
        &self.state
    }

    /// Zugriff auf Integrator
    pub fn integrator(&self) -> &StateIntegrator {
        &self.integrator
    }

    /// Erstelle Snapshot und speichere in History
    pub fn snapshot(&self) -> UnifiedSnapshot {
        let snapshot = self.state.snapshot();

        // In History speichern
        if let Ok(mut history) = self.snapshot_history.write() {
            history.push((Instant::now(), snapshot.clone()));
            // Max 1000 Snapshots (ca. 16 Minuten bei 1/s)
            if history.len() > 1000 {
                history.remove(0);
            }
        }

        snapshot
    }

    /// Prüfe alle Invarianten
    pub fn check_invariants(&self) -> Vec<InvariantResult> {
        let snapshot = self.state.snapshot();
        Invariant::all()
            .iter()
            .map(|inv| self.check_invariant(*inv, &snapshot))
            .collect()
    }

    /// Prüfe einzelne Invariante
    pub fn check_invariant(
        &self,
        invariant: Invariant,
        snapshot: &UnifiedSnapshot,
    ) -> InvariantResult {
        match invariant {
            Invariant::TrustAsymmetry => {
                let ratio = snapshot.core.trust.asymmetry_ratio;
                InvariantResult {
                    invariant,
                    passed: (1.5..=3.0).contains(&ratio),
                    current_value: ratio,
                    threshold: 2.0, // Expected ~2:1
                    message: format!("Trust asymmetry ratio: {:.2}", ratio),
                }
            }
            Invariant::EventFinality => {
                let latency = snapshot.core.events.avg_finality_latency_ms;
                let threshold = 5000.0;
                InvariantResult {
                    invariant,
                    passed: latency < threshold,
                    current_value: latency,
                    threshold,
                    message: format!("Avg finality latency: {:.0}ms", latency),
                }
            }
            Invariant::ConsensusSuccessRate => {
                let rate = snapshot.core.consensus.success_rate;
                let threshold = 0.9;
                InvariantResult {
                    invariant,
                    passed: rate >= threshold,
                    current_value: rate,
                    threshold,
                    message: format!("Consensus success rate: {:.1}%", rate * 100.0),
                }
            }
            Invariant::ExecutionSuccessRate => {
                let rate = snapshot.execution.executions.success_rate;
                let threshold = 0.95;
                InvariantResult {
                    invariant,
                    passed: rate >= threshold,
                    current_value: rate,
                    threshold,
                    message: format!("Execution success rate: {:.1}%", rate * 100.0),
                }
            }
            Invariant::ProtectionHealth => {
                let health = snapshot.protection.health_score;
                let threshold = 70.0;
                InvariantResult {
                    invariant,
                    passed: health >= threshold,
                    current_value: health,
                    threshold,
                    message: format!("Protection health score: {:.1}", health),
                }
            }
            Invariant::StorageGrowthRate => {
                let growth_rate = self.calculate_storage_growth_rate(&snapshot);
                let threshold = 10.0 * 1024.0 * 1024.0; // 10 MB/min
                InvariantResult {
                    invariant,
                    passed: growth_rate < threshold,
                    current_value: growth_rate,
                    threshold,
                    message: format!(
                        "Storage growth: {:.2} MB/min",
                        growth_rate / 1024.0 / 1024.0
                    ),
                }
            }
            Invariant::DiversityEntropy => {
                let entropy = snapshot.protection.diversity.min_entropy;
                let threshold = 0.7;
                InvariantResult {
                    invariant,
                    passed: entropy >= threshold,
                    current_value: entropy,
                    threshold,
                    message: format!("Min diversity entropy: {:.2}", entropy),
                }
            }
            Invariant::WorldFormulaPositive => {
                let e = snapshot.core.formula.current_e;
                InvariantResult {
                    invariant,
                    passed: e > 0.0,
                    current_value: e,
                    threshold: 0.0,
                    message: format!("World formula 𝔼: {:.4}", e),
                }
            }
        }
    }

    /// Berechne Storage Growth Rate (Bytes/Minute)
    fn calculate_storage_growth_rate(&self, snapshot: &UnifiedSnapshot) -> f64 {
        let current_bytes = snapshot.storage.total_bytes;
        let now = Instant::now();

        if let Ok(mut last) = self.last_storage_check.write() {
            let (last_time, last_bytes) = *last;
            let elapsed_mins = last_time.elapsed().as_secs_f64() / 60.0;

            // Update für nächsten Check
            *last = (now, current_bytes);

            if elapsed_mins > 0.0 {
                let diff = current_bytes.saturating_sub(last_bytes) as f64;
                return diff / elapsed_mins;
            }
        }

        0.0
    }

    /// Berechne Trend für eine Metrik (positiv = steigend)
    pub fn calculate_trend(&self, metric: &str) -> f64 {
        let history = match self.snapshot_history.read() {
            Ok(h) => h.clone(),
            Err(_) => return 0.0,
        };

        if history.len() < 10 {
            return 0.0;
        }

        // Letzte 10 vs vorherige 10
        let recent: Vec<f64> = history
            .iter()
            .rev()
            .take(10)
            .map(|(_, s)| self.extract_metric(s, metric))
            .collect();

        let older: Vec<f64> = history
            .iter()
            .rev()
            .skip(10)
            .take(10)
            .map(|(_, s)| self.extract_metric(s, metric))
            .collect();

        if recent.is_empty() || older.is_empty() {
            return 0.0;
        }

        let recent_avg: f64 = recent.iter().sum::<f64>() / recent.len() as f64;
        let older_avg: f64 = older.iter().sum::<f64>() / older.len() as f64;

        recent_avg - older_avg
    }

    fn extract_metric(&self, snapshot: &UnifiedSnapshot, metric: &str) -> f64 {
        match metric {
            "trust_updates" => snapshot.core.trust.updates_total as f64,
            "events_total" => snapshot.core.events.total as f64,
            "executions" => snapshot.execution.executions.total as f64,
            "health" => snapshot.health_score,
            "world_formula" => snapshot.core.formula.current_e,
            _ => 0.0,
        }
    }

    /// Aggregiere Health Score mit Gewichtung
    pub fn aggregate_health(&self) -> HealthReport {
        let snapshot = self.state.snapshot();
        let invariant_results = self.check_invariants();

        // Berechne gewichteten Score
        let mut total_weight = 0.0;
        let mut weighted_score = 0.0;

        for result in &invariant_results {
            let weight = match result.invariant.severity() {
                InvariantSeverity::Critical => 3.0,
                InvariantSeverity::Error => 2.0,
                InvariantSeverity::Warning => 1.0,
            };
            total_weight += weight;
            if result.passed {
                weighted_score += weight * 100.0;
            }
        }

        let invariant_score = if total_weight > 0.0 {
            weighted_score / total_weight
        } else {
            100.0
        };

        // Kombiniere mit Module-Health
        let module_scores = vec![
            (
                "core_trust",
                100.0 - (snapshot.core.trust.violations_count as f64).min(100.0),
            ),
            (
                "core_events",
                100.0 - (snapshot.core.events.validation_errors as f64 / 10.0).min(100.0),
            ),
            (
                "core_consensus",
                snapshot.core.consensus.success_rate * 100.0,
            ),
            (
                "execution",
                snapshot.execution.executions.success_rate * 100.0,
            ),
            ("protection", snapshot.protection.health_score),
        ];

        let module_avg: f64 =
            module_scores.iter().map(|(_, s)| s).sum::<f64>() / module_scores.len() as f64;

        // Finaler Score: 60% Invarianten, 40% Module
        let final_score = (invariant_score * 0.6 + module_avg * 0.4)
            .min(100.0)
            .max(0.0);

        // Status bestimmen
        let status = if final_score >= 90.0 {
            HealthStatus::Healthy
        } else if final_score >= 70.0 {
            HealthStatus::Degraded
        } else if final_score >= 50.0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Critical
        };

        HealthReport {
            overall_score: final_score,
            status,
            invariant_score,
            module_avg,
            module_scores: module_scores
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            invariant_results,
            warnings: snapshot.warnings.clone(),
            timestamp_ms: snapshot.timestamp_ms,
        }
    }
}

// ============================================================================
// HEALTH REPORT
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Warning,
    Critical,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "✓ Healthy"),
            HealthStatus::Degraded => write!(f, "◐ Degraded"),
            HealthStatus::Warning => write!(f, "⚠ Warning"),
            HealthStatus::Critical => write!(f, "✗ Critical"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthReport {
    pub overall_score: f64,
    pub status: HealthStatus,
    pub invariant_score: f64,
    pub module_avg: f64,
    pub module_scores: HashMap<String, f64>,
    pub invariant_results: Vec<InvariantResult>,
    pub warnings: Vec<String>,
    pub timestamp_ms: u64,
}

impl HealthReport {
    /// Formatiere als Text-Report
    pub fn format(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!(
            "╔═══════════════════════════════════════════════════════════╗\n"
        ));
        output.push_str(&format!(
            "║              ERYNOA HEALTH REPORT                         ║\n"
        ));
        output.push_str(&format!(
            "╠═══════════════════════════════════════════════════════════╣\n"
        ));
        output.push_str(&format!(
            "║ Status: {:<20} Score: {:.1}/100           ║\n",
            self.status, self.overall_score
        ));
        output.push_str(&format!(
            "╠═══════════════════════════════════════════════════════════╣\n"
        ));

        // Module Scores
        output.push_str("║ MODULE SCORES                                             ║\n");
        for (module, score) in &self.module_scores {
            let indicator = if *score >= 90.0 {
                "✓"
            } else if *score >= 70.0 {
                "◐"
            } else {
                "✗"
            };
            output.push_str(&format!(
                "║   {} {:<20} {:>6.1}                        ║\n",
                indicator, module, score
            ));
        }

        output.push_str(&format!(
            "╠═══════════════════════════════════════════════════════════╣\n"
        ));

        // Invariants
        output.push_str("║ INVARIANTS                                                ║\n");
        for result in &self.invariant_results {
            let indicator = if result.passed { "✓" } else { "✗" };
            output.push_str(&format!(
                "║   {} {:?}: {:<35} ║\n",
                indicator, result.invariant, result.message
            ));
        }

        if !self.warnings.is_empty() {
            output.push_str(&format!(
                "╠═══════════════════════════════════════════════════════════╣\n"
            ));
            output.push_str("║ WARNINGS                                                  ║\n");
            for warning in &self.warnings {
                output.push_str(&format!("║   ⚠ {:<50} ║\n", warning));
            }
        }

        output.push_str(&format!(
            "╚═══════════════════════════════════════════════════════════╝\n"
        ));
        output
    }
}

// ============================================================================
// STATE TRANSACTION
// ============================================================================

/// Typ-alias für State-Änderungen
pub type StateChange = Box<dyn FnOnce(&SharedUnifiedState) + Send + Sync>;

/// State-Transaktion für atomare Multi-Module-Updates
pub struct StateTransaction {
    coordinator: Arc<StateCoordinator>,
    changes: Vec<StateChange>,
    rollback: Option<Box<dyn Fn() + Send + Sync>>,
}

impl StateTransaction {
    /// Erstelle neue Transaktion
    pub fn new(coordinator: Arc<StateCoordinator>) -> Self {
        Self {
            coordinator,
            changes: Vec::new(),
            rollback: None,
        }
    }

    /// Füge Änderung hinzu
    pub fn add_change<F: FnOnce(&SharedUnifiedState) + Send + Sync + 'static>(
        &mut self,
        change: F,
    ) {
        self.changes.push(Box::new(change));
    }

    /// Setze Rollback-Handler
    pub fn set_rollback<F: Fn() + Send + Sync + 'static>(&mut self, rollback: F) {
        self.rollback = Some(Box::new(rollback));
    }

    /// Führe Transaktion aus
    pub fn execute(self) -> Result<(), TransactionError> {
        // Pre-Check: Invarianten prüfen
        let pre_results = self.coordinator.check_invariants();
        let pre_critical_violations: Vec<_> = pre_results
            .iter()
            .filter(|r| !r.passed && r.invariant.severity() == InvariantSeverity::Critical)
            .collect();

        if !pre_critical_violations.is_empty() {
            return Err(TransactionError::PreCheckFailed(
                pre_critical_violations
                    .iter()
                    .map(|r| r.message.clone())
                    .collect(),
            ));
        }

        // Execute changes
        let state = self.coordinator.state();
        for change in self.changes {
            change(state);
        }

        // Post-Check: Invarianten erneut prüfen
        let post_results = self.coordinator.check_invariants();
        let post_critical_violations: Vec<_> = post_results
            .iter()
            .filter(|r| !r.passed && r.invariant.severity() == InvariantSeverity::Critical)
            .collect();

        if !post_critical_violations.is_empty() {
            // Rollback
            if let Some(rollback) = self.rollback {
                rollback();
            }
            return Err(TransactionError::PostCheckFailed(
                post_critical_violations
                    .iter()
                    .map(|r| r.message.clone())
                    .collect(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum TransactionError {
    PreCheckFailed(Vec<String>),
    PostCheckFailed(Vec<String>),
    ExecutionFailed(String),
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::PreCheckFailed(msgs) => {
                write!(f, "Pre-check failed: {}", msgs.join(", "))
            }
            TransactionError::PostCheckFailed(msgs) => {
                write!(f, "Post-check failed (rolled back): {}", msgs.join(", "))
            }
            TransactionError::ExecutionFailed(msg) => {
                write!(f, "Execution failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for TransactionError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::create_unified_state;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_invariant_check() {
        let state = create_unified_state();
        let coordinator = StateCoordinator::new(state);

        let results = coordinator.check_invariants();
        assert!(!results.is_empty());

        // Anfangs sollten die meisten bestehen
        let passed = results.iter().filter(|r| r.passed).count();
        assert!(passed > 0);
    }

    #[test]
    fn test_health_report() {
        let state = create_unified_state();
        let coordinator = StateCoordinator::new(state);

        let report = coordinator.aggregate_health();
        assert!(report.overall_score >= 0.0 && report.overall_score <= 100.0);
        assert!(!report.module_scores.is_empty());
    }

    #[test]
    fn test_transaction() {
        let state = create_unified_state();
        let coordinator = Arc::new(StateCoordinator::new(state.clone()));

        let mut tx = StateTransaction::new(coordinator.clone());
        tx.add_change(|s| {
            s.core.trust.entities_count.fetch_add(1, Ordering::Relaxed);
        });

        let result = tx.execute();
        assert!(result.is_ok());
        assert_eq!(state.core.trust.entities_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_snapshot_history() {
        let state = create_unified_state();
        let coordinator = StateCoordinator::new(state);

        // Erstelle mehrere Snapshots
        for _ in 0..5 {
            coordinator.snapshot();
        }

        let trend = coordinator.calculate_trend("health");
        // Trend sollte ~0 sein da keine Änderungen
        assert!(trend.abs() < 1.0);
    }
}
