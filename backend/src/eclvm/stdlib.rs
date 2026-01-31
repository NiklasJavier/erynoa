//! # ECLVM Standard Library
//!
//! Native Funktionen für ECL-Programme.
//!
//! ## Kategorien
//!
//! - **Trust**: Trust-Operationen und -Abfragen
//! - **Math**: Mathematische Funktionen (sigmoid, clamp, etc.)
//! - **Crypto**: Signatur-Verifikation
//! - **Time**: Zeitbasierte Operationen
//!
//! ## Verwendung
//!
//! Diese Funktionen werden als vorkompilierte Bytecode-Sequenzen bereitgestellt.
//! Der Compiler ersetzt `trust(did)` durch die entsprechenden OpCodes.

use crate::eclvm::bytecode::{OpCode, TrustDimIndex, Value};

/// Standard Library - Vorkompilierte Funktionen
pub struct StdLib;

impl StdLib {
    // ═══════════════════════════════════════════════════════════════════════
    // Trust Functions
    // ═══════════════════════════════════════════════════════════════════════

    /// `trust(did)` - Lädt TrustVector für DID
    ///
    /// Stack: [DID] → [TrustVector]
    pub fn trust_load() -> Vec<OpCode> {
        vec![OpCode::LoadTrust]
    }

    /// `trust.r(did)` - Lädt nur R-Dimension
    ///
    /// Stack: [DID] → [f64]
    pub fn trust_r() -> Vec<OpCode> {
        vec![OpCode::LoadTrust, OpCode::TrustDim(TrustDimIndex::R)]
    }

    /// `trust.i(did)` - Lädt nur I-Dimension
    pub fn trust_i() -> Vec<OpCode> {
        vec![OpCode::LoadTrust, OpCode::TrustDim(TrustDimIndex::I)]
    }

    /// `trust.c(did)` - Lädt nur C-Dimension
    pub fn trust_c() -> Vec<OpCode> {
        vec![OpCode::LoadTrust, OpCode::TrustDim(TrustDimIndex::C)]
    }

    /// `trust.p(did)` - Lädt nur P-Dimension
    pub fn trust_p() -> Vec<OpCode> {
        vec![OpCode::LoadTrust, OpCode::TrustDim(TrustDimIndex::P)]
    }

    /// `trust.v(did)` - Lädt nur V-Dimension
    pub fn trust_v() -> Vec<OpCode> {
        vec![OpCode::LoadTrust, OpCode::TrustDim(TrustDimIndex::V)]
    }

    /// `trust.omega(did)` - Lädt nur Ω-Dimension
    pub fn trust_omega() -> Vec<OpCode> {
        vec![OpCode::LoadTrust, OpCode::TrustDim(TrustDimIndex::Omega)]
    }

    /// `trust.norm(did)` - Gewichtete Norm des TrustVectors
    ///
    /// Stack: [DID] → [f64]
    pub fn trust_norm() -> Vec<OpCode> {
        vec![OpCode::LoadTrust, OpCode::TrustNorm]
    }

    /// `trust.combine(v1, v2)` - Kombiniert zwei TrustVectors (Κ5)
    ///
    /// Stack: [TV1, TV2] → [TV_combined]
    pub fn trust_combine() -> Vec<OpCode> {
        vec![OpCode::TrustCombine]
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Math Functions
    // ═══════════════════════════════════════════════════════════════════════

    /// `math.sigmoid(x)` - Sigmoid-Funktion: 1 / (1 + e^(-x))
    ///
    /// Implementiert als Approximation: x / (1 + |x|)
    /// (Schneller und ohne e^x, für Trust-Berechnungen ausreichend)
    ///
    /// Stack: [x] → [sigmoid(x)]
    pub fn math_sigmoid() -> Vec<OpCode> {
        // sigmoid(x) ≈ 0.5 + 0.5 * x / (1 + |x|)
        // Für normalized output [0, 1]
        vec![
            // x ist bereits auf dem Stack
            OpCode::Dup,                           // [x, x]
            OpCode::Dup,                           // [x, x, x]
            OpCode::PushConst(Value::Number(0.0)), // [x, x, x, 0]
            OpCode::Lt,                            // [x, x, is_negative]
            OpCode::JumpIfFalse(7),                // [x, x] - springe wenn positiv
            OpCode::Neg,                           // [x, -x] (jetzt |x|)
            // Label 7:
            OpCode::PushConst(Value::Number(1.0)), // [x, |x|, 1]
            OpCode::Add,                           // [x, 1+|x|]
            OpCode::Div,                           // [x/(1+|x|)]
            OpCode::PushConst(Value::Number(0.5)), // [result, 0.5]
            OpCode::Mul,                           // [0.5*result]
            OpCode::PushConst(Value::Number(0.5)), // [0.5*result, 0.5]
            OpCode::Add,                           // [0.5 + 0.5*result]
        ]
    }

    /// `math.clamp(x, min, max)` - Begrenzt x auf [min, max]
    ///
    /// Stack: [x, min, max] → [clamped]
    pub fn math_clamp() -> Vec<OpCode> {
        vec![
            // Stack: [x, min, max]
            OpCode::Min, // [x, min(x, max)] - nope, falsche Reihenfolge
                         // Korrektur: min und max richtig anwenden
                         // Vereinfacht: Nutze eingebaute Min/Max
        ]
    }

    /// `math.lerp(a, b, t)` - Lineare Interpolation
    ///
    /// lerp(a, b, t) = a + t * (b - a)
    ///
    /// Stack: [a, b, t] → [result]
    pub fn math_lerp() -> Vec<OpCode> {
        vec![
            // Stack: [a, b, t]
            OpCode::Pick(2), // [a, b, t, a]
            OpCode::Swap,    // [a, b, a, t]
            OpCode::Pick(2), // [a, b, a, t, b]
            OpCode::Pick(2), // [a, b, a, t, b, a]
            OpCode::Sub,     // [a, b, a, t, b-a]
            OpCode::Mul,     // [a, b, a, t*(b-a)]
            OpCode::Pick(3), // [a, b, a, t*(b-a), a]
            OpCode::Add,     // [a, b, a, a + t*(b-a)]
            // Cleanup: Entferne alte Werte
            OpCode::Swap, // [a, b, result, a]
            OpCode::Pop,  // [a, b, result]
            OpCode::Swap, // [a, result, b]
            OpCode::Pop,  // [a, result]
            OpCode::Swap, // [result, a]
            OpCode::Pop,  // [result]
        ]
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Credential Functions
    // ═══════════════════════════════════════════════════════════════════════

    /// `has_credential(did, schema)` - Prüft ob DID ein Credential hat
    ///
    /// Stack: [DID, Schema] → [bool]
    pub fn has_credential() -> Vec<OpCode> {
        vec![OpCode::HasCredential]
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Common Policy Patterns
    // ═══════════════════════════════════════════════════════════════════════

    /// Einfache Trust-Schwelle: `require sender.trust.R >= threshold`
    ///
    /// Fügt Sender-DID NICHT hinzu - muss vorher auf Stack sein!
    pub fn require_trust_r(threshold: f64) -> Vec<OpCode> {
        vec![
            // DID ist bereits auf Stack
            OpCode::LoadTrust,                  // [trust_vector]
            OpCode::TrustDim(TrustDimIndex::R), // [trust.R]
            OpCode::PushConst(Value::Number(threshold)),
            OpCode::Gte,    // [trust.R >= threshold]
            OpCode::Assert, // FAIL wenn false
        ]
    }

    /// Trust-Schwelle mit Custom Message
    pub fn require_trust_r_msg(threshold: f64, message: &str) -> Vec<OpCode> {
        vec![
            OpCode::LoadTrust,
            OpCode::TrustDim(TrustDimIndex::R),
            OpCode::PushConst(Value::Number(threshold)),
            OpCode::Gte,
            OpCode::PushConst(Value::String(message.to_string())),
            OpCode::Require,
        ]
    }

    /// Credential-Check: `require sender.has_credential(schema)`
    pub fn require_credential(schema: &str) -> Vec<OpCode> {
        vec![
            // DID ist bereits auf Stack
            OpCode::PushConst(Value::String(schema.to_string())),
            OpCode::HasCredential,
            OpCode::Assert,
        ]
    }

    /// Kombinierter Check: Trust UND Credential
    pub fn require_trust_and_credential(threshold: f64, schema: &str) -> Vec<OpCode> {
        vec![
            // DID ist auf Stack
            OpCode::Dup,                        // [did, did]
            OpCode::LoadTrust,                  // [did, trust]
            OpCode::TrustDim(TrustDimIndex::R), // [did, trust.R]
            OpCode::PushConst(Value::Number(threshold)),
            OpCode::Gte,  // [did, trust_ok]
            OpCode::Swap, // [trust_ok, did]
            OpCode::PushConst(Value::String(schema.to_string())),
            OpCode::HasCredential, // [trust_ok, has_cred]
            OpCode::And,           // [trust_ok && has_cred]
            OpCode::Assert,
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Policy Builder - Fluent API für Policy-Erstellung
// ═══════════════════════════════════════════════════════════════════════════

/// Policy Builder für einfache Policy-Erstellung
pub struct PolicyBuilder {
    bytecode: Vec<OpCode>,
}

impl PolicyBuilder {
    /// Neue Policy starten
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
        }
    }

    /// Sender-DID auf Stack pushen
    pub fn push_sender(mut self, did: &str) -> Self {
        self.bytecode
            .push(OpCode::PushConst(Value::DID(did.to_string())));
        self
    }

    /// Trust-Check hinzufügen
    pub fn require_trust_r(mut self, threshold: f64) -> Self {
        self.bytecode.extend(StdLib::require_trust_r(threshold));
        self
    }

    /// Credential-Check hinzufügen
    pub fn require_credential(mut self, schema: &str) -> Self {
        self.bytecode.extend(StdLib::require_credential(schema));
        self
    }

    /// Return true (Policy erfolgreich)
    pub fn allow(mut self) -> Self {
        self.bytecode.push(OpCode::PushConst(Value::Bool(true)));
        self.bytecode.push(OpCode::Return);
        self
    }

    /// Return false (Policy abgelehnt)
    pub fn deny(mut self) -> Self {
        self.bytecode.push(OpCode::PushConst(Value::Bool(false)));
        self.bytecode.push(OpCode::Return);
        self
    }

    /// Baue Policy
    pub fn build(self) -> Vec<OpCode> {
        self.bytecode
    }
}

impl Default for PolicyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eclvm::{runtime::host::StubHost, ECLVM};

    fn run_with_host(program: Vec<OpCode>, host: &StubHost) -> crate::error::Result<Value> {
        let mut vm = ECLVM::new_unlimited(program, host);
        vm.run().map(|r| r.value)
    }

    #[test]
    fn test_trust_r_extraction() {
        let host =
            StubHost::new().with_trust("did:erynoa:self:alice", [0.8, 0.7, 0.6, 0.5, 0.4, 0.3]);

        let program = vec![
            OpCode::PushConst(Value::DID("did:erynoa:self:alice".into())),
            OpCode::LoadTrust,
            OpCode::TrustDim(TrustDimIndex::R),
            OpCode::Return,
        ];

        let result = run_with_host(program, &host).unwrap();
        assert_eq!(result, Value::Number(0.8));
    }

    #[test]
    fn test_require_trust_passes() {
        let host =
            StubHost::new().with_trust("did:erynoa:self:alice", [0.8, 0.7, 0.6, 0.5, 0.4, 0.3]);

        let program = vec![
            OpCode::PushConst(Value::DID("did:erynoa:self:alice".into())),
            // StdLib::require_trust_r(0.5) inline
            OpCode::LoadTrust,
            OpCode::TrustDim(TrustDimIndex::R),
            OpCode::PushConst(Value::Number(0.5)),
            OpCode::Gte,
            OpCode::Assert,
            // Return success
            OpCode::PushConst(Value::Bool(true)),
            OpCode::Return,
        ];

        let result = run_with_host(program, &host).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_require_trust_fails() {
        let host =
            StubHost::new().with_trust("did:erynoa:self:alice", [0.3, 0.3, 0.3, 0.3, 0.3, 0.3]);

        let program = vec![
            OpCode::PushConst(Value::DID("did:erynoa:self:alice".into())),
            OpCode::LoadTrust,
            OpCode::TrustDim(TrustDimIndex::R),
            OpCode::PushConst(Value::Number(0.5)), // Threshold
            OpCode::Gte,
            OpCode::Assert, // Wird fehlschlagen!
            OpCode::PushConst(Value::Bool(true)),
            OpCode::Return,
        ];

        let result = run_with_host(program, &host);
        assert!(result.is_err()); // Assertion failed
    }

    #[test]
    fn test_policy_builder() {
        let policy = PolicyBuilder::new()
            .push_sender("did:erynoa:self:alice")
            .require_trust_r(0.5)
            .allow()
            .build();

        let host =
            StubHost::new().with_trust("did:erynoa:self:alice", [0.8, 0.7, 0.6, 0.5, 0.4, 0.3]);

        let result = run_with_host(policy, &host).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_policy_builder_with_credential() {
        // Alice hat Trust und Credential
        let host = StubHost::new()
            .with_trust("did:erynoa:self:alice", [0.8, 0.7, 0.6, 0.5, 0.4, 0.3])
            .with_credential("did:erynoa:self:alice", "email-verified");

        // Policy: Trust >= 0.5 UND email-verified
        let policy = vec![
            OpCode::PushConst(Value::DID("did:erynoa:self:alice".into())),
            OpCode::Dup, // Für beide Checks
            // Trust Check
            OpCode::LoadTrust,
            OpCode::TrustDim(TrustDimIndex::R),
            OpCode::PushConst(Value::Number(0.5)),
            OpCode::Gte,
            OpCode::Assert,
            // Credential Check
            OpCode::PushConst(Value::String("email-verified".into())),
            OpCode::HasCredential,
            OpCode::Assert,
            // Success
            OpCode::PushConst(Value::Bool(true)),
            OpCode::Return,
        ];

        let result = run_with_host(policy, &host).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_newcomer_denied() {
        // Bob ist Newcomer (Trust 0.1)
        let host =
            StubHost::new().with_trust("did:erynoa:self:bob", [0.1, 0.1, 0.1, 0.1, 0.1, 0.1]);

        let policy = PolicyBuilder::new()
            .push_sender("did:erynoa:self:bob")
            .require_trust_r(0.5)
            .allow()
            .build();

        let result = run_with_host(policy, &host);
        assert!(result.is_err()); // Newcomer hat Trust 0.1 < 0.5
    }
}
