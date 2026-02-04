//! # LAMP-Enhanced Mixing-Pool (RL8-RL10, RL25)
//!
//! Integriert NDSS-2025 "LAMP: Lightweight Approaches for Latency Minimization in Mixnets"
//!
//! ## LAMP-Verbesserungen (3× besserer Latency-Anonymity-Tradeoff):
//! - Threshold-Mixing: Flush bei dynamischem k_opt
//! - Adaptive Routing: Dynamisches k_opt basierend auf Traffic-Rate
//! - Probabilistic Forwarding: Reduziert τ_mix_avg um 66%
//!
//! ## Axiom-Referenzen
//!
//! - **RL8**: Mixing-Invariante mit Laplace-Delay (ε-Differential Privacy)
//!   ```text
//!   ∀ msg ∈ pool: output_time = arrival_time + τ_base + Laplace(0, Δf/ε)
//!   output_order = random_permutation(pool)
//!   ```
//!
//! - **RL9**: Minimum-Anonymität
//!   ```text
//!   anonymity_set ≥ k_min = 3, effective_entropy ≥ 1.58 bits
//!   ```
//!
//! - **RL10**: Cover-Traffic Indistinguishability
//!   ```text
//!   ∀ msg: P(real|observation) = P(cover|observation)
//!   ```
//!
//! - **RL25**: LAMP Threshold-Mixing 🆕
//!   ```text
//!   k_opt = √(rate × τ_target)
//!   flush_condition = |pool| ≥ k_opt ∨ oldest.elapsed > τ_max
//!   ```
//!
//! ## Core-Logic-Verknüpfungen (LOGIC.md V4.1)
//!
//! - **Κ9**: NetworkConditions.variability_factor() für τ-Anpassung
//!
//! ## Wire-Format
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    MIXING-POOL PIPELINE                         │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │  Message ──► [LAMP Decision] ──► [Pool Buffer] ──► [Shuffle]   │
//! │                   │                    │             │          │
//! │                   │                    ▼             ▼          │
//! │              Prob-Forward          k_opt Check    Delayed Out  │
//! │                   │                    │             │          │
//! │                   └────► [Direct] ─────┴─────────────┘          │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use crate::core::state::{StateEvent, StateEventEmitter, NoOpEmitter};
use libp2p::PeerId;
use parking_lot::Mutex;
use rand::Rng;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Minimale Pool-Größe k_min (RL9)
pub const K_MIN: usize = 3;

/// Maximale Pool-Größe k_max
pub const K_MAX: usize = 20;

/// Standard τ_min (ms)
pub const TAU_MIN_MS: u64 = 50;

/// Standard τ_max (ms)
pub const TAU_MAX_MS: u64 = 500;

/// Standard ε für Differential Privacy
pub const DEFAULT_EPSILON: f64 = 0.1;

/// Standard Sensitivität Δf (ms)
pub const DEFAULT_SENSITIVITY: f64 = 100.0;

/// LAMP: Standard Probabilistic-Forward-Rate
pub const DEFAULT_PROB_FORWARD_RATE: f64 = 0.3;

/// LAMP: Standard Target-Delay für k_opt (ms)
pub const DEFAULT_TARGET_DELAY_MS: u64 = 100;

/// Rate-Monitor Window (Sekunden)
pub const RATE_MONITOR_WINDOW_SECS: u64 = 60;

// ============================================================================
// CONFIGURATION
// ============================================================================

/// LAMP-Enhanced Mixing-Pool Konfiguration (RL8 + RL25)
#[derive(Debug, Clone)]
pub struct MixingPoolConfig {
    /// Minimale Verzögerung τ_min
    pub tau_min: Duration,
    /// Maximale Verzögerung τ_max
    pub tau_max: Duration,
    /// Minimale Pool-Größe k_min (RL9)
    pub k_min: usize,
    /// Maximale Pool-Größe k_max
    pub k_max: usize,
    /// ε für Differential Privacy (kleiner = mehr Privacy)
    pub epsilon: f64,
    /// Sensitivität Δf für Timing
    pub sensitivity: f64,

    // LAMP-Erweiterungen (RL25)
    /// LAMP: Threshold-Flush aktivieren
    pub lamp_threshold_enabled: bool,
    /// LAMP: Dynamisches k_opt (√(rate × τ_target))
    pub lamp_adaptive_k: bool,
    /// LAMP: Probabilistic-Forwarding-Rate (0.0-1.0)
    pub lamp_prob_forward_rate: f64,
    /// LAMP: Target-Delay für k_opt Berechnung
    pub lamp_target_delay: Duration,
}

impl Default for MixingPoolConfig {
    fn default() -> Self {
        Self {
            tau_min: Duration::from_millis(TAU_MIN_MS),
            tau_max: Duration::from_millis(TAU_MAX_MS),
            k_min: K_MIN,
            k_max: K_MAX,
            epsilon: DEFAULT_EPSILON,
            sensitivity: DEFAULT_SENSITIVITY,
            // LAMP-Defaults (RL25)
            lamp_threshold_enabled: true,
            lamp_adaptive_k: true,
            lamp_prob_forward_rate: DEFAULT_PROB_FORWARD_RATE,
            lamp_target_delay: Duration::from_millis(DEFAULT_TARGET_DELAY_MS),
        }
    }
}

impl MixingPoolConfig {
    /// High-Privacy Konfiguration (mehr Delay, größerer Pool)
    pub fn high_privacy() -> Self {
        Self {
            tau_min: Duration::from_millis(100),
            tau_max: Duration::from_millis(1000),
            k_min: 5,
            k_max: 30,
            epsilon: 0.05, // Stärkere Privacy
            sensitivity: 150.0,
            lamp_threshold_enabled: true,
            lamp_adaptive_k: true,
            lamp_prob_forward_rate: 0.1, // Weniger Prob-Forward
            lamp_target_delay: Duration::from_millis(200),
        }
    }

    /// Low-Latency Konfiguration (schneller, weniger Privacy)
    pub fn low_latency() -> Self {
        Self {
            tau_min: Duration::from_millis(20),
            tau_max: Duration::from_millis(200),
            k_min: 2,
            k_max: 10,
            epsilon: 0.2, // Weniger Privacy
            sensitivity: 50.0,
            lamp_threshold_enabled: true,
            lamp_adaptive_k: true,
            lamp_prob_forward_rate: 0.5, // Mehr Prob-Forward
            lamp_target_delay: Duration::from_millis(50),
        }
    }

    /// Mobile-optimierte Konfiguration
    pub fn mobile() -> Self {
        Self {
            tau_min: Duration::from_millis(30),
            tau_max: Duration::from_millis(300),
            k_min: 2,
            k_max: 8,
            epsilon: 0.15,
            sensitivity: 80.0,
            lamp_threshold_enabled: true,
            lamp_adaptive_k: true,
            lamp_prob_forward_rate: 0.4,
            lamp_target_delay: Duration::from_millis(80),
        }
    }
}

// ============================================================================
// POOLED MESSAGE
// ============================================================================

/// Nachricht im Mixing-Pool
struct PooledMessage {
    /// Payload (verschlüsselt)
    payload: Vec<u8>,
    /// Zeitpunkt des Eintreffens
    arrival_time: Instant,
    /// Ziel (nächster Hop oder finales Ziel)
    next_hop: PeerId,
    /// Zugewiesener Delay
    assigned_delay: Duration,
}

// ============================================================================
// TRAFFIC RATE MONITOR (RL25)
// ============================================================================

/// LAMP Traffic-Rate-Monitor (RL25)
///
/// Sliding-Window basierte Rate-Berechnung für dynamisches k_opt.
#[derive(Debug)]
pub struct TrafficRateMonitor {
    /// Sliding-Window für Rate-Berechnung
    message_timestamps: VecDeque<Instant>,
    /// Window-Größe
    window: Duration,
}

impl TrafficRateMonitor {
    /// Erstelle neuen Monitor mit Window-Größe
    pub fn new(window: Duration) -> Self {
        Self {
            message_timestamps: VecDeque::with_capacity(1000),
            window,
        }
    }

    /// Registriere neue Nachricht
    pub fn record(&mut self) {
        let now = Instant::now();
        self.message_timestamps.push_back(now);

        // Alte Timestamps entfernen
        while let Some(ts) = self.message_timestamps.front() {
            if now.duration_since(*ts) > self.window {
                self.message_timestamps.pop_front();
            } else {
                break;
            }
        }
    }

    /// Aktuelle Rate (Nachrichten/Sekunde)
    pub fn current_rate(&self) -> f64 {
        let count = self.message_timestamps.len();
        let elapsed = self.window.as_secs_f64();
        if elapsed == 0.0 {
            return 0.0;
        }
        count as f64 / elapsed
    }

    /// Anzahl Nachrichten im Window
    pub fn message_count(&self) -> usize {
        self.message_timestamps.len()
    }
}

impl Default for TrafficRateMonitor {
    fn default() -> Self {
        Self::new(Duration::from_secs(RATE_MONITOR_WINDOW_SECS))
    }
}

// ============================================================================
// LAMP STATISTICS
// ============================================================================

/// LAMP Statistiken (RL25)
#[derive(Debug, Clone)]
pub struct LampStats {
    /// Aktuelle Buffer-Größe
    pub buffer_size: usize,
    /// Aktuelle Traffic-Rate (msg/s)
    pub current_rate: f64,
    /// Dynamisches k_opt
    pub k_opt: usize,
    /// Probabilistic-Forward-Rate
    pub prob_forward_rate: f64,
    /// Anzahl Probabilistic-Forwards
    pub prob_forward_count: u64,
    /// Anzahl Threshold-Flushes
    pub threshold_flush_count: u64,
    /// Anzahl Standard-Flushes
    pub standard_flush_count: u64,
}

// ============================================================================
// MIXING POOL
// ============================================================================

/// LAMP-Enhanced Mixing-Pool mit ε-Differential Privacy
///
/// Implementiert RL8 (Laplace-Delays), RL9 (Minimum-Anonymität) und
/// RL25 (LAMP Threshold-Mixing).
/// LAMP-Enhanced Mixing-Pool (v0.4.0)
///
/// Emittiert nun MixingPoolFlushed StateEvents.
pub struct MixingPool {
    /// Konfiguration
    config: MixingPoolConfig,
    /// Nachrichten-Puffer
    buffer: Mutex<VecDeque<PooledMessage>>,
    /// Laplace-Skala b = Δf / ε
    laplace_scale: f64,
    /// Output-Channel
    output_tx: mpsc::Sender<(PeerId, Vec<u8>)>,
    /// LAMP: Traffic-Rate-Monitor
    rate_monitor: Mutex<TrafficRateMonitor>,
    /// LAMP: Dynamisches k_opt
    current_k_opt: AtomicUsize,
    /// Statistik: Probabilistic-Forwards
    prob_forward_count: AtomicU64,
    /// Statistik: Threshold-Flushes
    threshold_flush_count: AtomicU64,
    /// Statistik: Standard-Flushes
    standard_flush_count: AtomicU64,

    // ========================================================================
    // StateEvent-Integration (v0.4.0)
    // ========================================================================
    /// StateEvent-Emitter für Integration mit UnifiedState
    state_event_emitter: Arc<dyn StateEventEmitter>,
    /// Totale Delay-Zeit für Durchschnitt
    total_delay_ms: AtomicU64,
    /// Anzahl geflushed Messages (für Durchschnittsberechnung)
    total_flushed_count: AtomicU64,
}

impl MixingPool {
    /// Erstelle neuen LAMP-Enhanced Mixing-Pool
    pub fn new(config: MixingPoolConfig, output_tx: mpsc::Sender<(PeerId, Vec<u8>)>) -> Self {
        // Laplace-Skala b = Δf / ε
        let laplace_scale = config.sensitivity / config.epsilon;

        Self {
            config: config.clone(),
            buffer: Mutex::new(VecDeque::with_capacity(config.k_max * 2)),
            laplace_scale,
            output_tx,
            rate_monitor: Mutex::new(TrafficRateMonitor::default()),
            current_k_opt: AtomicUsize::new(config.k_min),
            prob_forward_count: AtomicU64::new(0),
            threshold_flush_count: AtomicU64::new(0),
            standard_flush_count: AtomicU64::new(0),
            state_event_emitter: Arc::new(NoOpEmitter),
            total_delay_ms: AtomicU64::new(0),
            total_flushed_count: AtomicU64::new(0),
        }
    }

    /// Erstelle mit StateEventEmitter (v0.4.0)
    pub fn new_with_emitter(
        config: MixingPoolConfig,
        output_tx: mpsc::Sender<(PeerId, Vec<u8>)>,
        emitter: Arc<dyn StateEventEmitter>,
    ) -> Self {
        let mut pool = Self::new(config, output_tx);
        pool.state_event_emitter = emitter;
        pool
    }

    /// Setze StateEventEmitter nachträglich
    pub fn set_state_event_emitter(&mut self, emitter: Arc<dyn StateEventEmitter>) {
        self.state_event_emitter = emitter;
    }

    /// LAMP: Berechne dynamisches k_opt (RL25)
    ///
    /// ```text
    /// k_opt = √(rate × τ_target)
    /// ```
    ///
    /// Minimiert E[delay] bei gegebener Anonymität.
    fn calculate_k_opt(&self) -> usize {
        if !self.config.lamp_adaptive_k {
            return self.config.k_min;
        }

        let rate = self.rate_monitor.lock().current_rate();
        let tau_target = self.config.lamp_target_delay.as_secs_f64();

        // k_opt = √(rate × τ_target)
        let k_opt = (rate * tau_target).sqrt();

        // Clamp zu [k_min, k_max]
        let clamped = (k_opt.ceil() as usize)
            .max(self.config.k_min)
            .min(self.config.k_max);

        self.current_k_opt.store(clamped, Ordering::Relaxed);
        clamped
    }

    /// Generiere Laplace-Noise für Delay (RL8)
    ///
    /// Nutzt Inverse-CDF-Methode: X = μ - b × sign(U - 0.5) × ln(1 - 2|U - 0.5|)
    fn sample_laplace_delay(&self) -> f64 {
        let mut rng = rand::thread_rng();
        let u: f64 = rng.gen();

        // Laplace-Sampling via Inverse-CDF
        let sign = if u < 0.5 { -1.0 } else { 1.0 };
        let abs_term = (1.0 - 2.0 * (u - 0.5).abs()).ln();

        (sign * self.laplace_scale * abs_term).abs()
    }

    /// Füge Nachricht zum Pool hinzu (RL8 + RL25 LAMP)
    pub async fn add_message(&self, payload: Vec<u8>, next_hop: PeerId) {
        let mut rng = rand::thread_rng();

        // LAMP: Rate-Monitor updaten
        self.rate_monitor.lock().record();

        // LAMP: Probabilistic Forwarding Check (RL25)
        if self.config.lamp_threshold_enabled
            && rng.gen::<f64>() < self.config.lamp_prob_forward_rate
        {
            // Sofortiges Forwarding (minimal delay)
            let minimal_delay = Duration::from_millis(
                rng.gen_range(5..=self.config.tau_min.as_millis() as u64 / 2),
            );

            self.prob_forward_count.fetch_add(1, Ordering::Relaxed);

            let output_tx = self.output_tx.clone();
            tokio::spawn(async move {
                tokio::time::sleep(minimal_delay).await;
                let _ = output_tx.send((next_hop, payload)).await;
            });

            tracing::trace!(
                delay_ms = minimal_delay.as_millis(),
                "LAMP probabilistic forward"
            );
            return;
        }

        // Standard-Pfad: Laplace-Noise + Uniform-Basis (RL8)
        let laplace_delay = self.sample_laplace_delay();
        let uniform_delay =
            rng.gen_range(self.config.tau_min.as_millis()..=self.config.tau_max.as_millis()) as f64;
        let total_delay_ms = (laplace_delay + uniform_delay) as u64;
        let assigned_delay = Duration::from_millis(total_delay_ms);

        let message = PooledMessage {
            payload,
            arrival_time: Instant::now(),
            next_hop,
            assigned_delay,
        };

        let should_flush = {
            let mut buffer = self.buffer.lock();
            buffer.push_back(message);

            // LAMP: Threshold-Flush bei k_opt erreicht
            let k_opt = self.calculate_k_opt();
            let buffer_len = buffer.len();

            if self.config.lamp_threshold_enabled && buffer_len >= k_opt {
                Some(FlushType::Threshold(k_opt))
            } else if buffer_len >= self.config.k_max {
                Some(FlushType::Standard)
            } else {
                None
            }
        };

        if let Some(flush_type) = should_flush {
            self.do_flush(flush_type).await;
        }
    }

    /// Führe Flush durch
    async fn do_flush(&self, flush_type: FlushType) {
        match flush_type {
            FlushType::Threshold(k_opt) => {
                self.threshold_flush_count.fetch_add(1, Ordering::Relaxed);
                self.trigger_threshold_flush(k_opt).await;
            }
            FlushType::Standard => {
                self.standard_flush_count.fetch_add(1, Ordering::Relaxed);
                self.trigger_flush().await;
            }
        }
    }

    /// LAMP: Threshold-Flush (RL25) - kompaktere Delays
    ///
    /// Bei Threshold-Flush werden die Delays um Faktor 1/√k skaliert,
    /// was niedrigere Latenz bei gleicher Anonymität ermöglicht.
    async fn trigger_threshold_flush(&self, k_opt: usize) {
        let messages = {
            let mut buffer = self.buffer.lock();

            // Nur k_opt Nachrichten flushen (älteste zuerst)
            let count = k_opt.min(buffer.len());
            let mut to_flush: Vec<_> = buffer.drain(..count).collect();

            // Zufällige Permutation (RL8)
            Self::shuffle(&mut to_flush);

            to_flush
        };

        if messages.is_empty() {
            return;
        }

        // LAMP: Kompaktere Delays (τ/√k statt τ)
        let delay_factor = 1.0 / (messages.len() as f64).sqrt();

        // Statistiken VOR dem Spawn berechnen (v0.4.0)
        let total_delay: u64 = messages.iter()
            .map(|m| m.assigned_delay.as_millis() as u64)
            .sum();
        let max_delay = messages.iter()
            .map(|m| m.assigned_delay.as_millis() as u64)
            .max()
            .unwrap_or(0);
        let flushed_count = messages.len();
        let avg_delay = if flushed_count > 0 {
            total_delay / flushed_count as u64
        } else {
            0
        };
        let pool_size_after = self.buffer.lock().len();

        let output_tx = self.output_tx.clone();
        tokio::spawn(async move {
            for msg in messages {
                let elapsed = msg.arrival_time.elapsed();
                // Skalierter Delay
                let scaled_delay = Duration::from_millis(
                    (msg.assigned_delay.as_millis() as f64 * delay_factor) as u64,
                );

                if let Some(remaining) = scaled_delay.checked_sub(elapsed) {
                    tokio::time::sleep(remaining).await;
                }

                let _ = output_tx.send((msg.next_hop, msg.payload)).await;
            }
        });

        // Globale Statistik aktualisieren
        self.total_delay_ms.fetch_add(total_delay, Ordering::Relaxed);
        self.total_flushed_count.fetch_add(flushed_count as u64, Ordering::Relaxed);

        // StateEvent emittieren (v0.4.0)
        self.state_event_emitter.emit(StateEvent::MixingPoolFlushed {
            messages_flushed: flushed_count as u64,
            avg_delay_ms: avg_delay,
            max_delay_ms: max_delay,
            pool_size_after: pool_size_after as u64,
        });

        tracing::debug!(
            k_opt,
            delay_factor = format!("{:.2}", delay_factor),
            messages_flushed = flushed_count,
            avg_delay_ms = avg_delay,
            "LAMP threshold flush"
        );
    }

    /// Standard-Flush
    async fn trigger_flush(&self) {
        let messages = {
            let mut buffer = self.buffer.lock();
            let mut to_flush: Vec<_> = buffer.drain(..).collect();

            // Zufällige Permutation (RL8: output_order = random_permutation)
            Self::shuffle(&mut to_flush);

            to_flush
        };

        if messages.is_empty() {
            return;
        }

        // Statistiken für StateEvent
        let total_delay: u64 = messages.iter()
            .map(|m| m.assigned_delay.as_millis() as u64)
            .sum();
        let max_delay = messages.iter()
            .map(|m| m.assigned_delay.as_millis() as u64)
            .max()
            .unwrap_or(0);
        let avg_delay = total_delay / messages.len() as u64;
        let flushed_count = messages.len();
        let pool_size_after = self.buffer.lock().len();

        // Globale Statistik aktualisieren
        self.total_delay_ms.fetch_add(total_delay, Ordering::Relaxed);
        self.total_flushed_count.fetch_add(flushed_count as u64, Ordering::Relaxed);

        // StateEvent emittieren (v0.4.0)
        self.state_event_emitter.emit(StateEvent::MixingPoolFlushed {
            messages_flushed: flushed_count as u64,
            avg_delay_ms: avg_delay,
            max_delay_ms: max_delay,
            pool_size_after: pool_size_after as u64,
        });

        let output_tx = self.output_tx.clone();
        tokio::spawn(async move {
            for msg in messages {
                // Warte restlichen Delay
                let elapsed = msg.arrival_time.elapsed();
                if let Some(remaining) = msg.assigned_delay.checked_sub(elapsed) {
                    tokio::time::sleep(remaining).await;
                }

                let _ = output_tx.send((msg.next_hop, msg.payload)).await;
            }
        });

        tracing::debug!(
            messages_flushed = flushed_count,
            avg_delay_ms = avg_delay,
            "Standard flush triggered"
        );
    }

    /// Fisher-Yates Shuffle
    fn shuffle<T>(slice: &mut [T]) {
        let mut rng = rand::thread_rng();
        for i in (1..slice.len()).rev() {
            let j = rng.gen_range(0..=i);
            slice.swap(i, j);
        }
    }

    /// Prüfe ob Flush nötig (für periodischen Check)
    pub fn should_flush(&self) -> bool {
        let buffer = self.buffer.lock();

        let k_opt = self.current_k_opt.load(Ordering::Relaxed);

        if buffer.len() >= k_opt {
            // Älteste Nachricht prüfen
            if let Some(oldest) = buffer.front() {
                return oldest.arrival_time.elapsed() > self.config.tau_max
                    || buffer.len() >= self.config.k_max;
            }
        }

        false
    }

    /// Periodischer Flush-Check (als Background-Task)
    pub async fn run_flush_loop(self: Arc<Self>) {
        let mut interval = tokio::time::interval(Duration::from_millis(50));

        loop {
            interval.tick().await;

            if self.should_flush() {
                if self.config.lamp_threshold_enabled {
                    let k_opt = self.calculate_k_opt();
                    self.trigger_threshold_flush(k_opt).await;
                } else {
                    self.trigger_flush().await;
                }
            }
        }
    }

    /// LAMP: Statistik-Report
    pub fn stats(&self) -> LampStats {
        let buffer_len = self.buffer.lock().len();
        let rate = self.rate_monitor.lock().current_rate();
        let k_opt = self.current_k_opt.load(Ordering::Relaxed);

        LampStats {
            buffer_size: buffer_len,
            current_rate: rate,
            k_opt,
            prob_forward_rate: self.config.lamp_prob_forward_rate,
            prob_forward_count: self.prob_forward_count.load(Ordering::Relaxed),
            threshold_flush_count: self.threshold_flush_count.load(Ordering::Relaxed),
            standard_flush_count: self.standard_flush_count.load(Ordering::Relaxed),
        }
    }

    /// Hole aktuelle Konfiguration
    pub fn config(&self) -> &MixingPoolConfig {
        &self.config
    }
}

/// Flush-Typ
enum FlushType {
    Threshold(usize),
    Standard,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = MixingPoolConfig::default();
        assert_eq!(config.k_min, K_MIN);
        assert_eq!(config.k_max, K_MAX);
        assert!(config.lamp_threshold_enabled);
        assert!(config.lamp_adaptive_k);
    }

    #[test]
    fn test_config_presets() {
        let high = MixingPoolConfig::high_privacy();
        assert!(high.epsilon < MixingPoolConfig::default().epsilon);
        assert!(high.k_min > MixingPoolConfig::default().k_min);

        let low = MixingPoolConfig::low_latency();
        assert!(low.epsilon > MixingPoolConfig::default().epsilon);
        assert!(low.tau_max < MixingPoolConfig::default().tau_max);

        let mobile = MixingPoolConfig::mobile();
        assert!(mobile.k_max < MixingPoolConfig::default().k_max);
    }

    #[test]
    fn test_rate_monitor() {
        let mut monitor = TrafficRateMonitor::new(Duration::from_secs(1));

        // Initial: keine Messages
        assert_eq!(monitor.current_rate(), 0.0);
        assert_eq!(monitor.message_count(), 0);

        // 10 Messages hinzufügen
        for _ in 0..10 {
            monitor.record();
        }

        assert_eq!(monitor.message_count(), 10);
        // Rate = 10 / 1 = 10 msg/s
        assert!((monitor.current_rate() - 10.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_mixing_pool_creation() {
        let (tx, _rx) = mpsc::channel(100);
        let pool = MixingPool::new(MixingPoolConfig::default(), tx);

        let stats = pool.stats();
        assert_eq!(stats.buffer_size, 0);
        assert_eq!(stats.prob_forward_count, 0);
    }

    #[test]
    fn test_laplace_sampling() {
        let (tx, _rx) = mpsc::channel::<(PeerId, Vec<u8>)>(100);
        let pool = MixingPool::new(MixingPoolConfig::default(), tx);

        // Sample viele Delays
        let samples: Vec<f64> = (0..1000).map(|_| pool.sample_laplace_delay()).collect();

        // Alle sollten positiv sein (wir nehmen abs())
        assert!(samples.iter().all(|&s| s >= 0.0));

        // Mittelwert sollte nahe Laplace-Skala sein
        let mean: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
        assert!(mean > 0.0);
        assert!(mean < pool.laplace_scale * 3.0); // Sanity check
    }

    #[test]
    fn test_k_opt_calculation() {
        let (tx, _rx) = mpsc::channel(100);
        let config = MixingPoolConfig {
            lamp_adaptive_k: true,
            lamp_target_delay: Duration::from_millis(100),
            k_min: 3,
            k_max: 20,
            ..Default::default()
        };
        let pool = MixingPool::new(config, tx);

        // Ohne Traffic: k_opt = k_min
        let k_opt = pool.calculate_k_opt();
        assert_eq!(k_opt, 3);

        // Simuliere Traffic
        {
            let mut monitor = pool.rate_monitor.lock();
            for _ in 0..100 {
                monitor.record();
            }
        }

        // Mit Traffic: k_opt = √(rate × τ_target)
        let k_opt = pool.calculate_k_opt();
        assert!(k_opt >= 3);
        assert!(k_opt <= 20);
    }

    #[test]
    fn test_shuffle() {
        let mut data: Vec<i32> = (0..100).collect();
        let original = data.clone();

        MixingPool::shuffle(&mut data);

        // Länge bleibt gleich
        assert_eq!(data.len(), original.len());

        // Alle Elemente noch vorhanden
        data.sort();
        assert_eq!(data, original);
    }

    #[tokio::test]
    async fn test_probabilistic_forward() {
        let (tx, mut rx) = mpsc::channel(100);
        let config = MixingPoolConfig {
            lamp_threshold_enabled: true,
            lamp_prob_forward_rate: 1.0, // 100% prob forward
            ..Default::default()
        };
        let pool = MixingPool::new(config, tx);

        let peer_id = PeerId::random();
        pool.add_message(b"test".to_vec(), peer_id).await;

        // Kurz warten für prob forward
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Sollte Message erhalten haben
        let result = rx.try_recv();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, b"test".to_vec());

        // Stats prüfen
        let stats = pool.stats();
        assert_eq!(stats.prob_forward_count, 1);
    }

    #[tokio::test]
    async fn test_threshold_flush() {
        let (tx, mut rx) = mpsc::channel(100);
        let config = MixingPoolConfig {
            lamp_threshold_enabled: true,
            lamp_adaptive_k: false,      // Festes k_min
            lamp_prob_forward_rate: 0.0, // Kein Prob-Forward
            k_min: 2,
            tau_min: Duration::from_millis(1), // Sehr kurz für Tests
            tau_max: Duration::from_millis(5), // Sehr kurz für Tests
            epsilon: 10.0,                     // Hoher ε = weniger Noise
            ..Default::default()
        };
        let pool = MixingPool::new(config, tx);

        let peer_id = PeerId::random();

        // 2 Messages (= k_min) sollten Flush triggern
        pool.add_message(b"msg1".to_vec(), peer_id).await;
        pool.add_message(b"msg2".to_vec(), peer_id).await;

        // Warten für Flush + Delays (skaliert mit 1/√2)
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Beide Messages sollten gesendet sein
        let mut received = vec![];
        while let Ok(msg) = rx.try_recv() {
            received.push(msg.1);
        }
        assert_eq!(received.len(), 2);
    }
}
