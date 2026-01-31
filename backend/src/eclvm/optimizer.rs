//! # ECLVM Optimizer
//!
//! Bytecode-Optimierungen für bessere Laufzeit-Performance.
//!
//! ## Optimierungen
//!
//! - **Constant Folding**: Compile-Zeit-Berechnung konstanter Ausdrücke
//! - **Dead Code Elimination**: Entfernung unerreichbarer Instruktionen
//! - **Peephole Optimization**: Lokale Muster-Ersetzungen
//!
//! ## Beispiel
//!
//! ```rust,ignore
//! use erynoa_api::eclvm::{Optimizer, OpCode, Value};
//!
//! let program = vec![
//!     OpCode::PushConst(Value::Number(2.0)),
//!     OpCode::PushConst(Value::Number(3.0)),
//!     OpCode::Add,  // → wird zu PushConst(5.0)
//! ];
//!
//! let optimized = Optimizer::new().optimize(program);
//! ```

use crate::eclvm::bytecode::{OpCode, Value};

/// Optimizer für ECLVM Bytecode
pub struct Optimizer {
    /// Aktiviere Constant Folding
    constant_folding: bool,
    /// Aktiviere Dead Code Elimination
    dead_code_elimination: bool,
    /// Aktiviere Peephole Optimierung
    peephole: bool,
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Optimizer {
    /// Erstelle neuen Optimizer mit allen Optimierungen aktiviert
    pub fn new() -> Self {
        Self {
            constant_folding: true,
            dead_code_elimination: true,
            peephole: true,
        }
    }

    /// Konfiguriere Constant Folding
    pub fn with_constant_folding(mut self, enabled: bool) -> Self {
        self.constant_folding = enabled;
        self
    }

    /// Konfiguriere Dead Code Elimination
    pub fn with_dead_code_elimination(mut self, enabled: bool) -> Self {
        self.dead_code_elimination = enabled;
        self
    }

    /// Konfiguriere Peephole Optimierung
    pub fn with_peephole(mut self, enabled: bool) -> Self {
        self.peephole = enabled;
        self
    }

    /// Optimiere Bytecode
    pub fn optimize(&self, mut program: Vec<OpCode>) -> Vec<OpCode> {
        // Mehrere Durchläufe für bessere Ergebnisse
        for _ in 0..3 {
            let before_len = program.len();

            if self.constant_folding {
                program = self.fold_constants(program);
            }

            if self.peephole {
                program = self.peephole_optimize(program);
            }

            if self.dead_code_elimination {
                program = self.eliminate_dead_code(program);
            }

            // Keine Änderung mehr → fertig
            if program.len() == before_len {
                break;
            }
        }

        program
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Constant Folding
    // ═══════════════════════════════════════════════════════════════════════

    /// Berechne konstante Ausdrücke zur Compile-Zeit
    fn fold_constants(&self, program: Vec<OpCode>) -> Vec<OpCode> {
        let mut result = Vec::with_capacity(program.len());
        let mut i = 0;

        while i < program.len() {
            // Pattern: PushConst(a), PushConst(b), BinaryOp
            if i + 2 < program.len() {
                if let (
                    OpCode::PushConst(Value::Number(a)),
                    OpCode::PushConst(Value::Number(b)),
                    op,
                ) = (&program[i], &program[i + 1], &program[i + 2])
                {
                    if let Some(folded) = self.fold_binary_number(*a, *b, op) {
                        result.push(OpCode::PushConst(Value::Number(folded)));
                        i += 3;
                        continue;
                    }
                }

                // Boolean folding
                if let (OpCode::PushConst(Value::Bool(a)), OpCode::PushConst(Value::Bool(b)), op) =
                    (&program[i], &program[i + 1], &program[i + 2])
                {
                    if let Some(folded) = self.fold_binary_bool(*a, *b, op) {
                        result.push(OpCode::PushConst(Value::Bool(folded)));
                        i += 3;
                        continue;
                    }
                }
            }

            // Pattern: PushConst(a), UnaryOp
            if i + 1 < program.len() {
                if let (OpCode::PushConst(Value::Number(a)), OpCode::Neg) =
                    (&program[i], &program[i + 1])
                {
                    result.push(OpCode::PushConst(Value::Number(-a)));
                    i += 2;
                    continue;
                }

                if let (OpCode::PushConst(Value::Bool(a)), OpCode::Not) =
                    (&program[i], &program[i + 1])
                {
                    result.push(OpCode::PushConst(Value::Bool(!a)));
                    i += 2;
                    continue;
                }
            }

            // Keine Optimierung möglich
            result.push(program[i].clone());
            i += 1;
        }

        result
    }

    fn fold_binary_number(&self, a: f64, b: f64, op: &OpCode) -> Option<f64> {
        match op {
            OpCode::Add => Some(a + b),
            OpCode::Sub => Some(a - b),
            OpCode::Mul => Some(a * b),
            OpCode::Div if b != 0.0 => Some(a / b),
            OpCode::Mod if b != 0.0 => Some(a % b),
            OpCode::Min => Some(a.min(b)),
            OpCode::Max => Some(a.max(b)),
            _ => None,
        }
    }

    fn fold_binary_bool(&self, a: bool, b: bool, op: &OpCode) -> Option<bool> {
        match op {
            OpCode::And => Some(a && b),
            OpCode::Or => Some(a || b),
            _ => None,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Peephole Optimization
    // ═══════════════════════════════════════════════════════════════════════

    /// Lokale Muster-Ersetzungen
    fn peephole_optimize(&self, program: Vec<OpCode>) -> Vec<OpCode> {
        let mut result = Vec::with_capacity(program.len());
        let mut i = 0;

        while i < program.len() {
            // Pattern: Push + Pop → nichts
            if i + 1 < program.len() {
                if let (OpCode::PushConst(_), OpCode::Pop) = (&program[i], &program[i + 1]) {
                    i += 2;
                    continue;
                }
            }

            // Pattern: Dup + Pop → nichts
            if i + 1 < program.len() {
                if let (OpCode::Dup, OpCode::Pop) = (&program[i], &program[i + 1]) {
                    i += 2;
                    continue;
                }
            }

            // Pattern: Jump to next instruction → nichts
            if let OpCode::Jump(addr) = &program[i] {
                if *addr == i + 1 {
                    i += 1;
                    continue;
                }
            }

            // Pattern: Not + Not → nichts
            if i + 1 < program.len() {
                if let (OpCode::Not, OpCode::Not) = (&program[i], &program[i + 1]) {
                    i += 2;
                    continue;
                }
            }

            // Pattern: Neg + Neg → nichts
            if i + 1 < program.len() {
                if let (OpCode::Neg, OpCode::Neg) = (&program[i], &program[i + 1]) {
                    i += 2;
                    continue;
                }
            }

            // Pattern: Swap + Swap → nichts
            if i + 1 < program.len() {
                if let (OpCode::Swap, OpCode::Swap) = (&program[i], &program[i + 1]) {
                    i += 2;
                    continue;
                }
            }

            // Pattern: PushConst(true) + JumpIfFalse → nichts (Bedingung immer true)
            if i + 1 < program.len() {
                if let (OpCode::PushConst(Value::Bool(true)), OpCode::JumpIfFalse(_)) =
                    (&program[i], &program[i + 1])
                {
                    i += 2;
                    continue;
                }
            }

            // Pattern: PushConst(false) + JumpIfTrue → nichts (Bedingung immer false)
            if i + 1 < program.len() {
                if let (OpCode::PushConst(Value::Bool(false)), OpCode::JumpIfTrue(_)) =
                    (&program[i], &program[i + 1])
                {
                    i += 2;
                    continue;
                }
            }

            // Pattern: PushConst(0) + Add → nichts
            if i + 1 < program.len() {
                if let (OpCode::PushConst(Value::Number(n)), OpCode::Add) =
                    (&program[i], &program[i + 1])
                {
                    if *n == 0.0 {
                        i += 2;
                        continue;
                    }
                }
            }

            // Pattern: PushConst(1) + Mul → nichts
            if i + 1 < program.len() {
                if let (OpCode::PushConst(Value::Number(n)), OpCode::Mul) =
                    (&program[i], &program[i + 1])
                {
                    if *n == 1.0 {
                        i += 2;
                        continue;
                    }
                }
            }

            // Keine Optimierung möglich
            result.push(program[i].clone());
            i += 1;
        }

        result
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Dead Code Elimination
    // ═══════════════════════════════════════════════════════════════════════

    /// Entferne unerreichbaren Code
    fn eliminate_dead_code(&self, program: Vec<OpCode>) -> Vec<OpCode> {
        if program.is_empty() {
            return program;
        }

        // Finde alle erreichbaren Instruktionen
        let mut reachable = vec![false; program.len()];
        let mut worklist = vec![0usize]; // Start bei Instruktion 0

        while let Some(idx) = worklist.pop() {
            if idx >= program.len() || reachable[idx] {
                continue;
            }

            reachable[idx] = true;

            match &program[idx] {
                OpCode::Jump(target) => {
                    worklist.push(*target);
                }
                OpCode::JumpIfFalse(target) | OpCode::JumpIfTrue(target) => {
                    worklist.push(idx + 1); // Fallthrough
                    worklist.push(*target);
                }
                OpCode::Return | OpCode::Halt | OpCode::Abort => {
                    // Kein Nachfolger
                }
                OpCode::Call(target, _) => {
                    worklist.push(*target);
                    worklist.push(idx + 1);
                }
                _ => {
                    worklist.push(idx + 1);
                }
            }
        }

        // Erstelle neues Programm mit Adress-Mapping
        let mut addr_map = vec![0usize; program.len()];
        let mut new_idx = 0;
        for (old_idx, is_reachable) in reachable.iter().enumerate() {
            if *is_reachable {
                addr_map[old_idx] = new_idx;
                new_idx += 1;
            }
        }

        // Kopiere erreichbare Instruktionen und aktualisiere Adressen
        let mut result = Vec::new();
        for (idx, op) in program.into_iter().enumerate() {
            if !reachable[idx] {
                continue;
            }

            let new_op = match op {
                OpCode::Jump(target) => OpCode::Jump(addr_map[target]),
                OpCode::JumpIfFalse(target) => OpCode::JumpIfFalse(addr_map[target]),
                OpCode::JumpIfTrue(target) => OpCode::JumpIfTrue(addr_map[target]),
                OpCode::Call(target, argc) => OpCode::Call(addr_map[target], argc),
                other => other,
            };
            result.push(new_op);
        }

        result
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Statistics
// ═══════════════════════════════════════════════════════════════════════════

/// Statistiken über Optimierungen
#[derive(Debug, Clone, Default)]
pub struct OptimizationStats {
    /// Original-Größe
    pub original_size: usize,
    /// Optimierte Größe
    pub optimized_size: usize,
    /// Anzahl Constant Folding
    pub constants_folded: usize,
    /// Anzahl eliminierter Dead Code
    pub dead_code_eliminated: usize,
    /// Anzahl Peephole-Optimierungen
    pub peephole_applied: usize,
}

impl OptimizationStats {
    /// Berechne Einsparung in Prozent
    pub fn savings_percent(&self) -> f64 {
        if self.original_size == 0 {
            0.0
        } else {
            (1.0 - self.optimized_size as f64 / self.original_size as f64) * 100.0
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_folding_add() {
        let program = vec![
            OpCode::PushConst(Value::Number(2.0)),
            OpCode::PushConst(Value::Number(3.0)),
            OpCode::Add,
            OpCode::Return,
        ];

        let optimized = Optimizer::new().optimize(program);

        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized[0], OpCode::PushConst(Value::Number(5.0)));
        assert_eq!(optimized[1], OpCode::Return);
    }

    #[test]
    fn test_constant_folding_mul() {
        let program = vec![
            OpCode::PushConst(Value::Number(4.0)),
            OpCode::PushConst(Value::Number(5.0)),
            OpCode::Mul,
            OpCode::Return,
        ];

        let optimized = Optimizer::new().optimize(program);

        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized[0], OpCode::PushConst(Value::Number(20.0)));
    }

    #[test]
    fn test_constant_folding_nested() {
        // (2 + 3) * 4 = 20
        let program = vec![
            OpCode::PushConst(Value::Number(2.0)),
            OpCode::PushConst(Value::Number(3.0)),
            OpCode::Add,
            OpCode::PushConst(Value::Number(4.0)),
            OpCode::Mul,
            OpCode::Return,
        ];

        let optimized = Optimizer::new().optimize(program);

        // Nach mehreren Durchläufen sollte alles gefaltet sein
        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized[0], OpCode::PushConst(Value::Number(20.0)));
    }

    #[test]
    fn test_constant_folding_boolean() {
        let program = vec![
            OpCode::PushConst(Value::Bool(true)),
            OpCode::PushConst(Value::Bool(false)),
            OpCode::And,
            OpCode::Return,
        ];

        let optimized = Optimizer::new().optimize(program);

        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized[0], OpCode::PushConst(Value::Bool(false)));
    }

    #[test]
    fn test_constant_folding_negation() {
        let program = vec![
            OpCode::PushConst(Value::Number(5.0)),
            OpCode::Neg,
            OpCode::Return,
        ];

        let optimized = Optimizer::new().optimize(program);

        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized[0], OpCode::PushConst(Value::Number(-5.0)));
    }

    #[test]
    fn test_peephole_push_pop() {
        let program = vec![
            OpCode::PushConst(Value::Number(1.0)),
            OpCode::Pop,
            OpCode::PushConst(Value::Number(2.0)),
            OpCode::Return,
        ];

        let optimized = Optimizer::new().optimize(program);

        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized[0], OpCode::PushConst(Value::Number(2.0)));
    }

    #[test]
    fn test_peephole_double_not() {
        let program = vec![
            OpCode::PushConst(Value::Bool(true)),
            OpCode::Not,
            OpCode::Not,
            OpCode::Return,
        ];

        let optimized = Optimizer::new().optimize(program);

        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized[0], OpCode::PushConst(Value::Bool(true)));
    }

    #[test]
    fn test_peephole_double_neg() {
        let program = vec![
            OpCode::PushConst(Value::Number(5.0)),
            OpCode::Neg,
            OpCode::Neg,
            OpCode::Return,
        ];

        let optimized = Optimizer::new().optimize(program);

        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized[0], OpCode::PushConst(Value::Number(5.0)));
    }

    #[test]
    fn test_peephole_add_zero() {
        let program = vec![
            OpCode::PushConst(Value::Number(5.0)),
            OpCode::PushConst(Value::Number(0.0)),
            OpCode::Add,
            OpCode::Return,
        ];

        let optimized = Optimizer::new().optimize(program);

        // Erst constant folding: 5 + 0 = 5
        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized[0], OpCode::PushConst(Value::Number(5.0)));
    }

    #[test]
    fn test_dead_code_elimination() {
        // Code nach unbedingtem Return ist unerreichbar
        let program = vec![
            OpCode::PushConst(Value::Number(1.0)),
            OpCode::Return,
            OpCode::PushConst(Value::Number(999.0)), // Dead code (nach Return)
            OpCode::Return,
        ];

        let optimized = Optimizer::new().optimize(program);

        // Alles nach dem ersten Return sollte entfernt werden
        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized[0], OpCode::PushConst(Value::Number(1.0)));
        assert_eq!(optimized[1], OpCode::Return);
    }

    #[test]
    fn test_dead_code_after_return() {
        let program = vec![
            OpCode::PushConst(Value::Bool(true)),
            OpCode::Return,
            OpCode::PushConst(Value::Number(999.0)), // Dead code
            OpCode::Pop,
        ];

        let optimized = Optimizer::new().optimize(program);

        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized[0], OpCode::PushConst(Value::Bool(true)));
        assert_eq!(optimized[1], OpCode::Return);
    }

    #[test]
    fn test_optimizer_disabled() {
        let program = vec![
            OpCode::PushConst(Value::Number(2.0)),
            OpCode::PushConst(Value::Number(3.0)),
            OpCode::Add,
            OpCode::Return,
        ];

        let optimized = Optimizer::new()
            .with_constant_folding(false)
            .with_peephole(false)
            .with_dead_code_elimination(false)
            .optimize(program.clone());

        // Keine Optimierung
        assert_eq!(optimized.len(), program.len());
    }

    #[test]
    fn test_complex_optimization() {
        // Komplexes Beispiel mit Constant Folding: 2 + 3, dann 5 * 1
        let program = vec![
            OpCode::PushConst(Value::Number(2.0)),
            OpCode::PushConst(Value::Number(3.0)),
            OpCode::Add,
            OpCode::PushConst(Value::Number(1.0)),
            OpCode::Mul,
            OpCode::Return,
        ];

        let optimized = Optimizer::new().optimize(program);

        // Nach Constant Folding: 2+3=5, dann Peephole: 5*1=5 (Mul mit 1 wird entfernt)
        // Ergebnis: PushConst(5), Return
        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized[0], OpCode::PushConst(Value::Number(5.0)));
        assert_eq!(optimized[1], OpCode::Return);
    }

    #[test]
    fn test_optimization_stats() {
        let stats = OptimizationStats {
            original_size: 10,
            optimized_size: 4,
            constants_folded: 3,
            dead_code_eliminated: 2,
            peephole_applied: 1,
        };

        assert_eq!(stats.savings_percent(), 60.0);
    }
}
