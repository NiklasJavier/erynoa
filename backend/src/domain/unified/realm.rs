//! # Unified Data Model – Realm
//!
//! Realm-Hierarchie gemäß Axiom Κ1 (Monotone Regelvererbung).
//!
//! ## Axiom-Referenz
//!
//! - **Κ1 (Monotone Regelvererbung)**: `∀ 𝒞₁ ⊂ 𝒞₂ : rules(𝒞₁) ⊇ rules(𝒞₂)`
//!
//! ## Hierarchie
//!
//! ```text
//! RootRealm ⊃ VirtualRealm ⊃ Partition
//! ```
//!
//! ## Migration von domain/realm.rs
//!
//! - `RealmId` ist jetzt `UniversalId` (TAG_REALM)
//! - Starke Invariant-Prüfungen für Κ1

use super::primitives::{TemporalCoord, UniversalId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

// ============================================================================
// RealmId – Type Alias für UniversalId
// ============================================================================

/// Realm-Identifikator (Content-Addressed via UniversalId)
pub type RealmId = UniversalId;

/// Root-Realm ID (konstant)
pub const ROOT_REALM_ID: RealmId = UniversalId::NULL;

/// Erstelle RealmId aus Name
pub fn realm_id_from_name(name: &str) -> RealmId {
    UniversalId::new(UniversalId::TAG_REALM, 1, name.as_bytes())
}

// ============================================================================
// Rule
// ============================================================================

/// Eine Regel im Regelset eines Realms
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rule {
    /// Eindeutiger Regel-Identifikator
    pub id: String,
    /// Name der Regel (z.B. "GDPR", "MiCA")
    pub name: String,
    /// Kategorie
    pub category: RuleCategory,
    /// Beschreibung
    pub description: String,
    /// Ist diese Regel optional?
    pub optional: bool,
}

impl Rule {
    /// Erstelle neue Regel
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: RuleCategory,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            category,
            description: description.into(),
            optional: false,
        }
    }

    /// Als optionale Regel markieren
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Kern-Axiom Regel
    pub fn axiom(number: u8, description: impl Into<String>) -> Self {
        Self {
            id: format!("K{}", number),
            name: format!("Kern-Axiom Κ{}", number),
            category: RuleCategory::Technical,
            description: description.into(),
            optional: false,
        }
    }
}

/// Kategorie einer Regel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleCategory {
    /// Compliance/Regulatorisch (GDPR, MiCA, etc.)
    Compliance,
    /// Governance-Regeln
    Governance,
    /// Trust-Regeln
    Trust,
    /// Wirtschaftliche Regeln
    Economic,
    /// Technische Regeln
    Technical,
}

impl fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compliance => write!(f, "compliance"),
            Self::Governance => write!(f, "governance"),
            Self::Trust => write!(f, "trust"),
            Self::Economic => write!(f, "economic"),
            Self::Technical => write!(f, "technical"),
        }
    }
}

// ============================================================================
// RealmRules
// ============================================================================

/// Regelset eines Realms (Κ1)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RealmRules {
    /// Alle aktiven Regeln
    pub rules: HashSet<Rule>,
}

impl RealmRules {
    /// Erstelle leeres Regelset
    pub fn new() -> Self {
        Self {
            rules: HashSet::new(),
        }
    }

    /// Erstelle Regelset mit Kern-Axiomen (1-28)
    pub fn with_axioms() -> Self {
        let mut rules = Self::new();
        for i in 1..=28 {
            rules.add(Rule::axiom(
                i,
                format!("Fundamentales Axiom {} des Erynoa-Protokolls", i),
            ));
        }
        rules
    }

    /// Füge Regel hinzu (Κ1: nur hinzufügen, nie entfernen)
    pub fn add(&mut self, rule: Rule) {
        self.rules.insert(rule);
    }

    /// Prüft ob dieses Regelset ein anderes enthält (Superset)
    /// Κ1: rules(child) ⊇ rules(parent)
    pub fn is_superset_of(&self, other: &RealmRules) -> bool {
        other.rules.iter().all(|r| self.rules.contains(r))
    }

    /// Prüft ob eine bestimmte Regel aktiv ist
    pub fn has_rule(&self, rule_id: &str) -> bool {
        self.rules.iter().any(|r| r.id == rule_id)
    }

    /// Finde Regel nach ID
    pub fn get_rule(&self, rule_id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.id == rule_id)
    }

    /// Regeln einer bestimmten Kategorie
    pub fn by_category(&self, category: RuleCategory) -> Vec<&Rule> {
        self.rules
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    /// Anzahl der Regeln
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Ist leer?
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Merge mit anderem Regelset (nimmt alle Regeln)
    pub fn merge(&mut self, other: &RealmRules) {
        for rule in &other.rules {
            self.rules.insert(rule.clone());
        }
    }
}

// ============================================================================
// GovernanceType
// ============================================================================

/// Governance-Typ eines Realms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GovernanceType {
    /// Κ21: Quadratisches Voting
    Quadratic,
    /// Token-basiertes Voting (1 Token = 1 Vote)
    Token,
    /// Reputation-basiertes Voting
    Reputation,
    /// Delegiertes Voting (Liquid Democracy)
    Delegated,
}

impl Default for GovernanceType {
    fn default() -> Self {
        Self::Quadratic
    }
}

impl fmt::Display for GovernanceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Quadratic => write!(f, "quadratic"),
            Self::Token => write!(f, "token"),
            Self::Reputation => write!(f, "reputation"),
            Self::Delegated => write!(f, "delegated"),
        }
    }
}

// ============================================================================
// Realm Trait
// ============================================================================

/// Basis-Trait für alle Realm-Typen
pub trait Realm: Send + Sync {
    /// Eindeutige ID
    fn id(&self) -> &RealmId;

    /// Name des Realms
    fn name(&self) -> &str;

    /// Eltern-Realm (None für RootRealm)
    fn parent(&self) -> Option<&RealmId>;

    /// Regelset dieses Realms
    fn rules(&self) -> &RealmRules;

    /// Minimaler Trust für Beitritt
    fn min_trust(&self) -> f32;

    /// Governance-Typ
    fn governance_type(&self) -> GovernanceType;

    /// Ist dies das Root-Realm?
    fn is_root(&self) -> bool {
        self.parent().is_none()
    }
}

// ============================================================================
// RootRealm
// ============================================================================

/// Das Root-Realm (oberste Ebene)
///
/// Enthält die 28 Kern-Axiome (Κ1-Κ28) als unveränderliche Regeln.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootRealm {
    pub id: RealmId,
    pub name: String,
    pub rules: RealmRules,
    pub created_at: TemporalCoord,
}

impl Default for RootRealm {
    fn default() -> Self {
        Self {
            id: ROOT_REALM_ID,
            name: "Root Realm".to_string(),
            rules: RealmRules::with_axioms(),
            created_at: TemporalCoord::default(),
        }
    }
}

impl RootRealm {
    /// Erstelle neues Root-Realm
    pub fn new() -> Self {
        Self::default()
    }
}

impl Realm for RootRealm {
    fn id(&self) -> &RealmId {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn parent(&self) -> Option<&RealmId> {
        None // Root hat keinen Parent
    }

    fn rules(&self) -> &RealmRules {
        &self.rules
    }

    fn min_trust(&self) -> f32 {
        0.0 // Jeder kann dem Root-Realm beitreten
    }

    fn governance_type(&self) -> GovernanceType {
        GovernanceType::Quadratic
    }
}

// ============================================================================
// VirtualRealm
// ============================================================================

/// Ein VirtualRealm (mittlere Ebene)
///
/// Kann zusätzliche Regeln definieren, z.B. für regionale Compliance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualRealm {
    pub id: RealmId,
    pub name: String,
    pub parent_id: RealmId,
    pub rules: RealmRules,
    pub min_trust: f32,
    pub governance_type: GovernanceType,
    pub description: String,
    pub created_at: TemporalCoord,
    pub created_by: Option<UniversalId>,
    /// ECL-Policy die beim Join ausgeführt wird
    #[serde(default)]
    pub initial_setup_policy: Option<String>,
    /// Gemeinsame Stores für alle Mitglieder
    #[serde(default)]
    pub default_shared_stores: Vec<StoreTemplate>,
    /// Persönliche Stores die für jedes neue Mitglied erstellt werden
    #[serde(default)]
    pub default_personal_stores: Vec<StoreTemplate>,
}

/// Template für Store-Erstellung beim Realm-Join
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreTemplate {
    /// Name des Stores
    pub name: String,
    /// Store-Typ
    pub store_type: StoreType,
    /// Initiale Kapazität
    pub initial_capacity: u64,
    /// Optionale Beschreibung
    pub description: Option<String>,
}

/// Typ eines Stores
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoreType {
    /// Key-Value Store
    KeyValue,
    /// Event-Log
    EventLog,
    /// Blob-Store
    Blob,
    /// Queue
    Queue,
}

impl StoreTemplate {
    /// Erstelle neues StoreTemplate
    pub fn new(name: impl Into<String>, store_type: StoreType) -> Self {
        Self {
            name: name.into(),
            store_type,
            initial_capacity: 1024,
            description: None,
        }
    }

    /// Mit Kapazität
    pub fn with_capacity(mut self, capacity: u64) -> Self {
        self.initial_capacity = capacity;
        self
    }

    /// Mit Beschreibung
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

impl VirtualRealm {
    /// Erstelle neues VirtualRealm (erbt Parent-Regeln gemäß Κ1)
    pub fn new(
        name: impl Into<String>,
        parent_id: RealmId,
        parent_rules: &RealmRules,
        lamport: u32,
    ) -> Result<Self, RealmError> {
        let name = name.into();
        let id = realm_id_from_name(&name);
        let coord = TemporalCoord::now(lamport, &id);

        // Κ1: Kind erbt alle Regeln des Parents
        let mut rules = RealmRules::new();
        rules.merge(parent_rules);

        Ok(Self {
            id,
            name,
            parent_id,
            rules,
            min_trust: 0.3, // Default
            governance_type: GovernanceType::Quadratic,
            description: String::new(),
            created_at: coord,
            created_by: None,
            initial_setup_policy: None,
            default_shared_stores: Vec::new(),
            default_personal_stores: Vec::new(),
        })
    }

    /// Füge zusätzliche Regel hinzu (Κ1: nur hinzufügen)
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.add(rule);
    }

    /// Setze Initial-Setup-Policy
    pub fn with_initial_setup_policy(mut self, policy: impl Into<String>) -> Self {
        self.initial_setup_policy = Some(policy.into());
        self
    }

    /// Füge persönliches Store-Template hinzu
    pub fn with_personal_store(mut self, template: StoreTemplate) -> Self {
        self.default_personal_stores.push(template);
        self
    }

    /// Füge gemeinsames Store-Template hinzu
    pub fn with_shared_store(mut self, template: StoreTemplate) -> Self {
        self.default_shared_stores.push(template);
        self
    }

    /// Prüfe Κ1-Invariante gegen Parent
    pub fn validate_k1(&self, parent_rules: &RealmRules) -> Result<(), RealmError> {
        if !self.rules.is_superset_of(parent_rules) {
            return Err(RealmError::K1Violation {
                realm: self.id,
                missing_rules: parent_rules
                    .rules
                    .iter()
                    .filter(|r| !self.rules.has_rule(&r.id))
                    .map(|r| r.id.clone())
                    .collect(),
            });
        }
        Ok(())
    }

    /// Builder: Setze min_trust
    pub fn with_min_trust(mut self, min_trust: f32) -> Self {
        self.min_trust = min_trust;
        self
    }

    /// Builder: Setze Governance-Typ
    pub fn with_governance(mut self, governance_type: GovernanceType) -> Self {
        self.governance_type = governance_type;
        self
    }

    /// Builder: Setze Beschreibung
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Builder: Setze Ersteller
    pub fn with_creator(mut self, creator: UniversalId) -> Self {
        self.created_by = Some(creator);
        self
    }
}

impl Realm for VirtualRealm {
    fn id(&self) -> &RealmId {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn parent(&self) -> Option<&RealmId> {
        Some(&self.parent_id)
    }

    fn rules(&self) -> &RealmRules {
        &self.rules
    }

    fn min_trust(&self) -> f32 {
        self.min_trust
    }

    fn governance_type(&self) -> GovernanceType {
        self.governance_type
    }
}

// ============================================================================
// Partition
// ============================================================================

/// Eine Partition (unterste Ebene, für Sharding)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    pub id: RealmId,
    pub name: String,
    pub parent_id: RealmId,
    pub rules: RealmRules,
    pub shard_index: u32,
    pub total_shards: u32,
    pub created_at: TemporalCoord,
}

impl Partition {
    /// Erstelle neue Partition
    pub fn new(
        parent: &VirtualRealm,
        shard_index: u32,
        total_shards: u32,
        lamport: u32,
    ) -> Result<Self, RealmError> {
        if shard_index >= total_shards {
            return Err(RealmError::InvalidPartition {
                index: shard_index,
                total: total_shards,
            });
        }

        let name = format!("{}/shard-{}", parent.name, shard_index);
        let id = realm_id_from_name(&name);
        let coord = TemporalCoord::now(lamport, &id);

        // Κ1: Partition erbt alle Regeln des VirtualRealm
        let mut rules = RealmRules::new();
        rules.merge(&parent.rules);

        Ok(Self {
            id,
            name,
            parent_id: parent.id,
            rules,
            shard_index,
            total_shards,
            created_at: coord,
        })
    }
}

impl Realm for Partition {
    fn id(&self) -> &RealmId {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn parent(&self) -> Option<&RealmId> {
        Some(&self.parent_id)
    }

    fn rules(&self) -> &RealmRules {
        &self.rules
    }

    fn min_trust(&self) -> f32 {
        0.3 // Erbt von Parent
    }

    fn governance_type(&self) -> GovernanceType {
        GovernanceType::Quadratic
    }
}

// ============================================================================
// RealmMembership
// ============================================================================

/// Mitgliedschaft in einem Realm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmMembership {
    /// DID des Mitglieds
    pub member: UniversalId,
    /// Realm-ID
    pub realm_id: RealmId,
    /// Rolle im Realm
    pub role: MemberRole,
    /// Beitrittszeitpunkt
    pub joined_at: TemporalCoord,
    /// Trust zum Zeitpunkt des Beitritts
    pub trust_at_join: f32,
}

/// Rolle eines Mitglieds im Realm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemberRole {
    /// Normales Mitglied
    Member,
    /// Moderator
    Moderator,
    /// Administrator
    Admin,
    /// Gründer
    Founder,
}

impl Default for MemberRole {
    fn default() -> Self {
        Self::Member
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Fehler bei Realm-Operationen
#[derive(Debug, Clone, thiserror::Error)]
pub enum RealmError {
    #[error("Κ1 violated: Realm {realm} missing rules from parent: {missing_rules:?}")]
    K1Violation {
        realm: RealmId,
        missing_rules: Vec<String>,
    },

    #[error("Insufficient trust: required {required}, actual {actual}")]
    InsufficientTrust { required: f32, actual: f32 },

    #[error("Realm not found: {0}")]
    NotFound(RealmId),

    #[error("Already member of realm: {0}")]
    AlreadyMember(RealmId),

    #[error("Invalid partition: index {index} >= total {total}")]
    InvalidPartition { index: u32, total: u32 },
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_realm_has_axioms() {
        let root = RootRealm::default();
        assert_eq!(root.rules.len(), 28);
        assert!(root.rules.has_rule("K1"));
        assert!(root.rules.has_rule("K28"));
    }

    #[test]
    fn test_virtual_realm_inherits_rules() {
        let root = RootRealm::default();
        let virtual_realm = VirtualRealm::new("EU-Realm", root.id, &root.rules, 1).unwrap();

        assert!(virtual_realm.rules.is_superset_of(&root.rules));
        assert!(virtual_realm.validate_k1(&root.rules).is_ok());
    }

    #[test]
    fn test_virtual_realm_add_rule() {
        let root = RootRealm::default();
        let mut virtual_realm = VirtualRealm::new("EU-Realm", root.id, &root.rules, 1).unwrap();

        virtual_realm.add_rule(Rule::new(
            "GDPR",
            "General Data Protection Regulation",
            RuleCategory::Compliance,
            "EU GDPR compliance required",
        ));

        assert!(virtual_realm.rules.has_rule("GDPR"));
        assert_eq!(virtual_realm.rules.len(), 29);
        assert!(virtual_realm.validate_k1(&root.rules).is_ok());
    }

    #[test]
    fn test_k1_violation_detection() {
        let root = RootRealm::default();

        // Manuell ein Realm mit fehlenden Regeln erstellen
        let broken_realm = VirtualRealm {
            id: realm_id_from_name("broken"),
            name: "Broken Realm".into(),
            parent_id: root.id,
            rules: RealmRules::new(), // Leer! Verletzt Κ1
            min_trust: 0.3,
            governance_type: GovernanceType::Quadratic,
            description: String::new(),
            created_at: TemporalCoord::default(),
            created_by: None,
            initial_setup_policy: None,
            default_shared_stores: vec![],
            default_personal_stores: vec![],
        };

        let result = broken_realm.validate_k1(&root.rules);
        assert!(result.is_err());
    }

    #[test]
    fn test_partition_creation() {
        let root = RootRealm::default();
        let virtual_realm = VirtualRealm::new("EU-Realm", root.id, &root.rules, 1).unwrap();

        let partition = Partition::new(&virtual_realm, 0, 4, 2).unwrap();

        assert!(partition.rules.is_superset_of(&virtual_realm.rules));
        assert_eq!(partition.shard_index, 0);
        assert_eq!(partition.total_shards, 4);
    }

    #[test]
    fn test_invalid_partition() {
        let root = RootRealm::default();
        let virtual_realm = VirtualRealm::new("EU-Realm", root.id, &root.rules, 1).unwrap();

        // Index >= total ist ungültig
        let result = Partition::new(&virtual_realm, 5, 4, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_rules_by_category() {
        let mut rules = RealmRules::with_axioms();
        rules.add(Rule::new("GDPR", "GDPR", RuleCategory::Compliance, "GDPR"));
        rules.add(Rule::new("MiCA", "MiCA", RuleCategory::Compliance, "MiCA"));

        let compliance_rules = rules.by_category(RuleCategory::Compliance);
        assert_eq!(compliance_rules.len(), 2);

        let technical_rules = rules.by_category(RuleCategory::Technical);
        assert_eq!(technical_rules.len(), 28);
    }
}
