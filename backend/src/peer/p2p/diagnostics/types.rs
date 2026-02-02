//! Core Types für das Diagnostics-System

use serde::{Deserialize, Serialize};
use std::time::Duration;

// ============================================================================
// DIAGNOSTIC STATUS TYPES
// ============================================================================

/// Status einer einzelnen Komponente
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ComponentStatus {
    /// Komponente funktioniert perfekt
    Healthy,
    /// Komponente funktioniert mit Einschränkungen
    Degraded,
    /// Komponente ist nicht verfügbar
    Unavailable,
    /// Komponente ist nicht aktiviert (feature-gated)
    Disabled,
    /// Status unbekannt (noch nicht geprüft)
    Unknown,
}

impl ComponentStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Healthy => "✅",
            Self::Degraded => "⚠️",
            Self::Unavailable => "❌",
            Self::Disabled => "🔒",
            Self::Unknown => "❓",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Healthy => "status-healthy",
            Self::Degraded => "status-degraded",
            Self::Unavailable => "status-unavailable",
            Self::Disabled => "status-disabled",
            Self::Unknown => "status-unknown",
        }
    }
}

impl Default for ComponentStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

// ============================================================================
// DIAGNOSTIC CHECK
// ============================================================================

/// Einzelner Diagnose-Check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCheck {
    pub name: String,
    pub status: ComponentStatus,
    pub message: String,
    pub latency_ms: Option<u64>,
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_unit: Option<String>,
}

impl DiagnosticCheck {
    pub fn healthy(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: ComponentStatus::Healthy,
            message: message.into(),
            latency_ms: None,
            details: None,
            metric_value: None,
            metric_unit: None,
        }
    }

    pub fn degraded(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: ComponentStatus::Degraded,
            message: message.into(),
            latency_ms: None,
            details: None,
            metric_value: None,
            metric_unit: None,
        }
    }

    pub fn unavailable(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: ComponentStatus::Unavailable,
            message: message.into(),
            latency_ms: None,
            details: None,
            metric_value: None,
            metric_unit: None,
        }
    }

    pub fn disabled(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: ComponentStatus::Disabled,
            message: "Feature not enabled".into(),
            latency_ms: None,
            details: None,
            metric_value: None,
            metric_unit: None,
        }
    }

    pub fn with_latency(mut self, latency: Duration) -> Self {
        self.latency_ms = Some(latency.as_millis() as u64);
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_metric(mut self, value: f64, unit: impl Into<String>) -> Self {
        self.metric_value = Some(value);
        self.metric_unit = Some(unit.into());
        self
    }
}

// ============================================================================
// LAYER DIAGNOSTIC
// ============================================================================

/// Diagnose einer kompletten Schicht
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerDiagnostic {
    pub layer_name: String,
    pub layer_number: u8,
    pub overall_status: ComponentStatus,
    pub checks: Vec<DiagnosticCheck>,
    pub feature_flag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl LayerDiagnostic {
    pub fn new(name: impl Into<String>, number: u8) -> Self {
        Self {
            layer_name: name.into(),
            layer_number: number,
            overall_status: ComponentStatus::Unknown,
            checks: Vec::new(),
            feature_flag: None,
            description: None,
        }
    }

    pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
        self.feature_flag = Some(feature.into());
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn add_check(&mut self, check: DiagnosticCheck) {
        self.checks.push(check);
        self.update_overall_status();
    }

    fn update_overall_status(&mut self) {
        if self.checks.is_empty() {
            self.overall_status = ComponentStatus::Unknown;
            return;
        }

        let has_unavailable = self
            .checks
            .iter()
            .any(|c| c.status == ComponentStatus::Unavailable);
        let has_degraded = self
            .checks
            .iter()
            .any(|c| c.status == ComponentStatus::Degraded);
        let all_disabled = self
            .checks
            .iter()
            .all(|c| c.status == ComponentStatus::Disabled);

        if all_disabled {
            self.overall_status = ComponentStatus::Disabled;
        } else if has_unavailable {
            self.overall_status = ComponentStatus::Unavailable;
        } else if has_degraded {
            self.overall_status = ComponentStatus::Degraded;
        } else {
            self.overall_status = ComponentStatus::Healthy;
        }
    }
}

// ============================================================================
// P2P DIAGNOSTICS
// ============================================================================

/// Vollständige P2P-Diagnose
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PDiagnostics {
    pub timestamp: String,
    pub node_id: Option<String>,
    pub overall_status: ComponentStatus,
    pub layers: Vec<LayerDiagnostic>,
    pub summary: DiagnosticSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_metrics: Option<super::NetworkMetrics>,
    /// Live Swarm Snapshot mit echten Laufzeit-Daten
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swarm_snapshot: Option<super::SwarmSnapshot>,
}

/// Zusammenfassung der Diagnose
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticSummary {
    pub total_checks: usize,
    pub healthy_count: usize,
    pub degraded_count: usize,
    pub unavailable_count: usize,
    pub disabled_count: usize,
    pub health_percentage: f32,
}

impl P2PDiagnostics {
    pub fn new() -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            node_id: None,
            overall_status: ComponentStatus::Unknown,
            layers: Vec::new(),
            summary: DiagnosticSummary::default(),
            network_metrics: None,
            swarm_snapshot: None,
        }
    }

    pub fn with_node_id(mut self, node_id: impl Into<String>) -> Self {
        self.node_id = Some(node_id.into());
        self
    }

    pub fn with_metrics(mut self, metrics: super::NetworkMetrics) -> Self {
        self.network_metrics = Some(metrics);
        self
    }

    /// Füge SwarmSnapshot für echte Laufzeit-Daten hinzu
    pub fn with_swarm_snapshot(mut self, snapshot: super::SwarmSnapshot) -> Self {
        self.swarm_snapshot = Some(snapshot);
        self
    }

    pub fn add_layer(&mut self, layer: LayerDiagnostic) {
        self.layers.push(layer);
        self.update_summary();
    }

    fn update_summary(&mut self) {
        let all_checks: Vec<&DiagnosticCheck> =
            self.layers.iter().flat_map(|l| &l.checks).collect();

        self.summary.total_checks = all_checks.len();
        self.summary.healthy_count = all_checks
            .iter()
            .filter(|c| c.status == ComponentStatus::Healthy)
            .count();
        self.summary.degraded_count = all_checks
            .iter()
            .filter(|c| c.status == ComponentStatus::Degraded)
            .count();
        self.summary.unavailable_count = all_checks
            .iter()
            .filter(|c| c.status == ComponentStatus::Unavailable)
            .count();
        self.summary.disabled_count = all_checks
            .iter()
            .filter(|c| c.status == ComponentStatus::Disabled)
            .count();

        let active_checks = self.summary.total_checks - self.summary.disabled_count;
        if active_checks > 0 {
            self.summary.health_percentage =
                (self.summary.healthy_count as f32 / active_checks as f32) * 100.0;
        }

        // Overall Status
        if self.summary.unavailable_count > 0 {
            self.overall_status = ComponentStatus::Unavailable;
        } else if self.summary.degraded_count > 0 {
            self.overall_status = ComponentStatus::Degraded;
        } else if self.summary.healthy_count > 0 {
            self.overall_status = ComponentStatus::Healthy;
        } else {
            self.overall_status = ComponentStatus::Disabled;
        }
    }

    /// Formatierte Ausgabe für CLI
    pub fn to_cli_report(&self) -> String {
        let mut report = String::new();

        report.push_str("\n╔══════════════════════════════════════════════════════════════════╗\n");
        report.push_str("║           🔍 ERYNOA P2P DIAGNOSTICS REPORT                       ║\n");
        report.push_str("╠══════════════════════════════════════════════════════════════════╣\n");

        if let Some(ref node_id) = self.node_id {
            report.push_str(&format!(
                "║ Node: {:<56} ║\n",
                &node_id[..node_id.len().min(56)]
            ));
        }
        report.push_str(&format!(
            "║ Time: {:<56} ║\n",
            &self.timestamp[..self.timestamp.len().min(56)]
        ));
        report.push_str(&format!(
            "║ Overall: {} {:<52} ║\n",
            self.overall_status.emoji(),
            format!("{:?}", self.overall_status).to_uppercase()
        ));
        report.push_str("╠══════════════════════════════════════════════════════════════════╣\n");

        for layer in &self.layers {
            report
                .push_str("║                                                                  ║\n");
            report.push_str(&format!(
                "║ {} Layer {}: {:<48} ║\n",
                layer.overall_status.emoji(),
                layer.layer_number,
                layer.layer_name
            ));

            if let Some(ref feature) = layer.feature_flag {
                report.push_str(&format!("║   Feature: {:<53} ║\n", feature));
            }

            for check in &layer.checks {
                let latency_str = check
                    .latency_ms
                    .map(|ms| format!(" ({} ms)", ms))
                    .unwrap_or_default();

                let metric_str = check
                    .metric_value
                    .map(|v| {
                        let unit = check.metric_unit.as_deref().unwrap_or("");
                        format!(" [{:.1}{}]", v, unit)
                    })
                    .unwrap_or_default();

                report.push_str(&format!(
                    "║   {} {:<36}{:<10}{:<6} ║\n",
                    check.status.emoji(),
                    check.name,
                    latency_str,
                    metric_str
                ));

                if check.message.len() <= 50 {
                    report.push_str(&format!("║      └─ {:<54} ║\n", check.message));
                } else {
                    report.push_str(&format!("║      └─ {:<54} ║\n", &check.message[..50]));
                }
            }
        }

        report.push_str("║                                                                  ║\n");
        report.push_str("╠══════════════════════════════════════════════════════════════════╣\n");
        report.push_str("║ SUMMARY                                                          ║\n");
        report.push_str(&format!(
            "║   Total Checks: {:<48} ║\n",
            self.summary.total_checks
        ));
        report.push_str(&format!(
            "║   ✅ Healthy:   {:<48} ║\n",
            self.summary.healthy_count
        ));
        report.push_str(&format!(
            "║   ⚠️  Degraded:  {:<48} ║\n",
            self.summary.degraded_count
        ));
        report.push_str(&format!(
            "║   ❌ Unavailable: {:<46} ║\n",
            self.summary.unavailable_count
        ));
        report.push_str(&format!(
            "║   🔒 Disabled:  {:<48} ║\n",
            self.summary.disabled_count
        ));
        report.push_str(&format!(
            "║   Health:      {:.1}%{:<45} ║\n",
            self.summary.health_percentage, ""
        ));
        report.push_str("╚══════════════════════════════════════════════════════════════════╝\n");

        report
    }
}

impl Default for P2PDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// STREAM SNAPSHOT
// ============================================================================

/// Snapshot für SSE-Stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSnapshot {
    pub timestamp: String,
    pub metrics: super::NetworkMetrics,
    pub peer_count: usize,
    pub recent_events: Vec<super::DiagnosticEvent>,
    pub health: HealthStatus,
}

/// Vereinfachter Health-Status für Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: ComponentStatus,
    pub healthy_layers: u8,
    pub total_layers: u8,
    pub message: String,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_status() {
        assert!(ComponentStatus::Healthy.is_healthy());
        assert!(!ComponentStatus::Degraded.is_healthy());
        assert!(!ComponentStatus::Unavailable.is_healthy());
    }

    #[test]
    fn test_diagnostic_check() {
        let check = DiagnosticCheck::healthy("Test", "All good")
            .with_latency(std::time::Duration::from_millis(42))
            .with_metric(95.5, "%");

        assert_eq!(check.status, ComponentStatus::Healthy);
        assert_eq!(check.latency_ms, Some(42));
        assert_eq!(check.metric_value, Some(95.5));
    }

    #[test]
    fn test_layer_diagnostic() {
        let mut layer = LayerDiagnostic::new("Test Layer", 1);
        layer.add_check(DiagnosticCheck::healthy("Check 1", "OK"));
        layer.add_check(DiagnosticCheck::degraded("Check 2", "Warning"));

        assert_eq!(layer.overall_status, ComponentStatus::Degraded);
    }
}
