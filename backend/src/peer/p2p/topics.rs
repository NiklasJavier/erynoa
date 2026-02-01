//! # Realm-Topics für Gossipsub
//!
//! Topic-Management für realm-basiertes PubSub.
//!
//! ## Topic-Schema
//!
//! ```text
//! /erynoa/realm/{realm_id}/events/v1     - Event-Propagation
//! /erynoa/realm/{realm_id}/trust/v1      - Trust-Attestationen
//! /erynoa/realm/{realm_id}/sagas/v1      - Saga-Broadcasts
//! /erynoa/direct/{sender}/{receiver}     - Direct Messages
//! /erynoa/global/announcements/v1        - Netzwerk-Announcements
//! ```

use crate::domain::DID;
use anyhow::{anyhow, Result};
use libp2p::gossipsub::{IdentTopic, TopicHash};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Topic-Typen
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TopicType {
    /// Realm-Events
    RealmEvents,
    /// Realm-Trust-Attestationen
    RealmTrust,
    /// Realm-Sagas
    RealmSagas,
    /// Direct Messaging
    Direct,
    /// Global Announcements
    Global,
}

/// Realm-Topic
#[derive(Debug, Clone)]
pub struct RealmTopic {
    /// Topic-Typ
    pub topic_type: TopicType,

    /// Realm-ID (für Realm-Topics)
    pub realm_id: Option<String>,

    /// Sender-DID (für Direct-Topics)
    pub sender: Option<String>,

    /// Receiver-DID (für Direct-Topics)
    pub receiver: Option<String>,

    /// libp2p Topic
    topic: IdentTopic,
}

impl PartialEq for RealmTopic {
    fn eq(&self, other: &Self) -> bool {
        self.topic.hash() == other.topic.hash()
    }
}

impl Eq for RealmTopic {}

impl std::hash::Hash for RealmTopic {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.topic.hash().hash(state);
    }
}

impl RealmTopic {
    /// Erstelle Realm-Events-Topic
    pub fn realm_events(realm_id: &str) -> Self {
        let topic_str = format!("/erynoa/realm/{}/events/v1", realm_id);
        Self {
            topic_type: TopicType::RealmEvents,
            realm_id: Some(realm_id.to_string()),
            sender: None,
            receiver: None,
            topic: IdentTopic::new(topic_str),
        }
    }

    /// Erstelle Realm-Trust-Topic
    pub fn realm_trust(realm_id: &str) -> Self {
        let topic_str = format!("/erynoa/realm/{}/trust/v1", realm_id);
        Self {
            topic_type: TopicType::RealmTrust,
            realm_id: Some(realm_id.to_string()),
            sender: None,
            receiver: None,
            topic: IdentTopic::new(topic_str),
        }
    }

    /// Erstelle Realm-Sagas-Topic
    pub fn realm_sagas(realm_id: &str) -> Self {
        let topic_str = format!("/erynoa/realm/{}/sagas/v1", realm_id);
        Self {
            topic_type: TopicType::RealmSagas,
            realm_id: Some(realm_id.to_string()),
            sender: None,
            receiver: None,
            topic: IdentTopic::new(topic_str),
        }
    }

    /// Erstelle Direct-Message-Topic
    pub fn direct(sender: &DID, receiver: &DID) -> Self {
        let topic_str = format!("/erynoa/direct/{}/{}", sender.unique_id, receiver.unique_id);
        Self {
            topic_type: TopicType::Direct,
            realm_id: None,
            sender: Some(sender.to_uri()),
            receiver: Some(receiver.to_uri()),
            topic: IdentTopic::new(topic_str),
        }
    }

    /// Erstelle Global-Announcements-Topic
    pub fn global_announcements() -> Self {
        let topic_str = "/erynoa/global/announcements/v1".to_string();
        Self {
            topic_type: TopicType::Global,
            realm_id: None,
            sender: None,
            receiver: None,
            topic: IdentTopic::new(topic_str),
        }
    }

    /// Parse Topic aus String
    pub fn from_str(topic_str: &str) -> Result<Self> {
        let parts: Vec<&str> = topic_str.split('/').collect();

        if parts.len() < 3 || parts[1] != "erynoa" {
            return Err(anyhow!("Invalid topic format: {}", topic_str));
        }

        match parts[2] {
            "realm" if parts.len() >= 5 => {
                let realm_id = parts[3];
                match parts[4] {
                    "events" => Ok(Self::realm_events(realm_id)),
                    "trust" => Ok(Self::realm_trust(realm_id)),
                    "sagas" => Ok(Self::realm_sagas(realm_id)),
                    _ => Err(anyhow!("Unknown realm topic type: {}", parts[4])),
                }
            }
            "direct" if parts.len() >= 5 => {
                let sender = DID::new_self(parts[3]);
                let receiver = DID::new_self(parts[4]);
                Ok(Self::direct(&sender, &receiver))
            }
            "global" if parts.len() >= 4 && parts[3] == "announcements" => {
                Ok(Self::global_announcements())
            }
            _ => Err(anyhow!("Unknown topic category: {}", parts[2])),
        }
    }

    /// Erhalte libp2p IdentTopic
    pub fn ident_topic(&self) -> &IdentTopic {
        &self.topic
    }

    /// Erhalte TopicHash
    pub fn hash(&self) -> TopicHash {
        self.topic.hash()
    }

    /// Topic-String
    pub fn to_string(&self) -> String {
        self.topic.to_string()
    }
}

/// Topic-Manager für Subscription-Tracking
pub struct TopicManager {
    /// Abonnierte Topics
    subscribed: RwLock<HashSet<TopicHash>>,

    /// Topic-Details (Hash → RealmTopic)
    topics: RwLock<HashMap<TopicHash, RealmTopic>>,

    /// Realm-Memberships (Realm-ID → Set<Topic-Type>)
    realm_memberships: RwLock<HashMap<String, HashSet<TopicType>>>,

    /// Direct-Message-Topics (für schnellen Lookup)
    direct_topics: RwLock<HashSet<TopicHash>>,
}

impl TopicManager {
    /// Erstelle neuen TopicManager
    pub fn new() -> Self {
        Self {
            subscribed: RwLock::new(HashSet::new()),
            topics: RwLock::new(HashMap::new()),
            realm_memberships: RwLock::new(HashMap::new()),
            direct_topics: RwLock::new(HashSet::new()),
        }
    }

    /// Erstelle als Arc
    pub fn new_arc() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Abonniere Topic
    pub fn subscribe(&self, topic: RealmTopic) -> TopicHash {
        let hash = topic.hash();

        self.subscribed.write().insert(hash.clone());
        self.topics.write().insert(hash.clone(), topic.clone());

        // Track Realm-Membership
        if let Some(realm_id) = &topic.realm_id {
            self.realm_memberships
                .write()
                .entry(realm_id.clone())
                .or_insert_with(HashSet::new)
                .insert(topic.topic_type.clone());
        }

        // Track Direct-Topics
        if topic.topic_type == TopicType::Direct {
            self.direct_topics.write().insert(hash.clone());
        }

        tracing::debug!(topic = %topic.to_string(), "Subscribed to topic");
        hash
    }

    /// Kündige Topic-Abo
    pub fn unsubscribe(&self, topic: &RealmTopic) {
        let hash = topic.hash();

        self.subscribed.write().remove(&hash);
        self.topics.write().remove(&hash);

        // Update Realm-Membership (avoid deadlock by not nesting write() calls)
        if let Some(realm_id) = &topic.realm_id {
            let should_remove = {
                let mut memberships = self.realm_memberships.write();
                if let Some(types) = memberships.get_mut(realm_id) {
                    types.remove(&topic.topic_type);
                    types.is_empty()
                } else {
                    false
                }
            };
            if should_remove {
                self.realm_memberships.write().remove(realm_id);
            }
        }

        // Update Direct-Topics
        if topic.topic_type == TopicType::Direct {
            self.direct_topics.write().remove(&hash);
        }

        tracing::debug!(topic = %topic.to_string(), "Unsubscribed from topic");
    }

    /// Prüfe ob Topic abonniert ist
    pub fn is_subscribed(&self, hash: &TopicHash) -> bool {
        self.subscribed.read().contains(hash)
    }

    /// Erhalte Topic-Details
    pub fn get_topic(&self, hash: &TopicHash) -> Option<RealmTopic> {
        self.topics.read().get(hash).cloned()
    }

    /// Alle abonnierten Topics
    pub fn subscribed_topics(&self) -> Vec<RealmTopic> {
        self.topics.read().values().cloned().collect()
    }

    /// Alle Realm-Topics für eine Realm-ID
    pub fn realm_topics(&self, realm_id: &str) -> Vec<RealmTopic> {
        self.topics
            .read()
            .values()
            .filter(|t| t.realm_id.as_deref() == Some(realm_id))
            .cloned()
            .collect()
    }

    /// Join Realm (abonniere alle Standard-Topics)
    pub fn join_realm(&self, realm_id: &str) -> Vec<TopicHash> {
        let topics = vec![
            RealmTopic::realm_events(realm_id),
            RealmTopic::realm_trust(realm_id),
            RealmTopic::realm_sagas(realm_id),
        ];

        topics.into_iter().map(|t| self.subscribe(t)).collect()
    }

    /// Leave Realm (kündige alle Topics)
    pub fn leave_realm(&self, realm_id: &str) {
        let topics = self.realm_topics(realm_id);
        for topic in topics {
            self.unsubscribe(&topic);
        }
    }

    /// Prüfe Realm-Membership
    pub fn is_realm_member(&self, realm_id: &str) -> bool {
        self.realm_memberships.read().contains_key(realm_id)
    }

    /// Anzahl abonnierter Topics
    pub fn subscription_count(&self) -> usize {
        self.subscribed.read().len()
    }
}

impl Default for TopicManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Message-Typen für Topics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TopicMessage {
    /// Event-Broadcast
    Event {
        /// Event-ID
        event_id: String,
        /// Serialisiertes Event
        event_data: Vec<u8>,
        /// Sender-DID
        sender: String,
    },

    /// Trust-Attestation
    TrustAttestation {
        /// Attestierender
        attester: String,
        /// Attestierter
        subject: String,
        /// Trust-Update (R-Delta)
        trust_delta: f64,
        /// Begründung
        reason: Option<String>,
    },

    /// Saga-Broadcast
    SagaBroadcast {
        /// Saga-ID
        saga_id: String,
        /// Saga-Phase
        phase: String,
        /// Payload
        payload: Vec<u8>,
    },

    /// Direct Message
    DirectMessage {
        /// Sender
        from: String,
        /// Encrypted Payload
        encrypted_payload: Vec<u8>,
        /// Nonce für Decryption
        nonce: Vec<u8>,
    },

    /// Announcement
    Announcement {
        /// Typ
        announcement_type: String,
        /// Message
        message: String,
        /// Optional: Affected Realms
        affected_realms: Vec<String>,
    },
}

impl TopicMessage {
    /// Serialisiere für Gossipsub
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| anyhow!("Serialization failed: {}", e))
    }

    /// Deserialisiere aus Gossipsub
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(|e| anyhow!("Deserialization failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realm_topic_creation() {
        let topic = RealmTopic::realm_events("test-realm");
        assert_eq!(topic.topic_type, TopicType::RealmEvents);
        assert_eq!(topic.realm_id, Some("test-realm".to_string()));
        assert!(topic.to_string().contains("test-realm"));
    }

    #[test]
    fn test_direct_topic() {
        let sender = DID::new_self("alice");
        let receiver = DID::new_self("bob");
        let topic = RealmTopic::direct(&sender, &receiver);

        assert_eq!(topic.topic_type, TopicType::Direct);
        assert!(topic.to_string().contains("alice"));
        assert!(topic.to_string().contains("bob"));
    }

    #[test]
    fn test_topic_parsing() {
        let topic = RealmTopic::realm_events("my-realm");
        let parsed = RealmTopic::from_str(&topic.to_string()).unwrap();

        assert_eq!(parsed.topic_type, topic.topic_type);
        assert_eq!(parsed.realm_id, topic.realm_id);
    }

    #[test]
    fn test_topic_manager() {
        let manager = TopicManager::new();

        // Join Realm
        let hashes = manager.join_realm("test-realm");
        assert_eq!(hashes.len(), 3); // events, trust, sagas

        // Check Membership
        assert!(manager.is_realm_member("test-realm"));
        assert_eq!(manager.subscription_count(), 3);

        // Leave Realm
        manager.leave_realm("test-realm");
        assert!(!manager.is_realm_member("test-realm"));
        assert_eq!(manager.subscription_count(), 0);
    }

    #[test]
    fn test_topic_message_serialization() {
        let msg = TopicMessage::Event {
            event_id: "evt-123".to_string(),
            event_data: vec![1, 2, 3],
            sender: "did:erynoa:self:alice".to_string(),
        };

        let bytes = msg.to_bytes().unwrap();
        let decoded = TopicMessage::from_bytes(&bytes).unwrap();

        match decoded {
            TopicMessage::Event { event_id, .. } => {
                assert_eq!(event_id, "evt-123");
            }
            _ => panic!("Wrong message type"),
        }
    }
}
