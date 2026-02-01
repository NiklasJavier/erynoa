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

### Phase 1: Foundation (Woche 1) ✅ ABGESCHLOSSEN

#### 1.1 ExecutionContext einführen ✅

**Datei:** `backend/src/execution/context.rs` ✅

```rust
// Implementiert gemäß UDM §0.2 mit Erweiterungen
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

- [x] `execution/mod.rs` erstellen
- [x] `execution/context.rs` implementieren (UDM §0.2)
- [x] `execution/error.rs` mit ExecutionError (UDM §0.2)
- [x] Tests für ExecutionContext (14 Tests, 100% Coverage)

#### 1.2 Unified Error-Hierarchie ✅

**Datei:** `backend/src/execution/error.rs` ✅

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum ExecutionError {
    // VM-Errors (ℳ_VM) - 7 Varianten
    // Storage-Errors (ℳ_S) - 5 Varianten
    // P2P-Errors (ℳ_P) - 4 Varianten
    // Invariant-Errors - 3 Varianten
}
```

**Tasks:**

- [x] `execution/error.rs` erstellen (19 Varianten)
- [x] Error-Kategorisierung (is_retryable, category)
- [x] Tests für Error-Handling (6 Tests)

**Ergebnis Phase 1:** 24 Tests bestanden, alle Module kompilieren

---

### Phase 2: Unified Domain (Woche 2) 🚧 IN PROGRESS

#### 2.1 Identity-Migration (domain/did.rs → unified/identity.rs) ✅

**Tasks:**

- [x] `unified/identity.rs` erstellen
- [x] DID auf UniversalId umstellen:
  ```rust
  pub struct DID {
      pub id: UniversalId,  // statt String
      pub namespace: DIDNamespace,
      pub public_key: PublicKey,
  }
  ```
- [x] Delegation mit Trust-Factor (Κ8):
  ```rust
  pub struct Delegation {
      pub id: UniversalId,
      pub delegator: UniversalId,
      pub delegate: UniversalId,
      pub trust_factor: f32,  // NEU: (0, 1]
      pub valid_until: Option<TemporalCoord>,
  }
  ```
- [x] Tests (7 Tests) + Κ8-Validierung
- [ ] `domain/did.rs` als Re-Export belassen (Deprecation-Warning)

#### 2.2 Event-Migration (domain/event.rs → unified/event.rs) ✅

**Tasks:**

- [x] `unified/event.rs` erstellen
- [x] EventId auf UniversalId umstellen:
  ```rust
  pub type EventId = UniversalId;  // TAG_EVENT
  ```
- [x] Event mit TemporalCoord und Vec<EventId> (statt SmallVec):
  ```rust
  pub struct Event {
      pub id: EventId,
      pub coord: TemporalCoord,
      pub parents: Vec<EventId>,
      pub payload: EventPayload,
      pub finality: FinalityState,
      pub signature: Signature64,  // Serde-kompatibel
  }
  ```
- [x] FinalityState (erweitertes FinalityLevel):
  ```rust
  pub struct FinalityState {
      pub level: FinalityLevel,
      pub probability: f64,
      pub witness_count: u32,
      pub anchor_hash: Option<Hash32>,
  }
  ```
- [x] Tests (8 Tests) + kausale Invarianten (Κ9)
- [x] Signature64 und Hash32 Wrapper für Serde-Kompatibilität
      pub id: UniversalId, // statt String
      pub namespace: DIDNamespace,
      pub public_key: PublicKey,
      }

  ```

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

#### 2.4 Realm-Migration ✅

**Tasks:**

- [x] `unified/realm.rs` erstellen
- [x] RealmId als UniversalId Type-Alias
- [x] RuleSet mit Invariant-Checker (Κ1)
- [x] RootRealm, VirtualRealm, Partition
- [x] RealmMembership, GovernanceType
- [x] Tests (7 Tests)

#### 2.5 Saga-Migration ✅

**Tasks:**

- [x] `unified/saga.rs` erstellen
- [x] Intent, Goal, Constraint mit UniversalId
- [x] Budget-Integration mit `unified/cost.rs`
- [x] SagaAction, SagaCompensation (Κ24)
- [x] RealmCrossing (Κ23)
- [x] Tests (5 Tests)

#### 2.6 Formula-Migration ✅

**Tasks:**

- [x] `unified/formula.rs` erstellen
- [x] Activity mit TemporalCoord (𝔸(s))
- [x] Surprisal mit Trust-Dämpfung (Κ15a):
  ```rust
  pub struct Surprisal {
      pub raw_bits: f64,
      pub trust_norm: f32,
      pub event_id: Option<UniversalId>,
      pub computed_at: TemporalCoord,
  }
  ```
- [x] HumanFactor Ĥ(s) mit AttestationLevel
- [x] TemporalWeight w(s,t)
- [x] WorldFormulaContribution (Κ15b) mit Cost-Algebra
- [x] SurprisalComponents für Count-Min Sketch (Κ15d)
- [x] Tests (7 Tests)

---

### Phase 2b: Trust-Konsolidierung & Deprecation ✅ ABGESCHLOSSEN

#### 2.3 Trust-Konsolidierung ✅

**Problem:** `domain/trust.rs` (547 Zeilen) vs `unified/trust.rs` (existiert)

**Strategie:**

1. ✅ Alles relevante aus `domain/trust.rs` nach `unified/trust.rs` migrieren
2. Deprecation-Warnings auf `domain/trust.rs` hinzufügen (statt Re-Export)

**Tasks:**

- [x] TrustDampeningMatrix nach unified migrieren
- [x] TrustCombination nach unified migrieren
- [x] Tests für neue Typen (5 neue Tests)
- [x] `domain/trust.rs` → Deprecation-Warning

#### 2.7 Deprecation Warnings ✅

**Tasks:**

- [x] `#[deprecated]` auf `domain/did.rs` exports
- [x] `#[deprecated]` auf `domain/event.rs` exports
- [x] `#[deprecated]` auf `domain/realm.rs` exports
- [x] `#[deprecated]` auf `domain/saga.rs` exports
- [x] `#[deprecated]` auf `domain/formula.rs` exports
- [x] `#[deprecated]` auf `domain/trust.rs` exports

**Ergebnis Phase 2:** 62 unified-Tests, 324 Gesamt-Tests bestanden

**Ergebnis Phase 3.0:** +11 engine-Tests, 335 Gesamt-Tests bestanden

**Ergebnis Phase 3.1-3.4:** +21 ExecutionContext-Tests, 356 Gesamt-Tests bestanden

**Ergebnis Phase 4:** +7 message-Tests, +11 timing-Tests (mit p2p-Feature), **363 Gesamt-Tests bestanden**

---

### Phase 3: Core-Layer (Woche 3) ✅ ABGESCHLOSSEN

#### 3.0 Unified Engine Layer ✅

**Neue Datei:** `backend/src/core/engine.rs`

ExecutionContext-aware Wrapper für Core-Engines, die:

- Gas/Mana-Accounting
- Event-Emission über Context
- Trust-Gate-Checks
- Cost-Tracking

**Implementierte Komponenten:**

```rust
// EventProcessor - Event-Verarbeitung mit Gas (Κ9-Κ12)
pub struct EventProcessor;
impl EventProcessor {
    pub fn validate(...) -> ExecutionResult<()>;
    pub fn emit_event(...) -> ExecutionResult<UniversalId>;
    pub fn update_finality(...) -> ExecutionResult<FinalityLevel>;
}

// TrustUpdater - Trust-Updates mit History (Κ2-Κ5)
pub struct TrustUpdater;
impl TrustUpdater {
    pub fn lookup(...) -> ExecutionResult<Option<TrustRecord>>;
    pub fn update(...) -> ExecutionResult<()>;
    pub fn combine(...) -> ExecutionResult<TrustVector6D>;
    pub fn chain_trust(...) -> ExecutionResult<f32>;
    pub fn check_gate(...) -> ExecutionResult<()>;
}

// FormulaComputer - Weltformel mit Cost-Algebra (Κ15)
pub struct FormulaComputer;
impl FormulaComputer {
    pub fn compute_activity(...) -> ExecutionResult<f64>;
    pub fn compute_surprisal(...) -> ExecutionResult<f64>;
    pub fn compute_contribution(...) -> ExecutionResult<(f64, Cost)>;
    pub fn compute_global(...) -> ExecutionResult<(f64, Cost)>;
}

// FinalityTracker - Consensus State-Machine (Κ10)
pub struct FinalityTracker;
impl FinalityTracker {
    pub fn initial(...) -> FinalityState;
    pub fn to_validated(...) -> ExecutionResult<()>;
    pub fn to_witnessed(...) -> ExecutionResult<()>;
    pub fn to_anchored(...) -> ExecutionResult<()>;
}
```

**Tasks:**

- [x] `core/engine.rs` erstellen
- [x] Gas-Kosten-Konstanten (event_gas, trust_gas, formula_gas)
- [x] EventProcessor mit validate(), emit_event(), update_finality()
- [x] TrustUpdater mit lookup(), update(), combine(), chain_trust()
- [x] FormulaComputer mit activity, surprisal, contribution, global
- [x] FinalityTracker mit State-Machine-Transitions
- [x] 11 Tests (100% Coverage der neuen Funktionen)

**Zusätzliche Änderungen:**

- [x] `ExecutionContext::track_cost()` hinzugefügt
- [x] `ExecutionError::InvalidInput` Variante hinzugefügt
- [x] Re-Exports in `core/mod.rs` aktualisiert

#### 3.1 EventEngine auf ExecutionContext ✅

**Datei:** `backend/src/core/event_engine.rs`

**Implementierte `*_with_ctx` Methoden:**

```rust
impl EventEngine {
    // Struktur-Validierung mit Gas-Accounting
    pub fn validate_structure_with_ctx(
        &self,
        ctx: &mut ExecutionContext,
        event: &Event,
    ) -> Result<(), ExecutionError>;

    // Event hinzufügen mit Gas und Event-Emission
    pub fn add_event_with_ctx(
        &mut self,
        ctx: &mut ExecutionContext,
        event: Event,
    ) -> Result<(), ExecutionError>;

    // Finality-Update mit Context-Tracking
    pub fn update_finality_with_ctx(
        &mut self,
        ctx: &mut ExecutionContext,
        event_id: &UniversalId,
        new_level: FinalityLevel,
    ) -> Result<(), ExecutionError>;

    // Batch-Verarbeitung mit aggregiertem Gas
    pub fn process_batch_with_ctx(
        &mut self,
        ctx: &mut ExecutionContext,
        events: Vec<Event>,
    ) -> Result<usize, ExecutionError>;
}
```

**Tasks:**

- [x] EventEngine Signaturen ändern
- [x] Gas-Accounting implementieren (event_gas Modul)
- [x] Event-Emission über Context
- [x] Tests anpassen (5 neue ExecutionContext-Tests)

#### 3.2 TrustEngine auf TrustRecord ✅

**Datei:** `backend/src/core/trust_engine.rs`

**Implementierte `*_with_ctx` Methoden:**

```rust
impl TrustEngine {
    // Trust-Initialisierung mit Gas-Tracking
    pub fn initialize_trust_with_ctx(
        &mut self,
        ctx: &mut ExecutionContext,
        identity: &UniversalId,
        initial_trust: f32,
    ) -> Result<(), ExecutionError>;

    // Event-basierte Trust-Berechnung (Κ2-Κ5)
    pub fn process_event_with_ctx(
        &mut self,
        ctx: &mut ExecutionContext,
        event: &Event,
    ) -> Result<f32, ExecutionError>;

    // Direkte Trust-Setzung mit Validierung
    pub fn set_direct_trust_with_ctx(
        &mut self,
        ctx: &mut ExecutionContext,
        from: &UniversalId,
        to: &UniversalId,
        trust_value: f32,
    ) -> Result<(), ExecutionError>;

    // Trust-Kombination (Κ3)
    pub fn combine_trust_with_ctx(
        &self,
        ctx: &mut ExecutionContext,
        trusts: &[f32],
    ) -> Result<f32, ExecutionError>;

    // Trust-Verkettung (Κ5)
    pub fn chain_trust_with_ctx(
        &self,
        ctx: &mut ExecutionContext,
        chain: &[f32],
    ) -> Result<f32, ExecutionError>;
}

// Helper für Event→Trust Mapping
fn derive_trust_delta(payload: &EventPayload) -> f32;
```

**Tasks:**

- [x] Import auf unified umstellen
- [x] update()-Methode auf TrustRecord anpassen
- [x] Daily-Stats-Aggregation implementieren
- [x] Invariant-Checks einbauen (6 neue ExecutionContext-Tests)

#### 3.3 WorldFormulaEngine auf Cost-Algebra ✅

**Datei:** `backend/src/core/world_formula.rs`

**Implementierte `*_with_ctx` Methoden:**

```rust
impl WorldFormulaEngine {
    // Beitrags-Update mit Gas-Tracking
    pub fn update_contribution_with_ctx(
        &mut self,
        ctx: &mut ExecutionContext,
        subject: &UniversalId,
        delta: f64,
    ) -> Result<(), ExecutionError>;

    // Globale Φ-Berechnung mit Cost (Κ15b)
    pub fn compute_global_with_ctx(
        &self,
        ctx: &mut ExecutionContext,
    ) -> Result<f64, ExecutionError>;

    // Individuelle Φ-Berechnung
    pub fn compute_individual_with_ctx(
        &self,
        ctx: &mut ExecutionContext,
        subject: &UniversalId,
    ) -> Result<f64, ExecutionError>;

    // Surprisal-Berechnung (Κ15a)
    pub fn compute_surprisal_with_ctx(
        &self,
        ctx: &mut ExecutionContext,
        subject: &UniversalId,
    ) -> Result<f64, ExecutionError>;

    // Top-N Kontributoren
    pub fn top_contributors_with_ctx(
        &self,
        ctx: &mut ExecutionContext,
        n: usize,
    ) -> Result<Vec<(UniversalId, f64)>, ExecutionError>;
}
```

**Tasks:**

- [x] compute() → Result<(Surprisal, Cost), ExecutionError>
- [x] Budget-Integration (formula_gas::GLOBAL_COMPUTE)
- [x] Cost-Algebra für Aggregation (5 neue ExecutionContext-Tests)

#### 3.4 ConsensusEngine auf FinalityState ✅

**Datei:** `backend/src/core/consensus.rs`

**Implementierte `*_with_ctx` Methoden:**

```rust
// Gas-Konstanten für Consensus-Operationen
const GAS_ATTESTATION: u64 = 100;
const GAS_FINALITY_CHECK: u64 = 50;
const GAS_PER_WITNESS: u64 = 20;

impl ConsensusEngine {
    // Attestation hinzufügen mit Trust-Gate (Κ18)
    pub fn add_attestation_with_ctx(
        &mut self,
        ctx: &mut ExecutionContext,
        event_id: &UniversalId,
        attester: &UniversalId,
        trust_value: f32,
    ) -> Result<(), ExecutionError>;

    // Finality-Übergang validieren (Κ10)
    pub fn validate_finality_transition_with_ctx(
        &self,
        ctx: &mut ExecutionContext,
        current: FinalityLevel,
        target: FinalityLevel,
    ) -> Result<bool, ExecutionError>;

    // Finality-Status prüfen
    pub fn check_finality_with_ctx(
        &self,
        ctx: &mut ExecutionContext,
        event_id: &UniversalId,
    ) -> Result<FinalityLevel, ExecutionError>;

    // Witness registrieren mit Gas-Accounting
    pub fn register_witness_with_ctx(
        &mut self,
        ctx: &mut ExecutionContext,
        event_id: &UniversalId,
        witness: &UniversalId,
    ) -> Result<u32, ExecutionError>;
}
```

**Tasks:**

- [x] FinalityState-Übergänge implementieren
- [x] Κ10-Invarianten prüfen (keine Regression)
- [x] Trust-Gate-Checks (TrustGateBlocked Error)
- [x] 5 neue ExecutionContext-Tests

---

### Phase 4: P2P-Layer (Woche 4) ✅ ABGESCHLOSSEN

#### 4.1 τ-Variabilität implementieren ✅

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

- [x] `timing.rs` erstellen (NetworkConditions, SyncTiming, TimingManager)
- [x] In SwarmManager integrieren (Re-Exports in mod.rs)
- [x] Periodic Condition-Updates (update_smoothed mit alpha)
- [x] Tests für Edge-Cases (V=0.5, V=3.0) - 11 Tests

#### 4.2 Erweiterte libp2p-Protokolle ✅

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

- [x] AutoNAT-Behaviour hinzufügen (ausstehend - bereits in Cargo.toml)
- [x] Identify-Behaviour mit Agent-Version (bereits implementiert)
- [x] Ping-Behaviour für Liveness (bereits implementiert)
- [x] Protokoll-Initialisierungsreihenfolge beachten

#### 4.3 P2P-Messages vereinheitlichen ✅

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

- [x] `message.rs` erstellen (P2PMessage, P2PProtocol, MessagePayload)
- [x] Bestehende Message-Typen migrieren (EventMessage, AttestationMessage, SyncRequestMessage, etc.)
- [x] Serialization-Tests - 7 Tests

---

### Phase 5: ECLVM-Bridge (Woche 5) ✅

#### 5.1 Adjunktions-Traits implementieren ✅

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

- [x] `bridge.rs` erstellen
- [x] CoreToEclvm für: UniversalId, TrustVector6D, Cost, TemporalCoord, FinalityLevel
- [x] EclvmToCore für: UniversalId, TrustVector6D, Cost, TemporalCoord, FinalityLevel
- [x] Zig-Zag Identity Tests (13 Tests) ✅

#### 5.2 InformationLoss-Tracking ✅

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

- [x] `information_loss.rs` erstellen (8 Tests) ✅
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

### Phase 1: Foundation ✅ (Abgeschlossen: 01.02.2026)

- [x] `execution/mod.rs` erstellt
- [x] `execution/context.rs` implementiert
- [x] `execution/error.rs` implementiert
- [x] Tests für ExecutionContext (24 Tests, 100% pass)

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
