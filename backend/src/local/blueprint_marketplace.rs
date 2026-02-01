//! # Blueprint Marketplace
//!
//! Dezentraler, trust-basierter Marketplace für wiederverwendbare Templates.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                     BLUEPRINT MARKETPLACE                                   │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   Blueprint = Immutables, versioniertes, content-addressables Template      │
//! │   ┌─────────────────────────────────────────────────────────────────────┐  │
//! │   │ BLAKE3-Hash als ID → Garantierte Integrität                         │  │
//! │   │ Inhalt: Stores + Schemas + Policies + Sagas + Metadaten             │  │
//! │   │ Novelty-Score + Diversity-Contribution (automatisch berechnet)       │  │
//! │   └─────────────────────────────────────────────────────────────────────┘  │
//! │                                                                             │
//! │   Marketplace als dedizierter Realm:                                       │
//! │   ┌─────────────────────────────────────────────────────────────────────┐  │
//! │   │ shared:blueprints    → Listing (Hash → Metadaten)                   │  │
//! │   │ shared:ratings       → Attestation-Events (bayessche Trust-Updates) │  │
//! │   │ shared:deployments   → Statistiken + Contributor-Boost              │  │
//! │   │ shared:categories    → Tags/Hierarchie für Discovery                │  │
//! │   └─────────────────────────────────────────────────────────────────────┘  │
//! │                                                                             │
//! │   Intelligentes Ranking:                                                   │
//! │   ┌─────────────────────────────────────────────────────────────────────┐  │
//! │   │ 𝔼-Beitrag = Deployments × Ratings × Diversity × (1 - Calcification) │  │
//! │   │ Novelty-Score = Surprisal vs. existierende Blueprints               │  │
//! │   │ Trust-Gewichtung = Ω-Alignment der Bewerter                         │  │
//! │   └─────────────────────────────────────────────────────────────────────┘  │
//! │                                                                             │
//! │   Gaming-Resistenz (Axiome Κ19/Κ20):                                       │
//! │   ┌─────────────────────────────────────────────────────────────────────┐  │
//! │   │ Upload: Trust-R > 0.8, Ω > 1.5, Novelty > 3.0                       │  │
//! │   │ Ratings: Ω-Gewichtung, Anomaly-Detection, bayessche Updates         │  │
//! │   │ Power-Cap: Kein Creator dominiert Listings                          │  │
//! │   │ Diversity-Boost: Innovative Blueprints steigen automatisch auf      │  │
//! │   └─────────────────────────────────────────────────────────────────────┘  │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Mana-Kosten
//!
//! | Aktion | Basiskosten | Formel |
//! |--------|-------------|--------|
//! | Upload | 500 | `500 + complexity × 20 + (1.0 / novelty) × 100` |
//! | Rate | 10 | `10 + (1.0 - rater_trust) × 20` |
//! | Deploy | 50 | `50 + blueprint_complexity × 5` |
//! | Fork | 200 | `200 + original_complexity × 10` |
//!
//! ## Trust-Integration
//!
//! - Upload: Erfordert R > 0.8, Ω > 1.5
//! - Rating: Gewichtung nach Ω des Bewerters
//! - Deployment: Boost für Creator-Trust (kleine I-Erhöhung)
//! - Fork: Credit-Chain zum Original (Trust-Vererbung)

use crate::domain::DID;
use crate::local::realm_storage::{StoreSchema, StoreValue};
use anyhow::{anyhow, Result};
use fjall::{Keyspace, PartitionHandle};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// ═══════════════════════════════════════════════════════════════════════════
// Blueprint-Definitionen
// ═══════════════════════════════════════════════════════════════════════════

/// Eindeutiger Blueprint-Identifier (BLAKE3-Hash)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlueprintId(pub String);

impl BlueprintId {
    /// Erstelle neue ID aus Hash
    pub fn new(hash: impl Into<String>) -> Self {
        Self(hash.into())
    }

    /// Berechne ID aus Blueprint-Inhalt
    pub fn from_content(content: &[u8]) -> Self {
        let hash = blake3::hash(content);
        Self(hash.to_hex().to_string())
    }
}

impl std::fmt::Display for BlueprintId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Semantische Versionierung für Blueprints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn initial() -> Self {
        Self::new(1, 0, 0)
    }

    pub fn bump_major(&self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }

    pub fn bump_minor(&self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }

    pub fn bump_patch(&self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Lizenztyp für Blueprints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlueprintLicense {
    /// Frei verwendbar, keine Einschränkungen
    Open,
    /// Attribution erforderlich
    Attribution,
    /// Keine kommerzielle Nutzung
    NonCommercial,
    /// Mana-Gebühr bei Deployment
    Commercial { mana_fee: u64 },
    /// Nur für spezifische Realms
    Restricted { allowed_realms: Vec<String> },
}

/// Blueprint-Kategorie für Discovery
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlueprintCategory {
    /// Social: Profile, Posts, Kommentare
    Social,
    /// Governance: Abstimmungen, Proposals
    Governance,
    /// Commerce: Shops, Produkte, Bestellungen
    Commerce,
    /// Content: Artikel, Medien, Dokumentation
    Content,
    /// Gaming: Achievements, Leaderboards
    Gaming,
    /// Identity: Credentials, Verifikation
    Identity,
    /// Infrastructure: Monitoring, Logging
    Infrastructure,
    /// Benutzerdefinierte Kategorie
    Custom(String),
}

// ═══════════════════════════════════════════════════════════════════════════
// Blueprint-Inhalte
// ═══════════════════════════════════════════════════════════════════════════

/// Store-Definition innerhalb eines Blueprints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintStore {
    /// Store-Name
    pub name: String,
    /// Schema-Definition
    pub schema: StoreSchema,
    /// Ist Personal-Store?
    pub personal: bool,
    /// Optionale Beschreibung
    pub description: Option<String>,
    /// Initialdaten (z.B. für Config-Stores)
    pub initial_data: Option<HashMap<String, StoreValue>>,
}

/// ECL-Policy-Definition innerhalb eines Blueprints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintPolicy {
    /// Policy-Name
    pub name: String,
    /// ECL-Code (validiert bei Upload)
    pub ecl_code: String,
    /// Policy-Typ
    pub policy_type: PolicyType,
    /// Beschreibung
    pub description: Option<String>,
}

/// Typ einer Policy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyType {
    /// Gateway-Policy für Realm-Zugang
    Gateway,
    /// Initial-Setup bei Join
    InitialSetup,
    /// Store-Zugriffskontrolle
    StoreAccess,
    /// Governance-Regeln
    Governance,
    /// Benutzerdefiniert
    Custom(String),
}

/// Saga-Definition innerhalb eines Blueprints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintSaga {
    /// Saga-Name
    pub name: String,
    /// Saga-Schritte (JSON-definiert)
    pub steps: Vec<SagaStep>,
    /// Beschreibung
    pub description: Option<String>,
    /// Geschätzte Mana-Kosten
    pub estimated_mana: u64,
}

/// Ein Schritt in einer Saga
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStep {
    /// Schritt-ID
    pub id: String,
    /// Aktion
    pub action: SagaAction,
    /// Abhängigkeiten (vorherige Schritte)
    pub depends_on: Vec<String>,
    /// Kompensation bei Fehler
    pub compensate: Option<SagaAction>,
}

/// Saga-Aktion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SagaAction {
    /// Store erstellen
    CreateStore { store: BlueprintStore },
    /// Daten schreiben
    PutData {
        store: String,
        key: String,
        value: StoreValue,
    },
    /// Policy installieren
    InstallPolicy { policy: BlueprintPolicy },
    /// Trust prüfen
    CheckTrust { min_r: f64, min_omega: f64 },
    /// Mana reservieren
    ReserveMana { amount: u64 },
    /// Benutzerdefiniert (ECL-Aufruf)
    Custom { ecl_code: String },
}

// ═══════════════════════════════════════════════════════════════════════════
// Haupt-Blueprint-Struktur
// ═══════════════════════════════════════════════════════════════════════════

/// Ein Blueprint ist ein immutables, versioniertes Template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    /// Einzigartige ID (BLAKE3-Hash des Inhalts)
    pub id: BlueprintId,

    // ─────────────────────────────────────────────────────────────────────────
    // Metadaten
    // ─────────────────────────────────────────────────────────────────────────
    /// Name des Blueprints
    pub name: String,
    /// Beschreibung
    pub description: String,
    /// Semantische Version
    pub version: SemVer,
    /// Creator-DID
    pub creator_did: String,
    /// Erstellungszeitpunkt (Unix Seconds)
    pub created_at: u64,
    /// Tags für Discovery
    pub tags: Vec<String>,
    /// Kategorie
    pub category: BlueprintCategory,
    /// Lizenz
    pub license: BlueprintLicense,

    // ─────────────────────────────────────────────────────────────────────────
    // Inhalt
    // ─────────────────────────────────────────────────────────────────────────
    /// Store-Definitionen
    pub stores: Vec<BlueprintStore>,
    /// Policy-Definitionen
    pub policies: Vec<BlueprintPolicy>,
    /// Saga-Definitionen
    pub sagas: Vec<BlueprintSaga>,

    // ─────────────────────────────────────────────────────────────────────────
    // Versionierung
    // ─────────────────────────────────────────────────────────────────────────
    /// Vorgänger-Blueprint (für Versioning)
    pub predecessor: Option<BlueprintId>,
    /// Original-Blueprint (für Forks)
    pub forked_from: Option<BlueprintId>,

    // ─────────────────────────────────────────────────────────────────────────
    // Automatisch berechnete Metriken
    // ─────────────────────────────────────────────────────────────────────────
    /// Komplexität (automatisch berechnet)
    pub complexity: u64,
    /// Novelty-Score (Surprisal vs. existierende Blueprints)
    pub novelty_score: f64,
    /// Diversity-Contribution (neue Konzepte)
    pub diversity_contribution: f64,
}

impl Blueprint {
    /// Erstelle neuen Blueprint-Builder
    pub fn builder(name: impl Into<String>, creator_did: impl Into<String>) -> BlueprintBuilder {
        BlueprintBuilder::new(name, creator_did)
    }

    /// Berechne BLAKE3-Hash des Blueprint-Inhalts
    pub fn compute_id(&self) -> BlueprintId {
        let content = serde_json::to_vec(self).unwrap_or_default();
        BlueprintId::from_content(&content)
    }

    /// Berechne Komplexität des Blueprints
    pub fn compute_complexity(&self) -> u64 {
        let store_complexity: u64 = self.stores.iter().map(|s| s.schema.complexity()).sum();

        let policy_complexity: u64 = self
            .policies
            .iter()
            .map(|p| p.ecl_code.len() as u64 / 10)
            .sum();

        let saga_complexity: u64 = self.sagas.iter().map(|s| s.steps.len() as u64 * 10).sum();

        store_complexity + policy_complexity + saga_complexity
    }

    /// Berechne Mana-Kosten für Upload
    pub fn upload_mana_cost(&self) -> u64 {
        let base_cost = 500;
        let complexity_cost = self.complexity * 20;
        let novelty_discount = if self.novelty_score > 0.0 {
            (100.0 / self.novelty_score) as u64
        } else {
            100
        };

        base_cost + complexity_cost + novelty_discount
    }

    /// Berechne Mana-Kosten für Deployment
    pub fn deployment_mana_cost(&self) -> u64 {
        let base_cost = 50;
        let complexity_cost = self.complexity * 5;

        // Lizenzgebühr
        let license_fee = match &self.license {
            BlueprintLicense::Commercial { mana_fee } => *mana_fee,
            _ => 0,
        };

        base_cost + complexity_cost + license_fee
    }

    /// Validiere Blueprint
    pub fn validate(&self) -> Result<()> {
        // Name prüfen
        if self.name.is_empty() || self.name.len() > 100 {
            return Err(anyhow!("Blueprint name must be 1-100 characters"));
        }

        // Mindestens ein Store oder Policy
        if self.stores.is_empty() && self.policies.is_empty() {
            return Err(anyhow!(
                "Blueprint must contain at least one store or policy"
            ));
        }

        // Stores validieren
        for store in &self.stores {
            store.schema.validate_depth(5)?;
        }

        // Policies validieren (ECL-Syntax)
        for policy in &self.policies {
            if policy.ecl_code.is_empty() {
                return Err(anyhow!("Policy '{}' has empty ECL code", policy.name));
            }
        }

        Ok(())
    }
}

/// Builder für Blueprints
pub struct BlueprintBuilder {
    name: String,
    description: String,
    creator_did: String,
    version: SemVer,
    tags: Vec<String>,
    category: BlueprintCategory,
    license: BlueprintLicense,
    stores: Vec<BlueprintStore>,
    policies: Vec<BlueprintPolicy>,
    sagas: Vec<BlueprintSaga>,
    predecessor: Option<BlueprintId>,
    forked_from: Option<BlueprintId>,
}

impl BlueprintBuilder {
    pub fn new(name: impl Into<String>, creator_did: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            creator_did: creator_did.into(),
            version: SemVer::initial(),
            tags: Vec::new(),
            category: BlueprintCategory::Custom("general".to_string()),
            license: BlueprintLicense::Open,
            stores: Vec::new(),
            policies: Vec::new(),
            sagas: Vec::new(),
            predecessor: None,
            forked_from: None,
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn version(mut self, version: SemVer) -> Self {
        self.version = version;
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn category(mut self, category: BlueprintCategory) -> Self {
        self.category = category;
        self
    }

    pub fn license(mut self, license: BlueprintLicense) -> Self {
        self.license = license;
        self
    }

    pub fn store(mut self, store: BlueprintStore) -> Self {
        self.stores.push(store);
        self
    }

    pub fn policy(mut self, policy: BlueprintPolicy) -> Self {
        self.policies.push(policy);
        self
    }

    pub fn saga(mut self, saga: BlueprintSaga) -> Self {
        self.sagas.push(saga);
        self
    }

    pub fn predecessor(mut self, id: BlueprintId) -> Self {
        self.predecessor = Some(id);
        self
    }

    pub fn forked_from(mut self, id: BlueprintId) -> Self {
        self.forked_from = Some(id);
        self
    }

    pub fn build(self) -> Blueprint {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut blueprint = Blueprint {
            id: BlueprintId::new("pending"),
            name: self.name,
            description: self.description,
            version: self.version,
            creator_did: self.creator_did,
            created_at: timestamp,
            tags: self.tags,
            category: self.category,
            license: self.license,
            stores: self.stores,
            policies: self.policies,
            sagas: self.sagas,
            predecessor: self.predecessor,
            forked_from: self.forked_from,
            complexity: 0,
            novelty_score: 0.0,
            diversity_contribution: 0.0,
        };

        blueprint.complexity = blueprint.compute_complexity();
        blueprint.id = blueprint.compute_id();

        blueprint
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Marketplace-Metriken
// ═══════════════════════════════════════════════════════════════════════════

/// Statistiken für ein Blueprint im Marketplace
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlueprintStats {
    /// Anzahl Deployments
    pub deployment_count: u64,
    /// Durchschnittliche Bewertung (0-5)
    pub average_rating: f64,
    /// Anzahl Bewertungen
    pub rating_count: u64,
    /// Trust-gewichtete Bewertung (Ω-adjustiert)
    pub weighted_rating: f64,
    /// 𝔼-Beitrag (World-Formula Contribution)
    pub e_contribution: f64,
    /// Letztes Deployment
    pub last_deployment: u64,
    /// Einnahmen (Mana) für Creator
    pub total_earnings: u64,
}

impl BlueprintStats {
    /// Berechne 𝔼-Beitrag
    pub fn compute_e_contribution(&self, novelty: f64, diversity: f64) -> f64 {
        let deployment_factor = (self.deployment_count as f64).ln_1p();
        let rating_factor = self.weighted_rating / 5.0;
        let novelty_factor = novelty / 10.0;
        let diversity_factor = diversity;

        deployment_factor * rating_factor * novelty_factor * diversity_factor
    }
}

/// Eine Bewertung für ein Blueprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintRating {
    /// Bewerter-DID
    pub rater_did: String,
    /// Blueprint-ID
    pub blueprint_id: BlueprintId,
    /// Bewertung (1-5)
    pub score: u8,
    /// Kommentar
    pub comment: Option<String>,
    /// Zeitstempel
    pub timestamp: u64,
    /// Trust-R des Bewerters zum Zeitpunkt
    pub rater_trust_r: f64,
    /// Trust-Ω des Bewerters zum Zeitpunkt
    pub rater_trust_omega: f64,
}

impl BlueprintRating {
    /// Berechne gewichtete Bewertung (Ω-adjustiert)
    pub fn weighted_score(&self) -> f64 {
        // Höheres Ω = höheres Gewicht (alignierte Bewertungen zählen mehr)
        let omega_weight = self.rater_trust_omega.min(3.0) / 3.0;
        let trust_weight = self.rater_trust_r;

        self.score as f64 * omega_weight * trust_weight
    }

    /// Mana-Kosten für diese Bewertung
    pub fn mana_cost(&self) -> u64 {
        let base: u64 = 10;
        let trust_discount = (self.rater_trust_r * 20.0) as u64;
        base.saturating_sub(trust_discount).max(5)
    }
}

/// Deployment-Eintrag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintDeployment {
    /// Deployment-ID
    pub id: String,
    /// Blueprint-ID
    pub blueprint_id: BlueprintId,
    /// Deployer-DID
    pub deployer_did: String,
    /// Ziel-Realm
    pub target_realm: String,
    /// Zeitstempel
    pub timestamp: u64,
    /// Gezahlte Mana
    pub mana_paid: u64,
    /// Erfolg?
    pub success: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Marketplace-Konfiguration
// ═══════════════════════════════════════════════════════════════════════════

/// Konfiguration für den Blueprint Marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    /// Minimaler Trust-R für Upload
    pub min_upload_trust_r: f64,
    /// Minimaler Trust-Ω für Upload
    pub min_upload_trust_omega: f64,
    /// Minimaler Novelty-Score für Upload
    pub min_novelty_score: f64,
    /// Minimaler Trust-R für Rating
    pub min_rating_trust_r: f64,
    /// Maximale Ratings pro Tag pro DID
    pub max_ratings_per_day: u32,
    /// Anomaly-Detection: Max koordinierte Ratings
    pub max_coordinated_ratings: u32,
    /// Power-Cap: Max Blueprints pro Creator in Top-100
    pub creator_power_cap: u32,
    /// Diversity-Boost-Faktor
    pub diversity_boost_factor: f64,
    /// Creator-Trust-Boost pro Deployment
    pub deployment_trust_boost: f64,
}

impl Default for MarketplaceConfig {
    fn default() -> Self {
        Self {
            min_upload_trust_r: 0.8,
            min_upload_trust_omega: 1.5,
            min_novelty_score: 3.0,
            min_rating_trust_r: 0.3,
            max_ratings_per_day: 10,
            max_coordinated_ratings: 5,
            creator_power_cap: 10,
            diversity_boost_factor: 1.5,
            deployment_trust_boost: 0.001,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Novelty-Berechnung (Surprisal)
// ═══════════════════════════════════════════════════════════════════════════

/// Novelty-Calculator für Blueprints
#[derive(Debug)]
pub struct NoveltyCalculator {
    /// Bekannte Konzepte (Store-Namen, Policy-Typen, etc.)
    known_concepts: HashSet<String>,
    /// Konzept-Häufigkeiten
    concept_frequencies: HashMap<String, u64>,
    /// Gesamt-Blueprints
    total_blueprints: u64,
}

impl NoveltyCalculator {
    pub fn new() -> Self {
        Self {
            known_concepts: HashSet::new(),
            concept_frequencies: HashMap::new(),
            total_blueprints: 0,
        }
    }

    /// Extrahiere Konzepte aus einem Blueprint
    fn extract_concepts(&self, blueprint: &Blueprint) -> Vec<String> {
        let mut concepts = Vec::new();

        // Store-Namen und Schema-Felder
        for store in &blueprint.stores {
            concepts.push(format!("store:{}", store.name));
            for field_name in store.schema.fields.keys() {
                concepts.push(format!("field:{}", field_name));
            }
        }

        // Policy-Typen
        for policy in &blueprint.policies {
            concepts.push(format!("policy:{:?}", policy.policy_type));
        }

        // Tags
        for tag in &blueprint.tags {
            concepts.push(format!("tag:{}", tag));
        }

        // Kategorie
        concepts.push(format!("category:{:?}", blueprint.category));

        concepts
    }

    /// Berechne Novelty-Score (Surprisal)
    pub fn compute_novelty(&self, blueprint: &Blueprint) -> f64 {
        let concepts = self.extract_concepts(blueprint);

        if concepts.is_empty() || self.total_blueprints == 0 {
            return 10.0; // Maximum novelty für erste Blueprints
        }

        let mut total_surprisal = 0.0;
        let mut concept_count = 0;

        for concept in &concepts {
            let frequency = *self.concept_frequencies.get(concept).unwrap_or(&0);
            let probability = if frequency > 0 {
                frequency as f64 / self.total_blueprints as f64
            } else {
                1.0 / (self.total_blueprints as f64 + 1.0) // Laplace smoothing
            };

            // Surprisal = -log2(P)
            let surprisal = -probability.log2();
            total_surprisal += surprisal;
            concept_count += 1;
        }

        if concept_count > 0 {
            (total_surprisal / concept_count as f64).min(10.0)
        } else {
            5.0
        }
    }

    /// Berechne Diversity-Contribution
    pub fn compute_diversity(&self, blueprint: &Blueprint) -> f64 {
        let concepts = self.extract_concepts(blueprint);
        let mut new_concepts = 0;

        for concept in &concepts {
            if !self.known_concepts.contains(concept) {
                new_concepts += 1;
            }
        }

        if concepts.is_empty() {
            return 0.0;
        }

        new_concepts as f64 / concepts.len() as f64
    }

    /// Registriere Blueprint (für zukünftige Novelty-Berechnungen)
    pub fn register_blueprint(&mut self, blueprint: &Blueprint) {
        let concepts = self.extract_concepts(blueprint);

        for concept in concepts {
            self.known_concepts.insert(concept.clone());
            *self.concept_frequencies.entry(concept).or_insert(0) += 1;
        }

        self.total_blueprints += 1;
    }
}

impl Default for NoveltyCalculator {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Marketplace-Store
// ═══════════════════════════════════════════════════════════════════════════

/// Der Blueprint Marketplace
pub struct BlueprintMarketplace {
    /// Blueprints-Partition
    blueprints: PartitionHandle,
    /// Stats-Partition
    stats: PartitionHandle,
    /// Ratings-Partition
    ratings: PartitionHandle,
    /// Deployments-Partition
    deployments: PartitionHandle,
    /// Konfiguration
    config: MarketplaceConfig,
    /// Novelty-Calculator (cached)
    novelty: Arc<RwLock<NoveltyCalculator>>,
    /// Blueprint-Cache
    cache: Arc<RwLock<HashMap<BlueprintId, Blueprint>>>,
}

impl BlueprintMarketplace {
    /// Erstelle neuen Marketplace
    pub fn new(keyspace: &Keyspace, config: MarketplaceConfig) -> Result<Self> {
        let blueprints = keyspace.open_partition("marketplace_blueprints", Default::default())?;
        let stats = keyspace.open_partition("marketplace_stats", Default::default())?;
        let ratings = keyspace.open_partition("marketplace_ratings", Default::default())?;
        let deployments = keyspace.open_partition("marketplace_deployments", Default::default())?;

        let marketplace = Self {
            blueprints,
            stats,
            ratings,
            deployments,
            config,
            novelty: Arc::new(RwLock::new(NoveltyCalculator::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
        };

        // Lade existierende Blueprints in Novelty-Calculator
        marketplace.initialize_novelty_calculator()?;

        Ok(marketplace)
    }

    /// Initialisiere Novelty-Calculator mit existierenden Blueprints
    fn initialize_novelty_calculator(&self) -> Result<()> {
        let mut novelty = self.novelty.write();

        for entry in self.blueprints.iter() {
            let (_, value) = entry?;
            if let Ok(blueprint) = serde_json::from_slice::<Blueprint>(&value) {
                novelty.register_blueprint(&blueprint);
            }
        }

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Upload & Publishing
    // ─────────────────────────────────────────────────────────────────────────

    /// Publiziere ein neues Blueprint
    ///
    /// Prüft Trust-Anforderungen, berechnet Novelty und speichert.
    pub fn publish(
        &self,
        blueprint: Blueprint,
        creator_trust_r: f64,
        creator_trust_omega: f64,
    ) -> Result<PublishResult> {
        self.publish_internal(blueprint, creator_trust_r, creator_trust_omega, false)
    }

    /// Interne Publish-Methode mit Option zum Überspringen des Novelty-Checks
    ///
    /// Wird für Forks und neue Versionen verwendet, da diese absichtlich
    /// ähnlich zu existierenden Blueprints sind.
    fn publish_internal(
        &self,
        mut blueprint: Blueprint,
        creator_trust_r: f64,
        creator_trust_omega: f64,
        skip_novelty_check: bool,
    ) -> Result<PublishResult> {
        // Trust-Prüfung
        if creator_trust_r < self.config.min_upload_trust_r {
            return Err(anyhow!(
                "Insufficient Trust-R for upload: {} < {}",
                creator_trust_r,
                self.config.min_upload_trust_r
            ));
        }

        if creator_trust_omega < self.config.min_upload_trust_omega {
            return Err(anyhow!(
                "Insufficient Trust-Ω for upload: {} < {}",
                creator_trust_omega,
                self.config.min_upload_trust_omega
            ));
        }

        // Validierung
        blueprint.validate()?;

        // Novelty berechnen
        {
            let novelty_calc = self.novelty.read();
            blueprint.novelty_score = novelty_calc.compute_novelty(&blueprint);
            blueprint.diversity_contribution = novelty_calc.compute_diversity(&blueprint);
        }

        // Novelty-Prüfung (wird bei Forks/Versions übersprungen)
        if !skip_novelty_check && blueprint.novelty_score < self.config.min_novelty_score {
            return Err(anyhow!(
                "Insufficient novelty: {} < {}. Blueprint is too similar to existing ones.",
                blueprint.novelty_score,
                self.config.min_novelty_score
            ));
        }

        // ID neu berechnen (mit Novelty-Werten)
        blueprint.id = blueprint.compute_id();

        // Mana-Kosten berechnen
        let mana_cost = blueprint.upload_mana_cost();

        // Speichern
        let bytes = serde_json::to_vec(&blueprint)?;
        self.blueprints.insert(&blueprint.id.0, &bytes)?;

        // Stats initialisieren
        let stats = BlueprintStats::default();
        let stats_bytes = serde_json::to_vec(&stats)?;
        self.stats
            .insert(format!("stats:{}", blueprint.id), &stats_bytes)?;

        // Novelty-Calculator aktualisieren
        {
            let mut novelty_calc = self.novelty.write();
            novelty_calc.register_blueprint(&blueprint);
        }

        // Cache aktualisieren
        {
            let mut cache = self.cache.write();
            cache.insert(blueprint.id.clone(), blueprint.clone());
        }

        tracing::info!(
            blueprint_id = %blueprint.id,
            novelty = blueprint.novelty_score,
            diversity = blueprint.diversity_contribution,
            mana_cost = mana_cost,
            "Blueprint published"
        );

        Ok(PublishResult {
            blueprint_id: blueprint.id,
            mana_cost,
            novelty_score: blueprint.novelty_score,
            diversity_contribution: blueprint.diversity_contribution,
        })
    }

    /// Erstelle neue Version eines Blueprints
    pub fn publish_new_version(
        &self,
        predecessor_id: &BlueprintId,
        mut updated_blueprint: Blueprint,
        creator_trust_r: f64,
        creator_trust_omega: f64,
    ) -> Result<PublishResult> {
        // Predecessor prüfen
        let predecessor = self
            .get_blueprint(predecessor_id)?
            .ok_or_else(|| anyhow!("Predecessor blueprint not found"))?;

        // Creator-Check
        if predecessor.creator_did != updated_blueprint.creator_did {
            return Err(anyhow!(
                "Only the original creator can publish new versions"
            ));
        }

        // Predecessor setzen
        updated_blueprint.predecessor = Some(predecessor_id.clone());

        // Version bumpen (wenn nicht manuell gesetzt)
        if updated_blueprint.version == predecessor.version {
            updated_blueprint.version = predecessor.version.bump_minor();
        }

        // Neue Versionen überspringen Novelty-Check (absichtlich ähnlich)
        self.publish_internal(
            updated_blueprint,
            creator_trust_r,
            creator_trust_omega,
            true,
        )
    }

    /// Fork ein existierendes Blueprint
    pub fn fork(
        &self,
        original_id: &BlueprintId,
        mut forked_blueprint: Blueprint,
        creator_trust_r: f64,
        creator_trust_omega: f64,
    ) -> Result<PublishResult> {
        // Original prüfen
        let _original = self
            .get_blueprint(original_id)?
            .ok_or_else(|| anyhow!("Original blueprint not found"))?;

        // Fork-Referenz setzen
        forked_blueprint.forked_from = Some(original_id.clone());
        forked_blueprint.version = SemVer::initial();

        // Forks überspringen Novelty-Check (absichtlich ähnlich zum Original)
        let result =
            self.publish_internal(forked_blueprint, creator_trust_r, creator_trust_omega, true)?;

        tracing::info!(
            original_id = %original_id,
            fork_id = %result.blueprint_id,
            "Blueprint forked"
        );

        Ok(result)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Discovery & Suche
    // ─────────────────────────────────────────────────────────────────────────

    /// Hole Blueprint nach ID
    pub fn get_blueprint(&self, id: &BlueprintId) -> Result<Option<Blueprint>> {
        // Cache prüfen
        {
            let cache = self.cache.read();
            if let Some(bp) = cache.get(id) {
                return Ok(Some(bp.clone()));
            }
        }

        // Aus Storage laden
        if let Some(bytes) = self.blueprints.get(&id.0)? {
            let blueprint: Blueprint = serde_json::from_slice(&bytes)?;

            // Cache aktualisieren
            {
                let mut cache = self.cache.write();
                cache.insert(id.clone(), blueprint.clone());
            }

            return Ok(Some(blueprint));
        }

        Ok(None)
    }

    /// Hole Stats für ein Blueprint
    pub fn get_stats(&self, id: &BlueprintId) -> Result<Option<BlueprintStats>> {
        let key = format!("stats:{}", id);
        if let Some(bytes) = self.stats.get(&key)? {
            let stats: BlueprintStats = serde_json::from_slice(&bytes)?;
            return Ok(Some(stats));
        }
        Ok(None)
    }

    /// Suche Blueprints
    pub fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        for entry in self.blueprints.iter() {
            let (_, value) = entry?;
            let blueprint: Blueprint = serde_json::from_slice(&value)?;

            // Filter anwenden
            if let Some(ref category) = query.category {
                if &blueprint.category != category {
                    continue;
                }
            }

            if let Some(min_novelty) = query.min_novelty {
                if blueprint.novelty_score < min_novelty {
                    continue;
                }
            }

            if let Some(ref tag) = query.tag {
                if !blueprint.tags.contains(tag) {
                    continue;
                }
            }

            if let Some(ref text) = query.text {
                let text_lower = text.to_lowercase();
                if !blueprint.name.to_lowercase().contains(&text_lower)
                    && !blueprint.description.to_lowercase().contains(&text_lower)
                {
                    continue;
                }
            }

            // Stats laden
            let stats = self.get_stats(&blueprint.id)?.unwrap_or_default();

            // Ranking berechnen
            let ranking_score = self.compute_ranking_score(&blueprint, &stats);

            results.push(SearchResult {
                blueprint,
                stats,
                ranking_score,
            });
        }

        // Nach Ranking sortieren
        results.sort_by(|a, b| b.ranking_score.partial_cmp(&a.ranking_score).unwrap());

        // Power-Cap anwenden
        results = self.apply_power_cap(results);

        // Limit anwenden
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Berechne Ranking-Score
    fn compute_ranking_score(&self, blueprint: &Blueprint, stats: &BlueprintStats) -> f64 {
        let deployment_score = (stats.deployment_count as f64).ln_1p();
        let rating_score = stats.weighted_rating / 5.0;
        let novelty_score = blueprint.novelty_score / 10.0;
        let diversity_score = blueprint.diversity_contribution * self.config.diversity_boost_factor;

        // 𝔼-Beitrag
        deployment_score * (1.0 + rating_score) * (1.0 + novelty_score) * (1.0 + diversity_score)
    }

    /// Power-Cap anwenden (Axiom Κ19)
    fn apply_power_cap(&self, mut results: Vec<SearchResult>) -> Vec<SearchResult> {
        let mut creator_counts: HashMap<String, u32> = HashMap::new();

        results.retain(|result| {
            let count = creator_counts
                .entry(result.blueprint.creator_did.clone())
                .or_insert(0);
            if *count >= self.config.creator_power_cap {
                false
            } else {
                *count += 1;
                true
            }
        });

        results
    }

    /// Top-Blueprints nach Kategorie
    pub fn top_by_category(
        &self,
        category: BlueprintCategory,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        self.search(SearchQuery {
            category: Some(category),
            limit: Some(limit),
            ..Default::default()
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Bewertung & Feedback
    // ─────────────────────────────────────────────────────────────────────────

    /// Bewerte ein Blueprint
    pub fn rate(
        &self,
        blueprint_id: &BlueprintId,
        rater_did: &DID,
        score: u8,
        comment: Option<String>,
        rater_trust_r: f64,
        rater_trust_omega: f64,
    ) -> Result<RatingResult> {
        // Score validieren
        if score < 1 || score > 5 {
            return Err(anyhow!("Rating score must be 1-5"));
        }

        // Trust-Prüfung
        if rater_trust_r < self.config.min_rating_trust_r {
            return Err(anyhow!(
                "Insufficient Trust-R for rating: {} < {}",
                rater_trust_r,
                self.config.min_rating_trust_r
            ));
        }

        // Blueprint existiert?
        let _blueprint = self
            .get_blueprint(blueprint_id)?
            .ok_or_else(|| anyhow!("Blueprint not found"))?;

        // Anomaly-Check: Doppelte Bewertung?
        let rating_key = format!("rating:{}:{}", blueprint_id, rater_did.to_uri());
        if self.ratings.get(&rating_key)?.is_some() {
            return Err(anyhow!("Already rated this blueprint"));
        }

        // Rate-Limit prüfen (pro Tag)
        let daily_count = self.count_daily_ratings(rater_did)?;
        if daily_count >= self.config.max_ratings_per_day {
            return Err(anyhow!("Daily rating limit reached"));
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let rating = BlueprintRating {
            rater_did: rater_did.to_uri(),
            blueprint_id: blueprint_id.clone(),
            score,
            comment,
            timestamp,
            rater_trust_r,
            rater_trust_omega,
        };

        let mana_cost = rating.mana_cost();
        let weighted_score = rating.weighted_score();

        // Speichern
        let rating_bytes = serde_json::to_vec(&rating)?;
        self.ratings.insert(&rating_key, &rating_bytes)?;

        // Stats aktualisieren
        self.update_rating_stats(blueprint_id, &rating)?;

        tracing::info!(
            blueprint_id = %blueprint_id,
            rater = %rater_did.to_uri(),
            score = score,
            weighted = weighted_score,
            "Blueprint rated"
        );

        Ok(RatingResult {
            mana_cost,
            weighted_score,
        })
    }

    /// Zähle tägliche Bewertungen eines Raters
    fn count_daily_ratings(&self, rater_did: &DID) -> Result<u32> {
        let today_start = {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            now - (now % 86400) // Start des Tages
        };

        let mut count = 0;
        let prefix = format!("rating:");

        for entry in self.ratings.prefix(&prefix) {
            let (key, value) = entry?;
            let key_str = String::from_utf8_lossy(&key);

            if key_str.contains(&rater_did.to_uri()) {
                if let Ok(rating) = serde_json::from_slice::<BlueprintRating>(&value) {
                    if rating.timestamp >= today_start {
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    /// Aktualisiere Stats nach Rating
    fn update_rating_stats(
        &self,
        blueprint_id: &BlueprintId,
        rating: &BlueprintRating,
    ) -> Result<()> {
        let stats_key = format!("stats:{}", blueprint_id);

        let mut stats = self.get_stats(blueprint_id)?.unwrap_or_default();

        // Bayessches Update für gewichtete Bewertung
        let old_total = stats.weighted_rating * stats.rating_count as f64;
        stats.rating_count += 1;
        stats.weighted_rating = (old_total + rating.weighted_score()) / stats.rating_count as f64;

        // Einfacher Durchschnitt
        let old_avg_total = stats.average_rating * (stats.rating_count - 1) as f64;
        stats.average_rating = (old_avg_total + rating.score as f64) / stats.rating_count as f64;

        let stats_bytes = serde_json::to_vec(&stats)?;
        self.stats.insert(&stats_key, &stats_bytes)?;

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Deployment
    // ─────────────────────────────────────────────────────────────────────────

    /// Registriere ein Deployment
    pub fn record_deployment(
        &self,
        blueprint_id: &BlueprintId,
        deployer_did: &DID,
        target_realm: &str,
        mana_paid: u64,
        success: bool,
    ) -> Result<DeploymentResult> {
        // Blueprint existiert?
        let blueprint = self
            .get_blueprint(blueprint_id)?
            .ok_or_else(|| anyhow!("Blueprint not found"))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let deployment_id = format!("{}:{}:{}", blueprint_id, deployer_did.to_uri(), timestamp);

        let deployment = BlueprintDeployment {
            id: deployment_id.clone(),
            blueprint_id: blueprint_id.clone(),
            deployer_did: deployer_did.to_uri(),
            target_realm: target_realm.to_string(),
            timestamp,
            mana_paid,
            success,
        };

        // Speichern
        let deployment_key = format!("deployment:{}", deployment_id);
        let deployment_bytes = serde_json::to_vec(&deployment)?;
        self.deployments
            .insert(&deployment_key, &deployment_bytes)?;

        // Stats aktualisieren
        if success {
            self.update_deployment_stats(blueprint_id, mana_paid)?;
        }

        // Trust-Boost für Creator
        let creator_trust_boost = if success {
            self.config.deployment_trust_boost
        } else {
            0.0
        };

        tracing::info!(
            blueprint_id = %blueprint_id,
            deployer = %deployer_did.to_uri(),
            target = target_realm,
            success = success,
            "Blueprint deployment recorded"
        );

        Ok(DeploymentResult {
            deployment_id,
            mana_cost: blueprint.deployment_mana_cost(),
            creator_trust_boost,
        })
    }

    /// Aktualisiere Stats nach Deployment
    fn update_deployment_stats(&self, blueprint_id: &BlueprintId, mana_paid: u64) -> Result<()> {
        let stats_key = format!("stats:{}", blueprint_id);

        let mut stats = self.get_stats(blueprint_id)?.unwrap_or_default();
        stats.deployment_count += 1;
        stats.total_earnings += mana_paid;
        stats.last_deployment = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // 𝔼-Contribution neu berechnen
        if let Some(blueprint) = self.get_blueprint(blueprint_id)? {
            stats.e_contribution = stats
                .compute_e_contribution(blueprint.novelty_score, blueprint.diversity_contribution);
        }

        let stats_bytes = serde_json::to_vec(&stats)?;
        self.stats.insert(&stats_key, &stats_bytes)?;

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Analytics
    // ─────────────────────────────────────────────────────────────────────────

    /// Hole Marketplace-Statistiken
    pub fn marketplace_stats(&self) -> Result<MarketplaceStats> {
        let mut total_blueprints = 0;
        let mut total_deployments = 0;
        let mut total_ratings = 0;
        let mut total_earnings = 0;
        let mut category_counts: HashMap<String, u64> = HashMap::new();

        for entry in self.blueprints.iter() {
            let (_, value) = entry?;
            if let Ok(blueprint) = serde_json::from_slice::<Blueprint>(&value) {
                total_blueprints += 1;

                let category_key = format!("{:?}", blueprint.category);
                *category_counts.entry(category_key).or_insert(0) += 1;

                if let Some(stats) = self.get_stats(&blueprint.id)? {
                    total_deployments += stats.deployment_count;
                    total_ratings += stats.rating_count;
                    total_earnings += stats.total_earnings;
                }
            }
        }

        Ok(MarketplaceStats {
            total_blueprints,
            total_deployments,
            total_ratings,
            total_earnings,
            blueprints_by_category: category_counts,
        })
    }

    /// Hole Creator-Analytics
    pub fn creator_analytics(&self, creator_did: &DID) -> Result<CreatorAnalytics> {
        let mut blueprints = Vec::new();
        let mut total_deployments = 0;
        let mut total_ratings = 0;
        let mut total_earnings = 0;
        let mut average_rating = 0.0;
        let mut rating_count = 0;

        for entry in self.blueprints.iter() {
            let (_, value) = entry?;
            if let Ok(blueprint) = serde_json::from_slice::<Blueprint>(&value) {
                if blueprint.creator_did == creator_did.to_uri() {
                    blueprints.push(blueprint.id.clone());

                    if let Some(stats) = self.get_stats(&blueprint.id)? {
                        total_deployments += stats.deployment_count;
                        total_ratings += stats.rating_count;
                        total_earnings += stats.total_earnings;

                        if stats.rating_count > 0 {
                            average_rating += stats.average_rating * stats.rating_count as f64;
                            rating_count += stats.rating_count;
                        }
                    }
                }
            }
        }

        if rating_count > 0 {
            average_rating /= rating_count as f64;
        }

        Ok(CreatorAnalytics {
            creator_did: creator_did.to_uri(),
            blueprint_count: blueprints.len() as u64,
            blueprint_ids: blueprints,
            total_deployments,
            total_ratings,
            total_earnings,
            average_rating,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Ergebnis-Typen
// ═══════════════════════════════════════════════════════════════════════════

/// Ergebnis einer Publikation
#[derive(Debug, Clone)]
pub struct PublishResult {
    pub blueprint_id: BlueprintId,
    pub mana_cost: u64,
    pub novelty_score: f64,
    pub diversity_contribution: f64,
}

/// Suchanfrage
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub category: Option<BlueprintCategory>,
    pub tag: Option<String>,
    pub min_novelty: Option<f64>,
    pub min_rating: Option<f64>,
    pub creator_did: Option<String>,
    pub limit: Option<usize>,
}

/// Suchergebnis
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub blueprint: Blueprint,
    pub stats: BlueprintStats,
    pub ranking_score: f64,
}

/// Ergebnis einer Bewertung
#[derive(Debug, Clone)]
pub struct RatingResult {
    pub mana_cost: u64,
    pub weighted_score: f64,
}

/// Ergebnis eines Deployments
#[derive(Debug, Clone)]
pub struct DeploymentResult {
    pub deployment_id: String,
    pub mana_cost: u64,
    pub creator_trust_boost: f64,
}

/// Marketplace-Gesamtstatistiken
#[derive(Debug, Clone)]
pub struct MarketplaceStats {
    pub total_blueprints: u64,
    pub total_deployments: u64,
    pub total_ratings: u64,
    pub total_earnings: u64,
    pub blueprints_by_category: HashMap<String, u64>,
}

/// Creator-Analytics
#[derive(Debug, Clone)]
pub struct CreatorAnalytics {
    pub creator_did: String,
    pub blueprint_count: u64,
    pub blueprint_ids: Vec<BlueprintId>,
    pub total_deployments: u64,
    pub total_ratings: u64,
    pub total_earnings: u64,
    pub average_rating: f64,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local::test_utils::test_keyspace;
    use crate::local::{SchemaFieldType, StoreSchema};

    fn setup() -> (tempfile::TempDir, BlueprintMarketplace) {
        let (dir, keyspace) = test_keyspace();
        let marketplace = BlueprintMarketplace::new(&keyspace, MarketplaceConfig::default())
            .expect("Failed to create marketplace");
        (dir, marketplace)
    }

    /// Setup mit niedrigerem Novelty-Threshold für Tests mit mehreren Blueprints
    fn setup_low_novelty() -> (tempfile::TempDir, BlueprintMarketplace) {
        let (dir, keyspace) = test_keyspace();
        let config = MarketplaceConfig {
            min_novelty_score: 0.5, // Niedrigerer Threshold für Tests
            ..MarketplaceConfig::default()
        };
        let marketplace =
            BlueprintMarketplace::new(&keyspace, config).expect("Failed to create marketplace");
        (dir, marketplace)
    }

    fn create_test_blueprint(name: &str, creator: &str) -> Blueprint {
        let schema = StoreSchema::new("posts", false)
            .with_field("title", SchemaFieldType::String)
            .with_field("content", SchemaFieldType::String);

        Blueprint::builder(name, creator)
            .description("A test blueprint")
            .category(BlueprintCategory::Social)
            .tag("test")
            .store(BlueprintStore {
                name: "posts".to_string(),
                schema,
                personal: false,
                description: Some("Posts store".to_string()),
                initial_data: None,
            })
            .build()
    }

    /// Erstellt verschiedene Blueprints für unterschiedliche Kategorien.
    /// Jeder Blueprint hat unterschiedliche Stores, um den Novelty-Check zu bestehen.
    fn create_diverse_blueprint(
        index: usize,
        creator: &str,
        category: BlueprintCategory,
    ) -> Blueprint {
        let store_name = format!("store_{}", index);
        let schema = StoreSchema::new(&store_name, false)
            .with_field(&format!("field_a_{}", index), SchemaFieldType::String)
            .with_field(&format!("field_b_{}", index), SchemaFieldType::Number)
            .with_field(&format!("unique_{}", index), SchemaFieldType::Bool);

        let name = format!("Blueprint {} - {:?}", index, category);

        Blueprint::builder(&name, creator)
            .description(&format!(
                "Diverse blueprint #{} for category {:?}",
                index, category
            ))
            .category(category)
            .tag(&format!("tag_{}", index))
            .tag(&format!("unique_{}", index * 7))
            .store(BlueprintStore {
                name: store_name,
                schema,
                personal: index % 2 == 0,
                description: Some(format!("Store for blueprint {}", index)),
                initial_data: None,
            })
            .build()
    }

    #[test]
    fn test_blueprint_creation() {
        let blueprint = create_test_blueprint("Test Blueprint", "did:erynoa:self:alice");

        assert_eq!(blueprint.name, "Test Blueprint");
        assert!(!blueprint.id.0.is_empty());
        assert!(blueprint.complexity > 0);
    }

    #[test]
    fn test_semver() {
        let v1 = SemVer::initial();
        assert_eq!(v1.to_string(), "1.0.0");

        let v2 = v1.bump_minor();
        assert_eq!(v2.to_string(), "1.1.0");

        let v3 = v2.bump_major();
        assert_eq!(v3.to_string(), "2.0.0");
    }

    #[test]
    fn test_novelty_calculator() {
        let mut calc = NoveltyCalculator::new();

        let bp1 = create_test_blueprint("Blueprint 1", "did:erynoa:self:alice");
        let novelty1 = calc.compute_novelty(&bp1);
        assert!(novelty1 > 5.0); // Hohe Novelty für erstes Blueprint

        calc.register_blueprint(&bp1);

        // Ähnliches Blueprint hat niedrigere Novelty
        let bp2 = create_test_blueprint("Blueprint 2", "did:erynoa:self:bob");
        let novelty2 = calc.compute_novelty(&bp2);
        assert!(novelty2 < novelty1);
    }

    #[test]
    fn test_publish_blueprint() {
        let (_dir, marketplace) = setup();

        let blueprint = create_test_blueprint("Social Starter", "did:erynoa:self:alice");

        let result = marketplace.publish(blueprint, 0.9, 2.0).unwrap();

        assert!(!result.blueprint_id.0.is_empty());
        assert!(result.novelty_score > 0.0);
        assert!(result.mana_cost > 0);

        // Kann abgerufen werden
        let loaded = marketplace.get_blueprint(&result.blueprint_id).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().name, "Social Starter");
    }

    #[test]
    fn test_publish_insufficient_trust() {
        let (_dir, marketplace) = setup();

        let blueprint = create_test_blueprint("Test", "did:erynoa:self:alice");

        // Zu niedriger Trust-R
        let result = marketplace.publish(blueprint.clone(), 0.5, 2.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Trust-R"));

        // Zu niedriger Trust-Ω
        let result = marketplace.publish(blueprint, 0.9, 1.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Trust-Ω"));
    }

    #[test]
    fn test_rate_blueprint() {
        let (_dir, marketplace) = setup();

        let blueprint = create_test_blueprint("Rated Blueprint", "did:erynoa:self:alice");
        let publish_result = marketplace.publish(blueprint, 0.9, 2.0).unwrap();

        let rater = DID::new_self("bob");
        let rating_result = marketplace
            .rate(
                &publish_result.blueprint_id,
                &rater,
                5,
                Some("Great blueprint!".to_string()),
                0.5,
                1.5,
            )
            .unwrap();

        assert!(rating_result.mana_cost > 0);
        assert!(rating_result.weighted_score > 0.0);

        // Stats wurden aktualisiert
        let stats = marketplace
            .get_stats(&publish_result.blueprint_id)
            .unwrap()
            .unwrap();
        assert_eq!(stats.rating_count, 1);
        assert!(stats.average_rating > 0.0);
    }

    #[test]
    fn test_rate_duplicate() {
        let (_dir, marketplace) = setup();

        let blueprint = create_test_blueprint("Once Rated", "did:erynoa:self:alice");
        let publish_result = marketplace.publish(blueprint, 0.9, 2.0).unwrap();

        let rater = DID::new_self("bob");
        marketplace
            .rate(&publish_result.blueprint_id, &rater, 5, None, 0.5, 1.5)
            .unwrap();

        // Zweite Bewertung sollte fehlschlagen
        let result = marketplace.rate(&publish_result.blueprint_id, &rater, 4, None, 0.5, 1.5);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Already rated"));
    }

    #[test]
    fn test_record_deployment() {
        let (_dir, marketplace) = setup();

        let blueprint = create_test_blueprint("Deployable", "did:erynoa:self:alice");
        let publish_result = marketplace.publish(blueprint, 0.9, 2.0).unwrap();

        let deployer = DID::new_self("charlie");
        let deploy_result = marketplace
            .record_deployment(
                &publish_result.blueprint_id,
                &deployer,
                "my-realm",
                100,
                true,
            )
            .unwrap();

        assert!(!deploy_result.deployment_id.is_empty());
        assert!(deploy_result.creator_trust_boost > 0.0);

        // Stats wurden aktualisiert
        let stats = marketplace
            .get_stats(&publish_result.blueprint_id)
            .unwrap()
            .unwrap();
        assert_eq!(stats.deployment_count, 1);
        assert_eq!(stats.total_earnings, 100);
    }

    #[test]
    fn test_search_blueprints() {
        let (_dir, marketplace) = setup_low_novelty();

        // Mehrere diverse Blueprints publizieren
        let bp1 = Blueprint::builder("Social Pack", "did:erynoa:self:alice")
            .category(BlueprintCategory::Social)
            .tag("social")
            .store(BlueprintStore {
                name: "posts_social".to_string(),
                schema: StoreSchema::new("posts_social", false)
                    .with_field("title", SchemaFieldType::String)
                    .with_field("author", SchemaFieldType::String)
                    .with_field("likes", SchemaFieldType::Number),
                personal: false,
                description: Some("Social media posts".to_string()),
                initial_data: None,
            })
            .build();

        let bp2 = Blueprint::builder("Governance Kit", "did:erynoa:self:bob")
            .category(BlueprintCategory::Governance)
            .tag("governance")
            .store(BlueprintStore {
                name: "proposals_gov".to_string(),
                schema: StoreSchema::new("proposals_gov", false)
                    .with_field("proposal_id", SchemaFieldType::Number)
                    .with_field("description", SchemaFieldType::String)
                    .with_field("votes", SchemaFieldType::Number)
                    .with_field("deadline", SchemaFieldType::Timestamp),
                personal: false,
                description: Some("Governance proposals".to_string()),
                initial_data: None,
            })
            .build();

        marketplace.publish(bp1, 0.9, 2.0).unwrap();
        marketplace.publish(bp2, 0.9, 2.0).unwrap();

        // Suche nach Kategorie
        let results = marketplace
            .search(SearchQuery {
                category: Some(BlueprintCategory::Social),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].blueprint.name, "Social Pack");

        // Suche nach Text
        let results = marketplace
            .search(SearchQuery {
                text: Some("Governance".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].blueprint.name, "Governance Kit");
    }

    #[test]
    fn test_fork_blueprint() {
        let (_dir, marketplace) = setup();

        let original = create_test_blueprint("Original", "did:erynoa:self:alice");
        let original_result = marketplace.publish(original, 0.9, 2.0).unwrap();

        let mut forked = create_test_blueprint("Forked Version", "did:erynoa:self:bob");
        forked.description = "An improved version".to_string();

        let fork_result = marketplace
            .fork(&original_result.blueprint_id, forked, 0.9, 2.0)
            .unwrap();

        // Fork hat Referenz zum Original
        let loaded = marketplace
            .get_blueprint(&fork_result.blueprint_id)
            .unwrap()
            .unwrap();
        assert_eq!(loaded.forked_from, Some(original_result.blueprint_id));
        assert_eq!(loaded.version, SemVer::initial());
    }

    #[test]
    fn test_new_version() {
        let (_dir, marketplace) = setup();

        let original = create_test_blueprint("Versioned", "did:erynoa:self:alice");
        let original_result = marketplace.publish(original, 0.9, 2.0).unwrap();

        let mut updated = create_test_blueprint("Versioned", "did:erynoa:self:alice");
        updated.description = "Updated description".to_string();

        let version_result = marketplace
            .publish_new_version(&original_result.blueprint_id, updated, 0.9, 2.0)
            .unwrap();

        let loaded = marketplace
            .get_blueprint(&version_result.blueprint_id)
            .unwrap()
            .unwrap();
        assert_eq!(loaded.predecessor, Some(original_result.blueprint_id));
        assert_eq!(loaded.version, SemVer::new(1, 1, 0));
    }

    #[test]
    fn test_marketplace_stats() {
        let (_dir, marketplace) = setup();

        let bp = create_test_blueprint("Test", "did:erynoa:self:alice");
        marketplace.publish(bp, 0.9, 2.0).unwrap();

        let stats = marketplace.marketplace_stats().unwrap();
        assert_eq!(stats.total_blueprints, 1);
    }

    #[test]
    fn test_creator_analytics() {
        let (_dir, marketplace) = setup();

        let alice = DID::new_self("alice");
        let bp = create_test_blueprint("Alice's Blueprint", &alice.to_uri());
        marketplace.publish(bp, 0.9, 2.0).unwrap();

        let analytics = marketplace.creator_analytics(&alice).unwrap();
        assert_eq!(analytics.blueprint_count, 1);
    }

    #[test]
    fn test_power_cap() {
        let (_dir, marketplace) = setup_low_novelty();

        // Viele diverse Blueprints von einem Creator
        // Jeder Blueprint hat unterschiedliche Stores, Felder und Kategorien
        let categories = [
            BlueprintCategory::Social,
            BlueprintCategory::Governance,
            BlueprintCategory::Commerce,
            BlueprintCategory::Content,
            BlueprintCategory::Gaming,
            BlueprintCategory::Identity,
            BlueprintCategory::Infrastructure,
            BlueprintCategory::Social,
            BlueprintCategory::Governance,
            BlueprintCategory::Commerce,
            BlueprintCategory::Content,
            BlueprintCategory::Gaming,
            BlueprintCategory::Identity,
            BlueprintCategory::Infrastructure,
            BlueprintCategory::Social,
        ];

        for i in 0..15 {
            let bp = create_diverse_blueprint(i, "did:erynoa:self:alice", categories[i].clone());
            marketplace.publish(bp, 0.9, 2.0).unwrap();
        }

        // Suche sollte Power-Cap anwenden
        let results = marketplace.search(SearchQuery::default()).unwrap();

        // Maximal 10 vom selben Creator
        let alice_count = results
            .iter()
            .filter(|r| r.blueprint.creator_did == "did:erynoa:self:alice")
            .count();
        assert!(alice_count <= 10);
    }
}
