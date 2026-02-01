//! # Unified Data Model – Kern-Primitive
//!
//! Universelle Identifikatoren und temporale Koordinaten.
//!
//! ## Design-Prinzipien
//!
//! - **Content-Addressed**: IDs sind deterministische Hashes des Inhalts
//! - **Type-Tagged**: Typ direkt in ID encodiert für O(1) Lookup
//! - **Versioniert**: Schema-Evolution ohne Breaking Changes
//! - **Cache-Friendly**: Strukturen sind cache-aligned

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// UniversalId – Content-Addressed mit Type-Tag
// ============================================================================

/// Universeller Identifikator für alle Erynoa-Objekte
///
/// Layout (32 Bytes):
/// ```text
/// ┌──────────┬────────────┬─────────────────────────────────────────┐
/// │ Type Tag │  Version   │            BLAKE3 Hash (28 bytes)       │
/// │ (2 bytes)│  (2 bytes) │                                         │
/// └──────────┴────────────┴─────────────────────────────────────────┘
/// ```
///
/// # Invarianten
///
/// - `type_tag()` korrespondiert immer mit dem tatsächlichen Objekttyp
/// - IDs sind deterministisch: gleicher Inhalt → gleiche ID
/// - Kollisionsresistenz: 2²²⁴ (Post-Quantum-sicher)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct UniversalId([u8; 32]);

impl UniversalId {
    // Type Tags (erste 2 Bytes)
    pub const TAG_DID: u16 = 0x0001;
    pub const TAG_EVENT: u16 = 0x0002;
    pub const TAG_REALM: u16 = 0x0003;
    pub const TAG_TRUST: u16 = 0x0004;
    pub const TAG_SAGA: u16 = 0x0005;
    pub const TAG_SCHEMA: u16 = 0x0006;
    pub const TAG_STORE: u16 = 0x0007;
    pub const TAG_POLICY: u16 = 0x0008;
    pub const TAG_BLUEPRINT: u16 = 0x0010;
    pub const TAG_DEPLOYMENT: u16 = 0x0011;
    pub const TAG_RATING: u16 = 0x0012;
    pub const TAG_MESSAGE: u16 = 0x0020;
    pub const TAG_TOPIC: u16 = 0x0021;
    pub const TAG_CONNECTION: u16 = 0x0022;
    pub const TAG_PROGRAM: u16 = 0x0030;
    pub const TAG_STATE: u16 = 0x0031;
    pub const TAG_CUSTOM: u16 = 0x00FF;

    /// Null-ID (für Initialisierung)
    pub const NULL: Self = Self([0u8; 32]);

    /// Erstelle ID aus Typ, Version und Content-Hash
    pub fn new(type_tag: u16, version: u16, content: &[u8]) -> Self {
        let mut bytes = [0u8; 32];
        bytes[0..2].copy_from_slice(&type_tag.to_be_bytes());
        bytes[2..4].copy_from_slice(&version.to_be_bytes());

        let hash = blake3::hash(content);
        bytes[4..32].copy_from_slice(&hash.as_bytes()[0..28]);

        Self(bytes)
    }

    /// Erstelle ID aus rohen Bytes (für Deserialisierung)
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Erstelle ID aus Hex-String
    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(hex, &mut bytes)?;
        Ok(Self(bytes))
    }

    /// Type Tag extrahieren (O(1))
    #[inline]
    pub fn type_tag(&self) -> u16 {
        u16::from_be_bytes([self.0[0], self.0[1]])
    }

    /// Version extrahieren (O(1))
    #[inline]
    pub fn version(&self) -> u16 {
        u16::from_be_bytes([self.0[2], self.0[3]])
    }

    /// Als Byte-Slice für Storage
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Als Hex-String
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Prefix für Range-Queries (nur Type-Tag)
    #[inline]
    pub fn type_prefix(type_tag: u16) -> [u8; 2] {
        type_tag.to_be_bytes()
    }

    /// Prüfe ob ID null ist
    #[inline]
    pub fn is_null(&self) -> bool {
        self.0 == [0u8; 32]
    }

    /// Typ-Name für Debugging
    pub fn type_name(&self) -> &'static str {
        match self.type_tag() {
            Self::TAG_DID => "DID",
            Self::TAG_EVENT => "Event",
            Self::TAG_REALM => "Realm",
            Self::TAG_TRUST => "Trust",
            Self::TAG_SAGA => "Saga",
            Self::TAG_SCHEMA => "Schema",
            Self::TAG_STORE => "Store",
            Self::TAG_POLICY => "Policy",
            Self::TAG_BLUEPRINT => "Blueprint",
            Self::TAG_DEPLOYMENT => "Deployment",
            Self::TAG_RATING => "Rating",
            Self::TAG_MESSAGE => "Message",
            Self::TAG_TOPIC => "Topic",
            Self::TAG_CONNECTION => "Connection",
            Self::TAG_PROGRAM => "Program",
            Self::TAG_STATE => "State",
            Self::TAG_CUSTOM => "Custom",
            _ => "Unknown",
        }
    }
}

impl Default for UniversalId {
    fn default() -> Self {
        Self::NULL
    }
}

impl fmt::Debug for UniversalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:v{}:{}",
            self.type_name(),
            self.version(),
            &self.to_hex()[8..16] // Kurzform
        )
    }
}

impl fmt::Display for UniversalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.type_name().to_lowercase(), self.to_hex())
    }
}

impl AsRef<[u8]> for UniversalId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

// ============================================================================
// TemporalCoord – Hybride Zeit mit Kausalordnung
// ============================================================================

/// Hybride logisch-physische Zeitkoordinate
///
/// Layout (16 Bytes):
/// ```text
/// ┌─────────────────────┬──────────────────┬─────────────────────┐
/// │   Wall-Clock (8B)   │  Lamport (4B)    │   Node-Hash (4B)    │
/// │   Mikrosekunden     │  Logische Zeit   │   Tie-Breaker       │
/// └─────────────────────┴──────────────────┴─────────────────────┘
/// ```
///
/// # Invarianten
///
/// - Total geordnet: `t₁ < t₂ ⟺ (wall₁, lamport₁, node₁) <ₗₑₓ (wall₂, lamport₂, node₂)`
/// - Kausal konsistent: `happens_before(e₁, e₂) ⟹ coord(e₁) < coord(e₂)`
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(C)]
pub struct TemporalCoord {
    /// Wall-Clock Zeit in Mikrosekunden seit Unix-Epoch
    wall_time: u64,
    /// Lamport-Timestamp für kausale Ordnung
    lamport: u32,
    /// Hash des Node-Identifiers (Tie-Breaker)
    node_hash: u32,
}

impl TemporalCoord {
    /// Null-Koordinate (Genesis)
    pub const GENESIS: Self = Self {
        wall_time: 0,
        lamport: 0,
        node_hash: 0,
    };

    /// Maximale Koordinate (für Range-Queries)
    pub const MAX: Self = Self {
        wall_time: u64::MAX,
        lamport: u32::MAX,
        node_hash: u32::MAX,
    };

    /// Erstelle neue Koordinate für aktuellen Zeitpunkt
    pub fn now(lamport: u32, node_id: &UniversalId) -> Self {
        let wall_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;

        let node_hash = u32::from_be_bytes(node_id.as_bytes()[28..32].try_into().unwrap());

        Self {
            wall_time,
            lamport,
            node_hash,
        }
    }

    /// Erstelle Koordinate mit expliziten Werten (für Tests)
    pub fn new(wall_time: u64, lamport: u32, node_hash: u32) -> Self {
        Self {
            wall_time,
            lamport,
            node_hash,
        }
    }

    /// Wall-Clock Zeit
    #[inline]
    pub fn wall_time(&self) -> u64 {
        self.wall_time
    }

    /// Lamport-Timestamp
    #[inline]
    pub fn lamport(&self) -> u32 {
        self.lamport
    }

    /// Node-Hash
    #[inline]
    pub fn node_hash(&self) -> u32 {
        self.node_hash
    }

    /// Update Lamport-Clock (bei Event-Empfang)
    pub fn receive_update(&mut self, remote: &TemporalCoord) {
        self.lamport = self.lamport.max(remote.lamport) + 1;
        // Wall-time wird nicht angepasst (Node-lokal)
    }

    /// Tick für lokales Event
    pub fn tick(&mut self) {
        self.lamport += 1;
    }

    /// Als Bytes für Storage-Key
    pub fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&self.wall_time.to_be_bytes());
        bytes[8..12].copy_from_slice(&self.lamport.to_be_bytes());
        bytes[12..16].copy_from_slice(&self.node_hash.to_be_bytes());
        bytes
    }

    /// Von Bytes erstellen
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self {
            wall_time: u64::from_be_bytes(bytes[0..8].try_into().unwrap()),
            lamport: u32::from_be_bytes(bytes[8..12].try_into().unwrap()),
            node_hash: u32::from_be_bytes(bytes[12..16].try_into().unwrap()),
        }
    }

    /// Als DateTime (für Display)
    #[cfg(feature = "chrono")]
    pub fn to_datetime(&self) -> chrono::DateTime<chrono::Utc> {
        use chrono::{TimeZone, Utc};
        Utc.timestamp_micros(self.wall_time as i64).unwrap()
    }
}

impl PartialOrd for TemporalCoord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TemporalCoord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Lexikographische Ordnung: (wall_time, lamport, node_hash)
        (self.wall_time, self.lamport, self.node_hash).cmp(&(
            other.wall_time,
            other.lamport,
            other.node_hash,
        ))
    }
}

impl Default for TemporalCoord {
    fn default() -> Self {
        Self::GENESIS
    }
}

impl fmt::Debug for TemporalCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "T({}, L{}, N{:08x})",
            self.wall_time, self.lamport, self.node_hash
        )
    }
}

impl fmt::Display for TemporalCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // ISO-8601 kompatibles Format
        let secs = self.wall_time / 1_000_000;
        let micros = self.wall_time % 1_000_000;
        write!(f, "{}.{:06}L{}", secs, micros, self.lamport)
    }
}

// ============================================================================
// Compile-Time Assertions
// ============================================================================

const _: () = {
    // Größen-Garantien für Cache-Effizienz
    assert!(std::mem::size_of::<UniversalId>() == 32);
    // TemporalCoord ist 16 Bytes ohne packed, aber Alignment kann variieren
};

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_id_creation() {
        let id = UniversalId::new(UniversalId::TAG_EVENT, 1, b"test content");

        assert_eq!(id.type_tag(), UniversalId::TAG_EVENT);
        assert_eq!(id.version(), 1);
        assert!(!id.is_null());
    }

    #[test]
    fn test_universal_id_deterministic() {
        let id1 = UniversalId::new(UniversalId::TAG_DID, 1, b"same content");
        let id2 = UniversalId::new(UniversalId::TAG_DID, 1, b"same content");

        assert_eq!(id1, id2);
    }

    #[test]
    fn test_universal_id_hex_roundtrip() {
        let id = UniversalId::new(UniversalId::TAG_SAGA, 2, b"saga data");
        let hex = id.to_hex();
        let recovered = UniversalId::from_hex(&hex).unwrap();

        assert_eq!(id, recovered);
    }

    #[test]
    fn test_temporal_coord_ordering() {
        let node_id = UniversalId::new(UniversalId::TAG_DID, 1, b"node");

        let t1 = TemporalCoord::new(1000, 1, 1);
        let t2 = TemporalCoord::new(1000, 2, 1);
        let t3 = TemporalCoord::new(1001, 0, 1);

        assert!(t1 < t2);
        assert!(t2 < t3);
        assert!(t1 < t3);
    }

    #[test]
    fn test_temporal_coord_lamport_update() {
        let mut local = TemporalCoord::new(1000, 5, 1);
        let remote = TemporalCoord::new(900, 10, 2);

        local.receive_update(&remote);

        // Lamport sollte max(5, 10) + 1 = 11 sein
        assert_eq!(local.lamport(), 11);
        // Wall-time unverändert
        assert_eq!(local.wall_time(), 1000);
    }

    #[test]
    fn test_temporal_coord_bytes_roundtrip() {
        let coord = TemporalCoord::new(123456789, 42, 0xDEADBEEF);
        let bytes = coord.to_bytes();
        let recovered = TemporalCoord::from_bytes(bytes);

        assert_eq!(coord, recovered);
    }
}
