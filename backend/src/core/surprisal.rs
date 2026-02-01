//! # Surprisal Calculator
//!
//! Berechnet Shannon-Surprisal und Trust-gedämpfte Surprisal gemäß Κ15a.
//!
//! ## Axiom-Referenz
//!
//! - **Κ15a (Trust-gedämpfte Surprisal)**: `𝒮(s) = ‖𝕎(s)‖² · ℐ(s)`
//! - **Κ15d (Approximation)**: Count-Min Sketch für ℐ

use crate::domain::unified::TemporalCoord;
use crate::domain::{Event, Surprisal, TrustVector6D};
use std::collections::HashMap;

/// Surprisal Calculator - berechnet Information-Surprisal (Κ15a, Κ15d)
///
/// ```text
///                      ℐ(e|s) = −log₂ P(e | ℂ(s))
///                           │
///                           ▼
///          ┌────────────────────────────────┐
///          │    Count-Min Sketch (Κ15d)    │
///          │    für Frequenz-Schätzung     │
///          └────────────────────────────────┘
///                           │
///                           ▼
///                   𝒮(s) = ‖𝕎‖² · ℐ
/// ```
pub struct SurprisalCalculator {
    /// Count-Min Sketch für Event-Frequenzen
    sketch: CountMinSketch,

    /// Total Events gezählt
    total_count: u64,

    /// Event-Typ Zähler (für genauere Schätzung)
    type_counts: HashMap<String, u64>,
}

impl SurprisalCalculator {
    /// Erstelle neuen SurprisalCalculator
    pub fn new() -> Self {
        Self {
            sketch: CountMinSketch::new(1024, 5), // 1024 buckets, 5 hash functions
            total_count: 0,
            type_counts: HashMap::new(),
        }
    }

    /// Κ15a: Berechne Shannon-Surprisal für ein Event
    ///
    /// ```text
    /// ℐ(e|s) = −log₂ P(e | ℂ(s))
    /// ```
    pub fn calculate_surprisal(&self, event: &Event) -> f64 {
        let event_key = self.event_to_key(event);
        let frequency = self.sketch.estimate(&event_key) as f64;
        let total = self.total_count.max(1) as f64;

        // Laplace smoothing
        let probability = (frequency + 1.0) / (total + 2.0);

        // Shannon-Surprisal in bits
        -probability.log2()
    }

    /// Κ15a: Berechne Trust-gedämpfte Surprisal
    ///
    /// ```text
    /// 𝒮(s) = ‖𝕎(s)‖² · ℐ(s)
    /// ```
    pub fn calculate_dampened_surprisal(&self, event: &Event, trust: &TrustVector6D) -> Surprisal {
        let raw = self.calculate_surprisal(event);
        let norm = trust.weighted_norm(&[1.0; 6]);

        Surprisal {
            raw_bits: raw,
            trust_norm: norm,
            event_id: None,
            computed_at: TemporalCoord::default(),
        }
    }

    /// Beobachte ein Event (update Frequenz-Schätzung)
    pub fn observe(&mut self, event: &Event) {
        let key = self.event_to_key(event);
        self.sketch.increment(&key);
        self.total_count += 1;

        // Update Type-Counter
        let type_key = self.event_type_key(event);
        *self.type_counts.entry(type_key).or_default() += 1;
    }

    /// Berechne Surprisal für Event-Typ
    pub fn type_surprisal(&self, event: &Event) -> f64 {
        let type_key = self.event_type_key(event);
        let frequency = self.type_counts.get(&type_key).copied().unwrap_or(0) as f64;
        let total = self.total_count.max(1) as f64;

        let probability = (frequency + 1.0) / (total + 2.0);
        -probability.log2()
    }

    /// Erzeuge Key für Event (für Sketch)
    fn event_to_key(&self, event: &Event) -> Vec<u8> {
        // Kombiniere Author + Payload-Typ + relevante Felder
        let mut key = Vec::new();
        key.extend(event.author.to_hex().as_bytes());
        key.extend(self.event_type_key(event).as_bytes());
        key
    }

    /// Erzeuge Type-Key für Event
    fn event_type_key(&self, event: &Event) -> String {
        use crate::domain::EventPayload;
        match &event.payload {
            EventPayload::Genesis { .. } => "genesis".to_string(),
            EventPayload::Transfer { asset_type, .. } => format!("transfer:{}", asset_type),
            EventPayload::Mint { asset_type, .. } => format!("mint:{}", asset_type),
            EventPayload::Burn { asset_type, .. } => format!("burn:{}", asset_type),
            EventPayload::Attest { .. } => "attest".to_string(),
            EventPayload::CredentialIssue {
                credential_type, ..
            } => {
                format!("credential:{}", credential_type)
            }
            EventPayload::CredentialRevoke { .. } => "revoke".to_string(),
            EventPayload::Delegate { .. } => "delegate".to_string(),
            EventPayload::DelegationRevoke { .. } => "delegate_revoke".to_string(),
            EventPayload::Proposal { .. } => "proposal".to_string(),
            EventPayload::Vote { .. } => "vote".to_string(),
            EventPayload::SagaStep { action, .. } => format!("saga:{}", action),
            EventPayload::Custom { event_type, .. } => format!("custom:{}", event_type),
            EventPayload::Witness { .. } => "witness".to_string(),
            EventPayload::AnchorConfirm { anchor_system, .. } => format!("anchor:{}", anchor_system),
            EventPayload::TrustUpdate { dimension, .. } => format!("trust_update:{:?}", dimension),
        }
    }

    /// Statistiken
    pub fn stats(&self) -> SurprisalStats {
        SurprisalStats {
            total_observed: self.total_count,
            unique_types: self.type_counts.len(),
            sketch_size: self.sketch.width * self.sketch.depth,
        }
    }
}

impl Default for SurprisalCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistiken des SurprisalCalculator
#[derive(Debug, Clone)]
pub struct SurprisalStats {
    pub total_observed: u64,
    pub unique_types: usize,
    pub sketch_size: usize,
}

/// Count-Min Sketch (Κ15d)
///
/// Probabilistische Datenstruktur für Frequenz-Schätzung
/// mit garantierter Fehlergrenze.
pub struct CountMinSketch {
    /// Counters
    table: Vec<Vec<u64>>,
    /// Breite (Anzahl Buckets pro Row)
    width: usize,
    /// Tiefe (Anzahl Hash-Funktionen)
    depth: usize,
    /// Hash-Seeds
    seeds: Vec<u64>,
}

impl CountMinSketch {
    /// Erstelle neuen Count-Min Sketch
    pub fn new(width: usize, depth: usize) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let mut seeds = Vec::with_capacity(depth);
        let base_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);

        for i in 0..depth {
            // Verwende wrapping_mul für Overflow-sichere Multiplikation
            seeds.push(base_seed.wrapping_add((i as u64).wrapping_mul(0x9E3779B97F4A7C15)));
        }

        Self {
            table: vec![vec![0; width]; depth],
            width,
            depth,
            seeds,
        }
    }

    /// Inkrementiere Counter für Key
    pub fn increment(&mut self, key: &[u8]) {
        for row in 0..self.depth {
            let idx = self.hash(key, row);
            self.table[row][idx] = self.table[row][idx].saturating_add(1);
        }
    }

    /// Schätze Frequenz für Key (Minimum aller Rows)
    pub fn estimate(&self, key: &[u8]) -> u64 {
        let mut min = u64::MAX;
        for row in 0..self.depth {
            let idx = self.hash(key, row);
            min = min.min(self.table[row][idx]);
        }
        min
    }

    /// Hash-Funktion (MurmurHash-inspiriert)
    fn hash(&self, key: &[u8], row: usize) -> usize {
        let seed = self.seeds[row];
        let mut h = seed;

        for &byte in key {
            h = h.wrapping_mul(0x5BD1E995);
            h ^= byte as u64;
            h = h.wrapping_mul(0x5BD1E995);
        }

        h ^= h >> 33;
        h = h.wrapping_mul(0xFF51AFD7ED558CCD);
        h ^= h >> 33;

        (h as usize) % self.width
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{EventPayload, DID};

    #[test]
    fn test_surprisal_calculation() {
        let mut calc = SurprisalCalculator::new();
        let alice = DID::new_self(b"alice");

        // Erstes Event: hohe Surprisal
        let event1 = Event::new(
            alice.clone(),
            EventPayload::Transfer {
                from: alice.clone(),
                to: DID::new_self(b"bob"),
                amount: 100,
                asset_type: "ERY".to_string(),
            },
            vec![],
        );

        let surprisal_first = calc.calculate_surprisal(&event1);

        // Beobachte Event mehrfach
        for _ in 0..10 {
            calc.observe(&event1);
        }

        // Jetzt sollte Surprisal niedriger sein
        let surprisal_after = calc.calculate_surprisal(&event1);
        assert!(surprisal_after < surprisal_first);
    }

    #[test]
    fn test_dampened_surprisal() {
        let calc = SurprisalCalculator::new();
        let alice = DID::new_self(b"alice");

        let event = Event::new(
            alice.clone(),
            EventPayload::Attest {
                subject: DID::new_self(b"bob"),
                claim: "verified".to_string(),
                evidence: None,
            },
            vec![],
        );

        // Hoher Trust → weniger Dämpfung
        let high_trust = TrustVector6D::new(0.9, 0.9, 0.9, 0.9, 0.9, 0.9);
        let dampened_high = calc.calculate_dampened_surprisal(&event, &high_trust);

        // Niedriger Trust → mehr Dämpfung
        let low_trust = TrustVector6D::new(0.3, 0.3, 0.3, 0.3, 0.3, 0.3);
        let dampened_low = calc.calculate_dampened_surprisal(&event, &low_trust);

        // Κ15a: 𝒮 = ‖𝕎‖² · ℐ
        // Bei niedrigerem Trust ist ‖𝕎‖ kleiner, also 𝒮 kleiner
        assert!(dampened_low.dampened() < dampened_high.dampened());
    }

    #[test]
    fn test_count_min_sketch() {
        let mut sketch = CountMinSketch::new(1024, 5);

        let key1 = b"test_key_1";
        let key2 = b"test_key_2";

        // Key1: 100×
        for _ in 0..100 {
            sketch.increment(key1);
        }

        // Key2: 10×
        for _ in 0..10 {
            sketch.increment(key2);
        }

        // Estimates sollten >= tatsächliche Counts sein
        assert!(sketch.estimate(key1) >= 100);
        assert!(sketch.estimate(key2) >= 10);

        // Und nicht zu weit drüber (bei 1024 buckets, 5 depth sehr unwahrscheinlich)
        assert!(sketch.estimate(key1) < 150);
        assert!(sketch.estimate(key2) < 50);
    }
}
