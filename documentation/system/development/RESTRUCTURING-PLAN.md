# Erynoa Restrukturierungsplan

> **Version:** 1.0.0
> **Datum:** Februar 2026
> **Basis:** UNIFIED-DATA-MODEL.md v1.1.0 + IPS-01-imp.md v1.2.0
> **Ziel:** Vollständige Alignment der Codebase mit UDM/IPS

---

## Executive Summary

Dieser Plan beschreibt die **vollständige Restrukturierung** der Erynoa-Backend-Codebase,
um alle Komponenten auf das Unified Data Model (UDM) v1.1.0 und IPS v1.2.0 auszurichten.

**Kernänderungen:**

1. ExecutionContext Pattern einführen (ersetzt ad-hoc Error-Handling)
2. Adjunktions-basierte Core ↔ ECLVM Traits
3. τ-Variabilität in P2P-Layer
4. InformationLoss-Tracking für alle Kanäle
5. Unified IDs in allen Modulen

**Geschätzter Aufwand:** 4-6 Wochen (1 Entwickler)

---

## I. Ist-Analyse

### 1.1 Aktuelle Modulstruktur

```
backend/src/
├── domain/                    # Datentypen
│   ├── did.rs                 # ✓ Gut strukturiert (Κ6-Κ8)
│   ├── event.rs               # ⚠ Fehlt: UniversalId-Integration
│   ├── formula.rs             # ⚠ Fehlt: Surprisal als f32
│   ├── realm.rs               # ⚠ Fehlt: RealmId als UniversalId
│   ├── saga.rs                # ⚠ Fehlt: Budget-Integration
│   ├── trust.rs               # ⚠ Duplikat mit unified/trust.rs
│   └── unified/               # ✓ Neu, UDM-konform
│       ├── primitives.rs      # ✓ UniversalId, TemporalCoord
│       ├── cost.rs            # ✓ Cost, Budget
│       └── trust.rs           # ✓ TrustVector6D, TrustRecord
├── core/                      # Business-Logik
│   ├── event_engine.rs        # ⚠ Fehlt: ExecutionContext
│   ├── trust_engine.rs        # ⚠ Fehlt: TrustRecord aus unified
│   ├── world_formula.rs       # ⚠ Fehlt: Cost-Algebra
│   ├── surprisal.rs           # ⚠ Fehlt: Surprisal struct
│   └── consensus.rs           # ⚠ Fehlt: FinalityState
├── peer/                      # P2P-Layer
│   └── p2p/                   # ⚠ Fehlt: τ-Variabilität, autonat
│       ├── behaviour.rs
│       ├── protocol.rs
│       └── trust_gate.rs
├── eclvm/                     # ECLVM Runtime
│   └── ...                    # ⚠ Fehlt: CoreToEclvm Traits
└── error.rs                   # ⚠ Fragmentiert, nicht unifiziert
```

### 1.2 Identifizierte Probleme

| Problem                   | Beschreibung                            | Auswirkung                       |
| ------------------------- | --------------------------------------- | -------------------------------- |
| **Doppelte Typen**        | `domain/trust.rs` vs `unified/trust.rs` | Inkonsistente Verwendung         |
| **Fragmentierte Errors**  | Jedes Modul eigene Errors               | Kein unifiziertes Handling       |
| **Kein ExecutionContext** | Ad-hoc State-Passing                    | Schwer zu testen, fehleranfällig |
| **Keine Adjunktion**      | ECLVM ↔ Core hardcoded                  | Keine Beweisbarkeit              |
| **Statisches τ**          | Feste Sync-Timings                      | Nicht adaptiv                    |
| **Kein InformationLoss**  | Kompression undokumentiert              | Keine Audit-Trails               |

---

## II. Ziel-Architektur

### 2.1 Neue Modulstruktur

```
backend/src/
├── domain/
│   ├── mod.rs                 # Re-exports nur aus unified/
│   └── unified/               # SINGLE SOURCE OF TRUTH
│       ├── mod.rs
│       ├── primitives.rs      # UniversalId, TemporalCoord
│       ├── cost.rs            # Cost, Budget, CostTable
│       ├── trust.rs           # TrustVector6D, TrustRecord
│       ├── identity.rs        # DID, DIDDocument, Delegation (NEU)
│       ├── event.rs           # Event, EventId, FinalityState (NEU)
│       ├── realm.rs           # Realm, RealmRules, Partition (NEU)
│       ├── saga.rs            # Saga, Intent, Goal (NEU)
│       ├── formula.rs         # Surprisal, WorldFormulaContrib (NEU)
│       ├── message.rs         # P2P Messages, SyncRequest (NEU)
│       └── error.rs           # ExecutionError (NEU)
├── execution/                 # NEU: Execution Layer
│   ├── mod.rs
│   ├── context.rs             # ExecutionContext
│   ├── adjunction.rs          # CoreToEclvm, EclvmToCore
│   └── information_loss.rs    # InformationLoss, CompressionRecord
├── core/                      # Business-Logik (verwendet execution/)
│   ├── event_engine.rs        # → verwendet ExecutionContext
│   ├── trust_engine.rs        # → verwendet TrustRecord
│   ├── world_formula.rs       # → verwendet Cost-Algebra
│   ├── surprisal.rs           # → verwendet Surprisal struct
│   └── consensus.rs           # → verwendet FinalityState
├── peer/
│   └── p2p/
│       ├── behaviour.rs       # → erweitert um autonat, identify
│       ├── protocol.rs        # → P2PProtocol enum
│       ├── timing.rs          # NEU: NetworkConditions, SyncTiming
│       └── trust_gate.rs      # → verwendet TrustVector6D
├── eclvm/
│   └── bridge.rs              # NEU: Adjunktions-Implementierung
└── error.rs                   # → verweist auf unified/error.rs
```

### 2.2 Abhängigkeitsgraph (Post-Migration)

```
                    ┌─────────────────┐
                    │  unified/       │  ← Single Source of Truth
                    │  (Datentypen)   │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
       ┌──────────┐   ┌───────────┐   ┌──────────┐
       │execution/│   │   core/   │   │  peer/   │
       │(Context) │◄──│ (Engines) │──►│  (P2P)   │
       └──────────┘   └───────────┘   └──────────┘
              │              │              │
              └──────────────┼──────────────┘
                             │
                             ▼
                      ┌───────────┐
                      │  eclvm/   │
                      │(Adjunkt.) │
                      └───────────┘
```

---

## III. Migrations-Phasen

### Phase 1: Foundation (Woche 1)

#### 1.1 ExecutionContext einführen

**Datei:** `backend/src/execution/context.rs` (NEU)

```rust
// Kopie aus UDM §0.2 mit Erweiterungen
pub struct ExecutionContext {
    pub state: WorldState,
    pub gas_remaining: u64,
    pub mana_remaining: u64,
    pub trust_context: TrustContext,
    pub emitted_events: Vec<Event>,
    pub information_losses: Vec<InformationLoss>,  // NEU
    pub network_conditions: NetworkConditions,      // NEU
}
```

**Tasks:**

- [ ] `execution/mod.rs` erstellen
- [ ] `execution/context.rs` implementieren (UDM §0.2)
- [ ] `execution/error.rs` mit ExecutionError (UDM §0.2)
- [ ] Tests für ExecutionContext (≥80% Coverage)

#### 1.2 Unified Error-Hierarchie

**Datei:** `backend/src/domain/unified/error.rs` (NEU)

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum ExecutionError {
    // VM-Errors (ℳ_VM)
    #[error("Gas exhausted")]
    GasExhausted,

    #[error("Stack overflow")]
    StackOverflow,

    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    // Storage-Errors (ℳ_S)
    #[error("Schema violation: {0}")]
    SchemaViolation(String),

    #[error("Access denied")]
    AccessDenied,

    // P2P-Errors (ℳ_P)
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Trust gate blocked: required {required}, actual {actual}")]
    TrustGateBlocked { required: f32, actual: f32 },
}
```

**Tasks:**

- [ ] `unified/error.rs` erstellen
- [ ] Alle bestehenden Error-Typen mappen
- [ ] `From<T>` Implementierungen für Konversion

---

### Phase 2: Unified Domain (Woche 2)

#### 2.1 Identity-Migration (domain/did.rs → unified/identity.rs)

**Tasks:**

- [ ] `unified/identity.rs` erstellen
- [ ] DID auf UniversalId umstellen:
  ```rust
  pub struct DID {
      pub id: UniversalId,  // statt String
      pub namespace: DIDNamespace,
      pub public_key: PublicKey,
  }
  ```
- [ ] Delegation mit Trust-Factor (Κ8):
  ```rust
  pub struct Delegation {
      pub id: UniversalId,
      pub delegator: UniversalId,
      pub delegate: UniversalId,
      pub trust_factor: f32,  // NEU: (0, 1]
      pub valid_until: Option<TemporalCoord>,
  }
  ```
- [ ] Tests migrieren + erweitern
- [ ] `domain/did.rs` als Re-Export belassen (Deprecation-Warning)

#### 2.2 Event-Migration (domain/event.rs → unified/event.rs)

**Tasks:**

- [ ] `unified/event.rs` erstellen
- [ ] EventId auf UniversalId umstellen:
  ```rust
  pub type EventId = UniversalId;  // TAG_EVENT
  ```
- [ ] Event mit TemporalCoord:
  ```rust
  pub struct Event {
      pub id: EventId,
      pub coord: TemporalCoord,
      pub parents: SmallVec<[EventId; 2]>,
      pub payload: EventPayload,
      pub finality: FinalityState,  // NEU: erweiterter State
  }
  ```
- [ ] FinalityState (erweitertes FinalityLevel):
  ```rust
  pub struct FinalityState {
      pub level: FinalityLevel,
      pub probability: f64,
      pub witness_count: u32,
      pub anchor_hash: Option<[u8; 32]>,
  }
  ```
- [ ] Tests migrieren + kausale Invarianten (Κ9)

#### 2.3 Trust-Konsolidierung

**Problem:** `domain/trust.rs` (547 Zeilen) vs `unified/trust.rs` (existiert)

**Strategie:**

1. Alles relevante aus `domain/trust.rs` nach `unified/trust.rs` migrieren
2. `domain/trust.rs` auf Re-Exports reduzieren
3. Deprecation-Warnings hinzufügen

**Tasks:**

- [ ] TrustDampeningMatrix nach unified migrieren
- [ ] TrustCombination nach unified migrieren
- [ ] Alle Tests nach unified verschieben
- [ ] `domain/trust.rs` → Re-Export-Stub

#### 2.4 Realm-Migration

**Tasks:**

- [ ] `unified/realm.rs` erstellen
- [ ] RealmId auf UniversalId umstellen
- [ ] RuleSet mit Invariant-Checker (Κ1)
- [ ] Partition mit Cost-Accounting

#### 2.5 Saga-Migration

**Tasks:**

- [ ] `unified/saga.rs` erstellen
- [ ] Intent, Goal, Constraint mit UniversalId
- [ ] Budget-Integration (UDM §V):
  ```rust
  pub struct Saga {
      pub id: UniversalId,
      pub budget: Budget,
      pub steps: Vec<SagaStep>,
      pub compensations: Vec<SagaCompensation>,
  }
  ```

#### 2.6 Formula-Migration

**Tasks:**

- [ ] `unified/formula.rs` erstellen
- [ ] Surprisal als proper struct:
  ```rust
  pub struct Surprisal {
      pub value: f32,
      pub components: SurprisalComponents,
      pub computed_at: TemporalCoord,
  }
  ```
- [ ] WorldFormulaContribution mit Cost-Bindung

---

### Phase 3: Core-Layer (Woche 3)

#### 3.1 EventEngine auf ExecutionContext

**Datei:** `backend/src/core/event_engine.rs`

**Änderungen:**

```rust
// VORHER
impl EventEngine {
    pub async fn process(&mut self, event: Event) -> Result<(), EventError> {
        // ...
    }
}

// NACHHER
impl EventEngine {
    pub async fn process(
        &mut self,
        ctx: &mut ExecutionContext,
        event: Event
    ) -> Result<(), ExecutionError> {
        ctx.consume_gas(GAS_EVENT_PROCESS)?;
        // ...
        ctx.emit(event.clone());
        Ok(())
    }
}
```

**Tasks:**

- [ ] EventEngine Signaturen ändern
- [ ] Gas-Accounting implementieren
- [ ] Event-Emission über Context
- [ ] Tests anpassen (mit Mock-Context)

#### 3.2 TrustEngine auf TrustRecord

**Datei:** `backend/src/core/trust_engine.rs`

**Änderungen:**

- Verwendung von `unified::TrustRecord` statt ad-hoc Structs
- Trust-History über TrustHistoryEntry
- Asymmetrie-Faktor aus TrustDimension (Κ4)

**Tasks:**

- [ ] Import auf unified umstellen
- [ ] update()-Methode auf TrustRecord anpassen
- [ ] Daily-Stats-Aggregation implementieren
- [ ] Invariant-Checks einbauen

#### 3.3 WorldFormulaEngine auf Cost-Algebra

**Datei:** `backend/src/core/world_formula.rs`

**Änderungen:**

- Cost als primärer Return-Type
- Surprisal struct statt f32
- Budget-Checks bei Computation

**Tasks:**

- [ ] compute() → Result<(Surprisal, Cost), ExecutionError>
- [ ] Budget-Integration
- [ ] Cost-Algebra für Aggregation

#### 3.4 ConsensusEngine auf FinalityState

**Datei:** `backend/src/core/consensus.rs`

**Änderungen:**

- FinalityState statt FinalityLevel
- Probability-Tracking
- Witness-Counting

**Tasks:**

- [ ] FinalityState-Übergänge implementieren
- [ ] Κ10-Invarianten prüfen (keine Regression)

---

### Phase 4: P2P-Layer (Woche 4)

#### 4.1 τ-Variabilität implementieren

**Neue Datei:** `backend/src/peer/p2p/timing.rs`

```rust
// Aus UDM §IX.2
pub struct NetworkConditions {
    pub latency_ms: u32,
    pub packet_loss: f32,
    pub bandwidth_kbps: u32,
    pub peer_count: u32,
}

impl NetworkConditions {
    pub fn variability_factor(&self) -> f32 {
        let c = self;
        let latency_norm = (c.latency_ms as f32 / 100.0).min(1.0);
        let loss_factor = 1.0 + c.packet_loss * 2.0;
        let peer_factor = (c.peer_count as f32 / 10.0).max(0.5);

        (0.5 + latency_norm * loss_factor / peer_factor).clamp(0.5, 3.0)
    }
}

pub struct SyncTiming {
    pub base_interval: Duration,
    pub current_interval: Duration,
    pub backoff_count: u32,
}

impl SyncTiming {
    pub fn adjust(&mut self, conditions: &NetworkConditions) {
        let v = conditions.variability_factor();
        self.current_interval = Duration::from_secs_f32(
            self.base_interval.as_secs_f32() * v
        );
    }

    pub fn exponential_backoff(&mut self) {
        self.backoff_count += 1;
        let factor = 2.0_f32.powi(self.backoff_count.min(5) as i32);
        self.current_interval = Duration::from_secs_f32(
            self.current_interval.as_secs_f32() * factor
        );
    }
}
```

**Tasks:**

- [ ] `timing.rs` erstellen
- [ ] In SwarmManager integrieren
- [ ] Periodic Condition-Updates
- [ ] Tests für Edge-Cases (V=0.5, V=3.0)

#### 4.2 Erweiterte libp2p-Protokolle

**Datei:** `backend/src/peer/p2p/behaviour.rs`

**Änderungen:**

```rust
// NEU: Zusätzliche Protokolle
use libp2p::{
    autonat,      // NAT-Traversal
    identify,     // Peer-Identifikation
    ping,         // Liveness-Check
    // ... bestehende
};

pub struct ErynoaBehaviour {
    // Bestehende
    pub gossipsub: gossipsub::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub request_response: request_response::Behaviour<...>,

    // NEU
    pub autonat: autonat::Behaviour,
    pub identify: identify::Behaviour,
    pub ping: ping::Behaviour,
}
```

**Tasks:**

- [ ] AutoNAT-Behaviour hinzufügen
- [ ] Identify-Behaviour mit Agent-Version
- [ ] Ping-Behaviour für Liveness
- [ ] Protokoll-Initialisierungsreihenfolge beachten

#### 4.3 P2P-Messages vereinheitlichen

**Neue Datei:** `backend/src/domain/unified/message.rs`

```rust
pub struct P2PMessage {
    pub id: UniversalId,
    pub protocol: P2PProtocol,
    pub sender: UniversalId,
    pub payload: MessagePayload,
    pub timestamp: TemporalCoord,
}

pub enum P2PProtocol {
    Gossipsub,
    Kademlia,
    RequestResponse,
    AutoNat,
    Identify,
    Ping,
}
```

**Tasks:**

- [ ] `message.rs` erstellen
- [ ] Bestehende Message-Typen migrieren
- [ ] Serialization-Tests

---

### Phase 5: ECLVM-Bridge (Woche 5)

#### 5.1 Adjunktions-Traits implementieren

**Neue Datei:** `backend/src/eclvm/bridge.rs`

```rust
use crate::domain::unified::*;

/// Linker Adjunkt F: Core → ECLVM
pub trait CoreToEclvm {
    fn embed(&self) -> EclvmValue;
}

/// Rechter Adjunkt G: ECLVM → Core
pub trait EclvmToCore: Sized {
    fn interpret(value: &EclvmValue) -> Result<Self, InterpretError>;
}

// Implementierungen für alle Kern-Typen
impl CoreToEclvm for UniversalId {
    fn embed(&self) -> EclvmValue {
        EclvmValue::Bytes(self.as_bytes().to_vec())
    }
}

impl EclvmToCore for UniversalId {
    fn interpret(value: &EclvmValue) -> Result<Self, InterpretError> {
        match value {
            EclvmValue::Bytes(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(b);
                Ok(UniversalId::from_bytes(arr))
            }
            _ => Err(InterpretError::TypeMismatch),
        }
    }
}

// Zig-Zag Identity Test
#[cfg(test)]
mod tests {
    use super::*;

    fn zigzag_identity<T: CoreToEclvm + EclvmToCore + PartialEq + Clone>(x: T) {
        let embedded = x.embed();
        let interpreted = T::interpret(&embedded).unwrap();
        assert_eq!(x, interpreted);
    }
}
```

**Tasks:**

- [ ] `bridge.rs` erstellen
- [ ] CoreToEclvm für: UniversalId, TrustVector6D, Cost, Event
- [ ] EclvmToCore für: UniversalId, TrustVector6D, Cost, Event
- [ ] Zig-Zag Identity Tests (≥95% Coverage)

#### 5.2 InformationLoss-Tracking

**Neue Datei:** `backend/src/execution/information_loss.rs`

```rust
pub struct InformationLoss {
    pub channel: ChannelType,
    pub loss_bits: f64,
    pub reason: LossReason,
    pub recoverable: bool,
}

pub enum ChannelType {
    EclvmExec,
    P2PGossip,
    StoragePersist,
    ApiResponse,
    ConsensusVote,
}

pub struct CompressionRecord {
    pub original_hash: [u8; 32],
    pub compressed_hash: [u8; 32],
    pub merkle_root_preserved: bool,
}
```

**Tasks:**

- [ ] `information_loss.rs` erstellen
- [ ] In ExecutionContext integrieren
- [ ] Loss-Tracking in kritischen Pfaden:
  - P2P-Serialization
  - Storage-Compression
  - API-Response-Truncation

---

### Phase 6: Finalisierung (Woche 6)

#### 6.1 Alte Module entfernen

**Tasks:**

- [ ] `domain/did.rs` → Deprecation → Remove
- [ ] `domain/event.rs` → Deprecation → Remove
- [ ] `domain/trust.rs` → Deprecation → Remove (nur doppelte Teile)
- [ ] `domain/realm.rs` → Deprecation → Remove
- [ ] `domain/saga.rs` → Deprecation → Remove
- [ ] `domain/formula.rs` → Deprecation → Remove

**Strategie:**

1. Woche 5: `#[deprecated]` Attribut hinzufügen
2. Woche 6: CI-Check dass keine Deprecated-Warnings mehr existieren
3. Woche 6+: Module entfernen

#### 6.2 API-Kompatibilität

**Datei:** `backend/src/domain/mod.rs`

```rust
// Neuer mod.rs - nur Re-Exports aus unified/
pub mod unified;

// Re-Exports für Abwärtskompatibilität
pub use unified::{
    // Primitives
    UniversalId, TemporalCoord,
    // Identity (vorher did.rs)
    DID, DIDDocument, DIDNamespace, Delegation,
    // Event (vorher event.rs)
    Event, EventId, EventPayload, FinalityLevel, FinalityState,
    // Trust
    TrustVector6D, TrustRecord, TrustDimension,
    // ...
};
```

#### 6.3 Integration-Tests

**Neue Tests:**

```
backend/tests/
├── unified_integration.rs      # Unified-Domain Tests
├── execution_context.rs        # ExecutionContext-Workflow
├── adjunction_roundtrip.rs     # ECLVM ↔ Core Roundtrip
├── p2p_tau_variability.rs      # τ-Variabilität unter Last
└── information_loss.rs         # Loss-Tracking Accuracy
```

**Tasks:**

- [ ] Integration-Test Suite erstellen
- [ ] Property-Based Tests (proptest)
- [ ] Fuzzing für kritische Pfade
- [ ] Performance-Regression-Tests

#### 6.4 Dokumentation

**Tasks:**

- [ ] UNIFIED-DATA-MODEL.md: "Codegen"-Schritt auf ✓ setzen
- [ ] API-Docs generieren (cargo doc)
- [ ] Migration-Guide für externe Konsumenten
- [ ] Axiom-Referenzen in allen Doc-Comments

---

## IV. Migrations-Matrix

### 4.1 Typen-Migration

| Alt                     | Neu                                | Phase | Priorität |
| ----------------------- | ---------------------------------- | ----- | --------- |
| `domain::DID`           | `unified::DID`                     | 2     | Hoch      |
| `domain::EventId`       | `unified::EventId` (= UniversalId) | 2     | Hoch      |
| `domain::Event`         | `unified::Event`                   | 2     | Hoch      |
| `domain::TrustVector6D` | `unified::TrustVector6D`           | 2     | Mittel    |
| `domain::Realm`         | `unified::Realm`                   | 2     | Mittel    |
| `domain::Saga`          | `unified::Saga`                    | 2     | Mittel    |
| `domain::Surprisal`     | `unified::Surprisal`               | 2     | Niedrig   |
| (neu)                   | `execution::ExecutionContext`      | 1     | Kritisch  |
| (neu)                   | `execution::ExecutionError`        | 1     | Kritisch  |
| (neu)                   | `eclvm::CoreToEclvm`               | 5     | Hoch      |

### 4.2 Modul-Abhängigkeiten

```
Phase 1 (Foundation)
  └── Phase 2 (Unified Domain)
        └── Phase 3 (Core Layer)
              └── Phase 4 (P2P Layer)
                    └── Phase 5 (ECLVM Bridge)
                          └── Phase 6 (Finalisierung)
```

---

## V. Risiken & Mitigationen

| Risiko                  | Wahrscheinlichkeit | Impact | Mitigation                     |
| ----------------------- | ------------------ | ------ | ------------------------------ |
| Breaking Changes in API | Hoch               | Mittel | Semver, Deprecation-Cycle      |
| Performance-Regression  | Mittel             | Hoch   | Benchmarks vor/nach Migration  |
| Unentdeckte Bugs        | Mittel             | Hoch   | Erhöhte Test-Coverage (>85%)   |
| Scope Creep             | Mittel             | Mittel | Strikte Phasen-Grenzen         |
| libp2p-Inkompatibilität | Niedrig            | Hoch   | Version-Pinning, Feature-Flags |

---

## VI. Erfolgs-Metriken

| Metrik               | Ziel             | Messung                |
| -------------------- | ---------------- | ---------------------- |
| Test-Coverage        | ≥85%             | cargo tarpaulin        |
| Compile-Time         | ≤+10%            | CI-Benchmark           |
| Runtime-Performance  | ≤+5%             | Criterion.rs           |
| API-Breaking-Changes | 0 (public)       | cargo public-api       |
| Deprecation-Warnings | 0 (Ende Phase 6) | cargo build --warnings |
| Axiom-Coverage       | 100%             | Manuelles Review       |

---

## VII. Checkliste

### Phase 1: Foundation ☐

- [ ] `execution/mod.rs` erstellt
- [ ] `execution/context.rs` implementiert
- [ ] `execution/error.rs` implementiert
- [ ] Tests für ExecutionContext (≥80%)

### Phase 2: Unified Domain ☐

- [ ] `unified/identity.rs` (DID-Migration)
- [ ] `unified/event.rs` (Event-Migration)
- [ ] `unified/trust.rs` (Konsolidierung)
- [ ] `unified/realm.rs` (Realm-Migration)
- [ ] `unified/saga.rs` (Saga-Migration)
- [ ] `unified/formula.rs` (Formula-Migration)
- [ ] Deprecation-Warnings in alten Modulen

### Phase 3: Core Layer ☐

- [ ] EventEngine auf ExecutionContext
- [ ] TrustEngine auf TrustRecord
- [ ] WorldFormulaEngine auf Cost-Algebra
- [ ] ConsensusEngine auf FinalityState

### Phase 4: P2P Layer ☐

- [ ] `timing.rs` (τ-Variabilität)
- [ ] AutoNAT/Identify/Ping Behaviours
- [ ] `unified/message.rs`
- [ ] Trust-Gate auf TrustVector6D

### Phase 5: ECLVM Bridge ☐

- [ ] `bridge.rs` (Adjunktions-Traits)
- [ ] CoreToEclvm Implementierungen
- [ ] EclvmToCore Implementierungen
- [ ] `information_loss.rs`
- [ ] Zig-Zag Identity Tests

### Phase 6: Finalisierung ☐

- [ ] Alte Module entfernt
- [ ] API-Kompatibilität verifiziert
- [ ] Integration-Tests
- [ ] Dokumentation aktualisiert
- [ ] Performance-Benchmarks bestanden

---

## VIII. Referenzen

- [UNIFIED-DATA-MODEL.md v1.1.0](./UNIFIED-DATA-MODEL.md)
- [IPS-01-imp.md v1.2.0](./IPS-01-imp.md)
- [Erynoa Axiome V4.1](../concept-v4/FACHKONZEPT.md)

---

_Dieser Plan ist bindend für die Restrukturierung. Abweichungen erfordern Dokumentation._
_Erstellt: Februar 2026 | Basis: UDM v1.1.0 + IPS v1.2.0_
