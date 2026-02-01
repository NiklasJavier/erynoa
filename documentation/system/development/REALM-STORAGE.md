# Realm Storage - Implementierungsdokumentation

> **Modul:** `backend/src/local/realm_storage.rs`
> **Version:** 1.0.0
> **Stand:** Februar 2026
> **Umfang:** ~2.900 Zeilen Rust-Code, 25+ Tests

## Übersicht

Die **Realm Storage**-Komponente ist das intelligente, dynamische Speichersystem für Realm-basierte Daten in Erynoa. Sie ermöglicht hierarchische Datenstrukturen mit Schema-Validierung, Schema-Evolution, persönliche/geteilte Stores und effiziente Key-Prefixing-Strategien.

### Kernfeatures

- **Wenige globale Partitionen**: Nur 3-5 Fjall-Partitionen (skaliert auf Tausende Realms)
- **Intelligentes Prefixing**: Keys kodieren Realm, Typ und Ownership
- **Schema-Validierung**: Typisierte Stores mit automatischer Validierung
- **Schema-Evolution**: Versionierte Schemas mit Backward-Compatibility
- **Personal/Shared Stores**: Isolierung auf DID-Ebene möglich
- **Nested Operations**: Tiefe Strukturen mit Pfad-Navigation
- **Gaming-Resistenz**: Mana-Kosten, Tiefenlimits, Challenge-Perioden

---

## Architektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     REALM STORAGE LAYER                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   Globale Partitionen (nur 3-5):                                           │
│   ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│   │ realm_meta  │ │ realm_data  │ │realm_indices│ │realm_events │          │
│   │ (Schemas,   │ │ (Stores,    │ │ (Secondary  │ │ (DAG-Events)│          │
│   │  Policies)  │ │  User Data) │ │  Indices)   │ │             │          │
│   └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘          │
│                                                                             │
│   Intelligentes Prefixing für Isolation:                                   │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │ realm:{realm_id}:shared:store:{name}:{key}                          │  │
│   │ realm:{realm_id}:personal:{did}:store:{name}:{key}                  │  │
│   │ realm:{realm_id}:shared:store:{name}:_schema                        │  │
│   │ realm:{realm_id}:shared:store:{name}:_schema_v{version}             │  │
│   │ realm:{realm_id}:shared:store:{name}:_changelog_v{version}          │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│   Schema-Evolution (Versionierung):                                        │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │ _schema_v1 → _schema_v2 → _schema_v3 (immutable history)            │  │
│   │ _schema → aktuelle Version (Pointer)                                │  │
│   │ _changelog_vN → Änderungsbeschreibung pro Version                   │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Komponentendiagramm

```
                    ┌──────────────────────┐
                    │    RealmStorage      │
                    └──────────┬───────────┘
                               │
        ┌──────────────────────┼──────────────────────┐
        │                      │                      │
        ▼                      ▼                      ▼
┌───────────────┐    ┌─────────────────┐    ┌──────────────────┐
│  StoreSchema  │    │  PrefixBuilder  │    │RealmStorageConfig│
│ (Typdefinition)│    │ (Key-Konstrukt.) │    │  (Limits/Costs)  │
└───────────────┘    └─────────────────┘    └──────────────────┘
        │                      │
        ▼                      ▼
┌───────────────────────────────────────────────────────────────┐
│                     Fjall Storage                              │
├───────────────────┬───────────────────┬───────────────────────┤
│     realm_meta    │    realm_data     │    realm_indices      │
│   (Schemas)       │   (User Data)     │   (Secondary Index)   │
└───────────────────┴───────────────────┴───────────────────────┘
```

---

## Schema-System

### SchemaFieldType

Dynamischer Feldtyp für typisierte Stores.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchemaFieldType {
    // ═══════════════════════════════════════════════════════════════════════
    // Einfache Typen
    // ═══════════════════════════════════════════════════════════════════════

    String,      // UTF-8 Text
    Number,      // f64 (Floating Point)
    Bool,        // Boolean
    Did,         // DID-Referenz (als String gespeichert)
    Timestamp,   // Unix Timestamp (u64)
    Bytes,       // Binärdaten

    // ═══════════════════════════════════════════════════════════════════════
    // Komplexe Typen
    // ═══════════════════════════════════════════════════════════════════════

    /// Homogene Liste
    List { item_type: Box<SchemaFieldType> },

    /// Strukturiertes Objekt mit benannten Feldern
    Object { fields: HashMap<String, SchemaFieldType> },

    /// Referenz auf anderen Store (Foreign Key)
    Reference { target_store: String },

    /// Nullable Wrapper
    Optional { inner: Box<SchemaFieldType> },
}
```

### Typ-Komplexität

Jeder Typ hat eine berechnete Komplexität für Mana-Kosten:

| Typ           | Komplexität              | Beschreibung   |
| ------------- | ------------------------ | -------------- |
| `String`      | 1                        | Basis-Typ      |
| `Number`      | 1                        | Basis-Typ      |
| `Bool`        | 1                        | Basis-Typ      |
| `Did`         | 1                        | Basis-Typ      |
| `Timestamp`   | 1                        | Basis-Typ      |
| `Bytes`       | 1                        | Basis-Typ      |
| `Reference`   | 2                        | Fremdschlüssel |
| `List<T>`     | 2 + complexity(T)        | Rekursiv       |
| `Object`      | 2 + Σ complexity(fields) | Rekursiv       |
| `Optional<T>` | 1 + complexity(T)        | Wrapper        |

### Verschachtelungstiefe

```rust
impl SchemaFieldType {
    /// Berechne Verschachtelungstiefe
    pub fn depth(&self) -> u32 {
        match self {
            // Einfache Typen: Tiefe 0
            String | Number | Bool | Did | Timestamp | Bytes | Reference => 0,

            // Rekursive Typen
            List { item_type } => 1 + item_type.depth(),
            Object { fields } => 1 + fields.values().map(|f| f.depth()).max().unwrap_or(0),
            Optional { inner } => inner.depth(),  // Optional erhöht Tiefe nicht
        }
    }
}
```

**Beispiel:**

```rust
// Tiefe 0
SchemaFieldType::String

// Tiefe 1
SchemaFieldType::List {
    item_type: Box::new(SchemaFieldType::String)
}

// Tiefe 2
SchemaFieldType::Object {
    fields: hashmap! {
        "tags" => SchemaFieldType::List {
            item_type: Box::new(SchemaFieldType::String)
        }
    }
}
```

---

## StoreSchema

Definition eines dynamischen Stores.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreSchema {
    /// Name des Stores
    pub name: String,

    /// Versionsnummer (für Schema-Evolution)
    pub version: u32,

    /// Felder und ihre Typen
    pub fields: HashMap<String, SchemaFieldType>,

    /// Personal-Store (pro DID isoliert) oder Shared
    pub personal: bool,

    /// Maximale Anzahl Einträge (0 = unbegrenzt)
    pub max_entries: u64,

    /// Sekundäre Indices für Performance
    pub indices: Vec<String>,
}
```

### Builder-Pattern

```rust
let schema = StoreSchema::new("posts", false)  // name, personal
    .with_field("title", SchemaFieldType::String)
    .with_field("content", SchemaFieldType::String)
    .with_field("author", SchemaFieldType::Did)
    .with_field("created_at", SchemaFieldType::Timestamp)
    .with_field("tags", SchemaFieldType::List {
        item_type: Box::new(SchemaFieldType::String)
    })
    .with_field("metadata", SchemaFieldType::Optional {
        inner: Box::new(SchemaFieldType::Object {
            fields: hashmap! {
                "views" => SchemaFieldType::Number,
                "featured" => SchemaFieldType::Bool,
            }
        })
    })
    .with_index("author")
    .with_index("created_at")
    .with_max_entries(10000);
```

### Schema-Methoden

| Methode                     | Beschreibung                         |
| --------------------------- | ------------------------------------ |
| `new(name, personal)`       | Erstellt leeres Schema               |
| `with_field(name, type)`    | Fügt Feld hinzu                      |
| `with_index(field_name)`    | Fügt Sekundär-Index hinzu            |
| `with_max_entries(max)`     | Setzt Entry-Limit                    |
| `complexity()`              | Berechnet Gesamt-Komplexität         |
| `max_depth()`               | Berechnet maximale Tiefe             |
| `validate_depth(max)`       | Validiert gegen Tiefenlimit          |
| `is_compatible_with(other)` | Prüft Schema-Kompatibilität          |
| `evolve(changes)`           | Erstellt neue Version mit Änderungen |

---

## Schema-Evolution (Ψ-Adaptation)

Das System unterstützt dynamische Schema-Änderungen mit versionierter Historie.

### SchemaChange

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaChange {
    /// Neues Feld hinzufügen (Non-Breaking)
    AddField {
        name: String,
        field_type: SchemaFieldType,
        default: Option<StoreValue>,  // Für Migration
    },

    /// Feld entfernen (Breaking!)
    RemoveField { name: String },

    /// Feldtyp ändern (Breaking bei Inkompatibilität)
    ModifyField {
        name: String,
        new_type: SchemaFieldType,
    },

    /// Feld umbenennen (Breaking)
    RenameField {
        old_name: String,
        new_name: String,
    },

    /// Index hinzufügen (Non-Breaking)
    AddIndex { field_name: String },

    /// Index entfernen (Non-Breaking)
    RemoveIndex { field_name: String },
}
```

### Breaking vs. Non-Breaking Changes

| Änderung      | Breaking? | Challenge-Periode | Mana-Multiplikator |
| ------------- | --------- | ----------------- | ------------------ |
| `AddField`    | ❌        | Keine             | 1×                 |
| `RemoveField` | ✅        | 7 Tage            | 4×                 |
| `ModifyField` | ✅        | 7 Tage            | 4×                 |
| `RenameField` | ✅        | 7 Tage            | 4×                 |
| `AddIndex`    | ❌        | Keine             | 1×                 |
| `RemoveIndex` | ❌        | Keine             | 1×                 |

### Mana-Kosten für Änderungen

$$
\text{Mana}_{\text{change}} = \begin{cases}
50 + \text{complexity}(T) \times 10 & \text{für AddField} \\
200 & \text{für RemoveField} \\
300 + \text{complexity}(T_{new}) \times 10 & \text{für ModifyField} \\
100 & \text{für RenameField} \\
150 & \text{für AddIndex} \\
50 & \text{für RemoveIndex}
\end{cases}
$$

Bei Breaking Changes wird der Gesamt-Mana-Betrag mit 4 multipliziert.

### Schema-Evolution-Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     SCHEMA EVOLUTION FLOW                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   1. evolve_schema() aufrufen                                          │
│      ├── Validiere Änderungen                                          │
│      ├── Berechne Mana-Kosten                                          │
│      └── Erstelle neue Schema-Version                                  │
│                                                                         │
│   2. Speichern                                                         │
│      ├── _schema_v{N} → Versioniertes Schema                           │
│      └── _changelog_v{N} → Änderungsbeschreibung                       │
│                                                                         │
│   3a. Non-Breaking: Sofort aktiv                                       │
│       └── _schema → Update auf neue Version                            │
│                                                                         │
│   3b. Breaking: Challenge-Periode                                      │
│       ├── Status: Pending (7 Tage)                                     │
│       ├── Community kann ablehnen (reject)                             │
│       └── Nach Ablauf: activate_pending_schema()                       │
│                                                                         │
│   4. Lazy Migration bei Read                                           │
│      └── migrate_value() passt alte Daten an                           │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### SchemaChangelogEntry

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChangelogEntry {
    /// Version nach der Änderung
    pub version: u32,

    /// Zeitstempel (Unix Seconds)
    pub timestamp: u64,

    /// DID des Änderenden
    pub changed_by: String,

    /// Durchgeführte Änderungen
    pub changes: Vec<SchemaChange>,

    /// Beschreibung
    pub description: String,

    /// War Challenge-Periode erforderlich?
    pub required_challenge: bool,

    /// Aktueller Status
    pub status: SchemaChangeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaChangeStatus {
    /// Änderung aktiv
    Active,

    /// In Challenge-Periode
    Pending { challenge_ends: u64 },

    /// Abgelehnt
    Rejected { reason: String },

    /// Durch neuere Version ersetzt
    Superseded { by_version: u32 },
}
```

### Typ-Kompatibilität

```rust
impl SchemaFieldType {
    /// Prüfe Kompatibilität (kann alte Daten als neuen Typ lesen?)
    pub fn is_compatible_with(&self, other: &SchemaFieldType) -> bool {
        match (self, other) {
            // Gleiche Typen: kompatibel
            (a, b) if discriminant(a) == discriminant(b) => true,

            // T → Optional<T>: kompatibel (Erweiterung)
            (Optional { inner }, other_type) => inner.is_compatible_with(other_type),

            // Number → String: kompatibel (Formatierung)
            (String, Number) => true,

            // Alles andere: inkompatibel
            _ => false,
        }
    }
}
```

**Kompatibilitätsmatrix:**

| Von ↓ / Nach → | String | Number | Bool | Optional<T>            |
| -------------- | ------ | ------ | ---- | ---------------------- |
| String         | ✅     | ❌     | ❌   | ✅                     |
| Number         | ✅     | ✅     | ❌   | ✅                     |
| Bool           | ❌     | ❌     | ✅   | ✅                     |
| Optional<T>    | ❌     | ❌     | ❌   | ✅ (wenn T kompatibel) |

---

## StoreValue (Dynamischer Wert)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StoreValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    List(Vec<StoreValue>),
    Object(HashMap<String, StoreValue>),

    // Interne Typen (Serialisierung als Basis-Typ)
    Did(String),
    Timestamp(u64),
    Bytes(Vec<u8>),
}
```

### Validierung gegen Schema

```rust
impl StoreValue {
    pub fn validate(&self, schema_type: &SchemaFieldType) -> Result<()> {
        match (self, schema_type) {
            (Null, Optional { .. }) => Ok(()),
            (String(_), String) => Ok(()),
            (Number(_), Number) => Ok(()),
            (Bool(_), Bool) => Ok(()),
            (Did(_), Did) => Ok(()),
            (Timestamp(_), Timestamp) => Ok(()),
            (Bytes(_), Bytes) => Ok(()),

            (List(items), List { item_type }) => {
                for item in items {
                    item.validate(item_type)?;
                }
                Ok(())
            }

            (Object(obj), Object { fields }) => {
                for (key, field_type) in fields {
                    if let Some(value) = obj.get(key) {
                        value.validate(field_type)?;
                    } else if !matches!(field_type, Optional { .. }) {
                        return Err(anyhow!("Missing required field: {}", key));
                    }
                }
                Ok(())
            }

            _ => Err(anyhow!("Type mismatch")),
        }
    }
}
```

---

## PrefixBuilder (Key-Konstruktion)

Intelligente Key-Konstruktion für Realm-Isolation.

```rust
pub struct PrefixBuilder {
    realm_id: String,
    store_type: StoreType,  // Shared | Personal
    owner_did: Option<String>,
    store_name: String,
}
```

### Key-Formate

| Methode              | Format                           | Beispiel                                             |
| -------------------- | -------------------------------- | ---------------------------------------------------- |
| `store_prefix()`     | `realm:{id}:shared:store:{name}` | `realm:social.berlin:shared:store:posts`             |
| `key(k)`             | `{store_prefix}:{k}`             | `realm:social.berlin:shared:store:posts:post123`     |
| `schema_key()`       | `{store_prefix}:_schema`         | `realm:social.berlin:shared:store:posts:_schema`     |
| `index_prefix(f)`    | `{store_prefix}:_idx:{f}`        | `realm:social.berlin:shared:store:posts:_idx:author` |
| `index_key(f,v,k)`   | `{index_prefix}:{v}:{k}`         | `...:_idx:author:did:erynoa:alice:post123`           |
| `nested_key(k,path)` | `{store_prefix}:{k}:{path}`      | `...:posts:post123:metadata:views`                   |

### Personal Store Keys

```rust
// Shared Store
PrefixBuilder::shared(&realm_id, "posts")
// → realm:social.berlin:shared:store:posts

// Personal Store
PrefixBuilder::personal(&realm_id, &did, "notes")
// → realm:social.berlin:personal:alice123:store:notes
```

---

## RealmStorageConfig

```rust
#[derive(Debug, Clone)]
pub struct RealmStorageConfig {
    // ═══════════════════════════════════════════════════════════════════════
    // Tiefenlimits
    // ═══════════════════════════════════════════════════════════════════════

    /// Maximale Verschachtelungstiefe (Default: 5)
    pub max_depth: u32,

    /// Maximale Array-Länge (Default: 100)
    pub max_array_length: usize,

    // ═══════════════════════════════════════════════════════════════════════
    // Mana-Kosten
    // ═══════════════════════════════════════════════════════════════════════

    /// Basis-Mana für Store-Erstellung (Default: 100)
    pub base_mana_cost: u64,

    /// Mana pro Verschachtelungsebene (Default: 10)
    pub mana_per_depth: u64,

    /// Mana pro Put-Operation (Default: 5)
    pub mana_per_put: u64,

    // ═══════════════════════════════════════════════════════════════════════
    // Kapazitätslimits
    // ═══════════════════════════════════════════════════════════════════════

    /// Max Stores pro Realm (Default: 100)
    pub max_stores_per_realm: usize,

    /// Max Personal-Stores pro User (Default: 20)
    pub max_personal_stores_per_user: usize,

    // ═══════════════════════════════════════════════════════════════════════
    // Schema-Evolution
    // ═══════════════════════════════════════════════════════════════════════

    /// Challenge-Periode für Breaking Changes (Default: 7 Tage)
    pub breaking_change_challenge_period: u64,

    /// Min Trust-R für Schema-Änderungen (Default: 0.7)
    pub min_trust_for_schema_change: f64,

    /// Mana-Multiplikator für Breaking Changes (Default: 4)
    pub breaking_change_mana_multiplier: u64,
}
```

### Default-Werte

| Parameter                          | Default     | Beschreibung                    |
| ---------------------------------- | ----------- | ------------------------------- |
| `max_depth`                        | 5           | Tiefenlimit für Verschachtelung |
| `max_array_length`                 | 100         | Max Elemente pro Array          |
| `base_mana_cost`                   | 100         | Basis für Store-Erstellung      |
| `mana_per_depth`                   | 10          | Zusatzkosten pro Ebene          |
| `mana_per_put`                     | 5           | Kosten pro Write                |
| `max_stores_per_realm`             | 100         | Store-Limit pro Realm           |
| `max_personal_stores_per_user`     | 20          | Personal-Store-Limit            |
| `breaking_change_challenge_period` | 604800 (7d) | Challenge-Dauer                 |
| `min_trust_for_schema_change`      | 0.7         | Trust-Anforderung               |
| `breaking_change_mana_multiplier`  | 4           | Multiplikator                   |

---

## RealmStorage API

### Struktur

```rust
pub struct RealmStorage {
    /// Referenz auf Keyspace
    keyspace: Arc<Keyspace>,

    /// Partition für Metadaten (Schemas, Policies)
    pub meta: PartitionHandle,

    /// Partition für dynamische Daten
    pub data: PartitionHandle,

    /// Partition für Sekundär-Indices (optional)
    pub indices: Option<PartitionHandle>,

    /// Konfiguration
    config: RealmStorageConfig,

    /// Cache für Schemas
    schema_cache: RwLock<HashMap<String, StoreSchema>>,
}
```

### Konstruktor

```rust
pub fn new(keyspace: &Arc<Keyspace>, config: RealmStorageConfig) -> Result<Self>;
```

---

## CRUD-Operationen

### Store erstellen

```rust
pub fn create_store(
    &self,
    realm_id: &RealmId,
    sender_did: &DID,
    schema: StoreSchema,
) -> Result<()>;
```

**Ablauf:**

1. Tiefe validieren (`max_depth`)
2. Prüfen ob Store existiert
3. Schema serialisieren und speichern
4. Cache aktualisieren

### Wert schreiben (Put)

```rust
pub fn put(
    &self,
    realm_id: &RealmId,
    sender_did: &DID,
    store_name: &str,
    key: &str,
    value: StoreValue,
) -> Result<()>;
```

**Ablauf:**

1. Schema laden
2. Wert gegen Schema validieren
3. Prefix konstruieren (shared/personal)
4. Wert serialisieren und speichern
5. Sekundär-Indices aktualisieren

### Wert lesen (Get)

```rust
pub fn get(
    &self,
    realm_id: &RealmId,
    sender_did: &DID,
    store_name: &str,
    key: &str,
) -> Result<Option<StoreValue>>;
```

### Wert löschen (Delete)

```rust
pub fn delete(
    &self,
    realm_id: &RealmId,
    sender_did: &DID,
    store_name: &str,
    key: &str,
) -> Result<bool>;
```

---

## Nested Operations

### Verschachtelt schreiben

```rust
pub fn put_nested(
    &self,
    realm_id: &RealmId,
    sender_did: &DID,
    store_name: &str,
    key: &str,
    path: &[&str],
    value: StoreValue,
) -> Result<()>;
```

**Beispiel:**

```rust
storage.put_nested(
    &realm_id,
    &sender,
    "profiles",
    "user1",
    &["settings", "theme"],
    StoreValue::String("dark".to_string()),
)?;
```

### Verschachtelt lesen

```rust
pub fn get_nested(
    &self,
    realm_id: &RealmId,
    sender_did: &DID,
    store_name: &str,
    key: &str,
    path: &[&str],
) -> Result<Option<StoreValue>>;
```

### Liste erweitern

```rust
pub fn append_list(
    &self,
    realm_id: &RealmId,
    sender_did: &DID,
    store_name: &str,
    key: &str,
    path: &[&str],
    value: StoreValue,
) -> Result<usize>;  // Neue Länge
```

**Beispiel:**

```rust
let new_len = storage.append_list(
    &realm_id,
    &sender,
    "users",
    "user1",
    &["interests"],
    StoreValue::String("rust".to_string()),
)?;
assert_eq!(new_len, 1);
```

---

## Query-Operationen

### Alle Einträge

```rust
pub fn query_all(
    &self,
    realm_id: &RealmId,
    sender_did: &DID,
    store_name: &str,
    limit: Option<usize>,
) -> Result<Vec<(String, StoreValue)>>;
```

### Index-Query

```rust
pub fn query_by_index(
    &self,
    realm_id: &RealmId,
    sender_did: &DID,
    store_name: &str,
    field_name: &str,
    value: &str,
    limit: Option<usize>,
) -> Result<Vec<(String, StoreValue)>>;
```

**Beispiel:**

```rust
// Finde alle Posts eines Autors
let posts = storage.query_by_index(
    &realm_id,
    &sender,
    "posts",
    "author",              // Index-Feld
    "did:erynoa:alice",    // Wert
    Some(10),              // Limit
)?;
```

### Stores auflisten

```rust
pub fn list_stores(&self, realm_id: &RealmId) -> Result<Vec<StoreSchema>>;
```

---

## Schema-Evolution API

### Schema evolvieren

```rust
pub fn evolve_schema(
    &self,
    realm_id: &RealmId,
    sender_did: &DID,
    store_name: &str,
    changes: Vec<SchemaChange>,
    description: String,
) -> Result<SchemaEvolutionResult>;
```

**Rückgabe:**

```rust
pub struct SchemaEvolutionResult {
    pub new_version: u32,
    pub is_breaking: bool,
    pub status: SchemaChangeStatus,
    pub mana_cost: u64,
}
```

### Pending Schema aktivieren

```rust
pub fn activate_pending_schema(
    &self,
    realm_id: &RealmId,
    sender_did: &DID,
    store_name: &str,
    version: u32,
) -> Result<()>;
```

Aktiviert Schema nach Ablauf der Challenge-Periode.

### Pending Schema ablehnen

```rust
pub fn reject_pending_schema(
    &self,
    realm_id: &RealmId,
    sender_did: &DID,
    store_name: &str,
    version: u32,
    reason: String,
) -> Result<()>;
```

### Schema-Historie abrufen

```rust
pub fn get_schema_history(
    &self,
    realm_id: &RealmId,
    store_name: &str,
    sender_did: Option<&DID>,
) -> Result<SchemaHistory>;
```

**Rückgabe:**

```rust
pub struct SchemaHistory {
    pub store_name: String,
    pub current_version: u32,
    pub changelog: Vec<SchemaChangelogEntry>,
}
```

### Spezifische Version abrufen

```rust
pub fn get_schema_version(
    &self,
    realm_id: &RealmId,
    store_name: &str,
    version: u32,
    sender_did: Option<&DID>,
) -> Result<StoreSchema>;
```

### Wert migrieren

```rust
pub fn migrate_value(
    &self,
    value: StoreValue,
    from_schema: &StoreSchema,
    to_schema: &StoreSchema,
) -> Result<StoreValue>;
```

Passt alte Daten an neues Schema an (Lazy Migration).

---

## Mana-Kosten-Berechnung

### Store-Erstellung

$$
\text{Mana}_{\text{create}} = \text{base} + \text{depth} \times \text{mana\_per\_depth} + \text{complexity} \times 2
$$

```rust
pub fn calculate_create_cost(&self, schema: &StoreSchema) -> u64 {
    let depth_cost = schema.max_depth() as u64 * self.config.mana_per_depth;
    let complexity_cost = schema.complexity() * 2;
    self.config.base_mana_cost + depth_cost + complexity_cost
}
```

### Put-Operation

$$
\text{Mana}_{\text{put}} = \text{base} + \text{nested\_depth} \times \text{mana\_per\_depth} + \text{size\_cost}
$$

```rust
pub fn calculate_put_cost(&self, value: &StoreValue, nested_depth: u32) -> u64 {
    let depth_cost = nested_depth as u64 * self.config.mana_per_depth;
    let size_cost = match value {
        StoreValue::List(items) => items.len() as u64,
        StoreValue::Object(fields) => fields.len() as u64,
        _ => 0,
    };
    self.config.mana_per_put + depth_cost + size_cost
}
```

### Schema-Evolution

$$
\text{Mana}_{\text{evolve}} = \sum_{c \in \text{changes}} \text{mana}(c) \times \begin{cases}
4 & \text{wenn breaking} \\
1 & \text{sonst}
\end{cases}
$$

---

## Host-Interface (Convenience-Methoden)

Für die Integration mit ECL und dem ErynoaHost bietet RealmStorage Convenience-Methoden:

### Shared Store

```rust
// Get
pub fn get_shared(&self, realm_id: &str, store_name: &str, key: &str)
    -> Result<Option<StoreValue>>;

// Put
pub fn put_shared(&self, realm_id: &str, store_name: &str, key: &str, value: StoreValue)
    -> Result<()>;

// Delete
pub fn delete_shared(&self, realm_id: &str, store_name: &str, key: &str)
    -> Result<bool>;

// Nested Get
pub fn get_nested_shared(&self, realm_id: &str, store_name: &str, key: &str, path: &str)
    -> Result<Option<StoreValue>>;

// Nested Put
pub fn put_nested_shared(&self, realm_id: &str, store_name: &str, key: &str, path: &str, value: StoreValue)
    -> Result<()>;

// Append to List
pub fn append_to_list_shared(&self, realm_id: &str, store_name: &str, key: &str, path: &str, value: StoreValue)
    -> Result<usize>;

// Query
pub fn list_keys_shared(&self, realm_id: &str, store_name: &str, prefix: Option<&str>, limit: usize)
    -> Result<Vec<String>>;

pub fn query_by_index_shared(&self, realm_id: &str, store_name: &str, field: &str, value: &str, limit: usize)
    -> Result<Vec<String>>;

// Count
pub fn count_shared(&self, realm_id: &str, store_name: &str) -> Result<usize>;

// Exists
pub fn store_exists_shared(&self, realm_id: &str, store_name: &str) -> Result<bool>;
```

### Personal Store

```rust
// Get
pub fn get_personal(&self, realm_id: &str, did: &DID, store_name: &str, key: &str)
    -> Result<Option<StoreValue>>;

// Put
pub fn put_personal(&self, realm_id: &str, did: &DID, store_name: &str, key: &str, value: StoreValue)
    -> Result<()>;

// Delete
pub fn delete_personal(&self, realm_id: &str, did: &DID, store_name: &str, key: &str)
    -> Result<bool>;

// ... (analog zu Shared)
```

---

## Verwendungsbeispiele

### Store erstellen und Daten schreiben

```rust
use crate::local::{RealmStorage, RealmStorageConfig, StoreSchema, SchemaFieldType, StoreValue};
use crate::domain::{RealmId, DID};

// Storage initialisieren
let storage = RealmStorage::new(&keyspace, RealmStorageConfig::default())?;

let realm_id = RealmId::new("social.berlin");
let alice = DID::new_self("alice");

// Schema definieren
let schema = StoreSchema::new("posts", false)
    .with_field("title", SchemaFieldType::String)
    .with_field("content", SchemaFieldType::String)
    .with_field("author", SchemaFieldType::Did)
    .with_field("tags", SchemaFieldType::List {
        item_type: Box::new(SchemaFieldType::String)
    })
    .with_index("author");

// Store erstellen
storage.create_store(&realm_id, &alice, schema)?;

// Daten schreiben
let post = StoreValue::Object(hashmap! {
    "title" => StoreValue::String("Hello Erynoa".to_string()),
    "content" => StoreValue::String("My first post!".to_string()),
    "author" => StoreValue::Did(alice.to_uri()),
    "tags" => StoreValue::List(vec![
        StoreValue::String("introduction".to_string()),
        StoreValue::String("erynoa".to_string()),
    ]),
});

storage.put(&realm_id, &alice, "posts", "post1", post)?;
```

### Personal Store für Isolation

```rust
let alice = DID::new_self("alice");
let bob = DID::new_self("bob");

// Personal Schema (personal = true)
let notes_schema = StoreSchema::new("notes", true)
    .with_field("text", SchemaFieldType::String)
    .with_field("private", SchemaFieldType::Bool);

// Store für beide User erstellen
storage.create_store(&realm_id, &alice, notes_schema.clone())?;
storage.create_store(&realm_id, &bob, notes_schema)?;

// Alice schreibt
storage.put(&realm_id, &alice, "notes", "note1",
    StoreValue::Object(hashmap! {
        "text" => StoreValue::String("Alice's secret".to_string()),
        "private" => StoreValue::Bool(true),
    })
)?;

// Bob schreibt unter gleichem Key (isoliert!)
storage.put(&realm_id, &bob, "notes", "note1",
    StoreValue::Object(hashmap! {
        "text" => StoreValue::String("Bob's note".to_string()),
        "private" => StoreValue::Bool(false),
    })
)?;

// Alice sieht nur ihre Daten
let alice_note = storage.get(&realm_id, &alice, "notes", "note1")?;
// → "Alice's secret"

// Bob sieht nur seine Daten
let bob_note = storage.get(&realm_id, &bob, "notes", "note1")?;
// → "Bob's note"
```

### Schema evolvieren

```rust
// Feld hinzufügen (Non-Breaking)
let changes = vec![
    SchemaChange::AddField {
        name: "views".to_string(),
        field_type: SchemaFieldType::Number,
        default: Some(StoreValue::Number(0.0)),
    },
    SchemaChange::AddIndex {
        field_name: "views".to_string(),
    },
];

let result = storage.evolve_schema(
    &realm_id,
    &alice,
    "posts",
    changes,
    "Add view counter".to_string(),
)?;

assert_eq!(result.new_version, 2);
assert!(!result.is_breaking);
assert_eq!(result.status, SchemaChangeStatus::Active);
```

### Breaking Change mit Challenge-Periode

```rust
// Feld entfernen (Breaking!)
let changes = vec![
    SchemaChange::RemoveField {
        name: "legacy_field".to_string(),
    },
];

let result = storage.evolve_schema(
    &realm_id,
    &alice,
    "config",
    changes,
    "Remove deprecated field".to_string(),
)?;

assert!(result.is_breaking);
assert!(matches!(result.status, SchemaChangeStatus::Pending { .. }));

// Nach 7 Tagen aktivieren
// storage.activate_pending_schema(&realm_id, &alice, "config", result.new_version)?;

// Oder ablehnen
storage.reject_pending_schema(
    &realm_id,
    &alice,
    "config",
    result.new_version,
    "Community voted against".to_string(),
)?;
```

### Index-Query

```rust
// Alle Posts eines Autors
let alice_posts = storage.query_by_index(
    &realm_id,
    &alice,
    "posts",
    "author",
    "did:erynoa:self:alice",
    Some(20),
)?;

for (key, post) in alice_posts {
    println!("Post {}: {:?}", key, post);
}
```

### Nested Operations

```rust
// Schema mit Verschachtelung
let profile_schema = StoreSchema::new("profiles", false)
    .with_field("info", SchemaFieldType::Object {
        fields: hashmap! {
            "bio" => SchemaFieldType::String,
            "settings" => SchemaFieldType::Object {
                fields: hashmap! {
                    "theme" => SchemaFieldType::String,
                    "notifications" => SchemaFieldType::Bool,
                }
            },
        }
    });

storage.create_store(&realm_id, &alice, profile_schema)?;

// Tief verschachtelt schreiben
storage.put_nested(
    &realm_id,
    &alice,
    "profiles",
    "user1",
    &["info", "settings", "theme"],
    StoreValue::String("dark".to_string()),
)?;

// Tief verschachtelt lesen
let theme = storage.get_nested(
    &realm_id,
    &alice,
    "profiles",
    "user1",
    &["info", "settings", "theme"],
)?;
assert_eq!(theme, Some(StoreValue::String("dark".to_string())));
```

---

## Gaming-Resistenz

### Tiefenlimits

```
┌─────────────────────────────────────────────────────────────────────┐
│                      DEPTH PROTECTION                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Max Depth: 5 (konfigurierbar)                                     │
│                                                                     │
│  Level 0: { title: String }                                        │
│  Level 1: { meta: { author: String } }                             │
│  Level 2: { meta: { tags: [String] } }                             │
│  Level 3: { a: { b: { c: Object } } }                              │
│  Level 4: { a: { b: { c: { d: Object } } } }                       │
│  Level 5: { a: { b: { c: { d: { e: String } } } } }  ← MAX         │
│  Level 6: REJECTED                                                 │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Array-Limits

```rust
max_array_length: 100  // Pro Liste
```

Bei `append_list()` wird geprüft:

```rust
if list.len() >= self.config.max_array_length {
    return Err(anyhow!("List exceeds maximum length"));
}
```

### Mana-basierte Kosten

| Operation       | Basis | Zusatzkosten              |
| --------------- | ----- | ------------------------- |
| Store erstellen | 100   | +10/Tiefe, +2×Komplexität |
| Put             | 5     | +10/Nested-Depth, +1/Feld |
| Schema Add      | 50    | +10×Typ-Komplexität       |
| Schema Breaking | ×4    | Multiplikator             |

### Challenge-Periode

Breaking Changes erfordern 7 Tage Wartezeit:

- Community kann ablehnen
- Verhindert übereilte destruktive Änderungen
- Trust-R ≥ 0.7 erforderlich

---

## Tests

### Testübersicht (25+ Tests)

| Testgruppe    | Tests                                                                                                                                                    |
| ------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Schema        | `test_schema_complexity`, `test_max_depth_validation`                                                                                                    |
| Prefix        | `test_prefix_builder`                                                                                                                                    |
| CRUD          | `test_store_creation_and_crud`, `test_personal_store_isolation`                                                                                          |
| Nested        | `test_nested_operations`, `test_list_operations`                                                                                                         |
| Validation    | `test_value_validation`                                                                                                                                  |
| Evolution     | `test_schema_evolution_add_field`, `test_schema_evolution_remove_field_pending`, `test_schema_evolution_rename_field`, `test_schema_evolution_add_index` |
| History       | `test_schema_history`, `test_schema_version_retrieval`                                                                                                   |
| Validation    | `test_schema_change_validation`                                                                                                                          |
| Compatibility | `test_schema_field_type_compatibility`                                                                                                                   |
| Migration     | `test_value_migration`, `test_lazy_value_migration`                                                                                                      |
| Costs         | `test_schema_evolution_mana_costs`                                                                                                                       |
| Rejection     | `test_reject_pending_schema`                                                                                                                             |

### Tests ausführen

```bash
# Alle RealmStorage-Tests
cargo test realm_storage --features local

# Mit Output
cargo test realm_storage --features local -- --nocapture
```

---

## Integration mit anderen Modulen

### ECL/ErynoaHost

```rust
// ErynoaHost nutzt RealmStorage für Store-Operationen
impl ErynoaHost {
    fn store_get(&self, store: &str, key: &str) -> Option<StoreValue> {
        self.realm_storage.get_shared(&self.realm_id, store, key).ok()?
    }

    fn store_put(&self, store: &str, key: &str, value: StoreValue) -> Result<()> {
        self.realm_storage.put_shared(&self.realm_id, store, key, value)
    }
}
```

### Blueprint Marketplace

```rust
// Blueprints definieren Store-Templates
pub struct BlueprintStore {
    pub name: String,
    pub schema: StoreSchema,  // → RealmStorage StoreSchema
    pub personal: bool,
    pub description: Option<String>,
    pub initial_data: Option<HashMap<String, StoreValue>>,
}
```

### Realm Domain

```rust
// Realm-Templates verwenden StoreSchema
use crate::local::{SchemaFieldType, StoreSchema, StoreTemplate};

let social_stores = vec![
    StoreTemplate {
        schema: StoreSchema::new("posts", false)
            .with_field("title", SchemaFieldType::String)
            .with_field("content", SchemaFieldType::String),
        optional: false,
        description: "Public posts".to_string(),
    },
];
```

---

## Erweiterungsmöglichkeiten

### Kurzfristig

1. **Batch-Operations**: Multi-Put/Multi-Get für Performance
2. **Streaming-Queries**: Iterator statt Vec für große Datenmengen
3. **TTL-Support**: Automatische Ablaufzeiten für Einträge

### Mittelfristig

1. **Full-Text-Search**: Tantivy-Integration für Textsuche
2. **Computed Fields**: Berechnete Felder im Schema
3. **Triggers**: Events bei Änderungen

### Langfristig

1. **CRDT-Integration**: Konfliktfreie Replikation
2. **P2P-Sync**: Realm-Daten über libp2p verteilen
3. **Encryption-at-Rest**: Verschlüsselte Personal-Stores

---

## Referenzen

- **World-Formula**: [WORLD-FORMULA.md](../../concept-v3/WORLD-FORMULA.md) (Ψ-Adaptation)
- **ECL Runtime**: [erynoa_host.rs](../../../backend/src/eclvm/erynoa_host.rs)
- **Blueprint System**: [BLUEPRINT-MARKETPLACE.md](./BLUEPRINT-MARKETPLACE.md)
- **Realm Domain**: [realm.rs](../../../backend/src/domain/realm.rs)

---

_Dokumentation erstellt: Februar 2026_
_Modul-Version: 1.0.0_
_Tests: 25+ passing_
