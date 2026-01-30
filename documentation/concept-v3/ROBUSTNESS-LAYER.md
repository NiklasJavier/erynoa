# Erynoa Robustness Layer V5.1

> **Version:** 5.1 – Battle-Tested Architecture
> **Datum:** Januar 2026
> **Zweck:** Transformation von theoretischer Eleganz zu produktionsreifer Robustheit
> **Paradigma:** Red Team → Blue Team Defense

---

## Übersicht: Die 5 Verteidigungslinien

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ERYNOA ROBUSTNESS ARCHITECTURE V5.1                       │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  Layer 5: LEGAL WRAPPER                                               │  │
│  │  ────────────────────────────────────────────────────────────────────  │  │
│  │  Ricardian Contracts • Controller Chain • Jurisdiction Binding        │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                    │                                         │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  Layer 4: MARKET BOOTSTRAP                                            │  │
│  │  ────────────────────────────────────────────────────────────────────  │  │
│  │  Single-Player Mode • Federated Genesis • Retroactive Funding         │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                    │                                         │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  Layer 3: ANTI-GAMING                                                 │  │
│  │  ────────────────────────────────────────────────────────────────────  │  │
│  │  EigenTrust Topology • Stake-at-Risk • Sybil Resistance               │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                    │                                         │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  Layer 2: REALITY ANCHOR                                              │  │
│  │  ────────────────────────────────────────────────────────────────────  │  │
│  │  Hardware PUFs • Multi-Path Witnessing • Geo-Proofs                   │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                    │                                         │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  Layer 1: FUZZY INTERPRETATION                                        │  │
│  │  ────────────────────────────────────────────────────────────────────  │  │
│  │  Confidence Intervals • Hysteresis Buckets • Qualitative Output       │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                    │                                         │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  CORE: Weltformel V5.0 (Quantum • Category • Topology)                │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

# LAYER 1: FUZZY INTERPRETATION

## Problem: False Precision

Die Weltformel liefert `P(success) = 0.7234567`. Das suggeriert eine Präzision, die nicht existiert:

- Die Eingabedaten haben Rauschen
- Das Modell hat Unsicherheit
- Die Zukunft ist nicht determinierbar

**Gefahr:** Nutzer verlassen sich auf eine Scheingenauigkeit.

---

## Lösung: Konfidenz-Intervalle & Qualitative Buckets

### 1.1 Architektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         FUZZY INTERPRETATION LAYER                           │
│                                                                              │
│   RAW QUANTUM STATE                    INTERPRETED OUTPUT                    │
│   ─────────────────                    ─────────────────────                 │
│                                                                              │
│   |Ψ⟩ = 0.72|honest⟩ +                 ┌─────────────────────┐              │
│         0.31|reliable⟩ + ...    ──►    │  Level: VERIFIED    │              │
│                                        │  Confidence: MEDIUM │              │
│   ⟨Ψ|𝕎̂|Ψ⟩ = 0.7234567                 │                     │              │
│   σ = 0.15 (Standardabweichung)        │  "Vertrauenswürdig, │              │
│   n = 47 (Interaktionen)               │   aber noch wenige  │              │
│                                        │   Erfahrungswerte"  │              │
│                                        └─────────────────────┘              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Implementierung

```rust
// erynoa-core/src/robustness/fuzzy.rs

use statrs::distribution::{ContinuousCDF, Normal};

/// Qualitative Trust-Levels (keine Pseudopräzision)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TrustLevel {
    /// Nicht genug Daten für eine Aussage
    Unknown,
    /// Vorsicht geboten (Trust < 0.4 mit hoher Konfidenz)
    Caution,
    /// Neutral (Trust 0.4-0.6 oder niedrige Konfidenz)
    Neutral,
    /// Verifiziert gut (Trust 0.6-0.8 mit guter Konfidenz)
    Verified,
    /// Hohes Vertrauen (Trust > 0.8 mit hoher Konfidenz)
    HighTrust,
}

/// Konfidenz-Level basierend auf Datenmenge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ConfidenceLevel {
    /// < 5 Interaktionen
    VeryLow,
    /// 5-20 Interaktionen
    Low,
    /// 20-100 Interaktionen
    Medium,
    /// 100-500 Interaktionen
    High,
    /// > 500 Interaktionen
    VeryHigh,
}

/// Interpretiertes Trust-Ergebnis
#[derive(Debug, Clone, Serialize)]
pub struct InterpretedTrust {
    /// Qualitatives Level
    pub level: TrustLevel,
    
    /// Konfidenz in die Aussage
    pub confidence: ConfidenceLevel,
    
    /// Menschenlesbare Zusammenfassung
    pub summary: String,
    
    /// Konfidenzintervall [lower, upper] (95%)
    pub interval: (f64, f64),
    
    /// Rohdaten (für Power-User)
    pub raw: RawTrustData,
}

#[derive(Debug, Clone, Serialize)]
pub struct RawTrustData {
    pub expected_value: f64,
    pub std_deviation: f64,
    pub sample_size: u64,
    pub quantum_state: QuantumStateSnapshot,
}

/// Fuzzy Interpretation Engine
pub struct FuzzyInterpreter {
    /// Hysterese-Schwellwerte (verhindern Oszillation)
    thresholds: HysteresisThresholds,
    
    /// Vorherige Levels (für Hysterese)
    previous_levels: DashMap<DID, TrustLevel>,
}

impl FuzzyInterpreter {
    /// Interpretiert einen Quanten-Trust-Zustand
    pub fn interpret(&self, did: &DID, state: &QuantumState, history: &InteractionHistory) -> InterpretedTrust {
        // 1. Berechne Statistiken
        let expected = state.expected_trust();
        let std_dev = self.compute_std_deviation(state, history);
        let sample_size = history.interaction_count();
        
        // 2. Berechne Konfidenzintervall (95%)
        let z = 1.96; // 95% CI
        let margin = z * std_dev / (sample_size as f64).sqrt().max(1.0);
        let interval = (expected - margin, expected + margin);
        
        // 3. Bestimme Konfidenz-Level
        let confidence = match sample_size {
            0..=4 => ConfidenceLevel::VeryLow,
            5..=19 => ConfidenceLevel::Low,
            20..=99 => ConfidenceLevel::Medium,
            100..=499 => ConfidenceLevel::High,
            _ => ConfidenceLevel::VeryHigh,
        };
        
        // 4. Bestimme Trust-Level mit Hysterese
        let level = self.determine_level_with_hysteresis(did, expected, confidence);
        
        // 5. Generiere menschenlesbare Zusammenfassung
        let summary = self.generate_summary(level, confidence, sample_size);
        
        InterpretedTrust {
            level,
            confidence,
            summary,
            interval,
            raw: RawTrustData {
                expected_value: expected,
                std_deviation: std_dev,
                sample_size,
                quantum_state: state.snapshot(),
            },
        }
    }
    
    /// Hysterese verhindert Oszillation an Schwellwerten
    fn determine_level_with_hysteresis(
        &self,
        did: &DID,
        expected: f64,
        confidence: ConfidenceLevel,
    ) -> TrustLevel {
        // Bei zu niedriger Konfidenz: immer Unknown oder Neutral
        if confidence == ConfidenceLevel::VeryLow {
            return TrustLevel::Unknown;
        }
        
        let previous = self.previous_levels.get(did).map(|v| *v);
        
        // Hysterese-Bänder
        let (caution_up, caution_down) = (0.35, 0.45);
        let (neutral_up, neutral_down) = (0.55, 0.65);
        let (verified_up, verified_down) = (0.75, 0.85);
        
        let new_level = match previous {
            Some(TrustLevel::Caution) => {
                if expected > caution_down { TrustLevel::Neutral } 
                else { TrustLevel::Caution }
            }
            Some(TrustLevel::Neutral) => {
                if expected < caution_up { TrustLevel::Caution }
                else if expected > neutral_down { TrustLevel::Verified }
                else { TrustLevel::Neutral }
            }
            Some(TrustLevel::Verified) => {
                if expected < neutral_up { TrustLevel::Neutral }
                else if expected > verified_down { TrustLevel::HighTrust }
                else { TrustLevel::Verified }
            }
            Some(TrustLevel::HighTrust) => {
                if expected < verified_up { TrustLevel::Verified }
                else { TrustLevel::HighTrust }
            }
            _ => {
                // Kein vorheriger Zustand: direkte Klassifikation
                if expected < 0.4 { TrustLevel::Caution }
                else if expected < 0.6 { TrustLevel::Neutral }
                else if expected < 0.8 { TrustLevel::Verified }
                else { TrustLevel::HighTrust }
            }
        };
        
        self.previous_levels.insert(did.clone(), new_level);
        new_level
    }
    
    fn generate_summary(&self, level: TrustLevel, confidence: ConfidenceLevel, n: u64) -> String {
        match (level, confidence) {
            (TrustLevel::Unknown, _) => 
                "Zu wenige Daten für eine Bewertung. Kleinere Testinteraktion empfohlen.".into(),
            
            (TrustLevel::HighTrust, ConfidenceLevel::VeryHigh) =>
                format!("Sehr vertrauenswürdig. {} erfolgreiche Interaktionen.", n),
            
            (TrustLevel::HighTrust, ConfidenceLevel::High) =>
                format!("Vertrauenswürdig mit guter Datenbasis ({} Interaktionen).", n),
            
            (TrustLevel::HighTrust, _) =>
                format!("Hoher Trust, aber noch wenige Datenpunkte ({}).", n),
            
            (TrustLevel::Verified, ConfidenceLevel::VeryHigh | ConfidenceLevel::High) =>
                "Solide Vertrauensbasis. Normale Vorsichtsmaßnahmen ausreichend.".into(),
            
            (TrustLevel::Verified, _) =>
                "Gute Tendenz, aber Konfidenz noch aufzubauen.".into(),
            
            (TrustLevel::Neutral, _) =>
                "Weder positiv noch negativ aufgefallen. Abwarten empfohlen.".into(),
            
            (TrustLevel::Caution, ConfidenceLevel::VeryHigh | ConfidenceLevel::High) =>
                "⚠️ Vorsicht: Dokumentierte Probleme in der Vergangenheit.".into(),
            
            (TrustLevel::Caution, _) =>
                "⚠️ Vorsicht empfohlen, obwohl Datenbasis noch dünn.".into(),
        }
    }
}
```

### 1.3 API Output Beispiele

```json
// Beispiel 1: Neuer Agent mit gutem Start
{
  "level": "VERIFIED",
  "confidence": "LOW",
  "summary": "Gute Tendenz, aber Konfidenz noch aufzubauen.",
  "interval": [0.52, 0.88],
  "raw": {
    "expected_value": 0.70,
    "std_deviation": 0.18,
    "sample_size": 12
  }
}

// Beispiel 2: Etablierter Agent
{
  "level": "HIGH_TRUST",
  "confidence": "VERY_HIGH",
  "summary": "Sehr vertrauenswürdig. 1247 erfolgreiche Interaktionen.",
  "interval": [0.89, 0.93],
  "raw": {
    "expected_value": 0.91,
    "std_deviation": 0.08,
    "sample_size": 1247
  }
}

// Beispiel 3: Problematischer Agent
{
  "level": "CAUTION",
  "confidence": "HIGH",
  "summary": "⚠️ Vorsicht: Dokumentierte Probleme in der Vergangenheit.",
  "interval": [0.22, 0.38],
  "raw": {
    "expected_value": 0.30,
    "std_deviation": 0.12,
    "sample_size": 89
  }
}
```

---

# LAYER 2: REALITY ANCHOR

## Problem: Oracle Problem

Perfekte Kryptographie schützt nicht vor:
- Gefälschten Sensordaten
- Kompromittierten IoT-Geräten
- Kolludierten Zeugen

**Gefahr:** "Garbage In, Cryptographically Signed Garbage Out"

---

## Lösung: Hardware Binding + Multi-Path Witnessing

### 2.1 Architektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          REALITY ANCHOR LAYER                                │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    PHYSICAL UNCLONABLE FUNCTIONS (PUFs)              │    │
│  │                                                                      │    │
│  │   ┌──────────┐    ┌──────────┐    ┌──────────┐                      │    │
│  │   │ Sensor   │    │ EV       │    │ Smart    │                      │    │
│  │   │ Device   │    │ Charger  │    │ Meter    │                      │    │
│  │   │          │    │          │    │          │                      │    │
│  │   │ ┌──────┐ │    │ ┌──────┐ │    │ ┌──────┐ │                      │    │
│  │   │ │ PUF  │ │    │ │ PUF  │ │    │ │ PUF  │ │                      │    │
│  │   │ │ Chip │ │    │ │ Chip │ │    │ │ Chip │ │                      │    │
│  │   │ └──────┘ │    │ └──────┘ │    │ └──────┘ │                      │    │
│  │   └────┬─────┘    └────┬─────┘    └────┬─────┘                      │    │
│  │        │               │               │                             │    │
│  │        ▼               ▼               ▼                             │    │
│  │   ┌──────────────────────────────────────────────────────────────┐  │    │
│  │   │              Hardware-gebundene Signaturen                    │  │    │
│  │   │           (Private Key = f(Silizium-Eigenschaften))           │  │    │
│  │   └──────────────────────────────────────────────────────────────┘  │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                         │
│                                    ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    MULTI-PATH WITNESSING                             │    │
│  │                                                                      │    │
│  │          Witness A          Witness B          Witness C            │    │
│  │          (Utility)          (Regulator)        (Third-Party)        │    │
│  │              │                   │                  │                │    │
│  │              │   Independent     │    Independent   │                │    │
│  │              │   Verification    │    Verification  │                │    │
│  │              ▼                   ▼                  ▼                │    │
│  │         ┌─────────────────────────────────────────────┐             │    │
│  │         │  Consensus: Event gilt als "Wahr" wenn      │             │    │
│  │         │  ≥ k von n Witnesses mit Entanglement < θ   │             │    │
│  │         │  bestätigen                                  │             │    │
│  │         └─────────────────────────────────────────────┘             │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                         │
│                                    ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    GEO-LOCATION PROOFS                               │    │
│  │                                                                      │    │
│  │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                 │    │
│  │   │ GPS + Time  │  │ Helium PoC  │  │ Cell Tower  │                 │    │
│  │   │ Attestation │  │ (LoRaWAN)   │  │ Triangulat. │                 │    │
│  │   └─────────────┘  └─────────────┘  └─────────────┘                 │    │
│  │                         │                                            │    │
│  │         "Event E happened at location L at time T"                   │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Hardware Anchor (PUF Integration)

```rust
// erynoa-core/src/robustness/hardware.rs

/// Physical Unclonable Function Attestation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PUFAttestation {
    /// Challenge-Response Pair
    pub challenge: [u8; 32],
    pub response: [u8; 32],
    
    /// Hersteller-Signatur
    pub manufacturer_signature: Signature,
    pub manufacturer_did: DID,
    
    /// Zeitpunkt der Attestierung
    pub timestamp: Timestamp,
    
    /// Hardware-Metadaten
    pub hardware_info: HardwareInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub device_type: DeviceType,
    pub model: String,
    pub firmware_hash: Hash,
    pub secure_element_version: String,
}

/// DID-Dokument Erweiterung für Hardware-Binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareBoundDID {
    /// Standard DID-Dokument
    pub did_document: DIDDocument,
    
    /// Hardware-Attestierung
    pub hardware_attestation: Option<PUFAttestation>,
    
    /// Ist der Private Key hardware-gebunden?
    pub hardware_bound: bool,
    
    /// Supply-Chain-Trust (Hersteller → Distributor → Betreiber)
    pub supply_chain: Vec<SupplyChainEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyChainEntry {
    pub role: SupplyChainRole,
    pub entity_did: DID,
    pub attestation: Signature,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SupplyChainRole {
    Manufacturer,
    Distributor,
    Installer,
    Operator,
}

impl HardwareBoundDID {
    /// Verifiziert die gesamte Hardware-Vertrauenskette
    pub fn verify_hardware_trust(&self) -> Result<HardwareTrustLevel, HardwareError> {
        // 1. Prüfe PUF-Attestierung
        let puf_valid = if let Some(puf) = &self.hardware_attestation {
            self.verify_puf_attestation(puf)?
        } else {
            false
        };
        
        // 2. Prüfe Supply Chain
        let supply_chain_valid = self.verify_supply_chain()?;
        
        // 3. Bestimme Trust-Level
        match (puf_valid, supply_chain_valid, self.hardware_bound) {
            (true, true, true) => Ok(HardwareTrustLevel::Full),
            (true, true, false) => Ok(HardwareTrustLevel::AttestationOnly),
            (false, true, _) => Ok(HardwareTrustLevel::SupplyChainOnly),
            (true, false, _) => Ok(HardwareTrustLevel::PUFOnly),
            _ => Ok(HardwareTrustLevel::None),
        }
    }
    
    fn verify_puf_attestation(&self, puf: &PUFAttestation) -> Result<bool, HardwareError> {
        // Verifiziere Hersteller-Signatur
        let manufacturer = self.get_trusted_manufacturer(&puf.manufacturer_did)?;
        let message = self.construct_puf_message(puf);
        
        manufacturer.verify(&message, &puf.manufacturer_signature)
    }
}

/// Hardware Trust Levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum HardwareTrustLevel {
    /// Keine Hardware-Verifizierung
    None,
    /// Nur PUF-Attestierung
    PUFOnly,
    /// Nur Supply-Chain verifiziert
    SupplyChainOnly,
    /// PUF + Supply Chain, aber Key nicht hardware-gebunden
    AttestationOnly,
    /// Vollständige Hardware-Verifizierung
    Full,
}
```

### 2.3 Multi-Path Witnessing

```rust
// erynoa-core/src/robustness/witness.rs

/// Multi-Path Witness Consensus
pub struct WitnessConsensus {
    /// Minimale Anzahl unabhängiger Witnesses
    pub k_threshold: u32,
    
    /// Maximales Entanglement zwischen Witnesses
    pub max_entanglement: f64,
    
    /// Witness-Pool
    witnesses: Vec<WitnessInfo>,
}

#[derive(Debug, Clone)]
pub struct WitnessInfo {
    pub did: DID,
    pub category: WitnessCategory,
    pub trust_score: f64,
    pub hardware_level: HardwareTrustLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WitnessCategory {
    /// Utility-Provider (z.B. Stromanbieter)
    Utility,
    /// Regulator/Behörde
    Regulator,
    /// Unabhängiger Dritter
    ThirdParty,
    /// IoT-Netzwerk (z.B. Helium)
    IoTNetwork,
    /// Community Validator
    Community,
}

impl WitnessConsensus {
    /// Prüft ob ein Event ausreichend bezeugt ist
    pub fn verify_event(&self, event: &Event, attestations: &[Attestation]) -> WitnessResult {
        // 1. Filtere gültige Attestierungen
        let valid_attestations: Vec<_> = attestations.iter()
            .filter(|a| self.verify_attestation(a).is_ok())
            .collect();
        
        // 2. Prüfe Unabhängigkeit (kein Entanglement-Cluster)
        let independent_witnesses = self.filter_independent(&valid_attestations);
        
        // 3. Prüfe Diversität (verschiedene Kategorien)
        let categories: HashSet<_> = independent_witnesses.iter()
            .map(|a| self.get_witness_category(&a.witness_did))
            .collect();
        
        // 4. Berechne Consensus Score
        let count = independent_witnesses.len() as u32;
        let category_diversity = categories.len();
        
        let consensus_met = count >= self.k_threshold;
        let diversity_met = category_diversity >= 2;
        
        WitnessResult {
            event_id: event.id.clone(),
            attestation_count: count,
            independent_count: independent_witnesses.len() as u32,
            category_diversity,
            consensus_met,
            diversity_met,
            confidence: self.compute_confidence(count, category_diversity),
            details: independent_witnesses.iter().map(|a| WitnessDetail {
                did: a.witness_did.clone(),
                category: self.get_witness_category(&a.witness_did),
                timestamp: a.timestamp,
            }).collect(),
        }
    }
    
    /// Filtert Witnesses nach Unabhängigkeit (Entanglement < threshold)
    fn filter_independent(&self, attestations: &[&Attestation]) -> Vec<&Attestation> {
        let mut independent = Vec::new();
        let mut entanglement_graph = EntanglementGraph::new();
        
        for attestation in attestations {
            let witness_did = &attestation.witness_did;
            
            // Prüfe Entanglement zu allen bereits ausgewählten
            let max_entanglement_to_selected = independent.iter()
                .map(|a: &&Attestation| {
                    entanglement_graph.get_entanglement(witness_did, &a.witness_did)
                })
                .fold(0.0f64, f64::max);
            
            // Nur hinzufügen wenn unabhängig genug
            if max_entanglement_to_selected < self.max_entanglement {
                independent.push(attestation);
            }
        }
        
        independent
    }
    
    fn compute_confidence(&self, count: u32, diversity: usize) -> f64 {
        let count_factor = (count as f64 / self.k_threshold as f64).min(1.0);
        let diversity_factor = (diversity as f64 / 3.0).min(1.0);
        
        // Geometrisches Mittel
        (count_factor * diversity_factor).sqrt()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WitnessResult {
    pub event_id: EventId,
    pub attestation_count: u32,
    pub independent_count: u32,
    pub category_diversity: usize,
    pub consensus_met: bool,
    pub diversity_met: bool,
    pub confidence: f64,
    pub details: Vec<WitnessDetail>,
}
```

### 2.4 Geo-Location Proofs

```rust
// erynoa-core/src/robustness/geo.rs

/// Geo-Location Proof System
pub struct GeoProof {
    /// Behaupteter Ort
    pub claimed_location: GeoLocation,
    
    /// Zeitpunkt
    pub timestamp: Timestamp,
    
    /// Beweistypen
    pub proofs: Vec<LocationProofType>,
}

#[derive(Debug, Clone)]
pub enum LocationProofType {
    /// GPS mit Zeitstempel-Signatur
    GPS {
        coordinates: (f64, f64),
        accuracy_meters: f32,
        satellite_count: u8,
        time_signature: Signature,
    },
    
    /// Helium Network Proof of Coverage
    HeliumPoC {
        hotspot_ids: Vec<String>,
        rssi_values: Vec<i16>,
        snr_values: Vec<f32>,
        beacon_data: BeaconData,
    },
    
    /// Cell Tower Triangulation
    CellTower {
        towers: Vec<CellTowerInfo>,
        estimated_location: (f64, f64),
        accuracy_meters: f32,
    },
    
    /// WiFi-basierte Lokalisierung
    WiFi {
        access_points: Vec<WiFiAPInfo>,
        estimated_location: (f64, f64),
    },
}

impl GeoProof {
    /// Verifiziert den Geo-Proof
    pub fn verify(&self) -> GeoVerificationResult {
        let mut verified_proofs = Vec::new();
        let mut locations = Vec::new();
        
        for proof in &self.proofs {
            match self.verify_single_proof(proof) {
                Ok(location) => {
                    verified_proofs.push(proof.clone());
                    locations.push(location);
                }
                Err(e) => {
                    // Log but continue
                }
            }
        }
        
        // Konsistenz-Check: Alle Locations sollten nahe beieinander sein
        let consistent = self.check_location_consistency(&locations);
        
        // Berechne finale Location (gewichteter Durchschnitt)
        let final_location = self.compute_weighted_location(&locations, &verified_proofs);
        
        GeoVerificationResult {
            verified: verified_proofs.len() >= 2 && consistent,
            proof_count: verified_proofs.len(),
            consistency: consistent,
            estimated_location: final_location,
            accuracy_meters: self.compute_combined_accuracy(&verified_proofs),
        }
    }
    
    fn check_location_consistency(&self, locations: &[(f64, f64)]) -> bool {
        if locations.len() < 2 {
            return false;
        }
        
        // Alle Locations sollten innerhalb von 1km sein
        let max_distance = 1000.0; // meters
        
        for i in 0..locations.len() {
            for j in (i+1)..locations.len() {
                let distance = haversine_distance(locations[i], locations[j]);
                if distance > max_distance {
                    return false;
                }
            }
        }
        
        true
    }
}
```

---

# LAYER 3: ANTI-GAMING

## Problem: Trust Farming & Sybil Attacks

Bot-Netzwerke können:
- Tausende Fake-Identitäten erstellen
- Sich gegenseitig bewerten
- Legitimität simulieren

**Gefahr:** Das Trust-System wird zur Farce.

---

## Lösung: EigenTrust + Stake-at-Risk

### 3.1 Architektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          ANTI-GAMING LAYER                                   │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    EIGENTRUST TOPOLOGY                               │    │
│  │                                                                      │    │
│  │     Lokaler Trust        →        Globaler Trust                    │    │
│  │     "B sagt A ist gut"            "Wie sehr vertraut das            │    │
│  │                                    Netzwerk B?"                      │    │
│  │                                                                      │    │
│  │   ┌─────┐                         ┌─────────────────────────────┐   │    │
│  │   │  A  │◄────trust───────────────│         NETWORK             │   │    │
│  │   └──▲──┘                         │                             │   │    │
│  │      │                            │    T_global(A) =            │   │    │
│  │      │ local_trust                │    Σ T_global(B) · t(B→A)   │   │    │
│  │      │                            │    ─────────────────────    │   │    │
│  │   ┌──┴──┐                         │    (Iterative Berechnung)   │   │    │
│  │   │  B  │                         └─────────────────────────────┘   │    │
│  │   └─────┘                                                            │    │
│  │                                                                      │    │
│  │   Sybil-Cluster:                                                    │    │
│  │   ┌─────┐    ┌─────┐    ┌─────┐                                    │    │
│  │   │Fake1│◄──►│Fake2│◄──►│Fake3│  ← Hoher interner Trust           │    │
│  │   └─────┘    └─────┘    └─────┘  ← KEIN externer Trust             │    │
│  │        ╲         │         ╱     ← Cluster bleibt isoliert         │    │
│  │         ╲        │        ╱                                         │    │
│  │          ╲       │       ╱                                          │    │
│  │           ╳──────┼──────╳  ← Keine Verbindung zum echten Netzwerk  │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    STAKE-AT-RISK (SLASHING)                          │    │
│  │                                                                      │    │
│  │   Reputation-Level         Required Stake         Slash on Fraud    │    │
│  │   ─────────────────────────────────────────────────────────────────  │    │
│  │   FRESH                    0 ERY                  N/A               │    │
│  │   EMERGING                 100 ERY                50%               │    │
│  │   STABLE                   500 ERY                75%               │    │
│  │   TRUSTED                  2000 ERY               90%               │    │
│  │   WISE                     10000 ERY              100%              │    │
│  │                                                                      │    │
│  │   Fraud Detection:                                                  │    │
│  │   • Signierte Widersprüche (mathematisch beweisbar)                 │    │
│  │   • Witness-Konsens gegen Agent                                     │    │
│  │   • Automatisches Slashing durch Smart Contract                     │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 EigenTrust Implementation

```rust
// erynoa-core/src/robustness/eigentrust.rs

use nalgebra::{DMatrix, DVector};
use std::collections::HashMap;

/// EigenTrust-basierte globale Trust-Berechnung
pub struct EigenTrustEngine {
    /// Lokale Trust-Matrix C[i][j] = "wie sehr vertraut i dem j"
    local_trust: DMatrix<f64>,
    
    /// DID zu Index Mapping
    did_to_index: HashMap<DID, usize>,
    index_to_did: Vec<DID>,
    
    /// Konvergenz-Parameter
    epsilon: f64,
    max_iterations: usize,
    
    /// Pre-Trust Vektor (für bekannte gute Nodes)
    pre_trust: DVector<f64>,
    
    /// Pre-Trust Gewicht
    alpha: f64,
}

impl EigenTrustEngine {
    /// Berechnet globalen Trust für alle Agenten
    /// 
    /// t^(k+1) = (1-α) · C^T · t^(k) + α · p
    /// 
    /// wobei:
    /// - t = globaler Trust-Vektor
    /// - C = normierte lokale Trust-Matrix
    /// - p = Pre-Trust Vektor
    /// - α = Pre-Trust Gewicht
    pub fn compute_global_trust(&self) -> HashMap<DID, f64> {
        let n = self.local_trust.nrows();
        
        // 1. Normiere lokale Trust-Matrix (Zeilen-Summe = 1)
        let normalized = self.normalize_trust_matrix();
        
        // 2. Initialisiere mit Pre-Trust
        let mut trust = self.pre_trust.clone();
        
        // 3. Power Iteration
        for iteration in 0..self.max_iterations {
            let prev_trust = trust.clone();
            
            // t^(k+1) = (1-α) · C^T · t^(k) + α · p
            trust = &normalized.transpose() * &prev_trust * (1.0 - self.alpha)
                  + &self.pre_trust * self.alpha;
            
            // Prüfe Konvergenz
            let diff: f64 = (&trust - &prev_trust).norm();
            if diff < self.epsilon {
                break;
            }
        }
        
        // 4. Konvertiere zurück zu HashMap
        let mut result = HashMap::new();
        for (idx, did) in self.index_to_did.iter().enumerate() {
            result.insert(did.clone(), trust[idx]);
        }
        
        result
    }
    
    /// Aktualisiert lokalen Trust nach einer Interaktion
    pub fn update_local_trust(
        &mut self,
        from: &DID,
        to: &DID,
        outcome: InteractionOutcome,
    ) -> Result<(), EigenTrustError> {
        let from_idx = self.get_or_create_index(from);
        let to_idx = self.get_or_create_index(to);
        
        // Inkrementelle Update basierend auf Outcome
        let delta = match outcome {
            InteractionOutcome::Positive => 1.0,
            InteractionOutcome::Neutral => 0.0,
            InteractionOutcome::Negative => -1.0,
        };
        
        // Exponential Moving Average für Stabilität
        let current = self.local_trust[(from_idx, to_idx)];
        let alpha = 0.1; // Learning rate
        self.local_trust[(from_idx, to_idx)] = current * (1.0 - alpha) + delta * alpha;
        
        Ok(())
    }
    
    /// Erkennt Sybil-Cluster
    pub fn detect_sybil_clusters(&self) -> Vec<SybilCluster> {
        let global_trust = self.compute_global_trust();
        
        // 1. Finde Cluster mit hohem internem Trust aber niedrigem externem Trust
        let clusters = self.find_dense_subgraphs();
        
        let mut sybil_candidates = Vec::new();
        
        for cluster in clusters {
            // Berechne durchschnittlichen externen Trust
            let external_trust = self.compute_external_trust(&cluster, &global_trust);
            let internal_trust = self.compute_internal_trust(&cluster);
            
            // Sybil-Indikator: Hoher interner, niedriger externer Trust
            let sybil_score = internal_trust / (external_trust + 0.01);
            
            if sybil_score > 10.0 {
                sybil_candidates.push(SybilCluster {
                    members: cluster,
                    sybil_score,
                    internal_trust,
                    external_trust,
                });
            }
        }
        
        sybil_candidates
    }
    
    fn compute_external_trust(&self, cluster: &[DID], global_trust: &HashMap<DID, f64>) -> f64 {
        cluster.iter()
            .filter_map(|did| global_trust.get(did))
            .sum::<f64>() / cluster.len() as f64
    }
    
    fn compute_internal_trust(&self, cluster: &[DID]) -> f64 {
        let indices: Vec<_> = cluster.iter()
            .filter_map(|did| self.did_to_index.get(did))
            .copied()
            .collect();
        
        let mut sum = 0.0;
        let mut count = 0;
        
        for &i in &indices {
            for &j in &indices {
                if i != j {
                    sum += self.local_trust[(i, j)];
                    count += 1;
                }
            }
        }
        
        if count > 0 { sum / count as f64 } else { 0.0 }
    }
}

#[derive(Debug)]
pub struct SybilCluster {
    pub members: Vec<DID>,
    pub sybil_score: f64,
    pub internal_trust: f64,
    pub external_trust: f64,
}
```

### 3.3 Stake-at-Risk (Slashing)

```rust
// erynoa-core/src/robustness/staking.rs

/// Stake-at-Risk System
pub struct StakingContract {
    /// Minimaler Stake pro Tier
    tier_requirements: HashMap<TrustTier, u64>,
    
    /// Slash-Raten pro Tier
    slash_rates: HashMap<TrustTier, f64>,
    
    /// Aktive Stakes
    stakes: HashMap<DID, StakeInfo>,
}

#[derive(Debug, Clone)]
pub struct StakeInfo {
    pub did: DID,
    pub amount: u64,
    pub locked_until: Timestamp,
    pub tier: TrustTier,
}

/// Beweisbarer Betrug
#[derive(Debug, Clone)]
pub enum FraudProof {
    /// Zwei widersprüchliche signierte Aussagen
    SignedContradiction {
        statement_1: SignedStatement,
        statement_2: SignedStatement,
    },
    
    /// Witness-Konsens gegen Agent
    WitnessConsensus {
        event_id: EventId,
        attestations_against: Vec<Attestation>,
        claimed_by_agent: SignedStatement,
    },
    
    /// Double-Spend Attempt
    DoubleSpend {
        transaction_1: SignedTransaction,
        transaction_2: SignedTransaction,
    },
}

impl StakingContract {
    /// Sperrt Stake für höheren Tier
    pub fn stake(&mut self, did: &DID, amount: u64) -> Result<StakeResult, StakingError> {
        let current_stake = self.stakes.get(did).map(|s| s.amount).unwrap_or(0);
        let new_total = current_stake + amount;
        
        // Bestimme neuen Tier
        let new_tier = self.determine_tier(new_total);
        
        // Speichere Stake
        self.stakes.insert(did.clone(), StakeInfo {
            did: did.clone(),
            amount: new_total,
            locked_until: Timestamp::now() + Duration::days(30),
            tier: new_tier,
        });
        
        Ok(StakeResult {
            new_tier,
            total_staked: new_total,
        })
    }
    
    /// Führt Slashing durch bei nachgewiesenem Betrug
    pub fn slash(&mut self, did: &DID, proof: FraudProof) -> Result<SlashResult, StakingError> {
        // 1. Verifiziere Beweis
        self.verify_fraud_proof(&proof)?;
        
        // 2. Hole aktuellen Stake
        let stake = self.stakes.get(did)
            .ok_or(StakingError::NoStake)?;
        
        // 3. Berechne Slash-Amount
        let slash_rate = self.slash_rates.get(&stake.tier)
            .copied()
            .unwrap_or(0.5);
        let slash_amount = (stake.amount as f64 * slash_rate) as u64;
        
        // 4. Führe Slash durch
        let new_amount = stake.amount - slash_amount;
        let new_tier = self.determine_tier(new_amount);
        
        self.stakes.insert(did.clone(), StakeInfo {
            did: did.clone(),
            amount: new_amount,
            locked_until: stake.locked_until,
            tier: new_tier,
        });
        
        // 5. Verbrenne geslashten Amount
        self.burn(slash_amount);
        
        Ok(SlashResult {
            did: did.clone(),
            slashed_amount: slash_amount,
            remaining_stake: new_amount,
            new_tier,
            fraud_proof: proof,
        })
    }
    
    fn verify_fraud_proof(&self, proof: &FraudProof) -> Result<(), StakingError> {
        match proof {
            FraudProof::SignedContradiction { statement_1, statement_2 } => {
                // Beide Statements müssen vom gleichen DID signiert sein
                if statement_1.signer != statement_2.signer {
                    return Err(StakingError::InvalidProof("Different signers"));
                }
                
                // Signaturen müssen gültig sein
                statement_1.verify()?;
                statement_2.verify()?;
                
                // Statements müssen sich widersprechen
                if !statement_1.contradicts(statement_2) {
                    return Err(StakingError::InvalidProof("No contradiction"));
                }
                
                Ok(())
            }
            FraudProof::WitnessConsensus { event_id, attestations_against, claimed_by_agent } => {
                // Mindestens k Witnesses gegen Agent
                if attestations_against.len() < 3 {
                    return Err(StakingError::InvalidProof("Not enough witnesses"));
                }
                
                // Alle Attestierungen verifizieren
                for attestation in attestations_against {
                    attestation.verify()?;
                }
                
                Ok(())
            }
            FraudProof::DoubleSpend { transaction_1, transaction_2 } => {
                // Beide Transaktionen versuchen gleiche Assets zu transferieren
                if transaction_1.signer != transaction_2.signer {
                    return Err(StakingError::InvalidProof("Different signers"));
                }
                
                if !transaction_1.conflicts_with(transaction_2) {
                    return Err(StakingError::InvalidProof("No conflict"));
                }
                
                transaction_1.verify()?;
                transaction_2.verify()?;
                
                Ok(())
            }
        }
    }
    
    fn determine_tier(&self, stake: u64) -> TrustTier {
        if stake >= 10000 { TrustTier::WISE }
        else if stake >= 2000 { TrustTier::TRUSTED }
        else if stake >= 500 { TrustTier::STABLE }
        else if stake >= 100 { TrustTier::EMERGING }
        else { TrustTier::FRESH }
    }
}
```

---

# LAYER 4: MARKET BOOTSTRAP

## Problem: Cold Start

Niemand nutzt ein leeres Netzwerk. Klassisches Henne-Ei-Problem.

**Gefahr:** Das beste System stirbt ohne kritische Masse.

---

## Lösung: Single-Player First + Federated Genesis

### 4.1 Architektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         MARKET BOOTSTRAP LAYER                               │
│                                                                              │
│  PHASE 1: SINGLE-PLAYER MODE                                                 │
│  ────────────────────────────────────────────────────────────────────────    │
│                                                                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐             │
│  │   Enterprise A   │  │   Enterprise B   │  │   Enterprise C   │             │
│  │                  │  │                  │  │                  │             │
│  │  ┌────────────┐ │  │  ┌────────────┐ │  │  ┌────────────┐ │             │
│  │  │ Internal   │ │  │  │ Internal   │ │  │  │ Internal   │ │             │
│  │  │ Erynoa     │ │  │  │ Erynoa     │ │  │  │ Erynoa     │ │             │
│  │  │ Network    │ │  │  │ Network    │ │  │  │ Network    │ │             │
│  │  └────────────┘ │  │  └────────────┘ │  │  └────────────┘ │             │
│  │  100 devices    │  │  500 devices    │  │  1000 devices   │             │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘             │
│           │                    │                    │                        │
│           │                    │                    │                        │
│  PHASE 2: BRIDGE OPENING                                                     │
│  ────────────────────────────────────────────────────────────────────────    │
│           │                    │                    │                        │
│           ▼                    ▼                    ▼                        │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                       FEDERATED NETWORK                                │  │
│  │                                                                        │  │
│  │   Enterprise A ◄────────► Enterprise B ◄────────► Enterprise C        │  │
│  │       100             Bridge            500             Bridge     1000│  │
│  │                                                                        │  │
│  │                     Instant: 1600 devices                              │  │
│  │                     + Network Effects                                  │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  PHASE 3: PUBLIC NETWORK                                                     │
│  ────────────────────────────────────────────────────────────────────────    │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                       GLOBAL ERYNOA NETWORK                            │  │
│  │                                                                        │  │
│  │   Enterprises ◄────────► SMBs ◄────────► Individuals                  │  │
│  │                                                                        │  │
│  │                     Open Participation                                 │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Single-Player Mode

```rust
// erynoa-sdk/src/bootstrap/single_player.rs

/// Single-Player Erynoa Instance (Enterprise Intranet)
pub struct SinglePlayerMode {
    /// Lokale Instanz ohne externe Vernetzung
    local_network: LocalNetwork,
    
    /// Pre-configured Trust für interne Geräte
    internal_trust: InternalTrustConfig,
    
    /// Bridge-Ready Status
    bridge_config: Option<BridgeConfig>,
}

impl SinglePlayerMode {
    /// Erstellt isolierte Enterprise-Instanz
    pub fn new(enterprise_did: DID, config: EnterpriseConfig) -> Self {
        Self {
            local_network: LocalNetwork::new(enterprise_did.clone()),
            internal_trust: InternalTrustConfig {
                // Alle internen Geräte starten mit hohem Trust
                default_internal_trust: 0.8,
                // Aber mit Confidence "Enterprise-Verified"
                confidence_source: ConfidenceSource::EnterpriseAdmin,
            },
            bridge_config: None,
        }
    }
    
    /// Registriert internes Gerät
    pub fn register_device(&mut self, device: DeviceConfig) -> Result<DID, BootstrapError> {
        // 1. Erstelle Sub-DID unter Enterprise-DID
        let device_did = self.local_network.enterprise_did.sub(&device.name);
        
        // 2. Setze initialen Trust (Enterprise-garantiert)
        self.local_network.set_initial_trust(
            &device_did,
            self.internal_trust.default_internal_trust,
            TrustSource::EnterpriseAdmin,
        );
        
        // 3. Registriere im lokalen Netzwerk
        self.local_network.register(device_did.clone(), device)?;
        
        Ok(device_did)
    }
    
    /// Bereitet Bridge zum globalen Netzwerk vor
    pub fn prepare_bridge(&mut self, config: BridgeConfig) -> Result<BridgeReadyStatus, BootstrapError> {
        // 1. Prüfe interne Netzwerk-Gesundheit
        let health = self.local_network.health_check()?;
        if health.trust_coverage < 0.95 {
            return Err(BootstrapError::InsufficientCoverage);
        }
        
        // 2. Generiere Bridge-Credentials
        let bridge_credentials = self.generate_bridge_credentials(&config)?;
        
        // 3. Speichere Konfiguration
        self.bridge_config = Some(config);
        
        Ok(BridgeReadyStatus {
            device_count: self.local_network.device_count(),
            trust_coverage: health.trust_coverage,
            credentials: bridge_credentials,
        })
    }
    
    /// Aktiviert Bridge (Enterprise → Global Network)
    pub async fn activate_bridge(&mut self) -> Result<BridgeConnection, BootstrapError> {
        let config = self.bridge_config.as_ref()
            .ok_or(BootstrapError::BridgeNotPrepared)?;
        
        // 1. Verbinde zum globalen Netzwerk
        let connection = GlobalNetwork::connect(&config.endpoint).await?;
        
        // 2. Registriere Enterprise als Realm
        let realm_id = connection.register_realm(
            &self.local_network.enterprise_did,
            RealmConfig {
                name: config.realm_name.clone(),
                trust_policy: config.trust_policy.clone(),
            },
        ).await?;
        
        // 3. Publiziere aggregierte Trust-Daten (nicht individuelle!)
        connection.publish_realm_summary(
            &realm_id,
            self.local_network.compute_summary(),
        ).await?;
        
        // 4. Abonniere Cross-Realm Events
        connection.subscribe_cross_realm(&realm_id).await?;
        
        Ok(BridgeConnection {
            realm_id,
            connection,
            local_network: self.local_network.clone(),
        })
    }
}
```

### 4.3 Retroactive Public Goods Funding

```rust
// erynoa-core/src/bootstrap/funding.rs

/// Retroactive Funding für Blueprint-Ersteller
pub struct RetroactiveFunding {
    /// Pool für Blueprint-Rewards
    blueprint_pool: TokenPool,
    
    /// Usage Tracking
    usage_tracker: UsageTracker,
}

/// Blueprint mit Funding-Eligibility
#[derive(Debug, Clone)]
pub struct FundedBlueprint {
    pub blueprint_id: BlueprintId,
    pub creator_did: DID,
    pub schema: Schema,
    
    /// Funding-Metriken
    pub usage_count: u64,
    pub unique_users: u64,
    pub revenue_generated: u64,
}

impl RetroactiveFunding {
    /// Berechnet Reward für Blueprint-Ersteller
    pub fn compute_reward(&self, blueprint: &FundedBlueprint) -> RewardCalculation {
        // Formel: Reward = Base + Usage_Factor + Network_Effect
        
        // Base: Einmalig für Veröffentlichung
        let base_reward = 100; // ERY
        
        // Usage: Pro Nutzung (degressive Kurve)
        let usage_factor = self.compute_usage_factor(blueprint.usage_count);
        
        // Network Effect: Bonus wenn andere Blueprints drauf aufbauen
        let network_effect = self.compute_network_effect(&blueprint.blueprint_id);
        
        // Total
        let total = base_reward + usage_factor + network_effect;
        
        RewardCalculation {
            blueprint_id: blueprint.blueprint_id.clone(),
            creator_did: blueprint.creator_did.clone(),
            base_reward,
            usage_factor,
            network_effect,
            total_reward: total,
        }
    }
    
    fn compute_usage_factor(&self, usage_count: u64) -> u64 {
        // Degressive Kurve: sqrt(usage) * 10
        ((usage_count as f64).sqrt() * 10.0) as u64
    }
    
    fn compute_network_effect(&self, blueprint_id: &BlueprintId) -> u64 {
        let dependent_blueprints = self.usage_tracker.count_dependents(blueprint_id);
        
        // Exponentieller Bonus für "Foundational" Blueprints
        if dependent_blueprints > 100 {
            500
        } else if dependent_blueprints > 10 {
            100
        } else if dependent_blueprints > 0 {
            20
        } else {
            0
        }
    }
    
    /// Distribuiert Rewards monatlich
    pub async fn distribute_monthly_rewards(&mut self) -> Result<DistributionReport, FundingError> {
        let blueprints = self.usage_tracker.get_active_blueprints();
        
        let mut distributions = Vec::new();
        
        for blueprint in blueprints {
            let reward = self.compute_reward(&blueprint);
            
            // Transfer aus Pool an Creator
            self.blueprint_pool.transfer(
                &blueprint.creator_did,
                reward.total_reward,
            ).await?;
            
            distributions.push(reward);
        }
        
        Ok(DistributionReport {
            period: "2026-01",
            total_distributed: distributions.iter().map(|r| r.total_reward).sum(),
            recipient_count: distributions.len(),
            distributions,
        })
    }
}
```

---

# LAYER 5: LEGAL WRAPPER

## Problem: Rechtliche Haftung

Wenn ein autonomer Agent Schaden verursacht:
- Wer ist verantwortlich?
- Welches Recht gilt?
- Wie wird Streit gelöst?

**Gefahr:** Juristische Unsicherheit verhindert Enterprise-Adoption.

---

## Lösung: Ricardian Contracts + Controller Chain

### 5.1 Architektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           LEGAL WRAPPER LAYER                                │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    RICARDIAN CONTRACTS                               │    │
│  │                                                                      │    │
│  │   ┌──────────────────────┐      ┌──────────────────────┐            │    │
│  │   │     Smart Contract   │      │    Legal Document    │            │    │
│  │   │     (ECL Code)       │◄────►│    (PDF/Prose)       │            │    │
│  │   │                      │      │                      │            │    │
│  │   │  legal_hash:         │      │  Hash: 0x7f3a...     │            │    │
│  │   │    "0x7f3a..."       │      │                      │            │    │
│  │   │                      │      │  "Bei Streit gilt    │            │    │
│  │   │  legal_venue:        │      │   deutsches Recht,   │            │    │
│  │   │    "DE-Frankfurt"    │      │   Gerichtsstand      │            │    │
│  │   │                      │      │   Frankfurt a.M."    │            │    │
│  │   └──────────────────────┘      └──────────────────────┘            │    │
│  │              │                             │                         │    │
│  │              └─────────── Hash Match ──────┘                         │    │
│  │                                                                      │    │
│  │   Regel: Code ist Execution, PDF ist Interpretation                 │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    CONTROLLER CHAIN                                  │    │
│  │                                                                      │    │
│  │       Autonomous Agent                   Controller                  │    │
│  │       (Machine DID)                      (Human/Legal Entity)        │    │
│  │                                                                      │    │
│  │   ┌──────────────────────┐          ┌──────────────────────┐        │    │
│  │   │ did:erynoa:device:   │          │ did:erynoa:business: │        │    │
│  │   │   ev-charger-001     │──────────│   siemens-energy-gmbh│        │    │
│  │   │                      │controller│                      │        │    │
│  │   │ controller:          │          │ verified: KYC        │        │    │
│  │   │   did:erynoa:...     │          │ jurisdiction: DE     │        │    │
│  │   │                      │          │ legal_rep: "Max M."  │        │    │
│  │   └──────────────────────┘          └──────────────────────┘        │    │
│  │                                                                      │    │
│  │   Regel: Kein High-Value ohne verifizierten Controller              │    │
│  │   Regel: Controller haftet für Agent-Handlungen                     │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    VALUE THRESHOLDS                                  │    │
│  │                                                                      │    │
│  │   Transaction Value    │  Requirements                               │    │
│  │   ─────────────────────┼────────────────────────────────────────     │    │
│  │   < 100 EUR            │  None (pseudonymous)                        │    │
│  │   100 - 1000 EUR       │  DID with basic verification                │    │
│  │   1000 - 10000 EUR     │  Verified Controller required               │    │
│  │   > 10000 EUR          │  KYC Controller + Ricardian Contract        │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Ricardian Contracts

```rust
// erynoa-core/src/legal/ricardian.rs

/// Ricardian Contract: Code + Legal Document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RicardianContract {
    /// Smart Contract (ECL)
    pub code: ECLContract,
    
    /// Hash des zugehörigen Legal Documents
    pub legal_document_hash: Hash,
    
    /// Rechtsordnung
    pub jurisdiction: Jurisdiction,
    
    /// Gerichtsstand
    pub legal_venue: String,
    
    /// Streitbeilegung
    pub dispute_resolution: DisputeResolution,
    
    /// Signaturen beider Parteien
    pub signatures: Vec<ContractSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Jurisdiction {
    Germany,
    EU,
    Switzerland,
    USA(USState),
    International(ArbitrationBody),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisputeResolution {
    /// Gerichtsverfahren
    Court { venue: String },
    
    /// Schiedsverfahren
    Arbitration { body: ArbitrationBody, rules: String },
    
    /// On-Chain Mediation
    OnChainMediation { mediator_pool: MediatorPool },
    
    /// Eskalationspfad
    Escalation {
        level_1: Box<DisputeResolution>,
        level_2: Box<DisputeResolution>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalDocument {
    /// Menschenlesbarer Vertragstext
    pub prose: String,
    
    /// Sprache
    pub language: Language,
    
    /// Version
    pub version: String,
    
    /// Referenz zum Code
    pub code_reference: ContractId,
    
    /// Definitionen (Mapping von Code-Variablen zu Rechtsbegriffen)
    pub definitions: HashMap<String, String>,
}

impl RicardianContract {
    /// Erstellt einen neuen Ricardian Contract
    pub fn new(
        code: ECLContract,
        legal_doc: &LegalDocument,
        jurisdiction: Jurisdiction,
    ) -> Result<Self, LegalError> {
        // 1. Berechne Hash des Legal Documents
        let legal_hash = legal_doc.compute_hash();
        
        // 2. Verifiziere dass Code und Document zusammenpassen
        Self::verify_code_document_match(&code, legal_doc)?;
        
        Ok(Self {
            code,
            legal_document_hash: legal_hash,
            jurisdiction,
            legal_venue: Self::default_venue(&jurisdiction),
            dispute_resolution: DisputeResolution::Escalation {
                level_1: Box::new(DisputeResolution::OnChainMediation {
                    mediator_pool: MediatorPool::default(),
                }),
                level_2: Box::new(DisputeResolution::Court {
                    venue: Self::default_venue(&jurisdiction),
                }),
            },
            signatures: vec![],
        })
    }
    
    /// Verifiziert dass alle Code-Variablen im Legal Doc definiert sind
    fn verify_code_document_match(code: &ECLContract, doc: &LegalDocument) -> Result<(), LegalError> {
        let code_variables = code.extract_variables();
        
        for var in code_variables {
            if !doc.definitions.contains_key(&var) {
                return Err(LegalError::UndefinedVariable(var));
            }
        }
        
        Ok(())
    }
    
    /// Prüft ob Transaktion einen Ricardian Contract benötigt
    pub fn is_required(transaction_value: u64, parties: &[DID]) -> bool {
        // > 10000 EUR und mindestens eine Partei ist Enterprise
        transaction_value > 10000 && parties.iter().any(|d| d.is_enterprise())
    }
}
```

### 5.3 Controller Chain

```rust
// erynoa-core/src/legal/controller.rs

/// Controller-Beziehung im DID-Dokument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerChain {
    /// Das kontrollierte DID (z.B. Maschine)
    pub controlled_did: DID,
    
    /// Der Controller (Mensch oder juristische Person)
    pub controller_did: DID,
    
    /// Art der Kontrolle
    pub control_type: ControlType,
    
    /// Verifizierungslevel des Controllers
    pub controller_verification: VerificationLevel,
    
    /// Haftungsgrenzen
    pub liability_cap: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ControlType {
    /// Vollständige Kontrolle (kann alle Aktionen autorisieren)
    Full,
    
    /// Nur für bestimmte Wertgrenzen
    ValueCapped { max_value: u64 },
    
    /// Nur für bestimmte Transaktionstypen
    TypeRestricted { allowed_types: Vec<TransactionType> },
    
    /// Recovery-Only (kann DID wiederherstellen)
    RecoveryOnly,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerificationLevel {
    /// Keine Verifizierung
    None,
    
    /// Email verifiziert
    Email,
    
    /// Telefon verifiziert
    Phone,
    
    /// Basis KYC (ID-Dokument)
    BasicKYC,
    
    /// Erweiterte KYC (Video-Ident)
    EnhancedKYC,
    
    /// Regulierte Entity (z.B. eingetragene GmbH)
    RegulatedEntity,
}

/// Controller Registry
pub struct ControllerRegistry {
    chains: HashMap<DID, ControllerChain>,
}

impl ControllerRegistry {
    /// Prüft ob Transaktion erlaubt ist
    pub fn check_authorization(
        &self,
        actor: &DID,
        transaction: &Transaction,
    ) -> AuthorizationResult {
        // 1. Hole Controller-Chain
        let chain = match self.chains.get(actor) {
            Some(c) => c,
            None => {
                // Kein Controller = selbst verantwortlich
                return AuthorizationResult::SelfControlled;
            }
        };
        
        // 2. Prüfe Controller-Verifizierung vs. Transaction-Value
        let required_level = Self::required_verification_level(transaction.value);
        
        if chain.controller_verification < required_level {
            return AuthorizationResult::InsufficientVerification {
                required: required_level,
                actual: chain.controller_verification,
            };
        }
        
        // 3. Prüfe Control-Type
        match chain.control_type {
            ControlType::Full => AuthorizationResult::Authorized {
                controller: chain.controller_did.clone(),
            },
            ControlType::ValueCapped { max_value } if transaction.value <= max_value => {
                AuthorizationResult::Authorized {
                    controller: chain.controller_did.clone(),
                }
            }
            ControlType::ValueCapped { max_value } => {
                AuthorizationResult::ValueExceeded {
                    max: max_value,
                    attempted: transaction.value,
                }
            }
            ControlType::TypeRestricted { ref allowed_types } 
                if allowed_types.contains(&transaction.tx_type) => {
                AuthorizationResult::Authorized {
                    controller: chain.controller_did.clone(),
                }
            }
            ControlType::TypeRestricted { .. } => {
                AuthorizationResult::TypeNotAllowed {
                    attempted: transaction.tx_type.clone(),
                }
            }
            ControlType::RecoveryOnly => {
                AuthorizationResult::RecoveryOnly
            }
        }
    }
    
    fn required_verification_level(value: u64) -> VerificationLevel {
        match value {
            0..=99 => VerificationLevel::None,
            100..=999 => VerificationLevel::Email,
            1000..=9999 => VerificationLevel::BasicKYC,
            _ => VerificationLevel::EnhancedKYC,
        }
    }
}

#[derive(Debug)]
pub enum AuthorizationResult {
    /// Selbst kontrolliert (kein Controller)
    SelfControlled,
    
    /// Autorisiert durch Controller
    Authorized { controller: DID },
    
    /// Controller-Verifizierung nicht ausreichend
    InsufficientVerification {
        required: VerificationLevel,
        actual: VerificationLevel,
    },
    
    /// Wertgrenze überschritten
    ValueExceeded { max: u64, attempted: u64 },
    
    /// Transaktionstyp nicht erlaubt
    TypeNotAllowed { attempted: TransactionType },
    
    /// Nur Recovery erlaubt
    RecoveryOnly,
}
```

---

# ZUSAMMENFASSUNG: V5.1 HYBRID ARCHITECTURE

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     ERYNOA V5.1: BATTLE-TESTED ARCHITECTURE                  │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                           LAYER STACK                                  │  │
│  │                                                                        │  │
│  │  L5: LEGAL        │ Ricardian + Controller Chain                      │  │
│  │  ────────────────────────────────────────────────────────────────────  │  │
│  │  "Code ist Execution, Humans sind Liability"                          │  │
│  │                                                                        │  │
│  │  L4: MARKET       │ Single-Player → Federated → Public               │  │
│  │  ────────────────────────────────────────────────────────────────────  │  │
│  │  "Erst intern nützlich, dann extern vernetzt"                         │  │
│  │                                                                        │  │
│  │  L3: ANTI-GAMING  │ EigenTrust + Stake-at-Risk                        │  │
│  │  ────────────────────────────────────────────────────────────────────  │  │
│  │  "Topologische + ökonomische Sicherheit"                              │  │
│  │                                                                        │  │
│  │  L2: REALITY      │ PUF Hardware + Multi-Path Witness + Geo-Proof    │  │
│  │  ────────────────────────────────────────────────────────────────────  │  │
│  │  "Physische Welt → Kryptographische Verankerung"                      │  │
│  │                                                                        │  │
│  │  L1: FUZZY        │ Konfidenz-Intervalle + Qualitative Buckets        │  │
│  │  ────────────────────────────────────────────────────────────────────  │  │
│  │  "Keine Pseudopräzision, ehrliche Unsicherheit"                       │  │
│  │                                                                        │  │
│  │  CORE: V5.0       │ Quantum · Category · Topology · Cybernetic        │  │
│  │  ────────────────────────────────────────────────────────────────────  │  │
│  │  "Mathematische Eleganz als Fundament"                                │  │
│  │                                                                        │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                        DESIGN PHILOSOPHY                               │  │
│  │                                                                        │  │
│  │  1. MATHEMATIK:     Präzise im Kern, fuzzy an der Oberfläche          │  │
│  │  2. PHYSIK:         Hardware-verankert, nicht nur Software-signiert   │  │
│  │  3. ÖKONOMIE:       Skin-in-the-Game durch Staking                    │  │
│  │  4. NETZWERK:       Bottom-up Wachstum, nicht Top-down Launch         │  │
│  │  5. RECHT:          Code + Vertrag, Controller + Agent                │  │
│  │                                                                        │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  Status: Production-Ready Architecture                                       │
│  Komplexität: Hoch (aber versteckt hinter Abstraktionen)                    │
│  Zielgruppe: Enterprise-first, Consumer-later                               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

# TEIL II: V5.2 ANTIFRAGILE EXTENSIONS

> **Paradigma:** Second-Order Effects → Antifragilität
> **Prinzip:** Das System wird besser, je mehr es unter Stress gesetzt wird

---

## Übersicht: Die 5 Second-Order Risiken

Die Lösungen aus V5.1 erzeugen neue, subtilere Probleme:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     V5.1 LÖSUNG → V5.2 SECOND-ORDER PROBLEM                  │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                                                                      │    │
│  │  V5.1: EigenTrust + Staking                                         │    │
│  │      ↓                                                               │    │
│  │  V5.2 Problem: REPUTATIONS-ARISTOKRATIE                             │    │
│  │  "Alte, reiche Knoten sind mathematisch uneinholbar"                │    │
│  │                                                                      │    │
│  ├─────────────────────────────────────────────────────────────────────┤    │
│  │                                                                      │    │
│  │  V5.1: Hardware PUFs                                                │    │
│  │      ↓                                                               │    │
│  │  V5.2 Problem: HARDWARE-MONOKULTUR                                  │    │
│  │  "Kompromittierter Chiphersteller = System-Kollaps"                 │    │
│  │                                                                      │    │
│  ├─────────────────────────────────────────────────────────────────────┤    │
│  │                                                                      │    │
│  │  V5.1: Automatisierte Trust-Reaktionen                              │    │
│  │      ↓                                                               │    │
│  │  V5.2 Problem: AGENTEN-PSYCHOSE (Flash Crashes)                     │    │
│  │  "Algorithmische Kaskaden zerstören Netzwerk in Sekunden"           │    │
│  │                                                                      │    │
│  ├─────────────────────────────────────────────────────────────────────┤    │
│  │                                                                      │    │
│  │  V5.1: Transparenz für Trust-Aufbau                                 │    │
│  │      ↓                                                               │    │
│  │  V5.2 Problem: PRIVACY-PARADOXON                                    │    │
│  │  "Je besser Trust funktioniert, desto gläserner die Teilnehmer"     │    │
│  │                                                                      │    │
│  ├─────────────────────────────────────────────────────────────────────┤    │
│  │                                                                      │    │
│  │  V5.1: Kryptographie (Ed25519, BLS)                                 │    │
│  │      ↓                                                               │    │
│  │  V5.2 Problem: QUANTUM THREAT                                       │    │
│  │  "2035: Quantencomputer brechen alle heutigen Signaturen"           │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

# EXTENSION 1: ANTI-CALCIFICATION (Trust Decay + Novelty Bias)

## Problem: Reputations-Aristokratie

Durch EigenTrust und Staking haben alte, etablierte Knoten einen uneinholbaren Vorteil. Das System "verkalkt" und wird innovationsfeindlich.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     DAS TRUST-OLIGOPOL PROBLEM                               │
│                                                                              │
│   Trust                                                                      │
│    ▲                                                                         │
│    │                    ┌─────────────────────────────────────────┐          │
│  1.0├───────────────────│  "WISE" Agents (seit 2026)              │          │
│    │                    │  Mathematisch uneinholbar               │          │
│    │                    └─────────────────────────────────────────┘          │
│    │                                                                         │
│  0.5├─────────────────────────────────────────────────────────────────       │
│    │                                                                         │
│    │        ┌─────────────────────────────────────────┐                      │
│  0.1├───────│  Neue Agenten (2030+)                   │                      │
│    │        │  "Invisible" - niemand interagiert      │                      │
│    │        └─────────────────────────────────────────┘                      │
│    │                                                                         │
│    └────────────────────────────────────────────────────────► Zeit           │
│                                                                              │
│   Risiko: Erynoa wird statisch, neue Innovation wird unterdrückt            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Lösung: Trust-Halbwertszeit + Novelty Multiplier

### 1.1 Architektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     ANTI-CALCIFICATION ENGINE                                │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    TRUST DECAY (Halbwertszeit)                       │    │
│  │                                                                      │    │
│  │   T(t) = T₀ · e^(-λt) + T_active                                    │    │
│  │                                                                      │    │
│  │   ┌────────────────────────────────────────────────────────────┐    │    │
│  │   │  T₀ = Historischer Trust                                   │    │    │
│  │   │  λ = Zerfallskonstante (konfigurierbar pro Shard)          │    │    │
│  │   │  t = Zeit seit letzter Aktivität                           │    │    │
│  │   │  T_active = Frisch bestätigter Trust (zerfällt nicht)      │    │    │
│  │   └────────────────────────────────────────────────────────────┘    │    │
│  │                                                                      │    │
│  │   Halbwertszeit: ~90 Tage (ohne neue Interaktionen)                 │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                         │
│                                    ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    NOVELTY MULTIPLIER                                │    │
│  │                                                                      │    │
│  │   ΔT_transaction = base_delta · novelty_multiplier                  │    │
│  │                                                                      │    │
│  │   novelty_multiplier = {                                            │    │
│  │       3.0  wenn Partner noch nie interagiert ("Exploration")        │    │
│  │       1.5  wenn < 5 gemeinsame Transaktionen                        │    │
│  │       1.0  wenn 5-20 gemeinsame Transaktionen                       │    │
│  │       0.5  wenn > 20 gemeinsame Transaktionen ("Exploitation")      │    │
│  │   }                                                                  │    │
│  │                                                                      │    │
│  │   Effekt: System belohnt Exploration, bestraft Echo-Kammern         │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Implementation

```rust
// erynoa-core/src/antifragile/decay.rs

use std::time::Duration;

/// Anti-Calcification Engine
pub struct AntiCalcificationEngine {
    /// Zerfallskonstante λ (pro Tag)
    decay_constant: f64,
    
    /// Novelty-Multiplikatoren
    novelty_config: NoveltyConfig,
    
    /// Tracking der Interaktions-Historie
    interaction_tracker: InteractionTracker,
}

#[derive(Debug, Clone)]
pub struct NoveltyConfig {
    /// Multiplikator für erste Interaktion mit neuem Partner
    pub first_interaction: f64,      // 3.0
    /// Multiplikator für wenige Interaktionen (< 5)
    pub low_interaction: f64,        // 1.5
    /// Multiplikator für moderate Interaktionen (5-20)
    pub moderate_interaction: f64,   // 1.0
    /// Multiplikator für viele Interaktionen (> 20)
    pub high_interaction: f64,       // 0.5
}

impl Default for NoveltyConfig {
    fn default() -> Self {
        Self {
            first_interaction: 3.0,
            low_interaction: 1.5,
            moderate_interaction: 1.0,
            high_interaction: 0.5,
        }
    }
}

/// Trust-Score mit Zerfall
#[derive(Debug, Clone)]
pub struct DecayingTrust {
    /// Aktiver Trust (zerfällt nicht, da kürzlich bestätigt)
    pub active_trust: f64,
    
    /// Historischer Trust (zerfällt exponentiell)
    pub historical_trust: f64,
    
    /// Zeitpunkt der letzten Aktivität
    pub last_activity: Timestamp,
    
    /// Zeitpunkt der letzten Trust-Bestätigung
    pub last_confirmation: Timestamp,
}

impl AntiCalcificationEngine {
    /// Berechnet den effektiven Trust mit Zerfall
    /// 
    /// T(t) = T_active + T_historical · e^(-λt)
    pub fn compute_effective_trust(&self, trust: &DecayingTrust, now: Timestamp) -> f64 {
        let time_since_activity = (now - trust.last_activity).as_secs_f64() / 86400.0; // in Tagen
        
        // Exponentieller Zerfall des historischen Trusts
        let decayed_historical = trust.historical_trust * (-self.decay_constant * time_since_activity).exp();
        
        // Aktiver Trust zerfällt nicht
        let effective = trust.active_trust + decayed_historical;
        
        // Clamp auf [0, 1]
        effective.clamp(0.0, 1.0)
    }
    
    /// Berechnet die Trust-Halbwertszeit in Tagen
    pub fn half_life_days(&self) -> f64 {
        // t_½ = ln(2) / λ
        0.693 / self.decay_constant
    }
    
    /// Berechnet den Novelty-Multiplikator für eine Transaktion
    pub fn compute_novelty_multiplier(&self, did1: &DID, did2: &DID) -> f64 {
        let interaction_count = self.interaction_tracker.count_interactions(did1, did2);
        
        match interaction_count {
            0 => self.novelty_config.first_interaction,
            1..=4 => self.novelty_config.low_interaction,
            5..=20 => self.novelty_config.moderate_interaction,
            _ => self.novelty_config.high_interaction,
        }
    }
    
    /// Berechnet Trust-Delta für eine Transaktion
    pub fn compute_trust_delta(
        &self,
        base_delta: f64,
        did1: &DID,
        did2: &DID,
        outcome: TransactionOutcome,
    ) -> TrustDelta {
        let novelty = self.compute_novelty_multiplier(did1, did2);
        let outcome_factor = match outcome {
            TransactionOutcome::Success => 1.0,
            TransactionOutcome::Partial => 0.5,
            TransactionOutcome::Failure => -0.5,
            TransactionOutcome::Fraud => -2.0,
        };
        
        let delta = base_delta * novelty * outcome_factor;
        
        TrustDelta {
            value: delta,
            novelty_multiplier: novelty,
            outcome_factor,
            is_exploration: novelty > 1.0,
        }
    }
    
    /// Aktualisiert Trust nach einer Transaktion
    pub fn update_trust(
        &mut self,
        trust: &mut DecayingTrust,
        delta: &TrustDelta,
        now: Timestamp,
    ) {
        // 1. Berechne aktuellen effektiven Trust
        let current = self.compute_effective_trust(trust, now);
        
        // 2. Wende Delta an
        let new_trust = (current + delta.value).clamp(0.0, 1.0);
        
        // 3. Teile in aktiv und historisch auf
        // Frisch bestätigter Trust wird "aktiv", alter Trust wird "historisch"
        let active_portion = delta.value.abs().min(0.2); // Max 20% als aktiv
        
        trust.active_trust = active_portion;
        trust.historical_trust = new_trust - active_portion;
        trust.last_activity = now;
        trust.last_confirmation = now;
        
        // 4. Tracke Interaktion
        self.interaction_tracker.record_interaction(delta);
    }
}

#[derive(Debug, Clone)]
pub struct TrustDelta {
    pub value: f64,
    pub novelty_multiplier: f64,
    pub outcome_factor: f64,
    pub is_exploration: bool,
}

/// Exploration-Score für Discovery
pub struct ExplorationScore {
    engine: AntiCalcificationEngine,
}

impl ExplorationScore {
    /// Berechnet einen Score der Exploration vs. Exploitation balanciert
    /// 
    /// Höherer Score = besser für Discovery-Ranking
    pub fn compute(&self, seeker: &DID, candidate: &DID, candidate_trust: f64) -> f64 {
        let novelty = self.engine.compute_novelty_multiplier(seeker, candidate);
        
        // Balance zwischen Trust und Novelty
        // exploration_score = trust^0.5 · novelty^0.5
        // (Geometrisches Mittel)
        (candidate_trust.sqrt() * novelty.sqrt())
    }
}
```

### 1.3 Konfigurierbare Parameter pro Shard

```rust
/// Shard-spezifische Anti-Calcification Konfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardDecayConfig {
    /// Zerfalls-Halbwertszeit in Tagen
    pub half_life_days: f64,
    
    /// Novelty-Multiplikatoren
    pub novelty_config: NoveltyConfig,
    
    /// Minimaler Trust nach Zerfall (Boden)
    pub decay_floor: f64,
    
    /// Exploration-Bonus in Discovery
    pub exploration_weight: f64,
}

impl Default for ShardDecayConfig {
    fn default() -> Self {
        Self {
            half_life_days: 90.0,        // 3 Monate
            novelty_config: NoveltyConfig::default(),
            decay_floor: 0.1,            // Nie unter 10%
            exploration_weight: 0.3,     // 30% Exploration in Ranking
        }
    }
}

// Verschiedene Presets
impl ShardDecayConfig {
    /// Für schnelllebige Märkte (z.B. Spot-Trading)
    pub fn fast_decay() -> Self {
        Self {
            half_life_days: 30.0,
            novelty_config: NoveltyConfig {
                first_interaction: 5.0,  // Sehr hoher Explorations-Bonus
                ..Default::default()
            },
            decay_floor: 0.05,
            exploration_weight: 0.5,
        }
    }
    
    /// Für langfristige Beziehungen (z.B. Supply Chain)
    pub fn slow_decay() -> Self {
        Self {
            half_life_days: 365.0,       // 1 Jahr
            novelty_config: NoveltyConfig {
                first_interaction: 1.5,  // Moderater Explorations-Bonus
                high_interaction: 0.8,   // Loyalität wird weniger bestraft
                ..Default::default()
            },
            decay_floor: 0.3,
            exploration_weight: 0.1,
        }
    }
}
```

---

# EXTENSION 2: HARDWARE DIVERSITY (Heterogene Witness-Pflicht)

## Problem: Hardware-Monokultur

Wenn alle Geräte Chips vom gleichen Hersteller nutzen und dieser kompromittiert wird, kollabiert das Trust-System ohne dass die Mathematik es bemerkt.

---

## Lösung: Hersteller-Diversitäts-Constraint

### 2.1 Architektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     HARDWARE DIVERSITY ENGINE                                │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    MANUFACTURER REGISTRY                             │    │
│  │                                                                      │    │
│  │   Trusted Manufacturers:                                            │    │
│  │   ┌─────────────────────────────────────────────────────────────┐   │    │
│  │   │ Infineon    │ NXP       │ STMicro    │ Microchip │ Texas   │   │    │
│  │   │ (Germany)   │ (NL)      │ (CH/FR)    │ (USA)     │ Instr.  │   │    │
│  │   └─────────────────────────────────────────────────────────────┘   │    │
│  │                                                                      │    │
│  │   Geopolitische Diversität: EU, USA, Asia (mindestens 2 Regionen)   │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                         │
│                                    ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    WITNESS DIVERSITY CONSTRAINT                      │    │
│  │                                                                      │    │
│  │   Kritischer Event gilt als "wahr" wenn:                            │    │
│  │                                                                      │    │
│  │   ┌────────────────────────────────────────────────────────────┐    │    │
│  │   │                                                             │    │    │
│  │   │  (1) ≥ k Witnesses bestätigen (Standard: k=3)              │    │    │
│  │   │                                                             │    │    │
│  │   │  (2) Witnesses nutzen ≥ m verschiedene Hersteller (m=2)    │    │    │
│  │   │                                                             │    │    │
│  │   │  (3) Witnesses kommen aus ≥ r Regionen (r=2)               │    │    │
│  │   │                                                             │    │    │
│  │   └────────────────────────────────────────────────────────────┘    │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    EXAMPLE: VALID WITNESS SET                        │    │
│  │                                                                      │    │
│  │   Event: "EV Charger delivered 50 kWh"                              │    │
│  │                                                                      │    │
│  │   Witness 1: Smart Meter (Infineon chip, Germany)     ✓             │    │
│  │   Witness 2: Grid Sensor (NXP chip, Netherlands)      ✓             │    │
│  │   Witness 3: EV Charger itself (STMicro, France)      ✓             │    │
│  │                                                                      │    │
│  │   Check: 3 witnesses ✓, 3 manufacturers ✓, 2 regions (EU) ✓        │    │
│  │   Result: VALID (even if one manufacturer is compromised)           │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Implementation

```rust
// erynoa-core/src/antifragile/hardware_diversity.rs

use std::collections::{HashMap, HashSet};

/// Hardware-Hersteller Registry
#[derive(Debug, Clone)]
pub struct ManufacturerRegistry {
    /// Bekannte vertrauenswürdige Hersteller
    trusted_manufacturers: HashMap<ManufacturerId, ManufacturerInfo>,
    
    /// Geopolitische Regionen
    regions: HashMap<ManufacturerId, GeoRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturerInfo {
    pub id: ManufacturerId,
    pub name: String,
    pub country: String,
    pub region: GeoRegion,
    pub trust_level: ManufacturerTrustLevel,
    pub certification: Vec<Certification>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GeoRegion {
    EU,
    USA,
    Asia,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ManufacturerTrustLevel {
    Certified,       // Volle Zertifizierung
    Audited,         // Regelmäßig auditiert
    SelfDeclared,    // Nur Selbstdeklaration
    Unknown,
}

/// Diversitäts-Constraint für Witnessing
#[derive(Debug, Clone)]
pub struct DiversityConstraint {
    /// Minimale Anzahl Witnesses
    pub min_witnesses: u32,
    
    /// Minimale Anzahl verschiedener Hersteller
    pub min_manufacturers: u32,
    
    /// Minimale Anzahl geopolitischer Regionen
    pub min_regions: u32,
    
    /// Minimaler durchschnittlicher Trust-Level der Hersteller
    pub min_manufacturer_trust: ManufacturerTrustLevel,
}

impl Default for DiversityConstraint {
    fn default() -> Self {
        Self {
            min_witnesses: 3,
            min_manufacturers: 2,
            min_regions: 2,
            min_manufacturer_trust: ManufacturerTrustLevel::Audited,
        }
    }
}

/// Hardware Diversity Engine
pub struct HardwareDiversityEngine {
    registry: ManufacturerRegistry,
    constraints: HashMap<EventCriticality, DiversityConstraint>,
}

impl HardwareDiversityEngine {
    /// Prüft ob ein Witness-Set die Diversitäts-Anforderungen erfüllt
    pub fn verify_diversity(
        &self,
        witnesses: &[WitnessInfo],
        criticality: EventCriticality,
    ) -> DiversityResult {
        let constraint = self.constraints.get(&criticality)
            .cloned()
            .unwrap_or_default();
        
        // 1. Zähle unique Hersteller
        let manufacturers: HashSet<_> = witnesses.iter()
            .filter_map(|w| w.hardware_info.as_ref())
            .map(|h| &h.manufacturer_id)
            .collect();
        
        // 2. Zähle unique Regionen
        let regions: HashSet<_> = manufacturers.iter()
            .filter_map(|m| self.registry.regions.get(m))
            .collect();
        
        // 3. Prüfe Constraints
        let witness_count_ok = witnesses.len() as u32 >= constraint.min_witnesses;
        let manufacturer_count_ok = manufacturers.len() as u32 >= constraint.min_manufacturers;
        let region_count_ok = regions.len() as u32 >= constraint.min_regions;
        
        // 4. Prüfe Hersteller-Trust
        let manufacturer_trust_ok = manufacturers.iter().all(|m| {
            self.registry.trusted_manufacturers.get(m)
                .map(|info| info.trust_level >= constraint.min_manufacturer_trust)
                .unwrap_or(false)
        });
        
        let all_ok = witness_count_ok && manufacturer_count_ok && region_count_ok && manufacturer_trust_ok;
        
        DiversityResult {
            valid: all_ok,
            witness_count: witnesses.len() as u32,
            manufacturer_count: manufacturers.len() as u32,
            region_count: regions.len() as u32,
            manufacturers: manufacturers.into_iter().cloned().collect(),
            regions: regions.into_iter().cloned().collect(),
            violations: self.collect_violations(
                &constraint,
                witnesses.len() as u32,
                manufacturers.len() as u32,
                regions.len() as u32,
                manufacturer_trust_ok,
            ),
        }
    }
    
    fn collect_violations(
        &self,
        constraint: &DiversityConstraint,
        witness_count: u32,
        manufacturer_count: u32,
        region_count: u32,
        trust_ok: bool,
    ) -> Vec<DiversityViolation> {
        let mut violations = Vec::new();
        
        if witness_count < constraint.min_witnesses {
            violations.push(DiversityViolation::InsufficientWitnesses {
                required: constraint.min_witnesses,
                actual: witness_count,
            });
        }
        
        if manufacturer_count < constraint.min_manufacturers {
            violations.push(DiversityViolation::InsufficientManufacturers {
                required: constraint.min_manufacturers,
                actual: manufacturer_count,
            });
        }
        
        if region_count < constraint.min_regions {
            violations.push(DiversityViolation::InsufficientRegions {
                required: constraint.min_regions,
                actual: region_count,
            });
        }
        
        if !trust_ok {
            violations.push(DiversityViolation::InsufficientManufacturerTrust);
        }
        
        violations
    }
}

#[derive(Debug)]
pub struct DiversityResult {
    pub valid: bool,
    pub witness_count: u32,
    pub manufacturer_count: u32,
    pub region_count: u32,
    pub manufacturers: Vec<ManufacturerId>,
    pub regions: Vec<GeoRegion>,
    pub violations: Vec<DiversityViolation>,
}

#[derive(Debug)]
pub enum DiversityViolation {
    InsufficientWitnesses { required: u32, actual: u32 },
    InsufficientManufacturers { required: u32, actual: u32 },
    InsufficientRegions { required: u32, actual: u32 },
    InsufficientManufacturerTrust,
}
```

---

# EXTENSION 3: CIRCUIT BREAKERS (Kybernetische Dämpfung)

## Problem: Agenten-Psychose (Trust Flash Crashes)

Algorithmische Reaktionen können Kaskaden auslösen, die das Netzwerk in Sekunden destabilisieren.

---

## Lösung: Dampening + Cooldown + Rate Limiting

### 3.1 Architektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     CIRCUIT BREAKER ENGINE                                   │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    TRUST VELOCITY LIMITER                            │    │
│  │                                                                      │    │
│  │   Max Δ(Trust) pro Zeiteinheit:                                     │    │
│  │                                                                      │    │
│  │   ┌────────────────────────────────────────────────────────────┐    │    │
│  │   │  Individual Agent: max ±10% Trust / Stunde                  │    │    │
│  │   │  Global Network:   max ±5% Durchschnitts-Trust / Stunde     │    │    │
│  │   └────────────────────────────────────────────────────────────┘    │    │
│  │                                                                      │    │
│  │   Effekt: Kein "Flash Crash" möglich                                │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    VOLATILITY MONITOR                                │    │
│  │                                                                      │    │
│  │   Metriken:                                                         │    │
│  │   • abort_rate = Anzahl ABORT Events / Minute                       │    │
│  │   • trust_variance = Varianz der Trust-Änderungen                   │    │
│  │   • cascade_depth = Tiefe der Reaktionsketten                       │    │
│  │                                                                      │    │
│  │   Schwellenwerte:                                                   │    │
│  │   ┌────────────────────────────────────────────────────────────┐    │    │
│  │   │  NORMAL:   abort_rate < 100/min, variance < 0.01            │    │    │
│  │   │  ELEVATED: abort_rate 100-500/min, variance 0.01-0.05       │    │    │
│  │   │  CRITICAL: abort_rate > 500/min, variance > 0.05            │    │    │
│  │   └────────────────────────────────────────────────────────────┘    │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    AUTOMATIC RESPONSES                               │    │
│  │                                                                      │    │
│  │   ELEVATED Level:                                                   │    │
│  │   • Erhöhe Dampening-Faktor (Trust-Änderungen werden gedämpft)      │    │
│  │   • Warne Governance über erhöhte Volatilität                       │    │
│  │                                                                      │    │
│  │   CRITICAL Level (Circuit Breaker TRIP):                            │    │
│  │   • FREEZE: Keine neuen Verhandlungen (SEEK/PROPOSE) für 10 min     │    │
│  │   • CONTINUE: Laufende Streams dürfen weiterlaufen                  │    │
│  │   • ALERT: Governance wird sofort benachrichtigt                    │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Implementation

```rust
// erynoa-core/src/antifragile/circuit_breaker.rs

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use parking_lot::RwLock;

/// Volatilitäts-Level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VolatilityLevel {
    Normal,
    Elevated,
    Critical,
}

/// Circuit Breaker Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Tripped - blocking new operations
    HalfOpen,  // Testing if safe to resume
}

/// Circuit Breaker Engine
pub struct CircuitBreakerEngine {
    /// Aktueller Status
    state: RwLock<CircuitState>,
    
    /// Zeitpunkt des letzten Trips
    last_trip: RwLock<Option<Instant>>,
    
    /// Cooldown-Dauer nach Trip
    cooldown_duration: Duration,
    
    /// Metriken
    metrics: VolatilityMetrics,
    
    /// Konfiguration
    config: CircuitBreakerConfig,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Max Trust-Änderung pro Agent pro Stunde
    pub max_individual_delta_per_hour: f64,
    
    /// Max globale Trust-Änderung pro Stunde
    pub max_global_delta_per_hour: f64,
    
    /// Abort-Rate Schwellenwert für ELEVATED
    pub elevated_abort_threshold: u64,
    
    /// Abort-Rate Schwellenwert für CRITICAL
    pub critical_abort_threshold: u64,
    
    /// Cooldown nach Circuit Break
    pub cooldown_duration: Duration,
    
    /// Half-Open Test-Dauer
    pub half_open_duration: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            max_individual_delta_per_hour: 0.10,  // ±10%
            max_global_delta_per_hour: 0.05,      // ±5%
            elevated_abort_threshold: 100,         // pro Minute
            critical_abort_threshold: 500,
            cooldown_duration: Duration::from_secs(600),  // 10 Minuten
            half_open_duration: Duration::from_secs(60),  // 1 Minute Test
        }
    }
}

/// Volatilitäts-Metriken (Lock-free für Performance)
pub struct VolatilityMetrics {
    /// Abort-Counter für letzte Minute
    abort_count_minute: AtomicU64,
    
    /// Zeitfenster-Start
    window_start: RwLock<Instant>,
    
    /// Trust-Änderungs-Historie
    trust_changes: RwLock<Vec<TrustChange>>,
}

#[derive(Debug, Clone)]
struct TrustChange {
    timestamp: Instant,
    did: DID,
    delta: f64,
}

impl CircuitBreakerEngine {
    /// Prüft ob eine Operation erlaubt ist
    pub fn check_operation(&self, operation: &Operation) -> CircuitCheckResult {
        let state = *self.state.read();
        
        match state {
            CircuitState::Open => {
                // Prüfe ob Cooldown abgelaufen
                if self.is_cooldown_expired() {
                    self.transition_to_half_open();
                    CircuitCheckResult::AllowedWithWarning {
                        warning: "System recovering from volatility event".into(),
                    }
                } else {
                    CircuitCheckResult::Blocked {
                        reason: "Circuit breaker tripped".into(),
                        retry_after: self.time_until_half_open(),
                    }
                }
            }
            CircuitState::HalfOpen => {
                // Limitierte Operationen erlaubt
                match operation {
                    Operation::Seek { .. } | Operation::Propose { .. } => {
                        CircuitCheckResult::AllowedWithWarning {
                            warning: "Limited operations during recovery".into(),
                        }
                    }
                    _ => CircuitCheckResult::Allowed,
                }
            }
            CircuitState::Closed => {
                CircuitCheckResult::Allowed
            }
        }
    }
    
    /// Registriert ein Event und prüft Schwellenwerte
    pub fn record_event(&self, event: &Event) -> Option<CircuitBreakerAction> {
        match event.event_type {
            EventType::Abort => {
                self.metrics.abort_count_minute.fetch_add(1, Ordering::Relaxed);
            }
            EventType::TrustUpdate { did, delta } => {
                self.record_trust_change(&did, delta);
            }
            _ => {}
        }
        
        // Prüfe Volatilitäts-Level
        let level = self.compute_volatility_level();
        
        match level {
            VolatilityLevel::Critical if *self.state.read() != CircuitState::Open => {
                self.trip();
                Some(CircuitBreakerAction::Trip {
                    reason: "Critical volatility detected".into(),
                    metrics: self.get_metrics_snapshot(),
                })
            }
            VolatilityLevel::Elevated => {
                Some(CircuitBreakerAction::Warn {
                    level,
                    metrics: self.get_metrics_snapshot(),
                })
            }
            _ => None,
        }
    }
    
    /// Berechnet das aktuelle Volatilitäts-Level
    fn compute_volatility_level(&self) -> VolatilityLevel {
        let abort_rate = self.get_abort_rate_per_minute();
        let trust_variance = self.compute_trust_variance();
        
        if abort_rate > self.config.critical_abort_threshold || trust_variance > 0.05 {
            VolatilityLevel::Critical
        } else if abort_rate > self.config.elevated_abort_threshold || trust_variance > 0.01 {
            VolatilityLevel::Elevated
        } else {
            VolatilityLevel::Normal
        }
    }
    
    /// Trippt den Circuit Breaker
    fn trip(&self) {
        *self.state.write() = CircuitState::Open;
        *self.last_trip.write() = Some(Instant::now());
        
        // Log & Alert
        tracing::warn!("Circuit breaker TRIPPED - new negotiations frozen");
    }
    
    /// Dampening: Begrenzt Trust-Änderungen
    pub fn apply_dampening(&self, did: &DID, proposed_delta: f64) -> f64 {
        let recent_changes = self.get_recent_changes_for_did(did);
        let total_recent = recent_changes.iter().map(|c| c.delta.abs()).sum::<f64>();
        
        // Berechne verbleibende "Bandbreite"
        let remaining_budget = self.config.max_individual_delta_per_hour - total_recent;
        
        if remaining_budget <= 0.0 {
            // Komplett gedämpft
            0.0
        } else if proposed_delta.abs() > remaining_budget {
            // Teilweise gedämpft
            proposed_delta.signum() * remaining_budget
        } else {
            // Ungedämpft
            proposed_delta
        }
    }
}

#[derive(Debug)]
pub enum CircuitCheckResult {
    Allowed,
    AllowedWithWarning { warning: String },
    Blocked { reason: String, retry_after: Duration },
}

#[derive(Debug)]
pub enum CircuitBreakerAction {
    Trip { reason: String, metrics: MetricsSnapshot },
    Warn { level: VolatilityLevel, metrics: MetricsSnapshot },
}
```

---

# EXTENSION 4: ZERO-KNOWLEDGE REPUTATION (Privacy Layer)

## Problem: Privacy-Paradoxon

Je besser Trust funktioniert, desto gläserner werden die Teilnehmer. Business Intelligence Espionage wird zum Risiko.

---

## Lösung: ZK-Proofs für Reputation

### 4.1 Architektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     ZERO-KNOWLEDGE REPUTATION LAYER                          │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    PRIVATE COMPUTATION                               │    │
│  │                                                                      │    │
│  │   Agent's Local Device:                                             │    │
│  │   ┌────────────────────────────────────────────────────────────┐    │    │
│  │   │                                                             │    │    │
│  │   │  Private Inputs:                                           │    │    │
│  │   │  • Transaction history (wer, was, wann)                    │    │    │
│  │   │  • Partner DIDs                                            │    │    │
│  │   │  • Amounts, frequencies                                    │    │    │
│  │   │                                                             │    │    │
│  │   │           ↓ ZK-Circuit                                     │    │    │
│  │   │                                                             │    │    │
│  │   │  Public Output:                                            │    │    │
│  │   │  • "Reliability > 0.8" (ja/nein)                          │    │    │
│  │   │  • "Total transactions > 100" (ja/nein)                   │    │    │
│  │   │  • Proof π (kryptographischer Beweis)                     │    │    │
│  │   │                                                             │    │    │
│  │   └────────────────────────────────────────────────────────────┘    │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                         │
│                                    ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    PUBLIC VERIFICATION                               │    │
│  │                                                                      │    │
│  │   Anyone can verify:                                                │    │
│  │   ┌────────────────────────────────────────────────────────────┐    │    │
│  │   │                                                             │    │    │
│  │   │  verify(proof π, public_statement) → true/false            │    │    │
│  │   │                                                             │    │    │
│  │   │  "Ja, dieser Agent hat Reliability > 0.8"                  │    │    │
│  │   │  "Nein, ich weiß nicht mit wem er gehandelt hat"           │    │    │
│  │   │                                                             │    │    │
│  │   └────────────────────────────────────────────────────────────┘    │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    PRIVATE SHARDS                                    │    │
│  │                                                                      │    │
│  │   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐          │    │
│  │   │ Enterprise  │     │ Enterprise  │     │ Enterprise  │          │    │
│  │   │ Private     │     │ Private     │     │ Private     │          │    │
│  │   │ Shard A     │     │ Shard B     │     │ Shard C     │          │    │
│  │   └──────┬──────┘     └──────┬──────┘     └──────┬──────┘          │    │
│  │          │                   │                   │                  │    │
│  │          │    Only Hashes    │    Only Hashes    │                  │    │
│  │          └───────────────────┴───────────────────┘                  │    │
│  │                              │                                       │    │
│  │                              ▼                                       │    │
│  │                     ┌─────────────────┐                             │    │
│  │                     │   Public Chain  │                             │    │
│  │                     │   (Commitments) │                             │    │
│  │                     └─────────────────┘                             │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Implementation

```rust
// erynoa-core/src/antifragile/zk_reputation.rs

use ark_ff::PrimeField;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

/// ZK-Reputation Statement (was bewiesen wird)
#[derive(Debug, Clone)]
pub enum ReputationStatement {
    /// "Mein Trust-Score ist mindestens X"
    TrustAtLeast(f64),
    
    /// "Ich habe mindestens N Transaktionen abgeschlossen"
    TransactionCountAtLeast(u64),
    
    /// "Mein Reliability-Score ist mindestens X"
    ReliabilityAtLeast(f64),
    
    /// "Ich bin seit mindestens N Tagen aktiv"
    ActiveForAtLeast(u64),
    
    /// "Ich habe keine Fraud-Events"
    NoFraudEvents,
    
    /// Kombination mehrerer Statements
    And(Vec<ReputationStatement>),
}

/// ZK-Reputation Proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKReputationProof {
    /// Das bewiesene Statement
    pub statement: ReputationStatement,
    
    /// Der kryptographische Beweis
    pub proof: Vec<u8>,
    
    /// Zeitpunkt der Generierung
    pub generated_at: Timestamp,
    
    /// Gültigkeitsdauer
    pub valid_until: Timestamp,
}

/// ZK-Reputation Engine
pub struct ZKReputationEngine {
    /// Proving Key (für Proof-Generierung)
    proving_key: ProvingKey,
    
    /// Verification Key (öffentlich)
    verification_key: VerificationKey,
}

impl ZKReputationEngine {
    /// Generiert einen ZK-Proof für ein Reputation Statement
    pub fn prove(
        &self,
        statement: ReputationStatement,
        private_data: &PrivateReputationData,
    ) -> Result<ZKReputationProof, ZKError> {
        // 1. Erstelle ZK-Circuit
        let circuit = ReputationCircuit::new(&statement, private_data);
        
        // 2. Generiere Proof
        let proof = self.generate_proof(&circuit)?;
        
        // 3. Verpacke
        Ok(ZKReputationProof {
            statement,
            proof,
            generated_at: Timestamp::now(),
            valid_until: Timestamp::now() + Duration::hours(24),
        })
    }
    
    /// Verifiziert einen ZK-Proof
    pub fn verify(&self, proof: &ZKReputationProof) -> Result<bool, ZKError> {
        // 1. Prüfe Gültigkeit
        if Timestamp::now() > proof.valid_until {
            return Ok(false);
        }
        
        // 2. Verifiziere kryptographischen Beweis
        self.verify_proof(&proof.proof, &proof.statement)
    }
}

/// Private Reputations-Daten (werden nie veröffentlicht)
pub struct PrivateReputationData {
    /// Transaktions-Historie
    pub transactions: Vec<PrivateTransaction>,
    
    /// Trust-Scores (6-dimensional)
    pub trust_vector: [f64; 6],
    
    /// Fraud-Events (hoffentlich leer)
    pub fraud_events: Vec<FraudEvent>,
    
    /// Erste Aktivität
    pub first_activity: Timestamp,
}

/// ZK-Circuit für Reputation
struct ReputationCircuit<F: PrimeField> {
    /// Öffentlicher Input: Statement
    statement: ReputationStatement,
    
    /// Privater Input: Daten
    private_data: PrivateReputationData,
    
    _marker: std::marker::PhantomData<F>,
}

impl<F: PrimeField> ConstraintSynthesizer<F> for ReputationCircuit<F> {
    fn generate_constraints(
        self,
        cs: ConstraintSystemRef<F>,
    ) -> Result<(), SynthesisError> {
        match &self.statement {
            ReputationStatement::TrustAtLeast(threshold) => {
                // Berechne Trust aus privaten Daten
                let computed_trust = self.compute_trust_in_circuit(&cs)?;
                
                // Constraint: computed_trust >= threshold
                self.enforce_greater_equal(&cs, computed_trust, *threshold)?;
            }
            ReputationStatement::TransactionCountAtLeast(min_count) => {
                // Zähle Transaktionen (im Circuit)
                let count = cs.new_witness_variable(|| {
                    Ok(F::from(self.private_data.transactions.len() as u64))
                })?;
                
                // Constraint: count >= min_count
                self.enforce_greater_equal_u64(&cs, count, *min_count)?;
            }
            ReputationStatement::NoFraudEvents => {
                // Constraint: fraud_count == 0
                let fraud_count = cs.new_witness_variable(|| {
                    Ok(F::from(self.private_data.fraud_events.len() as u64))
                })?;
                
                cs.enforce_constraint(
                    fraud_count.into(),
                    cs.one().into(),
                    cs.zero().into(),
                )?;
            }
            ReputationStatement::And(statements) => {
                for stmt in statements {
                    // Rekursiv für jedes Sub-Statement
                    let sub_circuit = ReputationCircuit {
                        statement: stmt.clone(),
                        private_data: self.private_data.clone(),
                        _marker: std::marker::PhantomData,
                    };
                    sub_circuit.generate_constraints(cs.clone())?;
                }
            }
            _ => {}
        }
        
        Ok(())
    }
}

/// Selective Disclosure: Zeige nur bestimmte Attribute
pub struct SelectiveDisclosure {
    /// Welche Attribute offengelegt werden
    pub disclosed: Vec<DisclosedAttribute>,
    
    /// ZK-Proof für nicht offengelegte Attribute
    pub proof: ZKReputationProof,
}

#[derive(Debug, Clone)]
pub enum DisclosedAttribute {
    /// DID (Identität)
    DID(DID),
    
    /// Tier (FRESH, EMERGING, etc.)
    Tier(TrustTier),
    
    /// Shard-Mitgliedschaft
    ShardMembership(ShardId),
    
    /// Spezifischer Trust-Score (wenn gewünscht)
    TrustScore(f64),
}
```

---

# EXTENSION 5: POST-QUANTUM CRYPTOGRAPHY

## Problem: Quantum Threat

Ed25519 und BLS werden in 10-15 Jahren durch Quantencomputer gebrochen. Die gesamte Identitäts-Historie wird wertlos.

---

## Lösung: Krypto-Agilität + Hybrid-Signaturen

### 5.1 Architektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     POST-QUANTUM CRYPTOGRAPHY LAYER                          │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    HYBRID SIGNATURE SCHEME                           │    │
│  │                                                                      │    │
│  │   Jede Signatur besteht aus:                                        │    │
│  │   ┌────────────────────────────────────────────────────────────┐    │    │
│  │   │                                                             │    │    │
│  │   │  ┌─────────────────┐   ┌─────────────────┐                 │    │    │
│  │   │  │ Classical Sig   │ + │ Post-Quantum Sig│                 │    │    │
│  │   │  │ (Ed25519)       │   │ (Dilithium-3)   │                 │    │    │
│  │   │  │ 64 bytes        │   │ 2420 bytes      │                 │    │    │
│  │   │  │ Fast            │   │ Slower          │                 │    │    │
│  │   │  │ Small           │   │ Larger          │                 │    │    │
│  │   │  └─────────────────┘   └─────────────────┘                 │    │    │
│  │   │                                                             │    │    │
│  │   └────────────────────────────────────────────────────────────┘    │    │
│  │                                                                      │    │
│  │   Sicherheitsgarantie: Beide müssen gebrochen werden                │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    KEY ROTATION PROTOCOL                             │    │
│  │                                                                      │    │
│  │   ┌─────────────────────────────────────────────────────────────┐   │    │
│  │   │                                                              │   │    │
│  │   │  Old Key (Ed25519)                                          │   │    │
│  │   │       │                                                      │   │    │
│  │   │       │  Signs: "I authorize rotation to new key"           │   │    │
│  │   │       │         + New Public Key                            │   │    │
│  │   │       │         + Trust-History Commitment                  │   │    │
│  │   │       ▼                                                      │   │    │
│  │   │  New Key (Dilithium-3)                                      │   │    │
│  │   │       │                                                      │   │    │
│  │   │       │  Inherits: Full trust history                       │   │    │
│  │   │       │           All credentials                           │   │    │
│  │   │       │           All attestations                          │   │    │
│  │   │       ▼                                                      │   │    │
│  │   │  Continuous Identity                                        │   │    │
│  │   │                                                              │   │    │
│  │   └─────────────────────────────────────────────────────────────┘   │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    CRYPTO AGILITY REGISTRY                           │    │
│  │                                                                      │    │
│  │   Supported Algorithms:                                             │    │
│  │   ┌────────────────────────────────────────────────────────────┐    │    │
│  │   │  Type      │ Classical    │ Post-Quantum    │ Status       │    │    │
│  │   │  ──────────┼──────────────┼─────────────────┼────────────  │    │    │
│  │   │  Signature │ Ed25519      │ Dilithium-3     │ Active       │    │    │
│  │   │  Signature │ ECDSA P-256  │ Falcon-512      │ Supported    │    │    │
│  │   │  KEM       │ X25519       │ Kyber-768       │ Active       │    │    │
│  │   │  Hash      │ SHA-256      │ SHA-3-256       │ Active       │    │    │
│  │   └────────────────────────────────────────────────────────────┘    │    │
│  │                                                                      │    │
│  │   Policy: Neue DIDs MÜSSEN Hybrid unterstützen (ab 2026)            │    │
│  │                                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Implementation

```rust
// erynoa-core/src/antifragile/post_quantum.rs

use pqcrypto_dilithium::dilithium3;
use pqcrypto_kyber::kyber768;
use ed25519_dalek::{SigningKey, VerifyingKey};

/// Hybrid Keypair (Classical + Post-Quantum)
#[derive(Clone)]
pub struct HybridKeypair {
    /// Klassischer Schlüssel (Ed25519)
    pub classical: ClassicalKeypair,
    
    /// Post-Quantum Schlüssel (Dilithium-3)
    pub post_quantum: PostQuantumKeypair,
}

#[derive(Clone)]
pub struct ClassicalKeypair {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

#[derive(Clone)]
pub struct PostQuantumKeypair {
    pub secret_key: dilithium3::SecretKey,
    pub public_key: dilithium3::PublicKey,
}

/// Hybrid Signatur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSignature {
    /// Klassische Signatur (64 bytes)
    pub classical: Vec<u8>,
    
    /// Post-Quantum Signatur (~2420 bytes)
    pub post_quantum: Vec<u8>,
    
    /// Algorithmus-Identifikatoren
    pub algorithms: SignatureAlgorithms,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureAlgorithms {
    pub classical: String,      // "Ed25519"
    pub post_quantum: String,   // "Dilithium3"
}

impl HybridKeypair {
    /// Generiert ein neues Hybrid-Keypair
    pub fn generate() -> Self {
        // 1. Klassischer Schlüssel
        let mut csprng = rand::rngs::OsRng;
        let classical_signing = SigningKey::generate(&mut csprng);
        let classical_verifying = classical_signing.verifying_key();
        
        // 2. Post-Quantum Schlüssel
        let (pq_public, pq_secret) = dilithium3::keypair();
        
        Self {
            classical: ClassicalKeypair {
                signing_key: classical_signing,
                verifying_key: classical_verifying,
            },
            post_quantum: PostQuantumKeypair {
                secret_key: pq_secret,
                public_key: pq_public,
            },
        }
    }
    
    /// Signiert eine Nachricht mit beiden Algorithmen
    pub fn sign(&self, message: &[u8]) -> HybridSignature {
        // 1. Klassische Signatur
        let classical_sig = self.classical.signing_key.sign(message);
        
        // 2. Post-Quantum Signatur
        let pq_sig = dilithium3::sign(message, &self.post_quantum.secret_key);
        
        HybridSignature {
            classical: classical_sig.to_bytes().to_vec(),
            post_quantum: pq_sig.as_bytes().to_vec(),
            algorithms: SignatureAlgorithms {
                classical: "Ed25519".into(),
                post_quantum: "Dilithium3".into(),
            },
        }
    }
    
    /// Verifiziert eine Hybrid-Signatur
    /// 
    /// BEIDE Signaturen müssen gültig sein
    pub fn verify(
        &self,
        message: &[u8],
        signature: &HybridSignature,
    ) -> Result<bool, CryptoError> {
        // 1. Verifiziere klassische Signatur
        let classical_sig = ed25519_dalek::Signature::from_slice(&signature.classical)?;
        let classical_valid = self.classical.verifying_key
            .verify_strict(message, &classical_sig)
            .is_ok();
        
        // 2. Verifiziere Post-Quantum Signatur
        let pq_sig = dilithium3::DetachedSignature::from_bytes(&signature.post_quantum)?;
        let pq_valid = dilithium3::verify_detached_signature(
            &pq_sig,
            message,
            &self.post_quantum.public_key,
        ).is_ok();
        
        // Beide müssen gültig sein
        Ok(classical_valid && pq_valid)
    }
}

/// Key Rotation Service
pub struct KeyRotationService {
    /// Registry der aktuellen Schlüssel
    key_registry: HashMap<DID, HybridKeypair>,
    
    /// Rotations-Historie
    rotation_history: HashMap<DID, Vec<KeyRotationEvent>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationEvent {
    /// Zeitpunkt der Rotation
    pub timestamp: Timestamp,
    
    /// Alter öffentlicher Schlüssel
    pub old_public_key: PublicKeyInfo,
    
    /// Neuer öffentlicher Schlüssel
    pub new_public_key: PublicKeyInfo,
    
    /// Signatur des alten Schlüssels über die Rotation
    pub authorization_signature: HybridSignature,
    
    /// Trust-History Commitment (Hash)
    pub trust_history_commitment: Hash,
}

impl KeyRotationService {
    /// Führt eine Key-Rotation durch
    pub fn rotate_key(
        &mut self,
        did: &DID,
        old_keypair: &HybridKeypair,
        reason: RotationReason,
    ) -> Result<KeyRotationEvent, RotationError> {
        // 1. Generiere neuen Schlüssel
        let new_keypair = HybridKeypair::generate();
        
        // 2. Erstelle Rotations-Message
        let trust_commitment = self.compute_trust_commitment(did)?;
        let rotation_message = KeyRotationMessage {
            did: did.clone(),
            old_public: old_keypair.public_info(),
            new_public: new_keypair.public_info(),
            trust_commitment: trust_commitment.clone(),
            reason,
            timestamp: Timestamp::now(),
        };
        
        // 3. Signiere mit altem Schlüssel
        let message_bytes = rotation_message.to_bytes();
        let authorization = old_keypair.sign(&message_bytes);
        
        // 4. Erstelle Event
        let event = KeyRotationEvent {
            timestamp: rotation_message.timestamp,
            old_public_key: rotation_message.old_public,
            new_public_key: rotation_message.new_public,
            authorization_signature: authorization,
            trust_history_commitment: trust_commitment,
        };
        
        // 5. Speichere neuen Schlüssel
        self.key_registry.insert(did.clone(), new_keypair);
        self.rotation_history
            .entry(did.clone())
            .or_default()
            .push(event.clone());
        
        Ok(event)
    }
    
    /// Verifiziert eine Rotations-Kette (für Audit)
    pub fn verify_rotation_chain(&self, did: &DID) -> Result<bool, RotationError> {
        let history = self.rotation_history.get(did)
            .ok_or(RotationError::NoHistory)?;
        
        for i in 0..history.len() {
            let event = &history[i];
            
            // Rekonstruiere Message
            let message = KeyRotationMessage {
                did: did.clone(),
                old_public: event.old_public_key.clone(),
                new_public: event.new_public_key.clone(),
                trust_commitment: event.trust_history_commitment.clone(),
                reason: RotationReason::Unknown, // Nicht in Event gespeichert
                timestamp: event.timestamp,
            };
            
            // Verifiziere Signatur
            let old_keypair = self.reconstruct_keypair_at_time(did, event.timestamp)?;
            if !old_keypair.verify(&message.to_bytes(), &event.authorization_signature)? {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RotationReason {
    /// Planmäßige Rotation
    Scheduled,
    /// Kompromittierung vermutet
    Compromise,
    /// Upgrade auf neueren Algorithmus
    AlgorithmUpgrade,
    /// Unbekannt (für Verifikation)
    Unknown,
}
```

---

# ZUSAMMENFASSUNG: V5.2 ANTIFRAGILE ARCHITECTURE

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     ERYNOA V5.2: ANTIFRAGILE ARCHITECTURE                    │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                    SECOND-ORDER DEFENSES                               │  │
│  │                                                                        │  │
│  │  E1: ANTI-CALCIFICATION                                               │  │
│  │  ────────────────────────────────────────────────────────────────────  │  │
│  │  "Trust zerfällt exponentiell → Zwingt zu ständiger Innovation"       │  │
│  │  • T(t) = T₀·e^(-λt) + T_active                                       │  │
│  │  • Novelty Multiplier: 3x für neue Partner                            │  │
│  │                                                                        │  │
│  │  E2: HARDWARE DIVERSITY                                               │  │
│  │  ────────────────────────────────────────────────────────────────────  │  │
│  │  "Kein Single-Point-of-Failure durch Chip-Hersteller"                 │  │
│  │  • ≥ k Witnesses von ≥ m Herstellern aus ≥ r Regionen                 │  │
│  │                                                                        │  │
│  │  E3: CIRCUIT BREAKERS                                                 │  │
│  │  ────────────────────────────────────────────────────────────────────  │  │
│  │  "Keine algorithmischen Kaskaden möglich"                             │  │
│  │  • Max ±10% Trust/Stunde pro Agent                                    │  │
│  │  • Automatic Freeze bei kritischer Volatilität                        │  │
│  │                                                                        │  │
│  │  E4: ZERO-KNOWLEDGE REPUTATION                                        │  │
│  │  ────────────────────────────────────────────────────────────────────  │  │
│  │  "Beweise Trust ohne Historie offenzulegen"                           │  │
│  │  • ZK-SNARK: "Trust > 0.8" ohne Partner-DIDs                          │  │
│  │  • Private Shards mit nur Hash-Commitments                            │  │
│  │                                                                        │  │
│  │  E5: POST-QUANTUM CRYPTO                                              │  │
│  │  ────────────────────────────────────────────────────────────────────  │  │
│  │  "Quantum-safe ab Tag 1"                                              │  │
│  │  • Hybrid: Ed25519 + Dilithium-3                                      │  │
│  │  • Key Rotation mit Trust-History-Erhalt                              │  │
│  │                                                                        │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                    ANTIFRAGILITÄT                                      │  │
│  │                                                                        │  │
│  │  Das System wird BESSER unter Stress:                                 │  │
│  │                                                                        │  │
│  │  • Stagnation → Trust-Zerfall → Erzwungene Exploration               │  │
│  │  • Hardware-Hack → Diversitäts-Filter → Isolierter Schaden           │  │
│  │  • Flash Crash → Circuit Breaker → Kontrollierte Abkühlung           │  │
│  │  • Spionage → ZK-Proofs → Geschützte Geschäftsgeheimnisse            │  │
│  │  • Quantum Computer → Hybrid-Crypto → Nahtloses Upgrade              │  │
│  │                                                                        │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  Status: Antifragile, Production-Ready, Future-Proof                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

*Erynoa Robustness Layer V5.2*
*Von produktionsreifer Robustheit zu Antifragilität*
*"What doesn't kill the network makes it stronger"*
