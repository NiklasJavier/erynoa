//! # Intelligente Realm-Speicherverwaltung
//!
//! Dynamische, hierarchische Speicherstruktur für Realm-basierte Daten.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                     REALM STORAGE LAYER                                     │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   Globale Partitionen (nur 3-5):                                           │
//! │   ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
//! │   │ realm_meta  │ │ realm_data  │ │realm_events │ │realm_content│          │
//! │   │ (Schemas,   │ │ (Stores,    │ │ (DAG-Events)│ │ (BLAKE3-CAS)│          │
//! │   │  Policies)  │ │  User Data) │ │             │ │             │          │
//! │   └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘          │
//! │                                                                             │
//! │   Intelligentes Prefixing für Isolation:                                   │
//! │   ┌─────────────────────────────────────────────────────────────────────┐  │
//! │   │ realm:{realm_id}:shared:store:{name}:{key}                          │  │
//! │   │ realm:{realm_id}:personal:{did}:store:{name}:{key}                  │  │
//! │   │ realm:{realm_id}:shared:store:{name}:_schema_v{version}             │  │
//! │   └─────────────────────────────────────────────────────────────────────┘  │
//! │                                                                             │
//! │   Schema-Evolution (Versionierung):                                        │
//! │   ┌─────────────────────────────────────────────────────────────────────┐  │
//! │   │ _schema_v1 → _schema_v2 → _schema_v3 (immutable history)            │  │
//! │   │ _schema_current → aktuelle Version                                  │  │
//! │   │ _schema_changelog → Liste aller Änderungen                          │  │
//! │   └─────────────────────────────────────────────────────────────────────┘  │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Grundprinzipien
//!
//! - **Wenige Partitionen**: Nur 3-5 globale Partitionen (kein Overhead bei Tausenden Realms)
//! - **Intelligentes Prefixing**: Keys kodieren Realm, Typ und Ownership
//! - **Lazy-Creation**: Stores werden erst bei erstem Schreiben aktiviert
//! - **Schema-Validierung**: Schema als Meta-Entry unter `_schema` Key
//! - **Schema-Evolution**: Versionierte Schemas mit Backward-Compatibility
//! - **Gaming-Resistenz**: Mana-Kosten, Trust-Checks, Limits

use crate::domain::{realm_id_from_name, RealmId, DID};
use anyhow::{anyhow, Result};
use fjall::{Keyspace, PartitionHandle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// ═══════════════════════════════════════════════════════════════════════════
// Schema-Definitionen
// ═══════════════════════════════════════════════════════════════════════════

/// Schema-Feldtyp für dynamische Stores
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SchemaFieldType {
    /// Einfache Typen
    String,
    Number,
    Bool,
    Did,
    Timestamp,
    Bytes,

    /// Komplexe Typen
    List {
        item_type: Box<SchemaFieldType>,
    },
    Object {
        fields: HashMap<String, SchemaFieldType>,
    },
    Reference {
        target_store: String,
    },
    Optional {
        inner: Box<SchemaFieldType>,
    },
}

impl SchemaFieldType {
    /// Berechne Komplexität für Mana-Kosten
    pub fn complexity(&self) -> u64 {
        match self {
            SchemaFieldType::String
            | SchemaFieldType::Number
            | SchemaFieldType::Bool
            | SchemaFieldType::Did
            | SchemaFieldType::Timestamp
            | SchemaFieldType::Bytes => 1,

            SchemaFieldType::List { item_type } => 2 + item_type.complexity(),
            SchemaFieldType::Object { fields } => {
                2 + fields.values().map(|f| f.complexity()).sum::<u64>()
            }
            SchemaFieldType::Reference { .. } => 2,
            SchemaFieldType::Optional { inner } => 1 + inner.complexity(),
        }
    }

    /// Berechne Verschachtelungstiefe
    pub fn depth(&self) -> u32 {
        match self {
            SchemaFieldType::String
            | SchemaFieldType::Number
            | SchemaFieldType::Bool
            | SchemaFieldType::Did
            | SchemaFieldType::Timestamp
            | SchemaFieldType::Bytes
            | SchemaFieldType::Reference { .. } => 0,

            SchemaFieldType::List { item_type } => 1 + item_type.depth(),
            SchemaFieldType::Object { fields } => {
                1 + fields.values().map(|f| f.depth()).max().unwrap_or(0)
            }
            SchemaFieldType::Optional { inner } => inner.depth(),
        }
    }
}

/// Schema für einen dynamischen Store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreSchema {
    /// Name des Stores
    pub name: String,
    /// Version des Schemas
    pub version: u32,
    /// Felder und ihre Typen
    pub fields: HashMap<String, SchemaFieldType>,
    /// Ist der Store persönlich (pro DID) oder shared?
    pub personal: bool,
    /// Maximale Anzahl Einträge (0 = unbegrenzt)
    pub max_entries: u64,
    /// Indices für Performance
    pub indices: Vec<String>,
}

impl StoreSchema {
    /// Erstelle neues Schema
    pub fn new(name: impl Into<String>, personal: bool) -> Self {
        Self {
            name: name.into(),
            version: 1,
            fields: HashMap::new(),
            personal,
            max_entries: 0,
            indices: Vec::new(),
        }
    }

    /// Builder: Füge Feld hinzu
    pub fn with_field(mut self, name: impl Into<String>, field_type: SchemaFieldType) -> Self {
        self.fields.insert(name.into(), field_type);
        self
    }

    /// Builder: Füge Index hinzu
    pub fn with_index(mut self, field_name: impl Into<String>) -> Self {
        self.indices.push(field_name.into());
        self
    }

    /// Builder: Setze Max-Einträge
    pub fn with_max_entries(mut self, max: u64) -> Self {
        self.max_entries = max;
        self
    }

    /// Gesamtkomplexität für Mana-Berechnung
    pub fn complexity(&self) -> u64 {
        self.fields.values().map(|f| f.complexity()).sum::<u64>() + self.indices.len() as u64 * 10
    }

    /// Maximale Tiefe
    pub fn max_depth(&self) -> u32 {
        self.fields.values().map(|f| f.depth()).max().unwrap_or(0)
    }

    /// Validiere Tiefe gegen Limit
    pub fn validate_depth(&self, max_allowed: u32) -> Result<()> {
        let depth = self.max_depth();
        if depth > max_allowed {
            return Err(anyhow!(
                "Schema depth {} exceeds maximum allowed depth {}",
                depth,
                max_allowed
            ));
        }
        Ok(())
    }

    /// Prüfe Kompatibilität mit anderem Schema
    pub fn is_compatible_with(&self, other: &StoreSchema) -> bool {
        // Gleicher Name und Typ erforderlich
        if self.name != other.name || self.personal != other.personal {
            return false;
        }
        // Alle existierenden Felder im anderen Schema müssen kompatibel sein
        for (field_name, field_type) in &other.fields {
            if let Some(my_type) = self.fields.get(field_name) {
                if !field_type.is_compatible_with(my_type) {
                    return false;
                }
            }
            // Neue Felder im anderen Schema sind OK (Erweiterung)
        }
        true
    }

    /// Erstelle neue Version mit Änderungen
    pub fn evolve(&self, changes: &[SchemaChange]) -> Result<StoreSchema> {
        let mut new_schema = self.clone();
        new_schema.version += 1;

        for change in changes {
            match change {
                SchemaChange::AddField {
                    name,
                    field_type,
                    default,
                } => {
                    if new_schema.fields.contains_key(name) {
                        return Err(anyhow!("Field '{}' already exists", name));
                    }
                    new_schema.fields.insert(name.clone(), field_type.clone());
                    // Default wird für Migration verwendet, nicht im Schema gespeichert
                    let _ = default;
                }
                SchemaChange::RemoveField { name } => {
                    if !new_schema.fields.contains_key(name) {
                        return Err(anyhow!("Field '{}' does not exist", name));
                    }
                    new_schema.fields.remove(name);
                }
                SchemaChange::ModifyField { name, new_type } => {
                    if !new_schema.fields.contains_key(name) {
                        return Err(anyhow!("Field '{}' does not exist", name));
                    }
                    new_schema.fields.insert(name.clone(), new_type.clone());
                }
                SchemaChange::RenameField { old_name, new_name } => {
                    if let Some(field_type) = new_schema.fields.remove(old_name) {
                        new_schema.fields.insert(new_name.clone(), field_type);
                    } else {
                        return Err(anyhow!("Field '{}' does not exist", old_name));
                    }
                }
                SchemaChange::AddIndex { field_name } => {
                    if !new_schema.indices.contains(field_name) {
                        new_schema.indices.push(field_name.clone());
                    }
                }
                SchemaChange::RemoveIndex { field_name } => {
                    new_schema.indices.retain(|f| f != field_name);
                }
            }
        }

        Ok(new_schema)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Schema-Evolution: Dynamische Änderungen
// ═══════════════════════════════════════════════════════════════════════════

/// Art der Schema-Änderung
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SchemaChange {
    /// Neues Feld hinzufügen (backward-compatible)
    AddField {
        name: String,
        field_type: SchemaFieldType,
        #[serde(default)]
        default: Option<StoreValue>,
    },
    /// Feld entfernen (breaking change!)
    RemoveField { name: String },
    /// Feldtyp ändern (kompatibilität prüfen!)
    ModifyField {
        name: String,
        new_type: SchemaFieldType,
    },
    /// Feld umbenennen
    RenameField { old_name: String, new_name: String },
    /// Index hinzufügen
    AddIndex { field_name: String },
    /// Index entfernen
    RemoveIndex { field_name: String },
}

impl SchemaChange {
    /// Prüfe ob diese Änderung ein Breaking Change ist
    pub fn is_breaking(&self) -> bool {
        matches!(
            self,
            SchemaChange::RemoveField { .. }
                | SchemaChange::ModifyField { .. }
                | SchemaChange::RenameField { .. }
        )
    }

    /// Berechne Mana-Kosten für diese Änderung
    pub fn mana_cost(&self) -> u64 {
        match self {
            SchemaChange::AddField { field_type, .. } => 50 + field_type.complexity() * 10,
            SchemaChange::RemoveField { .. } => 200, // Breaking = teuer
            SchemaChange::ModifyField { new_type, .. } => 300 + new_type.complexity() * 10,
            SchemaChange::RenameField { .. } => 100,
            SchemaChange::AddIndex { .. } => 150, // Index-Aufbau teuer
            SchemaChange::RemoveIndex { .. } => 50,
        }
    }
}

/// Eintrag im Schema-Changelog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChangelogEntry {
    /// Version nach der Änderung
    pub version: u32,
    /// Zeitstempel der Änderung (Unix Seconds)
    pub timestamp: u64,
    /// DID des Änderenden
    pub changed_by: String,
    /// Durchgeführte Änderungen
    pub changes: Vec<SchemaChange>,
    /// Beschreibung der Änderung
    pub description: String,
    /// War eine Challenge-Periode erforderlich?
    pub required_challenge: bool,
    /// Status der Änderung
    pub status: SchemaChangeStatus,
}

/// Status einer Schema-Änderung
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SchemaChangeStatus {
    /// Änderung aktiv und angewendet
    Active,
    /// In Challenge-Periode (bei Breaking Changes)
    Pending { challenge_ends: u64 },
    /// Abgebrochen (z.B. durch zu viele Gegenstimmen)
    Rejected { reason: String },
    /// Durch neuere Version ersetzt
    Superseded { by_version: u32 },
}

/// Ergebnis einer Schema-Evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaEvolutionResult {
    /// Neue Schema-Version
    pub new_version: u32,
    /// Enthält Breaking Changes?
    pub is_breaking: bool,
    /// Status der Änderung
    pub status: SchemaChangeStatus,
    /// Mana-Kosten der Änderung
    pub mana_cost: u64,
}

/// Vollständige Schema-Historie für einen Store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaHistory {
    /// Store-Name
    pub store_name: String,
    /// Aktuelle Version
    pub current_version: u32,
    /// Changelog aller Änderungen
    pub changelog: Vec<SchemaChangelogEntry>,
}

impl SchemaHistory {
    pub fn new(store_name: impl Into<String>) -> Self {
        Self {
            store_name: store_name.into(),
            current_version: 1,
            changelog: Vec::new(),
        }
    }

    /// Füge neue Änderung zum Changelog hinzu
    pub fn add_entry(&mut self, entry: SchemaChangelogEntry) {
        self.current_version = entry.version;
        self.changelog.push(entry);
    }

    /// Hole Changelog-Eintrag für eine Version
    pub fn get_entry(&self, version: u32) -> Option<&SchemaChangelogEntry> {
        self.changelog.iter().find(|e| e.version == version)
    }
}

/// Template für automatische Store-Erstellung beim Join
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreTemplate {
    /// Schema des Stores
    pub schema: StoreSchema,
    /// Ist der Store optional?
    pub optional: bool,
    /// Beschreibung für Nutzer
    pub description: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// Kompatibilitätsprüfung für Typ-Änderungen
// ═══════════════════════════════════════════════════════════════════════════

impl SchemaFieldType {
    /// Prüfe ob dieser Typ kompatibel mit einem anderen ist
    ///
    /// Kompatibel bedeutet: Daten vom alten Typ können als neuer Typ gelesen werden.
    pub fn is_compatible_with(&self, other: &SchemaFieldType) -> bool {
        match (self, other) {
            // Gleiche Typen sind kompatibel
            (a, b) if std::mem::discriminant(a) == std::mem::discriminant(b) => {
                match (a, b) {
                    // Rekursiv für komplexe Typen
                    (
                        SchemaFieldType::List { item_type: a_item },
                        SchemaFieldType::List { item_type: b_item },
                    ) => a_item.is_compatible_with(b_item),
                    (
                        SchemaFieldType::Object { fields: a_fields },
                        SchemaFieldType::Object { fields: b_fields },
                    ) => {
                        // Alle Felder in b müssen in a kompatibel sein
                        b_fields.iter().all(|(k, v)| {
                            a_fields
                                .get(k)
                                .map(|av| av.is_compatible_with(v))
                                .unwrap_or(true)
                        })
                    }
                    (
                        SchemaFieldType::Optional { inner: a_inner },
                        SchemaFieldType::Optional { inner: b_inner },
                    ) => a_inner.is_compatible_with(b_inner),
                    _ => true, // Einfache Typen sind gleich
                }
            }
            // T → Optional<T> ist kompatibel (Erweiterung)
            (SchemaFieldType::Optional { inner }, other_type) => {
                inner.is_compatible_with(other_type)
            }
            // Optional<T> → T ist NICHT kompatibel (könnte null sein)
            (_, SchemaFieldType::Optional { .. }) => false,
            // Number kann zu String werden (Formatierung)
            (SchemaFieldType::String, SchemaFieldType::Number) => true,
            // Alles andere ist inkompatibel
            _ => false,
        }
    }

    /// Migriere einen Wert von einem Typ zu diesem Typ
    pub fn migrate_value(
        &self,
        value: StoreValue,
        from_type: &SchemaFieldType,
    ) -> Result<StoreValue> {
        match (self, from_type, &value) {
            // Gleiche Typen: keine Migration nötig
            (a, b, _) if std::mem::discriminant(a) == std::mem::discriminant(b) => Ok(value),

            // T → Optional<T>: Wrap in Some
            (SchemaFieldType::Optional { .. }, _, v) => Ok(v.clone()),

            // Number → String: Formatieren
            (SchemaFieldType::String, SchemaFieldType::Number, StoreValue::Number(n)) => {
                Ok(StoreValue::String(n.to_string()))
            }

            // Fehlende Felder in Object: Default hinzufügen
            (
                SchemaFieldType::Object { fields: new_fields },
                SchemaFieldType::Object { fields: old_fields },
                StoreValue::Object(obj),
            ) => {
                let mut new_obj = obj.clone();
                for (field_name, field_type) in new_fields {
                    if !old_fields.contains_key(field_name) {
                        // Neues Feld: Default-Wert einfügen
                        new_obj.insert(field_name.clone(), field_type.default_value());
                    }
                }
                Ok(StoreValue::Object(new_obj))
            }

            _ => Err(anyhow!(
                "Cannot migrate value from {:?} to {:?}",
                from_type,
                self
            )),
        }
    }

    /// Erzeuge Default-Wert für diesen Typ
    pub fn default_value(&self) -> StoreValue {
        match self {
            SchemaFieldType::String => StoreValue::String(String::new()),
            SchemaFieldType::Number => StoreValue::Number(0.0),
            SchemaFieldType::Bool => StoreValue::Bool(false),
            SchemaFieldType::Did => StoreValue::String(String::new()),
            SchemaFieldType::Timestamp => StoreValue::Number(0.0),
            SchemaFieldType::Bytes => StoreValue::List(vec![]),
            SchemaFieldType::List { .. } => StoreValue::List(vec![]),
            SchemaFieldType::Object { fields } => {
                let mut obj = HashMap::new();
                for (name, field_type) in fields {
                    obj.insert(name.clone(), field_type.default_value());
                }
                StoreValue::Object(obj)
            }
            SchemaFieldType::Reference { .. } => StoreValue::Null,
            SchemaFieldType::Optional { .. } => StoreValue::Null,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Store-Optionen und Konfiguration
// ═══════════════════════════════════════════════════════════════════════════

/// Optionen für Store-Erstellung
#[derive(Debug, Clone, Default)]
pub struct StoreOptions {
    /// Persönlicher Store (pro DID isoliert)
    pub personal: bool,
    /// Maximale Einträge
    pub max_entries: u64,
    /// Indices erstellen
    pub indices: Vec<String>,
    /// Beschreibung
    pub description: String,
}

/// Konfiguration für RealmStorage
#[derive(Debug, Clone)]
pub struct RealmStorageConfig {
    /// Maximale Verschachtelungstiefe
    pub max_depth: u32,
    /// Maximale Array-Länge
    pub max_array_length: usize,
    /// Basis-Mana-Kosten für Store-Erstellung
    pub base_mana_cost: u64,
    /// Mana-Kosten pro Nested-Level
    pub mana_per_depth: u64,
    /// Mana-Kosten pro Put-Operation
    pub mana_per_put: u64,
    /// Maximale Stores pro Realm
    pub max_stores_per_realm: usize,
    /// Maximale persönliche Stores pro User
    pub max_personal_stores_per_user: usize,
    /// Challenge-Periode für Breaking Changes (in Sekunden)
    pub breaking_change_challenge_period: u64,
    /// Minimaler Trust-R für Schema-Änderungen
    pub min_trust_for_schema_change: f64,
    /// Mana-Multiplikator für Breaking Changes
    pub breaking_change_mana_multiplier: u64,
}

impl Default for RealmStorageConfig {
    fn default() -> Self {
        Self {
            max_depth: 5,
            max_array_length: 100,
            base_mana_cost: 100,
            mana_per_depth: 10,
            mana_per_put: 5,
            max_stores_per_realm: 100,
            max_personal_stores_per_user: 20,
            breaking_change_challenge_period: 7 * 24 * 60 * 60, // 7 Tage
            min_trust_for_schema_change: 0.7,
            breaking_change_mana_multiplier: 4,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Dynamischer Value-Typ für gespeicherte Daten
// ═══════════════════════════════════════════════════════════════════════════

/// Dynamischer Wert für Store-Operationen
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StoreValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    // List muss VOR Object und Bytes kommen, da [] sonst als Bytes(Vec<u8>) oder Object deserialisiert werden könnte
    List(Vec<StoreValue>),
    Object(HashMap<String, StoreValue>),
    // Spezielle Typen (werden als ihre Basis-Typen serialisiert)
    // Diese werden nur intern verwendet und bei der Deserialisierung zu den Basis-Typen
    #[serde(skip_deserializing)]
    Did(String),
    #[serde(skip_deserializing)]
    Timestamp(u64),
    #[serde(skip_deserializing)]
    Bytes(Vec<u8>),
}

impl StoreValue {
    /// Validiere Wert gegen Schema-Typ
    pub fn validate(&self, schema_type: &SchemaFieldType) -> Result<()> {
        match (self, schema_type) {
            (StoreValue::Null, SchemaFieldType::Optional { .. }) => Ok(()),
            (StoreValue::String(_), SchemaFieldType::String) => Ok(()),
            (StoreValue::Number(_), SchemaFieldType::Number) => Ok(()),
            (StoreValue::Bool(_), SchemaFieldType::Bool) => Ok(()),
            (StoreValue::Did(_), SchemaFieldType::Did) => Ok(()),
            (StoreValue::Timestamp(_), SchemaFieldType::Timestamp) => Ok(()),
            (StoreValue::Bytes(_), SchemaFieldType::Bytes) => Ok(()),
            (StoreValue::String(_), SchemaFieldType::Reference { .. }) => Ok(()),

            (StoreValue::List(items), SchemaFieldType::List { item_type }) => {
                for item in items {
                    item.validate(item_type)?;
                }
                Ok(())
            }

            (StoreValue::Object(obj), SchemaFieldType::Object { fields }) => {
                for (key, field_type) in fields {
                    if let Some(value) = obj.get(key) {
                        value.validate(field_type)?;
                    } else if !matches!(field_type, SchemaFieldType::Optional { .. }) {
                        return Err(anyhow!("Missing required field: {}", key));
                    }
                }
                Ok(())
            }

            (value, SchemaFieldType::Optional { inner }) => {
                if matches!(value, StoreValue::Null) {
                    Ok(())
                } else {
                    value.validate(inner)
                }
            }

            _ => Err(anyhow!(
                "Type mismatch: got {:?}, expected {:?}",
                std::mem::discriminant(self),
                schema_type
            )),
        }
    }

    /// Konvertiere zu JSON-Bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    /// Parse von JSON-Bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Prefix-Builder für intelligente Key-Konstruktion
// ═══════════════════════════════════════════════════════════════════════════

/// Builder für intelligente Key-Prefixes
pub struct PrefixBuilder {
    realm_id: String,
    store_type: StoreType,
    owner_did: Option<String>,
    store_name: String,
}

#[derive(Debug, Clone, Copy)]
pub enum StoreType {
    Shared,
    Personal,
}

impl PrefixBuilder {
    /// Neuer Prefix für shared Store
    pub fn shared(realm_id: &RealmId, store_name: &str) -> Self {
        Self {
            realm_id: realm_id.0.clone(),
            store_type: StoreType::Shared,
            owner_did: None,
            store_name: store_name.to_string(),
        }
    }

    /// Neuer Prefix für personal Store
    pub fn personal(realm_id: &RealmId, owner_did: &DID, store_name: &str) -> Self {
        Self {
            realm_id: realm_id.0.clone(),
            store_type: StoreType::Personal,
            owner_did: Some(owner_did.unique_id.clone()),
            store_name: store_name.to_string(),
        }
    }

    /// Basis-Prefix für Store (ohne konkreten Key)
    pub fn store_prefix(&self) -> String {
        match self.store_type {
            StoreType::Shared => {
                format!("realm:{}:shared:store:{}", self.realm_id, self.store_name)
            }
            StoreType::Personal => {
                format!(
                    "realm:{}:personal:{}:store:{}",
                    self.realm_id,
                    self.owner_did.as_ref().unwrap_or(&String::new()),
                    self.store_name
                )
            }
        }
    }

    /// Vollständiger Key
    pub fn key(&self, key: &str) -> String {
        format!("{}:{}", self.store_prefix(), key)
    }

    /// Schema-Key
    pub fn schema_key(&self) -> String {
        format!("{}:_schema", self.store_prefix())
    }

    /// Index-Prefix
    pub fn index_prefix(&self, field_name: &str) -> String {
        format!("{}:_idx:{}", self.store_prefix(), field_name)
    }

    /// Index-Entry
    pub fn index_key(&self, field_name: &str, value: &str, entry_key: &str) -> String {
        format!("{}:{}:{}", self.index_prefix(field_name), value, entry_key)
    }

    /// Nested-Key für tiefere Strukturen
    pub fn nested_key(&self, key: &str, path: &[&str]) -> String {
        let path_str = path.join(":");
        format!("{}:{}:{}", self.store_prefix(), key, path_str)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Realm Storage Manager
// ═══════════════════════════════════════════════════════════════════════════

/// Haupt-Storage-Manager für Realm-Daten
pub struct RealmStorage {
    /// Referenz auf den Keyspace
    keyspace: Arc<Keyspace>,

    /// Partition für Metadaten (Schemas, Policies)
    pub meta: PartitionHandle,

    /// Partition für dynamische Daten
    pub data: PartitionHandle,

    /// Partition für Indices (optional)
    pub indices: Option<PartitionHandle>,

    /// Konfiguration
    config: RealmStorageConfig,

    /// Cache für Schemas (Realm:Store -> Schema)
    schema_cache: parking_lot::RwLock<HashMap<String, StoreSchema>>,
}

impl RealmStorage {
    /// Erstelle neuen RealmStorage
    pub fn new(keyspace: &Arc<Keyspace>, config: RealmStorageConfig) -> Result<Self> {
        let meta = keyspace.open_partition("realm_meta", Default::default())?;
        let data = keyspace.open_partition("realm_data", Default::default())?;
        let indices = keyspace
            .open_partition("realm_indices", Default::default())
            .ok();

        Ok(Self {
            keyspace: Arc::clone(keyspace),
            meta,
            data,
            indices,
            config,
            schema_cache: parking_lot::RwLock::new(HashMap::new()),
        })
    }

    /// Berechne Mana-Kosten für Store-Erstellung
    pub fn calculate_create_cost(&self, schema: &StoreSchema) -> u64 {
        let depth_cost = schema.max_depth() as u64 * self.config.mana_per_depth;
        let complexity_cost = schema.complexity() * 2;
        self.config.base_mana_cost + depth_cost + complexity_cost
    }

    /// Berechne Mana-Kosten für Put-Operation
    pub fn calculate_put_cost(&self, value: &StoreValue, nested_depth: u32) -> u64 {
        let depth_cost = nested_depth as u64 * self.config.mana_per_depth;
        let size_cost = match value {
            StoreValue::List(items) => items.len() as u64,
            StoreValue::Object(fields) => fields.len() as u64,
            _ => 0,
        };
        self.config.mana_per_put + depth_cost + size_cost
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Store-Erstellung
    // ─────────────────────────────────────────────────────────────────────────

    /// Erstelle neuen Store (shared oder personal)
    pub fn create_store(
        &self,
        realm_id: &RealmId,
        sender_did: &DID,
        schema: StoreSchema,
    ) -> Result<()> {
        // Validiere Tiefe
        schema.validate_depth(self.config.max_depth)?;

        let prefix = if schema.personal {
            PrefixBuilder::personal(realm_id, sender_did, &schema.name)
        } else {
            PrefixBuilder::shared(realm_id, &schema.name)
        };

        // Prüfe ob Store schon existiert
        let schema_key = prefix.schema_key();
        if self.meta.get(&schema_key)?.is_some() {
            return Err(anyhow!("Store '{}' already exists", schema.name));
        }

        // Speichere Schema
        let schema_bytes = serde_json::to_vec(&schema)?;
        self.meta.insert(&schema_key, &schema_bytes)?;

        // Cache aktualisieren
        let cache_key = format!("{}:{}", realm_id.0, schema.name);
        self.schema_cache.write().insert(cache_key, schema.clone());

        tracing::info!(
            realm = %realm_id,
            store = %schema.name,
            personal = schema.personal,
            "Store created"
        );

        Ok(())
    }

    /// Hole Schema für Store
    pub fn get_schema(
        &self,
        realm_id: &RealmId,
        store_name: &str,
        sender_did: Option<&DID>,
    ) -> Result<StoreSchema> {
        let cache_key = format!("{}:{}", realm_id.0, store_name);

        // Cache-Check
        if let Some(schema) = self.schema_cache.read().get(&cache_key) {
            return Ok(schema.clone());
        }

        // Versuche shared Store
        let shared_prefix = PrefixBuilder::shared(realm_id, store_name);
        if let Some(bytes) = self.meta.get(shared_prefix.schema_key())? {
            let schema: StoreSchema = serde_json::from_slice(&bytes)?;
            self.schema_cache.write().insert(cache_key, schema.clone());
            return Ok(schema);
        }

        // Versuche personal Store wenn DID gegeben
        if let Some(did) = sender_did {
            let personal_prefix = PrefixBuilder::personal(realm_id, did, store_name);
            if let Some(bytes) = self.meta.get(personal_prefix.schema_key())? {
                let schema: StoreSchema = serde_json::from_slice(&bytes)?;
                return Ok(schema);
            }
        }

        Err(anyhow!(
            "Store '{}' not found in realm '{}'",
            store_name,
            realm_id
        ))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Schema-Evolution: Dynamische Änderungen
    // ─────────────────────────────────────────────────────────────────────────

    /// Berechne Mana-Kosten für Schema-Änderungen
    pub fn calculate_evolution_cost(&self, changes: &[SchemaChange]) -> u64 {
        let base_cost: u64 = changes.iter().map(|c| c.mana_cost()).sum();
        let has_breaking = changes.iter().any(|c| c.is_breaking());

        if has_breaking {
            base_cost * self.config.breaking_change_mana_multiplier
        } else {
            base_cost
        }
    }

    /// Prüfe ob Änderungen Breaking Changes enthalten
    pub fn has_breaking_changes(changes: &[SchemaChange]) -> bool {
        changes.iter().any(|c| c.is_breaking())
    }

    /// Evolve ein Schema mit den gegebenen Änderungen
    ///
    /// Bei Breaking Changes wird eine Challenge-Periode gestartet.
    /// Bei Non-Breaking Changes wird sofort angewendet.
    pub fn evolve_schema(
        &self,
        realm_id: &RealmId,
        sender_did: &DID,
        store_name: &str,
        changes: Vec<SchemaChange>,
        description: String,
    ) -> Result<SchemaEvolutionResult> {
        // Hole aktuelles Schema
        let current_schema = self.get_schema(realm_id, store_name, Some(sender_did))?;

        // Validiere Änderungen
        for change in &changes {
            self.validate_schema_change(&current_schema, change)?;
        }

        // Erstelle neues Schema
        let new_schema = current_schema.evolve(&changes)?;

        // Validiere neues Schema
        new_schema.validate_depth(self.config.max_depth)?;

        let has_breaking = Self::has_breaking_changes(&changes);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Erstelle Changelog-Eintrag
        let status = if has_breaking {
            SchemaChangeStatus::Pending {
                challenge_ends: timestamp + self.config.breaking_change_challenge_period,
            }
        } else {
            SchemaChangeStatus::Active
        };

        let changelog_entry = SchemaChangelogEntry {
            version: new_schema.version,
            timestamp,
            changed_by: sender_did.to_uri(),
            changes: changes.clone(),
            description,
            required_challenge: has_breaking,
            status: status.clone(),
        };

        // Speichere versioniertes Schema
        let prefix = if current_schema.personal {
            PrefixBuilder::personal(realm_id, sender_did, store_name)
        } else {
            PrefixBuilder::shared(realm_id, store_name)
        };

        let versioned_schema_key =
            format!("{}:_schema_v{}", prefix.store_prefix(), new_schema.version);
        let schema_bytes = serde_json::to_vec(&new_schema)?;
        self.meta.insert(&versioned_schema_key, &schema_bytes)?;

        // Speichere Changelog-Eintrag
        let changelog_key = format!(
            "{}:_changelog_v{}",
            prefix.store_prefix(),
            new_schema.version
        );
        let changelog_bytes = serde_json::to_vec(&changelog_entry)?;
        self.meta.insert(&changelog_key, &changelog_bytes)?;

        // Bei Non-Breaking: Aktiviere sofort
        if !has_breaking {
            // Update Haupt-Schema
            let schema_key = prefix.schema_key();
            self.meta.insert(&schema_key, &schema_bytes)?;

            // Cache aktualisieren
            let cache_key = format!("{}:{}", realm_id.0, store_name);
            self.schema_cache
                .write()
                .insert(cache_key, new_schema.clone());
        }

        tracing::info!(
            realm = %realm_id,
            store = %store_name,
            version = new_schema.version,
            breaking = has_breaking,
            "Schema evolved"
        );

        Ok(SchemaEvolutionResult {
            new_version: new_schema.version,
            is_breaking: has_breaking,
            status,
            mana_cost: self.calculate_evolution_cost(&changes),
        })
    }

    /// Aktiviere eine pending Schema-Änderung nach Challenge-Periode
    pub fn activate_pending_schema(
        &self,
        realm_id: &RealmId,
        sender_did: &DID,
        store_name: &str,
        version: u32,
    ) -> Result<()> {
        let current_schema = self.get_schema(realm_id, store_name, Some(sender_did))?;

        let prefix = if current_schema.personal {
            PrefixBuilder::personal(realm_id, sender_did, store_name)
        } else {
            PrefixBuilder::shared(realm_id, store_name)
        };

        // Hole pending Schema
        let versioned_key = format!("{}:_schema_v{}", prefix.store_prefix(), version);
        let schema_slice = self
            .meta
            .get(&versioned_key)?
            .ok_or_else(|| anyhow!("Schema version {} not found", version))?;
        let pending_schema: StoreSchema = serde_json::from_slice(&schema_slice)?;
        // Konvertiere zu Vec für späteren Insert
        let schema_bytes: Vec<u8> = schema_slice.to_vec();

        // Hole Changelog
        let changelog_key = format!("{}:_changelog_v{}", prefix.store_prefix(), version);
        let changelog_bytes = self
            .meta
            .get(&changelog_key)?
            .ok_or_else(|| anyhow!("Changelog for version {} not found", version))?;
        let mut changelog_entry: SchemaChangelogEntry = serde_json::from_slice(&changelog_bytes)?;

        // Prüfe ob Challenge-Periode abgelaufen
        if let SchemaChangeStatus::Pending { challenge_ends } = changelog_entry.status {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            if now < challenge_ends {
                return Err(anyhow!(
                    "Challenge period not ended. {} seconds remaining.",
                    challenge_ends - now
                ));
            }
        } else {
            return Err(anyhow!("Schema version {} is not pending", version));
        }

        // Aktiviere Schema
        changelog_entry.status = SchemaChangeStatus::Active;
        let updated_changelog = serde_json::to_vec(&changelog_entry)?;
        self.meta.insert(&changelog_key, &updated_changelog)?;

        // Update Haupt-Schema
        let schema_key = prefix.schema_key();
        self.meta.insert(&schema_key, &schema_bytes)?;

        // Cache aktualisieren
        let cache_key = format!("{}:{}", realm_id.0, store_name);
        self.schema_cache.write().insert(cache_key, pending_schema);

        tracing::info!(
            realm = %realm_id,
            store = %store_name,
            version = version,
            "Pending schema activated"
        );

        Ok(())
    }

    /// Lehne eine pending Schema-Änderung ab
    pub fn reject_pending_schema(
        &self,
        realm_id: &RealmId,
        sender_did: &DID,
        store_name: &str,
        version: u32,
        reason: String,
    ) -> Result<()> {
        let current_schema = self.get_schema(realm_id, store_name, Some(sender_did))?;

        let prefix = if current_schema.personal {
            PrefixBuilder::personal(realm_id, sender_did, store_name)
        } else {
            PrefixBuilder::shared(realm_id, store_name)
        };

        // Hole Changelog
        let changelog_key = format!("{}:_changelog_v{}", prefix.store_prefix(), version);
        let changelog_bytes = self
            .meta
            .get(&changelog_key)?
            .ok_or_else(|| anyhow!("Changelog for version {} not found", version))?;
        let mut changelog_entry: SchemaChangelogEntry = serde_json::from_slice(&changelog_bytes)?;

        if !matches!(changelog_entry.status, SchemaChangeStatus::Pending { .. }) {
            return Err(anyhow!("Schema version {} is not pending", version));
        }

        // Setze Status auf Rejected
        changelog_entry.status = SchemaChangeStatus::Rejected {
            reason: reason.clone(),
        };
        let updated_changelog = serde_json::to_vec(&changelog_entry)?;
        self.meta.insert(&changelog_key, &updated_changelog)?;

        tracing::info!(
            realm = %realm_id,
            store = %store_name,
            version = version,
            reason = %reason,
            "Schema change rejected"
        );

        Ok(())
    }

    /// Hole Schema-Historie für einen Store
    pub fn get_schema_history(
        &self,
        realm_id: &RealmId,
        store_name: &str,
        sender_did: Option<&DID>,
    ) -> Result<SchemaHistory> {
        let current_schema = self.get_schema(realm_id, store_name, sender_did)?;

        let prefix = if current_schema.personal {
            sender_did
                .map(|did| PrefixBuilder::personal(realm_id, did, store_name))
                .ok_or_else(|| anyhow!("DID required for personal store history"))?
        } else {
            PrefixBuilder::shared(realm_id, store_name)
        };

        let mut history = SchemaHistory::new(store_name);
        history.current_version = current_schema.version;

        // Sammle alle Changelog-Einträge
        let changelog_prefix = format!("{}:_changelog_v", prefix.store_prefix());
        for entry in self.meta.prefix(&changelog_prefix) {
            let (_, value) = entry?;
            if let Ok(changelog_entry) = serde_json::from_slice::<SchemaChangelogEntry>(&value) {
                history.changelog.push(changelog_entry);
            }
        }

        // Sortiere nach Version
        history.changelog.sort_by_key(|e| e.version);

        Ok(history)
    }

    /// Hole eine spezifische Schema-Version
    pub fn get_schema_version(
        &self,
        realm_id: &RealmId,
        store_name: &str,
        version: u32,
        sender_did: Option<&DID>,
    ) -> Result<StoreSchema> {
        // Version 1 = originales Schema
        if version == 1 {
            // Versuche _schema_v1, falls nicht vorhanden verwende aktuelles wenn v1
            let current = self.get_schema(realm_id, store_name, sender_did)?;
            if current.version == 1 {
                return Ok(current);
            }
        }

        let current = self.get_schema(realm_id, store_name, sender_did)?;
        let prefix = if current.personal {
            sender_did
                .map(|did| PrefixBuilder::personal(realm_id, did, store_name))
                .ok_or_else(|| anyhow!("DID required for personal store"))?
        } else {
            PrefixBuilder::shared(realm_id, store_name)
        };

        let versioned_key = format!("{}:_schema_v{}", prefix.store_prefix(), version);
        let bytes = self
            .meta
            .get(&versioned_key)?
            .ok_or_else(|| anyhow!("Schema version {} not found", version))?;

        Ok(serde_json::from_slice(&bytes)?)
    }

    /// Validiere eine Schema-Änderung
    fn validate_schema_change(&self, schema: &StoreSchema, change: &SchemaChange) -> Result<()> {
        match change {
            SchemaChange::AddField {
                name, field_type, ..
            } => {
                if schema.fields.contains_key(name) {
                    return Err(anyhow!("Field '{}' already exists", name));
                }
                // Prüfe Tiefe des neuen Felds
                if field_type.depth() + 1 > self.config.max_depth {
                    return Err(anyhow!("New field would exceed max depth"));
                }
            }
            SchemaChange::RemoveField { name } => {
                if !schema.fields.contains_key(name) {
                    return Err(anyhow!("Field '{}' does not exist", name));
                }
            }
            SchemaChange::ModifyField { name, new_type } => {
                let old_type = schema
                    .fields
                    .get(name)
                    .ok_or_else(|| anyhow!("Field '{}' does not exist", name))?;

                // Warnung bei inkompatiblen Änderungen (aber nicht blockieren)
                if !new_type.is_compatible_with(old_type) {
                    tracing::warn!(
                        field = %name,
                        "Potentially incompatible type change"
                    );
                }
            }
            SchemaChange::RenameField { old_name, new_name } => {
                if !schema.fields.contains_key(old_name) {
                    return Err(anyhow!("Field '{}' does not exist", old_name));
                }
                if schema.fields.contains_key(new_name) {
                    return Err(anyhow!("Field '{}' already exists", new_name));
                }
            }
            SchemaChange::AddIndex { field_name } => {
                if !schema.fields.contains_key(field_name) {
                    return Err(anyhow!("Cannot index non-existent field '{}'", field_name));
                }
                if schema.indices.contains(field_name) {
                    return Err(anyhow!("Index on '{}' already exists", field_name));
                }
            }
            SchemaChange::RemoveIndex { field_name } => {
                if !schema.indices.contains(field_name) {
                    return Err(anyhow!("Index on '{}' does not exist", field_name));
                }
            }
        }
        Ok(())
    }

    /// Migriere einen Wert von einer Schema-Version zu einer anderen (Lazy)
    pub fn migrate_value(
        &self,
        value: StoreValue,
        from_schema: &StoreSchema,
        to_schema: &StoreSchema,
    ) -> Result<StoreValue> {
        if from_schema.version == to_schema.version {
            return Ok(value);
        }

        match value {
            StoreValue::Object(mut obj) => {
                // Entferne gelöschte Felder
                obj.retain(|k, _| to_schema.fields.contains_key(k));

                // Füge neue Felder mit Defaults hinzu
                for (field_name, field_type) in &to_schema.fields {
                    if !from_schema.fields.contains_key(field_name) {
                        obj.insert(field_name.clone(), field_type.default_value());
                    }
                }

                // Migriere Feld-Typen
                for (field_name, to_type) in &to_schema.fields {
                    if let Some(from_type) = from_schema.fields.get(field_name) {
                        if from_type != to_type {
                            if let Some(field_value) = obj.remove(field_name) {
                                let migrated = to_type.migrate_value(field_value, from_type)?;
                                obj.insert(field_name.clone(), migrated);
                            }
                        }
                    }
                }

                Ok(StoreValue::Object(obj))
            }
            _ => Ok(value),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // CRUD-Operationen
    // ─────────────────────────────────────────────────────────────────────────

    /// Speichere Wert im Store
    pub fn put(
        &self,
        realm_id: &RealmId,
        sender_did: &DID,
        store_name: &str,
        key: &str,
        value: StoreValue,
    ) -> Result<()> {
        let schema = self.get_schema(realm_id, store_name, Some(sender_did))?;

        // Validiere Wert gegen Schema
        if let StoreValue::Object(ref obj) = value {
            for (field_name, field_value) in obj {
                if let Some(field_type) = schema.fields.get(field_name) {
                    field_value.validate(field_type)?;
                }
            }
        }

        let prefix = if schema.personal {
            PrefixBuilder::personal(realm_id, sender_did, store_name)
        } else {
            PrefixBuilder::shared(realm_id, store_name)
        };

        let full_key = prefix.key(key);
        let value_bytes = value.to_bytes()?;

        self.data.insert(&full_key, &value_bytes)?;

        // Indices aktualisieren
        if let Some(ref indices_partition) = self.indices {
            if let StoreValue::Object(ref obj) = value {
                for index_field in &schema.indices {
                    if let Some(StoreValue::String(index_value)) = obj.get(index_field) {
                        let index_key = prefix.index_key(index_field, index_value, key);
                        indices_partition.insert(&index_key, key.as_bytes())?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Hole Wert aus Store
    pub fn get(
        &self,
        realm_id: &RealmId,
        sender_did: &DID,
        store_name: &str,
        key: &str,
    ) -> Result<Option<StoreValue>> {
        let schema = self.get_schema(realm_id, store_name, Some(sender_did))?;

        let prefix = if schema.personal {
            PrefixBuilder::personal(realm_id, sender_did, store_name)
        } else {
            PrefixBuilder::shared(realm_id, store_name)
        };

        let full_key = prefix.key(key);

        match self.data.get(&full_key)? {
            Some(bytes) => Ok(Some(StoreValue::from_bytes(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Lösche Wert aus Store
    pub fn delete(
        &self,
        realm_id: &RealmId,
        sender_did: &DID,
        store_name: &str,
        key: &str,
    ) -> Result<bool> {
        let schema = self.get_schema(realm_id, store_name, Some(sender_did))?;

        let prefix = if schema.personal {
            PrefixBuilder::personal(realm_id, sender_did, store_name)
        } else {
            PrefixBuilder::shared(realm_id, store_name)
        };

        let full_key = prefix.key(key);

        // Prüfe ob existiert
        if self.data.get(&full_key)?.is_none() {
            return Ok(false);
        }

        self.data.remove(&full_key)?;
        Ok(true)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Nested-Operationen für tiefere Strukturen
    // ─────────────────────────────────────────────────────────────────────────

    /// Speichere verschachtelten Wert
    pub fn put_nested(
        &self,
        realm_id: &RealmId,
        sender_did: &DID,
        store_name: &str,
        key: &str,
        path: &[&str],
        value: StoreValue,
    ) -> Result<()> {
        // Validiere Tiefe
        if path.len() as u32 > self.config.max_depth {
            return Err(anyhow!(
                "Nested path depth {} exceeds maximum {}",
                path.len(),
                self.config.max_depth
            ));
        }

        let schema = self.get_schema(realm_id, store_name, Some(sender_did))?;

        // Navigiere durch Schema um Typ zu validieren
        let mut current_type: Option<&SchemaFieldType> = None;
        let mut search_fields = &schema.fields;

        for segment in path {
            if let Some(field_type) = search_fields.get(*segment) {
                current_type = Some(field_type);
                if let SchemaFieldType::Object { fields } = field_type {
                    search_fields = fields;
                }
            }
        }

        // Validiere gegen gefundenen Typ
        if let Some(expected_type) = current_type {
            value.validate(expected_type)?;
        }

        let prefix = if schema.personal {
            PrefixBuilder::personal(realm_id, sender_did, store_name)
        } else {
            PrefixBuilder::shared(realm_id, store_name)
        };

        let nested_key = prefix.nested_key(key, path);
        let value_bytes = value.to_bytes()?;

        self.data.insert(&nested_key, &value_bytes)?;

        Ok(())
    }

    /// Hole verschachtelten Wert
    pub fn get_nested(
        &self,
        realm_id: &RealmId,
        sender_did: &DID,
        store_name: &str,
        key: &str,
        path: &[&str],
    ) -> Result<Option<StoreValue>> {
        let schema = self.get_schema(realm_id, store_name, Some(sender_did))?;

        let prefix = if schema.personal {
            PrefixBuilder::personal(realm_id, sender_did, store_name)
        } else {
            PrefixBuilder::shared(realm_id, store_name)
        };

        // Versuche zuerst den spezifischen nested key
        let nested_key = prefix.nested_key(key, path);
        if let Some(bytes) = self.data.get(&nested_key)? {
            return Ok(Some(StoreValue::from_bytes(&bytes)?));
        }

        // Fallback: Hole Hauptobjekt und navigiere durch den Pfad
        let main_key = prefix.key(key);
        match self.data.get(&main_key)? {
            Some(bytes) => {
                let mut value = StoreValue::from_bytes(&bytes)?;

                // Navigiere durch den Pfad
                for segment in path {
                    match value {
                        StoreValue::Object(ref obj) => {
                            if let Some(v) = obj.get(*segment) {
                                value = v.clone();
                            } else {
                                return Ok(None);
                            }
                        }
                        _ => return Ok(None),
                    }
                }

                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Füge Element zu Liste hinzu
    pub fn append_list(
        &self,
        realm_id: &RealmId,
        sender_did: &DID,
        store_name: &str,
        key: &str,
        path: &[&str],
        value: StoreValue,
    ) -> Result<usize> {
        let schema = self.get_schema(realm_id, store_name, Some(sender_did))?;

        let prefix = if schema.personal {
            PrefixBuilder::personal(realm_id, sender_did, store_name)
        } else {
            PrefixBuilder::shared(realm_id, store_name)
        };

        let main_key = prefix.key(key);

        // Hole oder erstelle Hauptobjekt
        let mut main_object = match self.data.get(&main_key)? {
            Some(bytes) => StoreValue::from_bytes(&bytes)?,
            None => {
                // Erstelle leeres Objekt - die Liste wird später hinzugefügt
                StoreValue::Object(HashMap::new())
            }
        };

        // Navigiere zum Eltern-Objekt und finde/erstelle die Liste
        let (parent_path, list_field) = if path.is_empty() {
            return Err(anyhow!("Path cannot be empty for append_list"));
        } else {
            let last = path.last().unwrap();
            let parent = &path[..path.len() - 1];
            (parent, *last)
        };

        // Navigiere zum Eltern (erstelle fehlende Objekte unterwegs)
        let mut current = &mut main_object;
        for segment in parent_path {
            match current {
                StoreValue::Object(ref mut obj) => {
                    // Erstelle fehlendes Objekt falls nötig
                    if !obj.contains_key(*segment) {
                        obj.insert(segment.to_string(), StoreValue::Object(HashMap::new()));
                    }
                    current = obj.get_mut(*segment).unwrap();
                }
                _ => return Err(anyhow!("Expected object at path segment '{}'", segment)),
            }
        }

        // Jetzt bei current sollte list_field als Liste sein (oder erstellt werden)
        let new_len = match current {
            StoreValue::Object(ref mut obj) => {
                if let Some(list_value) = obj.get_mut(list_field) {
                    match list_value {
                        StoreValue::List(ref mut list) => {
                            // Prüfe Limit
                            if list.len() >= self.config.max_array_length {
                                return Err(anyhow!(
                                    "List exceeds maximum length of {}",
                                    self.config.max_array_length
                                ));
                            }
                            list.push(value);
                            list.len()
                        }
                        _ => return Err(anyhow!("Field '{}' is not a list", list_field)),
                    }
                } else {
                    // Feld existiert nicht -> erstelle neue Liste
                    let list = vec![value];
                    let len = list.len();
                    obj.insert(list_field.to_string(), StoreValue::List(list));
                    len
                }
            }
            _ => return Err(anyhow!("Expected object to contain list field")),
        };

        // Speichere aktualisiertes Hauptobjekt
        let updated_bytes = main_object.to_bytes()?;
        self.data.insert(&main_key, &updated_bytes)?;

        Ok(new_len)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Query-Operationen
    // ─────────────────────────────────────────────────────────────────────────

    /// Query alle Einträge eines Stores (Range-Scan)
    pub fn query_all(
        &self,
        realm_id: &RealmId,
        sender_did: &DID,
        store_name: &str,
        limit: Option<usize>,
    ) -> Result<Vec<(String, StoreValue)>> {
        let schema = self.get_schema(realm_id, store_name, Some(sender_did))?;

        let prefix = if schema.personal {
            PrefixBuilder::personal(realm_id, sender_did, store_name)
        } else {
            PrefixBuilder::shared(realm_id, store_name)
        };

        let store_prefix = prefix.store_prefix();
        let mut results = Vec::new();

        for entry in self.data.prefix(&store_prefix) {
            let (key, value) = entry?;
            let key_str = String::from_utf8_lossy(&key);

            // Überspringe Schema und Index-Keys
            if key_str.ends_with(":_schema") || key_str.contains(":_idx:") {
                continue;
            }

            // Extrahiere den eigentlichen Key
            if let Some(actual_key) = key_str.strip_prefix(&format!("{}:", store_prefix)) {
                // Überspringe nested keys (enthalten weitere :)
                if !actual_key.contains(':') {
                    let store_value = StoreValue::from_bytes(&value)?;
                    results.push((actual_key.to_string(), store_value));

                    if let Some(max) = limit {
                        if results.len() >= max {
                            break;
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Query über Index
    pub fn query_by_index(
        &self,
        realm_id: &RealmId,
        sender_did: &DID,
        store_name: &str,
        field_name: &str,
        value: &str,
        limit: Option<usize>,
    ) -> Result<Vec<(String, StoreValue)>> {
        let schema = self.get_schema(realm_id, store_name, Some(sender_did))?;

        // Prüfe ob Index existiert
        if !schema.indices.contains(&field_name.to_string()) {
            return Err(anyhow!(
                "No index on field '{}' in store '{}'",
                field_name,
                store_name
            ));
        }

        let prefix = if schema.personal {
            PrefixBuilder::personal(realm_id, sender_did, store_name)
        } else {
            PrefixBuilder::shared(realm_id, store_name)
        };

        let index_prefix = format!("{}:{}:", prefix.index_prefix(field_name), value);
        let mut results = Vec::new();

        if let Some(ref indices_partition) = self.indices {
            for entry in indices_partition.prefix(&index_prefix) {
                let (_, key_bytes) = entry?;
                let key = String::from_utf8_lossy(&key_bytes).to_string();

                if let Some(store_value) = self.get(realm_id, sender_did, store_name, &key)? {
                    results.push((key, store_value));

                    if let Some(max) = limit {
                        if results.len() >= max {
                            break;
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Store-Verwaltung
    // ─────────────────────────────────────────────────────────────────────────

    /// Liste alle Stores eines Realms
    pub fn list_stores(&self, realm_id: &RealmId) -> Result<Vec<StoreSchema>> {
        let prefix = format!("realm:{}:", realm_id.0);
        let mut schemas = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for entry in self.meta.prefix(&prefix) {
            let (key, value) = entry?;
            let key_str = String::from_utf8_lossy(&key);

            if key_str.ends_with(":_schema") {
                let schema: StoreSchema = serde_json::from_slice(&value)?;
                if seen.insert(schema.name.clone()) {
                    schemas.push(schema);
                }
            }
        }

        Ok(schemas)
    }

    /// Lösche einen Store komplett
    pub fn delete_store(
        &self,
        realm_id: &RealmId,
        sender_did: &DID,
        store_name: &str,
    ) -> Result<()> {
        let schema = self.get_schema(realm_id, store_name, Some(sender_did))?;

        let prefix = if schema.personal {
            PrefixBuilder::personal(realm_id, sender_did, store_name)
        } else {
            PrefixBuilder::shared(realm_id, store_name)
        };

        let store_prefix = prefix.store_prefix();

        // Sammle alle zu löschenden Keys
        let keys_to_delete: Vec<Vec<u8>> = self
            .data
            .prefix(&store_prefix)
            .filter_map(|entry| entry.ok().map(|(k, _)| k.to_vec()))
            .collect();

        for key in keys_to_delete {
            self.data.remove(&key)?;
        }

        // Lösche Indices
        if let Some(ref indices_partition) = self.indices {
            let index_prefix = format!("{}:_idx:", store_prefix);
            let index_keys: Vec<Vec<u8>> = indices_partition
                .prefix(&index_prefix)
                .filter_map(|entry| entry.ok().map(|(k, _)| k.to_vec()))
                .collect();

            for key in index_keys {
                indices_partition.remove(&key)?;
            }
        }

        // Lösche Schema
        self.meta.remove(prefix.schema_key())?;

        // Cache aktualisieren
        let cache_key = format!("{}:{}", realm_id.0, store_name);
        self.schema_cache.write().remove(&cache_key);

        tracing::info!(
            realm = %realm_id,
            store = %store_name,
            "Store deleted"
        );

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Host-Interface Convenience-Methoden (für ErynoaHost)
    // ─────────────────────────────────────────────────────────────────────────

    /// Hole Wert aus Shared-Store (Convenience für Host-Interface)
    pub fn get_shared(
        &self,
        realm_id: &str,
        store_name: &str,
        key: &str,
    ) -> Result<Option<StoreValue>> {
        let realm = realm_id_from_name(realm_id);
        let dummy_did = DID::new_self(b"_system");
        self.get(&realm, &dummy_did, store_name, key)
    }

    /// Hole Wert aus Personal-Store (Convenience für Host-Interface)
    pub fn get_personal(
        &self,
        realm_id: &str,
        did: &DID,
        store_name: &str,
        key: &str,
    ) -> Result<Option<StoreValue>> {
        let realm = realm_id_from_name(realm_id);
        self.get(&realm, did, store_name, key)
    }

    /// Speichere Wert in Shared-Store (Convenience für Host-Interface)
    pub fn put_shared(
        &self,
        realm_id: &str,
        store_name: &str,
        key: &str,
        value: StoreValue,
    ) -> Result<()> {
        let realm = realm_id_from_name(realm_id);
        let dummy_did = DID::new_self(b"_system");
        self.put(&realm, &dummy_did, store_name, key, value)
    }

    /// Speichere Wert in Personal-Store (Convenience für Host-Interface)
    pub fn put_personal(
        &self,
        realm_id: &str,
        did: &DID,
        store_name: &str,
        key: &str,
        value: StoreValue,
    ) -> Result<()> {
        let realm = realm_id_from_name(realm_id);
        self.put(&realm, did, store_name, key, value)
    }

    /// Lösche Wert aus Shared-Store (Convenience für Host-Interface)
    pub fn delete_shared(&self, realm_id: &str, store_name: &str, key: &str) -> Result<bool> {
        let realm = realm_id_from_name(realm_id);
        let dummy_did = DID::new_self(b"_system");
        self.delete(&realm, &dummy_did, store_name, key)
    }

    /// Lösche Wert aus Personal-Store (Convenience für Host-Interface)
    pub fn delete_personal(
        &self,
        realm_id: &str,
        did: &DID,
        store_name: &str,
        key: &str,
    ) -> Result<bool> {
        let realm = realm_id_from_name(realm_id);
        self.delete(&realm, did, store_name, key)
    }

    /// Hole verschachtelten Wert aus Shared-Store
    pub fn get_nested_shared(
        &self,
        realm_id: &str,
        store_name: &str,
        key: &str,
        path: &str,
    ) -> Result<Option<StoreValue>> {
        let realm = realm_id_from_name(realm_id);
        let dummy_did = DID::new_self(b"_system");
        let path_parts: Vec<&str> = path.split('.').collect();
        self.get_nested(&realm, &dummy_did, store_name, key, &path_parts)
    }

    /// Hole verschachtelten Wert aus Personal-Store
    pub fn get_nested_personal(
        &self,
        realm_id: &str,
        did: &DID,
        store_name: &str,
        key: &str,
        path: &str,
    ) -> Result<Option<StoreValue>> {
        let realm = realm_id_from_name(realm_id);
        let path_parts: Vec<&str> = path.split('.').collect();
        self.get_nested(&realm, did, store_name, key, &path_parts)
    }

    /// Speichere verschachtelten Wert in Shared-Store
    pub fn put_nested_shared(
        &self,
        realm_id: &str,
        store_name: &str,
        key: &str,
        path: &str,
        value: StoreValue,
    ) -> Result<()> {
        let realm = realm_id_from_name(realm_id);
        let dummy_did = DID::new_self(b"_system");
        let path_parts: Vec<&str> = path.split('.').collect();
        self.put_nested(&realm, &dummy_did, store_name, key, &path_parts, value)
    }

    /// Speichere verschachtelten Wert in Personal-Store
    pub fn put_nested_personal(
        &self,
        realm_id: &str,
        did: &DID,
        store_name: &str,
        key: &str,
        path: &str,
        value: StoreValue,
    ) -> Result<()> {
        let realm = realm_id_from_name(realm_id);
        let path_parts: Vec<&str> = path.split('.').collect();
        self.put_nested(&realm, did, store_name, key, &path_parts, value)
    }

    /// Füge Element zu Liste in Shared-Store hinzu
    pub fn append_to_list_shared(
        &self,
        realm_id: &str,
        store_name: &str,
        key: &str,
        path: &str,
        value: StoreValue,
    ) -> Result<usize> {
        let realm = realm_id_from_name(realm_id);
        let dummy_did = DID::new_self(b"_system");
        let path_parts: Vec<&str> = path.split('.').collect();
        self.append_list(&realm, &dummy_did, store_name, key, &path_parts, value)
    }

    /// Füge Element zu Liste in Personal-Store hinzu
    pub fn append_to_list_personal(
        &self,
        realm_id: &str,
        did: &DID,
        store_name: &str,
        key: &str,
        path: &str,
        value: StoreValue,
    ) -> Result<usize> {
        let realm = realm_id_from_name(realm_id);
        let path_parts: Vec<&str> = path.split('.').collect();
        self.append_list(&realm, did, store_name, key, &path_parts, value)
    }

    /// Prüfe ob Shared-Store existiert
    pub fn store_exists_shared(&self, realm_id: &str, store_name: &str) -> Result<bool> {
        let realm = realm_id_from_name(realm_id);
        let dummy_did = DID::new_self(b"_system");
        Ok(self
            .get_schema(&realm, store_name, Some(&dummy_did))
            .is_ok())
    }

    /// Prüfe ob Personal-Store existiert
    pub fn store_exists_personal(
        &self,
        realm_id: &str,
        did: &DID,
        store_name: &str,
    ) -> Result<bool> {
        let realm = realm_id_from_name(realm_id);
        Ok(self.get_schema(&realm, store_name, Some(did)).is_ok())
    }

    /// Zähle Einträge in Shared-Store
    pub fn count_shared(&self, realm_id: &str, store_name: &str) -> Result<usize> {
        let realm = realm_id_from_name(realm_id);
        let dummy_did = DID::new_self(b"_system");
        let results = self.query_all(&realm, &dummy_did, store_name, None)?;
        Ok(results.len())
    }

    /// Zähle Einträge in Personal-Store
    pub fn count_personal(&self, realm_id: &str, did: &DID, store_name: &str) -> Result<usize> {
        let realm = realm_id_from_name(realm_id);
        let results = self.query_all(&realm, did, store_name, None)?;
        Ok(results.len())
    }

    /// Query über Index in Shared-Store
    pub fn query_by_index_shared(
        &self,
        realm_id: &str,
        store_name: &str,
        field: &str,
        value: &str,
        limit: usize,
    ) -> Result<Vec<String>> {
        let realm = realm_id_from_name(realm_id);
        let dummy_did = DID::new_self(b"_system");
        let results =
            self.query_by_index(&realm, &dummy_did, store_name, field, value, Some(limit))?;
        Ok(results.into_iter().map(|(k, _)| k).collect())
    }

    /// Liste Keys in Shared-Store
    pub fn list_keys_shared(
        &self,
        realm_id: &str,
        store_name: &str,
        prefix: Option<&str>,
        limit: usize,
    ) -> Result<Vec<String>> {
        let realm = realm_id_from_name(realm_id);
        let dummy_did = DID::new_self(b"_system");
        let results = self.query_all(&realm, &dummy_did, store_name, Some(limit))?;

        let keys: Vec<String> = results
            .into_iter()
            .filter(|(k, _)| prefix.map(|p| k.starts_with(p)).unwrap_or(true))
            .map(|(k, _)| k)
            .collect();

        Ok(keys)
    }

    /// Liste Keys in Personal-Store
    pub fn list_keys_personal(
        &self,
        realm_id: &str,
        did: &DID,
        store_name: &str,
        prefix: Option<&str>,
        limit: usize,
    ) -> Result<Vec<String>> {
        let realm = realm_id_from_name(realm_id);
        let results = self.query_all(&realm, did, store_name, Some(limit))?;

        let keys: Vec<String> = results
            .into_iter()
            .filter(|(k, _)| prefix.map(|p| k.starts_with(p)).unwrap_or(true))
            .map(|(k, _)| k)
            .collect();

        Ok(keys)
    }

    /// Registriere Schema für Store (für Gateway Join-Flow)
    pub fn register_schema(
        &self,
        realm_id: &str,
        _store_name: &str,
        schema: StoreSchema,
    ) -> Result<()> {
        let realm = realm_id_from_name(realm_id);
        let dummy_did = DID::new_self(b"_system");
        self.create_store(&realm, &dummy_did, schema)
    }
}

impl Clone for RealmStorage {
    fn clone(&self) -> Self {
        Self {
            keyspace: Arc::clone(&self.keyspace),
            meta: self.meta.clone(),
            data: self.data.clone(),
            indices: self.indices.clone(),
            config: self.config.clone(),
            schema_cache: parking_lot::RwLock::new(self.schema_cache.read().clone()),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local::test_utils::test_keyspace;

    fn setup() -> (tempfile::TempDir, RealmStorage) {
        let (dir, keyspace) = test_keyspace();
        let storage = RealmStorage::new(&keyspace, RealmStorageConfig::default()).unwrap();
        (dir, storage)
    }

    #[test]
    fn test_schema_complexity() {
        let schema = StoreSchema::new("test", false)
            .with_field("name", SchemaFieldType::String)
            .with_field("age", SchemaFieldType::Number)
            .with_field(
                "tags",
                SchemaFieldType::List {
                    item_type: Box::new(SchemaFieldType::String),
                },
            )
            .with_index("name");

        assert!(schema.complexity() > 0);
        assert_eq!(schema.max_depth(), 1);
    }

    #[test]
    fn test_prefix_builder() {
        let realm_id = realm_id_from_name("social.berlin");
        let did = DID::new_self(b"alice123");

        let shared = PrefixBuilder::shared(&realm_id, "posts");
        assert_eq!(
            shared.store_prefix(),
            "realm:social.berlin:shared:store:posts"
        );
        assert_eq!(
            shared.key("post1"),
            "realm:social.berlin:shared:store:posts:post1"
        );

        let personal = PrefixBuilder::personal(&realm_id, &did, "notes");
        assert!(personal.store_prefix().contains("personal"));
        assert!(personal.store_prefix().contains("alice123"));
    }

    #[test]
    fn test_store_creation_and_crud() {
        let (_dir, storage) = setup();
        let realm_id = realm_id_from_name("test-realm");
        let sender = DID::new_self(b"alice");

        // Schema erstellen
        let schema = StoreSchema::new("posts", false)
            .with_field("title", SchemaFieldType::String)
            .with_field("content", SchemaFieldType::String)
            .with_index("title");

        storage.create_store(&realm_id, &sender, schema).unwrap();

        // Wert speichern
        let mut post = HashMap::new();
        post.insert("title".to_string(), StoreValue::String("Hello".to_string()));
        post.insert(
            "content".to_string(),
            StoreValue::String("World".to_string()),
        );

        storage
            .put(
                &realm_id,
                &sender,
                "posts",
                "post1",
                StoreValue::Object(post),
            )
            .unwrap();

        // Wert lesen
        let result = storage.get(&realm_id, &sender, "posts", "post1").unwrap();
        assert!(result.is_some());

        if let Some(StoreValue::Object(obj)) = result {
            assert_eq!(
                obj.get("title"),
                Some(&StoreValue::String("Hello".to_string()))
            );
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_personal_store_isolation() {
        let (_dir, storage) = setup();
        let realm_id = realm_id_from_name("test-realm");
        let alice = DID::new_self(b"alice");
        let bob = DID::new_self(b"bob");

        // Personal schema
        let schema = StoreSchema::new("notes", true).with_field("text", SchemaFieldType::String);

        storage
            .create_store(&realm_id, &alice, schema.clone())
            .unwrap();
        storage.create_store(&realm_id, &bob, schema).unwrap();

        // Alice speichert
        storage
            .put(
                &realm_id,
                &alice,
                "notes",
                "note1",
                StoreValue::Object({
                    let mut m = HashMap::new();
                    m.insert(
                        "text".to_string(),
                        StoreValue::String("Alice's note".to_string()),
                    );
                    m
                }),
            )
            .unwrap();

        // Bob speichert unter gleichem Key
        storage
            .put(
                &realm_id,
                &bob,
                "notes",
                "note1",
                StoreValue::Object({
                    let mut m = HashMap::new();
                    m.insert(
                        "text".to_string(),
                        StoreValue::String("Bob's note".to_string()),
                    );
                    m
                }),
            )
            .unwrap();

        // Alice sieht nur ihre Note
        let alice_note = storage
            .get(&realm_id, &alice, "notes", "note1")
            .unwrap()
            .unwrap();
        if let StoreValue::Object(obj) = alice_note {
            assert_eq!(
                obj.get("text"),
                Some(&StoreValue::String("Alice's note".to_string()))
            );
        }

        // Bob sieht nur seine Note
        let bob_note = storage
            .get(&realm_id, &bob, "notes", "note1")
            .unwrap()
            .unwrap();
        if let StoreValue::Object(obj) = bob_note {
            assert_eq!(
                obj.get("text"),
                Some(&StoreValue::String("Bob's note".to_string()))
            );
        }
    }

    #[test]
    fn test_nested_operations() {
        let (_dir, storage) = setup();
        let realm_id = realm_id_from_name("test-realm");
        let sender = DID::new_self(b"alice");

        let schema = StoreSchema::new("profiles", false).with_field(
            "main",
            SchemaFieldType::Object {
                fields: {
                    let mut m = HashMap::new();
                    m.insert("bio".to_string(), SchemaFieldType::String);
                    m.insert(
                        "settings".to_string(),
                        SchemaFieldType::Object {
                            fields: {
                                let mut s = HashMap::new();
                                s.insert("theme".to_string(), SchemaFieldType::String);
                                s
                            },
                        },
                    );
                    m
                },
            },
        );

        storage.create_store(&realm_id, &sender, schema).unwrap();

        // Nested put
        storage
            .put_nested(
                &realm_id,
                &sender,
                "profiles",
                "user1",
                &["main", "bio"],
                StoreValue::String("Hello, I'm Alice!".to_string()),
            )
            .unwrap();

        // Nested get
        let bio = storage
            .get_nested(&realm_id, &sender, "profiles", "user1", &["main", "bio"])
            .unwrap();

        assert_eq!(
            bio,
            Some(StoreValue::String("Hello, I'm Alice!".to_string()))
        );
    }

    #[test]
    fn test_list_operations() {
        let (_dir, storage) = setup();
        let realm_id = realm_id_from_name("test-realm");
        let sender = DID::new_self(b"alice");

        let schema = StoreSchema::new("users", false).with_field(
            "interests",
            SchemaFieldType::List {
                item_type: Box::new(SchemaFieldType::String),
            },
        );

        storage.create_store(&realm_id, &sender, schema).unwrap();

        // Append to list (returns new length)
        let len1 = storage
            .append_list(
                &realm_id,
                &sender,
                "users",
                "user1",
                &["interests"],
                StoreValue::String("rust".to_string()),
            )
            .unwrap();
        assert_eq!(len1, 1); // Length after first append

        let len2 = storage
            .append_list(
                &realm_id,
                &sender,
                "users",
                "user1",
                &["interests"],
                StoreValue::String("music".to_string()),
            )
            .unwrap();
        assert_eq!(len2, 2); // Length after second append

        // Get list
        let list = storage
            .get_nested(&realm_id, &sender, "users", "user1", &["interests"])
            .unwrap();

        if let Some(StoreValue::List(items)) = list {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], StoreValue::String("rust".to_string()));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_value_validation() {
        let string_type = SchemaFieldType::String;
        let number_type = SchemaFieldType::Number;

        assert!(StoreValue::String("hello".to_string())
            .validate(&string_type)
            .is_ok());
        assert!(StoreValue::Number(42.0).validate(&string_type).is_err());
        assert!(StoreValue::Number(42.0).validate(&number_type).is_ok());
    }

    #[test]
    fn test_max_depth_validation() {
        // 6 Ebenen tief: level1 -> level2 -> level3 -> level4 -> level5 -> level6 = String
        // Depth berechnung: Object(1) + Object(1) + Object(1) + Object(1) + Object(1) = 5
        let deeply_nested = StoreSchema::new("deep", false).with_field(
            "level1",
            SchemaFieldType::Object {
                fields: {
                    let mut m = HashMap::new();
                    m.insert(
                        "level2".to_string(),
                        SchemaFieldType::Object {
                            fields: {
                                let mut m2 = HashMap::new();
                                m2.insert(
                                    "level3".to_string(),
                                    SchemaFieldType::Object {
                                        fields: {
                                            let mut m3 = HashMap::new();
                                            m3.insert(
                                                "level4".to_string(),
                                                SchemaFieldType::Object {
                                                    fields: {
                                                        let mut m4 = HashMap::new();
                                                        m4.insert(
                                                            "level5".to_string(),
                                                            SchemaFieldType::Object {
                                                                fields: {
                                                                    let mut m5 = HashMap::new();
                                                                    m5.insert(
                                                                        "level6".to_string(),
                                                                        SchemaFieldType::String,
                                                                    );
                                                                    m5
                                                                },
                                                            },
                                                        );
                                                        m4
                                                    },
                                                },
                                            );
                                            m3
                                        },
                                    },
                                );
                                m2
                            },
                        },
                    );
                    m
                },
            },
        );

        // Max depth von deeply_nested ist 5 (5 verschachtelte Objects)
        assert!(deeply_nested.validate_depth(4).is_err()); // 5 > 4 → Fehler
        assert!(deeply_nested.validate_depth(5).is_ok()); // 5 <= 5 → OK
        assert!(deeply_nested.validate_depth(10).is_ok()); // 5 <= 10 → OK
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Schema-Evolution Tests (Ψ-Adaptation)
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_schema_evolution_add_field() {
        let (_dir, storage) = setup();
        let realm_id = realm_id_from_name("test-realm");
        let sender = DID::new_self(b"alice");

        // Erstelle ursprüngliches Schema
        let schema = StoreSchema::new("posts", false)
            .with_field("title", SchemaFieldType::String)
            .with_field("content", SchemaFieldType::String);

        storage.create_store(&realm_id, &sender, schema).unwrap();

        // Füge neues Feld hinzu (Non-Breaking Change)
        let changes = vec![SchemaChange::AddField {
            name: "tags".to_string(),
            field_type: SchemaFieldType::List {
                item_type: Box::new(SchemaFieldType::String),
            },
            default: Some(StoreValue::List(vec![])),
        }];

        let result = storage
            .evolve_schema(
                &realm_id,
                &sender,
                "posts",
                changes,
                "Added tags field".to_string(),
            )
            .unwrap();

        assert_eq!(result.new_version, 2);
        assert!(!result.is_breaking);
        assert_eq!(result.status, SchemaChangeStatus::Active);

        // Neues Schema prüfen
        let updated = storage
            .get_schema(&realm_id, "posts", Some(&sender))
            .unwrap();
        assert_eq!(updated.version, 2);
        assert!(updated.fields.contains_key("tags"));
    }

    #[test]
    fn test_schema_evolution_remove_field_pending() {
        let (_dir, storage) = setup();
        let realm_id = realm_id_from_name("test-realm");
        let sender = DID::new_self(b"alice");

        let schema = StoreSchema::new("users", false)
            .with_field("name", SchemaFieldType::String)
            .with_field("legacy_field", SchemaFieldType::String);

        storage.create_store(&realm_id, &sender, schema).unwrap();

        // Entferne Feld (Breaking Change)
        let changes = vec![SchemaChange::RemoveField {
            name: "legacy_field".to_string(),
        }];

        let result = storage
            .evolve_schema(
                &realm_id,
                &sender,
                "users",
                changes,
                "Remove legacy field".to_string(),
            )
            .unwrap();

        assert_eq!(result.new_version, 2);
        assert!(result.is_breaking);

        // Sollte Pending sein (Challenge-Periode)
        if let SchemaChangeStatus::Pending { challenge_ends } = result.status {
            assert!(challenge_ends > 0);
        } else {
            panic!("Expected Pending status for breaking change");
        }

        // Aktuelles Schema sollte noch Version 1 sein
        let current = storage
            .get_schema(&realm_id, "users", Some(&sender))
            .unwrap();
        assert_eq!(current.version, 1);
        assert!(current.fields.contains_key("legacy_field"));
    }

    #[test]
    fn test_schema_evolution_rename_field() {
        let (_dir, storage) = setup();
        let realm_id = realm_id_from_name("test-realm");
        let sender = DID::new_self(b"alice");

        let schema =
            StoreSchema::new("items", false).with_field("old_name", SchemaFieldType::String);

        storage.create_store(&realm_id, &sender, schema).unwrap();

        let changes = vec![SchemaChange::RenameField {
            old_name: "old_name".to_string(),
            new_name: "new_name".to_string(),
        }];

        let result = storage
            .evolve_schema(
                &realm_id,
                &sender,
                "items",
                changes,
                "Rename field".to_string(),
            )
            .unwrap();

        // RenameField ist Breaking
        assert!(result.is_breaking);
        assert!(matches!(result.status, SchemaChangeStatus::Pending { .. }));
    }

    #[test]
    fn test_schema_evolution_add_index() {
        let (_dir, storage) = setup();
        let realm_id = realm_id_from_name("test-realm");
        let sender = DID::new_self(b"alice");

        let schema = StoreSchema::new("products", false)
            .with_field("name", SchemaFieldType::String)
            .with_field("category", SchemaFieldType::String);

        storage.create_store(&realm_id, &sender, schema).unwrap();

        let changes = vec![SchemaChange::AddIndex {
            field_name: "category".to_string(),
        }];

        let result = storage
            .evolve_schema(
                &realm_id,
                &sender,
                "products",
                changes,
                "Add category index".to_string(),
            )
            .unwrap();

        assert!(!result.is_breaking);
        assert_eq!(result.status, SchemaChangeStatus::Active);

        let updated = storage
            .get_schema(&realm_id, "products", Some(&sender))
            .unwrap();
        assert!(updated.indices.contains(&"category".to_string()));
    }

    #[test]
    fn test_schema_history() {
        let (_dir, storage) = setup();
        let realm_id = realm_id_from_name("test-realm");
        let sender = DID::new_self(b"alice");

        let schema = StoreSchema::new("events", false).with_field("name", SchemaFieldType::String);

        storage.create_store(&realm_id, &sender, schema).unwrap();

        // Erste Änderung: Add field
        let changes1 = vec![SchemaChange::AddField {
            name: "date".to_string(),
            field_type: SchemaFieldType::Timestamp,
            default: None,
        }];
        storage
            .evolve_schema(
                &realm_id,
                &sender,
                "events",
                changes1,
                "Add date field".to_string(),
            )
            .unwrap();

        // Zweite Änderung: Add index
        let changes2 = vec![SchemaChange::AddIndex {
            field_name: "date".to_string(),
        }];
        storage
            .evolve_schema(
                &realm_id,
                &sender,
                "events",
                changes2,
                "Index date".to_string(),
            )
            .unwrap();

        // Historie prüfen
        let history = storage
            .get_schema_history(&realm_id, "events", Some(&sender))
            .unwrap();

        assert_eq!(history.current_version, 3);
        assert_eq!(history.changelog.len(), 2); // 2 Änderungen (Version 2 und 3)

        // Prüfe erste Änderung
        let entry1 = history.get_entry(2).unwrap();
        assert_eq!(entry1.description, "Add date field");
        assert!(!entry1.required_challenge);

        // Prüfe zweite Änderung
        let entry2 = history.get_entry(3).unwrap();
        assert_eq!(entry2.description, "Index date");
    }

    #[test]
    fn test_schema_version_retrieval() {
        let (_dir, storage) = setup();
        let realm_id = realm_id_from_name("test-realm");
        let sender = DID::new_self(b"alice");

        let schema =
            StoreSchema::new("documents", false).with_field("title", SchemaFieldType::String);

        storage.create_store(&realm_id, &sender, schema).unwrap();

        // Änderung hinzufügen
        let changes = vec![SchemaChange::AddField {
            name: "author".to_string(),
            field_type: SchemaFieldType::Did,
            default: None,
        }];
        storage
            .evolve_schema(
                &realm_id,
                &sender,
                "documents",
                changes,
                "Add author".to_string(),
            )
            .unwrap();

        // Version 2 abrufen
        let v2 = storage
            .get_schema_version(&realm_id, "documents", 2, Some(&sender))
            .unwrap();
        assert_eq!(v2.version, 2);
        assert!(v2.fields.contains_key("author"));
    }

    #[test]
    fn test_schema_change_validation() {
        let (_dir, storage) = setup();
        let realm_id = realm_id_from_name("test-realm");
        let sender = DID::new_self(b"alice");

        let schema = StoreSchema::new("config", false)
            .with_field("setting", SchemaFieldType::String)
            .with_index("setting");

        storage.create_store(&realm_id, &sender, schema).unwrap();

        // Versuche nicht existierendes Feld zu entfernen → Fehler
        let invalid_remove = vec![SchemaChange::RemoveField {
            name: "nonexistent".to_string(),
        }];
        assert!(storage
            .evolve_schema(
                &realm_id,
                &sender,
                "config",
                invalid_remove,
                "Invalid".to_string()
            )
            .is_err());

        // Versuche doppeltes Feld hinzuzufügen → Fehler
        let duplicate_add = vec![SchemaChange::AddField {
            name: "setting".to_string(),
            field_type: SchemaFieldType::Number,
            default: None,
        }];
        assert!(storage
            .evolve_schema(
                &realm_id,
                &sender,
                "config",
                duplicate_add,
                "Invalid".to_string()
            )
            .is_err());

        // Versuche existierenden Index hinzuzufügen → Fehler
        let duplicate_index = vec![SchemaChange::AddIndex {
            field_name: "setting".to_string(),
        }];
        assert!(storage
            .evolve_schema(
                &realm_id,
                &sender,
                "config",
                duplicate_index,
                "Invalid".to_string()
            )
            .is_err());
    }

    #[test]
    fn test_schema_field_type_compatibility() {
        // String → Optional<String> ist kompatibel
        assert!(SchemaFieldType::Optional {
            inner: Box::new(SchemaFieldType::String)
        }
        .is_compatible_with(&SchemaFieldType::String));

        // Number → String: In dieser Richtung KOMPATIBEL (Formatierung möglich)
        assert!(SchemaFieldType::String.is_compatible_with(&SchemaFieldType::Number));

        // String → Number: NICHT kompatibel (Parsing kann fehlschlagen)
        assert!(!SchemaFieldType::Number.is_compatible_with(&SchemaFieldType::String));

        // List<String> → List<String> ist kompatibel
        let list_type = SchemaFieldType::List {
            item_type: Box::new(SchemaFieldType::String),
        };
        assert!(list_type.is_compatible_with(&list_type));
    }

    #[test]
    fn test_value_migration() {
        let (_dir, _storage) = setup();

        // Number zu String Migration (unterstützt)
        let migrated = SchemaFieldType::String
            .migrate_value(StoreValue::Number(42.0), &SchemaFieldType::Number)
            .unwrap();
        assert_eq!(migrated, StoreValue::String("42".to_string()));

        // String zu Number Migration ist NICHT unterstützt (nicht implementiert)
        // Dies ist ein bewusster Design-Entscheid: nur sichere Migrationen
        let result = SchemaFieldType::Number.migrate_value(
            StoreValue::String("123.5".to_string()),
            &SchemaFieldType::String,
        );
        assert!(result.is_err());

        // T zu Optional<T> Migration
        let migrated = SchemaFieldType::Optional {
            inner: Box::new(SchemaFieldType::String),
        }
        .migrate_value(
            StoreValue::String("hello".to_string()),
            &SchemaFieldType::String,
        )
        .unwrap();
        assert_eq!(migrated, StoreValue::String("hello".to_string()));
    }

    #[test]
    fn test_schema_evolution_mana_costs() {
        let (_dir, storage) = setup();

        // Non-Breaking Changes: 50 + complexity * 10
        // String hat complexity = 1, daher 50 + 10 = 60
        let add_field = SchemaChange::AddField {
            name: "test".to_string(),
            field_type: SchemaFieldType::String,
            default: None,
        };
        assert_eq!(add_field.mana_cost(), 60); // 50 + 1*10

        // Breaking Changes
        let remove_field = SchemaChange::RemoveField {
            name: "test".to_string(),
        };
        assert_eq!(remove_field.mana_cost(), 200);

        // Komplexe Typen kosten mehr
        // Object mit einem String-Feld: complexity = 2 + 1 = 3
        let add_complex = SchemaChange::AddField {
            name: "nested".to_string(),
            field_type: SchemaFieldType::Object {
                fields: {
                    let mut m = HashMap::new();
                    m.insert("sub".to_string(), SchemaFieldType::String);
                    m
                },
            },
            default: None,
        };
        assert_eq!(add_complex.mana_cost(), 50 + 3 * 10); // 80

        // Berechne Gesamtkosten mit Breaking-Multiplikator
        let changes = vec![remove_field.clone()];
        let cost = storage.calculate_evolution_cost(&changes);
        assert_eq!(cost, 200 * 4); // 4x Multiplikator für Breaking
    }

    #[test]
    fn test_lazy_value_migration() {
        let (_dir, storage) = setup();
        let _realm_id = realm_id_from_name("test-realm");
        let _sender = DID::new_self(b"alice");

        // Altes Schema
        let old_schema =
            StoreSchema::new("profiles", false).with_field("name", SchemaFieldType::String);

        // Neues Schema mit zusätzlichem Feld
        // Hinweis: default in SchemaChange wird aktuell nicht in migrate_value genutzt,
        // stattdessen wird der Type-Default (leerer String für String) verwendet.
        let new_schema = old_schema
            .clone()
            .evolve(&[SchemaChange::AddField {
                name: "bio".to_string(),
                field_type: SchemaFieldType::String,
                default: None, // Default kommt vom Type
            }])
            .unwrap();

        // Alter Wert ohne bio
        let old_value = StoreValue::Object({
            let mut m = HashMap::new();
            m.insert("name".to_string(), StoreValue::String("Alice".to_string()));
            m
        });

        // Migriere
        let migrated = storage
            .migrate_value(old_value, &old_schema, &new_schema)
            .unwrap();

        // Prüfe migrierten Wert
        if let StoreValue::Object(obj) = migrated {
            // Name sollte erhalten sein
            assert_eq!(
                obj.get("name"),
                Some(&StoreValue::String("Alice".to_string()))
            );
            // Bio sollte mit Type-Default (leerer String) hinzugefügt werden
            assert_eq!(obj.get("bio"), Some(&StoreValue::String("".to_string())));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_reject_pending_schema() {
        let (_dir, storage) = setup();
        let realm_id = realm_id_from_name("test-realm");
        let sender = DID::new_self(b"alice");

        let schema = StoreSchema::new("data", false).with_field("value", SchemaFieldType::String);

        storage.create_store(&realm_id, &sender, schema).unwrap();

        // Breaking Change erstellen
        let changes = vec![SchemaChange::RemoveField {
            name: "value".to_string(),
        }];
        let result = storage
            .evolve_schema(
                &realm_id,
                &sender,
                "data",
                changes,
                "Remove value".to_string(),
            )
            .unwrap();

        assert!(matches!(result.status, SchemaChangeStatus::Pending { .. }));

        // Ablehnen
        storage
            .reject_pending_schema(
                &realm_id,
                &sender,
                "data",
                result.new_version,
                "Not approved by community".to_string(),
            )
            .unwrap();

        // Prüfe Status in Historie
        let history = storage
            .get_schema_history(&realm_id, "data", Some(&sender))
            .unwrap();

        let entry = history.get_entry(result.new_version).unwrap();
        assert!(matches!(entry.status, SchemaChangeStatus::Rejected { .. }));
    }
}
