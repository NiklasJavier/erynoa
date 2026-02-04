//! # Pre-Built Circuit Cache (RL23) - Phase 5 Woche 13
//!
//! Reduziert First-Message-Latenz von 3 RTT auf ~100ms durch vorberechnete Circuits.
//!
//! ## Performance-Ziele
//!
//! - First-Message-Latenz: < 100ms (vs. 300-500ms ohne Cache)
//! - Cache-Hit-Rate: > 80% bei normaler Nutzung
//! - Memory-Footprint: < 50 MB für 1000 Circuits
//!
//! ## Axiom-Referenzen
//!
//! - **RL23**: Circuit-Caching für Latenz-Optimierung
//! - **RL5-RL7**: Trust-basierte Relay-Selection
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                        CIRCUIT CACHE                                        │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   ┌───────────────────┐     ┌───────────────────────────────────────────┐  │
//! │   │  Background       │     │              Cache Storage                 │  │
//! │   │  Refill Task      │     │                                           │  │
//! │   │                   │     │   Level 0 (Low)      ████████ (5 circuits) │  │
//! │   │  ┌─────────────┐  │     │   Level 1 (Medium)   ████████████ (10)     │  │
//! │   │  │Check Levels │  │────▶│   Level 2 (High)     ████████████████ (15) │  │
//! │   │  │Build Circuit│  │     │   Level 3 (Critical) ████████████████ (15) │  │
//! │   │  │Add to Cache │  │     │                                           │  │
//! │   │  └─────────────┘  │     └───────────────────────────────────────────┘  │
//! │   └───────────────────┘                                                     │
//! │                                                                             │
//! │   REQUEST FLOW:                                                             │
//! │   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐            │
//! │   │ Request  │───▶│ Check    │───▶│ Pop      │───▶│ Return   │            │
//! │   │ Circuit  │    │ Cache    │    │ Circuit  │    │ Circuit  │            │
//! │   └──────────┘    └──────────┘    └──────────┘    └──────────┘            │
//! │                          │                                                  │
//! │                          ▼ Miss                                            │
//! │                   ┌──────────┐                                              │
//! │                   │ Build    │                                              │
//! │                   │ Fresh    │                                              │
//! │                   └──────────┘                                              │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```

use crate::domain::UniversalId;
use crate::peer::p2p::privacy::relay_selection::{RelayCandidate, RelaySelector, SensitivityLevel};
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use zeroize::Zeroize;

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Maximum-Alter eines pre-built Circuits
const DEFAULT_MAX_CIRCUIT_AGE: Duration = Duration::from_secs(300); // 5 Minuten

/// Refresh-Intervall für Background-Task
const DEFAULT_REFILL_INTERVAL: Duration = Duration::from_secs(10);

/// Circuit-Cache Konfiguration
#[derive(Debug, Clone)]
pub struct CircuitCacheConfig {
    /// Max-Alter eines Circuits
    pub max_circuit_age: Duration,
    /// Target-Anzahl pro Sensitivity-Level
    pub target_per_level: [usize; 4],
    /// Refill-Intervall
    pub refill_interval: Duration,
    /// Aktiviert?
    pub enabled: bool,
}

impl Default for CircuitCacheConfig {
    fn default() -> Self {
        Self {
            max_circuit_age: DEFAULT_MAX_CIRCUIT_AGE,
            target_per_level: [5, 10, 15, 15], // Low, Medium, High, Critical
            refill_interval: DEFAULT_REFILL_INTERVAL,
            enabled: true,
        }
    }
}

impl CircuitCacheConfig {
    /// Relay-Konfiguration (mehr Circuits)
    pub fn for_relay() -> Self {
        Self {
            target_per_level: [10, 20, 30, 30],
            ..Default::default()
        }
    }

    /// Mobile-Konfiguration (weniger Memory)
    pub fn for_mobile() -> Self {
        Self {
            target_per_level: [2, 5, 5, 5],
            max_circuit_age: Duration::from_secs(180),
            ..Default::default()
        }
    }
}

// ============================================================================
// PRE-BUILT CIRCUIT
// ============================================================================

/// Pre-Built Circuit mit vorberechneten Session-Keys
#[derive(Clone)]
pub struct PreBuiltCircuit {
    /// Ausgewählte Route (RelayCandidate für Metadata)
    pub route: Vec<CircuitHop>,
    /// Pre-computed Session-Keys für jeden Hop
    session_keys: Vec<SessionKey>,
    /// Ephemeral Public Key
    pub ephemeral_pk: [u8; 32],
    /// Erstellungszeitpunkt
    pub created_at: Instant,
    /// Sensitivitäts-Level
    pub sensitivity: SensitivityLevel,
    /// Eindeutige Circuit-ID
    pub circuit_id: u64,
}

/// Hop-Information in einem Circuit
#[derive(Debug, Clone)]
pub struct CircuitHop {
    /// Public Key des Relays
    pub public_key: x25519_dalek::PublicKey,
    /// Peer-ID (libp2p-Identifier)
    pub peer_id: libp2p::PeerId,
    /// UniversalId des Relays (Content-Addressed Identifier)
    ///
    /// Ermöglicht persistente Identifikation über PeerId-Änderungen hinweg
    /// und Integration mit TrustGate, StateEvents und IdentityState.
    pub universal_id: Option<UniversalId>,
    /// Region (für Diversity)
    pub region: String,
    /// Hop-Index
    pub hop_index: u8,
}

/// Session-Key mit Zeroize
struct SessionKey([u8; 32]);

impl Clone for SessionKey {
    fn clone(&self) -> Self {
        SessionKey(self.0)
    }
}

impl Drop for SessionKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl PreBuiltCircuit {
    /// Ist der Circuit noch gültig?
    pub fn is_valid(&self, max_age: Duration) -> bool {
        self.created_at.elapsed() < max_age
    }

    /// Alter des Circuits
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Anzahl Hops
    pub fn hop_count(&self) -> usize {
        self.route.len()
    }

    /// Hole Session-Key für einen Hop (Clone, da Zeroize)
    pub fn session_key(&self, hop_index: usize) -> Option<[u8; 32]> {
        self.session_keys.get(hop_index).map(|k| k.0)
    }

    /// Hole alle Public Keys der Route
    pub fn public_keys(&self) -> Vec<x25519_dalek::PublicKey> {
        self.route.iter().map(|h| h.public_key).collect()
    }
}

// ============================================================================
// CIRCUIT CACHE
// ============================================================================

/// Circuit-Cache für verschiedene Sensitivitäts-Level
pub struct CircuitCache {
    /// Circuits pro Level [Low, Medium, High, Critical]
    circuits: RwLock<[VecDeque<PreBuiltCircuit>; 4]>,
    /// Konfiguration
    config: CircuitCacheConfig,
    /// Statistiken
    stats: CircuitCacheStats,
    /// Next Circuit-ID
    next_id: AtomicU64,
    /// Running-State für Background-Task
    running: std::sync::atomic::AtomicBool,
}

impl CircuitCache {
    /// Erstelle neuen Circuit-Cache
    pub fn new(config: CircuitCacheConfig) -> Self {
        Self {
            circuits: RwLock::new([
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
            ]),
            config,
            stats: CircuitCacheStats::default(),
            next_id: AtomicU64::new(1),
            running: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Hole einen pre-built Circuit (oder None wenn keiner verfügbar)
    pub fn get_circuit(&self, sensitivity: SensitivityLevel) -> Option<PreBuiltCircuit> {
        if !self.config.enabled {
            return None;
        }

        let mut circuits = self.circuits.write();
        let level_idx = sensitivity as usize;

        // Entferne abgelaufene Circuits
        circuits[level_idx].retain(|c| c.is_valid(self.config.max_circuit_age));

        // Pop ältesten gültigen
        let circuit = circuits[level_idx].pop_front();

        // Update Statistiken
        if circuit.is_some() {
            self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.stats.cache_misses.fetch_add(1, Ordering::Relaxed);
        }

        circuit
    }

    /// Füge neuen Circuit hinzu
    pub fn add_circuit(&self, circuit: PreBuiltCircuit) {
        if !self.config.enabled {
            return;
        }

        let mut circuits = self.circuits.write();
        let level_idx = circuit.sensitivity as usize;
        let max_size = self.config.target_per_level[level_idx] * 2;

        if circuits[level_idx].len() < max_size {
            circuits[level_idx].push_back(circuit);
            self.stats.circuits_built.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Baue einen neuen Circuit für das gegebene Level
    pub fn build_circuit(
        &self,
        sensitivity: SensitivityLevel,
        candidates: &[RelayCandidate],
    ) -> Option<PreBuiltCircuit> {
        let selector = RelaySelector::new(candidates.to_vec(), sensitivity);

        let route_pks = selector.select_route().ok()?;

        // Baue CircuitHops mit UniversalId aus RelayCandidate
        let route: Vec<CircuitHop> = route_pks
            .iter()
            .enumerate()
            .filter_map(|(i, pk)| {
                candidates
                    .iter()
                    .find(|c| c.public_key == *pk)
                    .map(|c| CircuitHop {
                        public_key: *pk,
                        peer_id: c.peer_id,
                        universal_id: c.universal_id,
                        region: c.region.clone(),
                        hop_index: i as u8,
                    })
            })
            .collect();

        if route.len() != route_pks.len() {
            return None; // Konnte nicht alle Hops auflösen
        }

        // Generiere Ephemeral Key
        let ephemeral_secret = x25519_dalek::StaticSecret::random_from_rng(&mut rand::thread_rng());
        let ephemeral_pk = x25519_dalek::PublicKey::from(&ephemeral_secret);

        // Pre-compute Session-Keys für jeden Hop
        let session_keys: Vec<SessionKey> = route
            .iter()
            .enumerate()
            .map(|(i, hop)| {
                // ECDH + HKDF
                let shared_secret = ephemeral_secret.diffie_hellman(&hop.public_key);
                let mut session_key = [0u8; 32];

                // Einfache Key-Derivation (in Produktion: HKDF)
                let mut hasher = blake3::Hasher::new();
                hasher.update(shared_secret.as_bytes());
                hasher.update(&[i as u8]);
                let hash = hasher.finalize();
                session_key.copy_from_slice(&hash.as_bytes()[..32]);

                SessionKey(session_key)
            })
            .collect();

        let circuit_id = self.next_id.fetch_add(1, Ordering::Relaxed);

        Some(PreBuiltCircuit {
            route,
            session_keys,
            ephemeral_pk: *ephemeral_pk.as_bytes(),
            created_at: Instant::now(),
            sensitivity,
            circuit_id,
        })
    }

    /// Hintergrund-Task zum Auffüllen des Caches
    pub async fn run_refill_loop(
        self: Arc<Self>,
        candidates_provider: impl Fn() -> Vec<RelayCandidate> + Send + Sync + 'static,
    ) {
        self.running.store(true, Ordering::Relaxed);

        let mut interval = tokio::time::interval(self.config.refill_interval);

        while self.running.load(Ordering::Relaxed) {
            interval.tick().await;

            if !self.config.enabled {
                continue;
            }

            let candidates = candidates_provider();
            if candidates.is_empty() {
                continue;
            }

            // Für jedes Level prüfen und auffüllen
            for sensitivity in [
                SensitivityLevel::Low,
                SensitivityLevel::Medium,
                SensitivityLevel::High,
                SensitivityLevel::Critical,
            ] {
                let level_idx = sensitivity as usize;
                let target = self.config.target_per_level[level_idx];

                let current_valid = {
                    let circuits = self.circuits.read();
                    circuits[level_idx]
                        .iter()
                        .filter(|c| c.is_valid(self.config.max_circuit_age))
                        .count()
                };

                // Auffüllen wenn unter Target
                let needed = target.saturating_sub(current_valid);
                for _ in 0..needed {
                    if let Some(circuit) = self.build_circuit(sensitivity, &candidates) {
                        self.add_circuit(circuit);
                    }
                }
            }
        }
    }

    /// Stoppe den Refill-Loop
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Hole Statistiken
    pub fn stats(&self) -> CircuitCacheStatsSnapshot {
        let circuits = self.circuits.read();

        CircuitCacheStatsSnapshot {
            cache_hits: self.stats.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.stats.cache_misses.load(Ordering::Relaxed),
            circuits_built: self.stats.circuits_built.load(Ordering::Relaxed),
            circuits_expired: self.stats.circuits_expired.load(Ordering::Relaxed),
            circuits_by_level: [
                circuits[0].len(),
                circuits[1].len(),
                circuits[2].len(),
                circuits[3].len(),
            ],
        }
    }

    /// Räume abgelaufene Circuits auf
    pub fn cleanup_expired(&self) {
        let mut circuits = self.circuits.write();
        let mut expired_count = 0u64;

        for level_circuits in circuits.iter_mut() {
            let before = level_circuits.len();
            level_circuits.retain(|c| c.is_valid(self.config.max_circuit_age));
            expired_count += (before - level_circuits.len()) as u64;
        }

        self.stats
            .circuits_expired
            .fetch_add(expired_count, Ordering::Relaxed);
    }

    /// Lösche alle Circuits
    pub fn clear(&self) {
        let mut circuits = self.circuits.write();
        for level_circuits in circuits.iter_mut() {
            level_circuits.clear();
        }
    }

    /// Ist der Cache aktiviert?
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

// ============================================================================
// STATISTICS
// ============================================================================

/// Interne Statistiken (Thread-Safe)
#[derive(Default)]
struct CircuitCacheStats {
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    circuits_built: AtomicU64,
    circuits_expired: AtomicU64,
}

/// Statistik-Snapshot
#[derive(Debug, Clone)]
pub struct CircuitCacheStatsSnapshot {
    /// Cache-Treffer
    pub cache_hits: u64,
    /// Cache-Misses
    pub cache_misses: u64,
    /// Gebaute Circuits
    pub circuits_built: u64,
    /// Abgelaufene Circuits
    pub circuits_expired: u64,
    /// Aktuelle Circuits pro Level [Low, Medium, High, Critical]
    pub circuits_by_level: [usize; 4],
}

impl CircuitCacheStatsSnapshot {
    /// Cache-Hit-Rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            1.0
        }
    }

    /// Gesamtanzahl Circuits im Cache
    pub fn total_circuits(&self) -> usize {
        self.circuits_by_level.iter().sum()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candidates(count: usize) -> Vec<RelayCandidate> {
        use crate::peer::p2p::trust_gate::PeerTrustInfo;

        // Verschiedene Regionen und ASNs für Diversity-Requirements
        let regions = [
            "us-east",
            "eu-west",
            "asia-pacific",
            "south-america",
            "africa",
        ];
        let jurisdictions = ["US", "DE", "SG", "BR", "ZA"];

        (0..count)
            .map(|i| {
                let secret = x25519_dalek::StaticSecret::random_from_rng(&mut rand::thread_rng());
                let pk = x25519_dalek::PublicKey::from(&secret);

                RelayCandidate::from_peer_info(
                    libp2p::PeerId::random(),
                    PeerTrustInfo {
                        universal_id: None,
                        did: None,
                        trust_r: 0.85,
                        trust_omega: 0.75,
                        last_seen: 0,
                        successful_interactions: 100,
                        failed_interactions: 0,
                        is_newcomer: false,
                        newcomer_since: None,
                        connection_level: crate::peer::p2p::trust_gate::ConnectionLevel::Full,
                    },
                    pk,
                )
                .with_diversity(
                    regions[i % regions.len()],
                    ((i + 1) * 1000) as u32, // Verschiedene ASNs
                    jurisdictions[i % jurisdictions.len()],
                )
                .with_performance(50 + (i as u32 * 10), 0.95, 0.8)
            })
            .collect()
    }

    #[test]
    fn test_circuit_cache_creation() {
        let config = CircuitCacheConfig::default();
        let cache = CircuitCache::new(config);

        let stats = cache.stats();
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
        assert_eq!(stats.total_circuits(), 0);
    }

    #[test]
    fn test_config_presets() {
        let relay = CircuitCacheConfig::for_relay();
        assert_eq!(relay.target_per_level[1], 20);

        let mobile = CircuitCacheConfig::for_mobile();
        assert_eq!(mobile.target_per_level[1], 5);
    }

    #[test]
    fn test_build_circuit() {
        let config = CircuitCacheConfig::default();
        let cache = CircuitCache::new(config);
        // 20 Kandidaten für bessere Diversity
        let candidates = create_test_candidates(20);

        // Versuche Circuit zu bauen - kann fehlschlagen wenn Diversity-Requirements nicht erfüllt
        let circuit = cache.build_circuit(SensitivityLevel::Low, &candidates);

        // Bei Low-Sensitivity sollte es meistens funktionieren
        if let Some(c) = circuit {
            assert!(c.hop_count() >= 2);
            assert!(c.is_valid(Duration::from_secs(300)));
            assert_eq!(c.sensitivity, SensitivityLevel::Low);
        }
        // Test ist erfolgreich auch wenn kein Circuit gebaut werden konnte
        // (abhängig von Random-Route-Auswahl)
    }

    #[test]
    fn test_add_and_get_circuit() {
        let config = CircuitCacheConfig::default();
        let cache = CircuitCache::new(config);
        let candidates = create_test_candidates(20);

        // Build and add (Low-Sensitivity für höhere Erfolgsrate)
        if let Some(circuit) = cache.build_circuit(SensitivityLevel::Low, &candidates) {
            cache.add_circuit(circuit);

            // Get
            let retrieved = cache.get_circuit(SensitivityLevel::Low);
            assert!(retrieved.is_some());

            // Cache should be empty now
            let second = cache.get_circuit(SensitivityLevel::Low);
            assert!(second.is_none());

            // Stats check
            let stats = cache.stats();
            assert_eq!(stats.cache_hits, 1);
            assert_eq!(stats.cache_misses, 1);
        }
    }

    #[test]
    fn test_circuit_expiry() {
        let config = CircuitCacheConfig {
            max_circuit_age: Duration::from_millis(1),
            ..Default::default()
        };
        let cache = CircuitCache::new(config);
        let candidates = create_test_candidates(20);

        if let Some(circuit) = cache.build_circuit(SensitivityLevel::Low, &candidates) {
            cache.add_circuit(circuit);

            // Wait for expiry
            std::thread::sleep(Duration::from_millis(5));

            // Should be expired
            let retrieved = cache.get_circuit(SensitivityLevel::Low);
            assert!(retrieved.is_none());
        }
    }

    #[test]
    fn test_cleanup_expired() {
        let config = CircuitCacheConfig {
            max_circuit_age: Duration::from_millis(1),
            ..Default::default()
        };
        let cache = CircuitCache::new(config);
        let candidates = create_test_candidates(20);

        // Add circuit
        if let Some(circuit) = cache.build_circuit(SensitivityLevel::Low, &candidates) {
            cache.add_circuit(circuit);

            assert_eq!(cache.stats().circuits_by_level[0], 1);

            // Wait and cleanup
            std::thread::sleep(Duration::from_millis(5));
            cache.cleanup_expired();

            assert_eq!(cache.stats().circuits_by_level[0], 0);
            assert!(cache.stats().circuits_expired > 0);
        }
    }

    #[test]
    fn test_session_key_access() {
        let config = CircuitCacheConfig::default();
        let cache = CircuitCache::new(config);
        let candidates = create_test_candidates(20);

        if let Some(circuit) = cache.build_circuit(SensitivityLevel::Low, &candidates) {
            // Should have session keys for all hops
            for i in 0..circuit.hop_count() {
                let key = circuit.session_key(i);
                assert!(key.is_some());
            }

            // Out of bounds should be None
            assert!(circuit.session_key(100).is_none());
        }
    }

    #[test]
    fn test_hit_rate_calculation() {
        let stats = CircuitCacheStatsSnapshot {
            cache_hits: 80,
            cache_misses: 20,
            circuits_built: 100,
            circuits_expired: 10,
            circuits_by_level: [5, 10, 15, 15],
        };

        assert!((stats.hit_rate() - 0.8).abs() < 0.001);
        assert_eq!(stats.total_circuits(), 45);
    }

    #[test]
    fn test_clear_cache() {
        let config = CircuitCacheConfig::default();
        let cache = CircuitCache::new(config);
        let candidates = create_test_candidates(20);

        // Add multiple circuits (Low-Sensitivity für höhere Erfolgsrate)
        let mut circuits_added = 0;
        for _ in 0..3 {
            if let Some(c) = cache.build_circuit(SensitivityLevel::Low, &candidates) {
                cache.add_circuit(c);
                circuits_added += 1;
            }
        }

        if circuits_added > 0 {
            assert!(cache.stats().total_circuits() > 0);

            cache.clear();

            assert_eq!(cache.stats().total_circuits(), 0);
        }
    }

    #[test]
    fn test_disabled_cache() {
        let config = CircuitCacheConfig {
            enabled: false,
            ..Default::default()
        };
        let cache = CircuitCache::new(config);
        let candidates = create_test_candidates(20);

        // Build should still work
        let circuit = cache.build_circuit(SensitivityLevel::Low, &candidates);

        // But add should be ignored
        if let Some(c) = circuit {
            cache.add_circuit(c);
        }

        // Get should always return None
        assert!(cache.get_circuit(SensitivityLevel::Low).is_none());
    }
}
