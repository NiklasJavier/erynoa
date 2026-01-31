//! # ECLVM Bytecode
//!
//! Definiert die OpCodes (Instruktionen) und Werte-Typen der ECLVM.
//!
//! ## OpCode Design
//!
//! Stack-basierte Architektur:
//! - Operanden werden vom Stack geholt
//! - Ergebnisse werden auf den Stack gelegt
//! - Keine Register (einfacher, deterministischer)

use serde::{Deserialize, Serialize};

/// OpCode - Eine einzelne VM-Instruktion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpCode {
    // ═══════════════════════════════════════════════════════════════
    // Stack Manipulation
    // ═══════════════════════════════════════════════════════════════

    /// Lade einen konstanten Wert auf den Stack
    PushConst(Value),

    /// Entferne das oberste Element vom Stack
    Pop,

    /// Dupliziere das oberste Element
    Dup,

    /// Tausche die obersten zwei Elemente
    Swap,

    /// Kopiere Element an Index n auf den Stack (0 = top)
    Pick(u8),

    // ═══════════════════════════════════════════════════════════════
    // Arithmetik
    // ═══════════════════════════════════════════════════════════════

    /// Addition: a + b
    Add,

    /// Subtraktion: a - b
    Sub,

    /// Multiplikation: a * b
    Mul,

    /// Division: a / b (mit Division-by-Zero Check)
    Div,

    /// Modulo: a % b
    Mod,

    /// Negation: -a
    Neg,

    /// Minimum: min(a, b)
    Min,

    /// Maximum: max(a, b)
    Max,

    // ═══════════════════════════════════════════════════════════════
    // Vergleiche
    // ═══════════════════════════════════════════════════════════════

    /// Gleichheit: a == b
    Eq,

    /// Ungleichheit: a != b
    Neq,

    /// Größer als: a > b
    Gt,

    /// Größer oder gleich: a >= b
    Gte,

    /// Kleiner als: a < b
    Lt,

    /// Kleiner oder gleich: a <= b
    Lte,

    // ═══════════════════════════════════════════════════════════════
    // Logik
    // ═══════════════════════════════════════════════════════════════

    /// Logisches UND: a && b
    And,

    /// Logisches ODER: a || b
    Or,

    /// Logisches NICHT: !a
    Not,

    // ═══════════════════════════════════════════════════════════════
    // Control Flow
    // ═══════════════════════════════════════════════════════════════

    /// Unbedingter Sprung zu Adresse
    Jump(usize),

    /// Sprung wenn Top-of-Stack false ist
    JumpIfFalse(usize),

    /// Sprung wenn Top-of-Stack true ist
    JumpIfTrue(usize),

    /// Funktion aufrufen (Adresse, Anzahl Argumente)
    Call(usize, u8),

    /// Aus Funktion zurückkehren
    Return,

    // ═══════════════════════════════════════════════════════════════
    // TrustVector6D Operationen (Erynoa-spezifisch)
    // ═══════════════════════════════════════════════════════════════

    /// Extrahiere Dimension aus TrustVector: tv[dim]
    /// Stack: [TrustVector] → [Number]
    TrustDim(TrustDimIndex),

    /// Berechne gewichtete Norm eines TrustVectors
    /// Stack: [TrustVector] → [Number]
    TrustNorm,

    /// Kombiniere zwei TrustVectors (Κ5: t₁ ⊕ t₂)
    /// Stack: [TrustVector, TrustVector] → [TrustVector]
    TrustCombine,

    /// Erstelle TrustVector aus 6 Zahlen auf dem Stack
    /// Stack: [r, i, c, p, v, omega] → [TrustVector]
    TrustCreate,

    // ═══════════════════════════════════════════════════════════════
    // Host Calls (Sandbox-Schnittstelle)
    // ═══════════════════════════════════════════════════════════════

    /// Lade Trust-Vektor für DID vom Host
    /// Stack: [DID] → [TrustVector]
    LoadTrust,

    /// Prüfe ob DID ein Credential hat
    /// Stack: [DID, Schema] → [Bool]
    HasCredential,

    /// Löse DID auf (prüfe Existenz)
    /// Stack: [DID] → [Bool]
    ResolveDID,

    /// Hole Balance für DID
    /// Stack: [DID] → [Number]
    GetBalance,

    /// Aktuellen Timestamp holen
    /// Stack: [] → [Number]
    GetTimestamp,

    /// Log-Nachricht ausgeben (für Debugging)
    /// Stack: [String] → []
    Log,

    // ═══════════════════════════════════════════════════════════════
    // Assertions & Guards
    // ═══════════════════════════════════════════════════════════════

    /// Assert: Wenn Top-of-Stack false, Execution abbrechen
    /// Stack: [Bool] → []
    Assert,

    /// Require mit Fehlermeldung
    /// Stack: [Bool, String] → []
    Require,

    // ═══════════════════════════════════════════════════════════════
    // Programm-Ende
    // ═══════════════════════════════════════════════════════════════

    /// Programm beenden (Success)
    Halt,

    /// Programm mit Fehler beenden
    Abort,
}

impl OpCode {
    /// Gas-Kosten für diese Operation
    pub fn gas_cost(&self) -> u64 {
        match self {
            // Billige Operationen
            OpCode::PushConst(_) => 1,
            OpCode::Pop => 1,
            OpCode::Dup => 1,
            OpCode::Swap => 1,
            OpCode::Pick(_) => 2,

            // Arithmetik
            OpCode::Add | OpCode::Sub => 2,
            OpCode::Mul => 3,
            OpCode::Div | OpCode::Mod => 5,
            OpCode::Neg => 1,
            OpCode::Min | OpCode::Max => 2,

            // Vergleiche
            OpCode::Eq | OpCode::Neq => 2,
            OpCode::Gt | OpCode::Gte | OpCode::Lt | OpCode::Lte => 2,

            // Logik
            OpCode::And | OpCode::Or => 2,
            OpCode::Not => 1,

            // Control Flow
            OpCode::Jump(_) => 1,
            OpCode::JumpIfFalse(_) | OpCode::JumpIfTrue(_) => 2,
            OpCode::Call(_, _) => 10,
            OpCode::Return => 5,

            // TrustVector (etwas teurer wegen Float-Operationen)
            OpCode::TrustDim(_) => 3,
            OpCode::TrustNorm => 10,
            OpCode::TrustCombine => 15,
            OpCode::TrustCreate => 8,

            // Host Calls (teuer wegen I/O)
            OpCode::LoadTrust => 100,
            OpCode::HasCredential => 50,
            OpCode::ResolveDID => 50,
            OpCode::GetBalance => 50,
            OpCode::GetTimestamp => 5,
            OpCode::Log => 20,

            // Assertions
            OpCode::Assert => 3,
            OpCode::Require => 5,

            // Ende
            OpCode::Halt => 0,
            OpCode::Abort => 0,
        }
    }
}

/// Index für TrustVector6D Dimensionen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustDimIndex {
    /// R - Reliability
    R = 0,
    /// I - Integrity
    I = 1,
    /// C - Competence
    C = 2,
    /// P - Prestige
    P = 3,
    /// V - Vigilance
    V = 4,
    /// Ω - Omega
    Omega = 5,
}

/// Value - Ein Wert auf dem VM-Stack
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// Null/None
    Null,

    /// Boolean
    Bool(bool),

    /// Zahl (f64 für Flexibilität)
    Number(f64),

    /// String
    String(String),

    /// DID (decentralized identifier)
    DID(String),

    /// 6D Trust-Vektor [R, I, C, P, V, Ω]
    TrustVector([f64; 6]),

    /// Array von Values
    Array(Vec<Value>),
}

impl Value {
    /// Typ-Name für Fehlermeldungen
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::DID(_) => "did",
            Value::TrustVector(_) => "trust_vector",
            Value::Array(_) => "array",
        }
    }

    /// Als Boolean interpretieren
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            Value::Null => Some(false),
            Value::Number(n) => Some(*n != 0.0),
            _ => None,
        }
    }

    /// Als Number interpretieren
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    /// Als String interpretieren
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            Value::DID(d) => Some(d),
            _ => None,
        }
    }

    /// Als TrustVector interpretieren
    pub fn as_trust_vector(&self) -> Option<[f64; 6]> {
        match self {
            Value::TrustVector(tv) => Some(*tv),
            _ => None,
        }
    }

    /// Ist der Wert "truthy"?
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::DID(d) => !d.is_empty(),
            Value::TrustVector(_) => true,
            Value::Array(a) => !a.is_empty(),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::DID(d) => write!(f, "did:{}", d),
            Value::TrustVector(tv) => {
                write!(f, "[R:{:.2}, I:{:.2}, C:{:.2}, P:{:.2}, V:{:.2}, Ω:{:.2}]",
                    tv[0], tv[1], tv[2], tv[3], tv[4], tv[5])
            }
            Value::Array(a) => {
                write!(f, "[")?;
                for (i, v) in a.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_types() {
        let null = Value::Null;
        assert_eq!(null.type_name(), "null");
        assert!(!null.is_truthy());

        let num = Value::Number(42.0);
        assert_eq!(num.as_number(), Some(42.0));
        assert!(num.is_truthy());

        let zero = Value::Number(0.0);
        assert!(!zero.is_truthy());

        let tv = Value::TrustVector([0.5, 0.5, 0.5, 0.5, 0.5, 0.5]);
        assert!(tv.is_truthy());
        assert_eq!(tv.as_trust_vector(), Some([0.5, 0.5, 0.5, 0.5, 0.5, 0.5]));
    }

    #[test]
    fn test_gas_costs() {
        // Host Calls sollten teurer sein
        assert!(OpCode::LoadTrust.gas_cost() > OpCode::Add.gas_cost());
        assert!(OpCode::HasCredential.gas_cost() > OpCode::Mul.gas_cost());

        // Einfache Ops sollten billig sein
        assert!(OpCode::PushConst(Value::Null).gas_cost() <= 2);
        assert!(OpCode::Pop.gas_cost() <= 2);
    }

    #[test]
    fn test_value_display() {
        let tv = Value::TrustVector([0.8, 0.7, 0.6, 0.5, 0.4, 0.3]);
        let display = format!("{}", tv);
        assert!(display.contains("R:0.80"));
        assert!(display.contains("Ω:0.30"));
    }
}
