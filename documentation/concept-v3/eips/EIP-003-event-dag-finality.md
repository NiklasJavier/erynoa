# EIP-003: Event-DAG & Finality Specification

> **EIP:** 003
> **Titel:** Event-DAG & Finality Specification
> **Status:** Draft
> **Version:** 0.2
> **Typ:** Standard
> **Ebene:** E1 (Fundament)
> **Erstellt:** Januar 2026
> **Aktualisiert:** Januar 2026
> **Abhängigkeiten:** EIP-001 (DID:erynoa), EIP-002 (Trust Vector 6D)

---

## Abstract

Diese Spezifikation definiert den Event-DAG (Directed Acyclic Graph) und das Finality-System für das Erynoa-Protokoll. Der Event-DAG ist die kausale Datenstruktur, die alle Zustandsänderungen im Netzwerk aufzeichnet. Das Finality-System definiert, wann und wie Events als "endgültig" (unveränderbar) gelten.

Kernkonzepte:
- **Content-Addressable Events** – Events werden durch ihren Hash identifiziert
- **Kausale Ordnung** – Events referenzieren ihre Vorgänger
- **Progressive Finality** – Events durchlaufen Finality-Levels
- **Witness-Konsens** – Unabhängige Zeugen mit Hardware-Attestierung bestätigen Events
- **Data Availability** – Witnesses garantieren Datenverfügbarkeit
- **Trust-basiertes Rate-Limiting** – Spam-Schutz via Trust-Score
- **Merkle Mountain Range** – Inkrementelle Merkle-Berechnung
- **Multi-Chain-Anchoring** – Endgültige Verankerung auf Blockchains

---

## Motivation

Ein dezentrales Vertrauenssystem benötigt eine unveränderliche Historie aller Interaktionen. Der Event-DAG erfüllt diese Anforderung durch:

1. **Kausale Ordnung** – "Was kam vor was?" ist eindeutig bestimmbar
2. **Dezentrale Erzeugung** – Jeder Agent kann Events erstellen
3. **Parallelität** – Unabhängige Events können gleichzeitig existieren
4. **Unveränderlichkeit** – Einmal finalisierte Events sind permanent
5. **Verifizierbarkeit** – Jeder kann die Integrität prüfen
6. **Skalierbarkeit** – DAG-Struktur ermöglicht hohen Durchsatz

---

## Spezifikation

### 1. Event-Struktur

#### 1.1 Event-Definition

Ein Event ist die atomare Einheit der Zustandsänderung im Erynoa-Netzwerk.

```rust
/// Ein Event im Erynoa Event-DAG
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    /// Eindeutige ID = SHA-256(canonical_serialize(self))
    pub id: EventId,
    
    /// Event-Version (für zukünftige Upgrades)
    pub version: u8,
    
    /// Typ des Events
    pub event_type: EventType,
    
    /// DID des Actors, der das Event erstellt hat
    pub actor: DID,
    
    /// Unix-Timestamp in Millisekunden
    pub timestamp: u64,
    
    /// Referenzen auf Parent-Events (kausale Vorgänger)
    pub parents: Vec<EventId>,
    
    /// Event-spezifische Payload
    pub payload: EventPayload,
    
    /// Signatur des Actors über den Event-Body
    pub signature: Signature,
    
    /// Optional: Realm-Kontext
    pub realm: Option<DID>,
}

/// Event-ID ist ein 32-Byte SHA-256 Hash
pub type EventId = [u8; 32];
```

#### 1.2 Event-Typen

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventType {
    // === Identität (E1) ===
    IdentityCreate,
    IdentityUpdate,
    IdentityDeactivate,
    IdentityRecover,
    
    // === Trust (E2) ===
    TrustAttestation,
    TrustChallenge,
    TrustResponse,
    
    // === Transaktion (E3) ===
    TransactionSeek,
    TransactionPropose,
    TransactionAgree,
    TransactionStream,
    TransactionClose,
    TransactionAbort,
    TransactionDispute,
    
    // === Assets (E4) ===
    AssetCreate,
    AssetTransfer,
    AssetBurn,
    
    // === Credentials (E4) ===
    CredentialIssue,
    CredentialPresent,
    CredentialRevoke,
    
    // === Realm (E4) ===
    RealmCreate,
    RealmJoin,
    RealmLeave,
    RealmUpdate,
    
    // === Governance (E5/E6) ===
    GovernancePropose,
    GovernanceVote,
    GovernanceExecute,
    
    // === Witnessing ===
    WitnessAttestation,
    WitnessChallenge,
    
    // === Anomalien (E6) ===
    AnomalyReport,
    AnomalyConfirm,
    AnomalyReject,
    
    // === System ===
    Anchor,
    Checkpoint,
    
    // === Custom ===
    Custom(String),
}
```

#### 1.3 Event-Payload

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventPayload {
    // Identity Events
    IdentityCreate {
        did_document: DIDDocument,
    },
    IdentityUpdate {
        did: DID,
        changes: DIDDocumentPatch,
        previous_version: String,
    },
    
    // Trust Events
    TrustAttestation {
        subject: DID,
        dimension: TrustDimension,
        value: f64,
        evidence: Option<String>,
        context: Option<String>,
    },
    
    // Transaction Events
    TransactionPropose {
        transaction_id: TransactionId,
        counterparty: DID,
        terms: Terms,
        blueprint: Option<BlueprintRef>,
        ricardian_hash: Option<[u8; 32]>,
    },
    TransactionAgree {
        transaction_id: TransactionId,
        escrow: Option<EscrowConfig>,
    },
    TransactionStream {
        transaction_id: TransactionId,
        progress: f64,
        milestone: Option<String>,
        payment: Option<Amount>,
    },
    TransactionClose {
        transaction_id: TransactionId,
        outcome: TransactionOutcome,
        rating: Option<Rating>,
    },
    
    // Asset Events
    AssetTransfer {
        asset_id: AssetId,
        from: DID,
        to: DID,
        amount: Amount,
    },
    
    // Credential Events
    CredentialIssue {
        credential: VerifiableCredential,
    },
    CredentialRevoke {
        credential_id: CredentialId,
        reason: String,
    },
    
    // Witness Events
    WitnessAttestation {
        event_id: EventId,
        valid: bool,
        proof: WitnessProof,
    },
    
    // Anchor Events
    Anchor {
        chain: String,
        tx_id: String,
        block: u64,
        merkle_root: [u8; 32],
        events: Vec<EventId>,
    },
    
    // Custom
    Custom(serde_json::Value),
}
```

#### 1.4 Kanonische Serialisierung

Für deterministische Hashes wird kanonische Serialisierung verwendet:

```rust
fn canonical_serialize(event: &Event) -> Vec<u8> {
    // 1. Alle Felder in definierter Reihenfolge
    // 2. Maps nach Schlüssel sortiert
    // 3. Keine optionalen Felder wenn None
    // 4. CBOR-Encoding (RFC 8949)
    
    let mut encoder = CborEncoder::new();
    encoder.write_u8(event.version);
    encoder.write_string(&event.event_type.to_string());
    encoder.write_string(&event.actor.to_string());
    encoder.write_u64(event.timestamp);
    
    // Parents sortiert
    let mut parents = event.parents.clone();
    parents.sort();
    encoder.write_array(&parents);
    
    encoder.write_value(&event.payload);
    
    if let Some(realm) = &event.realm {
        encoder.write_string(&realm.to_string());
    }
    
    encoder.finish()
}

fn compute_event_id(event: &Event) -> EventId {
    sha256(&canonical_serialize(event))
}
```

### 2. DAG-Struktur

#### 2.1 Grundprinzipien

Der Event-DAG ist ein gerichteter azyklischer Graph mit folgenden Eigenschaften:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          EVENT-DAG STRUKTUR                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│    Genesis                                                              │
│       │                                                                 │
│       ▼                                                                 │
│    [E001] ─────────────────────────────┐                               │
│       │                                │                               │
│       ▼                                ▼                               │
│    [E002]                           [E003]                             │
│       │                                │                               │
│       ├────────────────┬───────────────┤                               │
│       ▼                ▼               ▼                               │
│    [E004]           [E005]          [E006]                             │
│       │                │               │                               │
│       └────────────────┼───────────────┘                               │
│                        ▼                                               │
│                     [E007] ◄── Merge-Event                             │
│                        │                                               │
│                        ▼                                               │
│                     [E008]                                             │
│                                                                         │
│   LEGENDE:                                                              │
│   [Exxx] = Event mit ID xxx                                            │
│   ─────► = Parent-Referenz (kausale Abhängigkeit)                      │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 2.2 DAG-Regeln

| Regel | Beschreibung |
|-------|--------------|
| **R1: Azyklizität** | Keine Zyklen: Wenn E₁ → E₂, dann ¬(E₂ → E₁) |
| **R2: Genesis** | Genau ein Event ohne Parents (Genesis) |
| **R3: Erreichbarkeit** | Jedes Event ist von Genesis erreichbar |
| **R4: Parent-Existenz** | Alle referenzierten Parents müssen existieren |
| **R5: Parent-Finalität** | Parents müssen mindestens "Attested" sein |
| **R6: Zeitordnung** | timestamp(child) > max(timestamp(parents)) |
| **R7: Actor-Kausalität** | Eigene Events sind transitiv verknüpft |

#### 2.3 Datenstruktur

```rust
pub struct EventDAG {
    /// Event-Storage (Content-Addressable)
    events: HashMap<EventId, Event>,
    
    /// Forward-Edges: Parent → Children
    children: HashMap<EventId, HashSet<EventId>>,
    
    /// Finality-Status pro Event
    finality: HashMap<EventId, FinalityLevel>,
    
    /// Genesis-Event
    genesis: EventId,
    
    /// Aktuelle "Tips" (Events ohne Kinder)
    tips: HashSet<EventId>,
    
    /// Events pro Actor (für schnelle Abfrage)
    actor_events: HashMap<DID, Vec<EventId>>,
    
    /// Anchor-Index
    anchors: HashMap<EventId, Vec<AnchorInfo>>,
}

impl EventDAG {
    /// Neues Event hinzufügen
    pub fn add_event(&mut self, event: Event) -> Result<EventId, DAGError> {
        // 1. ID berechnen
        let id = compute_event_id(&event);
        
        // 2. Validierung
        self.validate_event(&event)?;
        
        // 3. DAG-Regeln prüfen
        self.check_dag_rules(&event)?;
        
        // 4. Einfügen
        self.events.insert(id, event.clone());
        
        // 5. Edges aktualisieren
        for parent_id in &event.parents {
            self.children.entry(*parent_id)
                .or_default()
                .insert(id);
            self.tips.remove(parent_id);
        }
        
        self.tips.insert(id);
        
        // 6. Actor-Index aktualisieren
        self.actor_events.entry(event.actor.clone())
            .or_default()
            .push(id);
        
        // 7. Initial Finality = Pending
        self.finality.insert(id, FinalityLevel::Pending);
        
        Ok(id)
    }
    
    /// Kausale Ordnung prüfen
    pub fn happened_before(&self, e1: EventId, e2: EventId) -> bool {
        // BFS von e2 rückwärts
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(e2);
        
        while let Some(current) = queue.pop_front() {
            if current == e1 {
                return true;
            }
            if visited.insert(current) {
                if let Some(event) = self.events.get(&current) {
                    for parent in &event.parents {
                        queue.push_back(*parent);
                    }
                }
            }
        }
        false
    }
    
    /// Gemeinsamer Vorfahre (für Merge)
    pub fn common_ancestor(&self, e1: EventId, e2: EventId) -> Option<EventId> {
        let ancestors_1 = self.all_ancestors(e1);
        let ancestors_2 = self.all_ancestors(e2);
        
        // Jüngster gemeinsamer Vorfahre
        ancestors_1.intersection(&ancestors_2)
            .max_by_key(|id| self.events.get(*id).map(|e| e.timestamp))
            .copied()
    }
}
```

### 3. Finality-System

#### 3.1 Finality-Levels

Events durchlaufen progressive Finality-Levels:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        FINALITY-LEVELS                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   PENDING ──────► ATTESTED ──────► ANCHORED ──────► FINAL              │
│      │                │                 │               │               │
│      │                │                 │               │               │
│   Signiert         k Witnesses      On-Chain       Tiefe ≥ d           │
│   vom Actor        bestätigt        verankert      erreicht            │
│                                                                         │
│   ──────────────────────────────────────────────────────────────────   │
│                                                                         │
│   EIGENSCHAFTEN:                                                        │
│                                                                         │
│   Level      Reversibel  Trust-Impact  Typische Latenz                 │
│   ─────────────────────────────────────────────────────────────────    │
│   PENDING       Ja          0%           0-1s                          │
│   ATTESTED      Schwer      50%          1-10s                         │
│   ANCHORED      Nein*       90%          10-60s                        │
│   FINAL         Nein        100%         1-10min                       │
│                                                                         │
│   *Nur durch Chain-Reorganisation (sehr unwahrscheinlich)              │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 3.2 Finality-Level Definition

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FinalityLevel {
    /// Event signiert, aber noch keine Witnesses
    Pending = 0,
    
    /// Von k unabhängigen Witnesses bestätigt
    Attested = 1,
    
    /// Auf Primary Chain (IOTA) verankert
    Anchored = 2,
    
    /// Tiefe d auf Chain erreicht (effektiv irreversibel)
    Final = 3,
}

#[derive(Clone, Debug)]
pub struct FinalityState {
    pub level: FinalityLevel,
    
    /// Witnesses, die dieses Event bestätigt haben
    pub witnesses: Vec<WitnessAttestation>,
    
    /// Anzahl unabhängiger Witnesses
    pub witness_count: u32,
    
    /// Anchor-Informationen (wenn anchored/final)
    pub anchor: Option<AnchorInfo>,
    
    /// Tiefe seit Verankerung (Blocks)
    pub depth: Option<u64>,
    
    /// Zeitpunkt des Level-Wechsels
    pub level_timestamps: HashMap<FinalityLevel, u64>,
}
```

#### 3.3 Level-Transitionen

```rust
impl EventDAG {
    /// Prüft und aktualisiert Finality-Level
    pub fn update_finality(&mut self, event_id: EventId) -> Result<FinalityLevel, FinalityError> {
        let current = self.finality.get(&event_id)
            .ok_or(FinalityError::EventNotFound)?;
        
        match current {
            FinalityLevel::Pending => {
                // Check: Genug Witnesses?
                let witnesses = self.get_witnesses(event_id);
                let required = self.get_required_witnesses(event_id);
                
                if self.count_independent_witnesses(&witnesses) >= required {
                    self.finality.insert(event_id, FinalityLevel::Attested);
                    return Ok(FinalityLevel::Attested);
                }
            }
            
            FinalityLevel::Attested => {
                // Check: Auf Chain verankert?
                if let Some(anchor) = self.anchors.get(&event_id) {
                    if anchor.iter().any(|a| a.chain == "iota") {
                        self.finality.insert(event_id, FinalityLevel::Anchored);
                        return Ok(FinalityLevel::Anchored);
                    }
                }
            }
            
            FinalityLevel::Anchored => {
                // Check: Tiefe erreicht?
                if let Some(anchor) = self.anchors.get(&event_id) {
                    if let Some(iota_anchor) = anchor.iter().find(|a| a.chain == "iota") {
                        let current_block = self.get_current_block("iota")?;
                        let depth = current_block - iota_anchor.block;
                        
                        if depth >= FINALITY_DEPTH {
                            self.finality.insert(event_id, FinalityLevel::Final);
                            return Ok(FinalityLevel::Final);
                        }
                    }
                }
            }
            
            FinalityLevel::Final => {
                // Bereits final, nichts zu tun
            }
        }
        
        Ok(*current)
    }
}

/// Konfigurierbare Parameter
const FINALITY_DEPTH: u64 = 10;  // Blocks auf IOTA für Finality
```

### 4. Witness-System

#### 4.1 Witness-Anforderungen

Die Anzahl erforderlicher Witnesses hängt vom Event-Typ und LoD ab:

```rust
fn get_required_witnesses(event: &Event, lod: LevelOfDetail) -> u32 {
    match lod {
        LevelOfDetail::Minimal => 0,   // Nur Signatur
        LevelOfDetail::Basic => 1,     // 1 automatischer Validator
        LevelOfDetail::Standard => 2,  // 2 unabhängige Zeugen
        LevelOfDetail::Enhanced => 3,  // 3 Zeugen, 2+ Regionen
        LevelOfDetail::Maximum => 5,   // 5 Zeugen, 3+ Regionen, 2+ HW
    }
}
```

#### 4.2 Witness-Unabhängigkeit & Hardware-Attestierung (V0.2)

Witnesses müssen unabhängig sein und ihre Hardware kryptographisch nachweisen:

```rust
#[derive(Clone, Debug)]
pub struct WitnessAttestation {
    pub witness: DID,
    pub event_id: EventId,
    pub timestamp: u64,
    pub valid: bool,
    pub signature: Signature,
    
    // Für Diversitäts-Prüfung
    pub metadata: WitnessMetadata,
    
    // V0.2: Data Availability Nachweis
    pub data_availability: DataAvailabilityProof,
}

#[derive(Clone, Debug)]
pub struct WitnessMetadata {
    pub region: String,          // z.B. "EU-WEST", "US-EAST"
    pub hardware_manufacturer: String,  // z.B. "Intel", "AMD", "ARM"
    pub software_version: String,
    pub ip_prefix: String,       // /24 Prefix für IP-Diversität
    
    // V0.2: Remote Attestation (kryptographischer Hardware-Nachweis)
    pub hardware_attestation: Option<HardwareAttestation>,
}

/// V0.2: Kryptographischer Hardware-Nachweis
#[derive(Clone, Debug)]
pub struct HardwareAttestation {
    /// Art der Attestierung
    pub attestation_type: AttestationType,
    
    /// Der kryptographische Beweis
    pub proof: Vec<u8>,
    
    /// Öffentlicher Schlüssel des TEE/TPM
    pub endorsement_key: Vec<u8>,
    
    /// Zertifikatskette zum Hersteller-Root
    pub certificate_chain: Vec<Vec<u8>>,
    
    /// Zeitstempel der Attestierung
    pub attestation_time: u64,
    
    /// Hash der ausgeführten Software (PCR values)
    pub software_measurements: Vec<[u8; 32]>,
}

#[derive(Clone, Debug)]
pub enum AttestationType {
    /// Intel SGX Remote Attestation
    IntelSGX {
        quote: Vec<u8>,
        ias_report: Option<Vec<u8>>,  // Intel Attestation Service
    },
    
    /// AMD SEV-SNP Attestation
    AmdSevSnp {
        report: Vec<u8>,
        vcek_cert: Vec<u8>,
    },
    
    /// ARM TrustZone Attestation
    ArmTrustZone {
        token: Vec<u8>,
    },
    
    /// TPM 2.0 Remote Attestation
    Tpm2 {
        quote: Vec<u8>,
        pcr_values: Vec<[u8; 32]>,
        aik_cert: Vec<u8>,
    },
    
    /// AWS Nitro Enclave Attestation
    AwsNitro {
        attestation_document: Vec<u8>,
    },
    
    /// Keine Hardware-Attestierung (nur für LoD < Maximum)
    None,
}

/// V0.2: Data Availability Nachweis
#[derive(Clone, Debug)]
pub struct DataAvailabilityProof {
    /// Hash des vollständigen Event-Bodies
    pub event_hash: EventId,
    
    /// Nachweis, dass Witness die Daten gespeichert hat
    pub storage_commitment: [u8; 32],
    
    /// Zeitraum, für den die Daten verfügbar gehalten werden
    pub retention_until: u64,
    
    /// Replikations-Nachweis (für LoD: Maximum)
    pub replication_proofs: Vec<ReplicationProof>,
}

#[derive(Clone, Debug)]
pub struct ReplicationProof {
    pub storage_node: DID,
    pub commitment: [u8; 32],
    pub signature: Signature,
}

fn count_independent_witnesses(witnesses: &[WitnessAttestation]) -> u32 {
    let mut regions = HashSet::new();
    let mut manufacturers = HashSet::new();
    let mut ip_prefixes = HashSet::new();
    let mut count = 0;
    
    for w in witnesses {
        // Neuer Witness zählt nur, wenn er Diversität hinzufügt
        let new_region = regions.insert(&w.metadata.region);
        let new_manufacturer = manufacturers.insert(&w.metadata.hardware_manufacturer);
        let new_ip = ip_prefixes.insert(&w.metadata.ip_prefix);
        
        // Mindestens eines muss neu sein
        if new_region || new_manufacturer || new_ip {
            count += 1;
        }
    }
    
    count
}

/// V0.2: Validiert Hardware-Attestierung
fn validate_hardware_attestation(
    attestation: &HardwareAttestation,
    lod: LevelOfDetail,
) -> Result<(), AttestationError> {
    // Für LoD: Maximum ist Hardware-Attestierung Pflicht
    if lod == LevelOfDetail::Maximum {
        match &attestation.attestation_type {
            AttestationType::None => {
                return Err(AttestationError::RequiredForMaximumLoD);
            }
            _ => {}
        }
    }
    
    // Zertifikatskette verifizieren
    verify_certificate_chain(&attestation.certificate_chain)?;
    
    // Attestierungs-Proof verifizieren
    match &attestation.attestation_type {
        AttestationType::IntelSGX { quote, ias_report } => {
            verify_sgx_quote(quote, ias_report.as_deref())?;
        }
        AttestationType::Tpm2 { quote, pcr_values, aik_cert } => {
            verify_tpm_quote(quote, pcr_values, aik_cert)?;
        }
        AttestationType::AwsNitro { attestation_document } => {
            verify_nitro_attestation(attestation_document)?;
        }
        // ... weitere Typen
        _ => {}
    }
    
    Ok(())
}
```

#### 4.3 Witness-Diversitäts-Constraints

Für höhere LoD-Levels (V0.2: mit Hardware-Attestierung):

| LoD | Witnesses | Regionen | HW-Hersteller | HW-Attestierung |
|-----|-----------|----------|---------------|-----------------|
| Minimal | 0 | - | - | Nein |
| Basic | 1 | 1 | 1 | Nein |
| Standard | 2 | 1 | 1 | Nein |
| Enhanced | 3 | 2+ | 1 | Optional |
| Maximum | 5 | 3+ | 2+ | **Pflicht** |

```rust
fn validate_witness_diversity(
    witnesses: &[WitnessAttestation],
    lod: LevelOfDetail
) -> Result<(), DiversityError> {
    let regions: HashSet<_> = witnesses.iter()
        .map(|w| &w.metadata.region)
        .collect();
    
    let manufacturers: HashSet<_> = witnesses.iter()
        .map(|w| &w.metadata.hardware_manufacturer)
        .collect();
    
    let (min_regions, min_manufacturers) = match lod {
        LevelOfDetail::Enhanced => (2, 1),
        LevelOfDetail::Maximum => (3, 2),
        _ => (1, 1),
    };
    
    if regions.len() < min_regions {
        return Err(DiversityError::InsufficientRegions {
            required: min_regions,
            found: regions.len(),
        });
    }
    
    if manufacturers.len() < min_manufacturers {
        return Err(DiversityError::InsufficientManufacturers {
            required: min_manufacturers,
            found: manufacturers.len(),
        });
    }
    
    // V0.2: Hardware-Attestierung für Maximum LoD
    if lod == LevelOfDetail::Maximum {
        let attested_count = witnesses.iter()
            .filter(|w| !matches!(
                w.metadata.hardware_attestation.as_ref().map(|a| &a.attestation_type),
                Some(AttestationType::None) | None
            ))
            .count();
        
        if attested_count < witnesses.len() {
            return Err(DiversityError::HardwareAttestationRequired {
                required: witnesses.len(),
                attested: attested_count,
            });
        }
    }
    
    Ok(())
}
```

#### 4.4 Data Availability Policy (V0.2)

Ein Event gilt erst als `Attested`, wenn die Witnesses die Datenverfügbarkeit garantieren:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     DATA AVAILABILITY ANFORDERUNGEN                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   REGEL: Ein Event ist ATTESTED genau dann, wenn:                       │
│                                                                         │
│   1. k unabhängige Witnesses das Event validiert haben                  │
│   2. Jeder Witness den vollständigen Event-Body gespeichert hat         │
│   3. Jeder Witness ein DataAvailabilityProof signiert hat               │
│   4. Mindestens 3 Witnesses das Event für min. 30 Tage speichern        │
│                                                                         │
│   ──────────────────────────────────────────────────────────────────    │
│                                                                         │
│   RETENTION POLICY:                                                     │
│                                                                         │
│   LoD Level    Witnesses   Min. Retention   Replikation                 │
│   ─────────────────────────────────────────────────────────────────     │
│   Minimal      0           Actor only       None                        │
│   Basic        1           7 Tage           1 Witness                   │
│   Standard     2           30 Tage          2 Witnesses                 │
│   Enhanced     3           90 Tage          3 Witnesses                 │
│   Maximum      5           365 Tage         5 Witnesses + DHT           │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

```rust
/// V0.2: Prüft Data Availability vor ATTESTED-Transition
fn validate_data_availability(
    event_id: EventId,
    witnesses: &[WitnessAttestation],
    lod: LevelOfDetail,
) -> Result<(), DataAvailabilityError> {
    let (min_witnesses, min_retention_days) = match lod {
        LevelOfDetail::Minimal => return Ok(()), // Keine DA-Anforderung
        LevelOfDetail::Basic => (1, 7),
        LevelOfDetail::Standard => (2, 30),
        LevelOfDetail::Enhanced => (3, 90),
        LevelOfDetail::Maximum => (5, 365),
    };
    
    let min_retention_ms = min_retention_days as u64 * 24 * 60 * 60 * 1000;
    let now = now_ms();
    
    let valid_da_proofs: Vec<_> = witnesses.iter()
        .filter(|w| {
            // Prüfe: Event-Hash stimmt überein
            w.data_availability.event_hash == event_id &&
            // Prüfe: Retention lange genug
            w.data_availability.retention_until >= now + min_retention_ms &&
            // Prüfe: Storage Commitment ist valide
            verify_storage_commitment(&w.data_availability.storage_commitment, event_id)
        })
        .collect();
    
    if valid_da_proofs.len() < min_witnesses {
        return Err(DataAvailabilityError::InsufficientReplicas {
            required: min_witnesses,
            found: valid_da_proofs.len(),
        });
    }
    
    // Für Maximum LoD: Zusätzliche DHT-Replikation erforderlich
    if lod == LevelOfDetail::Maximum {
        for proof in &valid_da_proofs {
            if proof.data_availability.replication_proofs.len() < 3 {
                return Err(DataAvailabilityError::InsufficientDHTReplication);
            }
        }
    }
    
    Ok(())
}

/// Retrieval: Event von Witnesses abrufen
async fn retrieve_event(event_id: EventId) -> Result<Event, RetrievalError> {
    // 1. Lokaler Cache
    if let Some(event) = local_cache.get(&event_id) {
        return Ok(event);
    }
    
    // 2. Witnesses abfragen
    let witnesses = get_witnesses_for_event(event_id).await?;
    for witness in witnesses {
        if let Ok(event) = request_event_from_witness(witness.witness, event_id).await {
            // Verify: Hash muss stimmen
            if compute_event_id(&event) == event_id {
                local_cache.insert(event_id, event.clone());
                return Ok(event);
            }
        }
    }
    
    // 3. DHT-Fallback
    if let Ok(event) = dht.get(event_id).await {
        return Ok(event);
    }
    
    Err(RetrievalError::EventNotAvailable(event_id))
}
```

#### 4.5 Trust-basiertes Rate-Limiting (V0.2)

Spam-Schutz durch Trust-abhängige Event-Limits:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     RATE-LIMITING REGELN                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   FORMEL:                                                               │
│   ────────────────────────────────────────────────────────────────      │
│                                                                         │
│   max_events_per_minute = BASE_RATE × (1 + TRUST_MULTIPLIER × T(s))    │
│                                                                         │
│   Wobei:                                                                │
│   - BASE_RATE = 1 Event/min (für Trust = 0)                            │
│   - TRUST_MULTIPLIER = 99 (für Trust = 1 → 100 Events/min)             │
│   - T(s) = Skalarer Trust-Score [0, 1]                                 │
│                                                                         │
│   ──────────────────────────────────────────────────────────────────    │
│                                                                         │
│   BEISPIELE:                                                            │
│                                                                         │
│   Trust-Level     Trust-Score   Max Events/min                          │
│   ─────────────────────────────────────────────────────────────────     │
│   Unknown         0.0           1                                       │
│   Caution         0.3           31                                      │
│   Neutral         0.5           51                                      │
│   Verified        0.7           71                                      │
│   HighTrust       0.9           91                                      │
│   Maximum         1.0           100                                     │
│                                                                         │
│   ──────────────────────────────────────────────────────────────────    │
│                                                                         │
│   BURST-ALLOWANCE:                                                      │
│   Agents können bis zu 10 × max_events_per_minute in einer              │
│   Minute senden, wenn sie in den vorherigen 9 Minuten unter             │
│   Limit waren (Token Bucket).                                           │
│                                                                         │
│   ──────────────────────────────────────────────────────────────────    │
│                                                                         │
│   ALTERNATIVE FÜR LOW-TRUST:                                            │
│   Agents mit Trust < 0.3 können alternativ Proof-of-Work               │
│   vorlegen (SHA-256 mit Difficulty 20) pro Event.                       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

```rust
const BASE_RATE: f64 = 1.0;
const TRUST_MULTIPLIER: f64 = 99.0;
const BUCKET_SIZE: u64 = 10;  // 10-Minuten-Bucket

pub struct RateLimiter {
    /// Token Bucket pro Actor
    buckets: HashMap<DID, TokenBucket>,
}

#[derive(Clone, Debug)]
pub struct TokenBucket {
    pub tokens: f64,
    pub last_refill: u64,
    pub max_tokens: f64,
    pub refill_rate: f64,  // Tokens pro Millisekunde
}

impl RateLimiter {
    /// Prüft, ob Event erlaubt ist
    pub fn check_rate_limit(
        &mut self,
        actor: &DID,
        trust_score: f64,
        pow_proof: Option<&ProofOfWork>,
    ) -> Result<(), RateLimitError> {
        let max_per_minute = BASE_RATE + TRUST_MULTIPLIER * trust_score;
        let bucket = self.get_or_create_bucket(actor, max_per_minute);
        
        // Refill tokens based on time passed
        let now = now_ms();
        let elapsed = now - bucket.last_refill;
        bucket.tokens = (bucket.tokens + elapsed as f64 * bucket.refill_rate)
            .min(bucket.max_tokens);
        bucket.last_refill = now;
        
        // Check if we have a token
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            return Ok(());
        }
        
        // Low-trust alternative: Proof of Work
        if trust_score < 0.3 {
            if let Some(pow) = pow_proof {
                if verify_pow(pow, 20) {  // Difficulty 20
                    return Ok(());
                }
            }
        }
        
        Err(RateLimitError::TooManyRequests {
            retry_after_ms: (1.0 / bucket.refill_rate) as u64,
        })
    }
    
    fn get_or_create_bucket(&mut self, actor: &DID, max_per_minute: f64) -> &mut TokenBucket {
        self.buckets.entry(actor.clone()).or_insert_with(|| {
            TokenBucket {
                tokens: max_per_minute,  // Start with full bucket
                last_refill: now_ms(),
                max_tokens: max_per_minute * BUCKET_SIZE as f64,
                refill_rate: max_per_minute / 60_000.0,  // per ms
            }
        })
    }
}

/// Proof of Work für Low-Trust Agents
#[derive(Clone, Debug)]
pub struct ProofOfWork {
    pub nonce: u64,
    pub event_hash: EventId,
    pub difficulty: u8,
}

fn verify_pow(pow: &ProofOfWork, required_difficulty: u8) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(&pow.event_hash);
    hasher.update(&pow.nonce.to_le_bytes());
    let hash = hasher.finalize();
    
    // Count leading zero bits
    let leading_zeros = hash.iter()
        .take_while(|&&b| b == 0)
        .count() * 8;
    
    leading_zeros >= required_difficulty as usize
}
```

### 5. Merkle Mountain Range & Proofs (V0.2)

#### 5.1 Merkle Mountain Range (MMR)

Statt klassischer Merkle-Trees verwendet Erynoa **Merkle Mountain Ranges (MMR)** für inkrementelle Berechnung:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    MERKLE MOUNTAIN RANGE (MMR)                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   VORTEIL gegenüber klassischem Merkle-Tree:                           │
│   ─────────────────────────────────────────────────────────────────     │
│   - Root kann inkrementell aktualisiert werden (O(log n) pro Event)    │
│   - Kein Neuberechnen des gesamten Trees bei jedem Checkpoint          │
│   - Append-only Struktur passt perfekt zum Event-DAG                   │
│                                                                         │
│   ──────────────────────────────────────────────────────────────────    │
│                                                                         │
│   STRUKTUR (nach 11 Events):                                            │
│                                                                         │
│              Peak 1 (8)           Peak 2 (2)     Peak 3 (1)            │
│                 │                    │              │                   │
│        ┌────────┴────────┐      ┌────┴────┐        │                   │
│        │                 │      │         │        │                   │
│    ┌───┴───┐         ┌───┴───┐  E9       E10      E11                  │
│    │       │         │       │                                          │
│  ┌─┴─┐   ┌─┴─┐     ┌─┴─┐   ┌─┴─┐                                       │
│  E1  E2  E3  E4    E5  E6  E7  E8                                      │
│                                                                         │
│   MMR-Root = Hash(Peak1 || Peak2 || Peak3)                             │
│                                                                         │
│   ──────────────────────────────────────────────────────────────────    │
│                                                                         │
│   PROOF für E6:                                                         │
│   [E5, Hash(E7,E8), Peak1, Peak2, Peak3]                               │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

```rust
/// Merkle Mountain Range für inkrementelles Hashing
pub struct MerkleMountainRange {
    /// Alle Blätter (Event-Hashes)
    leaves: Vec<[u8; 32]>,
    
    /// Peaks der "Berge" (perfect binary trees)
    peaks: Vec<[u8; 32]>,
    
    /// Höhe jedes Peaks
    peak_heights: Vec<u32>,
    
    /// Gesamtzahl Blätter
    size: u64,
}

impl MerkleMountainRange {
    pub fn new() -> Self {
        Self {
            leaves: Vec::new(),
            peaks: Vec::new(),
            peak_heights: Vec::new(),
            size: 0,
        }
    }
    
    /// Fügt neues Event hinzu (O(log n))
    pub fn append(&mut self, event_hash: [u8; 32]) -> u64 {
        self.leaves.push(event_hash);
        self.size += 1;
        
        let mut current_hash = event_hash;
        let mut current_height = 0;
        
        // Merge mit vorherigen Peaks gleicher Höhe
        while !self.peaks.is_empty() && 
              *self.peak_heights.last().unwrap() == current_height {
            let left_peak = self.peaks.pop().unwrap();
            self.peak_heights.pop();
            
            current_hash = sha256_pair(&left_peak, &current_hash);
            current_height += 1;
        }
        
        self.peaks.push(current_hash);
        self.peak_heights.push(current_height);
        
        self.size - 1  // Return index
    }
    
    /// Berechnet MMR-Root (O(peaks) = O(log n))
    pub fn root(&self) -> [u8; 32] {
        if self.peaks.is_empty() {
            return [0u8; 32];
        }
        
        // Bag all peaks from right to left
        let mut root = *self.peaks.last().unwrap();
        for peak in self.peaks.iter().rev().skip(1) {
            root = sha256_pair(peak, &root);
        }
        
        root
    }
    
    /// Erstellt Proof für Event an Index
    pub fn proof(&self, index: u64) -> MerkleProof {
        let mut path = Vec::new();
        let mut current_index = index;
        let mut current_size = self.size;
        let mut peak_offset = 0;
        
        // Finde den Peak, der dieses Blatt enthält
        for (peak_idx, &height) in self.peak_heights.iter().enumerate() {
            let peak_size = 1u64 << height;
            
            if current_index < peak_size {
                // Proof innerhalb dieses Peaks
                self.proof_within_peak(&mut path, current_index, height);
                
                // Füge andere Peaks zum Proof hinzu
                for (other_idx, &other_peak) in self.peaks.iter().enumerate() {
                    if other_idx != peak_idx {
                        path.push(MerkleNode {
                            hash: other_peak,
                            position: if other_idx < peak_idx { 
                                Position::Left 
                            } else { 
                                Position::Right 
                            },
                        });
                    }
                }
                break;
            }
            
            current_index -= peak_size;
            peak_offset += peak_size;
        }
        
        MerkleProof {
            event_id: self.leaves[index as usize],
            root: self.root(),
            path,
            index,
            total_events: self.size,
        }
    }
    
    fn proof_within_peak(&self, path: &mut Vec<MerkleNode>, index: u64, height: u32) {
        // Standard Merkle-Tree Proof innerhalb eines "Berges"
        // ... implementation details
    }
}
```

#### 5.2 Klassischer Merkle-Tree (Legacy-Kompatibilität)

Für Backwards-Kompatibilität wird auch der klassische Merkle-Tree unterstützt:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     KLASSISCHER MERKLE-TREE                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│                           Root Hash                                     │
│                          (verankert)                                    │
│                              │                                          │
│              ┌───────────────┴───────────────┐                         │
│              │                               │                         │
│           Hash(L)                         Hash(R)                      │
│              │                               │                         │
│       ┌──────┴──────┐                 ┌──────┴──────┐                  │
│       │             │                 │             │                  │
│   Hash(E1,E2)   Hash(E3,E4)      Hash(E5,E6)   Hash(E7,E8)            │
│       │             │                 │             │                  │
│    ┌──┴──┐       ┌──┴──┐          ┌──┴──┐       ┌──┴──┐               │
│    E1    E2      E3    E4         E5    E6      E7    E8              │
│                                                                         │
│   PROOF für E3:                                                         │
│   [E4, Hash(E1,E2), Hash(R)]                                           │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 5.2 Merkle-Proof Struktur

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Das Event, für das der Proof gilt
    pub event_id: EventId,
    
    /// Der Merkle-Root (verankert auf Chain)
    pub root: [u8; 32],
    
    /// Proof-Pfad
    pub path: Vec<MerkleNode>,
    
    /// Index des Events im Tree
    pub index: u64,
    
    /// Gesamtzahl Events im Tree
    pub total_events: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerkleNode {
    pub hash: [u8; 32],
    pub position: Position, // Left oder Right
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Position {
    Left,
    Right,
}

impl MerkleProof {
    /// Verifiziert den Proof
    pub fn verify(&self, event_id: EventId) -> bool {
        let mut current_hash = event_id;
        
        for node in &self.path {
            current_hash = match node.position {
                Position::Left => sha256_pair(&node.hash, &current_hash),
                Position::Right => sha256_pair(&current_hash, &node.hash),
            };
        }
        
        current_hash == self.root
    }
}

fn sha256_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}
```

#### 5.3 Checkpoint-Events

Periodisch werden Checkpoint-Events erstellt:

```rust
#[derive(Clone, Debug)]
pub struct Checkpoint {
    /// Checkpoint-Nummer
    pub sequence: u64,
    
    /// Merkle-Root aller Events seit letztem Checkpoint
    pub merkle_root: [u8; 32],
    
    /// Anzahl Events in diesem Checkpoint
    pub event_count: u64,
    
    /// Referenz auf vorherigen Checkpoint
    pub previous: Option<EventId>,
    
    /// Timestamp-Range
    pub start_time: u64,
    pub end_time: u64,
}

impl EventDAG {
    /// Erstellt einen Checkpoint
    pub fn create_checkpoint(&mut self) -> Result<EventId, CheckpointError> {
        let events_since_last = self.get_events_since_last_checkpoint();
        
        if events_since_last.is_empty() {
            return Err(CheckpointError::NoNewEvents);
        }
        
        let merkle_root = compute_merkle_root(&events_since_last);
        
        let checkpoint = Checkpoint {
            sequence: self.checkpoint_count + 1,
            merkle_root,
            event_count: events_since_last.len() as u64,
            previous: self.last_checkpoint,
            start_time: events_since_last.first().unwrap().timestamp,
            end_time: events_since_last.last().unwrap().timestamp,
        };
        
        let event = Event {
            version: 1,
            event_type: EventType::Checkpoint,
            actor: self.system_did.clone(),
            timestamp: now_ms(),
            parents: self.tips.iter().copied().collect(),
            payload: EventPayload::Checkpoint(checkpoint),
            signature: self.sign_checkpoint(&checkpoint),
            realm: None,
        };
        
        self.add_event(event)
    }
}
```

### 6. Chain-Anchoring

#### 6.1 Anchor-Prozess

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         ANCHORING-PROZESS                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   1. BATCH COLLECTION                                                   │
│      ├── Events mit Finality = Attested sammeln                        │
│      ├── Max. 1000 Events pro Batch                                    │
│      └── Max. 10 Sekunden Wartezeit                                    │
│                                                                         │
│   2. MERKLE TREE                                                        │
│      ├── Events sortieren (nach Timestamp, dann ID)                    │
│      ├── Merkle-Tree konstruieren                                      │
│      └── Root-Hash berechnen                                           │
│                                                                         │
│   3. CHAIN SUBMISSION                                                   │
│      ├── Transaktion mit Root-Hash erstellen                           │
│      ├── An Primary Chain (IOTA) senden                                │
│      └── Optional: An Secondary Chains senden                          │
│                                                                         │
│   4. CONFIRMATION                                                       │
│      ├── Auf Block-Inclusion warten                                    │
│      ├── Anchor-Event erstellen                                        │
│      └── Finality → Anchored für alle Events                           │
│                                                                         │
│   5. FINALITY                                                           │
│      ├── Auf Tiefe d warten (10 Blocks)                                │
│      └── Finality → Final für alle Events                              │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 6.2 Anchor-Event

```rust
#[derive(Clone, Debug)]
pub struct AnchorInfo {
    /// Blockchain-Name
    pub chain: String,
    
    /// Netzwerk (mainnet, testnet)
    pub network: String,
    
    /// Transaktions-ID auf der Chain
    pub tx_id: String,
    
    /// Block-Nummer
    pub block: u64,
    
    /// Block-Timestamp
    pub timestamp: u64,
    
    /// Merkle-Root der verankerten Events
    pub merkle_root: [u8; 32],
    
    /// Liste der verankerten Event-IDs
    pub events: Vec<EventId>,
    
    /// Contract-Adresse (falls applicable)
    pub contract: Option<String>,
}

impl EventDAG {
    /// Erstellt Anchor-Event nach Chain-Confirmation
    pub fn create_anchor(
        &mut self,
        chain: &str,
        tx_id: &str,
        block: u64,
        merkle_root: [u8; 32],
        events: Vec<EventId>,
    ) -> Result<EventId, AnchorError> {
        let anchor_info = AnchorInfo {
            chain: chain.to_string(),
            network: "mainnet".to_string(),
            tx_id: tx_id.to_string(),
            block,
            timestamp: now_ms(),
            merkle_root,
            events: events.clone(),
            contract: None,
        };
        
        let event = Event {
            version: 1,
            event_type: EventType::Anchor,
            actor: self.system_did.clone(),
            timestamp: now_ms(),
            parents: self.tips.iter().copied().collect(),
            payload: EventPayload::Anchor {
                chain: chain.to_string(),
                tx_id: tx_id.to_string(),
                block,
                merkle_root,
                events: events.clone(),
            },
            signature: self.sign_anchor(&anchor_info),
            realm: None,
        };
        
        let anchor_id = self.add_event(event)?;
        
        // Finality auf Anchored setzen
        for event_id in &events {
            self.finality.insert(*event_id, FinalityLevel::Anchored);
            self.anchors.entry(*event_id)
                .or_default()
                .push(anchor_info.clone());
        }
        
        Ok(anchor_id)
    }
}
```

#### 6.3 IOTA-spezifische Integration

```rust
pub struct IOTAAnchor {
    client: iota_sdk::Client,
    wallet: iota_sdk::Wallet,
}

impl IOTAAnchor {
    /// Verankert Merkle-Root auf IOTA
    pub async fn anchor(&self, merkle_root: [u8; 32]) -> Result<AnchorResult, IOTAError> {
        // IOTA Rebased mit MoveVM
        let tx = self.client
            .build_transaction()
            .with_move_call(
                "erynoa::anchor::submit",
                vec![
                    MoveValue::Vector(merkle_root.to_vec()),
                    MoveValue::U64(now_ms()),
                ],
            )
            .sign(&self.wallet)
            .await?;
        
        let result = self.client.submit_transaction(tx).await?;
        
        Ok(AnchorResult {
            tx_id: result.tx_id,
            block: result.block,
        })
    }
    
    /// Prüft Finality-Tiefe
    pub async fn check_depth(&self, block: u64) -> Result<u64, IOTAError> {
        let current = self.client.get_current_block().await?;
        Ok(current - block)
    }
}
```

### 7. Queries & Traversal

#### 7.1 Event-Abfragen

```rust
impl EventDAG {
    /// Alle Events eines Actors
    pub fn events_by_actor(&self, actor: &DID) -> Vec<&Event> {
        self.actor_events.get(actor)
            .map(|ids| ids.iter()
                .filter_map(|id| self.events.get(id))
                .collect())
            .unwrap_or_default()
    }
    
    /// Events in Zeitraum
    pub fn events_in_range(&self, start: u64, end: u64) -> Vec<&Event> {
        self.events.values()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect()
    }
    
    /// Events nach Typ
    pub fn events_by_type(&self, event_type: EventType) -> Vec<&Event> {
        self.events.values()
            .filter(|e| e.event_type == event_type)
            .collect()
    }
    
    /// Kausale Historie eines Events
    pub fn causal_history(&self, event_id: EventId) -> Vec<EventId> {
        let mut history = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(event_id);
        
        while let Some(current) = queue.pop_front() {
            if visited.insert(current) {
                history.push(current);
                if let Some(event) = self.events.get(&current) {
                    for parent in &event.parents {
                        queue.push_back(*parent);
                    }
                }
            }
        }
        
        history
    }
    
    /// Kausale Zukunft eines Events
    pub fn causal_future(&self, event_id: EventId) -> Vec<EventId> {
        let mut future = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(event_id);
        
        while let Some(current) = queue.pop_front() {
            if visited.insert(current) {
                future.push(current);
                if let Some(children) = self.children.get(&current) {
                    for child in children {
                        queue.push_back(*child);
                    }
                }
            }
        }
        
        future
    }
}
```

#### 7.2 Trust-relevante Abfragen

```rust
impl EventDAG {
    /// Berechnet C(s) - Anzahl der Events eines Actors
    pub fn event_count(&self, actor: &DID) -> u64 {
        self.actor_events.get(actor)
            .map(|v| v.len() as u64)
            .unwrap_or(0)
    }
    
    /// Events für Trust-Berechnung (nur finalisierte)
    pub fn trust_relevant_events(&self, actor: &DID) -> Vec<&Event> {
        self.events_by_actor(actor)
            .into_iter()
            .filter(|e| {
                let id = compute_event_id(e);
                matches!(
                    self.finality.get(&id),
                    Some(FinalityLevel::Attested) 
                    | Some(FinalityLevel::Anchored) 
                    | Some(FinalityLevel::Final)
                )
            })
            .collect()
    }
}
```

### 8. Synchronisation

#### 8.1 Sync-Protokoll

```rust
pub struct SyncProtocol {
    dag: Arc<RwLock<EventDAG>>,
    peers: Vec<PeerId>,
}

impl SyncProtocol {
    /// Pull-basierte Synchronisation
    pub async fn sync_with_peer(&self, peer: PeerId) -> Result<SyncStats, SyncError> {
        // 1. Tips austauschen
        let local_tips = self.dag.read().tips.clone();
        let remote_tips = self.request_tips(peer).await?;
        
        // 2. Fehlende Events identifizieren
        let missing = self.identify_missing(&local_tips, &remote_tips).await?;
        
        // 3. Events anfordern
        let mut received = 0;
        for event_id in missing {
            if let Some(event) = self.request_event(peer, event_id).await? {
                self.dag.write().add_event(event)?;
                received += 1;
            }
        }
        
        Ok(SyncStats { received })
    }
    
    /// Push: Neues Event an Peers propagieren
    pub async fn broadcast_event(&self, event: &Event) -> Result<(), BroadcastError> {
        for peer in &self.peers {
            self.send_event(*peer, event).await?;
        }
        Ok(())
    }
}
```

#### 8.2 Conflict Resolution

Bei parallelen Events (keine kausale Ordnung):

```rust
impl EventDAG {
    /// Deterministisches Ordering für parallele Events
    pub fn deterministic_order(&self, e1: EventId, e2: EventId) -> Ordering {
        // Wenn kausale Ordnung existiert, nutze diese
        if self.happened_before(e1, e2) {
            return Ordering::Less;
        }
        if self.happened_before(e2, e1) {
            return Ordering::Greater;
        }
        
        // Parallele Events: Deterministisches Ordering
        // 1. Timestamp
        let t1 = self.events.get(&e1).map(|e| e.timestamp).unwrap_or(0);
        let t2 = self.events.get(&e2).map(|e| e.timestamp).unwrap_or(0);
        
        match t1.cmp(&t2) {
            Ordering::Equal => {
                // 2. Event-ID (lexikographisch)
                e1.cmp(&e2)
            }
            other => other,
        }
    }
}
```

### 9. API

#### 9.1 Event-Erstellung

```
POST /v1/events
Content-Type: application/json

{
  "event_type": "TransactionPropose",
  "payload": {
    "transaction_id": "tx-2024-001",
    "counterparty": "did:erynoa:self:bob",
    "terms": { ... }
  },
  "parents": ["0x1234...", "0x5678..."],
  "signature": "z3FcQmH..."
}
```

**Response:**

```json
{
  "event_id": "0xabcdef...",
  "finality": "Pending",
  "timestamp": 1706540400000
}
```

#### 9.2 Event-Abfrage

```
GET /v1/events/{event_id}
```

**Response:**

```json
{
  "event": {
    "id": "0xabcdef...",
    "event_type": "TransactionPropose",
    "actor": "did:erynoa:self:alice",
    "timestamp": 1706540400000,
    "parents": ["0x1234...", "0x5678..."],
    "payload": { ... },
    "signature": "z3FcQmH..."
  },
  "finality": {
    "level": "Anchored",
    "witnesses": 3,
    "anchor": {
      "chain": "iota",
      "block": 12345678,
      "merkle_proof": { ... }
    }
  }
}
```

#### 9.3 Merkle-Proof anfordern

```
GET /v1/events/{event_id}/proof
```

**Response:**

```json
{
  "event_id": "0xabcdef...",
  "merkle_root": "0x123456...",
  "path": [
    { "hash": "0xaaa...", "position": "left" },
    { "hash": "0xbbb...", "position": "right" }
  ],
  "anchor": {
    "chain": "iota",
    "tx_id": "0xfff...",
    "block": 12345678
  }
}
```

### 10. CLI-Nutzung

```bash
# Event erstellen
erynoa event create --type TransactionPropose \
  --payload '{"transaction_id": "tx-001", ...}' \
  --sign-with key-1

# Event anzeigen
erynoa event show 0xabcdef...

# Event-Historie eines Actors
erynoa event list --actor did:erynoa:self:alice

# Kausale Historie
erynoa event history 0xabcdef...

# Finality-Status
erynoa event finality 0xabcdef...

# Merkle-Proof anfordern
erynoa event proof 0xabcdef...

# Proof verifizieren
erynoa event verify-proof 0xabcdef... --proof proof.json

# DAG-Statistiken
erynoa dag stats

# Tips anzeigen
erynoa dag tips

# Sync mit Peer
erynoa dag sync --peer /ip4/1.2.3.4/tcp/9000
```

### 11. SDK-Nutzung

#### 11.1 Rust

```rust
use erynoa_sdk::dag::{EventDAG, Event, EventType, FinalityLevel};

// Event erstellen
let event = Event::builder()
    .event_type(EventType::TransactionPropose)
    .actor(my_did.clone())
    .payload(TransactionPropose {
        transaction_id: "tx-001".into(),
        counterparty: bob_did.clone(),
        terms: terms,
    })
    .parents(dag.tips())
    .sign(&keypair)?
    .build()?;

let event_id = client.submit_event(event).await?;
println!("Event submitted: {:?}", event_id);

// Auf Finality warten
loop {
    let finality = client.get_finality(event_id).await?;
    println!("Finality: {:?}", finality.level);
    
    if finality.level >= FinalityLevel::Anchored {
        break;
    }
    
    tokio::time::sleep(Duration::from_secs(5)).await;
}

// Merkle-Proof abrufen
let proof = client.get_merkle_proof(event_id).await?;
assert!(proof.verify(event_id));
println!("Proof verified, anchored at block {}", proof.anchor.block);

// Kausale Historie
let history = client.get_causal_history(event_id).await?;
println!("Event has {} ancestors", history.len());
```

#### 11.2 TypeScript

```typescript
import { EventDAG, Event, EventType, FinalityLevel } from '@erynoa/sdk';

// Event erstellen
const event = await Event.builder()
  .eventType(EventType.TransactionPropose)
  .actor(myDid)
  .payload({
    transactionId: 'tx-001',
    counterparty: bobDid,
    terms: terms,
  })
  .parents(await dag.getTips())
  .sign(keypair)
  .build();

const eventId = await client.submitEvent(event);
console.log(`Event submitted: ${eventId}`);

// Auf Finality warten
const finality = await client.waitForFinality(eventId, FinalityLevel.Anchored, {
  timeout: 60_000,
  pollInterval: 5_000,
});

console.log(`Finalized at block ${finality.anchor.block}`);

// Merkle-Proof verifizieren
const proof = await client.getMerkleProof(eventId);
const isValid = proof.verify(eventId);
console.log(`Proof valid: ${isValid}`);
```

---

## Test-Vektoren

### TV-1: Event-ID Berechnung

**Input:**

```json
{
  "version": 1,
  "event_type": "TrustAttestation",
  "actor": "did:erynoa:self:alice",
  "timestamp": 1706540400000,
  "parents": ["0x1111111111111111111111111111111111111111111111111111111111111111"],
  "payload": {
    "subject": "did:erynoa:self:bob",
    "dimension": "R",
    "value": 0.8
  }
}
```

**Expected Event-ID:**

```
sha256(cbor_encode(event)) = 0x...
```

### TV-2: Merkle-Proof Verification

**Input:**

```json
{
  "event_id": "0xaaaa...",
  "root": "0xffff...",
  "path": [
    { "hash": "0xbbbb...", "position": "right" },
    { "hash": "0xcccc...", "position": "left" }
  ]
}
```

**Verification:**

```
hash1 = sha256(event_id || 0xbbbb) = 0xdddd
hash2 = sha256(0xcccc || hash1) = 0xffff (== root)
=> VALID
```

### TV-3: Finality Progression

**Timeline:**

```
t=0:    Event created, signed             → Pending
t=5s:   2 witnesses confirm               → Pending (need 3)
t=8s:   3rd witness confirms              → Attested
t=15s:  Anchored on IOTA block 100        → Anchored
t=120s: IOTA block 110 (depth=10)         → Final
```

---

## Referenzen

- [Erynoa Fachkonzept V6.1](../FACHKONZEPT.md)
- [EIP-001: DID:erynoa](./EIP-001-did-erynoa.md)
- [EIP-002: Trust Vector 6D](./EIP-002-trust-vector-6d.md)
- [EIP-004: Bayesian Trust Update](./EIP-004-bayesian-trust-update.md)
- [IOTA Tangle Specification](https://wiki.iota.org/)
- [Merkle Tree (RFC 6962)](https://tools.ietf.org/html/rfc6962)
- [Merkle Mountain Range](https://github.com/opentimestamps/opentimestamps-server/blob/master/doc/merkle-mountain-range.md)
- [CBOR (RFC 8949)](https://tools.ietf.org/html/rfc8949)

---

## Changelog

| Version | Datum | Änderung |
|---------|-------|----------|
| 0.1 | 2026-01-29 | Initial Draft |
| 0.2 | 2026-01-29 | Hardware-Attestierung (TPM/SGX/Nitro), Data Availability Policy, Trust-basiertes Rate-Limiting, Merkle Mountain Range (MMR) |

---

*EIP-003: Event-DAG & Finality Specification*
*Version: 0.2*
*Status: Draft*
*Ebene: E1 (Fundament)*
