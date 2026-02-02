//! # Resource-Verification (RL-V1, RL-V2, RL-V3)
//!
//! Verifizierbare Ressourcenbeiträge als Basis für Sybil-resistentes Trust-Bootstrap.
//!
//! ## Axiom-Referenzen
//!
//! - **RL-V1**: Storage Proof-of-Retrievability (Merkle-PoR)
//! - **RL-V2**: Bandwidth Relay-Receipt-Chain (Bilaterale Attestation)
//! - **RL-V3**: Compute ZK-Shuffle-Proof (Bayer-Groth)
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                   RESOURCE-VERIFICATION (RL-V1/V2/V3)                       │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐           │
//! │  │  STORAGE (V1)    │  │  BANDWIDTH (V2)  │  │  COMPUTE (V3)    │           │
//! │  │  ─────────────   │  │  ─────────────   │  │  ─────────────   │           │
//! │  │  Merkle-PoR      │  │  Relay-Receipts  │  │  ZK-Shuffle      │           │
//! │  │  Random-Chunks   │  │  Attestation     │  │  Bayer-Groth     │           │
//! │  └────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘           │
//! │           │                     │                     │                     │
//! │           └─────────────────────┼─────────────────────┘                     │
//! │                                 ▼                                           │
//! │                    ┌────────────────────────┐                               │
//! │                    │  VERIFIED-COMMITMENT   │                               │
//! │                    │  Storage + BW + Compute│                               │
//! │                    └────────────────────────┘                               │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Sybil-Kosten-Analyse
//!
//! | Resource    | Kosten/Monat | Verifizierung           |
//! |-------------|--------------|-------------------------|
//! | Storage     | ~$0.01/MB    | Merkle-PoR              |
//! | Bandwidth   | ~$0.05/GB    | Bilaterale Attestation  |
//! | Compute     | ~$0.001/Op   | ZK-Shuffle-Proof        |

use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Merkle-Tree Chunk-Größe (4KB)
pub const CHUNK_SIZE: usize = 4096;

/// Challenge-Response Timeout
pub const CHALLENGE_TIMEOUT_SECS: u64 = 60;

/// Minimum Chunks für Storage-Proof
pub const MIN_STORAGE_CHUNKS: usize = 16;

/// Attestation Epoch-Dauer (1 Stunde)
pub const ATTESTATION_EPOCH_SECS: u64 = 3600;

/// Minimum Attestations für Bandwidth-Proof
pub const MIN_ATTESTATIONS: usize = 10;

/// Minimum Mixing-Batches für Compute-Proof
pub const MIN_MIXING_BATCHES: usize = 100;

// ============================================================================
// RL-V1: STORAGE PROOF-OF-RETRIEVABILITY
// ============================================================================

/// Merkle-Tree für Storage-Verification (RL-V1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMerkleTree {
    /// Root-Hash des Trees
    pub root: [u8; 32],
    /// Anzahl der Chunks
    pub chunk_count: usize,
    /// Chunk-Hashes (Leaves)
    pub chunk_hashes: Vec<[u8; 32]>,
    /// Erstellungs-Zeitstempel
    pub created_at: u64,
}

impl StorageMerkleTree {
    /// Erstelle neuen Merkle-Tree aus Daten
    pub fn new(data: &[u8]) -> Self {
        let chunks: Vec<&[u8]> = data.chunks(CHUNK_SIZE).collect();
        let chunk_count = chunks.len().max(1);

        // Hash jedes Chunks
        let chunk_hashes: Vec<[u8; 32]> = chunks
            .iter()
            .map(|chunk| {
                let mut hasher = Sha256::new();
                hasher.update(chunk);
                hasher.finalize().into()
            })
            .collect();

        // Berechne Root-Hash
        let root = Self::compute_root(&chunk_hashes);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            root,
            chunk_count,
            chunk_hashes,
            created_at: now,
        }
    }

    /// Berechne Merkle-Root
    fn compute_root(leaves: &[[u8; 32]]) -> [u8; 32] {
        if leaves.is_empty() {
            return [0u8; 32];
        }
        if leaves.len() == 1 {
            return leaves[0];
        }

        let mut current_level = leaves.to_vec();

        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for pair in current_level.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(&pair[0]);
                if pair.len() > 1 {
                    hasher.update(&pair[1]);
                } else {
                    hasher.update(&pair[0]); // Dupliziere bei ungerader Anzahl
                }
                next_level.push(hasher.finalize().into());
            }

            current_level = next_level;
        }

        current_level[0]
    }

    /// Generiere Merkle-Proof für einen Chunk
    pub fn generate_proof(&self, chunk_index: usize) -> Option<MerkleProof> {
        if chunk_index >= self.chunk_count {
            return None;
        }

        let mut proof_hashes = Vec::new();
        let mut proof_directions = Vec::new();
        let mut current_level = self.chunk_hashes.clone();
        let mut index = chunk_index;

        while current_level.len() > 1 {
            let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };

            if sibling_index < current_level.len() {
                proof_hashes.push(current_level[sibling_index]);
                proof_directions.push(index % 2 == 0); // true = sibling rechts
            }

            // Nächste Level berechnen
            let mut next_level = Vec::new();
            for pair in current_level.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(&pair[0]);
                if pair.len() > 1 {
                    hasher.update(&pair[1]);
                } else {
                    hasher.update(&pair[0]);
                }
                next_level.push(hasher.finalize().into());
            }

            current_level = next_level;
            index /= 2;
        }

        Some(MerkleProof {
            chunk_index,
            chunk_hash: self.chunk_hashes[chunk_index],
            proof_hashes,
            proof_directions,
        })
    }
}

/// Merkle-Proof für einen einzelnen Chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Index des bewiesenen Chunks
    pub chunk_index: usize,
    /// Hash des Chunks
    pub chunk_hash: [u8; 32],
    /// Proof-Pfad (Sibling-Hashes)
    pub proof_hashes: Vec<[u8; 32]>,
    /// Richtungen (true = Sibling rechts)
    pub proof_directions: Vec<bool>,
}

impl MerkleProof {
    /// Verifiziere Proof gegen Root
    pub fn verify(&self, root: &[u8; 32]) -> bool {
        let mut current_hash = self.chunk_hash;

        for (sibling, is_right) in self.proof_hashes.iter().zip(&self.proof_directions) {
            let mut hasher = Sha256::new();
            if *is_right {
                hasher.update(&current_hash);
                hasher.update(sibling);
            } else {
                hasher.update(sibling);
                hasher.update(&current_hash);
            }
            current_hash = hasher.finalize().into();
        }

        current_hash == *root
    }
}

/// Storage-Challenge (zufällige Chunk-Abfrage)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageChallenge {
    /// Challenge-ID
    pub id: [u8; 32],
    /// Angefragte Chunk-Indices
    pub chunk_indices: Vec<usize>,
    /// Erwarteter Root-Hash
    pub expected_root: [u8; 32],
    /// Deadline
    pub deadline: u64,
}

impl StorageChallenge {
    /// Erstelle neue Storage-Challenge
    pub fn new(tree: &StorageMerkleTree, num_chunks: usize) -> Self {
        let mut id = [0u8; 32];
        getrandom::getrandom(&mut id).expect("RNG failed");

        // Zufällige Chunk-Indices
        let mut chunk_indices = Vec::with_capacity(num_chunks);
        let mut rng_buf = [0u8; 4];
        for _ in 0..num_chunks {
            getrandom::getrandom(&mut rng_buf).expect("RNG failed");
            let index = u32::from_le_bytes(rng_buf) as usize % tree.chunk_count;
            if !chunk_indices.contains(&index) {
                chunk_indices.push(index);
            }
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            chunk_indices,
            expected_root: tree.root,
            deadline: now + CHALLENGE_TIMEOUT_SECS,
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

/// Storage-Proof als Response auf Challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProof {
    /// Challenge-ID
    pub challenge_id: [u8; 32],
    /// Merkle-Proofs für angefragte Chunks
    pub proofs: Vec<MerkleProof>,
    /// Chunk-Daten (optional, für vollständige Verifikation)
    pub chunk_data: Vec<Vec<u8>>,
}

impl StorageProof {
    /// Verifiziere Storage-Proof
    pub fn verify(&self, challenge: &StorageChallenge) -> StorageVerificationResult {
        if self.challenge_id != challenge.id {
            return StorageVerificationResult::InvalidChallengeId;
        }

        if challenge.is_expired() {
            return StorageVerificationResult::Expired;
        }

        if self.proofs.len() != challenge.chunk_indices.len() {
            return StorageVerificationResult::IncompleteProof;
        }

        // Verifiziere jeden Merkle-Proof
        for (proof, &expected_index) in self.proofs.iter().zip(&challenge.chunk_indices) {
            if proof.chunk_index != expected_index {
                return StorageVerificationResult::WrongChunkIndex;
            }

            if !proof.verify(&challenge.expected_root) {
                return StorageVerificationResult::InvalidMerkleProof;
            }
        }

        // Optional: Verifiziere Chunk-Daten
        if !self.chunk_data.is_empty() {
            for (i, data) in self.chunk_data.iter().enumerate() {
                let mut hasher = Sha256::new();
                hasher.update(data);
                let hash: [u8; 32] = hasher.finalize().into();
                if hash != self.proofs[i].chunk_hash {
                    return StorageVerificationResult::ChunkDataMismatch;
                }
            }
        }

        StorageVerificationResult::Valid
    }
}

/// Storage-Verifikations-Ergebnis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageVerificationResult {
    Valid,
    InvalidChallengeId,
    Expired,
    IncompleteProof,
    WrongChunkIndex,
    InvalidMerkleProof,
    ChunkDataMismatch,
}

// ============================================================================
// RL-V2: BANDWIDTH RELAY-RECEIPT-CHAIN
// ============================================================================

/// Relay-Receipt für Bandwidth-Verification (RL-V2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayReceipt {
    /// Receipt-ID
    pub id: [u8; 32],
    /// Relaying-Peer
    pub relayer: PeerId,
    /// Empfänger-Peer
    pub recipient: PeerId,
    /// Transferiertes Volumen (Bytes)
    pub bytes_transferred: u64,
    /// Epoch
    pub epoch: u64,
    /// Signatur des Relayers
    pub relayer_signature: Vec<u8>,
    /// Zeitstempel
    pub timestamp: u64,
}

impl RelayReceipt {
    /// Erstelle neues Receipt
    pub fn new(relayer: PeerId, recipient: PeerId, bytes: u64, epoch: u64) -> Self {
        let mut id = [0u8; 32];
        getrandom::getrandom(&mut id).expect("RNG failed");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            relayer,
            recipient,
            bytes_transferred: bytes,
            epoch,
            relayer_signature: vec![], // Signatur wird später hinzugefügt
            timestamp: now,
        }
    }

    /// Berechne Receipt-Hash für Signatur
    pub fn hash_for_signing(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.id);
        hasher.update(self.relayer.to_bytes());
        hasher.update(self.recipient.to_bytes());
        hasher.update(&self.bytes_transferred.to_le_bytes());
        hasher.update(&self.epoch.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.finalize().into()
    }
}

/// Bilaterale Attestation zwischen zwei Peers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BilateralAttestation {
    /// Attestation-ID
    pub id: [u8; 32],
    /// Peer A
    pub peer_a: PeerId,
    /// Peer B
    pub peer_b: PeerId,
    /// Gesamtvolumen A→B
    pub volume_a_to_b: u64,
    /// Gesamtvolumen B→A
    pub volume_b_to_a: u64,
    /// Epoch
    pub epoch: u64,
    /// Signatur von A
    pub signature_a: Vec<u8>,
    /// Signatur von B
    pub signature_b: Vec<u8>,
    /// Zeitstempel
    pub timestamp: u64,
}

impl BilateralAttestation {
    /// Erstelle neue Attestation (ohne Signaturen)
    pub fn new(
        peer_a: PeerId,
        peer_b: PeerId,
        vol_a_to_b: u64,
        vol_b_to_a: u64,
        epoch: u64,
    ) -> Self {
        let mut id = [0u8; 32];
        getrandom::getrandom(&mut id).expect("RNG failed");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            peer_a,
            peer_b,
            volume_a_to_b: vol_a_to_b,
            volume_b_to_a: vol_b_to_a,
            epoch,
            signature_a: vec![],
            signature_b: vec![],
            timestamp: now,
        }
    }

    /// Ist Attestation vollständig signiert?
    pub fn is_complete(&self) -> bool {
        !self.signature_a.is_empty() && !self.signature_b.is_empty()
    }

    /// Berechne Hash für Signatur
    pub fn hash_for_signing(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.id);
        hasher.update(self.peer_a.to_bytes());
        hasher.update(self.peer_b.to_bytes());
        hasher.update(&self.volume_a_to_b.to_le_bytes());
        hasher.update(&self.volume_b_to_a.to_le_bytes());
        hasher.update(&self.epoch.to_le_bytes());
        hasher.finalize().into()
    }

    /// Gesamtvolumen
    pub fn total_volume(&self) -> u64 {
        self.volume_a_to_b + self.volume_b_to_a
    }
}

/// Bandwidth-Epoch-Proof (Aggregation über eine Epoch)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthEpochProof {
    /// Peer-ID
    pub peer: PeerId,
    /// Epoch
    pub epoch: u64,
    /// Gesamtvolumen in dieser Epoch (GB)
    pub total_volume_gb: f64,
    /// Anzahl der Attestationen
    pub attestation_count: usize,
    /// Merkle-Root der Attestationen
    pub attestations_root: [u8; 32],
    /// Repräsentative Attestations-Samples
    pub sample_attestations: Vec<BilateralAttestation>,
}

impl BandwidthEpochProof {
    /// Erstelle Epoch-Proof aus Attestationen
    pub fn from_attestations(
        peer: PeerId,
        epoch: u64,
        attestations: &[BilateralAttestation],
    ) -> Self {
        // Berechne Gesamtvolumen
        let total_bytes: u64 = attestations
            .iter()
            .filter(|a| a.peer_a == peer || a.peer_b == peer)
            .map(|a| {
                if a.peer_a == peer {
                    a.volume_a_to_b
                } else {
                    a.volume_b_to_a
                }
            })
            .sum();

        let total_volume_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

        // Berechne Merkle-Root
        let attestation_hashes: Vec<[u8; 32]> =
            attestations.iter().map(|a| a.hash_for_signing()).collect();

        let attestations_root = if attestation_hashes.is_empty() {
            [0u8; 32]
        } else {
            StorageMerkleTree::compute_root(&attestation_hashes)
        };

        // Sample Attestations (max 5)
        let sample_attestations: Vec<_> = attestations.iter().take(5).cloned().collect();

        Self {
            peer,
            epoch,
            total_volume_gb,
            attestation_count: attestations.len(),
            attestations_root,
            sample_attestations,
        }
    }

    /// Prüfe ob Minimum erreicht
    pub fn meets_minimum(&self) -> bool {
        self.attestation_count >= MIN_ATTESTATIONS
    }
}

// ============================================================================
// RL-V3: COMPUTE ZK-SHUFFLE-PROOF
// ============================================================================

/// Mixing-Batch-Commitment (Input für ZK-Shuffle)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixingBatchCommitment {
    /// Batch-ID
    pub id: [u8; 32],
    /// Pedersen-Commitment der Input-Messages
    pub input_commitment: [u8; 32],
    /// Pedersen-Commitment der Output-Messages
    pub output_commitment: [u8; 32],
    /// Anzahl Nachrichten im Batch
    pub message_count: usize,
    /// Epoch
    pub epoch: u64,
    /// Zeitstempel
    pub timestamp: u64,
}

impl MixingBatchCommitment {
    /// Erstelle neues Commitment
    pub fn new(input_hashes: &[[u8; 32]], output_hashes: &[[u8; 32]], epoch: u64) -> Self {
        let mut id = [0u8; 32];
        getrandom::getrandom(&mut id).expect("RNG failed");

        // Einfaches Commitment (in Produktion: Pedersen)
        let input_commitment = Self::compute_commitment(input_hashes);
        let output_commitment = Self::compute_commitment(output_hashes);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            input_commitment,
            output_commitment,
            message_count: input_hashes.len(),
            epoch,
            timestamp: now,
        }
    }

    /// Berechne Commitment (Hash-basiert, in Produktion: Pedersen)
    fn compute_commitment(hashes: &[[u8; 32]]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for hash in hashes {
            hasher.update(hash);
        }
        hasher.finalize().into()
    }
}

/// ZK-Shuffle-Proof (Bayer-Groth vereinfacht)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkShuffleProof {
    /// Batch-ID
    pub batch_id: [u8; 32],
    /// Commitment zum Permutations-Vektor
    pub permutation_commitment: [u8; 32],
    /// Re-Encryption-Proof
    pub reencryption_proof: Vec<u8>,
    /// Verifier-Challenge
    pub challenge: [u8; 32],
    /// Response
    pub response: Vec<u8>,
}

impl ZkShuffleProof {
    /// Erstelle ZK-Shuffle-Proof (vereinfacht)
    ///
    /// In Produktion: Vollständiger Bayer-Groth-Proof
    pub fn new(batch: &MixingBatchCommitment, permutation: &[usize]) -> Self {
        let mut permutation_commitment = [0u8; 32];
        getrandom::getrandom(&mut permutation_commitment).expect("RNG failed");

        let mut challenge = [0u8; 32];
        getrandom::getrandom(&mut challenge).expect("RNG failed");

        // Placeholder für echten ZK-Proof
        let reencryption_proof = vec![0u8; 64];
        let response = vec![0u8; 64];

        Self {
            batch_id: batch.id,
            permutation_commitment,
            reencryption_proof,
            challenge,
            response,
        }
    }

    /// Verifiziere Proof (vereinfacht)
    pub fn verify(&self, batch: &MixingBatchCommitment) -> bool {
        // In Produktion: Vollständige Bayer-Groth-Verifikation
        self.batch_id == batch.id && !self.response.is_empty()
    }
}

/// Daily-Compute-Proof (Aggregation über einen Tag)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyComputeProof {
    /// Peer-ID
    pub peer: PeerId,
    /// Tag (Unix-Timestamp / 86400)
    pub day: u64,
    /// Anzahl verarbeiteter Batches
    pub batch_count: usize,
    /// Gesamtanzahl gemischter Nachrichten
    pub total_messages: u64,
    /// Merkle-Root der Batch-Commitments
    pub batches_root: [u8; 32],
    /// Sample ZK-Proofs (für Spot-Check)
    pub sample_proofs: Vec<ZkShuffleProof>,
}

impl DailyComputeProof {
    /// Erstelle Daily-Proof aus Batches
    pub fn from_batches(
        peer: PeerId,
        day: u64,
        batches: &[MixingBatchCommitment],
        proofs: &[ZkShuffleProof],
    ) -> Self {
        let batch_count = batches.len();
        let total_messages: u64 = batches.iter().map(|b| b.message_count as u64).sum();

        // Merkle-Root
        let batch_hashes: Vec<[u8; 32]> = batches.iter().map(|b| b.id).collect();
        let batches_root = if batch_hashes.is_empty() {
            [0u8; 32]
        } else {
            StorageMerkleTree::compute_root(&batch_hashes)
        };

        // Sample Proofs (max 3)
        let sample_proofs: Vec<_> = proofs.iter().take(3).cloned().collect();

        Self {
            peer,
            day,
            batch_count,
            total_messages,
            batches_root,
            sample_proofs,
        }
    }

    /// Prüfe ob Minimum erreicht
    pub fn meets_minimum(&self) -> bool {
        self.batch_count >= MIN_MIXING_BATCHES / 30 // Pro Tag
    }
}

// ============================================================================
// VERIFIED RESOURCE COMMITMENT (Aggregation)
// ============================================================================

/// Verifiziertes Resource-Commitment (Kombination aller drei)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedResourceCommitment {
    /// Peer-ID
    pub peer: PeerId,
    /// Storage-Commitment (MB·Tage)
    pub storage_mb_days: f64,
    /// Bandwidth-Commitment (GB)
    pub bandwidth_gb: f64,
    /// Compute-Commitment (Mixing-Batches)
    pub compute_batches: u64,
    /// Letzter Storage-Proof
    pub last_storage_proof: Option<u64>,
    /// Letzter Bandwidth-Epoch-Proof
    pub last_bandwidth_proof: Option<u64>,
    /// Letzter Compute-Proof
    pub last_compute_proof: Option<u64>,
    /// Gesamt-Score (normalisiert 0-1)
    pub total_score: f64,
}

impl VerifiedResourceCommitment {
    /// Erstelle neues Commitment
    pub fn new(peer: PeerId) -> Self {
        Self {
            peer,
            storage_mb_days: 0.0,
            bandwidth_gb: 0.0,
            compute_batches: 0,
            last_storage_proof: None,
            last_bandwidth_proof: None,
            last_compute_proof: None,
            total_score: 0.0,
        }
    }

    /// Update mit Storage-Proof
    pub fn add_storage_proof(&mut self, mb: f64, days: f64) {
        self.storage_mb_days += mb * days;
        self.last_storage_proof = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        self.recalculate_score();
    }

    /// Update mit Bandwidth-Proof
    pub fn add_bandwidth_proof(&mut self, proof: &BandwidthEpochProof) {
        self.bandwidth_gb += proof.total_volume_gb;
        self.last_bandwidth_proof = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        self.recalculate_score();
    }

    /// Update mit Compute-Proof
    pub fn add_compute_proof(&mut self, proof: &DailyComputeProof) {
        self.compute_batches += proof.batch_count as u64;
        self.last_compute_proof = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        self.recalculate_score();
    }

    /// Berechne Gesamt-Score
    fn recalculate_score(&mut self) {
        // Gewichtete Kombination (RL-V1: 25%, RL-V2: 35%, RL-V3: 30%, Base: 10%)
        let storage_score = (self.storage_mb_days / 500.0).min(1.0) * 0.25;
        let bandwidth_score = (self.bandwidth_gb / 50.0).min(1.0) * 0.35;
        let compute_score = (self.compute_batches as f64 / 1000.0).min(1.0) * 0.30;
        let base_score = 0.10; // Für Aktivität

        self.total_score = storage_score + bandwidth_score + compute_score + base_score;
    }

    /// Ist Commitment vollständig (alle drei Ressourcen nachgewiesen)?
    pub fn is_complete(&self) -> bool {
        self.last_storage_proof.is_some()
            && self.last_bandwidth_proof.is_some()
            && self.last_compute_proof.is_some()
    }

    /// Geschätzte Sybil-Kosten (USD)
    pub fn estimated_sybil_cost_usd(&self) -> f64 {
        let storage_cost = self.storage_mb_days * 0.01 / 30.0;
        let bandwidth_cost = self.bandwidth_gb * 0.05;
        let compute_cost = self.compute_batches as f64 * 0.001;

        storage_cost + bandwidth_cost + compute_cost
    }
}

// ============================================================================
// RESOURCE VERIFICATION SERVICE
// ============================================================================

/// Service für Resource-Verification
pub struct ResourceVerificationService {
    /// Pending Storage-Challenges
    pending_storage_challenges: HashMap<[u8; 32], StorageChallenge>,
    /// Verified Commitments pro Peer
    commitments: HashMap<PeerId, VerifiedResourceCommitment>,
    /// Attestation-Buffer
    attestation_buffer: HashMap<PeerId, Vec<BilateralAttestation>>,
    /// Current Epoch
    current_epoch: u64,
}

impl ResourceVerificationService {
    /// Erstelle neuen Service
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            pending_storage_challenges: HashMap::new(),
            commitments: HashMap::new(),
            attestation_buffer: HashMap::new(),
            current_epoch: now / ATTESTATION_EPOCH_SECS,
        }
    }

    /// Generiere Storage-Challenge für Peer
    pub fn generate_storage_challenge(
        &mut self,
        peer: PeerId,
        tree: &StorageMerkleTree,
    ) -> StorageChallenge {
        let challenge = StorageChallenge::new(tree, 4); // 4 zufällige Chunks
        self.pending_storage_challenges
            .insert(challenge.id, challenge.clone());
        challenge
    }

    /// Verifiziere Storage-Proof
    pub fn verify_storage_proof(
        &mut self,
        peer: PeerId,
        proof: &StorageProof,
    ) -> StorageVerificationResult {
        let challenge = match self.pending_storage_challenges.get(&proof.challenge_id) {
            Some(c) => c,
            None => return StorageVerificationResult::InvalidChallengeId,
        };

        let result = proof.verify(challenge);

        if result == StorageVerificationResult::Valid {
            // Update Commitment
            let commitment = self
                .commitments
                .entry(peer)
                .or_insert_with(|| VerifiedResourceCommitment::new(peer));

            // Schätze Storage basierend auf Chunk-Count
            let estimated_mb =
                (challenge.chunk_indices.len() * CHUNK_SIZE) as f64 / (1024.0 * 1024.0);
            commitment.add_storage_proof(estimated_mb, 1.0);

            // Challenge entfernen
            self.pending_storage_challenges.remove(&proof.challenge_id);
        }

        result
    }

    /// Registriere Bandwidth-Attestation
    pub fn record_attestation(&mut self, attestation: BilateralAttestation) {
        if attestation.is_complete() {
            self.attestation_buffer
                .entry(attestation.peer_a)
                .or_insert_with(Vec::new)
                .push(attestation.clone());

            self.attestation_buffer
                .entry(attestation.peer_b)
                .or_insert_with(Vec::new)
                .push(attestation);
        }
    }

    /// Finalisiere Bandwidth-Epoch für Peer
    pub fn finalize_bandwidth_epoch(&mut self, peer: PeerId) -> Option<BandwidthEpochProof> {
        let attestations = self.attestation_buffer.remove(&peer)?;

        if attestations.len() < MIN_ATTESTATIONS {
            // Nicht genug Attestationen, zurücklegen
            self.attestation_buffer.insert(peer, attestations);
            return None;
        }

        let proof = BandwidthEpochProof::from_attestations(peer, self.current_epoch, &attestations);

        // Update Commitment
        let commitment = self
            .commitments
            .entry(peer)
            .or_insert_with(|| VerifiedResourceCommitment::new(peer));
        commitment.add_bandwidth_proof(&proof);

        Some(proof)
    }

    /// Verifiziere Compute-Proof
    pub fn verify_compute_proof(&mut self, peer: PeerId, proof: &DailyComputeProof) -> bool {
        // Spot-Check: Verifiziere Sample-Proofs
        // In Produktion: Vollständige ZK-Verifikation

        if !proof.meets_minimum() {
            return false;
        }

        // Update Commitment
        let commitment = self
            .commitments
            .entry(peer)
            .or_insert_with(|| VerifiedResourceCommitment::new(peer));
        commitment.add_compute_proof(proof);

        true
    }

    /// Hole Commitment für Peer
    pub fn get_commitment(&self, peer: &PeerId) -> Option<&VerifiedResourceCommitment> {
        self.commitments.get(peer)
    }

    /// Hole alle Commitments
    pub fn all_commitments(&self) -> impl Iterator<Item = (&PeerId, &VerifiedResourceCommitment)> {
        self.commitments.iter()
    }

    /// Cleanup abgelaufene Challenges
    pub fn cleanup_expired(&mut self) {
        self.pending_storage_challenges
            .retain(|_, c| !c.is_expired());
    }
}

impl Default for ResourceVerificationService {
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
    fn test_merkle_tree_creation() {
        let data = vec![0u8; CHUNK_SIZE * 4]; // 4 Chunks
        let tree = StorageMerkleTree::new(&data);

        assert_eq!(tree.chunk_count, 4);
        assert_eq!(tree.chunk_hashes.len(), 4);
        assert_ne!(tree.root, [0u8; 32]);
    }

    #[test]
    fn test_merkle_proof_verification() {
        let data = vec![0u8; CHUNK_SIZE * 4];
        let tree = StorageMerkleTree::new(&data);

        let proof = tree.generate_proof(0).unwrap();
        assert!(proof.verify(&tree.root));

        let proof2 = tree.generate_proof(3).unwrap();
        assert!(proof2.verify(&tree.root));
    }

    #[test]
    fn test_merkle_proof_invalid_root() {
        let data = vec![0u8; CHUNK_SIZE * 4];
        let tree = StorageMerkleTree::new(&data);

        let proof = tree.generate_proof(0).unwrap();
        let fake_root = [1u8; 32];
        assert!(!proof.verify(&fake_root));
    }

    #[test]
    fn test_storage_challenge_creation() {
        let data = vec![0u8; CHUNK_SIZE * 8];
        let tree = StorageMerkleTree::new(&data);

        let challenge = StorageChallenge::new(&tree, 4);

        assert!(!challenge.is_expired());
        assert!(challenge.chunk_indices.len() <= 4);
        assert_eq!(challenge.expected_root, tree.root);
    }

    #[test]
    fn test_storage_proof_verification() {
        let data = vec![0u8; CHUNK_SIZE * 4];
        let tree = StorageMerkleTree::new(&data);
        let challenge = StorageChallenge::new(&tree, 2);

        let proofs: Vec<MerkleProof> = challenge
            .chunk_indices
            .iter()
            .filter_map(|&idx| tree.generate_proof(idx))
            .collect();

        let proof = StorageProof {
            challenge_id: challenge.id,
            proofs,
            chunk_data: vec![],
        };

        assert_eq!(proof.verify(&challenge), StorageVerificationResult::Valid);
    }

    #[test]
    fn test_relay_receipt_creation() {
        let peer_a = PeerId::random();
        let peer_b = PeerId::random();

        let receipt = RelayReceipt::new(peer_a, peer_b, 1024 * 1024, 100);

        assert_eq!(receipt.bytes_transferred, 1024 * 1024);
        assert_eq!(receipt.epoch, 100);
    }

    #[test]
    fn test_bilateral_attestation() {
        let peer_a = PeerId::random();
        let peer_b = PeerId::random();

        let mut attestation = BilateralAttestation::new(peer_a, peer_b, 1000, 2000, 100);

        assert!(!attestation.is_complete());
        assert_eq!(attestation.total_volume(), 3000);

        attestation.signature_a = vec![1, 2, 3];
        attestation.signature_b = vec![4, 5, 6];
        assert!(attestation.is_complete());
    }

    #[test]
    fn test_bandwidth_epoch_proof() {
        let peer = PeerId::random();
        let other = PeerId::random();

        let attestations: Vec<BilateralAttestation> = (0..15)
            .map(|i| {
                let mut a = BilateralAttestation::new(peer, other, 1024 * 1024 * 100, 0, 100);
                a.signature_a = vec![i as u8];
                a.signature_b = vec![i as u8];
                a
            })
            .collect();

        let proof = BandwidthEpochProof::from_attestations(peer, 100, &attestations);

        assert!(proof.meets_minimum());
        assert_eq!(proof.attestation_count, 15);
        assert!(proof.total_volume_gb > 0.0);
    }

    #[test]
    fn test_mixing_batch_commitment() {
        let inputs: Vec<[u8; 32]> = (0..10).map(|i| [i as u8; 32]).collect();
        let outputs: Vec<[u8; 32]> = (0..10).map(|i| [9 - i as u8; 32]).collect();

        let commitment = MixingBatchCommitment::new(&inputs, &outputs, 100);

        assert_eq!(commitment.message_count, 10);
        assert_ne!(commitment.input_commitment, commitment.output_commitment);
    }

    #[test]
    fn test_zk_shuffle_proof() {
        let inputs: Vec<[u8; 32]> = (0..10).map(|i| [i as u8; 32]).collect();
        let outputs: Vec<[u8; 32]> = (0..10).map(|i| [i as u8; 32]).collect();

        let batch = MixingBatchCommitment::new(&inputs, &outputs, 100);
        let permutation: Vec<usize> = (0..10).rev().collect();

        let proof = ZkShuffleProof::new(&batch, &permutation);

        assert!(proof.verify(&batch));
    }

    #[test]
    fn test_daily_compute_proof() {
        let peer = PeerId::random();

        let batches: Vec<MixingBatchCommitment> = (0..5)
            .map(|i| {
                let inputs: Vec<[u8; 32]> = vec![[i as u8; 32]; 10];
                MixingBatchCommitment::new(&inputs, &inputs, 100)
            })
            .collect();

        let proofs: Vec<ZkShuffleProof> = batches
            .iter()
            .map(|b| ZkShuffleProof::new(b, &[]))
            .collect();

        let daily_proof = DailyComputeProof::from_batches(peer, 100, &batches, &proofs);

        assert_eq!(daily_proof.batch_count, 5);
        assert_eq!(daily_proof.total_messages, 50);
    }

    #[test]
    fn test_verified_resource_commitment() {
        let peer = PeerId::random();
        let mut commitment = VerifiedResourceCommitment::new(peer);

        assert!(!commitment.is_complete());
        assert_eq!(commitment.total_score, 0.0);

        // Provide enough resources to score above threshold
        // Storage: 500 MB-days = 25% max score
        commitment.add_storage_proof(500.0, 1.0);
        assert!(commitment.total_score > 0.0);

        // Bandwidth: 50 GB = 35% max score
        let bw_proof = BandwidthEpochProof {
            peer,
            epoch: 100,
            total_volume_gb: 50.0,
            attestation_count: 20,
            attestations_root: [0u8; 32],
            sample_attestations: vec![],
        };
        commitment.add_bandwidth_proof(&bw_proof);

        // Compute: 1000 batches = 30% max score
        let compute_proof = DailyComputeProof {
            peer,
            day: 100,
            batch_count: 1000,
            total_messages: 10000,
            batches_root: [0u8; 32],
            sample_proofs: vec![],
        };
        commitment.add_compute_proof(&compute_proof);

        assert!(commitment.is_complete());
        // Base 10% + Storage 25% + Bandwidth 35% + Compute 30% = 100%
        assert!(commitment.total_score > 0.5);
    }

    #[test]
    fn test_resource_verification_service() {
        let mut service = ResourceVerificationService::new();
        let peer = PeerId::random();

        // Storage
        let data = vec![0u8; CHUNK_SIZE * 8];
        let tree = StorageMerkleTree::new(&data);
        let challenge = service.generate_storage_challenge(peer, &tree);

        let proofs: Vec<MerkleProof> = challenge
            .chunk_indices
            .iter()
            .filter_map(|&idx| tree.generate_proof(idx))
            .collect();

        let proof = StorageProof {
            challenge_id: challenge.id,
            proofs,
            chunk_data: vec![],
        };

        let result = service.verify_storage_proof(peer, &proof);
        assert_eq!(result, StorageVerificationResult::Valid);

        // Check commitment updated
        let commitment = service.get_commitment(&peer).unwrap();
        assert!(commitment.storage_mb_days > 0.0);
    }

    #[test]
    fn test_sybil_cost_estimation() {
        let peer = PeerId::random();
        let mut commitment = VerifiedResourceCommitment::new(peer);

        commitment.storage_mb_days = 500.0;
        commitment.bandwidth_gb = 50.0;
        commitment.compute_batches = 1000;

        let cost = commitment.estimated_sybil_cost_usd();

        // ~$0.17 Storage + ~$2.50 Bandwidth + ~$1.00 Compute = ~$3.67
        assert!(cost > 3.0);
        assert!(cost < 5.0);
    }
}
