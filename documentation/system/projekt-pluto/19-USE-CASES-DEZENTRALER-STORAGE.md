# 📦 Use Cases: Pluto Realms als Dezentraler Binary-Storage

> **Teil von:** Projekt Pluto
> **Kategorie:** Anwendungsfälle & Spezifikation
> **Status:** Vollständige Integration mit Nervensystem-Architektur

---

## 1. Vision: Das Nervensystem als universelle Speicherschicht

### 1.1 Grundprinzip

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   PLUTO REALMS = DEZENTRALER BINARY-STORAGE IM NERVENSYSTEM                 ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Das Erynoa-Nervensystem bietet nicht nur dezentrale Identität und         ║
║   Governance, sondern auch einen hochleistungsfähigen Blob-Store:           ║
║                                                                              ║
║   🧬 CONTENT-ADDRESSED    → Blake3-Hash (32 Byte) = unique ID               ║
║   📦 CHUNKED STORAGE      → 4-64 MB dynamische Chunks                       ║
║   🗜️ ZSTD-COMPRESSED      → Level 3-15 adaptive Kompression                 ║
║   🌍 P2P-SYNC             → Gossip + BitSwap für globale Verfügbarkeit      ║
║   🔐 TRUST-GATED          → Nur vertrauenswürdige Peers speichern           ║
║   ⚡ MANA-METERED         → Self-Healing Quotas verhindern Abuse            ║
║                                                                              ║
║   KERNIDEE:                                                                  ║
║   Jedes Realm kann einen oder mehrere Blob-Stores betreiben, die durch      ║
║   Trust kontrolliert und durch Mana-Budgets beschränkt werden.              ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 1.2 Nervensystem-Integration

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   BLOB-STORE ALS ORGAN IM NERVENSYSTEM                                       ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║            ┌─────────────────────────────────────────────────────┐           ║
║            │              ERYNOA NERVENSYSTEM                    │           ║
║            │                                                     │           ║
║            │   ┌─────────┐  ┌─────────┐  ┌─────────────────┐    │           ║
║            │   │ Identity│  │ Trust   │  │   Governance    │    │           ║
║            │   │ (Herz)  │  │ (Immun) │  │   (Großhirn)    │    │           ║
║            │   └────┬────┘  └────┬────┘  └────────┬────────┘    │           ║
║            │        │            │                │             │           ║
║            │        └────────────┼────────────────┘             │           ║
║            │                     │                              │           ║
║            │                     ▼                              │           ║
║            │            ┌────────────────┐                      │           ║
║            │            │   BLOB-STORE   │  ← Langzeitgedächtnis│           ║
║            │            │ (Speicherorgan)│                      │           ║
║            │            └────────────────┘                      │           ║
║            │                     │                              │           ║
║            │      ┌──────────────┼──────────────┐               │           ║
║            │      ▼              ▼              ▼               │           ║
║            │  ┌───────┐    ┌──────────┐    ┌─────────┐          │           ║
║            │  │Docker │    │AI-Models │    │ Media   │          │           ║
║            │  │Images │    │ Weights  │    │ Assets  │          │           ║
║            │  └───────┘    └──────────┘    └─────────┘          │           ║
║            │                                                     │           ║
║            └─────────────────────────────────────────────────────┘           ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 2. Technische Basis: Blob-Store-Architektur

### 2.1 Content-Addressable Storage (CAS)

```rust
/// Blake3-basierte Content-Adressierung (Κ10-konform)
pub struct BlobId {
    /// 32-Byte Blake3-Hash des Inhalts
    hash: [u8; 32],
    /// Realm, in dem der Blob gespeichert ist
    realm_id: RealmId,
}

impl BlobId {
    /// Erzeugt Blob-ID aus Inhalt
    pub fn from_content(content: &[u8], realm: &RealmId) -> Self {
        let hash = blake3::hash(content);
        Self {
            hash: *hash.as_bytes(),
            realm_id: realm.clone(),
        }
    }

    /// URL-Darstellung
    pub fn to_url(&self) -> String {
        format!(
            "erynoa://{}/store/blob/{}",
            self.realm_id,
            hex::encode(&self.hash)
        )
    }
}

/// Chunk-Struktur für große Blobs
pub struct Chunk {
    /// Chunk-Index im Parent-Blob
    index: u32,
    /// Chunk-Daten (4-64 MB)
    data: Vec<u8>,
    /// Blake3-Hash dieses Chunks
    hash: [u8; 32],
    /// Kompressionsalgorithmus
    compression: CompressionAlgorithm,
}

/// Blob-Manifest für Multi-Chunk-Blobs
pub struct BlobManifest {
    /// Gesamter Blob-Hash (Root)
    root_hash: [u8; 32],
    /// Gesamtgröße (unkomprimiert)
    total_size: u64,
    /// Chunk-Hashes in Reihenfolge
    chunks: Vec<[u8; 32]>,
    /// Erstellungszeitpunkt
    created_at: Timestamp,
    /// Ersteller-DID
    creator: Did,
    /// Content-Type (MIME)
    content_type: String,
}
```

### 2.2 Realm-URL-Adressierung (Κ26)

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   BLOB-STORE URL-SCHEMA                                                      ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Generisches Format:                                                        ║
║   erynoa://<realm-id>/store/<store-name>/<key>[?params]                      ║
║                                                                              ║
║   Blob-spezifische Varianten:                                               ║
║   ├── erynoa://docker-registry/store/layers/<digest>                         ║
║   ├── erynoa://ai-models/store/weights/<model-id>/<version>                 ║
║   ├── erynoa://media-cdn/store/media/<content-hash>                         ║
║   ├── erynoa://game-assets/store/assets/<category>/<asset-id>               ║
║   ├── erynoa://enterprise-vault/store/binaries/<path>                        ║
║   └── erynoa://science-data/store/datasets/<dataset-id>/<version>           ║
║                                                                              ║
║   Query-Parameter:                                                           ║
║   ├── ?chunk=0-5       → Nur Chunks 0-5 abrufen (Range-Request)             ║
║   ├── ?version=latest  → Neueste Version                                     ║
║   ├── ?signature=did   → Nur signierte Versionen vom angegebenen DID        ║
║   └── ?pin=true        → Lokales Pinning anfordern                          ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 2.3 Trust/Gas/Mana-Kosten für Blob-Operationen

```rust
/// Kosten-Modell für Blob-Operationen (Nervensystem-konform)
pub struct BlobOperationCosts {
    /// Upload: 1 Mana pro MB + 0.1 Gas pro Chunk
    upload: CostFormula,
    /// Download: 0.1 Mana pro MB (für Bandbreite)
    download: CostFormula,
    /// Pin: 0.01 Mana pro MB pro Tag
    pin: CostFormula,
    /// Delete: 0.5 Gas (unwiderrufliche Operation)
    delete: CostFormula,
}

impl BlobOperationCosts {
    /// Standard-Kosten für alle Realms
    pub fn default() -> Self {
        Self {
            upload: CostFormula {
                mana_per_mb: 1.0,
                gas_per_operation: 0.1,
                trust_minimum: 0.3,
            },
            download: CostFormula {
                mana_per_mb: 0.1,
                gas_per_operation: 0.0,
                trust_minimum: 0.0,  // Öffentliche Blobs: kein Trust nötig
            },
            pin: CostFormula {
                mana_per_mb_per_day: 0.01,
                gas_per_operation: 0.0,
                trust_minimum: 0.5,  // Pinning erfordert höheres Vertrauen
            },
            delete: CostFormula {
                mana_per_operation: 0.0,
                gas_per_operation: 0.5,  // Gas = unwiderruflich
                trust_minimum: 0.7,      // Nur vertrauenswürdige Akteure löschen
            },
        }
    }
}
```

---

## 3. Use Case 1: Dezentrales Docker/OCI-Image Registry

### 3.1 Vision

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   🐳 DEZENTRALES DOCKER REGISTRY                                             ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   PROBLEM:                                                                   ║
║   - Docker Hub ist zentralisiert, hat Rate-Limits, kann ausfallen           ║
║   - Private Registries erfordern eigene Infrastruktur                       ║
║   - Keine dezentrale Vertrauenskette für Image-Integrität                   ║
║                                                                              ║
║   ERYNOA-LÖSUNG:                                                             ║
║   - Realm als dezentrales Registry mit Trust-Verification                   ║
║   - Layer-Deduplizierung via Blake3-CAS (globale Effizienz)                 ║
║   - P2P-Sync für schnelle Verteilung                                        ║
║   - Governance für Image-Approval und Security-Audits                       ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 3.2 Realm-Konfiguration

```ecl
// Docker Registry Realm Blueprint
realm DockerRegistry {
    type: VirtualRealm,
    parent: "erynoa://root",

    // Trust-Requirements
    trust: {
        min_join: 0.3,           // Lesen: niedriger Trust
        min_push: 0.6,           // Pushen: mittlerer Trust
        min_delete: 0.8,         // Löschen: hoher Trust
        min_admin: 0.95,         // Admin: sehr hoher Trust
    },

    // Mana-Budget
    mana: {
        total_budget: 1_000_000,  // 1M Mana für Storage
        per_user_daily: 10_000,   // 10k Mana pro User/Tag
        regeneration_rate: 0.1,   // 10% pro Stunde
    },

    // Store-Schema
    stores: {
        layers: {
            type: "blob",
            content_addressable: true,
            max_size_mb: 5120,    // 5 GB max pro Layer
            compression: "zstd-15",
        },
        manifests: {
            type: "json",
            schema: "oci-manifest-v2",
            max_size_kb: 512,
        },
        tags: {
            type: "keyvalue",
            key_pattern: "^[a-z0-9][a-z0-9._-]{0,127}$",
        },
    },

    // Governance
    governance: {
        type: "reputation",
        proposal_threshold: 0.7,
        security_audit_required: true,
    },
}
```

### 3.3 Realm-URL-Schema für Docker

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   DOCKER REGISTRY URL-MAPPING                                                ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   OCI-Spec URL                    →  Erynoa Realm-URL                        ║
║   ──────────────────────────────────────────────────────────────────────────  ║
║   /v2/<name>/blobs/<digest>       →  erynoa://docker-registry/store/layers/  ║
║                                        <digest>                              ║
║                                                                              ║
║   /v2/<name>/manifests/<ref>      →  erynoa://docker-registry/store/         ║
║                                        manifests/<name>/<ref>                ║
║                                                                              ║
║   /v2/<name>/tags/list            →  erynoa://docker-registry/store/         ║
║                                        tags/<name>                           ║
║                                                                              ║
║   Beispiel:                                                                  ║
║   docker pull erynoa.io/myapp:v1.0                                          ║
║   → erynoa://docker-registry/store/manifests/myapp/v1.0                     ║
║   → erynoa://docker-registry/store/layers/<sha256:abc123...>                ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 3.4 Agent Shell Integration: Registry-Bot

```rust
/// DevOps-Agent für automatisches Registry-Management
pub struct DockerRegistryAgent {
    /// Agent-DID
    did: Did,
    /// Shell-Capabilities
    capabilities: Vec<ShellCapability>,
    /// Realm-Zugehörigkeit
    realm: RealmId,
}

impl DockerRegistryAgent {
    /// Erforderliche Capabilities
    pub fn required_capabilities() -> Vec<ShellCapability> {
        vec![
            // Container-Verwaltung
            ShellCapability::ContainerControl {
                runtime: ContainerRuntime::Docker,
                containers: vec!["erynoa-registry-*".to_string()],
                actions: vec![
                    ContainerAction::Logs,
                    ContainerAction::Stats,
                    ContainerAction::Restart,
                ],
            },
            // Log-Zugriff
            ShellCapability::PathAccess {
                paths: vec![
                    PathPermission {
                        path: "/var/log/erynoa-registry/**".to_string(),
                        read: true,
                        write: false,
                        execute: false,
                        delete: false,
                    },
                ],
                user: "erynoa".to_string(),
            },
            // Scheduled Tasks für Cleanup
            ShellCapability::ScheduledTasks {
                namespace: "registry-agent-*".to_string(),
                max_concurrent: 3,
            },
        ]
    }
}
```

### 3.5 ECL-Policy für Image-Push

```ecl
// Policy: Wer darf Images pushen?
policy DockerPushPolicy {
    // Basis-Requirements
    require trust >= 0.6;
    require mana >= 1000;  // Mindest-Mana für Push

    // Governance-Approval für neue Images
    if image.is_new() {
        require governance.approved(image.name);
    }

    // Security-Scan erforderlich
    require security.scanned(image.digest);

    // Rate-Limiting
    rate_limit: 10 pushes per hour;

    // Gas-Kosten für permanente Speicherung
    cost: {
        gas: 0.5 * image.layers.count,
        mana: image.size_mb * 1.0,
    };
}
```

### 3.6 Vollständiger Workflow

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   DOCKER PUSH WORKFLOW                                                       ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   1. AUTHENTIFIZIERUNG                                                       ║
║      └── docker login erynoa.io                                             ║
║          └── DID-Challenge → Signature → JWT mit Trust-Level                ║
║                                                                              ║
║   2. LAYER-UPLOAD                                                            ║
║      └── docker push erynoa.io/myapp:v1.0                                   ║
║          ├── Layer 1: sha256:abc... (50 MB)                                 ║
║          │   ├── Blake3-Hash prüfen                                         ║
║          │   ├── Deduplizierung: existiert bereits? → Skip                  ║
║          │   ├── Chunk (4-64 MB) → Compress (zstd-15)                       ║
║          │   ├── Mana-Kosten: 50 Mana                                       ║
║          │   └── P2P-Sync: Gossip an 5+ Peers                               ║
║          └── Layer 2: sha256:def... (100 MB)                                ║
║              └── ... (analog)                                               ║
║                                                                              ║
║   3. MANIFEST-PUBLISH                                                        ║
║      └── Manifest JSON                                                       ║
║          ├── ECL-Policy prüfen                                              ║
║          ├── Gas-Kosten: 2.5 Gas (5 Layer × 0.5)                            ║
║          ├── Governance-Approval (falls neues Image)                        ║
║          └── Event: ImagePublished { name, tag, digest, creator }           ║
║                                                                              ║
║   4. SYNC                                                                    ║
║      └── Registry-Agent                                                      ║
║          ├── BitSwap: Andere Nodes pullen populäre Layer                    ║
║          ├── Pinning-Policy: Behalte Layer mit Downloads > 100              ║
║          └── Cleanup: Unpinned Layer nach 30 Tagen entfernen                ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 4. Use Case 2: AI-Modell-Registry & Weight-Sharing

### 4.1 Vision

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   🤖 DEZENTRALE AI-MODELL-REGISTRY                                          ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   PROBLEM:                                                                   ║
║   - HuggingFace/OpenAI kontrollieren Modell-Distribution                    ║
║   - Große Modelle (70B+) sind teuer zu hosten                               ║
║   - Keine dezentrale Provenance/Lineage für Modelle                         ║
║   - Fine-Tuning-Ergebnisse schwer zu teilen                                 ║
║                                                                              ║
║   ERYNOA-LÖSUNG:                                                             ║
║   - Realm für AI-Modelle mit Versioning und Lineage                         ║
║   - Delta-Weights für effizientes Fine-Tuning-Sharing                       ║
║   - Trust-basierte Qualitätsbewertung                                       ║
║   - P2P-Verteilung für große Modelle (BitTorrent-ähnlich)                  ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 4.2 Realm-Konfiguration

```ecl
// AI Model Registry Realm Blueprint
realm AIModelRegistry {
    type: VirtualRealm,
    parent: "erynoa://root",

    // Trust-Requirements
    trust: {
        min_join: 0.2,           // Lesen: sehr niedriger Trust
        min_upload: 0.7,         // Upload: hoher Trust (Qualitätssicherung)
        min_curate: 0.85,        // Kuratieren: sehr hoher Trust
    },

    // Mana-Budget (großzügig für große Modelle)
    mana: {
        total_budget: 100_000_000,  // 100M Mana
        per_user_daily: 1_000_000,  // 1M Mana pro User/Tag
        regeneration_rate: 0.05,    // 5% pro Stunde
    },

    // Store-Schema
    stores: {
        // Basis-Modelle (vollständige Weights)
        base_models: {
            type: "blob",
            content_addressable: true,
            max_size_gb: 500,      // Bis zu 500 GB pro Modell
            compression: "zstd-3", // Schnellere Kompression für große Dateien
            chunking: {
                min_size_mb: 64,
                max_size_mb: 256,  // Große Chunks für Streaming
            },
        },

        // Delta-Weights (LoRA, PEFT, etc.)
        deltas: {
            type: "blob",
            content_addressable: true,
            max_size_gb: 10,
            compression: "zstd-15",  // Hohe Kompression für kleine Deltas
            requires_parent: true,   // Muss Base-Modell referenzieren
        },

        // Modell-Metadaten
        metadata: {
            type: "json",
            schema: "model-card-v1",
            fields: [
                "architecture", "parameters", "license",
                "training_data", "performance_metrics",
                "parent_model", "lineage",
            ],
        },

        // Benchmark-Ergebnisse
        benchmarks: {
            type: "json",
            schema: "benchmark-results-v1",
            immutable_after: "7d",  // Nach 7 Tagen unveränderlich
        },
    },

    // Governance: Wissenschaftliche Community
    governance: {
        type: "reputation",
        proposal_threshold: 0.8,
        peer_review_required: true,
        citation_tracking: true,
    },
}
```

### 4.3 Realm-URL-Schema für AI-Modelle

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   AI MODEL REGISTRY URL-SCHEMA                                               ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Basis-Modelle:                                                             ║
║   erynoa://ai-models/store/base_models/<model-id>/<version>                 ║
║   Beispiel: erynoa://ai-models/store/base_models/llama-3-70b/v1.0           ║
║                                                                              ║
║   Delta-Weights:                                                             ║
║   erynoa://ai-models/store/deltas/<parent-model>/<delta-id>                 ║
║   Beispiel: erynoa://ai-models/store/deltas/llama-3-70b/german-finetune-v1  ║
║                                                                              ║
║   Metadaten:                                                                 ║
║   erynoa://ai-models/store/metadata/<model-id>                               ║
║   Beispiel: erynoa://ai-models/store/metadata/llama-3-70b                   ║
║                                                                              ║
║   Benchmarks:                                                                ║
║   erynoa://ai-models/store/benchmarks/<model-id>/<benchmark-name>           ║
║   Beispiel: erynoa://ai-models/store/benchmarks/llama-3-70b/mmlu            ║
║                                                                              ║
║   Query-Parameter:                                                           ║
║   ?shard=0-7       → Nur bestimmte Shards für paralleles Laden              ║
║   ?format=safetensors → Spezifisches Format                                 ║
║   ?quantization=q4 → Quantisierte Version                                   ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 4.4 Agent Shell Integration: Model-Curator-Bot

```rust
/// AI-Agent für Modell-Kuratierung und Benchmarking
pub struct ModelCuratorAgent {
    did: Did,
    capabilities: Vec<ShellCapability>,
    realm: RealmId,
}

impl ModelCuratorAgent {
    /// Erforderliche Capabilities für Benchmark-Ausführung
    pub fn required_capabilities() -> Vec<ShellCapability> {
        vec![
            // GPU-Container für Inference
            ShellCapability::ContainerControl {
                runtime: ContainerRuntime::Docker,
                containers: vec!["erynoa-inference-*".to_string()],
                actions: vec![
                    ContainerAction::Start,
                    ContainerAction::Stop,
                    ContainerAction::Logs,
                    ContainerAction::Exec,  // Für Benchmark-Ausführung
                ],
            },
            // Benchmark-Ergebnisse speichern
            ShellCapability::PathAccess {
                paths: vec![
                    PathPermission {
                        path: "/var/lib/erynoa/benchmarks/**".to_string(),
                        read: true,
                        write: true,
                        execute: false,
                        delete: false,
                    },
                ],
                user: "erynoa".to_string(),
            },
            // Scheduled Benchmarks
            ShellCapability::ScheduledTasks {
                namespace: "model-curator-*".to_string(),
                max_concurrent: 2,  // Max 2 parallele Benchmarks (GPU-limitiert)
            },
        ]
    }

    /// Automatischer Benchmark-Workflow
    pub async fn run_benchmark(&self, model_url: &str, benchmark: &str) -> Result<BenchmarkResult> {
        // 1. Modell herunterladen (P2P)
        let model = self.download_model(model_url).await?;

        // 2. Inference-Container starten
        let container = self.start_inference_container(&model).await?;

        // 3. Benchmark ausführen
        let result = self.execute_benchmark(&container, benchmark).await?;

        // 4. Ergebnis publizieren
        self.publish_result(model_url, benchmark, &result).await?;

        Ok(result)
    }
}
```

### 4.5 ECL-Policy für Modell-Upload

```ecl
// Policy: Wer darf Modelle hochladen?
policy ModelUploadPolicy {
    // Basis-Requirements
    require trust >= 0.7;
    require mana >= 100_000;  // Mindest-Mana für große Uploads

    // Lizenz-Prüfung
    require model.license in [
        "MIT", "Apache-2.0", "CC-BY-4.0", "CC-BY-SA-4.0",
        "OpenRAIL", "Llama-Community", "Gemma",
    ];

    // Model-Card erforderlich
    require model.has_metadata([
        "architecture", "parameters", "training_data",
    ]);

    // Peer-Review für große Modelle (>10B Parameter)
    if model.parameters > 10_000_000_000 {
        require governance.peer_reviewed(model.id);
    }

    // Delta-Weights müssen Parent referenzieren
    if model.is_delta {
        require model.parent_exists();
        require model.parent.license.allows_derivatives;
    }

    // Kosten (sehr große Modelle = hohe Kosten)
    cost: {
        gas: log2(model.size_gb) * 1.0,  // Logarithmisch
        mana: model.size_gb * 100,        // Linear
    };
}
```

### 4.6 Vollständiger Workflow

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   AI MODEL UPLOAD WORKFLOW                                                   ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   1. MODEL-CARD ERSTELLEN                                                    ║
║      └── erynoa model init llama-3-german-finetune                          ║
║          ├── Architecture: LlamaForCausalLM                                 ║
║          ├── Parameters: 8B (Delta: 100M trainable)                         ║
║          ├── Parent: erynoa://ai-models/store/base_models/llama-3-8b/v1.0   ║
║          ├── Training Data: German Wikipedia, News, Books                   ║
║          └── License: Llama-Community                                       ║
║                                                                              ║
║   2. WEIGHTS UPLOAD                                                          ║
║      └── erynoa model push llama-3-german-finetune                          ║
║          ├── Chunk 1: adapter_model.safetensors (400 MB)                    ║
║          │   ├── Blake3: 0xabc123...                                        ║
║          │   ├── Compress: zstd-15 → 280 MB                                 ║
║          │   ├── Mana: 28,000 Mana                                          ║
║          │   └── P2P-Sync: 3+ Peers                                         ║
║          └── Manifest: model-config.json                                     ║
║                                                                              ║
║   3. GOVERNANCE (optional für große Modelle)                                 ║
║      └── Peer-Review                                                         ║
║          ├── 3 Kuratoren mit Trust > 0.85 prüfen                            ║
║          ├── Benchmark-Ergebnisse werden verifiziert                        ║
║          └── 72h Review-Periode                                             ║
║                                                                              ║
║   4. DISTRIBUTION                                                            ║
║      └── Model-Curator-Agent                                                 ║
║          ├── Auto-Benchmark: MMLU, HellaSwag, German-Eval                   ║
║          ├── Ergebnis publizieren                                           ║
║          └── Featured-Liste bei guter Performance                           ║
║                                                                              ║
║   5. NUTZUNG                                                                 ║
║      └── erynoa model pull erynoa://ai-models/store/deltas/                 ║
║              llama-3-8b/german-finetune-v1                                  ║
║          ├── P2P-Download (BitSwap)                                         ║
║          ├── Mana-Kosten: 2,800 Mana (10% von Upload)                       ║
║          └── Auto-Merge mit Base-Model                                       ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 5. Use Case 3: Dezentrales Media-Storage für Social Realms

### 5.1 Vision

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   🎬 DEZENTRALES MEDIA-CDN                                                   ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   PROBLEM:                                                                   ║
║   - YouTube/TikTok kontrollieren Content und Monetarisierung                ║
║   - IPFS-basierte Alternativen haben keine Governance                       ║
║   - Keine dezentrale Moderation mit Community-Standards                     ║
║                                                                              ║
║   ERYNOA-LÖSUNG:                                                             ║
║   - Realm als Social Media mit eigenem CDN                                  ║
║   - Trust-basierte Content-Moderation                                       ║
║   - Creator-owned Monetarisierung via Governance                            ║
║   - Adaptive Streaming mit P2P-Delivery                                     ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 5.2 Realm-Konfiguration

```ecl
// Social Media Realm Blueprint
realm SocialMediaRealm {
    type: VirtualRealm,
    parent: "erynoa://root",

    // Trust-Requirements
    trust: {
        min_join: 0.0,            // Öffentlich lesbar
        min_post: 0.4,            // Posten: mittlerer Trust
        min_moderate: 0.8,        // Moderation: hoher Trust
        min_monetize: 0.6,        // Monetarisierung: höherer Trust
    },

    // Mana-Budget
    mana: {
        total_budget: 500_000_000,  // 500M Mana für CDN
        per_user_daily: 50_000,     // 50k Mana pro User/Tag
        regeneration_rate: 0.2,     // 20% pro Stunde
    },

    // Store-Schema
    stores: {
        // Video-Content
        videos: {
            type: "blob",
            content_addressable: true,
            max_size_gb: 50,
            compression: "none",  // Videos bereits komprimiert
            transcoding: {
                formats: ["mp4", "webm"],
                resolutions: ["1080p", "720p", "480p", "360p"],
                adaptive_streaming: true,
            },
        },

        // Bilder
        images: {
            type: "blob",
            content_addressable: true,
            max_size_mb: 100,
            formats: ["jpg", "png", "webp", "avif"],
            auto_resize: [2048, 1024, 512, 256, 64],  // Thumbnails
        },

        // Audio
        audio: {
            type: "blob",
            content_addressable: true,
            max_size_mb: 500,
            formats: ["mp3", "flac", "opus"],
        },

        // Posts (Text + References)
        posts: {
            type: "json",
            schema: "social-post-v1",
            max_size_kb: 64,
        },

        // Comments
        comments: {
            type: "json",
            schema: "comment-v1",
            threaded: true,
        },
    },

    // Governance: Community-driven
    governance: {
        type: "quadratic",  // Κ21: √tokens = votes
        moderation_council_size: 21,
        appeal_process: true,
        creator_revenue_share: 0.7,  // 70% an Creator
    },
}
```

### 5.3 Realm-URL-Schema für Media

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   SOCIAL MEDIA URL-SCHEMA                                                    ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Videos:                                                                    ║
║   erynoa://social-realm/store/videos/<content-hash>                         ║
║   erynoa://social-realm/store/videos/<content-hash>?quality=720p            ║
║   erynoa://social-realm/store/videos/<content-hash>?segment=0-10            ║
║                                                                              ║
║   Bilder:                                                                    ║
║   erynoa://social-realm/store/images/<content-hash>                         ║
║   erynoa://social-realm/store/images/<content-hash>?size=512                ║
║   erynoa://social-realm/store/images/<content-hash>?format=webp             ║
║                                                                              ║
║   Posts:                                                                     ║
║   erynoa://social-realm/store/posts/<post-id>                               ║
║   erynoa://social-realm/store/posts/<post-id>/comments                      ║
║                                                                              ║
║   Profile-Media:                                                             ║
║   erynoa://social-realm/profile/<did>/avatar                                ║
║   erynoa://social-realm/profile/<did>/banner                                ║
║   erynoa://social-realm/profile/<did>/gallery                               ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 5.4 Agent Shell Integration: Moderation-Bot

```rust
/// Content-Moderation-Agent für Social Realm
pub struct ModerationAgent {
    did: Did,
    capabilities: Vec<ShellCapability>,
    realm: RealmId,
}

impl ModerationAgent {
    /// Erforderliche Capabilities
    pub fn required_capabilities() -> Vec<ShellCapability> {
        vec![
            // ML-Container für Content-Analyse
            ShellCapability::ContainerControl {
                runtime: ContainerRuntime::Docker,
                containers: vec!["erynoa-moderation-*".to_string()],
                actions: vec![
                    ContainerAction::Start,
                    ContainerAction::Exec,  // ML-Inference
                    ContainerAction::Logs,
                ],
            },
            // Transcoding-Service
            ShellCapability::RestrictedCommands {
                allowed_commands: vec![
                    "ffmpeg -i * -vf scale=* -c:v libx264 *".to_string(),
                    "ffprobe -v quiet -print_format json *".to_string(),
                ],
                blocked_args: vec![
                    "-y".to_string(),  // Kein Überschreiben
                ],
                user: "erynoa".to_string(),
                timeout_secs: 3600,  // 1h für lange Videos
            },
            // Log-Zugriff für Audit
            ShellCapability::PathAccess {
                paths: vec![
                    PathPermission {
                        path: "/var/log/erynoa-moderation/**".to_string(),
                        read: true,
                        write: true,
                        execute: false,
                        delete: false,
                    },
                ],
                user: "erynoa".to_string(),
            },
        ]
    }

    /// Content-Moderation-Workflow
    pub async fn moderate_content(&self, content_url: &str) -> Result<ModerationResult> {
        // 1. Content analysieren (ML)
        let analysis = self.analyze_content(content_url).await?;

        // 2. Policy-Check
        let policy_result = self.check_policy(&analysis).await?;

        // 3. Bei Verletzung: Flag + Escalation
        if policy_result.violation_detected {
            self.flag_content(content_url, &policy_result).await?;
            self.notify_moderation_council(&policy_result).await?;
        }

        Ok(policy_result)
    }
}
```

### 5.5 ECL-Policy für Content-Upload

```ecl
// Policy: Content-Upload-Regeln
policy ContentUploadPolicy {
    // Basis-Requirements
    require trust >= 0.4;
    require mana >= 100;

    // Content-Typ-spezifische Limits
    match content.type {
        "video" => {
            require mana >= content.duration_minutes * 100;
            require content.duration_minutes <= 60;  // Max 1h
            require !content.ai_flagged("nsfw");
            require !content.ai_flagged("violence");
        },
        "image" => {
            require mana >= 10;
            require !content.ai_flagged("nsfw");
        },
        "audio" => {
            require mana >= content.duration_minutes * 10;
        },
    }

    // Monetarisierung erfordert höheren Trust
    if content.monetization_enabled {
        require trust >= 0.6;
        require account.age_days >= 30;
        require account.followers >= 100;
    }

    // Rate-Limiting
    rate_limit: match content.type {
        "video" => 5 per day,
        "image" => 50 per day,
        "audio" => 20 per day,
    };

    // Kosten
    cost: {
        mana: content.size_mb * 0.5,
        gas: 0.1,  // Minimal für schnellen Upload
    };
}
```

---

## 6. Use Case 4: Game-Asset-Registry

### 6.1 Vision

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   🎮 DEZENTRALE GAME-ASSET-REGISTRY                                          ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   PROBLEM:                                                                   ║
║   - Game-Assets sind an einzelne Spiele gebunden                            ║
║   - Keine Cross-Game-Interoperabilität                                      ║
║   - Spieler besitzen Assets nicht wirklich                                  ║
║   - Modding-Communities haben keine Standard-Distribution                   ║
║                                                                              ║
║   ERYNOA-LÖSUNG:                                                             ║
║   - Realm für Game-Assets mit echter Ownership (DID-basiert)               ║
║   - Cross-Game-Standards für Asset-Formate                                  ║
║   - Trust-basierte Qualitätskontrolle für Mods                             ║
║   - P2P-Distribution für schnelles Asset-Loading                            ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 6.2 Realm-Konfiguration

```ecl
// Game Asset Registry Realm Blueprint
realm GameAssetRegistry {
    type: VirtualRealm,
    parent: "erynoa://root",

    // Trust-Requirements
    trust: {
        min_join: 0.0,            // Öffentlich browsebar
        min_download: 0.2,        // Download: minimaler Trust
        min_upload: 0.5,          // Upload: mittlerer Trust
        min_curate: 0.8,          // Kuratierung: hoher Trust
        min_verify: 0.9,          // Verifizierung: sehr hoher Trust
    },

    // Mana-Budget
    mana: {
        total_budget: 200_000_000,
        per_user_daily: 100_000,
        regeneration_rate: 0.15,
    },

    // Store-Schema
    stores: {
        // 3D-Modelle
        models: {
            type: "blob",
            content_addressable: true,
            max_size_gb: 10,
            formats: ["glb", "gltf", "fbx", "obj"],
            metadata: ["polycount", "materials", "rigging", "lod_levels"],
        },

        // Texturen
        textures: {
            type: "blob",
            content_addressable: true,
            max_size_mb: 500,
            formats: ["png", "jpg", "dds", "ktx2"],
            auto_mipmap: true,
        },

        // Audio-Assets
        audio: {
            type: "blob",
            content_addressable: true,
            max_size_mb: 100,
            formats: ["wav", "ogg", "mp3"],
        },

        // Shader
        shaders: {
            type: "blob",
            content_addressable: true,
            max_size_kb: 512,
            formats: ["glsl", "hlsl", "wgsl"],
            sandboxed_execution: true,  // Sicherheit!
        },

        // Asset-Bundles (komplette Packs)
        bundles: {
            type: "blob",
            content_addressable: true,
            max_size_gb: 50,
            manifest_required: true,
        },

        // Asset-Metadaten
        metadata: {
            type: "json",
            schema: "game-asset-v1",
            fields: [
                "category", "tags", "license", "compatible_games",
                "creator", "version", "dependencies",
            ],
        },
    },

    // Governance: Community Curation
    governance: {
        type: "reputation",
        curator_rewards: true,
        quality_tiers: ["verified", "community", "experimental"],
    },
}
```

### 6.3 Realm-URL-Schema für Game-Assets

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   GAME ASSET URL-SCHEMA                                                      ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Modelle:                                                                   ║
║   erynoa://game-assets/store/models/<category>/<asset-id>                   ║
║   erynoa://game-assets/store/models/characters/knight-v1                    ║
║   erynoa://game-assets/store/models/vehicles/sports-car?lod=2               ║
║                                                                              ║
║   Texturen:                                                                  ║
║   erynoa://game-assets/store/textures/<category>/<asset-id>                 ║
║   erynoa://game-assets/store/textures/pbr/metal-rust?size=2048              ║
║                                                                              ║
║   Bundles:                                                                   ║
║   erynoa://game-assets/store/bundles/<bundle-id>                            ║
║   erynoa://game-assets/store/bundles/fantasy-village-pack-v2                ║
║                                                                              ║
║   Inventar (Spieler-owned):                                                 ║
║   erynoa://game-assets/profile/<did>/inventory                              ║
║   erynoa://game-assets/profile/<did>/inventory/<asset-id>                   ║
║                                                                              ║
║   Query-Parameter:                                                           ║
║   ?format=glb      → Spezifisches Format                                    ║
║   ?lod=0-3         → Level of Detail                                        ║
║   ?compatible=unity → Nur Unity-kompatible                                  ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 6.4 Agent Shell Integration: Asset-Validator-Bot

```rust
/// Asset-Validation-Agent für Game-Assets
pub struct AssetValidatorAgent {
    did: Did,
    capabilities: Vec<ShellCapability>,
    realm: RealmId,
}

impl AssetValidatorAgent {
    /// Erforderliche Capabilities
    pub fn required_capabilities() -> Vec<ShellCapability> {
        vec![
            // Validation-Container
            ShellCapability::ContainerControl {
                runtime: ContainerRuntime::Docker,
                containers: vec!["erynoa-asset-validator-*".to_string()],
                actions: vec![
                    ContainerAction::Start,
                    ContainerAction::Exec,
                    ContainerAction::Logs,
                ],
            },
            // Asset-Processing-Commands
            ShellCapability::RestrictedCommands {
                allowed_commands: vec![
                    "gltf-validator *".to_string(),
                    "meshlab-server *".to_string(),
                    "imagemagick identify *".to_string(),
                ],
                blocked_args: vec![],
                user: "erynoa".to_string(),
                timeout_secs: 300,
            },
            // Temp-Storage für Validation
            ShellCapability::PathAccess {
                paths: vec![
                    PathPermission {
                        path: "/tmp/erynoa-asset-validation/**".to_string(),
                        read: true,
                        write: true,
                        execute: true,
                        delete: true,
                    },
                ],
                user: "erynoa".to_string(),
            },
        ]
    }

    /// Validiere Asset
    pub async fn validate_asset(&self, asset_url: &str) -> Result<ValidationResult> {
        // 1. Asset herunterladen
        let asset = self.download_asset(asset_url).await?;

        // 2. Format-spezifische Validation
        let format_result = match asset.format {
            "glb" | "gltf" => self.validate_gltf(&asset).await?,
            "fbx" => self.validate_fbx(&asset).await?,
            "png" | "jpg" => self.validate_texture(&asset).await?,
            _ => ValidationResult::unsupported(),
        };

        // 3. Sicherheits-Check (Malware, übergroße Meshes, etc.)
        let security_result = self.security_check(&asset).await?;

        // 4. Qualitäts-Bewertung
        let quality_score = self.compute_quality_score(&format_result).await?;

        // 5. Ergebnis publizieren
        self.publish_validation(asset_url, &format_result, quality_score).await?;

        Ok(ValidationResult {
            valid: format_result.valid && security_result.safe,
            quality_score,
            issues: format_result.issues,
        })
    }
}
```

### 6.5 ECL-Policy für Asset-Upload

```ecl
// Policy: Game-Asset-Upload-Regeln
policy AssetUploadPolicy {
    // Basis-Requirements
    require trust >= 0.5;
    require mana >= 500;

    // Format-Validation erforderlich
    require asset.validated == true;
    require asset.validation_score >= 0.7;  // Mindestqualität

    // Lizenz erforderlich
    require asset.license in [
        "CC0", "CC-BY", "CC-BY-SA", "MIT", "Apache-2.0",
        "Unity-Asset-Store-EULA", "Custom-Commercial",
    ];

    // Sicherheits-Checks
    require !asset.flagged("malware");
    require !asset.flagged("copyright_violation");

    // Größenlimits
    match asset.type {
        "model" => require asset.polycount <= 10_000_000,
        "texture" => require asset.resolution <= 8192,
        "bundle" => require asset.size_gb <= 50,
    }

    // Kosten
    cost: {
        mana: asset.size_mb * 2.0,
        gas: 0.2,
    };

    // Rate-Limiting
    rate_limit: 20 assets per day;
}
```

---

## 7. Use Case 5: Secure Binary Vault für Enterprise

### 7.1 Vision

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   🔐 SECURE BINARY VAULT FÜR ENTERPRISE                                     ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   PROBLEM:                                                                   ║
║   - Sensible Binaries (Lizenzen, Firmware, Secrets) zentral gespeichert    ║
║   - Keine dezentrale Backup-Strategie                                       ║
║   - Komplexe Audit-Anforderungen (SOC2, ISO27001)                          ║
║   - Supply-Chain-Angriffe auf Build-Artifacts                              ║
║                                                                              ║
║   ERYNOA-LÖSUNG:                                                             ║
║   - Private Realm mit Ende-zu-Ende-Verschlüsselung                         ║
║   - Trust-basierte Zugriffskontrolle mit Audit-Trail                       ║
║   - Multi-Region-Replikation für Compliance                                ║
║   - Signierte Artifacts mit DID-basierter Provenance                       ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 7.2 Realm-Konfiguration

```ecl
// Enterprise Binary Vault Realm Blueprint
realm EnterpriseBinaryVault {
    type: VirtualRealm,
    parent: "erynoa://root",
    visibility: "private",  // Nicht öffentlich discoverable

    // Trust-Requirements (sehr streng)
    trust: {
        min_join: 0.9,            // Nur hochvertrauenswürdige Mitglieder
        min_read: 0.85,           // Lesen: hoher Trust
        min_write: 0.95,          // Schreiben: sehr hoher Trust
        min_admin: 0.99,          // Admin: maximaler Trust
    },

    // Mana-Budget (Enterprise-skaliert)
    mana: {
        total_budget: 10_000_000_000,  // 10B Mana
        per_user_daily: 10_000_000,    // 10M Mana pro User
        regeneration_rate: 0.01,       // Langsam (1% pro Stunde)
    },

    // Encryption-Settings
    encryption: {
        at_rest: "AES-256-GCM",
        in_transit: "TLS-1.3",
        key_management: "realm-kms",  // Realm-eigener KMS
        key_rotation: "90d",
    },

    // Store-Schema
    stores: {
        // Build-Artifacts
        artifacts: {
            type: "blob",
            content_addressable: true,
            max_size_gb: 100,
            encryption: "client-side",  // Client verschlüsselt vor Upload
            signature_required: true,   // Jeder Upload muss signiert sein
            retention: "7y",            // 7 Jahre Aufbewahrung
        },

        // Lizenzdateien
        licenses: {
            type: "blob",
            content_addressable: true,
            max_size_mb: 10,
            encryption: "client-side",
            access_logging: true,  // Jeder Zugriff wird geloggt
        },

        // Firmware-Images
        firmware: {
            type: "blob",
            content_addressable: true,
            max_size_gb: 10,
            encryption: "client-side",
            signature_required: true,
            versioning: true,
        },

        // Secrets (Zertifikate, Keys, etc.)
        secrets: {
            type: "blob",
            content_addressable: false,  // Keine Dedup für Secrets!
            max_size_mb: 1,
            encryption: "double-encrypted",  // Realm + Client
            access_logging: true,
            audit_required: true,
        },

        // Audit-Logs
        audit_logs: {
            type: "append-only",
            immutable: true,
            retention: "10y",
            schema: "audit-log-v1",
        },
    },

    // Governance: Enterprise-style
    governance: {
        type: "delegated",
        admin_quorum: 3,  // 3 von 5 Admins für kritische Operationen
        emergency_access: true,
        compliance_frameworks: ["SOC2", "ISO27001", "GDPR"],
    },

    // Compliance-Regionen
    replication: {
        regions: ["eu-west", "eu-central", "us-east"],
        min_replicas: 3,
        geo_restrictions: ["EU"],  // Daten nur in EU
    },
}
```

### 7.3 Realm-URL-Schema für Enterprise Vault

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   ENTERPRISE VAULT URL-SCHEMA                                                ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Build-Artifacts:                                                           ║
║   erynoa://acme-vault/store/artifacts/<project>/<version>/<artifact>        ║
║   erynoa://acme-vault/store/artifacts/backend-api/v2.3.1/linux-amd64.tar.gz ║
║                                                                              ║
║   Lizenzen:                                                                  ║
║   erynoa://acme-vault/store/licenses/<product>/<license-id>                 ║
║   erynoa://acme-vault/store/licenses/enterprise-suite/lic-2024-001         ║
║                                                                              ║
║   Firmware:                                                                  ║
║   erynoa://acme-vault/store/firmware/<device-type>/<version>                ║
║   erynoa://acme-vault/store/firmware/iot-sensor-v3/fw-1.2.0                 ║
║                                                                              ║
║   Secrets:                                                                   ║
║   erynoa://acme-vault/store/secrets/<category>/<secret-id>                  ║
║   erynoa://acme-vault/store/secrets/tls-certs/api-gateway-2024             ║
║                                                                              ║
║   Audit-Logs:                                                                ║
║   erynoa://acme-vault/store/audit_logs/<year>/<month>                       ║
║   erynoa://acme-vault/store/audit_logs/2024/06?from=2024-06-01              ║
║                                                                              ║
║   Query-Parameter:                                                           ║
║   ?audit=true      → Zugriff wird auditiert (immer für secrets)             ║
║   ?reason="..."    → Zugriffsgrund (für Compliance)                         ║
║   ?approval=<tx>   → Approval-Transaction-ID                                ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 7.4 Agent Shell Integration: Security-Audit-Bot

```rust
/// Security-Audit-Agent für Enterprise Vault
pub struct SecurityAuditAgent {
    did: Did,
    capabilities: Vec<ShellCapability>,
    realm: RealmId,
}

impl SecurityAuditAgent {
    /// Erforderliche Capabilities (sehr restriktiv)
    pub fn required_capabilities() -> Vec<ShellCapability> {
        vec![
            // Nur Log-Zugriff (KEINE Schreibrechte auf Vault!)
            ShellCapability::PathAccess {
                paths: vec![
                    PathPermission {
                        path: "/var/log/erynoa-vault/**".to_string(),
                        read: true,
                        write: false,
                        execute: false,
                        delete: false,
                    },
                ],
                user: "erynoa-audit".to_string(),  // Spezieller Audit-User
            },
            // Audit-Report-Generation
            ShellCapability::RestrictedCommands {
                allowed_commands: vec![
                    "erynoa-audit-report generate *".to_string(),
                    "erynoa-audit-report verify *".to_string(),
                ],
                blocked_args: vec![
                    "--delete".to_string(),
                    "--modify".to_string(),
                ],
                user: "erynoa-audit".to_string(),
                timeout_secs: 3600,
            },
            // Scheduled Audits
            ShellCapability::ScheduledTasks {
                namespace: "security-audit-*".to_string(),
                max_concurrent: 1,  // Nur ein Audit gleichzeitig
            },
        ]
    }

    /// Täglicher Security-Audit
    pub async fn run_daily_audit(&self) -> Result<AuditReport> {
        // 1. Zugriffs-Anomalien erkennen
        let access_anomalies = self.detect_access_anomalies().await?;

        // 2. Signaturen verifizieren
        let signature_issues = self.verify_all_signatures().await?;

        // 3. Encryption-Status prüfen
        let encryption_status = self.check_encryption_status().await?;

        // 4. Compliance-Check
        let compliance_result = self.check_compliance(["SOC2", "ISO27001"]).await?;

        // 5. Report generieren und signieren
        let report = AuditReport {
            timestamp: Utc::now(),
            access_anomalies,
            signature_issues,
            encryption_status,
            compliance_result,
        };

        // 6. Report im Audit-Log speichern
        self.store_audit_report(&report).await?;

        // 7. Bei kritischen Findings: Alert
        if report.has_critical_findings() {
            self.alert_security_team(&report).await?;
        }

        Ok(report)
    }
}
```

### 7.5 ECL-Policy für Vault-Zugriff

```ecl
// Policy: Enterprise Vault Zugriffs-Regeln
policy VaultAccessPolicy {
    // Basis-Requirements (sehr streng)
    require trust >= 0.85;
    require membership.status == "active";
    require membership.mfa_enabled == true;

    // Secrets erfordern Approval
    if resource.type == "secrets" {
        require approval.exists();
        require approval.quorum >= 2;  // 2 von 3 Admins
        require approval.age < "1h";   // Approval max 1h alt
        require access.reason.length >= 20;  // Grund erforderlich
    }

    // Firmware-Downloads brauchen Device-Context
    if resource.type == "firmware" {
        require device.registered == true;
        require device.attestation.valid == true;
    }

    // Geo-Restriction
    require access.ip.geo in ["DE", "AT", "CH", "FR", "NL", "BE"];

    // Zeitfenster für kritische Zugriffe
    if resource.sensitivity == "critical" {
        require time.hour in [8, 9, 10, 11, 12, 13, 14, 15, 16, 17];  // Business Hours
        require time.weekday in [1, 2, 3, 4, 5];  // Mo-Fr
    }

    // Audit-Trail
    audit: {
        log_access: true,
        log_content_hash: true,
        notify_on_access: ["security-team@acme.com"],
    };

    // Kosten (Premium für Enterprise)
    cost: {
        mana: resource.size_mb * 10.0,
        gas: 1.0,  // Höhere Gas-Kosten für Permanenz
    };
}
```

---

## 8. Use Case 6: Wissenschaftliches Data-Sharing

### 8.1 Vision

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   🔬 DEZENTRALES WISSENSCHAFTLICHES DATA-SHARING                            ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   PROBLEM:                                                                   ║
║   - Forschungsdaten in Silos (Uni-Server, Cloud, USB-Sticks)               ║
║   - Keine einheitliche Provenance/Citation                                  ║
║   - Reproduzierbarkeit schwierig ohne Daten-Zugang                         ║
║   - Kein Anreiz, Daten zu teilen                                           ║
║                                                                              ║
║   ERYNOA-LÖSUNG:                                                             ║
║   - Realm für wissenschaftliche Daten mit DOI-Integration                  ║
║   - Versionierte Datasets mit vollständiger Lineage                        ║
║   - Reputation-basierte Incentives für Data-Sharing                        ║
║   - Dezentrale Peer-Review für Datenqualität                               ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 8.2 Realm-Konfiguration

```ecl
// Scientific Data Sharing Realm Blueprint
realm ScienceDataHub {
    type: VirtualRealm,
    parent: "erynoa://root",

    // Trust-Requirements
    trust: {
        min_join: 0.3,            // Lesen: niedriger Trust
        min_download: 0.4,        // Download: etwas höher
        min_upload: 0.6,          // Upload: mittlerer Trust
        min_peer_review: 0.8,     // Peer-Review: hoher Trust
        min_curate: 0.9,          // Kuratierung: sehr hoher Trust
    },

    // Mana-Budget (großzügig für Open Science)
    mana: {
        total_budget: 1_000_000_000,
        per_user_daily: 500_000,
        regeneration_rate: 0.1,
        // Bonus für verifizierte Institutionen
        institution_multiplier: 5.0,
    },

    // Store-Schema
    stores: {
        // Rohdaten
        raw_data: {
            type: "blob",
            content_addressable: true,
            max_size_tb: 10,  // Große Datasets!
            compression: "zstd-3",
            chunking: {
                min_size_mb: 256,
                max_size_mb: 1024,
            },
            immutable: true,  // Rohdaten unveränderlich
        },

        // Prozessierte Daten
        processed_data: {
            type: "blob",
            content_addressable: true,
            max_size_tb: 5,
            compression: "zstd-10",
            versioning: true,
            requires_raw_link: true,  // Muss Rohdaten referenzieren
        },

        // Metadaten (Dublin Core + DataCite)
        metadata: {
            type: "json",
            schema: "datacite-v4.4",
            fields: [
                "title", "creators", "publisher", "year",
                "subjects", "description", "methodology",
                "related_identifiers", "funding_references",
            ],
        },

        // Code-Notebooks (für Reproduzierbarkeit)
        notebooks: {
            type: "blob",
            content_addressable: true,
            max_size_mb: 100,
            formats: ["ipynb", "rmd", "qmd"],
            execution_environment: true,  // Binder-ähnlich
        },

        // Peer-Reviews
        reviews: {
            type: "json",
            schema: "peer-review-v1",
            signed: true,
            immutable_after_publish: true,
        },

        // Citations
        citations: {
            type: "graph",
            schema: "citation-graph-v1",
            bidirectional: true,  // Zitiert & Zitiert-von
        },
    },

    // Governance: Academic Community
    governance: {
        type: "reputation",
        peer_review_required: true,
        citation_rewards: true,
        institution_verification: true,
        data_use_agreements: true,
    },

    // DOI-Integration
    identifiers: {
        doi_prefix: "10.erynoa",
        auto_register: true,
        orcid_integration: true,
    },
}
```

### 8.3 Realm-URL-Schema für Wissenschaftliche Daten

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   SCIENCE DATA URL-SCHEMA                                                    ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Rohdaten:                                                                  ║
║   erynoa://science-hub/store/raw_data/<dataset-id>/<version>                ║
║   erynoa://science-hub/store/raw_data/climate-obs-2024/v1.0                 ║
║                                                                              ║
║   Prozessierte Daten:                                                        ║
║   erynoa://science-hub/store/processed_data/<dataset-id>/<version>          ║
║   erynoa://science-hub/store/processed_data/climate-analysis/v2.1           ║
║   erynoa://science-hub/store/processed_data/climate-analysis/v2.1           ║
║       ?raw_link=erynoa://science-hub/store/raw_data/climate-obs-2024/v1.0   ║
║                                                                              ║
║   Metadaten:                                                                 ║
║   erynoa://science-hub/store/metadata/<dataset-id>                          ║
║   erynoa://science-hub/store/metadata/climate-obs-2024                      ║
║                                                                              ║
║   Notebooks:                                                                 ║
║   erynoa://science-hub/store/notebooks/<dataset-id>/<notebook-name>         ║
║   erynoa://science-hub/store/notebooks/climate-obs-2024/analysis.ipynb      ║
║   erynoa://science-hub/store/notebooks/climate-obs-2024/analysis.ipynb?run  ║
║                                                                              ║
║   Reviews:                                                                   ║
║   erynoa://science-hub/store/reviews/<dataset-id>/<review-id>               ║
║                                                                              ║
║   DOI-Mapping:                                                               ║
║   https://doi.org/10.erynoa/climate-obs-2024.v1.0                          ║
║   → erynoa://science-hub/store/raw_data/climate-obs-2024/v1.0               ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 8.4 Agent Shell Integration: Reproducibility-Bot

```rust
/// Reproducibility-Agent für wissenschaftliche Daten
pub struct ReproducibilityAgent {
    did: Did,
    capabilities: Vec<ShellCapability>,
    realm: RealmId,
}

impl ReproducibilityAgent {
    /// Erforderliche Capabilities
    pub fn required_capabilities() -> Vec<ShellCapability> {
        vec![
            // Jupyter/RStudio Container
            ShellCapability::ContainerControl {
                runtime: ContainerRuntime::Docker,
                containers: vec![
                    "erynoa-jupyter-*".to_string(),
                    "erynoa-rstudio-*".to_string(),
                    "erynoa-binder-*".to_string(),
                ],
                actions: vec![
                    ContainerAction::Start,
                    ContainerAction::Stop,
                    ContainerAction::Exec,
                    ContainerAction::Logs,
                ],
            },
            // Conda/Pip für Environment-Setup
            ShellCapability::PackageManagement {
                manager: PackageManager::Conda,
                allowed_packages: vec!["*".to_string()],  // Wissenschaftliche Pakete
                update_only: false,
            },
            // Workspace für Reproduktion
            ShellCapability::PathAccess {
                paths: vec![
                    PathPermission {
                        path: "/var/lib/erynoa/reproducibility/**".to_string(),
                        read: true,
                        write: true,
                        execute: true,
                        delete: true,
                    },
                ],
                user: "erynoa-science".to_string(),
            },
            // Scheduled Reproducibility-Checks
            ShellCapability::ScheduledTasks {
                namespace: "reproducibility-*".to_string(),
                max_concurrent: 5,
            },
        ]
    }

    /// Reproduzierbarkeits-Check für Dataset
    pub async fn check_reproducibility(&self, dataset_url: &str) -> Result<ReproducibilityReport> {
        // 1. Dataset + Notebooks herunterladen
        let dataset = self.download_dataset(dataset_url).await?;
        let notebooks = self.download_notebooks(dataset_url).await?;

        // 2. Environment aus requirements.txt/environment.yml erstellen
        let env = self.setup_environment(&dataset).await?;

        // 3. Alle Notebooks ausführen
        let execution_results = self.execute_notebooks(&env, &notebooks).await?;

        // 4. Output vergleichen mit gespeicherten Ergebnissen
        let comparison = self.compare_outputs(&execution_results, &dataset).await?;

        // 5. Report generieren
        let report = ReproducibilityReport {
            dataset_url: dataset_url.to_string(),
            reproducible: comparison.all_match(),
            execution_results,
            comparison,
            environment_hash: env.hash(),
            timestamp: Utc::now(),
        };

        // 6. Report publizieren (erhöht Trust des Datasets!)
        self.publish_report(dataset_url, &report).await?;

        Ok(report)
    }
}
```

### 8.5 ECL-Policy für Data-Upload

```ecl
// Policy: Wissenschaftliche Daten-Upload-Regeln
policy ScienceDataPolicy {
    // Basis-Requirements
    require trust >= 0.6;
    require mana >= 10_000;

    // Metadaten erforderlich
    require dataset.metadata.complete([
        "title", "creators", "description", "methodology",
    ]);

    // ORCID-Verifikation für Creator
    require any(dataset.creators, creator => creator.orcid.verified);

    // Data-Use-Agreement
    require dataset.license in [
        "CC0", "CC-BY", "CC-BY-SA", "CC-BY-NC",
        "ODC-PDDL", "ODC-BY", "ODC-ODbL",
    ];

    // Für prozessierte Daten: Lineage erforderlich
    if dataset.type == "processed" {
        require dataset.raw_data_link.exists();
        require dataset.processing_notebook.exists();
    }

    // Peer-Review für Kuratierung
    if dataset.request_curation {
        require governance.peer_review_count >= 2;
        require governance.peer_review_score >= 0.8;
    }

    // Institution-Bonus
    if uploader.institution.verified {
        mana_discount: 0.5;  // 50% Rabatt
        trust_bonus: 0.1;
    }

    // Kosten (großzügig für Open Science)
    cost: {
        mana: dataset.size_gb * 10.0,  // Niedrig
        gas: 0.5,
    };

    // Rate-Limiting (großzügig)
    rate_limit: 100 datasets per day;
}
```

### 8.6 Vollständiger Workflow

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   SCIENTIFIC DATA SHARING WORKFLOW                                           ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   1. DATEN VORBEREITEN                                                       ║
║      └── erynoa data init climate-observation-2024                          ║
║          ├── Title: "Global Climate Observations 2024"                      ║
║          ├── Creators: [Dr. A (ORCID: 0000-...), Prof. B (ORCID: ...)]     ║
║          ├── Methodology: "Satellite + Ground Station Fusion"               ║
║          ├── License: CC-BY-4.0                                             ║
║          └── Keywords: ["climate", "observation", "satellite"]              ║
║                                                                              ║
║   2. UPLOAD                                                                  ║
║      └── erynoa data push climate-observation-2024                          ║
║          ├── raw_data/temperature.nc (50 GB)                                ║
║          │   ├── Chunk: 64 Chunks à 800 MB                                  ║
║          │   ├── Blake3: 0xabc123...                                        ║
║          │   ├── P2P-Sync: 10+ Peers (wissenschaftl. Institutionen)        ║
║          │   └── Mana: 500,000 (50 GB × 10,000)                            ║
║          ├── notebooks/analysis.ipynb                                       ║
║          └── requirements.txt                                                ║
║                                                                              ║
║   3. DOI-REGISTRIERUNG                                                       ║
║      └── Automatisch: 10.erynoa/climate-obs-2024.v1.0                       ║
║          ├── DataCite-Metadaten                                             ║
║          └── ORCID-Linking                                                   ║
║                                                                              ║
║   4. PEER-REVIEW (optional für Kuratierung)                                 ║
║      └── 2 Reviewer mit Trust > 0.8                                         ║
║          ├── Datenqualität: ✓                                               ║
║          ├── Dokumentation: ✓                                               ║
║          └── Reproduzierbarkeit: ✓ (Agent-verifiziert)                      ║
║                                                                              ║
║   5. NUTZUNG                                                                 ║
║      └── erynoa data pull 10.erynoa/climate-obs-2024.v1.0                  ║
║          ├── P2P-Download (BitSwap)                                         ║
║          ├── Citation-Event emittiert                                       ║
║          └── Creator erhält Reputation-Bonus                                ║
║                                                                              ║
║   6. ZITATION                                                                ║
║      └── Automatisches Citation-Tracking                                    ║
║          ├── Citation-Graph aktualisiert                                    ║
║          ├── h-Index für Daten-Creator berechnet                            ║
║          └── Impact-Metrics für Institution                                 ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 9. Querschnittsthemen

### 9.1 Globale Deduplizierung

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   GLOBALE DEDUPLIZIERUNG ÜBER ALLE USE CASES                                 ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Das Erynoa-Nervensystem erkennt identische Blobs realm-übergreifend:      ║
║                                                                              ║
║   Beispiel:                                                                  ║
║   ├── erynoa://docker-registry/store/layers/sha256:abc123...               ║
║   ├── erynoa://ai-models/store/base_models/llama-3/layer-0                  ║
║   └── erynoa://science-hub/store/raw_data/llama-training/checkpoint-0      ║
║                                                                              ║
║   → Alle 3 URLs zeigen auf denselben physischen Blob!                       ║
║   → Speicher: 1× statt 3×                                                   ║
║   → Bandbreite: Einmal gepullt, überall verfügbar                          ║
║                                                                              ║
║   ABER: Zugriffskontrolle bleibt realm-spezifisch!                         ║
║   → docker-registry: public read                                            ║
║   → ai-models: Trust >= 0.2                                                 ║
║   → science-hub: Trust >= 0.4 + institution verified                        ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 9.2 P2P-Sync-Strategien

```rust
/// P2P-Sync-Konfiguration für verschiedene Use Cases
pub struct SyncStrategy {
    /// Minimum Peers für Verfügbarkeit
    min_peers: u32,
    /// Prioritäts-Regionen
    priority_regions: Vec<String>,
    /// Sync-Protokoll
    protocol: SyncProtocol,
    /// Bandwidth-Budget
    bandwidth_budget_mbps: u32,
}

impl SyncStrategy {
    /// Docker-Registry: Schnelle Verfügbarkeit
    pub fn docker_registry() -> Self {
        Self {
            min_peers: 5,
            priority_regions: vec!["eu-*".to_string(), "us-*".to_string()],
            protocol: SyncProtocol::BitSwap,  // Aggressiv
            bandwidth_budget_mbps: 1000,
        }
    }

    /// AI-Models: Hohe Bandbreite, weniger Peers
    pub fn ai_models() -> Self {
        Self {
            min_peers: 3,
            priority_regions: vec!["gpu-enabled".to_string()],
            protocol: SyncProtocol::Streaming,  // Sequentiell
            bandwidth_budget_mbps: 10000,  // 10 Gbps für große Modelle
        }
    }

    /// Enterprise-Vault: Geo-restricted, hohe Redundanz
    pub fn enterprise_vault() -> Self {
        Self {
            min_peers: 7,
            priority_regions: vec!["eu-west".to_string(), "eu-central".to_string()],
            protocol: SyncProtocol::Encrypted,  // E2E
            bandwidth_budget_mbps: 100,  // Niedrig (Sicherheit > Speed)
        }
    }

    /// Science-Hub: Institutional Peers bevorzugt
    pub fn science_hub() -> Self {
        Self {
            min_peers: 10,
            priority_regions: vec!["university-*".to_string(), "research-*".to_string()],
            protocol: SyncProtocol::BitSwap,
            bandwidth_budget_mbps: 5000,
        }
    }
}
```

### 9.3 Mana-Regeneration pro Use Case

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   MANA-REGENERATION NACH USE CASE                                            ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Use Case              Daily Budget    Regen/h    Begründung                ║
║   ──────────────────────────────────────────────────────────────────────────  ║
║   Docker Registry       10,000          10%        Häufige kleine Ops        ║
║   AI Models             1,000,000       5%         Seltene große Uploads     ║
║   Social Media          50,000          20%        Viele kleine Interaktionen║
║   Game Assets           100,000         15%        Mittlere Frequenz         ║
║   Enterprise Vault      10,000,000      1%         Kritisch, langsam         ║
║   Science Hub           500,000         10%        Mittlere Frequenz         ║
║                                                                              ║
║   FORMEL für Regeneration:                                                   ║
║   mana_new = min(mana_current + (budget × regen_rate), budget)              ║
║                                                                              ║
║   Beispiel (Docker Registry, 50% verbraucht):                               ║
║   mana_new = min(5,000 + (10,000 × 0.10), 10,000) = 6,000                   ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 9.4 Trust-Schwellwerte Übersicht

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   TRUST-SCHWELLWERTE NACH USE CASE UND OPERATION                            ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Operation           Docker  AI    Social  Games  Vault  Science           ║
║   ──────────────────────────────────────────────────────────────────────────  ║
║   Join/Browse         0.3     0.2   0.0     0.0    0.9    0.3                ║
║   Read/Download       0.3     0.2   0.0     0.2    0.85   0.4                ║
║   Write/Upload        0.6     0.7   0.4     0.5    0.95   0.6                ║
║   Delete              0.8     0.8   0.7     0.7    N/A    N/A                ║
║   Moderate/Curate     0.9     0.85  0.8     0.8    N/A    0.9                ║
║   Admin               0.95    0.95  0.95    0.95   0.99   0.95               ║
║                                                                              ║
║   LEGENDE:                                                                   ║
║   0.0-0.3  → Öffentlich/Sehr niedrig                                        ║
║   0.4-0.6  → Mittel (normale Mitglieder)                                    ║
║   0.7-0.8  → Hoch (erfahrene/vertrauenswürdige)                            ║
║   0.9+     → Sehr hoch (Admin/Kurator)                                      ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 10. Zusammenfassung

### 10.1 Kernerkenntnisse

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   PLUTO REALMS ALS UNIVERSELLER BINARY-STORAGE                               ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   1. ARCHITEKTUR                                                             ║
║      └── Content-Addressed (Blake3) + Chunked + Compressed + P2P            ║
║                                                                              ║
║   2. ADRESSIERUNG                                                            ║
║      └── erynoa://<realm>/<resource-type>/<path>[?params]                   ║
║                                                                              ║
║   3. ZUGRIFFSKONTROLLE                                                       ║
║      └── Trust-basiert + ECL-Policies + Governance                          ║
║                                                                              ║
║   4. AUTOMATION                                                              ║
║      └── Agent-Shell mit Capability-based Security                          ║
║                                                                              ║
║   5. KOSTEN                                                                  ║
║      └── Mana (regenerierbar) + Gas (permanent)                             ║
║                                                                              ║
║   6. USE CASES                                                               ║
║      ├── 🐳 Docker Registry: Layer-Dedup, Trust-Verification                ║
║      ├── 🤖 AI Models: Delta-Weights, Peer-Review, Lineage                  ║
║      ├── 🎬 Social Media: Transcoding, Moderation, Creator-Economy          ║
║      ├── 🎮 Game Assets: Cross-Game, Quality-Tiers, Validation              ║
║      ├── 🔐 Enterprise Vault: E2E-Encryption, Audit, Compliance             ║
║      └── 🔬 Science Hub: DOI, Reproducibility, Citation-Tracking            ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 10.2 Nächste Schritte

| Phase       | Fokus                                     | Zeitrahmen  |
| ----------- | ----------------------------------------- | ----------- |
| **Phase 1** | Blob-Store-Core + CAS + Chunking          | Woche 1-2   |
| **Phase 2** | Realm-URL-Integration + Resolution Engine | Woche 3-4   |
| **Phase 3** | Trust/Mana-Integration + ECL-Policies     | Woche 5-6   |
| **Phase 4** | Agent-Shell-Capabilities für Blob-Ops     | Woche 7-8   |
| **Phase 5** | P2P-Sync (Gossip + BitSwap)               | Woche 9-10  |
| **Phase 6** | Use-Case-spezifische Blueprints           | Woche 11-12 |
| **Phase 7** | Performance-Optimierung + Monitoring      | Woche 13-14 |

---

## Anhang A: Axiom-Referenzen

| Axiom                            | Anwendung in Use Cases                       |
| -------------------------------- | -------------------------------------------- |
| **Κ1** (Monotone Regelvererbung) | Realm-Schema-Vererbung, Policy-Inheritance   |
| **Κ8** (Capability-Kontrolle)    | Agent-Shell-Capabilities für alle Use Cases  |
| **Κ10** (Event-Integrität)       | Blob-Upload-Events, Audit-Trails             |
| **Κ21** (Quadratische Fairness)  | Social Media Governance, Science Peer-Review |
| **Κ24** (Realm-lokaler Trust)    | Use-Case-spezifische Trust-Schwellwerte      |
| **Κ26** (URL-Schema)             | Alle Realm-URLs (erynoa://...)               |
| **Κ27** (Resource-Resolution)    | URL-zu-Blob-Mapping                          |

---

## Anhang B: Glossar

| Begriff   | Definition                                                      |
| --------- | --------------------------------------------------------------- |
| **Blob**  | Binary Large Object, Content-addressierter Datenblock           |
| **CAS**   | Content-Addressable Storage, Speicher basierend auf Inhaltshash |
| **Chunk** | Teilstück eines großen Blobs (4-64 MB)                          |
| **DID**   | Decentralized Identifier, dezentrale Identität                  |
| **ECL**   | Erynoa Configuration Language, Policy-Sprache                   |
| **Gas**   | Unwiderrufliche Systemressource für permanente Operationen      |
| **Mana**  | Regenerierbare Systemressource für tägliche Operationen         |
| **Realm** | Souveräne Einheit im Erynoa-Nervensystem                        |
| **Trust** | Emergenter Vertrauenswert (0.0 - 1.0)                           |
