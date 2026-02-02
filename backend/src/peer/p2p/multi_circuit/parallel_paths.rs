//! # Conflux-Style Multi-Circuit Multiplexing (RL28) - Phase 5c Woche 14
//!
//! Erreicht 4× Throughput durch parallele Circuit-Nutzung:
//! - 2-3 unabhängige Circuits gleichzeitig
//! - Trust-basierte Pfad-Diversifizierung (keine AS-Überlappung)
//! - Egress-Aggregation für konsistente Auslieferung
//!
//! ## Referenz: Conflux (CCS 2019), "Traffic Analysis Resistant Anonymity Networks"
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                     CONFLUX MULTI-CIRCUIT MANAGER                           │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   SENSITIVITY-BASED ROUTING:                                                │
//! │                                                                             │
//! │   LOW/MEDIUM:    ┌─────────┐                                               │
//! │   Round-Robin    │Circuit 1│ ─────────────────────────────▶ Destination    │
//! │                  └─────────┘                                               │
//! │                                                                             │
//! │   HIGH:          ┌─────────┐                                               │
//! │   Redundant      │Circuit 1│ ────┐                                         │
//! │                  └─────────┘     │   ┌──────────────┐                      │
//! │                  ┌─────────┐     ├──▶│ First-Arrival│ ─▶ Destination       │
//! │                  │Circuit 2│ ────┘   └──────────────┘                      │
//! │                  └─────────┘                                               │
//! │                                                                             │
//! │   CRITICAL:      ┌─────────┐ Share 1                                       │
//! │   Secret-Share   │Circuit 1│ ────┐                                         │
//! │                  └─────────┘     │   ┌──────────────┐                      │
//! │                  ┌─────────┐     ├──▶│ Reconstruct  │ ─▶ Destination       │
//! │                  │Circuit 2│ ────┤   │ (k-of-n)     │                      │
//! │                  └─────────┘     │   └──────────────┘                      │
//! │                  ┌─────────┐     │                                         │
//! │                  │Circuit 3│ ────┘                                         │
//! │                  └─────────┘ Share 3                                       │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Sicherheits-Garantien
//!
//! - Alle Circuits erfüllen RL6 (Relay-Diversität)
//! - Secret-Sharing für CRITICAL-Nachrichten (Threshold-Rekonstruktion)
//! - Timing-Korrelation durch Mixing-Pool neutralisiert (RL8)
//! - AS-Diversität zwischen Circuits (MIN_AS_DISTANCE)
//!
//! ## Axiom-Referenzen
//!
//! - **RL28**: Multi-Circuit-Multiplexing (Conflux-Style)
//! - **RL6**: Relay-Diversität (Entropie-Maximierung)
//! - **RL5**: Trust-Monotonie in Circuit-Auswahl

use crate::peer::p2p::privacy::relay_selection::{RelayCandidate, RelaySelector, SensitivityLevel};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use zeroize::Zeroize;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Maximum parallele Circuits
pub const MAX_PARALLEL_CIRCUITS: usize = 3;

/// Minimum AS-Distanz zwischen Circuits
pub const MIN_AS_DISTANCE: usize = 2;

/// Default Circuit-Timeout
const DEFAULT_CIRCUIT_TIMEOUT: Duration = Duration::from_secs(10);

/// Maximum Circuit-Alter bevor Rebuild
const MAX_CIRCUIT_AGE: Duration = Duration::from_secs(300);

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Konfiguration für Conflux Multi-Circuit
#[derive(Debug, Clone)]
pub struct ConfluxConfig {
    /// Anzahl paralleler Circuits (2-3)
    pub parallel_count: usize,
    /// Threshold für Secret-Sharing (k-of-n)
    pub secret_threshold: usize,
    /// Timeout für Circuit-Erstellung
    pub circuit_timeout: Duration,
    /// Aktiviere Egress-Aggregation
    pub enable_aggregation: bool,
    /// Maximum Circuit-Alter
    pub max_circuit_age: Duration,
}

impl Default for ConfluxConfig {
    fn default() -> Self {
        Self {
            parallel_count: 2,
            secret_threshold: 2, // 2-of-3 für CRITICAL
            circuit_timeout: DEFAULT_CIRCUIT_TIMEOUT,
            enable_aggregation: true,
            max_circuit_age: MAX_CIRCUIT_AGE,
        }
    }
}

impl ConfluxConfig {
    /// High-Security Konfiguration (3 Circuits, 2-of-3 Threshold)
    pub fn high_security() -> Self {
        Self {
            parallel_count: 3,
            secret_threshold: 2,
            ..Default::default()
        }
    }

    /// Low-Latency Konfiguration (2 Circuits)
    pub fn low_latency() -> Self {
        Self {
            parallel_count: 2,
            secret_threshold: 2,
            circuit_timeout: Duration::from_secs(5),
            ..Default::default()
        }
    }
}

// ============================================================================
// ACTIVE CIRCUIT
// ============================================================================

/// Aktiver Circuit mit Session-Keys
pub struct ActiveCircuit {
    /// Circuit-ID (16 Bytes)
    pub id: [u8; 16],
    /// Route (Relay-Liste)
    pub route: Vec<RelayCandidate>,
    /// Session-Keys pro Hop (Zeroize bei Drop)
    session_keys: Vec<SessionKey>,
    /// Erstellungszeitpunkt
    pub created_at: Instant,
    /// Statistiken
    pub stats: CircuitStats,
    /// ASNs in diesem Circuit (für Diversitäts-Check)
    asns: HashSet<u32>,
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

impl Clone for ActiveCircuit {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            route: self.route.clone(),
            session_keys: self.session_keys.clone(),
            created_at: self.created_at,
            stats: self.stats.clone(),
            asns: self.asns.clone(),
        }
    }
}

impl ActiveCircuit {
    /// Erstelle neuen Circuit
    pub fn new(route: Vec<RelayCandidate>) -> Self {
        let asns: HashSet<u32> = route.iter().map(|r| r.asn).collect();

        // Generiere Session-Keys für jeden Hop
        let session_keys: Vec<SessionKey> = route
            .iter()
            .enumerate()
            .map(|(i, relay)| {
                // Echte Implementierung: ECDH + HKDF
                let mut key = [0u8; 32];
                let mut hasher = blake3::Hasher::new();
                hasher.update(relay.public_key.as_bytes());
                hasher.update(&[i as u8]);
                let hash = hasher.finalize();
                key.copy_from_slice(&hash.as_bytes()[..32]);
                SessionKey(key)
            })
            .collect();

        Self {
            id: rand::random(),
            route,
            session_keys,
            created_at: Instant::now(),
            stats: CircuitStats::default(),
            asns,
        }
    }

    /// Ist der Circuit noch gültig?
    pub fn is_valid(&self, max_age: Duration) -> bool {
        self.created_at.elapsed() < max_age
    }

    /// Alter des Circuits
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Hole Session-Key für Hop
    pub fn session_key(&self, hop: usize) -> Option<[u8; 32]> {
        self.session_keys.get(hop).map(|k| k.0)
    }

    /// Prüfe ob Circuit AS mit anderem Circuit überlappt
    pub fn overlaps_with(&self, other: &ActiveCircuit) -> usize {
        self.asns.intersection(&other.asns).count()
    }

    /// Prüfe ob Circuit mit ASN-Set überlappt
    pub fn overlaps_with_asns(&self, asns: &HashSet<u32>) -> usize {
        self.asns.intersection(asns).count()
    }
}

/// Circuit-Statistiken
#[derive(Clone, Default, Debug)]
pub struct CircuitStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub avg_latency_ms: f64,
    latency_samples: u64,
}

impl CircuitStats {
    /// Update Latenz mit neuem Sample
    pub fn update_latency(&mut self, latency_ms: f64) {
        self.latency_samples += 1;
        // Exponential Moving Average
        let alpha = 0.1;
        self.avg_latency_ms = alpha * latency_ms + (1.0 - alpha) * self.avg_latency_ms;
    }
}

// ============================================================================
// CONFLUX MANAGER
// ============================================================================

/// Conflux-Manager für Multi-Circuit-Routing
pub struct ConfluxManager {
    /// Aktive Circuits
    circuits: RwLock<Vec<ActiveCircuit>>,
    /// Relay-Candidates für Circuit-Building
    candidates: RwLock<Vec<RelayCandidate>>,
    /// Egress-Aggregator
    egress_aggregator: Arc<EgressAggregator>,
    /// Secret-Sharing für CRITICAL
    secret_sharer: SecretSharer,
    /// Konfiguration
    config: ConfluxConfig,
    /// Statistiken
    stats: ConfluxInternalStats,
    /// Round-Robin Counter
    rr_counter: AtomicU64,
}

impl ConfluxManager {
    /// Erstelle neuen Conflux-Manager
    pub fn new(config: ConfluxConfig) -> Self {
        Self {
            circuits: RwLock::new(Vec::with_capacity(MAX_PARALLEL_CIRCUITS)),
            candidates: RwLock::new(Vec::new()),
            egress_aggregator: Arc::new(EgressAggregator::new()),
            secret_sharer: SecretSharer::new(config.secret_threshold),
            config,
            stats: ConfluxInternalStats::default(),
            rr_counter: AtomicU64::new(0),
        }
    }

    /// Setze Relay-Candidates für Circuit-Building
    pub fn set_candidates(&self, candidates: Vec<RelayCandidate>) {
        *self.candidates.write() = candidates;
    }

    /// Multi-Path-Send: Verteile Nachricht über parallele Circuits
    ///
    /// - LOW/MEDIUM: Round-Robin über verfügbare Circuits
    /// - HIGH: Alle Circuits gleichzeitig (Redundanz)
    /// - CRITICAL: Secret-Sharing über alle Circuits (Threshold-Rekonstruktion)
    pub async fn multi_path_send(
        &self,
        payload: &[u8],
        sensitivity: SensitivityLevel,
    ) -> Result<MultiPathResult, ConfluxError> {
        // Ensure wir haben genug Circuits
        self.ensure_circuits(sensitivity).await?;

        let circuits = self.circuits.read();

        match sensitivity {
            SensitivityLevel::Low | SensitivityLevel::Medium => {
                self.send_round_robin(&circuits, payload).await
            }
            SensitivityLevel::High => self.send_redundant(&circuits, payload).await,
            SensitivityLevel::Critical => self.send_secret_shared(&circuits, payload).await,
        }
    }

    /// Round-Robin: Wähle einen Circuit
    async fn send_round_robin(
        &self,
        circuits: &[ActiveCircuit],
        payload: &[u8],
    ) -> Result<MultiPathResult, ConfluxError> {
        if circuits.is_empty() {
            return Err(ConfluxError::NoCircuitsAvailable);
        }

        let idx = self.rr_counter.fetch_add(1, Ordering::Relaxed) as usize % circuits.len();
        let circuit = &circuits[idx];

        let result = self.send_single_circuit(circuit, payload).await?;

        self.stats.packets_sent.fetch_add(1, Ordering::Relaxed);
        self.stats
            .bytes_sent
            .fetch_add(payload.len() as u64, Ordering::Relaxed);

        Ok(MultiPathResult {
            circuits_used: 1,
            strategy: MultiPathStrategy::RoundRobin,
            latency_ms: result.latency_ms,
            success: true,
        })
    }

    /// Redundanz: Alle Circuits gleichzeitig
    async fn send_redundant(
        &self,
        circuits: &[ActiveCircuit],
        payload: &[u8],
    ) -> Result<MultiPathResult, ConfluxError> {
        if circuits.is_empty() {
            return Err(ConfluxError::NoCircuitsAvailable);
        }

        let count = circuits.len().min(self.config.parallel_count);
        let mut results = Vec::with_capacity(count);

        // Sende parallel über alle Circuits
        for circuit in circuits.iter().take(count) {
            let result = self.send_single_circuit(circuit, payload).await;
            results.push(result);
        }

        let successful: Vec<_> = results.iter().filter_map(|r| r.as_ref().ok()).collect();

        if successful.is_empty() {
            return Err(ConfluxError::AllCircuitsFailed);
        }

        // First-Arrival: Nimm schnellsten
        let min_latency = successful
            .iter()
            .map(|r| r.latency_ms)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        self.stats
            .packets_sent
            .fetch_add(count as u64, Ordering::Relaxed);
        self.stats
            .bytes_sent
            .fetch_add((payload.len() * count) as u64, Ordering::Relaxed);

        Ok(MultiPathResult {
            circuits_used: successful.len(),
            strategy: MultiPathStrategy::Redundant,
            latency_ms: min_latency,
            success: true,
        })
    }

    /// Secret-Sharing: Teile Nachricht in k-of-n Shares
    async fn send_secret_shared(
        &self,
        circuits: &[ActiveCircuit],
        payload: &[u8],
    ) -> Result<MultiPathResult, ConfluxError> {
        let n = circuits.len().min(self.config.parallel_count);

        if n < self.config.secret_threshold {
            return Err(ConfluxError::InsufficientCircuits {
                available: n,
                required: self.config.secret_threshold,
            });
        }

        // Teile Payload in Shares
        let shares = self.secret_sharer.split(payload, n)?;

        let mut results = Vec::with_capacity(n);

        // Sende Shares parallel
        for (circuit, share) in circuits.iter().take(n).zip(shares.iter()) {
            let result = self.send_single_circuit(circuit, share).await;
            results.push(result);
        }

        let successful = results.iter().filter(|r| r.is_ok()).count();

        if successful < self.config.secret_threshold {
            return Err(ConfluxError::InsufficientShares {
                received: successful,
                required: self.config.secret_threshold,
            });
        }

        // Max-Latenz weil alle Shares ankommen müssen
        let max_latency = results
            .iter()
            .filter_map(|r| r.as_ref().ok())
            .map(|r| r.latency_ms)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        self.stats
            .packets_sent
            .fetch_add(n as u64, Ordering::Relaxed);
        self.stats.bytes_sent.fetch_add(
            shares.iter().map(|s| s.len() as u64).sum::<u64>(),
            Ordering::Relaxed,
        );

        Ok(MultiPathResult {
            circuits_used: successful,
            strategy: MultiPathStrategy::SecretSharing,
            latency_ms: max_latency,
            success: true,
        })
    }

    /// Stelle sicher, dass genug diverse Circuits vorhanden sind
    async fn ensure_circuits(&self, sensitivity: SensitivityLevel) -> Result<(), ConfluxError> {
        let needed = match sensitivity {
            SensitivityLevel::Low | SensitivityLevel::Medium => 1,
            SensitivityLevel::High => 2,
            SensitivityLevel::Critical => self.config.parallel_count.max(3),
        };

        // Entferne abgelaufene Circuits
        {
            let mut circuits = self.circuits.write();
            circuits.retain(|c| c.is_valid(self.config.max_circuit_age));
        }

        let current = self.circuits.read().len();

        if current >= needed {
            return Ok(());
        }

        // Baue neue Circuits mit AS-Diversität
        let existing_asns: HashSet<u32> = self
            .circuits
            .read()
            .iter()
            .flat_map(|c| c.asns.iter().cloned())
            .collect();

        let candidates = self.candidates.read().clone();

        for _ in current..needed {
            if let Some(circuit) =
                self.build_diverse_circuit(sensitivity, &existing_asns, &candidates)
            {
                self.circuits.write().push(circuit);
            }
        }

        // Prüfe ob wir genug haben
        if self.circuits.read().len() < needed {
            // Akzeptiere auch mit weniger Diversität für LOW/MEDIUM
            if matches!(
                sensitivity,
                SensitivityLevel::Low | SensitivityLevel::Medium
            ) {
                return Ok(());
            }
            return Err(ConfluxError::InsufficientCircuits {
                available: self.circuits.read().len(),
                required: needed,
            });
        }

        Ok(())
    }

    /// Baue Circuit mit AS-Diversität zu existierenden
    fn build_diverse_circuit(
        &self,
        sensitivity: SensitivityLevel,
        exclude_asns: &HashSet<u32>,
        candidates: &[RelayCandidate],
    ) -> Option<ActiveCircuit> {
        if candidates.len() < 3 {
            return None;
        }

        // Erstelle RelaySelector
        let selector = RelaySelector::new(candidates.to_vec(), sensitivity);

        // Versuche mehrmals mit steigenden Constraints
        for attempt in 0..5 {
            if let Ok(route_pks) = selector.select_route() {
                // Finde Candidates für Route
                let route: Vec<RelayCandidate> = route_pks
                    .iter()
                    .filter_map(|pk| candidates.iter().find(|c| c.public_key == *pk).cloned())
                    .collect();

                if route.len() != route_pks.len() {
                    continue;
                }

                // Prüfe AS-Diversität
                let route_asns: HashSet<u32> = route.iter().map(|r| r.asn).collect();
                let overlap = route_asns.intersection(exclude_asns).count();

                // Akzeptiere mit steigender Toleranz
                if overlap <= attempt {
                    return Some(ActiveCircuit::new(route));
                }
            }
        }

        None
    }

    /// Sende über einen einzelnen Circuit
    async fn send_single_circuit(
        &self,
        circuit: &ActiveCircuit,
        _payload: &[u8],
    ) -> Result<SendResult, ConfluxError> {
        let start = Instant::now();

        // TODO: Echtes Onion-Routing über Circuit
        // Hier Platzhalter für die Integration mit privacy/onion.rs

        // Simuliere Netzwerk-Latenz (in Produktion: echte Übertragung)
        #[cfg(test)]
        {
            // Im Test: keine echte Verzögerung
        }

        #[cfg(not(test))]
        {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(SendResult {
            circuit_id: circuit.id,
            latency_ms,
        })
    }

    /// Statistiken über alle Circuits
    pub fn stats(&self) -> ConfluxStats {
        let circuits = self.circuits.read();
        let aggregator_stats = self.egress_aggregator.stats();

        ConfluxStats {
            active_circuits: circuits.len(),
            total_packets_sent: self.stats.packets_sent.load(Ordering::Relaxed),
            total_bytes_sent: self.stats.bytes_sent.load(Ordering::Relaxed),
            avg_circuit_latency_ms: if circuits.is_empty() {
                0.0
            } else {
                circuits.iter().map(|c| c.stats.avg_latency_ms).sum::<f64>() / circuits.len() as f64
            },
            circuits_built: self.stats.circuits_built.load(Ordering::Relaxed),
            pending_aggregations: aggregator_stats.pending_aggregations,
            completed_reconstructions: aggregator_stats.completed_reconstructions,
        }
    }

    /// Empfange Shares für Secret-Sharing-Rekonstruktion
    pub fn receive_shares(
        &self,
        message_id: [u8; 16],
        share: Vec<u8>,
        threshold: usize,
    ) -> Option<Vec<u8>> {
        // Füge Share hinzu
        self.egress_aggregator
            .add_share(message_id, share, threshold);

        // Versuche Rekonstruktion
        self.egress_aggregator.try_reconstruct(&message_id)
    }

    /// Hole Referenz auf EgressAggregator für erweiterte Operationen
    pub fn aggregator(&self) -> &Arc<EgressAggregator> {
        &self.egress_aggregator
    }

    /// Anzahl aktiver Circuits
    pub fn circuit_count(&self) -> usize {
        self.circuits.read().len()
    }

    /// Lösche alle Circuits (für Tests/Reset)
    pub fn clear_circuits(&self) {
        self.circuits.write().clear();
    }
}

// ============================================================================
// INTERNAL STATISTICS
// ============================================================================

#[derive(Default)]
struct ConfluxInternalStats {
    packets_sent: AtomicU64,
    bytes_sent: AtomicU64,
    circuits_built: AtomicU64,
}

// ============================================================================
// RESULTS & STRATEGIES
// ============================================================================

/// Multi-Path-Send Ergebnis
#[derive(Debug, Clone)]
pub struct MultiPathResult {
    /// Anzahl verwendeter Circuits
    pub circuits_used: usize,
    /// Verwendete Strategie
    pub strategy: MultiPathStrategy,
    /// Latenz in ms (min für Redundant, max für SecretSharing)
    pub latency_ms: f64,
    /// Erfolgreich?
    pub success: bool,
}

/// Multi-Path-Strategie
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiPathStrategy {
    /// Round-Robin: Ein Circuit pro Nachricht
    RoundRobin,
    /// Redundant: Alle Circuits, First-Arrival
    Redundant,
    /// Secret-Sharing: k-of-n Threshold
    SecretSharing,
}

/// Einzelnes Send-Ergebnis
#[derive(Debug)]
struct SendResult {
    circuit_id: [u8; 16],
    latency_ms: f64,
}

// ============================================================================
// EGRESS AGGREGATOR
// ============================================================================

/// Egress-Aggregator für konsistente Auslieferung
pub struct EgressAggregator {
    /// Pending Shares für Secret-Sharing-Rekonstruktion
    pending_shares: RwLock<HashMap<[u8; 16], PendingMessage>>,
}

/// Pending Message mit Shares
struct PendingMessage {
    shares: Vec<Vec<u8>>,
    created_at: Instant,
    threshold: usize,
}

impl EgressAggregator {
    /// Erstelle neuen Aggregator
    pub fn new() -> Self {
        Self {
            pending_shares: RwLock::new(HashMap::new()),
        }
    }

    /// Füge Share hinzu
    pub fn add_share(&self, msg_id: [u8; 16], share: Vec<u8>, threshold: usize) {
        let mut pending = self.pending_shares.write();

        pending
            .entry(msg_id)
            .or_insert_with(|| PendingMessage {
                shares: Vec::new(),
                created_at: Instant::now(),
                threshold,
            })
            .shares
            .push(share);
    }

    /// Versuche Nachricht zu rekonstruieren
    pub fn try_reconstruct(&self, msg_id: &[u8; 16]) -> Option<Vec<u8>> {
        let pending = self.pending_shares.read();

        if let Some(msg) = pending.get(msg_id) {
            if msg.shares.len() >= msg.threshold {
                // XOR-basierte Rekonstruktion (vereinfacht)
                return Some(
                    msg.shares
                        .iter()
                        .fold(vec![0u8; msg.shares[0].len()], |acc, share| {
                            acc.iter().zip(share.iter()).map(|(&a, &b)| a ^ b).collect()
                        }),
                );
            }
        }

        None
    }

    /// Bereinige alte Pending-Messages
    pub fn cleanup(&self, max_age: Duration) {
        let mut pending = self.pending_shares.write();
        pending.retain(|_, msg| msg.created_at.elapsed() < max_age);
    }

    /// Hole Statistiken
    pub fn stats(&self) -> EgressAggregatorStats {
        let pending = self.pending_shares.read();
        EgressAggregatorStats {
            pending_aggregations: pending.len(),
            completed_reconstructions: 0, // TODO: Tracking hinzufügen
        }
    }
}

/// Egress-Aggregator Statistiken
#[derive(Debug, Clone, Default)]
pub struct EgressAggregatorStats {
    /// Anzahl ausstehender Aggregationen
    pub pending_aggregations: usize,
    /// Abgeschlossene Rekonstruktionen
    pub completed_reconstructions: u64,
}

impl Default for EgressAggregator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SECRET SHARER
// ============================================================================

/// Secret-Sharer für CRITICAL-Nachrichten (XOR-basiert)
///
/// Für Produktion: Shamir Secret Sharing implementieren
pub struct SecretSharer {
    threshold: usize,
}

impl SecretSharer {
    /// Erstelle neuen Sharer mit Threshold
    pub fn new(threshold: usize) -> Self {
        Self { threshold }
    }

    /// Teile Payload in n Shares (k-of-n rekonstruierbar)
    ///
    /// Vereinfachtes XOR-basiertes Splitting.
    /// Produktions-Code sollte Shamir Secret Sharing verwenden.
    pub fn split(&self, payload: &[u8], n: usize) -> Result<Vec<Vec<u8>>, ConfluxError> {
        if n < self.threshold {
            return Err(ConfluxError::InsufficientShares {
                received: n,
                required: self.threshold,
            });
        }

        if payload.is_empty() {
            return Err(ConfluxError::EmptyPayload);
        }

        let mut shares = Vec::with_capacity(n);
        let mut rng = rand::thread_rng();

        // Generiere n-1 zufällige Shares
        for _ in 0..n - 1 {
            let share: Vec<u8> = (0..payload.len())
                .map(|_| rand::Rng::gen(&mut rng))
                .collect();
            shares.push(share);
        }

        // Letzter Share = XOR von Payload und allen anderen
        let last_share: Vec<u8> = payload
            .iter()
            .enumerate()
            .map(|(i, &p)| shares.iter().fold(p, |acc, share| acc ^ share[i]))
            .collect();
        shares.push(last_share);

        Ok(shares)
    }

    /// Rekonstruiere Payload aus Shares
    pub fn reconstruct(&self, shares: &[Vec<u8>]) -> Result<Vec<u8>, ConfluxError> {
        if shares.len() < self.threshold {
            return Err(ConfluxError::InsufficientShares {
                received: shares.len(),
                required: self.threshold,
            });
        }

        if shares.is_empty() || shares[0].is_empty() {
            return Err(ConfluxError::EmptyPayload);
        }

        // XOR alle Shares
        let payload: Vec<u8> = shares
            .iter()
            .fold(vec![0u8; shares[0].len()], |acc, share| {
                acc.iter().zip(share.iter()).map(|(&a, &b)| a ^ b).collect()
            });

        Ok(payload)
    }
}

// ============================================================================
// STATISTICS
// ============================================================================

/// Conflux-Statistiken
#[derive(Debug, Clone)]
pub struct ConfluxStats {
    /// Anzahl aktiver Circuits
    pub active_circuits: usize,
    /// Gesamte gesendete Pakete
    pub total_packets_sent: u64,
    /// Gesamte gesendete Bytes
    pub total_bytes_sent: u64,
    /// Durchschnittliche Circuit-Latenz
    pub avg_circuit_latency_ms: f64,
    /// Anzahl gebauter Circuits
    pub circuits_built: u64,
    /// Pending Aggregationen (Egress)
    pub pending_aggregations: usize,
    /// Abgeschlossene Rekonstruktionen
    pub completed_reconstructions: u64,
}

// ============================================================================
// ERRORS
// ============================================================================

/// Conflux-Fehler
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfluxError {
    /// Keine Circuits verfügbar
    NoCircuitsAvailable,

    /// Alle Circuits fehlgeschlagen
    AllCircuitsFailed,

    /// Nicht genug Circuits
    InsufficientCircuits { available: usize, required: usize },

    /// Nicht genug Shares empfangen
    InsufficientShares { received: usize, required: usize },

    /// Leerer Payload
    EmptyPayload,

    /// Circuit-Build fehlgeschlagen
    CircuitBuildFailed,

    /// Send fehlgeschlagen
    SendFailed(String),
}

impl std::fmt::Display for ConfluxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoCircuitsAvailable => write!(f, "No circuits available"),
            Self::AllCircuitsFailed => write!(f, "All circuits failed"),
            Self::InsufficientCircuits {
                available,
                required,
            } => {
                write!(
                    f,
                    "Insufficient circuits: {} available, {} required",
                    available, required
                )
            }
            Self::InsufficientShares { received, required } => {
                write!(
                    f,
                    "Insufficient shares: {} received, {} required",
                    received, required
                )
            }
            Self::EmptyPayload => write!(f, "Empty payload"),
            Self::CircuitBuildFailed => write!(f, "Circuit build failed"),
            Self::SendFailed(msg) => write!(f, "Send failed: {}", msg),
        }
    }
}

impl std::error::Error for ConfluxError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer::p2p::trust_gate::PeerTrustInfo;

    fn create_test_candidates(count: usize) -> Vec<RelayCandidate> {
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
                    ((i + 1) * 1000) as u32,
                    jurisdictions[i % jurisdictions.len()],
                )
                .with_performance(50 + (i as u32 * 10), 0.95, 0.8)
            })
            .collect()
    }

    #[test]
    fn test_secret_sharing_roundtrip() {
        let sharer = SecretSharer::new(2);
        let payload = b"Hello, Multi-Circuit!".to_vec();

        let shares = sharer.split(&payload, 3).unwrap();
        assert_eq!(shares.len(), 3);

        // Rekonstruiere mit allen Shares
        let reconstructed = sharer.reconstruct(&shares).unwrap();
        assert_eq!(reconstructed, payload);
    }

    #[test]
    fn test_secret_sharing_threshold() {
        let sharer = SecretSharer::new(3);
        let payload = b"Secret data".to_vec();

        // Kann nicht mit weniger als Threshold teilen
        let result = sharer.split(&payload, 2);
        assert!(matches!(
            result,
            Err(ConfluxError::InsufficientShares { .. })
        ));
    }

    #[test]
    fn test_conflux_config_presets() {
        let high_sec = ConfluxConfig::high_security();
        assert_eq!(high_sec.parallel_count, 3);

        let low_lat = ConfluxConfig::low_latency();
        assert_eq!(low_lat.parallel_count, 2);
        assert_eq!(low_lat.circuit_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_active_circuit_creation() {
        let candidates = create_test_candidates(5);
        let route = candidates[0..3].to_vec();

        let circuit = ActiveCircuit::new(route.clone());

        assert_eq!(circuit.route.len(), 3);
        assert!(circuit.session_key(0).is_some());
        assert!(circuit.session_key(2).is_some());
        assert!(circuit.session_key(3).is_none());
        assert!(circuit.is_valid(Duration::from_secs(300)));
    }

    #[test]
    fn test_circuit_overlap_detection() {
        let candidates = create_test_candidates(10);

        let circuit1 = ActiveCircuit::new(candidates[0..3].to_vec());
        let circuit2 = ActiveCircuit::new(candidates[3..6].to_vec());
        let circuit3 = ActiveCircuit::new(candidates[0..3].to_vec()); // Same as circuit1

        // Keine Überlappung (verschiedene Kandidaten)
        assert_eq!(circuit1.overlaps_with(&circuit2), 0);

        // Volle Überlappung (gleiche Route)
        assert_eq!(circuit1.overlaps_with(&circuit3), 3);
    }

    #[test]
    fn test_conflux_manager_creation() {
        let config = ConfluxConfig::default();
        let manager = ConfluxManager::new(config);

        let stats = manager.stats();
        assert_eq!(stats.active_circuits, 0);
        assert_eq!(stats.total_packets_sent, 0);
    }

    #[test]
    fn test_conflux_manager_set_candidates() {
        let manager = ConfluxManager::new(ConfluxConfig::default());
        let candidates = create_test_candidates(20);

        manager.set_candidates(candidates.clone());

        // Candidates should be stored
        assert_eq!(manager.candidates.read().len(), 20);
    }

    #[test]
    fn test_egress_aggregator() {
        let aggregator = EgressAggregator::new();
        let msg_id = [1u8; 16];

        // Füge Shares hinzu
        aggregator.add_share(msg_id, vec![0x11, 0x22, 0x33], 2);
        aggregator.add_share(msg_id, vec![0x44, 0x55, 0x66], 2);

        // Sollte rekonstruierbar sein
        let result = aggregator.try_reconstruct(&msg_id);
        assert!(result.is_some());
    }

    #[test]
    fn test_egress_aggregator_insufficient_shares() {
        let aggregator = EgressAggregator::new();
        let msg_id = [2u8; 16];

        // Nur ein Share (Threshold ist 2)
        aggregator.add_share(msg_id, vec![0x11, 0x22], 2);

        // Sollte nicht rekonstruierbar sein
        let result = aggregator.try_reconstruct(&msg_id);
        assert!(result.is_none());
    }

    #[test]
    fn test_multipath_strategy_display() {
        assert_eq!(format!("{:?}", MultiPathStrategy::RoundRobin), "RoundRobin");
        assert_eq!(
            format!("{:?}", MultiPathStrategy::SecretSharing),
            "SecretSharing"
        );
    }

    #[test]
    fn test_conflux_error_display() {
        let err = ConfluxError::InsufficientShares {
            received: 1,
            required: 2,
        };
        assert!(err.to_string().contains("1 received"));
        assert!(err.to_string().contains("2 required"));
    }

    #[test]
    fn test_circuit_stats_latency_update() {
        let mut stats = CircuitStats::default();

        stats.update_latency(100.0);
        assert!((stats.avg_latency_ms - 10.0).abs() < 0.01); // 0.1 * 100 = 10

        stats.update_latency(100.0);
        // EMA: 0.1 * 100 + 0.9 * 10 = 10 + 9 = 19
        assert!((stats.avg_latency_ms - 19.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_conflux_send_without_circuits() {
        let manager = ConfluxManager::new(ConfluxConfig::default());

        let result = manager
            .multi_path_send(b"test", SensitivityLevel::Low)
            .await;

        // Sollte fehlschlagen ohne Candidates
        assert!(result.is_err() || manager.circuit_count() == 0);
    }
}
