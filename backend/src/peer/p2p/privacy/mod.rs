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
