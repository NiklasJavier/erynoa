//! # Cold Storage / Archive (Priorität 3)
//!
//! ψ_archive Morphismus für Event-Archivierung mit Merkle-Root Preservation.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                         ARCHIVE LAYER (ψ_archive)                       │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                                                                         │
//! │  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐          │
//! │  │  Hot Store   │ ──→  │  Archive     │ ──→  │  Cold Store  │          │
//! │  │  (Recent)    │      │  Processor   │      │  (Finalized) │          │
//! │  └──────────────┘      └──────────────┘      └──────────────┘          │
//! │         │                     │                     │                   │
//! │         │              ┌──────┴──────┐              │                   │
//! │         │              │ Merkle Tree │              │                   │
//! │         │              │  Builder    │              │                   │
//! │         │              └──────┬──────┘              │                   │
//! │         │                     │                     │                   │
//! │  ┌──────┴─────────────────────┴─────────────────────┴──────┐           │
//! │  │                    Archive Index                         │           │
//! │  │  • Merkle Roots per Epoch                               │           │
//! │  │  • Event Count / Size Stats                             │           │
//! │  │  • Retrieval Proofs                                     │           │
//! │  └──────────────────────────────────────────────────────────┘           │
//! │                                                                         │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Axiom-Referenz
//!
//! - **Κ9 (Kausale Struktur)**: Archivierte Events behalten ihre kausale Ordnung
//! - **Κ10 (Finalität)**: Nur finalisierte Events werden archiviert
//! - **IPS §IV.1**: ψ_archive Morphismus erhält Merkle-Root

use crate::domain::{Event, EventId, FinalityLevel, Hash32};
use crate::local::kv_store::KvStore;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use thiserror::Error;

/// Archive-Fehler
#[derive(Debug, Error)]
pub enum ArchiveError {
    #[error("Event not finalized: {event_id}")]
    NotFinalized { event_id: String },

    #[error("Epoch not found: {epoch}")]
    EpochNotFound { epoch: u64 },

    #[error("Merkle proof invalid for event {event_id}")]
    InvalidProof { event_id: String },

    #[error("Archive corrupted: {reason}")]
    Corrupted { reason: String },

    #[error("Storage error: {0}")]
    Storage(#[from] anyhow::Error),
}

/// Archive-Resultat
pub type ArchiveResult<T> = Result<T, ArchiveError>;

/// Archivierungs-Konfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveConfig {
    /// Events älter als diese Dauer werden archiviert
    pub archive_after_days: u32,

    /// Epoch-Größe (Events pro Merkle-Baum)
    pub epoch_size: u32,

    /// Minimum Finalitätslevel für Archivierung
    pub min_finality: FinalityLevel,

    /// Komprimierung aktivieren
    pub enable_compression: bool,

    /// Automatische Archivierung im Hintergrund
    pub auto_archive: bool,

    /// Archivierungs-Intervall (Sekunden)
    pub archive_interval_secs: u64,
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        Self {
            archive_after_days: 90,
            epoch_size: 10_000,
            min_finality: FinalityLevel::Witnessed, // Witnessed als guter Kompromiss
            enable_compression: true,
            auto_archive: true,
            archive_interval_secs: 3600, // 1h
        }
    }
}

/// Merkle-Baum-Knoten
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNode {
    /// Hash dieses Knotens
    pub hash: Hash32,

    /// Linker Kind-Hash (None für Blätter)
    pub left: Option<Box<MerkleNode>>,

    /// Rechter Kind-Hash (None für Blätter)
    pub right: Option<Box<MerkleNode>>,
}

impl MerkleNode {
    /// Erstelle Blatt-Knoten aus Event-Hash
    pub fn leaf(event_hash: Hash32) -> Self {
        Self {
            hash: event_hash,
            left: None,
            right: None,
        }
    }

    /// Erstelle inneren Knoten aus zwei Kindern
    pub fn branch(left: MerkleNode, right: MerkleNode) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(left.hash.as_bytes());
        hasher.update(right.hash.as_bytes());
        let hash_bytes: [u8; 32] = hasher.finalize().into();

        Self {
            hash: Hash32::from_slice(&hash_bytes).unwrap_or(Hash32::NULL),
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }

    /// Ist dies ein Blatt?
    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

/// Merkle-Beweis für ein archiviertes Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Event-Hash (Blatt)
    pub event_hash: Hash32,

    /// Pfad zur Wurzel (Sibling-Hashes mit Richtung)
    pub path: Vec<(Hash32, bool)>, // (sibling_hash, is_left)

    /// Epoch des Events
    pub epoch: u64,

    /// Merkle-Root der Epoch
    pub root: Hash32,
}

impl MerkleProof {
    /// Verifiziere den Beweis
    pub fn verify(&self) -> bool {
        let mut current_hash = self.event_hash;

        for (sibling, is_left) in &self.path {
            let mut hasher = Sha256::new();
            if *is_left {
                hasher.update(sibling.as_bytes());
                hasher.update(current_hash.as_bytes());
            } else {
                hasher.update(current_hash.as_bytes());
                hasher.update(sibling.as_bytes());
            }
            let hash_bytes: [u8; 32] = hasher.finalize().into();
            current_hash = Hash32::from_slice(&hash_bytes).unwrap_or(Hash32::NULL);
        }

        current_hash == self.root
    }
}

/// Epoch-Metadaten
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochMetadata {
    /// Epoch-Nummer
    pub epoch: u64,

    /// Merkle-Root dieser Epoch
    pub merkle_root: Hash32,

    /// Anzahl Events in dieser Epoch
    pub event_count: u32,

    /// Gesamtgröße der Events (Bytes)
    pub total_size: u64,

    /// Zeitstempel des ersten Events (Unix-Timestamp ms)
    pub first_event_time: u64,

    /// Zeitstempel des letzten Events (Unix-Timestamp ms)
    pub last_event_time: u64,

    /// Archivierungszeitpunkt
    pub archived_at: DateTime<Utc>,

    /// Komprimiert?
    pub compressed: bool,
}

/// Cold Storage Archive
///
/// Verwaltet archivierte Events mit Merkle-Root Preservation.
pub struct Archive {
    /// Konfiguration
    config: ArchiveConfig,

    /// Hot-Store Referenz (für Event-Zugriff)
    #[allow(dead_code)]
    hot_store: KvStore,

    /// Cold-Store (archivierte Events)
    cold_store: KvStore,

    /// Epoch-Index
    epochs: BTreeMap<u64, EpochMetadata>,

    /// Aktuelle Epoch (noch nicht archiviert)
    current_epoch: u64,

    /// Events in aktueller Epoch
    #[allow(dead_code)]
    current_epoch_events: Vec<EventId>,
}

impl Archive {
    /// Erstelle neues Archive
    pub fn new(
        config: ArchiveConfig,
        hot_store: KvStore,
        cold_store: KvStore,
    ) -> ArchiveResult<Self> {
        Ok(Self {
            config,
            hot_store,
            cold_store,
            epochs: BTreeMap::new(),
            current_epoch: 0,
            current_epoch_events: Vec::new(),
        })
    }

    /// ψ_archive Morphismus: Archiviere finalisierte Events
    ///
    /// Verschiebt Events vom Hot-Store in den Cold-Store und
    /// erstellt einen Merkle-Baum für die Epoch.
    pub fn archive_events(&mut self, events: Vec<Event>) -> ArchiveResult<EpochMetadata> {
        // Prüfe Finalität
        for event in &events {
            if event.finality.level < self.config.min_finality {
                return Err(ArchiveError::NotFinalized {
                    event_id: event.id.to_string(),
                });
            }
        }

        if events.is_empty() {
            return Err(ArchiveError::Corrupted {
                reason: "No events to archive".to_string(),
            });
        }

        // Berechne Event-Hashes (verwende Event-ID als Hash)
        let event_hashes: Vec<Hash32> = events
            .iter()
            .map(|e| {
                // Event-ID ist bereits ein Content-Hash
                let id_bytes = e.id.as_bytes();
                Hash32::from_slice(id_bytes).unwrap_or(Hash32::NULL)
            })
            .collect();

        // Baue Merkle-Baum
        let merkle_root = Self::build_merkle_tree(&event_hashes);

        // Berechne Statistiken
        let total_size: u64 = events.iter().map(|e| std::mem::size_of_val(e) as u64).sum();

        let first_event_time = events.first().map(|e| e.timestamp()).unwrap_or(0);

        let last_event_time = events.last().map(|e| e.timestamp()).unwrap_or(0);

        // Erstelle Epoch-Metadaten
        let epoch = self.current_epoch;
        let metadata = EpochMetadata {
            epoch,
            merkle_root,
            event_count: events.len() as u32,
            total_size,
            first_event_time,
            last_event_time,
            archived_at: Utc::now(),
            compressed: self.config.enable_compression,
        };

        // Speichere Events im Cold-Store
        for event in &events {
            let key = format!("archive:epoch:{}:event:{}", epoch, event.id);
            let value = serde_json::to_vec(event).map_err(|e| ArchiveError::Corrupted {
                reason: e.to_string(),
            })?;
            self.cold_store
                .put(&key, &value)
                .map_err(|e| ArchiveError::Storage(e.into()))?;
        }

        // Speichere Epoch-Metadaten
        let meta_key = format!("archive:epoch:{}:meta", epoch);
        let meta_value = serde_json::to_vec(&metadata).map_err(|e| ArchiveError::Corrupted {
            reason: e.to_string(),
        })?;
        self.cold_store
            .put(&meta_key, &meta_value)
            .map_err(|e| ArchiveError::Storage(e.into()))?;

        // Aktualisiere Index
        self.epochs.insert(epoch, metadata.clone());
        self.current_epoch += 1;
        self.current_epoch_events.clear();

        Ok(metadata)
    }

    /// Baue Merkle-Baum aus Event-Hashes
    fn build_merkle_tree(hashes: &[Hash32]) -> Hash32 {
        if hashes.is_empty() {
            return Hash32::NULL;
        }

        if hashes.len() == 1 {
            return hashes[0];
        }

        // Erstelle Blätter
        let mut nodes: Vec<MerkleNode> = hashes.iter().map(|h| MerkleNode::leaf(*h)).collect();

        // Padding auf Zweierpotenz
        while nodes.len().count_ones() != 1 {
            nodes.push(MerkleNode::leaf(Hash32::NULL));
        }

        // Baue Baum von unten nach oben
        while nodes.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in nodes.chunks(2) {
                let left = chunk[0].clone();
                let right = chunk.get(1).cloned().unwrap_or_else(|| left.clone());
                next_level.push(MerkleNode::branch(left, right));
            }
            nodes = next_level;
        }

        nodes[0].hash
    }

    /// Erstelle Merkle-Beweis für ein Event
    pub fn create_proof(&self, event_id: &EventId, epoch: u64) -> ArchiveResult<MerkleProof> {
        let metadata = self
            .epochs
            .get(&epoch)
            .ok_or(ArchiveError::EpochNotFound { epoch })?;

        // Lade Event
        let key = format!("archive:epoch:{}:event:{}", epoch, event_id);
        let event_data: Vec<u8> = self
            .cold_store
            .get(&key)
            .map_err(|e| ArchiveError::Storage(e.into()))?
            .ok_or_else(|| ArchiveError::Corrupted {
                reason: format!("Event {} not found in epoch {}", event_id, epoch),
            })?;

        let event: Event =
            serde_json::from_slice(&event_data).map_err(|e| ArchiveError::Corrupted {
                reason: e.to_string(),
            })?;

        // Event-Hash aus ID
        let event_hash = Hash32::from_slice(event.id.as_bytes()).unwrap_or(Hash32::NULL);

        // Für einen vollständigen Beweis müssten wir alle Events der Epoch laden
        // und den Merkle-Pfad rekonstruieren. Hier vereinfacht:
        let proof = MerkleProof {
            event_hash,
            path: Vec::new(), // TODO: Vollständiger Pfad
            epoch,
            root: metadata.merkle_root,
        };

        Ok(proof)
    }

    /// Lade archiviertes Event
    pub fn get_archived_event(&self, event_id: &EventId, epoch: u64) -> ArchiveResult<Event> {
        let key = format!("archive:epoch:{}:event:{}", epoch, event_id);
        let event_data: Vec<u8> = self
            .cold_store
            .get(&key)
            .map_err(|e| ArchiveError::Storage(e.into()))?
            .ok_or_else(|| ArchiveError::Corrupted {
                reason: format!("Event {} not found", event_id),
            })?;

        serde_json::from_slice(&event_data).map_err(|e| ArchiveError::Corrupted {
            reason: e.to_string(),
        })
    }

    /// Liste aller Epochs
    pub fn list_epochs(&self) -> Vec<&EpochMetadata> {
        self.epochs.values().collect()
    }

    /// Statistiken
    pub fn stats(&self) -> ArchiveStats {
        let total_events: u32 = self.epochs.values().map(|e| e.event_count).sum();
        let total_size: u64 = self.epochs.values().map(|e| e.total_size).sum();

        ArchiveStats {
            epoch_count: self.epochs.len() as u64,
            total_events,
            total_size,
            oldest_epoch: self.epochs.keys().next().copied(),
            newest_epoch: self.epochs.keys().last().copied(),
        }
    }

    /// Prüfe ob Event archivierbar ist (alt genug + finalisiert)
    pub fn is_archivable(&self, event: &Event) -> bool {
        let now_ms = Utc::now().timestamp_millis() as u64;
        let event_ts = event.timestamp();
        let age_days = (now_ms.saturating_sub(event_ts)) / (24 * 60 * 60 * 1000);

        age_days >= self.config.archive_after_days as u64
            && event.finality.level >= self.config.min_finality
    }
}

/// Archive-Statistiken
#[derive(Debug, Clone)]
pub struct ArchiveStats {
    pub epoch_count: u64,
    pub total_events: u32,
    pub total_size: u64,
    pub oldest_epoch: Option<u64>,
    pub newest_epoch: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree_single() {
        let hash = Hash32::from_slice(&[1u8; 32]).unwrap();
        let root = Archive::build_merkle_tree(&[hash]);
        assert_eq!(root, hash);
    }

    #[test]
    fn test_merkle_tree_two() {
        let h1 = Hash32::from_slice(&[1u8; 32]).unwrap();
        let h2 = Hash32::from_slice(&[2u8; 32]).unwrap();
        let root = Archive::build_merkle_tree(&[h1, h2]);

        // Root sollte Hash von h1 || h2 sein
        let mut hasher = Sha256::new();
        hasher.update(h1.as_bytes());
        hasher.update(h2.as_bytes());
        let expected: [u8; 32] = hasher.finalize().into();

        assert_eq!(root, Hash32::from_slice(&expected).unwrap());
    }

    #[test]
    fn test_merkle_proof_verify_empty() {
        let proof = MerkleProof {
            event_hash: Hash32::from_slice(&[1u8; 32]).unwrap(),
            path: vec![],
            epoch: 0,
            root: Hash32::from_slice(&[1u8; 32]).unwrap(),
        };

        // Leerer Pfad: event_hash == root
        assert!(proof.verify());
    }

    #[test]
    fn test_merkle_proof_verify_single_sibling() {
        let h1 = Hash32::from_slice(&[1u8; 32]).unwrap();
        let h2 = Hash32::from_slice(&[2u8; 32]).unwrap();

        // Berechne erwartete Root
        let mut hasher = Sha256::new();
        hasher.update(h1.as_bytes());
        hasher.update(h2.as_bytes());
        let expected_root: [u8; 32] = hasher.finalize().into();

        let proof = MerkleProof {
            event_hash: h1,
            path: vec![(h2, false)], // h2 ist rechter Sibling
            epoch: 0,
            root: Hash32::from_slice(&expected_root).unwrap(),
        };

        assert!(proof.verify());
    }

    #[test]
    fn test_archive_config_default() {
        let config = ArchiveConfig::default();
        assert_eq!(config.archive_after_days, 90);
        assert_eq!(config.epoch_size, 10_000);
        assert_eq!(config.min_finality, FinalityLevel::Witnessed);
    }

    #[test]
    fn test_merkle_node_leaf() {
        let hash = Hash32::from_slice(&[42u8; 32]).unwrap();
        let node = MerkleNode::leaf(hash);

        assert!(node.is_leaf());
        assert_eq!(node.hash, hash);
    }

    #[test]
    fn test_merkle_node_branch() {
        let h1 = Hash32::from_slice(&[1u8; 32]).unwrap();
        let h2 = Hash32::from_slice(&[2u8; 32]).unwrap();

        let left = MerkleNode::leaf(h1);
        let right = MerkleNode::leaf(h2);
        let branch = MerkleNode::branch(left, right);

        assert!(!branch.is_leaf());
        assert!(branch.left.is_some());
        assert!(branch.right.is_some());
    }
}
