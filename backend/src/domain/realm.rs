//! # Realm Types
//!
//! Realm-Hierarchie gemäß Axiom Κ1.
//!
//! ## Axiom-Referenz
//!
//! - **Κ1 (Monotone Regelvererbung)**: `∀ 𝒞₁ ⊂ 𝒞₂ : rules(𝒞₁) ⊇ rules(𝒞₂)`
//!   "Kind-Kategorien können Regeln hinzufügen, nie entfernen."
//!
//! ## Hierarchie
//!
//! ```text
//! 𝒞_RootRealm ⊃ 𝒞_VirtualRealm ⊃ 𝒞_Partition
//! ```
//!
//! ## Speicherverwaltung
//!
//! Realms können dynamische Stores definieren:
//! - `initial_setup_policy`: ECL-Policy für automatische Einrichtung beim Join
//! - `default_shared_stores`: Gemeinsame Stores für alle Mitglieder
//! - `default_personal_stores`: Persönliche Stores pro Mitglied

use crate::local::{SchemaFieldType, StoreSchema, StoreTemplate};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Realm-Identifikator
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RealmId(pub String);

impl RealmId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Root-Realm ID
    pub fn root() -> Self {
        Self("realm:root".to_string())
    }
}

impl std::fmt::Display for RealmId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Eine Regel im Regelset eines Realms
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rule {
    /// Eindeutiger Regel-Identifikator
    pub id: String,
    /// Name der Regel (z.B. "GDPR", "MiCA")
    pub name: String,
    /// Kategorie (compliance, governance, trust, economic)
    pub category: RuleCategory,
    /// Beschreibung
    pub description: String,
    /// Ist diese Regel optional?
    pub optional: bool,
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

    /// Anzahl der Regeln
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

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
    fn min_trust(&self) -> f64;
}

/// Das Root-Realm (oberste Ebene)
///
/// Enthält die 28 Kern-Axiome (Κ1-Κ28) als unveränderliche Regeln.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootRealm {
    pub id: RealmId,
    pub name: String,
    pub rules: RealmRules,
}

impl Default for RootRealm {
    fn default() -> Self {
        let mut rules = RealmRules::new();

        // Die 28 Kern-Axiome als Regeln
        for i in 1..=28 {
            rules.add(Rule {
                id: format!("K{}", i),
                name: format!("Kern-Axiom Κ{}", i),
                category: RuleCategory::Technical,
                description: format!("Fundamentales Axiom {} des Erynoa-Protokolls", i),
                optional: false,
            });
        }

        Self {
            id: RealmId::root(),
            name: "Root Realm".to_string(),
            rules,
        }
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

    fn min_trust(&self) -> f64 {
        0.0 // Jeder kann dem Root-Realm beitreten
    }
}

/// Ein VirtualRealm (mittlere Ebene)
///
/// Kann zusätzliche Regeln definieren, z.B. für regionale Compliance.
/// Definiert auch die Speicherstruktur für Mitglieder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualRealm {
    pub id: RealmId,
    pub name: String,
    pub parent_id: RealmId,
    pub rules: RealmRules,
    pub min_trust: f64,
    pub governance_type: GovernanceType,
    /// ECL-Policy die beim Join ausgeführt wird
    #[serde(default)]
    pub initial_setup_policy: Option<String>,
    /// Gemeinsame Stores für alle Mitglieder
    #[serde(default)]
    pub default_shared_stores: Vec<StoreTemplate>,
    /// Persönliche Stores die für jedes neue Mitglied erstellt werden
    #[serde(default)]
    pub default_personal_stores: Vec<StoreTemplate>,
    /// Beschreibung des Realms
    #[serde(default)]
    pub description: String,
}

/// Governance-Typ eines Realms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GovernanceType {
    /// Κ21: Quadratisches Voting
    Quadratic,
    /// Token-basiertes Voting
    Token,
    /// Reputation-basiertes Voting
    Reputation,
}

impl Default for GovernanceType {
    fn default() -> Self {
        Self::Quadratic
    }
}

impl VirtualRealm {
    /// Erstelle neues VirtualRealm
    pub fn new(
        id: RealmId,
        name: impl Into<String>,
        parent_id: RealmId,
        parent_rules: &RealmRules,
    ) -> Self {
        // Κ1: Kind erbt alle Regeln des Parents
        let mut rules = RealmRules::new();
        for rule in &parent_rules.rules {
            rules.add(rule.clone());
        }

        Self {
            id,
            name: name.into(),
            parent_id,
            rules,
            min_trust: 0.3, // Default
            governance_type: GovernanceType::Quadratic,
            initial_setup_policy: None,
            default_shared_stores: Vec::new(),
            default_personal_stores: Vec::new(),
            description: String::new(),
        }
    }

    /// Füge zusätzliche Regel hinzu (Κ1: nur hinzufügen)
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.add(rule);
    }

    /// Builder: Setze min_trust
    pub fn with_min_trust(mut self, min_trust: f64) -> Self {
        self.min_trust = min_trust;
        self
    }

    /// Builder: Setze Governance-Typ
    pub fn with_governance(mut self, governance_type: GovernanceType) -> Self {
        self.governance_type = governance_type;
        self
    }

    /// Builder: Setze Initial-Setup-Policy (ECL)
    pub fn with_setup_policy(mut self, policy: impl Into<String>) -> Self {
        self.initial_setup_policy = Some(policy.into());
        self
    }

    /// Builder: Füge shared Store hinzu
    pub fn with_shared_store(mut self, template: StoreTemplate) -> Self {
        self.default_shared_stores.push(template);
        self
    }

    /// Builder: Füge personal Store hinzu
    pub fn with_personal_store(mut self, template: StoreTemplate) -> Self {
        self.default_personal_stores.push(template);
        self
    }

    /// Builder: Setze Beschreibung
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Erstelle Standard-Social-Realm mit typischen Stores
    pub fn social_realm(id: RealmId, name: impl Into<String>, parent_id: RealmId, parent_rules: &RealmRules) -> Self {
        use std::collections::HashMap;

        Self::new(id, name, parent_id, parent_rules)
            .with_description("Social Realm für Austausch und Kommunikation")
            .with_shared_store(StoreTemplate {
                schema: StoreSchema::new("posts", false)
                    .with_field("id", SchemaFieldType::String)
                    .with_field("text", SchemaFieldType::String)
                    .with_field("author", SchemaFieldType::Did)
                    .with_field("timestamp", SchemaFieldType::Timestamp)
                    .with_field("replies", SchemaFieldType::List {
                        item_type: Box::new(SchemaFieldType::Reference {
                            target_store: "posts".to_string(),
                        }),
                    })
                    .with_index("author")
                    .with_index("timestamp"),
                optional: false,
                description: "Öffentliche Posts".to_string(),
            })
            .with_personal_store(StoreTemplate {
                schema: StoreSchema::new("profile", true)
                    .with_field("bio", SchemaFieldType::String)
                    .with_field("avatar_hash", SchemaFieldType::Optional {
                        inner: Box::new(SchemaFieldType::String),
                    })
                    .with_field("settings", SchemaFieldType::Object {
                        fields: {
                            let mut m = HashMap::new();
                            m.insert("theme".to_string(), SchemaFieldType::String);
                            m.insert("notifications".to_string(), SchemaFieldType::Bool);
                            m
                        },
                    }),
                optional: false,
                description: "Dein persönliches Profil".to_string(),
            })
            .with_personal_store(StoreTemplate {
                schema: StoreSchema::new("drafts", true)
                    .with_field("id", SchemaFieldType::String)
                    .with_field("text", SchemaFieldType::String)
                    .with_field("private", SchemaFieldType::Bool),
                optional: true,
                description: "Private Entwürfe".to_string(),
            })
    }

    /// Erstelle Standard-Marketplace-Realm
    pub fn marketplace_realm(id: RealmId, name: impl Into<String>, parent_id: RealmId, parent_rules: &RealmRules) -> Self {
        Self::new(id, name, parent_id, parent_rules)
            .with_description("Marketplace Realm für Handel")
            .with_min_trust(0.5)
            .with_shared_store(StoreTemplate {
                schema: StoreSchema::new("listings", false)
                    .with_field("id", SchemaFieldType::String)
                    .with_field("title", SchemaFieldType::String)
                    .with_field("description", SchemaFieldType::String)
                    .with_field("price", SchemaFieldType::Number)
                    .with_field("seller", SchemaFieldType::Did)
                    .with_field("category", SchemaFieldType::String)
                    .with_field("active", SchemaFieldType::Bool)
                    .with_index("seller")
                    .with_index("category"),
                optional: false,
                description: "Aktive Angebote".to_string(),
            })
            .with_personal_store(StoreTemplate {
                schema: StoreSchema::new("favorites", true)
                    .with_field("listing_ids", SchemaFieldType::List {
                        item_type: Box::new(SchemaFieldType::String),
                    }),
                optional: true,
                description: "Deine Favoriten".to_string(),
            })
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

    fn min_trust(&self) -> f64 {
        self.min_trust
    }
}

/// Eine Partition (unterste Ebene)
///
/// Spezialisierter Bereich innerhalb eines VirtualRealms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    pub id: RealmId,
    pub name: String,
    pub virtual_realm_id: RealmId,
    pub rules: RealmRules,
    pub min_trust: f64,
    /// Validator-DIDs für diesen Partition
    pub validators: Vec<crate::domain::DID>,
}

impl Partition {
    /// Erstelle neue Partition
    pub fn new(
        id: RealmId,
        name: impl Into<String>,
        virtual_realm_id: RealmId,
        virtual_realm_rules: &RealmRules,
    ) -> Self {
        // Κ1: Partition erbt alle Regeln des VirtualRealms
        let mut rules = RealmRules::new();
        for rule in &virtual_realm_rules.rules {
            rules.add(rule.clone());
        }

        Self {
            id,
            name: name.into(),
            virtual_realm_id,
            rules,
            min_trust: 0.5, // Default höher als VirtualRealm
            validators: vec![],
        }
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
        Some(&self.virtual_realm_id)
    }

    fn rules(&self) -> &RealmRules {
        &self.rules
    }

    fn min_trust(&self) -> f64 {
        self.min_trust
    }
}

/// Prüft ob ein Realm-Crossing gültig ist
pub fn validate_realm_crossing(from: &dyn Realm, to: &dyn Realm) -> RealmCrossingResult {
    // Κ1: Finde gemeinsamen Vorfahren
    let common_ancestor = find_common_ancestor(from, to);

    // Berechne Pfadlänge
    let path_length = calculate_path_length(from, to, &common_ancestor);

    RealmCrossingResult {
        valid: true, // Basis-Validierung, weitere Checks in Gateway
        common_ancestor,
        path_length,
        additional_rules: to
            .rules()
            .rules
            .difference(&from.rules().rules)
            .cloned()
            .collect(),
    }
}

fn find_common_ancestor(_from: &dyn Realm, _to: &dyn Realm) -> RealmId {
    // Simplified: In der echten Implementierung würde man den Baum traversieren
    RealmId::root()
}

fn calculate_path_length(_from: &dyn Realm, _to: &dyn Realm, _ancestor: &RealmId) -> usize {
    // Simplified
    2
}

/// Ergebnis einer Realm-Crossing-Validierung
#[derive(Debug, Clone)]
pub struct RealmCrossingResult {
    pub valid: bool,
    pub common_ancestor: RealmId,
    pub path_length: usize,
    pub additional_rules: HashSet<Rule>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_realm() {
        let root = RootRealm::default();
        assert_eq!(root.rules.len(), 28); // 28 Kern-Axiome
        assert!(root.rules.has_rule("K1"));
        assert!(root.rules.has_rule("K28"));
    }

    #[test]
    fn test_virtual_realm_inherits_rules() {
        let root = RootRealm::default();
        let virtual_realm = VirtualRealm::new(
            RealmId::new("realm:erynoa:eu-trade"),
            "EU Trade",
            root.id.clone(),
            &root.rules,
        );

        // Κ1: VirtualRealm erbt alle Root-Regeln
        assert!(virtual_realm.rules.is_superset_of(&root.rules));
        assert_eq!(virtual_realm.rules.len(), 28);
    }

    #[test]
    fn test_add_rule_to_virtual_realm() {
        let root = RootRealm::default();
        let mut virtual_realm = VirtualRealm::new(
            RealmId::new("realm:erynoa:eu-trade"),
            "EU Trade",
            root.id.clone(),
            &root.rules,
        );

        let gdpr = Rule {
            id: "GDPR".to_string(),
            name: "General Data Protection Regulation".to_string(),
            category: RuleCategory::Compliance,
            description: "EU Datenschutzverordnung".to_string(),
            optional: false,
        };

        virtual_realm.add_rule(gdpr);

        // Hat jetzt 28 + 1 = 29 Regeln
        assert_eq!(virtual_realm.rules.len(), 29);
        assert!(virtual_realm.rules.has_rule("GDPR"));

        // Κ1: Immer noch Superset von Root
        assert!(virtual_realm.rules.is_superset_of(&root.rules));
    }

    #[test]
    fn test_partition_inherits_from_virtual_realm() {
        let root = RootRealm::default();
        let mut virtual_realm = VirtualRealm::new(
            RealmId::new("realm:erynoa:eu-trade"),
            "EU Trade",
            root.id.clone(),
            &root.rules,
        );

        virtual_realm.add_rule(Rule {
            id: "GDPR".to_string(),
            name: "GDPR".to_string(),
            category: RuleCategory::Compliance,
            description: "".to_string(),
            optional: false,
        });

        let partition = Partition::new(
            RealmId::new("partition:eu-trade:energy"),
            "Energy Trading",
            virtual_realm.id.clone(),
            &virtual_realm.rules,
        );

        // Κ1: Partition erbt alle VirtualRealm-Regeln (inkl. GDPR)
        assert!(partition.rules.is_superset_of(&virtual_realm.rules));
        assert!(partition.rules.has_rule("GDPR"));
        assert!(partition.rules.has_rule("K1"));
    }
}
