//! # Execution Error Types
//!
//! Unifizierte Fehler-Hierarchie für alle Ausführungskontexte.
//!
//! ## IPS v1.2.0 Referenz
//!
//! Die Fehler-Hierarchie entspricht der monadischen Komposition:
//! - `ℳ_VM` → VM-Errors (Gas, Stack, Policy)
//! - `ℳ_S`  → Storage-Errors (Schema, Access, Capacity)
//! - `ℳ_P`  → P2P-Errors (Connection, Trust, Topic)
//!
//! ## Axiom-Referenz
//!
//! - **Κ11 (Prozess-Korrektheit)**: Fehler propagieren korrekt durch Result
//! - **Κ4 (Asymmetrische Evolution)**: TrustGateBlocked enthält Trust-Werte

use thiserror::Error;

// ============================================================================
// ExecutionError – Unifizierte Fehler-Hierarchie
// ============================================================================

/// Unifizierte Fehler-Hierarchie für alle Execution-Kontexte
///
/// Entspricht der Vereinigung `ℳ_VM + ℳ_S + ℳ_P` aus IPS v1.2.0.
///
/// # Beispiel
///
/// ```rust
/// use erynoa_api::execution::{ExecutionError, ExecutionResult};
///
/// fn process_operation() -> ExecutionResult<u64> {
///     // Gas-Check
///     if gas_remaining < 100 {
///         return Err(ExecutionError::GasExhausted { required: 100, available: gas_remaining });
///     }
///     Ok(42)
/// }
/// ```
#[derive(Debug, Clone, Error)]
pub enum ExecutionError {
    // =========================================================================
    // VM-Errors (ℳ_VM) – ECLVM Execution
    // =========================================================================
    /// Gas erschöpft – Computation kann nicht fortgesetzt werden
    #[error("Gas exhausted: required {required}, available {available}")]
    GasExhausted {
        /// Benötigte Gas-Menge
        required: u64,
        /// Verfügbare Gas-Menge
        available: u64,
    },

    /// Mana erschöpft – Ressourcen-Limit erreicht
    #[error("Mana exhausted: required {required}, available {available}")]
    ManaExhausted {
        /// Benötigte Mana-Menge
        required: u64,
        /// Verfügbare Mana-Menge
        available: u64,
    },

    /// Stack-Überlauf in ECLVM
    #[error("Stack overflow: depth {depth} exceeds limit {limit}")]
    StackOverflow {
        /// Aktuelle Stack-Tiefe
        depth: usize,
        /// Maximale Stack-Tiefe
        limit: usize,
    },

    /// Policy-Verletzung (Realm-Regeln)
    #[error("Policy violation: {policy} - {reason}")]
    PolicyViolation {
        /// Name der verletzten Policy
        policy: String,
        /// Beschreibung der Verletzung
        reason: String,
    },

    /// Ungültiger Opcode in ECLVM
    #[error("Invalid opcode: 0x{opcode:02X} at position {position}")]
    InvalidOpcode {
        /// Der ungültige Opcode
        opcode: u8,
        /// Position im Bytecode
        position: usize,
    },

    /// Division durch Null
    #[error("Division by zero at position {position}")]
    DivisionByZero {
        /// Position im Bytecode
        position: usize,
    },

    // =========================================================================
    // Storage-Errors (ℳ_S) – Fjall/Local Storage
    // =========================================================================
    /// Schema-Verletzung bei Datenvalidierung
    #[error("Schema violation: {schema} - {reason}")]
    SchemaViolation {
        /// Name des verletzten Schemas
        schema: String,
        /// Beschreibung der Verletzung
        reason: String,
    },

    /// Zugriff verweigert (Berechtigungsfehler)
    #[error("Access denied: {resource} requires {required_permission}")]
    AccessDenied {
        /// Betroffene Ressource
        resource: String,
        /// Benötigte Berechtigung
        required_permission: String,
    },

    /// Storage-Kapazität erschöpft
    #[error("Storage full: {store} at {used_bytes}/{max_bytes} bytes")]
    StoreFull {
        /// Name des betroffenen Stores
        store: String,
        /// Verwendete Bytes
        used_bytes: u64,
        /// Maximale Bytes
        max_bytes: u64,
    },

    /// Objekt nicht gefunden
    #[error("Not found: {resource_type} with id {id}")]
    NotFound {
        /// Typ der Ressource
        resource_type: String,
        /// ID der Ressource (Hex-String)
        id: String,
    },

    /// Serialisierungsfehler
    #[error("Serialization error: {reason}")]
    SerializationError {
        /// Beschreibung des Fehlers
        reason: String,
    },

    // =========================================================================
    // P2P-Errors (ℳ_P) – libp2p Network
    // =========================================================================
    /// Verbindungsfehler
    #[error("Connection failed to {peer_id}: {reason}")]
    ConnectionFailed {
        /// Peer-ID (Hex-String)
        peer_id: String,
        /// Fehlergrund
        reason: String,
    },

    /// Trust-Gate blockiert (Κ4)
    #[error("Trust gate blocked: required {required:.3}, actual {actual:.3}")]
    TrustGateBlocked {
        /// Erforderlicher Trust-Wert
        required: f32,
        /// Tatsächlicher Trust-Wert
        actual: f32,
    },

    /// Topic nicht abonniert
    #[error("Topic not subscribed: {topic}")]
    TopicNotSubscribed {
        /// Topic-Name
        topic: String,
    },

    /// Peer nicht erreichbar
    #[error("Peer unreachable: {peer_id} after {attempts} attempts")]
    PeerUnreachable {
        /// Peer-ID (Hex-String)
        peer_id: String,
        /// Anzahl der Versuche
        attempts: u32,
    },

    /// Netzwerk-Timeout
    #[error("Network timeout after {timeout_ms}ms")]
    NetworkTimeout {
        /// Timeout in Millisekunden
        timeout_ms: u64,
    },

    // =========================================================================
    // Invariant-Errors (Κ-Axiome)
    // =========================================================================
    /// Kausale Ordnung verletzt (Κ9)
    #[error("Causal order violated: parent {parent_id} not before event {event_id}")]
    CausalOrderViolation {
        /// Event-ID (Hex-String)
        event_id: String,
        /// Parent-ID (Hex-String)
        parent_id: String,
    },

    /// Finalität-Regression (Κ10)
    #[error("Finality regression: {event_id} from level {old_level} to {new_level}")]
    FinalityRegression {
        /// Event-ID (Hex-String)
        event_id: String,
        /// Alter Finalitäts-Level
        old_level: u8,
        /// Neuer Finalitäts-Level
        new_level: u8,
    },

    /// Trust-Decay-Verletzung (Κ8)
    #[error("Trust decay violation: factor {factor:.3} not in (0, 1]")]
    TrustDecayViolation {
        /// Ungültiger Trust-Faktor
        factor: f32,
    },

    // =========================================================================
    // Generic Errors
    // =========================================================================
    /// Interner Fehler (Catch-All)
    #[error("Internal error: {0}")]
    Internal(String),

    /// Ungültige Eingabe
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Operation abgebrochen
    #[error("Operation cancelled: {reason}")]
    Cancelled {
        /// Grund für den Abbruch
        reason: String,
    },
}

// ============================================================================
// Error Categories
// ============================================================================

/// Fehler-Kategorie für Metriken und Logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// VM/Computation-Fehler
    Vm,
    /// Storage-Fehler
    Storage,
    /// P2P/Netzwerk-Fehler
    P2P,
    /// Invarianten-Verletzung
    Invariant,
    /// Sonstige Fehler
    Other,
}

impl ExecutionError {
    /// Kategorie des Fehlers
    pub fn category(&self) -> ErrorCategory {
        match self {
            // VM
            Self::GasExhausted { .. }
            | Self::ManaExhausted { .. }
            | Self::StackOverflow { .. }
            | Self::PolicyViolation { .. }
            | Self::InvalidOpcode { .. }
            | Self::DivisionByZero { .. } => ErrorCategory::Vm,

            // Storage
            Self::SchemaViolation { .. }
            | Self::AccessDenied { .. }
            | Self::StoreFull { .. }
            | Self::NotFound { .. }
            | Self::SerializationError { .. } => ErrorCategory::Storage,

            // P2P
            Self::ConnectionFailed { .. }
            | Self::TrustGateBlocked { .. }
            | Self::TopicNotSubscribed { .. }
            | Self::PeerUnreachable { .. }
            | Self::NetworkTimeout { .. } => ErrorCategory::P2P,

            // Invariant
            Self::CausalOrderViolation { .. }
            | Self::FinalityRegression { .. }
            | Self::TrustDecayViolation { .. } => ErrorCategory::Invariant,

            // Other
            Self::Internal(_) | Self::InvalidInput(_) | Self::Cancelled { .. } => {
                ErrorCategory::Other
            }
        }
    }

    /// Ist der Fehler wiederholbar?
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::NetworkTimeout { .. }
                | Self::PeerUnreachable { .. }
                | Self::ConnectionFailed { .. }
        )
    }

    /// Ist der Fehler ein Ressourcen-Problem?
    pub fn is_resource_exhaustion(&self) -> bool {
        matches!(
            self,
            Self::GasExhausted { .. } | Self::ManaExhausted { .. } | Self::StoreFull { .. }
        )
    }

    /// Soll der Fehler geloggt werden?
    pub fn should_log(&self) -> bool {
        // Invarianten-Verletzungen immer loggen
        self.category() == ErrorCategory::Invariant
    }

    /// Vorgeschlagene Retry-Wartezeit in Millisekunden
    pub fn suggested_retry_ms(&self) -> Option<u64> {
        match self {
            Self::NetworkTimeout { timeout_ms } => Some(timeout_ms * 2),
            Self::PeerUnreachable { attempts, .. } => Some(100 * 2u64.pow(*attempts)),
            Self::ConnectionFailed { .. } => Some(1000),
            Self::ManaExhausted { .. } => Some(60_000), // Mana regeneriert
            _ => None,
        }
    }
}

// ============================================================================
// Type Aliases
// ============================================================================

/// Result-Typ für Execution-Operationen
pub type ExecutionResult<T> = Result<T, ExecutionError>;

// ============================================================================
// From-Implementierungen
// ============================================================================

impl From<std::io::Error> for ExecutionError {
    fn from(err: std::io::Error) -> Self {
        Self::Internal(format!("IO error: {}", err))
    }
}

impl From<serde_json::Error> for ExecutionError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError {
            reason: err.to_string(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        let gas_err = ExecutionError::GasExhausted {
            required: 100,
            available: 50,
        };
        assert_eq!(gas_err.category(), ErrorCategory::Vm);

        let trust_err = ExecutionError::TrustGateBlocked {
            required: 0.8,
            actual: 0.5,
        };
        assert_eq!(trust_err.category(), ErrorCategory::P2P);

        let causal_err = ExecutionError::CausalOrderViolation {
            event_id: "abc".into(),
            parent_id: "def".into(),
        };
        assert_eq!(causal_err.category(), ErrorCategory::Invariant);
    }

    #[test]
    fn test_retryable() {
        let timeout = ExecutionError::NetworkTimeout { timeout_ms: 5000 };
        assert!(timeout.is_retryable());

        let gas = ExecutionError::GasExhausted {
            required: 100,
            available: 0,
        };
        assert!(!gas.is_retryable());
    }

    #[test]
    fn test_resource_exhaustion() {
        let gas = ExecutionError::GasExhausted {
            required: 100,
            available: 0,
        };
        assert!(gas.is_resource_exhaustion());

        let mana = ExecutionError::ManaExhausted {
            required: 50,
            available: 10,
        };
        assert!(mana.is_resource_exhaustion());

        let store = ExecutionError::StoreFull {
            store: "events".into(),
            used_bytes: 1000,
            max_bytes: 1000,
        };
        assert!(store.is_resource_exhaustion());
    }

    #[test]
    fn test_suggested_retry() {
        let timeout = ExecutionError::NetworkTimeout { timeout_ms: 1000 };
        assert_eq!(timeout.suggested_retry_ms(), Some(2000));

        let unreachable = ExecutionError::PeerUnreachable {
            peer_id: "abc".into(),
            attempts: 3,
        };
        assert_eq!(unreachable.suggested_retry_ms(), Some(800)); // 100 * 2^3

        let gas = ExecutionError::GasExhausted {
            required: 100,
            available: 0,
        };
        assert_eq!(gas.suggested_retry_ms(), None);
    }

    #[test]
    fn test_error_display() {
        let err = ExecutionError::TrustGateBlocked {
            required: 0.8,
            actual: 0.5,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("0.800"));
        assert!(msg.contains("0.500"));
    }

    #[test]
    fn test_should_log() {
        let invariant = ExecutionError::CausalOrderViolation {
            event_id: "abc".into(),
            parent_id: "def".into(),
        };
        assert!(invariant.should_log());

        let gas = ExecutionError::GasExhausted {
            required: 100,
            available: 0,
        };
        assert!(!gas.should_log());
    }
}
