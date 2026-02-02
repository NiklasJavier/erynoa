//! # Hardware-Accelerated Crypto (RL26) - Phase 5 Woche 14
//!
//! Plattform-spezifische Beschleunigung für kryptographische Operationen.
//!
//! ## Performance-Ziele
//!
//! - ChaCha20-Poly1305: 2-4× Speedup mit AVX2/AVX-512
//! - X25519: 2× Speedup mit BMI2 (Broadwell+)
//! - ARM: NEON-Optimierung für Apple Silicon und ARM64
//!
//! ## Axiom-Referenzen
//!
//! - **RL26**: Hardware-Crypto-Acceleration
//! - **RL20**: Batch-Processing Integration
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                    HARDWARE CRYPTO ENGINE                                   │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   ┌───────────────────────────────────────────────────────────────────┐    │
//! │   │                      CPU FEATURE DETECTION                        │    │
//! │   │                                                                   │    │
//! │   │   x86_64:  AVX2  AVX-512  AES-NI  BMI2  PCLMULQDQ               │    │
//! │   │   ARM64:   NEON  AES  SHA  PMULL                                 │    │
//! │   │                                                                   │    │
//! │   └───────────────────────────────────────────────────────────────────┘    │
//! │                                │                                            │
//! │                                ▼                                            │
//! │   ┌───────────────────────────────────────────────────────────────────┐    │
//! │   │                     SIMD LEVEL SELECTION                          │    │
//! │   │                                                                   │    │
//! │   │     Scalar ──▶ AVX2 ──▶ AVX-512 ──▶ ARM NEON                    │    │
//! │   │       │                                                          │    │
//! │   │       └── Fallback for all platforms                            │    │
//! │   │                                                                   │    │
//! │   └───────────────────────────────────────────────────────────────────┘    │
//! │                                │                                            │
//! │                                ▼                                            │
//! │   ┌───────────────────────────────────────────────────────────────────┐    │
//! │   │                  OPTIMIZED IMPLEMENTATIONS                        │    │
//! │   │                                                                   │    │
//! │   │   ChaCha20-Poly1305    │    X25519 ECDH    │    BLAKE3          │    │
//! │   │   ──────────────────   │    ───────────    │    ──────          │    │
//! │   │   SIMD quarter-round   │    Field ops      │    Hash parallel   │    │
//! │   │   Parallel blocks      │    Scalar mul     │    Tree hashing    │    │
//! │   │                                                                   │    │
//! │   └───────────────────────────────────────────────────────────────────┘    │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

// ============================================================================
// CPU FEATURE DETECTION
// ============================================================================

/// Erkannte CPU-Capabilities
#[derive(Debug, Clone, Copy)]
pub struct CpuCapabilities {
    // x86_64 Features
    pub has_avx2: bool,
    pub has_avx512f: bool,
    pub has_aes_ni: bool,
    pub has_bmi2: bool,
    pub has_pclmulqdq: bool,

    // ARM64 Features
    pub has_neon: bool,
    pub has_aes_arm: bool,
    pub has_sha_arm: bool,
    pub has_pmull: bool,
}

impl CpuCapabilities {
    /// Erkenne CPU-Features zur Runtime
    pub fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            Self {
                has_avx2: std::arch::is_x86_feature_detected!("avx2"),
                has_avx512f: std::arch::is_x86_feature_detected!("avx512f"),
                has_aes_ni: std::arch::is_x86_feature_detected!("aes"),
                has_bmi2: std::arch::is_x86_feature_detected!("bmi2"),
                has_pclmulqdq: std::arch::is_x86_feature_detected!("pclmulqdq"),
                has_neon: false,
                has_aes_arm: false,
                has_sha_arm: false,
                has_pmull: false,
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            Self {
                has_avx2: false,
                has_avx512f: false,
                has_aes_ni: false,
                has_bmi2: false,
                has_pclmulqdq: false,
                // ARM64 hat NEON immer
                has_neon: true,
                // Features müssen zur Compile-Zeit aktiviert werden
                has_aes_arm: cfg!(target_feature = "aes"),
                has_sha_arm: cfg!(target_feature = "sha2"),
                has_pmull: cfg!(target_feature = "aes"),
            }
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            Self {
                has_avx2: false,
                has_avx512f: false,
                has_aes_ni: false,
                has_bmi2: false,
                has_pclmulqdq: false,
                has_neon: false,
                has_aes_arm: false,
                has_sha_arm: false,
                has_pmull: false,
            }
        }
    }

    /// Beste SIMD-Stufe für diese CPU
    pub fn best_simd_level(&self) -> SimdLevel {
        if self.has_avx512f {
            SimdLevel::Avx512
        } else if self.has_avx2 {
            SimdLevel::Avx2
        } else if self.has_neon {
            SimdLevel::ArmNeon
        } else {
            SimdLevel::Scalar
        }
    }
}

/// Globale CPU-Capabilities (einmal bei Start ermittelt)
static CPU_CAPABILITIES: OnceLock<CpuCapabilities> = OnceLock::new();

/// Hole globale CPU-Capabilities
pub fn cpu_capabilities() -> &'static CpuCapabilities {
    CPU_CAPABILITIES.get_or_init(CpuCapabilities::detect)
}

// ============================================================================
// SIMD LEVEL
// ============================================================================

/// SIMD-Instruction-Level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SimdLevel {
    /// Keine SIMD (Fallback)
    Scalar = 0,
    /// ARM NEON (128-bit)
    ArmNeon = 1,
    /// Intel AVX2 (256-bit)
    Avx2 = 2,
    /// Intel AVX-512 (512-bit)
    Avx512 = 3,
}

impl SimdLevel {
    /// Vector-Breite in Bits
    pub fn vector_width(&self) -> usize {
        match self {
            SimdLevel::Scalar => 0,
            SimdLevel::ArmNeon => 128,
            SimdLevel::Avx2 => 256,
            SimdLevel::Avx512 => 512,
        }
    }

    /// Parallele ChaCha20-Blöcke
    pub fn chacha_parallel_blocks(&self) -> usize {
        match self {
            SimdLevel::Scalar => 1,
            SimdLevel::ArmNeon => 2,
            SimdLevel::Avx2 => 4,
            SimdLevel::Avx512 => 8,
        }
    }

    /// Name für Logging
    pub fn name(&self) -> &'static str {
        match self {
            SimdLevel::Scalar => "Scalar",
            SimdLevel::ArmNeon => "ARM NEON",
            SimdLevel::Avx2 => "AVX2",
            SimdLevel::Avx512 => "AVX-512",
        }
    }
}

// ============================================================================
// HARDWARE CRYPTO ENGINE
// ============================================================================

/// Hardware-Accelerated Crypto Engine
pub struct HwCryptoEngine {
    /// Erkannte CPU-Capabilities
    capabilities: CpuCapabilities,
    /// Aktive SIMD-Stufe
    simd_level: SimdLevel,
    /// Statistiken
    stats: HwCryptoStats,
}

impl HwCryptoEngine {
    /// Erstelle neue Engine mit bester verfügbarer SIMD-Stufe
    pub fn new() -> Self {
        let capabilities = *cpu_capabilities();
        let simd_level = capabilities.best_simd_level();

        Self {
            capabilities,
            simd_level,
            stats: HwCryptoStats::default(),
        }
    }

    /// Erstelle Engine mit spezifischer SIMD-Stufe (für Tests)
    pub fn with_simd_level(level: SimdLevel) -> Self {
        let capabilities = *cpu_capabilities();
        // Fallback auf niedrigere Stufe wenn nicht verfügbar
        let simd_level = std::cmp::min(level, capabilities.best_simd_level());

        Self {
            capabilities,
            simd_level,
            stats: HwCryptoStats::default(),
        }
    }

    /// Aktive SIMD-Stufe
    pub fn simd_level(&self) -> SimdLevel {
        self.simd_level
    }

    /// CPU-Capabilities
    pub fn capabilities(&self) -> &CpuCapabilities {
        &self.capabilities
    }

    /// ChaCha20-Poly1305 Encryption (SIMD-optimiert wo möglich)
    pub fn chacha20_poly1305_encrypt(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 12],
        aad: &[u8],
        plaintext: &[u8],
    ) -> Result<Vec<u8>, HwCryptoError> {
        use chacha20poly1305::{
            aead::{Aead, KeyInit, Payload},
            ChaCha20Poly1305, Nonce,
        };

        let cipher = ChaCha20Poly1305::new_from_slice(key)
            .map_err(|_| HwCryptoError::InvalidKey)?;

        let nonce = Nonce::from_slice(nonce);

        let payload = Payload {
            msg: plaintext,
            aad,
        };

        let ciphertext = cipher
            .encrypt(nonce, payload)
            .map_err(|_| HwCryptoError::EncryptionFailed)?;

        self.stats.encryptions.fetch_add(1, Ordering::Relaxed);
        self.stats
            .bytes_encrypted
            .fetch_add(plaintext.len() as u64, Ordering::Relaxed);

        Ok(ciphertext)
    }

    /// ChaCha20-Poly1305 Decryption (SIMD-optimiert wo möglich)
    pub fn chacha20_poly1305_decrypt(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 12],
        aad: &[u8],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, HwCryptoError> {
        use chacha20poly1305::{
            aead::{Aead, KeyInit, Payload},
            ChaCha20Poly1305, Nonce,
        };

        let cipher = ChaCha20Poly1305::new_from_slice(key)
            .map_err(|_| HwCryptoError::InvalidKey)?;

        let nonce = Nonce::from_slice(nonce);

        let payload = Payload {
            msg: ciphertext,
            aad,
        };

        let plaintext = cipher
            .decrypt(nonce, payload)
            .map_err(|_| HwCryptoError::DecryptionFailed)?;

        self.stats.decryptions.fetch_add(1, Ordering::Relaxed);
        self.stats
            .bytes_decrypted
            .fetch_add(plaintext.len() as u64, Ordering::Relaxed);

        Ok(plaintext)
    }

    /// X25519 Key Exchange (SIMD-optimiert wo möglich)
    pub fn x25519_diffie_hellman(
        &self,
        secret: &x25519_dalek::StaticSecret,
        public: &x25519_dalek::PublicKey,
    ) -> x25519_dalek::SharedSecret {
        self.stats.key_exchanges.fetch_add(1, Ordering::Relaxed);
        secret.diffie_hellman(public)
    }

    /// BLAKE3 Hash (nutzt Tree-Hashing für große Inputs)
    pub fn blake3_hash(&self, data: &[u8]) -> [u8; 32] {
        self.stats.hashes.fetch_add(1, Ordering::Relaxed);
        *blake3::hash(data).as_bytes()
    }

    /// BLAKE3 Keyed Hash
    pub fn blake3_keyed_hash(&self, key: &[u8; 32], data: &[u8]) -> [u8; 32] {
        self.stats.hashes.fetch_add(1, Ordering::Relaxed);
        *blake3::keyed_hash(key, data).as_bytes()
    }

    /// Statistiken
    pub fn stats(&self) -> HwCryptoStatsSnapshot {
        HwCryptoStatsSnapshot {
            encryptions: self.stats.encryptions.load(Ordering::Relaxed),
            decryptions: self.stats.decryptions.load(Ordering::Relaxed),
            bytes_encrypted: self.stats.bytes_encrypted.load(Ordering::Relaxed),
            bytes_decrypted: self.stats.bytes_decrypted.load(Ordering::Relaxed),
            key_exchanges: self.stats.key_exchanges.load(Ordering::Relaxed),
            hashes: self.stats.hashes.load(Ordering::Relaxed),
            simd_level: self.simd_level,
        }
    }

    /// Reset Statistiken
    pub fn reset_stats(&self) {
        self.stats.encryptions.store(0, Ordering::Relaxed);
        self.stats.decryptions.store(0, Ordering::Relaxed);
        self.stats.bytes_encrypted.store(0, Ordering::Relaxed);
        self.stats.bytes_decrypted.store(0, Ordering::Relaxed);
        self.stats.key_exchanges.store(0, Ordering::Relaxed);
        self.stats.hashes.store(0, Ordering::Relaxed);
    }
}

impl Default for HwCryptoEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SIMD CHACHA20 (Platzhalter für tiefere Optimierung)
// ============================================================================

/// SIMD-Optimierte ChaCha20 Quarter-Round
///
/// Hinweis: Die chacha20poly1305 Crate nutzt bereits SIMD intern.
/// Dieser Code ist für zukünftige custom Optimierungen gedacht.
#[cfg(target_arch = "x86_64")]
mod simd_chacha {
    /// ChaCha20 State (4x4 u32 Matrix)
    #[repr(C, align(32))]
    pub struct ChaChaState {
        pub state: [u32; 16],
    }

    impl ChaChaState {
        /// Initialisiere State mit Key und Nonce
        pub fn new(key: &[u8; 32], nonce: &[u8; 12], counter: u32) -> Self {
            let mut state = [0u32; 16];

            // Constants "expand 32-byte k"
            state[0] = 0x61707865;
            state[1] = 0x3320646e;
            state[2] = 0x79622d32;
            state[3] = 0x6b206574;

            // Key
            for i in 0..8 {
                state[4 + i] = u32::from_le_bytes([
                    key[i * 4],
                    key[i * 4 + 1],
                    key[i * 4 + 2],
                    key[i * 4 + 3],
                ]);
            }

            // Counter
            state[12] = counter;

            // Nonce
            for i in 0..3 {
                state[13 + i] = u32::from_le_bytes([
                    nonce[i * 4],
                    nonce[i * 4 + 1],
                    nonce[i * 4 + 2],
                    nonce[i * 4 + 3],
                ]);
            }

            Self { state }
        }
    }
}

// ============================================================================
// BATCH OPERATIONS
// ============================================================================

impl HwCryptoEngine {
    /// Batch-Encryption für mehrere Messages
    pub fn batch_encrypt(
        &self,
        key: &[u8; 32],
        items: &[(Vec<u8>, [u8; 12], Vec<u8>)], // (aad, nonce, plaintext)
    ) -> Vec<Result<Vec<u8>, HwCryptoError>> {
        // Für kleine Batches: sequentiell
        // Für große Batches: könnte rayon nutzen (bereits in batch_crypto.rs)
        items
            .iter()
            .map(|(aad, nonce, plaintext)| {
                self.chacha20_poly1305_encrypt(key, nonce, aad, plaintext)
            })
            .collect()
    }

    /// Batch-Decryption für mehrere Messages
    pub fn batch_decrypt(
        &self,
        key: &[u8; 32],
        items: &[(Vec<u8>, [u8; 12], Vec<u8>)], // (aad, nonce, ciphertext)
    ) -> Vec<Result<Vec<u8>, HwCryptoError>> {
        items
            .iter()
            .map(|(aad, nonce, ciphertext)| {
                self.chacha20_poly1305_decrypt(key, nonce, aad, ciphertext)
            })
            .collect()
    }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Hardware-Crypto Fehler
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HwCryptoError {
    /// Ungültiger Key
    InvalidKey,
    /// Encryption fehlgeschlagen
    EncryptionFailed,
    /// Decryption fehlgeschlagen (Auth-Tag invalid)
    DecryptionFailed,
    /// SIMD-Stufe nicht verfügbar
    SimdNotAvailable,
}

impl std::fmt::Display for HwCryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidKey => write!(f, "Invalid key"),
            Self::EncryptionFailed => write!(f, "Encryption failed"),
            Self::DecryptionFailed => write!(f, "Decryption failed (invalid auth tag)"),
            Self::SimdNotAvailable => write!(f, "Requested SIMD level not available"),
        }
    }
}

impl std::error::Error for HwCryptoError {}

// ============================================================================
// STATISTICS
// ============================================================================

/// Interne Statistiken
#[derive(Default)]
struct HwCryptoStats {
    encryptions: AtomicU64,
    decryptions: AtomicU64,
    bytes_encrypted: AtomicU64,
    bytes_decrypted: AtomicU64,
    key_exchanges: AtomicU64,
    hashes: AtomicU64,
}

/// Statistik-Snapshot
#[derive(Debug, Clone)]
pub struct HwCryptoStatsSnapshot {
    pub encryptions: u64,
    pub decryptions: u64,
    pub bytes_encrypted: u64,
    pub bytes_decrypted: u64,
    pub key_exchanges: u64,
    pub hashes: u64,
    pub simd_level: SimdLevel,
}

impl HwCryptoStatsSnapshot {
    /// Durchschnittliche Bytes pro Encryption
    pub fn avg_encrypt_size(&self) -> f64 {
        if self.encryptions > 0 {
            self.bytes_encrypted as f64 / self.encryptions as f64
        } else {
            0.0
        }
    }

    /// Durchschnittliche Bytes pro Decryption
    pub fn avg_decrypt_size(&self) -> f64 {
        if self.decryptions > 0 {
            self.bytes_decrypted as f64 / self.decryptions as f64
        } else {
            0.0
        }
    }
}

// ============================================================================
// GLOBAL ENGINE
// ============================================================================

/// Globale HwCryptoEngine (Lazy-Initialized)
static GLOBAL_HW_CRYPTO: OnceLock<HwCryptoEngine> = OnceLock::new();

/// Hole globale HwCryptoEngine
pub fn hw_crypto() -> &'static HwCryptoEngine {
    GLOBAL_HW_CRYPTO.get_or_init(HwCryptoEngine::new)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_capabilities_detect() {
        let caps = CpuCapabilities::detect();
        // Sollte nicht crashen
        let level = caps.best_simd_level();
        assert!(level >= SimdLevel::Scalar);
    }

    #[test]
    fn test_simd_level_properties() {
        assert_eq!(SimdLevel::Scalar.vector_width(), 0);
        assert_eq!(SimdLevel::Avx2.vector_width(), 256);
        assert_eq!(SimdLevel::Avx512.vector_width(), 512);
        assert_eq!(SimdLevel::ArmNeon.vector_width(), 128);

        assert_eq!(SimdLevel::Scalar.chacha_parallel_blocks(), 1);
        assert_eq!(SimdLevel::Avx2.chacha_parallel_blocks(), 4);
        assert_eq!(SimdLevel::Avx512.chacha_parallel_blocks(), 8);
    }

    #[test]
    fn test_hw_crypto_engine_creation() {
        let engine = HwCryptoEngine::new();
        let stats = engine.stats();
        assert_eq!(stats.encryptions, 0);
        assert_eq!(stats.decryptions, 0);
    }

    #[test]
    fn test_chacha20_poly1305_roundtrip() {
        let engine = HwCryptoEngine::new();

        let key = [42u8; 32];
        let nonce = [1u8; 12];
        let aad = b"associated data";
        let plaintext = b"Hello, Privacy World!";

        let ciphertext = engine
            .chacha20_poly1305_encrypt(&key, &nonce, aad, plaintext)
            .expect("Encryption failed");

        let decrypted = engine
            .chacha20_poly1305_decrypt(&key, &nonce, aad, &ciphertext)
            .expect("Decryption failed");

        assert_eq!(decrypted, plaintext);

        // Stats check
        let stats = engine.stats();
        assert_eq!(stats.encryptions, 1);
        assert_eq!(stats.decryptions, 1);
    }

    #[test]
    fn test_chacha20_poly1305_auth_failure() {
        let engine = HwCryptoEngine::new();

        let key = [42u8; 32];
        let nonce = [1u8; 12];
        let aad = b"associated data";
        let plaintext = b"Hello!";

        let mut ciphertext = engine
            .chacha20_poly1305_encrypt(&key, &nonce, aad, plaintext)
            .expect("Encryption failed");

        // Manipuliere ciphertext
        ciphertext[0] ^= 0xFF;

        let result = engine.chacha20_poly1305_decrypt(&key, &nonce, aad, &ciphertext);
        assert!(matches!(result, Err(HwCryptoError::DecryptionFailed)));
    }

    #[test]
    fn test_x25519_diffie_hellman() {
        let engine = HwCryptoEngine::new();

        let alice_secret = x25519_dalek::StaticSecret::random_from_rng(&mut rand::thread_rng());
        let alice_public = x25519_dalek::PublicKey::from(&alice_secret);

        let bob_secret = x25519_dalek::StaticSecret::random_from_rng(&mut rand::thread_rng());
        let bob_public = x25519_dalek::PublicKey::from(&bob_secret);

        let alice_shared = engine.x25519_diffie_hellman(&alice_secret, &bob_public);
        let bob_shared = engine.x25519_diffie_hellman(&bob_secret, &alice_public);

        assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());

        let stats = engine.stats();
        assert_eq!(stats.key_exchanges, 2);
    }

    #[test]
    fn test_blake3_hash() {
        let engine = HwCryptoEngine::new();

        let data = b"test data";
        let hash = engine.blake3_hash(data);

        // Verify it's deterministic
        let hash2 = engine.blake3_hash(data);
        assert_eq!(hash, hash2);

        let stats = engine.stats();
        assert_eq!(stats.hashes, 2);
    }

    #[test]
    fn test_blake3_keyed_hash() {
        let engine = HwCryptoEngine::new();

        let key = [42u8; 32];
        let data = b"test data";

        let hash = engine.blake3_keyed_hash(&key, data);

        // Different key should produce different hash
        let key2 = [43u8; 32];
        let hash2 = engine.blake3_keyed_hash(&key2, data);

        assert_ne!(hash, hash2);
    }

    #[test]
    fn test_batch_encrypt_decrypt() {
        let engine = HwCryptoEngine::new();
        let key = [42u8; 32];

        let items: Vec<(Vec<u8>, [u8; 12], Vec<u8>)> = (0..5)
            .map(|i| {
                let aad = format!("aad-{}", i).into_bytes();
                let mut nonce = [0u8; 12];
                nonce[0] = i as u8;
                let plaintext = format!("message-{}", i).into_bytes();
                (aad, nonce, plaintext)
            })
            .collect();

        let ciphertexts = engine.batch_encrypt(&key, &items);
        assert_eq!(ciphertexts.len(), 5);

        // Verify all succeeded
        for ct in &ciphertexts {
            assert!(ct.is_ok());
        }

        // Decrypt
        let decrypt_items: Vec<_> = items
            .iter()
            .zip(ciphertexts.iter())
            .map(|((aad, nonce, _), ct)| {
                (aad.clone(), *nonce, ct.as_ref().unwrap().clone())
            })
            .collect();

        let plaintexts = engine.batch_decrypt(&key, &decrypt_items);

        for (i, pt) in plaintexts.iter().enumerate() {
            assert!(pt.is_ok());
            let expected = format!("message-{}", i);
            assert_eq!(pt.as_ref().unwrap(), expected.as_bytes());
        }
    }

    #[test]
    fn test_stats_reset() {
        let engine = HwCryptoEngine::new();

        let key = [42u8; 32];
        let nonce = [1u8; 12];
        let _ = engine.chacha20_poly1305_encrypt(&key, &nonce, b"", b"test");

        assert_eq!(engine.stats().encryptions, 1);

        engine.reset_stats();

        assert_eq!(engine.stats().encryptions, 0);
    }

    #[test]
    fn test_global_hw_crypto() {
        let engine = hw_crypto();
        let _ = engine.blake3_hash(b"test");

        // Should be same instance
        let engine2 = hw_crypto();
        assert!(std::ptr::eq(engine, engine2));
    }

    #[test]
    fn test_simd_level_ordering() {
        assert!(SimdLevel::Avx512 > SimdLevel::Avx2);
        assert!(SimdLevel::Avx2 > SimdLevel::ArmNeon);
        assert!(SimdLevel::ArmNeon > SimdLevel::Scalar);
    }

    #[test]
    fn test_with_simd_level() {
        let engine = HwCryptoEngine::with_simd_level(SimdLevel::Scalar);
        assert_eq!(engine.simd_level(), SimdLevel::Scalar);

        // Should still work
        let key = [42u8; 32];
        let nonce = [1u8; 12];
        let result = engine.chacha20_poly1305_encrypt(&key, &nonce, b"", b"test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_display() {
        assert_eq!(format!("{}", HwCryptoError::InvalidKey), "Invalid key");
        assert_eq!(
            format!("{}", HwCryptoError::DecryptionFailed),
            "Decryption failed (invalid auth tag)"
        );
    }

    #[test]
    fn test_stats_snapshot_calculations() {
        let stats = HwCryptoStatsSnapshot {
            encryptions: 100,
            decryptions: 50,
            bytes_encrypted: 10000,
            bytes_decrypted: 5000,
            key_exchanges: 25,
            hashes: 200,
            simd_level: SimdLevel::Avx2,
        };

        assert!((stats.avg_encrypt_size() - 100.0).abs() < 0.001);
        assert!((stats.avg_decrypt_size() - 100.0).abs() < 0.001);
    }
}
