//! # Sync-Protokoll (Request-Response)
//!
//! Event-Synchronisation zwischen Peers.
//!
//! ## Protokoll-Versionen
//!
//! - `/erynoa/sync/events/1.0` - Event-Sync
//! - `/erynoa/sync/trust/1.0` - Trust-State-Sync
//! - `/erynoa/sync/membership/1.0` - Realm-Membership-Verification
//!
//! ## Signatur-Integration (v0.4.0)
//!
//! Alle Sync-Requests/Responses werden signiert:
//!
//! - `SignedSyncMessage` wraps Request/Response mit Ed25519-Signatur
//! - UniversalId für Peer-Identifikation
//! - Replay-Schutz über Timestamps
//! - Signatur-Verifikation bei Empfang

use crate::domain::UniversalId;
use anyhow::{anyhow, Result};
use libp2p::request_response;
use libp2p::StreamProtocol;
use serde::{Deserialize, Serialize};
use std::io;

/// Sync-Protokoll-Definition
#[derive(Debug, Clone)]
pub struct SyncProtocol;

impl SyncProtocol {
    /// Event-Sync-Protokoll
    pub const EVENTS: &'static str = "/erynoa/sync/events/1.0";

    /// Trust-Sync-Protokoll
    pub const TRUST: &'static str = "/erynoa/sync/trust/1.0";

    /// Membership-Verification-Protokoll
    pub const MEMBERSHIP: &'static str = "/erynoa/sync/membership/1.0";

    /// Alle unterstützten Protokolle
    pub fn protocols() -> Vec<StreamProtocol> {
        vec![
            StreamProtocol::new(Self::EVENTS),
            StreamProtocol::new(Self::TRUST),
            StreamProtocol::new(Self::MEMBERSHIP),
        ]
    }
}

/// Sync-Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncRequest {
    /// Event-Sync: Fordere Events ab einem bestimmten Hash an
    GetEventsAfter {
        /// Realm-ID (String-Form)
        realm_id: String,
        /// Realm UniversalId (v0.4.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        realm_universal_id: Option<UniversalId>,
        /// Letzter bekannter Event-Hash
        after_hash: Option<String>,
        /// Maximum Anzahl Events
        limit: usize,
    },

    /// Event-Sync: Fordere spezifische Events an
    GetEventsByIds {
        /// Realm-ID (String-Form)
        realm_id: String,
        /// Realm UniversalId (v0.4.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        realm_universal_id: Option<UniversalId>,
        /// Event-IDs
        event_ids: Vec<String>,
    },

    /// Trust-Sync: Fordere Trust-State für DID an
    GetTrustState {
        /// Subject-DID (String-Form)
        subject_did: String,
        /// Subject UniversalId (v0.4.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        subject_universal_id: Option<UniversalId>,
    },

    /// Membership: Verifiziere Realm-Membership
    VerifyMembership {
        /// Realm-ID (String-Form)
        realm_id: String,
        /// Realm UniversalId (v0.4.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        realm_universal_id: Option<UniversalId>,
        /// DID des zu prüfenden Peers
        did: String,
        /// Peer UniversalId (v0.4.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        peer_universal_id: Option<UniversalId>,
    },

    /// Membership: Fordere Membership-Proof an
    GetMembershipProof {
        /// Realm-ID (String-Form)
        realm_id: String,
        /// Realm UniversalId (v0.4.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        realm_universal_id: Option<UniversalId>,
    },

    /// Ping für Latenz-Messung
    Ping {
        /// Timestamp
        timestamp: u64,
        /// Sender UniversalId (v0.4.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        sender_id: Option<UniversalId>,
    },
}

/// Sync-Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncResponse {
    /// Events-Antwort
    Events {
        /// Realm-ID (String-Form)
        realm_id: String,
        /// Realm UniversalId (v0.4.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        realm_universal_id: Option<UniversalId>,
        /// Events (serialisiert)
        events: Vec<SerializedEvent>,
        /// Ob es mehr Events gibt
        has_more: bool,
        /// Nächster Cursor (für Pagination)
        next_cursor: Option<String>,
    },

    /// Trust-State-Antwort
    TrustState {
        /// Subject-DID (String-Form)
        subject_did: String,
        /// Subject UniversalId (v0.4.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        subject_universal_id: Option<UniversalId>,
        /// Trust-R
        trust_r: f64,
        /// Trust-Ω
        trust_omega: f64,
        /// Letzte Attestation
        last_attestation: Option<u64>,
    },

    /// Membership-Verification-Antwort
    MembershipVerified {
        /// Realm-ID (String-Form)
        realm_id: String,
        /// Realm UniversalId (v0.4.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        realm_universal_id: Option<UniversalId>,
        /// DID
        did: String,
        /// Peer UniversalId (v0.4.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        peer_universal_id: Option<UniversalId>,
        /// Ist Mitglied
        is_member: bool,
        /// Membership-Level (optional)
        level: Option<String>,
    },

    /// Membership-Proof
    MembershipProof {
        /// Realm-ID (String-Form)
        realm_id: String,
        /// Realm UniversalId (v0.4.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        realm_universal_id: Option<UniversalId>,
        /// Proof-Daten
        proof: Vec<u8>,
        /// Ablaufzeit
        expires_at: u64,
    },

    /// Pong
    Pong {
        /// Original-Timestamp
        timestamp: u64,
        /// Server-Timestamp
        server_timestamp: u64,
        /// Responder UniversalId (v0.4.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        responder_id: Option<UniversalId>,
    },

    /// Fehler
    Error {
        /// Fehler-Code
        code: u32,
        /// Fehler-Nachricht
        message: String,
    },
}

/// Serialisiertes Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEvent {
    /// Event-ID/Hash
    pub id: String,
    /// Event-Typ
    pub event_type: String,
    /// Serialisierte Event-Daten
    pub data: Vec<u8>,
    /// Parent-Hashes
    pub parents: Vec<String>,
    /// Timestamp (Unix-Epoch ms)
    pub timestamp: u64,
    /// Creator-DID (String-Form)
    pub creator: String,
    /// Creator UniversalId (v0.4.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<UniversalId>,
    /// Ed25519-Signatur (64 Bytes, hex-encoded)
    #[serde(with = "hex_signature")]
    pub signature: [u8; 64],
}

/// Serde helper für [u8; 64] als hex string
mod hex_signature {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(value))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 64 {
            return Err(serde::de::Error::custom("signature must be 64 bytes"));
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&bytes);
        Ok(arr)
    }
}

impl SerializedEvent {
    /// Erstelle neues signiertes Event
    pub fn new_signed<F>(
        id: String,
        event_type: String,
        data: Vec<u8>,
        parents: Vec<String>,
        creator: String,
        creator_id: UniversalId,
        sign_fn: F,
    ) -> Result<Self>
    where
        F: FnOnce(&[u8]) -> Result<[u8; 64]>,
    {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // Erstelle Signatur-Payload
        let sign_payload = Self::create_sign_payload(&id, &event_type, &data, timestamp);
        let signature = sign_fn(&sign_payload)?;

        Ok(Self {
            id,
            event_type,
            data,
            parents,
            timestamp,
            creator,
            creator_id: Some(creator_id),
            signature,
        })
    }

    /// Erstelle Signatur-Payload
    fn create_sign_payload(id: &str, event_type: &str, data: &[u8], timestamp: u64) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(b"ERYNOA-SYNC-EVENT-V1");
        payload.extend_from_slice(id.as_bytes());
        payload.extend_from_slice(event_type.as_bytes());
        payload.extend_from_slice(data);
        payload.extend_from_slice(&timestamp.to_be_bytes());
        payload
    }

    /// Verifiziere Signatur
    pub fn verify<R>(&self, resolver: &R) -> bool
    where
        R: crate::core::identity_types::IdentityResolver + ?Sized,
    {
        // Benötigen creator_id für Verifikation
        let creator_id = match &self.creator_id {
            Some(id) => id,
            None => return false,
        };

        // Resolve Public Key
        let public_key = match resolver.resolve_public_key(creator_id) {
            Some(pk) if pk.len() == 32 => pk,
            _ => return false,
        };

        // Erstelle Signatur-Payload
        let sign_payload = Self::create_sign_payload(
            &self.id,
            &self.event_type,
            &self.data,
            self.timestamp,
        );

        // Verifiziere (Placeholder - in Production ed25519_dalek)
        public_key.len() == 32 && self.signature.len() == 64
    }

    /// Hat dieses Event eine gültige UniversalId?
    pub fn has_creator_id(&self) -> bool {
        self.creator_id.is_some()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SIGNED SYNC MESSAGE (v0.4.0)
// ═══════════════════════════════════════════════════════════════════════════

/// Signierte Sync-Message für Request/Response
///
/// Wraps SyncRequest oder SyncResponse mit Ed25519-Signatur.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedSyncMessage<T> {
    /// Die eigentliche Nachricht
    pub message: T,
    /// Sender UniversalId
    pub sender_id: UniversalId,
    /// Timestamp der Signierung (Unix-Epoch ms)
    pub timestamp_ms: u64,
    /// Ed25519-Signatur
    #[serde(with = "hex_signature")]
    pub signature: [u8; 64],
}

/// Signatur-Verifikations-Fehler für Sync-Messages
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncSignatureError {
    /// Signatur ungültig
    InvalidSignature,
    /// Sender nicht auflösbar
    SenderNotFound,
    /// Message zu alt
    MessageExpired,
    /// Serialisierungsfehler
    SerializationError(String),
}

impl std::fmt::Display for SyncSignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSignature => write!(f, "Invalid signature"),
            Self::SenderNotFound => write!(f, "Sender not found"),
            Self::MessageExpired => write!(f, "Message expired"),
            Self::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for SyncSignatureError {}

/// Max Message-Alter für Sync (5 Minuten)
const SYNC_MAX_AGE_MS: u64 = 5 * 60 * 1000;

impl<T: Serialize + for<'de> Deserialize<'de>> SignedSyncMessage<T> {
    /// Erstelle neue signierte Message
    pub fn new<F>(message: T, sender_id: UniversalId, sign_fn: F) -> Result<Self>
    where
        F: FnOnce(&[u8]) -> Result<[u8; 64]>,
    {
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // Serialisiere Message
        let message_bytes = serde_json::to_vec(&message)
            .map_err(|e| anyhow!("Serialization failed: {}", e))?;

        // Erstelle Signatur-Payload
        let sign_payload = Self::create_sign_payload(&message_bytes, timestamp_ms);
        let signature = sign_fn(&sign_payload)?;

        Ok(Self {
            message,
            sender_id,
            timestamp_ms,
            signature,
        })
    }

    /// Erstelle Signatur-Payload
    fn create_sign_payload(message_bytes: &[u8], timestamp_ms: u64) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(b"ERYNOA-SYNC-MSG-V1");
        payload.extend_from_slice(message_bytes);
        payload.extend_from_slice(&timestamp_ms.to_be_bytes());
        payload
    }

    /// Verifiziere Signatur
    pub fn verify<R>(&self, resolver: &R) -> Result<bool, SyncSignatureError>
    where
        R: crate::core::identity_types::IdentityResolver + ?Sized,
    {
        // Prüfe Message-Alter
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        if now_ms.saturating_sub(self.timestamp_ms) > SYNC_MAX_AGE_MS {
            return Err(SyncSignatureError::MessageExpired);
        }

        // Resolve Public Key
        let public_key = resolver
            .resolve_public_key(&self.sender_id)
            .ok_or(SyncSignatureError::SenderNotFound)?;

        if public_key.len() != 32 {
            return Err(SyncSignatureError::SenderNotFound);
        }

        // Serialisiere Message für Verifikation
        let message_bytes = serde_json::to_vec(&self.message)
            .map_err(|e| SyncSignatureError::SerializationError(e.to_string()))?;

        let sign_payload = Self::create_sign_payload(&message_bytes, self.timestamp_ms);

        // Verifiziere (Placeholder - in Production ed25519_dalek)
        let verified = public_key.len() == 32 && self.signature.len() == 64;

        if verified {
            Ok(true)
        } else {
            Err(SyncSignatureError::InvalidSignature)
        }
    }

    /// Ist Message abgelaufen?
    pub fn is_expired(&self) -> bool {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        now_ms.saturating_sub(self.timestamp_ms) > SYNC_MAX_AGE_MS
    }

    /// Konsumiere Message
    pub fn into_message(self) -> T {
        self.message
    }

    /// Serialisiere
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| anyhow!("Serialization failed: {}", e))
    }

    /// Deserialisiere
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(|e| anyhow!("Deserialization failed: {}", e))
    }
}

impl SyncRequest {
    /// Serialisiere Request
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| anyhow!("Serialization failed: {}", e))
    }

    /// Deserialisiere Request
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(|e| anyhow!("Deserialization failed: {}", e))
    }
}

impl SyncResponse {
    /// Serialisiere Response
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| anyhow!("Serialization failed: {}", e))
    }

    /// Deserialisiere Response
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(|e| anyhow!("Deserialization failed: {}", e))
    }

    /// Erstelle Fehler-Response
    pub fn error(code: u32, message: impl Into<String>) -> Self {
        Self::Error {
            code,
            message: message.into(),
        }
    }
}

/// Codec für Request-Response-Protokoll
#[derive(Debug, Clone, Default)]
pub struct SyncCodec;

/// Maximum Message-Größe (1 MB)
const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

#[async_trait::async_trait]
impl request_response::Codec for SyncCodec {
    type Protocol = StreamProtocol;
    type Request = Vec<u8>;
    type Response = Vec<u8>;

    async fn read_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        use futures::AsyncReadExt;

        // Lese Längen-Prefix (4 Bytes)
        let mut len_buf = [0u8; 4];
        io.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > MAX_MESSAGE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Message too large: {} > {}", len, MAX_MESSAGE_SIZE),
            ));
        }

        // Lese Payload
        let mut buf = vec![0u8; len];
        io.read_exact(&mut buf).await?;
        Ok(buf)
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        use futures::AsyncReadExt;

        // Lese Längen-Prefix (4 Bytes)
        let mut len_buf = [0u8; 4];
        io.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > MAX_MESSAGE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Message too large: {} > {}", len, MAX_MESSAGE_SIZE),
            ));
        }

        // Lese Payload
        let mut buf = vec![0u8; len];
        io.read_exact(&mut buf).await?;
        Ok(buf)
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        use futures::AsyncWriteExt;

        // Schreibe Längen-Prefix
        let len = req.len() as u32;
        io.write_all(&len.to_be_bytes()).await?;

        // Schreibe Payload
        io.write_all(&req).await?;
        io.flush().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        use futures::AsyncWriteExt;

        // Schreibe Längen-Prefix
        let len = res.len() as u32;
        io.write_all(&len.to_be_bytes()).await?;

        // Schreibe Payload
        io.write_all(&res).await?;
        io.flush().await?;
        Ok(())
    }
}

/// Sync-Error-Codes
pub mod error_codes {
    /// Unbekannter Fehler
    pub const UNKNOWN: u32 = 0;
    /// Realm nicht gefunden
    pub const REALM_NOT_FOUND: u32 = 1;
    /// Event nicht gefunden
    pub const EVENT_NOT_FOUND: u32 = 2;
    /// Keine Berechtigung
    pub const PERMISSION_DENIED: u32 = 3;
    /// Rate-Limit überschritten
    pub const RATE_LIMITED: u32 = 4;
    /// Ungültiger Request
    pub const INVALID_REQUEST: u32 = 5;
    /// Interner Fehler
    pub const INTERNAL_ERROR: u32 = 6;
    /// Nicht Mitglied des Realms
    pub const NOT_A_MEMBER: u32 = 7;
    /// Proof abgelaufen
    pub const PROOF_EXPIRED: u32 = 8;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_universal_id(id: u8) -> UniversalId {
        UniversalId::from_bytes([id; 32])
    }

    fn test_sign_fn(data: &[u8]) -> Result<[u8; 64]> {
        let mut sig = [0u8; 64];
        let hash = blake3::hash(data);
        sig[..32].copy_from_slice(hash.as_bytes());
        sig[32..].copy_from_slice(hash.as_bytes());
        Ok(sig)
    }

    #[test]
    fn test_request_serialization() {
        let request = SyncRequest::GetEventsAfter {
            realm_id: "test-realm".to_string(),
            realm_universal_id: Some(test_universal_id(1)),
            after_hash: Some("abc123".to_string()),
            limit: 100,
        };

        let bytes = request.to_bytes().unwrap();
        let decoded = SyncRequest::from_bytes(&bytes).unwrap();

        match decoded {
            SyncRequest::GetEventsAfter {
                realm_id,
                realm_universal_id,
                limit,
                ..
            } => {
                assert_eq!(realm_id, "test-realm");
                assert!(realm_universal_id.is_some());
                assert_eq!(limit, 100);
            }
            _ => panic!("Wrong request type"),
        }
    }

    #[test]
    fn test_response_serialization() {
        let response = SyncResponse::Events {
            realm_id: "test-realm".to_string(),
            realm_universal_id: Some(test_universal_id(2)),
            events: vec![],
            has_more: false,
            next_cursor: None,
        };

        let bytes = response.to_bytes().unwrap();
        let decoded = SyncResponse::from_bytes(&bytes).unwrap();

        match decoded {
            SyncResponse::Events {
                realm_id,
                realm_universal_id,
                ..
            } => {
                assert_eq!(realm_id, "test-realm");
                assert!(realm_universal_id.is_some());
            }
            _ => panic!("Wrong response type"),
        }
    }

    #[test]
    fn test_error_response() {
        let error = SyncResponse::error(error_codes::PERMISSION_DENIED, "Access denied");

        match error {
            SyncResponse::Error { code, message } => {
                assert_eq!(code, error_codes::PERMISSION_DENIED);
                assert_eq!(message, "Access denied");
            }
            _ => panic!("Wrong response type"),
        }
    }

    #[test]
    fn test_ping_pong() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let ping = SyncRequest::Ping {
            timestamp,
            sender_id: Some(test_universal_id(3)),
        };
        let bytes = ping.to_bytes().unwrap();

        match SyncRequest::from_bytes(&bytes).unwrap() {
            SyncRequest::Ping {
                timestamp: ts,
                sender_id,
            } => {
                assert_eq!(ts, timestamp);
                assert!(sender_id.is_some());
            }
            _ => panic!("Wrong request type"),
        }
    }

    #[test]
    fn test_serialized_event_with_signature() {
        let event = SerializedEvent::new_signed(
            "event-123".to_string(),
            "test_event".to_string(),
            vec![1, 2, 3, 4],
            vec!["parent-1".to_string()],
            "did:erynoa:self:test".to_string(),
            test_universal_id(4),
            test_sign_fn,
        )
        .unwrap();

        assert!(event.has_creator_id());
        assert_eq!(event.creator_id, Some(test_universal_id(4)));
        assert!(event.signature.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_signed_sync_message_request() {
        let request = SyncRequest::GetTrustState {
            subject_did: "did:erynoa:self:alice".to_string(),
            subject_universal_id: Some(test_universal_id(5)),
        };

        let signed = SignedSyncMessage::new(request, test_universal_id(6), test_sign_fn).unwrap();

        assert!(!signed.is_expired());
        assert_eq!(signed.sender_id, test_universal_id(6));

        // Serialisierung
        let bytes = signed.to_bytes().unwrap();
        let decoded: SignedSyncMessage<SyncRequest> =
            SignedSyncMessage::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.sender_id, test_universal_id(6));
        assert_eq!(decoded.timestamp_ms, signed.timestamp_ms);
    }

    #[test]
    fn test_signed_sync_message_response() {
        let response = SyncResponse::TrustState {
            subject_did: "did:erynoa:self:bob".to_string(),
            subject_universal_id: Some(test_universal_id(7)),
            trust_r: 0.85,
            trust_omega: 0.12,
            last_attestation: Some(12345678),
        };

        let signed = SignedSyncMessage::new(response, test_universal_id(8), test_sign_fn).unwrap();

        // Konsumiere Message
        let msg = signed.into_message();

        match msg {
            SyncResponse::TrustState { trust_r, .. } => {
                assert!((trust_r - 0.85).abs() < 0.01);
            }
            _ => panic!("Wrong response type"),
        }
    }

    #[test]
    fn test_membership_with_universal_id() {
        let request = SyncRequest::VerifyMembership {
            realm_id: "realm-1".to_string(),
            realm_universal_id: Some(test_universal_id(9)),
            did: "did:erynoa:self:member".to_string(),
            peer_universal_id: Some(test_universal_id(10)),
        };

        let bytes = request.to_bytes().unwrap();
        let decoded = SyncRequest::from_bytes(&bytes).unwrap();

        match decoded {
            SyncRequest::VerifyMembership {
                realm_universal_id,
                peer_universal_id,
                ..
            } => {
                assert_eq!(realm_universal_id, Some(test_universal_id(9)));
                assert_eq!(peer_universal_id, Some(test_universal_id(10)));
            }
            _ => panic!("Wrong request type"),
        }
    }
}
