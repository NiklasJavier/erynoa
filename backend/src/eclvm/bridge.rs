//! # ECLVM Bridge – Core ↔ ECLVM Adjunktion
//!
//! Implementiert die Adjunktions-Traits für kategorien-theoretisch
//! korrekte Transformationen zwischen Core-Domäne und ECLVM-Werten.
//!
//! ## Mathematische Grundlage
//!
//! ```text
//! F: Core → ECLVM    (Linker Adjunkt: embed)
//! G: ECLVM → Core    (Rechter Adjunkt: interpret)
//!
//! Zig-Zag Identity: ∀x ∈ Core: G(F(x)) ≅ x
//! ```
//!
//! ## Design-Prinzipien
//!
//! 1. **Verlustfreie Einbettung**: `embed()` bewahrt alle Informationen
//! 2. **Partielle Interpretation**: `interpret()` kann fehlschlagen
//! 3. **Zig-Zag-Identität**: Roundtrip Core → ECLVM → Core ist isomorph
//! 4. **Typ-Sicherheit**: Explizite Fehler bei Type Mismatch

use crate::domain::unified::{Cost, FinalityLevel, TemporalCoord, TrustVector6D, UniversalId};

use super::bytecode::Value;
use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// InterpretError – Fehler bei ECLVM → Core Transformation
// ============================================================================

/// Fehler bei der Interpretation von ECLVM-Werten
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InterpretError {
    /// Typ-Mismatch (erwartet vs. erhalten)
    TypeMismatch {
        expected: &'static str,
        actual: String,
    },
    /// Ungültige Länge für Byte-Arrays
    InvalidLength { expected: usize, actual: usize },
    /// Wert außerhalb des gültigen Bereichs
    OutOfRange { field: &'static str, value: f64 },
    /// Fehlendes Feld in Struktur
    MissingField { field: &'static str },
    /// Ungültiger Enum-Wert
    InvalidEnumValue { enum_name: &'static str, value: i64 },
    /// Nested Error
    Nested {
        field: &'static str,
        error: Box<InterpretError>,
    },
}

impl fmt::Display for InterpretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeMismatch { expected, actual } => {
                write!(
                    f,
                    "Typ-Mismatch: erwartet {}, erhalten {}",
                    expected, actual
                )
            }
            Self::InvalidLength { expected, actual } => {
                write!(
                    f,
                    "Ungültige Länge: erwartet {}, erhalten {}",
                    expected, actual
                )
            }
            Self::OutOfRange { field, value } => {
                write!(f, "Wert außerhalb Bereich für '{}': {}", field, value)
            }
            Self::MissingField { field } => {
                write!(f, "Fehlendes Feld: {}", field)
            }
            Self::InvalidEnumValue { enum_name, value } => {
                write!(f, "Ungültiger {}-Wert: {}", enum_name, value)
            }
            Self::Nested { field, error } => {
                write!(f, "Fehler in '{}': {}", field, error)
            }
        }
    }
}

impl std::error::Error for InterpretError {}

// ============================================================================
// CoreToEclvm – Linker Adjunkt F: Core → ECLVM
// ============================================================================

/// Linker Adjunkt F: Core → ECLVM
///
/// Wandelt Core-Typen in ECLVM-Werte um.
/// Diese Transformation ist immer erfolgreich (total function).
///
/// # Invariante
///
/// `embed()` bewahrt alle Informationen für späteren Roundtrip.
pub trait CoreToEclvm {
    /// Konvertiere in ECLVM-Value
    fn embed(&self) -> Value;
}

// ============================================================================
// EclvmToCore – Rechter Adjunkt G: ECLVM → Core
// ============================================================================

/// Rechter Adjunkt G: ECLVM → Core
///
/// Wandelt ECLVM-Werte zurück in Core-Typen.
/// Diese Transformation kann fehlschlagen (partial function).
///
/// # Invariante
///
/// Wenn `x: T`, dann `T::interpret(&x.embed()) == Ok(x)` (Zig-Zag)
pub trait EclvmToCore: Sized {
    /// Versuche ECLVM-Value als Core-Typ zu interpretieren
    fn interpret(value: &Value) -> Result<Self, InterpretError>;
}

// ============================================================================
// Implementierungen: UniversalId
// ============================================================================

impl CoreToEclvm for UniversalId {
    fn embed(&self) -> Value {
        // UniversalId als Array von Bytes (32 Zahlen)
        let bytes = self.as_bytes();
        Value::Array(bytes.iter().map(|&b| Value::Number(b as f64)).collect())
    }
}

impl EclvmToCore for UniversalId {
    fn interpret(value: &Value) -> Result<Self, InterpretError> {
        match value {
            Value::Array(arr) if arr.len() == 32 => {
                let mut bytes = [0u8; 32];
                for (i, v) in arr.iter().enumerate() {
                    if let Value::Number(n) = v {
                        if *n < 0.0 || *n > 255.0 || n.fract() != 0.0 {
                            return Err(InterpretError::OutOfRange {
                                field: "byte",
                                value: *n,
                            });
                        }
                        bytes[i] = *n as u8;
                    } else {
                        return Err(InterpretError::TypeMismatch {
                            expected: "number",
                            actual: v.type_name().to_string(),
                        });
                    }
                }
                Ok(UniversalId::from_bytes(bytes))
            }
            Value::Array(arr) => Err(InterpretError::InvalidLength {
                expected: 32,
                actual: arr.len(),
            }),
            _ => Err(InterpretError::TypeMismatch {
                expected: "array[32]",
                actual: value.type_name().to_string(),
            }),
        }
    }
}

// ============================================================================
// Implementierungen: TrustVector6D
// ============================================================================

impl CoreToEclvm for TrustVector6D {
    fn embed(&self) -> Value {
        // Direkt als ECLVM TrustVector (nativ unterstützt)
        Value::TrustVector([
            self.r as f64,
            self.i as f64,
            self.c as f64,
            self.p as f64,
            self.v as f64,
            self.omega as f64,
        ])
    }
}

impl EclvmToCore for TrustVector6D {
    fn interpret(value: &Value) -> Result<Self, InterpretError> {
        match value {
            Value::TrustVector(arr) => {
                // Validiere Bereich [0, 1]
                for (i, &v) in arr.iter().enumerate() {
                    if v < 0.0 || v > 1.0 {
                        let dim_name = match i {
                            0 => "r",
                            1 => "i",
                            2 => "c",
                            3 => "p",
                            4 => "v",
                            5 => "omega",
                            _ => "unknown",
                        };
                        return Err(InterpretError::OutOfRange {
                            field: dim_name,
                            value: v,
                        });
                    }
                }
                Ok(TrustVector6D::new(
                    arr[0] as f32,
                    arr[1] as f32,
                    arr[2] as f32,
                    arr[3] as f32,
                    arr[4] as f32,
                    arr[5] as f32,
                ))
            }
            Value::Array(arr) if arr.len() == 6 => {
                let mut dims = [0.0f32; 6];
                for (i, v) in arr.iter().enumerate() {
                    if let Value::Number(n) = v {
                        if *n < 0.0 || *n > 1.0 {
                            return Err(InterpretError::OutOfRange {
                                field: "dimension",
                                value: *n,
                            });
                        }
                        dims[i] = *n as f32;
                    } else {
                        return Err(InterpretError::TypeMismatch {
                            expected: "number",
                            actual: v.type_name().to_string(),
                        });
                    }
                }
                Ok(TrustVector6D::from_array(dims))
            }
            Value::Array(arr) => Err(InterpretError::InvalidLength {
                expected: 6,
                actual: arr.len(),
            }),
            _ => Err(InterpretError::TypeMismatch {
                expected: "trust_vector or array[6]",
                actual: value.type_name().to_string(),
            }),
        }
    }
}

// ============================================================================
// Implementierungen: Cost
// ============================================================================

impl CoreToEclvm for Cost {
    fn embed(&self) -> Value {
        // Cost als Array [gas, mana, trust_risk]
        Value::Array(vec![
            Value::Number(self.gas as f64),
            Value::Number(self.mana as f64),
            Value::Number(self.trust_risk as f64),
        ])
    }
}

impl EclvmToCore for Cost {
    fn interpret(value: &Value) -> Result<Self, InterpretError> {
        match value {
            Value::Array(arr) if arr.len() == 3 => {
                let gas = match &arr[0] {
                    Value::Number(n) if *n >= 0.0 => *n as u64,
                    Value::Number(n) => {
                        return Err(InterpretError::OutOfRange {
                            field: "gas",
                            value: *n,
                        })
                    }
                    v => {
                        return Err(InterpretError::Nested {
                            field: "gas",
                            error: Box::new(InterpretError::TypeMismatch {
                                expected: "number",
                                actual: v.type_name().to_string(),
                            }),
                        })
                    }
                };

                let mana = match &arr[1] {
                    Value::Number(n) if *n >= 0.0 => *n as u64,
                    Value::Number(n) => {
                        return Err(InterpretError::OutOfRange {
                            field: "mana",
                            value: *n,
                        })
                    }
                    v => {
                        return Err(InterpretError::Nested {
                            field: "mana",
                            error: Box::new(InterpretError::TypeMismatch {
                                expected: "number",
                                actual: v.type_name().to_string(),
                            }),
                        })
                    }
                };

                let trust_risk = match &arr[2] {
                    Value::Number(n) if (0.0..=1.0).contains(n) => *n as f32,
                    Value::Number(n) => {
                        return Err(InterpretError::OutOfRange {
                            field: "trust_risk",
                            value: *n,
                        })
                    }
                    v => {
                        return Err(InterpretError::Nested {
                            field: "trust_risk",
                            error: Box::new(InterpretError::TypeMismatch {
                                expected: "number",
                                actual: v.type_name().to_string(),
                            }),
                        })
                    }
                };

                Ok(Cost::new(gas, mana, trust_risk))
            }
            Value::Array(arr) => Err(InterpretError::InvalidLength {
                expected: 3,
                actual: arr.len(),
            }),
            _ => Err(InterpretError::TypeMismatch {
                expected: "array[3]",
                actual: value.type_name().to_string(),
            }),
        }
    }
}

// ============================================================================
// Implementierungen: TemporalCoord
// ============================================================================

impl CoreToEclvm for TemporalCoord {
    fn embed(&self) -> Value {
        // TemporalCoord als Array [wall_time, lamport, node_hash]
        Value::Array(vec![
            Value::Number(self.wall_time() as f64),
            Value::Number(self.lamport() as f64),
            Value::Number(self.node_hash() as f64),
        ])
    }
}

impl EclvmToCore for TemporalCoord {
    fn interpret(value: &Value) -> Result<Self, InterpretError> {
        match value {
            Value::Array(arr) if arr.len() == 3 => {
                let wall_time = match &arr[0] {
                    Value::Number(n) if *n >= 0.0 => *n as u64,
                    Value::Number(n) => {
                        return Err(InterpretError::OutOfRange {
                            field: "wall_time",
                            value: *n,
                        })
                    }
                    v => {
                        return Err(InterpretError::TypeMismatch {
                            expected: "number",
                            actual: v.type_name().to_string(),
                        })
                    }
                };

                let lamport = match &arr[1] {
                    Value::Number(n) if *n >= 0.0 => *n as u32,
                    Value::Number(n) => {
                        return Err(InterpretError::OutOfRange {
                            field: "lamport",
                            value: *n,
                        })
                    }
                    v => {
                        return Err(InterpretError::TypeMismatch {
                            expected: "number",
                            actual: v.type_name().to_string(),
                        })
                    }
                };

                let node_hash = match &arr[2] {
                    Value::Number(n) if *n >= 0.0 => *n as u32,
                    Value::Number(n) => {
                        return Err(InterpretError::OutOfRange {
                            field: "node_hash",
                            value: *n,
                        })
                    }
                    v => {
                        return Err(InterpretError::TypeMismatch {
                            expected: "number",
                            actual: v.type_name().to_string(),
                        })
                    }
                };

                Ok(TemporalCoord::new(wall_time, lamport, node_hash))
            }
            Value::Array(arr) => Err(InterpretError::InvalidLength {
                expected: 3,
                actual: arr.len(),
            }),
            _ => Err(InterpretError::TypeMismatch {
                expected: "array[3]",
                actual: value.type_name().to_string(),
            }),
        }
    }
}

// ============================================================================
// Implementierungen: FinalityLevel
// ============================================================================

impl CoreToEclvm for FinalityLevel {
    fn embed(&self) -> Value {
        let level = match self {
            FinalityLevel::Nascent => 0,
            FinalityLevel::Validated => 1,
            FinalityLevel::Witnessed => 2,
            FinalityLevel::Anchored => 3,
            FinalityLevel::Eternal => 4,
        };
        Value::Number(level as f64)
    }
}

impl EclvmToCore for FinalityLevel {
    fn interpret(value: &Value) -> Result<Self, InterpretError> {
        match value {
            Value::Number(n) => {
                let level = *n as u8;
                match level {
                    0 => Ok(FinalityLevel::Nascent),
                    1 => Ok(FinalityLevel::Validated),
                    2 => Ok(FinalityLevel::Witnessed),
                    3 => Ok(FinalityLevel::Anchored),
                    4 => Ok(FinalityLevel::Eternal),
                    _ => Err(InterpretError::InvalidEnumValue {
                        enum_name: "FinalityLevel",
                        value: *n as i64,
                    }),
                }
            }
            _ => Err(InterpretError::TypeMismatch {
                expected: "number",
                actual: value.type_name().to_string(),
            }),
        }
    }
}

// ============================================================================
// Zig-Zag Identity Prüfung
// ============================================================================

/// Prüfe Zig-Zag Identity für einen Typ
///
/// Diese Funktion validiert: `T::interpret(&x.embed()) == Ok(x)`
pub fn verify_zigzag<T>(value: &T) -> bool
where
    T: CoreToEclvm + EclvmToCore + PartialEq + Clone,
{
    let embedded = value.embed();
    match T::interpret(&embedded) {
        Ok(interpreted) => *value == interpreted,
        Err(_) => false,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ────────────────────────────────────────────────────────────────────────
    // Zig-Zag Identity Tests
    // ────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_zigzag_universal_id() {
        let id = UniversalId::new(UniversalId::TAG_EVENT, 1, b"test-event");
        assert!(verify_zigzag(&id), "UniversalId Zig-Zag failed");

        // Null ID
        assert!(verify_zigzag(&UniversalId::NULL), "NULL ID Zig-Zag failed");
    }

    #[test]
    fn test_zigzag_trust_vector() {
        let trust = TrustVector6D::new(0.8, 0.7, 0.6, 0.5, 0.4, 0.3);
        assert!(verify_zigzag(&trust), "TrustVector6D Zig-Zag failed");

        // Edge cases
        assert!(
            verify_zigzag(&TrustVector6D::NEWCOMER),
            "NEWCOMER Zig-Zag failed"
        );
        assert!(verify_zigzag(&TrustVector6D::MAX), "MAX Zig-Zag failed");
        assert!(verify_zigzag(&TrustVector6D::ZERO), "ZERO Zig-Zag failed");
    }

    #[test]
    fn test_zigzag_cost() {
        let cost = Cost::new(1000, 500, 0.25);
        assert!(verify_zigzag(&cost), "Cost Zig-Zag failed");

        // Edge cases
        assert!(verify_zigzag(&Cost::ZERO), "ZERO Cost Zig-Zag failed");

        // Large values within f64 precision (~2^53)
        let large_cost = Cost::new(1_000_000_000_000, 1_000_000_000_000, 1.0);
        assert!(verify_zigzag(&large_cost), "Large Cost Zig-Zag failed");
    }

    #[test]
    fn test_zigzag_temporal_coord() {
        let coord = TemporalCoord::new(1234567890123, 42, 0xDEADBEEF);
        assert!(verify_zigzag(&coord), "TemporalCoord Zig-Zag failed");

        // Genesis
        assert!(
            verify_zigzag(&TemporalCoord::GENESIS),
            "GENESIS Zig-Zag failed"
        );
    }

    #[test]
    fn test_zigzag_finality_level() {
        for level in [
            FinalityLevel::Nascent,
            FinalityLevel::Validated,
            FinalityLevel::Witnessed,
            FinalityLevel::Anchored,
            FinalityLevel::Eternal,
        ] {
            assert!(
                verify_zigzag(&level),
                "FinalityLevel {:?} Zig-Zag failed",
                level
            );
        }
    }

    // ────────────────────────────────────────────────────────────────────────
    // Error Handling Tests
    // ────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_interpret_type_mismatch() {
        // UniversalId erwartet Array, bekommt String
        let result = UniversalId::interpret(&Value::String("not-an-id".to_string()));
        assert!(matches!(result, Err(InterpretError::TypeMismatch { .. })));

        // TrustVector erwartet 6 Elemente
        let result = TrustVector6D::interpret(&Value::Array(vec![Value::Number(0.5)]));
        assert!(matches!(result, Err(InterpretError::InvalidLength { .. })));
    }

    #[test]
    fn test_interpret_out_of_range() {
        // Trust dimension > 1.0
        let invalid_trust = Value::TrustVector([0.5, 0.5, 1.5, 0.5, 0.5, 0.5]);
        let result = TrustVector6D::interpret(&invalid_trust);
        assert!(matches!(result, Err(InterpretError::OutOfRange { .. })));

        // Negative gas
        let invalid_cost = Value::Array(vec![
            Value::Number(-100.0),
            Value::Number(50.0),
            Value::Number(0.1),
        ]);
        let result = Cost::interpret(&invalid_cost);
        assert!(matches!(result, Err(InterpretError::OutOfRange { .. })));
    }

    #[test]
    fn test_interpret_invalid_enum() {
        // Invalid finality level
        let result = FinalityLevel::interpret(&Value::Number(99.0));
        assert!(matches!(
            result,
            Err(InterpretError::InvalidEnumValue { .. })
        ));
    }

    // ────────────────────────────────────────────────────────────────────────
    // Embedding Tests
    // ────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_embed_trust_vector_uses_native_type() {
        let trust = TrustVector6D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5);
        let embedded = trust.embed();
        assert!(matches!(embedded, Value::TrustVector(_)));
    }

    #[test]
    fn test_embed_cost_structure() {
        let cost = Cost::new(100, 50, 0.25);
        let embedded = cost.embed();

        if let Value::Array(arr) = embedded {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Number(100.0));
            assert_eq!(arr[1], Value::Number(50.0));
            assert_eq!(arr[2], Value::Number(0.25));
        } else {
            panic!("Cost should embed as Array");
        }
    }

    #[test]
    fn test_embed_universal_id_structure() {
        let id = UniversalId::new(UniversalId::TAG_EVENT, 1, b"test");
        let embedded = id.embed();

        if let Value::Array(arr) = embedded {
            assert_eq!(arr.len(), 32);
            // Type tag should be in first 2 bytes
            if let (Value::Number(b0), Value::Number(b1)) = (&arr[0], &arr[1]) {
                let tag = ((*b0 as u16) << 8) | (*b1 as u16);
                assert_eq!(tag, UniversalId::TAG_EVENT);
            }
        } else {
            panic!("UniversalId should embed as Array[32]");
        }
    }

    // ────────────────────────────────────────────────────────────────────────
    // Alternative Interpretation Paths
    // ────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_trust_vector_from_array() {
        // TrustVector6D kann auch von Array[6] interpretiert werden
        let array_form = Value::Array(vec![
            Value::Number(0.1),
            Value::Number(0.2),
            Value::Number(0.3),
            Value::Number(0.4),
            Value::Number(0.5),
            Value::Number(0.6),
        ]);

        let result = TrustVector6D::interpret(&array_form);
        assert!(result.is_ok());
        let trust = result.unwrap();
        assert!((trust.r - 0.1).abs() < 0.001);
        assert!((trust.omega - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_temporal_coord_structure() {
        let coord = TemporalCoord::new(1000000, 42, 0x12345678);
        let embedded = coord.embed();

        if let Value::Array(arr) = embedded {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Number(1000000.0));
            assert_eq!(arr[1], Value::Number(42.0));
            assert_eq!(arr[2], Value::Number(0x12345678 as f64));
        } else {
            panic!("TemporalCoord should embed as Array[3]");
        }
    }
}
