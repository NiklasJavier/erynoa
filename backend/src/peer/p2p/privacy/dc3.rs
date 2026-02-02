//! # DC3 – Dynamic Challenge-based Cumulative Contribution (V2.5)
//!
//! Vollständig automatisiertes Trust-Bootstrap-System ohne Token oder soziale Elemente.
//!
//! ## Motivation
//!
//! Ersetzt Token-Stake und Guild-Vouching durch:
//! - **Keine Eintrittsbarriere** durch Kapitalbedarf
//! - **Direkte Korrelation** zu Netzwerk-Nutzen
//! - **Nicht-übertragbar** (kein Markt für "Trust")
//! - **Keine Kollusion** (keine sozialen Elemente)
//!
//! ## Axiom-Referenzen
//!
//! - **RL1a**: Cold-Start Bootstrap mit DC3
//! - **RL-V1**: Storage Proof-of-Retrievability
//! - **RL-V2**: Bandwidth Relay-Receipt-Chain
//! - **RL-V3**: Compute ZK-Shuffle-Proof
//!
//! ## Wire-Format
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │               DC3 – DYNAMIC CHALLENGE-BASED CUMULATIVE CONTRIBUTION         │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐           │
//! │  │  STORAGE-CHAL.   │  │   RELAY-CHAL.    │  │   MIXING-CHAL.   │           │
//! │  │  ─────────────   │  │   ──────────     │  │   ───────────    │           │
//! │  │  Merkle-PoR      │  │  Attestationen   │  │  ZK-Shuffle      │           │
//! │  └────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘           │
//! │           │                     │                     │                     │
//! │           └─────────────────────┼─────────────────────┘                     │
//! │                                 ▼                                           │
//! │                    ┌────────────────────────┐                               │
//! │                    │   VRF-CHALLENGE-GEN    │                               │
//! │                    │   (Nicht vorhersagbar) │                               │
//! │                    └────────────┬───────────┘                               │
//! │                                 ▼                                           │
//! │                    ┌────────────────────────┐                               │
//! │                    │  CUMULATIVE SCORE      │                               │
//! │                    │  = Σ(quality × weight) │                               │
//! │                    └────────────────────────┘                               │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```

use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ============================================================================
// CONSTANTS
// ============================================================================

/// VRF-Challenge Interval (durchschnittlich)
pub const CHALLENGE_INTERVAL_SECS: u64 = 3600; // 1 Stunde

/// Challenge-Response Timeout
pub const CHALLENGE_TIMEOUT_SECS: u64 = 300; // 5 Minuten

/// Minimum Score für Contribution-Tracking
pub const MIN_CONTRIBUTION_SCORE: f64 = 0.01;

/// Maximum Score-Akkumulation pro Tag
pub const MAX_DAILY_SCORE: f64 = 0.1;

/// Quality-Bonus Threshold
pub const QUALITY_BONUS_THRESHOLD: f64 = 0.95;

// ============================================================================
// CHALLENGE TYPES
// ============================================================================

/// Challenge-Typ (VRF-basiert ausgewählt)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChallengeType {
    /// Storage-Challenge: Proof-of-Retrievability (RL-V1)
    Storage,

    /// Relay-Challenge: Bandwidth-Attestation (RL-V2)
    Relay,

    /// Mixing-Challenge: ZK-Shuffle-Proof (RL-V3)
    Mixing,

    /// Uptime-Challenge: Verfügbarkeitsnachweis
    Uptime,
}

impl ChallengeType {
    /// Gewicht für Score-Berechnung
    pub fn weight(&self) -> f64 {
        match self {
            Self::Storage => 0.25,
            Self::Relay => 0.35,
            Self::Mixing => 0.30,
            Self::Uptime => 0.10,
        }
    }

    /// Durchschnittliche Challenge-Dauer
    pub fn expected_duration(&self) -> Duration {
        match self {
            Self::Storage => Duration::from_secs(60),
            Self::Relay => Duration::from_secs(120),
            Self::Mixing => Duration::from_secs(30),
            Self::Uptime => Duration::from_secs(5),
        }
    }

    /// Name für Logging
    pub fn name(&self) -> &'static str {
        match self {
            Self::Storage => "storage",
            Self::Relay => "relay",
            Self::Mixing => "mixing",
            Self::Uptime => "uptime",
        }
    }
}

// ============================================================================
// DYNAMIC CHALLENGE (VRF-BASED)
// ============================================================================

/// Dynamische Challenge (VRF-basiert)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicChallenge {
    /// Challenge-ID (unique)
    pub id: [u8; 32],

    /// Challenge-Typ
    pub challenge_type: ChallengeType,

    /// VRF-Proof (für Verifizierbarkeit)
    pub vrf_proof: Vec<u8>,

    /// Challenge-Parameter
    pub params: ChallengeParams,

    /// Erstellt-Zeitstempel
    pub created_at: u64,

    /// Deadline
    pub deadline: u64,
}

impl DynamicChallenge {
    /// Erstelle neue Challenge
    pub fn new(challenge_type: ChallengeType, params: ChallengeParams) -> Self {
        let mut id = [0u8; 32];
        getrandom::getrandom(&mut id).expect("RNG failed");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            challenge_type,
            vrf_proof: vec![], // Placeholder
            params,
            created_at: now,
            deadline: now + CHALLENGE_TIMEOUT_SECS,
        }
    }

    /// Ist Challenge abgelaufen?
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        now > self.deadline
    }

    /// Verbleibende Zeit
    pub fn remaining(&self) -> Duration {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now >= self.deadline {
            Duration::ZERO
        } else {
            Duration::from_secs(self.deadline - now)
        }
    }
}

// ============================================================================
// CHALLENGE PARAMETERS
// ============================================================================

/// Challenge-Parameter (typ-spezifisch)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeParams {
    /// Storage: Merkle-Proof Challenge
    Storage {
        /// Root-Hash des zu prüfenden Chunks
        root_hash: [u8; 32],
        /// Index des abzufragenden Leaves
        leaf_index: u64,
        /// Erwartete Chunk-Größe
        expected_size: usize,
    },

    /// Relay: Bandwidth-Attestation Challenge
    Relay {
        /// Ziel-Peer für Test-Relay
        target_peer: PeerId,
        /// Minimum Bandwidth (bytes/sec)
        min_bandwidth: u64,
        /// Test-Payload-Größe
        payload_size: usize,
    },

    /// Mixing: ZK-Shuffle Challenge
    Mixing {
        /// Batch-ID
        batch_id: [u8; 32],
        /// Anzahl zu mischender Messages
        batch_size: usize,
    },

    /// Uptime: Ping-Challenge
    Uptime {
        /// Nonce für Antwort
        nonce: [u8; 16],
    },
}

// ============================================================================
// CHALLENGE RESPONSE
// ============================================================================

/// Challenge-Antwort
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    /// Challenge-ID
    pub challenge_id: [u8; 32],

    /// Antwort-Proof
    pub proof: ResponseProof,

    /// Antwort-Zeitpunkt
    pub responded_at: u64,

    /// Response-Latenz (ms)
    pub latency_ms: u64,
}

/// Antwort-Proof (typ-spezifisch)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseProof {
    /// Storage: Merkle-Proof
    Storage {
        /// Merkle-Path
        merkle_path: Vec<[u8; 32]>,
        /// Leaf-Data
        leaf_data: Vec<u8>,
    },

    /// Relay: Bilateral-Attestation
    Relay {
        /// Sender-Signatur
        sender_sig: Vec<u8>,
        /// Empfänger-Signatur
        receiver_sig: Vec<u8>,
        /// Übertragene Bytes
        bytes_transferred: u64,
    },

    /// Mixing: ZK-Shuffle-Proof
    Mixing {
        /// ZK-Proof (komprimiert)
        zk_proof: Vec<u8>,
        /// Output-Commitment
        output_commitment: [u8; 32],
    },

    /// Uptime: Signed-Pong
    Uptime {
        /// Signierte Nonce
        signed_nonce: Vec<u8>,
    },
}

// ============================================================================
// CHALLENGE RESULT
// ============================================================================

/// Challenge-Ergebnis
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChallengeResult {
    /// Erfolgreich bestanden
    Passed {
        /// Quality-Score (0.0 - 1.0)
        quality: f64,
    },

    /// Fehlgeschlagen
    Failed {
        /// Fehler-Grund
        reason: ChallengeFailReason,
    },

    /// Timeout (keine Antwort)
    Timeout,

    /// Pending (noch nicht bewertet)
    Pending,
}

/// Gründe für Challenge-Fehlschlag
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeFailReason {
    /// Ungültiger Proof
    InvalidProof,
    /// Falsche Daten
    WrongData,
    /// Zu langsam
    TooSlow,
    /// Unvollständig
    Incomplete,
}

impl ChallengeResult {
    /// Ist bestanden?
    pub fn is_passed(&self) -> bool {
        matches!(self, Self::Passed { .. })
    }

    /// Quality-Score (0.0 wenn nicht bestanden)
    pub fn quality(&self) -> f64 {
        match self {
            Self::Passed { quality } => *quality,
            _ => 0.0,
        }
    }
}

// ============================================================================
// CUMULATIVE CONTRIBUTION SCORE
// ============================================================================

/// Kumulativer Contributions-Score (DC3)
///
/// ```text
/// Score = Σ(challenge_score × category_weight × quality_bonus)
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CumulativeContributionScore {
    /// Storage-Beiträge
    pub storage_score: f64,
    /// Relay-Beiträge
    pub relay_score: f64,
    /// Mixing-Beiträge
    pub mixing_score: f64,
    /// Uptime-Beiträge
    pub uptime_score: f64,

    /// Gesamtzahl abgeschlossener Challenges
    pub total_challenges: u32,
    /// Erfolgreiche Challenges
    pub successful_challenges: u32,

    /// Letztes Update
    pub last_updated: u64,
}

impl CumulativeContributionScore {
    /// Gesamtscore berechnen
    pub fn total_score(&self) -> f64 {
        self.storage_score * ChallengeType::Storage.weight()
            + self.relay_score * ChallengeType::Relay.weight()
            + self.mixing_score * ChallengeType::Mixing.weight()
            + self.uptime_score * ChallengeType::Uptime.weight()
    }

    /// Success-Rate
    pub fn success_rate(&self) -> f64 {
        if self.total_challenges == 0 {
            return 0.0;
        }
        self.successful_challenges as f64 / self.total_challenges as f64
    }

    /// DC3-Score für Eligibility
    pub fn dc3_score(&self) -> f64 {
        let base = self.total_score();

        // Quality-Bonus für hohe Success-Rate
        let quality_bonus = if self.success_rate() >= QUALITY_BONUS_THRESHOLD {
            1.1
        } else {
            1.0
        };

        (base * quality_bonus).min(1.0)
    }

    /// Registriere Challenge-Ergebnis
    pub fn record_challenge(&mut self, challenge_type: ChallengeType, result: ChallengeResult) {
        self.total_challenges += 1;

        if let ChallengeResult::Passed { quality } = result {
            self.successful_challenges += 1;

            // Score-Inkrement basierend auf Quality
            let increment = quality * 0.02; // Max 0.02 pro Challenge

            match challenge_type {
                ChallengeType::Storage => {
                    self.storage_score = (self.storage_score + increment).min(1.0)
                }
                ChallengeType::Relay => self.relay_score = (self.relay_score + increment).min(1.0),
                ChallengeType::Mixing => {
                    self.mixing_score = (self.mixing_score + increment).min(1.0)
                }
                ChallengeType::Uptime => {
                    self.uptime_score = (self.uptime_score + increment).min(1.0)
                }
            }
        }

        self.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

// ============================================================================
// CHALLENGE GENERATOR
// ============================================================================

/// VRF-basierter Challenge-Generator
pub struct ChallengeGenerator {
    /// Pending Challenges pro Peer
    pending: HashMap<PeerId, Vec<DynamicChallenge>>,
    /// Challenge-History für Rate-Limiting
    history: HashMap<PeerId, Vec<u64>>,
    /// Maximum pending Challenges pro Peer
    max_pending: usize,
}

impl ChallengeGenerator {
    /// Erstelle neuen Challenge-Generator
    pub fn new(max_pending: usize) -> Self {
        Self {
            pending: HashMap::new(),
            history: HashMap::new(),
            max_pending,
        }
    }

    /// Generiere neue Challenge für Peer (VRF-basiert)
    pub fn generate_challenge(&mut self, peer_id: PeerId) -> Option<DynamicChallenge> {
        // Rate-Limiting Check
        let pending_count = self.pending.get(&peer_id).map(|v| v.len()).unwrap_or(0);
        if pending_count >= self.max_pending {
            return None;
        }

        // VRF-basierte Typ-Auswahl (vor mutable borrow)
        let challenge_type = self.select_challenge_type();
        let params = self.generate_params(challenge_type);
        let challenge = DynamicChallenge::new(challenge_type, params);

        // Jetzt mutable borrow für insert
        self.pending
            .entry(peer_id)
            .or_default()
            .push(challenge.clone());

        // History updaten
        self.history
            .entry(peer_id)
            .or_default()
            .push(challenge.created_at);

        Some(challenge)
    }

    /// Wähle Challenge-Typ (VRF-simuliert)
    fn select_challenge_type(&self) -> ChallengeType {
        let mut rng_bytes = [0u8; 4];
        getrandom::getrandom(&mut rng_bytes).expect("RNG failed");
        let rng = u32::from_le_bytes(rng_bytes) as f64 / u32::MAX as f64;

        // Gewichtete Auswahl
        if rng < 0.25 {
            ChallengeType::Storage
        } else if rng < 0.60 {
            ChallengeType::Relay
        } else if rng < 0.90 {
            ChallengeType::Mixing
        } else {
            ChallengeType::Uptime
        }
    }

    /// Generiere Challenge-Parameter
    fn generate_params(&self, challenge_type: ChallengeType) -> ChallengeParams {
        match challenge_type {
            ChallengeType::Storage => {
                let mut root_hash = [0u8; 32];
                getrandom::getrandom(&mut root_hash).ok();
                let mut rng_bytes = [0u8; 8];
                getrandom::getrandom(&mut rng_bytes).ok();
                let leaf_index = u64::from_le_bytes(rng_bytes) % 1000;

                ChallengeParams::Storage {
                    root_hash,
                    leaf_index,
                    expected_size: 1024,
                }
            }
            ChallengeType::Relay => ChallengeParams::Relay {
                target_peer: PeerId::random(),
                min_bandwidth: 1_000_000, // 1 MB/s
                payload_size: 65536,
            },
            ChallengeType::Mixing => {
                let mut batch_id = [0u8; 32];
                getrandom::getrandom(&mut batch_id).ok();

                ChallengeParams::Mixing {
                    batch_id,
                    batch_size: 10,
                }
            }
            ChallengeType::Uptime => {
                let mut nonce = [0u8; 16];
                getrandom::getrandom(&mut nonce).ok();

                ChallengeParams::Uptime { nonce }
            }
        }
    }

    /// Hole pending Challenge für Peer
    pub fn get_pending(&self, peer_id: &PeerId) -> Option<&[DynamicChallenge]> {
        self.pending.get(peer_id).map(|v| v.as_slice())
    }

    /// Entferne Challenge nach Bearbeitung
    pub fn remove_challenge(&mut self, peer_id: &PeerId, challenge_id: &[u8; 32]) -> bool {
        if let Some(pending) = self.pending.get_mut(peer_id) {
            if let Some(pos) = pending.iter().position(|c| &c.id == challenge_id) {
                pending.remove(pos);
                return true;
            }
        }
        false
    }

    /// Bereinige abgelaufene Challenges
    pub fn cleanup_expired(&mut self) {
        for pending in self.pending.values_mut() {
            pending.retain(|c| !c.is_expired());
        }
    }

    /// Anzahl aktiver Challenges
    pub fn active_count(&self) -> usize {
        self.pending.values().map(|v| v.len()).sum()
    }
}

impl Default for ChallengeGenerator {
    fn default() -> Self {
        Self::new(5) // Max 5 pending pro Peer
    }
}

// ============================================================================
// DC3 SERVICE
// ============================================================================

/// DC3 Service - Koordiniert Challenge-System
pub struct DC3Service {
    /// Challenge-Generator
    generator: ChallengeGenerator,
    /// Scores pro Peer
    scores: HashMap<PeerId, CumulativeContributionScore>,
    /// Service-Start
    started_at: Instant,
}

impl DC3Service {
    /// Erstelle neuen DC3-Service
    pub fn new() -> Self {
        Self {
            generator: ChallengeGenerator::default(),
            scores: HashMap::new(),
            started_at: Instant::now(),
        }
    }

    /// Registriere neuen Peer
    pub fn register_peer(&mut self, peer_id: PeerId) {
        self.scores
            .entry(peer_id)
            .or_insert_with(CumulativeContributionScore::default);
    }

    /// Generiere Challenge für Peer
    pub fn issue_challenge(&mut self, peer_id: PeerId) -> Option<DynamicChallenge> {
        self.register_peer(peer_id);
        self.generator.generate_challenge(peer_id)
    }

    /// Verarbeite Challenge-Response
    pub fn process_response(
        &mut self,
        peer_id: &PeerId,
        response: &ChallengeResponse,
    ) -> ChallengeResult {
        // Finde zugehörige Challenge
        let challenge = self
            .generator
            .get_pending(peer_id)
            .and_then(|pending| pending.iter().find(|c| c.id == response.challenge_id));

        let Some(challenge) = challenge else {
            return ChallengeResult::Failed {
                reason: ChallengeFailReason::InvalidProof,
            };
        };

        // Verifiziere Response (vereinfacht)
        let result = self.verify_response(challenge, response);

        // Score updaten
        if let Some(score) = self.scores.get_mut(peer_id) {
            score.record_challenge(challenge.challenge_type, result);
        }

        // Challenge entfernen
        self.generator
            .remove_challenge(peer_id, &response.challenge_id);

        result
    }

    /// Verifiziere Challenge-Response
    fn verify_response(
        &self,
        challenge: &DynamicChallenge,
        response: &ChallengeResponse,
    ) -> ChallengeResult {
        // Timeout-Check
        if challenge.is_expired() {
            return ChallengeResult::Timeout;
        }

        // Latenz-basierte Quality
        let max_latency = challenge.challenge_type.expected_duration().as_millis() as u64;
        let quality = if response.latency_ms <= max_latency {
            1.0 - (response.latency_ms as f64 / max_latency as f64) * 0.3
        } else {
            0.7 * (max_latency as f64 / response.latency_ms as f64)
        };

        // Proof-Verifikation (vereinfacht - Placeholder)
        let proof_valid = match (&challenge.params, &response.proof) {
            (ChallengeParams::Storage { .. }, ResponseProof::Storage { merkle_path, .. }) => {
                !merkle_path.is_empty()
            }
            (
                ChallengeParams::Relay { .. },
                ResponseProof::Relay {
                    bytes_transferred, ..
                },
            ) => *bytes_transferred > 0,
            (ChallengeParams::Mixing { .. }, ResponseProof::Mixing { zk_proof, .. }) => {
                !zk_proof.is_empty()
            }
            (ChallengeParams::Uptime { .. }, ResponseProof::Uptime { signed_nonce }) => {
                !signed_nonce.is_empty()
            }
            _ => false,
        };

        if proof_valid {
            ChallengeResult::Passed {
                quality: quality.max(0.1).min(1.0),
            }
        } else {
            ChallengeResult::Failed {
                reason: ChallengeFailReason::InvalidProof,
            }
        }
    }

    /// Hole DC3-Score für Peer
    pub fn get_score(&self, peer_id: &PeerId) -> Option<&CumulativeContributionScore> {
        self.scores.get(peer_id)
    }

    /// Hole DC3-Score Wert für Eligibility
    pub fn get_dc3_score(&self, peer_id: &PeerId) -> f64 {
        self.scores
            .get(peer_id)
            .map(|s| s.dc3_score())
            .unwrap_or(0.0)
    }

    /// Hole abgeschlossene Challenges
    pub fn get_completed_challenges(&self, peer_id: &PeerId) -> u32 {
        self.scores
            .get(peer_id)
            .map(|s| s.successful_challenges)
            .unwrap_or(0)
    }

    /// Bereinige abgelaufene Challenges
    pub fn cleanup(&mut self) {
        self.generator.cleanup_expired();
    }

    /// Statistiken
    pub fn stats(&self) -> DC3Stats {
        DC3Stats {
            registered_peers: self.scores.len(),
            active_challenges: self.generator.active_count(),
            uptime_secs: self.started_at.elapsed().as_secs(),
        }
    }
}

impl Default for DC3Service {
    fn default() -> Self {
        Self::new()
    }
}

/// DC3 Service Statistiken
#[derive(Debug, Clone)]
pub struct DC3Stats {
    /// Registrierte Peers
    pub registered_peers: usize,
    /// Aktive Challenges
    pub active_challenges: usize,
    /// Uptime in Sekunden
    pub uptime_secs: u64,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_challenge_type_weights() {
        let total: f64 = [
            ChallengeType::Storage,
            ChallengeType::Relay,
            ChallengeType::Mixing,
            ChallengeType::Uptime,
        ]
        .iter()
        .map(|t| t.weight())
        .sum();

        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_dynamic_challenge_creation() {
        let params = ChallengeParams::Uptime { nonce: [0u8; 16] };
        let challenge = DynamicChallenge::new(ChallengeType::Uptime, params);

        assert!(!challenge.is_expired());
        assert!(challenge.remaining() > Duration::ZERO);
        assert_eq!(challenge.id.len(), 32);
    }

    #[test]
    fn test_cumulative_score_calculation() {
        let mut score = CumulativeContributionScore::default();

        // Registriere einige Challenges
        for _ in 0..10 {
            score.record_challenge(
                ChallengeType::Storage,
                ChallengeResult::Passed { quality: 0.9 },
            );
        }
        for _ in 0..5 {
            score.record_challenge(
                ChallengeType::Relay,
                ChallengeResult::Passed { quality: 0.8 },
            );
        }

        assert_eq!(score.total_challenges, 15);
        assert_eq!(score.successful_challenges, 15);
        assert!((score.success_rate() - 1.0).abs() < 0.001);
        assert!(score.dc3_score() > 0.0);
    }

    #[test]
    fn test_score_with_failures() {
        let mut score = CumulativeContributionScore::default();

        // 7 Erfolge, 3 Fehlschläge
        for _ in 0..7 {
            score.record_challenge(
                ChallengeType::Mixing,
                ChallengeResult::Passed { quality: 0.9 },
            );
        }
        for _ in 0..3 {
            score.record_challenge(
                ChallengeType::Mixing,
                ChallengeResult::Failed {
                    reason: ChallengeFailReason::TooSlow,
                },
            );
        }

        assert_eq!(score.total_challenges, 10);
        assert_eq!(score.successful_challenges, 7);
        assert!((score.success_rate() - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_challenge_generator() {
        let mut generator = ChallengeGenerator::new(3);
        let peer_id = PeerId::random();

        // Generiere Challenges
        let c1 = generator.generate_challenge(peer_id);
        let c2 = generator.generate_challenge(peer_id);
        let c3 = generator.generate_challenge(peer_id);
        let c4 = generator.generate_challenge(peer_id); // Sollte None sein

        assert!(c1.is_some());
        assert!(c2.is_some());
        assert!(c3.is_some());
        assert!(c4.is_none()); // Max pending erreicht

        // Entferne eine Challenge
        let id = c1.unwrap().id;
        assert!(generator.remove_challenge(&peer_id, &id));

        // Jetzt sollte wieder eine möglich sein
        assert!(generator.generate_challenge(peer_id).is_some());
    }

    #[test]
    fn test_dc3_service() {
        let mut service = DC3Service::new();
        let peer_id = PeerId::random();

        // Registriere Peer
        service.register_peer(peer_id);
        assert!(service.get_score(&peer_id).is_some());

        // Issue Challenge
        let challenge = service.issue_challenge(peer_id);
        assert!(challenge.is_some());

        // Stats prüfen
        let stats = service.stats();
        assert_eq!(stats.registered_peers, 1);
        assert!(stats.active_challenges > 0);
    }

    #[test]
    fn test_challenge_result_quality() {
        let passed = ChallengeResult::Passed { quality: 0.85 };
        let failed = ChallengeResult::Failed {
            reason: ChallengeFailReason::InvalidProof,
        };
        let timeout = ChallengeResult::Timeout;

        assert!(passed.is_passed());
        assert!((passed.quality() - 0.85).abs() < 0.001);

        assert!(!failed.is_passed());
        assert!((failed.quality() - 0.0).abs() < 0.001);

        assert!(!timeout.is_passed());
    }

    #[test]
    fn test_quality_bonus() {
        let mut high_quality = CumulativeContributionScore::default();
        let mut low_quality = CumulativeContributionScore::default();

        // High quality: 100% success
        for _ in 0..20 {
            high_quality.record_challenge(
                ChallengeType::Relay,
                ChallengeResult::Passed { quality: 1.0 },
            );
        }

        // Low quality: 50% success
        for _ in 0..10 {
            low_quality.record_challenge(
                ChallengeType::Relay,
                ChallengeResult::Passed { quality: 1.0 },
            );
        }
        for _ in 0..10 {
            low_quality.record_challenge(
                ChallengeType::Relay,
                ChallengeResult::Failed {
                    reason: ChallengeFailReason::TooSlow,
                },
            );
        }

        // High quality sollte Bonus bekommen
        assert!(high_quality.success_rate() >= QUALITY_BONUS_THRESHOLD);
        assert!(low_quality.success_rate() < QUALITY_BONUS_THRESHOLD);
    }
}
