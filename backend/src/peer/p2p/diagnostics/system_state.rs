//! # System State - Echtzeitdaten für alle Erynoa-Module
//!
//! Thread-safe State für Core, ECLVM, Local, Protection und Execution.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                         SYSTEM STATE                                        │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
//! │  │    CORE      │  │    ECLVM     │  │    LOCAL     │  │  PROTECTION  │    │
//! │  │  - Trust     │  │  - Mana      │  │  - Storage   │  │  - Anomaly   │    │
//! │  │  - Events    │  │  - Gas       │  │  - Archive   │  │  - Diversity │    │
//! │  │  - Formula   │  │  - Policies  │  │  - Realms    │  │  - Quadratic │    │
//! │  │  - Consensus │  │  - Runtime   │  │  - Blueprint │  │  - Calibrat. │    │
//! │  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘    │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::RwLock;
use std::time::Instant;

// ============================================================================
// CORE ENGINE STATS
// ============================================================================

/// Trust Engine Statistiken
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrustEngineStats {
    /// Anzahl registrierter Entitäten
    pub entities_count: usize,
    /// Anzahl Trust-Beziehungen
    pub relationships_count: usize,
    /// Durchschnittlicher Trust-Wert
    pub avg_trust_value: f64,
    /// Anzahl Trust-Updates
    pub trust_updates_total: u64,
    /// Positive Updates
    pub positive_updates: u64,
    /// Negative Updates (sollten 2x weniger sein wegen Κ4)
    pub negative_updates: u64,
    /// Trust-Verletzungen (unter Threshold)
    pub trust_violations: u64,
}

/// Event Engine Statistiken
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventEngineStats {
    /// Gesamtzahl Events im DAG
    pub events_total: u64,
    /// Genesis Events (keine Parents)
    pub genesis_events: u64,
    /// Finalisierte Events
    pub finalized_events: u64,
    /// Witnessed Events
    pub witnessed_events: u64,
    /// Events in den letzten 60 Sekunden
    pub events_per_minute: u64,
    /// Durchschnittliche Parents pro Event
    pub avg_parents_per_event: f64,
    /// Validierungsfehler
    pub validation_errors: u64,
    /// Zyklen erkannt
    pub cycles_detected: u64,
}

/// World Formula Statistiken (Κ15b)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorldFormulaStats {
    /// Aktuelle Weltformel-Berechnung (𝔼)
    pub current_e_value: f64,
    /// Anzahl Contributors
    pub contributors_count: usize,
    /// Durchschnittliche Aktivität
    pub avg_activity: f64,
    /// Durchschnittlicher Trust-Norm
    pub avg_trust_norm: f64,
    /// Human-verifizierte Entitäten
    pub human_verified_count: usize,
    /// Berechnungen insgesamt
    pub computations_total: u64,
    /// Letzte Berechnung (ms)
    pub last_computation_ms: u64,
}

/// Consensus Engine Statistiken
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsensusStats {
    /// Aktuelle Epoche
    pub current_epoch: u64,
    /// Aktiver Konsens-Algorithmus
    pub algorithm: String,
    /// Anzahl Validatoren/Witnesses
    pub validators_count: usize,
    /// Erfolgreiche Konsens-Runden
    pub successful_rounds: u64,
    /// Fehlgeschlagene Runden
    pub failed_rounds: u64,
    /// Durchschnittliche Konsens-Zeit (ms)
    pub avg_consensus_time_ms: f64,
}

// ============================================================================
// ECLVM STATS
// ============================================================================

/// ECLVM Runtime Statistiken
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EclvmStats {
    /// Gesamte Programme ausgeführt
    pub programs_executed: u64,
    /// Erfolgreiche Ausführungen
    pub successful_executions: u64,
    /// Fehlgeschlagene Ausführungen
    pub failed_executions: u64,
    /// Gas verbraucht insgesamt
    pub total_gas_consumed: u64,
    /// Durchschnittlicher Gas-Verbrauch pro Programm
    pub avg_gas_per_program: f64,
    /// Gas-Limit-Überschreitungen
    pub out_of_gas_count: u64,
    /// Aktive VM-Instanzen
    pub active_vms: usize,
}

/// Mana System Statistiken
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManaStats {
    /// Registrierte Mana-Konten
    pub accounts_count: usize,
    /// Gesamt-Mana im System
    pub total_mana: u64,
    /// Durchschnittliches Mana pro Account
    pub avg_mana_per_account: f64,
    /// Mana verbraucht insgesamt
    pub total_mana_consumed: u64,
    /// Mana regeneriert insgesamt
    pub total_mana_regenerated: u64,
    /// Rate-Limited Requests
    pub rate_limited_requests: u64,
    /// Mana pro Bandwidth-Tier
    pub mana_by_tier: HashMap<String, u64>,
}

/// Policy Gateway Statistiken
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyStats {
    /// Registrierte Policies
    pub policies_count: usize,
    /// Policy-Evaluierungen insgesamt
    pub evaluations_total: u64,
    /// Erlaubte Requests
    pub allowed_requests: u64,
    /// Abgelehnte Requests
    pub denied_requests: u64,
    /// Durchschnittliche Evaluierungszeit (µs)
    pub avg_evaluation_us: f64,
}

// ============================================================================
// LOCAL STORAGE STATS
// ============================================================================

/// Storage Statistiken
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageStats {
    /// KV-Store Größe (Bytes)
    pub kv_store_bytes: u64,
    /// Anzahl Keys im KV-Store
    pub kv_store_keys: u64,
    /// Event Store Größe
    pub event_store_events: u64,
    /// Content Store Größe
    pub content_store_items: u64,
    /// Content Store Bytes
    pub content_store_bytes: u64,
    /// Identity Store Einträge
    pub identity_store_entries: u64,
    /// Trust Store Einträge
    pub trust_store_entries: u64,
    /// Realm Storage Realms
    pub realm_count: usize,
    /// Reads pro Sekunde
    pub reads_per_sec: f64,
    /// Writes pro Sekunde
    pub writes_per_sec: f64,
}

/// Archive Statistiken
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArchiveStats {
    /// Archivierte Epochen
    pub archived_epochs: u64,
    /// Archivierte Events
    pub archived_events: u64,
    /// Archive-Größe (Bytes)
    pub archive_size_bytes: u64,
    /// Merkle-Roots gespeichert
    pub merkle_roots_count: u64,
    /// Letzte Archivierung (Unix Timestamp)
    pub last_archive_timestamp: u64,
    /// Komprimierungsrate
    pub compression_ratio: f64,
}

/// Blueprint Marketplace Statistiken
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlueprintStats {
    /// Veröffentlichte Blueprints
    pub published_blueprints: u64,
    /// Aktive Deployments
    pub active_deployments: u64,
    /// Downloads insgesamt
    pub total_downloads: u64,
    /// Durchschnittliche Bewertung
    pub avg_rating: f64,
    /// Top-Kategorien
    pub top_categories: Vec<(String, u64)>,
}

// ============================================================================
// PROTECTION STATS
// ============================================================================

/// Anomaly Detector Statistiken
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnomalyStats {
    /// Erkannte Anomalien insgesamt
    pub anomalies_detected: u64,
    /// Nach Severity
    pub anomalies_by_severity: HashMap<String, u64>,
    /// Nach Typ
    pub anomalies_by_type: HashMap<String, u64>,
    /// Aktive Überwachungen
    pub active_monitors: usize,
    /// False Positives (manuell markiert)
    pub false_positives: u64,
}

/// Diversity Monitor Statistiken (Κ20)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiversityStats {
    /// Dimensionen überwacht
    pub monitored_dimensions: usize,
    /// Entropie pro Dimension
    pub entropy_by_dimension: HashMap<String, f64>,
    /// Normalisierte Entropie pro Dimension
    pub normalized_entropy: HashMap<String, f64>,
    /// Mono-Kultur-Warnungen
    pub monoculture_warnings: u64,
    /// Minimum Entropie (aktuell)
    pub min_entropy_current: f64,
    /// Threshold erfüllt?
    pub threshold_met: bool,
}

/// Quadratic Governance Statistiken (Κ21)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuadraticStats {
    /// Aktive Abstimmungen
    pub active_votes: usize,
    /// Abgeschlossene Abstimmungen
    pub completed_votes: u64,
    /// Teilnehmer insgesamt
    pub total_participants: u64,
    /// Durchschnittliche Stimmen-Kosten
    pub avg_vote_cost: f64,
    /// Quadratische Reduktion wirksam
    pub quadratic_reduction_applied: u64,
}

/// Anti-Calcification Statistiken (Κ19)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AntiCalcificationStats {
    /// Macht-Konzentrations-Index
    pub power_concentration_index: f64,
    /// Gini-Koeffizient
    pub gini_coefficient: f64,
    /// Interventionen durchgeführt
    pub interventions_count: u64,
    /// Entitäten unter Beobachtung
    pub entities_under_watch: usize,
    /// Threshold-Verletzungen
    pub threshold_violations: u64,
}

/// Adaptive Calibration Statistiken (§IX)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalibrationStats {
    /// Parameter-Updates insgesamt
    pub parameter_updates: u64,
    /// Aktuelle Network Metrics
    pub network_load: f64,
    pub event_rate: f64,
    pub avg_latency_ms: f64,
    /// Kalibrierte Parameter (Name → Wert)
    pub calibrated_params: HashMap<String, f64>,
}

// ============================================================================
// EXECUTION CONTEXT STATS
// ============================================================================

/// Execution Context Statistiken
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionStats {
    /// Aktive Kontexte
    pub active_contexts: usize,
    /// Gesamte Ausführungen
    pub total_executions: u64,
    /// Erfolgreiche Ausführungen
    pub successful_executions: u64,
    /// Fehlgeschlagene Ausführungen
    pub failed_executions: u64,
    /// Gesamt-Gas verbraucht
    pub total_gas_consumed: u64,
    /// Emittierte Events
    pub events_emitted: u64,
    /// Durchschnittliche Ausführungszeit (ms)
    pub avg_execution_time_ms: f64,
    /// Aktuelle Epoche
    pub current_epoch: u64,
    /// Aktueller Lamport-Clock
    pub current_lamport: u64,
}

// ============================================================================
// AGGREGATED SYSTEM SNAPSHOT
// ============================================================================

/// Vollständiger System-Snapshot
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemSnapshot {
    /// Zeitstempel
    pub timestamp_ms: u64,
    /// Uptime in Sekunden
    pub uptime_secs: u64,

    // Core Engine
    pub trust: TrustEngineStats,
    pub events: EventEngineStats,
    pub world_formula: WorldFormulaStats,
    pub consensus: ConsensusStats,

    // ECLVM
    pub eclvm: EclvmStats,
    pub mana: ManaStats,
    pub policies: PolicyStats,

    // Local Storage
    pub storage: StorageStats,
    pub archive: ArchiveStats,
    pub blueprints: BlueprintStats,

    // Protection
    pub anomaly: AnomalyStats,
    pub diversity: DiversityStats,
    pub quadratic: QuadraticStats,
    pub anti_calcification: AntiCalcificationStats,
    pub calibration: CalibrationStats,

    // Execution
    pub execution: ExecutionStats,

    // System Health
    pub health_score: f64,
    pub active_warnings: Vec<String>,
}

// ============================================================================
// THREAD-SAFE SYSTEM STATE
// ============================================================================

/// Thread-safe System State mit atomaren Countern
pub struct SystemState {
    start_time: Instant,

    // === Core Engine Counters ===
    // Trust
    pub trust_entities: AtomicUsize,
    pub trust_relationships: AtomicUsize,
    pub trust_updates_total: AtomicU64,
    pub trust_positive_updates: AtomicU64,
    pub trust_negative_updates: AtomicU64,
    pub trust_violations: AtomicU64,

    // Events
    pub events_total: AtomicU64,
    pub events_finalized: AtomicU64,
    pub events_witnessed: AtomicU64,
    pub events_genesis: AtomicU64,
    pub event_validation_errors: AtomicU64,
    pub event_cycles_detected: AtomicU64,

    // World Formula
    pub formula_computations: AtomicU64,
    pub formula_contributors: AtomicUsize,
    pub formula_human_verified: AtomicUsize,

    // Consensus
    pub consensus_epoch: AtomicU64,
    pub consensus_successful: AtomicU64,
    pub consensus_failed: AtomicU64,
    pub consensus_validators: AtomicUsize,

    // === ECLVM Counters ===
    pub eclvm_programs_executed: AtomicU64,
    pub eclvm_successful: AtomicU64,
    pub eclvm_failed: AtomicU64,
    pub eclvm_gas_consumed: AtomicU64,
    pub eclvm_out_of_gas: AtomicU64,
    pub eclvm_active_vms: AtomicUsize,

    // Mana
    pub mana_accounts: AtomicUsize,
    pub mana_total: AtomicU64,
    pub mana_consumed: AtomicU64,
    pub mana_regenerated: AtomicU64,
    pub mana_rate_limited: AtomicU64,

    // Policies
    pub policy_count: AtomicUsize,
    pub policy_evaluations: AtomicU64,
    pub policy_allowed: AtomicU64,
    pub policy_denied: AtomicU64,

    // === Local Storage Counters ===
    pub storage_keys: AtomicU64,
    pub storage_bytes: AtomicU64,
    pub storage_reads: AtomicU64,
    pub storage_writes: AtomicU64,
    pub storage_events: AtomicU64,
    pub storage_content_items: AtomicU64,
    pub storage_identities: AtomicU64,
    pub storage_trusts: AtomicU64,
    pub storage_realms: AtomicUsize,

    // Archive
    pub archive_epochs: AtomicU64,
    pub archive_events: AtomicU64,
    pub archive_bytes: AtomicU64,
    pub archive_merkle_roots: AtomicU64,

    // Blueprints
    pub blueprint_published: AtomicU64,
    pub blueprint_deployments: AtomicU64,
    pub blueprint_downloads: AtomicU64,

    // === Protection Counters ===
    // Anomaly
    pub anomalies_detected: AtomicU64,
    pub anomalies_low: AtomicU64,
    pub anomalies_medium: AtomicU64,
    pub anomalies_high: AtomicU64,
    pub anomalies_critical: AtomicU64,

    // Diversity
    pub diversity_dimensions: AtomicUsize,
    pub diversity_warnings: AtomicU64,

    // Quadratic
    pub quadratic_active_votes: AtomicUsize,
    pub quadratic_completed: AtomicU64,
    pub quadratic_participants: AtomicU64,

    // Anti-Calcification
    pub anticac_interventions: AtomicU64,
    pub anticac_violations: AtomicU64,
    pub anticac_watched: AtomicUsize,

    // Calibration
    pub calibration_updates: AtomicU64,

    // === Execution Counters ===
    pub exec_active_contexts: AtomicUsize,
    pub exec_total: AtomicU64,
    pub exec_successful: AtomicU64,
    pub exec_failed: AtomicU64,
    pub exec_gas_consumed: AtomicU64,
    pub exec_events_emitted: AtomicU64,
    pub exec_epoch: AtomicU64,
    pub exec_lamport: AtomicU64,

    // === Complex State (needs RwLock) ===
    pub world_formula_value: RwLock<f64>,
    pub avg_trust_value: RwLock<f64>,
    pub entropy_values: RwLock<HashMap<String, f64>>,
    pub calibrated_params: RwLock<HashMap<String, f64>>,
    pub active_warnings: RwLock<Vec<String>>,
}

impl SystemState {
    /// Erstelle neuen SystemState
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),

            // Core
            trust_entities: AtomicUsize::new(0),
            trust_relationships: AtomicUsize::new(0),
            trust_updates_total: AtomicU64::new(0),
            trust_positive_updates: AtomicU64::new(0),
            trust_negative_updates: AtomicU64::new(0),
            trust_violations: AtomicU64::new(0),

            events_total: AtomicU64::new(0),
            events_finalized: AtomicU64::new(0),
            events_witnessed: AtomicU64::new(0),
            events_genesis: AtomicU64::new(0),
            event_validation_errors: AtomicU64::new(0),
            event_cycles_detected: AtomicU64::new(0),

            formula_computations: AtomicU64::new(0),
            formula_contributors: AtomicUsize::new(0),
            formula_human_verified: AtomicUsize::new(0),

            consensus_epoch: AtomicU64::new(0),
            consensus_successful: AtomicU64::new(0),
            consensus_failed: AtomicU64::new(0),
            consensus_validators: AtomicUsize::new(0),

            // ECLVM
            eclvm_programs_executed: AtomicU64::new(0),
            eclvm_successful: AtomicU64::new(0),
            eclvm_failed: AtomicU64::new(0),
            eclvm_gas_consumed: AtomicU64::new(0),
            eclvm_out_of_gas: AtomicU64::new(0),
            eclvm_active_vms: AtomicUsize::new(0),

            mana_accounts: AtomicUsize::new(0),
            mana_total: AtomicU64::new(0),
            mana_consumed: AtomicU64::new(0),
            mana_regenerated: AtomicU64::new(0),
            mana_rate_limited: AtomicU64::new(0),

            policy_count: AtomicUsize::new(0),
            policy_evaluations: AtomicU64::new(0),
            policy_allowed: AtomicU64::new(0),
            policy_denied: AtomicU64::new(0),

            // Storage
            storage_keys: AtomicU64::new(0),
            storage_bytes: AtomicU64::new(0),
            storage_reads: AtomicU64::new(0),
            storage_writes: AtomicU64::new(0),
            storage_events: AtomicU64::new(0),
            storage_content_items: AtomicU64::new(0),
            storage_identities: AtomicU64::new(0),
            storage_trusts: AtomicU64::new(0),
            storage_realms: AtomicUsize::new(0),

            archive_epochs: AtomicU64::new(0),
            archive_events: AtomicU64::new(0),
            archive_bytes: AtomicU64::new(0),
            archive_merkle_roots: AtomicU64::new(0),

            blueprint_published: AtomicU64::new(0),
            blueprint_deployments: AtomicU64::new(0),
            blueprint_downloads: AtomicU64::new(0),

            // Protection
            anomalies_detected: AtomicU64::new(0),
            anomalies_low: AtomicU64::new(0),
            anomalies_medium: AtomicU64::new(0),
            anomalies_high: AtomicU64::new(0),
            anomalies_critical: AtomicU64::new(0),

            diversity_dimensions: AtomicUsize::new(0),
            diversity_warnings: AtomicU64::new(0),

            quadratic_active_votes: AtomicUsize::new(0),
            quadratic_completed: AtomicU64::new(0),
            quadratic_participants: AtomicU64::new(0),

            anticac_interventions: AtomicU64::new(0),
            anticac_violations: AtomicU64::new(0),
            anticac_watched: AtomicUsize::new(0),

            calibration_updates: AtomicU64::new(0),

            // Execution
            exec_active_contexts: AtomicUsize::new(0),
            exec_total: AtomicU64::new(0),
            exec_successful: AtomicU64::new(0),
            exec_failed: AtomicU64::new(0),
            exec_gas_consumed: AtomicU64::new(0),
            exec_events_emitted: AtomicU64::new(0),
            exec_epoch: AtomicU64::new(0),
            exec_lamport: AtomicU64::new(0),

            // Complex
            world_formula_value: RwLock::new(0.0),
            avg_trust_value: RwLock::new(0.5),
            entropy_values: RwLock::new(HashMap::new()),
            calibrated_params: RwLock::new(HashMap::new()),
            active_warnings: RwLock::new(Vec::new()),
        }
    }

    /// Uptime in Sekunden
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    // ========================================================================
    // TRUST ENGINE METHODS
    // ========================================================================

    pub fn trust_entity_registered(&self) {
        self.trust_entities.fetch_add(1, Ordering::Relaxed);
    }

    pub fn trust_relationship_added(&self) {
        self.trust_relationships.fetch_add(1, Ordering::Relaxed);
    }

    pub fn trust_updated(&self, positive: bool) {
        self.trust_updates_total.fetch_add(1, Ordering::Relaxed);
        if positive {
            self.trust_positive_updates.fetch_add(1, Ordering::Relaxed);
        } else {
            self.trust_negative_updates.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn trust_violation_detected(&self) {
        self.trust_violations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_avg_trust(&self, value: f64) {
        if let Ok(mut guard) = self.avg_trust_value.write() {
            *guard = value;
        }
    }

    // ========================================================================
    // EVENT ENGINE METHODS
    // ========================================================================

    pub fn event_added(&self, is_genesis: bool) {
        self.events_total.fetch_add(1, Ordering::Relaxed);
        if is_genesis {
            self.events_genesis.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn event_finalized(&self) {
        self.events_finalized.fetch_add(1, Ordering::Relaxed);
    }

    pub fn event_witnessed(&self) {
        self.events_witnessed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn event_validation_error(&self) {
        self.event_validation_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn event_cycle_detected(&self) {
        self.event_cycles_detected.fetch_add(1, Ordering::Relaxed);
    }

    // ========================================================================
    // WORLD FORMULA METHODS
    // ========================================================================

    pub fn formula_computed(&self, value: f64, contributors: usize, human_verified: usize) {
        self.formula_computations.fetch_add(1, Ordering::Relaxed);
        self.formula_contributors
            .store(contributors, Ordering::Relaxed);
        self.formula_human_verified
            .store(human_verified, Ordering::Relaxed);
        if let Ok(mut guard) = self.world_formula_value.write() {
            *guard = value;
        }
    }

    // ========================================================================
    // CONSENSUS METHODS
    // ========================================================================

    pub fn consensus_round_completed(&self, success: bool) {
        if success {
            self.consensus_successful.fetch_add(1, Ordering::Relaxed);
        } else {
            self.consensus_failed.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn set_consensus_epoch(&self, epoch: u64) {
        self.consensus_epoch.store(epoch, Ordering::Relaxed);
    }

    pub fn set_validators_count(&self, count: usize) {
        self.consensus_validators.store(count, Ordering::Relaxed);
    }

    // ========================================================================
    // ECLVM METHODS
    // ========================================================================

    pub fn eclvm_program_executed(&self, success: bool, gas_used: u64) {
        self.eclvm_programs_executed.fetch_add(1, Ordering::Relaxed);
        self.eclvm_gas_consumed
            .fetch_add(gas_used, Ordering::Relaxed);
        if success {
            self.eclvm_successful.fetch_add(1, Ordering::Relaxed);
        } else {
            self.eclvm_failed.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn eclvm_out_of_gas(&self) {
        self.eclvm_out_of_gas.fetch_add(1, Ordering::Relaxed);
    }

    pub fn eclvm_vm_started(&self) {
        self.eclvm_active_vms.fetch_add(1, Ordering::Relaxed);
    }

    pub fn eclvm_vm_stopped(&self) {
        self.eclvm_active_vms.fetch_sub(1, Ordering::Relaxed);
    }

    // ========================================================================
    // MANA METHODS
    // ========================================================================

    pub fn mana_account_created(&self) {
        self.mana_accounts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn mana_consumed(&self, amount: u64) {
        self.mana_consumed.fetch_add(amount, Ordering::Relaxed);
    }

    pub fn mana_regenerated(&self, amount: u64) {
        self.mana_regenerated.fetch_add(amount, Ordering::Relaxed);
    }

    pub fn mana_rate_limited(&self) {
        self.mana_rate_limited.fetch_add(1, Ordering::Relaxed);
    }

    // ========================================================================
    // POLICY METHODS
    // ========================================================================

    pub fn policy_registered(&self) {
        self.policy_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn policy_evaluated(&self, allowed: bool) {
        self.policy_evaluations.fetch_add(1, Ordering::Relaxed);
        if allowed {
            self.policy_allowed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.policy_denied.fetch_add(1, Ordering::Relaxed);
        }
    }

    // ========================================================================
    // STORAGE METHODS
    // ========================================================================

    pub fn storage_read(&self) {
        self.storage_reads.fetch_add(1, Ordering::Relaxed);
    }

    pub fn storage_write(&self, bytes: u64) {
        self.storage_writes.fetch_add(1, Ordering::Relaxed);
        self.storage_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn storage_key_added(&self) {
        self.storage_keys.fetch_add(1, Ordering::Relaxed);
    }

    pub fn storage_set_counts(
        &self,
        events: u64,
        content: u64,
        identities: u64,
        trusts: u64,
        realms: usize,
    ) {
        self.storage_events.store(events, Ordering::Relaxed);
        self.storage_content_items.store(content, Ordering::Relaxed);
        self.storage_identities.store(identities, Ordering::Relaxed);
        self.storage_trusts.store(trusts, Ordering::Relaxed);
        self.storage_realms.store(realms, Ordering::Relaxed);
    }

    // ========================================================================
    // ARCHIVE METHODS
    // ========================================================================

    pub fn archive_epoch_completed(&self, events: u64, bytes: u64) {
        self.archive_epochs.fetch_add(1, Ordering::Relaxed);
        self.archive_events.fetch_add(events, Ordering::Relaxed);
        self.archive_bytes.fetch_add(bytes, Ordering::Relaxed);
        self.archive_merkle_roots.fetch_add(1, Ordering::Relaxed);
    }

    // ========================================================================
    // BLUEPRINT METHODS
    // ========================================================================

    pub fn blueprint_published(&self) {
        self.blueprint_published.fetch_add(1, Ordering::Relaxed);
    }

    pub fn blueprint_deployed(&self) {
        self.blueprint_deployments.fetch_add(1, Ordering::Relaxed);
    }

    pub fn blueprint_downloaded(&self) {
        self.blueprint_downloads.fetch_add(1, Ordering::Relaxed);
    }

    // ========================================================================
    // ANOMALY METHODS
    // ========================================================================

    pub fn anomaly_detected(&self, severity: &str) {
        self.anomalies_detected.fetch_add(1, Ordering::Relaxed);
        match severity {
            "low" => self.anomalies_low.fetch_add(1, Ordering::Relaxed),
            "medium" => self.anomalies_medium.fetch_add(1, Ordering::Relaxed),
            "high" => self.anomalies_high.fetch_add(1, Ordering::Relaxed),
            "critical" => self.anomalies_critical.fetch_add(1, Ordering::Relaxed),
            _ => 0,
        };
    }

    // ========================================================================
    // DIVERSITY METHODS
    // ========================================================================

    pub fn diversity_set_entropy(&self, dimension: &str, value: f64) {
        if let Ok(mut guard) = self.entropy_values.write() {
            guard.insert(dimension.to_string(), value);
        }
    }

    pub fn diversity_warning(&self) {
        self.diversity_warnings.fetch_add(1, Ordering::Relaxed);
    }

    // ========================================================================
    // QUADRATIC METHODS
    // ========================================================================

    pub fn quadratic_vote_started(&self) {
        self.quadratic_active_votes.fetch_add(1, Ordering::Relaxed);
    }

    pub fn quadratic_vote_completed(&self, participants: u64) {
        self.quadratic_active_votes.fetch_sub(1, Ordering::Relaxed);
        self.quadratic_completed.fetch_add(1, Ordering::Relaxed);
        self.quadratic_participants
            .fetch_add(participants, Ordering::Relaxed);
    }

    // ========================================================================
    // ANTI-CALCIFICATION METHODS
    // ========================================================================

    pub fn anticac_intervention(&self) {
        self.anticac_interventions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn anticac_violation(&self) {
        self.anticac_violations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn anticac_set_watched(&self, count: usize) {
        self.anticac_watched.store(count, Ordering::Relaxed);
    }

    // ========================================================================
    // CALIBRATION METHODS
    // ========================================================================

    pub fn calibration_update(&self, param: &str, value: f64) {
        self.calibration_updates.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut guard) = self.calibrated_params.write() {
            guard.insert(param.to_string(), value);
        }
    }

    // ========================================================================
    // EXECUTION METHODS
    // ========================================================================

    pub fn execution_started(&self) {
        self.exec_active_contexts.fetch_add(1, Ordering::Relaxed);
        self.exec_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn execution_completed(&self, success: bool, gas_used: u64, events_emitted: u64) {
        self.exec_active_contexts.fetch_sub(1, Ordering::Relaxed);
        self.exec_gas_consumed
            .fetch_add(gas_used, Ordering::Relaxed);
        self.exec_events_emitted
            .fetch_add(events_emitted, Ordering::Relaxed);
        if success {
            self.exec_successful.fetch_add(1, Ordering::Relaxed);
        } else {
            self.exec_failed.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn execution_set_epoch_lamport(&self, epoch: u64, lamport: u64) {
        self.exec_epoch.store(epoch, Ordering::Relaxed);
        self.exec_lamport.store(lamport, Ordering::Relaxed);
    }

    // ========================================================================
    // WARNING METHODS
    // ========================================================================

    pub fn add_warning(&self, warning: String) {
        if let Ok(mut guard) = self.active_warnings.write() {
            if !guard.contains(&warning) {
                guard.push(warning);
                // Limit to 50 warnings
                if guard.len() > 50 {
                    guard.remove(0);
                }
            }
        }
    }

    pub fn clear_warning(&self, warning: &str) {
        if let Ok(mut guard) = self.active_warnings.write() {
            guard.retain(|w| w != warning);
        }
    }

    // ========================================================================
    // SNAPSHOT
    // ========================================================================

    /// Erstelle vollständigen System-Snapshot
    pub fn snapshot(&self) -> SystemSnapshot {
        let entropy_values = self
            .entropy_values
            .read()
            .map(|g| g.clone())
            .unwrap_or_default();
        let calibrated_params = self
            .calibrated_params
            .read()
            .map(|g| g.clone())
            .unwrap_or_default();
        let active_warnings = self
            .active_warnings
            .read()
            .map(|g| g.clone())
            .unwrap_or_default();
        let world_formula_value = self.world_formula_value.read().map(|g| *g).unwrap_or(0.0);
        let avg_trust = self.avg_trust_value.read().map(|g| *g).unwrap_or(0.5);

        let events_total = self.events_total.load(Ordering::Relaxed);
        let eclvm_executed = self.eclvm_programs_executed.load(Ordering::Relaxed);
        let eclvm_gas = self.eclvm_gas_consumed.load(Ordering::Relaxed);

        SystemSnapshot {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            uptime_secs: self.uptime_secs(),

            trust: TrustEngineStats {
                entities_count: self.trust_entities.load(Ordering::Relaxed),
                relationships_count: self.trust_relationships.load(Ordering::Relaxed),
                avg_trust_value: avg_trust,
                trust_updates_total: self.trust_updates_total.load(Ordering::Relaxed),
                positive_updates: self.trust_positive_updates.load(Ordering::Relaxed),
                negative_updates: self.trust_negative_updates.load(Ordering::Relaxed),
                trust_violations: self.trust_violations.load(Ordering::Relaxed),
            },

            events: EventEngineStats {
                events_total,
                genesis_events: self.events_genesis.load(Ordering::Relaxed),
                finalized_events: self.events_finalized.load(Ordering::Relaxed),
                witnessed_events: self.events_witnessed.load(Ordering::Relaxed),
                events_per_minute: 0, // TODO: Calculate from ring buffer
                avg_parents_per_event: 0.0,
                validation_errors: self.event_validation_errors.load(Ordering::Relaxed),
                cycles_detected: self.event_cycles_detected.load(Ordering::Relaxed),
            },

            world_formula: WorldFormulaStats {
                current_e_value: world_formula_value,
                contributors_count: self.formula_contributors.load(Ordering::Relaxed),
                avg_activity: 0.0,
                avg_trust_norm: 0.0,
                human_verified_count: self.formula_human_verified.load(Ordering::Relaxed),
                computations_total: self.formula_computations.load(Ordering::Relaxed),
                last_computation_ms: 0,
            },

            consensus: ConsensusStats {
                current_epoch: self.consensus_epoch.load(Ordering::Relaxed),
                algorithm: "BFT".to_string(),
                validators_count: self.consensus_validators.load(Ordering::Relaxed),
                successful_rounds: self.consensus_successful.load(Ordering::Relaxed),
                failed_rounds: self.consensus_failed.load(Ordering::Relaxed),
                avg_consensus_time_ms: 0.0,
            },

            eclvm: EclvmStats {
                programs_executed: eclvm_executed,
                successful_executions: self.eclvm_successful.load(Ordering::Relaxed),
                failed_executions: self.eclvm_failed.load(Ordering::Relaxed),
                total_gas_consumed: eclvm_gas,
                avg_gas_per_program: if eclvm_executed > 0 {
                    eclvm_gas as f64 / eclvm_executed as f64
                } else {
                    0.0
                },
                out_of_gas_count: self.eclvm_out_of_gas.load(Ordering::Relaxed),
                active_vms: self.eclvm_active_vms.load(Ordering::Relaxed),
            },

            mana: ManaStats {
                accounts_count: self.mana_accounts.load(Ordering::Relaxed),
                total_mana: self.mana_total.load(Ordering::Relaxed),
                avg_mana_per_account: 0.0,
                total_mana_consumed: self.mana_consumed.load(Ordering::Relaxed),
                total_mana_regenerated: self.mana_regenerated.load(Ordering::Relaxed),
                rate_limited_requests: self.mana_rate_limited.load(Ordering::Relaxed),
                mana_by_tier: HashMap::new(),
            },

            policies: PolicyStats {
                policies_count: self.policy_count.load(Ordering::Relaxed),
                evaluations_total: self.policy_evaluations.load(Ordering::Relaxed),
                allowed_requests: self.policy_allowed.load(Ordering::Relaxed),
                denied_requests: self.policy_denied.load(Ordering::Relaxed),
                avg_evaluation_us: 0.0,
            },

            storage: StorageStats {
                kv_store_bytes: self.storage_bytes.load(Ordering::Relaxed),
                kv_store_keys: self.storage_keys.load(Ordering::Relaxed),
                event_store_events: self.storage_events.load(Ordering::Relaxed),
                content_store_items: self.storage_content_items.load(Ordering::Relaxed),
                content_store_bytes: 0,
                identity_store_entries: self.storage_identities.load(Ordering::Relaxed),
                trust_store_entries: self.storage_trusts.load(Ordering::Relaxed),
                realm_count: self.storage_realms.load(Ordering::Relaxed),
                reads_per_sec: 0.0,
                writes_per_sec: 0.0,
            },

            archive: ArchiveStats {
                archived_epochs: self.archive_epochs.load(Ordering::Relaxed),
                archived_events: self.archive_events.load(Ordering::Relaxed),
                archive_size_bytes: self.archive_bytes.load(Ordering::Relaxed),
                merkle_roots_count: self.archive_merkle_roots.load(Ordering::Relaxed),
                last_archive_timestamp: 0,
                compression_ratio: 0.0,
            },

            blueprints: BlueprintStats {
                published_blueprints: self.blueprint_published.load(Ordering::Relaxed),
                active_deployments: self.blueprint_deployments.load(Ordering::Relaxed),
                total_downloads: self.blueprint_downloads.load(Ordering::Relaxed),
                avg_rating: 0.0,
                top_categories: Vec::new(),
            },

            anomaly: AnomalyStats {
                anomalies_detected: self.anomalies_detected.load(Ordering::Relaxed),
                anomalies_by_severity: {
                    let mut m = HashMap::new();
                    m.insert(
                        "low".to_string(),
                        self.anomalies_low.load(Ordering::Relaxed),
                    );
                    m.insert(
                        "medium".to_string(),
                        self.anomalies_medium.load(Ordering::Relaxed),
                    );
                    m.insert(
                        "high".to_string(),
                        self.anomalies_high.load(Ordering::Relaxed),
                    );
                    m.insert(
                        "critical".to_string(),
                        self.anomalies_critical.load(Ordering::Relaxed),
                    );
                    m
                },
                anomalies_by_type: HashMap::new(),
                active_monitors: 0,
                false_positives: 0,
            },

            diversity: DiversityStats {
                monitored_dimensions: self.diversity_dimensions.load(Ordering::Relaxed),
                entropy_by_dimension: entropy_values.clone(),
                normalized_entropy: HashMap::new(),
                monoculture_warnings: self.diversity_warnings.load(Ordering::Relaxed),
                min_entropy_current: entropy_values.values().copied().fold(f64::MAX, f64::min),
                threshold_met: true,
            },

            quadratic: QuadraticStats {
                active_votes: self.quadratic_active_votes.load(Ordering::Relaxed),
                completed_votes: self.quadratic_completed.load(Ordering::Relaxed),
                total_participants: self.quadratic_participants.load(Ordering::Relaxed),
                avg_vote_cost: 0.0,
                quadratic_reduction_applied: 0,
            },

            anti_calcification: AntiCalcificationStats {
                power_concentration_index: 0.0,
                gini_coefficient: 0.0,
                interventions_count: self.anticac_interventions.load(Ordering::Relaxed),
                entities_under_watch: self.anticac_watched.load(Ordering::Relaxed),
                threshold_violations: self.anticac_violations.load(Ordering::Relaxed),
            },

            calibration: CalibrationStats {
                parameter_updates: self.calibration_updates.load(Ordering::Relaxed),
                network_load: 0.0,
                event_rate: 0.0,
                avg_latency_ms: 0.0,
                calibrated_params,
            },

            execution: ExecutionStats {
                active_contexts: self.exec_active_contexts.load(Ordering::Relaxed),
                total_executions: self.exec_total.load(Ordering::Relaxed),
                successful_executions: self.exec_successful.load(Ordering::Relaxed),
                failed_executions: self.exec_failed.load(Ordering::Relaxed),
                total_gas_consumed: self.exec_gas_consumed.load(Ordering::Relaxed),
                events_emitted: self.exec_events_emitted.load(Ordering::Relaxed),
                avg_execution_time_ms: 0.0,
                current_epoch: self.exec_epoch.load(Ordering::Relaxed),
                current_lamport: self.exec_lamport.load(Ordering::Relaxed),
            },

            health_score: self.calculate_health_score(),
            active_warnings,
        }
    }

    /// Berechne System-Health-Score (0-100)
    fn calculate_health_score(&self) -> f64 {
        let mut score = 100.0;

        // Anomalien reduzieren Score
        let critical = self.anomalies_critical.load(Ordering::Relaxed);
        let high = self.anomalies_high.load(Ordering::Relaxed);
        score -= (critical * 20) as f64;
        score -= (high * 10) as f64;

        // Trust Violations reduzieren Score
        let violations = self.trust_violations.load(Ordering::Relaxed);
        score -= (violations.min(10) * 2) as f64;

        // Event Validation Errors
        let errors = self.event_validation_errors.load(Ordering::Relaxed);
        score -= (errors.min(20)) as f64;

        // Diversity Warnings
        let div_warnings = self.diversity_warnings.load(Ordering::Relaxed);
        score -= (div_warnings * 5) as f64;

        // Anti-Calcification Violations
        let anticac = self.anticac_violations.load(Ordering::Relaxed);
        score -= (anticac * 10) as f64;

        score.max(0.0).min(100.0)
    }
}

impl Default for SystemState {
    fn default() -> Self {
        Self::new()
    }
}
