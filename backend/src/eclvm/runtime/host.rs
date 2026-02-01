//! # Host Interface
//!
//! Die Sandbox-Schnittstelle zwischen ECLVM und dem Erynoa-Backend.
//!
//! Die VM darf nicht direkt auf Datenbank/Storage zugreifen.
//! Stattdessen werden alle externen Operationen über dieses Interface geleitet.
//!
//! ## Erweiterung: Realm Storage
//!
//! Das Interface wurde um Speicher-Operationen erweitert, die es ECL-Policies
//! erlauben, strukturiert mit Realm-Daten zu arbeiten:
//!
//! - `store_get/put` - Einfache Key-Value Operationen
//! - `store_get_nested/put_nested` - Zugriff auf verschachtelte Felder
//! - `store_query` - Indexbasierte Abfragen
//!
//! ## Erweiterung: Schema-Evolution (Ψ-Adaptation)
//!
//! Das Interface unterstützt dynamische Schema-Änderungen:
//!
//! - `store_evolve_schema` - Neue Schema-Version mit Änderungen erstellen
//! - `store_get_schema_version` - Spezifische Schema-Version abrufen
//! - `store_get_schema_history` - Vollständige Schema-Historie abrufen
//! - `store_activate_schema` - Pending Schema nach Challenge aktivieren
//! - `store_reject_schema` - Pending Schema ablehnen
//!
//! Jede Operation hat Mana-Kosten und wird durch Schema-Validierung geschützt.

use crate::error::Result;

// ═══════════════════════════════════════════════════════════════════════════
// Host-Typen für Speicher-Operationen
// ═══════════════════════════════════════════════════════════════════════════

/// Speicher-Wert für Host-Interface (vereinfacht für VM-Kompatibilität)
#[derive(Debug, Clone, PartialEq)]
pub enum HostStoreValue {
    Null,
    String(String),
    Number(f64),
    Bool(bool),
    List(Vec<HostStoreValue>),
    Object(std::collections::HashMap<String, HostStoreValue>),
}

impl HostStoreValue {
    /// Konvertiere zu JSON-String für Serialisierung
    pub fn to_json(&self) -> String {
        match self {
            Self::Null => "null".to_string(),
            Self::String(s) => format!("\"{}\"", s.replace('\"', "\\\"")),
            Self::Number(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::List(items) => {
                let inner: Vec<String> = items.iter().map(|v| v.to_json()).collect();
                format!("[{}]", inner.join(","))
            }
            Self::Object(map) => {
                let pairs: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, v.to_json()))
                    .collect();
                format!("{{{}}}", pairs.join(","))
            }
        }
    }

    /// Versuche als String zu extrahieren
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Versuche als Number zu extrahieren
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Versuche als Bool zu extrahieren
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

/// Speicher-Kontext für Host-Operationen
#[derive(Debug, Clone)]
pub struct StoreContext {
    /// Aktuelles Realm
    pub realm_id: String,
    /// Ausführende DID (für Personal-Stores)
    pub caller_did: String,
}

impl StoreContext {
    pub fn new(realm_id: impl Into<String>, caller_did: impl Into<String>) -> Self {
        Self {
            realm_id: realm_id.into(),
            caller_did: caller_did.into(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Host-Typen für Schema-Evolution
// ═══════════════════════════════════════════════════════════════════════════

/// Feld-Typ für Host-Interface (vereinfacht)
#[derive(Debug, Clone, PartialEq)]
pub enum HostFieldType {
    String,
    Number,
    Bool,
    List(Box<HostFieldType>),
    Object(std::collections::HashMap<String, HostFieldType>),
    Optional(Box<HostFieldType>),
    DID,
    Timestamp,
    Ref(String), // Referenz auf anderen Store
}

/// Schema-Änderung für Host-Interface
#[derive(Debug, Clone)]
pub enum HostSchemaChange {
    /// Neues Feld hinzufügen
    AddField {
        name: String,
        field_type: HostFieldType,
        default_value: Option<HostStoreValue>,
    },
    /// Feld entfernen (Breaking Change)
    RemoveField { name: String },
    /// Feld-Typ ändern
    ModifyField {
        name: String,
        new_type: HostFieldType,
    },
    /// Feld umbenennen
    RenameField { old_name: String, new_name: String },
    /// Index hinzufügen
    AddIndex { field_name: String },
    /// Index entfernen
    RemoveIndex { field_name: String },
}

impl HostSchemaChange {
    /// Prüft ob die Änderung ein Breaking Change ist
    pub fn is_breaking(&self) -> bool {
        matches!(
            self,
            Self::RemoveField { .. } | Self::ModifyField { .. } | Self::RenameField { .. }
        )
    }

    /// Berechne Mana-Kosten für diese Änderung
    pub fn mana_cost(&self) -> u64 {
        match self {
            Self::AddField { field_type, .. } => 50 + Self::type_complexity(field_type),
            Self::RemoveField { .. } => 200,
            Self::ModifyField { new_type, .. } => 300 + Self::type_complexity(new_type),
            Self::RenameField { .. } => 100,
            Self::AddIndex { .. } => 150,
            Self::RemoveIndex { .. } => 50,
        }
    }

    fn type_complexity(field_type: &HostFieldType) -> u64 {
        match field_type {
            HostFieldType::String | HostFieldType::Number | HostFieldType::Bool => 0,
            HostFieldType::DID | HostFieldType::Timestamp => 5,
            HostFieldType::Ref(_) => 10,
            HostFieldType::Optional(inner) => 5 + Self::type_complexity(inner),
            HostFieldType::List(inner) => 10 + Self::type_complexity(inner),
            HostFieldType::Object(fields) => {
                20 + fields.values().map(Self::type_complexity).sum::<u64>()
            }
        }
    }
}

/// Ergebnis einer Schema-Evolution
#[derive(Debug, Clone)]
pub struct HostSchemaEvolutionResult {
    /// Neue Schema-Version
    pub new_version: u32,
    /// Enthält Breaking Changes?
    pub is_breaking: bool,
    /// Status: "active", "pending", "rejected"
    pub status: String,
    /// Ende der Challenge-Periode (Unix timestamp, nur bei pending)
    pub challenge_ends: Option<u64>,
    /// Mana-Kosten der Änderung
    pub mana_cost: u64,
}

/// Vereinfachtes Schema für Host-Interface
#[derive(Debug, Clone)]
pub struct HostStoreSchema {
    /// Schema-Name
    pub name: String,
    /// Version
    pub version: u32,
    /// Felder
    pub fields: std::collections::HashMap<String, HostFieldType>,
    /// Indizierte Felder
    pub indices: Vec<String>,
    /// Ist Personal-Store?
    pub personal: bool,
}

/// Schema-Historie-Eintrag
#[derive(Debug, Clone)]
pub struct HostSchemaHistoryEntry {
    /// Version
    pub version: u32,
    /// Zeitstempel
    pub timestamp: u64,
    /// Wer hat geändert
    pub changed_by: String,
    /// Status: "active", "pending", "rejected", "superseded"
    pub status: String,
    /// Beschreibung
    pub description: String,
    /// War Breaking Change?
    pub was_breaking: bool,
}

/// Schema-Historie
#[derive(Debug, Clone)]
pub struct HostSchemaHistory {
    /// Store-Name
    pub store_name: String,
    /// Aktuelle Version
    pub current_version: u32,
    /// Historie-Einträge
    pub entries: Vec<HostSchemaHistoryEntry>,
}

/// Host Interface - Schnittstelle zum Erynoa Backend
///
/// Implementiere dieses Trait um der ECLVM Zugriff auf
/// Trust-Daten, Credentials und andere Erynoa-Funktionen zu geben.
///
/// ## Speicher-Operationen (Κ24: Datenintegrität)
///
/// Die Speicher-Methoden sind optional - Default-Implementierungen geben
/// `NotSupported` zurück. Implementiere sie für volle Realm-Storage-Funktionalität.
pub trait HostInterface: Send + Sync {
    /// Hole Trust-Vektor für eine DID
    ///
    /// Gibt [R, I, C, P, V, Ω] zurück
    fn get_trust_vector(&self, did: &str) -> Result<[f64; 6]>;

    /// Prüfe ob DID ein bestimmtes Credential hat
    fn has_credential(&self, did: &str, schema: &str) -> Result<bool>;

    /// Hole Balance für DID
    fn get_balance(&self, did: &str) -> Result<u64>;

    /// Prüfe ob DID existiert
    fn resolve_did(&self, did: &str) -> Result<bool>;

    /// Aktueller Timestamp (Unix Seconds)
    fn get_timestamp(&self) -> u64;

    /// Log-Nachricht (für Debugging)
    fn log(&self, message: &str);

    // ═══════════════════════════════════════════════════════════════════════
    // Realm Storage Operationen (Optional - Default: NotSupported)
    // ═══════════════════════════════════════════════════════════════════════

    /// Setze den Store-Kontext für nachfolgende Operationen
    ///
    /// Muss vor allen store_* Operationen aufgerufen werden.
    fn set_store_context(&mut self, _ctx: StoreContext) -> Result<()> {
        Err(crate::error::ApiError::NotSupported(
            "Store-Kontext nicht unterstützt".into(),
        ))
    }

    /// Hole Wert aus einem Store
    ///
    /// # Arguments
    /// - `store_name`: Name des Stores
    /// - `key`: Schlüssel innerhalb des Stores
    /// - `is_personal`: true = Personal-Store, false = Shared-Store
    ///
    /// # Mana Cost: 5
    fn store_get(
        &self,
        _store_name: &str,
        _key: &str,
        _is_personal: bool,
    ) -> Result<Option<HostStoreValue>> {
        Err(crate::error::ApiError::NotSupported(
            "store_get nicht unterstützt".into(),
        ))
    }

    /// Setze Wert in einem Store
    ///
    /// # Arguments
    /// - `store_name`: Name des Stores
    /// - `key`: Schlüssel innerhalb des Stores
    /// - `value`: Zu speichernder Wert
    /// - `is_personal`: true = Personal-Store, false = Shared-Store
    ///
    /// # Mana Cost: 10 + value_complexity
    fn store_put(
        &mut self,
        _store_name: &str,
        _key: &str,
        _value: HostStoreValue,
        _is_personal: bool,
    ) -> Result<()> {
        Err(crate::error::ApiError::NotSupported(
            "store_put nicht unterstützt".into(),
        ))
    }

    /// Lösche Wert aus einem Store
    ///
    /// # Mana Cost: 5
    fn store_delete(&mut self, _store_name: &str, _key: &str, _is_personal: bool) -> Result<bool> {
        Err(crate::error::ApiError::NotSupported(
            "store_delete nicht unterstützt".into(),
        ))
    }

    /// Hole verschachtelten Wert (z.B. "user.profile.name")
    ///
    /// # Arguments
    /// - `store_name`: Name des Stores
    /// - `key`: Haupt-Schlüssel
    /// - `path`: Punktseparierter Pfad (z.B. "profile.name")
    /// - `is_personal`: true = Personal-Store, false = Shared-Store
    ///
    /// # Mana Cost: 5 + path_depth
    fn store_get_nested(
        &self,
        _store_name: &str,
        _key: &str,
        _path: &str,
        _is_personal: bool,
    ) -> Result<Option<HostStoreValue>> {
        Err(crate::error::ApiError::NotSupported(
            "store_get_nested nicht unterstützt".into(),
        ))
    }

    /// Setze verschachtelten Wert
    ///
    /// # Arguments
    /// - `store_name`: Name des Stores
    /// - `key`: Haupt-Schlüssel
    /// - `path`: Punktseparierter Pfad (z.B. "profile.name")
    /// - `value`: Zu speichernder Wert
    /// - `is_personal`: true = Personal-Store, false = Shared-Store
    ///
    /// # Mana Cost: 10 + path_depth + value_complexity
    fn store_put_nested(
        &mut self,
        _store_name: &str,
        _key: &str,
        _path: &str,
        _value: HostStoreValue,
        _is_personal: bool,
    ) -> Result<()> {
        Err(crate::error::ApiError::NotSupported(
            "store_put_nested nicht unterstützt".into(),
        ))
    }

    /// Füge Element zu einer Liste hinzu
    ///
    /// # Returns
    /// Neue Länge der Liste
    ///
    /// # Mana Cost: 10 + value_complexity
    fn store_append_list(
        &mut self,
        _store_name: &str,
        _key: &str,
        _path: &str,
        _value: HostStoreValue,
        _is_personal: bool,
    ) -> Result<usize> {
        Err(crate::error::ApiError::NotSupported(
            "store_append_list nicht unterstützt".into(),
        ))
    }

    /// Prüfe ob Store existiert
    ///
    /// # Mana Cost: 1
    fn store_exists(&self, _store_name: &str, _is_personal: bool) -> Result<bool> {
        Err(crate::error::ApiError::NotSupported(
            "store_exists nicht unterstützt".into(),
        ))
    }

    /// Zähle Einträge in einem Store
    ///
    /// # Mana Cost: 5
    fn store_count(&self, _store_name: &str, _is_personal: bool) -> Result<usize> {
        Err(crate::error::ApiError::NotSupported(
            "store_count nicht unterstützt".into(),
        ))
    }

    /// Abfrage über Index
    ///
    /// # Arguments
    /// - `store_name`: Name des Stores
    /// - `index_field`: Indiziertes Feld
    /// - `value`: Gesuchter Wert
    /// - `limit`: Max. Anzahl Ergebnisse
    ///
    /// # Returns
    /// Liste von Keys die dem Index entsprechen
    ///
    /// # Mana Cost: 15 + result_count
    fn store_query_by_index(
        &self,
        _store_name: &str,
        _index_field: &str,
        _value: &HostStoreValue,
        _limit: usize,
    ) -> Result<Vec<String>> {
        Err(crate::error::ApiError::NotSupported(
            "store_query_by_index nicht unterstützt".into(),
        ))
    }

    /// Iteriere über alle Schlüssel eines Stores (mit Prefix-Filter)
    ///
    /// # Arguments
    /// - `store_name`: Name des Stores
    /// - `prefix`: Optionaler Key-Prefix
    /// - `limit`: Max. Anzahl Keys
    ///
    /// # Mana Cost: 10 + result_count
    fn store_list_keys(
        &self,
        _store_name: &str,
        _prefix: Option<&str>,
        _limit: usize,
        _is_personal: bool,
    ) -> Result<Vec<String>> {
        Err(crate::error::ApiError::NotSupported(
            "store_list_keys nicht unterstützt".into(),
        ))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Schema-Evolution Operationen (Ψ-Adaptation)
    // ═══════════════════════════════════════════════════════════════════════

    /// Evolve ein Store-Schema mit den gegebenen Änderungen
    ///
    /// Bei Breaking Changes wird eine Challenge-Periode gestartet.
    /// Nur DIDs mit ausreichend Trust (R ≥ 0.7) dürfen Schemas ändern.
    ///
    /// # Arguments
    /// - `store_name`: Name des Stores
    /// - `changes`: Liste der Schema-Änderungen
    /// - `description`: Beschreibung der Änderung
    ///
    /// # Returns
    /// Ergebnis mit neuer Version, Status und Kosten
    ///
    /// # Mana Cost: 50-500+ abhängig von Änderungen (4x bei Breaking)
    fn store_evolve_schema(
        &mut self,
        _store_name: &str,
        _changes: Vec<HostSchemaChange>,
        _description: &str,
    ) -> Result<HostSchemaEvolutionResult> {
        Err(crate::error::ApiError::NotSupported(
            "store_evolve_schema nicht unterstützt".into(),
        ))
    }

    /// Hole spezifische Schema-Version
    ///
    /// # Arguments
    /// - `store_name`: Name des Stores
    /// - `version`: Schema-Version (1, 2, 3, ...)
    ///
    /// # Mana Cost: 5
    fn store_get_schema_version(
        &self,
        _store_name: &str,
        _version: u32,
        _is_personal: bool,
    ) -> Result<Option<HostStoreSchema>> {
        Err(crate::error::ApiError::NotSupported(
            "store_get_schema_version nicht unterstützt".into(),
        ))
    }

    /// Hole Schema-Historie für einen Store
    ///
    /// # Returns
    /// Vollständige Historie mit allen Versionen und Änderungen
    ///
    /// # Mana Cost: 10 + version_count
    fn store_get_schema_history(
        &self,
        _store_name: &str,
        _is_personal: bool,
    ) -> Result<HostSchemaHistory> {
        Err(crate::error::ApiError::NotSupported(
            "store_get_schema_history nicht unterstützt".into(),
        ))
    }

    /// Aktiviere ein pending Schema nach Challenge-Periode
    ///
    /// Kann nur aufgerufen werden wenn die Challenge-Periode abgelaufen ist.
    ///
    /// # Arguments
    /// - `store_name`: Name des Stores
    /// - `version`: Die zu aktivierende Version
    ///
    /// # Mana Cost: 20
    fn store_activate_schema(
        &mut self,
        _store_name: &str,
        _version: u32,
        _is_personal: bool,
    ) -> Result<()> {
        Err(crate::error::ApiError::NotSupported(
            "store_activate_schema nicht unterstützt".into(),
        ))
    }

    /// Lehne ein pending Schema ab
    ///
    /// Kann von DIDs mit Trust ≥ 0.8 aufgerufen werden während der Challenge-Periode.
    ///
    /// # Arguments
    /// - `store_name`: Name des Stores
    /// - `version`: Die abzulehnende Version
    /// - `reason`: Begründung der Ablehnung
    ///
    /// # Mana Cost: 30
    fn store_reject_schema(
        &mut self,
        _store_name: &str,
        _version: u32,
        _reason: &str,
        _is_personal: bool,
    ) -> Result<()> {
        Err(crate::error::ApiError::NotSupported(
            "store_reject_schema nicht unterstützt".into(),
        ))
    }

    /// Berechne die Mana-Kosten für Schema-Änderungen (ohne Ausführung)
    ///
    /// # Mana Cost: 1
    fn store_calculate_evolution_cost(&self, _changes: &[HostSchemaChange]) -> Result<u64> {
        Err(crate::error::ApiError::NotSupported(
            "store_calculate_evolution_cost nicht unterstützt".into(),
        ))
    }
}

/// Stub-Implementation für Tests (gibt Default-Werte zurück)
#[derive(Debug, Clone, Default)]
pub struct StubHost {
    /// Default Trust-Wert
    pub default_trust: [f64; 6],
    /// Simulierte Balances (DID -> Balance)
    pub balances: std::collections::HashMap<String, u64>,
    /// Simulierte Credentials (DID -> Vec<Schema>)
    pub credentials: std::collections::HashMap<String, Vec<String>>,
    /// Simulierte DIDs
    pub known_dids: std::collections::HashSet<String>,
    /// Log-Nachrichten
    pub logs: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    /// Simulierter In-Memory Store für Tests
    /// Key: (store_name, is_personal, key) -> Value
    pub store_data: std::sync::Arc<
        std::sync::Mutex<std::collections::HashMap<(String, bool, String), HostStoreValue>>,
    >,
    /// Aktueller Store-Kontext
    pub store_context: Option<StoreContext>,
}

impl StubHost {
    /// Erstelle neuen StubHost mit Default-Trust 0.5
    pub fn new() -> Self {
        Self {
            default_trust: [0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            balances: std::collections::HashMap::new(),
            credentials: std::collections::HashMap::new(),
            known_dids: std::collections::HashSet::new(),
            logs: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            store_data: std::sync::Arc::new(
                std::sync::Mutex::new(std::collections::HashMap::new()),
            ),
            store_context: None,
        }
    }

    /// Setze Trust für eine DID
    pub fn with_trust(mut self, did: &str, trust: [f64; 6]) -> Self {
        // Speichere spezifischen Trust (hier vereinfacht über default)
        self.default_trust = trust;
        self.known_dids.insert(did.to_string());
        self
    }

    /// Füge DID mit Balance hinzu
    pub fn with_balance(mut self, did: &str, balance: u64) -> Self {
        self.balances.insert(did.to_string(), balance);
        self.known_dids.insert(did.to_string());
        self
    }

    /// Füge Credential hinzu
    pub fn with_credential(mut self, did: &str, schema: &str) -> Self {
        self.credentials
            .entry(did.to_string())
            .or_default()
            .push(schema.to_string());
        self.known_dids.insert(did.to_string());
        self
    }

    /// Hole geloggte Nachrichten
    pub fn get_logs(&self) -> Vec<String> {
        self.logs.lock().unwrap().clone()
    }

    /// Pre-populate Store mit Testdaten
    pub fn with_store_data(
        self,
        store_name: &str,
        is_personal: bool,
        key: &str,
        value: HostStoreValue,
    ) -> Self {
        self.store_data.lock().unwrap().insert(
            (store_name.to_string(), is_personal, key.to_string()),
            value,
        );
        self
    }
}

impl HostInterface for StubHost {
    fn get_trust_vector(&self, did: &str) -> Result<[f64; 6]> {
        if self.known_dids.contains(did) || self.known_dids.is_empty() {
            Ok(self.default_trust)
        } else {
            // Unbekannte DID: Newcomer Trust
            Ok([0.1, 0.1, 0.1, 0.1, 0.1, 0.1])
        }
    }

    fn has_credential(&self, did: &str, schema: &str) -> Result<bool> {
        Ok(self
            .credentials
            .get(did)
            .map(|creds| creds.contains(&schema.to_string()))
            .unwrap_or(false))
    }

    fn get_balance(&self, did: &str) -> Result<u64> {
        Ok(*self.balances.get(did).unwrap_or(&0))
    }

    fn resolve_did(&self, did: &str) -> Result<bool> {
        Ok(self.known_dids.contains(did) || self.known_dids.is_empty())
    }

    fn get_timestamp(&self) -> u64 {
        // Für Tests: fester Timestamp
        1700000000
    }

    fn log(&self, message: &str) {
        self.logs.lock().unwrap().push(message.to_string());
    }

    // Store-Operationen für Tests

    fn set_store_context(&mut self, ctx: StoreContext) -> Result<()> {
        self.store_context = Some(ctx);
        Ok(())
    }

    fn store_get(
        &self,
        store_name: &str,
        key: &str,
        is_personal: bool,
    ) -> Result<Option<HostStoreValue>> {
        let data = self.store_data.lock().unwrap();
        Ok(data
            .get(&(store_name.to_string(), is_personal, key.to_string()))
            .cloned())
    }

    fn store_put(
        &mut self,
        store_name: &str,
        key: &str,
        value: HostStoreValue,
        is_personal: bool,
    ) -> Result<()> {
        let mut data = self.store_data.lock().unwrap();
        data.insert(
            (store_name.to_string(), is_personal, key.to_string()),
            value,
        );
        Ok(())
    }

    fn store_delete(&mut self, store_name: &str, key: &str, is_personal: bool) -> Result<bool> {
        let mut data = self.store_data.lock().unwrap();
        Ok(data
            .remove(&(store_name.to_string(), is_personal, key.to_string()))
            .is_some())
    }

    fn store_exists(&self, store_name: &str, is_personal: bool) -> Result<bool> {
        let data = self.store_data.lock().unwrap();
        // Prüfe ob irgendein Key für diesen Store existiert
        Ok(data
            .keys()
            .any(|(s, p, _)| s == store_name && *p == is_personal))
    }

    fn store_count(&self, store_name: &str, is_personal: bool) -> Result<usize> {
        let data = self.store_data.lock().unwrap();
        Ok(data
            .keys()
            .filter(|(s, p, _)| s == store_name && *p == is_personal)
            .count())
    }

    fn store_list_keys(
        &self,
        store_name: &str,
        prefix: Option<&str>,
        limit: usize,
        is_personal: bool,
    ) -> Result<Vec<String>> {
        let data = self.store_data.lock().unwrap();
        let keys: Vec<String> = data
            .keys()
            .filter(|(s, p, k)| {
                s == store_name
                    && *p == is_personal
                    && prefix.map(|pre| k.starts_with(pre)).unwrap_or(true)
            })
            .map(|(_, _, k)| k.clone())
            .take(limit)
            .collect();
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_host_default() {
        let host = StubHost::new();

        let trust = host.get_trust_vector("did:erynoa:self:alice").unwrap();
        assert_eq!(trust, [0.5, 0.5, 0.5, 0.5, 0.5, 0.5]);

        let balance = host.get_balance("did:erynoa:self:alice").unwrap();
        assert_eq!(balance, 0);
    }

    #[test]
    fn test_stub_host_with_balance() {
        let host = StubHost::new().with_balance("did:erynoa:self:alice", 1000);

        let balance = host.get_balance("did:erynoa:self:alice").unwrap();
        assert_eq!(balance, 1000);
    }

    #[test]
    fn test_stub_host_with_credential() {
        let host = StubHost::new().with_credential("did:erynoa:self:alice", "email-verified");

        assert!(host
            .has_credential("did:erynoa:self:alice", "email-verified")
            .unwrap());
        assert!(!host
            .has_credential("did:erynoa:self:alice", "kyc-verified")
            .unwrap());
    }

    #[test]
    fn test_stub_host_logging() {
        let host = StubHost::new();

        host.log("Test message 1");
        host.log("Test message 2");

        let logs = host.get_logs();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0], "Test message 1");
    }
}
