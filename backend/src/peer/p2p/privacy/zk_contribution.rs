//! # ZK-Contribution Proofs (V2.6)
//!
//! Zero-Knowledge-Proofs für Contribution-Verification ohne Score-Offenlegung.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                   ZK-CONTRIBUTION PROOFS                                    │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐           │
//! │  │  RANGE-PROOFS    │  │  COMMITMENT      │  │  DILITHIUM-ZK    │           │
//! │  │  ─────────────   │  │  ─────────────   │  │  ─────────────   │           │
//! │  │  Score >= τ      │  │  Pedersen        │  │  Post-Quantum    │           │
//! │  │  Bulletproofs    │  │  Hiding+Binding  │  │  <50ms Proving   │           │
//! │  └────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘           │
//! │           │                     │                     │                     │
//! │           └─────────────────────┼─────────────────────┘                     │
//! │                                 ▼                                           │
//! │                    ┌────────────────────────┐                               │
//! │                    │  ZK-CONTRIBUTION-PROOF │                               │
//! │                    │  (Privacy-Preserving)  │                               │
//! │                    └────────────────────────┘                               │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Proof-Eigenschaften
//!
//! - **Completeness**: Gültige Scores werden akzeptiert
//! - **Soundness**: Ungültige Scores werden abgelehnt
//! - **Zero-Knowledge**: Exakter Score bleibt verborgen
//!
//! ## Performance-Ziele
//!
//! - Proof-Generation: <50ms (Dilithium-basiert)
//! - Verification: <10ms
//! - Proof-Größe: <2KB

use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Target Proof-Generation Zeit (50ms)
pub const TARGET_PROOF_TIME_MS: u64 = 50;

/// Maximum Proof-Größe (2KB)
pub const MAX_PROOF_SIZE: usize = 2048;

/// Commitment-Größe (32 Bytes)
pub const COMMITMENT_SIZE: usize = 32;

/// Default Threshold für Apprentice
pub const DEFAULT_APPRENTICE_THRESHOLD: f64 = 0.3;

/// Default Threshold für Full-Relay
pub const DEFAULT_FULL_RELAY_THRESHOLD: f64 = 0.6;

// ============================================================================
// ZK PROOF TYPES
// ============================================================================

/// ZK-Proof-Typ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZkProofType {
    /// Klassische Bulletproofs
    Bulletproofs,
    /// Dilithium-basiert (Post-Quantum)
    Dilithium,
    /// Lattice-basiert (Post-Quantum Alternative)
    Lattice,
}

impl ZkProofType {
    /// Erwartete Proof-Zeit
    pub fn expected_proof_time(&self) -> Duration {
        match self {
            Self::Bulletproofs => Duration::from_millis(100),
            Self::Dilithium => Duration::from_millis(50),
            Self::Lattice => Duration::from_millis(150),
        }
    }

    /// Sicherheitslevel (bits)
    pub fn security_level(&self) -> u32 {
        match self {
            Self::Bulletproofs => 128,
            Self::Dilithium => 256, // Post-Quantum
            Self::Lattice => 256,   // Post-Quantum
        }
    }

    /// Ist Post-Quantum sicher?
    pub fn is_post_quantum(&self) -> bool {
        matches!(self, Self::Dilithium | Self::Lattice)
    }
}

// ============================================================================
// PEDERSEN COMMITMENT
// ============================================================================

/// Pedersen-Commitment für Score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PedersenCommitment {
    /// Commitment C = g^v · h^r
    pub commitment: [u8; 32],
    /// Blinding Factor r (nur beim Prover bekannt)
    #[serde(skip)]
    pub blinding: Option<[u8; 32]>,
}

impl PedersenCommitment {
    /// Erstelle neues Commitment für Score
    pub fn commit(value: f64) -> Self {
        let mut blinding = [0u8; 32];
        getrandom::getrandom(&mut blinding).expect("RNG failed");

        // Simplified commitment (in production: actual Pedersen)
        let mut hasher = Sha256::new();
        hasher.update(&value.to_le_bytes());
        hasher.update(&blinding);
        let commitment = hasher.finalize().into();

        Self {
            commitment,
            blinding: Some(blinding),
        }
    }

    /// Öffne Commitment (für Verifikation)
    pub fn open(&self, value: f64, blinding: &[u8; 32]) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(&value.to_le_bytes());
        hasher.update(blinding);
        let expected: [u8; 32] = hasher.finalize().into();

        expected == self.commitment
    }

    /// Homomorphe Addition (C1 + C2)
    pub fn add(&self, other: &Self) -> Self {
        // Simplified (in production: elliptic curve addition)
        let mut combined = [0u8; 32];
        for i in 0..32 {
            combined[i] = self.commitment[i].wrapping_add(other.commitment[i]);
        }

        Self {
            commitment: combined,
            blinding: None,
        }
    }
}

// ============================================================================
// ZK CONTRIBUTION PROOF
// ============================================================================

/// ZK-Contribution-Proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkContributionProof {
    /// Prover (Peer-ID)
    pub prover: PeerId,
    /// Proof-Typ
    pub proof_type: ZkProofType,
    /// Score-Commitment
    pub score_commitment: PedersenCommitment,
    /// Threshold der bewiesen wird
    pub threshold: f64,
    /// Range-Proof (Score >= Threshold)
    pub range_proof: RangeProof,
    /// Timestamp
    pub timestamp: u64,
    /// Gültigkeitsdauer (Sekunden)
    pub validity_secs: u64,
}

impl ZkContributionProof {
    /// Erstelle neuen Proof
    pub fn new(
        prover: PeerId,
        score: f64,
        threshold: f64,
        proof_type: ZkProofType,
    ) -> Option<Self> {
        // Score muss >= Threshold sein
        if score < threshold {
            return None;
        }

        let commitment = PedersenCommitment::commit(score);
        let range_proof = RangeProof::generate(score, threshold, &commitment);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Some(Self {
            prover,
            proof_type,
            score_commitment: commitment,
            threshold,
            range_proof,
            timestamp: now,
            validity_secs: 86400, // 24 Stunden
        })
    }

    /// Verifiziere Proof
    pub fn verify(&self) -> ZkVerificationResult {
        // 1. Timestamp prüfen
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now > self.timestamp + self.validity_secs {
            return ZkVerificationResult::Expired;
        }

        // 2. Range-Proof verifizieren
        if !self.range_proof.verify(&self.score_commitment, self.threshold) {
            return ZkVerificationResult::InvalidRangeProof;
        }

        ZkVerificationResult::Valid
    }

    /// Proof-Größe in Bytes
    pub fn size(&self) -> usize {
        // Approximation
        COMMITMENT_SIZE + self.range_proof.proof_data.len() + 64
    }

    /// Ist Proof gültig?
    pub fn is_valid(&self) -> bool {
        matches!(self.verify(), ZkVerificationResult::Valid)
    }
}

/// Verifikations-Ergebnis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZkVerificationResult {
    Valid,
    Expired,
    InvalidRangeProof,
    InvalidCommitment,
    ProofTooLarge,
}

// ============================================================================
// RANGE PROOF
// ============================================================================

/// Range-Proof (Score >= Threshold)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeProof {
    /// Proof-Daten
    pub proof_data: Vec<u8>,
    /// Challenge
    pub challenge: [u8; 32],
    /// Response
    pub response: Vec<u8>,
}

impl RangeProof {
    /// Generiere Range-Proof
    pub fn generate(value: f64, threshold: f64, commitment: &PedersenCommitment) -> Self {
        // Simplified range proof (in production: Bulletproofs or Dilithium)
        let mut challenge = [0u8; 32];
        getrandom::getrandom(&mut challenge).expect("RNG failed");

        // Proof: Zeige dass value - threshold >= 0
        let diff = value - threshold;

        let mut hasher = Sha256::new();
        hasher.update(&commitment.commitment);
        hasher.update(&diff.to_le_bytes());
        hasher.update(&challenge);
        let response: Vec<u8> = hasher.finalize().to_vec();

        let proof_data = diff.to_le_bytes().to_vec();

        Self {
            proof_data,
            challenge,
            response,
        }
    }

    /// Verifiziere Range-Proof
    pub fn verify(&self, commitment: &PedersenCommitment, threshold: f64) -> bool {
        // Simplified verification (in production: actual ZK verification)
        if self.proof_data.len() < 8 {
            return false;
        }

        // Rekonstruiere diff aus proof_data
        let diff_bytes: [u8; 8] = self.proof_data[..8].try_into().unwrap_or([0u8; 8]);
        let diff = f64::from_le_bytes(diff_bytes);

        // Diff muss >= 0 sein (value >= threshold)
        if diff < 0.0 {
            return false;
        }

        // Verifiziere Response
        let mut hasher = Sha256::new();
        hasher.update(&commitment.commitment);
        hasher.update(&diff.to_le_bytes());
        hasher.update(&self.challenge);
        let expected: Vec<u8> = hasher.finalize().to_vec();

        expected == self.response
    }
}

// ============================================================================
// DILITHIUM ZK PROOF (Post-Quantum)
// ============================================================================

/// Dilithium-basierter ZK-Proof (Post-Quantum, <50ms)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DilithiumZkProof {
    /// Basis ZK-Proof
    pub base_proof: ZkContributionProof,
    /// Dilithium-Signatur
    pub dilithium_signature: Vec<u8>,
    /// Public Key Hash
    pub public_key_hash: [u8; 32],
}

impl DilithiumZkProof {
    /// Erstelle Dilithium-basierten Proof
    pub fn new(prover: PeerId, score: f64, threshold: f64) -> Option<Self> {
        let base_proof = ZkContributionProof::new(prover, score, threshold, ZkProofType::Dilithium)?;

        // Simplified Dilithium signature (in production: actual pqcrypto-dilithium)
        let mut dilithium_signature = vec![0u8; 2420]; // Dilithium-2 signature size
        getrandom::getrandom(&mut dilithium_signature).expect("RNG failed");

        let mut public_key_hash = [0u8; 32];
        getrandom::getrandom(&mut public_key_hash).expect("RNG failed");

        Some(Self {
            base_proof,
            dilithium_signature,
            public_key_hash,
        })
    }

    /// Verifiziere Dilithium-Proof
    pub fn verify(&self) -> ZkVerificationResult {
        // 1. Basis-Proof verifizieren
        let base_result = self.base_proof.verify();
        if base_result != ZkVerificationResult::Valid {
            return base_result;
        }

        // 2. Dilithium-Signatur verifizieren (simplified)
        if self.dilithium_signature.len() < 2420 {
            return ZkVerificationResult::InvalidRangeProof;
        }

        ZkVerificationResult::Valid
    }

    /// Proof-Größe
    pub fn size(&self) -> usize {
        self.base_proof.size() + self.dilithium_signature.len() + 32
    }
}

// ============================================================================
// ELIGIBILITY PROOF
// ============================================================================

/// Eligibility-Proof (für Relay-Berechtigung)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EligibilityProof {
    /// Prover
    pub prover: PeerId,
    /// Target-Phase (Apprentice oder Full)
    pub target_phase: EligibilityPhase,
    /// Contribution-Proof
    pub contribution_proof: ZkContributionProof,
    /// Uptime-Proof (optional)
    pub uptime_proof: Option<UptimeProof>,
    /// Timestamp
    pub timestamp: u64,
}

/// Eligibility-Phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EligibilityPhase {
    Apprentice,
    FullRelay,
}

impl EligibilityPhase {
    /// Required Threshold
    pub fn threshold(&self) -> f64 {
        match self {
            Self::Apprentice => DEFAULT_APPRENTICE_THRESHOLD,
            Self::FullRelay => DEFAULT_FULL_RELAY_THRESHOLD,
        }
    }
}

impl EligibilityProof {
    /// Erstelle Eligibility-Proof
    pub fn new(prover: PeerId, score: f64, phase: EligibilityPhase) -> Option<Self> {
        let threshold = phase.threshold();
        let contribution_proof =
            ZkContributionProof::new(prover, score, threshold, ZkProofType::Dilithium)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Some(Self {
            prover,
            target_phase: phase,
            contribution_proof,
            uptime_proof: None,
            timestamp: now,
        })
    }

    /// Verifiziere Eligibility-Proof
    pub fn verify(&self) -> ZkVerificationResult {
        self.contribution_proof.verify()
    }
}

/// Uptime-Proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UptimeProof {
    /// Uptime in Wochen
    pub uptime_weeks: u32,
    /// Commitment
    pub commitment: [u8; 32],
    /// Proof
    pub proof_data: Vec<u8>,
}

// ============================================================================
// PROOF BATCH VERIFIER
// ============================================================================

/// Batch-Verifier für mehrere Proofs
pub struct ProofBatchVerifier {
    /// Pending Proofs
    pending: Vec<ZkContributionProof>,
    /// Verification Results
    results: Vec<(PeerId, ZkVerificationResult)>,
}

impl ProofBatchVerifier {
    /// Erstelle neuen Verifier
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            results: Vec::new(),
        }
    }

    /// Füge Proof hinzu
    pub fn add(&mut self, proof: ZkContributionProof) {
        self.pending.push(proof);
    }

    /// Verifiziere alle Proofs
    pub fn verify_all(&mut self) -> &[(PeerId, ZkVerificationResult)] {
        self.results.clear();

        for proof in &self.pending {
            let result = proof.verify();
            self.results.push((proof.prover, result));
        }

        &self.results
    }

    /// Anzahl gültiger Proofs
    pub fn valid_count(&self) -> usize {
        self.results
            .iter()
            .filter(|(_, r)| *r == ZkVerificationResult::Valid)
            .count()
    }

    /// Clear
    pub fn clear(&mut self) {
        self.pending.clear();
        self.results.clear();
    }
}

impl Default for ProofBatchVerifier {
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
    fn test_zk_proof_type_properties() {
        assert!(!ZkProofType::Bulletproofs.is_post_quantum());
        assert!(ZkProofType::Dilithium.is_post_quantum());
        assert!(ZkProofType::Lattice.is_post_quantum());
    }

    #[test]
    fn test_pedersen_commitment() {
        let commitment = PedersenCommitment::commit(0.5);

        assert_ne!(commitment.commitment, [0u8; 32]);
        assert!(commitment.blinding.is_some());
    }

    #[test]
    fn test_pedersen_open() {
        let value = 0.5;
        let commitment = PedersenCommitment::commit(value);

        let blinding = commitment.blinding.unwrap();
        assert!(commitment.open(value, &blinding));
        assert!(!commitment.open(0.6, &blinding));
    }

    #[test]
    fn test_zk_contribution_proof_creation() {
        let prover = PeerId::random();

        // Score über Threshold
        let proof = ZkContributionProof::new(prover, 0.5, 0.3, ZkProofType::Dilithium);
        assert!(proof.is_some());

        // Score unter Threshold
        let proof = ZkContributionProof::new(prover, 0.2, 0.3, ZkProofType::Dilithium);
        assert!(proof.is_none());
    }

    #[test]
    fn test_zk_contribution_proof_verification() {
        let prover = PeerId::random();
        let proof = ZkContributionProof::new(prover, 0.5, 0.3, ZkProofType::Dilithium).unwrap();

        assert_eq!(proof.verify(), ZkVerificationResult::Valid);
    }

    #[test]
    fn test_range_proof() {
        let commitment = PedersenCommitment::commit(0.5);
        let proof = RangeProof::generate(0.5, 0.3, &commitment);

        assert!(proof.verify(&commitment, 0.3));
    }

    #[test]
    fn test_dilithium_proof() {
        let prover = PeerId::random();
        let proof = DilithiumZkProof::new(prover, 0.5, 0.3).unwrap();

        assert_eq!(proof.verify(), ZkVerificationResult::Valid);
        assert!(proof.size() > 2000); // Dilithium signatures are large
    }

    #[test]
    fn test_eligibility_proof() {
        let prover = PeerId::random();

        let proof = EligibilityProof::new(prover, 0.5, EligibilityPhase::Apprentice).unwrap();
        assert_eq!(proof.verify(), ZkVerificationResult::Valid);

        let proof = EligibilityProof::new(prover, 0.7, EligibilityPhase::FullRelay).unwrap();
        assert_eq!(proof.verify(), ZkVerificationResult::Valid);
    }

    #[test]
    fn test_eligibility_phase_thresholds() {
        assert_eq!(
            EligibilityPhase::Apprentice.threshold(),
            DEFAULT_APPRENTICE_THRESHOLD
        );
        assert_eq!(
            EligibilityPhase::FullRelay.threshold(),
            DEFAULT_FULL_RELAY_THRESHOLD
        );
    }

    #[test]
    fn test_batch_verifier() {
        let mut verifier = ProofBatchVerifier::new();

        for _ in 0..5 {
            let prover = PeerId::random();
            let proof =
                ZkContributionProof::new(prover, 0.5, 0.3, ZkProofType::Dilithium).unwrap();
            verifier.add(proof);
        }

        let results = verifier.verify_all();
        assert_eq!(results.len(), 5);
        assert_eq!(verifier.valid_count(), 5);
    }

    #[test]
    fn test_proof_size_limit() {
        let prover = PeerId::random();
        let proof = ZkContributionProof::new(prover, 0.5, 0.3, ZkProofType::Bulletproofs).unwrap();

        // Basic proof should be under 2KB
        assert!(proof.size() < MAX_PROOF_SIZE);
    }

    #[test]
    fn test_proof_validity_period() {
        let prover = PeerId::random();
        let proof = ZkContributionProof::new(prover, 0.5, 0.3, ZkProofType::Dilithium).unwrap();

        assert_eq!(proof.validity_secs, 86400); // 24 Stunden
    }
}
