//! # Erynoa Privacy-Layer (P2P-PRIVATE-RELAY-LOGIC V2.6)
//!
//! Anonymes Multi-Hop-Routing mit Trust-basierter Relay-Auswahl.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                        PRIVACY-LAYER                                    │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                                                                         │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
//! │  │    ONION     │  │    RELAY     │  │   MIXING     │                  │
//! │  │   ROUTING    │  │  SELECTION   │  │    POOLS     │                  │
//! │  │  (RL2-RL4)   │  │  (RL5-RL7)   │  │  (RL8-RL10)  │                  │
//! │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                  │
//! │         │                 │                 │                           │
//! │  ┌──────┴─────────────────┴─────────────────┴──────┐                   │
//! │  │              PRIVACY SERVICE                     │                   │
//! │  │  • Onion-Paket-Konstruktion                     │                   │
//! │  │  • Trust-basierte Route-Auswahl                 │                   │
//! │  │  • ε-DP Mixing-Delays                           │                   │
//! │  │  • Cover-Traffic-Generation                     │                   │
//! │  └──────────────────────────────────────────────────┘                   │
//! │                            │                                            │
//! │  ┌────────────────────────┴────────────────────────┐                   │
//! │  │              QUIC TRANSPORT                      │                   │
//! │  │  • 0-RTT Circuit-Setup                          │                   │
//! │  │  • Multi-Stream Multiplexing                    │                   │
//! │  │  • TCP Fallback für NAT-Traversal               │                   │
//! │  └──────────────────────────────────────────────────┘                   │
//! │                                                                         │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Axiom-Referenzen
//!
//! - **RL1**: Relay-Eligibility (ZK-Proofs + Trust-Schwellen)
//! - **RL2**: Wissens-Separation (Informationstheoretisch)
//! - **RL3**: Schichten-Integrität (Authenticated Encryption)
//! - **RL4**: Forward + Backward Secrecy (Ephemeral Keys)
//! - **RL5**: Trust-Monotonie (Spieltheoretische Anreize)
//! - **RL6**: Relay-Diversität (Entropie-Maximierung)
//! - **RL7**: Adaptive Hop-Anzahl (Sensitivitäts-basiert)
//! - **RL8**: ε-DP Mixing (Laplace-Delays)
//! - **RL15**: Replay-Protection (Nonce-Cache)
//!
//! ## Core-Logic-Verknüpfungen (LOGIC.md V4.1)
//!
//! - **Κ3**: 6D Trust-Vektor für Relay-Scoring
//! - **Κ4**: Asymmetrische Evolution (Misbehavior-Penalties)
//! - **Κ5**: Probabilistische Kombination (Trust-Aggregation)
//! - **Κ19**: Anti-Calcification (Relay-Power-Limits)
//! - **Κ20**: Diversity-Requirement (Multi-Jurisdiction)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use erynoa_api::peer::p2p::privacy::{
//!     OnionBuilder, OnionDecryptor, RelaySelector, SensitivityLevel
//! };
//!
//! // Route auswählen
//! let selector = RelaySelector::new(candidates, SensitivityLevel::High);
//! let route = selector.select_route()?;
//!
//! // Onion-Paket bauen
//! let builder = OnionBuilder::new(route);
//! let packet = builder.build(plaintext, &dest_addr);
//!
//! // Entschlüsselung (auf Relay)
//! let decryptor = OnionDecryptor::new(private_key);
//! let layer = decryptor.decrypt_layer(&packet)?;
//! ```

#[cfg(feature = "privacy")]
pub mod onion;

#[cfg(feature = "privacy")]
pub mod relay_selection;

#[cfg(feature = "privacy")]
pub mod mixing;

#[cfg(feature = "privacy")]
pub mod cover_traffic;

#[cfg(feature = "privacy")]
pub mod eligibility;

#[cfg(feature = "privacy")]
pub mod dc3;

// Phase 2 Woche 8: Integration Service
#[cfg(feature = "privacy")]
pub mod service;

// Phase 4 Woche 11: Wire-Format
#[cfg(feature = "privacy")]
pub mod wire_format;

// Phase 3 Woche 9-12: ZK-Contribution & Post-Quantum
#[cfg(feature = "privacy-zk")]
pub mod resource_verification;

#[cfg(feature = "privacy-zk")]
pub mod proof_of_useful_work;

#[cfg(feature = "privacy-zk")]
pub mod contribution_scoring;

#[cfg(feature = "privacy-zk")]
pub mod zk_contribution;

#[cfg(feature = "privacy-zk")]
pub mod lattice_zk;

// Re-exports für einfachen Zugriff
#[cfg(feature = "privacy")]
pub use onion::{
    DecryptedLayer, EphemeralKeyAgreement, OnionBuilder, OnionDecryptor, OnionError, OnionLayer,
    SessionKey, LAYER_HEADER_SIZE, MAX_HOPS,
};

#[cfg(feature = "privacy")]
pub use relay_selection::{
    DiversityConstraints, RelayCandidate, RelaySelectionError, RelaySelector, RelayTrustScore,
    SensitivityLevel,
};

#[cfg(feature = "privacy")]
pub use mixing::{
    LampStats, MixingPool, MixingPoolConfig, TrafficRateMonitor, K_MAX, K_MIN, TAU_MAX_MS,
    TAU_MIN_MS,
};

#[cfg(feature = "privacy")]
pub use cover_traffic::{
    quantize_size, random_size_class, ComplianceMonitor, ComplianceResult, ComplianceStatus,
    CoverGeneratorStats, CoverMessage, CoverTrafficConfig, CoverTrafficGenerator,
    CoverTrafficStats, PeerType, SelfComplianceResult, SIZE_CLASSES,
};

#[cfg(feature = "privacy")]
pub use eligibility::{
    check_eligibility, ApprenticeConstraints, ApprenticeStats, BootstrapPhase, BootstrapStatus,
    EligibilityResult, EligibilityThresholds, FoundationTrust, MinimumCommitment,
    ZkEligibilityProof, MIN_APPRENTICE_WEEKS, MIN_DC3_SCORE, MIN_SUCCESS_RATE_FULL,
    MIN_TRUST_R_APPRENTICE, MIN_TRUST_R_FULL,
};

#[cfg(feature = "privacy")]
pub use dc3::{
    ChallengeGenerator, ChallengeParams, ChallengeResponse, ChallengeResult, ChallengeType,
    CumulativeContributionScore, DC3Service, DC3Stats, DynamicChallenge, ResponseProof,
};

// Phase 2 Woche 8: Privacy-Service Integration Re-Exports
#[cfg(feature = "privacy")]
pub use service::{
    PrivacyError, PrivacyMessage, PrivacyService, PrivacyServiceConfig, PrivacyServiceStats,
    ProcessingResult,
};

// Phase 4 Woche 11: Wire-Format Re-Exports
#[cfg(feature = "privacy")]
pub use wire_format::{
    quantize_to_size_class, unpad_from_size_class, MessageType, OnionLayerHeader, PacketFlags,
    PacketHeader, PrivacyPacket, WireError, SIZE_CLASSES as WIRE_SIZE_CLASSES,
};

// Phase 3 Woche 9-12: ZK-Contribution & Post-Quantum Re-Exports
#[cfg(feature = "privacy-zk")]
pub use resource_verification::{
    BandwidthEpochProof, BilateralAttestation, DailyComputeProof, MerkleProof,
    MixingBatchCommitment, RelayReceipt, ResourceVerificationService, StorageChallenge,
    StorageMerkleTree, StorageProof, VerifiedResourceCommitment, ZkShuffleProof,
};

#[cfg(feature = "privacy-zk")]
pub use proof_of_useful_work::{
    ContentRoutingReceipt, ContentRoutingRequest, DailyWorkSummary, DhtIndexUpdate,
    DhtQueryChallenge, DhtQueryResponse, ProofOfUsefulWorkService, WorkAttestation, WorkType,
    ZkVerificationChallenge, ZkVerificationResult as PoUWZkVerificationResult,
};

#[cfg(feature = "privacy-zk")]
pub use contribution_scoring::{
    ContributionSummary, CumulativeContributionScore as ZkCumulativeContributionScore,
    ExponentialDecayCalculator, ScoreAggregator,
};

#[cfg(feature = "privacy-zk")]
pub use zk_contribution::{
    DilithiumZkProof, EligibilityPhase, EligibilityProof, PedersenCommitment, ProofBatchVerifier,
    RangeProof, ZkContributionProof, ZkProofType, ZkVerificationResult,
};

#[cfg(feature = "privacy-zk")]
pub use lattice_zk::{
    ClassicalProofPart, HybridVerificationResult, HybridZkProof, LatticeCommitment,
    LatticeProofAggregator, LatticeRangeProof, LatticeVerificationResult, LatticeZkProof,
    LweParameters, ProofTranscript, QuantumProofPart,
};
