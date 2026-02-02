//! # Wire-Format (Byte-Level Protocol) - Phase 4 Woche 11
//!
//! Definiert das Byte-Format für Onion-Pakete auf dem Netzwerk.
//!
//! ## Referenz: P2P-PRIVATE-RELAY-LOGIC.md Section XII
//!
//! ## Axiom-Referenzen
//!
//! - **RL21**: Size-Quantization (8 Stufen für Traffic-Analysis-Resistenz)
//! - **RL14**: Timing-Obfuscation (Timestamp in Header)
//!
//! ## Wire-Format Layout
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                        ERYNOA PRIVACY PACKET                                │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │  ┌───────────────────────────────────────────────────────────────────────┐  │
//! │  │                    PACKET HEADER (16 Bytes)                           │  │
//! │  │  ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬────────────────────┐    │  │
//! │  │  │Magic│Magic│Magic│Magic│ Ver │Type │Class│Flags│  Timestamp   │    │  │
//! │  │  │ 'E' │ 'R' │ 'Y' │ 'N' │  1  │     │ 0-7 │     │   (64-bit)   │    │  │
//! │  │  └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴──────────────┘    │  │
//! │  └───────────────────────────────────────────────────────────────────────┘  │
//! │                                                                             │
//! │  ┌───────────────────────────────────────────────────────────────────────┐  │
//! │  │                    ONION LAYER HEADER (60 Bytes)                      │  │
//! │  │  ┌────────────────────────────────────────────────────────────────┐   │  │
//! │  │  │  Ephemeral Public Key (32 Bytes, X25519)                       │   │  │
//! │  │  ├────────────────────────────────────────────────────────────────┤   │  │
//! │  │  │  Nonce (12 Bytes)                                              │   │  │
//! │  │  ├────────────────────────────────────────────────────────────────┤   │  │
//! │  │  │  Auth Tag (16 Bytes, Poly1305)                                 │   │  │
//! │  │  └────────────────────────────────────────────────────────────────┘   │  │
//! │  └───────────────────────────────────────────────────────────────────────┘  │
//! │                                                                             │
//! │  ┌───────────────────────────────────────────────────────────────────────┐  │
//! │  │                    PAYLOAD (Variable, Quantized)                      │  │
//! │  │  Size-Class: 256, 512, 1024, 2048, 4096, 8192, 16384, 32768 Bytes    │  │
//! │  │  [Encrypted Inner Layers + PKCS#7-like Padding]                      │  │
//! │  └───────────────────────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```

use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Protocol Version
pub const PROTOCOL_VERSION: u8 = 1;

/// Magic Bytes für Erkennung: "ERYN"
pub const MAGIC: [u8; 4] = [0x45, 0x52, 0x59, 0x4E];

/// Size-Classes für Traffic-Analysis-Resistenz (RL21)
///
/// 8 Stufen von 256 Bytes bis 32 KB
/// Jede Nachricht wird auf die nächsthöhere Klasse aufgerundet
pub const SIZE_CLASSES: [u16; 8] = [256, 512, 1024, 2048, 4096, 8192, 16384, 32768];

/// Maximum Size-Class Index
pub const MAX_SIZE_CLASS: u8 = 7;

// ============================================================================
// MESSAGE TYPE
// ============================================================================

/// Message-Typ im Wire-Format
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageType {
    /// Normales Onion-Paket (RL2-RL4)
    Onion = 0x01,
    /// Cover-Traffic (RL10, RL18)
    /// Nur für Egress-Relay erkennbar
    Cover = 0x02,
    /// Circuit-Setup Request
    CircuitSetup = 0x10,
    /// Circuit-Teardown Request
    CircuitTeardown = 0x11,
    /// Acknowledgment
    Ack = 0x20,
    /// Negative Acknowledgment / Error
    Nack = 0x21,
    /// Keepalive / Heartbeat
    Keepalive = 0x30,
    /// Error Response
    Error = 0xFF,
}

impl MessageType {
    /// Konvertiere von u8
    pub fn from_byte(byte: u8) -> Result<Self, WireError> {
        match byte {
            0x01 => Ok(MessageType::Onion),
            0x02 => Ok(MessageType::Cover),
            0x10 => Ok(MessageType::CircuitSetup),
            0x11 => Ok(MessageType::CircuitTeardown),
            0x20 => Ok(MessageType::Ack),
            0x21 => Ok(MessageType::Nack),
            0x30 => Ok(MessageType::Keepalive),
            0xFF => Ok(MessageType::Error),
            _ => Err(WireError::InvalidMessageType(byte)),
        }
    }

    /// Ist dies ein Daten-Paket?
    pub fn is_data(&self) -> bool {
        matches!(self, MessageType::Onion | MessageType::Cover)
    }

    /// Ist dies ein Control-Paket?
    pub fn is_control(&self) -> bool {
        matches!(
            self,
            MessageType::CircuitSetup
                | MessageType::CircuitTeardown
                | MessageType::Ack
                | MessageType::Nack
                | MessageType::Keepalive
        )
    }
}

// ============================================================================
// PACKET FLAGS
// ============================================================================

/// Paket-Flags (1 Byte)
#[derive(Debug, Clone, Copy, Default)]
pub struct PacketFlags {
    /// Bit 0: Ist dies das finale Ziel?
    pub is_final: bool,
    /// Bit 1: Erfordert Acknowledgment
    pub requires_ack: bool,
    /// Bit 2: Ist Priority-Traffic
    pub is_priority: bool,
    /// Bit 3: Padding-Only (für Cover-Traffic)
    pub padding_only: bool,
    /// Bit 4-7: Reserved
    pub reserved: u8,
}

impl PacketFlags {
    /// Serialisiere zu Byte
    pub fn to_byte(&self) -> u8 {
        let mut flags = 0u8;
        if self.is_final {
            flags |= 0x01;
        }
        if self.requires_ack {
            flags |= 0x02;
        }
        if self.is_priority {
            flags |= 0x04;
        }
        if self.padding_only {
            flags |= 0x08;
        }
        flags |= (self.reserved & 0x0F) << 4;
        flags
    }

    /// Deserialisiere von Byte
    pub fn from_byte(byte: u8) -> Self {
        Self {
            is_final: byte & 0x01 != 0,
            requires_ack: byte & 0x02 != 0,
            is_priority: byte & 0x04 != 0,
            padding_only: byte & 0x08 != 0,
            reserved: (byte >> 4) & 0x0F,
        }
    }
}

// ============================================================================
// PACKET HEADER
// ============================================================================

/// Paket-Header (Fixed 16 Bytes)
///
/// Layout:
/// ```text
/// ┌─────────────────────────────────────────────────────────────────────────────┐
/// │  0     1     2     3     4     5     6     7     8-15                       │
/// ├─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬────────────────────────────┤
/// │Magic│Magic│Magic│Magic│ Ver │Type │Class│Flags│      Timestamp (64-bit)   │
/// │ 'E' │ 'R' │ 'Y' │ 'N' │  1  │     │ 0-7 │     │   Milliseconds seit Epoch │
/// └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴────────────────────────────┘
/// ```
#[derive(Debug, Clone)]
pub struct PacketHeader {
    /// Protocol Version (aktuell: 1)
    pub version: u8,
    /// Message Type
    pub msg_type: MessageType,
    /// Size-Class Index (0-7)
    pub size_class: u8,
    /// Flags
    pub flags: PacketFlags,
    /// Unix Timestamp (Millisekunden seit Epoch)
    pub timestamp_ms: u64,
}

impl PacketHeader {
    /// Header-Größe in Bytes
    pub const SIZE: usize = 16;

    /// Erstelle neuen Header mit aktuellem Timestamp
    pub fn new(msg_type: MessageType, size_class: u8, flags: PacketFlags) -> Self {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            version: PROTOCOL_VERSION,
            msg_type,
            size_class: size_class.min(MAX_SIZE_CLASS),
            flags,
            timestamp_ms,
        }
    }

    /// Erstelle Onion-Header
    pub fn onion(size_class: u8, is_final: bool) -> Self {
        Self::new(
            MessageType::Onion,
            size_class,
            PacketFlags {
                is_final,
                ..Default::default()
            },
        )
    }

    /// Erstelle Cover-Traffic-Header
    pub fn cover(size_class: u8) -> Self {
        Self::new(
            MessageType::Cover,
            size_class,
            PacketFlags {
                padding_only: true,
                ..Default::default()
            },
        )
    }

    /// Serialisiere zu Bytes
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        buf[0..4].copy_from_slice(&MAGIC);
        buf[4] = self.version;
        buf[5] = self.msg_type as u8;
        buf[6] = self.size_class;
        buf[7] = self.flags.to_byte();
        buf[8..16].copy_from_slice(&self.timestamp_ms.to_be_bytes());
        buf
    }

    /// Deserialisiere von Bytes
    pub fn from_bytes(buf: &[u8]) -> Result<Self, WireError> {
        if buf.len() < Self::SIZE {
            return Err(WireError::BufferTooSmall {
                expected: Self::SIZE,
                actual: buf.len(),
            });
        }

        // Magic-Check
        if buf[0..4] != MAGIC {
            return Err(WireError::InvalidMagic);
        }

        // Version-Check
        let version = buf[4];
        if version != PROTOCOL_VERSION {
            return Err(WireError::UnsupportedVersion(version));
        }

        let msg_type = MessageType::from_byte(buf[5])?;
        let size_class = buf[6];
        if size_class > MAX_SIZE_CLASS {
            return Err(WireError::InvalidSizeClass(size_class));
        }

        let flags = PacketFlags::from_byte(buf[7]);
        let timestamp_ms = u64::from_be_bytes(buf[8..16].try_into().unwrap());

        Ok(Self {
            version,
            msg_type,
            size_class,
            flags,
            timestamp_ms,
        })
    }

    /// Hole erwartete Payload-Größe basierend auf Size-Class
    pub fn expected_payload_size(&self) -> usize {
        SIZE_CLASSES
            .get(self.size_class as usize)
            .copied()
            .unwrap_or(SIZE_CLASSES[MAX_SIZE_CLASS as usize]) as usize
    }
}

// ============================================================================
// ONION LAYER HEADER
// ============================================================================

/// Onion-Layer Header (60 Bytes)
///
/// Layout:
/// ```text
/// ┌────────────────────────────────────────────────────────────────────────────┐
/// │  0-31: Ephemeral Public Key (X25519)                                       │
/// │ 32-43: Nonce (12 Bytes)                                                    │
/// │ 44-59: Auth Tag (16 Bytes, Poly1305)                                       │
/// └────────────────────────────────────────────────────────────────────────────┘
/// ```
#[derive(Debug, Clone)]
pub struct OnionLayerHeader {
    /// Ephemeral X25519 Public Key (32 Bytes)
    pub ephemeral_pk: [u8; 32],
    /// ChaCha20-Poly1305 Nonce (12 Bytes)
    pub nonce: [u8; 12],
    /// Poly1305 Auth Tag (16 Bytes)
    pub auth_tag: [u8; 16],
}

impl OnionLayerHeader {
    /// Header-Größe in Bytes
    pub const SIZE: usize = 60;

    /// Erstelle neuen Header
    pub fn new(ephemeral_pk: [u8; 32], nonce: [u8; 12], auth_tag: [u8; 16]) -> Self {
        Self {
            ephemeral_pk,
            nonce,
            auth_tag,
        }
    }

    /// Serialisiere zu Bytes
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        buf[0..32].copy_from_slice(&self.ephemeral_pk);
        buf[32..44].copy_from_slice(&self.nonce);
        buf[44..60].copy_from_slice(&self.auth_tag);
        buf
    }

    /// Deserialisiere von Bytes
    pub fn from_bytes(buf: &[u8]) -> Result<Self, WireError> {
        if buf.len() < Self::SIZE {
            return Err(WireError::BufferTooSmall {
                expected: Self::SIZE,
                actual: buf.len(),
            });
        }

        Ok(Self {
            ephemeral_pk: buf[0..32].try_into().unwrap(),
            nonce: buf[32..44].try_into().unwrap(),
            auth_tag: buf[44..60].try_into().unwrap(),
        })
    }
}

// ============================================================================
// SIZE-CLASS QUANTIZATION (RL21)
// ============================================================================

/// Finde die passende Size-Class für eine Payload-Größe
///
/// Rundet auf die nächsthöhere Klasse auf für Traffic-Analysis-Resistenz.
pub fn find_size_class(payload_len: usize) -> u8 {
    for (i, &size) in SIZE_CLASSES.iter().enumerate() {
        if payload_len <= size as usize {
            return i as u8;
        }
    }
    MAX_SIZE_CLASS
}

/// Quantisiere Payload auf Size-Class (RL21)
///
/// Fügt PKCS#7-ähnliches Padding hinzu um die Ziel-Größe zu erreichen.
///
/// ## Beispiel
///
/// ```rust,ignore
/// let payload = vec![1, 2, 3]; // 3 Bytes
/// let (class, padded) = quantize_to_size_class(&payload);
/// assert_eq!(class, 0); // 256 Bytes class
/// assert_eq!(padded.len(), 256);
/// ```
pub fn quantize_to_size_class(payload: &[u8]) -> (u8, Vec<u8>) {
    let class_idx = find_size_class(payload.len());
    let target_size = SIZE_CLASSES[class_idx as usize] as usize;

    let mut padded = Vec::with_capacity(target_size);
    padded.extend_from_slice(payload);

    // PKCS#7-ähnliches Padding
    // Jedes Padding-Byte enthält die Anzahl der Padding-Bytes
    let padding_len = target_size - payload.len();
    if padding_len > 0 {
        // Für große Padding-Längen verwenden wir 255 + Länge als erste 2 Bytes
        if padding_len > 255 {
            // Extended Padding: [0x00, len_high, len_low, padding_byte...]
            let len_bytes = (padding_len as u16).to_be_bytes();
            padded.push(0x00); // Marker für extended padding
            padded.push(len_bytes[0]);
            padded.push(len_bytes[1]);
            padded.resize(target_size, 0x00);
        } else {
            // Standard PKCS#7: Alle Padding-Bytes = padding_len
            padded.resize(target_size, padding_len as u8);
        }
    }

    (class_idx, padded)
}

/// Entferne Padding von quantisierter Payload
///
/// Erkennt PKCS#7-Padding und entfernt es.
pub fn unpad_from_size_class(padded: &[u8]) -> Result<Vec<u8>, WireError> {
    if padded.is_empty() {
        return Err(WireError::InvalidPadding);
    }

    let last_byte = padded[padded.len() - 1] as usize;

    // Extended Padding Check
    if padded.len() >= 3 && padded[padded.len() - 1] == 0x00 {
        // Könnte extended padding sein - suche nach Marker
        // Für jetzt: Einfacher Heuristik
        // Wenn letzte Bytes alle 0 sind, suche nach Pattern
        let mut zero_count = 0;
        for &b in padded.iter().rev() {
            if b == 0x00 {
                zero_count += 1;
            } else {
                break;
            }
        }

        if zero_count >= 3 {
            // Prüfe auf extended padding marker
            let potential_marker_pos = padded.len() - zero_count;
            if potential_marker_pos > 0 && padded[potential_marker_pos - 1] == 0x00 {
                // Extended padding detected - extrahiere Länge
                if potential_marker_pos >= 3 {
                    let len = u16::from_be_bytes([
                        padded[potential_marker_pos],
                        padded[potential_marker_pos + 1],
                    ]) as usize;
                    if len <= padded.len() && len >= 3 {
                        return Ok(padded[..padded.len() - len].to_vec());
                    }
                }
            }
        }
    }

    // Standard PKCS#7 Padding
    if last_byte == 0 || last_byte > padded.len() {
        return Err(WireError::InvalidPadding);
    }

    // Verifiziere dass alle Padding-Bytes korrekt sind
    for &b in &padded[padded.len() - last_byte..] {
        if b as usize != last_byte {
            return Err(WireError::InvalidPadding);
        }
    }

    Ok(padded[..padded.len() - last_byte].to_vec())
}

// ============================================================================
// COMPLETE PACKET
// ============================================================================

/// Vollständiges Privacy-Paket
#[derive(Debug, Clone)]
pub struct PrivacyPacket {
    /// Packet-Header (16 Bytes)
    pub header: PacketHeader,
    /// Onion-Layer-Header (60 Bytes, optional für Control-Pakete)
    pub onion_header: Option<OnionLayerHeader>,
    /// Payload (quantisiert)
    pub payload: Vec<u8>,
}

impl PrivacyPacket {
    /// Minimale Paket-Größe (Header only)
    pub const MIN_SIZE: usize = PacketHeader::SIZE;

    /// Erstelle neues Onion-Paket
    pub fn new_onion(
        ephemeral_pk: [u8; 32],
        nonce: [u8; 12],
        auth_tag: [u8; 16],
        payload: Vec<u8>,
        is_final: bool,
    ) -> Self {
        let (size_class, quantized) = quantize_to_size_class(&payload);

        Self {
            header: PacketHeader::onion(size_class, is_final),
            onion_header: Some(OnionLayerHeader::new(ephemeral_pk, nonce, auth_tag)),
            payload: quantized,
        }
    }

    /// Erstelle Cover-Traffic-Paket
    pub fn new_cover(size_class: u8) -> Self {
        let size = SIZE_CLASSES[size_class.min(MAX_SIZE_CLASS) as usize] as usize;
        let dummy_payload = vec![0u8; size];

        Self {
            header: PacketHeader::cover(size_class),
            onion_header: Some(OnionLayerHeader::new([0u8; 32], [0u8; 12], [0u8; 16])),
            payload: dummy_payload,
        }
    }

    /// Erstelle Control-Paket (ohne Onion-Header)
    pub fn new_control(msg_type: MessageType, payload: Vec<u8>) -> Self {
        let (size_class, quantized) = quantize_to_size_class(&payload);

        Self {
            header: PacketHeader::new(msg_type, size_class, PacketFlags::default()),
            onion_header: None,
            payload: quantized,
        }
    }

    /// Serialisiere zu Bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let has_onion = self.onion_header.is_some();
        let total_size = PacketHeader::SIZE
            + if has_onion { OnionLayerHeader::SIZE } else { 0 }
            + self.payload.len();

        let mut buf = Vec::with_capacity(total_size);
        buf.extend_from_slice(&self.header.to_bytes());

        if let Some(ref onion) = self.onion_header {
            buf.extend_from_slice(&onion.to_bytes());
        }

        buf.extend_from_slice(&self.payload);
        buf
    }

    /// Deserialisiere von Bytes
    pub fn from_bytes(buf: &[u8]) -> Result<Self, WireError> {
        if buf.len() < PacketHeader::SIZE {
            return Err(WireError::BufferTooSmall {
                expected: PacketHeader::SIZE,
                actual: buf.len(),
            });
        }

        let header = PacketHeader::from_bytes(&buf[0..PacketHeader::SIZE])?;

        // Prüfe ob Onion-Header erwartet wird
        let has_onion = header.msg_type.is_data();

        let (onion_header, payload_start) = if has_onion {
            let onion_start = PacketHeader::SIZE;
            let onion_end = onion_start + OnionLayerHeader::SIZE;

            if buf.len() < onion_end {
                return Err(WireError::BufferTooSmall {
                    expected: onion_end,
                    actual: buf.len(),
                });
            }

            let onion = OnionLayerHeader::from_bytes(&buf[onion_start..onion_end])?;
            (Some(onion), onion_end)
        } else {
            (None, PacketHeader::SIZE)
        };

        let payload = buf[payload_start..].to_vec();

        // Validiere Payload-Größe gegen Size-Class
        let expected_size = header.expected_payload_size();
        if payload.len() != expected_size {
            return Err(WireError::PayloadSizeMismatch {
                expected: expected_size,
                actual: payload.len(),
            });
        }

        Ok(Self {
            header,
            onion_header,
            payload,
        })
    }

    /// Hole unpadded Payload
    pub fn unpadded_payload(&self) -> Result<Vec<u8>, WireError> {
        unpad_from_size_class(&self.payload)
    }

    /// Gesamtgröße in Bytes
    pub fn wire_size(&self) -> usize {
        PacketHeader::SIZE
            + if self.onion_header.is_some() {
                OnionLayerHeader::SIZE
            } else {
                0
            }
            + self.payload.len()
    }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Wire-Format Fehler
#[derive(Debug, thiserror::Error)]
pub enum WireError {
    #[error("Invalid magic bytes (expected 'ERYN')")]
    InvalidMagic,

    #[error("Unsupported protocol version: {0} (expected {PROTOCOL_VERSION})")]
    UnsupportedVersion(u8),

    #[error("Invalid message type: 0x{0:02X}")]
    InvalidMessageType(u8),

    #[error("Invalid size class: {0} (max {MAX_SIZE_CLASS})")]
    InvalidSizeClass(u8),

    #[error("Invalid padding")]
    InvalidPadding,

    #[error("Buffer too small: expected {expected} bytes, got {actual}")]
    BufferTooSmall { expected: usize, actual: usize },

    #[error("Payload size mismatch: expected {expected}, got {actual}")]
    PayloadSizeMismatch { expected: usize, actual: usize },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_roundtrip() {
        for msg_type in [
            MessageType::Onion,
            MessageType::Cover,
            MessageType::CircuitSetup,
            MessageType::CircuitTeardown,
            MessageType::Ack,
            MessageType::Nack,
            MessageType::Keepalive,
            MessageType::Error,
        ] {
            let byte = msg_type as u8;
            let parsed = MessageType::from_byte(byte).unwrap();
            assert_eq!(parsed, msg_type);
        }
    }

    #[test]
    fn test_invalid_message_type() {
        assert!(MessageType::from_byte(0x99).is_err());
    }

    #[test]
    fn test_packet_flags_roundtrip() {
        let flags = PacketFlags {
            is_final: true,
            requires_ack: true,
            is_priority: false,
            padding_only: true,
            reserved: 0x0A,
        };

        let byte = flags.to_byte();
        let parsed = PacketFlags::from_byte(byte);

        assert_eq!(parsed.is_final, flags.is_final);
        assert_eq!(parsed.requires_ack, flags.requires_ack);
        assert_eq!(parsed.is_priority, flags.is_priority);
        assert_eq!(parsed.padding_only, flags.padding_only);
        assert_eq!(parsed.reserved, flags.reserved);
    }

    #[test]
    fn test_packet_header_roundtrip() {
        let header = PacketHeader::onion(3, true);
        let bytes = header.to_bytes();

        assert_eq!(&bytes[0..4], &MAGIC);
        assert_eq!(bytes[4], PROTOCOL_VERSION);

        let parsed = PacketHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.version, header.version);
        assert_eq!(parsed.msg_type, header.msg_type);
        assert_eq!(parsed.size_class, header.size_class);
        assert_eq!(parsed.flags.is_final, header.flags.is_final);
    }

    #[test]
    fn test_invalid_magic() {
        let mut bytes = PacketHeader::onion(0, false).to_bytes();
        bytes[0] = 0x00; // Corrupt magic

        assert!(matches!(
            PacketHeader::from_bytes(&bytes),
            Err(WireError::InvalidMagic)
        ));
    }

    #[test]
    fn test_onion_layer_header_roundtrip() {
        let header = OnionLayerHeader::new([1u8; 32], [2u8; 12], [3u8; 16]);

        let bytes = header.to_bytes();
        let parsed = OnionLayerHeader::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.ephemeral_pk, header.ephemeral_pk);
        assert_eq!(parsed.nonce, header.nonce);
        assert_eq!(parsed.auth_tag, header.auth_tag);
    }

    #[test]
    fn test_find_size_class() {
        assert_eq!(find_size_class(0), 0); // -> 256
        assert_eq!(find_size_class(100), 0); // -> 256
        assert_eq!(find_size_class(256), 0); // -> 256
        assert_eq!(find_size_class(257), 1); // -> 512
        assert_eq!(find_size_class(1000), 2); // -> 1024
        assert_eq!(find_size_class(32000), 7); // -> 32768
        assert_eq!(find_size_class(50000), 7); // -> 32768 (max)
    }

    #[test]
    fn test_quantize_small_payload() {
        let payload = vec![1, 2, 3, 4, 5];
        let (class, padded) = quantize_to_size_class(&payload);

        assert_eq!(class, 0); // 256 bytes
        assert_eq!(padded.len(), 256);
        assert_eq!(&padded[0..5], &payload);
    }

    #[test]
    fn test_unpad_roundtrip() {
        let original = vec![10, 20, 30, 40, 50, 60, 70, 80];
        let (_, padded) = quantize_to_size_class(&original);

        let unpadded = unpad_from_size_class(&padded).unwrap();
        assert_eq!(unpadded, original);
    }

    #[test]
    fn test_privacy_packet_onion_roundtrip() {
        let payload = b"Hello, Erynoa!".to_vec();
        let packet =
            PrivacyPacket::new_onion([1u8; 32], [2u8; 12], [3u8; 16], payload.clone(), true);

        let bytes = packet.to_bytes();
        let parsed = PrivacyPacket::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.header.msg_type, MessageType::Onion);
        assert_eq!(parsed.header.flags.is_final, true);
        assert!(parsed.onion_header.is_some());

        let unpadded = parsed.unpadded_payload().unwrap();
        assert_eq!(unpadded, payload);
    }

    #[test]
    fn test_privacy_packet_cover() {
        let packet = PrivacyPacket::new_cover(2);

        assert_eq!(packet.header.msg_type, MessageType::Cover);
        assert_eq!(packet.header.size_class, 2);
        assert_eq!(packet.payload.len(), 1024);
    }

    #[test]
    fn test_privacy_packet_control() {
        let payload = vec![0x01, 0x02];
        let packet = PrivacyPacket::new_control(MessageType::Ack, payload);

        assert_eq!(packet.header.msg_type, MessageType::Ack);
        assert!(packet.onion_header.is_none());

        let bytes = packet.to_bytes();
        let parsed = PrivacyPacket::from_bytes(&bytes).unwrap();
        assert!(parsed.onion_header.is_none());
    }

    #[test]
    fn test_size_classes_coverage() {
        // Teste alle Size-Classes
        for (i, &size) in SIZE_CLASSES.iter().enumerate() {
            let payload = vec![0xAB; size as usize - 10];
            let (class, padded) = quantize_to_size_class(&payload);

            assert_eq!(class, i as u8);
            assert_eq!(padded.len(), size as usize);
        }
    }

    #[test]
    fn test_message_type_categories() {
        assert!(MessageType::Onion.is_data());
        assert!(MessageType::Cover.is_data());
        assert!(!MessageType::Ack.is_data());

        assert!(MessageType::CircuitSetup.is_control());
        assert!(MessageType::Keepalive.is_control());
        assert!(!MessageType::Onion.is_control());
    }

    #[test]
    fn test_expected_payload_size() {
        let header = PacketHeader::onion(0, false);
        assert_eq!(header.expected_payload_size(), 256);

        let header = PacketHeader::onion(4, false);
        assert_eq!(header.expected_payload_size(), 4096);

        let header = PacketHeader::onion(7, false);
        assert_eq!(header.expected_payload_size(), 32768);
    }
}
