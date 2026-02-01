//! # Schema-Registry & Versionierung
//!
//! Implementierung gemäß UDM §XIII - Versionierung & Migration.
//!
//! ## Überblick
//!
//! Die Schema-Registry ermöglicht:
//! - Versionsverwaltung für alle Datentypen
//! - Automatische Datenmigration bei Schema-Änderungen
//! - Forward-Compatibility durch strukturierte Migrations-Pfade
//!
//! ## Beispiel
//!
//! ```rust,ignore
//! use erynoa_api::domain::unified::schema::{SchemaRegistry, MigrationError};
//! use erynoa_api::domain::unified::UniversalId;
//!
//! let mut registry = SchemaRegistry::new();
//!
//! // Registriere aktuelle Versionen
//! registry.register_current(UniversalId::TAG_EVENT, 1);
//! registry.register_current(UniversalId::TAG_DID, 2);
//!
//! // Registriere Migration von v1 -> v2
//! registry.register_migration(
//!     UniversalId::TAG_DID,
//!     1,
//!     2,
//!     Box::new(|data| {
//!         // Migration-Logik
//!         Ok(data.to_vec())
//!     }),
//! );
//! ```

use std::collections::BTreeMap;
use std::sync::Arc;

use super::UniversalId;

// ============================================================================
// Types
// ============================================================================

/// Migration-Funktion Signatur
///
/// Nimmt Byte-Daten der alten Version und gibt Byte-Daten der neuen Version zurück.
pub type MigrationFn = Arc<dyn Fn(&[u8]) -> Result<Vec<u8>, MigrationError> + Send + Sync>;

/// Migrations-Pfad Schlüssel: (type_tag, from_version, to_version)
type MigrationKey = (u16, u16, u16);

// ============================================================================
// SchemaRegistry
// ============================================================================

/// Schema-Registry für Versionierung und automatische Migration
///
/// Verwaltet Schema-Versionen für alle `UniversalId`-basierten Datentypen
/// und stellt automatische Migrations-Pfade bereit.
///
/// # Design
///
/// - Thread-safe durch `Arc<dyn Fn>` Migrations-Funktionen
/// - Unterstützt Multi-Step-Migrationen (v1 → v2 → v3)
/// - Fehlende Migrations-Pfade werden als `MigrationError` gemeldet
#[derive(Default)]
pub struct SchemaRegistry {
    /// Aktuelle Versionen pro Type-Tag
    current_versions: BTreeMap<u16, u16>,
    /// Registrierte Migrations-Funktionen
    migrations: BTreeMap<MigrationKey, MigrationFn>,
}

impl SchemaRegistry {
    /// Erstellt eine neue leere Registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Erstellt eine Registry mit Standard-Versionen für alle bekannten Typen
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Registriere aktuelle Versionen für alle Kern-Typen
        registry.register_current(UniversalId::TAG_EVENT, 1);
        registry.register_current(UniversalId::TAG_DID, 1);
        registry.register_current(UniversalId::TAG_REALM, 1);
        registry.register_current(UniversalId::TAG_SAGA, 1);
        registry.register_current(UniversalId::TAG_BLUEPRINT, 1);
        registry.register_current(UniversalId::TAG_TRUST, 1);
        registry.register_current(UniversalId::TAG_STATE, 1);

        registry
    }

    /// Registriert die aktuelle Schema-Version für einen Typ
    ///
    /// # Parameter
    /// - `type_tag`: Der Type-Tag aus `UniversalId::TAG_*`
    /// - `version`: Die aktuelle Schema-Version (1-based)
    pub fn register_current(&mut self, type_tag: u16, version: u16) {
        self.current_versions.insert(type_tag, version);
    }

    /// Gibt die aktuelle Version für einen Typ zurück
    pub fn current_version(&self, type_tag: u16) -> Option<u16> {
        self.current_versions.get(&type_tag).copied()
    }

    /// Registriert eine Migration von einer Version zur nächsten
    ///
    /// # Parameter
    /// - `type_tag`: Der Type-Tag des zu migrierenden Typs
    /// - `from_version`: Ausgangs-Version
    /// - `to_version`: Ziel-Version
    /// - `migration`: Die Migration-Funktion
    ///
    /// # Hinweis
    ///
    /// Migrationen sollten nur zwischen aufeinanderfolgenden Versionen
    /// registriert werden (v1→v2, v2→v3). Multi-Step-Migrationen werden
    /// automatisch aufgelöst.
    pub fn register_migration(
        &mut self,
        type_tag: u16,
        from_version: u16,
        to_version: u16,
        migration: MigrationFn,
    ) {
        self.migrations
            .insert((type_tag, from_version, to_version), migration);
    }

    /// Migriert Daten wenn nötig auf die aktuelle Version
    ///
    /// # Parameter
    /// - `id`: Die UniversalId des Datensatzes (enthält Type-Tag und Version)
    /// - `data`: Die rohen Byte-Daten
    ///
    /// # Rückgabe
    /// - `Ok(Vec<u8>)` - Die (möglicherweise migrierten) Daten
    /// - `Err(MigrationError)` - Falls Migration fehlschlägt
    ///
    /// # Beispiel
    ///
    /// ```rust
    /// # use erynoa_api::domain::unified::schema::SchemaRegistry;
    /// # use erynoa_api::domain::unified::UniversalId;
    /// let registry = SchemaRegistry::with_defaults();
    /// let id = UniversalId::new(UniversalId::TAG_EVENT, 1, b"test");
    /// let data = vec![1, 2, 3];
    ///
    /// let result = registry.maybe_migrate(&id, &data);
    /// assert!(result.is_ok());
    /// ```
    pub fn maybe_migrate(&self, id: &UniversalId, data: &[u8]) -> Result<Vec<u8>, MigrationError> {
        let type_tag = id.type_tag();
        let data_version = id.version();

        let current = self
            .current_versions
            .get(&type_tag)
            .ok_or(MigrationError::UnknownType { type_tag })?;

        // Keine Migration nötig
        if data_version == *current {
            return Ok(data.to_vec());
        }

        // Downgrade nicht erlaubt
        if data_version > *current {
            return Err(MigrationError::VersionTooHigh {
                type_tag,
                data_version,
                current_version: *current,
            });
        }

        // Finde und führe Migrations-Pfad aus
        let mut version = data_version;
        let mut result = data.to_vec();

        while version < *current {
            let next_version = version + 1;
            let key = (type_tag, version, next_version);

            let migration = self
                .migrations
                .get(&key)
                .ok_or(MigrationError::NoMigrationPath {
                    type_tag,
                    from_version: version,
                    to_version: next_version,
                })?;

            result = migration(&result)?;
            version = next_version;
        }

        Ok(result)
    }

    /// Prüft ob eine Migration erforderlich ist
    pub fn needs_migration(&self, id: &UniversalId) -> bool {
        let type_tag = id.type_tag();
        let data_version = id.version();

        self.current_versions
            .get(&type_tag)
            .map(|current| data_version < *current)
            .unwrap_or(false)
    }

    /// Gibt alle registrierten Migrationen zurück (für Debugging)
    pub fn list_migrations(&self) -> Vec<(u16, u16, u16)> {
        self.migrations.keys().copied().collect()
    }

    /// Validiert, dass für alle Versionen lückenlose Migrations-Pfade existieren
    pub fn validate_migration_paths(&self) -> Result<(), MigrationError> {
        for (&type_tag, &current) in &self.current_versions {
            // Prüfe ob Pfad von v1 bis current existiert
            for version in 1..current {
                let key = (type_tag, version, version + 1);
                if !self.migrations.contains_key(&key) {
                    return Err(MigrationError::NoMigrationPath {
                        type_tag,
                        from_version: version,
                        to_version: version + 1,
                    });
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for SchemaRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SchemaRegistry")
            .field("current_versions", &self.current_versions)
            .field("migration_count", &self.migrations.len())
            .finish()
    }
}

// ============================================================================
// MigrationError
// ============================================================================

/// Fehler bei Schema-Migration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    /// Unbekannter Typ (nicht in Registry registriert)
    UnknownType { type_tag: u16 },

    /// Kein Migrations-Pfad gefunden
    NoMigrationPath {
        type_tag: u16,
        from_version: u16,
        to_version: u16,
    },

    /// Daten-Version ist höher als aktuelle Version (Downgrade nicht möglich)
    VersionTooHigh {
        type_tag: u16,
        data_version: u16,
        current_version: u16,
    },

    /// Migration fehlgeschlagen
    MigrationFailed { reason: String },

    /// Ungültige Daten für Migration
    InvalidData { reason: String },
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownType { type_tag } => {
                write!(f, "Unknown type tag: 0x{:04X}", type_tag)
            }
            Self::NoMigrationPath {
                type_tag,
                from_version,
                to_version,
            } => {
                write!(
                    f,
                    "No migration path for type 0x{:04X} from v{} to v{}",
                    type_tag, from_version, to_version
                )
            }
            Self::VersionTooHigh {
                type_tag,
                data_version,
                current_version,
            } => {
                write!(
                    f,
                    "Data version {} is higher than current {} for type 0x{:04X}",
                    data_version, current_version, type_tag
                )
            }
            Self::MigrationFailed { reason } => {
                write!(f, "Migration failed: {}", reason)
            }
            Self::InvalidData { reason } => {
                write!(f, "Invalid data for migration: {}", reason)
            }
        }
    }
}

impl std::error::Error for MigrationError {}

// ============================================================================
// Vordefinierte Migrationen
// ============================================================================

/// Erstellt eine Identity-Migration (Daten unverändert kopieren)
pub fn identity_migration() -> MigrationFn {
    Arc::new(|data| Ok(data.to_vec()))
}

/// Erstellt eine Migration die ein Feld anhängt
pub fn append_field_migration(default_value: Vec<u8>) -> MigrationFn {
    Arc::new(move |data| {
        let mut result = data.to_vec();
        result.extend_from_slice(&default_value);
        Ok(result)
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_defaults() {
        let registry = SchemaRegistry::with_defaults();

        assert_eq!(registry.current_version(UniversalId::TAG_EVENT), Some(1));
        assert_eq!(registry.current_version(UniversalId::TAG_DID), Some(1));
        assert_eq!(registry.current_version(UniversalId::TAG_REALM), Some(1));
        assert_eq!(registry.current_version(0xFFFF), None);
    }

    #[test]
    fn test_no_migration_needed() {
        let registry = SchemaRegistry::with_defaults();
        let id = UniversalId::new(UniversalId::TAG_EVENT, 1, b"test");
        let data = vec![1, 2, 3, 4];

        let result = registry.maybe_migrate(&id, &data).unwrap();
        assert_eq!(result, data);
        assert!(!registry.needs_migration(&id));
    }

    #[test]
    fn test_migration_path() {
        let mut registry = SchemaRegistry::new();
        registry.register_current(UniversalId::TAG_DID, 3);

        // Registriere Migrations-Pfad: v1 → v2 → v3
        registry.register_migration(
            UniversalId::TAG_DID,
            1,
            2,
            Arc::new(|data| {
                let mut result = data.to_vec();
                result.push(0xAA); // Füge Byte hinzu
                Ok(result)
            }),
        );

        registry.register_migration(
            UniversalId::TAG_DID,
            2,
            3,
            Arc::new(|data| {
                let mut result = data.to_vec();
                result.push(0xBB); // Füge weiteres Byte hinzu
                Ok(result)
            }),
        );

        // Migriere von v1 zu v3
        let id = UniversalId::new(UniversalId::TAG_DID, 1, b"old-did");
        let data = vec![1, 2, 3];

        let result = registry.maybe_migrate(&id, &data).unwrap();
        assert_eq!(result, vec![1, 2, 3, 0xAA, 0xBB]);
    }

    #[test]
    fn test_unknown_type() {
        let registry = SchemaRegistry::new();
        let id = UniversalId::new(0x9999, 1, b"unknown");
        let data = vec![1, 2, 3];

        let result = registry.maybe_migrate(&id, &data);
        assert!(matches!(result, Err(MigrationError::UnknownType { .. })));
    }

    #[test]
    fn test_missing_migration_path() {
        let mut registry = SchemaRegistry::new();
        registry.register_current(UniversalId::TAG_EVENT, 2);
        // Keine Migration registriert!

        let id = UniversalId::new(UniversalId::TAG_EVENT, 1, b"old");
        let data = vec![1, 2, 3];

        let result = registry.maybe_migrate(&id, &data);
        assert!(matches!(
            result,
            Err(MigrationError::NoMigrationPath { .. })
        ));
    }

    #[test]
    fn test_version_too_high() {
        let mut registry = SchemaRegistry::new();
        registry.register_current(UniversalId::TAG_EVENT, 1);

        // Daten mit Version 5 (höher als aktuell)
        let id = UniversalId::new(UniversalId::TAG_EVENT, 5, b"future");
        let data = vec![1, 2, 3];

        let result = registry.maybe_migrate(&id, &data);
        assert!(matches!(result, Err(MigrationError::VersionTooHigh { .. })));
    }

    #[test]
    fn test_validate_migration_paths() {
        let mut registry = SchemaRegistry::new();
        registry.register_current(UniversalId::TAG_DID, 3);

        // Nur v1→v2 registriert, v2→v3 fehlt
        registry.register_migration(UniversalId::TAG_DID, 1, 2, identity_migration());

        let result = registry.validate_migration_paths();
        assert!(matches!(
            result,
            Err(MigrationError::NoMigrationPath {
                from_version: 2,
                to_version: 3,
                ..
            })
        ));

        // Jetzt v2→v3 hinzufügen
        registry.register_migration(UniversalId::TAG_DID, 2, 3, identity_migration());

        assert!(registry.validate_migration_paths().is_ok());
    }

    #[test]
    fn test_list_migrations() {
        let mut registry = SchemaRegistry::new();
        registry.register_migration(UniversalId::TAG_DID, 1, 2, identity_migration());
        registry.register_migration(UniversalId::TAG_EVENT, 1, 2, identity_migration());

        let migrations = registry.list_migrations();
        assert_eq!(migrations.len(), 2);
    }

    #[test]
    fn test_append_field_migration() {
        let migration = append_field_migration(vec![0xFF, 0x00]);
        let result = migration(&[1, 2, 3]).unwrap();
        assert_eq!(result, vec![1, 2, 3, 0xFF, 0x00]);
    }
}
