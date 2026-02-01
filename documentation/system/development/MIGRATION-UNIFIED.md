# Vollständiger Migrationsplan: Alte Module → Unified

> **Version:** 1.0.0
> **Datum:** 1. Februar 2026
> **Ziel:** Alle Consumer auf `domain::unified::*` Typen migrieren

---

## Executive Summary

Dieser Plan beschreibt die **vollständige Migration** aller Module von den alten
`domain::{did,event,trust,realm,saga,formula}` Typen auf die neuen
`domain::unified::*` Strukturen.

**Prinzip:** Die neuen `unified` Strukturen sind die **Single Source of Truth**.
Alle alten Strukturen werden entfernt.

---

## I. Typ-Differenz-Matrix

### 1.1 Identity (DID)

| Alt (`domain::did`)                                                   | Neu (`unified::identity`)                                                             | Migration         |
| --------------------------------------------------------------------- | ------------------------------------------------------------------------------------- | ----------------- |
| `DID { id: String, namespace, unique_id, created_at: DateTime<Utc> }` | `DID { id: UniversalId, namespace, public_key: [u8; 32], created_at: TemporalCoord }` | **Breaking**      |
| `DID::new(ns, unique_id)`                                             | `DID::new(ns, public_key: &[u8])`                                                     | Signatur-Änderung |
| `DID::new_self(unique_id)`                                            | `DID::new_self(public_key: &[u8])`                                                    | Signatur-Änderung |
| `DID::to_uri()` → `String`                                            | `DID::to_uri()` → `String`                                                            | ✓ Kompatibel      |
| `DID::from_str()`                                                     | `FromStr` nicht implementiert                                                         | **Hinzufügen**    |
| `DIDError`                                                            | `IdentityError`                                                                       | Umbenennen        |

**Aktionen:**

1. [ ] `FromStr` für `unified::DID` implementieren
2. [ ] Alle `DID::new*(unique_id)` → `DID::new*(public_key)` ändern
3. [ ] Tests: unique_id → public_key (bei Tests oft `b"test"` etc.)

### 1.2 Event

| Alt (`domain::event`)                                         | Neu (`unified::event`)                                    | Migration         |
| ------------------------------------------------------------- | --------------------------------------------------------- | ----------------- |
| `EventId(String)`                                             | `EventId = UniversalId`                                   | **Breaking**      |
| `EventId::new(hash)`                                          | `event_id_from_content(content)`                          | Neue Funktion     |
| `Event { timestamp: DateTime<Utc>, finality: FinalityLevel }` | `Event { coord: TemporalCoord, finality: FinalityState }` | **Breaking**      |
| `Event::new(subject, parents, payload, realm)`                | `Event::new(author, parents, payload, lamport)`           | Signatur-Änderung |
| `Event::genesis(subject, realm)`                              | `Event::genesis(author, lamport)`                         | Signatur-Änderung |
| `Event.subject`                                               | `Event.author`                                            | Feld umbenennen   |
| `Event.timestamp`                                             | `Event.coord.wall_time()`                                 | Accessor          |
| `EventPayload::CredentialIssue/Revoke`                        | Nicht vorhanden                                           | **Hinzufügen**    |
| `FinalityLevel` (enum)                                        | `FinalityLevel` (enum) + `FinalityState` (struct)         | Erweitert         |

**Aktionen:**

1. [ ] `EventPayload::CredentialIssue`, `CredentialRevoke` hinzufügen
2. [ ] `Event.timestamp()` Accessor hinzufügen → `self.coord.wall_time()`
3. [ ] `Event::primary_trust_dimension()` hinzufügen
4. [ ] `Event::is_negative_trust()` hinzufügen
5. [ ] Alle `Event::new(subject, ..., realm)` → `Event::new(author, ..., lamport)` ändern

### 1.3 Trust

| Alt (`domain::trust`)                   | Neu (`unified::trust`)                  | Migration      |
| --------------------------------------- | --------------------------------------- | -------------- |
| `TrustVector6D { r: f64, i: f64, ... }` | `TrustVector6D { r: f32, i: f32, ... }` | **f64→f32**    |
| `TrustVector6D::new(r,i,c,p,v,omega)`   | `TrustVector6D::new(r,i,c,p,v,omega)`   | ✓ Aber f32     |
| `TrustVector6D::newcomer()`             | `TrustVector6D::NEWCOMER`               | Fn→Const       |
| `TrustVector6D.r` (f64)                 | `TrustVector6D.r` (f32)                 | Cast nötig     |
| `TrustVector6D::min_component()`        | Nicht vorhanden                         | **Hinzufügen** |
| `ContextType::weights()`                | Nicht vorhanden                         | **Hinzufügen** |

**Aktionen:**

1. [ ] `TrustVector6D::min_component()` hinzufügen (→ `self.min()`)
2. [ ] `ContextType::weights()` hinzufügen
3. [ ] Alle `f64` Trust-Werte → `f32` konvertieren
4. [ ] `newcomer()` → `NEWCOMER` ändern

### 1.4 Realm

| Alt (`domain::realm`)                  | Neu (`unified::realm`)     | Migration      |
| -------------------------------------- | -------------------------- | -------------- |
| `RealmId(String)`                      | `RealmId = UniversalId`    | **Breaking**   |
| `RealmId::new(name)`                   | `realm_id_from_name(name)` | Neue Funktion  |
| `RealmId::root()`                      | `ROOT_REALM_ID` const      | Fn→Const       |
| `VirtualRealm.initial_setup_policy`    | Nicht vorhanden            | **Hinzufügen** |
| `VirtualRealm.default_personal_stores` | Nicht vorhanden            | **Hinzufügen** |

**Aktionen:**

1. [ ] `VirtualRealm.initial_setup_policy` hinzufügen
2. [ ] `VirtualRealm.default_personal_stores` hinzufügen
3. [ ] Alle `RealmId::new(name)` → `realm_id_from_name(name)` ändern
4. [ ] Alle `RealmId::root()` → `ROOT_REALM_ID` ändern

### 1.5 Saga

| Alt (`domain::saga`)                                     | Neu (`unified::saga`)                          | Migration       |
| -------------------------------------------------------- | ---------------------------------------------- | --------------- |
| `Intent { source_did: DID }`                             | `Intent { source: UniversalId }`               | Feld umbenennen |
| `Intent::new(source_did, goal, realm)`                   | `Intent::new(source, goal, realm, lamport)`    | +lamport        |
| `Goal::Delegate { ttl_seconds }`                         | `Goal::Delegate { trust_factor, ttl_seconds }` | +trust_factor   |
| `Goal::Complex { parsed_goals }`                         | `Goal::Complex { sub_goals }`                  | Feld umbenennen |
| `Constraint::MaxCost { amount: u64 }`                    | `Constraint::MaxCost { cost: Cost }`           | Typ-Änderung    |
| `SagaAction::Lock/Unlock/WaitFor/Mint/Burn/GatewayCheck` | Nicht vorhanden                                | **Hinzufügen**  |
| `Budget` (in saga.rs)                                    | `Budget` (in cost.rs)                          | Import-Änderung |

**Aktionen:**

1. [ ] `SagaAction` Varianten hinzufügen: `Lock`, `Unlock`, `WaitFor`, `Mint`, `Burn`, `GatewayCheck`
2. [ ] Alle `intent.source_did` → `intent.source` ändern
3. [ ] Alle `Goal::Complex { parsed_goals }` → `{ sub_goals }` ändern
4. [ ] Alle `Constraint::MaxCost { amount }` → `{ cost: Cost::new(amount, 0, 0.0) }` ändern
5. [ ] Lamport-Parameter bei `Intent::new` hinzufügen

### 1.6 Formula

| Alt (`domain::formula`)               | Neu (`unified::formula`)      | Migration        |
| ------------------------------------- | ----------------------------- | ---------------- |
| `Surprisal { raw_surprisal: f64 }`    | `Surprisal { raw_bits: f64 }` | Feld umbenennen  |
| `Surprisal::dampened(trust)`          | `Surprisal::dampened()`       | trust im struct  |
| `Activity { tau_days }`               | `Activity { tau_seconds }`    | Einheit-Änderung |
| `WorldFormulaContribution::new(...)`  | Builder-Pattern               | **Hinzufügen**   |
| `WorldFormulaContribution::compute()` | Nicht vorhanden               | **Hinzufügen**   |
| `WorldFormulaContribution.trust`      | Nicht vorhanden               | **Hinzufügen**   |
| `WorldFormulaContribution.context`    | Nicht vorhanden               | **Hinzufügen**   |
| `WorldFormulaStatus`                  | Nicht in unified              | **Migration**    |

**Aktionen:**

1. [ ] `WorldFormulaContribution::new()` Factory hinzufügen
2. [ ] `WorldFormulaContribution::compute()` hinzufügen
3. [ ] `WorldFormulaContribution.trust`, `.context` Felder hinzufügen
4. [ ] `WorldFormulaStatus` nach unified migrieren
5. [ ] Alle `tau_days` → `tau_seconds * 86400` konvertieren

### 1.7 Primitives

| Feature                            | Status          | Aktion                          |
| ---------------------------------- | --------------- | ------------------------------- |
| `UniversalId.0` private            | Private         | `as_bytes()` Accessor verwenden |
| `UniversalId::from_bytes([u8;32])` | Nicht vorhanden | **Hinzufügen**                  |
| `UniversalId::root()`              | Nicht vorhanden | **Hinzufügen**                  |

---

## II. Migrations-Reihenfolge

### Phase 1: Unified Module erweitern (Tag 1)

```
unified/
├── primitives.rs    # +from_bytes, +root(), +as_inner()
├── identity.rs      # +FromStr für DID
├── event.rs         # +CredentialIssue/Revoke, +timestamp(), +primary_trust_dimension()
├── trust.rs         # +min_component(), +ContextType::weights()
├── realm.rs         # +initial_setup_policy, +default_personal_stores
├── saga.rs          # +Lock/Unlock/WaitFor/Mint/Burn/GatewayCheck
└── formula.rs       # +WorldFormulaContribution::new/compute, +WorldFormulaStatus
```

**Geschätzter Aufwand:** 4-6 Stunden

### Phase 2: Core-Layer migrieren (Tag 2)

```
core/
├── event_engine.rs     # Event::new Signaturen, timestamp→coord
├── trust_engine.rs     # f64→f32, ContextType::weights
├── consensus.rs        # FinalityLevel→FinalityState, f64→f32
├── world_formula.rs    # WorldFormulaContribution, WorldFormulaStatus
└── surprisal.rs        # Surprisal.raw_surprisal→raw_bits
```

**Geschätzter Aufwand:** 6-8 Stunden

### Phase 3: Local Storage migrieren (Tag 3)

```
local/
├── event_store.rs      # Event, EventId
├── trust_store.rs      # TrustVector6D (f32), DID
├── identity_store.rs   # DID, DIDNamespace
├── content_store.rs    # DID
├── realm_storage.rs    # RealmId, DID
└── mod.rs              # DIDNamespace Test
```

**Geschätzter Aufwand:** 4-6 Stunden

### Phase 4: Peer/P2P migrieren (Tag 4)

```
peer/
├── gateway.rs          # RealmId, TrustVector6D, VirtualRealm
├── intent_parser.rs    # Intent, Constraint, Goal
├── saga_composer.rs    # Saga, SagaAction, Budget
└── p2p/
    ├── behaviour.rs    # (minimal)
    └── trust_gate.rs   # TrustVector6D
```

**Geschätzter Aufwand:** 4-6 Stunden

### Phase 5: ECLVM/Protection migrieren (Tag 5)

```
eclvm/
├── mana.rs               # TrustVector6D (f32)
└── programmable_gateway.rs # RealmId, TrustVector6D, DID

protection/
├── anomaly.rs          # Event, EventPayload, DID
└── ...
```

**Geschätzter Aufwand:** 2-4 Stunden

### Phase 6: API-Layer migrieren (Tag 5-6)

```
api/
├── v1/intent/handlers.rs  # Intent, Constraint, Goal, RealmId, DID
├── v1/auth/handlers.rs    # DID
└── error.rs               # DIDError → IdentityError
```

**Geschätzter Aufwand:** 4-6 Stunden

### Phase 7: Alte Module entfernen (Tag 6)

```
1. domain/mod.rs → nur unified re-exports
2. rm domain/did.rs
3. rm domain/event.rs
4. rm domain/trust.rs
5. rm domain/realm.rs
6. rm domain/saga.rs
7. rm domain/formula.rs
```

**Geschätzter Aufwand:** 2-4 Stunden

---

## III. Detaillierte Änderungen pro Modul

### 3.1 unified/primitives.rs erweitern

```rust
impl UniversalId {
    /// Erstelle aus Byte-Array (für Deserialisierung)
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Root ID (für RealmId::root() Ersatz)
    pub const ROOT: Self = Self([0u8; 32]);

    /// Innere Bytes (für Consumer die .0 brauchen)
    pub fn as_inner(&self) -> &[u8; 32] {
        &self.0
    }
}
```

### 3.2 unified/identity.rs erweitern

```rust
impl FromStr for DID {
    type Err = IdentityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Parse "did:erynoa:namespace:hex-id"
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 4 || parts[0] != "did" || parts[1] != "erynoa" {
            return Err(IdentityError::InvalidFormat(s.to_string()));
        }

        let namespace = parts[2].parse::<DIDNamespace>()?;
        let id_hex = parts[3];
        let id_bytes = hex::decode(id_hex)
            .map_err(|_| IdentityError::InvalidFormat(s.to_string()))?;

        if id_bytes.len() != 32 {
            return Err(IdentityError::InvalidFormat(s.to_string()));
        }

        let mut arr = [0u8; 32];
        arr.copy_from_slice(&id_bytes);

        Ok(Self {
            id: UniversalId::from_bytes(arr),
            namespace,
            public_key: [0u8; 32], // Unbekannt bei Parse
            created_at: TemporalCoord::default(),
        })
    }
}
```

### 3.3 unified/event.rs erweitern

```rust
/// Zusätzliche EventPayload-Varianten
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventPayload {
    // ... bestehende ...

    /// Credential Issue (für Attestations)
    CredentialIssue {
        subject: UniversalId,
        credential_type: String,
        claims: HashMap<String, serde_json::Value>,
    },

    /// Credential Revoke
    CredentialRevoke {
        credential_id: UniversalId,
        reason: String,
    },
}

impl Event {
    /// Timestamp Accessor (Kompatibilität)
    pub fn timestamp(&self) -> u64 {
        self.coord.wall_time()
    }

    /// Primäre Trust-Dimension dieses Events
    pub fn primary_trust_dimension(&self) -> Option<TrustDimension> {
        match &self.payload {
            EventPayload::TrustUpdate { dimension, .. } => Some(*dimension),
            EventPayload::Attestation { .. } => Some(TrustDimension::Prestige),
            EventPayload::CredentialIssue { .. } => Some(TrustDimension::Competence),
            _ => None,
        }
    }

    /// Ist dieses Event ein negativer Trust-Event?
    pub fn is_negative_trust(&self) -> bool {
        match &self.payload {
            EventPayload::TrustUpdate { delta, .. } => *delta < 0.0,
            EventPayload::CredentialRevoke { .. } => true,
            _ => false,
        }
    }
}
```

### 3.4 unified/trust.rs erweitern

```rust
impl TrustVector6D {
    /// Minimum-Komponente (Alias für min())
    #[inline]
    pub fn min_component(&self) -> f32 {
        self.min()
    }
}

impl ContextType {
    /// Gewichtungen für diesen Kontext
    pub fn weights(&self) -> [f32; 6] {
        match self {
            Self::Default => [1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            Self::Financial => [1.5, 1.5, 1.2, 1.0, 1.3, 1.0],
            Self::Identity => [1.0, 1.5, 1.0, 1.5, 1.2, 1.5],
            Self::IoT => [1.5, 1.0, 1.5, 0.8, 1.5, 1.0],
            Self::AI => [1.2, 1.2, 1.5, 1.0, 1.5, 1.5],
            Self::Governance => [1.3, 1.5, 1.2, 1.5, 1.0, 1.5],
        }
    }
}
```

### 3.5 unified/saga.rs erweitern

```rust
/// Erweiterte SagaAction-Varianten
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SagaAction {
    // ... bestehende ...

    /// Wert sperren
    Lock {
        did: UniversalId,
        amount: u64,
        asset_type: String,
    },

    /// Wert entsperren
    Unlock {
        did: UniversalId,
        amount: u64,
        asset_type: String,
    },

    /// Auf Bedingung warten
    WaitFor {
        condition: String,
        timeout_seconds: u64,
    },

    /// Mint neuer Assets
    Mint {
        recipient: UniversalId,
        amount: u64,
        asset_type: String,
    },

    /// Burn von Assets
    Burn {
        from: UniversalId,
        amount: u64,
        asset_type: String,
    },

    /// Gateway-Check (Κ23)
    GatewayCheck {
        subject: UniversalId,
        target_realm: RealmId,
    },
}
```

### 3.6 unified/formula.rs erweitern

```rust
/// WorldFormulaContribution mit allen benötigten Feldern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFormulaContribution {
    /// Subjekt-ID
    pub subject: UniversalId,
    /// Aktivitäts-Präsenz 𝔸(s)
    pub activity: Activity,
    /// Surprisal 𝒮(s)
    pub surprisal: Surprisal,
    /// Human-Factor Ĥ(s)
    pub human_factor: HumanFactor,
    /// Temporales Gewicht w(s,t)
    pub temporal_weight: TemporalWeight,
    /// Trust-Vektor 𝕎(s)
    pub trust: TrustVector6D,
    /// Kontext
    pub context: ContextType,
    /// Berechnetes Φ(s)
    pub computed_value: Option<f64>,
}

impl WorldFormulaContribution {
    /// Factory-Methode
    pub fn new(
        subject: UniversalId,
        trust: TrustVector6D,
        context: ContextType,
    ) -> Self {
        Self {
            subject,
            activity: Activity::default(),
            surprisal: Surprisal::default(),
            human_factor: HumanFactor::default(),
            temporal_weight: TemporalWeight::default(),
            trust,
            context,
            computed_value: None,
        }
    }

    /// Berechne Beitrag Φ(s) (Κ15b)
    pub fn compute(&mut self) -> f64 {
        let a = self.activity.value();
        let weights = self.context.weights();
        let trust_norm = self.trust.weighted_norm(&weights) as f64;
        let history = (trust_norm * 10.0).ln_1p(); // ln|ℂ(s)| approximiert
        let s = self.surprisal.dampened();
        let sigmoid_input = trust_norm * history * s;
        let sigmoid = 1.0 / (1.0 + (-sigmoid_input).exp());
        let h = self.human_factor.value();
        let w = self.temporal_weight.value();

        let phi = a * sigmoid * h * w;
        self.computed_value = Some(phi);
        phi
    }
}

/// WorldFormulaStatus (migriert aus domain/formula.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFormulaStatus {
    /// Anzahl der Entitäten
    pub entity_count: u64,
    /// Gesamtes 𝔼
    pub total_e: f64,
    /// Durchschnittliche Aktivität
    pub avg_activity: f64,
    /// Human-Verified Ratio
    pub human_verified_ratio: f64,
    /// Letztes Update
    pub updated_at: TemporalCoord,
}
```

---

## IV. Test-Migration

### Pattern für Test-Änderungen

```rust
// ALT:
let did = DID::new_self("alice");
let trust = TrustVector6D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5); // f64
let realm_id = RealmId::new("test-realm");
let event = Event::new(did.clone(), vec![], EventPayload::Genesis, realm_id);

// NEU:
let did = DID::new_self(b"alice_public_key_32_bytes_here_");
let trust = TrustVector6D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5); // f32
let realm_id = realm_id_from_name("test-realm");
let event = Event::new(did.id, vec![], EventPayload::Genesis, 0);
```

---

## V. Checkliste

### unified/ erweitern

- [ ] `primitives.rs`: `from_bytes()`, `ROOT`, `as_inner()`
- [ ] `identity.rs`: `FromStr` für DID
- [ ] `event.rs`: `CredentialIssue/Revoke`, `timestamp()`, `primary_trust_dimension()`, `is_negative_trust()`
- [ ] `trust.rs`: `min_component()`, `ContextType::weights()`
- [ ] `realm.rs`: `initial_setup_policy`, `default_personal_stores`
- [ ] `saga.rs`: `Lock/Unlock/WaitFor/Mint/Burn/GatewayCheck`
- [ ] `formula.rs`: `WorldFormulaContribution::new/compute`, `WorldFormulaStatus`

### Consumer migrieren

- [ ] `core/event_engine.rs`
- [ ] `core/trust_engine.rs`
- [ ] `core/consensus.rs`
- [ ] `core/world_formula.rs`
- [ ] `core/surprisal.rs`
- [ ] `local/event_store.rs`
- [ ] `local/trust_store.rs`
- [ ] `local/identity_store.rs`
- [ ] `local/content_store.rs`
- [ ] `local/realm_storage.rs`
- [ ] `local/mod.rs`
- [ ] `peer/gateway.rs`
- [ ] `peer/intent_parser.rs`
- [ ] `peer/saga_composer.rs`
- [ ] `peer/p2p/trust_gate.rs`
- [ ] `eclvm/mana.rs`
- [ ] `eclvm/programmable_gateway.rs`
- [ ] `protection/anomaly.rs`
- [ ] `api/v1/intent/handlers.rs`
- [ ] `api/v1/auth/handlers.rs`
- [ ] `error.rs`

### Finale Bereinigung

- [ ] `domain/mod.rs` → nur unified re-exports
- [ ] Alte Module löschen
- [ ] Alle Tests anpassen
- [ ] `cargo test` → 0 Fehler
- [ ] `cargo clippy` → 0 Warnungen (außer deprecated)

---

## VI. Geschätzter Gesamtaufwand

| Phase                      | Aufwand | Kumulativ |
| -------------------------- | ------- | --------- |
| Phase 1: unified erweitern | 4-6h    | 6h        |
| Phase 2: Core-Layer        | 6-8h    | 14h       |
| Phase 3: Local Storage     | 4-6h    | 20h       |
| Phase 4: Peer/P2P          | 4-6h    | 26h       |
| Phase 5: ECLVM/Protection  | 2-4h    | 30h       |
| Phase 6: API-Layer         | 4-6h    | 36h       |
| Phase 7: Bereinigung       | 2-4h    | 40h       |

**Gesamt: 30-46 Stunden (4-6 Arbeitstage)**

---

_Dieser Plan ist bindend. Die neuen unified-Strukturen sind die Wahrheit._
_Erstellt: 1. Februar 2026_
