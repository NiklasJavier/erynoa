//! # Peer-Identität (DID ↔ libp2p PeerId ↔ UniversalId)
//!
//! Konvertierung zwischen Erynoa-DID, libp2p-PeerId und UniversalId.
//!
//! ## Konzept
//!
//! - DID basiert auf Ed25519-Public-Key
//! - libp2p PeerId = Multihash(PublicKey)
//! - UniversalId = Content-addressed 32-byte Identifier
//! - Bidirektionale Konvertierung möglich
//!
//! ## UniversalId-Integration (v0.4.0)
//!
//! Jede Peer-Identität hat nun eine `UniversalId`, die direkt aus der DID abgeleitet wird.
//! Dies ermöglicht konsistente Identifikation über alle Erynoa-Komponenten hinweg:
//!
//! - `state.rs`: SwarmState.peer_universal_id
//! - `trust_gate.rs`: PeerTrustInfo.universal_id
//! - `relay_selection.rs`: RelayCandidate.universal_id

use crate::domain::{UniversalId, DID};
use anyhow::{anyhow, Result};
use ed25519_dalek::SigningKey;
use libp2p::identity::{ed25519, Keypair, PublicKey};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};

/// Peer-Identität mit DID, UniversalId und libp2p-Keypair
///
/// Die `UniversalId` wird aus der DID abgeleitet und ermöglicht
/// einheitliche Identifikation über alle Erynoa-Subsysteme:
///
/// - Trust-Gate für Verbindungssteuerung
/// - Relay-Selection für Privacy-Layer
/// - State-Management für P2P-Metriken
/// - Event-System für Signatur-Zuordnung
#[derive(Clone)]
pub struct PeerIdentity {
    /// Erynoa DID
    pub did: DID,

    /// UniversalId (aus DID.id abgeleitet)
    ///
    /// Content-addressed 32-byte Identifier für systemweite Konsistenz.
    universal_id: UniversalId,

    /// libp2p Keypair (Ed25519)
    keypair: Keypair,

    /// libp2p PeerId
    pub peer_id: PeerId,
}

impl std::fmt::Debug for PeerIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PeerIdentity")
            .field("did", &self.did)
            .field("universal_id", &self.universal_id)
            .field("peer_id", &self.peer_id)
            .finish()
    }
}

impl PeerIdentity {
    /// Erstelle neue Identität aus Ed25519-Keypair
    pub fn from_ed25519_keypair(signing_key: &SigningKey, did: DID) -> Result<Self> {
        // Konvertiere ed25519-dalek zu libp2p-ed25519
        let secret_bytes = signing_key.to_bytes();
        let ed25519_keypair = ed25519::Keypair::try_from_bytes(&mut secret_bytes.to_vec())
            .map_err(|e| anyhow!("Failed to create libp2p keypair: {}", e))?;

        let keypair = Keypair::from(ed25519_keypair);
        let peer_id = PeerId::from(keypair.public());

        // UniversalId aus DID ableiten
        let universal_id = did.id.clone();

        Ok(Self {
            did,
            universal_id,
            keypair,
            peer_id,
        })
    }

    /// Erstelle neue zufällige Identität
    pub fn generate() -> Self {
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());

        // DID aus Public-Key ableiten
        let public_key_bytes = keypair
            .public()
            .try_into_ed25519()
            .map(|pk| pk.to_bytes())
            .unwrap_or_else(|_| [0u8; 32]);

        let did = DID::new_self(&public_key_bytes);

        // UniversalId aus DID
        let universal_id = did.id.clone();

        Self {
            did,
            universal_id,
            keypair,
            peer_id,
        }
    }

    /// Erstelle Identität aus existierender DID und Keypair
    ///
    /// Nützlich für Migration von Legacy-Identitäten.
    pub fn from_did_and_keypair(did: DID, keypair: Keypair) -> Self {
        let peer_id = PeerId::from(keypair.public());
        let universal_id = did.id.clone();

        Self {
            did,
            universal_id,
            keypair,
            peer_id,
        }
    }

    /// Erhalte die UniversalId
    ///
    /// Content-addressed Identifier für systemweite Konsistenz.
    /// Kann für Lookups in TrustGate, RelaySelection, StateEvents verwendet werden.
    pub fn universal_id(&self) -> &UniversalId {
        &self.universal_id
    }

    /// Erhalte Kopie der UniversalId
    pub fn universal_id_owned(&self) -> UniversalId {
        self.universal_id.clone()
    }

    /// Erhalte das Keypair (für Swarm)
    pub fn keypair(&self) -> Keypair {
        self.keypair.clone()
    }

    /// Erhalte den Public-Key
    pub fn public_key(&self) -> PublicKey {
        self.keypair.public()
    }

    /// Signiere Daten mit dem Private-Key
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        self.keypair
            .sign(data)
            .map_err(|e| anyhow!("Signing failed: {}", e))
    }

    /// Verifiziere Signatur eines anderen Peers
    pub fn verify(public_key: &PublicKey, data: &[u8], signature: &[u8]) -> bool {
        public_key.verify(data, signature)
    }

    /// Konvertiere PeerId zu UniversalId (mit bekanntem Public-Key)
    ///
    /// Erstellt eine temporäre DID und gibt deren UniversalId zurück.
    pub fn peer_id_to_universal_id(public_key: &PublicKey) -> Result<UniversalId> {
        let bytes = public_key
            .clone()
            .try_into_ed25519()
            .map(|pk| pk.to_bytes())
            .map_err(|_| anyhow!("Only Ed25519 keys can be converted"))?;

        let did = DID::new_self(&bytes);
        Ok(did.id)
    }
}

/// Signierte Peer-Info für DHT-Publishing
///
/// Enthält nun auch die `UniversalId` für konsistente Identifikation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedPeerInfo {
    /// DID des Peers (URI-Format)
    pub did: String,

    /// UniversalId (32 Bytes als Hex)
    ///
    /// Content-addressed Identifier aus der DID abgeleitet.
    /// Ermöglicht direkte Lookups in TrustGate und RelaySelection.
    pub universal_id: String,

    /// libp2p PeerId (als String)
    pub peer_id: String,

    /// Multiaddrs des Peers
    pub addresses: Vec<String>,

    /// Timestamp der Erstellung
    pub timestamp: u64,

    /// Signatur über (universal_id || did || peer_id || addresses || timestamp)
    pub signature: Vec<u8>,

    /// Public-Key für Verifikation (Ed25519)
    pub public_key: Vec<u8>,
}

impl SignedPeerInfo {
    /// Erstelle signierte Peer-Info
    pub fn new(identity: &PeerIdentity, addresses: Vec<String>) -> Result<Self> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let did = identity.did.to_uri();
        let universal_id = identity.universal_id().to_hex();
        let peer_id = identity.peer_id.to_string();

        // Daten zum Signieren (universal_id zuerst für eindeutige Bindung)
        let mut signing_data = Vec::new();
        signing_data.extend_from_slice(universal_id.as_bytes());
        signing_data.extend_from_slice(did.as_bytes());
        signing_data.extend_from_slice(peer_id.as_bytes());
        for addr in &addresses {
            signing_data.extend_from_slice(addr.as_bytes());
        }
        signing_data.extend_from_slice(&timestamp.to_le_bytes());

        let signature = identity.sign(&signing_data)?;

        let public_key = identity
            .public_key()
            .try_into_ed25519()
            .map(|pk| pk.to_bytes().to_vec())
            .map_err(|_| anyhow!("Only Ed25519 keys supported"))?;

        Ok(Self {
            did,
            universal_id,
            peer_id,
            addresses,
            timestamp,
            signature,
            public_key,
        })
    }

    /// Verifiziere die Signatur
    pub fn verify(&self) -> Result<bool> {
        // Public-Key rekonstruieren
        let ed25519_pk = ed25519::PublicKey::try_from_bytes(&self.public_key)
            .map_err(|e| anyhow!("Invalid public key: {}", e))?;
        let public_key = PublicKey::from(ed25519_pk);

        // Daten rekonstruieren (gleiche Reihenfolge wie beim Signieren)
        let mut signing_data = Vec::new();
        signing_data.extend_from_slice(self.universal_id.as_bytes());
        signing_data.extend_from_slice(self.did.as_bytes());
        signing_data.extend_from_slice(self.peer_id.as_bytes());
        for addr in &self.addresses {
            signing_data.extend_from_slice(addr.as_bytes());
        }
        signing_data.extend_from_slice(&self.timestamp.to_le_bytes());

        Ok(PeerIdentity::verify(
            &public_key,
            &signing_data,
            &self.signature,
        ))
    }

    /// Parse UniversalId aus dem Hex-String
    pub fn parse_universal_id(&self) -> Result<UniversalId> {
        UniversalId::from_hex(&self.universal_id)
            .map_err(|e| anyhow!("Invalid UniversalId hex: {}", e))
    }

    /// Prüfe ob Info noch gültig ist (nicht älter als max_age_secs)
    pub fn is_valid(&self, max_age_secs: u64) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now - self.timestamp < max_age_secs
    }

    /// Konvertiere zu DHT-Record-Key
    pub fn record_key(&self) -> Vec<u8> {
        format!("/erynoa/peer/{}", self.peer_id).into_bytes()
    }

    /// Serialisiere für DHT-Storage
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| anyhow!("Serialization failed: {}", e))
    }

    /// Deserialisiere aus DHT-Storage
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(|e| anyhow!("Deserialization failed: {}", e))
    }
}

/// Konvertiere DID zu PeerId (wenn Public-Key bekannt)
pub fn did_to_peer_id(did: &DID) -> Result<PeerId> {
    // public_key ist direkt im DID
    let public_key_bytes = did.public_key;

    let ed25519_pk = ed25519::PublicKey::try_from_bytes(&public_key_bytes)
        .map_err(|e| anyhow!("Invalid Ed25519 public key: {}", e))?;

    let public_key = PublicKey::from(ed25519_pk);
    Ok(PeerId::from(public_key))
}

/// Konvertiere PeerId zu DID (falls Ed25519)
pub fn peer_id_to_did(_peer_id: &PeerId, public_key: &PublicKey) -> Result<DID> {
    let bytes = public_key
        .clone()
        .try_into_ed25519()
        .map(|pk| pk.to_bytes())
        .map_err(|_| anyhow!("Only Ed25519 keys can be converted to DID"))?;

    Ok(DID::new_self(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_identity() {
        let identity = PeerIdentity::generate();
        // DID hat public_key, nicht unique_id
        assert!(identity.did.public_key != [0u8; 32]);
        assert!(!identity.peer_id.to_string().is_empty());
    }

    #[test]
    fn test_universal_id_derived_from_did() {
        let identity = PeerIdentity::generate();

        // UniversalId sollte mit DID.id übereinstimmen
        assert_eq!(*identity.universal_id(), identity.did.id);

        // UniversalId sollte nicht leer sein
        assert!(!identity.universal_id().to_hex().is_empty());
    }

    #[test]
    fn test_universal_id_consistency() {
        // Zwei Identitäten mit gleichem Public-Key sollten gleiche UniversalId haben
        let keypair1 = Keypair::generate_ed25519();
        let keypair2 = keypair1.clone();

        let peer_id1 = PeerId::from(keypair1.public());
        let peer_id2 = PeerId::from(keypair2.public());

        assert_eq!(peer_id1, peer_id2);

        // Public-Key extrahieren
        let pk_bytes = keypair1
            .public()
            .try_into_ed25519()
            .map(|pk| pk.to_bytes())
            .unwrap();

        let did1 = DID::new_self(&pk_bytes);
        let did2 = DID::new_self(&pk_bytes);

        // DIDs mit gleichem Public-Key haben gleiche UniversalId
        assert_eq!(did1.id, did2.id);
    }

    #[test]
    fn test_sign_verify() {
        let identity = PeerIdentity::generate();
        let data = b"test message";

        let signature = identity.sign(data).unwrap();
        assert!(PeerIdentity::verify(
            &identity.public_key(),
            data,
            &signature
        ));
    }

    #[test]
    fn test_signed_peer_info() {
        let identity = PeerIdentity::generate();
        let addresses = vec!["/ip4/127.0.0.1/tcp/4001".to_string()];

        let info = SignedPeerInfo::new(&identity, addresses).unwrap();
        assert!(info.verify().unwrap());
        assert!(info.is_valid(60));
    }

    #[test]
    fn test_signed_peer_info_universal_id() {
        let identity = PeerIdentity::generate();
        let addresses = vec!["/ip4/127.0.0.1/tcp/4001".to_string()];

        let info = SignedPeerInfo::new(&identity, addresses).unwrap();

        // UniversalId sollte enthalten und parsebar sein
        assert!(!info.universal_id.is_empty());

        let parsed_uid = info.parse_universal_id().unwrap();
        assert_eq!(parsed_uid, *identity.universal_id());
    }

    #[test]
    fn test_did_peer_id_roundtrip() {
        let identity = PeerIdentity::generate();

        // DID → PeerId
        let peer_id = did_to_peer_id(&identity.did).unwrap();
        assert_eq!(peer_id, identity.peer_id);

        // PeerId → DID
        let did = peer_id_to_did(&identity.peer_id, &identity.public_key()).unwrap();
        assert_eq!(did.public_key, identity.did.public_key);
    }

    #[test]
    fn test_peer_id_to_universal_id() {
        let identity = PeerIdentity::generate();

        // Konvertiere über Helper-Funktion
        let uid = PeerIdentity::peer_id_to_universal_id(&identity.public_key()).unwrap();

        // Sollte mit der Identitäts-UniversalId übereinstimmen
        assert_eq!(uid, *identity.universal_id());
    }
}
