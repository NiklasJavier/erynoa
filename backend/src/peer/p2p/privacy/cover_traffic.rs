//! # Cover-Traffic Generator (RL10, RL18)
//!
//! Protocol-Pledge basierter Cover-Traffic für Indistinguishability.
//!
//! ## Axiom-Referenzen
//!
//! - **RL10**: Cover-Traffic Indistinguishability
//!   ```text
//!   ∀ msg: P(real|observation) = P(cover|observation)
//!   Size-Classes: {256, 512, 1K, 2K, 4K, 8K, 16K, 32K} bytes
//!   ```
//!
//! - **RL18**: Cover-Traffic als Protocol Pledge
//!   ```text
//!   Minimum-Rate: λ_peer ≥ λ_min(peer_type)
//!   - Full-Relay: 0.2/s (12 Dummies/Minute)
//!   - Apprentice-Relay: 0.1/s (6 Dummies/Minute)
//!   - Active-User: 0.05/s (3 Dummies/Minute)
//!   - Passive-User: 0.01/s (0.6 Dummies/Minute)
//!
//!   Non-Compliance-Penalty:
//!   deficit := (λ_min - λ_observed) / λ_min
//!   penalty_V := 0.02 × deficit × days
//!   penalty_Ω := 0.03 × deficit × days
//!   ```
//!
//! - **RL21**: Size-Class Quantization
//!   ```text
//!   quantize(size) = min{c ∈ SIZE_CLASSES | c ≥ size}
//!   ```
//!
//! ## Wire-Format
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                  COVER-TRAFFIC PIPELINE                         │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │  [Poisson Timer] ──► [Size-Class] ──► [CSPRNG Pad] ──► [Route] │
//! │        │                  │               │              │      │
//! │        ▼                  ▼               ▼              ▼      │
//! │   Exp(λ_min)         Quantized       Random Fill   Valid Path  │
//! │                                                                 │
//! │  [Compliance Monitor] ──► [Trust Penalty] ──► [Level Downgrade]│
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use libp2p::PeerId;
use parking_lot::RwLock;
use rand::Rng;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Size-Classes für Cover-Traffic (RL10, RL21)
pub const SIZE_CLASSES: [usize; 8] = [256, 512, 1024, 2048, 4096, 8192, 16384, 32768];

/// Minimum Size-Class
pub const MIN_SIZE_CLASS: usize = 256;

/// Maximum Size-Class
pub const MAX_SIZE_CLASS: usize = 32768;

/// Compliance-Schwelle für Warnung
pub const COMPLIANCE_WARNING_THRESHOLD: f64 = 0.8;

/// Compliance-Schwelle für Violation
pub const COMPLIANCE_VIOLATION_THRESHOLD: f64 = 0.5;

/// Default Observation-Period für Compliance (1 Tag)
pub const DEFAULT_OBSERVATION_PERIOD_SECS: u64 = 86400;

// ============================================================================
// SIZE-CLASS QUANTIZATION (RL21)
// ============================================================================

/// Quantisiere Größe auf nächste Size-Class (RL21)
///
/// ```text
/// quantize(size) = min{c ∈ SIZE_CLASSES | c ≥ size}
/// ```
///
/// # Beispiel
///
/// ```rust,ignore
/// assert_eq!(quantize_size(100), 256);
/// assert_eq!(quantize_size(1000), 1024);
/// assert_eq!(quantize_size(50000), 32768);
/// ```
pub fn quantize_size(size: usize) -> usize {
    *SIZE_CLASSES
        .iter()
        .find(|&&s| s >= size)
        .unwrap_or(&MAX_SIZE_CLASS)
}

/// Zufällige Size-Class wählen (für Cover-Traffic)
pub fn random_size_class() -> usize {
    let mut rng = rand::thread_rng();
    SIZE_CLASSES[rng.gen_range(0..SIZE_CLASSES.len())]
}

// ============================================================================
// PEER TYPE (RL18)
// ============================================================================

/// Peer-Typ für Cover-Rate (RL18)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PeerType {
    /// Full-Relay: 0.2/s - 12 Dummies/Minute
    FullRelay,
    /// Apprentice-Relay: 0.1/s - 6 Dummies/Minute
    ApprenticeRelay,
    /// Active-User: 0.05/s - 3 Dummies/Minute
    ActiveUser,
    /// Passive-User: 0.01/s - 0.6 Dummies/Minute
    PassiveUser,
}

impl PeerType {
    /// Minimale Cover-Rate λ_min (RL18)
    ///
    /// ```text
    /// λ_min(Full-Relay) = 0.2/s
    /// λ_min(Apprentice-Relay) = 0.1/s
    /// λ_min(Active-User) = 0.05/s
    /// λ_min(Passive-User) = 0.01/s
    /// ```
    pub fn min_rate(&self) -> f64 {
        match self {
            Self::FullRelay => 0.2,
            Self::ApprenticeRelay => 0.1,
            Self::ActiveUser => 0.05,
            Self::PassiveUser => 0.01,
        }
    }

    /// Durchschnittliche Inter-Arrival-Zeit (1/λ)
    pub fn mean_interval(&self) -> Duration {
        let lambda = self.min_rate();
        if lambda == 0.0 {
            return Duration::from_secs(3600); // Fallback: 1 Stunde
        }
        Duration::from_secs_f64(1.0 / lambda)
    }

    /// Name für Logging
    pub fn name(&self) -> &'static str {
        match self {
            Self::FullRelay => "full-relay",
            Self::ApprenticeRelay => "apprentice-relay",
            Self::ActiveUser => "active-user",
            Self::PassiveUser => "passive-user",
        }
    }
}

impl Default for PeerType {
    fn default() -> Self {
        Self::ActiveUser
    }
}

// ============================================================================
// COVER TRAFFIC CONFIG
// ============================================================================

/// Cover-Traffic Konfiguration (RL18: Protocol Pledge)
#[derive(Debug, Clone)]
pub struct CoverTrafficConfig {
    /// Peer-Typ bestimmt minimale Rate
    pub peer_type: PeerType,
    /// Overhead-Ratio ρ (cover/real) - optional extra Cover
    pub overhead_ratio: f64,
    /// Aktiviert/Deaktiviert
    pub enabled: bool,
}

impl Default for CoverTrafficConfig {
    fn default() -> Self {
        Self {
            peer_type: PeerType::ActiveUser,
            overhead_ratio: 1.0, // 1:1 Cover zu Real
            enabled: true,
        }
    }
}

impl CoverTrafficConfig {
    /// Relay-Konfiguration
    pub fn for_relay() -> Self {
        Self {
            peer_type: PeerType::FullRelay,
            overhead_ratio: 1.5,
            enabled: true,
        }
    }

    /// Apprentice-Konfiguration
    pub fn for_apprentice() -> Self {
        Self {
            peer_type: PeerType::ApprenticeRelay,
            overhead_ratio: 1.2,
            enabled: true,
        }
    }

    /// Mobile/Low-Power-Konfiguration
    pub fn for_mobile() -> Self {
        Self {
            peer_type: PeerType::PassiveUser,
            overhead_ratio: 0.5,
            enabled: true,
        }
    }

    /// Effektive Rate (λ_min × overhead_ratio)
    pub fn effective_rate(&self) -> f64 {
        self.peer_type.min_rate() * self.overhead_ratio
    }

    /// Minimum Compliance-Ratio (unter dieser Rate wird gewarnt)
    pub fn min_compliance_ratio(&self) -> f64 {
        COMPLIANCE_WARNING_THRESHOLD
    }
}

// ============================================================================
// COVER MESSAGE
// ============================================================================

/// Generierte Cover-Nachricht
#[derive(Debug)]
pub struct CoverMessage {
    /// Padding auf Size-Class (CSPRNG-gefüllt)
    pub payload: Vec<u8>,
    /// Zufällige Route (gültig, aber Egress verwirft)
    pub route: Vec<PeerId>,
    /// Flag für Egress (nur intern erkennbar)
    pub is_dummy: bool,
    /// Size-Class die verwendet wurde
    pub size_class: usize,
    /// Generierungszeitpunkt
    pub created_at: Instant,
    /// Boost-Request Flag (für Compliance-Wiederherstellung)
    pub is_boost_request: bool,
}

impl CoverMessage {
    /// Erstelle neue Cover-Nachricht mit zufälliger Size-Class
    pub fn new_random(route: Vec<PeerId>) -> Self {
        let size_class = random_size_class();
        Self::new_with_size(route, size_class)
    }

    /// Erstelle Cover-Nachricht mit spezifischer Size-Class
    pub fn new_with_size(route: Vec<PeerId>, size_class: usize) -> Self {
        let quantized = quantize_size(size_class);

        // CSPRNG-Payload
        let mut payload = vec![0u8; quantized];
        getrandom::getrandom(&mut payload).expect("RNG failed");

        Self {
            payload,
            route,
            is_dummy: true,
            size_class: quantized,
            created_at: Instant::now(),
            is_boost_request: false,
        }
    }

    /// Erstelle Boost-Request (signalisiert dem Generator mehr Cover-Traffic zu senden)
    pub fn new_boost_request() -> Self {
        Self {
            payload: vec![],
            route: vec![],
            is_dummy: true,
            size_class: 0,
            created_at: Instant::now(),
            is_boost_request: true,
        }
    }
}

// ============================================================================
// COVER TRAFFIC STATISTICS
// ============================================================================

/// Cover-Traffic Statistiken für Compliance-Monitoring (RL18)
#[derive(Debug, Clone)]
pub struct CoverTrafficStats {
    /// Gesendete Cover-Nachrichten
    pub cover_sent: u64,
    /// Gesendete echte Nachrichten
    pub real_sent: u64,
    /// Zeitraum der Beobachtung (Sekunden)
    pub observation_secs: f64,
    /// Start-Zeitpunkt
    observation_start: Option<Instant>,
}

impl Default for CoverTrafficStats {
    fn default() -> Self {
        Self {
            cover_sent: 0,
            real_sent: 0,
            observation_secs: 0.0,
            observation_start: None,
        }
    }
}

impl CoverTrafficStats {
    /// Starte Beobachtungszeitraum
    pub fn start_observation(&mut self) {
        self.observation_start = Some(Instant::now());
    }

    /// Aktualisiere Beobachtungszeit
    pub fn update_observation_time(&mut self) {
        if let Some(start) = self.observation_start {
            self.observation_secs = start.elapsed().as_secs_f64();
        }
    }

    /// Beobachtete Cover-Rate
    pub fn observed_cover_rate(&self) -> f64 {
        if self.observation_secs == 0.0 {
            return 0.0;
        }
        self.cover_sent as f64 / self.observation_secs
    }

    /// Berechne Compliance-Ratio (RL18)
    ///
    /// ```text
    /// compliance = observed_rate / expected_rate
    /// ```
    pub fn compliance_ratio(&self, expected_rate: f64) -> f64 {
        if expected_rate == 0.0 {
            return 1.0;
        }

        let observed = self.observed_cover_rate();
        (observed / expected_rate).min(1.5) // Cap bei 150%
    }
}

// ============================================================================
// COMPLIANCE RESULT
// ============================================================================

/// Compliance-Prüfergebnis (RL18)
#[derive(Debug, Clone)]
pub enum ComplianceResult {
    /// Unbekannt (zu wenig Daten)
    Unknown,
    /// Compliant (ratio >= 0.8)
    Compliant {
        /// Compliance-Ratio (0.0 - 1.5)
        ratio: f64,
    },
    /// Warnung (0.5 <= ratio < 0.8)
    Warning {
        /// Deficit (1.0 - ratio)
        deficit: f64,
        /// Trust-Penalty für V-Dimension
        trust_penalty_v: f64,
        /// Trust-Penalty für Ω-Dimension
        trust_penalty_omega: f64,
    },
    /// Violation (ratio < 0.5)
    Violation {
        /// Deficit
        deficit: f64,
        /// Downgrade auf niedrigeres Level empfohlen
        downgrade_level: bool,
    },
}

impl ComplianceResult {
    /// Ist compliant?
    pub fn is_compliant(&self) -> bool {
        matches!(self, Self::Compliant { .. })
    }

    /// Ist Violation?
    pub fn is_violation(&self) -> bool {
        matches!(self, Self::Violation { .. })
    }
}

// ============================================================================
// COMPLIANCE MONITOR
// ============================================================================

/// Compliance-Monitor für Cover-Traffic (RL18)
pub struct ComplianceMonitor {
    /// Beobachtete Peers
    peers: RwLock<HashMap<PeerId, CoverTrafficStats>>,
    /// Beobachtungszeitraum
    observation_period: Duration,
}

impl ComplianceMonitor {
    /// Erstelle neuen Compliance-Monitor
    pub fn new(observation_period: Duration) -> Self {
        Self {
            peers: RwLock::new(HashMap::new()),
            observation_period,
        }
    }

    /// Registriere Peer für Monitoring
    pub fn register_peer(&self, peer_id: PeerId) {
        let mut peers = self.peers.write();
        if !peers.contains_key(&peer_id) {
            let mut stats = CoverTrafficStats::default();
            stats.start_observation();
            peers.insert(peer_id, stats);
        }
    }

    /// Registriere gesendete Cover-Nachricht
    pub fn record_cover_sent(&self, peer_id: &PeerId) {
        let mut peers = self.peers.write();
        if let Some(stats) = peers.get_mut(peer_id) {
            stats.cover_sent += 1;
            stats.update_observation_time();
        }
    }

    /// Registriere gesendete echte Nachricht
    pub fn record_real_sent(&self, peer_id: &PeerId) {
        let mut peers = self.peers.write();
        if let Some(stats) = peers.get_mut(peer_id) {
            stats.real_sent += 1;
            stats.update_observation_time();
        }
    }

    /// Prüfe Compliance eines Peers (RL18)
    pub fn check_compliance(&self, peer_id: &PeerId, peer_type: PeerType) -> ComplianceResult {
        let peers = self.peers.read();
        let stats = match peers.get(peer_id) {
            Some(s) => s.clone(),
            None => return ComplianceResult::Unknown,
        };

        // Mindestens 1 Stunde Beobachtung nötig
        if stats.observation_secs < 3600.0 {
            return ComplianceResult::Unknown;
        }

        let expected_rate = peer_type.min_rate();
        let compliance = stats.compliance_ratio(expected_rate);

        if compliance >= COMPLIANCE_WARNING_THRESHOLD {
            ComplianceResult::Compliant { ratio: compliance }
        } else if compliance >= COMPLIANCE_VIOLATION_THRESHOLD {
            // RL18: Abgestuftes Penalty-System
            let deficit = 1.0 - compliance;
            let days = stats.observation_secs / 86400.0;

            ComplianceResult::Warning {
                deficit,
                trust_penalty_v: 0.02 * deficit * days,
                trust_penalty_omega: 0.03 * deficit * days,
            }
        } else {
            ComplianceResult::Violation {
                deficit: 1.0 - compliance,
                downgrade_level: true,
            }
        }
    }

    /// Hole Stats für einen Peer
    pub fn get_stats(&self, peer_id: &PeerId) -> Option<CoverTrafficStats> {
        self.peers.read().get(peer_id).cloned()
    }

    /// Anzahl überwachter Peers
    pub fn peer_count(&self) -> usize {
        self.peers.read().len()
    }

    /// Prüfe eigene Compliance (für PrivacyService)
    pub fn check_self_compliance(
        &self,
        expected_rate: f64,
        actual_rate: f64,
        min_ratio: f64,
    ) -> SelfComplianceResult {
        let ratio = if expected_rate > 0.0 {
            actual_rate / expected_rate
        } else {
            1.0
        };

        SelfComplianceResult {
            is_compliant: ratio >= min_ratio,
            deficit: (min_ratio - ratio).max(0.0),
        }
    }

    /// Aktualisiere mit Cover-Generator-Stats (für Self-Monitoring)
    pub fn record_cover_stats(&self, _stats: &CoverGeneratorStats) {
        // Speichere für spätere Analyse
        // In einer vollständigen Implementierung würde hier ein Zeitreihen-Buffer verwendet
    }

    /// Hole aktuellen Status (für PrivacyServiceStats)
    pub fn current_status(&self) -> ComplianceStatus {
        let peers = self.peers.read();
        let compliant = peers
            .values()
            .filter(|s| s.compliance_ratio(0.1) >= 0.8)
            .count();

        ComplianceStatus {
            monitored_peers: peers.len(),
            compliant_peers: compliant,
            non_compliant_peers: peers.len() - compliant,
            self_compliant: true, // Default bis Self-Monitoring implementiert
            self_rate: 0.0,
            last_cover_stats: None,
        }
    }
}

impl Default for ComplianceMonitor {
    fn default() -> Self {
        Self::new(Duration::from_secs(DEFAULT_OBSERVATION_PERIOD_SECS))
    }
}

/// Compliance-Check-Ergebnis für Service-interne Nutzung
#[derive(Debug, Clone)]
pub struct SelfComplianceResult {
    /// Ist compliant?
    pub is_compliant: bool,
    /// Deficit (wie viel fehlt)
    pub deficit: f64,
}

/// Aktueller Compliance-Status (für Stats)
#[derive(Debug, Clone, Default)]
pub struct ComplianceStatus {
    /// Gesamtzahl überwachter Peers
    pub monitored_peers: usize,
    /// Anzahl complianter Peers
    pub compliant_peers: usize,
    /// Anzahl nicht-complianter Peers
    pub non_compliant_peers: usize,
    /// Eigene Compliance (Self-Monitoring)
    pub self_compliant: bool,
    /// Eigene Rate
    pub self_rate: f64,
    /// Letzte Cover-Stats
    pub last_cover_stats: Option<CoverGeneratorStats>,
}

// ============================================================================
// COVER TRAFFIC GENERATOR
// ============================================================================

/// Cover-Traffic Generator (RL10, RL18)
pub struct CoverTrafficGenerator {
    /// Konfiguration
    config: CoverTrafficConfig,
    /// Channel für generierte Dummies
    output_tx: mpsc::Sender<CoverMessage>,
    /// Gesendete Cover-Nachrichten
    cover_sent: AtomicU64,
    /// Start-Zeitpunkt
    started_at: Instant,
}

impl CoverTrafficGenerator {
    /// Erstelle neuen Cover-Traffic Generator
    pub fn new(config: CoverTrafficConfig, output_tx: mpsc::Sender<CoverMessage>) -> Self {
        Self {
            config,
            output_tx,
            cover_sent: AtomicU64::new(0),
            started_at: Instant::now(),
        }
    }

    /// Generiere eine Dummy-Nachricht (RL10)
    fn generate_dummy(&self, route: Vec<PeerId>) -> CoverMessage {
        CoverMessage::new_random(route)
    }

    /// Exponentieller Delay für Poisson-Prozess
    ///
    /// ```text
    /// delay = -1/λ × ln(U), U ~ Uniform(0, 1)
    /// ```
    fn sample_poisson_delay(&self) -> Duration {
        let lambda = self.config.effective_rate();
        if lambda == 0.0 {
            return Duration::from_secs(3600);
        }

        let u: f64 = rand::thread_rng().gen();
        // Vermeiden von ln(0)
        let u_clamped = u.max(1e-10);
        let delay_secs = -1.0 / lambda * u_clamped.ln();

        Duration::from_secs_f64(delay_secs.abs())
    }

    /// Starte Cover-Traffic Loop (RL18)
    ///
    /// Generiert Dummy-Nachrichten nach Poisson-Prozess mit Rate λ_min.
    pub async fn run<F>(&self, route_generator: F)
    where
        F: Fn() -> Vec<PeerId> + Send + Sync + 'static,
    {
        if !self.config.enabled {
            tracing::info!("Cover-Traffic disabled");
            return;
        }

        tracing::info!(
            peer_type = self.config.peer_type.name(),
            rate = self.config.effective_rate(),
            "Starting cover-traffic generator"
        );

        loop {
            // Exponential-Delay für Poisson-Prozess
            let delay = self.sample_poisson_delay();
            tokio::time::sleep(delay).await;

            // Generiere und sende Dummy
            let route = route_generator();
            if route.is_empty() {
                tracing::trace!("No route available for cover traffic");
                continue;
            }

            let dummy = self.generate_dummy(route);
            let size = dummy.size_class;

            if self.output_tx.send(dummy).await.is_err() {
                tracing::warn!("Cover-traffic output channel closed");
                break;
            }

            self.cover_sent.fetch_add(1, Ordering::Relaxed);

            tracing::trace!(
                size_class = size,
                total_sent = self.cover_sent.load(Ordering::Relaxed),
                "Cover message sent"
            );
        }
    }

    /// Hole Statistiken
    pub fn stats(&self) -> CoverGeneratorStats {
        let elapsed = self.started_at.elapsed();
        let sent = self.cover_sent.load(Ordering::Relaxed);

        CoverGeneratorStats {
            cover_sent: sent,
            elapsed_secs: elapsed.as_secs_f64(),
            effective_rate: if elapsed.as_secs() > 0 {
                sent as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            },
            config_rate: self.config.effective_rate(),
        }
    }
}

/// Generator-Statistiken
#[derive(Debug, Clone)]
pub struct CoverGeneratorStats {
    /// Gesendete Cover-Nachrichten
    pub cover_sent: u64,
    /// Vergangene Zeit (Sekunden)
    pub elapsed_secs: f64,
    /// Effektive Rate (msg/s)
    pub effective_rate: f64,
    /// Konfigurierte Rate
    pub config_rate: f64,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantize_size() {
        assert_eq!(quantize_size(100), 256);
        assert_eq!(quantize_size(256), 256);
        assert_eq!(quantize_size(257), 512);
        assert_eq!(quantize_size(1000), 1024);
        assert_eq!(quantize_size(50000), 32768); // Max-Clamp
    }

    #[test]
    fn test_peer_type_rates() {
        assert!((PeerType::FullRelay.min_rate() - 0.2).abs() < 0.001);
        assert!((PeerType::ApprenticeRelay.min_rate() - 0.1).abs() < 0.001);
        assert!((PeerType::ActiveUser.min_rate() - 0.05).abs() < 0.001);
        assert!((PeerType::PassiveUser.min_rate() - 0.01).abs() < 0.001);
    }

    #[test]
    fn test_cover_message_creation() {
        let route = vec![PeerId::random(), PeerId::random()];
        let msg = CoverMessage::new_random(route.clone());

        assert!(msg.is_dummy);
        assert!(SIZE_CLASSES.contains(&msg.size_class));
        assert_eq!(msg.payload.len(), msg.size_class);
        assert_eq!(msg.route.len(), 2);
    }

    #[test]
    fn test_cover_message_specific_size() {
        let route = vec![PeerId::random()];
        let msg = CoverMessage::new_with_size(route, 1000);

        assert_eq!(msg.size_class, 1024); // Quantized
        assert_eq!(msg.payload.len(), 1024);
    }

    #[test]
    fn test_compliance_stats() {
        let mut stats = CoverTrafficStats::default();
        stats.start_observation();

        // Simuliere 100 Cover-Nachrichten in 1000 Sekunden
        stats.cover_sent = 100;
        stats.observation_secs = 1000.0;

        // Rate = 0.1/s
        let observed = stats.observed_cover_rate();
        assert!((observed - 0.1).abs() < 0.001);

        // Compliance bei expected_rate = 0.1
        let compliance = stats.compliance_ratio(0.1);
        assert!((compliance - 1.0).abs() < 0.001);

        // Compliance bei expected_rate = 0.2 (50%)
        let compliance = stats.compliance_ratio(0.2);
        assert!((compliance - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_compliance_monitor() {
        let monitor = ComplianceMonitor::default();
        let peer_id = PeerId::random();

        // Registriere Peer
        monitor.register_peer(peer_id);
        assert_eq!(monitor.peer_count(), 1);

        // Initial: Unknown (zu wenig Zeit)
        let result = monitor.check_compliance(&peer_id, PeerType::ActiveUser);
        assert!(matches!(result, ComplianceResult::Unknown));

        // Record einige Cover-Nachrichten
        for _ in 0..10 {
            monitor.record_cover_sent(&peer_id);
        }

        let stats = monitor.get_stats(&peer_id).unwrap();
        assert_eq!(stats.cover_sent, 10);
    }

    #[test]
    fn test_compliance_result_states() {
        // Compliant
        let compliant = ComplianceResult::Compliant { ratio: 0.95 };
        assert!(compliant.is_compliant());
        assert!(!compliant.is_violation());

        // Warning
        let warning = ComplianceResult::Warning {
            deficit: 0.3,
            trust_penalty_v: 0.006,
            trust_penalty_omega: 0.009,
        };
        assert!(!warning.is_compliant());
        assert!(!warning.is_violation());

        // Violation
        let violation = ComplianceResult::Violation {
            deficit: 0.6,
            downgrade_level: true,
        };
        assert!(!violation.is_compliant());
        assert!(violation.is_violation());
    }

    #[test]
    fn test_config_presets() {
        let relay = CoverTrafficConfig::for_relay();
        assert_eq!(relay.peer_type, PeerType::FullRelay);

        let mobile = CoverTrafficConfig::for_mobile();
        assert_eq!(mobile.peer_type, PeerType::PassiveUser);
        assert!(mobile.overhead_ratio < 1.0);
    }

    #[tokio::test]
    async fn test_cover_generator_creation() {
        let (tx, _rx) = mpsc::channel(100);
        let config = CoverTrafficConfig::default();
        let generator = CoverTrafficGenerator::new(config, tx);

        let stats = generator.stats();
        assert_eq!(stats.cover_sent, 0);
        assert!(stats.config_rate > 0.0);
    }

    #[test]
    fn test_poisson_delay_sampling() {
        let (tx, _rx) = mpsc::channel(100);
        let config = CoverTrafficConfig {
            peer_type: PeerType::FullRelay, // 0.2/s = 5s mean
            overhead_ratio: 1.0,
            enabled: true,
        };
        let generator = CoverTrafficGenerator::new(config, tx);

        // Sample viele Delays
        let delays: Vec<Duration> = (0..100).map(|_| generator.sample_poisson_delay()).collect();

        // Alle sollten positiv sein
        assert!(delays.iter().all(|d| d.as_secs_f64() > 0.0));

        // Mittelwert sollte nahe 1/λ = 5s sein
        let mean: f64 = delays.iter().map(|d| d.as_secs_f64()).sum::<f64>() / 100.0;
        // Erlauben hohe Varianz bei nur 100 Samples
        assert!(mean > 1.0 && mean < 15.0);
    }

    #[test]
    fn test_size_classes_coverage() {
        // Test alle Size-Classes
        for &size in &SIZE_CLASSES {
            let msg = CoverMessage::new_with_size(vec![PeerId::random()], size);
            assert_eq!(msg.size_class, size);
            assert_eq!(msg.payload.len(), size);
        }
    }
}
