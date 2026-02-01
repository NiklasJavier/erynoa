//! # Information Loss Tracking
//!
//! Tracking von Informationsverlust bei Transformationen gemäß IPS v1.2.0.
//!
//! ## Motivation
//!
//! In verteilten Systemen ist Informationsverlust unvermeidlich:
//! - **Serialisierung**: Nicht alle Rust-Typen lassen sich verlustfrei in JSON/CBOR abbilden
//! - **Kompression**: Storage-Optimierung verliert Redundanz
//! - **Truncation**: API-Responses haben Größenlimits
//! - **Consensus**: Nicht alle Details erreichen alle Nodes
//!
//! ## Design-Prinzipien
//!
//! 1. **Explizite Verfolgung**: Jeder Verlust wird dokumentiert
//! 2. **Kategorisierung**: Nach Channel-Typ und Wiederherstellbarkeit
//! 3. **Quantifizierung**: Loss in Bits oder relative Prozente
//! 4. **Audit-Trail**: Für Debugging und Compliance

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;
use std::time::{Duration, Instant};

// ============================================================================
// ChannelType – Kategorien von Transformationskanälen
// ============================================================================

/// Kategorie des Informationskanals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelType {
    /// ECLVM-Ausführung (Value ↔ Core)
    EclvmExec,
    /// P2P-Gossip (Serialisierung für Netzwerk)
    P2PGossip,
    /// Storage-Persistierung (Disk-Serialisierung)
    StoragePersist,
    /// API-Response (HTTP/gRPC Response)
    ApiResponse,
    /// Consensus-Voting (Aggregation von Stimmen)
    ConsensusVote,
    /// Cross-Realm-Transfer (Realm-übergreifende Kommunikation)
    CrossRealm,
    /// Schema-Migration (Versionsupgrade)
    SchemaMigration,
}

impl ChannelType {
    /// Kurze Beschreibung
    pub fn description(&self) -> &'static str {
        match self {
            Self::EclvmExec => "ECLVM Execution",
            Self::P2PGossip => "P2P Gossip Protocol",
            Self::StoragePersist => "Storage Persistence",
            Self::ApiResponse => "API Response",
            Self::ConsensusVote => "Consensus Voting",
            Self::CrossRealm => "Cross-Realm Transfer",
            Self::SchemaMigration => "Schema Migration",
        }
    }

    /// Ist der Kanal typischerweise verlustfrei?
    pub fn typically_lossless(&self) -> bool {
        matches!(self, Self::StoragePersist | Self::SchemaMigration)
    }
}

impl fmt::Display for ChannelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

// ============================================================================
// LossReason – Gründe für Informationsverlust
// ============================================================================

/// Grund für Informationsverlust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LossReason {
    /// Typ-Konvertierung mit Präzisionsverlust (z.B. f64 → f32)
    PrecisionLoss {
        /// Ursprünglicher Typ
        from_type: String,
        /// Zieltyp
        to_type: String,
    },
    /// Truncation (Längenkürzung)
    Truncation {
        /// Ursprüngliche Länge
        original_len: usize,
        /// Gekürzte Länge
        truncated_len: usize,
    },
    /// Kompression
    Compression {
        /// Kompressionsalgorithmus
        algorithm: String,
        /// Kompressionsverhältnis
        ratio: f64,
    },
    /// Aggregation (mehrere Werte → ein Wert)
    Aggregation {
        /// Anzahl der aggregierten Werte
        input_count: usize,
        /// Art der Aggregation
        aggregation_type: String,
    },
    /// Filter (Werte wurden ausgelassen)
    Filtered {
        /// Anzahl der gefilterten Elemente
        filtered_count: usize,
        /// Filter-Kriterium
        filter_reason: String,
    },
    /// Schema-Downgrade (neueres Schema → älteres Schema)
    SchemaDowngrade {
        /// Ursprüngliche Version
        from_version: u32,
        /// Zielversion
        to_version: u32,
    },
    /// Unbekannter Grund
    Unknown {
        /// Beschreibung
        description: String,
    },
}

impl fmt::Display for LossReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrecisionLoss { from_type, to_type } => {
                write!(f, "Precision loss: {} → {}", from_type, to_type)
            }
            Self::Truncation {
                original_len,
                truncated_len,
            } => {
                write!(
                    f,
                    "Truncation: {} → {} bytes ({:.1}% lost)",
                    original_len,
                    truncated_len,
                    (1.0 - *truncated_len as f64 / *original_len as f64) * 100.0
                )
            }
            Self::Compression { algorithm, ratio } => {
                write!(f, "Compression ({}): {:.2}x", algorithm, ratio)
            }
            Self::Aggregation {
                input_count,
                aggregation_type,
            } => {
                write!(
                    f,
                    "Aggregation ({}): {} inputs",
                    aggregation_type, input_count
                )
            }
            Self::Filtered {
                filtered_count,
                filter_reason,
            } => {
                write!(f, "Filtered {} items: {}", filtered_count, filter_reason)
            }
            Self::SchemaDowngrade {
                from_version,
                to_version,
            } => {
                write!(f, "Schema downgrade: v{} → v{}", from_version, to_version)
            }
            Self::Unknown { description } => {
                write!(f, "Unknown: {}", description)
            }
        }
    }
}

// ============================================================================
// InformationLoss – Einzelner Verlust-Eintrag
// ============================================================================

/// Ein einzelner Informationsverlust-Eintrag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationLoss {
    /// Kanal-Typ
    pub channel: ChannelType,
    /// Geschätzter Verlust in Bits
    pub loss_bits: f64,
    /// Grund für den Verlust
    pub reason: LossReason,
    /// Ist der Verlust wiederherstellbar?
    pub recoverable: bool,
    /// Kontext-Information (z.B. betroffenes Feld)
    pub context: Option<String>,
}

impl InformationLoss {
    /// Erstelle neuen InformationLoss-Eintrag
    pub fn new(channel: ChannelType, loss_bits: f64, reason: LossReason) -> Self {
        Self {
            channel,
            loss_bits,
            reason,
            recoverable: false,
            context: None,
        }
    }

    /// Mit Wiederherstellbarkeits-Flag
    pub fn with_recoverable(mut self, recoverable: bool) -> Self {
        self.recoverable = recoverable;
        self
    }

    /// Mit Kontext
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Precision Loss Factory
    pub fn precision(from_type: &str, to_type: &str, bits_lost: f64) -> Self {
        Self::new(
            ChannelType::EclvmExec,
            bits_lost,
            LossReason::PrecisionLoss {
                from_type: from_type.to_string(),
                to_type: to_type.to_string(),
            },
        )
    }

    /// Truncation Factory
    pub fn truncation(channel: ChannelType, original: usize, truncated: usize) -> Self {
        let bits_lost = ((original - truncated) * 8) as f64;
        Self::new(
            channel,
            bits_lost,
            LossReason::Truncation {
                original_len: original,
                truncated_len: truncated,
            },
        )
    }

    /// Compression Factory
    pub fn compression(algorithm: &str, original_size: usize, compressed_size: usize) -> Self {
        let ratio = original_size as f64 / compressed_size as f64;
        // Bei Kompression geht keine Information verloren, nur Redundanz
        // Aber wir tracken es für Audit-Zwecke
        Self::new(
            ChannelType::StoragePersist,
            0.0, // Verlustfreie Kompression
            LossReason::Compression {
                algorithm: algorithm.to_string(),
                ratio,
            },
        )
        .with_recoverable(true)
    }

    /// Filter Factory
    pub fn filtered(channel: ChannelType, count: usize, reason: &str) -> Self {
        Self::new(
            channel,
            0.0, // Bits müssten pro Element geschätzt werden
            LossReason::Filtered {
                filtered_count: count,
                filter_reason: reason.to_string(),
            },
        )
    }
}

impl fmt::Display for InformationLoss {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} ({:.1} bits{})",
            self.channel,
            self.reason,
            self.loss_bits,
            if self.recoverable {
                ", recoverable"
            } else {
                ""
            }
        )?;
        if let Some(ctx) = &self.context {
            write!(f, " @ {}", ctx)?;
        }
        Ok(())
    }
}

// ============================================================================
// CompressionRecord – Audit-Trail für Kompression
// ============================================================================

/// Record für Kompressions-Audit
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompressionRecord {
    /// Hash der Originaldaten
    pub original_hash: [u8; 32],
    /// Hash der komprimierten Daten
    pub compressed_hash: [u8; 32],
    /// Ist der Merkle-Root erhalten?
    pub merkle_root_preserved: bool,
    /// Kompressionsalgorithmus
    pub algorithm: String,
    /// Originalgröße in Bytes
    pub original_size: usize,
    /// Komprimierte Größe in Bytes
    pub compressed_size: usize,
}

impl CompressionRecord {
    /// Erstelle neuen CompressionRecord
    pub fn new(
        original_hash: [u8; 32],
        compressed_hash: [u8; 32],
        algorithm: impl Into<String>,
        original_size: usize,
        compressed_size: usize,
    ) -> Self {
        Self {
            original_hash,
            compressed_hash,
            merkle_root_preserved: true,
            algorithm: algorithm.into(),
            original_size,
            compressed_size,
        }
    }

    /// Kompressionsverhältnis
    pub fn ratio(&self) -> f64 {
        if self.compressed_size == 0 {
            0.0
        } else {
            self.original_size as f64 / self.compressed_size as f64
        }
    }

    /// Platzersparnis in Prozent
    pub fn savings_percent(&self) -> f64 {
        if self.original_size == 0 {
            0.0
        } else {
            (1.0 - self.compressed_size as f64 / self.original_size as f64) * 100.0
        }
    }
}

// ============================================================================
// LossTracker – Aggregierter Tracker
// ============================================================================

/// Tracker für Informationsverluste in einer Operation
#[derive(Debug, Clone, Default)]
pub struct LossTracker {
    /// Alle aufgezeichneten Verluste
    losses: Vec<InformationLoss>,
    /// Kompressions-Records
    compressions: Vec<CompressionRecord>,
    /// Start-Zeit
    start_time: Option<Instant>,
}

impl LossTracker {
    /// Erstelle neuen Tracker
    pub fn new() -> Self {
        Self {
            losses: Vec::new(),
            compressions: Vec::new(),
            start_time: Some(Instant::now()),
        }
    }

    /// Füge Verlust hinzu
    pub fn record(&mut self, loss: InformationLoss) {
        self.losses.push(loss);
    }

    /// Füge Kompression hinzu
    pub fn record_compression(&mut self, record: CompressionRecord) {
        self.compressions.push(record);
    }

    /// Gesamtverlust in Bits
    pub fn total_loss_bits(&self) -> f64 {
        self.losses.iter().map(|l| l.loss_bits).sum()
    }

    /// Nicht-wiederherstellbarer Verlust in Bits
    pub fn unrecoverable_loss_bits(&self) -> f64 {
        self.losses
            .iter()
            .filter(|l| !l.recoverable)
            .map(|l| l.loss_bits)
            .sum()
    }

    /// Verluste nach Channel
    pub fn by_channel(&self, channel: ChannelType) -> Vec<&InformationLoss> {
        self.losses
            .iter()
            .filter(|l| l.channel == channel)
            .collect()
    }

    /// Hat kritische (nicht-wiederherstellbare) Verluste?
    pub fn has_critical_loss(&self) -> bool {
        self.losses
            .iter()
            .any(|l| !l.recoverable && l.loss_bits > 0.0)
    }

    /// Alle Verluste
    pub fn losses(&self) -> &[InformationLoss] {
        &self.losses
    }

    /// Alle Kompressions-Records
    pub fn compressions(&self) -> &[CompressionRecord] {
        &self.compressions
    }

    /// Tracking-Dauer
    pub fn duration(&self) -> Option<Duration> {
        self.start_time.map(|t| t.elapsed())
    }

    /// Zusammenfassung erstellen
    pub fn summary(&self) -> LossSummary {
        LossSummary {
            total_loss_bits: self.total_loss_bits(),
            unrecoverable_loss_bits: self.unrecoverable_loss_bits(),
            loss_count: self.losses.len(),
            compression_count: self.compressions.len(),
            has_critical: self.has_critical_loss(),
            duration: self.duration(),
        }
    }
}

/// Zusammenfassung der Verluste
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LossSummary {
    /// Gesamtverlust in Bits
    pub total_loss_bits: f64,
    /// Nicht-wiederherstellbarer Verlust
    pub unrecoverable_loss_bits: f64,
    /// Anzahl der Verlust-Events
    pub loss_count: usize,
    /// Anzahl der Kompressions-Records
    pub compression_count: usize,
    /// Hat kritische Verluste?
    pub has_critical: bool,
    /// Dauer des Trackings
    #[serde(skip)]
    pub duration: Option<Duration>,
}

impl fmt::Display for LossSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Loss Summary: {:.1} bits total ({:.1} unrecoverable), {} events",
            self.total_loss_bits, self.unrecoverable_loss_bits, self.loss_count
        )?;
        if self.has_critical {
            write!(f, " [CRITICAL]")?;
        }
        Ok(())
    }
}

// ============================================================================
// Global Loss Registry (für Langzeit-Monitoring)
// ============================================================================

/// Globales Registry für Langzeit-Monitoring (Thread-Local)
pub struct LossRegistry {
    /// Letzte N Verluste (Ring-Buffer)
    recent_losses: VecDeque<InformationLoss>,
    /// Maximale Größe
    max_size: usize,
    /// Gesamtzähler
    total_count: u64,
    /// Gesamtverlust in Bits
    total_bits: f64,
}

impl LossRegistry {
    /// Erstelle Registry mit gegebener Kapazität
    pub fn new(max_size: usize) -> Self {
        Self {
            recent_losses: VecDeque::with_capacity(max_size),
            max_size,
            total_count: 0,
            total_bits: 0.0,
        }
    }

    /// Registriere Verlust
    pub fn register(&mut self, loss: InformationLoss) {
        self.total_count += 1;
        self.total_bits += loss.loss_bits;

        if self.recent_losses.len() >= self.max_size {
            self.recent_losses.pop_front();
        }
        self.recent_losses.push_back(loss);
    }

    /// Letzte N Verluste
    pub fn recent(&self, n: usize) -> impl Iterator<Item = &InformationLoss> {
        self.recent_losses.iter().rev().take(n)
    }

    /// Statistiken
    pub fn stats(&self) -> (u64, f64) {
        (self.total_count, self.total_bits)
    }
}

impl Default for LossRegistry {
    fn default() -> Self {
        Self::new(1000)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_information_loss_creation() {
        let loss = InformationLoss::precision("f64", "f32", 32.0)
            .with_context("trust.reliability")
            .with_recoverable(false);

        assert_eq!(loss.channel, ChannelType::EclvmExec);
        assert_eq!(loss.loss_bits, 32.0);
        assert!(!loss.recoverable);
        assert_eq!(loss.context, Some("trust.reliability".to_string()));
    }

    #[test]
    fn test_truncation_loss() {
        let loss = InformationLoss::truncation(ChannelType::ApiResponse, 1000, 100);

        assert_eq!(loss.channel, ChannelType::ApiResponse);
        assert_eq!(loss.loss_bits, (900 * 8) as f64);

        if let LossReason::Truncation {
            original_len,
            truncated_len,
        } = &loss.reason
        {
            assert_eq!(*original_len, 1000);
            assert_eq!(*truncated_len, 100);
        } else {
            panic!("Expected Truncation reason");
        }
    }

    #[test]
    fn test_compression_record() {
        let record = CompressionRecord::new([1u8; 32], [2u8; 32], "zstd", 1000, 250);

        assert_eq!(record.ratio(), 4.0);
        assert_eq!(record.savings_percent(), 75.0);
        assert!(record.merkle_root_preserved);
    }

    #[test]
    fn test_loss_tracker() {
        let mut tracker = LossTracker::new();

        tracker.record(InformationLoss::precision("f64", "f32", 32.0));
        tracker.record(
            InformationLoss::truncation(ChannelType::ApiResponse, 1000, 900).with_recoverable(true),
        );

        assert_eq!(tracker.total_loss_bits(), 32.0 + 800.0);
        assert_eq!(tracker.unrecoverable_loss_bits(), 32.0);
        assert!(tracker.has_critical_loss());
    }

    #[test]
    fn test_loss_tracker_summary() {
        let mut tracker = LossTracker::new();

        tracker.record(InformationLoss::precision("f64", "f32", 32.0));

        let summary = tracker.summary();
        assert_eq!(summary.loss_count, 1);
        assert_eq!(summary.total_loss_bits, 32.0);
        assert!(summary.has_critical);
    }

    #[test]
    fn test_loss_registry() {
        let mut registry = LossRegistry::new(3);

        registry.register(InformationLoss::precision("a", "b", 10.0));
        registry.register(InformationLoss::precision("c", "d", 20.0));
        registry.register(InformationLoss::precision("e", "f", 30.0));
        registry.register(InformationLoss::precision("g", "h", 40.0));

        let (count, bits) = registry.stats();
        assert_eq!(count, 4);
        assert_eq!(bits, 100.0);

        // Ring-Buffer sollte nur 3 behalten
        assert_eq!(registry.recent_losses.len(), 3);
    }

    #[test]
    fn test_channel_type_properties() {
        assert!(ChannelType::StoragePersist.typically_lossless());
        assert!(!ChannelType::ApiResponse.typically_lossless());
    }

    #[test]
    fn test_loss_display() {
        let loss = InformationLoss::precision("f64", "f32", 32.0).with_context("field.x");

        let display = format!("{}", loss);
        assert!(display.contains("ECLVM"));
        assert!(display.contains("32.0 bits"));
        assert!(display.contains("field.x"));
    }
}
