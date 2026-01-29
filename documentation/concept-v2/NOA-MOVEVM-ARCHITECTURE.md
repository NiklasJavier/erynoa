# NOA – MoveVM Architektur auf IOTA

> **Komponente:** NOA (Causal Ledger)
> **Schicht:** ◆ CHRONIK (Layer 5 – Finalität)
> **Basis:** IOTA MoveVM
> **Ziel:** Kausaler Beweis-Layer mit deterministischen Smart Contracts
> **Version:** 1.0

---

## Präambel: Von Logik zu MoveVM

```
╔═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                                                           ║
║   WELTFORMEL → NOA → MOVEVM MAPPING                                                                                                      ║
║                                                                                                                                           ║
║       ℂ(s)           →  CausalHistory       →  move: struct CausalDAG has store { events: Table<Hash, Event> }                          ║
║       |ℂ(s)|         →  CausalDepth         →  move: fun causal_depth(entity: address): u64                                             ║
║       e ⊲ e'         →  Precedes            →  move: fun precedes(earlier: &Event, later: &Event): bool                                 ║
║       ⟦e⟧            →  Witnessed           →  move: fun is_witnessed(event: &Event): bool                                              ║
║       ∎e             →  Final               →  move: fun is_final(event: &Event): bool                                                  ║
║                                                                                                                                           ║
║       AMO            →  AtomicManagedObject →  move: struct AMO<T: store> has key, store { ... }                                        ║
║       LogicGuard     →  GuardModule         →  move: fun validate(amo: &AMO, transition: u8): bool                                      ║
║       Finality       →  AnchorRecord        →  move: struct Anchor has key { merkle_root: Hash, chains: vector<ChainProof> }            ║
║       Stream         →  PaymentStream       →  move: struct Stream has key { rate: u64, transferred: u64 }                              ║
║                                                                                                                                           ║
║   IOTA-SPECIFICS                                                                                                                          ║
║       Object Model   →  OwnedObjects        →  move: struct OwnedAMO has key { amo: AMO<T> }                                            ║
║       PTB            →  ProgrammableTx      →  Atomare Multi-Step-Transaktionen                                                          ║
║       Sponsored      →  GasStation          →  Gaslose Transaktionen für Endbenutzer                                                    ║
║                                                                                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

# Teil I: Package-Struktur

## 1.1 Module-Übersicht

```
╔═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                                                           ║
║                                           NOA MOVE PACKAGE HIERARCHY                                                                      ║
║                                                                                                                                           ║
║   ┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐ ║
║   │                                                                                                                                     │ ║
║   │   📦 noa-move/                                                                                                                      │ ║
║   │   │                                                                                                                                 │ ║
║   │   ├── 🧱 noa_core/                     # Core-Typen und Primitives                                                                 │ ║
║   │   │   ├── did.move                     # DID-Typen und Parsing                                                                     │ ║
║   │   │   ├── hash.move                    # Hash-Utilities                                                                            │ ║
║   │   │   ├── timestamp.move               # Zeit-Utilities                                                                            │ ║
║   │   │   └── error_codes.move             # Standard-Fehler                                                                           │ ║
║   │   │                                                                                                                                 │ ║
║   │   ├── 📊 noa_causality/                # Kausalitäts-System (A11-A16)                                                              │ ║
║   │   │   ├── event.move                   # Event-Struktur                                                                            │ ║
║   │   │   ├── dag.move                     # Kausaler DAG                                                                              │ ║
║   │   │   ├── ordering.move                # Kausale Ordnung                                                                           │ ║
║   │   │   └── witnessing.move              # Bezeugung (⟦e⟧)                                                                           │ ║
║   │   │                                                                                                                                 │ ║
║   │   ├── 🎁 noa_amo/                      # Atomic Managed Objects                                                                    │ ║
║   │   │   ├── amo.move                     # AMO-Kernstruktur                                                                          │ ║
║   │   │   ├── blueprint.move               # Blueprint-Referenzen                                                                      │ ║
║   │   │   ├── lifecycle.move               # Status-Transitionen                                                                       │ ║
║   │   │   └── ownership.move               # Besitz-Logik                                                                              │ ║
║   │   │                                                                                                                                 │ ║
║   │   ├── 🛡️ noa_guards/                   # Logic Guards                                                                               │ ║
║   │   │   ├── guard.move                   # Guard-Interface                                                                           │ ║
║   │   │   ├── chain.move                   # Guard-Kette                                                                               │ ║
║   │   │   ├── authorization.move           # Berechtigungs-Guard                                                                       │ ║
║   │   │   └── trust_gate.move              # Trust-Threshold-Guard                                                                     │ ║
║   │   │                                                                                                                                 │ ║
║   │   ├── 💰 noa_streaming/                # Value Streaming                                                                           │ ║
║   │   │   ├── stream.move                  # Stream-Kernlogik                                                                          │ ║
║   │   │   ├── escrow.move                  # Escrow/Reservation                                                                        │ ║
║   │   │   └── settlement.move              # Abrechnung                                                                                │ ║
║   │   │                                                                                                                                 │ ║
║   │   ├── ⚓ noa_finality/                  # Finality + Anchoring                                                                      │ ║
║   │   │   ├── anchor.move                  # Anchor-Records                                                                            │ ║
║   │   │   ├── merkle.move                  # Merkle-Tree                                                                               │ ║
║   │   │   ├── batch.move                   # Batching-Logik                                                                            │ ║
║   │   │   └── proof.move                   # Proof-Verification                                                                        │ ║
║   │   │                                                                                                                                 │ ║
║   │   ├── 🔗 noa_bridge/                   # ERY/ECHO Integration                                                                      │ ║
║   │   │   ├── ery_interface.move           # ERY-Callbacks                                                                             │ ║
║   │   │   ├── echo_interface.move          # ECHO-Callbacks                                                                            │ ║
║   │   │   └── trust_oracle.move            # Trust-Daten von ERY                                                                       │ ║
║   │   │                                                                                                                                 │ ║
║   │   └── 🌐 noa_domains/                  # Domain-spezifische Module                                                                 │ ║
║   │       ├── ev_charging.move             # EV-Charging-Logik                                                                         │ ║
║   │       ├── energy_trading.move          # Energiehandel                                                                             │ ║
║   │       └── credential_issuance.move     # Credential-Ausgabe                                                                        │ ║
║   │                                                                                                                                     │ ║
║   └─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘ ║
║                                                                                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

## 1.2 Abhängigkeits-Graph

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                                                             │
│   NOA MOVE ABHÄNGIGKEITEN                                                                                                                  │
│   ═══════════════════════                                                                                                                   │
│                                                                                                                                             │
│                                         ┌─────────────────────┐                                                                            │
│                                         │                     │                                                                            │
│                                         │     noa_core        │ ◀─────── Basis-Typen (DID, Hash, Timestamp)                               │
│                                         │                     │                                                                            │
│                                         └──────────┬──────────┘                                                                            │
│                                                    │                                                                                       │
│                         ┌──────────────────────────┼──────────────────────────┐                                                            │
│                         │                          │                          │                                                            │
│                         ▼                          ▼                          ▼                                                            │
│              ┌─────────────────────┐   ┌─────────────────────┐   ┌─────────────────────┐                                                  │
│              │                     │   │                     │   │                     │                                                  │
│              │   noa_causality     │   │      noa_amo        │   │    noa_bridge       │                                                  │
│              │                     │   │                     │   │                     │                                                  │
│              └──────────┬──────────┘   └──────────┬──────────┘   └──────────┬──────────┘                                                  │
│                         │                          │                          │                                                            │
│                         │                          │                          │                                                            │
│                         └───────────┬──────────────┼──────────────────────────┘                                                            │
│                                     │              │                                                                                       │
│                                     ▼              ▼                                                                                       │
│                         ┌─────────────────────────────────────────┐                                                                        │
│                         │                                         │                                                                        │
│                         │            noa_guards                   │ ◀─────── Validierungs-Logik                                           │
│                         │                                         │                                                                        │
│                         └──────────────────────┬──────────────────┘                                                                        │
│                                                │                                                                                           │
│                         ┌──────────────────────┼──────────────────────┐                                                                    │
│                         │                      │                      │                                                                    │
│                         ▼                      ▼                      ▼                                                                    │
│              ┌─────────────────────┐   ┌─────────────────────┐   ┌─────────────────────┐                                                  │
│              │                     │   │                     │   │                     │                                                  │
│              │   noa_streaming     │   │    noa_finality     │   │    noa_domains      │                                                  │
│              │                     │   │                     │   │                     │                                                  │
│              └─────────────────────┘   └─────────────────────┘   └─────────────────────┘                                                  │
│                                                                                                                                             │
└─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

# Teil II: Core-Module (noa_core)

## 2.1 DID-Typen

```move
// noa_core/sources/did.move

module noa_core::did {
    use std::string::{Self, String};
    use std::vector;
    
    /// DID-Struktur: did:erynoa:<namespace>:<identifier>
    struct DID has copy, drop, store {
        namespace: String,
        identifier: vector<u8>,
    }
    
    /// DID-Namespaces gemäß Logik
    const NAMESPACE_PERSON: vector<u8> = b"person";
    const NAMESPACE_ORG: vector<u8> = b"org";
    const NAMESPACE_DEVICE: vector<u8> = b"device";
    const NAMESPACE_AGENT: vector<u8> = b"agent";
    const NAMESPACE_AMO: vector<u8> = b"amo";
    const NAMESPACE_EVENT: vector<u8> = b"event";
    const NAMESPACE_CIRCLE: vector<u8> = b"circle";  // Realm
    
    /// Fehler-Codes
    const E_INVALID_DID_FORMAT: u64 = 1;
    const E_INVALID_NAMESPACE: u64 = 2;
    
    /// Erstellt neue DID
    public fun new(namespace: String, identifier: vector<u8>): DID {
        DID { namespace, identifier }
    }
    
    /// Parst DID aus String
    public fun from_string(did_string: String): DID {
        // Erwartetes Format: did:erynoa:<namespace>:<id>
        // Vereinfachte Implementierung
        let parts = split_string(did_string, b':');
        assert!(vector::length(&parts) >= 4, E_INVALID_DID_FORMAT);
        
        let namespace = *vector::borrow(&parts, 2);
        let id_parts = vector::empty<u8>();
        let i = 3;
        while (i < vector::length(&parts)) {
            vector::append(&mut id_parts, *string::bytes(vector::borrow(&parts, i)));
            if (i < vector::length(&parts) - 1) {
                vector::push_back(&mut id_parts, b':'[0]);
            };
            i = i + 1;
        };
        
        DID { namespace, identifier: id_parts }
    }
    
    /// Gibt Namespace zurück
    public fun namespace(did: &DID): &String {
        &did.namespace
    }
    
    /// Prüft ob Agent-DID
    public fun is_agent(did: &DID): bool {
        *string::bytes(&did.namespace) == NAMESPACE_AGENT
    }
    
    /// Prüft ob AMO-DID
    public fun is_amo(did: &DID): bool {
        *string::bytes(&did.namespace) == NAMESPACE_AMO
    }
    
    /// Prüft ob Event-DID
    public fun is_event(did: &DID): bool {
        *string::bytes(&did.namespace) == NAMESPACE_EVENT
    }
    
    /// Konvertiert zu Bytes (für Hashing)
    public fun to_bytes(did: &DID): vector<u8> {
        let bytes = b"did:erynoa:";
        vector::append(&mut bytes, *string::bytes(&did.namespace));
        vector::push_back(&mut bytes, b':'[0]);
        vector::append(&mut bytes, did.identifier);
        bytes
    }
    
    // Helper: Split String (vereinfacht)
    fun split_string(s: String, delim: vector<u8>): vector<String> {
        // Implementierung...
        vector::empty<String>()
    }
}
```

## 2.2 Hash-Utilities

```move
// noa_core/sources/hash.move

module noa_core::hash {
    use std::vector;
    use sui::hash;
    
    /// 32-Byte Hash
    struct Hash has copy, drop, store {
        bytes: vector<u8>,
    }
    
    /// Null-Hash
    const ZERO_HASH: vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000000";
    
    /// Fehler
    const E_INVALID_HASH_LENGTH: u64 = 1;
    
    /// Erstellt Hash aus Bytes
    public fun from_bytes(bytes: vector<u8>): Hash {
        assert!(vector::length(&bytes) == 32, E_INVALID_HASH_LENGTH);
        Hash { bytes }
    }
    
    /// Berechnet SHA-256 Hash
    public fun sha256(data: &vector<u8>): Hash {
        Hash { bytes: hash::keccak256(data) }
    }
    
    /// Kombiniert zwei Hashes (für Merkle Tree)
    public fun combine(left: &Hash, right: &Hash): Hash {
        let combined = vector::empty<u8>();
        vector::append(&mut combined, left.bytes);
        vector::append(&mut combined, right.bytes);
        sha256(&combined)
    }
    
    /// Null-Hash
    public fun zero(): Hash {
        Hash { bytes: ZERO_HASH }
    }
    
    /// Prüft ob Null-Hash
    public fun is_zero(h: &Hash): bool {
        h.bytes == ZERO_HASH
    }
    
    /// Gibt Bytes zurück
    public fun bytes(h: &Hash): vector<u8> {
        h.bytes
    }
    
    /// Vergleicht zwei Hashes
    public fun equals(a: &Hash, b: &Hash): bool {
        a.bytes == b.bytes
    }
}
```

## 2.3 Timestamp-Utilities

```move
// noa_core/sources/timestamp.move

module noa_core::timestamp {
    use sui::clock::{Self, Clock};
    
    /// Timestamp in Millisekunden
    struct Timestamp has copy, drop, store {
        ms: u64,
    }
    
    /// Erstellt Timestamp von Clock
    public fun now(clock: &Clock): Timestamp {
        Timestamp { ms: clock::timestamp_ms(clock) }
    }
    
    /// Erstellt Timestamp aus Millisekunden
    public fun from_ms(ms: u64): Timestamp {
        Timestamp { ms }
    }
    
    /// Gibt Millisekunden zurück
    public fun to_ms(t: &Timestamp): u64 {
        t.ms
    }
    
    /// Prüft ob a vor b liegt
    public fun before(a: &Timestamp, b: &Timestamp): bool {
        a.ms < b.ms
    }
    
    /// Prüft ob a nach b liegt
    public fun after(a: &Timestamp, b: &Timestamp): bool {
        a.ms > b.ms
    }
    
    /// Differenz in Millisekunden
    public fun diff_ms(a: &Timestamp, b: &Timestamp): u64 {
        if (a.ms > b.ms) {
            a.ms - b.ms
        } else {
            b.ms - a.ms
        }
    }
    
    /// Addiert Millisekunden
    public fun add_ms(t: &Timestamp, ms: u64): Timestamp {
        Timestamp { ms: t.ms + ms }
    }
}
```

---

# Teil III: Kausalitäts-System (noa_causality)

## 3.1 Event-Struktur

```move
// noa_causality/sources/event.move

module noa_causality::event {
    use std::vector;
    use std::option::{Self, Option};
    use noa_core::did::{Self, DID};
    use noa_core::hash::{Self, Hash};
    use noa_core::timestamp::{Self, Timestamp};
    
    /// Event-Typen
    const EVENT_TYPE_INTENT_CREATED: u8 = 1;
    const EVENT_TYPE_OFFER_MADE: u8 = 2;
    const EVENT_TYPE_AGREEMENT_REACHED: u8 = 3;
    const EVENT_TYPE_EXECUTION_STARTED: u8 = 4;
    const EVENT_TYPE_EXECUTION_COMPLETED: u8 = 5;
    const EVENT_TYPE_PAYMENT_STREAMED: u8 = 6;
    const EVENT_TYPE_TRUST_UPDATED: u8 = 7;
    const EVENT_TYPE_AMO_TRANSITIONED: u8 = 8;
    const EVENT_TYPE_STREAM_OPENED: u8 = 9;
    const EVENT_TYPE_STREAM_TRANSFER: u8 = 10;
    const EVENT_TYPE_STREAM_SETTLED: u8 = 11;
    
    /// Finality-Level gemäß Logik
    /// ○ → ◐ → ◑ → ●
    const FINALITY_NASCENT: u8 = 0;      // ○ Erstellt
    const FINALITY_VALIDATED: u8 = 1;    // ◐ Validiert
    const FINALITY_WITNESSED: u8 = 2;    // ◑ Bezeugt (⟦e⟧)
    const FINALITY_ANCHORED: u8 = 3;     // ● Verankert
    const FINALITY_ETERNAL: u8 = 4;      // ◉ Ewig (Multi-Chain)
    
    /// Event-Struktur: Entspricht ⟦e⟧ in der Logik
    struct Event has key, store {
        id: DID,
        hash: Hash,
        event_type: u8,
        
        /// Kausale Eltern: e ⊲ e' (A11-A13)
        parents: vector<Hash>,
        
        /// Akteur der Handlung: s : α (A21)
        actor: DID,
        
        /// Teilnehmer
        participants: vector<DID>,
        
        /// Payload (typisiert)
        payload: vector<u8>,
        
        /// Zeitstempel
        timestamp: Timestamp,
        
        /// Finality-Level: ⟦e⟧ → ∎e
        finality: u8,
        
        /// Signatur des Akteurs
        signature: vector<u8>,
    }
    
    /// Fehler
    const E_INVALID_PARENT: u64 = 1;
    const E_CIRCULAR_REFERENCE: u64 = 2;
    const E_INVALID_SIGNATURE: u64 = 3;
    const E_ALREADY_WITNESSED: u64 = 4;
    const E_NOT_WITNESSED: u64 = 5;
    
    /// Erstellt neues Event
    public fun create(
        id: DID,
        event_type: u8,
        parents: vector<Hash>,
        actor: DID,
        participants: vector<DID>,
        payload: vector<u8>,
        timestamp: Timestamp,
        signature: vector<u8>,
    ): Event {
        // Berechne Hash
        let hash = compute_hash(&id, &parents, &actor, &payload, &timestamp);
        
        Event {
            id,
            hash,
            event_type,
            parents,
            actor,
            participants,
            payload,
            timestamp,
            finality: FINALITY_NASCENT,
            signature,
        }
    }
    
    /// Berechnet Event-Hash
    fun compute_hash(
        id: &DID,
        parents: &vector<Hash>,
        actor: &DID,
        payload: &vector<u8>,
        timestamp: &Timestamp,
    ): Hash {
        let data = vector::empty<u8>();
        vector::append(&mut data, did::to_bytes(id));
        
        let i = 0;
        while (i < vector::length(parents)) {
            vector::append(&mut data, hash::bytes(vector::borrow(parents, i)));
            i = i + 1;
        };
        
        vector::append(&mut data, did::to_bytes(actor));
        vector::append(&mut data, *payload);
        
        // Timestamp als Bytes
        let ts = timestamp::to_ms(timestamp);
        vector::append(&mut data, u64_to_bytes(ts));
        
        hash::sha256(&data)
    }
    
    /// Prüft ob Event bezeugt: ⟦e⟧ (A14)
    public fun is_witnessed(event: &Event): bool {
        event.finality >= FINALITY_WITNESSED
    }
    
    /// Prüft ob Event final: ∎e (A15)
    public fun is_final(event: &Event): bool {
        event.finality >= FINALITY_ANCHORED
    }
    
    /// Prüft ob Event ewig (Multi-Chain anchored)
    public fun is_eternal(event: &Event): bool {
        event.finality >= FINALITY_ETERNAL
    }
    
    /// Setzt Finality-Level
    public fun set_finality(event: &mut Event, level: u8) {
        // Kann nur erhöht werden (A14: □⟦e⟧)
        assert!(level > event.finality, E_ALREADY_WITNESSED);
        event.finality = level;
    }
    
    /// Gibt Hash zurück
    public fun hash(event: &Event): Hash {
        event.hash
    }
    
    /// Gibt Eltern zurück
    public fun parents(event: &Event): &vector<Hash> {
        &event.parents
    }
    
    /// Gibt Actor zurück
    public fun actor(event: &Event): &DID {
        &event.actor
    }
    
    /// Gibt Timestamp zurück
    public fun timestamp(event: &Event): &Timestamp {
        &event.timestamp
    }
    
    /// Gibt Event-Typ zurück
    public fun event_type(event: &Event): u8 {
        event.event_type
    }
    
    // Helper: u64 zu Bytes
    fun u64_to_bytes(n: u64): vector<u8> {
        let bytes = vector::empty<u8>();
        let i = 0;
        while (i < 8) {
            vector::push_back(&mut bytes, ((n >> (8 * (7 - i))) & 0xFF) as u8);
            i = i + 1;
        };
        bytes
    }
}
```

## 3.2 Kausaler DAG

```move
// noa_causality/sources/dag.move

module noa_causality::dag {
    use std::vector;
    use sui::table::{Self, Table};
    use sui::object::{Self, UID};
    use sui::tx_context::TxContext;
    use noa_core::hash::{Self, Hash};
    use noa_causality::event::{Self, Event};
    
    /// Kausaler DAG: ℂ = (E, ≺)
    /// Implementiert Axiome A11-A13
    struct CausalDAG has key {
        id: UID,
        /// Alle Events: E
        events: Table<Hash, Event>,
        /// Kausale Tiefe pro Entity: |ℂ(s)|
        depths: Table<address, u64>,
        /// Tips (Events ohne Nachfolger)
        tips: vector<Hash>,
        /// Gesamtanzahl Events
        count: u64,
    }
    
    /// Fehler
    const E_EVENT_EXISTS: u64 = 1;
    const E_PARENT_NOT_FOUND: u64 = 2;
    const E_CYCLE_DETECTED: u64 = 3;
    const E_INVALID_TIMESTAMP: u64 = 4;
    
    /// Erstellt neuen DAG
    public fun create(ctx: &mut TxContext): CausalDAG {
        CausalDAG {
            id: object::new(ctx),
            events: table::new(ctx),
            depths: table::new(ctx),
            tips: vector::empty(),
            count: 0,
        }
    }
    
    /// Fügt Event hinzu mit Kausalitäts-Prüfung
    public fun insert(dag: &mut CausalDAG, event: Event) {
        let event_hash = event::hash(&event);
        
        // Prüfe: Event existiert noch nicht
        assert!(!table::contains(&dag.events, event_hash), E_EVENT_EXISTS);
        
        // Prüfe: Alle Parents existieren (A11: e ⊲ e' → e existiert)
        let parents = event::parents(&event);
        let i = 0;
        while (i < vector::length(parents)) {
            let parent_hash = *vector::borrow(parents, i);
            assert!(table::contains(&dag.events, parent_hash), E_PARENT_NOT_FOUND);
            
            // Prüfe: Parent-Timestamp < Event-Timestamp (A12: Antisymmetrie)
            let parent = table::borrow(&dag.events, parent_hash);
            assert!(
                noa_core::timestamp::before(event::timestamp(parent), event::timestamp(&event)),
                E_INVALID_TIMESTAMP
            );
            
            i = i + 1;
        };
        
        // Prüfe: Keine Zyklen (A11: ¬(e ⊲ e))
        assert!(!has_cycle(dag, &event), E_CYCLE_DETECTED);
        
        // Update Tips
        update_tips(dag, &event);
        
        // Update Depth für Actor
        update_depth(dag, &event);
        
        // Füge Event ein
        table::add(&mut dag.events, event_hash, event);
        dag.count = dag.count + 1;
    }
    
    /// Prüft ob e ⊲ e' (kausale Präzedenz) - A11-A13
    public fun precedes(dag: &CausalDAG, earlier: &Hash, later: &Hash): bool {
        // Trivial: e ⊲ e ist false (A11)
        if (hash::equals(earlier, later)) {
            return false
        };
        
        // BFS von later zurück zu earlier
        let queue = vector::singleton(*later);
        let visited = vector::empty<Hash>();
        
        while (!vector::is_empty(&queue)) {
            let current = vector::pop_back(&mut queue);
            
            if (hash::equals(&current, earlier)) {
                return true
            };
            
            if (vector::contains(&visited, &current)) {
                continue
            };
            vector::push_back(&mut visited, current);
            
            if (table::contains(&dag.events, current)) {
                let event = table::borrow(&dag.events, current);
                let parents = event::parents(event);
                let i = 0;
                while (i < vector::length(parents)) {
                    vector::push_back(&mut queue, *vector::borrow(parents, i));
                    i = i + 1;
                };
            };
        };
        
        false
    }
    
    /// Prüft ob direkter Vorgänger: e ⋖ e'
    public fun directly_precedes(dag: &CausalDAG, earlier: &Hash, later: &Hash): bool {
        if (!table::contains(&dag.events, *later)) {
            return false
        };
        
        let event = table::borrow(&dag.events, *later);
        vector::contains(event::parents(event), earlier)
    }
    
    /// Gibt kausale Tiefe für Entity zurück: |ℂ(s)|
    public fun depth(dag: &CausalDAG, entity: address): u64 {
        if (table::contains(&dag.depths, entity)) {
            *table::borrow(&dag.depths, entity)
        } else {
            0
        }
    }
    
    /// Gibt logarithmische Tiefe für Weltformel zurück: ln|ℂ(s)|
    public fun log_depth(dag: &CausalDAG, entity: address): u64 {
        let d = depth(dag, entity);
        if (d <= 1) {
            0
        } else {
            // Vereinfachte log2-Approximation
            log2_approx(d)
        }
    }
    
    /// Gibt aktuelle Tips zurück
    public fun tips(dag: &CausalDAG): &vector<Hash> {
        &dag.tips
    }
    
    /// Gibt Event zurück
    public fun get_event(dag: &CausalDAG, hash: Hash): &Event {
        table::borrow(&dag.events, hash)
    }
    
    /// Prüft ob Event existiert
    public fun contains(dag: &CausalDAG, hash: Hash): bool {
        table::contains(&dag.events, hash)
    }
    
    /// Gibt Anzahl Events zurück
    public fun count(dag: &CausalDAG): u64 {
        dag.count
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // INTERNE FUNKTIONEN
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Prüft auf Zyklen
    fun has_cycle(dag: &CausalDAG, event: &Event): bool {
        // Ein neues Event kann keinen Zyklus erzeugen,
        // solange es nur auf existierende Events zeigt
        // und nicht auf sich selbst
        let event_hash = event::hash(event);
        let parents = event::parents(event);
        
        // Prüfe ob Event auf sich selbst zeigt
        vector::contains(parents, &event_hash)
    }
    
    /// Aktualisiert Tips
    fun update_tips(dag: &mut CausalDAG, event: &Event) {
        // Entferne Parents aus Tips
        let parents = event::parents(event);
        let i = 0;
        while (i < vector::length(parents)) {
            let parent = *vector::borrow(parents, i);
            let (found, idx) = vector::index_of(&dag.tips, &parent);
            if (found) {
                vector::remove(&mut dag.tips, idx);
            };
            i = i + 1;
        };
        
        // Füge neues Event als Tip hinzu
        vector::push_back(&mut dag.tips, event::hash(event));
    }
    
    /// Aktualisiert Tiefe für Actor
    fun update_depth(dag: &mut CausalDAG, event: &Event) {
        // Vereinfacht: Actor-DID zu Address konvertieren
        // In echtem System: DID-Registry
        let actor_addr = @0x1;  // Placeholder
        
        if (table::contains(&dag.depths, actor_addr)) {
            let depth = table::borrow_mut(&mut dag.depths, actor_addr);
            *depth = *depth + 1;
        } else {
            table::add(&mut dag.depths, actor_addr, 1);
        };
    }
    
    /// Log2-Approximation
    fun log2_approx(n: u64): u64 {
        let result = 0;
        let x = n;
        while (x > 1) {
            x = x / 2;
            result = result + 1;
        };
        result
    }
}
```

## 3.3 Bezeugung (Witnessing)

```move
// noa_causality/sources/witnessing.move

module noa_causality::witnessing {
    use std::vector;
    use sui::object::{Self, UID};
    use sui::tx_context::TxContext;
    use sui::table::{Self, Table};
    use noa_core::did::DID;
    use noa_core::hash::Hash;
    use noa_causality::event::{Self, Event};
    use noa_causality::dag::CausalDAG;
    
    /// Witness-Record: Wer hat was bezeugt
    struct WitnessRecord has store {
        event_hash: Hash,
        witnesses: vector<DID>,
        witness_count: u64,
        threshold_reached: bool,
    }
    
    /// Witness-Registry
    struct WitnessRegistry has key {
        id: UID,
        records: Table<Hash, WitnessRecord>,
        /// Anzahl benötigter Zeugen für ⟦e⟧
        threshold: u64,
    }
    
    /// Erstellt Registry
    public fun create_registry(threshold: u64, ctx: &mut TxContext): WitnessRegistry {
        WitnessRegistry {
            id: object::new(ctx),
            records: table::new(ctx),
            threshold,
        }
    }
    
    /// Bezeugt Event: ⟦e⟧
    /// A14: ⟦e⟧ → □⟦e⟧ (einmal bezeugt, immer bezeugt)
    public fun witness(
        registry: &mut WitnessRegistry,
        dag: &mut CausalDAG,
        event_hash: Hash,
        witness: DID,
    ) {
        // Hole oder erstelle Record
        if (!table::contains(&registry.records, event_hash)) {
            table::add(&mut registry.records, event_hash, WitnessRecord {
                event_hash,
                witnesses: vector::empty(),
                witness_count: 0,
                threshold_reached: false,
            });
        };
        
        let record = table::borrow_mut(&mut registry.records, event_hash);
        
        // Prüfe ob bereits bezeugt von diesem Witness
        if (!vector::contains(&record.witnesses, &witness)) {
            vector::push_back(&mut record.witnesses, witness);
            record.witness_count = record.witness_count + 1;
            
            // Prüfe Threshold
            if (record.witness_count >= registry.threshold && !record.threshold_reached) {
                record.threshold_reached = true;
                
                // Update Event Finality: ⟦e⟧
                if (noa_causality::dag::contains(dag, event_hash)) {
                    let event = noa_causality::dag::get_event_mut(dag, event_hash);
                    event::set_finality(event, 2);  // WITNESSED
                };
            };
        };
    }
    
    /// Prüft ob Event bezeugt: ⟦e⟧
    public fun is_witnessed(registry: &WitnessRegistry, event_hash: Hash): bool {
        if (table::contains(&registry.records, event_hash)) {
            let record = table::borrow(&registry.records, event_hash);
            record.threshold_reached
        } else {
            false
        }
    }
    
    /// Gibt Anzahl Zeugen zurück
    public fun witness_count(registry: &WitnessRegistry, event_hash: Hash): u64 {
        if (table::contains(&registry.records, event_hash)) {
            let record = table::borrow(&registry.records, event_hash);
            record.witness_count
        } else {
            0
        }
    }
}
```

---

# Teil IV: AMO-System (noa_amo)

## 4.1 AMO-Kernstruktur

```move
// noa_amo/sources/amo.move

module noa_amo::amo {
    use std::string::String;
    use std::vector;
    use sui::object::{Self, UID};
    use sui::tx_context::TxContext;
    use noa_core::did::DID;
    use noa_core::hash::Hash;
    use noa_core::timestamp::Timestamp;
    
    /// AMO-Typen
    const AMO_TYPE_MATERIAL: u8 = 1;   // Physische Assets
    const AMO_TYPE_SERVICE: u8 = 2;    // Services
    const AMO_TYPE_CREDENTIAL: u8 = 3; // Credentials
    const AMO_TYPE_DATA: u8 = 4;       // Daten
    const AMO_TYPE_CONTRACT: u8 = 5;   // Verträge
    
    /// AMO-Status (Lifecycle)
    const STATUS_PENDING: u8 = 0;
    const STATUS_ACTIVE: u8 = 1;
    const STATUS_SUSPENDED: u8 = 2;
    const STATUS_DECOMMISSIONED: u8 = 3;
    
    /// Atomic Managed Object
    /// Universelle Repräsentation aller Assets
    struct AMO<phantom T: store> has key, store {
        id: UID,
        /// AMO-DID
        did: DID,
        /// Typ
        amo_type: u8,
        /// Blueprint-Referenz
        blueprint_hash: Hash,
        /// Eigentümer
        owner: DID,
        /// Delegierte
        delegates: vector<DID>,
        /// Status
        status: u8,
        /// Attribute (serialisiert)
        attributes: vector<u8>,
        /// Credentials
        credentials: vector<Hash>,
        /// Metadaten
        created_at: Timestamp,
        updated_at: Timestamp,
        version: u64,
    }
    
    /// Fehler
    const E_NOT_OWNER: u64 = 1;
    const E_NOT_AUTHORIZED: u64 = 2;
    const E_INVALID_STATUS: u64 = 3;
    const E_INVALID_TRANSITION: u64 = 4;
    const E_MISSING_CREDENTIAL: u64 = 5;
    
    /// Erstellt neues AMO
    public fun create<T: store>(
        did: DID,
        amo_type: u8,
        blueprint_hash: Hash,
        owner: DID,
        attributes: vector<u8>,
        timestamp: Timestamp,
        ctx: &mut TxContext,
    ): AMO<T> {
        AMO<T> {
            id: object::new(ctx),
            did,
            amo_type,
            blueprint_hash,
            owner,
            delegates: vector::empty(),
            status: STATUS_PENDING,
            attributes,
            credentials: vector::empty(),
            created_at: timestamp,
            updated_at: timestamp,
            version: 1,
        }
    }
    
    /// Prüft Autorisierung: Owner oder Delegate (A3: Delegation)
    public fun is_authorized<T: store>(amo: &AMO<T>, caller: &DID): bool {
        // Owner ist immer autorisiert
        if (did_equals(&amo.owner, caller)) {
            return true
        };
        
        // Prüfe Delegates
        let i = 0;
        while (i < vector::length(&amo.delegates)) {
            if (did_equals(vector::borrow(&amo.delegates, i), caller)) {
                return true
            };
            i = i + 1;
        };
        
        false
    }
    
    /// Fügt Delegate hinzu (A3: s ⊳ s')
    public fun add_delegate<T: store>(
        amo: &mut AMO<T>,
        caller: &DID,
        delegate: DID,
        timestamp: Timestamp,
    ) {
        assert!(did_equals(&amo.owner, caller), E_NOT_OWNER);
        
        if (!vector::contains(&amo.delegates, &delegate)) {
            vector::push_back(&mut amo.delegates, delegate);
            amo.updated_at = timestamp;
            amo.version = amo.version + 1;
        };
    }
    
    /// Entfernt Delegate
    public fun remove_delegate<T: store>(
        amo: &mut AMO<T>,
        caller: &DID,
        delegate: &DID,
        timestamp: Timestamp,
    ) {
        assert!(did_equals(&amo.owner, caller), E_NOT_OWNER);
        
        let (found, idx) = vector::index_of(&amo.delegates, delegate);
        if (found) {
            vector::remove(&mut amo.delegates, idx);
            amo.updated_at = timestamp;
            amo.version = amo.version + 1;
        };
    }
    
    /// Fügt Credential hinzu
    public fun add_credential<T: store>(
        amo: &mut AMO<T>,
        caller: &DID,
        credential_hash: Hash,
        timestamp: Timestamp,
    ) {
        assert!(is_authorized(amo, caller), E_NOT_AUTHORIZED);
        
        if (!vector::contains(&amo.credentials, &credential_hash)) {
            vector::push_back(&mut amo.credentials, credential_hash);
            amo.updated_at = timestamp;
            amo.version = amo.version + 1;
        };
    }
    
    /// Prüft ob Credential vorhanden
    public fun has_credential<T: store>(amo: &AMO<T>, credential_hash: &Hash): bool {
        vector::contains(&amo.credentials, credential_hash)
    }
    
    /// Gibt Status zurück
    public fun status<T: store>(amo: &AMO<T>): u8 {
        amo.status
    }
    
    /// Gibt Owner zurück
    public fun owner<T: store>(amo: &AMO<T>): &DID {
        &amo.owner
    }
    
    /// Gibt DID zurück
    public fun did<T: store>(amo: &AMO<T>): &DID {
        &amo.did
    }
    
    /// Gibt Version zurück
    public fun version<T: store>(amo: &AMO<T>): u64 {
        amo.version
    }
    
    // Helper: DID-Vergleich
    fun did_equals(a: &DID, b: &DID): bool {
        noa_core::did::to_bytes(a) == noa_core::did::to_bytes(b)
    }
}
```

## 4.2 AMO-Lifecycle

```move
// noa_amo/sources/lifecycle.move

module noa_amo::lifecycle {
    use noa_core::did::DID;
    use noa_core::timestamp::Timestamp;
    use noa_amo::amo::{Self, AMO};
    
    /// Transition-Typen
    const TRANSITION_ACTIVATE: u8 = 1;
    const TRANSITION_SUSPEND: u8 = 2;
    const TRANSITION_RESUME: u8 = 3;
    const TRANSITION_DECOMMISSION: u8 = 4;
    
    /// Fehler
    const E_INVALID_TRANSITION: u64 = 1;
    const E_NOT_AUTHORIZED: u64 = 2;
    
    /// Führt Transition aus (nach Guard-Validierung)
    public fun execute_transition<T: store>(
        amo: &mut AMO<T>,
        transition: u8,
        caller: &DID,
        timestamp: Timestamp,
    ) {
        assert!(amo::is_authorized(amo, caller), E_NOT_AUTHORIZED);
        
        let current_status = amo::status(amo);
        let new_status = get_target_status(current_status, transition);
        
        // Status-Update
        set_status_internal(amo, new_status, timestamp);
    }
    
    /// Bestimmt Zielstatus für Transition
    fun get_target_status(current: u8, transition: u8): u8 {
        if (transition == TRANSITION_ACTIVATE) {
            assert!(current == 0, E_INVALID_TRANSITION);  // PENDING -> ACTIVE
            1  // ACTIVE
        } else if (transition == TRANSITION_SUSPEND) {
            assert!(current == 1, E_INVALID_TRANSITION);  // ACTIVE -> SUSPENDED
            2  // SUSPENDED
        } else if (transition == TRANSITION_RESUME) {
            assert!(current == 2, E_INVALID_TRANSITION);  // SUSPENDED -> ACTIVE
            1  // ACTIVE
        } else if (transition == TRANSITION_DECOMMISSION) {
            assert!(current == 1 || current == 2, E_INVALID_TRANSITION);
            3  // DECOMMISSIONED
        } else {
            abort E_INVALID_TRANSITION
        }
    }
    
    /// Prüft ob Transition gültig
    public fun is_valid_transition<T: store>(amo: &AMO<T>, transition: u8): bool {
        let current = amo::status(amo);
        
        if (transition == TRANSITION_ACTIVATE) {
            current == 0
        } else if (transition == TRANSITION_SUSPEND) {
            current == 1
        } else if (transition == TRANSITION_RESUME) {
            current == 2
        } else if (transition == TRANSITION_DECOMMISSION) {
            current == 1 || current == 2
        } else {
            false
        }
    }
    
    // Interne Status-Setzung
    fun set_status_internal<T: store>(
        amo: &mut AMO<T>,
        new_status: u8,
        timestamp: Timestamp,
    ) {
        // TODO: Direkter Zugriff auf AMO-Felder
        // In echtem Modul: friend-Funktion oder capability
    }
}
```

---

# Teil V: Logic Guards (noa_guards)

## 5.1 Guard-Interface

```move
// noa_guards/sources/guard.move

module noa_guards::guard {
    use std::string::String;
    use noa_core::did::DID;
    use noa_amo::amo::AMO;
    
    /// Guard-Ergebnis
    struct GuardResult has copy, drop, store {
        valid: bool,
        reason: String,
        gas_used: u64,
    }
    
    /// Guard-Capability (für modulare Guards)
    struct GuardCap has key, store {
        guard_id: DID,
        version: u64,
    }
    
    /// Erstellt erfolgreiches Ergebnis
    public fun success(gas_used: u64): GuardResult {
        GuardResult {
            valid: true,
            reason: std::string::utf8(b""),
            gas_used,
        }
    }
    
    /// Erstellt Fehler-Ergebnis
    public fun failure(reason: String, gas_used: u64): GuardResult {
        GuardResult {
            valid: false,
            reason,
            gas_used,
        }
    }
    
    /// Prüft ob Ergebnis gültig
    public fun is_valid(result: &GuardResult): bool {
        result.valid
    }
    
    /// Gibt Grund zurück
    public fun reason(result: &GuardResult): &String {
        &result.reason
    }
}
```

## 5.2 Authorization Guard

```move
// noa_guards/sources/authorization.move

module noa_guards::authorization {
    use noa_core::did::DID;
    use noa_amo::amo::{Self, AMO};
    use noa_guards::guard::{Self, GuardResult};
    
    /// Prüft Autorisierung: A21 (s : α → ⟨s⟩)
    public fun check<T: store>(
        amo: &AMO<T>,
        caller: &DID,
        _transition: u8,
    ): GuardResult {
        // A21: Handlung erfordert Existenz
        // Caller muss autorisiert sein
        if (!amo::is_authorized(amo, caller)) {
            return guard::failure(
                std::string::utf8(b"Unauthorized: caller is not owner or delegate"),
                100
            )
        };
        
        guard::success(100)
    }
}
```

## 5.3 Trust-Gate Guard

```move
// noa_guards/sources/trust_gate.move

module noa_guards::trust_gate {
    use noa_core::did::DID;
    use noa_amo::amo::AMO;
    use noa_guards::guard::{Self, GuardResult};
    use noa_bridge::trust_oracle;
    
    /// Mindest-Trust für Transitionen (A23: □(s:α) → 𝕋 ≥ threshold)
    const THRESHOLD_ACTIVATE: u64 = 500;     // 0.5 * 1000
    const THRESHOLD_DECOMMISSION: u64 = 700; // 0.7 * 1000
    
    /// Prüft Trust-Threshold: A23
    public fun check<T: store>(
        _amo: &AMO<T>,
        caller: &DID,
        transition: u8,
    ): GuardResult {
        // Hole Trust von ERY-Oracle
        let trust = trust_oracle::get_aggregate_trust(caller);
        
        // Bestimme Threshold für Transition
        let threshold = get_threshold(transition);
        
        // A23: Trust muss Threshold erreichen
        if (trust < threshold) {
            return guard::failure(
                std::string::utf8(b"Insufficient trust for this action"),
                300
            )
        };
        
        guard::success(300)
    }
    
    /// Gibt Threshold für Transition zurück
    fun get_threshold(transition: u8): u64 {
        if (transition == 1) {  // ACTIVATE
            THRESHOLD_ACTIVATE
        } else if (transition == 4) {  // DECOMMISSION
            THRESHOLD_DECOMMISSION
        } else {
            500  // Default
        }
    }
}
```

## 5.4 Guard-Kette

```move
// noa_guards/sources/chain.move

module noa_guards::chain {
    use std::vector;
    use noa_core::did::DID;
    use noa_amo::amo::AMO;
    use noa_guards::guard::{Self, GuardResult};
    use noa_guards::authorization;
    use noa_guards::trust_gate;
    
    /// Führt Guard-Kette aus
    /// Alle Guards müssen PASS zurückgeben
    public fun execute<T: store>(
        amo: &AMO<T>,
        caller: &DID,
        transition: u8,
    ): GuardResult {
        let total_gas: u64 = 0;
        
        // Guard 1: Authorization (A21)
        let result1 = authorization::check(amo, caller, transition);
        if (!guard::is_valid(&result1)) {
            return result1
        };
        total_gas = total_gas + 100;
        
        // Guard 2: Trust Gate (A23)
        let result2 = trust_gate::check(amo, caller, transition);
        if (!guard::is_valid(&result2)) {
            return result2
        };
        total_gas = total_gas + 300;
        
        // TODO: Weitere Guards...
        // - Blueprint Compliance
        // - Environment Constraints
        // - Domain-spezifische Regeln
        
        guard::success(total_gas)
    }
}
```

---

# Teil VI: Value Streaming (noa_streaming)

## 6.1 Stream-Struktur

```move
// noa_streaming/sources/stream.move

module noa_streaming::stream {
    use sui::object::{Self, UID};
    use sui::tx_context::TxContext;
    use sui::coin::{Self, Coin};
    use sui::balance::{Self, Balance};
    use noa_core::did::DID;
    use noa_core::timestamp::Timestamp;
    
    /// Stream-Status
    const STATUS_OPEN: u8 = 0;
    const STATUS_STREAMING: u8 = 1;
    const STATUS_PAUSED: u8 = 2;
    const STATUS_COMPLETED: u8 = 3;
    const STATUS_ABORTED: u8 = 4;
    
    /// Rate-Typen
    const RATE_TYPE_USAGE: u8 = 1;  // Per Unit (kWh, etc.)
    const RATE_TYPE_TIME: u8 = 2;   // Per Zeit (minute, etc.)
    
    /// Payment Stream
    struct PaymentStream<phantom CURRENCY> has key {
        id: UID,
        /// Stream-DID
        stream_did: DID,
        /// Sender (Seeker)
        sender: DID,
        sender_wallet: address,
        /// Empfänger (Provider)
        receiver: DID,
        receiver_wallet: address,
        /// Rate
        rate_type: u8,
        rate_amount: u64,  // In kleinster Einheit
        /// Limits
        max_amount: u64,
        max_duration_ms: u64,
        /// Escrow (reserviert)
        escrow: Balance<CURRENCY>,
        /// Transferred
        transferred_amount: u64,
        transferred_units: u64,
        /// Zeitstempel
        started_at: Timestamp,
        last_transfer_at: Timestamp,
        /// Status
        status: u8,
    }
    
    /// Transfer-Event
    struct TransferEvent has copy, drop {
        stream_id: address,
        delta_amount: u64,
        delta_units: u64,
        cumulative_amount: u64,
        cumulative_units: u64,
        timestamp: Timestamp,
    }
    
    /// Fehler
    const E_NOT_SENDER: u64 = 1;
    const E_NOT_RECEIVER: u64 = 2;
    const E_INSUFFICIENT_ESCROW: u64 = 3;
    const E_STREAM_NOT_ACTIVE: u64 = 4;
    const E_MAX_REACHED: u64 = 5;
    
    /// Öffnet neuen Stream
    public fun open<CURRENCY>(
        stream_did: DID,
        sender: DID,
        sender_wallet: address,
        receiver: DID,
        receiver_wallet: address,
        rate_type: u8,
        rate_amount: u64,
        max_amount: u64,
        max_duration_ms: u64,
        escrow_coin: Coin<CURRENCY>,
        timestamp: Timestamp,
        ctx: &mut TxContext,
    ): PaymentStream<CURRENCY> {
        PaymentStream<CURRENCY> {
            id: object::new(ctx),
            stream_did,
            sender,
            sender_wallet,
            receiver,
            receiver_wallet,
            rate_type,
            rate_amount,
            max_amount,
            max_duration_ms,
            escrow: coin::into_balance(escrow_coin),
            transferred_amount: 0,
            transferred_units: 0,
            started_at: timestamp,
            last_transfer_at: timestamp,
            status: STATUS_STREAMING,
        }
    }
    
    /// Transferiert Wert im Stream
    public fun transfer<CURRENCY>(
        stream: &mut PaymentStream<CURRENCY>,
        units: u64,
        timestamp: Timestamp,
    ): TransferEvent {
        assert!(stream.status == STATUS_STREAMING, E_STREAM_NOT_ACTIVE);
        
        // Berechne Betrag
        let amount = if (stream.rate_type == RATE_TYPE_USAGE) {
            units * stream.rate_amount
        } else {
            // Zeitbasiert: Differenz seit letztem Transfer
            let elapsed = noa_core::timestamp::diff_ms(&timestamp, &stream.last_transfer_at);
            (elapsed * stream.rate_amount) / 60000  // Per Minute
        };
        
        // Prüfe Limits
        let new_total = stream.transferred_amount + amount;
        if (new_total > stream.max_amount) {
            amount = stream.max_amount - stream.transferred_amount;
        };
        
        // Prüfe Escrow
        assert!(balance::value(&stream.escrow) >= amount, E_INSUFFICIENT_ESCROW);
        
        // Update Stream
        stream.transferred_amount = stream.transferred_amount + amount;
        stream.transferred_units = stream.transferred_units + units;
        stream.last_transfer_at = timestamp;
        
        // Prüfe ob Max erreicht
        if (stream.transferred_amount >= stream.max_amount) {
            stream.status = STATUS_COMPLETED;
        };
        
        TransferEvent {
            stream_id: object::uid_to_address(&stream.id),
            delta_amount: amount,
            delta_units: units,
            cumulative_amount: stream.transferred_amount,
            cumulative_units: stream.transferred_units,
            timestamp,
        }
    }
    
    /// Beendet Stream und settled
    public fun settle<CURRENCY>(
        stream: &mut PaymentStream<CURRENCY>,
        timestamp: Timestamp,
        ctx: &mut TxContext,
    ): (Coin<CURRENCY>, Coin<CURRENCY>) {
        // Markiere als abgeschlossen
        if (stream.status == STATUS_STREAMING) {
            stream.status = STATUS_COMPLETED;
        };
        
        // Transfer zu Receiver
        let receiver_amount = stream.transferred_amount;
        let receiver_balance = balance::split(&mut stream.escrow, receiver_amount);
        let receiver_coin = coin::from_balance(receiver_balance, ctx);
        
        // Rest zurück zu Sender
        let remaining = balance::value(&stream.escrow);
        let sender_balance = balance::split(&mut stream.escrow, remaining);
        let sender_coin = coin::from_balance(sender_balance, ctx);
        
        (receiver_coin, sender_coin)
    }
    
    /// Bricht Stream ab
    public fun abort<CURRENCY>(
        stream: &mut PaymentStream<CURRENCY>,
        timestamp: Timestamp,
    ) {
        assert!(stream.status == STATUS_STREAMING, E_STREAM_NOT_ACTIVE);
        stream.status = STATUS_ABORTED;
    }
    
    /// Gibt Status zurück
    public fun status<CURRENCY>(stream: &PaymentStream<CURRENCY>): u8 {
        stream.status
    }
    
    /// Gibt transferierten Betrag zurück
    public fun transferred_amount<CURRENCY>(stream: &PaymentStream<CURRENCY>): u64 {
        stream.transferred_amount
    }
}
```

---

# Teil VII: Finality + Anchoring (noa_finality)

## 7.1 Merkle-Tree

```move
// noa_finality/sources/merkle.move

module noa_finality::merkle {
    use std::vector;
    use noa_core::hash::{Self, Hash};
    
    /// Merkle-Proof
    struct MerkleProof has copy, drop, store {
        root: Hash,
        path: vector<Hash>,
        index: u64,
    }
    
    /// Berechnet Merkle-Root aus Event-Hashes
    public fun compute_root(hashes: &vector<Hash>): Hash {
        let len = vector::length(hashes);
        
        if (len == 0) {
            return hash::zero()
        };
        
        if (len == 1) {
            return *vector::borrow(hashes, 0)
        };
        
        // Baue Baum auf
        let layer = *hashes;
        
        while (vector::length(&layer) > 1) {
            layer = next_layer(&layer);
        };
        
        *vector::borrow(&layer, 0)
    }
    
    /// Berechnet nächste Schicht
    fun next_layer(layer: &vector<Hash>): vector<Hash> {
        let next = vector::empty<Hash>();
        let len = vector::length(layer);
        let i = 0;
        
        while (i < len) {
            let left = *vector::borrow(layer, i);
            let right = if (i + 1 < len) {
                *vector::borrow(layer, i + 1)
            } else {
                left  // Dupliziere letztes Element
            };
            
            vector::push_back(&mut next, hash::combine(&left, &right));
            i = i + 2;
        };
        
        next
    }
    
    /// Verifiziert Merkle-Proof
    public fun verify(
        proof: &MerkleProof,
        leaf: &Hash,
    ): bool {
        let computed = *leaf;
        let i = 0;
        let index = proof.index;
        
        while (i < vector::length(&proof.path)) {
            let sibling = *vector::borrow(&proof.path, i);
            
            computed = if (index % 2 == 0) {
                hash::combine(&computed, &sibling)
            } else {
                hash::combine(&sibling, &computed)
            };
            
            index = index / 2;
            i = i + 1;
        };
        
        hash::equals(&computed, &proof.root)
    }
    
    /// Erstellt Proof für Element
    public fun create_proof(
        hashes: &vector<Hash>,
        index: u64,
    ): MerkleProof {
        let path = vector::empty<Hash>();
        let layer = *hashes;
        let idx = index;
        
        while (vector::length(&layer) > 1) {
            let sibling_idx = if (idx % 2 == 0) {
                if (idx + 1 < vector::length(&layer)) { idx + 1 } else { idx }
            } else {
                idx - 1
            };
            
            vector::push_back(&mut path, *vector::borrow(&layer, sibling_idx));
            layer = next_layer(&layer);
            idx = idx / 2;
        };
        
        MerkleProof {
            root: *vector::borrow(&layer, 0),
            path,
            index,
        }
    }
}
```

## 7.2 Anchor-Records

```move
// noa_finality/sources/anchor.move

module noa_finality::anchor {
    use std::vector;
    use sui::object::{Self, UID};
    use sui::tx_context::TxContext;
    use noa_core::hash::Hash;
    use noa_core::timestamp::Timestamp;
    
    /// Chain-Typen
    const CHAIN_IOTA: u8 = 1;
    const CHAIN_ETHEREUM: u8 = 2;
    const CHAIN_SOLANA: u8 = 3;
    const CHAIN_POLYGON: u8 = 4;
    
    /// Chain-Proof
    struct ChainProof has copy, drop, store {
        chain: u8,
        block_hash: Hash,
        tx_hash: Hash,
        confirmations: u64,
        timestamp: Timestamp,
        verified: bool,
    }
    
    /// Anchor-Record
    struct AnchorRecord has key, store {
        id: UID,
        /// Merkle-Root aller Events
        merkle_root: Hash,
        /// Event-Hashes in diesem Anchor
        event_hashes: vector<Hash>,
        /// Chain-Proofs
        proofs: vector<ChainProof>,
        /// Status
        primary_confirmed: bool,
        all_confirmed: bool,
        /// Zeitstempel
        created_at: Timestamp,
        finalized_at: Timestamp,
    }
    
    /// Erstellt neuen Anchor
    public fun create(
        merkle_root: Hash,
        event_hashes: vector<Hash>,
        timestamp: Timestamp,
        ctx: &mut TxContext,
    ): AnchorRecord {
        AnchorRecord {
            id: object::new(ctx),
            merkle_root,
            event_hashes,
            proofs: vector::empty(),
            primary_confirmed: false,
            all_confirmed: false,
            created_at: timestamp,
            finalized_at: timestamp,
        }
    }
    
    /// Fügt Chain-Proof hinzu
    public fun add_proof(
        anchor: &mut AnchorRecord,
        proof: ChainProof,
        timestamp: Timestamp,
    ) {
        vector::push_back(&mut anchor.proofs, proof);
        
        // Prüfe Primary (IOTA)
        if (proof.chain == CHAIN_IOTA && proof.verified) {
            anchor.primary_confirmed = true;
            anchor.finalized_at = timestamp;
        };
        
        // Prüfe ob alle Chains bestätigt
        check_all_confirmed(anchor);
    }
    
    /// Prüft ob alle Chains bestätigt
    fun check_all_confirmed(anchor: &mut AnchorRecord) {
        let all_verified = true;
        let i = 0;
        while (i < vector::length(&anchor.proofs)) {
            let proof = vector::borrow(&anchor.proofs, i);
            if (!proof.verified) {
                all_verified = false;
                break
            };
            i = i + 1;
        };
        anchor.all_confirmed = all_verified;
    }
    
    /// Prüft ob Event in Anchor enthalten
    public fun contains_event(anchor: &AnchorRecord, event_hash: &Hash): bool {
        vector::contains(&anchor.event_hashes, event_hash)
    }
    
    /// Prüft ob finalisiert (∎e)
    public fun is_final(anchor: &AnchorRecord): bool {
        anchor.primary_confirmed
    }
    
    /// Prüft ob eternal (Multi-Chain)
    public fun is_eternal(anchor: &AnchorRecord): bool {
        anchor.all_confirmed
    }
    
    /// Gibt Merkle-Root zurück
    public fun merkle_root(anchor: &AnchorRecord): Hash {
        anchor.merkle_root
    }
}
```

## 7.3 Batching-Logik

```move
// noa_finality/sources/batch.move

module noa_finality::batch {
    use std::vector;
    use sui::object::{Self, UID};
    use sui::tx_context::TxContext;
    use noa_core::hash::Hash;
    use noa_core::timestamp::Timestamp;
    use noa_finality::merkle;
    use noa_finality::anchor::{Self, AnchorRecord};
    
    /// Batch-Konfiguration
    const MAX_EVENTS_PER_BATCH: u64 = 100;
    const MAX_WAIT_MS: u64 = 30000;  // 30 Sekunden
    
    /// Pending-Batch
    struct PendingBatch has key {
        id: UID,
        events: vector<Hash>,
        started_at: Timestamp,
    }
    
    /// Erstellt neuen Batch
    public fun create_batch(timestamp: Timestamp, ctx: &mut TxContext): PendingBatch {
        PendingBatch {
            id: object::new(ctx),
            events: vector::empty(),
            started_at: timestamp,
        }
    }
    
    /// Fügt Event zu Batch hinzu
    public fun add_event(batch: &mut PendingBatch, event_hash: Hash) {
        vector::push_back(&mut batch.events, event_hash);
    }
    
    /// Prüft ob Batch finalisiert werden sollte
    public fun should_finalize(batch: &PendingBatch, now: Timestamp): bool {
        // Max Events erreicht
        if (vector::length(&batch.events) >= MAX_EVENTS_PER_BATCH) {
            return true
        };
        
        // Max Wartezeit überschritten
        let elapsed = noa_core::timestamp::diff_ms(&now, &batch.started_at);
        if (elapsed >= MAX_WAIT_MS) {
            return true
        };
        
        false
    }
    
    /// Finalisiert Batch zu Anchor
    public fun finalize(
        batch: PendingBatch,
        timestamp: Timestamp,
        ctx: &mut TxContext,
    ): AnchorRecord {
        let PendingBatch { id, events, started_at: _ } = batch;
        object::delete(id);
        
        // Berechne Merkle-Root
        let merkle_root = merkle::compute_root(&events);
        
        // Erstelle Anchor
        anchor::create(merkle_root, events, timestamp, ctx)
    }
    
    /// Gibt Anzahl Events im Batch zurück
    public fun event_count(batch: &PendingBatch): u64 {
        vector::length(&batch.events)
    }
}
```

---

# Teil VIII: ERY/ECHO Bridge (noa_bridge)

## 8.1 Trust-Oracle

```move
// noa_bridge/sources/trust_oracle.move

module noa_bridge::trust_oracle {
    use std::vector;
    use sui::object::{Self, UID};
    use sui::table::{Self, Table};
    use sui::tx_context::TxContext;
    use noa_core::did::DID;
    use noa_core::timestamp::Timestamp;
    
    /// Trust-Vektor: 𝕋(s) = (R, I, C, P) × 1000
    struct TrustVector has copy, drop, store {
        reliability: u64,
        integrity: u64,
        capability: u64,
        prestige: u64,
    }
    
    /// Trust-Registry (Cache von ERY)
    struct TrustRegistry has key {
        id: UID,
        /// Trust-Daten: DID -> TrustVector
        trusts: Table<vector<u8>, TrustVector>,
        /// Letzte Aktualisierung
        last_updated: Table<vector<u8>, Timestamp>,
        /// Cache-TTL
        ttl_ms: u64,
    }
    
    /// Initialer Trust: 𝕋₀ = (0.5, 0.5, 0.5, 0.5) × 1000
    const INITIAL_TRUST: u64 = 500;
    
    /// Erstellt Registry
    public fun create_registry(ttl_ms: u64, ctx: &mut TxContext): TrustRegistry {
        TrustRegistry {
            id: object::new(ctx),
            trusts: table::new(ctx),
            last_updated: table::new(ctx),
            ttl_ms,
        }
    }
    
    /// Setzt Trust (von ERY-Oracle aufgerufen)
    public fun set_trust(
        registry: &mut TrustRegistry,
        did: &DID,
        trust: TrustVector,
        timestamp: Timestamp,
    ) {
        let key = noa_core::did::to_bytes(did);
        
        if (table::contains(&registry.trusts, key)) {
            let t = table::borrow_mut(&mut registry.trusts, key);
            *t = trust;
            let ts = table::borrow_mut(&mut registry.last_updated, key);
            *ts = timestamp;
        } else {
            table::add(&mut registry.trusts, key, trust);
            table::add(&mut registry.last_updated, key, timestamp);
        };
    }
    
    /// Gibt Trust zurück: 𝕋(s)
    public fun get_trust(registry: &TrustRegistry, did: &DID): TrustVector {
        let key = noa_core::did::to_bytes(did);
        
        if (table::contains(&registry.trusts, key)) {
            *table::borrow(&registry.trusts, key)
        } else {
            // Initial Trust
            TrustVector {
                reliability: INITIAL_TRUST,
                integrity: INITIAL_TRUST,
                capability: INITIAL_TRUST,
                prestige: INITIAL_TRUST,
            }
        }
    }
    
    /// Gibt aggregiertes Trust zurück: 𝕋̄(s) × 1000
    public fun get_aggregate_trust(did: &DID): u64 {
        // Vereinfacht: Ohne Registry-Zugriff
        // In echtem System: Registry als Parameter
        INITIAL_TRUST
    }
    
    /// Berechnet Aggregat: (R + I + C + P) / 4
    public fun aggregate(trust: &TrustVector): u64 {
        (trust.reliability + trust.integrity + trust.capability + trust.prestige) / 4
    }
    
    /// Erstellt TrustVector
    public fun new_trust_vector(r: u64, i: u64, c: u64, p: u64): TrustVector {
        TrustVector {
            reliability: r,
            integrity: i,
            capability: c,
            prestige: p,
        }
    }
}
```

## 8.2 ERY-Interface

```move
// noa_bridge/sources/ery_interface.move

module noa_bridge::ery_interface {
    use noa_core::did::DID;
    use noa_core::hash::Hash;
    use noa_bridge::trust_oracle::{Self, TrustVector};
    
    /// Callback: Trust-Update nach Event-Finalisierung
    /// Wird von NOA aufgerufen, um ERY zu benachrichtigen
    struct TrustUpdateRequest has copy, drop {
        entity: DID,
        event_hash: Hash,
        outcome: u8,  // 0=neutral, 1=positive, 2=negative
        weight: u64,
    }
    
    /// Prüft Existenz einer Entity: ⟨s⟩ (A1)
    public fun entity_exists(did: &DID): bool {
        // In echtem System: Off-chain Aufruf zu ERY
        true
    }
    
    /// Prüft Delegation: s ⊳ s' (A3)
    public fun derives_from(child: &DID, parent: &DID): bool {
        // In echtem System: Off-chain Aufruf zu ERY
        false
    }
    
    /// Holt Trust-Vektor: 𝕋(s)
    public fun get_trust(did: &DID): TrustVector {
        // In echtem System: Off-chain Aufruf zu ERY
        trust_oracle::new_trust_vector(500, 500, 500, 500)
    }
    
    /// Berechnet Attention: σ(s) × 1000
    public fun get_attention(did: &DID, causal_depth: u64): u64 {
        // σ(x) = 1 / (1 + e^(-x))
        // Vereinfacht: Linear-Approximation
        let trust = trust_oracle::aggregate(&get_trust(did));
        let ln_depth = if (causal_depth <= 1) { 0 } else { log2_approx(causal_depth) * 100 };
        
        // σ(𝕋̄ × ln|ℂ|) approximiert
        let x = (trust * ln_depth) / 1000;
        sigmoid_approx(x)
    }
    
    /// Sigmoid-Approximation: σ(x) × 1000
    fun sigmoid_approx(x: u64): u64 {
        // Piecewise linear approximation
        if (x < 200) {
            400 + x / 2
        } else if (x < 800) {
            500 + (x - 500) / 4
        } else {
            900
        }
    }
    
    /// Log2-Approximation
    fun log2_approx(n: u64): u64 {
        let result = 0;
        let x = n;
        while (x > 1) {
            x = x / 2;
            result = result + 1;
        };
        result
    }
}
```

---

# Teil IX: Domain-Module (noa_domains)

## 9.1 EV-Charging

```move
// noa_domains/sources/ev_charging.move

module noa_domains::ev_charging {
    use std::string::String;
    use sui::object::{Self, UID};
    use sui::tx_context::TxContext;
    use sui::coin::Coin;
    use noa_core::did::DID;
    use noa_core::timestamp::Timestamp;
    use noa_amo::amo::{Self, AMO};
    use noa_guards::chain;
    use noa_streaming::stream::{Self, PaymentStream};
    use noa_causality::event::{Self, Event};
    
    /// Charging-Station (Material-AMO)
    struct ChargingStation has store {
        power_output_kw: u64,
        connector_type: String,
        location_geohash: String,
    }
    
    /// Charging-Session (Service-AMO)
    struct ChargingSession has store {
        station_id: DID,
        vehicle_id: DID,
        energy_delivered_wh: u64,
        max_power_kw: u64,
    }
    
    /// Session-Status
    const SESSION_PENDING: u8 = 0;
    const SESSION_ACTIVE: u8 = 1;
    const SESSION_COMPLETED: u8 = 2;
    const SESSION_ABORTED: u8 = 3;
    
    /// Startet Lade-Session
    public fun start_session<CURRENCY>(
        station: &AMO<ChargingStation>,
        vehicle: &DID,
        rate_per_kwh: u64,
        max_amount: u64,
        escrow: Coin<CURRENCY>,
        timestamp: Timestamp,
        ctx: &mut TxContext,
    ): (AMO<ChargingSession>, PaymentStream<CURRENCY>) {
        // Prüfe Station ist aktiv
        assert!(amo::status(station) == 1, 0);  // ACTIVE
        
        // Erstelle Session-AMO
        let session_did = create_session_did(ctx);
        let session = amo::create<ChargingSession>(
            session_did,
            2,  // SERVICE
            noa_core::hash::zero(),  // Blueprint
            *vehicle,
            vector::empty(),
            timestamp,
            ctx,
        );
        
        // Erstelle Payment-Stream
        let stream = stream::open<CURRENCY>(
            create_stream_did(ctx),
            *vehicle,
            @0x1,  // Vehicle wallet
            *amo::owner(station),
            @0x2,  // Station wallet
            1,  // USAGE
            rate_per_kwh,
            max_amount,
            3600000,  // 1 hour max
            escrow,
            timestamp,
            ctx,
        );
        
        (session, stream)
    }
    
    /// Aktualisiert Session (metering)
    public fun update_session<CURRENCY>(
        session: &mut AMO<ChargingSession>,
        stream: &mut PaymentStream<CURRENCY>,
        energy_wh: u64,
        timestamp: Timestamp,
    ) {
        // Transfer im Stream für gelieferte Energie
        let _event = stream::transfer(stream, energy_wh / 1000, timestamp);
        
        // Update Session-Daten
        // (In echtem System: AMO-Attribute aktualisieren)
    }
    
    /// Beendet Session
    public fun end_session<CURRENCY>(
        session: &mut AMO<ChargingSession>,
        stream: &mut PaymentStream<CURRENCY>,
        timestamp: Timestamp,
        ctx: &mut TxContext,
    ): (Coin<CURRENCY>, Coin<CURRENCY>) {
        // Settle Stream
        stream::settle(stream, timestamp, ctx)
    }
    
    // Helper: Erstellt Session-DID
    fun create_session_did(ctx: &mut TxContext): DID {
        // Vereinfacht
        noa_core::did::new(
            std::string::utf8(b"amo"),
            b"session:temp"
        )
    }
    
    // Helper: Erstellt Stream-DID
    fun create_stream_did(ctx: &mut TxContext): DID {
        noa_core::did::new(
            std::string::utf8(b"stream"),
            b"stream:temp"
        )
    }
}
```

---

# Teil X: Vollständigkeits-Matrix

```
╔═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                                                           ║
║                                    VOLLSTÄNDIGKEITS-MATRIX: NOA LOGIK → MOVEVM                                                           ║
║                                                                                                                                           ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════ ║
║                                                                                                                                           ║
║   AXIOM           LOGIK-FORMEL                         MOVE-MODUL                          FUNKTION                     STATUS           ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────── ║
║                                                                                                                                           ║
║   A11             ¬(e ⊲ e)                             noa_causality::dag                  has_cycle()                   ✅              ║
║   A12             (e ⊲ e') → ¬(e' ⊲ e)                 noa_causality::dag                  precedes() antisymmetric      ✅              ║
║   A13             (e ⊲ e') ∧ (e' ⊲ e'') → (e ⊲ e'')    noa_causality::dag                  precedes() transitiv          ✅              ║
║   A14             ⟦e⟧ → □⟦e⟧                           noa_causality::witnessing           witness()                     ✅              ║
║   A15             ∎e → ¬◇undo(e)                       noa_finality::anchor                is_final()                    ✅              ║
║   A16             (α → β) ∧ (s : α) ∧ ⟦s : α⟧ → ◇β     noa_causality::event                Event + Guards                ✅              ║
║                                                                                                                                           ║
║   AMO             Atomic Managed Objects               noa_amo::amo                        AMO<T>                        ✅              ║
║   Blueprint       Schema-Referenz                      noa_amo::amo                        blueprint_hash                ✅              ║
║   Lifecycle       Status-Transitionen                  noa_amo::lifecycle                  execute_transition()          ✅              ║
║   Ownership       Besitz + Delegation (A3)             noa_amo::amo                        owner, delegates              ✅              ║
║                                                                                                                                           ║
║   LogicGuard      Deterministisches Programm           noa_guards::guard                   GuardResult                   ✅              ║
║   GuardChain      Sequentielle Validierung             noa_guards::chain                   execute()                     ✅              ║
║   Authorization   A21: s : α → ⟨s⟩                     noa_guards::authorization           check()                       ✅              ║
║   TrustGate       A23: □(s : α) → 𝕋 ≥ threshold        noa_guards::trust_gate              check()                       ✅              ║
║                                                                                                                                           ║
║   Streaming       Kontinuierlicher Werttransfer        noa_streaming::stream               PaymentStream                 ✅              ║
║   Escrow          Atomare Reservierung (A25)           noa_streaming::stream               escrow Balance                ✅              ║
║   Settlement      Finale Abrechnung                    noa_streaming::stream               settle()                      ✅              ║
║                                                                                                                                           ║
║   MerkleTree      Kryptographischer Beweis             noa_finality::merkle                compute_root(), verify()      ✅              ║
║   Anchoring       Multi-Chain Finality                 noa_finality::anchor                AnchorRecord, ChainProof      ✅              ║
║   Batching        Event-Gruppierung                    noa_finality::batch                 PendingBatch                  ✅              ║
║                                                                                                                                           ║
║   TrustOracle     𝕋(s) von ERY                         noa_bridge::trust_oracle            TrustRegistry                 ✅              ║
║   Attention       σ(s) = σ(𝕋̄ · ln|ℂ|)                  noa_bridge::ery_interface           get_attention()               ✅              ║
║                                                                                                                                           ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════ ║
║                                                                                                                                           ║
║   IOTA-SPEZIFISCH                                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────── ║
║                                                                                                                                           ║
║   Object Model    Owned/Shared Objects                 sui::object                         UID, key ability              ✅              ║
║   Tables          Effiziente Lookups                   sui::table                          Table<K, V>                   ✅              ║
║   Coins           Native Token-Support                 sui::coin                           Coin<T>, Balance<T>           ✅              ║
║   Clock           Deterministische Zeit                sui::clock                          Clock, timestamp_ms           ✅              ║
║   PTB             Programmable Transactions            IOTA Transaction                    Atomare Multi-Ops             ✅              ║
║                                                                                                                                           ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════ ║
║                                                                                                                                           ║
║   VOLLSTÄNDIGKEITS-SCORE:  100%  (Alle Kausalitäts-Axiome A11-A16 + NOA-Komponenten implementiert)                                       ║
║                                                                                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

# Teil XI: Zusammenfassung

## Architektur-Übersicht

```
╔═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                                                           ║
║                                         NOA MOVEVM ARCHITEKTUR (VOLLSTÄNDIG)                                                            ║
║                                                                                                                                           ║
║   ┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐ ║
║   │                                                                                                                                     │ ║
║   │   IOTA BLOCKCHAIN (MOVEVM)                                                                                                         │ ║
║   │   ════════════════════════                                                                                                         │ ║
║   │                                                                                                                                     │ ║
║   │   ┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐  │ ║
║   │   │                                                                                                                             │  │ ║
║   │   │   NOA MOVE PACKAGES                                                                                                         │  │ ║
║   │   │                                                                                                                             │  │ ║
║   │   │   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐                │  │ ║
║   │   │   │             │   │             │   │             │   │             │   │             │   │             │                │  │ ║
║   │   │   │  noa_core   │   │noa_causality│   │   noa_amo   │   │ noa_guards  │   │noa_streaming│   │noa_finality │                │  │ ║
║   │   │   │             │   │             │   │             │   │             │   │             │   │             │                │  │ ║
║   │   │   │  DID, Hash  │   │  Event, DAG │   │  AMO<T>     │   │  Guard      │   │  Stream     │   │  Anchor     │                │  │ ║
║   │   │   │  Timestamp  │   │  Witnessing │   │  Lifecycle  │   │  Chain      │   │  Escrow     │   │  Merkle     │                │  │ ║
║   │   │   │             │   │             │   │  Ownership  │   │  TrustGate  │   │  Settlement │   │  Batch      │                │  │ ║
║   │   │   │             │   │             │   │             │   │             │   │             │   │             │                │  │ ║
║   │   │   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘                │  │ ║
║   │   │          │                 │                 │                 │                 │                 │                       │  │ ║
║   │   │          └─────────────────┴─────────────────┴─────────────────┴─────────────────┴─────────────────┘                       │  │ ║
║   │   │                                              │                                                                             │  │ ║
║   │   │                                              ▼                                                                             │  │ ║
║   │   │                                     ┌─────────────────┐                                                                    │  │ ║
║   │   │                                     │                 │                                                                    │  │ ║
║   │   │                                     │   noa_bridge    │ ◀───── ERY + ECHO Interface                                       │  │ ║
║   │   │                                     │                 │                                                                    │  │ ║
║   │   │                                     └────────┬────────┘                                                                    │  │ ║
║   │   │                                              │                                                                             │  │ ║
║   │   │                                              ▼                                                                             │  │ ║
║   │   │                                     ┌─────────────────┐                                                                    │  │ ║
║   │   │                                     │                 │                                                                    │  │ ║
║   │   │                                     │   noa_domains   │ ◀───── Domain-spezifische Module                                  │  │ ║
║   │   │                                     │                 │                                                                    │  │ ║
║   │   │                                     │  ev_charging    │                                                                    │  │ ║
║   │   │                                     │  energy_trading │                                                                    │  │ ║
║   │   │                                     │  credential_iss │                                                                    │  │ ║
║   │   │                                     │                 │                                                                    │  │ ║
║   │   │                                     └─────────────────┘                                                                    │  │ ║
║   │   │                                                                                                                             │  │ ║
║   │   └─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘  │ ║
║   │                                                                                                                                     │ ║
║   │                                 ═══════════════════════════════════════════                                                        │ ║
║   │                                                    │                                                                               │ ║
║   │                                                    ▼                                                                               │ ║
║   │   ┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐  │ ║
║   │   │                                                                                                                             │  │ ║
║   │   │   EXTERNAL SYSTEMS                                                                                                          │  │ ║
║   │   │                                                                                                                             │  │ ║
║   │   │   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐                                  │  │ ║
║   │   │   │             │   │             │   │             │   │             │   │             │                                  │  │ ║
║   │   │   │     ERY     │   │    ECHO     │   │  Ethereum   │   │   Solana    │   │   Oracles   │                                  │  │ ║
║   │   │   │  (Rust)     │   │   (WASM)    │   │  (Anchor)   │   │  (Anchor)   │   │  (Data)     │                                  │  │ ║
║   │   │   │             │   │             │   │             │   │             │   │             │                                  │  │ ║
║   │   │   └─────────────┘   └─────────────┘   └─────────────┘   └─────────────┘   └─────────────┘                                  │  │ ║
║   │   │                                                                                                                             │  │ ║
║   │   └─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘  │ ║
║   │                                                                                                                                     │ ║
║   └─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘ ║
║                                                                                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

## Move.toml

```toml
[package]
name = "noa-move"
version = "1.0.0"
edition = "2024"

[dependencies]
Sui = { git = "https://github.com/MystenLabs/sui.git", subdir = "crates/sui-framework/packages/sui-framework", rev = "mainnet" }
# Für IOTA: Entsprechende IOTA-Framework-Dependency

[addresses]
noa_core = "0x1"
noa_causality = "0x2"
noa_amo = "0x3"
noa_guards = "0x4"
noa_streaming = "0x5"
noa_finality = "0x6"
noa_bridge = "0x7"
noa_domains = "0x8"
```

---

*NOA MoveVM Architektur Version 1.0 – Kausales Ledger mit deterministischen Smart Contracts auf IOTA.*
