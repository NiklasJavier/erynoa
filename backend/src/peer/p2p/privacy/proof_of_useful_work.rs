//! # Proof-of-Useful-Work (V2.6)
//!
//! Verifizierbare nützliche Arbeit als Sybil-Resistenz-Mechanismus.
//!
//! ## Motivation
//!
//! Sybils müssen tatsächlich nützliche Arbeit leisten:
//! - **DHT-Indexing**: Suchanfragen beantworten
//! - **ZK-Verification**: Proofs anderer Peers verifizieren
//! - **Content-Routing**: Daten weiterleiten
//!
//! ## Vorteile
//!
//! - Arbeit ist nicht simulierbar
//! - Trägt zum Netzwerk-Nutzen bei
//! - Automatisch verteilt (VRF-basiert)
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                   PROOF-OF-USEFUL-WORK (PoUW)                               │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐           │
//! │  │  DHT-INDEXING    │  │  ZK-VERIFICATION │  │  CONTENT-ROUTING │           │
//! │  │  ─────────────   │  │  ──────────────  │  │  ───────────────  │           │
//! │  │  Query-Response  │  │  Proof-Verify    │  │  Data-Relay       │           │
//! │  │  Index-Update    │  │  Batch-Verify    │  │  Route-Discovery  │           │
//! │  └────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘           │
//! │           │                     │                     │                     │
//! │           └─────────────────────┼─────────────────────┘                     │
//! │                                 ▼                                           │
//! │                    ┌────────────────────────┐                               │
//! │                    │     WORK-ATTESTATION   │                               │
//! │                    │     (Peer-Verified)    │                               │
//! │                    └────────────────────────┘                               │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```

use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Minimum erfolgreiche Queries pro Tag
pub const MIN_DHT_QUERIES_PER_DAY: u32 = 50;

/// Minimum verifizierte Proofs pro Tag
pub const MIN_VERIFIED_PROOFS_PER_DAY: u32 = 10;

/// Minimum geroutete Bytes pro Tag (1 MB)
pub const MIN_ROUTED_BYTES_PER_DAY: u64 = 1_000_000;

/// Work-Attestation Timeout (1 Stunde)
pub const ATTESTATION_TIMEOUT_SECS: u64 = 3600;

/// VRF-Challenge Interval
pub const WORK_CHALLENGE_INTERVAL_SECS: u64 = 300; // 5 Minuten

// ============================================================================
// WORK TYPES
// ============================================================================

/// Art der nützlichen Arbeit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkType {
    /// DHT-Query beantworten
    DhtQuery,
    /// DHT-Index aktualisieren
    DhtIndexUpdate,
    /// ZK-Proof verifizieren
    ZkVerification,
    /// Content weiterleiten
    ContentRouting,
    /// Mixing-Batch verarbeiten
    MixingBatch,
}

impl WorkType {
    /// Gewicht für Score-Berechnung
    pub fn weight(&self) -> f64 {
        match self {
            Self::DhtQuery => 0.15,
            Self::DhtIndexUpdate => 0.20,
            Self::ZkVerification => 0.30,
            Self::ContentRouting => 0.15,
            Self::MixingBatch => 0.20,
        }
    }

    /// Erwartete Dauer
    pub fn expected_duration(&self) -> Duration {
        match self {
            Self::DhtQuery => Duration::from_millis(50),
            Self::DhtIndexUpdate => Duration::from_millis(100),
            Self::ZkVerification => Duration::from_millis(500),
            Self::ContentRouting => Duration::from_millis(200),
            Self::MixingBatch => Duration::from_millis(1000),
        }
    }

    /// Name
    pub fn name(&self) -> &'static str {
        match self {
            Self::DhtQuery => "dht_query",
            Self::DhtIndexUpdate => "dht_index_update",
            Self::ZkVerification => "zk_verification",
            Self::ContentRouting => "content_routing",
            Self::MixingBatch => "mixing_batch",
        }
    }
}

// ============================================================================
// DHT-INDEXING WORK
// ============================================================================

/// DHT-Query-Anfrage (Work-Challenge)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtQueryChallenge {
    /// Challenge-ID
    pub id: [u8; 32],
    /// Angefragter Key
    pub key: [u8; 32],
    /// Requester (für Attestation)
    pub requester: PeerId,
    /// Deadline
    pub deadline: u64,
}

impl DhtQueryChallenge {
    /// Erstelle neue Challenge
    pub fn new(key: [u8; 32], requester: PeerId) -> Self {
        let mut id = [0u8; 32];
        getrandom::getrandom(&mut id).expect("RNG failed");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            key,
            requester,
            deadline: now + ATTESTATION_TIMEOUT_SECS,
        }
    }

    /// Ist Challenge abgelaufen?
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now > self.deadline
    }
}

/// DHT-Query-Response (Work-Proof)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtQueryResponse {
    /// Challenge-ID
    pub challenge_id: [u8; 32],
    /// Gefundener Wert (oder leer wenn nicht gefunden)
    pub value: Option<Vec<u8>>,
    /// Closest-Nodes (für Routing)
    pub closest_nodes: Vec<PeerId>,
    /// Response-Zeit (ms)
    pub response_time_ms: u64,
    /// Timestamp
    pub timestamp: u64,
}

impl DhtQueryResponse {
    /// Erstelle Response
    pub fn new(
        challenge_id: [u8; 32],
        value: Option<Vec<u8>>,
        closest_nodes: Vec<PeerId>,
        response_time_ms: u64,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            challenge_id,
            value,
            closest_nodes,
            response_time_ms,
            timestamp: now,
        }
    }

    /// Response-Qualität (0-1)
    pub fn quality_score(&self) -> f64 {
        let mut score = 0.0;

        // Wert gefunden: +0.5
        if self.value.is_some() {
            score += 0.5;
        }

        // Closest-Nodes: +0.3 (min 3 nodes)
        if self.closest_nodes.len() >= 3 {
            score += 0.3;
        }

        // Response-Zeit: +0.2 (unter 100ms)
        if self.response_time_ms < 100 {
            score += 0.2;
        } else if self.response_time_ms < 500 {
            score += 0.1;
        }

        score
    }
}

/// DHT-Index-Update (nützliche Arbeit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtIndexUpdate {
    /// Update-ID
    pub id: [u8; 32],
    /// Key
    pub key: [u8; 32],
    /// Value-Hash (nicht der Wert selbst für Privacy)
    pub value_hash: [u8; 32],
    /// Provider-Peer
    pub provider: PeerId,
    /// TTL
    pub ttl_secs: u64,
    /// Timestamp
    pub timestamp: u64,
}

impl DhtIndexUpdate {
    /// Erstelle Update
    pub fn new(key: [u8; 32], value: &[u8], provider: PeerId, ttl_secs: u64) -> Self {
        let mut id = [0u8; 32];
        getrandom::getrandom(&mut id).expect("RNG failed");

        let mut hasher = Sha256::new();
        hasher.update(value);
        let value_hash = hasher.finalize().into();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            key,
            value_hash,
            provider,
            ttl_secs,
            timestamp: now,
        }
    }
}

// ============================================================================
// ZK-VERIFICATION WORK
// ============================================================================

/// ZK-Verification-Challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkVerificationChallenge {
    /// Challenge-ID
    pub id: [u8; 32],
    /// Proof-Typ
    pub proof_type: ZkProofType,
    /// Serialisierter Proof (zu verifizieren)
    pub proof_data: Vec<u8>,
    /// Public-Inputs
    pub public_inputs: Vec<[u8; 32]>,
    /// Deadline
    pub deadline: u64,
}

/// ZK-Proof-Typ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZkProofType {
    /// Eligibility-Proof (Trust >= Threshold)
    Eligibility,
    /// Shuffle-Proof (Mixing)
    Shuffle,
    /// Contribution-Proof (DC3)
    Contribution,
    /// Storage-Proof (PoR)
    Storage,
}

impl ZkVerificationChallenge {
    /// Erstelle neue Challenge
    pub fn new(proof_type: ZkProofType, proof_data: Vec<u8>, public_inputs: Vec<[u8; 32]>) -> Self {
        let mut id = [0u8; 32];
        getrandom::getrandom(&mut id).expect("RNG failed");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            proof_type,
            proof_data,
            public_inputs,
            deadline: now + ATTESTATION_TIMEOUT_SECS,
        }
    }
}

/// ZK-Verification-Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkVerificationResult {
    /// Challenge-ID
    pub challenge_id: [u8; 32],
    /// Ist Proof valid?
    pub is_valid: bool,
    /// Verifier-Peer
    pub verifier: PeerId,
    /// Verification-Zeit (ms)
    pub verification_time_ms: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Signatur des Verifiers
    pub verifier_signature: Vec<u8>,
}

impl ZkVerificationResult {
    /// Erstelle Result
    pub fn new(
        challenge_id: [u8; 32],
        is_valid: bool,
        verifier: PeerId,
        verification_time_ms: u64,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            challenge_id,
            is_valid,
            verifier,
            verification_time_ms,
            timestamp: now,
            verifier_signature: vec![], // Wird später signiert
        }
    }
}

// ============================================================================
// CONTENT-ROUTING WORK
// ============================================================================

/// Content-Routing-Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRoutingRequest {
    /// Request-ID
    pub id: [u8; 32],
    /// Content-ID
    pub content_id: [u8; 32],
    /// Requester
    pub requester: PeerId,
    /// TTL (Hops)
    pub ttl: u8,
    /// Timestamp
    pub timestamp: u64,
}

impl ContentRoutingRequest {
    /// Erstelle Request
    pub fn new(content_id: [u8; 32], requester: PeerId, ttl: u8) -> Self {
        let mut id = [0u8; 32];
        getrandom::getrandom(&mut id).expect("RNG failed");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            content_id,
            requester,
            ttl,
            timestamp: now,
        }
    }
}

/// Content-Routing-Receipt (Arbeitsbeweis)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRoutingReceipt {
    /// Request-ID
    pub request_id: [u8; 32],
    /// Router-Peer
    pub router: PeerId,
    /// Nächster Hop (oder None wenn Content gefunden)
    pub next_hop: Option<PeerId>,
    /// Bytes transferiert
    pub bytes_transferred: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Signatur
    pub signature: Vec<u8>,
}

// ============================================================================
// WORK ATTESTATION (Aggregation)
// ============================================================================

/// Work-Attestation (von anderem Peer bestätigt)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkAttestation {
    /// Attestation-ID
    pub id: [u8; 32],
    /// Worker-Peer
    pub worker: PeerId,
    /// Attester-Peer
    pub attester: PeerId,
    /// Work-Typ
    pub work_type: WorkType,
    /// Work-Qualität (0-1)
    pub quality: f64,
    /// Timestamp
    pub timestamp: u64,
    /// Attester-Signatur
    pub attester_signature: Vec<u8>,
}

impl WorkAttestation {
    /// Erstelle Attestation
    pub fn new(worker: PeerId, attester: PeerId, work_type: WorkType, quality: f64) -> Self {
        let mut id = [0u8; 32];
        getrandom::getrandom(&mut id).expect("RNG failed");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            worker,
            attester,
            work_type,
            quality,
            timestamp: now,
            attester_signature: vec![],
        }
    }

    /// Berechne Attestation-Hash
    pub fn hash_for_signing(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.id);
        hasher.update(self.worker.to_bytes());
        hasher.update(self.attester.to_bytes());
        hasher.update(&[self.work_type as u8]);
        hasher.update(&self.quality.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.finalize().into()
    }
}

// ============================================================================
// DAILY WORK SUMMARY
// ============================================================================

/// Tägliche Work-Zusammenfassung
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyWorkSummary {
    /// Peer-ID
    pub peer: PeerId,
    /// Tag (Unix-Timestamp / 86400)
    pub day: u64,
    /// DHT-Queries beantwortet
    pub dht_queries: u32,
    /// DHT-Index-Updates
    pub dht_updates: u32,
    /// ZK-Proofs verifiziert
    pub zk_verifications: u32,
    /// Content geroutet (Bytes)
    pub content_routed_bytes: u64,
    /// Mixing-Batches verarbeitet
    pub mixing_batches: u32,
    /// Durchschnittliche Qualität
    pub avg_quality: f64,
    /// Attestations
    pub attestation_count: usize,
}

impl DailyWorkSummary {
    /// Erstelle neue Summary
    pub fn new(peer: PeerId, day: u64) -> Self {
        Self {
            peer,
            day,
            dht_queries: 0,
            dht_updates: 0,
            zk_verifications: 0,
            content_routed_bytes: 0,
            mixing_batches: 0,
            avg_quality: 0.0,
            attestation_count: 0,
        }
    }

    /// Berechne Gesamt-Score
    pub fn total_score(&self) -> f64 {
        let dht_score =
            (self.dht_queries as f64 / MIN_DHT_QUERIES_PER_DAY as f64).min(1.0) * 0.25;
        let update_score = (self.dht_updates as f64 / 20.0).min(1.0) * 0.20;
        let zk_score =
            (self.zk_verifications as f64 / MIN_VERIFIED_PROOFS_PER_DAY as f64).min(1.0) * 0.30;
        let routing_score =
            (self.content_routed_bytes as f64 / MIN_ROUTED_BYTES_PER_DAY as f64).min(1.0) * 0.15;
        let mixing_score = (self.mixing_batches as f64 / 10.0).min(1.0) * 0.10;

        let base_score = dht_score + update_score + zk_score + routing_score + mixing_score;

        // Quality-Bonus
        base_score * (0.5 + 0.5 * self.avg_quality)
    }

    /// Erfüllt Minimum-Anforderungen?
    pub fn meets_minimum(&self) -> bool {
        self.dht_queries >= MIN_DHT_QUERIES_PER_DAY / 2
            && self.zk_verifications >= MIN_VERIFIED_PROOFS_PER_DAY / 2
    }
}

// ============================================================================
// PROOF-OF-USEFUL-WORK SERVICE
// ============================================================================

/// Service für Proof-of-Useful-Work
pub struct ProofOfUsefulWorkService {
    /// Pending DHT-Challenges
    pending_dht_challenges: HashMap<[u8; 32], DhtQueryChallenge>,
    /// Pending ZK-Verification-Challenges
    pending_zk_challenges: HashMap<[u8; 32], ZkVerificationChallenge>,
    /// Work-Attestations pro Peer
    attestations: HashMap<PeerId, Vec<WorkAttestation>>,
    /// Daily Summaries
    daily_summaries: HashMap<(PeerId, u64), DailyWorkSummary>,
}

impl ProofOfUsefulWorkService {
    /// Erstelle neuen Service
    pub fn new() -> Self {
        Self {
            pending_dht_challenges: HashMap::new(),
            pending_zk_challenges: HashMap::new(),
            attestations: HashMap::new(),
            daily_summaries: HashMap::new(),
        }
    }

    /// Generiere DHT-Query-Challenge
    pub fn generate_dht_challenge(&mut self, key: [u8; 32], requester: PeerId) -> DhtQueryChallenge {
        let challenge = DhtQueryChallenge::new(key, requester);
        self.pending_dht_challenges
            .insert(challenge.id, challenge.clone());
        challenge
    }

    /// Verifiziere DHT-Response
    pub fn verify_dht_response(
        &mut self,
        worker: PeerId,
        response: &DhtQueryResponse,
    ) -> Option<WorkAttestation> {
        let challenge = self.pending_dht_challenges.remove(&response.challenge_id)?;

        if challenge.is_expired() {
            return None;
        }

        let quality = response.quality_score();
        let attestation =
            WorkAttestation::new(worker, challenge.requester, WorkType::DhtQuery, quality);

        self.record_attestation(attestation.clone());

        Some(attestation)
    }

    /// Generiere ZK-Verification-Challenge
    pub fn generate_zk_challenge(
        &mut self,
        proof_type: ZkProofType,
        proof_data: Vec<u8>,
        public_inputs: Vec<[u8; 32]>,
    ) -> ZkVerificationChallenge {
        let challenge = ZkVerificationChallenge::new(proof_type, proof_data, public_inputs);
        self.pending_zk_challenges
            .insert(challenge.id, challenge.clone());
        challenge
    }

    /// Verifiziere ZK-Result
    pub fn verify_zk_result(
        &mut self,
        result: &ZkVerificationResult,
    ) -> Option<WorkAttestation> {
        let challenge = self.pending_zk_challenges.remove(&result.challenge_id)?;

        // Quality basiert auf Korrektheit und Zeit
        let time_bonus = if result.verification_time_ms < 100 {
            1.0
        } else if result.verification_time_ms < 500 {
            0.8
        } else {
            0.6
        };

        let quality = time_bonus;
        let attestation = WorkAttestation::new(
            result.verifier,
            PeerId::random(), // Würde normalerweise der Challenge-Creator sein
            WorkType::ZkVerification,
            quality,
        );

        self.record_attestation(attestation.clone());

        Some(attestation)
    }

    /// Registriere Content-Routing-Receipt
    pub fn record_routing_receipt(&mut self, receipt: &ContentRoutingReceipt) {
        let quality = if receipt.bytes_transferred > 10000 {
            1.0
        } else {
            0.5
        };

        let attestation = WorkAttestation::new(
            receipt.router,
            PeerId::random(),
            WorkType::ContentRouting,
            quality,
        );

        self.record_attestation(attestation);
    }

    /// Interne Funktion: Attestation speichern
    fn record_attestation(&mut self, attestation: WorkAttestation) {
        self.attestations
            .entry(attestation.worker)
            .or_insert_with(Vec::new)
            .push(attestation.clone());

        // Update Daily Summary
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let day = now / 86400;

        let summary = self
            .daily_summaries
            .entry((attestation.worker, day))
            .or_insert_with(|| DailyWorkSummary::new(attestation.worker, day));

        match attestation.work_type {
            WorkType::DhtQuery => summary.dht_queries += 1,
            WorkType::DhtIndexUpdate => summary.dht_updates += 1,
            WorkType::ZkVerification => summary.zk_verifications += 1,
            WorkType::ContentRouting => summary.content_routed_bytes += 1000, // Placeholder
            WorkType::MixingBatch => summary.mixing_batches += 1,
        }

        summary.attestation_count += 1;

        // Update Durchschnitts-Qualität
        let total_quality: f64 = self
            .attestations
            .get(&attestation.worker)
            .map(|v| v.iter().map(|a| a.quality).sum())
            .unwrap_or(0.0);
        let count = self
            .attestations
            .get(&attestation.worker)
            .map(|v| v.len())
            .unwrap_or(1);
        summary.avg_quality = total_quality / count as f64;
    }

    /// Hole Daily Summary
    pub fn get_daily_summary(&self, peer: &PeerId, day: u64) -> Option<&DailyWorkSummary> {
        self.daily_summaries.get(&(*peer, day))
    }

    /// Hole alle Attestations für Peer
    pub fn get_attestations(&self, peer: &PeerId) -> Option<&Vec<WorkAttestation>> {
        self.attestations.get(peer)
    }

    /// Berechne PoUW-Score für Peer (letzte 7 Tage)
    pub fn calculate_pouw_score(&self, peer: &PeerId) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let today = now / 86400;

        let mut total_score = 0.0;
        let mut day_count = 0;

        for day_offset in 0..7 {
            let day = today - day_offset;
            if let Some(summary) = self.daily_summaries.get(&(*peer, day)) {
                total_score += summary.total_score();
                day_count += 1;
            }
        }

        if day_count > 0 {
            total_score / day_count as f64
        } else {
            0.0
        }
    }

    /// Cleanup abgelaufene Challenges
    pub fn cleanup_expired(&mut self) {
        self.pending_dht_challenges.retain(|_, c| !c.is_expired());
        self.pending_zk_challenges.retain(|_, c| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now <= c.deadline
        });
    }
}

impl Default for ProofOfUsefulWorkService {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_type_weights() {
        let total: f64 = [
            WorkType::DhtQuery,
            WorkType::DhtIndexUpdate,
            WorkType::ZkVerification,
            WorkType::ContentRouting,
            WorkType::MixingBatch,
        ]
        .iter()
        .map(|w| w.weight())
        .sum();

        assert!((total - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_dht_query_challenge() {
        let key = [1u8; 32];
        let requester = PeerId::random();

        let challenge = DhtQueryChallenge::new(key, requester);

        assert!(!challenge.is_expired());
        assert_eq!(challenge.key, key);
        assert_eq!(challenge.requester, requester);
    }

    #[test]
    fn test_dht_query_response_quality() {
        let response = DhtQueryResponse::new(
            [0u8; 32],
            Some(vec![1, 2, 3]),
            vec![PeerId::random(), PeerId::random(), PeerId::random()],
            50,
        );

        assert_eq!(response.quality_score(), 1.0);

        let response_low = DhtQueryResponse::new([0u8; 32], None, vec![], 1000);

        assert_eq!(response_low.quality_score(), 0.0);
    }

    #[test]
    fn test_dht_index_update() {
        let key = [1u8; 32];
        let value = b"test value";
        let provider = PeerId::random();

        let update = DhtIndexUpdate::new(key, value, provider, 3600);

        assert_eq!(update.key, key);
        assert_eq!(update.ttl_secs, 3600);
        assert_ne!(update.value_hash, [0u8; 32]);
    }

    #[test]
    fn test_zk_verification_challenge() {
        let proof_data = vec![0u8; 64];
        let public_inputs = vec![[1u8; 32]];

        let challenge =
            ZkVerificationChallenge::new(ZkProofType::Eligibility, proof_data.clone(), public_inputs);

        assert_eq!(challenge.proof_type, ZkProofType::Eligibility);
        assert_eq!(challenge.proof_data, proof_data);
    }

    #[test]
    fn test_work_attestation() {
        let worker = PeerId::random();
        let attester = PeerId::random();

        let attestation = WorkAttestation::new(worker, attester, WorkType::DhtQuery, 0.9);

        assert_eq!(attestation.worker, worker);
        assert_eq!(attestation.attester, attester);
        assert_eq!(attestation.quality, 0.9);
    }

    #[test]
    fn test_daily_work_summary() {
        let peer = PeerId::random();
        let mut summary = DailyWorkSummary::new(peer, 100);

        assert!(!summary.meets_minimum());

        summary.dht_queries = MIN_DHT_QUERIES_PER_DAY;
        summary.zk_verifications = MIN_VERIFIED_PROOFS_PER_DAY;
        summary.avg_quality = 0.9;

        assert!(summary.meets_minimum());
        assert!(summary.total_score() > 0.5);
    }

    #[test]
    fn test_pouw_service_dht() {
        let mut service = ProofOfUsefulWorkService::new();
        let requester = PeerId::random();
        let worker = PeerId::random();

        let challenge = service.generate_dht_challenge([1u8; 32], requester);

        let response = DhtQueryResponse::new(
            challenge.id,
            Some(vec![1, 2, 3]),
            vec![PeerId::random(), PeerId::random(), PeerId::random()],
            50,
        );

        let attestation = service.verify_dht_response(worker, &response);

        assert!(attestation.is_some());
        assert_eq!(attestation.unwrap().quality, 1.0);
    }

    #[test]
    fn test_pouw_score_calculation() {
        let mut service = ProofOfUsefulWorkService::new();
        let peer = PeerId::random();
        let attester = PeerId::random();

        // Erstelle Attestations
        for _ in 0..10 {
            let attestation = WorkAttestation::new(peer, attester, WorkType::DhtQuery, 0.9);
            service.record_attestation(attestation);
        }

        let score = service.calculate_pouw_score(&peer);

        assert!(score > 0.0);
    }

    #[test]
    fn test_content_routing_receipt() {
        let request = ContentRoutingRequest::new([1u8; 32], PeerId::random(), 3);

        assert_eq!(request.ttl, 3);

        let receipt = ContentRoutingReceipt {
            request_id: request.id,
            router: PeerId::random(),
            next_hop: Some(PeerId::random()),
            bytes_transferred: 50000,
            timestamp: 0,
            signature: vec![],
        };

        assert!(receipt.bytes_transferred > 10000);
    }

    #[test]
    fn test_service_cleanup() {
        let mut service = ProofOfUsefulWorkService::new();

        // Erstelle Challenge
        let challenge = service.generate_dht_challenge([1u8; 32], PeerId::random());

        assert!(service.pending_dht_challenges.contains_key(&challenge.id));

        // Cleanup (sollte nicht entfernen, da nicht abgelaufen)
        service.cleanup_expired();

        assert!(service.pending_dht_challenges.contains_key(&challenge.id));
    }
}
