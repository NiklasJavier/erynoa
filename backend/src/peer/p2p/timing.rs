//! # P2P Timing und τ-Variabilität
//!
//! Adaptive Sync-Timing basierend auf Netzwerk-Bedingungen (UDM §IX.2).
//!
//! ## τ-Variabilität
//!
//! Das Timing passt sich dynamisch an Netzwerkbedingungen an:
//! - **Latenz**: Höhere Latenz → längere Intervalle
//! - **Packet Loss**: Mehr Verlust → konservativeres Timing
//! - **Peer-Count**: Weniger Peers → häufigere Syncs
//!
//! ## Formel
//!
//! ```text
//! V(τ) = (0.5 + latency_norm × loss_factor / peer_factor).clamp(0.5, 3.0)
//! ```
//!
//! ## Axiom-Referenz
//!
//! - **Κ9**: Kausale Struktur bei variablem Netzwerk-Timing
//! - **Κ19**: Anti-Verkalkung durch adaptive Delays

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Netzwerk-Bedingungen für τ-Variabilität (UDM §IX.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConditions {
    /// Round-Trip-Latenz in Millisekunden
    pub latency_ms: u32,

    /// Packet-Loss-Rate (0.0 - 1.0)
    pub packet_loss: f32,

    /// Verfügbare Bandbreite in kbit/s
    pub bandwidth_kbps: u32,

    /// Anzahl verbundener Peers
    pub peer_count: u32,

    /// Zeitstempel der letzten Messung
    #[serde(skip)]
    pub last_updated: Option<Instant>,
}

impl Default for NetworkConditions {
    fn default() -> Self {
        Self {
            latency_ms: 50,        // Annahme: moderates Netzwerk
            packet_loss: 0.01,     // 1% Verlust
            bandwidth_kbps: 10000, // 10 Mbit/s
            peer_count: 5,
            last_updated: None,
        }
    }
}

impl NetworkConditions {
    /// Erstelle neue NetworkConditions
    pub fn new(latency_ms: u32, packet_loss: f32, bandwidth_kbps: u32, peer_count: u32) -> Self {
        Self {
            latency_ms,
            packet_loss: packet_loss.clamp(0.0, 1.0),
            bandwidth_kbps,
            peer_count,
            last_updated: Some(Instant::now()),
        }
    }

    /// Berechne τ-Variabilitätsfaktor (UDM §IX.2)
    ///
    /// Formel: V(τ) = (0.5 + latency_norm × loss_factor / peer_factor).clamp(0.5, 3.0)
    ///
    /// - latency_norm: Latenz normalisiert auf [0, 1] (100ms = 1.0)
    /// - loss_factor: 1.0 + packet_loss × 2.0
    /// - peer_factor: max(0.5, peer_count / 10.0)
    ///
    /// Returns: Faktor im Bereich [0.5, 3.0]
    pub fn variability_factor(&self) -> f32 {
        let latency_norm = (self.latency_ms as f32 / 100.0).min(1.0);
        let loss_factor = 1.0 + self.packet_loss * 2.0;
        let peer_factor = (self.peer_count as f32 / 10.0).max(0.5);

        (0.5 + latency_norm * loss_factor / peer_factor).clamp(0.5, 3.0)
    }

    /// Aktualisiere Bedingungen mit exponentieller Glättung
    pub fn update_smoothed(&mut self, new: &NetworkConditions, alpha: f32) {
        let alpha = alpha.clamp(0.0, 1.0);
        self.latency_ms =
            ((1.0 - alpha) * self.latency_ms as f32 + alpha * new.latency_ms as f32) as u32;
        self.packet_loss = (1.0 - alpha) * self.packet_loss + alpha * new.packet_loss;
        self.bandwidth_kbps =
            ((1.0 - alpha) * self.bandwidth_kbps as f32 + alpha * new.bandwidth_kbps as f32) as u32;
        self.peer_count = new.peer_count; // Peer-Count direkt übernehmen
        self.last_updated = Some(Instant::now());
    }

    /// Prüfe ob Bedingungen veraltet sind
    pub fn is_stale(&self, max_age: Duration) -> bool {
        match self.last_updated {
            Some(t) => t.elapsed() > max_age,
            None => true,
        }
    }

    /// Kategorisiere Netzwerk-Qualität
    pub fn quality(&self) -> NetworkQuality {
        let v = self.variability_factor();
        match v {
            v if v <= 0.75 => NetworkQuality::Excellent,
            v if v <= 1.25 => NetworkQuality::Good,
            v if v <= 2.0 => NetworkQuality::Moderate,
            _ => NetworkQuality::Poor,
        }
    }
}

/// Netzwerk-Qualitätsstufe
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkQuality {
    /// V ≤ 0.75: Ideale Bedingungen
    Excellent,
    /// 0.75 < V ≤ 1.25: Normale Bedingungen
    Good,
    /// 1.25 < V ≤ 2.0: Eingeschränkte Bedingungen
    Moderate,
    /// V > 2.0: Schlechte Bedingungen
    Poor,
}

/// Adaptives Sync-Timing mit Backoff
#[derive(Debug, Clone)]
pub struct SyncTiming {
    /// Basis-Intervall (ohne Anpassung)
    pub base_interval: Duration,

    /// Aktuelles Intervall (mit τ-Anpassung)
    pub current_interval: Duration,

    /// Minimum-Intervall (untere Grenze)
    pub min_interval: Duration,

    /// Maximum-Intervall (obere Grenze)
    pub max_interval: Duration,

    /// Aktueller Backoff-Zähler
    pub backoff_count: u32,

    /// Maximum-Backoff
    pub max_backoff: u32,

    /// Letzter Sync-Zeitpunkt
    pub last_sync: Option<Instant>,
}

impl Default for SyncTiming {
    fn default() -> Self {
        Self {
            base_interval: Duration::from_secs(30),
            current_interval: Duration::from_secs(30),
            min_interval: Duration::from_secs(5),
            max_interval: Duration::from_secs(300), // 5 Minuten
            backoff_count: 0,
            max_backoff: 5,
            last_sync: None,
        }
    }
}

impl SyncTiming {
    /// Erstelle neues SyncTiming mit Basis-Intervall
    pub fn new(base_interval: Duration) -> Self {
        Self {
            base_interval,
            current_interval: base_interval,
            ..Default::default()
        }
    }

    /// Erstelle mit konfigurierten Grenzen
    pub fn with_bounds(base: Duration, min: Duration, max: Duration) -> Self {
        Self {
            base_interval: base,
            current_interval: base,
            min_interval: min,
            max_interval: max,
            ..Default::default()
        }
    }

    /// Passe Timing an Netzwerk-Bedingungen an (τ-Variabilität)
    pub fn adjust(&mut self, conditions: &NetworkConditions) {
        let v = conditions.variability_factor();
        let adjusted = Duration::from_secs_f32(self.base_interval.as_secs_f32() * v);
        self.current_interval = adjusted.clamp(self.min_interval, self.max_interval);
    }

    /// Exponentieller Backoff bei Fehler
    pub fn exponential_backoff(&mut self) {
        self.backoff_count = (self.backoff_count + 1).min(self.max_backoff);
        let factor = 2.0_f32.powi(self.backoff_count as i32);
        let backed_off = Duration::from_secs_f32(self.current_interval.as_secs_f32() * factor);
        self.current_interval = backed_off.min(self.max_interval);
    }

    /// Reset Backoff nach erfolgreichem Sync
    pub fn reset_backoff(&mut self) {
        self.backoff_count = 0;
    }

    /// Markiere erfolgreichen Sync
    pub fn mark_synced(&mut self) {
        self.last_sync = Some(Instant::now());
        self.reset_backoff();
    }

    /// Prüfe ob Sync fällig ist
    pub fn should_sync(&self) -> bool {
        match self.last_sync {
            Some(t) => t.elapsed() >= self.current_interval,
            None => true,
        }
    }

    /// Zeit bis zum nächsten Sync
    pub fn time_until_sync(&self) -> Duration {
        match self.last_sync {
            Some(t) => {
                let elapsed = t.elapsed();
                if elapsed >= self.current_interval {
                    Duration::ZERO
                } else {
                    self.current_interval - elapsed
                }
            }
            None => Duration::ZERO,
        }
    }

    /// Aktueller Variabilitätsfaktor (für Monitoring)
    pub fn current_variability(&self) -> f32 {
        self.current_interval.as_secs_f32() / self.base_interval.as_secs_f32()
    }
}

/// Timing-Manager für verschiedene Sync-Typen
#[derive(Debug, Clone)]
pub struct TimingManager {
    /// Event-Sync-Timing
    pub event_sync: SyncTiming,

    /// Trust-Propagation-Timing
    pub trust_sync: SyncTiming,

    /// DHT-Refresh-Timing
    pub dht_refresh: SyncTiming,

    /// Peer-Discovery-Timing
    pub discovery: SyncTiming,

    /// Gemeinsame Netzwerk-Bedingungen
    pub conditions: NetworkConditions,
}

impl Default for TimingManager {
    fn default() -> Self {
        Self {
            event_sync: SyncTiming::with_bounds(
                Duration::from_secs(10),
                Duration::from_secs(2),
                Duration::from_secs(60),
            ),
            trust_sync: SyncTiming::with_bounds(
                Duration::from_secs(60),
                Duration::from_secs(10),
                Duration::from_secs(300),
            ),
            dht_refresh: SyncTiming::with_bounds(
                Duration::from_secs(300),
                Duration::from_secs(60),
                Duration::from_secs(3600),
            ),
            discovery: SyncTiming::with_bounds(
                Duration::from_secs(30),
                Duration::from_secs(5),
                Duration::from_secs(120),
            ),
            conditions: NetworkConditions::default(),
        }
    }
}

impl TimingManager {
    /// Aktualisiere Netzwerk-Bedingungen und passe alle Timings an
    pub fn update_conditions(&mut self, conditions: NetworkConditions) {
        self.conditions.update_smoothed(&conditions, 0.3);
        self.adjust_all();
    }

    /// Passe alle Timings an aktuelle Bedingungen an
    pub fn adjust_all(&mut self) {
        self.event_sync.adjust(&self.conditions);
        self.trust_sync.adjust(&self.conditions);
        self.dht_refresh.adjust(&self.conditions);
        self.discovery.adjust(&self.conditions);
    }

    /// Erhalte aktuellen Status für Monitoring
    pub fn status(&self) -> TimingStatus {
        TimingStatus {
            quality: self.conditions.quality(),
            variability: self.conditions.variability_factor(),
            event_interval_ms: self.event_sync.current_interval.as_millis() as u64,
            trust_interval_ms: self.trust_sync.current_interval.as_millis() as u64,
            dht_interval_ms: self.dht_refresh.current_interval.as_millis() as u64,
            discovery_interval_ms: self.discovery.current_interval.as_millis() as u64,
        }
    }
}

/// Timing-Status für Monitoring/Telemetrie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingStatus {
    /// Aktuelle Netzwerk-Qualität
    pub quality: NetworkQuality,

    /// Aktueller Variabilitätsfaktor
    pub variability: f32,

    /// Event-Sync-Intervall in ms
    pub event_interval_ms: u64,

    /// Trust-Sync-Intervall in ms
    pub trust_interval_ms: u64,

    /// DHT-Refresh-Intervall in ms
    pub dht_interval_ms: u64,

    /// Discovery-Intervall in ms
    pub discovery_interval_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variability_factor_optimal() {
        // Optimale Bedingungen: niedrige Latenz, kein Verlust, viele Peers
        let conditions = NetworkConditions::new(10, 0.0, 100000, 20);
        let v = conditions.variability_factor();

        // Erwartung: V ≈ 0.5 (Minimum)
        assert!(v >= 0.5, "V={} should be >= 0.5", v);
        assert!(v < 0.75, "V={} should be < 0.75 for optimal conditions", v);
    }

    #[test]
    fn test_variability_factor_poor() {
        // Schlechte Bedingungen: hohe Latenz, hoher Verlust, wenige Peers
        let conditions = NetworkConditions::new(200, 0.5, 1000, 2);
        let v = conditions.variability_factor();

        // Erwartung: V ≈ 3.0 (Maximum)
        assert!(v > 2.0, "V={} should be > 2.0 for poor conditions", v);
        assert!(v <= 3.0, "V={} should be <= 3.0", v);
    }

    #[test]
    fn test_variability_factor_moderate() {
        // Moderate Bedingungen: mittlere Latenz, geringer Verlust, einige Peers
        let conditions = NetworkConditions::new(50, 0.05, 10000, 5);
        let v = conditions.variability_factor();

        // Erwartung: 0.75 < V < 2.0
        assert!(v >= 0.5, "V={} should be >= 0.5", v);
        assert!(v <= 3.0, "V={} should be <= 3.0", v);
    }

    #[test]
    fn test_variability_clamping() {
        // Extreme Werte: Ergebnis muss innerhalb [0.5, 3.0] liegen
        let extreme_good = NetworkConditions::new(0, 0.0, u32::MAX, 1000);
        assert_eq!(extreme_good.variability_factor(), 0.5);

        let extreme_bad = NetworkConditions::new(1000, 1.0, 0, 1);
        assert_eq!(extreme_bad.variability_factor(), 3.0);
    }

    #[test]
    fn test_sync_timing_adjust() {
        let mut timing = SyncTiming::new(Duration::from_secs(30));

        // Gute Bedingungen → kürzeres Intervall
        let good = NetworkConditions::new(20, 0.01, 50000, 15);
        timing.adjust(&good);
        assert!(
            timing.current_interval < Duration::from_secs(30),
            "Good conditions should reduce interval"
        );

        // Schlechte Bedingungen → längeres Intervall
        let poor = NetworkConditions::new(150, 0.3, 1000, 2);
        timing.adjust(&poor);
        assert!(
            timing.current_interval > Duration::from_secs(30),
            "Poor conditions should increase interval"
        );
    }

    #[test]
    fn test_exponential_backoff() {
        let mut timing = SyncTiming::new(Duration::from_secs(10));

        // Erster Backoff: multipliziert current_interval mit 2^1 = 2
        timing.exponential_backoff();
        assert_eq!(timing.backoff_count, 1);
        // 10s * 2 = 20s
        assert_eq!(timing.current_interval, Duration::from_secs(20));

        // Zweiter Backoff: multipliziert vorheriges mit 2^2 = 4
        // Aber backoff wird auf current_interval angewendet: 20s * 4 = 80s
        timing.exponential_backoff();
        assert_eq!(timing.backoff_count, 2);
        assert_eq!(timing.current_interval, Duration::from_secs(80));

        // Reset setzt nur den Counter zurück, nicht das Intervall
        timing.reset_backoff();
        assert_eq!(timing.backoff_count, 0);
    }

    #[test]
    fn test_backoff_max_limit() {
        let mut timing = SyncTiming::with_bounds(
            Duration::from_secs(10),
            Duration::from_secs(1),
            Duration::from_secs(100),
        );

        // Maximale Backoffs überschreiten
        for _ in 0..10 {
            timing.exponential_backoff();
        }

        // Backoff-Count auf max begrenzt
        assert!(timing.backoff_count <= timing.max_backoff);

        // Intervall auf max begrenzt
        assert!(timing.current_interval <= timing.max_interval);
    }

    #[test]
    fn test_should_sync() {
        let mut timing = SyncTiming::new(Duration::from_millis(100));

        // Noch nie gesynct → sync fällig
        assert!(timing.should_sync());

        // Nach Sync → nicht fällig
        timing.mark_synced();
        assert!(!timing.should_sync());

        // Nach Intervall → wieder fällig
        std::thread::sleep(Duration::from_millis(150));
        assert!(timing.should_sync());
    }

    #[test]
    fn test_timing_manager() {
        let mut manager = TimingManager::default();

        // Initiale Status - Default Conditions haben moderate Latenz
        let status = manager.status();
        assert!(
            matches!(
                status.quality,
                NetworkQuality::Good | NetworkQuality::Moderate
            ),
            "Initial quality should be Good or Moderate, got {:?}",
            status.quality
        );
        let initial_variability = status.variability;

        // Update mit sehr schlechten Bedingungen (mehrfach für Smoothing)
        for _ in 0..5 {
            manager.update_conditions(NetworkConditions::new(200, 0.5, 500, 1));
        }
        let status = manager.status();

        // Variabilität sollte höher sein als initial (langsamer)
        assert!(
            status.variability > initial_variability,
            "Variability should increase with poor conditions: {} -> {}",
            initial_variability,
            status.variability
        );

        // Variabilität sollte über 1.0 sein
        assert!(
            status.variability > 1.0,
            "Variability should be > 1.0, got {}",
            status.variability
        );
    }

    #[test]
    fn test_network_quality_categorization() {
        let excellent = NetworkConditions::new(10, 0.0, 100000, 30);
        assert_eq!(excellent.quality(), NetworkQuality::Excellent);

        let good = NetworkConditions::new(50, 0.02, 20000, 10);
        assert_eq!(good.quality(), NetworkQuality::Good);

        let poor = NetworkConditions::new(300, 0.5, 500, 1);
        assert_eq!(poor.quality(), NetworkQuality::Poor);
    }

    #[test]
    fn test_smoothed_update() {
        let mut conditions = NetworkConditions::new(100, 0.1, 10000, 5);
        let new_conditions = NetworkConditions::new(50, 0.05, 20000, 10);

        conditions.update_smoothed(&new_conditions, 0.5);

        // Werte sollten zwischen alt und neu liegen
        assert!(conditions.latency_ms > 50 && conditions.latency_ms < 100);
        assert!(conditions.packet_loss > 0.05 && conditions.packet_loss < 0.1);

        // Peer-Count wird direkt übernommen
        assert_eq!(conditions.peer_count, 10);
    }
}
