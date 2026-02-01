# libp2p P2P-Netzwerk-Implementierung in Erynoa

> **Version:** 1.1.0
> **Feature-Flag:** `p2p`
> **Axiome:** Κ9 (Kausale Struktur), Κ10 (Bezeugung-Finalität), Κ15–Κ17 (Consensus-Finality), Κ19 (Anti-Verkalkung), Κ23 (Gateway), Κ26 (Anomaly Detection), PR5 (Schlüssel-Isolation)

## Übersicht

Die P2P-Implementierung ermöglicht eine **vollständig dezentrale, realm-spezifische Kommunikation** zwischen Erynoa-Peers. Sie ist:

- **Offline-First**: Lokale Fjall-DB als Cache, Delta-Sync bei Reconnect
- **Realm-zentriert**: Sync nur für beigetretene Realms (kein globaler Broadcast)
- **Trust-gesteuert**: Verbindungen basieren auf Trust-Werten (nur Trust.R > 0.5)
- **Gaming-resistent**: Anomaly-Checks, Mana-Kosten, Novelty-Prüfung
- **Lazy**: Nur Delta-Daten (ab letztem Hash, Merkle-Proofs)
- **Leaderless**: Reines P2P, kein zentraler Koordinator

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          P2P NETWORK LAYER                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
│  │   SWARM      │  │   GOSSIPSUB  │  │   KADEMLIA   │                  │
│  │   MANAGER    │  │   (PubSub)   │  │   (DHT)      │                  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                  │
│         │                 │                 │                           │
│  ┌──────┴─────────────────┴─────────────────┴──────┐                   │
│  │              ERYNOA BEHAVIOUR                    │                   │
│  │  • Realm-Topics (/erynoa/realm/{id}/events/v1)  │                   │
│  │  • DID-based PeerID (Ed25519)                   │                   │
│  │  • Trust-gated Connections                      │                   │
│  │  • Event Sync Protocol                          │                   │
│  └──────────────────────────────────────────────────┘                   │
│                            │                                            │
│  ┌────────────────────────┴────────────────────────┐                   │
│  │              TRANSPORT LAYER                     │                   │
│  │  TCP + Noise (Encryption) + Yamux (Mux)         │                   │
│  └──────────────────────────────────────────────────┘                   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Sync-Architektur

### Was wird synchronisiert?

Nicht alles wird synchronisiert – nur das Nötige, um Konsistenz zu halten. Sync ist **realm-beschränkt** für Skalierbarkeit (Millionen Events/Tag ohne Overload).

| Datentyp              | Sync    | Priorität | Details                                                                         |
| --------------------- | ------- | --------- | ------------------------------------------------------------------------------- |
| **Events** (DAG)      | ✅ MUSS | Kern      | Posts, Attestationen, Sagas, Alarme – alle neuen Events in beigetretenen Realms |
| **Trust-Vektoren**    | ⚠️ SOLL | Hoch      | Nur Attestation-Events syncen (R/Ω lokal berechnet aus Events)                  |
| **Schemas/Stores**    | ✅ MUSS | Mittel    | Schema-Meta (`_schema`) + Daten-Changes für gemeinsame Stores                   |
| **Blueprints**        | ⚠️ SOLL | Niedrig   | Hashes + Meta für Discovery, Inhalte lazy bei Deployment                        |
| **Realms**            | ⚠️ SOLL | Mittel    | Membership-Changes, Policies, Visibility                                        |
| **DID-Updates**       | ⚠️ SOLL | Niedrig   | Nur Updates (neue Adressen), nicht vollständig (DHT-basiert)                    |
| **𝔼-Wert/Formula**    | ❌ NEIN | -         | Lokal berechnet aus Events                                                      |
| **Persönliche Daten** | ❌ NEIN | -         | Nur lokal im `personal` Prefix, außer explicit geteilt                          |

**Regel:** Sync nur wenn relevant für den Realm und der Peer Mitglied ist. Persönliche Stores syncen **nie**.

### Sync-Flows

Der Sync ist **hybrid**: Push (Gossip für Hot-Data), Pull (Request für Cold-Data).

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         SYNC FLOW OVERVIEW                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────┐         ┌──────────────┐         ┌──────────────┐    │
│  │   STARTUP    │────────>│  DISCOVERY   │────────>│  REALM-JOIN  │    │
│  │  Bootstrap   │         │ Kademlia+mDNS│         │  Subscribe   │    │
│  └──────────────┘         └──────────────┘         └──────┬───────┘    │
│                                                           │            │
│                                    ┌──────────────────────┤            │
│                                    │                      │            │
│                                    ▼                      ▼            │
│                           ┌──────────────┐       ┌──────────────┐      │
│                           │  PUSH SYNC   │       │  PULL SYNC   │      │
│                           │  (Gossipsub) │       │  (Request)   │      │
│                           │  Hot-Data    │       │  Cold-Data   │      │
│                           └──────────────┘       └──────────────┘      │
│                                    │                      │            │
│                                    └──────────┬───────────┘            │
│                                               │                        │
│                                               ▼                        │
│                                    ┌──────────────────┐                │
│                                    │  OFFLINE CACHE   │                │
│                                    │    (Fjall-DB)    │                │
│                                    │  Delta-Sync bei  │                │
│                                    │    Reconnect     │                │
│                                    └──────────────────┘                │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 1. Peer-Startup & Discovery

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      STARTUP SEQUENCE                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  1. libp2p Swarm initialisiert (TCP/Noise/Yamux)                       │
│  2. DID als Peer-ID (Ed25519 Public-Key)                               │
│  3. Bootstrap: Connect zu 5-10 Seed-Nodes                              │
│  4. Kademlia DHT: Store/Get Peer-Info (Multiaddr + DID + Trust)        │
│  5. mDNS für LAN-Discovery (lokale Tests)                              │
│  6. Trust-Filter: Nur connect zu Peers mit Trust.R > 0.5               │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 2. Realm-Join & Membership-Sync

Bei Intent "Join Realm X" → Saga → Gateway-Policy (ECLVM):

1. Subscribe zu Topic `/erynoa/realm/{realm_id}/events/v1` (Gossipsub)
2. Request Membership-List von 3–5 bekannten Peers
3. Sync Schema/Stores: Pull Meta (`_schema`) + Delta-Daten (ab letztem Hash)
4. Bei hohem Trust (sender.trust.R > 0.8) → schneller Sync (mehr Peers fragen)

#### 3. Event-Sync (Push & Pull)

**Push (Gossipsub):**

- Neue Events publish auf Realm-Topic → nur an Mitglieder (Membership-Filter)
- Validation vor Publish: Event-DAG-Check, Novelty-Prüfung, Trust des Senders
- Hohe Novelty → priorisierter Gossip (mehr Hops)

**Pull (Request/Response):**

- Bei Reconnect oder Join: "Gib mir Events ab Hash Y" → von 2–3 Peers (rotierend)
- Protokoll: `/erynoa/sync/events/1.0` – Response mit Event-Batch + Merkle-Proof

**Offline-Handhabung:**

- Lokale Fjall-DB cachet alles
- Bei Reconnect nur Delta-Sync (letzter Hash als Checkpoint)

#### 4. Trust & Attestation-Sync

- **Push:** Attestation-Events (positiv/negativ) als normale Events syncen → lokal Bayessch updaten
- **Pull:** Kein globaler Trust-Sync (emergent) – bei Bedarf Attestation-Historie requesten
- **Trust-Gate:** Nur Attestationen von Peers mit Ω > 1.2 voll gewichten

#### 5. Schemas/Stores-Sync (dynamisch)

- **Push:** Schema-Changes als Events (`schema_alter_v2`) → propagiert
- **Pull:** Bei Join: Request Realm-Schemas + Meta → lokale Migration
- **Dynamische Daten:** Put/Get/Query als Events oder direct P2P-Requests
- **Lazy:** Nur bei Zugriff migrieren, Mana-Kosten pro Sync

#### 6. Blueprints-Sync (Marketplace)

- **Push:** Neue Blueprints als Events in Marketplace-Realm
- **Pull:** Bei Deployment: Request per Hash (CAS)
- **Novelty-Filter:** Nur hoch-novel Blueprints priorisieren

### Intelligente Optimierungen

| Optimierung             | Beschreibung                                                                      |
| ----------------------- | --------------------------------------------------------------------------------- |
| **Realm-spezifisch**    | Sync nur für beigetretene Realms → Bandbreite-Sparsamkeit (10 Realms = 10 Topics) |
| **Trust-Gated Sync**    | Low-Trust → max. 100 Events pro Request akzeptieren                               |
| **Anomaly-Überwachung** | Zu viele Sync-Requests → Velocity-Alert → temporärer Ban                          |
| **Mana-Integration**    | Sync kostet Mana (z.B. 100 Events = 50 Mana) → Spam teuer                         |
| **Delta-Sync**          | Immer nur ab letztem Hash (Merkle-Proofs für Verifizierung)                       |
| **Witness-Consensus**   | Bei Sync-Konflikt: Κ15–Κ17 Finality-Regeln anwenden                               |

---

## Modul-Struktur

```
backend/src/peer/p2p/
├── mod.rs          # Modul-Organisation & Re-exports
├── config.rs       # P2P-Konfiguration (alle Sub-Configs)
├── identity.rs     # DID ↔ PeerId Konvertierung
├── topics.rs       # Realm-basierte Gossipsub Topics
├── protocol.rs     # Sync-Protokoll (Request-Response)
├── trust_gate.rs   # Trust-basierte Verbindungssteuerung
├── behaviour.rs    # Custom NetworkBehaviour
└── swarm.rs        # SwarmManager (Lifecycle)
```

---

## 1. Konfiguration (`config.rs`)

### P2PConfig

Die Hauptkonfiguration für das gesamte P2P-Netzwerk:

```rust
pub struct P2PConfig {
    /// TCP-Listen-Adressen (Default: 0.0.0.0:0, [::]:0)
    pub listen_addresses: Vec<String>,

    /// Bootstrap-Peers (Erynoa Foundation Nodes)
    pub bootstrap_peers: Vec<String>,

    /// mDNS für LAN-Discovery
    pub enable_mdns: bool,

    /// Sub-Konfigurationen
    pub kademlia: KademliaConfig,
    pub gossipsub: GossipsubConfig,
    pub trust_gate: TrustGateConfig,
    pub sync: SyncConfig,
    pub connection_limits: ConnectionLimitsConfig,
}
```

### KademliaConfig (DHT)

```rust
pub struct KademliaConfig {
    pub replication_factor: usize,  // Default: 20
    pub parallelism: usize,         // Default: 3
    pub record_ttl: Duration,       // Default: 24h
    pub provider_interval: Duration, // Default: 12h
}
```

### GossipsubConfig (PubSub)

```rust
pub struct GossipsubConfig {
    pub heartbeat_interval: Duration, // Default: 1s
    pub mesh_n: usize,                // Default: 6 (D)
    pub mesh_n_low: usize,            // Default: 4 (D_lo)
    pub mesh_n_high: usize,           // Default: 12 (D_hi)
    pub gossip_factor: f64,           // Default: 0.25
    pub history_length: usize,        // Default: 5
    pub history_gossip: usize,        // Default: 3
    pub flood_publish: bool,          // Default: true
    pub max_transmit_size: usize,     // Default: 64 KB
}
```

### TrustGateConfig

```rust
pub struct TrustGateConfig {
    /// Minimum Trust-R für eingehende Verbindungen
    pub min_incoming_trust_r: f64,      // Default: 0.1

    /// Minimum Trust-Ω für Relay-Funktionen
    pub min_relay_trust_omega: f64,     // Default: 0.5

    /// Unbekannte Peers automatisch ablehnen
    pub reject_unknown_peers: bool,     // Default: false

    /// Grace-Period für Newcomer
    pub newcomer_grace_period: Duration, // Default: 60s
}
```

### SyncConfig

```rust
pub struct SyncConfig {
    pub max_events_per_request: usize,    // Default: 100
    pub request_timeout: Duration,         // Default: 30s
    pub max_concurrent_requests: usize,    // Default: 5
    pub delta_sync: bool,                  // Default: true
}
```

### ConnectionLimitsConfig

```rust
pub struct ConnectionLimitsConfig {
    pub max_incoming: u32,   // Default: 100
    pub max_outgoing: u32,   // Default: 50
    pub max_per_peer: u32,   // Default: 2
    pub idle_timeout: Duration, // Default: 60s
}
```

---

## 2. Peer-Identität (`identity.rs`)

### PeerIdentity

Kapselt die Erynoa-DID mit dem libp2p-Keypair:

```rust
pub struct PeerIdentity {
    /// Erynoa DID (did:erynoa:self:...)
    pub did: DID,

    /// libp2p Keypair (Ed25519)
    keypair: Keypair,

    /// libp2p PeerId (Multihash des Public-Key)
    pub peer_id: PeerId,
}
```

**Funktionen:**

| Funktion                 | Beschreibung                      |
| ------------------------ | --------------------------------- |
| `generate()`             | Erstellt neue zufällige Identität |
| `from_ed25519_keypair()` | Erstellt aus bestehendem Keypair  |
| `sign(data)`             | Signiert Daten mit Private-Key    |
| `verify(pk, data, sig)`  | Verifiziert Signatur              |
| `keypair()`              | Gibt Keypair für Swarm zurück     |

### SignedPeerInfo

Für DHT-Publishing signierte Peer-Informationen:

```rust
pub struct SignedPeerInfo {
    pub did: String,
    pub peer_id: String,
    pub addresses: Vec<String>,
    pub timestamp: u64,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}
```

### Konvertierungsfunktionen

```rust
/// DID → PeerId (unique_id ist Base58-encoded Public-Key)
pub fn did_to_peer_id(did: &DID) -> Result<PeerId>;

/// PeerId → DID (benötigt Public-Key)
pub fn peer_id_to_did(peer_id: &PeerId, public_key: &PublicKey) -> Result<DID>;
```

---

## 3. Topic-System (`topics.rs`)

### Topic-Schema

```
/erynoa/realm/{realm_id}/events/v1     - Event-Propagation
/erynoa/realm/{realm_id}/trust/v1      - Trust-Attestationen
/erynoa/realm/{realm_id}/sagas/v1      - Saga-Broadcasts
/erynoa/direct/{sender}/{receiver}     - Direct Messages
/erynoa/global/announcements/v1        - Netzwerk-Announcements
```

### TopicType

```rust
pub enum TopicType {
    RealmEvents,    // Events in Realm propagieren
    RealmTrust,     // Trust-Attestationen
    RealmSagas,     // Saga-Broadcasts
    Direct,         // Direct Messaging
    Global,         // Globale Announcements
}
```

### RealmTopic

```rust
pub struct RealmTopic {
    pub topic_type: TopicType,
    pub realm_id: Option<String>,
    pub sender: Option<String>,
    pub receiver: Option<String>,
    topic: IdentTopic,  // libp2p Topic
}
```

**Factory-Methoden:**

```rust
// Realm-Topics
RealmTopic::realm_events("my-realm")
RealmTopic::realm_trust("my-realm")
RealmTopic::realm_sagas("my-realm")

// Direct-Topic
RealmTopic::direct(&sender_did, &receiver_did)

// Global
RealmTopic::global_announcements()

// Parsing
RealmTopic::from_str("/erynoa/realm/my-realm/events/v1")
```

### TopicManager

Verwaltet Subscriptions und Realm-Memberships:

```rust
pub struct TopicManager {
    subscribed: RwLock<HashSet<TopicHash>>,
    topics: RwLock<HashMap<TopicHash, RealmTopic>>,
    realm_memberships: RwLock<HashMap<String, HashSet<TopicType>>>,
    direct_topics: RwLock<HashSet<TopicHash>>,
}
```

**API:**

| Methode                     | Beschreibung                  |
| --------------------------- | ----------------------------- |
| `subscribe(topic)`          | Abonniere Topic               |
| `unsubscribe(topic)`        | Kündige Abo                   |
| `join_realm(realm_id)`      | Abonniere alle 3 Realm-Topics |
| `leave_realm(realm_id)`     | Kündige alle Realm-Topics     |
| `is_realm_member(realm_id)` | Prüfe Membership              |
| `realm_topics(realm_id)`    | Alle Topics eines Realms      |

### TopicMessage

```rust
pub enum TopicMessage {
    /// Event-Broadcast
    Event {
        event_id: String,
        event_data: Vec<u8>,
        sender: String,
    },

    /// Trust-Attestation
    TrustAttestation {
        attester: String,
        subject: String,
        trust_delta: f64,
        reason: Option<String>,
    },

    /// Saga-Broadcast
    SagaBroadcast {
        saga_id: String,
        phase: String,
        payload: Vec<u8>,
    },

    /// Direct Message (encrypted)
    DirectMessage {
        from: String,
        encrypted_payload: Vec<u8>,
        nonce: Vec<u8>,
    },

    /// Announcement
    Announcement {
        announcement_type: String,
        message: String,
        affected_realms: Vec<String>,
    },
}
```

---

## 4. Sync-Protokoll (`protocol.rs`)

### Protokoll-Versionen

| Protokoll                     | Verwendung                    |
| ----------------------------- | ----------------------------- |
| `/erynoa/sync/events/1.0`     | Event-Synchronisation         |
| `/erynoa/sync/trust/1.0`      | Trust-State-Abfragen          |
| `/erynoa/sync/membership/1.0` | Realm-Membership-Verification |

### SyncRequest

```rust
pub enum SyncRequest {
    /// Events ab einem Hash anfordern
    GetEventsAfter {
        realm_id: String,
        after_hash: Option<String>,
        limit: usize,
    },

    /// Spezifische Events anfordern
    GetEventsByIds {
        realm_id: String,
        event_ids: Vec<String>,
    },

    /// Trust-State abfragen
    GetTrustState {
        subject_did: String,
    },

    /// Membership verifizieren
    VerifyMembership {
        realm_id: String,
        did: String,
    },

    /// Membership-Proof anfordern
    GetMembershipProof {
        realm_id: String,
    },

    /// Ping für Latenz
    Ping { timestamp: u64 },
}
```

### SyncResponse

```rust
pub enum SyncResponse {
    /// Events-Antwort
    Events {
        realm_id: String,
        events: Vec<SerializedEvent>,
        has_more: bool,
        next_cursor: Option<String>,
    },

    /// Trust-State
    TrustState {
        subject_did: String,
        trust_r: f64,
        trust_omega: f64,
        last_attestation: Option<u64>,
    },

    /// Membership-Verification
    MembershipVerified {
        realm_id: String,
        did: String,
        is_member: bool,
        level: Option<String>,
    },

    /// Membership-Proof
    MembershipProof {
        realm_id: String,
        proof: Vec<u8>,
        expires_at: u64,
    },

    /// Pong
    Pong {
        timestamp: u64,
        server_timestamp: u64,
    },

    /// Fehler
    Error { code: u32, message: String },
}
```

### SerializedEvent

```rust
pub struct SerializedEvent {
    pub id: String,
    pub event_type: String,
    pub data: Vec<u8>,
    pub parents: Vec<String>,
    pub timestamp: u64,
    pub creator: String,
    pub signature: Vec<u8>,
}
```

### SyncCodec

Implementiert `request_response::Codec` mit Length-Prefixed Messages:

```
┌──────────────────────────────────────┐
│ Length (4 Bytes, Big-Endian)         │
├──────────────────────────────────────┤
│ Payload (JSON-serialisiert)          │
└──────────────────────────────────────┘
```

**Maximum Message Size:** 1 MB

### Error-Codes

| Code | Bedeutung         |
| ---- | ----------------- |
| 0    | UNKNOWN           |
| 1    | REALM_NOT_FOUND   |
| 2    | EVENT_NOT_FOUND   |
| 3    | PERMISSION_DENIED |
| 4    | RATE_LIMITED      |
| 5    | INVALID_REQUEST   |
| 6    | INTERNAL_ERROR    |

---

## 5. Trust-Gate (`trust_gate.rs`)

### Konzept

Trust-basierte Verbindungssteuerung gemäß Κ23 (Gateway):

- Eingehende Verbindungen werden gegen Trust-DB geprüft
- Niedrig-Trust-Peers: Limitierte Verbindung oder Ablehnung
- Hoch-Trust-Peers: Volle Verbindung + Relay-Privileges
- Anomaly-Integration: Verdächtige Peers werden temporär gebannt

### ConnectionLevel

```rust
pub enum ConnectionLevel {
    Blocked,   // Keine Verbindung
    Limited,   // Nur lesen
    Standard,  // Normal
    Full,      // Mit Relay-Privileges
    Trusted,   // Bootstrap/Validator
}
```

**Berechtigungen:**

| Level    | receive_events | send_events | relay | sync |
| -------- | -------------- | ----------- | ----- | ---- |
| Blocked  | ❌             | ❌          | ❌    | ❌   |
| Limited  | ✅             | ❌          | ❌    | ❌   |
| Standard | ✅             | ✅          | ❌    | ✅   |
| Full     | ✅             | ✅          | ✅    | ✅   |
| Trusted  | ✅             | ✅          | ✅    | ✅   |

### Trust → Level Mapping

| Trust-R   | Trust-Ω | Level    |
| --------- | ------- | -------- |
| < 0.1     | \*      | Blocked  |
| 0.1 - 0.5 | \*      | Limited  |
| 0.5 - 0.7 | \*      | Standard |
| 0.7 - 0.9 | ≥ 0.5   | Full     |
| ≥ 0.9     | ≥ 2.0   | Trusted  |

### PeerTrustInfo

```rust
pub struct PeerTrustInfo {
    pub did: Option<String>,
    pub trust_r: f64,
    pub trust_omega: f64,
    pub last_seen: u64,
    pub successful_interactions: u64,
    pub failed_interactions: u64,
    pub is_newcomer: bool,
    pub newcomer_since: Option<u64>,
    pub connection_level: ConnectionLevel,
}
```

### TrustGate API

| Methode                               | Beschreibung                    |
| ------------------------------------- | ------------------------------- |
| `check_connection(peer_id)`           | Prüft ob Verbindung erlaubt     |
| `register_peer(peer_id, signed_info)` | Registriert neuen Peer          |
| `update_trust(peer_id, r, ω)`         | Aktualisiert Trust-Werte        |
| `report_success(peer_id)`             | Meldet erfolgreiche Interaktion |
| `report_failure(peer_id, severity)`   | Meldet Fehler (reduziert Trust) |
| `ban_peer(peer_id, duration)`         | Temporärer Ban                  |
| `unban_peer(peer_id)`                 | Hebt Ban auf                    |
| `is_banned(peer_id)`                  | Prüft Ban-Status                |

### FailureSeverity

```rust
pub enum FailureSeverity {
    Minor,    // Timeout, temporärer Fehler
    Major,    // Ungültige Daten (Trust * 0.9)
    Critical, // Malicious Verhalten (Trust * 0.5 + 5min Ban)
}
```

---

## 6. Network Behaviour (`behaviour.rs`)

### ErynoaBehaviour

Custom `NetworkBehaviour` das mehrere Protokolle kombiniert:

```rust
#[derive(NetworkBehaviour)]
pub struct ErynoaBehaviour {
    /// Kademlia DHT (Peer Discovery, Record Storage)
    pub kademlia: kad::Behaviour<MemoryStore>,

    /// Gossipsub PubSub (Realm Topics)
    pub gossipsub: gossipsub::Behaviour,

    /// Request-Response (Sync Protocol)
    pub request_response: request_response::Behaviour<SyncCodec>,

    /// Identify (Peer Information Exchange)
    pub identify: identify::Behaviour,

    /// mDNS (LAN Discovery)
    pub mdns: mdns::tokio::Behaviour,

    /// Ping (Connection Health)
    pub ping: ping::Behaviour,
}
```

### Protokoll-IDs

| Protokoll | ID                        |
| --------- | ------------------------- |
| Kademlia  | `/erynoa/kad/1.0.0`       |
| Identify  | `/erynoa/id/1.0.0`        |
| Sync      | `/erynoa/sync/{type}/1.0` |

### Message-ID Funktion

Gossipsub Message-IDs basieren auf Content-Hash für Deduplizierung:

```rust
let message_id_fn = |message: &gossipsub::Message| {
    let mut hasher = DefaultHasher::new();
    message.data.hash(&mut hasher);
    message.source.hash(&mut hasher);
    MessageId::from(hasher.finish().to_string())
};
```

---

## 7. Swarm Manager (`swarm.rs`)

### SwarmManager

Lifecycle-Management für das libp2p Swarm:

```rust
pub struct SwarmManager {
    config: P2PConfig,
    identity: PeerIdentity,
    topics: Arc<TopicManager>,
    trust_gate: Arc<TrustGate>,
    command_tx: mpsc::Sender<SwarmCommand>,
    event_tx: broadcast::Sender<SwarmEvent2>,
    sync_request_tx: mpsc::Sender<IncomingSyncRequest>,
    running: Arc<RwLock<bool>>,
    pending_dht_gets: Arc<RwLock<HashMap<QueryId, oneshot::Sender<...>>>>,
    pending_requests: Arc<RwLock<HashMap<OutboundRequestId, oneshot::Sender<...>>>>,
}
```

### Konstruktor

```rust
// Gibt Manager + Sync-Request-Receiver zurück
let (manager, sync_rx) = SwarmManager::new(config, identity);
```

### SwarmCommand

```rust
pub enum SwarmCommand {
    Start,
    Stop,
    Connect { addr, response },
    Publish { topic, message, response },
    Subscribe { topic, response },
    Unsubscribe { topic, response },
    SendRequest { peer_id, request, response },
    DhtPut { key, value, response },
    DhtGet { key, response },
    GetConnectedPeers { response },
    GetListenAddresses { response },
}
```

### SwarmEvent2

Clone-fähige Events für Applikation:

```rust
pub enum SwarmEvent2 {
    PeerConnected { peer_id },
    PeerDisconnected { peer_id },
    GossipMessage { topic, message, source },
    MdnsDiscovered { peer_id, addresses },
    BootstrapComplete,
}
```

### IncomingSyncRequest

Nicht Clone-fähig (wegen ResponseChannel):

```rust
pub struct IncomingSyncRequest {
    pub peer_id: PeerId,
    pub request: SyncRequest,
    pub channel: ResponseChannel<Vec<u8>>,
}
```

### Transport-Stack

```
TCP → Noise (Encryption) → Yamux (Multiplexing)
```

### High-Level API

| Methode                                                | Beschreibung                    |
| ------------------------------------------------------ | ------------------------------- |
| `run()`                                                | Startet Event-Loop (blocking)   |
| `join_realm(realm_id)`                                 | Joint Realm (subscribes Topics) |
| `leave_realm(realm_id)`                                | Verlässt Realm                  |
| `publish_event(realm_id, data, sender)`                | Publiziert Event                |
| `request_events(peer_id, realm_id, after_hash, limit)` | Fordert Events an               |
| `connected_peers()`                                    | Liste verbundener Peers         |
| `is_running()`                                         | Prüft ob Swarm läuft            |

---

## Verwendung

### Basic Setup

```rust
use erynoa_api::peer::p2p::{P2PConfig, PeerIdentity, SwarmManager};

// Konfiguration
let config = P2PConfig::default();

// Identität generieren oder laden
let identity = PeerIdentity::generate();

// SwarmManager erstellen
let (manager, sync_rx) = SwarmManager::new(config, identity);

// Event-Receiver holen
let mut events = manager.event_receiver();

// Swarm in Background-Task starten
tokio::spawn(async move {
    manager.run().await.expect("Swarm failed");
});

// Events verarbeiten
tokio::spawn(async move {
    while let Ok(event) = events.recv().await {
        match event {
            SwarmEvent2::PeerConnected { peer_id } => {
                println!("Connected: {}", peer_id);
            }
            SwarmEvent2::GossipMessage { topic, message, .. } => {
                // Handle message
            }
            _ => {}
        }
    }
});

// Sync-Requests verarbeiten
tokio::spawn(async move {
    while let Some(req) = sync_rx.recv().await {
        // Handle sync request, send response via channel
    }
});
```

### Realm-Join

```rust
// Join Realm (abonniert events, trust, sagas Topics)
manager.join_realm("my-realm").await?;

// Event publizieren
let event_data = serde_json::to_vec(&my_event)?;
manager.publish_event("my-realm", event_data, &my_did).await?;

// Leave Realm
manager.leave_realm("my-realm").await?;
```

### Event-Sync (Pull)

```rust
// Events von Peer anfordern
let response = manager
    .request_events(peer_id, "my-realm", Some("last-hash"), 100)
    .await?;

match response {
    SyncResponse::Events { events, has_more, .. } => {
        for event in events {
            // Process event
        }
        if has_more {
            // Request more with next_cursor
        }
    }
    SyncResponse::Error { code, message } => {
        eprintln!("Sync error {}: {}", code, message);
    }
    _ => {}
}
```

---

## Dependencies

```toml
[dependencies]
libp2p = { version = "0.54", features = [
    "tokio",
    "tcp",
    "noise",
    "yamux",
    "gossipsub",
    "kad",
    "mdns",
    "request-response",
    "identify",
    "ping",
    "macros",
    "serde",
    "ed25519"
], optional = true }
futures = { version = "0.3", optional = true }

[features]
p2p = ["dep:libp2p", "dep:futures"]
```

---

## Tests

Alle Module enthalten Unit-Tests:

```bash
# Alle P2P-Tests ausführen
cargo test --features p2p peer::p2p

# Spezifische Module
cargo test --features p2p peer::p2p::identity
cargo test --features p2p peer::p2p::topics
cargo test --features p2p peer::p2p::trust_gate
cargo test --features p2p peer::p2p::protocol
cargo test --features p2p peer::p2p::swarm
```

---

## Roadmap

### Phase 1: Basis-Swarm (Woche 1-2) ✅

- [x] Transport-Stack (TCP + Noise + Yamux)
- [x] PeerIdentity (DID ↔ PeerId)
- [x] Konfigurationssystem
- [x] Bootstrap-Peer-Support

### Phase 2: Discovery & Auth (Woche 3) ✅

- [x] Kademlia DHT für Peer-Discovery
- [x] mDNS für LAN-Discovery
- [x] SignedPeerInfo für DHT
- [x] DID-basierte Authentifizierung

### Phase 3: Gossipsub & Topics (Woche 4) ✅

- [x] Gossipsub mit Realm-Topics
- [x] TopicManager (Join/Leave Realms)
- [x] TopicMessage-Typen (Event, Trust, Saga, Direct)
- [x] Membership-Filter in Gossipsub-Validator

### Phase 4: Pull-Sync & Offline-Cache (Woche 5) ✅

- [x] Sync-Protokoll (Request-Response)
- [x] Event-Sync (GetEventsAfter, GetEventsByIds)
- [x] Trust-State-Abfragen
- [x] Membership-Verification
- [x] Offline-Cache Konzept (Fjall-Integration pending)

### Phase 5: Trust-Gate & Anomaly (Woche 6-7) ✅

- [x] TrustGate-System
- [x] ConnectionLevel (Blocked → Trusted)
- [x] Ban-Mechanismus
- [x] Failure-Reporting (Minor/Major/Critical)
- [x] Rate-Limiting Konzept

### Phase 6: Integration (Woche 8) 🚧

- [ ] EventEngine-Integration (Event-DAG-Sync)
- [ ] TrustEngine-Integration (Attestation-Sync)
- [ ] Fjall-DB Delta-Sync (Merkle-Proofs)
- [ ] Mana-Kosten-Enforcement

### Phase 7: Testing & Optimierung 📋

- [ ] Multi-Node-Tests (Docker Compose)
- [ ] Performance-Benchmarks (Events/s)
- [ ] Anomaly-Detection-Tests
- [ ] Sybil-Resistenz-Tests
- [ ] Offline/Reconnect-Tests

### Phase 8: Production-Ready 📋

- [ ] Bootstrap-Node-Deployment
- [ ] Monitoring & Telemetry
- [ ] Circuit-Breaker für Netzwerk-Partitionen
- [ ] Geographic Distribution

---

## Sicherheit & Gaming-Resistenz

### Authentifizierung

| Mechanismus            | Details                                                   |
| ---------------------- | --------------------------------------------------------- |
| **Signierte Messages** | Jede Message Ed25519-signiert → Replay/Spoofing unmöglich |
| **Replay-Protection**  | Timestamps + Message-IDs (Content-Hash)                   |
| **DID-Binding**        | PeerId direkt aus DID abgeleitet → Identitätsnachweis     |

### Trust-Gated Connections

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    TRUST-GATED CONNECTION FLOW                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Incoming Connection                                                    │
│         │                                                               │
│         ▼                                                               │
│  ┌──────────────┐                                                       │
│  │ Trust Lookup │──────> Trust.R < 0.1 ──────> REJECT + BAN             │
│  └──────┬───────┘                                                       │
│         │                                                               │
│         ▼                                                               │
│  Trust.R 0.1-0.5 ────────────────────────────> LIMITED (nur lesen)     │
│         │                                                               │
│  Trust.R 0.5-0.7 ────────────────────────────> STANDARD (send/sync)    │
│         │                                                               │
│  Trust.R 0.7-0.9 + Ω ≥ 0.5 ──────────────────> FULL (+ Relay)          │
│         │                                                               │
│  Trust.R ≥ 0.9 + Ω ≥ 2.0 ────────────────────> TRUSTED (Bootstrap)     │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Anomaly-Integration

| Anomaly                    | Reaktion                              |
| -------------------------- | ------------------------------------- |
| Zu viele Messages/Minute   | Velocity-Alert → Rate-Limit           |
| Ungültige Event-Signaturen | Critical Failure → Trust \* 0.5 + Ban |
| Unerreichbare Parents      | Major Failure → Trust \* 0.9          |
| Sync-Flood                 | Temporärer Ban (5 Min)                |
| Sybil-Verdacht             | DHT-Entry ablehnen                    |

### Sybil-Schutz

- **DHT-Entries:** Nur von Peers mit Trust.R > 0.5 akzeptieren
- **Bootstrap-Verifizierung:** Signierte PeerInfo mit Timestamp
- **Reputation-Akkumulation:** Newcomer-Grace-Period (60s), danach Trust erforderlich

### Privatsphäre

| Feature                     | Implementierung                                           |
| --------------------------- | --------------------------------------------------------- |
| **Private Realms**          | Direct-Topics (`/erynoa/direct/{sender}/{receiver}`)      |
| **Encrypted Payloads**      | Optional Payload-Encryption (ChaCha20-Poly1305)           |
| **Lokale Berechnung**       | Trust/𝔼-Werte nie gesynct, nur lokal aus Events berechnet |
| **PR5 Schlüssel-Isolation** | Signing-Key ≠ Encryption-Key                              |

### Mana-Kosten für Sync

Spam-Prevention durch Mana-Costs:

| Operation              | Mana-Kosten           |
| ---------------------- | --------------------- |
| Event-Publish (Gossip) | 10 Mana               |
| Sync-Request (Pull)    | 5 Mana pro 100 Events |
| DHT-Put                | 20 Mana               |
| Blueprint-Publish      | 50 Mana               |

---

## Referenzen

### Externe Dokumentation

- [libp2p Rust Documentation](https://docs.rs/libp2p/)
- [Gossipsub Specification](https://github.com/libp2p/specs/tree/master/pubsub/gossipsub)
- [Kademlia DHT](https://docs.rs/libp2p/latest/libp2p/kad/)

### Erynoa Axiome

| Axiom                            | Relevanz für P2P                       |
| -------------------------------- | -------------------------------------- |
| **Κ9 (Kausale Struktur)**        | Event-DAG über P2P synchronisiert      |
| **Κ10 (Bezeugung-Finalität)**    | Attestationen via Gossipsub propagiert |
| **Κ15–Κ17 (Consensus-Finality)** | Witness-Consensus bei Sync-Konflikten  |
| **Κ19 (Anti-Verkalkung)**        | Power-Cap bei Peer-Connections         |
| **Κ23 (Gateway)**                | Realm-Join via P2P + Policy-Check      |
| **Κ26 (Anomaly Detection)**      | Velocity-Alerts für Sync-Flood         |
| **PR5 (Schlüssel-Isolation)**    | Signing-Key ≠ Encryption-Key           |

### Interne Dokumentation

- [SYSTEM-ARCHITECTURE.md](../../../documentation/concept-v4/SYSTEM-ARCHITECTURE.md)
- [FACHKONZEPT.md](../../../documentation/concept-v4/FACHKONZEPT.md)
- [Trust-Engine](../reference/trust-engine.md)
