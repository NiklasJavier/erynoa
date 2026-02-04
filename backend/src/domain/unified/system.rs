//! # System-Modi und Prioritäten
//!
//! Definiert System-Betriebsmodi und Event-Prioritäten für das Erynoa-System.
//!
//! ## Typen
//!
//! - [`SystemMode`]: Circuit Breaker Pattern für System-Stabilität
//! - [`EventPriority`]: Prioritätsstufen für Event-Verarbeitung
//! - [`AnomalySeverity`]: Schweregrade für erkannte Anomalien
//!
//! ## Verwendung
//!
//! ```rust
//! use erynoa_api::domain::unified::system::{SystemMode, EventPriority, AnomalySeverity};
//!
//! let mode = SystemMode::Normal;
//! assert!(mode.is_operational());
//! assert!(mode.allows_execution());
//!
//! let priority = EventPriority::Critical;
//! assert!(priority.is_urgent());
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// SystemMode - Circuit Breaker Pattern
// ============================================================================

/// System-Betriebsmodus (Circuit Breaker Pattern)
///
/// Implementiert ein 3-Stufen-Modell für System-Stabilität:
/// - `Normal`: Volle Funktionalität
/// - `Degraded`: Eingeschränkte Funktionalität, präventive Maßnahmen aktiv
/// - `EmergencyShutdown`: Minimal-Betrieb, nur Recovery möglich
///
/// ## Übergänge
///
/// ```text
/// Normal ──► Degraded ──► EmergencyShutdown
///    │          │                  │
///    │          ▼                  │
///    │       Normal ◄──────────────┘ (mit Admin-Reset)
///    ▼
/// EmergencyShutdown
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum SystemMode {
    /// Normaler Betrieb - volle Funktionalität
    #[default]
    Normal = 0,

    /// Degradierter Modus - eingeschränkte Funktionalität
    /// - ExecutionState pausiert (keine neuen Contexts)
    /// - Gateway-Crossings blockiert
    /// - Mana-Regeneration auf 0
    Degraded = 1,

    /// Notfall-Shutdown - Node offline bis Manual Reset
    /// - Alle eingehenden Requests abgelehnt
    /// - Nur Admin-Recovery-Endpoint aktiv
    EmergencyShutdown = 2,
}

impl SystemMode {
    /// Konvertiere von u8
    #[inline]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => SystemMode::Normal,
            1 => SystemMode::Degraded,
            2 => SystemMode::EmergencyShutdown,
            _ => SystemMode::Normal,
        }
    }

    /// Prüfe ob System operationell ist (Normal oder Degraded)
    #[inline]
    pub fn is_operational(&self) -> bool {
        matches!(self, SystemMode::Normal | SystemMode::Degraded)
    }

    /// Prüfe ob Execution erlaubt ist (nur Normal)
    #[inline]
    pub fn allows_execution(&self) -> bool {
        matches!(self, SystemMode::Normal)
    }

    /// Prüfe ob Gateway-Crossings erlaubt sind (nur Normal)
    #[inline]
    pub fn allows_crossings(&self) -> bool {
        matches!(self, SystemMode::Normal)
    }

    /// Prüfe ob Mana-Regeneration aktiv ist (nur Normal)
    #[inline]
    pub fn allows_mana_regen(&self) -> bool {
        matches!(self, SystemMode::Normal)
    }

    /// Prüfe ob System in Notfall-Modus ist
    #[inline]
    pub fn is_emergency(&self) -> bool {
        matches!(self, SystemMode::EmergencyShutdown)
    }

    /// Human-readable Beschreibung
    pub fn description(&self) -> &'static str {
        match self {
            SystemMode::Normal => "Normal operation - full functionality",
            SystemMode::Degraded => "Degraded mode - limited functionality",
            SystemMode::EmergencyShutdown => "Emergency shutdown - admin recovery only",
        }
    }
}

impl std::fmt::Display for SystemMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemMode::Normal => write!(f, "Normal"),
            SystemMode::Degraded => write!(f, "Degraded"),
            SystemMode::EmergencyShutdown => write!(f, "EmergencyShutdown"),
        }
    }
}

// ============================================================================
// EventPriority - Event-Priorisierung
// ============================================================================

/// Prioritätsstufe für Events
///
/// Verwendet für:
/// - Queue-Priorisierung in EventBus
/// - Scheduling-Entscheidungen
/// - Resource-Allokation unter Last
///
/// ## Reihenfolge (absteigend)
///
/// 1. `Critical` (0) - Consensus, Trust-Critical
/// 2. `High` (1) - Gateway-Crossings, Governance-Votes
/// 3. `Normal` (2) - Standard-Events
/// 4. `Low` (3) - Metrics, Telemetry
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum EventPriority {
    /// Höchste Priorität: Consensus, Trust-Critical
    Critical = 0,
    /// Hohe Priorität: Gateway-Crossings, Governance-Votes
    High = 1,
    /// Normale Priorität: Standard-Events
    Normal = 2,
    /// Niedrige Priorität: Metrics, Telemetry
    Low = 3,
}

impl EventPriority {
    /// Konvertiere von u8
    #[inline]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => EventPriority::Critical,
            1 => EventPriority::High,
            2 => EventPriority::Normal,
            _ => EventPriority::Low,
        }
    }

    /// Prüfe ob Priority dringend ist (Critical oder High)
    #[inline]
    pub fn is_urgent(&self) -> bool {
        matches!(self, EventPriority::Critical | EventPriority::High)
    }

    /// Prüfe ob Priority kritisch ist
    #[inline]
    pub fn is_critical(&self) -> bool {
        matches!(self, EventPriority::Critical)
    }

    /// Hole numerischen Wert (kleiner = höhere Priorität)
    #[inline]
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    /// Human-readable Beschreibung
    pub fn description(&self) -> &'static str {
        match self {
            EventPriority::Critical => "Critical priority - immediate processing",
            EventPriority::High => "High priority - expedited processing",
            EventPriority::Normal => "Normal priority - standard processing",
            EventPriority::Low => "Low priority - background processing",
        }
    }
}

impl Default for EventPriority {
    fn default() -> Self {
        EventPriority::Normal
    }
}

impl std::fmt::Display for EventPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventPriority::Critical => write!(f, "Critical"),
            EventPriority::High => write!(f, "High"),
            EventPriority::Normal => write!(f, "Normal"),
            EventPriority::Low => write!(f, "Low"),
        }
    }
}

// ============================================================================
// AnomalySeverity - Anomalie-Schweregrade
// ============================================================================

/// Schweregrad einer erkannten Anomalie
///
/// Verwendet von:
/// - Protection-Layer für Anomalie-Klassifikation
/// - Circuit Breaker für Eskalations-Entscheidungen
/// - Alert-System für Benachrichtigungen
///
/// ## Reaktionen nach Schweregrad
///
/// | Severity | Reaktion |
/// |----------|----------|
/// | Critical | Circuit Breaker trigger, sofortige Isolation |
/// | High | Intensive Überwachung, Rate-Limiting |
/// | Medium | Logging, Trend-Analyse |
/// | Low | Nur informativ, keine Aktion |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum AnomalySeverity {
    /// Kritisch: Sofortige Reaktion erforderlich (Circuit Breaker)
    Critical = 0,
    /// Hoch: Dringende Aufmerksamkeit
    High = 1,
    /// Mittel: Monitoring erforderlich
    Medium = 2,
    /// Niedrig: Informativ
    Low = 3,
}

impl AnomalySeverity {
    /// Konvertiere von u8
    #[inline]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => AnomalySeverity::Critical,
            1 => AnomalySeverity::High,
            2 => AnomalySeverity::Medium,
            _ => AnomalySeverity::Low,
        }
    }

    /// Prüfe ob Severity kritisch ist
    #[inline]
    pub fn is_critical(&self) -> bool {
        matches!(self, AnomalySeverity::Critical)
    }

    /// Prüfe ob Severity dringend ist (Critical oder High)
    #[inline]
    pub fn requires_attention(&self) -> bool {
        matches!(self, AnomalySeverity::Critical | AnomalySeverity::High)
    }

    /// Prüfe ob Circuit Breaker getriggert werden sollte
    #[inline]
    pub fn triggers_circuit_breaker(&self) -> bool {
        matches!(self, AnomalySeverity::Critical)
    }

    /// Hole numerischen Wert (kleiner = höhere Severity)
    #[inline]
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    /// Human-readable Beschreibung
    pub fn description(&self) -> &'static str {
        match self {
            AnomalySeverity::Critical => "Critical - immediate action required",
            AnomalySeverity::High => "High - urgent attention needed",
            AnomalySeverity::Medium => "Medium - monitoring required",
            AnomalySeverity::Low => "Low - informational only",
        }
    }
}

impl Default for AnomalySeverity {
    fn default() -> Self {
        AnomalySeverity::Low
    }
}

impl std::fmt::Display for AnomalySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnomalySeverity::Critical => write!(f, "Critical"),
            AnomalySeverity::High => write!(f, "High"),
            AnomalySeverity::Medium => write!(f, "Medium"),
            AnomalySeverity::Low => write!(f, "Low"),
        }
    }
}

// ============================================================================
// Compile-Time Size Checks
// ============================================================================

const _: () = {
    assert!(std::mem::size_of::<SystemMode>() == 1);
    assert!(std::mem::size_of::<EventPriority>() == 1);
    assert!(std::mem::size_of::<AnomalySeverity>() == 1);
};

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_mode_default() {
        let mode = SystemMode::default();
        assert_eq!(mode, SystemMode::Normal);
        assert!(mode.is_operational());
        assert!(mode.allows_execution());
    }

    #[test]
    fn test_system_mode_degraded() {
        let mode = SystemMode::Degraded;
        assert!(mode.is_operational());
        assert!(!mode.allows_execution());
        assert!(!mode.allows_crossings());
        assert!(!mode.allows_mana_regen());
    }

    #[test]
    fn test_system_mode_emergency() {
        let mode = SystemMode::EmergencyShutdown;
        assert!(!mode.is_operational());
        assert!(!mode.allows_execution());
        assert!(mode.is_emergency());
    }

    #[test]
    fn test_system_mode_from_u8() {
        assert_eq!(SystemMode::from_u8(0), SystemMode::Normal);
        assert_eq!(SystemMode::from_u8(1), SystemMode::Degraded);
        assert_eq!(SystemMode::from_u8(2), SystemMode::EmergencyShutdown);
        assert_eq!(SystemMode::from_u8(255), SystemMode::Normal); // fallback
    }

    #[test]
    fn test_event_priority_ordering() {
        assert!(EventPriority::Critical < EventPriority::High);
        assert!(EventPriority::High < EventPriority::Normal);
        assert!(EventPriority::Normal < EventPriority::Low);
    }

    #[test]
    fn test_event_priority_urgent() {
        assert!(EventPriority::Critical.is_urgent());
        assert!(EventPriority::High.is_urgent());
        assert!(!EventPriority::Normal.is_urgent());
        assert!(!EventPriority::Low.is_urgent());
    }

    #[test]
    fn test_event_priority_from_u8() {
        assert_eq!(EventPriority::from_u8(0), EventPriority::Critical);
        assert_eq!(EventPriority::from_u8(1), EventPriority::High);
        assert_eq!(EventPriority::from_u8(2), EventPriority::Normal);
        assert_eq!(EventPriority::from_u8(3), EventPriority::Low);
        assert_eq!(EventPriority::from_u8(255), EventPriority::Low); // fallback
    }

    #[test]
    fn test_anomaly_severity_critical() {
        let severity = AnomalySeverity::Critical;
        assert!(severity.is_critical());
        assert!(severity.requires_attention());
        assert!(severity.triggers_circuit_breaker());
    }

    #[test]
    fn test_anomaly_severity_levels() {
        assert!(AnomalySeverity::High.requires_attention());
        assert!(!AnomalySeverity::Medium.requires_attention());
        assert!(!AnomalySeverity::Low.triggers_circuit_breaker());
    }

    #[test]
    fn test_anomaly_severity_from_u8() {
        assert_eq!(AnomalySeverity::from_u8(0), AnomalySeverity::Critical);
        assert_eq!(AnomalySeverity::from_u8(1), AnomalySeverity::High);
        assert_eq!(AnomalySeverity::from_u8(2), AnomalySeverity::Medium);
        assert_eq!(AnomalySeverity::from_u8(3), AnomalySeverity::Low);
    }

    #[test]
    fn test_display_impls() {
        assert_eq!(format!("{}", SystemMode::Normal), "Normal");
        assert_eq!(format!("{}", EventPriority::Critical), "Critical");
        assert_eq!(format!("{}", AnomalySeverity::High), "High");
    }

    #[test]
    fn test_serde_roundtrip() {
        let mode = SystemMode::Degraded;
        let json = serde_json::to_string(&mode).unwrap();
        let parsed: SystemMode = serde_json::from_str(&json).unwrap();
        assert_eq!(mode, parsed);

        let priority = EventPriority::High;
        let json = serde_json::to_string(&priority).unwrap();
        let parsed: EventPriority = serde_json::from_str(&json).unwrap();
        assert_eq!(priority, parsed);
    }
}
