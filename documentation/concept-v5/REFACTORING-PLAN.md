# REFACTORING-PLAN: IST → SOLL

## Erynoa Backend – ECL-Engine-Integration

**Version:** 2.0
**Datum:** 3. Februar 2026
**Autor:** GitHub Copilot
**Basierend auf:** 03-SYSTEM-ARCHITECTURE.md (Abschnitte 3.0.1 – 3.0.9)

---

## Inhaltsverzeichnis

1. [Executive Summary](#1-executive-summary)
2. [IST-Zustand Analyse](#2-ist-zustand-analyse)
3. [SOLL-Zustand Definition](#3-soll-zustand-definition)
4. [Abhängigkeits-Graph](#4-abhängigkeits-graph)
5. [Refactoring-Phasen](#5-refactoring-phasen)
6. [Detaillierte Änderungen pro Modul](#6-detaillierte-änderungen-pro-modul)
7. [Migrations-Strategie](#7-migrations-strategie)
8. [Test-Strategie](#8-test-strategie)
9. [Risiken & Mitigationen](#9-risiken--mitigationen)
10. [Feingranulare Gap-Analyse](#10-feingranulare-gap-analyse)
11. [Mathematische Refactoring-Logik](#11-mathematische-refactoring-logik)
12. [P2P-Kommunikations-Architektur](#12-p2p-kommunikations-architektur)
13. [**NEU: StateManager Refactoring-Plan (Detailliert)**](#13-statemanager-refactoring-plan-detailliert)

---

## 1. Executive Summary

### 1.1 Ziel

Erweiterung des ECLVM-Layers um **6 spezialisierte Engines**:

1. **UI-Engine** – Deklaratives, Trust-basiertes Interface-Rendering
2. **DataLogic-Engine** – Reaktive Event-Verarbeitung und Aggregation
3. **API-Engine** – Dynamische REST-API-Definition per ECL
4. **Governance-Engine** – DAO-Prinzipien und Abstimmungsmechanismen
5. **Controller-Engine** – Berechtigungsverwaltung mit Delegation
6. **Blueprint-Engine** – Template-Deployment und Komposition

### 1.2 Umfang

| Kategorie               | Anzahl         |
| ----------------------- | -------------- |
| Neue Dateien            | ~28-35         |
| Modifizierte Dateien    | ~15-20         |
| Neue Zeilen (geschätzt) | ~10.000-14.000 |
| Betroffene Module       | 6              |
| Proto-Dateien (gRPC)    | 1-2            |

### 1.3 Geschätzte Dauer

| Phase                              | Dauer            |
| ---------------------------------- | ---------------- |
| Phase 1: Grundlagen                | 2-3 Wochen       |
| Phase 2: Kern-Engines              | 4-6 Wochen       |
| Phase 3: Integration               | 2-3 Wochen       |
| Phase 4: UI-Engine + Realm-UI gRPC | 4-5 Wochen       |
| Phase 5: API & Governance          | 3-4 Wochen       |
| Phase 6: Blueprint-System          | 3-4 Wochen       |
| Phase 7: Stabilisierung            | 2 Wochen         |
| **Gesamt**                         | **20-27 Wochen** |

### 1.4 Realm-UI Integration

Die UI-Engine kommuniziert **nicht** direkt mit dem Browser, sondern über **gRPC** mit dem **Realm-UI Frontend**:

```
┌──────────────┐     gRPC      ┌──────────────┐     DOM     ┌──────────────┐
│   Backend    │◄────────────►│   Realm-UI   │────────────►│   Browser    │
│  UI-Engine   │  bidirektion. │   (Frontend) │             │              │
└──────────────┘               └──────────────┘             └──────────────┘
```

**Vorteile:**

- **Trennung von Concerns**: Backend kontrolliert UI-Logik, Frontend rendert
- **Trust-basiertes Rendering**: Backend filtert nach Peer-Trust
- **Live-Updates**: Bidirektionaler gRPC-Stream für Echtzeit
- **Multi-Platform**: Realm-UI kann Web, Mobile, Desktop sein

---

## 10. Feingranulare Gap-Analyse

### 10.1 Übersichts-Matrix: IST vs. SOLL

```
┌────────────────────────────────────────────────────────────────────────────────────────────┐
│                           FEINGRANULARE GAP-ANALYSE                                       │
├────────────────────────────────────────────────────────────────────────────────────────────┤
│  Legende: ✅ = Vollständig vorhanden | ⚠️ = Teilweise | ❌ = Fehlt komplett | 🔧 = Erweiterung │
└────────────────────────────────────────────────────────────────────────────────────────────┘
```

#### 10.1.1 ECLVM Layer (`backend/src/eclvm/`)

| Datei             | IST-Zustand                                | SOLL-Zustand                              | Gap                   | Prio |
| ----------------- | ------------------------------------------ | ----------------------------------------- | --------------------- | ---- |
| `mod.rs`          | ✅ Re-exports für VM, Gas, Host            | 🔧 + engines/, types/ exports             | Export-Erweiterung    | P1   |
| `bytecode.rs`     | ✅ 65+ OpCodes (Stack, Arith, Trust, Host) | 🔧 + 32 neue OpCodes (UI, API, Gov, Ctrl) | OpCode-Erweiterung    | P1   |
| `runtime/host.rs` | ✅ HostInterface mit Store-Ops             | 🔧 + UI/API/Gov/Ctrl-Host-Functions       | Interface-Erweiterung | P2   |
| `runtime/vm.rs`   | ✅ Stack-VM mit Gas-Metering               | 🔧 + Engine-Hooks, Context-Switching      | VM-Erweiterung        | P2   |
| `engines/`        | ❌ Existiert nicht                         | NEU: 6 Engine-Submodule                   | Komplett neu          | P1   |
| `types/`          | ❌ Existiert nicht                         | NEU: Shared Types für Engines             | Komplett neu          | P1   |

#### 10.1.2 Domain Layer (`backend/src/domain/unified/`)

| Datei           | IST-Zustand                                                  | SOLL-Zustand                          | Gap                  | Prio |
| --------------- | ------------------------------------------------------------ | ------------------------------------- | -------------------- | ---- |
| `realm.rs`      | ✅ RealmId, Rule, RealmRules, VirtualRealm                   | 🔧 + Room, Partition, ControllerScope | Struktur-Erweiterung | P1   |
| `saga.rs`       | ✅ Goal (Transfer, Attest, Delegate, Query, Create, Complex) | 🔧 + 10 neue Goal-Typen               | Enum-Erweiterung     | P1   |
| `ui.rs`         | ❌ Existiert nicht                                           | NEU: UI-Domain-Types                  | Komplett neu         | P2   |
| `api.rs`        | ❌ Existiert nicht                                           | NEU: API-Domain-Types                 | Komplett neu         | P3   |
| `governance.rs` | ❌ Existiert nicht                                           | NEU: Governance-Domain-Types          | Komplett neu         | P3   |
| `controller.rs` | ❌ Existiert nicht                                           | NEU: Controller-Domain-Types          | Komplett neu         | P2   |

#### 10.1.3 Local/Storage Layer (`backend/src/local/`)

| Datei                      | IST-Zustand                                            | SOLL-Zustand                                               | Gap            | Prio |
| -------------------------- | ------------------------------------------------------ | ---------------------------------------------------------- | -------------- | ---- |
| `blueprint_marketplace.rs` | ✅ Blueprint mit stores, policies, sagas (1949 Zeilen) | 🔧 + structure, ui, datalogic, api, governance, controller | Major Refactor | P2   |
| `realm_storage.rs`         | ✅ Basis-Storage                                       | 🔧 + Room/Partition-Storage                                | Erweiterung    | P2   |
| `ui_store.rs`              | ❌ Existiert nicht                                     | NEU: UI-State-Storage                                      | Komplett neu   | P3   |
| `api_store.rs`             | ❌ Existiert nicht                                     | NEU: API-Registry-Storage                                  | Komplett neu   | P3   |
| `governance_store.rs`      | ❌ Existiert nicht                                     | NEU: Proposal/Vote-Storage                                 | Komplett neu   | P3   |

#### 10.1.4 Peer Layer (`backend/src/peer/`)

| Datei              | IST-Zustand                                      | SOLL-Zustand                     | Gap                  | Prio |
| ------------------ | ------------------------------------------------ | -------------------------------- | -------------------- | ---- |
| `saga_composer.rs` | ✅ compose_transfer, compose_attest (640 Zeilen) | 🔧 + 8 neue compose-Methoden     | Methoden-Erweiterung | P2   |
| `gateway.rs`       | ✅ GatewayGuard, CrossingResult (591 Zeilen)     | 🔧 + Engine-Integration          | Integration          | P3   |
| `ui_renderer.rs`   | ❌ Existiert nicht                               | NEU: Peer-spezifisches Rendering | Komplett neu         | P2   |
| `api_handler.rs`   | ❌ Existiert nicht                               | NEU: API-Request-Handler         | Komplett neu         | P3   |

#### 10.1.5 Proto/gRPC Layer (`backend/proto/erynoa/v1/`)

| Datei            | IST-Zustand                                | SOLL-Zustand                                               | Gap          | Prio |
| ---------------- | ------------------------------------------ | ---------------------------------------------------------- | ------------ | ---- |
| `peer.proto`     | ✅ PeerService, IntentService, SagaService | ⚠️ Ausreichend für Basis                                   | Minimal      | P4   |
| `realm_ui.proto` | ❌ Existiert nicht                         | NEU: RealmUIService (Connect, GetInitialUI, ExecuteAction) | Komplett neu | P1   |

---

### 10.2 Detaillierte Änderungen pro Datei

#### 10.2.1 `eclvm/bytecode.rs` – OpCode-Erweiterungen

**IST-Zustand (65+ OpCodes):**

```rust
pub enum OpCode {
    // Stack: PushConst, Pop, Dup, Swap, Pick
    // Arithmetik: Add, Sub, Mul, Div, Mod, Neg, Min, Max
    // Vergleiche: Eq, Neq, Gt, Gte, Lt, Lte
    // Logik: And, Or, Not
    // Control: Jump, JumpIfFalse, JumpIfTrue, Call, Return
    // Trust: TrustDim, TrustNorm, TrustCombine, TrustCreate
    // Host: LoadTrust, HasCredential, ResolveDID, GetBalance, GetTimestamp, Log
    // Assertions: Assert, Require
    // Erweitert: Surprisal, TrustAboveThreshold, TrustWeightedAvg, TrustDistance
    // String: StrLen, StrEqIgnoreCase, StrContains
    // Math: MathAbs, MathSqrt, MathFloor, MathCeil, MathRound, Clamp, Lerp
    // Array: ArrayLen, ArrayGet, Contains
    // Ende: Halt, Abort
}
```

**SOLL-Zustand (+32 neue OpCodes):**

```rust
pub enum OpCode {
    // ... alle bestehenden ...

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: UI Operations (10 OpCodes)
    // ═══════════════════════════════════════════════════════════════════════
    UIRender,              // Rendere UI für aktuellen Peer
    UIBindGet,             // Stack: [binding_id] → [Value]
    UIBindSet,             // Stack: [binding_id, value] → []
    UIComponentVisible,    // Stack: [component_id] → [Bool]
    UIApplyDelta,          // Stack: [delta] → []
    UITrustGate,           // Stack: [component_id, trust] → [Bool]
    UICredentialGate,      // Stack: [component_id, credentials] → [Bool]
    UIEmitUpdate,          // Stack: [update] → []
    UIGetLayout,           // Stack: [room_id] → [Layout]
    UIGetComponent,        // Stack: [component_id] → [Component]

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: API Operations (8 OpCodes)
    // ═══════════════════════════════════════════════════════════════════════
    APIRespond,            // Stack: [status, body] → []
    APIValidateSchema,     // Stack: [data, schema] → [Bool]
    APIRateCheck,          // Stack: [endpoint_id] → [Bool]
    APIGetHeader,          // Stack: [header_name] → [String?]
    APIGetParam,           // Stack: [param_name] → [String?]
    APISetHeader,          // Stack: [header_name, value] → []
    APIError,              // Stack: [status, message] → []
    APILog,                // Stack: [level, message] → []

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: Governance Operations (8 OpCodes)
    // ═══════════════════════════════════════════════════════════════════════
    GovPropose,            // Stack: [proposal] → [proposal_id]
    GovVote,               // Stack: [proposal_id, choice] → []
    GovExecute,            // Stack: [proposal_id] → [Bool]
    GovVetoPower,          // Stack: [actor_did] → [Bool]
    GovGetProposal,        // Stack: [proposal_id] → [Proposal?]
    GovGetVotes,           // Stack: [proposal_id] → [Votes]
    GovCheckQuorum,        // Stack: [proposal_id] → [Bool]
    GovTimelockRemaining,  // Stack: [proposal_id] → [Number]

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: Controller Operations (6 OpCodes)
    // ═══════════════════════════════════════════════════════════════════════
    CtrlValidate,          // Stack: [scope, actor, action] → [Bool]
    CtrlDelegate,          // Stack: [from, to, permissions] → []
    CtrlRevoke,            // Stack: [delegation_id] → []
    CtrlGetPermissions,    // Stack: [actor, scope] → [Permissions]
    CtrlCheckScope,        // Stack: [actor, scope, permission] → [Bool]
    CtrlAuditLog,          // Stack: [action_description] → []
}

impl OpCode {
    pub fn gas_cost(&self) -> u64 {
        match self {
            // UI Ops: Moderate Cost
            Self::UIRender => 100,
            Self::UIBindGet => 10,
            Self::UIBindSet => 25,
            Self::UIComponentVisible => 15,
            Self::UIApplyDelta => 200,
            Self::UITrustGate => 20,
            Self::UICredentialGate => 25,
            Self::UIEmitUpdate => 50,
            Self::UIGetLayout => 30,
            Self::UIGetComponent => 20,

            // API Ops: Variable
            Self::APIRespond => 50,
            Self::APIValidateSchema => 100,
            Self::APIRateCheck => 5,
            Self::APIGetHeader => 5,
            Self::APIGetParam => 5,
            Self::APISetHeader => 10,
            Self::APIError => 30,
            Self::APILog => 20,

            // Governance Ops: High Cost (State-Changing)
            Self::GovPropose => 500,
            Self::GovVote => 100,
            Self::GovExecute => 1000,
            Self::GovVetoPower => 50,
            Self::GovGetProposal => 30,
            Self::GovGetVotes => 50,
            Self::GovCheckQuorum => 30,
            Self::GovTimelockRemaining => 10,

            // Controller Ops: Moderate (Security-Critical)
            Self::CtrlValidate => 50,
            Self::CtrlDelegate => 200,
            Self::CtrlRevoke => 100,
            Self::CtrlGetPermissions => 30,
            Self::CtrlCheckScope => 40,
            Self::CtrlAuditLog => 25,

            // ... existing costs ...
        }
    }
}
```

#### 10.2.2 `domain/unified/saga.rs` – Goal-Erweiterungen

**IST-Zustand (6 Goal-Typen):**

```rust
pub enum Goal {
    Transfer { to, amount, asset_type },
    Attest { subject, claim },
    Delegate { to, capabilities, trust_factor, ttl_seconds },
    Query { predicate },
    Create { entity_type, params },
    Complex { description, sub_goals },
}
```

**SOLL-Zustand (+10 neue Goal-Typen):**

```rust
pub enum Goal {
    // ... bestehende 6 ...

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: Realm/Room-Management (Κ1)
    // ═══════════════════════════════════════════════════════════════════════
    RealmModify {
        realm_id: RealmId,
        modification: RealmModification,
    },
    RoomCreate {
        realm_id: RealmId,
        room_config: RoomConfig,
    },
    PartitionCreate {
        room_id: RoomId,
        partition_config: PartitionConfig,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: UI-Modifikation (für Realm-UI)
    // ═══════════════════════════════════════════════════════════════════════
    UIModify {
        scope: ScopeId,
        ui_delta: UIDelta,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: Governance (DAO)
    // ═══════════════════════════════════════════════════════════════════════
    Governance {
        proposal: ProposalDraft,
    },
    Vote {
        proposal_id: ProposalId,
        choice: VoteChoice,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: Cross-Realm (Κ23)
    // ═══════════════════════════════════════════════════════════════════════
    CrossRealm {
        from_realm: RealmId,
        to_realm: RealmId,
        action: Box<Goal>,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: API Management
    // ═══════════════════════════════════════════════════════════════════════
    APIRegister {
        scope: ScopeId,
        endpoint_config: APIEndpointConfig,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: Blueprint Operations
    // ═══════════════════════════════════════════════════════════════════════
    BlueprintDeploy {
        blueprint_id: BlueprintId,
        target_realm: RealmId,
        config: DeploymentConfig,
    },
    BlueprintUpgrade {
        deployment_id: DeploymentId,
        new_version: BlueprintId,
    },
}
```

#### 10.2.3 `domain/unified/realm.rs` – Room/Partition-Erweiterungen

**IST-Zustand:**

- `RealmId` als `UniversalId`
- `Rule`, `RuleCategory`, `RealmRules`
- `GovernanceType` (Quadratic, Token, Reputation, Delegated)
- `Realm` Trait, `RootRealm`, `VirtualRealm`
- `Partition` (Basic: id, realm_id, name, store_types)

**SOLL-Zustand (Erweiterungen):**

```rust
// ═══════════════════════════════════════════════════════════════════════════
// NEU: Room (Zwischen VirtualRealm und Partition)
// ═══════════════════════════════════════════════════════════════════════════

pub type RoomId = UniversalId;

pub fn room_id_from_name(realm_id: &RealmId, name: &str) -> RoomId {
    let content = format!("{}:{}", realm_id.to_hex(), name);
    UniversalId::new(UniversalId::TAG_ROOM, 1, content.as_bytes())
}

/// Room ist ein logischer Container innerhalb eines VirtualRealm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub name: String,
    pub realm_id: RealmId,
    pub description: Option<String>,

    // Untergeordnete Partitionen
    pub partitions: Vec<PartitionId>,

    // Controller für diesen Room
    pub controller: Option<UniversalId>,

    // ECL-Referenzen
    pub policy_id: Option<String>,
    pub ui_id: Option<String>,
    pub datalogic_id: Option<String>,

    // Governance für diesen Room
    pub governance_mode: Option<GovernanceMode>,

    pub created_at: TemporalCoord,
    pub modified_at: TemporalCoord,
}

/// Erweitertes Partition mit detaillierter Access-Policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedPartition {
    pub id: PartitionId,
    pub name: String,
    pub room_id: RoomId,

    // Schema-Definition
    pub store_schema: StoreSchema,

    // Detaillierte Access-Policies
    pub access_policy: AccessPolicy,

    // Optionale Verschlüsselung
    pub encryption: Option<EncryptionConfig>,

    pub created_at: TemporalCoord,
}

/// Detaillierte Access-Policy für Partitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    pub read_access: AccessRule,
    pub write_access: AccessRule,
    pub delete_access: AccessRule,
    pub schema_modify_access: AccessRule,
}

/// Access-Regel mit verschiedenen Modi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessRule {
    /// Alle können zugreifen
    All,
    /// Niemand kann zugreifen
    None,
    /// Nur der Besitzer
    OwnerOnly,
    /// Minimaler Trust erforderlich
    TrustMin { threshold: f32, dimension: Option<TrustDimension> },
    /// Bestimmtes Credential erforderlich
    Credential { schema: String },
    /// Nur der Controller
    Controller,
    /// Controller oder Delegates
    ControllerOrDelegate,
    /// Custom ECL-Policy
    Custom { ecl_policy_id: String },
    /// Kombinierte Regeln (AND)
    All(Vec<AccessRule>),
    /// Kombinierte Regeln (OR)
    Any(Vec<AccessRule>),
}

/// Controller-Scope für hierarchische Berechtigungen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControllerScope {
    /// Realm-weite Kontrolle
    Realm(RealmId),
    /// Room-Kontrolle
    Room(RoomId),
    /// Partition-Kontrolle
    Partition(PartitionId),
}
```

#### 10.2.4 `local/blueprint_marketplace.rs` – Blueprint-Erweiterungen

**IST-Zustand (Blueprint-Struktur):**

```rust
pub struct Blueprint {
    pub id: BlueprintId,
    pub name: String,
    pub version: SemVer,
    pub creator_did: String,
    pub tags: Vec<String>,
    pub category: BlueprintCategory,
    pub license: BlueprintLicense,

    // Inhalt
    pub stores: Vec<BlueprintStore>,
    pub policies: Vec<BlueprintPolicy>,
    pub sagas: Vec<BlueprintSaga>,

    // Versionierung
    pub predecessor: Option<BlueprintId>,
    pub forked_from: Option<BlueprintId>,

    // Metriken
    pub complexity: u64,
    pub novelty_score: f64,
    pub diversity_contribution: f64,
}
```

**SOLL-Zustand (ExtendedBlueprint):**

```rust
/// Erweitertes Blueprint mit allen ECL-Komponenten
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedBlueprint {
    // ─────────────────────────────────────────────────────────────────────────
    // Identifikation (unverändert)
    // ─────────────────────────────────────────────────────────────────────────
    pub id: BlueprintId,
    pub version: SemVer,
    pub content_hash: Hash,

    // ─────────────────────────────────────────────────────────────────────────
    // Metadaten (unverändert)
    // ─────────────────────────────────────────────────────────────────────────
    pub name: String,
    pub description: String,
    pub creator_did: UniversalId,
    pub created_at: TemporalCoord,
    pub tags: Vec<String>,
    pub category: BlueprintCategory,
    pub license: BlueprintLicense,

    // ─────────────────────────────────────────────────────────────────────────
    // NEU: ECL-Komponenten (Unified Configuration)
    // ─────────────────────────────────────────────────────────────────────────

    /// Struktur: Räume, Partitionen, Store-Schemas
    pub structure: BlueprintStructure,

    /// Policy: Trust-Gates, Regeln, Validierung
    pub policy: BlueprintPolicySection,

    /// UI: Layouts, Pages, Components (für Realm-UI)
    pub ui: BlueprintUI,

    /// DataLogic: Event-Handler, Aggregationen
    pub datalogic: BlueprintDataLogic,

    /// API: REST-Endpoints, Schemas, Auth
    pub api: BlueprintAPI,

    /// Governance: DAO-Modus, Voting-Regeln
    pub governance: BlueprintGovernance,

    /// Controller: Permissions, Delegation, Automation
    pub controller: BlueprintController,

    // ─────────────────────────────────────────────────────────────────────────
    // Versionierung (unverändert)
    // ─────────────────────────────────────────────────────────────────────────
    pub predecessor: Option<BlueprintId>,
    pub forked_from: Option<BlueprintId>,
    pub dependencies: Vec<BlueprintDependency>,

    // ─────────────────────────────────────────────────────────────────────────
    // Metriken (erweitert)
    // ─────────────────────────────────────────────────────────────────────────
    pub complexity: u64,
    pub novelty_score: f64,
    pub diversity_contribution: f64,
    pub omega_contribution: f64,  // NEU: Beitrag zur Weltformel
}

// ═══════════════════════════════════════════════════════════════════════════
// NEU: Blueprint-Komponenten
// ═══════════════════════════════════════════════════════════════════════════

/// Blueprint-Struktur (Räume, Partitionen)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlueprintStructure {
    pub rooms: Vec<RoomTemplate>,
    pub partitions: Vec<PartitionTemplate>,
    pub stores: Vec<BlueprintStore>,  // Migration von alt
    pub allow_dynamic_rooms: bool,
    pub dynamic_room_template: Option<String>,
}

/// Blueprint-UI (für Realm-UI)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlueprintUI {
    pub theme: HashMap<String, String>,
    pub layouts: Vec<UILayoutDef>,
    pub pages: Vec<UIPageDef>,
    pub components: Vec<UIComponentDef>,
    pub default_trust_gates: HashMap<String, f32>,
    pub default_credential_gates: HashMap<String, Vec<String>>,
}

/// Blueprint-DataLogic
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlueprintDataLogic {
    pub handlers: Vec<EventHandlerDef>,
    pub aggregations: Vec<AggregationDef>,
    pub outputs: Vec<OutputDef>,
    pub transforms: Vec<TransformDef>,
}

/// Blueprint-API
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlueprintAPI {
    pub version: String,
    pub base_path: String,
    pub endpoints: Vec<APIEndpointDef>,
    pub default_auth: EndpointAuthDef,
    pub default_rate_limit: RateLimitDef,
    pub schemas: HashMap<String, serde_json::Value>,
    pub webhooks: Vec<WebhookDef>,
}

/// Blueprint-Governance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlueprintGovernance {
    pub mode: GovernanceModeDef,
    pub voting_rules: VotingRulesDef,
    pub proposal_templates: Vec<ProposalTemplateDef>,
    pub action_overrides: HashMap<String, VotingRulesDef>,
    pub veto_config: Option<VetoConfigDef>,
    pub timelock_config: TimelockConfigDef,
}

/// Blueprint-Controller
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlueprintController {
    pub primary: Option<ControllerSpecDef>,
    pub governance_controlled: bool,
    pub permissions: PermissionsDef,
    pub automation: Vec<AutomationRuleDef>,
    pub delegation_config: DelegationConfigDef,
    pub audit_config: AuditConfigDef,
}
```

#### 10.2.5 `runtime/host.rs` – HostInterface-Erweiterungen

**IST-Zustand (HostInterface Trait):**

```rust
pub trait HostInterface: Send + Sync {
    // Trust & Identity
    fn get_trust_vector(&self, did: &str) -> Result<[f64; 6]>;
    fn has_credential(&self, did: &str, schema: &str) -> Result<bool>;
    fn get_balance(&self, did: &str) -> Result<u64>;
    fn resolve_did(&self, did: &str) -> Result<bool>;
    fn get_timestamp(&self) -> u64;
    fn log(&self, message: &str);

    // Store Operations
    fn set_store_context(&mut self, ctx: StoreContext) -> Result<()>;
    fn store_get(&self, store_name: &str, key: &str, is_personal: bool) -> Result<Option<HostStoreValue>>;
    fn store_put(&mut self, store_name: &str, key: &str, value: HostStoreValue, is_personal: bool) -> Result<()>;
    fn store_delete(&mut self, store_name: &str, key: &str, is_personal: bool) -> Result<bool>;
    fn store_get_nested(&self, ...) -> Result<Option<HostStoreValue>>;
    fn store_put_nested(&mut self, ...) -> Result<()>;
    fn store_append_list(&mut self, ...) -> Result<usize>;
    // ... weitere Store-Ops

    // Schema Evolution
    fn store_evolve_schema(&mut self, ...) -> Result<HostSchemaEvolutionResult>;
    fn store_get_schema_version(&self, ...) -> Result<Option<HostStoreSchema>>;
    fn store_get_schema_history(&self, ...) -> Result<HostSchemaHistory>;
    fn store_activate_schema(&mut self, ...) -> Result<bool>;
    fn store_reject_schema(&mut self, ...) -> Result<bool>;
}
```

**SOLL-Zustand (Erweiterungen):**

```rust
pub trait HostInterface: Send + Sync {
    // ... alle bestehenden Methoden ...

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: UI-Host-Functions (für Realm-UI gRPC)
    // ═══════════════════════════════════════════════════════════════════════

    /// Rendere UI für einen Peer
    fn ui_render(&self, room_id: &str, peer_did: &str) -> Result<HostRenderedUI> {
        Err(ApiError::NotSupported("ui_render nicht unterstützt".into()))
    }

    /// Hole gebundenen Wert
    fn ui_bind_get(&self, binding_id: &str) -> Result<HostStoreValue> {
        Err(ApiError::NotSupported("ui_bind_get nicht unterstützt".into()))
    }

    /// Setze gebundenen Wert
    fn ui_bind_set(&mut self, binding_id: &str, value: HostStoreValue) -> Result<()> {
        Err(ApiError::NotSupported("ui_bind_set nicht unterstützt".into()))
    }

    /// Prüfe Komponenten-Sichtbarkeit
    fn ui_component_visible(&self, component_id: &str, peer_did: &str) -> Result<bool> {
        Err(ApiError::NotSupported("ui_component_visible nicht unterstützt".into()))
    }

    /// Emit UI-Update an Realm-UI
    fn ui_emit_update(&self, session_id: &str, update: HostUIUpdate) -> Result<()> {
        Err(ApiError::NotSupported("ui_emit_update nicht unterstützt".into()))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: API-Host-Functions
    // ═══════════════════════════════════════════════════════════════════════

    /// Sende API-Response
    fn api_respond(&self, status: u16, body: HostStoreValue) -> Result<()> {
        Err(ApiError::NotSupported("api_respond nicht unterstützt".into()))
    }

    /// Validiere Daten gegen JSON-Schema
    fn api_validate_schema(&self, data: &HostStoreValue, schema_name: &str) -> Result<bool> {
        Err(ApiError::NotSupported("api_validate_schema nicht unterstützt".into()))
    }

    /// Prüfe Rate-Limit
    fn api_rate_check(&self, endpoint_id: &str, client_id: &str) -> Result<bool> {
        Err(ApiError::NotSupported("api_rate_check nicht unterstützt".into()))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: Governance-Host-Functions
    // ═══════════════════════════════════════════════════════════════════════

    /// Hole Proposal
    fn gov_get_proposal(&self, proposal_id: &str) -> Result<Option<HostProposal>> {
        Err(ApiError::NotSupported("gov_get_proposal nicht unterstützt".into()))
    }

    /// Erstelle Proposal
    fn gov_create_proposal(&mut self, scope: &str, proposal: HostProposalDraft) -> Result<String> {
        Err(ApiError::NotSupported("gov_create_proposal nicht unterstützt".into()))
    }

    /// Stimme ab
    fn gov_cast_vote(&mut self, proposal_id: &str, voter: &str, choice: &str) -> Result<()> {
        Err(ApiError::NotSupported("gov_cast_vote nicht unterstützt".into()))
    }

    /// Berechne Voting-Power
    fn gov_voting_power(&self, scope: &str, voter: &str) -> Result<f32> {
        Err(ApiError::NotSupported("gov_voting_power nicht unterstützt".into()))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: Controller-Host-Functions
    // ═══════════════════════════════════════════════════════════════════════

    /// Validiere Controller-Aktion
    fn ctrl_validate_action(&self, scope: &str, actor: &str, action: &str) -> Result<bool> {
        Err(ApiError::NotSupported("ctrl_validate_action nicht unterstützt".into()))
    }

    /// Delegiere Kontrolle
    fn ctrl_delegate(&mut self, from: &str, to: &str, permissions: HostPermissions, ttl: Option<u64>) -> Result<String> {
        Err(ApiError::NotSupported("ctrl_delegate nicht unterstützt".into()))
    }

    /// Widerrufe Delegation
    fn ctrl_revoke(&mut self, delegation_id: &str) -> Result<()> {
        Err(ApiError::NotSupported("ctrl_revoke nicht unterstützt".into()))
    }

    /// Hole Permissions für Actor
    fn ctrl_get_permissions(&self, actor: &str, scope: &str) -> Result<HostPermissions> {
        Err(ApiError::NotSupported("ctrl_get_permissions nicht unterstützt".into()))
    }
}
```

---

### 10.3 Neue Dateien (komplett neu zu erstellen)

#### 10.3.1 Engine-Module (`eclvm/engines/`)

| Datei                                             | Zeilen | Beschreibung                   |
| ------------------------------------------------- | ------ | ------------------------------ |
| `engines/mod.rs`                                  | ~50    | Re-exports für alle Engines    |
| `engines/ui_engine/mod.rs`                        | ~150   | UIEngine Hauptstruct           |
| `engines/ui_engine/renderer.rs`                   | ~600   | Trust-basiertes Rendering      |
| `engines/ui_engine/components.rs`                 | ~400   | Component-Registry             |
| `engines/ui_engine/bindings.rs`                   | ~500   | Reactive Bindings              |
| `engines/ui_engine/layout.rs`                     | ~300   | Layout-System                  |
| `engines/ui_engine/delta.rs`                      | ~300   | Delta-Updates                  |
| `engines/ui_engine/grpc_bridge.rs`                | ~800   | Realm-UI gRPC-Bridge           |
| `engines/ui_engine/session.rs`                    | ~400   | Session-Management             |
| `engines/ui_engine/streaming.rs`                  | ~500   | Bidirektionales Streaming      |
| `engines/datalogic_engine/mod.rs`                 | ~100   | DataLogicEngine Hauptstruct    |
| `engines/datalogic_engine/handlers.rs`            | ~500   | Event-Handler                  |
| `engines/datalogic_engine/aggregations.rs`        | ~400   | Aggregation-State              |
| `engines/datalogic_engine/transforms.rs`          | ~300   | Data Transforms                |
| `engines/datalogic_engine/outputs.rs`             | ~200   | Output-Emitter                 |
| `engines/api_engine/mod.rs`                       | ~100   | APIEngine Hauptstruct          |
| `engines/api_engine/endpoints.rs`                 | ~500   | Endpoint-Registry              |
| `engines/api_engine/router.rs`                    | ~400   | Request-Router                 |
| `engines/api_engine/auth.rs`                      | ~400   | Authentication                 |
| `engines/api_engine/rate_limit.rs`                | ~300   | Rate-Limiting                  |
| `engines/api_engine/schema.rs`                    | ~300   | JSON Schema Validation         |
| `engines/api_engine/openapi.rs`                   | ~400   | OpenAPI Generation             |
| `engines/governance_engine/mod.rs`                | ~100   | GovernanceEngine Hauptstruct   |
| `engines/governance_engine/modes.rs`              | ~600   | GovernanceMode Implementations |
| `engines/governance_engine/proposals.rs`          | ~500   | Proposal-Management            |
| `engines/governance_engine/voting.rs`             | ~500   | Voting-Logic                   |
| `engines/governance_engine/timelock.rs`           | ~300   | Timelock-Queue                 |
| `engines/governance_engine/delegation.rs`         | ~400   | Liquid Democracy               |
| `engines/governance_engine/veto.rs`               | ~200   | Veto-Mechanism                 |
| `engines/controller_engine/mod.rs`                | ~100   | ControllerEngine Hauptstruct   |
| `engines/controller_engine/permissions.rs`        | ~400   | Permission-Management          |
| `engines/controller_engine/delegation.rs`         | ~500   | Controller-Delegation          |
| `engines/controller_engine/automation.rs`         | ~300   | Automation-Rules               |
| `engines/controller_engine/audit.rs`              | ~200   | Audit-Logging                  |
| `engines/controller_engine/governance_binding.rs` | ~300   | DAO-Integration                |
| `engines/blueprint_engine/mod.rs`                 | ~100   | BlueprintEngine Hauptstruct    |
| `engines/blueprint_engine/deployer.rs`            | ~700   | Blueprint-Deployment           |
| `engines/blueprint_engine/upgrader.rs`            | ~500   | Version-Upgrades               |
| `engines/blueprint_engine/composer.rs`            | ~600   | Blueprint-Composition          |
| `engines/blueprint_engine/migrator.rs`            | ~400   | Migration-Planning             |
| `engines/blueprint_engine/validator.rs`           | ~300   | Blueprint-Validation           |

**Gesamt Engine-Module: ~12.300 Zeilen**

#### 10.3.2 Domain-Types (`domain/unified/`)

| Datei           | Zeilen | Beschreibung                                          |
| --------------- | ------ | ----------------------------------------------------- |
| `ui.rs`         | ~400   | UI-Domain-Types (Component, Layout, Binding)          |
| `api.rs`        | ~350   | API-Domain-Types (Endpoint, Schema, Auth)             |
| `governance.rs` | ~500   | Governance-Types (Proposal, Vote, Mode)               |
| `controller.rs` | ~400   | Controller-Types (Permission, Delegation, Automation) |

**Gesamt Domain-Types: ~1.650 Zeilen**

#### 10.3.3 Storage-Module (`local/`)

| Datei                 | Zeilen | Beschreibung              |
| --------------------- | ------ | ------------------------- |
| `ui_store.rs`         | ~300   | UI-State Persistence      |
| `api_store.rs`        | ~250   | API-Registry Persistence  |
| `governance_store.rs` | ~400   | Proposal/Vote Persistence |

**Gesamt Storage: ~950 Zeilen**

#### 10.3.4 Proto/gRPC (`proto/erynoa/v1/`)

| Datei            | Zeilen | Beschreibung              |
| ---------------- | ------ | ------------------------- |
| `realm_ui.proto` | ~350   | RealmUIService Definition |

---

### 10.4 Modifikations-Checkliste

```
┌────────────────────────────────────────────────────────────────────────────────────────────┐
│                           MODIFIKATIONS-CHECKLISTE                                        │
├────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                            │
│  PHASE 1: GRUNDLAGEN (Woche 1-3)                                                          │
│  ═══════════════════════════════                                                          │
│  ☐ eclvm/bytecode.rs         +32 OpCodes, +gas_cost() cases                              │
│  ☐ domain/unified/saga.rs    +10 Goal-Varianten, +type_tag() cases                       │
│  ☐ domain/unified/realm.rs   +Room, +ExtendedPartition, +AccessPolicy, +ControllerScope  │
│  ☐ domain/unified/mod.rs     +exports für neue Types                                     │
│  ☐ eclvm/mod.rs              +engines/, +types/ exports                                  │
│                                                                                            │
│  PHASE 2: DOMAIN-TYPES (Woche 4-5)                                                        │
│  ═══════════════════════════════                                                          │
│  ☐ NEU: domain/unified/ui.rs                                                             │
│  ☐ NEU: domain/unified/api.rs                                                            │
│  ☐ NEU: domain/unified/governance.rs                                                     │
│  ☐ NEU: domain/unified/controller.rs                                                     │
│                                                                                            │
│  PHASE 3: CONTROLLER-ENGINE (Woche 6-9)                                                   │
│  ═══════════════════════════════════════                                                  │
│  ☐ NEU: eclvm/engines/mod.rs                                                             │
│  ☐ NEU: eclvm/engines/controller_engine/* (6 Dateien)                                    │
│  ☐ eclvm/runtime/host.rs     +ctrl_* Host-Functions                                      │
│  ☐ eclvm/runtime/vm.rs       +Ctrl* OpCode Handling                                      │
│                                                                                            │
│  PHASE 4: DATALOGIC-ENGINE (Woche 10-12)                                                  │
│  ═══════════════════════════════════════                                                  │
│  ☐ NEU: eclvm/engines/datalogic_engine/* (5 Dateien)                                     │
│  ☐ core/event_engine.rs      +DataLogic-Integration                                      │
│                                                                                            │
│  PHASE 5: UI-ENGINE + REALM-UI (Woche 13-17)                                              │
│  ════════════════════════════════════════════                                             │
│  ☐ NEU: eclvm/engines/ui_engine/* (10 Dateien)                                           │
│  ☐ NEU: proto/erynoa/v1/realm_ui.proto                                                   │
│  ☐ NEU: gen/realm_ui.rs (auto-generated)                                                 │
│  ☐ eclvm/runtime/host.rs     +ui_* Host-Functions                                        │
│  ☐ eclvm/runtime/vm.rs       +UI* OpCode Handling                                        │
│  ☐ NEU: local/ui_store.rs                                                                │
│  ☐ NEU: peer/ui_renderer.rs                                                              │
│                                                                                            │
│  PHASE 6: API-ENGINE (Woche 18-20)                                                        │
│  ═══════════════════════════════════                                                      │
│  ☐ NEU: eclvm/engines/api_engine/* (7 Dateien)                                           │
│  ☐ eclvm/runtime/host.rs     +api_* Host-Functions                                       │
│  ☐ eclvm/runtime/vm.rs       +API* OpCode Handling                                       │
│  ☐ NEU: local/api_store.rs                                                               │
│  ☐ NEU: peer/api_handler.rs                                                              │
│  ☐ api/routes.rs             +ECL-defined routes integration                             │
│                                                                                            │
│  PHASE 7: GOVERNANCE-ENGINE (Woche 21-24)                                                 │
│  ═════════════════════════════════════════                                                │
│  ☐ NEU: eclvm/engines/governance_engine/* (7 Dateien)                                    │
│  ☐ eclvm/runtime/host.rs     +gov_* Host-Functions                                       │
│  ☐ eclvm/runtime/vm.rs       +Gov* OpCode Handling                                       │
│  ☐ NEU: local/governance_store.rs                                                        │
│                                                                                            │
│  PHASE 8: BLUEPRINT-ENGINE (Woche 25-28)                                                  │
│  ═══════════════════════════════════════                                                  │
│  ☐ NEU: eclvm/engines/blueprint_engine/* (5 Dateien)                                     │
│  ☐ local/blueprint_marketplace.rs  MAJOR REFACTOR (ExtendedBlueprint)                    │
│  ☐ peer/saga_composer.rs     +8 neue compose_* Methoden                                  │
│                                                                                            │
│  PHASE 9: INTEGRATION (Woche 29-31)                                                       │
│  ════════════════════════════════════                                                     │
│  ☐ core/state.rs             +UIState, +APIState, +GovernanceState                       │
│  ☐ core/state_integration.rs +UIObserver, +APIObserver, +GovernanceObserver              │
│  ☐ peer/gateway.rs           +Engine-Integration                                         │
│                                                                                            │
└────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### 10.5 Abhängigkeitsmatrix für Änderungen

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                          ÄNDERUNGS-ABHÄNGIGKEITSMATRIX                                 │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                         │
│  Legende: → = "hängt ab von", ⟳ = "gegenseitige Abhängigkeit"                          │
│                                                                                         │
│  bytecode.rs (OpCodes)                                                                 │
│       │                                                                                 │
│       ├──→ vm.rs (OpCode Handling)                                                     │
│       │         │                                                                       │
│       │         └──→ host.rs (Host-Function Aufrufe)                                   │
│       │                   │                                                             │
│       │                   └──→ erynoa_host.rs (Implementation)                         │
│       │                             │                                                   │
│       │                             └──→ core/*.rs (State-Zugriff)                     │
│       │                                       │                                         │
│       │                                       └──→ local/*.rs (Persistence)            │
│       │                                                                                 │
│       └──→ compiler.rs (OpCode Generation)                                             │
│                 │                                                                       │
│                 └──→ parser.rs (AST Production)                                        │
│                           │                                                             │
│                           └──→ ast.rs (AST Types)                                      │
│                                                                                         │
│  ─────────────────────────────────────────────────────────────────────────────────────  │
│                                                                                         │
│  saga.rs (Goal Enum)                                                                   │
│       │                                                                                 │
│       ├──→ saga_composer.rs (Goal → Saga Steps)                                        │
│       │         │                                                                       │
│       │         └──→ gateway.rs (Cross-Realm Validation)                               │
│       │                                                                                 │
│       └──→ intent_parser.rs (Intent → Goal)                                            │
│                                                                                         │
│  ─────────────────────────────────────────────────────────────────────────────────────  │
│                                                                                         │
│  realm.rs (Room, Partition)                                                            │
│       │                                                                                 │
│       ├──→ realm_storage.rs (Persistence)                                              │
│       │                                                                                 │
│       ├──→ blueprint_marketplace.rs (BlueprintStructure)                               │
│       │                                                                                 │
│       └──→ controller_engine/* (ControllerScope)                                       │
│                                                                                         │
│  ─────────────────────────────────────────────────────────────────────────────────────  │
│                                                                                         │
│  blueprint_marketplace.rs (ExtendedBlueprint)                                          │
│       │                                                                                 │
│       ├──→ blueprint_engine/* (Deployment, Composition)                                │
│       │                                                                                 │
│       ├──→ ui_engine/* (BlueprintUI)                                                   │
│       │                                                                                 │
│       ├──→ api_engine/* (BlueprintAPI)                                                 │
│       │                                                                                 │
│       ├──→ governance_engine/* (BlueprintGovernance)                                   │
│       │                                                                                 │
│       └──→ controller_engine/* (BlueprintController)                                   │
│                                                                                         │
│  ─────────────────────────────────────────────────────────────────────────────────────  │
│                                                                                         │
│  realm_ui.proto (gRPC)                                                                 │
│       │                                                                                 │
│       ├──→ gen/realm_ui.rs (auto-generated)                                            │
│       │         │                                                                       │
│       │         └──→ ui_engine/grpc_bridge.rs (Implementation)                         │
│       │                       │                                                         │
│       │                       └──⟳ ui_engine/session.rs (Session-Management)           │
│       │                       │                                                         │
│       │                       └──⟳ ui_engine/streaming.rs (Bidirectional Stream)       │
│       │                                                                                 │
│       └──→ REALM-UI Frontend (external dependency)                                     │
│                                                                                         │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### 10.6 Kritische Pfade und Blocker

| Blocker | Beschreibung                               | Lösung                  | Verantwortlich |
| ------- | ------------------------------------------ | ----------------------- | -------------- |
| **B1**  | OpCodes müssen vor Engines existieren      | Phase 1 abschließen     | Backend        |
| **B2**  | HostInterface muss Engine-Functions haben  | Parallel mit Engines    | Backend        |
| **B3**  | Proto muss vor gRPC-Bridge existieren      | Proto zuerst generieren | Backend        |
| **B4**  | Realm-UI muss gRPC-Client implementieren   | Parallel-Entwicklung    | Frontend       |
| **B5**  | Blueprint-Migration bricht alte Blueprints | Migration-Script        | Backend        |
| **B6**  | Storage-Partitionen müssen existieren      | Early in Phase          | Backend        |

---

### 10.7 Zeilen-Schätzung Zusammenfassung

| Kategorie      | Neue Zeilen | Modifizierte Zeilen | Gesamt      |
| -------------- | ----------- | ------------------- | ----------- |
| Engine-Module  | ~12.300     | -                   | 12.300      |
| Domain-Types   | ~1.650      | ~500                | 2.150       |
| Storage        | ~950        | -                   | 950         |
| Proto/gRPC     | ~350        | -                   | 350         |
| Bytecode/VM    | -           | ~400                | 400         |
| Host-Interface | -           | ~300                | 300         |
| Saga/Realm     | -           | ~400                | 400         |
| Blueprint      | -           | ~800                | 800         |
| Peer-Layer     | ~400        | ~200                | 600         |
| Core-State     | -           | ~300                | 300         |
| **GESAMT**     | **~15.650** | **~2.900**          | **~18.550** |

---

## 11. Mathematische Refactoring-Logik

### 11.1 Grundlegende Definitionen

#### 11.1.1 System-Universum

Sei $\mathfrak{S}$ das **Erynoa-System-Universum**, definiert als:

$$
\mathfrak{S} = \langle \mathcal{M}, \mathcal{T}, \mathcal{R}, \mathcal{O}, \Phi, \Psi \rangle
$$

wobei:

- $\mathcal{M}$ = Menge aller **Module** (Dateien, Strukturen, Funktionen)
- $\mathcal{T}$ = Menge aller **Typen** (Structs, Enums, Traits)
- $\mathcal{R}$ = **Abhängigkeits-Relation** $\mathcal{R} \subseteq \mathcal{M} \times \mathcal{M}$
- $\mathcal{O}$ = Menge aller **OpCodes** (VM-Instruktionen)
- $\Phi$ = **Transformation-Funktion** (IST → SOLL)
- $\Psi$ = **Invarianten-Prädikat** (Korrektheit)

#### 11.1.2 Modul-Algebra

Ein **Modul** $m \in \mathcal{M}$ ist ein 6-Tupel:

$$
m = \langle \text{id}, \text{path}, \mathcal{D}_m, \mathcal{E}_m, \mathcal{I}_m, \sigma_m \rangle
$$

- $\text{id}$ : Eindeutiger Bezeichner
- $\text{path}$ : Dateisystem-Pfad
- $\mathcal{D}_m \subseteq \mathcal{T}$ : **Definierte Typen** (exports)
- $\mathcal{E}_m \subseteq \mathcal{T}$ : **Externe Abhängigkeiten** (imports)
- $\mathcal{I}_m$ : **Implementierungen** (Funktionen, Methoden)
- $\sigma_m \in \mathbb{N}$ : **Komplexitäts-Score** (Lines of Code)

Die **Abhängigkeits-Relation** ist definiert als:

$$
m_1 \xrightarrow{\mathcal{R}} m_2 \iff \mathcal{E}_{m_1} \cap \mathcal{D}_{m_2} \neq \emptyset
$$

#### 11.1.3 Typ-Hierarchie

Die **Typ-Menge** $\mathcal{T}$ ist stratifiziert:

$$
\mathcal{T} = \mathcal{T}_{\text{prim}} \cup \mathcal{T}_{\text{comp}} \cup \mathcal{T}_{\text{trait}}
$$

- $\mathcal{T}_{\text{prim}}$ : Primitive Typen (`u64`, `String`, `bool`, ...)
- $\mathcal{T}_{\text{comp}}$ : Zusammengesetzte Typen (`struct`, `enum`)
- $\mathcal{T}_{\text{trait}}$ : Trait-Definitionen

Mit der **Subtyp-Relation** $\preccurlyeq$:

$$
\tau_1 \preccurlyeq \tau_2 \iff \tau_1 \text{ implementiert } \tau_2 \text{ oder } \tau_1 \in \text{fields}(\tau_2)
$$

---

### 11.2 IST-SOLL Transformation

#### 11.2.1 System-Zustände

Definiere den **IST-Zustand** $\mathfrak{S}_0$ und **SOLL-Zustand** $\mathfrak{S}_1$:

$$
\mathfrak{S}_0 = \langle \mathcal{M}_0, \mathcal{T}_0, \mathcal{R}_0, \mathcal{O}_0, \_, \_ \rangle
$$

$$
\mathfrak{S}_1 = \langle \mathcal{M}_1, \mathcal{T}_1, \mathcal{R}_1, \mathcal{O}_1, \_, \_ \rangle
$$

Die **Transformation** $\Phi: \mathfrak{S}_0 \to \mathfrak{S}_1$ ist eine **monotone Erweiterung**:

$$
\Phi(\mathfrak{S}_0) = \mathfrak{S}_1 \implies \mathcal{M}_0 \subseteq \mathcal{M}_1 \land \mathcal{T}_0 \subseteq \mathcal{T}_1 \land \mathcal{O}_0 \subseteq \mathcal{O}_1
$$

#### 11.2.2 Delta-Mengen

Die **Änderungsmengen** (Deltas) sind:

$$
\Delta\mathcal{M} = \mathcal{M}_1 \setminus \mathcal{M}_0 \quad \text{(neue Module)}
$$

$$
\Delta\mathcal{T} = \mathcal{T}_1 \setminus \mathcal{T}_0 \quad \text{(neue Typen)}
$$

$$
\Delta\mathcal{O} = \mathcal{O}_1 \setminus \mathcal{O}_0 \quad \text{(neue OpCodes)}
$$

$$
\nabla\mathcal{M} = \{m \in \mathcal{M}_0 : m_0 \neq m_1\} \quad \text{(modifizierte Module)}
$$

**Quantifizierung aus Gap-Analyse:**

| Delta                   | Kardinalität            |
| ----------------------- | ----------------------- |
| $\|\Delta\mathcal{M}\|$ | 45 neue Dateien         |
| $\|\Delta\mathcal{T}\|$ | ~120 neue Typen         |
| $\|\Delta\mathcal{O}\|$ | 32 neue OpCodes         |
| $\|\nabla\mathcal{M}\|$ | 15 modifizierte Dateien |

---

### 11.3 Engine-Algebra

#### 11.3.1 Engine-Definition

Eine **Engine** $\mathcal{E}$ ist ein Modul-Cluster mit spezieller Struktur:

$$
\mathcal{E} = \langle \text{name}, \mathcal{M}_\mathcal{E}, \mathcal{O}_\mathcal{E}, \mathcal{H}_\mathcal{E}, \pi_\mathcal{E} \rangle
$$

- $\mathcal{M}_\mathcal{E} \subseteq \mathcal{M}$ : Module der Engine
- $\mathcal{O}_\mathcal{E} \subseteq \mathcal{O}$ : OpCodes der Engine
- $\mathcal{H}_\mathcal{E}$ : **Host-Functions** (HostInterface-Methoden)
- $\pi_\mathcal{E}: \mathcal{O}_\mathcal{E} \to \mathcal{H}_\mathcal{E}$ : **OpCode-Host-Mapping**

#### 11.3.2 Die 6 Engines

$$
\mathbb{E} = \{\mathcal{E}_{\text{UI}}, \mathcal{E}_{\text{DataLogic}}, \mathcal{E}_{\text{API}}, \mathcal{E}_{\text{Gov}}, \mathcal{E}_{\text{Ctrl}}, \mathcal{E}_{\text{Blueprint}}\}
$$

Mit expliziten Definitionen:

**UI-Engine:**

$$
\mathcal{E}_{\text{UI}} = \langle \text{"ui"}, \{m_{\text{renderer}}, m_{\text{components}}, m_{\text{bindings}}, m_{\text{grpc}}, ...\}, \mathcal{O}_{\text{UI}}, \mathcal{H}_{\text{UI}}, \pi_{\text{UI}} \rangle
$$

$$
\mathcal{O}_{\text{UI}} = \{\texttt{UIRender}, \texttt{UIBindGet}, \texttt{UIBindSet}, \texttt{UIApplyDelta}, \texttt{UITrustGate}, ...\}
$$

$$
\mathcal{H}_{\text{UI}} = \{\texttt{ui\_render}, \texttt{ui\_bind\_get}, \texttt{ui\_bind\_set}, \texttt{ui\_emit\_update}, ...\}
$$

**Governance-Engine:**

$$
\mathcal{E}_{\text{Gov}} = \langle \text{"gov"}, \{m_{\text{proposals}}, m_{\text{voting}}, m_{\text{timelock}}, m_{\text{delegation}}, ...\}, \mathcal{O}_{\text{Gov}}, \mathcal{H}_{\text{Gov}}, \pi_{\text{Gov}} \rangle
$$

$$
\mathcal{O}_{\text{Gov}} = \{\texttt{GovPropose}, \texttt{GovVote}, \texttt{GovExecute}, \texttt{GovCheckQuorum}, ...\}
$$

#### 11.3.3 Engine-Komposition

Engines können **komponiert** werden über gemeinsame Typen:

$$
\mathcal{E}_1 \otimes \mathcal{E}_2 = \langle \mathcal{M}_{\mathcal{E}_1} \cup \mathcal{M}_{\mathcal{E}_2}, \mathcal{O}_{\mathcal{E}_1} \cup \mathcal{O}_{\mathcal{E}_2}, ..., \pi_{\mathcal{E}_1} \cup \pi_{\mathcal{E}_2} \rangle
$$

Die **Kompatibilitätsbedingung**:

$$
\mathcal{E}_1 \bowtie \mathcal{E}_2 \iff \mathcal{D}_{\mathcal{E}_1} \cap \mathcal{D}_{\mathcal{E}_2} = \emptyset \lor \mathcal{D}_{\mathcal{E}_1} \cap \mathcal{D}_{\mathcal{E}_2} \subseteq \mathcal{T}_{\text{shared}}
$$

---

### 11.4 Abhängigkeits-Ordnung

#### 11.4.1 Partielle Ordnung auf Modulen

Die Abhängigkeits-Relation $\mathcal{R}$ induziert eine **partielle Ordnung** $\leq_\mathcal{R}$:

$$
m_1 \leq_\mathcal{R} m_2 \iff m_1 \xrightarrow{\mathcal{R}^*} m_2 \quad \text{(transitiver Abschluss)}
$$

Ein Modul $m$ ist **Fundament**, wenn:

$$
\text{Fund}(m) \iff \nexists m' : m' <_\mathcal{R} m
$$

#### 11.4.2 Schichten-Dekomposition

Das System zerfällt in **Schichten** $L_0, L_1, ..., L_n$:

$$
L_k = \{m \in \mathcal{M} : \max_{m' \in \mathcal{R}^{-1}(m)} \text{layer}(m') = k-1\}
$$

$$
L_0 = \{m \in \mathcal{M} : \text{Fund}(m)\}
$$

**Erynoa-Schichten:**

| Schicht | Module                                           | Beschreibung    |
| ------- | ------------------------------------------------ | --------------- |
| $L_0$   | `primitives.rs`, `error.rs`                      | Fundament-Typen |
| $L_1$   | `bytecode.rs`, `trust.rs`, `saga.rs`, `realm.rs` | Kern-Domain     |
| $L_2$   | `vm.rs`, `host.rs`, `state.rs`                   | Runtime-Layer   |
| $L_3$   | `engines/*`                                      | Engine-Layer    |
| $L_4$   | `peer/*`, `api/*`                                | Interface-Layer |
| $L_5$   | `proto/*`, `grpc_bridge.rs`                      | External-Layer  |

#### 11.4.3 Topologische Implementierungs-Ordnung

Die **Implementierungs-Reihenfolge** ist eine topologische Sortierung:

$$
\text{order}: \Delta\mathcal{M} \cup \nabla\mathcal{M} \to \mathbb{N}
$$

sodass:

$$
m_1 \xrightarrow{\mathcal{R}_1} m_2 \implies \text{order}(m_1) < \text{order}(m_2)
$$

**Konkret:**

$$
\text{order}(\texttt{bytecode.rs}) < \text{order}(\texttt{vm.rs}) < \text{order}(\texttt{host.rs}) < \text{order}(\texttt{engines/*})
$$

---

### 11.5 Typ-Erweiterungs-Kalkül

#### 11.5.1 Enum-Erweiterung

Für ein Enum $E \in \mathcal{T}_{\text{comp}}$ mit Varianten $V(E) = \{v_1, ..., v_n\}$:

Die **Erweiterungs-Operation** $\oplus$:

$$
E' = E \oplus \{v_{n+1}, ..., v_{n+k}\}
$$

**Constraint:** Alle Pattern-Matches müssen aktualisiert werden:

$$
\forall f \in \mathcal{I} : \text{matches}(f, E) \implies \text{update}(f, E')
$$

**Beispiel Goal-Enum:**

$$
\texttt{Goal}_0 = \{\texttt{Transfer}, \texttt{Attest}, \texttt{Delegate}, \texttt{Query}, \texttt{Create}, \texttt{Complex}\}
$$

$$
\texttt{Goal}_1 = \texttt{Goal}_0 \oplus \{\texttt{RoomCreate}, \texttt{UIModify}, \texttt{Vote}, \texttt{CrossRealm}, ...\}
$$

$$
|\texttt{Goal}_1| = |\texttt{Goal}_0| + 10 = 16
$$

#### 11.5.2 Struct-Erweiterung

Für ein Struct $S$ mit Feldern $F(S)$:

$$
S' = S \oplus_{\text{fields}} \{f_{n+1}, ..., f_{n+k}\}
$$

**Kompatibilitäts-Constraint:**

$$
\text{Backward}(S, S') \iff \forall f \in F(S) : f \in F(S') \land \text{type}(f, S) = \text{type}(f, S')
$$

**Beispiel Blueprint:**

$$
\texttt{Blueprint}_0.\text{fields} = \{\texttt{stores}, \texttt{policies}, \texttt{sagas}, ...\}
$$

$$
\texttt{Blueprint}_1.\text{fields} = \texttt{Blueprint}_0.\text{fields} \cup \{\texttt{structure}, \texttt{ui}, \texttt{datalogic}, \texttt{api}, \texttt{governance}, \texttt{controller}\}
$$

#### 11.5.3 Trait-Erweiterung

Für ein Trait $T$ mit Methoden $M(T)$:

$$
T' = T \oplus_{\text{methods}} \{m_{n+1}, ..., m_{n+k}\}
$$

**Default-Implementation-Constraint:**

$$
\forall m \in M(T') \setminus M(T) : \exists \text{default}(m) \lor \forall \text{impl}(T') : \text{defines}(m)
$$

**Beispiel HostInterface:**

$$
|\texttt{HostInterface}_0.\text{methods}| = 25
$$

$$
|\texttt{HostInterface}_1.\text{methods}| = 25 + 20 = 45
$$

---

### 11.6 OpCode-Erweiterungs-Algebra

#### 11.6.1 OpCode-Raum

Der OpCode-Raum $\mathcal{O}$ ist strukturiert als:

$$
\mathcal{O} = \bigsqcup_{c \in \mathcal{C}} \mathcal{O}_c
$$

wobei $\mathcal{C}$ die Kategorien sind:

$$
\mathcal{C} = \{\text{Stack}, \text{Arith}, \text{Logic}, \text{Control}, \text{Trust}, \text{Host}, \text{UI}, \text{API}, \text{Gov}, \text{Ctrl}\}
$$

#### 11.6.2 Gas-Kosten-Funktion

Die **Gas-Funktion** $\gamma: \mathcal{O} \to \mathbb{N}^+$ muss erweitert werden:

$$
\gamma_1 = \gamma_0 \cup \{(o, g) : o \in \Delta\mathcal{O}\}
$$

**Gas-Kategorien:**

| Kategorie                            | Gas-Bereich   | Begründung         |
| ------------------------------------ | ------------- | ------------------ |
| $\gamma(\mathcal{O}_{\text{Stack}})$ | $[1, 5]$      | Reine Stack-Ops    |
| $\gamma(\mathcal{O}_{\text{Trust}})$ | $[10, 50]$    | Trust-Berechnungen |
| $\gamma(\mathcal{O}_{\text{UI}})$    | $[10, 200]$   | Rendering-Overhead |
| $\gamma(\mathcal{O}_{\text{Gov}})$   | $[100, 1000]$ | State-Changing     |
| $\gamma(\mathcal{O}_{\text{Ctrl}})$  | $[50, 200]$   | Security-Critical  |

#### 11.6.3 VM-Dispatch-Erweiterung

Die VM-Dispatch-Funktion $\delta: \mathcal{O} \times \Sigma \to \Sigma$ (Stack-Transformation):

$$
\delta_1 = \delta_0 \cup \{(o, \sigma) \mapsto \sigma' : o \in \Delta\mathcal{O}\}
$$

Jeder neue OpCode benötigt einen **Dispatch-Case**:

$$
|\text{match-arms}(\delta_1)| = |\text{match-arms}(\delta_0)| + |\Delta\mathcal{O}| = 65 + 32 = 97
$$

---

### 11.7 Realm-Hierarchie-Algebra

#### 11.7.1 Hierarchie-Struktur

Die **Realm-Hierarchie** ist ein Baum:

$$
\mathcal{H} = \langle \mathcal{V}, \mathcal{A}, r \rangle
$$

- $\mathcal{V} = \mathcal{V}_{\text{Realm}} \cup \mathcal{V}_{\text{Room}} \cup \mathcal{V}_{\text{Partition}}$
- $\mathcal{A} \subseteq \mathcal{V} \times \mathcal{V}$ : Eltern-Kind-Relation
- $r \in \mathcal{V}_{\text{Realm}}$ : Wurzel (ROOT_REALM)

**Hierarchie-Constraint:**

$$
\forall v \in \mathcal{V}_{\text{Room}} : \exists! u \in \mathcal{V}_{\text{Realm}} : (u, v) \in \mathcal{A}
$$

$$
\forall v \in \mathcal{V}_{\text{Partition}} : \exists! u \in \mathcal{V}_{\text{Room}} : (u, v) \in \mathcal{A}
$$

#### 11.7.2 Scope-Propagation

Controller-Scope propagiert durch die Hierarchie:

$$
\text{scope}: \mathcal{V} \to \mathcal{P}(\text{Permission})
$$

$$
\text{scope}(v) \supseteq \bigcup_{(u,v) \in \mathcal{A}} \text{inherit}(\text{scope}(u))
$$

Mit **Einschränkung**:

$$
\text{restrict}: \text{Permission} \times \mathcal{V} \to \text{Permission}
$$

---

### 11.8 gRPC-Stream-Algebra

#### 11.8.1 Bidirektionaler Stream

Der Realm-UI gRPC-Stream ist ein **bidirektionaler Kanal**:

$$
\text{Stream} = \langle \mathcal{C}_{\text{in}}, \mathcal{C}_{\text{out}}, \sigma \rangle
$$

- $\mathcal{C}_{\text{in}}$ : Client → Server Nachrichten (Actions)
- $\mathcal{C}_{\text{out}}$ : Server → Client Nachrichten (Updates)
- $\sigma$ : Session-State

**Nachrichtentypen:**

$$
\mathcal{C}_{\text{in}} = \{\texttt{Connect}, \texttt{ExecuteAction}, \texttt{NavigateTo}, \texttt{Disconnect}\}
$$

$$
\mathcal{C}_{\text{out}} = \{\texttt{InitialUI}, \texttt{UIDelta}, \texttt{Error}, \texttt{SessionExpired}\}
$$

#### 11.8.2 Session-Invariante

Für eine Session $s$ mit Peer $p$ und Trust $\tau(p)$:

$$
\forall \text{ui} \in \mathcal{C}_{\text{out}} : \text{visible}(\text{ui}, p) \implies \tau(p) \geq \text{trust\_gate}(\text{ui})
$$

---

### 11.9 Invarianten und Korrektheit

#### 11.9.1 System-Invarianten

Das **Invarianten-Prädikat** $\Psi$ garantiert:

**I1 - Typ-Konsistenz:**

$$
\Psi_1: \forall m \in \mathcal{M} : \forall t \in \mathcal{E}_m : \exists m' \in \mathcal{M} : t \in \mathcal{D}_{m'}
$$

**I2 - OpCode-Vollständigkeit:**

$$
\Psi_2: \forall o \in \mathcal{O} : \exists \text{dispatch}(o) \land \exists \gamma(o)
$$

**I3 - Host-Function-Mapping:**

$$
\Psi_3: \forall \mathcal{E} \in \mathbb{E} : \forall o \in \mathcal{O}_\mathcal{E} : \exists h \in \mathcal{H}_\mathcal{E} : \pi_\mathcal{E}(o) = h
$$

**I4 - Trait-Implementation:**

$$
\Psi_4: \forall T \in \mathcal{T}_{\text{trait}} : \forall m \in M(T) : \text{has\_default}(m) \lor \forall \text{impl}(T) : \text{defines}(m)
$$

**I5 - Hierarchie-Wohlgeformtheit:**

$$
\Psi_5: \mathcal{H} \text{ ist azyklisch} \land \forall v \in \mathcal{V} \setminus \{r\} : |\{u : (u,v) \in \mathcal{A}\}| = 1
$$

#### 11.9.2 Transformations-Korrektheit

Die Transformation $\Phi$ ist **korrekt**, wenn:

$$
\Psi(\mathfrak{S}_0) \land \Phi(\mathfrak{S}_0) = \mathfrak{S}_1 \implies \Psi(\mathfrak{S}_1)
$$

**Beweisstrategie:** Induktion über Phasen:

$$
\Phi = \Phi_9 \circ \Phi_8 \circ ... \circ \Phi_1
$$

$$
\forall i : \Psi(\mathfrak{S}_{i-1}) \implies \Psi(\Phi_i(\mathfrak{S}_{i-1}))
$$

---

### 11.10 Phasen-Sequenz-Kalkül

#### 11.10.1 Phasen als Transformationen

Jede Phase $\Phi_i$ ist eine **lokale Transformation**:

$$
\Phi_i: \mathfrak{S}_{i-1} \to \mathfrak{S}_i
$$

mit **Fokus-Menge** $\mathcal{F}_i \subseteq \mathcal{M}$:

$$
\forall m \notin \mathcal{F}_i : m \in \mathfrak{S}_{i-1} \iff m \in \mathfrak{S}_i
$$

#### 11.10.2 Phasen-Abhängigkeiten

Die Phasen-Abhängigkeit ist eine **strikte Ordnung**:

$$
\Phi_i \prec \Phi_j \iff \mathcal{F}_j \cap \text{depends}(\mathcal{F}_i) \neq \emptyset
$$

**Phasen-DAG:**

```
Φ₁ (Grundlagen)
 │
 ├──→ Φ₂ (Domain-Types)
 │         │
 │         ├──→ Φ₃ (Controller-Engine)
 │         │         │
 │         │         ├──→ Φ₄ (DataLogic-Engine)
 │         │         │
 │         │         └──→ Φ₅ (UI-Engine + gRPC)
 │         │                   │
 │         │                   └──→ Φ₆ (API-Engine)
 │         │                             │
 │         │                             └──→ Φ₇ (Governance-Engine)
 │         │                                       │
 │         └─────────────────────────────────────→ Φ₈ (Blueprint-Engine)
 │                                                       │
 └──────────────────────────────────────────────────────→ Φ₉ (Integration)
```

#### 11.10.3 Parallelisierbarkeit

Phasen $\Phi_i, \Phi_j$ sind **parallelisierbar**, wenn:

$$
\Phi_i \| \Phi_j \iff \neg(\Phi_i \prec \Phi_j) \land \neg(\Phi_j \prec \Phi_i) \land \mathcal{F}_i \cap \mathcal{F}_j = \emptyset
$$

**Parallele Gruppen:**

$$
\mathcal{P}_1 = \{\Phi_4, \Phi_5\} \quad \text{(DataLogic + UI parallel)}
$$

$$
\mathcal{P}_2 = \{\Phi_6, \Phi_7\} \quad \text{(API + Governance teilweise parallel)}
$$

---

### 11.11 Metriken und Komplexität

#### 11.11.1 Transformations-Kosten

Die **Gesamtkosten** $\mathcal{K}$ der Transformation:

$$
\mathcal{K}(\Phi) = \sum_{m \in \Delta\mathcal{M}} \kappa_{\text{new}}(m) + \sum_{m \in \nabla\mathcal{M}} \kappa_{\text{mod}}(m)
$$

mit:

- $\kappa_{\text{new}}(m) = \sigma_m$ (Lines of Code für neue Module)
- $\kappa_{\text{mod}}(m) = |\text{diff}(m_0, m_1)|$ (geänderte Zeilen)

**Aus Gap-Analyse:**

$$
\mathcal{K}(\Phi) = 15.650 + 2.900 = 18.550 \text{ LOC}
$$

#### 11.11.2 Komplexitäts-Wachstum

Das **Komplexitäts-Verhältnis**:

$$
\rho = \frac{\sigma(\mathfrak{S}_1)}{\sigma(\mathfrak{S}_0)} = \frac{\sum_{m \in \mathcal{M}_1} \sigma_m}{\sum_{m \in \mathcal{M}_0} \sigma_m}
$$

Geschätzt:

$$
\rho \approx \frac{45.000 + 18.550}{45.000} \approx 1.41 \quad \text{(41\% Wachstum)}
$$

#### 11.11.3 Abhängigkeits-Dichte

Die **Abhängigkeits-Dichte** vor und nach:

$$
\delta_\mathcal{R} = \frac{|\mathcal{R}|}{|\mathcal{M}|^2}
$$

**Ziel:** $\delta_{\mathcal{R}_1} \leq \delta_{\mathcal{R}_0}$ (keine Erhöhung der Kopplung)

---

### 11.12 Zusammenfassung: Refactoring-Axiome

Die **Refactoring-Logik** basiert auf folgenden Axiomen:

$$
\boxed{
\begin{aligned}
\textbf{A1 (Monotonie):} \quad & \mathfrak{S}_0 \subseteq \mathfrak{S}_1 \\
\textbf{A2 (Schichtung):} \quad & \forall m \in \Delta\mathcal{M} : \text{layer}(m) \geq \max_{m' \in \mathcal{R}^{-1}(m)} \text{layer}(m') \\
\textbf{A3 (Invarianz):} \quad & \Psi(\mathfrak{S}_0) \implies \Psi(\Phi(\mathfrak{S}_0)) \\
\textbf{A4 (Komposition):} \quad & \Phi = \Phi_n \circ ... \circ \Phi_1 \\
\textbf{A5 (Lokalität):} \quad & |\mathcal{F}_i| \ll |\mathcal{M}| \quad \text{für alle } \Phi_i \\
\textbf{A6 (Testbarkeit):} \quad & \forall \Phi_i : \exists \mathcal{T}_i : \text{verifies}(\mathcal{T}_i, \Phi_i)
\end{aligned}
}
$$

Diese Axiome garantieren eine **sichere, inkrementelle Transformation** von $\mathfrak{S}_0$ nach $\mathfrak{S}_1$.

---

## 12. P2P-Kommunikations-Architektur

### 12.1 Übersicht: Was wird P2P kommuniziert?

```
┌────────────────────────────────────────────────────────────────────────────────────────────┐
│                         P2P-KOMMUNIKATIONS-MATRIX                                         │
├────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                            │
│   ┌─────────────┐    Gossipsub Topics     ┌─────────────┐                                │
│   │   PEER A    │◄─────────────────────►│   PEER B    │                                │
│   │  (Node)     │    Request-Response     │  (Node)     │                                │
│   └──────┬──────┘◄─────────────────────►└──────┬──────┘                                │
│          │                                      │                                          │
│   ┌──────┴──────────────────────────────────────┴──────┐                                  │
│   │              KOMMUNIZIERTE DATENTYPEN              │                                  │
│   ├────────────────────────────────────────────────────┤                                  │
│   │  📦 Events         - Realm-Events, DAG-Sync        │                                  │
│   │  🔒 Trust          - Attestationen, Trust-Vektoren │                                  │
│   │  📋 Sagas          - Cross-Peer-Intents, Status    │                                  │
│   │  🏛️ Realms        - Membership, Rules, Config     │                                  │
│   │  📐 Blueprints     - Templates, Versionen, Sync    │                                  │
│   │  🖼️ UI-State      - Realm-UI-Deltas (NEU)         │                                  │
│   │  🗳️ Governance    - Proposals, Votes (NEU)        │                                  │
│   │  🔑 Controller    - Delegations, Permissions (NEU) │                                  │
│   │  🌐 API-Registry  - Endpoint-Announcements (NEU)   │                                  │
│   └────────────────────────────────────────────────────┘                                  │
│                                                                                            │
└────────────────────────────────────────────────────────────────────────────────────────────┘
```

### 12.2 Mathematisches P2P-Modell

#### 12.2.1 Netzwerk-Topologie

Das P2P-Netzwerk $\mathcal{N}$ ist ein **dynamischer Graph**:

$$
\mathcal{N}(t) = \langle \mathcal{P}(t), \mathcal{C}(t), \tau \rangle
$$

- $\mathcal{P}(t)$ : Menge aktiver **Peers** zum Zeitpunkt $t$
- $\mathcal{C}(t) \subseteq \mathcal{P}(t) \times \mathcal{P}(t)$ : **Verbindungen**
- $\tau: \mathcal{C}(t) \to \mathbb{R}^6$ : **Trust-Annotation** pro Verbindung

#### 12.2.2 Nachrichten-Algebra

Eine **P2P-Nachricht** $\mu$ ist formal:

$$
\mu = \langle \text{type}, \text{topic}, \text{payload}, \text{sig}, \text{meta} \rangle
$$

Mit **Nachrichtentypen** $\mathcal{M}_{\text{type}}$:

$$
\mathcal{M}_{\text{type}} = \{\text{Event}, \text{Trust}, \text{Saga}, \text{Realm}, \text{Blueprint}, \text{UI}, \text{Gov}, \text{Ctrl}, \text{API}\}
$$

#### 12.2.3 Topic-Struktur

Topics $\mathcal{T}$ sind hierarchisch organisiert:

$$
\text{topic}(\text{realm}, \text{type}) = \texttt{/erynoa/realm/}\langle\text{realm\_id}\rangle\texttt{/}\langle\text{type}\rangle\texttt{/v1}
$$

**Aktuelle Topics:**

| Topic-Pattern                        | Beschreibung           | Nachrichtentyp     |
| ------------------------------------ | ---------------------- | ------------------ |
| `/erynoa/realm/{id}/events/v1`       | Event-DAG-Propagation  | `RealmEvent`       |
| `/erynoa/realm/{id}/trust/v1`        | Trust-Attestationen    | `TrustAttestation` |
| `/erynoa/realm/{id}/sagas/v1`        | Saga-Broadcasts        | `SagaStatus`       |
| `/erynoa/direct/{sender}/{receiver}` | Direct Messages        | `DirectMessage`    |
| `/erynoa/global/announcements/v1`    | Netzwerk-Announcements | `Announcement`     |

**NEU zu implementierende Topics:**

| Topic-Pattern                      | Beschreibung            | Nachrichtentyp          |
| ---------------------------------- | ----------------------- | ----------------------- |
| `/erynoa/realm/{id}/blueprints/v1` | Blueprint-Announcements | `BlueprintAnnouncement` |
| `/erynoa/realm/{id}/ui/v1`         | UI-State-Deltas         | `UIStateDelta`          |
| `/erynoa/realm/{id}/governance/v1` | Proposals & Votes       | `GovernanceMessage`     |
| `/erynoa/realm/{id}/controller/v1` | Controller-Updates      | `ControllerMessage`     |
| `/erynoa/realm/{id}/api/v1`        | API-Registry-Updates    | `APIRegistryMessage`    |
| `/erynoa/room/{id}/events/v1`      | Room-spezifische Events | `RoomEvent`             |
| `/erynoa/room/{id}/ui/v1`          | Room-UI-Deltas          | `RoomUIDelta`           |

---

### 12.3 Protokoll-Spezifikationen

#### 12.3.1 Sync-Protokolle (Request-Response)

**IST-Zustand:**

```rust
// Existierende Protokolle in protocol.rs
pub const EVENTS: &'static str = "/erynoa/sync/events/1.0";
pub const TRUST: &'static str = "/erynoa/sync/trust/1.0";
pub const MEMBERSHIP: &'static str = "/erynoa/sync/membership/1.0";
```

**SOLL-Zustand (Erweiterungen):**

```rust
// NEU: Erweiterte Sync-Protokolle
pub const BLUEPRINTS: &'static str = "/erynoa/sync/blueprints/1.0";
pub const UI_STATE: &'static str = "/erynoa/sync/ui/1.0";
pub const GOVERNANCE: &'static str = "/erynoa/sync/governance/1.0";
pub const CONTROLLER: &'static str = "/erynoa/sync/controller/1.0";
pub const API_REGISTRY: &'static str = "/erynoa/sync/api/1.0";
pub const ROOM_STATE: &'static str = "/erynoa/sync/rooms/1.0";
```

#### 12.3.2 Nachrichtenstrukturen

**Erweiterte SyncRequest:**

```rust
pub enum SyncRequest {
    // ... bestehende Varianten ...

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: Blueprint-Sync
    // ═══════════════════════════════════════════════════════════════════════

    /// Hole Blueprint-Katalog für Realm
    GetBlueprintCatalog {
        realm_id: String,
        category: Option<BlueprintCategory>,
        after_version: Option<SemVer>,
        limit: usize,
    },

    /// Hole Blueprint-Details
    GetBlueprint {
        blueprint_id: String,
        include_bytecode: bool,
    },

    /// Suche Blueprints
    SearchBlueprints {
        query: String,
        tags: Vec<String>,
        min_trust: Option<f32>,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: Room-Sync
    // ═══════════════════════════════════════════════════════════════════════

    /// Hole Room-Struktur
    GetRoomStructure {
        realm_id: String,
    },

    /// Hole Room-State
    GetRoomState {
        room_id: String,
        include_partitions: bool,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: UI-State-Sync
    // ═══════════════════════════════════════════════════════════════════════

    /// Hole UI-State für Room
    GetUIState {
        room_id: String,
        peer_did: String,  // Für Trust-Gate-Filterung
    },

    /// Synchronisiere UI-Bindings
    SyncUIBindings {
        room_id: String,
        binding_hashes: Vec<String>,  // Nur geänderte anfordern
    },

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: Governance-Sync
    // ═══════════════════════════════════════════════════════════════════════

    /// Hole aktive Proposals
    GetActiveProposals {
        scope: String,  // realm_id oder room_id
        status_filter: Option<ProposalStatus>,
    },

    /// Hole Votes für Proposal
    GetVotes {
        proposal_id: String,
    },

    /// Verifiziere Vote
    VerifyVote {
        proposal_id: String,
        voter_did: String,
        vote_hash: String,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: Controller-Sync
    // ═══════════════════════════════════════════════════════════════════════

    /// Hole Controller-Hierarchie
    GetControllerHierarchy {
        scope: String,
    },

    /// Hole Delegations für Actor
    GetDelegations {
        actor_did: String,
        scope: Option<String>,
    },

    /// Verifiziere Permission
    VerifyPermission {
        actor_did: String,
        scope: String,
        action: String,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: API-Registry-Sync
    // ═══════════════════════════════════════════════════════════════════════

    /// Hole API-Endpoints für Scope
    GetAPIEndpoints {
        scope: String,
    },

    /// Hole OpenAPI-Spec
    GetOpenAPISpec {
        scope: String,
        version: Option<String>,
    },
}
```

---

### 12.4 Trust-basierte P2P-Sicherheit

#### 12.4.1 Connection-Level-Matrix

Die **Connection-Level** bestimmen P2P-Fähigkeiten:

$$
\text{level}: \mathcal{P} \times \mathcal{P} \to \{\text{Blocked}, \text{Limited}, \text{Standard}, \text{Full}, \text{Trusted}\}
$$

| Level        | Receive Events | Send Events | Relay | Sync | Gov-Vote | Ctrl-Delegate |
| ------------ | -------------- | ----------- | ----- | ---- | -------- | ------------- |
| **Blocked**  | ❌             | ❌          | ❌    | ❌   | ❌       | ❌            |
| **Limited**  | ✅             | ❌          | ❌    | ❌   | ❌       | ❌            |
| **Standard** | ✅             | ✅          | ❌    | ✅   | ✅       | ❌            |
| **Full**     | ✅             | ✅          | ✅    | ✅   | ✅       | ✅            |
| **Trusted**  | ✅             | ✅          | ✅    | ✅   | ✅       | ✅            |

#### 12.4.2 Trust-Gate-Berechnung

Der **Connection-Level** wird berechnet aus Trust-Vektor:

$$
\text{level}(p_1, p_2) = f(\tau(p_1, p_2), \text{config})
$$

Wobei $\tau = [r, i, c, p, v, \omega]$ und:

$$
f(\tau) = \begin{cases}
\text{Trusted} & \text{wenn } r \geq 0.9 \land \omega \geq 0.8 \\
\text{Full} & \text{wenn } r \geq 0.7 \land i \geq 0.6 \\
\text{Standard} & \text{wenn } r \geq 0.4 \land v \geq 0.3 \\
\text{Limited} & \text{wenn } r \geq 0.1 \\
\text{Blocked} & \text{sonst (oder explizit gebannt)}
\end{cases}
$$

#### 12.4.3 Newcomer-Grace-Period

Neue Peers erhalten eine **Grace-Period** (Κ2):

$$
\tau_{\text{newcomer}}(t) = \tau_{\text{base}} \cdot \min\left(1, \frac{t - t_{\text{join}}}{T_{\text{grace}}}\right)
$$

Mit $T_{\text{grace}} = 7 \text{ Tage}$ und $\tau_{\text{base}} = [0.5, 0.5, 0.5, 0.3, 0.5, 0.1]$.

---

### 12.5 Realm-Synchronisation

#### 12.5.1 Realm-State-Komponenten

Ein **Realm-State** $\mathcal{S}_R$ besteht aus:

$$
\mathcal{S}_R = \langle \text{config}, \text{rules}, \text{rooms}, \text{members}, \text{blueprints}, \text{controller}, \text{gov} \rangle
$$

#### 12.5.2 Sync-Strategie: CRDT-basiert

Realm-State verwendet **CRDTs** (Conflict-free Replicated Data Types):

| Komponente             | CRDT-Typ     | Merge-Semantik   |
| ---------------------- | ------------ | ---------------- |
| `config`               | LWW-Register | Last-Writer-Wins |
| `rules`                | OR-Set       | Union            |
| `rooms`                | OR-Set       | Union            |
| `members`              | OR-Set + LWW | Union, Level=LWW |
| `blueprints`           | G-Set        | Append-only      |
| `controller`           | LWW-Map      | Per-Scope LWW    |
| `governance.proposals` | OR-Set       | Union            |
| `governance.votes`     | G-Set        | Append-only      |

#### 12.5.3 Sync-Protokoll-Flow

```
┌──────────┐                              ┌──────────┐
│  Peer A  │                              │  Peer B  │
└────┬─────┘                              └────┬─────┘
     │                                         │
     │  1. VerifyMembership(realm_id, did_A)  │
     │────────────────────────────────────────►│
     │                                         │
     │  2. MembershipVerified(is_member, level)│
     │◄────────────────────────────────────────│
     │                                         │
     │  [if is_member && level >= Standard]    │
     │                                         │
     │  3. GetRoomStructure(realm_id)          │
     │────────────────────────────────────────►│
     │                                         │
     │  4. RoomStructure(rooms[], partitions[])│
     │◄────────────────────────────────────────│
     │                                         │
     │  5. Subscribe(topic: /realm/{id}/*)     │
     │────────────────────────────────────────►│
     │                                         │
     │  6. Gossipsub: Events, Trust, UI, Gov   │
     │◄────────────────────────────────────────│
     │                                         │
```

---

### 12.6 Blueprint-Propagation

#### 12.6.1 Blueprint-Announcement-Protokoll

Wenn ein Peer ein neues Blueprint erstellt/deployed:

$$
\text{announce}(\text{blueprint}) \to \text{topic}(\text{realm}, \texttt{blueprints})
$$

**BlueprintAnnouncement Message:**

```rust
pub struct BlueprintAnnouncement {
    /// Blueprint-ID (Content-Hash)
    pub blueprint_id: BlueprintId,

    /// Version
    pub version: SemVer,

    /// Ersteller-DID
    pub creator_did: String,

    /// Signatur des Erstellers
    pub signature: Signature,

    /// Metadaten (Name, Beschreibung, Tags)
    pub metadata: BlueprintMetadata,

    /// Kategorie
    pub category: BlueprintCategory,

    /// Trust-Minimum für Download
    pub trust_gate: f32,

    /// Komponenten-Summary (ohne Bytecode)
    pub components: BlueprintComponentsSummary,

    /// Where to fetch full blueprint
    pub fetch_info: FetchInfo,
}

pub struct BlueprintComponentsSummary {
    pub has_structure: bool,
    pub has_ui: bool,
    pub has_datalogic: bool,
    pub has_api: bool,
    pub has_governance: bool,
    pub has_controller: bool,
    pub store_count: u32,
    pub policy_count: u32,
    pub complexity_score: u64,
}
```

#### 12.6.2 Blueprint-Fetch-Strategie

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    BLUEPRINT FETCH FLOW                                  │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   1. Announcement via Gossipsub                                          │
│   ┌────────┐  BlueprintAnnouncement   ┌────────┐  ┌────────┐            │
│   │Creator │ ─────────────────────►  │ DHT    │  │ Peers  │            │
│   └────────┘                          └────────┘  └────────┘            │
│                                                                          │
│   2. Interest Peer requests via Request-Response                         │
│   ┌────────┐  GetBlueprint(id)        ┌────────┐                        │
│   │ Peer   │ ─────────────────────►  │ DHT    │ (find providers)        │
│   └────────┘                          └────────┘                        │
│       │                                    │                             │
│       │         GetBlueprint(id)           ▼                             │
│       └─────────────────────────────► ┌────────┐                        │
│                                        │Provider│                        │
│       ◄─────────────────────────────  └────────┘                        │
│            Blueprint(full data)                                          │
│                                                                          │
│   3. Verify & Store                                                      │
│   - Verify content_hash == blueprint_id                                  │
│   - Verify creator_signature                                             │
│   - Check trust_gate against peer trust                                  │
│   - Store in local BlueprintMarketplace                                  │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

#### 12.6.3 Trust-gated Blueprint Access

Blueprints können **Trust-Gates** haben:

$$
\text{can\_access}(p, b) = \tau_r(p) \geq b.\text{trust\_gate}
$$

**Access-Levels:**

| Trust-Gate | Beschreibung | Typische Blueprints    |
| ---------- | ------------ | ---------------------- |
| 0.0        | Public       | Open-Source, Community |
| 0.3        | Basic        | Standard-Features      |
| 0.5        | Verified     | Erweiterte Features    |
| 0.7        | Premium      | Business-Critical      |
| 0.9        | Exclusive    | High-Security          |

---

### 12.7 UI-State-Propagation (Realm-UI Integration)

#### 12.7.1 UI-State-Sync-Architektur

```
┌────────────────────────────────────────────────────────────────────────────────────┐
│                    UI-STATE P2P PROPAGATION                                       │
├────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                    │
│   ┌─────────────┐      gRPC        ┌─────────────┐     P2P      ┌─────────────┐  │
│   │  Realm-UI   │◄───────────────►│  Backend A  │◄────────────►│  Backend B  │  │
│   │  (Browser)  │   Bidirectional  │  UI-Engine  │  Gossipsub   │  UI-Engine  │  │
│   └─────────────┘                  └──────┬──────┘              └──────┬──────┘  │
│                                           │                            │          │
│                                    ┌──────┴──────┐              ┌──────┴──────┐  │
│                                    │  UI-State   │    CRDT      │  UI-State   │  │
│                                    │  (Local)    │◄────────────►│  (Local)    │  │
│                                    └─────────────┘    Merge     └─────────────┘  │
│                                                                                    │
└────────────────────────────────────────────────────────────────────────────────────┘
```

#### 12.7.2 UIStateDelta Message

```rust
pub struct UIStateDelta {
    /// Room-ID
    pub room_id: RoomId,

    /// Delta-Typ
    pub delta_type: UIDeltaType,

    /// Betroffene Components
    pub affected_components: Vec<ComponentId>,

    /// Binding-Updates
    pub binding_updates: Vec<BindingUpdate>,

    /// Trust-gated (welcher Trust sieht was)
    pub trust_visibility: HashMap<String, f32>,

    /// Vector-Clock für Ordering
    pub vector_clock: VectorClock,

    /// Signatur
    pub signature: Signature,
}

pub enum UIDeltaType {
    /// Binding-Wert geändert
    BindingChange,
    /// Component hinzugefügt
    ComponentAdded,
    /// Component entfernt
    ComponentRemoved,
    /// Layout geändert
    LayoutChange,
    /// Visibility geändert (Trust-Gate)
    VisibilityChange,
}
```

#### 12.7.3 Trust-Filtered UI Propagation

UI-Deltas werden **Trust-gefiltert** propagiert:

$$
\text{propagate}(\delta, p) = \begin{cases}
\delta & \text{wenn } \tau_r(p) \geq \delta.\text{trust\_visibility} \\
\text{filter}(\delta, \tau_r(p)) & \text{sonst}
\end{cases}
$$

---

### 12.8 Governance P2P-Protokoll

#### 12.8.1 Proposal-Lifecycle P2P

```
┌────────────────────────────────────────────────────────────────────────────────────┐
│                    GOVERNANCE P2P FLOW                                            │
├────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                    │
│   1. PROPOSAL CREATION                                                             │
│   ┌────────────┐  ProposalCreated   ┌────────────┐                               │
│   │ Proposer   │ ─────────────────► │ Gossipsub  │ → All Realm Members           │
│   └────────────┘                    │ /gov/v1    │                               │
│                                     └────────────┘                               │
│                                                                                    │
│   2. VOTING PERIOD                                                                 │
│   ┌────────────┐  VoteCast          ┌────────────┐                               │
│   │ Voter      │ ─────────────────► │ Gossipsub  │ → All Realm Members           │
│   └────────────┘                    │ /gov/v1    │                               │
│         │                           └────────────┘                               │
│         │                                                                         │
│         ▼                                                                         │
│   ┌────────────────────────────────────────────────────┐                         │
│   │ Vote = { proposal_id, choice, voting_power, sig }  │                         │
│   │ Votes are CRDT (G-Set): append-only, no conflicts  │                         │
│   └────────────────────────────────────────────────────┘                         │
│                                                                                    │
│   3. EXECUTION (after timelock)                                                   │
│   ┌────────────┐  ProposalExecuted  ┌────────────┐                               │
│   │ Executor   │ ─────────────────► │ Gossipsub  │ → All Realm Members           │
│   └────────────┘                    │ /gov/v1    │                               │
│                                     └────────────┘                               │
│                                                                                    │
└────────────────────────────────────────────────────────────────────────────────────┘
```

#### 12.8.2 Governance Messages

```rust
pub enum GovernanceMessage {
    /// Neues Proposal erstellt
    ProposalCreated {
        proposal_id: ProposalId,
        proposer_did: String,
        scope: ScopeId,
        title: String,
        description: String,
        actions: Vec<ProposedAction>,
        voting_starts: u64,
        voting_ends: u64,
        quorum: f32,
        signature: Signature,
    },

    /// Vote abgegeben
    VoteCast {
        proposal_id: ProposalId,
        voter_did: String,
        choice: VoteChoice,  // For, Against, Abstain
        voting_power: f32,
        delegated_from: Option<String>,  // Liquid Democracy
        signature: Signature,
    },

    /// Proposal-Status geändert
    ProposalStatusChanged {
        proposal_id: ProposalId,
        old_status: ProposalStatus,
        new_status: ProposalStatus,
        reason: Option<String>,
    },

    /// Proposal ausgeführt
    ProposalExecuted {
        proposal_id: ProposalId,
        executor_did: String,
        execution_proof: Vec<u8>,
        signature: Signature,
    },

    /// Veto eingelegt
    VetoCast {
        proposal_id: ProposalId,
        vetoer_did: String,
        reason: String,
        signature: Signature,
    },
}
```

#### 12.8.3 Vote-Verification

Votes werden **kryptographisch verifiziert**:

$$
\text{verify}(v) = \text{ed25519\_verify}(v.\text{sig}, v.\text{voter\_did}, \text{hash}(v))
$$

Und gegen **Realm-Membership** geprüft:

$$
\text{valid}(v) = \text{verify}(v) \land \text{is\_member}(v.\text{voter\_did}, v.\text{scope})
$$

---

### 12.9 Controller P2P-Protokoll

#### 12.9.1 Delegation-Propagation

```rust
pub enum ControllerMessage {
    /// Controller für Scope gesetzt
    ControllerSet {
        scope: ScopeId,
        controller_did: String,
        set_by: String,  // Previous controller or Governance
        signature: Signature,
        proof: ControllerProof,
    },

    /// Delegation erstellt
    DelegationCreated {
        delegation_id: DelegationId,
        from_did: String,
        to_did: String,
        scope: ScopeId,
        permissions: Permissions,
        ttl: Option<u64>,
        signature: Signature,
    },

    /// Delegation widerrufen
    DelegationRevoked {
        delegation_id: DelegationId,
        revoked_by: String,
        reason: Option<String>,
        signature: Signature,
    },

    /// Permission-Check-Result (für Audit)
    PermissionCheckResult {
        actor_did: String,
        scope: ScopeId,
        action: String,
        result: bool,
        delegation_chain: Vec<DelegationId>,
    },
}
```

#### 12.9.2 Controller-Sync bei Realm-Join

```
┌────────────────────────────────────────────────────────────────────────────────────┐
│                    CONTROLLER SYNC ON REALM JOIN                                  │
├────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                    │
│   1. Peer joins Realm                                                             │
│   ┌──────────┐  GetControllerHierarchy(realm_id)  ┌──────────┐                   │
│   │ New Peer │ ─────────────────────────────────► │ Existing │                   │
│   └──────────┘                                    │   Peer   │                   │
│                                                   └──────────┘                   │
│       ◄───────────────────────────────────────────────│                          │
│         ControllerHierarchy {                         │                          │
│           realm_controller,                           │                          │
│           rooms: [{ room_id, controller }],           │                          │
│           delegations: [...]                          │                          │
│         }                                             │                          │
│                                                                                    │
│   2. Subscribe to Controller-Topic                                                │
│   ┌──────────┐  Subscribe(/realm/{id}/controller/v1)                              │
│   │ New Peer │ ─────────────────────────────────────────────────────►            │
│   └──────────┘                                                                    │
│                                                                                    │
│   3. Receive live updates                                                         │
│                ◄──── DelegationCreated, DelegationRevoked, ...                   │
│                                                                                    │
└────────────────────────────────────────────────────────────────────────────────────┘
```

---

### 12.10 API-Registry P2P-Protokoll

#### 12.10.1 API-Endpoint-Announcement

Wenn ein Peer API-Endpoints via ECL definiert:

```rust
pub struct APIRegistryMessage {
    /// Scope (Realm oder Room)
    pub scope: ScopeId,

    /// Message-Typ
    pub message_type: APIRegistryMessageType,
}

pub enum APIRegistryMessageType {
    /// Endpoint registriert
    EndpointRegistered {
        endpoint_id: EndpointId,
        method: HttpMethod,
        path: String,
        auth_required: bool,
        trust_gate: Option<f32>,
        rate_limit: Option<RateLimit>,
        schema_ref: Option<String>,
        signature: Signature,
    },

    /// Endpoint deregistriert
    EndpointDeregistered {
        endpoint_id: EndpointId,
        reason: Option<String>,
        signature: Signature,
    },

    /// OpenAPI-Spec aktualisiert
    OpenAPIUpdated {
        scope: ScopeId,
        version: String,
        spec_hash: String,
        signature: Signature,
    },
}
```

#### 12.10.2 API-Discovery via DHT

API-Endpoints werden auch im **Kademlia DHT** publiziert:

```
Key: /erynoa/api/{scope_id}/{endpoint_id}
Value: EndpointInfo { method, path, provider_peer_id, trust_gate }
```

---

### 12.11 Sicherheits-Garantien

#### 12.11.1 Nachrichten-Authentizität

Alle P2P-Nachrichten sind **signiert**:

$$
\mu.\text{sig} = \text{sign}(\text{sk}_{\text{sender}}, \text{hash}(\mu.\text{payload}))
$$

**Verification:**

$$
\text{verify}(\mu) = \text{ed25519\_verify}(\mu.\text{sig}, \mu.\text{sender\_pk}, \text{hash}(\mu.\text{payload}))
$$

#### 12.11.2 Replay-Attack-Prevention

Nachrichten enthalten **Nonce + Timestamp**:

$$
\text{fresh}(\mu) = (t_{\text{now}} - \mu.\text{timestamp}) < T_{\text{max}} \land \mu.\text{nonce} \notin \text{seen}
$$

Mit $T_{\text{max}} = 60\text{s}$ und Nonce-Cache für $2 \cdot T_{\text{max}}$.

#### 12.11.3 Sybil-Resistenz via Trust

**Trust-gated Operations** verhindern Sybil-Attacken:

| Operation          | Min Trust-R | Zusätzliche Checks        |
| ------------------ | ----------- | ------------------------- |
| Send Events        | 0.4         | -                         |
| Create Proposal    | 0.5         | Realm-Membership          |
| Cast Vote          | 0.3         | Realm-Membership          |
| Create Delegation  | 0.6         | Is Controller or Delegate |
| Relay Messages     | 0.7         | -                         |
| Announce Blueprint | 0.5         | -                         |
| Register API       | 0.5         | Is Controller or Delegate |

#### 12.11.4 Privacy-Layer-Integration

Das Privacy-Layer (Onion-Routing) schützt:

- **Sender-Anonymität**: Multi-Hop-Relays
- **Receiver-Anonymität**: Rendezvous-Points
- **Content-Privacy**: End-to-End-Encryption

```
┌─────────┐    Onion     ┌─────────┐    Onion     ┌─────────┐    Onion     ┌─────────┐
│ Sender  │─────────────►│ Relay 1 │─────────────►│ Relay 2 │─────────────►│Receiver │
└─────────┘  Encrypted   └─────────┘  Encrypted   └─────────┘  Encrypted   └─────────┘
              Layer 3                  Layer 2                  Layer 1
```

---

### 12.12 P2P-Erweiterungen: Gap-Analyse

#### 12.12.1 Neue Dateien (`peer/p2p/`)

| Datei                     | Zeilen | Beschreibung                                 |
| ------------------------- | ------ | -------------------------------------------- |
| `topics_extended.rs`      | ~250   | Neue Topics (blueprints, ui, gov, ctrl, api) |
| `protocol_extended.rs`    | ~600   | Neue SyncRequest/Response Varianten          |
| `messages/blueprint.rs`   | ~200   | BlueprintAnnouncement, BlueprintFetch        |
| `messages/ui.rs`          | ~300   | UIStateDelta, UISync                         |
| `messages/governance.rs`  | ~400   | GovernanceMessage Varianten                  |
| `messages/controller.rs`  | ~300   | ControllerMessage Varianten                  |
| `messages/api.rs`         | ~200   | APIRegistryMessage                           |
| `sync/blueprint_sync.rs`  | ~400   | Blueprint-Sync-Logic                         |
| `sync/ui_sync.rs`         | ~350   | UI-State-CRDT-Sync                           |
| `sync/governance_sync.rs` | ~400   | Governance-Vote-Sync                         |
| `sync/controller_sync.rs` | ~300   | Controller-Delegation-Sync                   |
| `propagation.rs`          | ~500   | Trust-filtered Propagation                   |

**Gesamt: ~4.200 neue Zeilen im P2P-Modul**

#### 12.12.2 Modifikationen

| Datei           | Änderung                        |
| --------------- | ------------------------------- |
| `topics.rs`     | +TopicType Varianten            |
| `protocol.rs`   | +SyncRequest/Response Varianten |
| `behaviour.rs`  | +Neue Topic-Subscriptions       |
| `trust_gate.rs` | +Operation-spezifische Checks   |
| `swarm.rs`      | +Message-Handler für neue Typen |

#### 12.12.3 Proto-Erweiterungen (`proto/erynoa/v1/`)

```protobuf
// NEU: p2p_messages.proto

message BlueprintAnnouncement {
    string blueprint_id = 1;
    string version = 2;
    string creator_did = 3;
    bytes signature = 4;
    BlueprintMetadata metadata = 5;
    float trust_gate = 6;
}

message UIStateDelta {
    string room_id = 1;
    UIDeltaType delta_type = 2;
    repeated string affected_components = 3;
    repeated BindingUpdate binding_updates = 4;
    map<string, float> trust_visibility = 5;
    VectorClock vector_clock = 6;
    bytes signature = 7;
}

message GovernanceMessage {
    oneof message {
        ProposalCreated proposal_created = 1;
        VoteCast vote_cast = 2;
        ProposalExecuted proposal_executed = 3;
        VetoCast veto_cast = 4;
    }
}

message ControllerMessage {
    oneof message {
        ControllerSet controller_set = 1;
        DelegationCreated delegation_created = 2;
        DelegationRevoked delegation_revoked = 3;
    }
}

message APIRegistryMessage {
    string scope = 1;
    oneof message {
        EndpointRegistered endpoint_registered = 2;
        EndpointDeregistered endpoint_deregistered = 3;
        OpenAPIUpdated openapi_updated = 4;
    }
}
```

---

### 12.13 Zusammenfassung: P2P-Axiome

Die **P2P-Kommunikation** folgt diesen Axiomen:

$$
\boxed{
\begin{aligned}
\textbf{P1 (Authentizität):} \quad & \forall \mu : \text{verify}(\mu.\text{sig}, \mu.\text{sender}) \\
\textbf{P2 (Trust-Gating):} \quad & \forall \text{op} : \tau(\text{sender}) \geq \tau_{\min}(\text{op}) \\
\textbf{P3 (Eventual Consistency):} \quad & \lim_{t \to \infty} \mathcal{S}_i(t) = \mathcal{S}_j(t) \quad \forall i,j \in \mathcal{P} \\
\textbf{P4 (Realm-Scoping):} \quad & \text{topic}(\mu) \implies \text{membership}(\text{sender}, \text{realm}) \\
\textbf{P5 (Privacy):} \quad & \text{onion}(\mu) \implies \neg\text{observable}(\text{sender}, \text{receiver}) \\
\textbf{P6 (Replay-Resistance):} \quad & \text{nonce}(\mu) \notin \text{seen} \land \text{fresh}(\mu.\text{timestamp})
\end{aligned}
}
$$

---

│ └── runtime/
│ ├── mod.rs
│ ├── vm.rs # ECLVM Runtime (1416 Zeilen)
│ ├── gas.rs # Gas Metering
│ └── host.rs # HostInterface Trait
│
├── core/ # Business Logic Layer
│ ├── mod.rs
│ ├── state.rs # Unified State (~3000 Zeilen)
│ ├── state_integration.rs # Observer-based Integration
│ ├── state_coordination.rs # State Coordination
│ ├── event_engine.rs # Event Processing (Κ9-Κ12)
│ ├── trust_engine.rs # Trust Calculation (Κ2-Κ5)
│ ├── world_formula.rs # World Formula (Κ15)
│ ├── consensus.rs # Consensus (Κ18)
│ ├── surprisal.rs # Surprisal Calculation
│ └── engine.rs # Execution Context Wrapper
│
├── domain/unified/ # Domain Model
│ ├── mod.rs
│ ├── primitives.rs # UniversalId, TemporalCoord
│ ├── realm.rs # Realm, Rules (769 Zeilen)
│ ├── saga.rs # Intent, Saga, Goal (833 Zeilen)
│ ├── trust.rs # TrustVector6D
│ ├── event.rs # Event Types
│ ├── identity.rs # Identity Types
│ ├── cost.rs # Cost/Budget Types
│ ├── formula.rs # World Formula Types
│ └── schema.rs # Schema Types
│
├── peer/ # Peer Layer
│ ├── mod.rs
│ ├── gateway.rs # Gateway Guard (591 Zeilen)
│ ├── saga_composer.rs # Saga Composition (640 Zeilen)
│ ├── intent_parser.rs # Intent Parsing
│ └── p2p/ # P2P Networking
│
├── local/ # Storage Layer
│ ├── mod.rs
│ ├── blueprint_marketplace.rs # Blueprint Marketplace (1949 Zeilen)
│ ├── realm_storage.rs # Realm Storage
│ ├── event_store.rs # Event Storage
│ ├── trust_store.rs # Trust Storage
│ ├── identity_store.rs # Identity Storage
│ ├── content_store.rs # Content Storage
│ ├── kv_store.rs # Key-Value Store
│ └── archive.rs # Cold Storage
│
├── protection/ # Protection Layer
└── execution/ # Execution Context

````

### 2.2 Aktuelle Komponenten-Analyse

#### 2.2.1 ECLVM (`eclvm/`)

**Stärken:**

- ✅ Stack-basierte VM vollständig implementiert
- ✅ Gas-Metering funktioniert
- ✅ Policy-Ausführung via `ProgrammableGateway`
- ✅ Host Interface für Trust/Event-Zugriff
- ✅ Mana-Management implementiert

**Lücken für SOLL:**

- ❌ Keine UI-Rendering-Capabilities
- ❌ Keine DataLogic/Event-Handler-Engine
- ❌ Keine API-Endpoint-Definition
- ❌ Keine Governance-Integration
- ❌ Keine Controller-Delegation
- ❌ Blueprint-Deployment nur rudimentär

#### 2.2.2 Blueprint Marketplace (`local/blueprint_marketplace.rs`)

**IST-Zustand (1949 Zeilen):**

```rust
pub struct Blueprint {
    pub id: BlueprintId,
    pub name: String,
    pub version: SemVer,
    pub stores: Vec<BlueprintStore>,      // ✅ Vorhanden
    pub policies: Vec<BlueprintPolicy>,   // ✅ Vorhanden
    pub sagas: Vec<BlueprintSaga>,        // ✅ Vorhanden
    // --- FEHLT ---
    // pub structure: BlueprintStructure,
    // pub ui: BlueprintUI,
    // pub datalogic: BlueprintDataLogic,
    // pub api: BlueprintAPI,
    // pub governance: BlueprintGovernance,
    // pub controller: BlueprintController,

}
````

**SOLL-Erweiterungen:**

- `BlueprintStructure` (Räume, Partitionen)
- `BlueprintUI` (Layouts, Pages, Components)
- `BlueprintDataLogic` (Handler, Aggregations)
- `BlueprintAPI` (Endpoints, Schemas)
- `BlueprintGovernance` (Voting, Proposals)

- `BlueprintController` (Permissions, Automation)

#### 2.2.3 Saga/Intent System (`domain/unified/saga.rs`, `peer/saga_composer.rs`)

**IST-Zustand:**

```rust
pub enum Goal {
    Transfer { to, amount, asset_type },
    Attest { subject, claim },
    Delegate { to, capabilities, trust_factor, ttl_seconds },
    Query { predicate },

    Create { entity_type, params },
    Complex { description, sub_goals },
}
```

**SOLL-Erweiterungen:**

```rust
pub enum Goal {
    // ... bestehende ...

    // NEU: Realm/Raum-Management
    RealmModify { realm_id, modification },
    RoomCreate { realm_id, room_config },
    PartitionCreate { room_id, partition_config },

    // NEU: UI-Modifikation
    UIModify { scope, ui_delta },

    // NEU: Governance
    Governance { proposal },
    Vote { proposal_id, choice },

    // NEU: Cross-Realm
    CrossRealm { from_realm, to_realm, action },

    // NEU: API
    APIRegister { scope, endpoint_config },

    // NEU: Blueprint
    BlueprintDeploy { blueprint_id, target_realm, config },

    BlueprintUpgrade { deployment_id, new_version },
}
```

#### 2.2.4 Realm System (`domain/unified/realm.rs`)

**IST-Zustand:**

- `RealmId` als `UniversalId`
- `Rule`, `RealmRules` für Regelsets
- `RootRealm`, `VirtualRealm` Basis-Strukturen
- Keine Room/Partition-Hierarchie

**SOLL-Erweiterungen:**

- `Room` als Untereinheit von `VirtualRealm`
- `Partition` als Untereinheit von `Room`
- `StoreSchema` pro Partition
- Controller-Zuordnung pro Scope

#### 2.2.5 Gateway (`peer/gateway.rs`, `eclvm/programmable_gateway.rs`)

**IST-Zustand:**

- Basic Trust-Prüfung
- Credential-Prüfung
- Policy-Ausführung

**SOLL-Erweiterungen:**

- UI-Rendering-Kontext
- API-Request-Routing
- Governance-Action-Validation

### 2.3 Bestehende Abhängigkeiten

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        IST-ZUSTAND ABHÄNGIGKEITEN                              │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   eclvm/runtime/vm.rs                                                          │
│         │                                                                       │
│         ├──► eclvm/runtime/host.rs (HostInterface)                             │
│         ├──► eclvm/runtime/gas.rs (GasMeter)                                   │
│         └──► eclvm/bytecode.rs (OpCode, Value)                                 │
│                                                                                 │
│   eclvm/programmable_gateway.rs                                                │
│         │                                                                       │
│         ├──► eclvm/runtime/vm.rs (ECLVM)                                       │
│         ├──► eclvm/mana.rs (ManaManager)                                       │
│         └──► domain/unified/* (RealmId, TrustVector6D)                         │
│                                                                                 │
│   peer/gateway.rs                                                              │
│         │                                                                       │
│         ├──► domain/unified/realm.rs (RealmId, VirtualRealm)                   │
│         └──► domain/unified/trust.rs (TrustVector6D)                           │
│                                                                                 │
│   peer/saga_composer.rs                                                        │
│         │                                                                       │
│         └──► domain/unified/saga.rs (Intent, Goal, Saga)                       │
│                                                                                 │
│   local/blueprint_marketplace.rs                                               │
│         │                                                                       │
│         ├──► local/realm_storage.rs (StoreSchema, StoreValue)                  │
│         └──► domain/unified/* (für Serialisierung)                             │
│                                                                                 │
│   core/state.rs                                                                │
│         │                                                                       │
│         └──► ALL modules (zentrale State-Verwaltung)                           │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. SOLL-Zustand Definition

### 3.1 Neue Verzeichnisstruktur

```
backend/src/
├── eclvm/
│   ├── mod.rs                      # Erweiterte Re-exports
│   ├── ast.rs                      # Unverändert
│   ├── bytecode.rs                 # + Neue OpCodes für UI/API/Gov
│   ├── compiler.rs                 # + Compile-Targets für Engines
│   ├── parser.rs                   # + Erweiterte ECL-Syntax
│   ├── optimizer.rs                # Unverändert
│   ├── bridge.rs                   # + Engine-Bridges
│   ├── erynoa_host.rs              # + Erweiterte Host-Functions
│   ├── mana.rs                     # Unverändert
│   ├── stdlib.rs                   # + UI/API/Gov Functions
│   ├── programmable_gateway.rs     # + Engine-Integration
│   ├── cli.rs                      # Unverändert
│   │
│   ├── runtime/
│   │   ├── mod.rs
│   │   ├── vm.rs                   # + Engine-Hooks
│   │   ├── gas.rs                  # + Neue Gas-Costs
│   │   └── host.rs                 # + Erweiterte HostInterface
│   │
│   │   # ═══════════════════════════════════════════════════════
│   │   # NEU: ENGINE-MODULE
│   │   # ═══════════════════════════════════════════════════════
│   │
│   ├── engines/
│   │   ├── mod.rs                  # Engine Re-exports
│   │   │
│   │   ├── ui_engine/              # UI-Engine (3.0.1) - gRPC-basiert für Realm-UI
│   │   │   ├── mod.rs
│   │   │   ├── renderer.rs         # Trust-basiertes Rendering
│   │   │   ├── components.rs       # Component-Registry
│   │   │   ├── bindings.rs         # Reactive Data Bindings
│   │   │   ├── layout.rs           # Layout-System
│   │   │   ├── delta.rs            # Hot-Reload/Delta-Updates
│   │   │   ├── grpc_bridge.rs      # gRPC ↔ Realm-UI Bridge
│   │   │   ├── session.rs          # UI-Session-Management
│   │   │   └── streaming.rs        # Bidirektionales Streaming
│   │   │
│   │   ├── datalogic_engine/       # DataLogic-Engine (3.0.2)
│   │   │   ├── mod.rs
│   │   │   ├── handlers.rs         # Event-Handler
│   │   │   ├── aggregations.rs     # Aggregation-State
│   │   │   ├── transforms.rs       # Data Transforms
│   │   │   └── outputs.rs          # Output-Emitter
│   │   │
│   │   ├── api_engine/             # API-Engine (3.0.4)
│   │   │   ├── mod.rs
│   │   │   ├── endpoints.rs        # Endpoint-Registry
│   │   │   ├── router.rs           # Request-Router
│   │   │   ├── auth.rs             # Authentication
│   │   │   ├── rate_limit.rs       # Rate-Limiting
│   │   │   ├── schema.rs           # JSON Schema Validation
│   │   │   └── openapi.rs          # OpenAPI Generation
│   │   │
│   │   ├── governance_engine/      # Governance-Engine (3.0.5)
│   │   │   ├── mod.rs
│   │   │   ├── modes.rs            # GovernanceMode Implementations
│   │   │   ├── proposals.rs        # Proposal-Management
│   │   │   ├── voting.rs           # Voting-Logic
│   │   │   ├── timelock.rs         # Timelock-Queue
│   │   │   ├── delegation.rs       # Liquid Democracy
│   │   │   └── veto.rs             # Veto-Mechanism
│   │   │
│   │   ├── controller_engine/      # Controller-Engine (3.0.3 + 3.0.6)
│   │   │   ├── mod.rs
│   │   │   ├── permissions.rs      # Permission-Management
│   │   │   ├── delegation.rs       # Controller-Delegation
│   │   │   ├── automation.rs       # Automation-Rules
│   │   │   ├── audit.rs            # Audit-Logging
│   │   │   └── governance_binding.rs  # DAO-Integration
│   │   │
│   │   └── blueprint_engine/       # Blueprint-Engine (3.0.9)
│   │       ├── mod.rs
│   │       ├── deployer.rs         # Blueprint-Deployment
│   │       ├── upgrader.rs         # Version-Upgrades
│   │       ├── composer.rs         # Blueprint-Composition
│   │       ├── migrator.rs         # Migration-Planning
│   │       └── validator.rs        # Blueprint-Validation
│   │
│   └── types/                      # Shared Types für Engines
│       ├── mod.rs
│       ├── ui_types.rs             # UI-spezifische Types
│       ├── api_types.rs            # API-spezifische Types
│       ├── governance_types.rs     # Governance-spezifische Types
│       └── controller_types.rs     # Controller-spezifische Types
│
├── domain/unified/
│   ├── mod.rs                      # + Neue Exports
│   ├── primitives.rs               # Unverändert
│   ├── realm.rs                    # + Room, Partition, Controller
│   ├── saga.rs                     # + Neue Goal-Typen
│   ├── trust.rs                    # Unverändert
│   ├── event.rs                    # + UI/API/Gov Events
│   ├── identity.rs                 # Unverändert
│   ├── cost.rs                     # + Neue Kostentypen
│   ├── formula.rs                  # Unverändert
│   ├── schema.rs                   # + UI/API Schemas
│   │
│   │   # NEU: Engine-Domain-Typen
│   │
│   ├── ui.rs                       # UI-Domain-Types
│   ├── api.rs                      # API-Domain-Types
│   ├── governance.rs               # Governance-Domain-Types
│   └── controller.rs               # Controller-Domain-Types
│
├── local/
│   ├── mod.rs                      # + Neue Exports
│   ├── blueprint_marketplace.rs    # MAJOR REFACTOR (siehe 6.4)
│   ├── realm_storage.rs            # + Room/Partition-Storage
│   ├── ui_store.rs                 # NEU: UI-State-Storage
│   ├── api_store.rs                # NEU: API-Registry-Storage
│   ├── governance_store.rs         # NEU: Proposal/Vote-Storage
│   └── ... (bestehende)
│
├── peer/
│   ├── mod.rs                      # + Neue Exports
│   ├── gateway.rs                  # + Engine-Integration
│   ├── saga_composer.rs            # + Neue Goal-Composer
│   ├── intent_parser.rs            # + Erweiterte Intent-Typen
│   │
│   │   # NEU: Engine-Peer-Integration
│   │
│   ├── ui_renderer.rs              # Peer-spezifisches Rendering
│   ├── api_handler.rs              # API-Request-Handler
│   └── governance_executor.rs      # Governance-Action-Executor
│
└── core/
    ├── mod.rs                      # + Neue Exports
    ├── state.rs                    # + Engine-States
    ├── state_integration.rs        # + Engine-Observers
    │
    │   # NEU: Engine-State-Management
    │
    ├── ui_state.rs                 # UI-State-Integration
    ├── api_state.rs                # API-State-Integration
    └── governance_state.rs         # Governance-State-Integration
```

### 3.2 Neue Typen und Strukturen

#### 3.2.1 UI-Engine Types (gRPC-basiert für Realm-UI)

```rust
// eclvm/types/ui_types.rs

/// Kompilierte UI-Definition (wird an Realm-UI übertragen)
pub struct CompiledUI {
    pub bytecode: Vec<OpCode>,
    pub layout: LayoutDefinition,
    pub components: Vec<CompiledComponent>,
    pub trust_gates: HashMap<String, f32>,
    pub credential_gates: HashMap<String, Vec<String>>,
}

/// UI-Komponente
pub struct CompiledComponent {
    pub id: String,
    pub component_type: ComponentType,
    pub bytecode: Vec<OpCode>,
    pub bindings: Vec<CompiledBinding>,
    pub trust_gate: Option<f32>,
    pub credential_gate: Vec<String>,
}

/// Reaktives Binding
pub struct CompiledBinding {
    pub source_expression: Vec<OpCode>,
    pub target_path: String,
    pub transform: Option<Vec<OpCode>>,
    pub update_trigger: UpdateTrigger,
}

/// Gerenderte UI (für Peer) - wird via gRPC an Realm-UI gesendet
pub struct RenderedUI {
    pub layout: LayoutDefinition,
    pub components: Vec<RenderedComponent>,
    pub peer_trust: f32,
}

// ============================================================================
// gRPC Types für Realm-UI Kommunikation
// ============================================================================

/// UI-State-Update (Backend → Realm-UI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIStateUpdate {
    pub room_id: RoomId,
    pub update_type: UIUpdateType,
    pub payload: UIPayload,
    pub sequence_number: u64,
    pub timestamp: TemporalCoord,
}

/// Update-Typen für Realm-UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIUpdateType {
    /// Vollständiges UI-Layout (Initial-Load)
    FullLayout,
    /// Inkrementelles Delta-Update
    DeltaUpdate,
    /// Binding-Wert-Update
    BindingUpdate,
    /// Komponenten-Sichtbarkeit geändert (Trust-Gate)
    VisibilityChange,
    /// Navigation/Routing
    NavigationChange,
    /// Theme-Änderung
    ThemeUpdate,
    /// Error/Notification
    Notification,
}

/// Payload für UI-Updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIPayload {
    Layout(RenderedUI),
    Delta(UIDelta),
    Binding { path: String, value: serde_json::Value },
    Visibility { component_ids: Vec<String>, visible: bool },
    Navigation { route: String, params: HashMap<String, String> },
    Theme(ThemeDefinition),
    Notification { level: NotificationLevel, message: String, action: Option<UIAction> },
}

/// UI-Event von Realm-UI (Realm-UI → Backend)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIEvent {
    pub room_id: RoomId,
    pub event_type: UIEventType,
    pub source_component: String,
    pub payload: serde_json::Value,
    pub peer_did: UniversalId,
    pub timestamp: TemporalCoord,
}

/// Event-Typen von Realm-UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIEventType {
    /// User-Interaktion (Click, Input, etc.)
    Interaction,
    /// Form-Submit
    FormSubmit,
    /// Navigation-Request
    NavigationRequest,
    /// Binding-Change (User-initiated)
    BindingChange,
    /// Lifecycle (Mount, Unmount, Focus, Blur)
    Lifecycle,
    /// Error in Realm-UI
    ClientError,
}

/// UI-Command (Backend → Realm-UI, direkter Befehl)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UICommand {
    pub command_type: UICommandType,
    pub target: Option<String>,  // Component-ID oder Route
    pub params: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UICommandType {
    Navigate,
    Focus,
    Scroll,
    OpenModal,
    CloseModal,
    ShowToast,
    TriggerAnimation,
    RefreshComponent,
    InvalidateCache,
}
```

#### 3.2.2 API-Engine Types

```rust
// eclvm/types/api_types.rs

/// Kompilierter API-Endpoint
pub struct CompiledEndpoint {
    pub path: String,
    pub method: HttpMethod,
    pub handler: Vec<OpCode>,
    pub request_schema: Option<JSONSchema>,
    pub response_schema: Option<JSONSchema>,
    pub auth: EndpointAuth,
    pub rate_limit: RateLimitConfig,
    pub trust_gate: Option<f32>,
    pub credential_gate: Vec<String>,
    pub caching: CacheConfig,
}

/// Authentifizierungsmodi
pub enum EndpointAuth {
    Public,
    APIKey { scopes: Vec<String> },
    PeerAuth { required_trust: f32, required_credentials: Vec<String> },
    WebhookSignature { algorithm: SignatureAlgorithm, header_name: String },
    OAuth2 { provider: String, scopes: Vec<String> },
}

/// API-Request
pub struct APIRequest {
    pub path: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: serde_json::Value,
}

/// API-Response
pub struct APIResponse {
    pub status: u16,
    pub body: serde_json::Value,
    pub headers: HashMap<String, String>,
}
```

#### 3.2.3 Governance-Engine Types

```rust
// eclvm/types/governance_types.rs

/// Governance-Modus
pub enum GovernanceMode {
    SingleController { controller: UniversalId },
    MultiSig { signers: Vec<UniversalId>, threshold: u32 },
    DAO { voting_power: VotingPowerSource },
    Optimistic { challenge_period: Duration, veto_threshold: f32 },
    Futarchy { market_duration: Duration, resolution_source: ResolutionSource },
    ConvictionVoting { decay_rate: f32, max_conviction: f32 },
    LiquidDemocracy { max_delegation_depth: u32 },
}

/// Abstimmungskraft-Quelle
pub enum VotingPowerSource {
    EqualVoting,
    TrustBased { dimension_weights: [f32; 6] },
    ReputationBased { reputation_metric: String },
    ActivityBased { lookback_period: Duration, activity_weights: ActivityWeights },
    Quadratic,
    Custom { calculator: Vec<OpCode> },
}

/// Proposal
pub struct Proposal {
    pub id: ProposalId,
    pub proposer: UniversalId,
    pub title: String,
    pub description: String,
    pub actions: Vec<ProposalAction>,
    pub created_at: TemporalCoord,
    pub voting_starts: TemporalCoord,
    pub voting_ends: TemporalCoord,
    pub state: ProposalState,
    pub votes: HashMap<UniversalId, Vote>,
    pub execution_hash: Hash,
}

/// Proposal-Aktionen
pub enum ProposalAction {
    ModifyPolicy { scope: ScopeId, policy_delta: ECLPolicyDelta },
    ModifyUI { room_id: RoomId, ui_delta: UIDelta },
    ModifyAPI { endpoint_changes: Vec<APIChange> },
    ModifyStructure { structure_delta: StructureDelta },
    ModifyController { new_controller: ControllerConfig },
    ModifyGovernance { governance_delta: GovernanceDelta },
    AllocateResources { allocations: Vec<ResourceAllocation> },
    ExecuteECL { code: Vec<OpCode> },
}
```

#### 3.2.4 Controller-Engine Types

```rust
// eclvm/types/controller_types.rs

/// Controller-Konfiguration
pub struct ControllerConfig {
    pub primary_controller: UniversalId,
    pub permissions: ControllerPermissions,
    pub delegates: Vec<Delegation>,
    pub governance_override: bool,
}

/// Delegation
pub struct Delegation {
    pub delegate: UniversalId,
    pub scope: ControlScope,
    pub permissions: ControllerPermissions,
    pub trust_factor: f32,
    pub expires_at: Option<TemporalCoord>,
}

/// Automatisierungsregel
pub struct AutomationRule {
    pub id: String,
    pub trigger: AutomationTrigger,
    pub condition: Option<Vec<OpCode>>,
    pub action: AutomationAction,
    pub requires_governance: bool,
}

/// Automation-Trigger
pub enum AutomationTrigger {
    Schedule { cron: String },
    OnEvent { event_pattern: String },
    Threshold { metric: String, operator: ThresholdOp, value: f32 },
    Webhook { endpoint_id: EndpointId },
}
```

### 3.3 Erweiterte Blueprint-Struktur

```rust
// local/blueprint_marketplace.rs (refactored)

/// Erweitertes Blueprint mit allen ECL-Komponenten
pub struct ExtendedBlueprint {
    // Identifikation
    pub id: BlueprintId,
    pub version: SemVer,
    pub content_hash: Hash,

    // Metadaten
    pub name: String,
    pub description: String,
    pub creator_did: UniversalId,
    pub created_at: TemporalCoord,
    pub tags: Vec<String>,
    pub category: BlueprintCategory,
    pub license: BlueprintLicense,

    // ECL-Komponenten (NEU)
    pub structure: BlueprintStructure,
    pub policy: BlueprintPolicy,
    pub ui: BlueprintUI,
    pub datalogic: BlueprintDataLogic,
    pub api: BlueprintAPI,
    pub governance: BlueprintGovernance,
    pub controller: BlueprintController,

    // Versionierung
    pub predecessor: Option<BlueprintId>,
    pub forked_from: Option<BlueprintId>,
    pub dependencies: Vec<BlueprintDependency>,

    // Metriken
    pub complexity: u64,
    pub novelty_score: f64,
    pub diversity_contribution: f64,
    pub omega_contribution: f64,
}
```

---

## 4. Abhängigkeits-Graph

### 4.1 Engine-Abhängigkeiten

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        SOLL-ZUSTAND ABHÄNGIGKEITEN                             │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   ┌─────────────────────────────────────────────────────────────────────────┐  │
│   │                         REALM-UI (Frontend)                             │  │
│   │              React/Vue/Svelte - Universal UI Renderer                   │  │
│   └───────────────────────────────────┬─────────────────────────────────────┘  │
│                                       │                                        │
│                                       │ gRPC (bidirektional)                   │
│                                       │ - UIStateUpdate (Backend → Realm-UI)   │
│                                       │ - UIEvent (Realm-UI → Backend)         │
│                                       │                                        │
│   ┌───────────────────────────────────▼─────────────────────────────────────┐  │
│   │                      RealmUIBridge (gRPC Layer)                         │  │
│   │              Session-Management, Streaming, Auth                        │  │
│   └───────────────────────────────────┬─────────────────────────────────────┘  │
│                                       │                                        │
│                           ┌───────────┴───────────┐                            │
│                           │   BlueprintEngine     │                            │
│                           │   (Koordinator)       │                            │
│                           └─────────┬─────────────┘                            │
│                                     │                                          │
│           ┌─────────────────────────┼─────────────────────────┐                │
│           │                         │                         │                │
│           ▼                         ▼                         ▼                │
│   ┌───────────────┐         ┌───────────────┐         ┌───────────────┐       │
│   │  UIEngine     │         │  APIEngine    │         │GovernanceEng. │       │
│   │  (gRPC-based) │         └───────┬───────┘         └───────┬───────┘       │
│   └───────┬───────┘                 │                         │                │
│           │                         │                         │                │
│           │      ┌──────────────────┼──────────────────┐      │                │
│           │      │                  │                  │      │                │
│           ▼      ▼                  ▼                  ▼      ▼                │
│   ┌─────────────────────────────────────────────────────────────────┐         │
│   │                      DataLogicEngine                            │         │
│   │              (Event-Handler, Aggregationen)                     │         │
│   └───────────────────────────────┬─────────────────────────────────┘         │
│                                   │                                            │
│                                   ▼                                            │
│   ┌─────────────────────────────────────────────────────────────────┐         │
│   │                     ControllerEngine                            │         │
│   │              (Permissions, Delegation, Audit)                   │         │
│   └───────────────────────────────┬─────────────────────────────────┘         │
│                                   │                                            │
│       ┌───────────────────────────┼───────────────────────────┐               │
│       │                           │                           │               │
│       ▼                           ▼                           ▼               │
│   ┌─────────┐               ┌─────────┐               ┌─────────┐            │
│   │ ECLVM   │               │PolicyEng│               │TrustEng │            │
│   │ Runtime │◄──────────────│(besteh.)│◄──────────────│(besteh.)│            │
│   └────┬────┘               └─────────┘               └─────────┘            │
│        │                                                                       │
│        ▼                                                                       │
│   ┌─────────────────────────────────────────────────────────────────┐         │
│   │                        Storage Layer                            │         │
│   │   (RealmStorage, UIStore, APIStore, GovernanceStore)           │         │
│   └─────────────────────────────────────────────────────────────────┘         │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Realm-UI Kommunikations-Architektur

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                    REALM-UI ↔ BACKEND KOMMUNIKATION                            │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   REALM-UI (Frontend)                      ERYNOA BACKEND                      │
│   ══════════════════                       ══════════════                       │
│                                                                                 │
│   ┌──────────────────┐                     ┌──────────────────┐                │
│   │  Component Tree  │                     │    UI-Engine     │                │
│   │  ┌────────────┐  │                     │  ┌────────────┐  │                │
│   │  │  Renderer  │  │                     │  │  Compiler  │  │                │
│   │  └─────┬──────┘  │                     │  └─────┬──────┘  │                │
│   │        │         │                     │        │         │                │
│   │  ┌─────▼──────┐  │                     │  ┌─────▼──────┐  │                │
│   │  │  Binding   │  │◄────────────────────│  │  Binding   │  │                │
│   │  │  Manager   │  │   UIStateUpdate     │  │  Manager   │  │                │
│   │  └─────┬──────┘  │   (gRPC Stream)     │  └─────┬──────┘  │                │
│   │        │         │                     │        │         │                │
│   │  ┌─────▼──────┐  │                     │  ┌─────▼──────┐  │                │
│   │  │   Event    │──┼────────────────────►│  │   Event    │  │                │
│   │  │  Emitter   │  │      UIEvent        │  │  Handler   │  │                │
│   │  └────────────┘  │   (gRPC Stream)     │  └────────────┘  │                │
│   └──────────────────┘                     └──────────────────┘                │
│                                                                                 │
│   ═══════════════════════════════════════════════════════════════════════════  │
│                              gRPC PROTOCOL FLOW                                │
│   ═══════════════════════════════════════════════════════════════════════════  │
│                                                                                 │
│   1. CONNECT (Realm-UI → Backend)                                              │
│      ┌─────────────────────────────────────────────────────────────────────┐   │
│      │  InitialUIRequest {                                                 │   │
│      │    room_id: "realm:community/chat",                                 │   │
│      │    peer_did: "did:erynoa:abc123...",                                │   │
│      │    trust_vector: { r: 0.8, i: 0.7, c: 0.9, p: 0.6, v: 0.8, ω: 0.76 }│   │
│      │    credentials: ["member", "moderator"]                             │   │
│      │  }                                                                  │   │
│      └─────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
│   2. INITIAL UI (Backend → Realm-UI)                                           │
│      ┌─────────────────────────────────────────────────────────────────────┐   │
│      │  RenderedUI {                                                       │   │
│      │    layout: { type: "flex", direction: "column" },                   │   │
│      │    components: [                                                    │   │
│      │      { id: "header", type: "Header", visible: true, ... },          │   │
│      │      { id: "chat", type: "ChatWindow", visible: true, ... },        │   │
│      │      { id: "admin-panel", type: "AdminPanel", visible: true },      │   │
│      │    ],                                                               │   │
│      │    initial_bindings: { "messages": [...], "users": [...] }          │   │
│      │  }                                                                  │   │
│      └─────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
│   3. LIVE UPDATES (Bidirektionaler Stream)                                     │
│                                                                                 │
│      Backend → Realm-UI:                                                       │
│      ┌─────────────────────────────────────────────────────────────────────┐   │
│      │  UIStateUpdate { type: BINDING, binding: { path: "messages",       │   │
│      │                  value: [...new_messages] }, seq: 42 }              │   │
│      └─────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
│      Realm-UI → Backend:                                                       │
│      ┌─────────────────────────────────────────────────────────────────────┐   │
│      │  UIEvent { type: FORM_SUBMIT, source: "message-input",              │   │
│      │            payload: { text: "Hello World!" } }                      │   │
│      └─────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
│   4. TRUST-GATE ÄNDERUNG (Backend → Realm-UI)                                  │
│      ┌─────────────────────────────────────────────────────────────────────┐   │
│      │  UIStateUpdate { type: VISIBILITY,                                  │   │
│      │                  visibility: { component_ids: ["admin-panel"],      │   │
│      │                               visible: false,                       │   │
│      │                               reason: "trust_gate" } }              │   │
│      └─────────────────────────────────────────────────────────────────────┘   │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 4.3 Initialisierungs-Reihenfolge

```
1. Storage Layer (bereits vorhanden)
   └──► RealmStorage, TrustStore, EventStore

2. Core Engines (bereits vorhanden)
   └──► TrustEngine, EventEngine, WorldFormulaEngine

3. ECLVM Runtime (bereits vorhanden)
   └──► ECLVM, GasMeter, HostInterface

4. Controller-Engine (NEU - Basis für alle anderen)
   └──► Permissions, Delegation-Graph, Audit

5. DataLogic-Engine (NEU - Event-Verarbeitung)
   └──► Handler-Registry, Aggregation-State

6. UI-Engine mit Realm-UI Bridge (NEU - gRPC-basiert)
   └──► Component-Registry, Binding-Manager
   └──► RealmUIBridge (gRPC-Server starten)
   └──► Session-Management initialisieren

7. API-Engine (NEU - hängt von DataLogic ab)
   └──► Endpoint-Registry, Rate-Limiter

8. Governance-Engine (NEU - hängt von Controller ab)
   └──► Proposal-Store, Voting-Logic

9. Blueprint-Engine (NEU - orchestriert alle Engines)
   └──► Deployer, Composer, Migrator

10. Realm-UI Connection (Frontend startet)
    └──► gRPC Connect → Session erstellen
    └──► Initial UI laden → Render
    └──► Bidirektionaler Stream aktiv
```

### 4.4 Realm-UI Integration Checklist

| Komponente    | Backend               | Realm-UI           | Protokoll   |
| ------------- | --------------------- | ------------------ | ----------- |
| Initial Load  | `GetInitialUI()`      | `onMount()`        | gRPC Unary  |
| Live Bindings | `push_state_update()` | `bindingStore`     | gRPC Stream |
| User Events   | `handle_ui_event()`   | `dispatch()`       | gRPC Stream |
| Navigation    | `Navigate()`          | `router.push()`    | gRPC Unary  |
| Trust-Gates   | Automatisch           | Re-render          | gRPC Stream |
| Theme         | `ThemeUpdate`         | CSS Vars           | gRPC Stream |
| Commands      | `send_command()`      | `executeCommand()` | gRPC Stream |

---

## 5. Refactoring-Phasen

### Phase 1: Grundlagen & Domain-Erweiterungen (2-3 Wochen)

#### 5.1.1 Domain-Types erweitern

| Datei                          | Änderungen                               |
| ------------------------------ | ---------------------------------------- |
| `domain/unified/mod.rs`        | + Neue Modul-Exports                     |
| `domain/unified/realm.rs`      | + `Room`, `Partition`, `ControllerScope` |
| `domain/unified/saga.rs`       | + Neue `Goal`-Typen                      |
| `domain/unified/ui.rs`         | NEU: UI-Domain-Types                     |
| `domain/unified/api.rs`        | NEU: API-Domain-Types                    |
| `domain/unified/governance.rs` | NEU: Governance-Domain-Types             |
| `domain/unified/controller.rs` | NEU: Controller-Domain-Types             |

#### 5.1.2 Bytecode erweitern

```rust
// eclvm/bytecode.rs - Neue OpCodes

pub enum OpCode {
    // ... bestehende ...

    // NEU: UI Operations
    UIRender,           // Rendere UI für aktuellen Peer
    UIBindGet,          // Hole gebundenen Wert
    UIBindSet,          // Setze gebundenen Wert
    UIComponentVisible, // Prüfe Komponenten-Sichtbarkeit

    // NEU: API Operations
    APIRespond,         // Sende API-Response
    APIValidateSchema,  // Validiere gegen Schema
    APIRateCheck,       // Prüfe Rate-Limit

    // NEU: Governance Operations
    GovPropose,         // Erstelle Proposal
    GovVote,            // Stimme ab
    GovExecute,         // Führe angenommenes Proposal aus
    GovVetoPower,       // Prüfe Veto-Berechtigung

    // NEU: Controller Operations
    CtrlValidate,       // Validiere Controller-Aktion
    CtrlDelegate,       // Delegiere Kontrolle
    CtrlRevoke,         // Widerrufe Delegation
}
```

### Phase 2: Controller-Engine (3-4 Wochen)

**Warum zuerst?** Controller-Engine ist Voraussetzung für alle anderen Engines, da Berechtigungen überall gebraucht werden.

#### 5.2.1 Dateien erstellen

```
eclvm/engines/controller_engine/
├── mod.rs              # ~100 Zeilen
├── permissions.rs      # ~400 Zeilen
├── delegation.rs       # ~500 Zeilen
├── automation.rs       # ~300 Zeilen
├── audit.rs            # ~200 Zeilen
└── governance_binding.rs # ~300 Zeilen
```

#### 5.2.2 Haupt-Implementierung

```rust
// eclvm/engines/controller_engine/mod.rs

pub struct ControllerEngine {
    controllers: HashMap<ScopeId, ControllerConfig>,
    delegations: DelegationGraph,
    automation_rules: HashMap<String, AutomationRule>,
    audit: AuditLog,
}

impl ControllerEngine {
    pub fn validate_action(&self, scope: &ScopeId, actor: &UniversalId, action: &ControllerAction) -> Result<ValidationResult>;
    pub fn delegate(&mut self, scope: &ScopeId, from: &UniversalId, to: &UniversalId, delegation: Delegation) -> Result<()>;
    pub fn revoke(&mut self, delegation_id: &DelegationId) -> Result<()>;
    pub fn execute_automation(&mut self, rule_id: &str, context: &AutomationContext) -> Result<()>;
}
```

### Phase 3: DataLogic-Engine (2-3 Wochen)

#### 5.3.1 Dateien erstellen

```
eclvm/engines/datalogic_engine/
├── mod.rs              # ~100 Zeilen
├── handlers.rs         # ~500 Zeilen
├── aggregations.rs     # ~400 Zeilen
├── transforms.rs       # ~300 Zeilen
└── outputs.rs          # ~200 Zeilen
```

#### 5.3.2 Haupt-Implementierung

```rust
// eclvm/engines/datalogic_engine/mod.rs

pub struct DataLogicEngine {
    handlers: HashMap<RoomId, Vec<CompiledEventHandler>>,
    aggregations: HashMap<String, AggregationState>,
    outputs: OutputEmitter,
}

impl DataLogicEngine {
    pub fn register_handler(&mut self, room_id: &RoomId, handler: CompiledEventHandler) -> Result<()>;
    pub fn process_event(&mut self, room_id: &RoomId, event: &Event, vm: &mut ECLVM) -> Result<Vec<DataLogicOutput>>;
    pub fn get_aggregation(&self, aggregation_id: &str) -> Option<&AggregationState>;
}
```

### Phase 4: UI-Engine mit Realm-UI gRPC-Integration (4-5 Wochen)

#### 5.4.1 Dateien erstellen

```
eclvm/engines/ui_engine/
├── mod.rs              # ~150 Zeilen
├── renderer.rs         # ~600 Zeilen
├── components.rs       # ~400 Zeilen
├── bindings.rs         # ~500 Zeilen
├── layout.rs           # ~300 Zeilen
├── delta.rs            # ~300 Zeilen
├── grpc_bridge.rs      # ~800 Zeilen  (NEU: Realm-UI Kommunikation)
├── session.rs          # ~400 Zeilen  (NEU: Session-Management)
└── streaming.rs        # ~500 Zeilen  (NEU: Bidirektionales Streaming)
```

#### 5.4.2 Haupt-Implementierung (gRPC-basiert)

```rust
// eclvm/engines/ui_engine/mod.rs

/// UI-Engine mit gRPC-Bridge zu Realm-UI Frontend
pub struct UIEngine {
    compiled_uis: HashMap<RoomId, CompiledUI>,
    components: ComponentRegistry,
    bindings: BindingManager,
    /// gRPC-Bridge für Realm-UI Kommunikation
    grpc_bridge: RealmUIBridge,
    /// Aktive UI-Sessions pro Peer
    sessions: HashMap<UniversalId, UISession>,
}

impl UIEngine {
    pub fn compile_ui(&mut self, room_id: &RoomId, ecl_ui: &ECLUI) -> Result<()>;

    /// Rendert UI und sendet via gRPC an Realm-UI
    pub async fn render_for_peer(
        &self,
        room_id: &RoomId,
        peer: &DID,
        peer_trust: &TrustVector6D,
        peer_credentials: &[String]
    ) -> Result<RenderedUI>;

    pub fn apply_delta(&mut self, room_id: &RoomId, delta: &UIDelta, controller: &DID) -> Result<()>;

    // ═══════════════════════════════════════════════════════════════════════
    // NEU: Realm-UI gRPC-Methoden
    // ═══════════════════════════════════════════════════════════════════════

    /// Startet UI-Session für einen Peer (Realm-UI connected)
    pub async fn start_session(&mut self, peer: &UniversalId, room_id: &RoomId) -> Result<UISessionId>;

    /// Sendet State-Update an verbundenes Realm-UI
    pub async fn push_state_update(&self, session_id: &UISessionId, update: UIStateUpdate) -> Result<()>;

    /// Sendet direkten Command an Realm-UI
    pub async fn send_command(&self, session_id: &UISessionId, command: UICommand) -> Result<()>;

    /// Verarbeitet eingehendes Event von Realm-UI
    pub async fn handle_ui_event(&mut self, event: UIEvent) -> Result<Vec<UIStateUpdate>>;

    /// Broadcast Delta-Update an alle Sessions in einem Room
    pub async fn broadcast_delta(&self, room_id: &RoomId, delta: &UIDelta) -> Result<()>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// UI-Session (Verbindung zu einer Realm-UI Instanz)
// ═══════════════════════════════════════════════════════════════════════════════

pub type UISessionId = UniversalId;

/// Repräsentiert eine aktive Verbindung zu Realm-UI
pub struct UISession {
    pub id: UISessionId,
    pub peer_did: UniversalId,
    pub room_id: RoomId,
    pub peer_trust: TrustVector6D,
    pub peer_credentials: Vec<String>,
    /// Letzter bekannter UI-State (für Delta-Berechnung)
    pub last_rendered: Option<RenderedUI>,
    /// Sequence-Counter für geordnete Updates
    pub sequence: AtomicU64,
    /// gRPC-Stream-Handle
    pub stream_handle: StreamHandle,
    /// Session-Start
    pub started_at: TemporalCoord,
    /// Letzte Aktivität
    pub last_activity: TemporalCoord,
}
```

#### 5.4.3 gRPC-Bridge für Realm-UI

```rust
// eclvm/engines/ui_engine/grpc_bridge.rs

use tonic::{Request, Response, Status, Streaming};
use tokio_stream::wrappers::ReceiverStream;

/// Bridge zwischen UI-Engine und Realm-UI Frontend
pub struct RealmUIBridge {
    /// Aktive Streams zu Realm-UI Clients
    active_streams: HashMap<UISessionId, StreamSender>,
    /// Event-Handler für eingehende Events
    event_handler: Arc<dyn UIEventHandler>,
    /// Metrics
    metrics: BridgeMetrics,
}

impl RealmUIBridge {
    pub fn new(event_handler: Arc<dyn UIEventHandler>) -> Self;

    /// Registriert einen neuen Realm-UI Client
    pub async fn register_client(
        &mut self,
        peer_did: &UniversalId,
        room_id: &RoomId,
        auth_token: &str,
    ) -> Result<(UISessionId, ReceiverStream<UIStateUpdate>)>;

    /// Sendet Update an spezifischen Client
    pub async fn send_to_client(
        &self,
        session_id: &UISessionId,
        update: UIStateUpdate,
    ) -> Result<()>;

    /// Broadcast an alle Clients in einem Room
    pub async fn broadcast_to_room(
        &self,
        room_id: &RoomId,
        update: UIStateUpdate,
    ) -> Result<BroadcastStats>;

    /// Verarbeitet eingehenden Event-Stream von Realm-UI
    pub async fn process_event_stream(
        &self,
        session_id: &UISessionId,
        events: Streaming<UIEvent>,
    ) -> Result<()>;
}

/// gRPC Service Implementation für Realm-UI
#[tonic::async_trait]
impl RealmUIService for RealmUIBridgeService {
    type ConnectStream = ReceiverStream<Result<UIStateUpdate, Status>>;

    /// Bidirektionaler Stream: Realm-UI ↔ Backend
    async fn connect(
        &self,
        request: Request<Streaming<UIEvent>>,
    ) -> Result<Response<Self::ConnectStream>, Status> {
        let peer_did = extract_peer_did(&request)?;
        let room_id = extract_room_id(&request)?;

        // Session erstellen
        let (session_id, update_stream) = self.bridge
            .register_client(&peer_did, &room_id, "")
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // Event-Stream verarbeiten (im Hintergrund)
        let events = request.into_inner();
        let bridge = self.bridge.clone();
        tokio::spawn(async move {
            if let Err(e) = bridge.process_event_stream(&session_id, events).await {
                tracing::error!("Event stream error: {}", e);
            }
        });

        Ok(Response::new(update_stream))
    }

    /// Initialer UI-Load
    async fn get_initial_ui(
        &self,
        request: Request<InitialUIRequest>,
    ) -> Result<Response<RenderedUI>, Status> {
        let req = request.into_inner();
        let peer_did = UniversalId::from_hex(&req.peer_did)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let room_id = RoomId::from_hex(&req.room_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let ui = self.ui_engine
            .render_for_peer(&room_id, &peer_did, &req.trust_vector, &req.credentials)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ui))
    }

    /// Command von Realm-UI ausführen
    async fn execute_action(
        &self,
        request: Request<UIActionRequest>,
    ) -> Result<Response<UIActionResponse>, Status> {
        let req = request.into_inner();

        // Action verarbeiten und Response generieren
        let result = self.ui_engine
            .handle_ui_event(req.into())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UIActionResponse {
            success: true,
            updates: result,
        }))
    }
}
```

#### 5.4.4 gRPC Proto-Definition für Realm-UI

```protobuf
// proto/erynoa/realm_ui.proto

syntax = "proto3";

package erynoa.realm_ui;

import "google/protobuf/struct.proto";
import "google/protobuf/timestamp.proto";

// ============================================================================
// Realm-UI Service - Bidirektionale Kommunikation zwischen Backend und Realm-UI
// ============================================================================

service RealmUIService {
    // Bidirektionaler Stream für Live-Updates
    rpc Connect(stream UIEvent) returns (stream UIStateUpdate);

    // Initialer UI-Load für einen Room
    rpc GetInitialUI(InitialUIRequest) returns (RenderedUI);

    // Action von Realm-UI ausführen
    rpc ExecuteAction(UIActionRequest) returns (UIActionResponse);

    // Navigation zu einem anderen Room
    rpc Navigate(NavigationRequest) returns (NavigationResponse);

    // Binding-Wert abrufen
    rpc GetBindingValue(BindingRequest) returns (BindingResponse);

    // Bulk-Bindings abrufen (Performance)
    rpc GetBindingValues(BulkBindingRequest) returns (BulkBindingResponse);
}

// ============================================================================
// Messages
// ============================================================================

message InitialUIRequest {
    string room_id = 1;
    string peer_did = 2;
    TrustVector6D trust_vector = 3;
    repeated string credentials = 4;
    string locale = 5;
    string theme_preference = 6;
}

message RenderedUI {
    LayoutDefinition layout = 1;
    repeated RenderedComponent components = 2;
    float peer_trust = 3;
    map<string, google.protobuf.Value> initial_bindings = 4;
    ThemeDefinition theme = 5;
    uint64 sequence_number = 6;
}

message UIStateUpdate {
    string room_id = 1;
    UIUpdateType update_type = 2;
    oneof payload {
        RenderedUI full_layout = 3;
        UIDelta delta = 4;
        BindingUpdate binding = 5;
        VisibilityUpdate visibility = 6;
        NavigationUpdate navigation = 7;
        ThemeDefinition theme = 8;
        Notification notification = 9;
    }
    uint64 sequence_number = 10;
    google.protobuf.Timestamp timestamp = 11;
}

message UIEvent {
    string room_id = 1;
    UIEventType event_type = 2;
    string source_component = 3;
    google.protobuf.Struct payload = 4;
    string peer_did = 5;
    google.protobuf.Timestamp timestamp = 6;
}

message UIActionRequest {
    string session_id = 1;
    string action_type = 2;
    string target_component = 3;
    google.protobuf.Struct params = 4;
}

message UIActionResponse {
    bool success = 1;
    repeated UIStateUpdate updates = 2;
    string error_message = 3;
}

// ============================================================================
// Enums
// ============================================================================

enum UIUpdateType {
    UI_UPDATE_TYPE_UNSPECIFIED = 0;
    UI_UPDATE_TYPE_FULL_LAYOUT = 1;
    UI_UPDATE_TYPE_DELTA = 2;
    UI_UPDATE_TYPE_BINDING = 3;
    UI_UPDATE_TYPE_VISIBILITY = 4;
    UI_UPDATE_TYPE_NAVIGATION = 5;
    UI_UPDATE_TYPE_THEME = 6;
    UI_UPDATE_TYPE_NOTIFICATION = 7;
}

enum UIEventType {
    UI_EVENT_TYPE_UNSPECIFIED = 0;
    UI_EVENT_TYPE_INTERACTION = 1;
    UI_EVENT_TYPE_FORM_SUBMIT = 2;
    UI_EVENT_TYPE_NAVIGATION_REQUEST = 3;
    UI_EVENT_TYPE_BINDING_CHANGE = 4;
    UI_EVENT_TYPE_LIFECYCLE = 5;
    UI_EVENT_TYPE_CLIENT_ERROR = 6;
}

// ============================================================================
// Sub-Messages
// ============================================================================

message LayoutDefinition {
    string layout_type = 1;  // "grid", "flex", "stack", etc.
    map<string, string> properties = 2;
    repeated LayoutSlot slots = 3;
}

message LayoutSlot {
    string name = 1;
    repeated string component_ids = 2;
    map<string, string> slot_properties = 3;
}

message RenderedComponent {
    string id = 1;
    string component_type = 2;
    map<string, google.protobuf.Value> props = 3;
    repeated string children = 4;
    bool visible = 5;
    map<string, string> bindings = 6;
    repeated string event_handlers = 7;
}

message UIDelta {
    repeated ComponentChange component_changes = 1;
    repeated BindingChange binding_changes = 2;
    optional LayoutDefinition layout_change = 3;
}

message ComponentChange {
    string component_id = 1;
    ChangeType change_type = 2;
    optional RenderedComponent new_component = 3;
    map<string, google.protobuf.Value> prop_changes = 4;
}

enum ChangeType {
    CHANGE_TYPE_UNSPECIFIED = 0;
    CHANGE_TYPE_ADD = 1;
    CHANGE_TYPE_UPDATE = 2;
    CHANGE_TYPE_REMOVE = 3;
    CHANGE_TYPE_MOVE = 4;
}

message BindingUpdate {
    string path = 1;
    google.protobuf.Value value = 2;
}

message BindingChange {
    string path = 1;
    google.protobuf.Value old_value = 2;
    google.protobuf.Value new_value = 3;
}

message VisibilityUpdate {
    repeated string component_ids = 1;
    bool visible = 2;
    string reason = 3;  // "trust_gate", "credential_gate", "condition"
}

message NavigationUpdate {
    string route = 1;
    map<string, string> params = 2;
    bool replace_history = 3;
}

message ThemeDefinition {
    string theme_id = 1;
    map<string, string> colors = 2;
    map<string, string> typography = 3;
    map<string, string> spacing = 4;
    map<string, string> custom = 5;
}

message Notification {
    NotificationLevel level = 1;
    string message = 2;
    string title = 3;
    int32 duration_ms = 4;
    optional UIAction action = 5;
}

enum NotificationLevel {
    NOTIFICATION_LEVEL_UNSPECIFIED = 0;
    NOTIFICATION_LEVEL_INFO = 1;
    NOTIFICATION_LEVEL_SUCCESS = 2;
    NOTIFICATION_LEVEL_WARNING = 3;
    NOTIFICATION_LEVEL_ERROR = 4;
}

message UIAction {
    string label = 1;
    string action_type = 2;
    google.protobuf.Struct params = 3;
}

message TrustVector6D {
    float r = 1;  // Reliability
    float i = 2;  // Integrity
    float c = 3;  // Competence
    float p = 4;  // Predictability
    float v = 5;  // Value Alignment
    float omega = 6;  // Overall Trust
}

message NavigationRequest {
    string session_id = 1;
    string target_room_id = 2;
    map<string, string> params = 3;
}

message NavigationResponse {
    bool allowed = 1;
    string redirect_room_id = 2;
    string error_message = 3;
}

message BindingRequest {
    string session_id = 1;
    string binding_path = 2;
}

message BindingResponse {
    google.protobuf.Value value = 1;
    bool found = 2;
}

message BulkBindingRequest {
    string session_id = 1;
    repeated string binding_paths = 2;
}

message BulkBindingResponse {
    map<string, google.protobuf.Value> values = 1;
}
```

```

### Phase 5: API-Engine (2-3 Wochen)

#### 5.5.1 Dateien erstellen

```

eclvm/engines/api_engine/
├── mod.rs # ~100 Zeilen
├── endpoints.rs # ~500 Zeilen
├── router.rs # ~400 Zeilen
├── auth.rs # ~400 Zeilen
├── rate_limit.rs # ~300 Zeilen
├── schema.rs # ~300 Zeilen
└── openapi.rs # ~400 Zeilen

````

#### 5.5.2 Haupt-Implementierung

```rust
// eclvm/engines/api_engine/mod.rs

pub struct APIEngine {
    endpoints: HashMap<ScopeId, Vec<CompiledEndpoint>>,
    rate_limiters: HashMap<UniversalId, RateLimiter>,
    api_keys: APIKeyRegistry,
    schema_validator: SchemaValidator,
}

impl APIEngine {
    pub fn register_endpoint(&mut self, scope: &ScopeId, definition: &ECLAPIEndpoint, controller: &UniversalId) -> Result<EndpointId>;
    pub async fn handle_request(&self, scope: &ScopeId, request: APIRequest, vm: &mut ECLVM) -> Result<APIResponse>;
    pub fn generate_openapi_spec(&self, scope: &ScopeId) -> Result<OpenAPISpec>;
}
````

### Phase 6: Governance-Engine (3-4 Wochen)

#### 5.6.1 Dateien erstellen

```
eclvm/engines/governance_engine/
├── mod.rs              # ~100 Zeilen
├── modes.rs            # ~600 Zeilen
├── proposals.rs        # ~500 Zeilen
├── voting.rs           # ~500 Zeilen
├── timelock.rs         # ~300 Zeilen
├── delegation.rs       # ~400 Zeilen (Liquid Democracy)
└── veto.rs             # ~200 Zeilen
```

#### 5.6.2 Haupt-Implementierung

```rust
// eclvm/engines/governance_engine/mod.rs

pub struct GovernanceEngine {
    configs: HashMap<ScopeId, GovernanceConfig>,
    proposals: HashMap<ProposalId, Proposal>,
    vote_history: VoteHistory,
    timelock_queue: TimelockQueue,
}

impl GovernanceEngine {
    pub fn create_proposal(&mut self, scope: &ScopeId, proposer: &UniversalId, proposal: ProposalDraft) -> Result<ProposalId>;
    pub fn cast_vote(&mut self, proposal_id: &ProposalId, voter: &UniversalId, vote: VoteChoice) -> Result<()>;
    pub fn execute_proposal(&mut self, proposal_id: &ProposalId, executor: &UniversalId) -> Result<ExecutionReceipt>;
    pub fn calculate_voting_power(&self, scope: &ScopeId, voter: &UniversalId) -> Result<f32>;
}
```

### Phase 7: Blueprint-Engine (3-4 Wochen)

#### 5.7.1 Dateien erstellen

```
eclvm/engines/blueprint_engine/
├── mod.rs              # ~100 Zeilen
├── deployer.rs         # ~700 Zeilen
├── upgrader.rs         # ~500 Zeilen
├── composer.rs         # ~600 Zeilen
├── migrator.rs         # ~400 Zeilen
└── validator.rs        # ~300 Zeilen
```

#### 5.7.2 Blueprint-Marketplace Refactoring

```rust
// local/blueprint_marketplace.rs - Änderungen

// VORHER:
pub struct Blueprint {
    pub stores: Vec<BlueprintStore>,
    pub policies: Vec<BlueprintPolicy>,
    pub sagas: Vec<BlueprintSaga>,
}

// NACHHER:
pub struct ExtendedBlueprint {
    pub structure: BlueprintStructure,
    pub policy: BlueprintPolicy,
    pub ui: BlueprintUI,
    pub datalogic: BlueprintDataLogic,
    pub api: BlueprintAPI,
    pub governance: BlueprintGovernance,
    pub controller: BlueprintController,
}

// Migration: Bestehende Blueprints werden in "legacy" Modus geladen
impl Blueprint {
    pub fn to_extended(&self) -> ExtendedBlueprint {
        ExtendedBlueprint {
            structure: BlueprintStructure::from_stores(&self.stores),
            policy: BlueprintPolicy::from_legacy(&self.policies),
            // ... defaults für neue Felder
        }
    }
}
```

### Phase 8: Integration & State Management (2 Wochen)

#### 5.8.1 State-Erweiterungen

```rust
// core/state.rs - Neue State-Komponenten

pub struct UnifiedState {
    // ... bestehende ...

    // NEU: Engine-States
    pub ui_state: UIState,
    pub api_state: APIState,
    pub governance_state: GovernanceState,
    pub controller_state: ControllerState,
}

pub struct UIState {
    pub active_uis: HashMap<RoomId, CompiledUISnapshot>,
    pub binding_states: HashMap<String, Value>,
    pub render_cache: HashMap<(RoomId, DID), RenderedUISnapshot>,
}

pub struct APIState {
    pub registered_endpoints: HashMap<ScopeId, Vec<EndpointSnapshot>>,
    pub rate_limit_states: HashMap<UniversalId, RateLimitSnapshot>,
    pub api_key_count: u64,
}

pub struct GovernanceState {
    pub active_proposals: HashMap<ProposalId, ProposalSnapshot>,
    pub completed_votes: u64,
    pub timelock_queue_size: u64,
}
```

#### 5.8.2 Observer-Erweiterungen

```rust
// core/state_integration.rs - Neue Observers

pub trait UIObserver: Send + Sync {
    fn on_ui_compiled(&self, room_id: &RoomId, component_count: usize);
    fn on_ui_rendered(&self, room_id: &RoomId, peer: &DID, trust: f64);
    fn on_ui_delta_applied(&self, room_id: &RoomId, delta_type: &str);
}

pub trait APIObserver: Send + Sync {
    fn on_endpoint_registered(&self, scope: &ScopeId, path: &str, method: &str);
    fn on_api_request(&self, scope: &ScopeId, path: &str, method: &str, status: u16);
    fn on_rate_limited(&self, client_id: &UniversalId, endpoint: &str);
}

pub trait GovernanceObserver: Send + Sync {
    fn on_proposal_created(&self, proposal_id: &ProposalId, proposer: &UniversalId);
    fn on_vote_cast(&self, proposal_id: &ProposalId, voter: &UniversalId, power: f32);
    fn on_proposal_executed(&self, proposal_id: &ProposalId, success: bool);
}
```

---

## 6. Detaillierte Änderungen pro Modul

### 6.1 `eclvm/` Modul

#### 6.1.1 `mod.rs` Änderungen

```diff
// eclvm/mod.rs

+ pub mod engines;
+ pub mod types;

  pub mod ast;
  pub mod bridge;
  pub mod bytecode;
  // ... rest unchanged ...

+ // Re-exports für Engines
+ pub use engines::{
+     UIEngine, DataLogicEngine, APIEngine,
+     GovernanceEngine, ControllerEngine, BlueprintEngine
+ };
+ pub use types::*;
```

#### 6.1.2 `bytecode.rs` Änderungen (~50 neue Zeilen)

```rust
// Neue OpCodes hinzufügen

pub enum OpCode {
    // ... existing 40+ opcodes ...

    // UI Operations (10 neue)
    UIRender,
    UIBindGet,
    UIBindSet,
    UIComponentVisible,
    UIApplyDelta,
    UITrustGate,
    UICredentialGate,
    UIEmitUpdate,
    UIGetLayout,
    UIGetComponent,

    // API Operations (8 neue)
    APIRespond,
    APIValidateSchema,
    APIRateCheck,
    APIGetHeader,
    APIGetParam,
    APISetHeader,
    APIError,
    APILog,

    // Governance Operations (8 neue)
    GovPropose,
    GovVote,
    GovExecute,
    GovVetoPower,
    GovGetProposal,
    GovGetVotes,
    GovCheckQuorum,
    GovTimelockRemaining,

    // Controller Operations (6 neue)
    CtrlValidate,
    CtrlDelegate,
    CtrlRevoke,
    CtrlGetPermissions,
    CtrlCheckScope,
    CtrlAuditLog,
}

impl OpCode {
    pub fn gas_cost(&self) -> u64 {
        match self {
            // UI Ops: Moderate Cost
            Self::UIRender => 100,
            Self::UIBindGet => 10,
            Self::UIBindSet => 25,
            Self::UIComponentVisible => 15,
            Self::UIApplyDelta => 200,

            // API Ops: Variable
            Self::APIRespond => 50,
            Self::APIValidateSchema => 100,
            Self::APIRateCheck => 5,

            // Governance Ops: High Cost
            Self::GovPropose => 500,
            Self::GovVote => 100,
            Self::GovExecute => 1000,

            // Controller Ops: Moderate
            Self::CtrlValidate => 50,
            Self::CtrlDelegate => 200,
            Self::CtrlRevoke => 100,

            // ... existing costs ...
        }
    }
}
```

#### 6.1.3 `runtime/host.rs` Erweiterungen

```rust
// HostInterface erweitern

pub trait HostInterface: Send + Sync {
    // ... existing methods ...

    // NEU: UI-Host-Functions
    fn ui_render(&self, room_id: &str, peer: &str) -> Result<Value>;
    fn ui_bind_get(&self, binding_id: &str) -> Result<Value>;
    fn ui_bind_set(&self, binding_id: &str, value: Value) -> Result<()>;

    // NEU: API-Host-Functions
    fn api_respond(&self, status: u16, body: Value) -> Result<()>;
    fn api_validate_schema(&self, data: &Value, schema: &str) -> Result<bool>;

    // NEU: Governance-Host-Functions
    fn gov_get_proposal(&self, proposal_id: &str) -> Result<Value>;
    fn gov_cast_vote(&self, proposal_id: &str, choice: &str) -> Result<()>;

    // NEU: Controller-Host-Functions
    fn ctrl_validate_action(&self, scope: &str, actor: &str, action: &str) -> Result<bool>;
    fn ctrl_delegate(&self, from: &str, to: &str, permissions: Value) -> Result<()>;
}
```

### 6.2 `domain/unified/` Modul

#### 6.2.1 `realm.rs` Erweiterungen (~200 neue Zeilen)

```rust
// Nach bestehenden Realm-Definitionen hinzufügen

// ============================================================================
// Room (Sub-Container in VirtualRealm)
// ============================================================================

pub type RoomId = UniversalId;

pub fn room_id_from_name(realm_id: &RealmId, name: &str) -> RoomId {
    let content = format!("{}:{}", realm_id.to_hex(), name);
    UniversalId::new(UniversalId::TAG_ROOM, 1, content.as_bytes())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub name: String,
    pub realm_id: RealmId,
    pub description: Option<String>,
    pub partitions: Vec<Partition>,
    pub controller: Option<UniversalId>,
    pub policy: Option<String>,  // ECL-Policy-ID
    pub ui: Option<String>,      // ECL-UI-ID
    pub created_at: TemporalCoord,
}

// ============================================================================
// Partition (Sub-Container in Room)
// ============================================================================

pub type PartitionId = UniversalId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    pub id: PartitionId,
    pub name: String,
    pub room_id: RoomId,
    pub store_schema: StoreSchema,
    pub access_policy: AccessPolicy,
    pub created_at: TemporalCoord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    pub read_access: AccessRule,
    pub write_access: AccessRule,
    pub delete_access: AccessRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessRule {
    All,
    None,
    OwnerOnly,
    TrustMin(f32),
    Credential(String),
    Controller,
    Custom(String),  // ECL-Code
}
```

#### 6.2.2 `saga.rs` Erweiterungen (~150 neue Zeilen)

```rust
// Goal-Enum erweitern

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Goal {
    // ... bestehende Goals ...

    // Realm/Raum-Management
    RealmModify {
        realm_id: RealmId,
        modification: RealmModification,
    },
    RoomCreate {
        realm_id: RealmId,
        room_config: RoomConfig,
    },
    PartitionCreate {
        room_id: RoomId,
        partition_config: PartitionConfig,
    },

    // UI-Modifikation
    UIModify {
        scope: ScopeId,
        ui_delta: UIDelta,
    },

    // Governance
    Governance {
        proposal: ProposalDraft,
    },
    Vote {
        proposal_id: ProposalId,
        choice: VoteChoice,
    },

    // Cross-Realm
    CrossRealm {
        from_realm: RealmId,
        to_realm: RealmId,
        action: Box<Goal>,
    },

    // API
    APIRegister {
        scope: ScopeId,
        endpoint_config: APIEndpointConfig,
    },

    // Blueprint
    BlueprintDeploy {
        blueprint_id: BlueprintId,
        target_realm: RealmId,
        config: DeploymentConfig,
    },
    BlueprintUpgrade {
        deployment_id: DeploymentId,
        new_version: BlueprintId,
    },
}

// SagaAction erweitern
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SagaAction {
    // ... bestehende Actions ...

    // Realm-Management
    CreateRoom { room_config: RoomConfig },
    CreatePartition { partition_config: PartitionConfig },
    ModifyRealm { realm_id: RealmId, modification: RealmModification },

    // UI
    CompileUI { room_id: RoomId, ecl_ui: String },
    ApplyUIDelta { room_id: RoomId, delta: UIDelta },

    // API
    RegisterEndpoint { scope: ScopeId, endpoint: APIEndpointConfig },
    RemoveEndpoint { endpoint_id: EndpointId },

    // Governance
    CreateProposal { proposal: ProposalDraft },
    CastVote { proposal_id: ProposalId, choice: VoteChoice },
    ExecuteProposal { proposal_id: ProposalId },

    // Controller
    SetController { scope: ScopeId, controller: ControllerConfig },
    Delegate { from: UniversalId, to: UniversalId, delegation: Delegation },

    // Blueprint
    DeployBlueprint { blueprint_id: BlueprintId, target: RealmId },
    UpgradeDeployment { deployment_id: DeploymentId, new_version: BlueprintId },
}
```

### 6.3 `peer/saga_composer.rs` Erweiterungen (~300 neue Zeilen)

```rust
// Neue compose-Methoden hinzufügen

impl SagaComposer {
    // ... bestehende Methoden ...

    /// Komponiere Room-Creation-Saga
    fn compose_room_create(
        &self,
        creator: &UniversalId,
        realm_id: &RealmId,
        config: &RoomConfig,
    ) -> CompositionResult<Vec<SagaStep>> {
        let mut steps = Vec::new();

        // Step 1: Validate Controller-Permission
        steps.push(SagaStep::new(
            0,
            "Validate controller permission",
            SagaAction::Custom {
                ecl_code: format!(
                    "ctrl.validate({}, {}, 'create_room')",
                    realm_id.to_hex(),
                    creator.to_hex()
                ),
            },
        ));

        // Step 2: Create Room
        steps.push(SagaStep::new(
            1,
            format!("Create room '{}'", config.name),
            SagaAction::CreateRoom {
                room_config: config.clone(),
            },
        ).with_dependencies(vec![0]));

        // Step 3: Compile UI (if provided)
        if let Some(ui_ecl) = &config.ui {
            steps.push(SagaStep::new(
                2,
                "Compile UI definition",
                SagaAction::CompileUI {
                    room_id: RoomId::NULL, // Placeholder
                    ecl_ui: ui_ecl.clone(),
                },
            ).with_dependencies(vec![1]));
        }

        Ok(steps)
    }

    /// Komponiere Governance-Saga
    fn compose_governance(
        &self,
        proposer: &UniversalId,
        proposal: &ProposalDraft,
    ) -> CompositionResult<Vec<SagaStep>> {
        let mut steps = Vec::new();

        // Step 1: Check proposal threshold
        steps.push(SagaStep::new(
            0,
            "Check proposal threshold",
            SagaAction::CheckTrust {
                min_r: 0.5,
                min_omega: 1.0,
            },
        ));

        // Step 2: Create proposal
        steps.push(SagaStep::new(
            1,
            "Create proposal",
            SagaAction::CreateProposal {
                proposal: proposal.clone(),
            },
        ).with_dependencies(vec![0]));

        Ok(steps)
    }

    /// Komponiere Blueprint-Deployment-Saga
    fn compose_blueprint_deploy(
        &self,
        deployer: &UniversalId,
        blueprint_id: &BlueprintId,
        target_realm: &RealmId,
        config: &DeploymentConfig,
    ) -> CompositionResult<Vec<SagaStep>> {
        // Multi-Step Deployment:
        // 1. Load Blueprint
        // 2. Validate compatibility
        // 3. Deploy structure
        // 4. Deploy policies
        // 5. Deploy UI
        // 6. Deploy DataLogic
        // 7. Deploy API
        // 8. Deploy Governance
        // 9. Configure Controller
        // 10. Emit deployment event

        // ... Implementation ...
    }
}
```

### 6.4 `local/blueprint_marketplace.rs` Refactoring

#### 6.4.1 Blueprint-Struktur erweitern

```rust
// Am Anfang der Datei nach bestehenden Imports

// ============================================================================
// Erweiterte Blueprint-Komponenten (NEU)
// ============================================================================

/// Blueprint-Struktur (Räume, Partitionen)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlueprintStructure {
    pub rooms: Vec<RoomTemplate>,
    pub partitions: Vec<PartitionTemplate>,
    pub stores: Vec<BlueprintStore>,  // Bestehendes Feld migriert
    pub allow_dynamic_rooms: bool,
    pub dynamic_room_template: Option<String>,
}

/// Blueprint-UI (Layouts, Pages, Components)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlueprintUI {
    pub theme: HashMap<String, String>,
    pub layouts: Vec<UILayoutDef>,
    pub pages: Vec<UIPageDef>,
    pub components: Vec<UIComponentDef>,
    pub default_trust_gates: HashMap<String, f32>,
}

/// Blueprint-DataLogic (Handler, Aggregations)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlueprintDataLogic {
    pub handlers: Vec<EventHandlerDef>,
    pub aggregations: Vec<AggregationDef>,
    pub outputs: Vec<OutputDef>,
}

/// Blueprint-API (Endpoints, Schemas)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlueprintAPI {
    pub version: String,
    pub base_path: String,
    pub endpoints: Vec<APIEndpointDef>,
    pub default_auth: EndpointAuthDef,
    pub default_rate_limit: RateLimitDef,
    pub schemas: HashMap<String, serde_json::Value>,
}

/// Blueprint-Governance (Voting, Proposals)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlueprintGovernance {
    pub mode: GovernanceModeDef,
    pub voting_rules: VotingRulesDef,
    pub proposal_templates: Vec<ProposalTemplateDef>,
    pub action_overrides: HashMap<String, VotingRulesDef>,
    pub veto_config: Option<VetoConfigDef>,
}

/// Blueprint-Controller (Permissions, Automation)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlueprintController {
    pub primary: Option<ControllerSpecDef>,
    pub governance_controlled: bool,
    pub permissions: PermissionsDef,
    pub automation: Vec<AutomationRuleDef>,
    pub delegation_config: DelegationConfigDef,
    pub audit_config: AuditConfigDef,
}
```

#### 6.4.2 Blueprint-Struktur migrieren

```rust
// Bestehende Blueprint-Struktur wird zu ExtendedBlueprint

/// Legacy Blueprint (für Abwärtskompatibilität)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyBlueprint {
    pub id: BlueprintId,
    pub name: String,
    pub version: SemVer,
    pub stores: Vec<BlueprintStore>,
    pub policies: Vec<BlueprintPolicy>,
    pub sagas: Vec<BlueprintSaga>,
    // ... andere bestehende Felder
}

/// Erweitertes Blueprint mit allen ECL-Komponenten
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedBlueprint {
    // Identifikation
    pub id: BlueprintId,
    pub version: SemVer,
    pub content_hash: Hash,

    // Metadaten
    pub name: String,
    pub description: String,
    pub creator_did: UniversalId,
    pub created_at: TemporalCoord,
    pub tags: Vec<String>,
    pub category: BlueprintCategory,
    pub license: BlueprintLicense,

    // ECL-Komponenten
    pub structure: BlueprintStructure,
    pub policy: BlueprintPolicySection,
    pub ui: BlueprintUI,
    pub datalogic: BlueprintDataLogic,
    pub api: BlueprintAPI,
    pub governance: BlueprintGovernance,
    pub controller: BlueprintController,

    // Versionierung
    pub predecessor: Option<BlueprintId>,
    pub forked_from: Option<BlueprintId>,
    pub dependencies: Vec<BlueprintDependency>,

    // Metriken
    pub complexity: u64,
    pub novelty_score: f64,
    pub diversity_contribution: f64,
    pub omega_contribution: f64,
}

impl LegacyBlueprint {
    /// Konvertiere zu ExtendedBlueprint
    pub fn to_extended(&self) -> ExtendedBlueprint {
        ExtendedBlueprint {
            id: self.id.clone(),
            version: self.version.clone(),
            content_hash: Hash::default(),
            name: self.name.clone(),
            description: self.description.clone(),
            creator_did: UniversalId::from_did(&self.creator_did),
            // ...
            structure: BlueprintStructure {
                stores: self.stores.clone(),
                ..Default::default()
            },
            policy: BlueprintPolicySection::from_legacy(&self.policies),
            // ... defaults für neue Felder
            ..Default::default()
        }
    }
}

// Type Alias für Kompatibilität
pub type Blueprint = ExtendedBlueprint;
```

---

## 7. Migrations-Strategie

### 7.1 Datenbank-Migration

#### 7.1.1 Neue Partitionen

```rust
// local/mod.rs - Neue Storage-Partitionen

pub struct Storage {
    // Bestehende
    pub identities: IdentityStore,
    pub events: EventStore,
    pub trust: TrustStore,
    pub content: ContentStore,
    pub realm_storage: RealmStorage,
    pub blueprint_marketplace: BlueprintMarketplace,

    // NEU
    pub ui_store: UIStore,
    pub api_store: APIStore,
    pub governance_store: GovernanceStore,
    pub controller_store: ControllerStore,
}
```

#### 7.1.2 Blueprint-Migration

```rust
// Migration Script

pub async fn migrate_blueprints(
    old_marketplace: &LegacyBlueprintMarketplace,
    new_marketplace: &mut BlueprintMarketplace,
) -> Result<MigrationReport> {
    let mut report = MigrationReport::default();

    for legacy_bp in old_marketplace.list_all()? {
        let extended = legacy_bp.to_extended();

        // Validiere extended
        if let Err(e) = extended.validate() {
            report.errors.push((legacy_bp.id.clone(), e.to_string()));
            continue;
        }

        // Speichere mit neuem Format
        new_marketplace.store_extended(&extended)?;
        report.migrated += 1;
    }

    Ok(report)
}
```

### 7.2 API-Kompatibilität

#### 7.2.1 Versioned Endpoints

```rust
// Bestehende API bleibt unter /api/v1
// Neue API unter /api/v2

// api/router.rs
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // Legacy V1 API
        .service(
            web::scope("/api/v1")
                .configure(legacy_routes)
        )
        // New V2 API with ECL-defined endpoints
        .service(
            web::scope("/api/v2")
                .configure(ecl_api_routes)
        );
}
```

### 7.3 Feature Flags

```rust
// Cargo.toml
[features]
default = ["legacy_blueprints"]
legacy_blueprints = []
extended_blueprints = ["legacy_blueprints"]
ui_engine = ["extended_blueprints"]
api_engine = ["extended_blueprints"]
governance_engine = ["extended_blueprints"]
all_engines = ["ui_engine", "api_engine", "governance_engine"]

// Verwendung im Code
#[cfg(feature = "ui_engine")]
pub mod ui_engine;

#[cfg(feature = "governance_engine")]
pub mod governance_engine;
```

---

## 8. Test-Strategie

### 8.1 Unit-Tests pro Engine

```rust
// eclvm/engines/ui_engine/tests.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_gate_filtering() {
        let mut engine = UIEngine::new();
        let ui = create_test_ui_with_trust_gates();
        engine.compile_ui(&room_id, &ui).unwrap();

        // Peer mit Trust 0.5 sollte Components mit gate <= 0.5 sehen
        let rendered = engine.render_for_peer(
            &room_id,
            &peer_did,
            &TrustVector6D::uniform(0.5),
            &[],
        ).unwrap();

        assert_eq!(rendered.components.len(), 3);  // 5 total, 2 gated out
    }

    #[test]
    fn test_data_binding() {
        // ...
    }
}
```

### 8.2 Integrations-Tests

```rust
// tests/engine_integration.rs

#[tokio::test]
async fn test_full_blueprint_deployment() {
    let (storage, engines) = setup_test_environment().await;

    // 1. Create Blueprint
    let blueprint = create_test_blueprint();

    // 2. Deploy
    let deployment = engines.blueprint_engine
        .deploy(&blueprint.id, &realm_id, &deployer, config)
        .await
        .unwrap();

    // 3. Verify all engines configured
    assert!(engines.ui_engine.has_ui(&room_id));
    assert!(engines.api_engine.has_endpoints(&realm_id));
    assert!(engines.governance_engine.has_config(&realm_id));

    // 4. Test API endpoint works
    let response = engines.api_engine
        .handle_request(&realm_id, test_request)
        .await
        .unwrap();
    assert_eq!(response.status, 200);
}
```

### 8.3 Property-Based Tests

```rust
// tests/property_tests.rs

use proptest::prelude::*;

proptest! {
    #[test]
    fn governance_voting_power_sums_correctly(
        votes in prop::collection::vec(arbitrary_vote(), 1..100)
    ) {
        let engine = GovernanceEngine::new_test();
        let proposal = create_test_proposal();

        for vote in &votes {
            engine.cast_vote(&proposal.id, &vote.voter, vote.choice).unwrap();
        }

        let result = engine.tally_votes(&proposal.id).unwrap();
        let expected_total: f32 = votes.iter().map(|v| v.power).sum();

        prop_assert!((result.total_power - expected_total).abs() < 0.001);
    }
}
```

---

## 9. Risiken & Mitigationen

### 9.1 Technische Risiken

| Risiko                                    | Wahrscheinlichkeit | Impact | Mitigation                            |
| ----------------------------------------- | ------------------ | ------ | ------------------------------------- |
| Breaking Changes in Domain-Types          | Hoch               | Hoch   | Versionierte Types, Migration-Scripts |
| Performance-Regression durch neue Engines | Mittel             | Mittel | Benchmarks, Lazy-Loading              |
| Speicher-Overflow bei großen UIs          | Mittel             | Hoch   | Pagination, Lazy-Rendering            |
| Race Conditions in Governance             | Niedrig            | Hoch   | Atomic Operations, Locking            |
| Gas-Exhaustion bei komplexen Blueprints   | Mittel             | Mittel | Gas-Limits, Progressive Loading       |

### 9.2 Projekt-Risiken

| Risiko               | Wahrscheinlichkeit | Impact | Mitigation                         |
| -------------------- | ------------------ | ------ | ---------------------------------- |
| Scope Creep          | Hoch               | Hoch   | Klare Phasen-Grenzen, MVP-First    |
| Unvollständige Tests | Mittel             | Hoch   | Test-Coverage-Metriken, CI-Gates   |
| Dokumentations-Drift | Hoch               | Mittel | Doc-as-Code, Automatisierte Checks |

### 9.3 Rollback-Strategie

```rust
// Jede Engine kann einzeln deaktiviert werden

pub struct EngineManager {
    pub ui_enabled: bool,
    pub api_enabled: bool,
    pub governance_enabled: bool,
    // ...
}

impl EngineManager {
    pub fn disable_engine(&mut self, engine: EngineType) {
        match engine {
            EngineType::UI => {
                self.ui_enabled = false;
                // Fallback zu statischem UI
            }
            EngineType::Governance => {
                self.governance_enabled = false;
                // Fallback zu Controller-only
            }
            // ...
        }
    }
}
```

---

## Anhang A: Checkliste pro Phase

### Phase 1 Checkliste

- [ ] `domain/unified/ui.rs` erstellt
- [ ] `domain/unified/api.rs` erstellt
- [ ] `domain/unified/governance.rs` erstellt
- [ ] `domain/unified/controller.rs` erstellt
- [ ] `domain/unified/realm.rs` erweitert (Room, Partition)
- [ ] `domain/unified/saga.rs` erweitert (neue Goals)
- [ ] `eclvm/bytecode.rs` erweitert (neue OpCodes)
- [ ] Tests für neue Domain-Types

### Phase 2 Checkliste (Controller-Engine)

- [ ] `eclvm/engines/controller_engine/mod.rs`
- [ ] `eclvm/engines/controller_engine/permissions.rs`
- [ ] `eclvm/engines/controller_engine/delegation.rs`
- [ ] `eclvm/engines/controller_engine/automation.rs`
- [ ] `eclvm/engines/controller_engine/audit.rs`
- [ ] `eclvm/engines/controller_engine/governance_binding.rs`
- [ ] Unit-Tests für Controller-Engine
- [ ] Integration mit bestehender Gateway

### (Weitere Checklisten für Phasen 3-8...)

---

## Anhang B: Glossar

| Begriff         | Definition                                   |
| --------------- | -------------------------------------------- |
| ECL             | Erynoa Configuration Language                |
| Blueprint       | Immutables, versioniertes Template           |
| Engine          | Spezialisierte VM-Erweiterung                |
| Scope           | Berechtigungs-Kontext (Realm/Room/Partition) |
| Trust-Gate      | Trust-basierte Sichtbarkeits-Kontrolle       |
| Credential-Gate | Credential-basierte Zugriffskontrolle        |
| Binding         | Reaktive Daten-Verbindung                    |
| Proposal        | Governance-Vorschlag                         |
| Delegation      | Kontroll-Übertragung                         |

---

## 13. StateManager Refactoring-Plan (Detailliert)

### 13.1 Executive Summary

Der StateManager (`core/state.rs`, `core/state_integration.rs`) ist das **Nervenzentrum** des Erynoa-Systems. Er verwaltet ~7.375 Zeilen Code und muss für die 6 neuen Engines erweitert werden.

#### 13.1.1 Ziel-Metriken

| Metrik                   | IST   | SOLL    | Delta  |
| ------------------------ | ----- | ------- | ------ |
| `state.rs` Zeilen        | 4.389 | ~5.800  | +1.411 |
| `state_integration.rs`   | 2.986 | ~4.100  | +1.114 |
| StateComponent-Varianten | 24    | **32**  | +8     |
| StateGraph-Edges         | ~50   | **~85** | +35    |
| Observer-Traits          | 14    | **20**  | +6     |
| State-Structs            | 22    | **28**  | +6     |

---

### 13.2 StateComponent Enum Erweiterung

**Datei:** `backend/src/core/state.rs`
**Zeile:** ~107-150 (nach `Privacy`)

#### 13.2.1 IST-Zustand

```rust
pub enum StateComponent {
    // Core
    Trust,
    Event,
    WorldFormula,
    Consensus,
    // Execution
    Gas,
    Mana,
    Execution,
    // ECLVM Layer
    ECLVM,
    ECLPolicy,
    ECLBlueprint,
    // Protection
    Anomaly,
    Diversity,
    Quadratic,
    AntiCalcification,
    Calibration,
    // Storage
    KvStore,
    EventStore,
    Archive,
    Blueprint,
    // Peer Layer
    Gateway,
    SagaComposer,
    IntentParser,
    Realm,
    // P2P Network Layer
    Swarm,
    Gossip,
    Kademlia,
    Relay,
    NatTraversal,
    Privacy,
}
```

#### 13.2.2 SOLL-Zustand (8 neue Varianten)

```rust
pub enum StateComponent {
    // Core (unverändert: 4)
    Trust,
    Event,
    WorldFormula,
    Consensus,

    // Execution (unverändert: 3)
    Gas,
    Mana,
    Execution,

    // ECLVM Layer (unverändert: 3)
    ECLVM,
    ECLPolicy,
    ECLBlueprint,

    // Protection (unverändert: 5)
    Anomaly,
    Diversity,
    Quadratic,
    AntiCalcification,
    Calibration,

    // Storage (unverändert: 4)
    KvStore,
    EventStore,
    Archive,
    Blueprint,

    // Peer Layer (erweitert: 4 → 6)
    Gateway,
    SagaComposer,
    IntentParser,
    Realm,
    /// NEU: Room-basierte Isolation (Sub-Realm Partitionen)
    Room,
    /// NEU: Partition-Management (Controller-Scope)
    Partition,

    // P2P Network Layer (unverändert: 6)
    Swarm,
    Gossip,
    Kademlia,
    Relay,
    NatTraversal,
    Privacy,

    // ══════════════════════════════════════════════════════════════════════════
    // NEU: ENGINE-LAYER KOMPONENTEN (6 neue Engines)
    // ══════════════════════════════════════════════════════════════════════════

    /// UI-Engine State: Component-Tree, Bindings, Render-Cache
    /// Axiom-Referenz: Κ22 (Per-Realm UI-Isolation)
    UI,

    /// DataLogic-Engine State: Reactive Streams, Aggregations
    /// Axiom-Referenz: Κ9-Κ12 (Event-Verarbeitung)
    DataLogic,

    /// API-Engine State: Endpoint-Registry, Rate-Limits
    /// Axiom-Referenz: Κ23 (Gateway-Integration)
    API,

    /// Governance-Engine State: Proposals, Votes, Delegation-Graph
    /// Axiom-Referenz: Κ21 (Quadratic), Κ19 (Anti-Calc)
    Governance,

    /// Controller-Engine State: Permissions, Delegations, Audit-Log
    /// Axiom-Referenz: Κ5 (Trust-Relationship)
    Controller,

    /// Extended Blueprint-Engine State: Composition, Versioning
    /// Axiom-Referenz: Blueprint-Marketplace (Storage)
    BlueprintComposer,
}
```

#### 13.2.3 Code-Änderung

```diff
// backend/src/core/state.rs @ Zeile ~107

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StateComponent {
    // ... bestehende Varianten ...
    Privacy,
+
+   // ══════════════════════════════════════════════════════════════════════════
+   // PEER-LAYER ERWEITERUNG (Κ22-Κ24: Rooms & Partitions)
+   // ══════════════════════════════════════════════════════════════════════════
+   /// Room: Sub-Realm-Isolation mit eigenem Controller-Scope
+   Room,
+   /// Partition: Trust-basierte Berechtigungspartition
+   Partition,
+
+   // ══════════════════════════════════════════════════════════════════════════
+   // ENGINE-LAYER KOMPONENTEN (6 neue Engines für SOLL-Zustand)
+   // ══════════════════════════════════════════════════════════════════════════
+   /// UI-Engine: Deklaratives, Trust-basiertes Interface-Rendering
+   UI,
+   /// DataLogic-Engine: Reaktive Event-Verarbeitung und Aggregation
+   DataLogic,
+   /// API-Engine: Dynamische REST-API-Definition per ECL
+   API,
+   /// Governance-Engine: DAO-Prinzipien und Abstimmungsmechanismen
+   Governance,
+   /// Controller-Engine: Berechtigungsverwaltung mit Delegation
+   Controller,
+   /// BlueprintComposer: Template-Komposition und Vererbung
+   BlueprintComposer,
}
```

---

### 13.3 StateGraph Edge-Erweiterung

**Datei:** `backend/src/core/state.rs`
**Funktion:** `StateGraph::erynoa_graph()`
**Zeile:** ~160-245

#### 13.3.1 Neue Edges für Engine-Layer (~35 neue Beziehungen)

```rust
// StateGraph::erynoa_graph() - NEU HINZUZUFÜGEN nach bestehenden edges

// ══════════════════════════════════════════════════════════════════════════════
// ROOM & PARTITION BEZIEHUNGEN
// ══════════════════════════════════════════════════════════════════════════════
(Room, DependsOn, Realm),            // Room ist Sub-Einheit eines Realms
(Room, DependsOn, Trust),            // Room-Access prüft Trust
(Room, Triggers, Event),             // Room-Aktionen erzeugen Events
(Room, Aggregates, Controller),      // Room trackt Controller-Permissions
(Partition, DependsOn, Room),        // Partition ist Sub-Einheit eines Rooms
(Partition, DependsOn, Trust),       // Partition-Access prüft Trust
(Partition, Validates, Controller),  // Partition validiert Controller-Scope

// ══════════════════════════════════════════════════════════════════════════════
// UI-ENGINE BEZIEHUNGEN
// ══════════════════════════════════════════════════════════════════════════════
(UI, DependsOn, Trust),              // UI-Sichtbarkeit basiert auf Trust
(UI, DependsOn, Realm),              // UI ist per-Realm isoliert
(UI, DependsOn, Room),               // UI-Scoping auf Room-Ebene
(UI, DependsOn, Controller),         // UI nutzt Controller für Permissions
(UI, Triggers, Event),               // UI-Actions erzeugen Events
(UI, Aggregates, DataLogic),         // UI nutzt DataLogic für Bindings
(UI, DependsOn, ECLVM),              // UI-Logik läuft in ECLVM
(UI, DependsOn, Gas),                // UI-Rendering verbraucht Gas
(UI, DependsOn, Mana),               // UI-Events verbrauchen Mana

// ══════════════════════════════════════════════════════════════════════════════
// DATALOGIC-ENGINE BEZIEHUNGEN
// ══════════════════════════════════════════════════════════════════════════════
(DataLogic, DependsOn, Event),       // DataLogic verarbeitet Events
(DataLogic, Aggregates, Event),      // DataLogic aggregiert Event-Streams
(DataLogic, Triggers, Event),        // Aggregationen emittieren Events
(DataLogic, DependsOn, Trust),       // DataAccess prüft Trust
(DataLogic, DependsOn, ECLVM),       // DataLogic-Funktionen in ECLVM
(DataLogic, DependsOn, Gas),         // Compute verbraucht Gas
(DataLogic, Validates, UI),          // DataLogic validiert UI-Bindings

// ══════════════════════════════════════════════════════════════════════════════
// API-ENGINE BEZIEHUNGEN
// ══════════════════════════════════════════════════════════════════════════════
(API, DependsOn, Trust),             // API-Access basiert auf Trust
(API, DependsOn, Controller),        // API nutzt Controller für AuthZ
(API, Validates, Gateway),           // API validiert External-Gateway
(API, Triggers, Event),              // API-Calls erzeugen Events
(API, DependsOn, ECLVM),             // API-Handler laufen in ECLVM
(API, DependsOn, Gas),               // API-Processing verbraucht Gas
(API, DependsOn, Mana),              // API-Responses verbrauchen Mana
(API, Aggregates, DataLogic),        // API nutzt DataLogic für Queries

// ══════════════════════════════════════════════════════════════════════════════
// GOVERNANCE-ENGINE BEZIEHUNGEN
// ══════════════════════════════════════════════════════════════════════════════
(Governance, DependsOn, Trust),      // Voting-Power basiert auf Trust
(Governance, DependsOn, Quadratic),  // Governance nutzt Quadratic-Voting
(Governance, Validates, Controller), // Governance validiert Controller-Changes
(Governance, Triggers, Controller),  // Governance-Votes ändern Controller
(Governance, Triggers, Event),       // Proposals/Votes erzeugen Events
(Governance, DependsOn, ECLVM),      // Governance-Regeln in ECLVM
(Governance, DependsOn, Realm),      // Governance ist per-Realm
(Governance, Validates, AntiCalcification), // Governance prüft Machtkonzentration

// ══════════════════════════════════════════════════════════════════════════════
// CONTROLLER-ENGINE BEZIEHUNGEN
// ══════════════════════════════════════════════════════════════════════════════
(Controller, DependsOn, Trust),      // Permissions basieren auf Trust
(Controller, Triggers, Event),       // Permission-Changes erzeugen Events
(Controller, Validates, Gateway),    // Controller validiert Crossings
(Controller, Validates, API),        // Controller validiert API-Access
(Controller, Validates, UI),         // Controller validiert UI-Access
(Controller, DependsOn, Realm),      // Controller-Scope ist per-Realm
(Controller, DependsOn, Room),       // Controller-Scope ist per-Room
(Controller, DependsOn, Partition),  // Controller-Scope ist per-Partition
(Controller, Aggregates, Governance), // Controller trackt Governance-Delegations
(Controller, DependsOn, ECLVM),      // Permission-Rules in ECLVM

// ══════════════════════════════════════════════════════════════════════════════
// BLUEPRINTCOMPOSER-ENGINE BEZIEHUNGEN
// ══════════════════════════════════════════════════════════════════════════════
(BlueprintComposer, DependsOn, Blueprint),  // Composer nutzt Blueprint-Storage
(BlueprintComposer, Aggregates, ECLBlueprint), // Composer aggregiert Blueprint-Instanzen
(BlueprintComposer, Triggers, Event),       // Composition erzeugt Events
(BlueprintComposer, DependsOn, ECLVM),      // Composition läuft in ECLVM
(BlueprintComposer, DependsOn, Trust),      // Blueprint-Publish prüft Trust
(BlueprintComposer, Validates, Realm),      // Composer validiert Realm-Compatibility
(BlueprintComposer, DependsOn, Gas),        // Composition verbraucht Gas
```

---

### 13.4 Neue State-Structs

#### 13.4.1 UIState

**Datei:** `backend/src/core/state.rs`
**Position:** Nach `P2PState` (ca. Zeile 3900)

````rust
// ============================================================================
// UI-ENGINE STATE LAYER
// ============================================================================

/// UI-Engine State mit Component-Tree und Binding-Tracking
///
/// # Design
///
/// Die UI-Engine verwaltet deklarative, Trust-basierte Interfaces:
/// - **Component-Tree**: Hierarchischer UI-Aufbau
/// - **Bindings**: Reaktive Daten-Verbindungen
/// - **Trust-Gates**: Sichtbarkeit basierend auf Trust
/// - **Render-Cache**: Optimierte Re-Renders
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// UI ──DependsOn──▶ Trust (Sichtbarkeit)
/// UI ──DependsOn──▶ Realm (Isolation)
/// UI ──DependsOn──▶ Room (Scoping)
/// UI ──DependsOn──▶ Controller (Permissions)
/// UI ──Triggers───▶ Event (UI-Actions)
/// UI ──Aggregates─▶ DataLogic (Bindings)
/// ```
#[derive(Debug)]
pub struct UIState {
    // ─────────────────────────────────────────────────────────────────────────
    // Component-Tree Metriken
    // ─────────────────────────────────────────────────────────────────────────
    /// Total registrierte UI-Components
    pub components_registered: AtomicU64,
    /// Aktuell aktive Components
    pub components_active: AtomicU64,
    /// Component-Updates durchgeführt
    pub component_updates: AtomicU64,
    /// Component-Renders durchgeführt
    pub renders: AtomicU64,
    /// Cached Renders (keine Änderung)
    pub cache_hits: AtomicU64,
    /// Re-Renders (State-Änderung)
    pub cache_misses: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Binding-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Aktive Bindings
    pub bindings_active: AtomicU64,
    /// Binding-Updates propagiert
    pub binding_updates: AtomicU64,
    /// Binding-Fehler
    pub binding_errors: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Trust-Gates
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Gate Evaluationen
    pub trust_gate_evaluations: AtomicU64,
    /// Trust-Gate Allowed
    pub trust_gate_allowed: AtomicU64,
    /// Trust-Gate Denied
    pub trust_gate_denied: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Credential-Gates
    // ─────────────────────────────────────────────────────────────────────────
    /// Credential-Gate Evaluationen
    pub credential_gate_evaluations: AtomicU64,
    /// Credential-Gate Allowed
    pub credential_gate_allowed: AtomicU64,
    /// Credential-Gate Denied
    pub credential_gate_denied: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Per-Realm UI-State
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-spezifische UI-Metriken
    pub realm_ui: RwLock<HashMap<String, RealmUIState>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Resource-Verbrauch
    // ─────────────────────────────────────────────────────────────────────────
    /// Gas verbraucht für UI-Rendering
    pub gas_consumed: AtomicU64,
    /// Mana verbraucht für UI-Events
    pub mana_consumed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (UI ← Trust)
    pub trust_dependency_updates: AtomicU64,
    /// DataLogic-Aggregations (UI ⊃ DataLogic)
    pub datalogic_aggregations: AtomicU64,
    /// Controller-Validations (Controller ✓ UI)
    pub controller_validations: AtomicU64,
    /// Events getriggert (UI → Event)
    pub events_triggered: AtomicU64,
}

/// Per-Realm UI-State
#[derive(Debug)]
pub struct RealmUIState {
    pub realm_id: String,
    pub components: AtomicU64,
    pub renders: AtomicU64,
    pub bindings: AtomicU64,
}

impl UIState {
    pub fn new() -> Self {
        Self {
            components_registered: AtomicU64::new(0),
            components_active: AtomicU64::new(0),
            component_updates: AtomicU64::new(0),
            renders: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            bindings_active: AtomicU64::new(0),
            binding_updates: AtomicU64::new(0),
            binding_errors: AtomicU64::new(0),
            trust_gate_evaluations: AtomicU64::new(0),
            trust_gate_allowed: AtomicU64::new(0),
            trust_gate_denied: AtomicU64::new(0),
            credential_gate_evaluations: AtomicU64::new(0),
            credential_gate_allowed: AtomicU64::new(0),
            credential_gate_denied: AtomicU64::new(0),
            realm_ui: RwLock::new(HashMap::new()),
            gas_consumed: AtomicU64::new(0),
            mana_consumed: AtomicU64::new(0),
            trust_dependency_updates: AtomicU64::new(0),
            datalogic_aggregations: AtomicU64::new(0),
            controller_validations: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
        }
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits.load(Ordering::Relaxed) + self.cache_misses.load(Ordering::Relaxed);
        if total > 0 {
            self.cache_hits.load(Ordering::Relaxed) as f64 / total as f64
        } else {
            1.0
        }
    }

    pub fn trust_gate_allow_rate(&self) -> f64 {
        let total = self.trust_gate_evaluations.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.trust_gate_allowed.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> UIStateSnapshot {
        UIStateSnapshot {
            components_registered: self.components_registered.load(Ordering::Relaxed),
            components_active: self.components_active.load(Ordering::Relaxed),
            component_updates: self.component_updates.load(Ordering::Relaxed),
            renders: self.renders.load(Ordering::Relaxed),
            cache_hit_rate: self.cache_hit_rate(),
            bindings_active: self.bindings_active.load(Ordering::Relaxed),
            binding_updates: self.binding_updates.load(Ordering::Relaxed),
            trust_gate_evaluations: self.trust_gate_evaluations.load(Ordering::Relaxed),
            trust_gate_allow_rate: self.trust_gate_allow_rate(),
            gas_consumed: self.gas_consumed.load(Ordering::Relaxed),
            mana_consumed: self.mana_consumed.load(Ordering::Relaxed),
            events_triggered: self.events_triggered.load(Ordering::Relaxed),
        }
    }
}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIStateSnapshot {
    pub components_registered: u64,
    pub components_active: u64,
    pub component_updates: u64,
    pub renders: u64,
    pub cache_hit_rate: f64,
    pub bindings_active: u64,
    pub binding_updates: u64,
    pub trust_gate_evaluations: u64,
    pub trust_gate_allow_rate: f64,
    pub gas_consumed: u64,
    pub mana_consumed: u64,
    pub events_triggered: u64,
}
````

#### 13.4.2 APIState

````rust
// ============================================================================
// API-ENGINE STATE LAYER
// ============================================================================

/// API-Engine State mit Endpoint-Registry und Rate-Limiting
///
/// # Design
///
/// Die API-Engine ermöglicht dynamische REST-API-Definition per ECL:
/// - **Endpoint-Registry**: Routing-Tabelle per Realm
/// - **Rate-Limits**: Trust-basierte Throttling
/// - **Metrics**: Request/Response-Tracking
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// API ──DependsOn──▶ Trust (Access-Control)
/// API ──DependsOn──▶ Controller (AuthZ)
/// API ──Validates──▶ Gateway (External)
/// API ──Triggers───▶ Event (API-Calls)
/// API ──Aggregates─▶ DataLogic (Queries)
/// ```
#[derive(Debug)]
pub struct APIState {
    // ─────────────────────────────────────────────────────────────────────────
    // Endpoint-Registry
    // ─────────────────────────────────────────────────────────────────────────
    /// Registrierte Endpoints total
    pub endpoints_registered: AtomicU64,
    /// Aktive Endpoints
    pub endpoints_active: AtomicU64,
    /// Endpoint-Updates
    pub endpoint_updates: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Request-Metriken
    // ─────────────────────────────────────────────────────────────────────────
    /// Total Requests
    pub requests_total: AtomicU64,
    /// Erfolgreiche Requests (2xx)
    pub requests_success: AtomicU64,
    /// Client-Errors (4xx)
    pub requests_client_error: AtomicU64,
    /// Server-Errors (5xx)
    pub requests_server_error: AtomicU64,
    /// Rate-Limited Requests (429)
    pub requests_rate_limited: AtomicU64,
    /// Auth-Failed Requests (401/403)
    pub requests_auth_failed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Latenz-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Durchschnittliche Latenz (µs)
    pub avg_latency_us: RwLock<f64>,
    /// P95 Latenz (µs)
    pub p95_latency_us: RwLock<f64>,
    /// P99 Latenz (µs)
    pub p99_latency_us: RwLock<f64>,
    /// Latenz-Historie
    pub latency_history: RwLock<Vec<u64>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Rate-Limiting
    // ─────────────────────────────────────────────────────────────────────────
    /// Aktive Rate-Limit-Buckets
    pub rate_limit_buckets: AtomicU64,
    /// Rate-Limit-Resets
    pub rate_limit_resets: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Per-Realm API-State
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-spezifische API-Metriken
    pub realm_api: RwLock<HashMap<String, RealmAPIState>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Resource-Verbrauch
    // ─────────────────────────────────────────────────────────────────────────
    /// Gas verbraucht für API-Processing
    pub gas_consumed: AtomicU64,
    /// Mana verbraucht für Responses
    pub mana_consumed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (API ← Trust)
    pub trust_dependency_updates: AtomicU64,
    /// Controller-Validations (Controller ✓ API)
    pub controller_validations: AtomicU64,
    /// Gateway-Validations (API ✓ Gateway)
    pub gateway_validations: AtomicU64,
    /// Events getriggert (API → Event)
    pub events_triggered: AtomicU64,
}

/// Per-Realm API-State
#[derive(Debug)]
pub struct RealmAPIState {
    pub realm_id: String,
    pub endpoints: AtomicU64,
    pub requests: AtomicU64,
    pub rate_limited: AtomicU64,
}

impl APIState {
    pub fn new() -> Self {
        Self {
            endpoints_registered: AtomicU64::new(0),
            endpoints_active: AtomicU64::new(0),
            endpoint_updates: AtomicU64::new(0),
            requests_total: AtomicU64::new(0),
            requests_success: AtomicU64::new(0),
            requests_client_error: AtomicU64::new(0),
            requests_server_error: AtomicU64::new(0),
            requests_rate_limited: AtomicU64::new(0),
            requests_auth_failed: AtomicU64::new(0),
            avg_latency_us: RwLock::new(0.0),
            p95_latency_us: RwLock::new(0.0),
            p99_latency_us: RwLock::new(0.0),
            latency_history: RwLock::new(Vec::with_capacity(1000)),
            rate_limit_buckets: AtomicU64::new(0),
            rate_limit_resets: AtomicU64::new(0),
            realm_api: RwLock::new(HashMap::new()),
            gas_consumed: AtomicU64::new(0),
            mana_consumed: AtomicU64::new(0),
            trust_dependency_updates: AtomicU64::new(0),
            controller_validations: AtomicU64::new(0),
            gateway_validations: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
        }
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.requests_total.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.requests_success.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn record_request(&self, latency_us: u64, success: bool, status_category: u16) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);

        match status_category {
            200..=299 => { self.requests_success.fetch_add(1, Ordering::Relaxed); }
            400..=499 => {
                self.requests_client_error.fetch_add(1, Ordering::Relaxed);
                if status_category == 429 {
                    self.requests_rate_limited.fetch_add(1, Ordering::Relaxed);
                } else if status_category == 401 || status_category == 403 {
                    self.requests_auth_failed.fetch_add(1, Ordering::Relaxed);
                }
            }
            500..=599 => { self.requests_server_error.fetch_add(1, Ordering::Relaxed); }
            _ => {}
        }

        // Update latency tracking
        if let Ok(mut history) = self.latency_history.write() {
            history.push(latency_us);
            if history.len() > 1000 {
                history.remove(0);
            }

            // Update averages
            let avg = history.iter().sum::<u64>() as f64 / history.len() as f64;
            if let Ok(mut a) = self.avg_latency_us.write() {
                *a = avg;
            }

            // Calculate percentiles
            let mut sorted = history.clone();
            sorted.sort_unstable();
            let p95_idx = (sorted.len() as f64 * 0.95) as usize;
            let p99_idx = (sorted.len() as f64 * 0.99) as usize;

            if let Ok(mut p95) = self.p95_latency_us.write() {
                *p95 = sorted.get(p95_idx.min(sorted.len() - 1)).copied().unwrap_or(0) as f64;
            }
            if let Ok(mut p99) = self.p99_latency_us.write() {
                *p99 = sorted.get(p99_idx.min(sorted.len() - 1)).copied().unwrap_or(0) as f64;
            }
        }
    }

    pub fn snapshot(&self) -> APIStateSnapshot {
        APIStateSnapshot {
            endpoints_registered: self.endpoints_registered.load(Ordering::Relaxed),
            endpoints_active: self.endpoints_active.load(Ordering::Relaxed),
            requests_total: self.requests_total.load(Ordering::Relaxed),
            requests_success: self.requests_success.load(Ordering::Relaxed),
            success_rate: self.success_rate(),
            requests_rate_limited: self.requests_rate_limited.load(Ordering::Relaxed),
            requests_auth_failed: self.requests_auth_failed.load(Ordering::Relaxed),
            avg_latency_us: self.avg_latency_us.read().map(|v| *v).unwrap_or(0.0),
            p95_latency_us: self.p95_latency_us.read().map(|v| *v).unwrap_or(0.0),
            p99_latency_us: self.p99_latency_us.read().map(|v| *v).unwrap_or(0.0),
            gas_consumed: self.gas_consumed.load(Ordering::Relaxed),
            mana_consumed: self.mana_consumed.load(Ordering::Relaxed),
            events_triggered: self.events_triggered.load(Ordering::Relaxed),
        }
    }
}

impl Default for APIState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIStateSnapshot {
    pub endpoints_registered: u64,
    pub endpoints_active: u64,
    pub requests_total: u64,
    pub requests_success: u64,
    pub success_rate: f64,
    pub requests_rate_limited: u64,
    pub requests_auth_failed: u64,
    pub avg_latency_us: f64,
    pub p95_latency_us: f64,
    pub p99_latency_us: f64,
    pub gas_consumed: u64,
    pub mana_consumed: u64,
    pub events_triggered: u64,
}
````

#### 13.4.3 GovernanceState

````rust
// ============================================================================
// GOVERNANCE-ENGINE STATE LAYER
// ============================================================================

/// Governance-Engine State mit Proposal-Tracking und Delegation-Graph
///
/// # Design
///
/// Die Governance-Engine implementiert DAO-Prinzipien:
/// - **Quadratic Voting**: √-basierte Stimmgewichtung (Κ21)
/// - **Delegation**: Transitive Trust-Delegation
/// - **Anti-Calcification**: Machtkonzentrations-Check (Κ19)
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// Governance ──DependsOn──▶ Trust (Voting-Power)
/// Governance ──DependsOn──▶ Quadratic (Voting-Mechanik)
/// Governance ──Validates──▶ Controller (Permission-Changes)
/// Governance ──Triggers───▶ Controller (Vote-Results)
/// Governance ──Validates──▶ AntiCalcification (Power-Check)
/// ```
#[derive(Debug)]
pub struct GovernanceState {
    // ─────────────────────────────────────────────────────────────────────────
    // Proposal-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Total erstellte Proposals
    pub proposals_created: AtomicU64,
    /// Aktive Proposals
    pub proposals_active: AtomicU64,
    /// Abgeschlossene Proposals
    pub proposals_completed: AtomicU64,
    /// Angenommene Proposals
    pub proposals_accepted: AtomicU64,
    /// Abgelehnte Proposals
    pub proposals_rejected: AtomicU64,
    /// Abgebrochene Proposals (Quorum nicht erreicht)
    pub proposals_expired: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Voting-Metriken
    // ─────────────────────────────────────────────────────────────────────────
    /// Total abgegebene Votes
    pub votes_cast: AtomicU64,
    /// Unique Voters
    pub unique_voters: AtomicU64,
    /// Delegierte Votes
    pub votes_delegated: AtomicU64,
    /// Quadratische Reduktionen angewendet
    pub quadratic_reductions: AtomicU64,
    /// Durchschnittliche Voting-Power
    pub avg_voting_power: RwLock<f64>,

    // ─────────────────────────────────────────────────────────────────────────
    // Delegation-Graph
    // ─────────────────────────────────────────────────────────────────────────
    /// Aktive Delegationen
    pub delegations_active: AtomicU64,
    /// Delegations-Ketten-Tiefe (max)
    pub max_delegation_depth: AtomicU64,
    /// Zirkuläre Delegationen verhindert
    pub circular_delegations_prevented: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Anti-Calcification
    // ─────────────────────────────────────────────────────────────────────────
    /// Power-Concentration-Checks
    pub power_checks: AtomicU64,
    /// Power-Concentration-Violations
    pub power_violations: AtomicU64,
    /// Gini-Koeffizient der Voting-Power
    pub voting_power_gini: RwLock<f64>,

    // ─────────────────────────────────────────────────────────────────────────
    // Per-Realm Governance-State
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-spezifische Governance-Metriken
    pub realm_governance: RwLock<HashMap<String, RealmGovernanceState>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (Governance ← Trust)
    pub trust_dependency_updates: AtomicU64,
    /// Quadratic-Validations (Governance ← Quadratic)
    pub quadratic_validations: AtomicU64,
    /// Controller-Triggers (Governance → Controller)
    pub controller_triggers: AtomicU64,
    /// AntiCalc-Validations (Governance ✓ AntiCalcification)
    pub anticalc_validations: AtomicU64,
    /// Events getriggert (Governance → Event)
    pub events_triggered: AtomicU64,
}

/// Per-Realm Governance-State
#[derive(Debug)]
pub struct RealmGovernanceState {
    pub realm_id: String,
    pub proposals: AtomicU64,
    pub votes: AtomicU64,
    pub delegations: AtomicU64,
    pub governance_type: RwLock<String>, // "council", "direct", "liquid", "quadratic"
}

impl GovernanceState {
    pub fn new() -> Self {
        Self {
            proposals_created: AtomicU64::new(0),
            proposals_active: AtomicU64::new(0),
            proposals_completed: AtomicU64::new(0),
            proposals_accepted: AtomicU64::new(0),
            proposals_rejected: AtomicU64::new(0),
            proposals_expired: AtomicU64::new(0),
            votes_cast: AtomicU64::new(0),
            unique_voters: AtomicU64::new(0),
            votes_delegated: AtomicU64::new(0),
            quadratic_reductions: AtomicU64::new(0),
            avg_voting_power: RwLock::new(1.0),
            delegations_active: AtomicU64::new(0),
            max_delegation_depth: AtomicU64::new(0),
            circular_delegations_prevented: AtomicU64::new(0),
            power_checks: AtomicU64::new(0),
            power_violations: AtomicU64::new(0),
            voting_power_gini: RwLock::new(0.0),
            realm_governance: RwLock::new(HashMap::new()),
            trust_dependency_updates: AtomicU64::new(0),
            quadratic_validations: AtomicU64::new(0),
            controller_triggers: AtomicU64::new(0),
            anticalc_validations: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
        }
    }

    pub fn proposal_success_rate(&self) -> f64 {
        let completed = self.proposals_completed.load(Ordering::Relaxed) as f64;
        if completed > 0.0 {
            self.proposals_accepted.load(Ordering::Relaxed) as f64 / completed
        } else {
            0.0
        }
    }

    pub fn delegation_rate(&self) -> f64 {
        let total = self.votes_cast.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.votes_delegated.load(Ordering::Relaxed) as f64 / total
        } else {
            0.0
        }
    }

    pub fn snapshot(&self) -> GovernanceStateSnapshot {
        GovernanceStateSnapshot {
            proposals_created: self.proposals_created.load(Ordering::Relaxed),
            proposals_active: self.proposals_active.load(Ordering::Relaxed),
            proposals_completed: self.proposals_completed.load(Ordering::Relaxed),
            proposal_success_rate: self.proposal_success_rate(),
            votes_cast: self.votes_cast.load(Ordering::Relaxed),
            unique_voters: self.unique_voters.load(Ordering::Relaxed),
            delegation_rate: self.delegation_rate(),
            delegations_active: self.delegations_active.load(Ordering::Relaxed),
            max_delegation_depth: self.max_delegation_depth.load(Ordering::Relaxed),
            quadratic_reductions: self.quadratic_reductions.load(Ordering::Relaxed),
            voting_power_gini: self.voting_power_gini.read().map(|v| *v).unwrap_or(0.0),
            power_violations: self.power_violations.load(Ordering::Relaxed),
            events_triggered: self.events_triggered.load(Ordering::Relaxed),
        }
    }
}

impl Default for GovernanceState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceStateSnapshot {
    pub proposals_created: u64,
    pub proposals_active: u64,
    pub proposals_completed: u64,
    pub proposal_success_rate: f64,
    pub votes_cast: u64,
    pub unique_voters: u64,
    pub delegation_rate: f64,
    pub delegations_active: u64,
    pub max_delegation_depth: u64,
    pub quadratic_reductions: u64,
    pub voting_power_gini: f64,
    pub power_violations: u64,
    pub events_triggered: u64,
}
````

#### 13.4.4 ControllerState

````rust
// ============================================================================
// CONTROLLER-ENGINE STATE LAYER
// ============================================================================

/// Controller-Engine State mit Permission-Registry und Audit-Log
///
/// # Design
///
/// Die Controller-Engine verwaltet Berechtigungen:
/// - **Scoped Permissions**: Realm > Room > Partition
/// - **Delegation**: Transitive Permission-Vererbung
/// - **Audit-Trail**: Vollständige Permission-History
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// Controller ──DependsOn──▶ Trust (Permission-Basis)
/// Controller ──Validates──▶ Gateway (Crossing-Auth)
/// Controller ──Validates──▶ API (API-Auth)
/// Controller ──Validates──▶ UI (UI-Auth)
/// Controller ──Aggregates─▶ Governance (Delegation-Sync)
/// ```
#[derive(Debug)]
pub struct ControllerState {
    // ─────────────────────────────────────────────────────────────────────────
    // Permission-Registry
    // ─────────────────────────────────────────────────────────────────────────
    /// Total registrierte Permissions
    pub permissions_registered: AtomicU64,
    /// Aktive Permissions
    pub permissions_active: AtomicU64,
    /// Permission-Grants
    pub permission_grants: AtomicU64,
    /// Permission-Revokes
    pub permission_revokes: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Authorization-Checks
    // ─────────────────────────────────────────────────────────────────────────
    /// AuthZ-Checks total
    pub authz_checks: AtomicU64,
    /// AuthZ-Allowed
    pub authz_allowed: AtomicU64,
    /// AuthZ-Denied
    pub authz_denied: AtomicU64,
    /// Durchschnittliche Check-Latenz (µs)
    pub avg_check_latency_us: RwLock<f64>,

    // ─────────────────────────────────────────────────────────────────────────
    // Delegation
    // ─────────────────────────────────────────────────────────────────────────
    /// Aktive Delegationen
    pub delegations_active: AtomicU64,
    /// Delegations-Ketten (max depth)
    pub max_delegation_depth: AtomicU64,
    /// Delegations-Nutzungen
    pub delegations_used: AtomicU64,
    /// Abgelaufene Delegationen
    pub delegations_expired: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Scope-Tracking (Realm > Room > Partition)
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-Scope Checks
    pub realm_scope_checks: AtomicU64,
    /// Room-Scope Checks
    pub room_scope_checks: AtomicU64,
    /// Partition-Scope Checks
    pub partition_scope_checks: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Audit-Log
    // ─────────────────────────────────────────────────────────────────────────
    /// Audit-Entries geschrieben
    pub audit_entries: AtomicU64,
    /// Audit-Log-Größe (Bytes)
    pub audit_log_bytes: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Per-Realm Controller-State
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-spezifische Controller-Metriken
    pub realm_controller: RwLock<HashMap<String, RealmControllerState>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (Controller ← Trust)
    pub trust_dependency_updates: AtomicU64,
    /// Gateway-Validations (Controller ✓ Gateway)
    pub gateway_validations: AtomicU64,
    /// API-Validations (Controller ✓ API)
    pub api_validations: AtomicU64,
    /// UI-Validations (Controller ✓ UI)
    pub ui_validations: AtomicU64,
    /// Governance-Aggregations (Controller ⊃ Governance)
    pub governance_aggregations: AtomicU64,
    /// Events getriggert (Controller → Event)
    pub events_triggered: AtomicU64,
}

/// Per-Realm Controller-State
#[derive(Debug)]
pub struct RealmControllerState {
    pub realm_id: String,
    pub permissions: AtomicU64,
    pub authz_checks: AtomicU64,
    pub delegations: AtomicU64,
    pub rooms: AtomicU64,
    pub partitions: AtomicU64,
}

impl ControllerState {
    pub fn new() -> Self {
        Self {
            permissions_registered: AtomicU64::new(0),
            permissions_active: AtomicU64::new(0),
            permission_grants: AtomicU64::new(0),
            permission_revokes: AtomicU64::new(0),
            authz_checks: AtomicU64::new(0),
            authz_allowed: AtomicU64::new(0),
            authz_denied: AtomicU64::new(0),
            avg_check_latency_us: RwLock::new(0.0),
            delegations_active: AtomicU64::new(0),
            max_delegation_depth: AtomicU64::new(0),
            delegations_used: AtomicU64::new(0),
            delegations_expired: AtomicU64::new(0),
            realm_scope_checks: AtomicU64::new(0),
            room_scope_checks: AtomicU64::new(0),
            partition_scope_checks: AtomicU64::new(0),
            audit_entries: AtomicU64::new(0),
            audit_log_bytes: AtomicU64::new(0),
            realm_controller: RwLock::new(HashMap::new()),
            trust_dependency_updates: AtomicU64::new(0),
            gateway_validations: AtomicU64::new(0),
            api_validations: AtomicU64::new(0),
            ui_validations: AtomicU64::new(0),
            governance_aggregations: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
        }
    }

    pub fn authz_success_rate(&self) -> f64 {
        let total = self.authz_checks.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.authz_allowed.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn check_authorization(&self, allowed: bool, latency_us: u64) {
        self.authz_checks.fetch_add(1, Ordering::Relaxed);
        if allowed {
            self.authz_allowed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.authz_denied.fetch_add(1, Ordering::Relaxed);
        }

        // Update latency
        if let Ok(mut avg) = self.avg_check_latency_us.write() {
            let total = self.authz_checks.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + latency_us as f64) / total;
        }
    }

    pub fn snapshot(&self) -> ControllerStateSnapshot {
        ControllerStateSnapshot {
            permissions_registered: self.permissions_registered.load(Ordering::Relaxed),
            permissions_active: self.permissions_active.load(Ordering::Relaxed),
            permission_grants: self.permission_grants.load(Ordering::Relaxed),
            permission_revokes: self.permission_revokes.load(Ordering::Relaxed),
            authz_checks: self.authz_checks.load(Ordering::Relaxed),
            authz_success_rate: self.authz_success_rate(),
            avg_check_latency_us: self.avg_check_latency_us.read().map(|v| *v).unwrap_or(0.0),
            delegations_active: self.delegations_active.load(Ordering::Relaxed),
            max_delegation_depth: self.max_delegation_depth.load(Ordering::Relaxed),
            realm_scope_checks: self.realm_scope_checks.load(Ordering::Relaxed),
            room_scope_checks: self.room_scope_checks.load(Ordering::Relaxed),
            partition_scope_checks: self.partition_scope_checks.load(Ordering::Relaxed),
            audit_entries: self.audit_entries.load(Ordering::Relaxed),
            events_triggered: self.events_triggered.load(Ordering::Relaxed),
        }
    }
}

impl Default for ControllerState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerStateSnapshot {
    pub permissions_registered: u64,
    pub permissions_active: u64,
    pub permission_grants: u64,
    pub permission_revokes: u64,
    pub authz_checks: u64,
    pub authz_success_rate: f64,
    pub avg_check_latency_us: f64,
    pub delegations_active: u64,
    pub max_delegation_depth: u64,
    pub realm_scope_checks: u64,
    pub room_scope_checks: u64,
    pub partition_scope_checks: u64,
    pub audit_entries: u64,
    pub events_triggered: u64,
}
````

---

### 13.5 UnifiedState Erweiterung

**Datei:** `backend/src/core/state.rs`
**Struct:** `UnifiedState`
**Zeile:** ~4045

#### 13.5.1 SOLL-Zustand

```rust
pub struct UnifiedState {
    /// Startzeit
    pub started_at: Instant,

    // ═══════════════════════════════════════════════════════════════════════════
    // BESTEHENDE LAYER (unverändert)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Core Logic Layer (Κ2-Κ18)
    pub core: CoreState,

    /// Execution Layer (IPS ℳ)
    pub execution: ExecutionState,

    /// ECLVM Layer (Erynoa Core Language Virtual Machine)
    pub eclvm: ECLVMState,

    /// Protection Layer (Κ19-Κ21)
    pub protection: ProtectionState,

    /// Storage Layer
    pub storage: StorageState,

    /// Peer Layer (Κ22-Κ24)
    pub peer: PeerState,

    /// P2P Network Layer
    pub p2p: P2PState,

    // ═══════════════════════════════════════════════════════════════════════════
    // NEU: ENGINE-LAYER (6 neue Engines)
    // ═══════════════════════════════════════════════════════════════════════════

    /// UI-Engine State: Deklaratives, Trust-basiertes Interface-Rendering
    pub ui: UIState,

    /// API-Engine State: Dynamische REST-API-Definition per ECL
    pub api: APIState,

    /// Governance-Engine State: DAO-Prinzipien und Abstimmungsmechanismen
    pub governance: GovernanceState,

    /// Controller-Engine State: Berechtigungsverwaltung mit Delegation
    pub controller: ControllerState,

    /// DataLogic-Engine State: Reaktive Event-Verarbeitung
    pub datalogic: DataLogicState,

    /// BlueprintComposer-Engine State: Template-Komposition
    pub blueprint_composer: BlueprintComposerState,

    // ═══════════════════════════════════════════════════════════════════════════
    // META-STATE
    // ═══════════════════════════════════════════════════════════════════════════

    /// State-Beziehungs-Graph
    pub graph: StateGraph,

    /// Aktive Warnings
    pub warnings: RwLock<Vec<String>>,

    /// Global Health Score (cached)
    pub health_score: RwLock<f64>,
}
```

#### 13.5.2 UnifiedState::new() Anpassung

```rust
impl UnifiedState {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
            core: CoreState::new(),
            execution: ExecutionState::new(),
            eclvm: ECLVMState::new(),
            protection: ProtectionState::new(),
            storage: StorageState::new(),
            peer: PeerState::new(),
            p2p: P2PState::new(),
            // NEU: Engine-Layer
            ui: UIState::new(),
            api: APIState::new(),
            governance: GovernanceState::new(),
            controller: ControllerState::new(),
            datalogic: DataLogicState::new(),
            blueprint_composer: BlueprintComposerState::new(),
            // Meta
            graph: StateGraph::erynoa_graph(),
            warnings: RwLock::new(Vec::new()),
            health_score: RwLock::new(100.0),
        }
    }
}
```

#### 13.5.3 calculate_health() Erweiterung

```rust
pub fn calculate_health(&self) -> f64 {
    let mut score: f64 = 100.0;

    // === Bestehende Health-Faktoren (unverändert) ===
    score -= (100.0 - self.protection.health_score()) * 0.15;
    score -= (1.0 - self.core.consensus.success_rate()) * 10.0;
    score -= (1.0 - self.execution.success_rate()) * 8.0;
    score -= (1.0 - self.eclvm.policy_success_rate()) * 8.0;
    score -= (100.0 - self.p2p.health_score()) * 0.15;

    // === NEU: Engine-Layer Health (25% Gewicht) ===

    // UI-Engine (5% Gewicht)
    score -= (1.0 - self.ui.cache_hit_rate()) * 2.5;
    score -= (1.0 - self.ui.trust_gate_allow_rate()) * 2.5;

    // API-Engine (5% Gewicht)
    score -= (1.0 - self.api.success_rate()) * 5.0;

    // Governance-Engine (5% Gewicht)
    let gov_health = if self.governance.power_violations.load(Ordering::Relaxed) > 0 {
        0.0  // Power-Violations sind kritisch
    } else {
        1.0
    };
    score -= (1.0 - gov_health) * 5.0;

    // Controller-Engine (5% Gewicht)
    score -= (1.0 - self.controller.authz_success_rate()) * 5.0;

    // DataLogic-Engine (2.5% Gewicht)
    score -= (1.0 - self.datalogic.success_rate()) * 2.5;

    // BlueprintComposer-Engine (2.5% Gewicht)
    score -= (1.0 - self.blueprint_composer.composition_success_rate()) * 2.5;

    let final_score = score.max(0.0).min(100.0);
    if let Ok(mut cached) = self.health_score.write() {
        *cached = final_score;
    }
    final_score
}
```

---

### 13.6 Observer-Trait Erweiterungen

**Datei:** `backend/src/core/state_integration.rs`

#### 13.6.1 Neue Observer-Traits

```rust
// ============================================================================
// UI-ENGINE OBSERVER
// ============================================================================

/// UI-Engine Observer - Tracks UI-Component und Binding-Events
pub trait UIObserver: Send + Sync {
    /// Component registriert
    fn on_component_registered(&self, component_id: &str, component_type: &str, realm_id: &str);

    /// Component gerendert
    fn on_component_rendered(&self, component_id: &str, from_cache: bool, gas_used: u64);

    /// Binding aktualisiert
    fn on_binding_updated(&self, binding_id: &str, source: &str, target: &str);

    /// Trust-Gate evaluiert
    fn on_trust_gate_evaluated(
        &self,
        component_id: &str,
        entity: &EntityId,
        required_trust: f64,
        actual_trust: f64,
        allowed: bool
    );

    /// Credential-Gate evaluiert
    fn on_credential_gate_evaluated(
        &self,
        component_id: &str,
        entity: &EntityId,
        credential_type: &str,
        allowed: bool
    );

    /// UI-Action ausgelöst
    fn on_ui_action(&self, component_id: &str, action_type: &str, realm_id: &str);
}

pub type SharedUIObserver = Arc<dyn UIObserver>;

// ============================================================================
// API-ENGINE OBSERVER
// ============================================================================

/// API-Engine Observer - Tracks API-Requests und Rate-Limiting
pub trait APIObserver: Send + Sync {
    /// Endpoint registriert
    fn on_endpoint_registered(&self, endpoint_path: &str, method: &str, realm_id: &str);

    /// Request empfangen
    fn on_request_received(
        &self,
        endpoint_path: &str,
        method: &str,
        realm_id: &str,
        entity: Option<&EntityId>
    );

    /// Response gesendet
    fn on_response_sent(
        &self,
        endpoint_path: &str,
        status: u16,
        latency_us: u64,
        gas_used: u64,
        mana_used: u64
    );

    /// Rate-Limited
    fn on_rate_limited(&self, endpoint_path: &str, entity: &EntityId, limit_type: &str);

    /// Auth-Failed
    fn on_auth_failed(&self, endpoint_path: &str, entity: Option<&EntityId>, reason: &str);
}

pub type SharedAPIObserver = Arc<dyn APIObserver>;

// ============================================================================
// GOVERNANCE-ENGINE OBSERVER
// ============================================================================

/// Governance-Engine Observer - Tracks Proposals, Votes und Delegations
pub trait GovernanceObserver: Send + Sync {
    /// Proposal erstellt
    fn on_proposal_created(
        &self,
        proposal_id: &str,
        proposal_type: &str,
        realm_id: &str,
        author: &EntityId,
        quorum_required: f64
    );

    /// Vote abgegeben
    fn on_vote_cast(
        &self,
        proposal_id: &str,
        voter: &EntityId,
        choice: &str,  // "yes", "no", "abstain"
        voting_power: f64,
        is_delegated: bool
    );

    /// Proposal abgeschlossen
    fn on_proposal_completed(
        &self,
        proposal_id: &str,
        result: &str,  // "accepted", "rejected", "expired"
        yes_votes: f64,
        no_votes: f64,
        participation_rate: f64
    );

    /// Delegation erstellt
    fn on_delegation_created(
        &self,
        delegator: &EntityId,
        delegate: &EntityId,
        scope: &str,  // "realm", "room", "partition", "proposal"
        expires_at: Option<u64>
    );

    /// Delegation widerrufen
    fn on_delegation_revoked(&self, delegator: &EntityId, delegate: &EntityId, reason: &str);

    /// Quadratic-Reduktion angewendet
    fn on_quadratic_reduction(
        &self,
        voter: &EntityId,
        original_power: f64,
        reduced_power: f64
    );

    /// Power-Violation erkannt
    fn on_power_violation(
        &self,
        entity: &EntityId,
        concentration: f64,
        threshold: f64
    );
}

pub type SharedGovernanceObserver = Arc<dyn GovernanceObserver>;

// ============================================================================
// CONTROLLER-ENGINE OBSERVER
// ============================================================================

/// Controller-Engine Observer - Tracks Permissions und AuthZ-Decisions
pub trait ControllerObserver: Send + Sync {
    /// Permission gewährt
    fn on_permission_granted(
        &self,
        entity: &EntityId,
        permission: &str,
        scope: &str,  // "realm:xxx", "room:xxx", "partition:xxx"
        granted_by: &EntityId
    );

    /// Permission widerrufen
    fn on_permission_revoked(
        &self,
        entity: &EntityId,
        permission: &str,
        scope: &str,
        revoked_by: &EntityId,
        reason: &str
    );

    /// AuthZ-Check durchgeführt
    fn on_authz_check(
        &self,
        entity: &EntityId,
        permission: &str,
        resource: &str,
        allowed: bool,
        latency_us: u64,
        via_delegation: bool
    );

    /// Delegation erstellt (Controller-Scope)
    fn on_controller_delegation(
        &self,
        delegator: &EntityId,
        delegate: &EntityId,
        permissions: &[String],
        scope: &str,
        expires_at: Option<u64>
    );

    /// Audit-Entry geschrieben
    fn on_audit_entry(
        &self,
        entry_type: &str,  // "grant", "revoke", "check", "delegation"
        entity: &EntityId,
        details: &str
    );

    /// Scope-Check durchgeführt
    fn on_scope_check(
        &self,
        entity: &EntityId,
        scope_type: &str,  // "realm", "room", "partition"
        scope_id: &str,
        allowed: bool
    );
}

pub type SharedControllerObserver = Arc<dyn ControllerObserver>;

// ============================================================================
// DATALOGIC-ENGINE OBSERVER
// ============================================================================

/// DataLogic-Engine Observer - Tracks Reactive Streams und Aggregations
pub trait DataLogicObserver: Send + Sync {
    /// Stream registriert
    fn on_stream_registered(&self, stream_id: &str, source_event_type: &str, realm_id: &str);

    /// Aggregation definiert
    fn on_aggregation_defined(
        &self,
        aggregation_id: &str,
        aggregation_type: &str,  // "count", "sum", "avg", "window"
        source_stream: &str
    );

    /// Event verarbeitet
    fn on_event_processed(
        &self,
        stream_id: &str,
        event_id: &str,
        gas_used: u64
    );

    /// Aggregation berechnet
    fn on_aggregation_computed(
        &self,
        aggregation_id: &str,
        result_count: u64,
        latency_us: u64
    );

    /// Binding-Update propagiert
    fn on_binding_propagated(
        &self,
        source_stream: &str,
        target_binding: &str,
        latency_us: u64
    );
}

pub type SharedDataLogicObserver = Arc<dyn DataLogicObserver>;

// ============================================================================
// BLUEPRINTCOMPOSER-ENGINE OBSERVER
// ============================================================================

/// BlueprintComposer-Engine Observer - Tracks Blueprint-Composition
pub trait BlueprintComposerObserver: Send + Sync {
    /// Composition erstellt
    fn on_composition_created(
        &self,
        composition_id: &str,
        base_blueprint_id: &str,
        extends: &[String]
    );

    /// Blueprint instanziiert (aus Composition)
    fn on_blueprint_instantiated_from_composition(
        &self,
        composition_id: &str,
        instance_id: &str,
        realm_id: &str,
        gas_used: u64
    );

    /// Inheritance-Chain aufgelöst
    fn on_inheritance_resolved(
        &self,
        composition_id: &str,
        chain_depth: usize,
        conflicts_resolved: usize
    );

    /// Blueprint-Version migriert
    fn on_blueprint_migrated(
        &self,
        blueprint_id: &str,
        from_version: &str,
        to_version: &str,
        instances_affected: u64
    );
}

pub type SharedBlueprintComposerObserver = Arc<dyn BlueprintComposerObserver>;
```

---

### 13.7 StateIntegrator Erweiterung

**Datei:** `backend/src/core/state_integration.rs`

```rust
/// Erweiterter StateIntegrator mit Engine-Observer-Support
pub struct StateIntegrator {
    // Bestehende Observer
    pub trust_observers: RwLock<Vec<SharedTrustObserver>>,
    pub event_observers: RwLock<Vec<SharedEventObserver>>,
    pub execution_observers: RwLock<Vec<SharedExecutionObserver>>,
    pub protection_observers: RwLock<Vec<SharedProtectionObserver>>,
    pub formula_observers: RwLock<Vec<SharedFormulaObserver>>,
    pub consensus_observers: RwLock<Vec<SharedConsensusObserver>>,
    pub storage_observers: RwLock<Vec<SharedStorageObserver>>,
    pub gateway_observers: RwLock<Vec<SharedGatewayObserver>>,
    pub saga_observers: RwLock<Vec<SharedSagaObserver>>,
    pub intent_observers: RwLock<Vec<SharedIntentObserver>>,
    pub realm_observers: RwLock<Vec<SharedRealmObserver>>,
    pub eclvm_observers: RwLock<Vec<SharedECLVMObserver>>,
    pub swarm_observers: RwLock<Vec<SharedSwarmObserver>>,
    pub gossip_observers: RwLock<Vec<SharedGossipObserver>>,

    // ═══════════════════════════════════════════════════════════════════════════
    // NEU: ENGINE-LAYER OBSERVER
    // ═══════════════════════════════════════════════════════════════════════════

    /// UI-Engine Observer
    pub ui_observers: RwLock<Vec<SharedUIObserver>>,

    /// API-Engine Observer
    pub api_observers: RwLock<Vec<SharedAPIObserver>>,

    /// Governance-Engine Observer
    pub governance_observers: RwLock<Vec<SharedGovernanceObserver>>,

    /// Controller-Engine Observer
    pub controller_observers: RwLock<Vec<SharedControllerObserver>>,

    /// DataLogic-Engine Observer
    pub datalogic_observers: RwLock<Vec<SharedDataLogicObserver>>,

    /// BlueprintComposer-Engine Observer
    pub blueprint_composer_observers: RwLock<Vec<SharedBlueprintComposerObserver>>,

    /// Shared State Reference
    pub state: SharedUnifiedState,
}

impl StateIntegrator {
    pub fn new(state: SharedUnifiedState) -> Self {
        Self {
            // Bestehende
            trust_observers: RwLock::new(Vec::new()),
            event_observers: RwLock::new(Vec::new()),
            execution_observers: RwLock::new(Vec::new()),
            protection_observers: RwLock::new(Vec::new()),
            formula_observers: RwLock::new(Vec::new()),
            consensus_observers: RwLock::new(Vec::new()),
            storage_observers: RwLock::new(Vec::new()),
            gateway_observers: RwLock::new(Vec::new()),
            saga_observers: RwLock::new(Vec::new()),
            intent_observers: RwLock::new(Vec::new()),
            realm_observers: RwLock::new(Vec::new()),
            eclvm_observers: RwLock::new(Vec::new()),
            swarm_observers: RwLock::new(Vec::new()),
            gossip_observers: RwLock::new(Vec::new()),
            // NEU: Engine-Layer
            ui_observers: RwLock::new(Vec::new()),
            api_observers: RwLock::new(Vec::new()),
            governance_observers: RwLock::new(Vec::new()),
            controller_observers: RwLock::new(Vec::new()),
            datalogic_observers: RwLock::new(Vec::new()),
            blueprint_composer_observers: RwLock::new(Vec::new()),
            state,
        }
    }

    // === NEU: Engine-Observer-Registration ===

    pub fn register_ui_observer(&self, observer: SharedUIObserver) {
        if let Ok(mut observers) = self.ui_observers.write() {
            observers.push(observer);
        }
    }

    pub fn register_api_observer(&self, observer: SharedAPIObserver) {
        if let Ok(mut observers) = self.api_observers.write() {
            observers.push(observer);
        }
    }

    pub fn register_governance_observer(&self, observer: SharedGovernanceObserver) {
        if let Ok(mut observers) = self.governance_observers.write() {
            observers.push(observer);
        }
    }

    pub fn register_controller_observer(&self, observer: SharedControllerObserver) {
        if let Ok(mut observers) = self.controller_observers.write() {
            observers.push(observer);
        }
    }

    pub fn register_datalogic_observer(&self, observer: SharedDataLogicObserver) {
        if let Ok(mut observers) = self.datalogic_observers.write() {
            observers.push(observer);
        }
    }

    pub fn register_blueprint_composer_observer(&self, observer: SharedBlueprintComposerObserver) {
        if let Ok(mut observers) = self.blueprint_composer_observers.write() {
            observers.push(observer);
        }
    }
}
```

---

### 13.8 core/mod.rs Erweiterung

**Datei:** `backend/src/core/mod.rs`

```rust
// Re-exports für neue Engine-States und Observer

// State-Types (NEU)
pub use state::{
    // Bestehende
    UnifiedState, SharedUnifiedState, create_unified_state,
    CoreState, TrustState, EventState, FormulaState, ConsensusState,
    ExecutionState, ProtectionState, StorageState, PeerState, P2PState,
    ECLVMState, GatewayState, SagaComposerState,
    StateGraph, StateComponent, StateRelation,
    // NEU: Engine-Layer States
    UIState, UIStateSnapshot, RealmUIState,
    APIState, APIStateSnapshot, RealmAPIState,
    GovernanceState, GovernanceStateSnapshot, RealmGovernanceState,
    ControllerState, ControllerStateSnapshot, RealmControllerState,
    DataLogicState, DataLogicStateSnapshot,
    BlueprintComposerState, BlueprintComposerStateSnapshot,
};

// Observer-Traits (NEU)
pub use state_integration::{
    // Bestehende
    TrustObserver, EventObserver, ExecutionObserver, ProtectionObserver,
    FormulaObserver, ConsensusObserver, StorageObserver, GatewayObserver,
    SagaObserver, IntentObserver, RealmObserver, ECLVMObserver,
    SwarmObserver, GossipObserver,
    // NEU: Engine-Layer Observer
    UIObserver, SharedUIObserver,
    APIObserver, SharedAPIObserver,
    GovernanceObserver, SharedGovernanceObserver,
    ControllerObserver, SharedControllerObserver,
    DataLogicObserver, SharedDataLogicObserver,
    BlueprintComposerObserver, SharedBlueprintComposerObserver,
    // Integrator
    StateIntegrator,
};
```

---

### 13.9 Implementierungs-Reihenfolge

#### Phase 1: StateComponent & StateGraph (2 Tage)

| Schritt | Datei              | Änderung                        | LOC |
| ------- | ------------------ | ------------------------------- | --- |
| 1.1     | `state.rs:107-150` | 8 neue StateComponent-Varianten | +15 |
| 1.2     | `state.rs:160-245` | 35 neue StateGraph-Edges        | +80 |
| 1.3     | `state.rs` (Tests) | Tests für neue Components       | +50 |

#### Phase 2: State-Structs (5 Tage)

| Schritt | Datei      | Änderung                          | LOC  |
| ------- | ---------- | --------------------------------- | ---- |
| 2.1     | `state.rs` | UIState + Snapshot                | ~200 |
| 2.2     | `state.rs` | APIState + Snapshot               | ~180 |
| 2.3     | `state.rs` | GovernanceState + Snapshot        | ~200 |
| 2.4     | `state.rs` | ControllerState + Snapshot        | ~180 |
| 2.5     | `state.rs` | DataLogicState + Snapshot         | ~150 |
| 2.6     | `state.rs` | BlueprintComposerState + Snapshot | ~150 |

#### Phase 3: UnifiedState-Integration (2 Tage)

| Schritt | Datei           | Änderung                       | LOC |
| ------- | --------------- | ------------------------------ | --- |
| 3.1     | `state.rs:4045` | 6 neue Fields in UnifiedState  | +12 |
| 3.2     | `state.rs`      | UnifiedState::new() erweitern  | +12 |
| 3.3     | `state.rs`      | calculate_health() erweitern   | +25 |
| 3.4     | `state.rs`      | snapshot() erweitern           | +12 |
| 3.5     | `state.rs`      | UnifiedStateSnapshot erweitern | +12 |

#### Phase 4: Observer-Traits (3 Tage)

| Schritt | Datei                  | Änderung                        | LOC |
| ------- | ---------------------- | ------------------------------- | --- |
| 4.1     | `state_integration.rs` | UIObserver trait                | ~50 |
| 4.2     | `state_integration.rs` | APIObserver trait               | ~50 |
| 4.3     | `state_integration.rs` | GovernanceObserver trait        | ~70 |
| 4.4     | `state_integration.rs` | ControllerObserver trait        | ~60 |
| 4.5     | `state_integration.rs` | DataLogicObserver trait         | ~40 |
| 4.6     | `state_integration.rs` | BlueprintComposerObserver trait | ~40 |

#### Phase 5: StateIntegrator-Erweiterung (2 Tage)

| Schritt | Datei                  | Änderung                        | LOC  |
| ------- | ---------------------- | ------------------------------- | ---- |
| 5.1     | `state_integration.rs` | 6 neue Observer-Vektoren        | +12  |
| 5.2     | `state_integration.rs` | 6 neue register\_\*\_observer() | +60  |
| 5.3     | `state_integration.rs` | Notification-Helper             | +100 |

#### Phase 6: Re-Exports & Tests (2 Tage)

| Schritt | Datei         | Änderung                         | LOC  |
| ------- | ------------- | -------------------------------- | ---- |
| 6.1     | `core/mod.rs` | Neue Re-Exports                  | +30  |
| 6.2     | Tests         | Unit-Tests für alle neuen States | +300 |
| 6.3     | Tests         | Integration-Tests                | +200 |

---

### 13.10 Mathematische Konsistenz-Prüfung

Die StateGraph-Erweiterung muss mathematisch konsistent sein:

#### 13.10.1 Axiom-Verifikation

$$
\forall c \in \text{StateComponent}_{\text{neu}}: \exists r \in \text{StateRelation}: (c, r, \text{Trust}) \in \text{Edges}
$$

**Beweis:** Jede neue Komponente hat mindestens eine Trust-Beziehung:

- UI → DependsOn → Trust ✓
- API → DependsOn → Trust ✓
- Governance → DependsOn → Trust ✓
- Controller → DependsOn → Trust ✓
- DataLogic → DependsOn → Trust ✓
- BlueprintComposer → DependsOn → Trust ✓

#### 13.10.2 Zyklen-Freiheit

Der erweiterte StateGraph ist **azyklisch** für DependsOn-Beziehungen:

$$
\nexists \text{Pfad } c_1 \xrightarrow{\text{DependsOn}} c_2 \xrightarrow{\text{DependsOn}} \cdots \xrightarrow{\text{DependsOn}} c_1
$$

**Verifikation:** Die Abhängigkeits-Hierarchie ist:

```
Trust (Κ2-Κ5) → Basis für alle
    ↑
ECLVM → Gas/Mana → Execution
    ↑
UI/API/Governance/Controller/DataLogic/BlueprintComposer
```

Keine zirkulären Dependencies.

#### 13.10.3 Kritikalitäts-Score Update

Mit den neuen Komponenten:

| Component        | IST-Score | SOLL-Score | Δ   |
| ---------------- | --------- | ---------- | --- |
| Trust            | ~25       | ~35        | +10 |
| Event            | ~15       | ~22        | +7  |
| ECLVM            | ~12       | ~18        | +6  |
| Controller (NEU) | -         | ~12        | +12 |
| Governance (NEU) | -         | ~8         | +8  |

---

### 13.11 Zusammenfassung

| Metrik                               | Wert                 |
| ------------------------------------ | -------------------- |
| **Geschätzte Implementierungszeit**  | 16 Tage (3-4 Wochen) |
| **Neue Zeilen state.rs**             | ~1.411               |
| **Neue Zeilen state_integration.rs** | ~1.114               |
| **Neue StateComponent-Varianten**    | 8                    |
| **Neue StateGraph-Edges**            | ~35                  |
| **Neue State-Structs**               | 6                    |
| **Neue Observer-Traits**             | 6                    |
| **Test-Coverage-Ziel**               | >80%                 |

---

_Dieser StateManager Refactoring-Plan ist die Grundlage für die Integration der 6 neuen Engines in das Erynoa-System._
