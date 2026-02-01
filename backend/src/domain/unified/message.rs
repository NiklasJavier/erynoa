//! # P2P Message-Typen
//!
//! Vereinheitlichte Message-Strukturen für die P2P-Kommunikation.
//!
//! ## Protokolle
//!
//! - **Gossipsub**: Realm-Events, Trust-Attestationen
//! - **Kademlia**: DHT-Lookups, Provider-Records
//! - **Request-Response**: Sync-Anfragen, Saga-Intents
//! - **AutoNat**: NAT-Traversal
//! - **Identify**: Peer-Identifikation
//! - **Ping**: Liveness-Checks
//!
//! ## Axiom-Referenz
//!
//! - **Κ9**: Kausale Event-Propagation
//! - **Κ10**: Finality-Attestationen
//! - **Κ23**: Realm-Gateway-Messages

use super::event::Signature64;
use super::primitives::{TemporalCoord, UniversalId};
use serde::{Deserialize, Serialize};

/// P2P-Protokoll-Typ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum P2PProtocol {
    /// Gossipsub PubSub
    Gossipsub,
    /// Kademlia DHT
    Kademlia,
    /// Request-Response
    RequestResponse,
    /// AutoNAT (NAT-Traversal)
    AutoNat,
    /// Identify (Peer-Identifikation)
    Identify,
    /// Ping (Liveness)
    Ping,
    /// mDNS (LAN-Discovery)
    Mdns,
    /// Erynoa Sync-Protokoll
    ErynoaSync,
    /// Future-Slot
    #[serde(other)]
    Unknown,
}

impl P2PProtocol {
    /// Protocol-String für libp2p
    pub fn protocol_string(&self) -> &'static str {
        match self {
            Self::Gossipsub => "/meshsub/1.1.0",
            Self::Kademlia => "/erynoa/kad/1.0.0",
            Self::RequestResponse => "/erynoa/sync/1.0.0",
            Self::AutoNat => "/libp2p/autonat/1.0.0",
            Self::Identify => "/erynoa/id/1.0.0",
            Self::Ping => "/ipfs/ping/1.0.0",
            Self::Mdns => "mdns",
            Self::ErynoaSync => "/erynoa/sync/1.0.0",
            Self::Unknown => "unknown",
        }
    }
}

/// Vereinheitlichte P2P-Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PMessage {
    /// Eindeutige Message-ID
    pub id: UniversalId,

    /// Verwendetes Protokoll
    pub protocol: P2PProtocol,

    /// Sender-ID (DID-basiert)
    pub sender: UniversalId,

    /// Optionaler Empfänger (None = Broadcast)
    pub recipient: Option<UniversalId>,

    /// Message-Payload
    pub payload: MessagePayload,

    /// Zeitstempel
    pub timestamp: TemporalCoord,

    /// TTL (Time-To-Live) in Hops
    pub ttl: u8,

    /// Priorität (0 = niedrig, 255 = höchste)
    pub priority: u8,
}

impl P2PMessage {
    /// Erstelle neue Message
    pub fn new(
        protocol: P2PProtocol,
        sender: UniversalId,
        payload: MessagePayload,
        timestamp: TemporalCoord,
    ) -> Self {
        // Generiere Message-ID basierend auf Inhalt
        let id = UniversalId::new(
            UniversalId::TAG_MESSAGE,
            1,
            &[sender.as_bytes().as_slice(), &timestamp.to_bytes()].concat(),
        );
        Self {
            id,
            protocol,
            sender,
            recipient: None,
            payload,
            timestamp,
            ttl: 7,       // Default: 7 Hops
            priority: 128, // Default: mittlere Priorität
        }
    }

    /// Mit spezifischem Empfänger (Unicast)
    pub fn with_recipient(mut self, recipient: UniversalId) -> Self {
        self.recipient = Some(recipient);
        self
    }

    /// Mit TTL
    pub fn with_ttl(mut self, ttl: u8) -> Self {
        self.ttl = ttl;
        self
    }

    /// Mit Priorität
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Prüfe ob Message abgelaufen ist (TTL = 0)
    pub fn is_expired(&self) -> bool {
        self.ttl == 0
    }

    /// Decrementiere TTL für Weiterleitung
    pub fn decrement_ttl(&mut self) -> bool {
        if self.ttl > 0 {
            self.ttl -= 1;
            true
        } else {
            false
        }
    }

    /// Ist Broadcast-Message?
    pub fn is_broadcast(&self) -> bool {
        self.recipient.is_none()
    }
}

/// Message-Payload-Typen
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessagePayload {
    /// Event-Propagation (Gossipsub)
    Event(EventMessage),

    /// Trust-Attestation (Gossipsub)
    Attestation(AttestationMessage),

    /// Sync-Request (Request-Response)
    SyncRequest(SyncRequestMessage),

    /// Sync-Response (Request-Response)
    SyncResponse(SyncResponseMessage),

    /// DHT-Record (Kademlia)
    DhtRecord(DhtRecordMessage),

    /// Peer-Info (Identify)
    PeerInfo(PeerInfoMessage),

    /// Realm-Join (Gateway)
    RealmJoin(RealmJoinMessage),

    /// Saga-Intent (Cross-Peer)
    SagaIntent(SagaIntentMessage),

    /// Ping
    Ping(PingMessage),

    /// Pong (Ping-Response)
    Pong(PongMessage),

    /// Raw-Bytes (Fallback)
    Raw(Vec<u8>),
}

/// Event-Message (Κ9)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage {
    /// Event-ID
    pub event_id: UniversalId,

    /// Realm-ID
    pub realm_id: UniversalId,

    /// Serialisiertes Event
    pub event_data: Vec<u8>,

    /// Kausal-Parent-IDs
    pub parents: Vec<UniversalId>,

    /// Finality-Level
    pub finality_level: u8,
}

/// Attestation-Message (Κ10)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationMessage {
    /// Attestierte Event-ID
    pub event_id: UniversalId,

    /// Attester-DID
    pub attester: UniversalId,

    /// Trust-Score des Attesters
    pub attester_trust: f32,

    /// Signatur
    pub signature: Signature64,
}

/// Sync-Request-Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequestMessage {
    /// Realm-ID
    pub realm_id: UniversalId,

    /// Sync-Typ
    pub sync_type: SyncType,

    /// Seit welchem Zeitpunkt
    pub since: TemporalCoord,

    /// Maximale Anzahl Events
    pub limit: u32,

    /// Bekannte Event-IDs (für Delta-Sync)
    pub known_events: Vec<UniversalId>,
}

/// Sync-Typ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncType {
    /// Vollständiger Sync
    Full,
    /// Delta-Sync (nur neue Events)
    Delta,
    /// Trust-Sync (Trust-Records)
    Trust,
    /// Finality-Sync (Attestationen)
    Finality,
}

/// Sync-Response-Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponseMessage {
    /// Anfrage-ID (für Korrelation)
    pub request_id: UniversalId,

    /// Events
    pub events: Vec<EventMessage>,

    /// Attestationen
    pub attestations: Vec<AttestationMessage>,

    /// Weitere Events verfügbar?
    pub has_more: bool,

    /// Nächster Cursor (für Pagination)
    pub next_cursor: Option<TemporalCoord>,
}

/// DHT-Record-Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtRecordMessage {
    /// Record-Key
    pub key: Vec<u8>,

    /// Record-Value
    pub value: Vec<u8>,

    /// Publisher-ID
    pub publisher: UniversalId,

    /// Gültig bis (Unix-Timestamp)
    pub expires_at: u64,
}

/// Peer-Info-Message (Identify)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfoMessage {
    /// Peer-DID
    pub peer_id: UniversalId,

    /// Agent-Version
    pub agent_version: String,

    /// Supported-Protocols
    pub protocols: Vec<String>,

    /// Listen-Addresses (Multiaddrs als Strings)
    pub listen_addrs: Vec<String>,

    /// Observed-Address (wie wir von anderen gesehen werden)
    pub observed_addr: Option<String>,

    /// Realms, in denen der Peer aktiv ist
    pub active_realms: Vec<UniversalId>,
}

/// Realm-Join-Message (Κ23 Gateway)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmJoinMessage {
    /// Realm-ID
    pub realm_id: UniversalId,

    /// Beitretender Peer
    pub peer_id: UniversalId,

    /// Join-Proof (z.B. Invite-Signatur)
    pub proof: Vec<u8>,

    /// Initiale Trust-Claims
    pub trust_claims: Vec<TrustClaimMessage>,
}

/// Trust-Claim für Realm-Join
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustClaimMessage {
    /// Vouching-Peer
    pub voucher: UniversalId,

    /// Trust-Score
    pub trust_score: f32,

    /// Signatur des Vouchers
    pub signature: Signature64,
}

/// Saga-Intent-Message (Cross-Peer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaIntentMessage {
    /// Saga-ID
    pub saga_id: UniversalId,

    /// Intent-Typ
    pub intent_type: String,

    /// Serialisierter Intent
    pub intent_data: Vec<u8>,

    /// Deadline (Unix-Timestamp)
    pub deadline: u64,

    /// Budget (in Mana)
    pub budget: u64,
}

/// Ping-Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingMessage {
    /// Nonce für Round-Trip-Messung
    pub nonce: u64,

    /// Sender-Timestamp (für RTT)
    pub sent_at_ms: u64,
}

/// Pong-Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongMessage {
    /// Nonce (Echo vom Ping)
    pub nonce: u64,

    /// Original-Ping-Timestamp
    pub ping_sent_at_ms: u64,

    /// Receiver-Timestamp
    pub received_at_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: Test-ID generieren
    fn test_id(tag: u16, suffix: &str) -> UniversalId {
        UniversalId::new(tag, 1, suffix.as_bytes())
    }

    #[test]
    fn test_p2p_message_creation() {
        let sender = test_id(UniversalId::TAG_DID, "sender");
        let timestamp = TemporalCoord::now(42, &sender);
        let payload = MessagePayload::Ping(PingMessage {
            nonce: 12345,
            sent_at_ms: 1700000000000,
        });

        let msg = P2PMessage::new(P2PProtocol::Ping, sender, payload, timestamp);

        assert_eq!(msg.protocol, P2PProtocol::Ping);
        assert_eq!(msg.ttl, 7);
        assert!(msg.is_broadcast());
    }

    #[test]
    fn test_message_with_recipient() {
        let sender = test_id(UniversalId::TAG_DID, "sender");
        let recipient = test_id(UniversalId::TAG_DID, "recipient");
        let timestamp = TemporalCoord::now(42, &sender);

        let msg = P2PMessage::new(
            P2PProtocol::RequestResponse,
            sender,
            MessagePayload::Raw(vec![1, 2, 3]),
            timestamp,
        )
        .with_recipient(recipient);

        assert!(!msg.is_broadcast());
        assert_eq!(msg.recipient, Some(recipient));
    }

    #[test]
    fn test_ttl_decrement() {
        let sender = test_id(UniversalId::TAG_DID, "sender");
        let timestamp = TemporalCoord::now(42, &sender);

        let mut msg = P2PMessage::new(
            P2PProtocol::Gossipsub,
            sender,
            MessagePayload::Raw(vec![]),
            timestamp,
        )
        .with_ttl(2);

        assert!(!msg.is_expired());
        assert!(msg.decrement_ttl());
        assert_eq!(msg.ttl, 1);
        assert!(msg.decrement_ttl());
        assert_eq!(msg.ttl, 0);
        assert!(msg.is_expired());
        assert!(!msg.decrement_ttl()); // Kann nicht unter 0
    }

    #[test]
    fn test_protocol_strings() {
        assert_eq!(P2PProtocol::Gossipsub.protocol_string(), "/meshsub/1.1.0");
        assert_eq!(P2PProtocol::Kademlia.protocol_string(), "/erynoa/kad/1.0.0");
        assert_eq!(P2PProtocol::Ping.protocol_string(), "/ipfs/ping/1.0.0");
    }

    #[test]
    fn test_event_message_serialization() {
        let event_msg = EventMessage {
            event_id: test_id(UniversalId::TAG_EVENT, "event1"),
            realm_id: test_id(UniversalId::TAG_REALM, "realm1"),
            event_data: vec![1, 2, 3, 4],
            parents: vec![test_id(UniversalId::TAG_EVENT, "parent1")],
            finality_level: 2,
        };

        let json = serde_json::to_string(&event_msg).unwrap();
        let parsed: EventMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.event_id, event_msg.event_id);
        assert_eq!(parsed.finality_level, 2);
    }

    #[test]
    fn test_sync_request_message() {
        let sync_req = SyncRequestMessage {
            realm_id: test_id(UniversalId::TAG_REALM, "realm1"),
            sync_type: SyncType::Delta,
            since: TemporalCoord::GENESIS,
            limit: 100,
            known_events: vec![],
        };

        assert_eq!(sync_req.sync_type, SyncType::Delta);
        assert_eq!(sync_req.limit, 100);
    }

    #[test]
    fn test_message_payload_tagging() {
        let ping = MessagePayload::Ping(PingMessage {
            nonce: 1,
            sent_at_ms: 0,
        });

        let json = serde_json::to_string(&ping).unwrap();
        assert!(json.contains("\"type\":\"ping\""));

        let attestation = MessagePayload::Attestation(AttestationMessage {
            event_id: test_id(UniversalId::TAG_EVENT, "event1"),
            attester: test_id(UniversalId::TAG_DID, "attester"),
            attester_trust: 0.85,
            signature: Signature64::default(),
        });

        let json = serde_json::to_string(&attestation).unwrap();
        assert!(json.contains("\"type\":\"attestation\""));
    }
}
