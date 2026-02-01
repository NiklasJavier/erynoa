//! # Sync-Protokoll (Request-Response)
//!
//! Event-Synchronisation zwischen Peers.
//!
//! ## Protokoll-Versionen
//!
//! - `/erynoa/sync/events/1.0` - Event-Sync
//! - `/erynoa/sync/trust/1.0` - Trust-State-Sync
//! - `/erynoa/sync/membership/1.0` - Realm-Membership-Verification

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
        /// Realm-ID
        realm_id: String,
        /// Letzter bekannter Event-Hash
        after_hash: Option<String>,
        /// Maximum Anzahl Events
        limit: usize,
    },

    /// Event-Sync: Fordere spezifische Events an
    GetEventsByIds {
        /// Realm-ID
        realm_id: String,
        /// Event-IDs
        event_ids: Vec<String>,
    },

    /// Trust-Sync: Fordere Trust-State für DID an
    GetTrustState {
        /// Subject-DID
        subject_did: String,
    },

    /// Membership: Verifiziere Realm-Membership
    VerifyMembership {
        /// Realm-ID
        realm_id: String,
        /// DID des zu prüfenden Peers
        did: String,
    },

    /// Membership: Fordere Membership-Proof an
    GetMembershipProof {
        /// Realm-ID
        realm_id: String,
    },

    /// Ping für Latenz-Messung
    Ping {
        /// Timestamp
        timestamp: u64,
    },
}

/// Sync-Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncResponse {
    /// Events-Antwort
    Events {
        /// Realm-ID
        realm_id: String,
        /// Events (serialisiert)
        events: Vec<SerializedEvent>,
        /// Ob es mehr Events gibt
        has_more: bool,
        /// Nächster Cursor (für Pagination)
        next_cursor: Option<String>,
    },

    /// Trust-State-Antwort
    TrustState {
        /// Subject-DID
        subject_did: String,
        /// Trust-R
        trust_r: f64,
        /// Trust-Ω
        trust_omega: f64,
        /// Letzte Attestation
        last_attestation: Option<u64>,
    },

    /// Membership-Verification-Antwort
    MembershipVerified {
        /// Realm-ID
        realm_id: String,
        /// DID
        did: String,
        /// Ist Mitglied
        is_member: bool,
        /// Membership-Level (optional)
        level: Option<String>,
    },

    /// Membership-Proof
    MembershipProof {
        /// Realm-ID
        realm_id: String,
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
    /// Timestamp
    pub timestamp: u64,
    /// Creator-DID
    pub creator: String,
    /// Signatur
    pub signature: Vec<u8>,
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

    #[test]
    fn test_request_serialization() {
        let request = SyncRequest::GetEventsAfter {
            realm_id: "test-realm".to_string(),
            after_hash: Some("abc123".to_string()),
            limit: 100,
        };

        let bytes = request.to_bytes().unwrap();
        let decoded = SyncRequest::from_bytes(&bytes).unwrap();

        match decoded {
            SyncRequest::GetEventsAfter {
                realm_id, limit, ..
            } => {
                assert_eq!(realm_id, "test-realm");
                assert_eq!(limit, 100);
            }
            _ => panic!("Wrong request type"),
        }
    }

    #[test]
    fn test_response_serialization() {
        let response = SyncResponse::Events {
            realm_id: "test-realm".to_string(),
            events: vec![],
            has_more: false,
            next_cursor: None,
        };

        let bytes = response.to_bytes().unwrap();
        let decoded = SyncResponse::from_bytes(&bytes).unwrap();

        match decoded {
            SyncResponse::Events { realm_id, .. } => {
                assert_eq!(realm_id, "test-realm");
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

        let ping = SyncRequest::Ping { timestamp };
        let bytes = ping.to_bytes().unwrap();

        match SyncRequest::from_bytes(&bytes).unwrap() {
            SyncRequest::Ping { timestamp: ts } => {
                assert_eq!(ts, timestamp);
            }
            _ => panic!("Wrong request type"),
        }
    }
}
