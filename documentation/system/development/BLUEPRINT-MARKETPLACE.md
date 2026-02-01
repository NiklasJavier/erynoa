# Blueprint Marketplace - Implementierungsdokumentation

> **Modul:** `backend/src/local/blueprint_marketplace.rs`
> **Version:** 1.0.0
> **Stand:** Juni 2025
> **Umfang:** ~1.950 Zeilen Rust-Code, 20 Tests

## Übersicht

Der **Blueprint Marketplace** ist ein dezentraler, trust-basierter Marktplatz für wiederverwendbare Realm-Templates in Erynoa. Er ermöglicht das Teilen, Entdecken und Deployen von vorgefertigten Blueprints, die Stores, Policies und Sagas enthalten.

### Kernfeatures

- **Content-Addressable IDs**: BLAKE3-Hash garantiert Integrität
- **Trust-gated Uploads**: Nur vertrauenswürdige Akteure können publizieren
- **Novelty-Scoring**: Surprisal-basierte Bewertung der Einzigartigkeit
- **Ω-gewichtete Ratings**: Alignierte Bewerter haben mehr Einfluss
- **Power-Cap**: Verhindert Creator-Dominanz (Axiom Κ19)
- **Diversity-Boost**: Innovative Blueprints steigen automatisch

---

## Architektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     BLUEPRINT MARKETPLACE                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   Blueprint = Immutables, versioniertes, content-addressables Template      │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │ BLAKE3-Hash als ID → Garantierte Integrität                         │  │
│   │ Inhalt: Stores + Schemas + Policies + Sagas + Metadaten             │  │
│   │ Novelty-Score + Diversity-Contribution (automatisch berechnet)       │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│   Marketplace als dedizierter Realm:                                       │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │ marketplace_blueprints   → Listing (Hash → Metadaten)               │  │
│   │ marketplace_ratings      → Attestation-Events (bayessche Updates)   │  │
│   │ marketplace_deployments  → Statistiken + Contributor-Boost          │  │
│   │ marketplace_stats        → Aggregierte Blueprint-Statistiken        │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│   Intelligentes Ranking:                                                   │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │ 𝔼-Beitrag = Deployments × Ratings × Diversity × (1 - Calcification) │  │
│   │ Novelty-Score = Surprisal vs. existierende Blueprints               │  │
│   │ Trust-Gewichtung = Ω-Alignment der Bewerter                         │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│   Gaming-Resistenz (Axiome Κ19/Κ20):                                       │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │ Upload: Trust-R > 0.8, Ω > 1.5, Novelty > 3.0                       │  │
│   │ Ratings: Ω-Gewichtung, Anomaly-Detection, bayessche Updates         │  │
│   │ Power-Cap: Kein Creator dominiert Listings                          │  │
│   │ Diversity-Boost: Innovative Blueprints steigen automatisch auf      │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Komponentendiagramm

```
                    ┌──────────────────────┐
                    │  BlueprintMarketplace │
                    └──────────┬───────────┘
                               │
        ┌──────────────────────┼──────────────────────┐
        │                      │                      │
        ▼                      ▼                      ▼
┌───────────────┐    ┌─────────────────┐    ┌──────────────┐
│   Blueprint   │    │ NoveltyCalculator│    │ MarketplaceConfig│
│   Builder     │    │   (Surprisal)   │    │   (Thresholds)│
└───────────────┘    └─────────────────┘    └──────────────┘
        │                      │
        ▼                      ▼
┌───────────────────────────────────────────────────────────┐
│                     Fjall Storage                          │
├───────────────┬───────────────┬────────────┬──────────────┤
│  blueprints   │    stats      │   ratings  │ deployments  │
└───────────────┴───────────────┴────────────┴──────────────┘
```

---

## Datenmodell

### BlueprintId

Content-addressabler Identifier basierend auf BLAKE3-Hash.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlueprintId(pub String);

impl BlueprintId {
    /// Erstelle ID aus Hash
    pub fn new(hash: impl Into<String>) -> Self;

    /// Berechne ID aus Blueprint-Inhalt
    pub fn from_content(content: &[u8]) -> Self;
}
```

**Eigenschaften:**

- Deterministisch: Gleicher Inhalt → Gleiche ID
- Immutabel: Änderungen erzeugen neue ID
- Verifizierbar: Integrität prüfbar durch Neuberechnung

### SemVer

Semantische Versionierung für Blueprints.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}
```

**Methoden:**

| Methode        | Beschreibung          | Beispiel          |
| -------------- | --------------------- | ----------------- |
| `initial()`    | Erstellt v1.0.0       | `1.0.0`           |
| `bump_major()` | Major-Version erhöhen | `1.0.0` → `2.0.0` |
| `bump_minor()` | Minor-Version erhöhen | `1.0.0` → `1.1.0` |
| `bump_patch()` | Patch-Version erhöhen | `1.0.0` → `1.0.1` |

### BlueprintLicense

Lizenztypen für Blueprints.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
```

**Lizenzmatrix:**

| Lizenz          | Deployment | Kommerzielle Nutzung | Mana-Gebühr | Einschränkungen       |
| --------------- | ---------- | -------------------- | ----------- | --------------------- |
| `Open`          | ✅         | ✅                   | -           | Keine                 |
| `Attribution`   | ✅         | ✅                   | -           | Credit erforderlich   |
| `NonCommercial` | ✅         | ❌                   | -           | Nur nicht-kommerziell |
| `Commercial`    | ✅         | ✅                   | Variabel    | Mana-Fee an Creator   |
| `Restricted`    | Begrenzt   | ✅                   | -           | Nur erlaubte Realms   |

### BlueprintCategory

Kategorien für Discovery und Filterung.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
```

---

## Blueprint-Inhalte

### BlueprintStore

Store-Definition innerhalb eines Blueprints.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintStore {
    /// Store-Name
    pub name: String,

    /// Schema-Definition (StoreSchema aus realm_storage)
    pub schema: StoreSchema,

    /// Personal-Store (pro DID) oder Shared
    pub personal: bool,

    /// Optionale Beschreibung
    pub description: Option<String>,

    /// Initialdaten (z.B. für Config-Stores)
    pub initial_data: Option<HashMap<String, StoreValue>>,
}
```

### BlueprintPolicy

ECL-Policy-Definition.

```rust
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
```

### BlueprintSaga

Multi-Step-Workflow-Definition.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintSaga {
    /// Saga-Name
    pub name: String,

    /// Saga-Schritte
    pub steps: Vec<SagaStep>,

    /// Beschreibung
    pub description: Option<String>,

    /// Geschätzte Mana-Kosten
    pub estimated_mana: u64,
}

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
```

**SagaAction-Varianten:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SagaAction {
    /// Store erstellen
    CreateStore { store: BlueprintStore },

    /// Daten schreiben
    PutData { store: String, key: String, value: StoreValue },

    /// Policy installieren
    InstallPolicy { policy: BlueprintPolicy },

    /// Trust prüfen
    CheckTrust { min_r: f64, min_omega: f64 },

    /// Mana reservieren
    ReserveMana { amount: u64 },

    /// Benutzerdefiniert (ECL-Aufruf)
    Custom { ecl_code: String },
}
```

---

## Haupt-Blueprint-Struktur

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    // ═══════════════════════════════════════════════════════════════════════
    // Identifier
    // ═══════════════════════════════════════════════════════════════════════

    /// Einzigartige ID (BLAKE3-Hash des Inhalts)
    pub id: BlueprintId,

    // ═══════════════════════════════════════════════════════════════════════
    // Metadaten
    // ═══════════════════════════════════════════════════════════════════════

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

    // ═══════════════════════════════════════════════════════════════════════
    // Inhalt
    // ═══════════════════════════════════════════════════════════════════════

    /// Store-Definitionen
    pub stores: Vec<BlueprintStore>,

    /// Policy-Definitionen
    pub policies: Vec<BlueprintPolicy>,

    /// Saga-Definitionen
    pub sagas: Vec<BlueprintSaga>,

    // ═══════════════════════════════════════════════════════════════════════
    // Versionierung
    // ═══════════════════════════════════════════════════════════════════════

    /// Vorgänger-Blueprint (für Versioning)
    pub predecessor: Option<BlueprintId>,

    /// Original-Blueprint (für Forks)
    pub forked_from: Option<BlueprintId>,

    // ═══════════════════════════════════════════════════════════════════════
    // Automatisch berechnete Metriken
    // ═══════════════════════════════════════════════════════════════════════

    /// Komplexität (automatisch berechnet)
    pub complexity: u64,

    /// Novelty-Score (Surprisal vs. existierende Blueprints)
    pub novelty_score: f64,

    /// Diversity-Contribution (neue Konzepte)
    pub diversity_contribution: f64,
}
```

### Blueprint-Methoden

| Methode                  | Beschreibung                      |
| ------------------------ | --------------------------------- |
| `builder()`              | Erstellt einen `BlueprintBuilder` |
| `compute_id()`           | Berechnet BLAKE3-Hash             |
| `compute_complexity()`   | Berechnet Komplexitätsscore       |
| `upload_mana_cost()`     | Berechnet Upload-Mana-Kosten      |
| `deployment_mana_cost()` | Berechnet Deployment-Mana-Kosten  |
| `validate()`             | Validiert Blueprint-Struktur      |

---

## BlueprintBuilder

Fluent-Builder-Pattern für Blueprint-Erstellung.

```rust
// Beispiel: Social Blueprint erstellen
let blueprint = Blueprint::builder("Social Starter", "did:erynoa:self:alice")
    .description("Ein komplettes Social-Network-Template")
    .category(BlueprintCategory::Social)
    .tag("social")
    .tag("posts")
    .tag("profiles")
    .license(BlueprintLicense::Attribution)
    .store(BlueprintStore {
        name: "posts".to_string(),
        schema: StoreSchema::new("posts", false)
            .with_field("title", SchemaFieldType::String)
            .with_field("content", SchemaFieldType::String)
            .with_field("author", SchemaFieldType::String)
            .with_field("created_at", SchemaFieldType::Timestamp),
        personal: false,
        description: Some("Öffentliche Posts".to_string()),
        initial_data: None,
    })
    .policy(BlueprintPolicy {
        name: "post_access".to_string(),
        ecl_code: "allow if trust_r >= 0.5".to_string(),
        policy_type: PolicyType::StoreAccess,
        description: Some("Mindest-Trust für Posting".to_string()),
    })
    .build();
```

### Builder-Methoden

| Methode         | Parameter         | Beschreibung                    |
| --------------- | ----------------- | ------------------------------- |
| `new()`         | name, creator_did | Erstellt neuen Builder          |
| `description()` | text              | Setzt Beschreibung              |
| `version()`     | SemVer            | Setzt Version (Standard: 1.0.0) |
| `tag()`         | text              | Fügt Tag hinzu                  |
| `category()`    | BlueprintCategory | Setzt Kategorie                 |
| `license()`     | BlueprintLicense  | Setzt Lizenz                    |
| `store()`       | BlueprintStore    | Fügt Store hinzu                |
| `policy()`      | BlueprintPolicy   | Fügt Policy hinzu               |
| `saga()`        | BlueprintSaga     | Fügt Saga hinzu                 |
| `predecessor()` | BlueprintId       | Setzt Vorgänger (Versioning)    |
| `forked_from()` | BlueprintId       | Setzt Fork-Quelle               |
| `build()`       | -                 | Erstellt finales Blueprint      |

---

## Marketplace-Metriken

### BlueprintStats

Statistiken für ein publiziertes Blueprint.

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlueprintStats {
    /// Anzahl erfolgreicher Deployments
    pub deployment_count: u64,

    /// Durchschnittliche Bewertung (0-5)
    pub average_rating: f64,

    /// Anzahl abgegebener Bewertungen
    pub rating_count: u64,

    /// Trust-gewichtete Bewertung (Ω-adjustiert)
    pub weighted_rating: f64,

    /// 𝔼-Beitrag (World-Formula Contribution)
    pub e_contribution: f64,

    /// Letztes Deployment (Unix Timestamp)
    pub last_deployment: u64,

    /// Gesamteinnahmen (Mana) für Creator
    pub total_earnings: u64,
}
```

**𝔼-Contribution-Formel:**

$$
\mathbb{E}_{\text{contribution}} = \ln(1 + \text{deployments}) \times \frac{\text{weighted\_rating}}{5} \times \frac{\text{novelty}}{10} \times \text{diversity}
$$

### BlueprintRating

Einzelne Bewertung mit Trust-Gewichtung.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintRating {
    /// Bewerter-DID
    pub rater_did: String,

    /// Blueprint-ID
    pub blueprint_id: BlueprintId,

    /// Bewertung (1-5)
    pub score: u8,

    /// Optionaler Kommentar
    pub comment: Option<String>,

    /// Zeitstempel
    pub timestamp: u64,

    /// Trust-R des Bewerters
    pub rater_trust_r: f64,

    /// Trust-Ω des Bewerters
    pub rater_trust_omega: f64,
}
```

**Gewichtete Bewertung:**

$$
\text{weighted\_score} = \text{score} \times \frac{\min(\Omega, 3.0)}{3.0} \times R
$$

### BlueprintDeployment

Deployment-Aufzeichnung.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintDeployment {
    /// Einzigartige Deployment-ID
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

    /// Erfolgreicht?
    pub success: bool,
}
```

---

## Marketplace-Konfiguration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    // ═══════════════════════════════════════════════════════════════════════
    // Upload-Anforderungen
    // ═══════════════════════════════════════════════════════════════════════

    /// Minimaler Trust-R für Upload
    pub min_upload_trust_r: f64,           // Default: 0.8

    /// Minimaler Trust-Ω für Upload
    pub min_upload_trust_omega: f64,        // Default: 1.5

    /// Minimaler Novelty-Score für Upload
    pub min_novelty_score: f64,             // Default: 3.0

    // ═══════════════════════════════════════════════════════════════════════
    // Rating-Anforderungen
    // ═══════════════════════════════════════════════════════════════════════

    /// Minimaler Trust-R für Rating
    pub min_rating_trust_r: f64,            // Default: 0.3

    /// Maximale Ratings pro Tag pro DID
    pub max_ratings_per_day: u32,           // Default: 10

    /// Anomaly-Detection: Max koordinierte Ratings
    pub max_coordinated_ratings: u32,       // Default: 5

    // ═══════════════════════════════════════════════════════════════════════
    // Ranking-Parameter
    // ═══════════════════════════════════════════════════════════════════════

    /// Power-Cap: Max Blueprints pro Creator in Listings
    pub creator_power_cap: u32,             // Default: 10

    /// Diversity-Boost-Faktor
    pub diversity_boost_factor: f64,        // Default: 1.5

    /// Creator-Trust-Boost pro erfolgreichem Deployment
    pub deployment_trust_boost: f64,        // Default: 0.001
}
```

### Gaming-Resistenz-Thresholds

| Parameter                | Default | Beschreibung             | Axiom |
| ------------------------ | ------- | ------------------------ | ----- |
| `min_upload_trust_r`     | 0.8     | Hohe Vertrauensschwelle  | Κ19   |
| `min_upload_trust_omega` | 1.5     | Alignierung erforderlich | Κ20   |
| `min_novelty_score`      | 3.0     | Spam-Prävention          | Κ19   |
| `creator_power_cap`      | 10      | Dominanz-Prävention      | Κ19   |

---

## Novelty-Berechnung

Der `NoveltyCalculator` bewertet die Einzigartigkeit eines Blueprints mittels Information-Theoretic Surprisal.

```rust
pub struct NoveltyCalculator {
    /// Bekannte Konzepte (Store-Namen, Policy-Typen, etc.)
    known_concepts: HashSet<String>,

    /// Konzept-Häufigkeiten
    concept_frequencies: HashMap<String, u64>,

    /// Gesamt-Blueprints
    total_blueprints: u64,
}
```

### Konzept-Extraktion

Aus einem Blueprint werden folgende Konzepte extrahiert:

```rust
fn extract_concepts(&self, blueprint: &Blueprint) -> Vec<String> {
    // Store-Namen: "store:posts"
    // Schema-Felder: "field:title", "field:content"
    // Policy-Typen: "policy:StoreAccess"
    // Tags: "tag:social"
    // Kategorie: "category:Social"
}
```

### Surprisal-Formel

$$
\text{Novelty} = \frac{1}{|C|} \sum_{c \in C} -\log_2 P(c)
$$

Wobei:

- $C$ = Menge aller Konzepte im Blueprint
- $P(c)$ = Frequenz des Konzepts / Gesamt-Blueprints
- Laplace-Smoothing für unbekannte Konzepte

### Diversity-Contribution

$$
\text{Diversity} = \frac{|\text{neue Konzepte}|}{|\text{alle Konzepte}|}
$$

---

## Mana-Kosten

### Upload-Kosten

$$
\text{Mana}_{\text{upload}} = 500 + \text{complexity} \times 20 + \frac{100}{\text{novelty}}
$$

| Komponente       | Basis | Faktor      |
| ---------------- | ----- | ----------- |
| Grundkosten      | 500   | -           |
| Komplexität      | -     | ×20         |
| Novelty-Discount | -     | 100/novelty |

### Rating-Kosten

$$
\text{Mana}_{\text{rate}} = \max(5, 10 - R \times 20)
$$

Vertrauenswürdige Rater zahlen weniger.

### Deployment-Kosten

$$
\text{Mana}_{\text{deploy}} = 50 + \text{complexity} \times 5 + \text{license\_fee}
$$

### Fork-Kosten

$$
\text{Mana}_{\text{fork}} = 200 + \text{original\_complexity} \times 10
$$

---

## BlueprintMarketplace API

### Struktur

```rust
pub struct BlueprintMarketplace {
    /// Blueprints-Partition (Fjall)
    blueprints: PartitionHandle,

    /// Stats-Partition
    stats: PartitionHandle,

    /// Ratings-Partition
    ratings: PartitionHandle,

    /// Deployments-Partition
    deployments: PartitionHandle,

    /// Konfiguration
    config: MarketplaceConfig,

    /// Novelty-Calculator (cached, thread-safe)
    novelty: Arc<RwLock<NoveltyCalculator>>,

    /// Blueprint-Cache
    cache: Arc<RwLock<HashMap<BlueprintId, Blueprint>>>,
}
```

### Konstruktor

```rust
pub fn new(keyspace: &Keyspace, config: MarketplaceConfig) -> Result<Self>;
```

Erstellt neue Marketplace-Instanz und initialisiert:

1. Vier Fjall-Partitionen für Persistenz
2. NoveltyCalculator mit existierenden Blueprints
3. Leeren Blueprint-Cache

---

## API-Methoden

### Publishing

#### `publish()`

```rust
pub fn publish(
    &self,
    blueprint: Blueprint,
    creator_trust_r: f64,
    creator_trust_omega: f64,
) -> Result<PublishResult>;
```

**Ablauf:**

1. Trust-Prüfung (R ≥ 0.8, Ω ≥ 1.5)
2. Blueprint-Validierung
3. Novelty-Berechnung
4. Novelty-Prüfung (≥ 3.0)
5. ID-Berechnung (BLAKE3)
6. Persistierung
7. Stats initialisieren
8. NoveltyCalculator aktualisieren
9. Cache aktualisieren

**Rückgabe:**

```rust
pub struct PublishResult {
    pub blueprint_id: BlueprintId,
    pub mana_cost: u64,
    pub novelty_score: f64,
    pub diversity_contribution: f64,
}
```

#### `publish_new_version()`

```rust
pub fn publish_new_version(
    &self,
    predecessor_id: &BlueprintId,
    updated_blueprint: Blueprint,
    creator_trust_r: f64,
    creator_trust_omega: f64,
) -> Result<PublishResult>;
```

Erstellt neue Version eines existierenden Blueprints:

- Nur Original-Creator erlaubt
- Novelty-Check wird übersprungen (absichtlich ähnlich)
- Version wird automatisch gebumpt (Minor)
- Predecessor-Referenz wird gesetzt

#### `fork()`

```rust
pub fn fork(
    &self,
    original_id: &BlueprintId,
    forked_blueprint: Blueprint,
    creator_trust_r: f64,
    creator_trust_omega: f64,
) -> Result<PublishResult>;
```

Erstellt Fork eines Blueprints:

- Neuer Creator erlaubt
- Novelty-Check wird übersprungen
- Fork-Referenz zum Original gesetzt
- Version startet bei 1.0.0

---

### Discovery

#### `get_blueprint()`

```rust
pub fn get_blueprint(&self, id: &BlueprintId) -> Result<Option<Blueprint>>;
```

Lädt Blueprint mit Caching:

1. Cache prüfen
2. Aus Storage laden
3. Cache aktualisieren

#### `get_stats()`

```rust
pub fn get_stats(&self, id: &BlueprintId) -> Result<Option<BlueprintStats>>;
```

Lädt Statistiken für ein Blueprint.

#### `search()`

```rust
pub fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>>;
```

**SearchQuery:**

```rust
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub text: Option<String>,              // Volltext-Suche
    pub category: Option<BlueprintCategory>, // Kategorie-Filter
    pub tag: Option<String>,               // Tag-Filter
    pub min_novelty: Option<f64>,          // Minimum-Novelty
    pub min_rating: Option<f64>,           // Minimum-Rating
    pub creator_did: Option<String>,       // Creator-Filter
    pub limit: Option<usize>,              // Ergebnis-Limit
}
```

**SearchResult:**

```rust
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub blueprint: Blueprint,
    pub stats: BlueprintStats,
    pub ranking_score: f64,
}
```

**Ranking-Algorithmus:**

$$
\text{score} = \ln(1 + \text{deployments}) \times (1 + \frac{\text{rating}}{5}) \times (1 + \frac{\text{novelty}}{10}) \times (1 + \text{diversity} \times \text{boost})
$$

**Power-Cap (Axiom Κ19):**

- Maximal `creator_power_cap` (Standard: 10) Blueprints pro Creator
- Verhindert Dominanz einzelner Akteure

#### `top_by_category()`

```rust
pub fn top_by_category(
    &self,
    category: BlueprintCategory,
    limit: usize,
) -> Result<Vec<SearchResult>>;
```

Convenience-Methode für Kategorie-Top-Listen.

---

### Rating

#### `rate()`

```rust
pub fn rate(
    &self,
    blueprint_id: &BlueprintId,
    rater_did: &DID,
    score: u8,           // 1-5
    comment: Option<String>,
    rater_trust_r: f64,
    rater_trust_omega: f64,
) -> Result<RatingResult>;
```

**Validierungen:**

1. Score 1-5
2. Trust-R ≥ 0.3
3. Blueprint existiert
4. Nicht bereits bewertet (Doppel-Rating-Schutz)
5. Tägliches Rate-Limit nicht überschritten

**Rückgabe:**

```rust
pub struct RatingResult {
    pub mana_cost: u64,
    pub weighted_score: f64,
}
```

**Stats-Update (Bayessches Update):**

```rust
let old_total = stats.weighted_rating * stats.rating_count;
stats.rating_count += 1;
stats.weighted_rating = (old_total + rating.weighted_score()) / stats.rating_count;
```

---

### Deployment

#### `record_deployment()`

```rust
pub fn record_deployment(
    &self,
    blueprint_id: &BlueprintId,
    deployer_did: &DID,
    target_realm: &str,
    mana_paid: u64,
    success: bool,
) -> Result<DeploymentResult>;
```

**Rückgabe:**

```rust
pub struct DeploymentResult {
    pub deployment_id: String,
    pub mana_cost: u64,
    pub creator_trust_boost: f64,  // 0.001 bei Erfolg
}
```

**Stats-Update bei Erfolg:**

- `deployment_count` erhöhen
- `total_earnings` aktualisieren
- `last_deployment` setzen
- `e_contribution` neu berechnen

---

### Analytics

#### `marketplace_stats()`

```rust
pub fn marketplace_stats(&self) -> Result<MarketplaceStats>;
```

**Rückgabe:**

```rust
pub struct MarketplaceStats {
    pub total_blueprints: u64,
    pub total_deployments: u64,
    pub total_ratings: u64,
    pub total_earnings: u64,
    pub blueprints_by_category: HashMap<String, u64>,
}
```

#### `creator_analytics()`

```rust
pub fn creator_analytics(&self, creator_did: &DID) -> Result<CreatorAnalytics>;
```

**Rückgabe:**

```rust
pub struct CreatorAnalytics {
    pub creator_did: String,
    pub blueprint_count: u64,
    pub blueprint_ids: Vec<BlueprintId>,
    pub total_deployments: u64,
    pub total_ratings: u64,
    pub total_earnings: u64,
    pub average_rating: f64,
}
```

---

## Verwendungsbeispiele

### Blueprint erstellen und publizieren

```rust
use crate::local::{
    Blueprint, BlueprintCategory, BlueprintLicense, BlueprintMarketplace,
    BlueprintStore, MarketplaceConfig, StoreSchema, SchemaFieldType,
};

// Marketplace initialisieren
let marketplace = BlueprintMarketplace::new(&keyspace, MarketplaceConfig::default())?;

// Blueprint erstellen
let blueprint = Blueprint::builder("E-Commerce Starter", "did:erynoa:self:alice")
    .description("Komplettes E-Commerce-Template mit Shop und Checkout")
    .category(BlueprintCategory::Commerce)
    .license(BlueprintLicense::Commercial { mana_fee: 100 })
    .tag("ecommerce")
    .tag("shop")
    .tag("checkout")
    .store(BlueprintStore {
        name: "products".to_string(),
        schema: StoreSchema::new("products", false)
            .with_field("name", SchemaFieldType::String)
            .with_field("price", SchemaFieldType::Number)
            .with_field("stock", SchemaFieldType::Number)
            .with_field("image", SchemaFieldType::Blob),
        personal: false,
        description: Some("Produktkatalog".to_string()),
        initial_data: None,
    })
    .store(BlueprintStore {
        name: "orders".to_string(),
        schema: StoreSchema::new("orders", true)  // Personal
            .with_field("items", SchemaFieldType::Array(
                Box::new(SchemaFieldType::String)
            ))
            .with_field("total", SchemaFieldType::Number)
            .with_field("status", SchemaFieldType::String),
        personal: true,
        description: Some("Bestellungen pro User".to_string()),
        initial_data: None,
    })
    .build();

// Publizieren (erfordert Trust-R > 0.8, Trust-Ω > 1.5)
let result = marketplace.publish(blueprint, 0.9, 2.0)?;
println!("Published: {} (Mana: {})", result.blueprint_id, result.mana_cost);
```

### Blueprints suchen

```rust
use crate::local::{SearchQuery, BlueprintCategory};

// Suche nach Kategorie und Text
let results = marketplace.search(SearchQuery {
    category: Some(BlueprintCategory::Commerce),
    text: Some("shop".to_string()),
    min_novelty: Some(5.0),
    limit: Some(10),
    ..Default::default()
})?;

for result in results {
    println!(
        "{} (Score: {:.2}, Deployments: {})",
        result.blueprint.name,
        result.ranking_score,
        result.stats.deployment_count
    );
}
```

### Blueprint bewerten

```rust
use crate::domain::DID;

let rater = DID::new_self("bob");
let rating_result = marketplace.rate(
    &blueprint_id,
    &rater,
    5,  // Score: 1-5
    Some("Excellent template! Saved me hours of work.".to_string()),
    0.6,  // Rater Trust-R
    1.2,  // Rater Trust-Ω
)?;

println!(
    "Rated! Mana cost: {}, Weighted: {:.2}",
    rating_result.mana_cost,
    rating_result.weighted_score
);
```

### Deployment registrieren

```rust
let deployer = DID::new_self("charlie");
let deploy_result = marketplace.record_deployment(
    &blueprint_id,
    &deployer,
    "my-realm",
    150,   // Mana paid
    true,  // Success
)?;

println!(
    "Deployed! ID: {}, Creator boost: {:.4}",
    deploy_result.deployment_id,
    deploy_result.creator_trust_boost
);
```

### Fork erstellen

```rust
// Original laden
let original = marketplace.get_blueprint(&original_id)?.unwrap();

// Verbesserte Version erstellen
let mut forked = Blueprint::builder(
    "E-Commerce Pro",
    "did:erynoa:self:dave"
)
    .description("Enhanced version with subscription support")
    .category(original.category.clone())
    // ... weitere Anpassungen
    .build();

// Fork publizieren
let fork_result = marketplace.fork(
    &original_id,
    forked,
    0.85,
    1.8,
)?;

// Fork hat Referenz zum Original
let loaded = marketplace.get_blueprint(&fork_result.blueprint_id)?.unwrap();
assert_eq!(loaded.forked_from, Some(original_id));
```

### Analytics abrufen

```rust
// Marketplace-Gesamt
let stats = marketplace.marketplace_stats()?;
println!("Total Blueprints: {}", stats.total_blueprints);
println!("Total Deployments: {}", stats.total_deployments);
println!("By Category: {:?}", stats.blueprints_by_category);

// Creator-spezifisch
let alice = DID::new_self("alice");
let analytics = marketplace.creator_analytics(&alice)?;
println!("Alice's Blueprints: {}", analytics.blueprint_count);
println!("Total Earnings: {} Mana", analytics.total_earnings);
println!("Average Rating: {:.1}", analytics.average_rating);
```

---

## Gaming-Resistenz

### Axiom Κ19: Power-Begrenzung

| Mechanismus           | Beschreibung                              | Parameter                 |
| --------------------- | ----------------------------------------- | ------------------------- |
| **Creator Power-Cap** | Max 10 Blueprints pro Creator in Listings | `creator_power_cap: 10`   |
| **Novelty-Threshold** | Minimum 3.0 Surprisal für Upload          | `min_novelty_score: 3.0`  |
| **Rate-Limiting**     | Max 10 Ratings pro Tag                    | `max_ratings_per_day: 10` |

### Axiom Κ20: Alignment-Gewichtung

| Mechanismus            | Beschreibung                  | Formel                                          |
| ---------------------- | ----------------------------- | ----------------------------------------------- |
| **Ω-weighted Ratings** | Alignierte Rater zählen mehr  | $\text{weight} = \frac{\min(\Omega, 3.0)}{3.0}$ |
| **Trust-gated Upload** | Nur vertrauenswürdige Creator | R ≥ 0.8, Ω ≥ 1.5                                |
| **Diversity-Boost**    | Innovative Blueprints steigen | `diversity × 1.5`                               |

### Anti-Spam-Maßnahmen

```
┌─────────────────────────────────────────────────────────────────────┐
│                      GAMING-RESISTENZ                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Upload-Gate:                                                       │
│  ├── Trust-R ≥ 0.8 (hohe Reputation erforderlich)                  │
│  ├── Trust-Ω ≥ 1.5 (Alignment mit Systemzielen)                    │
│  └── Novelty ≥ 3.0 (keine Duplikate)                               │
│                                                                     │
│  Rating-Protection:                                                 │
│  ├── Doppel-Rating verhindert                                      │
│  ├── 10 Ratings/Tag Maximum                                        │
│  ├── Trust-R ≥ 0.3 für Rating                                      │
│  └── Ω-gewichtete Scores                                           │
│                                                                     │
│  Ranking-Fairness:                                                  │
│  ├── Power-Cap: 10 Blueprints/Creator                              │
│  ├── Diversity-Boost für Innovation                                │
│  └── Bayessche Rating-Aggregation                                  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Persistenz-Schema

### Fjall-Partitionen

| Partition                 | Key-Format                          | Value                        | Beschreibung            |
| ------------------------- | ----------------------------------- | ---------------------------- | ----------------------- |
| `marketplace_blueprints`  | `{blueprint_id}`                    | `Blueprint` (JSON)           | Blueprint-Daten         |
| `marketplace_stats`       | `stats:{blueprint_id}`              | `BlueprintStats` (JSON)      | Aggregierte Statistiken |
| `marketplace_ratings`     | `rating:{blueprint_id}:{rater_did}` | `BlueprintRating` (JSON)     | Einzelne Bewertungen    |
| `marketplace_deployments` | `deployment:{deployment_id}`        | `BlueprintDeployment` (JSON) | Deployment-Records      |

### Cache-Strategie

```rust
cache: Arc<RwLock<HashMap<BlueprintId, Blueprint>>>
```

- **Read-Through**: Cache wird bei `get_blueprint()` gefüllt
- **Write-Through**: Cache wird bei `publish()` aktualisiert
- **Thread-Safe**: `RwLock` für concurrent access
- **No Eviction**: Unbegrenzter Cache (für MVP)

---

## Tests

### Testübersicht (20 Tests)

| Test                              | Beschreibung                        |
| --------------------------------- | ----------------------------------- |
| `test_blueprint_creation`         | Blueprint-Builder und ID-Berechnung |
| `test_semver`                     | Semantische Versionierung           |
| `test_novelty_calculator`         | Surprisal-Berechnung                |
| `test_publish_blueprint`          | Upload mit Trust-Prüfung            |
| `test_publish_insufficient_trust` | Trust-R/Ω Ablehnung                 |
| `test_rate_blueprint`             | Bewertung mit Ω-Gewichtung          |
| `test_rate_duplicate`             | Doppel-Rating-Schutz                |
| `test_record_deployment`          | Deployment-Tracking                 |
| `test_search_blueprints`          | Suche mit Filtern                   |
| `test_fork_blueprint`             | Fork-Erstellung                     |
| `test_new_version`                | Versions-Publishing                 |
| `test_marketplace_stats`          | Gesamt-Statistiken                  |
| `test_creator_analytics`          | Creator-Dashboard                   |
| `test_power_cap`                  | Dominanz-Prävention                 |

### Test ausführen

```bash
# Alle Marketplace-Tests
cargo test blueprint_marketplace --features local

# Mit Output
cargo test blueprint_marketplace --features local -- --nocapture
```

---

## Integration mit anderen Modulen

### Abhängigkeiten

```rust
use crate::domain::DID;                           // Identity
use crate::local::realm_storage::{StoreSchema, StoreValue};  // Storage
use fjall::{Keyspace, PartitionHandle};           // Persistenz
use parking_lot::RwLock;                          // Thread-Safety
```

### Realm-Integration (geplant)

```rust
// Deployment-Flow:
// 1. Blueprint aus Marketplace laden
// 2. Stores im Realm erstellen
// 3. Policies installieren
// 4. Sagas ausführen
// 5. Deployment registrieren
```

### Trust-System-Integration

```rust
// Bei Publish:
// - Prüfe creator_trust_r >= 0.8
// - Prüfe creator_trust_omega >= 1.5

// Bei Rating:
// - Prüfe rater_trust_r >= 0.3
// - Gewichte mit rater_trust_omega

// Bei Deployment:
// - Boost creator_trust_r um 0.001
```

---

## Erweiterungsmöglichkeiten

### Kurzfristig

1. **ECL-Validierung**: Syntax-Check bei Policy-Upload
2. **Anomaly-Detection**: Koordinierte Rating-Erkennung
3. **Cache-Eviction**: LRU-Cache für große Marketplaces

### Mittelfristig

1. **Search-Index**: Volltextsuche mit Tantivy
2. **Dependency-Graph**: Blueprint-Abhängigkeiten
3. **Audit-Trail**: Änderungshistorie

### Langfristig

1. **P2P-Sync**: Marketplace über libp2p verteilen
2. **AI-Recommendations**: ML-basierte Empfehlungen
3. **Smart-Contracts**: On-chain Lizenz-Enforcement

---

## Referenzen

- **World-Formula**: [WORLD-FORMULA.md](../../concept-v3/WORLD-FORMULA.md) (Axiome Κ19, Κ20)
- **Trust-System**: [LOGIC.md](../../concept-v4/LOGIC.md)
- **Storage-Layer**: [realm_storage.rs](../../../backend/src/local/realm_storage.rs)
- **DID-System**: [domain/mod.rs](../../../backend/src/domain/mod.rs)

---

_Dokumentation erstellt: Juni 2025_
_Modul-Version: 1.0.0_
_Tests: 20 passing_
1. **P2P-Sync**: Marketplace über libp2p verteilen
2. **AI-Recommendations**: ML-basierte Empfehlungen
3. **Smart-Contracts**: On-chain Lizenz-Enforcement

---

## Referenzen

- **World-Formula**: [WORLD-FORMULA.md](../../concept-v3/WORLD-FORMULA.md) (Axiome Κ19, Κ20)
- **Trust-System**: [LOGIC.md](../../concept-v4/LOGIC.md)
- **Storage-Layer**: [realm_storage.rs](../../../backend/src/local/realm_storage.rs)
- **DID-System**: [domain/mod.rs](../../../backend/src/domain/mod.rs)

---

*Dokumentation erstellt: Juni 2025*
*Modul-Version: 1.0.0*
*Tests: 20 passing*
