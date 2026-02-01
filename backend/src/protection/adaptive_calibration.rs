//! # Adaptive Kalibrierung der Weltformel
//!
//! Dynamische Parameteranpassung basierend auf Netzwerk-Metriken (Κ19, §IX).
//!
//! ## Axiom-Referenz
//!
//! - **Κ19 (Macht-Verdünnung)**: Adaptive Power-Caps basierend auf Netzwerk-Zustand
//! - **§IX.2 (Emergenz)**: Selbst-organisierende Parameter-Korrektur
//!
//! ## Funktionsweise
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                    ADAPTIVE KALIBRIERUNG                                     │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │  NetworkMetrics ──────► CalibrationEngine ──────► AdjustedParameters        │
//! │        │                       │                          │                 │
//! │        │                       │                          │                 │
//! │        ▼                       ▼                          ▼                 │
//! │  ┌──────────┐          ┌──────────────┐          ┌──────────────┐          │
//! │  │ Gini     │          │ PID Controller│         │ entity_exp   │          │
//! │  │ Churn    │    ─►    │ EMA Filter    │   ─►    │ decay_rate   │          │
//! │  │ Sybil    │          │ Boundary Check│         │ alarm_params │          │
//! │  │ Latency  │          └──────────────┘          └──────────────┘          │
//! │  └──────────┘                                                               │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Parameter-Anpassung
//!
//! Die Kalibrierung erfolgt auf Basis von:
//!
//! 1. **Gini-Koeffizient**: Ungleichheit → Entity-Exponent erhöhen
//! 2. **Churn-Rate**: Hoher Churn → Decay-Rate senken (Stabilität)
//! 3. **Sybil-Score**: Hohe Sybil-Wahrscheinlichkeit → Alarm-Schwellen senken
//! 4. **Network-Latency**: Hohe Latenz → Tolerantere Parameter

use crate::protection::AntiCalcificationConfig;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Netzwerk-Metriken für adaptive Kalibrierung
#[derive(Debug, Clone, Default)]
pub struct NetworkMetrics {
    /// Gini-Koeffizient der Power-Verteilung (0.0 = perfekte Gleichheit, 1.0 = maximale Ungleichheit)
    pub gini_coefficient: f64,

    /// Churn-Rate: Anteil der Nodes die in den letzten 24h joined/left haben
    pub churn_rate_24h: f64,

    /// Sybil-Score: Geschätzter Anteil verdächtiger Nodes (0.0-1.0)
    pub estimated_sybil_ratio: f64,

    /// Durchschnittliche Netzwerk-Latenz in Millisekunden
    pub avg_latency_ms: f64,

    /// Anzahl aktiver Nodes
    pub active_node_count: u64,

    /// Totale Power im Netzwerk
    pub total_power: f64,

    /// Top-3% Power-Konzentration
    pub top_concentration: f64,

    /// Trust-Entropie (höher = diversere Trust-Beziehungen)
    pub trust_entropy: f64,
}

/// Kalibrierte Parameter-Adjustments
#[derive(Debug, Clone)]
pub struct CalibratedParameters {
    /// Angepasster Entity-Exponent
    pub entity_exponent: f64,

    /// Angepasste Decay-Rate
    pub decay_rate_per_day: f64,

    /// Angepasste Alarm Top-Percentage
    pub alarm_top_percentage: f64,

    /// Angepasste Alarm Power-Threshold
    pub alarm_power_threshold: f64,

    /// Zeitstempel der Kalibrierung
    pub calibrated_at: Instant,

    /// Confidence-Score der Kalibrierung (0.0-1.0)
    pub confidence: f64,
}

/// Kalibrierungs-Engine für adaptive Parameter-Anpassung
///
/// Verwendet einen PID-ähnlichen Controller mit EMA-Glättung,
/// um Oszillationen zu vermeiden.
pub struct CalibrationEngine {
    /// Basis-Konfiguration (Simulation-Optima)
    base_config: AntiCalcificationConfig,

    /// Kalibrierungs-Konfiguration
    config: CalibrationConfig,

    /// Historische Metriken (für EMA)
    metrics_history: VecDeque<NetworkMetrics>,

    /// Letzte kalibrierte Parameter
    last_calibration: Option<CalibratedParameters>,

    /// PID-Controller State
    pid_state: PidState,
}

/// Konfiguration für die Kalibrierungs-Engine
#[derive(Debug, Clone)]
pub struct CalibrationConfig {
    /// EMA-Alpha für Metrik-Glättung (0.0-1.0, höher = weniger Glättung)
    pub ema_alpha: f64,

    /// Minimale Zeit zwischen Kalibrierungen
    pub min_calibration_interval: Duration,

    /// Maximale Metrik-History-Größe
    pub max_history_size: usize,

    /// Parameter-Grenzen
    pub bounds: ParameterBounds,

    /// Mindest-Confidence für Kalibrierung
    pub min_confidence: f64,

    /// Mindest-Nodes für adaptive Kalibrierung
    pub min_nodes_for_adaptation: u64,
}

impl Default for CalibrationConfig {
    fn default() -> Self {
        Self {
            ema_alpha: 0.2,
            min_calibration_interval: Duration::from_secs(3600), // 1 Stunde
            max_history_size: 168,                               // 1 Woche bei stündlicher Messung
            bounds: ParameterBounds::default(),
            min_confidence: 0.3,
            min_nodes_for_adaptation: 100,
        }
    }
}

/// Parameter-Grenzen für sichere Kalibrierung
#[derive(Debug, Clone)]
pub struct ParameterBounds {
    /// Entity-Exponent Grenzen
    pub entity_exponent: (f64, f64),

    /// Decay-Rate Grenzen
    pub decay_rate: (f64, f64),

    /// Alarm Top-Percentage Grenzen
    pub alarm_top_pct: (f64, f64),

    /// Alarm Power-Threshold Grenzen
    pub alarm_power_thresh: (f64, f64),
}

impl Default for ParameterBounds {
    fn default() -> Self {
        Self {
            // Aus Simulation: sicherer Bereich
            entity_exponent: (0.15, 0.35),    // ±0.10 vom Optimum 0.25
            decay_rate: (0.002, 0.015),       // ±0.009 vom Optimum 0.006
            alarm_top_pct: (0.01, 0.15),      // Erweitert für verschiedene Netzgrößen
            alarm_power_thresh: (0.30, 0.60), // ±0.12 vom Optimum 0.42
        }
    }
}

/// PID-Controller State für stabile Anpassung
#[derive(Debug, Clone, Default)]
struct PidState {
    /// Letzter Gini-Error
    last_gini_error: f64,

    /// Integrierter Gini-Error
    integral_gini_error: f64,

    /// Letzter Churn-Error
    last_churn_error: f64,

    /// Integrierter Churn-Error
    integral_churn_error: f64,
}

impl CalibrationEngine {
    /// Erstelle neue Kalibrierungs-Engine
    pub fn new(base_config: AntiCalcificationConfig, config: CalibrationConfig) -> Self {
        Self {
            base_config,
            config,
            metrics_history: VecDeque::with_capacity(168),
            last_calibration: None,
            pid_state: PidState::default(),
        }
    }

    /// Erstelle mit Default-Config
    pub fn with_defaults() -> Self {
        Self::new(
            AntiCalcificationConfig::default(),
            CalibrationConfig::default(),
        )
    }

    /// Füge neue Metriken hinzu
    pub fn record_metrics(&mut self, metrics: NetworkMetrics) {
        self.metrics_history.push_back(metrics);

        // History-Größe begrenzen
        while self.metrics_history.len() > self.config.max_history_size {
            self.metrics_history.pop_front();
        }
    }

    /// Berechne EMA-geglättete Metriken
    fn smoothed_metrics(&self) -> Option<NetworkMetrics> {
        if self.metrics_history.is_empty() {
            return None;
        }

        let alpha = self.config.ema_alpha;
        let mut ema = self.metrics_history.front()?.clone();

        for m in self.metrics_history.iter().skip(1) {
            ema.gini_coefficient =
                alpha * m.gini_coefficient + (1.0 - alpha) * ema.gini_coefficient;
            ema.churn_rate_24h = alpha * m.churn_rate_24h + (1.0 - alpha) * ema.churn_rate_24h;
            ema.estimated_sybil_ratio =
                alpha * m.estimated_sybil_ratio + (1.0 - alpha) * ema.estimated_sybil_ratio;
            ema.avg_latency_ms = alpha * m.avg_latency_ms + (1.0 - alpha) * ema.avg_latency_ms;
            ema.active_node_count = m.active_node_count; // Aktuellster Wert
            ema.total_power = m.total_power;
            ema.top_concentration =
                alpha * m.top_concentration + (1.0 - alpha) * ema.top_concentration;
            ema.trust_entropy = alpha * m.trust_entropy + (1.0 - alpha) * ema.trust_entropy;
        }

        Some(ema)
    }

    /// Kalibriere Parameter basierend auf aktuellen Metriken
    ///
    /// Gibt `None` zurück wenn:
    /// - Nicht genügend History
    /// - Zu wenig Zeit seit letzter Kalibrierung
    /// - Confidence zu niedrig
    pub fn calibrate(&mut self) -> Option<CalibratedParameters> {
        // Prüfe Kalibrierungs-Intervall
        if let Some(ref last) = self.last_calibration {
            if last.calibrated_at.elapsed() < self.config.min_calibration_interval {
                return None;
            }
        }

        // Hole geglättete Metriken
        let metrics = self.smoothed_metrics()?;

        // Prüfe Mindest-Nodes
        if metrics.active_node_count < self.config.min_nodes_for_adaptation {
            return None;
        }

        // Berechne Confidence basierend auf History-Größe und Varianz
        let confidence = self.calculate_confidence();
        if confidence < self.config.min_confidence {
            return None;
        }

        // Berechne angepasste Parameter
        let params = self.compute_adjusted_parameters(&metrics);

        self.last_calibration = Some(params.clone());
        Some(params)
    }

    /// Berechne angepasste Parameter
    fn compute_adjusted_parameters(&mut self, metrics: &NetworkMetrics) -> CalibratedParameters {
        let bounds = &self.config.bounds;

        // === Entity Exponent Anpassung ===
        // Hoher Gini → höherer Exponent (strengere Power-Caps)
        let gini_target = 0.35; // Ziel-Gini für gesundes Netzwerk
        let gini_error = metrics.gini_coefficient - gini_target;

        // PID für Gini
        let gini_p = 0.15 * gini_error;
        self.pid_state.integral_gini_error += gini_error * 0.01;
        self.pid_state.integral_gini_error = self.pid_state.integral_gini_error.clamp(-0.1, 0.1); // Anti-Windup
        let gini_i = 0.05 * self.pid_state.integral_gini_error;
        let gini_d = 0.02 * (gini_error - self.pid_state.last_gini_error);
        self.pid_state.last_gini_error = gini_error;

        let entity_exponent_adjust = gini_p + gini_i + gini_d;
        let entity_exponent = (self.base_config.entity_exponent + entity_exponent_adjust)
            .clamp(bounds.entity_exponent.0, bounds.entity_exponent.1);

        // === Decay Rate Anpassung ===
        // Hoher Churn → niedrigere Decay-Rate (mehr Stabilität)
        // Niedriger Trust-Entropy → höhere Decay (Ossifikation aufbrechen)
        let churn_target = 0.05; // 5% Churn ist normal
        let churn_error = metrics.churn_rate_24h - churn_target;

        // PID für Churn
        let churn_p = -0.02 * churn_error; // Negativ: hoher Churn → niedrigerer Decay
        self.pid_state.integral_churn_error += churn_error * 0.005;
        self.pid_state.integral_churn_error =
            self.pid_state.integral_churn_error.clamp(-0.05, 0.05);
        let churn_i = -0.01 * self.pid_state.integral_churn_error;
        let churn_d = -0.005 * (churn_error - self.pid_state.last_churn_error);
        self.pid_state.last_churn_error = churn_error;

        // Trust-Entropy Faktor (niedrige Entropie → mehr Decay nötig)
        let entropy_factor = if metrics.trust_entropy < 2.0 {
            1.0 + (2.0 - metrics.trust_entropy) * 0.3
        } else {
            1.0
        };

        let decay_adjust = (churn_p + churn_i + churn_d) * entropy_factor;
        let decay_rate_per_day = (self.base_config.decay_rate_per_day + decay_adjust)
            .clamp(bounds.decay_rate.0, bounds.decay_rate.1);

        // === Alarm Parameters Anpassung ===
        // Hoher Sybil-Score → strengere Alarms
        // Große Netzwerke → niedrigere Top-Percentage
        let sybil_factor = 1.0 - metrics.estimated_sybil_ratio * 0.3;
        let size_factor = if metrics.active_node_count > 1000 {
            0.8 // Große Netzwerke: engere Überwachung
        } else if metrics.active_node_count > 500 {
            0.9
        } else {
            1.0
        };

        let alarm_top_percentage = (self.base_config.alarm_top_percentage * size_factor)
            .clamp(bounds.alarm_top_pct.0, bounds.alarm_top_pct.1);

        let alarm_power_threshold = (self.base_config.alarm_power_threshold * sybil_factor)
            .clamp(bounds.alarm_power_thresh.0, bounds.alarm_power_thresh.1);

        CalibratedParameters {
            entity_exponent,
            decay_rate_per_day,
            alarm_top_percentage,
            alarm_power_threshold,
            calibrated_at: Instant::now(),
            confidence: self.calculate_confidence(),
        }
    }

    /// Berechne Confidence-Score
    fn calculate_confidence(&self) -> f64 {
        if self.metrics_history.is_empty() {
            return 0.0;
        }

        // Basis-Confidence aus History-Größe
        let history_factor =
            (self.metrics_history.len() as f64 / self.config.max_history_size as f64).min(1.0);

        // Varianz-Faktor (niedrige Varianz = höhere Confidence)
        let variance = self.calculate_gini_variance();
        let variance_factor = (1.0 - variance * 2.0).max(0.0);

        // Kombinierte Confidence
        (history_factor * 0.6 + variance_factor * 0.4).clamp(0.0, 1.0)
    }

    /// Berechne Gini-Varianz über History
    fn calculate_gini_variance(&self) -> f64 {
        if self.metrics_history.len() < 2 {
            return 1.0;
        }

        let values: Vec<f64> = self
            .metrics_history
            .iter()
            .map(|m| m.gini_coefficient)
            .collect();

        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
        let variance: f64 =
            values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;

        variance.sqrt()
    }

    /// Hole aktuelle kalibrierte Parameter (oder Basis-Config falls keine Kalibrierung)
    pub fn current_parameters(&self) -> CalibratedParameters {
        self.last_calibration
            .clone()
            .unwrap_or_else(|| CalibratedParameters {
                entity_exponent: self.base_config.entity_exponent,
                decay_rate_per_day: self.base_config.decay_rate_per_day,
                alarm_top_percentage: self.base_config.alarm_top_percentage,
                alarm_power_threshold: self.base_config.alarm_power_threshold,
                calibrated_at: Instant::now(),
                confidence: 0.0,
            })
    }

    /// Konvertiere zu AntiCalcificationConfig
    pub fn to_anti_calcification_config(&self) -> AntiCalcificationConfig {
        let params = self.current_parameters();
        AntiCalcificationConfig {
            entity_exponent: params.entity_exponent,
            decay_rate_per_day: params.decay_rate_per_day,
            alarm_top_percentage: params.alarm_top_percentage,
            alarm_power_threshold: params.alarm_power_threshold,
        }
    }

    /// Statistiken
    pub fn stats(&self) -> CalibrationStats {
        let params = self.current_parameters();
        CalibrationStats {
            history_size: self.metrics_history.len(),
            last_calibration: self.last_calibration.as_ref().map(|c| c.calibrated_at),
            current_confidence: params.confidence,
            current_entity_exponent: params.entity_exponent,
            current_decay_rate: params.decay_rate_per_day,
            deviation_from_base: self.calculate_deviation_from_base(),
        }
    }

    /// Berechne Abweichung von Basis-Konfiguration
    fn calculate_deviation_from_base(&self) -> f64 {
        let params = self.current_parameters();

        let exp_dev = ((params.entity_exponent - self.base_config.entity_exponent) / 0.1).abs();
        let decay_dev =
            ((params.decay_rate_per_day - self.base_config.decay_rate_per_day) / 0.005).abs();

        (exp_dev + decay_dev) / 2.0
    }
}

/// Statistiken der Kalibrierungs-Engine
#[derive(Debug, Clone)]
pub struct CalibrationStats {
    pub history_size: usize,
    pub last_calibration: Option<Instant>,
    pub current_confidence: f64,
    pub current_entity_exponent: f64,
    pub current_decay_rate: f64,
    pub deviation_from_base: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_metrics(gini: f64, churn: f64, nodes: u64) -> NetworkMetrics {
        NetworkMetrics {
            gini_coefficient: gini,
            churn_rate_24h: churn,
            estimated_sybil_ratio: 0.05,
            avg_latency_ms: 50.0,
            active_node_count: nodes,
            total_power: 1000.0,
            top_concentration: 0.35,
            trust_entropy: 3.5,
        }
    }

    #[test]
    fn test_calibration_engine_creation() {
        let engine = CalibrationEngine::with_defaults();
        let stats = engine.stats();

        assert_eq!(stats.history_size, 0);
        assert!(stats.current_confidence == 0.0);
    }

    #[test]
    fn test_record_metrics() {
        let mut engine = CalibrationEngine::with_defaults();

        for i in 0..10 {
            engine.record_metrics(sample_metrics(0.3 + i as f64 * 0.01, 0.05, 500));
        }

        assert_eq!(engine.metrics_history.len(), 10);
    }

    #[test]
    fn test_smoothed_metrics() {
        let mut engine = CalibrationEngine::with_defaults();

        // Variierende Gini-Werte
        for i in 0..20 {
            let gini = if i % 2 == 0 { 0.3 } else { 0.4 };
            engine.record_metrics(sample_metrics(gini, 0.05, 500));
        }

        let smoothed = engine.smoothed_metrics().unwrap();

        // EMA sollte zwischen 0.3 und 0.4 liegen
        assert!(smoothed.gini_coefficient > 0.3);
        assert!(smoothed.gini_coefficient < 0.4);
    }

    #[test]
    fn test_calibration_requires_min_nodes() {
        let mut engine = CalibrationEngine::with_defaults();

        // Nur 50 Nodes - unter Minimum
        for _ in 0..50 {
            engine.record_metrics(sample_metrics(0.5, 0.1, 50));
        }

        // Sollte None zurückgeben wegen zu wenig Nodes
        assert!(engine.calibrate().is_none());
    }

    #[test]
    fn test_high_gini_increases_exponent() {
        let mut engine = CalibrationEngine::with_defaults();
        // Keine Kalibrierungs-Interval-Wartezeit für Test
        engine.config.min_calibration_interval = Duration::ZERO;
        engine.config.min_confidence = 0.0;

        // Hoher Gini (Ungleichheit)
        for _ in 0..50 {
            engine.record_metrics(sample_metrics(0.7, 0.05, 500));
        }

        let params = engine.calibrate().unwrap();

        // Entity-Exponent sollte erhöht sein
        assert!(
            params.entity_exponent > 0.25,
            "Expected exponent > 0.25, got {}",
            params.entity_exponent
        );
    }

    #[test]
    fn test_high_churn_decreases_decay() {
        let mut engine = CalibrationEngine::with_defaults();
        engine.config.min_calibration_interval = Duration::ZERO;
        engine.config.min_confidence = 0.0;

        // Hoher Churn
        for _ in 0..50 {
            engine.record_metrics(sample_metrics(0.35, 0.20, 500)); // 20% Churn
        }

        let params = engine.calibrate().unwrap();

        // Decay-Rate sollte niedriger sein (mehr Stabilität bei Churn)
        assert!(
            params.decay_rate_per_day < 0.006,
            "Expected decay < 0.006, got {}",
            params.decay_rate_per_day
        );
    }

    #[test]
    fn test_parameters_within_bounds() {
        let mut engine = CalibrationEngine::with_defaults();
        engine.config.min_calibration_interval = Duration::ZERO;
        engine.config.min_confidence = 0.0;

        // Extreme Werte
        for _ in 0..50 {
            engine.record_metrics(sample_metrics(0.99, 0.50, 500));
        }

        let params = engine.calibrate().unwrap();
        let bounds = &engine.config.bounds;

        assert!(params.entity_exponent >= bounds.entity_exponent.0);
        assert!(params.entity_exponent <= bounds.entity_exponent.1);
        assert!(params.decay_rate_per_day >= bounds.decay_rate.0);
        assert!(params.decay_rate_per_day <= bounds.decay_rate.1);
    }

    #[test]
    fn test_to_anti_calcification_config() {
        let mut engine = CalibrationEngine::with_defaults();
        engine.config.min_calibration_interval = Duration::ZERO;
        engine.config.min_confidence = 0.0;

        for _ in 0..50 {
            engine.record_metrics(sample_metrics(0.4, 0.05, 500));
        }

        engine.calibrate();
        let config = engine.to_anti_calcification_config();

        // Config sollte gültige Werte haben
        assert!(config.entity_exponent > 0.0);
        assert!(config.decay_rate_per_day > 0.0);
        assert!(config.alarm_top_percentage > 0.0);
        assert!(config.alarm_power_threshold > 0.0);
    }
}
