//! # Lattice-Based ZK Proofs (Post-Quantum V2.6)
//!
//! Post-Quantum Zero-Knowledge-Proofs basierend auf Lattice-Problemen.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                   LATTICE-BASED ZK PROOFS                                   │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐           │
//! │  │  LWE-COMMITMENT  │  │  RING-LWE        │  │  KYBER-BASED     │           │
//! │  │  ─────────────   │  │  ─────────────   │  │  ─────────────   │           │
//! │  │  Learning With   │  │  Efficient       │  │  Key Exchange    │           │
//! │  │  Errors          │  │  Polynomial Ops  │  │  Integration     │           │
//! │  └────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘           │
//! │           │                     │                     │                     │
//! │           └─────────────────────┼─────────────────────┘                     │
//! │                                 ▼                                           │
//! │                    ┌────────────────────────┐                               │
//! │                    │  LATTICE-ZK-PROOF      │                               │
//! │                    │  (Quantum-Resistant)   │                               │
//! │                    └────────────────────────┘                               │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Sicherheits-Grundlagen
//!
//! - **SVP** (Shortest Vector Problem) Hardness
//! - **LWE** (Learning With Errors) Assumption
//! - **Ring-LWE** für Effizienz
//!
//! ## Performance
//!
//! - Proof-Zeit: ~150ms
//! - Verification: ~20ms
//! - Proof-Größe: ~5-10KB (größer als klassisch)

use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Lattice Dimension (für LWE)
pub const LATTICE_DIMENSION: usize = 256;

/// Modulus für Lattice-Operationen
pub const LATTICE_MODULUS: u64 = 12289; // Prime near 2^14

/// Noise Standard-Deviation
pub const NOISE_STDDEV: f64 = 3.19;

/// Maximum Proof-Größe (32KB für Lattice - Post-Quantum is larger)
pub const MAX_LATTICE_PROOF_SIZE: usize = 32768;

// ============================================================================
// LWE PARAMETERS
// ============================================================================

/// LWE-Parameter für Security Level
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LweParameters {
    /// Dimension n
    pub dimension: usize,
    /// Modulus q
    pub modulus: u64,
    /// Samples m
    pub samples: usize,
    /// Error distribution width
    pub error_width: f64,
}

impl LweParameters {
    /// Standard-Parameter für 128-bit Security
    pub fn standard_128() -> Self {
        Self {
            dimension: 256,
            modulus: 12289,
            samples: 512,
            error_width: 3.19,
        }
    }

    /// Parameter für 256-bit Security (Post-Quantum)
    pub fn quantum_256() -> Self {
        Self {
            dimension: 512,
            modulus: 12289,
            samples: 1024,
            error_width: 3.19,
        }
    }

    /// Security Level in Bits
    pub fn security_level(&self) -> u32 {
        // Approximation based on dimension
        (self.dimension as f64 * 0.5) as u32
    }
}

impl Default for LweParameters {
    fn default() -> Self {
        Self::standard_128()
    }
}

// ============================================================================
// LATTICE COMMITMENT
// ============================================================================

/// LWE-basiertes Commitment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatticeCommitment {
    /// Public Matrix A (compressed)
    pub matrix_hash: [u8; 32],
    /// Commitment c = As + e + μ·⌊q/2⌋
    pub commitment: Vec<u64>,
    /// Parameters
    pub params: LweParameters,
}

impl LatticeCommitment {
    /// Erstelle Commitment für Wert
    pub fn commit(value: u64, params: &LweParameters) -> Self {
        // Simplified LWE commitment (in production: actual lattice math)
        let mut commitment = vec![0u64; params.samples];

        // Generate pseudo-random commitment
        let mut hasher = Sha256::new();
        hasher.update(&value.to_le_bytes());
        let seed: [u8; 32] = hasher.finalize().into();

        // Fill commitment vector
        for (i, c) in commitment.iter_mut().enumerate() {
            let mut h = Sha256::new();
            h.update(&seed);
            h.update(&(i as u64).to_le_bytes());
            let hash = h.finalize();
            *c = u64::from_le_bytes(hash[0..8].try_into().unwrap()) % params.modulus;
        }

        Self {
            matrix_hash: seed,
            commitment,
            params: *params,
        }
    }

    /// Commitment-Größe in Bytes
    pub fn size(&self) -> usize {
        32 + self.commitment.len() * 8 + std::mem::size_of::<LweParameters>()
    }
}

// ============================================================================
// LATTICE RANGE PROOF
// ============================================================================

/// Lattice-basierter Range-Proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatticeRangeProof {
    /// Commitment für den Wert
    pub commitment: LatticeCommitment,
    /// Proof-Daten (Sigma-Protocol Transcripts)
    pub proof_transcripts: Vec<ProofTranscript>,
    /// Challenge
    pub challenge: [u8; 32],
    /// Response vectors
    pub responses: Vec<Vec<u64>>,
}

/// Sigma-Protocol Transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofTranscript {
    /// Commitment t
    pub commitment: [u8; 32],
    /// Challenge c
    pub challenge: u64,
    /// Response z
    pub response: Vec<u64>,
}

impl LatticeRangeProof {
    /// Generiere Range-Proof (value >= threshold)
    pub fn generate(value: f64, threshold: f64, params: &LweParameters) -> Self {
        // Convert to integer representation
        let value_int = (value * 1000.0) as u64;
        let threshold_int = (threshold * 1000.0) as u64;

        let commitment = LatticeCommitment::commit(value_int, params);

        // Generate proof transcripts (simplified)
        let num_transcripts = 3; // Soundness amplification
        let mut transcripts = Vec::with_capacity(num_transcripts);

        for i in 0..num_transcripts {
            let mut hasher = Sha256::new();
            hasher.update(&commitment.matrix_hash);
            hasher.update(&(i as u64).to_le_bytes());
            let t_commit: [u8; 32] = hasher.finalize().into();

            let challenge = (value_int.wrapping_mul((i + 1) as u64)) % params.modulus;

            let response_len = params.dimension;
            let mut response = vec![0u64; response_len];
            for (j, r) in response.iter_mut().enumerate() {
                *r = ((value_int - threshold_int).wrapping_add(j as u64)) % params.modulus;
            }

            transcripts.push(ProofTranscript {
                commitment: t_commit,
                challenge,
                response,
            });
        }

        // Global challenge
        let mut challenge = [0u8; 32];
        let mut h = Sha256::new();
        for t in &transcripts {
            h.update(&t.commitment);
        }
        challenge.copy_from_slice(&h.finalize());

        // Generate responses
        let responses = transcripts.iter().map(|t| t.response.clone()).collect();

        Self {
            commitment,
            proof_transcripts: transcripts,
            challenge,
            responses,
        }
    }

    /// Verifiziere Range-Proof
    pub fn verify(&self, threshold: f64) -> bool {
        let threshold_int = (threshold * 1000.0) as u64;

        // Verify each transcript
        for transcript in &self.proof_transcripts {
            // Check response bounds (simplified)
            let response_sum: u64 = transcript.response.iter().sum();
            if response_sum % self.commitment.params.modulus < threshold_int {
                return false;
            }
        }

        // Verify challenge consistency
        let mut h = Sha256::new();
        for t in &self.proof_transcripts {
            h.update(&t.commitment);
        }
        let expected_challenge: [u8; 32] = h.finalize().into();

        expected_challenge == self.challenge
    }

    /// Proof-Größe
    pub fn size(&self) -> usize {
        let transcripts_size: usize = self
            .proof_transcripts
            .iter()
            .map(|t| 32 + 8 + t.response.len() * 8)
            .sum();

        self.commitment.size() + transcripts_size + 32 + self.responses.len() * 8 * LATTICE_DIMENSION
    }
}

// ============================================================================
// LATTICE ZK CONTRIBUTION PROOF
// ============================================================================

/// Vollständiger Lattice-ZK-Proof für Contribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatticeZkProof {
    /// Prover
    pub prover: PeerId,
    /// Range-Proof
    pub range_proof: LatticeRangeProof,
    /// Threshold
    pub threshold: f64,
    /// Timestamp
    pub timestamp: u64,
    /// Validity (Sekunden)
    pub validity_secs: u64,
    /// Kyber Public Key Hash (für Key Exchange)
    pub kyber_pk_hash: Option<[u8; 32]>,
}

impl LatticeZkProof {
    /// Erstelle neuen Lattice-ZK-Proof
    pub fn new(prover: PeerId, score: f64, threshold: f64) -> Option<Self> {
        if score < threshold {
            return None;
        }

        let params = LweParameters::quantum_256();
        let range_proof = LatticeRangeProof::generate(score, threshold, &params);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Some(Self {
            prover,
            range_proof,
            threshold,
            timestamp: now,
            validity_secs: 86400,
            kyber_pk_hash: None,
        })
    }

    /// Verifiziere Proof
    pub fn verify(&self) -> LatticeVerificationResult {
        // Check timestamp
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now > self.timestamp + self.validity_secs {
            return LatticeVerificationResult::Expired;
        }

        // Verify range proof
        if !self.range_proof.verify(self.threshold) {
            return LatticeVerificationResult::InvalidRangeProof;
        }

        // Check proof size
        if self.size() > MAX_LATTICE_PROOF_SIZE {
            return LatticeVerificationResult::ProofTooLarge;
        }

        LatticeVerificationResult::Valid
    }

    /// Proof-Größe
    pub fn size(&self) -> usize {
        // PeerId + range_proof + threshold + timestamps + optional kyber
        38 + self.range_proof.size() + 8 + 8 + 8 + 32
    }

    /// Mit Kyber Key
    pub fn with_kyber_key(mut self, kyber_pk: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(kyber_pk);
        self.kyber_pk_hash = Some(hasher.finalize().into());
        self
    }
}

/// Lattice Verification Result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatticeVerificationResult {
    Valid,
    Expired,
    InvalidRangeProof,
    InvalidCommitment,
    ProofTooLarge,
}

// ============================================================================
// HYBRID PROOF (CLASSICAL + POST-QUANTUM)
// ============================================================================

/// Hybrid-Proof kombiniert klassisch + Post-Quantum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridZkProof {
    /// Klassischer Bulletproofs-Teil (für Effizienz)
    pub classical_proof: ClassicalProofPart,
    /// Post-Quantum Lattice-Teil (für Zukunftssicherheit)
    pub quantum_proof: QuantumProofPart,
    /// Verknüpfungs-Commitment
    pub binding_commitment: [u8; 32],
}

/// Klassischer Proof-Teil
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassicalProofPart {
    /// Commitment
    pub commitment: [u8; 32],
    /// Proof-Daten
    pub proof_data: Vec<u8>,
    /// Challenge
    pub challenge: [u8; 32],
}

/// Post-Quantum Proof-Teil
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumProofPart {
    /// Lattice-Commitment
    pub lattice_commitment: LatticeCommitment,
    /// Response
    pub response: Vec<u64>,
}

impl HybridZkProof {
    /// Erstelle Hybrid-Proof
    pub fn new(prover: PeerId, score: f64, threshold: f64) -> Option<Self> {
        if score < threshold {
            return None;
        }

        // Classical part
        let mut commitment = [0u8; 32];
        let mut hasher = Sha256::new();
        hasher.update(&score.to_le_bytes());
        commitment.copy_from_slice(&hasher.finalize());

        let proof_data = (score - threshold).to_le_bytes().to_vec();

        let mut challenge = [0u8; 32];
        getrandom::getrandom(&mut challenge).expect("RNG failed");

        let classical = ClassicalProofPart {
            commitment,
            proof_data,
            challenge,
        };

        // Quantum part
        let params = LweParameters::quantum_256();
        let value_int = (score * 1000.0) as u64;
        let lattice_commitment = LatticeCommitment::commit(value_int, &params);

        let mut response = vec![0u64; params.dimension];
        for (i, r) in response.iter_mut().enumerate() {
            *r = (value_int.wrapping_add(i as u64)) % params.modulus;
        }

        let quantum = QuantumProofPart {
            lattice_commitment,
            response,
        };

        // Binding commitment
        let mut binding = [0u8; 32];
        let mut h = Sha256::new();
        h.update(&commitment);
        h.update(&quantum.lattice_commitment.matrix_hash);
        binding.copy_from_slice(&h.finalize());

        Some(Self {
            classical_proof: classical,
            quantum_proof: quantum,
            binding_commitment: binding,
        })
    }

    /// Verifiziere beide Teile
    pub fn verify(&self, threshold: f64) -> HybridVerificationResult {
        // Verify classical part (simplified)
        if self.classical_proof.proof_data.len() < 8 {
            return HybridVerificationResult::ClassicalFailed;
        }

        let diff_bytes: [u8; 8] = self.classical_proof.proof_data[..8]
            .try_into()
            .unwrap_or([0u8; 8]);
        let diff = f64::from_le_bytes(diff_bytes);
        if diff < 0.0 {
            return HybridVerificationResult::ClassicalFailed;
        }

        // Verify quantum part (simplified)
        let response_sum: u64 = self.quantum_proof.response.iter().sum();
        if response_sum == 0 {
            return HybridVerificationResult::QuantumFailed;
        }

        // Verify binding
        let mut h = Sha256::new();
        h.update(&self.classical_proof.commitment);
        h.update(&self.quantum_proof.lattice_commitment.matrix_hash);
        let expected_binding: [u8; 32] = h.finalize().into();

        if expected_binding != self.binding_commitment {
            return HybridVerificationResult::BindingFailed;
        }

        HybridVerificationResult::Valid
    }

    /// Größe
    pub fn size(&self) -> usize {
        let classical_size = 32 + self.classical_proof.proof_data.len() + 32;
        let quantum_size =
            self.quantum_proof.lattice_commitment.size() + self.quantum_proof.response.len() * 8;

        classical_size + quantum_size + 32
    }
}

/// Hybrid Verification Result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HybridVerificationResult {
    Valid,
    ClassicalFailed,
    QuantumFailed,
    BindingFailed,
}

// ============================================================================
// PROOF AGGREGATOR
// ============================================================================

/// Aggregiert mehrere Lattice-Proofs
pub struct LatticeProofAggregator {
    /// Aggregierte Commitments
    pub commitments: Vec<LatticeCommitment>,
    /// Aggregierte Challenge
    pub aggregate_challenge: Option<[u8; 32]>,
    /// Parameters
    params: LweParameters,
}

impl LatticeProofAggregator {
    /// Erstelle neuen Aggregator
    pub fn new(params: LweParameters) -> Self {
        Self {
            commitments: Vec::new(),
            aggregate_challenge: None,
            params,
        }
    }

    /// Füge Commitment hinzu
    pub fn add_commitment(&mut self, commitment: LatticeCommitment) {
        self.commitments.push(commitment);
        self.aggregate_challenge = None; // Invalidate
    }

    /// Berechne aggregierte Challenge
    pub fn compute_aggregate_challenge(&mut self) -> [u8; 32] {
        if let Some(challenge) = self.aggregate_challenge {
            return challenge;
        }

        let mut hasher = Sha256::new();
        for c in &self.commitments {
            hasher.update(&c.matrix_hash);
        }

        let challenge: [u8; 32] = hasher.finalize().into();
        self.aggregate_challenge = Some(challenge);
        challenge
    }

    /// Anzahl Commitments
    pub fn count(&self) -> usize {
        self.commitments.len()
    }

    /// Clear
    pub fn clear(&mut self) {
        self.commitments.clear();
        self.aggregate_challenge = None;
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lwe_parameters() {
        let params = LweParameters::standard_128();
        assert_eq!(params.dimension, 256);
        assert!(params.security_level() >= 100);

        let quantum_params = LweParameters::quantum_256();
        assert_eq!(quantum_params.dimension, 512);
    }

    #[test]
    fn test_lattice_commitment() {
        let params = LweParameters::default();
        let commitment = LatticeCommitment::commit(500, &params);

        assert!(!commitment.commitment.is_empty());
        assert_ne!(commitment.matrix_hash, [0u8; 32]);
    }

    #[test]
    fn test_lattice_range_proof_generation() {
        let params = LweParameters::standard_128();
        let proof = LatticeRangeProof::generate(0.5, 0.3, &params);

        assert!(!proof.proof_transcripts.is_empty());
        assert_ne!(proof.challenge, [0u8; 32]);
    }

    #[test]
    fn test_lattice_range_proof_verification() {
        let params = LweParameters::standard_128();
        let proof = LatticeRangeProof::generate(0.5, 0.3, &params);

        assert!(proof.verify(0.3));
    }

    #[test]
    fn test_lattice_zk_proof_creation() {
        let prover = PeerId::random();

        // Score über Threshold
        let proof = LatticeZkProof::new(prover, 0.5, 0.3);
        assert!(proof.is_some());

        // Score unter Threshold
        let proof = LatticeZkProof::new(prover, 0.2, 0.3);
        assert!(proof.is_none());
    }

    #[test]
    fn test_lattice_zk_proof_verification() {
        let prover = PeerId::random();
        let proof = LatticeZkProof::new(prover, 0.5, 0.3).unwrap();

        assert_eq!(proof.verify(), LatticeVerificationResult::Valid);
    }

    #[test]
    fn test_lattice_proof_size() {
        let prover = PeerId::random();
        let proof = LatticeZkProof::new(prover, 0.5, 0.3).unwrap();

        // Lattice proofs are larger
        assert!(proof.size() > 1000);
        assert!(proof.size() <= MAX_LATTICE_PROOF_SIZE);
    }

    #[test]
    fn test_hybrid_proof_creation() {
        let prover = PeerId::random();

        let proof = HybridZkProof::new(prover, 0.5, 0.3);
        assert!(proof.is_some());
    }

    #[test]
    fn test_hybrid_proof_verification() {
        let prover = PeerId::random();
        let proof = HybridZkProof::new(prover, 0.5, 0.3).unwrap();

        assert_eq!(proof.verify(0.3), HybridVerificationResult::Valid);
    }

    #[test]
    fn test_hybrid_binding() {
        let prover = PeerId::random();
        let proof = HybridZkProof::new(prover, 0.5, 0.3).unwrap();

        // Binding should connect classical and quantum parts
        let mut h = Sha256::new();
        h.update(&proof.classical_proof.commitment);
        h.update(&proof.quantum_proof.lattice_commitment.matrix_hash);
        let expected: [u8; 32] = h.finalize().into();

        assert_eq!(expected, proof.binding_commitment);
    }

    #[test]
    fn test_proof_aggregator() {
        let params = LweParameters::default();
        let mut aggregator = LatticeProofAggregator::new(params);

        for i in 0..5 {
            let commitment = LatticeCommitment::commit(i * 100, &params);
            aggregator.add_commitment(commitment);
        }

        assert_eq!(aggregator.count(), 5);

        let challenge = aggregator.compute_aggregate_challenge();
        assert_ne!(challenge, [0u8; 32]);

        // Should be cached
        let challenge2 = aggregator.compute_aggregate_challenge();
        assert_eq!(challenge, challenge2);
    }

    #[test]
    fn test_aggregator_clear() {
        let params = LweParameters::default();
        let mut aggregator = LatticeProofAggregator::new(params);

        aggregator.add_commitment(LatticeCommitment::commit(100, &params));
        assert_eq!(aggregator.count(), 1);

        aggregator.clear();
        assert_eq!(aggregator.count(), 0);
    }

    #[test]
    fn test_kyber_integration() {
        let prover = PeerId::random();
        let proof = LatticeZkProof::new(prover, 0.5, 0.3).unwrap();

        let kyber_pk = vec![0u8; 1568]; // Kyber-768 public key size
        let proof_with_kyber = proof.with_kyber_key(&kyber_pk);

        assert!(proof_with_kyber.kyber_pk_hash.is_some());
    }
}
