//! # ECLVM - Die Virtual Machine
//!
//! Stack-basierte, Gas-metered VM für deterministische Policy-Ausführung.
//!
//! ## Architektur
//!
//! ```text
//! ┌───────────────────────────────────────────────────────────┐
//! │                        ECLVM                              │
//! │  ┌─────────────────────────────────────────────────────┐  │
//! │  │  Stack: [Value, Value, Value, ...]                  │  │
//! │  │         └───────────────────────▲                   │  │
//! │  │                                 │ push/pop          │  │
//! │  └─────────────────────────────────┼───────────────────┘  │
//! │                                    │                      │
//! │  ┌─────────────┐  ┌───────────────┴─────────────────┐    │
//! │  │ IP: 0x0042  │  │  OpCode: Add, Sub, LoadTrust... │    │
//! │  └─────────────┘  └─────────────────────────────────┘    │
//! │                                    │                      │
//! │  ┌─────────────┐                   │                      │
//! │  │ Gas: 847    │◀──────────────────┘ consume()           │
//! │  └─────────────┘                                         │
//! └───────────────────────────────────────────────────────────┘
//! ```

use super::gas::GasMeter;
use super::host::HostInterface;
#[cfg(test)]
use super::host::StubHost;
#[cfg(test)]
use crate::eclvm::bytecode::TrustDimIndex;
use crate::eclvm::bytecode::{OpCode, Value};
use crate::error::{ApiError, Result};
use anyhow::anyhow;

/// Kontrollfluss-Ergebnis einer Instruktion
#[derive(Debug)]
enum ControlFlow {
    /// Weitermachen mit nächster Instruktion
    Continue,
    /// Programm beenden mit Rückgabewert
    Return(Value),
    /// Fehler aufgetreten
    Error(String),
}

/// ECLVM - Die Erynoa Configuration Language Virtual Machine
pub struct ECLVM<'a> {
    /// Der Operanden-Stack
    stack: Vec<Value>,

    /// Instruction Pointer
    ip: usize,

    /// Das Bytecode-Programm
    program: Vec<OpCode>,

    /// Gas Meter für DoS-Schutz
    gas: GasMeter,

    /// Host Interface für externe Aufrufe
    host: &'a dyn HostInterface,

    /// Call Stack für Funktionsaufrufe
    call_stack: Vec<usize>,

    /// Max Stack-Tiefe (DoS-Schutz)
    max_stack_depth: usize,
}

/// Ergebnis einer VM-Ausführung
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Rückgabewert
    pub value: Value,
    /// Verbrauchtes Gas
    pub gas_used: u64,
    /// Log-Nachrichten während der Ausführung
    pub logs: Vec<String>,
}

impl<'a> ECLVM<'a> {
    /// Erstelle neue VM mit Programm und Gas-Limit
    pub fn new(program: Vec<OpCode>, gas_limit: u64, host: &'a dyn HostInterface) -> Self {
        Self {
            stack: Vec::with_capacity(256),
            ip: 0,
            program,
            gas: GasMeter::new(gas_limit),
            host,
            call_stack: Vec::with_capacity(64),
            max_stack_depth: 1024,
        }
    }

    /// Erstelle VM mit unbegrenztem Gas (nur für Tests!)
    pub fn new_unlimited(program: Vec<OpCode>, host: &'a dyn HostInterface) -> Self {
        Self {
            stack: Vec::with_capacity(256),
            ip: 0,
            program,
            gas: GasMeter::unlimited(),
            host,
            call_stack: Vec::with_capacity(64),
            max_stack_depth: 1024,
        }
    }

    /// Führe das Programm aus
    ///
    /// Optimierte Main-Loop mit inline Handler-Funktionen für bessere
    /// Branch Prediction und Instruction Cache Locality.
    pub fn run(&mut self) -> Result<ExecutionResult> {
        while self.ip < self.program.len() {
            let op = self.program[self.ip].clone();
            self.ip += 1;

            // 1. Gas abziehen
            self.gas.consume(op.gas_cost())?;

            // 2. Stack-Tiefe prüfen
            if self.stack.len() > self.max_stack_depth {
                return Err(ApiError::Internal(anyhow!("Stack overflow")));
            }

            // 3. Operation ausführen (mit inline Dispatch)
            match self.execute_instruction(op)? {
                ControlFlow::Continue => {}
                ControlFlow::Return(result) => {
                    return Ok(ExecutionResult {
                        value: result,
                        gas_used: self.gas.consumed(),
                        logs: Vec::new(),
                    });
                }
                ControlFlow::Error(msg) => {
                    return Err(ApiError::Internal(anyhow!("{}", msg)));
                }
            }
        }

        // Programm zu Ende ohne explizites Return/Halt
        Ok(ExecutionResult {
            value: self.stack.pop().unwrap_or(Value::Null),
            gas_used: self.gas.consumed(),
            logs: Vec::new(),
        })
    }

    /// Dispatch einer einzelnen Instruktion.
    /// Separate Funktion für bessere Compiler-Optimierung.
    #[inline(always)]
    fn execute_instruction(&mut self, op: OpCode) -> Result<ControlFlow> {
        match op {
            // Stack Manipulation - hot path, inline
            OpCode::PushConst(v) => {
                self.stack.push(v);
                Ok(ControlFlow::Continue)
            }
            OpCode::Pop => {
                self.pop()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Dup => {
                let v = self.peek()?.clone();
                self.stack.push(v);
                Ok(ControlFlow::Continue)
            }
            OpCode::Swap => {
                self.exec_swap()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Pick(n) => {
                self.exec_pick(n)?;
                Ok(ControlFlow::Continue)
            }

            // Arithmetik - häufig, inline
            OpCode::Add => {
                self.binary_op(|a, b| a + b)?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Sub => {
                self.binary_op(|a, b| a - b)?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Mul => {
                self.binary_op(|a, b| a * b)?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Div => {
                self.exec_div()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Mod => {
                self.exec_mod()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Neg => {
                self.exec_neg()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Min => {
                self.binary_op(|a, b| a.min(b))?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Max => {
                self.binary_op(|a, b| a.max(b))?;
                Ok(ControlFlow::Continue)
            }

            // Vergleiche
            OpCode::Eq => {
                self.exec_eq()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Neq => {
                self.exec_neq()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Gt => {
                self.compare_op(|a, b| a > b)?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Gte => {
                self.compare_op(|a, b| a >= b)?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Lt => {
                self.compare_op(|a, b| a < b)?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Lte => {
                self.compare_op(|a, b| a <= b)?;
                Ok(ControlFlow::Continue)
            }

            // Logik
            OpCode::And => {
                self.exec_and()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Or => {
                self.exec_or()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Not => {
                self.exec_not()?;
                Ok(ControlFlow::Continue)
            }

            // Control Flow
            OpCode::Jump(addr) => {
                self.ip = addr;
                Ok(ControlFlow::Continue)
            }
            OpCode::JumpIfFalse(addr) => {
                self.exec_jump_if_false(addr)?;
                Ok(ControlFlow::Continue)
            }
            OpCode::JumpIfTrue(addr) => {
                self.exec_jump_if_true(addr)?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Call(addr, _argc) => {
                self.exec_call(addr)?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Return => self.exec_return(),

            // TrustVector Operationen
            OpCode::TrustDim(dim) => {
                self.exec_trust_dim(dim as u8)?;
                Ok(ControlFlow::Continue)
            }
            OpCode::TrustNorm => {
                self.exec_trust_norm()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::TrustCombine => {
                self.exec_trust_combine()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::TrustCreate => {
                self.exec_trust_create()?;
                Ok(ControlFlow::Continue)
            }

            // Host Calls
            OpCode::LoadTrust => {
                self.exec_load_trust()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::HasCredential => {
                self.exec_has_credential()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::ResolveDID => {
                self.exec_resolve_did()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::GetBalance => {
                self.exec_get_balance()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::GetTimestamp => {
                self.exec_get_timestamp()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Log => {
                self.exec_log()?;
                Ok(ControlFlow::Continue)
            }

            // Assertions
            OpCode::Assert => self.exec_assert(),
            OpCode::Require => self.exec_require(),

            // Erweiterte Built-ins
            OpCode::Surprisal => {
                self.exec_surprisal()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::TrustAboveThreshold => {
                self.exec_trust_above_threshold()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::TrustWeightedAvg => {
                self.exec_trust_weighted_avg()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::TrustDistance => {
                self.exec_trust_distance()?;
                Ok(ControlFlow::Continue)
            }

            // String-Operationen
            OpCode::StrLen => {
                self.exec_str_len()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::StrEqIgnoreCase => {
                self.exec_str_eq_ignore_case()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::StrContains => {
                self.exec_str_contains()?;
                Ok(ControlFlow::Continue)
            }

            // Math-Operationen
            OpCode::MathAbs => {
                self.exec_math_abs()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::MathSqrt => {
                self.exec_math_sqrt()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::MathFloor => {
                self.exec_math_floor()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::MathCeil => {
                self.exec_math_ceil()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::MathRound => {
                self.exec_math_round()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Clamp => {
                self.exec_clamp()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::Lerp => {
                self.exec_lerp()?;
                Ok(ControlFlow::Continue)
            }

            // Zeit
            OpCode::TimeSince => {
                self.exec_time_since()?;
                Ok(ControlFlow::Continue)
            }

            // Array-Operationen
            OpCode::Contains => {
                self.exec_contains()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::ArrayLen => {
                self.exec_array_len()?;
                Ok(ControlFlow::Continue)
            }
            OpCode::ArrayGet => {
                self.exec_array_get()?;
                Ok(ControlFlow::Continue)
            }

            // Ende
            OpCode::Halt => self.exec_halt(),
            OpCode::Abort => Ok(ControlFlow::Error("Program aborted".to_string())),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Inline Handler Methods - Stack Operations
    // ═══════════════════════════════════════════════════════════════

    #[inline(always)]
    fn exec_swap(&mut self) -> Result<()> {
        let a = self.pop()?;
        let b = self.pop()?;
        self.stack.push(a);
        self.stack.push(b);
        Ok(())
    }

    #[inline(always)]
    fn exec_pick(&mut self, n: u8) -> Result<()> {
        let idx = self
            .stack
            .len()
            .checked_sub(n as usize + 1)
            .ok_or_else(|| ApiError::Internal(anyhow!("Stack underflow")))?;
        let v = self.stack[idx].clone();
        self.stack.push(v);
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════
    // Inline Handler Methods - Arithmetic
    // ═══════════════════════════════════════════════════════════════

    #[inline(always)]
    fn exec_div(&mut self) -> Result<()> {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        if b == 0.0 {
            return Err(ApiError::Internal(anyhow!("Division by zero")));
        }
        self.stack.push(Value::Number(a / b));
        Ok(())
    }

    #[inline(always)]
    fn exec_mod(&mut self) -> Result<()> {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        if b == 0.0 {
            return Err(ApiError::Internal(anyhow!("Modulo by zero")));
        }
        self.stack.push(Value::Number(a % b));
        Ok(())
    }

    #[inline(always)]
    fn exec_neg(&mut self) -> Result<()> {
        let a = self.pop_number()?;
        self.stack.push(Value::Number(-a));
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════
    // Inline Handler Methods - Comparison
    // ═══════════════════════════════════════════════════════════════

    #[inline(always)]
    fn exec_eq(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.stack.push(Value::Bool(a == b));
        Ok(())
    }

    #[inline(always)]
    fn exec_neq(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.stack.push(Value::Bool(a != b));
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════
    // Inline Handler Methods - Logic
    // ═══════════════════════════════════════════════════════════════

    #[inline(always)]
    fn exec_and(&mut self) -> Result<()> {
        let b = self.pop_bool()?;
        let a = self.pop_bool()?;
        self.stack.push(Value::Bool(a && b));
        Ok(())
    }

    #[inline(always)]
    fn exec_or(&mut self) -> Result<()> {
        let b = self.pop_bool()?;
        let a = self.pop_bool()?;
        self.stack.push(Value::Bool(a || b));
        Ok(())
    }

    #[inline(always)]
    fn exec_not(&mut self) -> Result<()> {
        let a = self.pop_bool()?;
        self.stack.push(Value::Bool(!a));
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════
    // Inline Handler Methods - Control Flow
    // ═══════════════════════════════════════════════════════════════

    #[inline(always)]
    fn exec_jump_if_false(&mut self, addr: usize) -> Result<()> {
        let cond = self.pop_bool()?;
        if !cond {
            self.ip = addr;
        }
        Ok(())
    }

    #[inline(always)]
    fn exec_jump_if_true(&mut self, addr: usize) -> Result<()> {
        let cond = self.pop_bool()?;
        if cond {
            self.ip = addr;
        }
        Ok(())
    }

    #[inline(always)]
    fn exec_call(&mut self, addr: usize) -> Result<()> {
        self.call_stack.push(self.ip);
        self.ip = addr;
        Ok(())
    }

    #[inline(always)]
    fn exec_return(&mut self) -> Result<ControlFlow> {
        if let Some(ret_addr) = self.call_stack.pop() {
            self.ip = ret_addr;
            Ok(ControlFlow::Continue)
        } else {
            // Return aus main: Programm beenden
            let result = self.stack.pop().unwrap_or(Value::Null);
            Ok(ControlFlow::Return(result))
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Inline Handler Methods - TrustVector
    // ═══════════════════════════════════════════════════════════════

    #[inline(always)]
    fn exec_trust_dim(&mut self, dim: u8) -> Result<()> {
        let tv = self.pop_trust_vector()?;
        let value = tv[dim as usize];
        self.stack.push(Value::Number(value));
        Ok(())
    }

    #[inline(always)]
    fn exec_trust_norm(&mut self) -> Result<()> {
        let tv = self.pop_trust_vector()?;
        // Gewichtete Norm (vereinfacht: Durchschnitt)
        let sum: f64 = tv.iter().sum();
        let norm = sum / 6.0;
        self.stack.push(Value::Number(norm));
        Ok(())
    }

    #[inline(always)]
    fn exec_trust_combine(&mut self) -> Result<()> {
        let tv2 = self.pop_trust_vector()?;
        let tv1 = self.pop_trust_vector()?;
        // Κ5: t₁ ⊕ t₂ = 1 - (1-t₁)(1-t₂)
        let mut result = [0.0; 6];
        for i in 0..6 {
            result[i] = 1.0 - (1.0 - tv1[i]) * (1.0 - tv2[i]);
        }
        self.stack.push(Value::TrustVector(result));
        Ok(())
    }

    #[inline(always)]
    fn exec_trust_create(&mut self) -> Result<()> {
        let omega = self.pop_number()?;
        let v = self.pop_number()?;
        let p = self.pop_number()?;
        let c = self.pop_number()?;
        let i = self.pop_number()?;
        let r = self.pop_number()?;
        self.stack.push(Value::TrustVector([r, i, c, p, v, omega]));
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════
    // Inline Handler Methods - Host Calls
    // ═══════════════════════════════════════════════════════════════

    #[inline(always)]
    fn exec_load_trust(&mut self) -> Result<()> {
        let did = self.pop_did()?;
        let tv = self.host.get_trust_vector(&did)?;
        self.stack.push(Value::TrustVector(tv));
        Ok(())
    }

    #[inline(always)]
    fn exec_has_credential(&mut self) -> Result<()> {
        let schema = self.pop_string()?;
        let did = self.pop_did()?;
        let has = self.host.has_credential(&did, &schema)?;
        self.stack.push(Value::Bool(has));
        Ok(())
    }

    #[inline(always)]
    fn exec_resolve_did(&mut self) -> Result<()> {
        let did = self.pop_did()?;
        let exists = self.host.resolve_did(&did)?;
        self.stack.push(Value::Bool(exists));
        Ok(())
    }

    #[inline(always)]
    fn exec_get_balance(&mut self) -> Result<()> {
        let did = self.pop_did()?;
        let balance = self.host.get_balance(&did)?;
        self.stack.push(Value::Number(balance as f64));
        Ok(())
    }

    #[inline(always)]
    fn exec_get_timestamp(&mut self) -> Result<()> {
        let ts = self.host.get_timestamp();
        self.stack.push(Value::Number(ts as f64));
        Ok(())
    }

    #[inline(always)]
    fn exec_log(&mut self) -> Result<()> {
        let msg = self.pop_string()?;
        self.host.log(&msg);
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════
    // Inline Handler Methods - Assertions & Termination
    // ═══════════════════════════════════════════════════════════════

    #[inline(always)]
    fn exec_assert(&mut self) -> Result<ControlFlow> {
        let cond = self.pop_bool()?;
        if !cond {
            Ok(ControlFlow::Error("Assertion failed".to_string()))
        } else {
            Ok(ControlFlow::Continue)
        }
    }

    #[inline(always)]
    fn exec_require(&mut self) -> Result<ControlFlow> {
        let msg = self.pop_string()?;
        let cond = self.pop_bool()?;
        if !cond {
            Ok(ControlFlow::Error(format!("Require failed: {}", msg)))
        } else {
            Ok(ControlFlow::Continue)
        }
    }

    #[inline(always)]
    fn exec_halt(&mut self) -> Result<ControlFlow> {
        let result = self.stack.pop().unwrap_or(Value::Null);
        Ok(ControlFlow::Return(result))
    }

    // ═══════════════════════════════════════════════════════════════
    // Extended Built-in Functions
    // ═══════════════════════════════════════════════════════════════

    /// Surprisal: S(p) = -log₂(p)
    /// Misst "Überraschung" einer Wahrscheinlichkeit
    #[inline(always)]
    fn exec_surprisal(&mut self) -> Result<()> {
        let p = self.pop_number()?;
        if p <= 0.0 || p > 1.0 {
            return Err(ApiError::Internal(anyhow!(
                "Surprisal requires probability in (0, 1], got {}",
                p
            )));
        }
        let surprisal = -p.log2();
        self.stack.push(Value::Number(surprisal));
        Ok(())
    }

    /// Prüfe ob alle Trust-Dimensionen über threshold liegen
    #[inline(always)]
    fn exec_trust_above_threshold(&mut self) -> Result<()> {
        let threshold = self.pop_number()?;
        let tv = self.pop_trust_vector()?;
        let above = tv.iter().all(|&dim| dim >= threshold);
        self.stack.push(Value::Bool(above));
        Ok(())
    }

    /// Gewichteter Durchschnitt des Trust-Vektors
    #[inline(always)]
    fn exec_trust_weighted_avg(&mut self) -> Result<()> {
        let weights = self.pop_array()?;
        let tv = self.pop_trust_vector()?;

        if weights.len() != 6 {
            return Err(ApiError::Internal(anyhow!(
                "TrustWeightedAvg requires 6 weights, got {}",
                weights.len()
            )));
        }

        let mut sum = 0.0;
        let mut weight_sum = 0.0;
        for (i, w) in weights.iter().enumerate() {
            let weight = w
                .as_number()
                .ok_or_else(|| ApiError::Internal(anyhow!("Weight must be a number")))?;
            sum += tv[i] * weight;
            weight_sum += weight;
        }

        let avg = if weight_sum > 0.0 {
            sum / weight_sum
        } else {
            0.0
        };
        self.stack.push(Value::Number(avg));
        Ok(())
    }

    /// Euklidische Distanz zwischen zwei Trust-Vektoren
    #[inline(always)]
    fn exec_trust_distance(&mut self) -> Result<()> {
        let tv2 = self.pop_trust_vector()?;
        let tv1 = self.pop_trust_vector()?;

        let mut sum_sq = 0.0;
        for i in 0..6 {
            let diff = tv1[i] - tv2[i];
            sum_sq += diff * diff;
        }
        let distance = sum_sq.sqrt();
        self.stack.push(Value::Number(distance));
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════
    // String Functions
    // ═══════════════════════════════════════════════════════════════

    #[inline(always)]
    fn exec_str_len(&mut self) -> Result<()> {
        let s = self.pop_string()?;
        self.stack.push(Value::Number(s.len() as f64));
        Ok(())
    }

    #[inline(always)]
    fn exec_str_eq_ignore_case(&mut self) -> Result<()> {
        let b = self.pop_string()?;
        let a = self.pop_string()?;
        self.stack
            .push(Value::Bool(a.to_lowercase() == b.to_lowercase()));
        Ok(())
    }

    #[inline(always)]
    fn exec_str_contains(&mut self) -> Result<()> {
        let needle = self.pop_string()?;
        let haystack = self.pop_string()?;
        self.stack.push(Value::Bool(haystack.contains(&needle)));
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════
    // Math Functions
    // ═══════════════════════════════════════════════════════════════

    #[inline(always)]
    fn exec_math_abs(&mut self) -> Result<()> {
        let n = self.pop_number()?;
        self.stack.push(Value::Number(n.abs()));
        Ok(())
    }

    #[inline(always)]
    fn exec_math_sqrt(&mut self) -> Result<()> {
        let n = self.pop_number()?;
        if n < 0.0 {
            return Err(ApiError::Internal(anyhow!(
                "sqrt requires non-negative number, got {}",
                n
            )));
        }
        self.stack.push(Value::Number(n.sqrt()));
        Ok(())
    }

    #[inline(always)]
    fn exec_math_floor(&mut self) -> Result<()> {
        let n = self.pop_number()?;
        self.stack.push(Value::Number(n.floor()));
        Ok(())
    }

    #[inline(always)]
    fn exec_math_ceil(&mut self) -> Result<()> {
        let n = self.pop_number()?;
        self.stack.push(Value::Number(n.ceil()));
        Ok(())
    }

    #[inline(always)]
    fn exec_math_round(&mut self) -> Result<()> {
        let n = self.pop_number()?;
        self.stack.push(Value::Number(n.round()));
        Ok(())
    }

    #[inline(always)]
    fn exec_clamp(&mut self) -> Result<()> {
        let max = self.pop_number()?;
        let min = self.pop_number()?;
        let value = self.pop_number()?;
        self.stack.push(Value::Number(value.clamp(min, max)));
        Ok(())
    }

    #[inline(always)]
    fn exec_lerp(&mut self) -> Result<()> {
        let t = self.pop_number()?;
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        // lerp(a, b, t) = a + t * (b - a)
        let result = a + t * (b - a);
        self.stack.push(Value::Number(result));
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════
    // Time Functions
    // ═══════════════════════════════════════════════════════════════

    #[inline(always)]
    fn exec_time_since(&mut self) -> Result<()> {
        let timestamp = self.pop_number()? as u64;
        let now = self.host.get_timestamp();
        let diff = now.saturating_sub(timestamp);
        self.stack.push(Value::Number(diff as f64));
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════
    // Array Functions
    // ═══════════════════════════════════════════════════════════════

    #[inline(always)]
    fn exec_contains(&mut self) -> Result<()> {
        let arr = self.pop_array()?;
        let value = self.pop()?;
        let contains = arr.iter().any(|v| v == &value);
        self.stack.push(Value::Bool(contains));
        Ok(())
    }

    #[inline(always)]
    fn exec_array_len(&mut self) -> Result<()> {
        let arr = self.pop_array()?;
        self.stack.push(Value::Number(arr.len() as f64));
        Ok(())
    }

    #[inline(always)]
    fn exec_array_get(&mut self) -> Result<()> {
        let index = self.pop_number()? as usize;
        let arr = self.pop_array()?;
        if index >= arr.len() {
            return Err(ApiError::Internal(anyhow!(
                "Array index {} out of bounds (len: {})",
                index,
                arr.len()
            )));
        }
        self.stack.push(arr[index].clone());
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════
    // Core Helper Methods (inline for performance)
    // ═══════════════════════════════════════════════════════════════

    #[inline(always)]
    fn pop(&mut self) -> Result<Value> {
        self.stack
            .pop()
            .ok_or_else(|| ApiError::Internal(anyhow!("Stack underflow")))
    }

    #[inline(always)]
    fn peek(&self) -> Result<&Value> {
        self.stack
            .last()
            .ok_or_else(|| ApiError::Internal(anyhow!("Stack underflow")))
    }

    #[inline(always)]
    fn pop_number(&mut self) -> Result<f64> {
        let v = self.pop()?;
        v.as_number()
            .ok_or_else(|| ApiError::Internal(anyhow!("Expected number, got {}", v.type_name())))
    }

    #[inline(always)]
    fn pop_bool(&mut self) -> Result<bool> {
        let v = self.pop()?;
        v.as_bool()
            .ok_or_else(|| ApiError::Internal(anyhow!("Expected bool, got {}", v.type_name())))
    }

    #[inline(always)]
    fn pop_string(&mut self) -> Result<String> {
        let v = self.pop()?;
        v.as_string()
            .map(|s| s.to_string())
            .ok_or_else(|| ApiError::Internal(anyhow!("Expected string, got {}", v.type_name())))
    }

    #[inline(always)]
    fn pop_did(&mut self) -> Result<String> {
        let v = self.pop()?;
        match v {
            Value::DID(d) => Ok(d),
            Value::String(s) => Ok(s),
            _ => Err(ApiError::Internal(anyhow!(
                "Expected DID, got {}",
                v.type_name()
            ))),
        }
    }

    #[inline(always)]
    fn pop_trust_vector(&mut self) -> Result<[f64; 6]> {
        let v = self.pop()?;
        v.as_trust_vector().ok_or_else(|| {
            ApiError::Internal(anyhow!("Expected TrustVector, got {}", v.type_name()))
        })
    }

    #[inline(always)]
    fn pop_array(&mut self) -> Result<Vec<Value>> {
        let v = self.pop()?;
        match v {
            Value::Array(arr) => Ok(arr),
            _ => Err(ApiError::Internal(anyhow!(
                "Expected Array, got {}",
                v.type_name()
            ))),
        }
    }

    #[inline(always)]
    fn binary_op<F>(&mut self, f: F) -> Result<()>
    where
        F: Fn(f64, f64) -> f64,
    {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        self.stack.push(Value::Number(f(a, b)));
        Ok(())
    }

    #[inline(always)]
    fn compare_op<F>(&mut self, f: F) -> Result<()>
    where
        F: Fn(f64, f64) -> bool,
    {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        self.stack.push(Value::Bool(f(a, b)));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_program(program: Vec<OpCode>) -> Result<Value> {
        let host = StubHost::new();
        let mut vm = ECLVM::new_unlimited(program, &host);
        vm.run().map(|r| r.value)
    }

    #[test]
    fn test_push_and_return() {
        let result =
            run_program(vec![OpCode::PushConst(Value::Number(42.0)), OpCode::Return]).unwrap();

        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_arithmetic_add() {
        let result = run_program(vec![
            OpCode::PushConst(Value::Number(5.0)),
            OpCode::PushConst(Value::Number(3.0)),
            OpCode::Add,
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::Number(8.0));
    }

    #[test]
    fn test_arithmetic_complex() {
        // (10 - 3) * 2 = 14
        let result = run_program(vec![
            OpCode::PushConst(Value::Number(10.0)),
            OpCode::PushConst(Value::Number(3.0)),
            OpCode::Sub,
            OpCode::PushConst(Value::Number(2.0)),
            OpCode::Mul,
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::Number(14.0));
    }

    #[test]
    fn test_comparison() {
        let result = run_program(vec![
            OpCode::PushConst(Value::Number(5.0)),
            OpCode::PushConst(Value::Number(3.0)),
            OpCode::Gt,
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_logic_and() {
        let result = run_program(vec![
            OpCode::PushConst(Value::Bool(true)),
            OpCode::PushConst(Value::Bool(false)),
            OpCode::And,
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_conditional_jump() {
        // if (5 > 3) return 100 else return 200
        let result = run_program(vec![
            OpCode::PushConst(Value::Number(5.0)),   // 0
            OpCode::PushConst(Value::Number(3.0)),   // 1
            OpCode::Gt,                              // 2
            OpCode::JumpIfFalse(6),                  // 3 -> springe zu 6 wenn false
            OpCode::PushConst(Value::Number(100.0)), // 4 (true branch)
            OpCode::Return,                          // 5
            OpCode::PushConst(Value::Number(200.0)), // 6 (false branch)
            OpCode::Return,                          // 7
        ])
        .unwrap();

        assert_eq!(result, Value::Number(100.0));
    }

    #[test]
    fn test_trust_vector_dim() {
        let result = run_program(vec![
            OpCode::PushConst(Value::TrustVector([0.8, 0.7, 0.6, 0.5, 0.4, 0.3])),
            OpCode::TrustDim(TrustDimIndex::R),
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::Number(0.8));
    }

    #[test]
    fn test_trust_combine() {
        // Κ5: t₁ ⊕ t₂ = 1 - (1-t₁)(1-t₂)
        // 0.5 ⊕ 0.5 = 1 - (0.5)(0.5) = 0.75
        let result = run_program(vec![
            OpCode::PushConst(Value::TrustVector([0.5, 0.5, 0.5, 0.5, 0.5, 0.5])),
            OpCode::PushConst(Value::TrustVector([0.5, 0.5, 0.5, 0.5, 0.5, 0.5])),
            OpCode::TrustCombine,
            OpCode::TrustDim(TrustDimIndex::R),
            OpCode::Return,
        ])
        .unwrap();

        if let Value::Number(n) = result {
            assert!((n - 0.75).abs() < 0.001);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_load_trust_from_host() {
        let host =
            StubHost::new().with_trust("did:erynoa:self:alice", [0.9, 0.8, 0.7, 0.6, 0.5, 0.4]);

        let program = vec![
            OpCode::PushConst(Value::DID("did:erynoa:self:alice".into())),
            OpCode::LoadTrust,
            OpCode::TrustDim(TrustDimIndex::R),
            OpCode::Return,
        ];

        let mut vm = ECLVM::new_unlimited(program, &host);
        let result = vm.run().unwrap();

        assert_eq!(result.value, Value::Number(0.9));
    }

    #[test]
    fn test_gas_metering() {
        let host = StubHost::new();
        let program = vec![
            OpCode::PushConst(Value::Number(1.0)),
            OpCode::PushConst(Value::Number(2.0)),
            OpCode::Add,
            OpCode::Return,
        ];

        let mut vm = ECLVM::new(program, 10, &host);
        let result = vm.run().unwrap();

        assert_eq!(result.value, Value::Number(3.0));
        assert!(result.gas_used > 0);
        assert!(result.gas_used <= 10);
    }

    #[test]
    fn test_out_of_gas() {
        let host = StubHost::new();
        let program = vec![
            OpCode::PushConst(Value::Number(1.0)),
            OpCode::PushConst(Value::Number(2.0)),
            OpCode::Add,
            OpCode::Return,
        ];

        let mut vm = ECLVM::new(program, 1, &host); // Nur 1 Gas
        let result = vm.run();

        assert!(result.is_err());
    }

    #[test]
    fn test_assert_success() {
        let result = run_program(vec![
            OpCode::PushConst(Value::Bool(true)),
            OpCode::Assert,
            OpCode::PushConst(Value::Number(42.0)),
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_assert_failure() {
        let result = run_program(vec![
            OpCode::PushConst(Value::Bool(false)),
            OpCode::Assert,
            OpCode::PushConst(Value::Number(42.0)),
            OpCode::Return,
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn test_division_by_zero() {
        let result = run_program(vec![
            OpCode::PushConst(Value::Number(10.0)),
            OpCode::PushConst(Value::Number(0.0)),
            OpCode::Div,
            OpCode::Return,
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn test_has_credential() {
        let host = StubHost::new().with_credential("did:erynoa:self:alice", "email-verified");

        let program = vec![
            OpCode::PushConst(Value::DID("did:erynoa:self:alice".into())),
            OpCode::PushConst(Value::String("email-verified".into())),
            OpCode::HasCredential,
            OpCode::Return,
        ];

        let mut vm = ECLVM::new_unlimited(program, &host);
        let result = vm.run().unwrap();

        assert_eq!(result.value, Value::Bool(true));
    }

    // ═══════════════════════════════════════════════════════════════
    // Tests für erweiterte Built-in Funktionen
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_surprisal() {
        // S(0.5) = -log₂(0.5) = 1.0
        let result = run_program(vec![
            OpCode::PushConst(Value::Number(0.5)),
            OpCode::Surprisal,
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_surprisal_low_probability() {
        // S(0.125) = -log₂(0.125) = 3.0
        let result = run_program(vec![
            OpCode::PushConst(Value::Number(0.125)),
            OpCode::Surprisal,
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_trust_above_threshold() {
        let result = run_program(vec![
            OpCode::PushConst(Value::TrustVector([0.8, 0.7, 0.9, 0.6, 0.8, 0.7])),
            OpCode::PushConst(Value::Number(0.6)),
            OpCode::TrustAboveThreshold,
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_trust_above_threshold_fails() {
        let result = run_program(vec![
            OpCode::PushConst(Value::TrustVector([0.8, 0.7, 0.9, 0.5, 0.8, 0.7])),
            OpCode::PushConst(Value::Number(0.6)),
            OpCode::TrustAboveThreshold,
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_trust_distance() {
        // Distanz zwischen [1,1,1,1,1,1] und [0,0,0,0,0,0] = sqrt(6) ≈ 2.449
        let result = run_program(vec![
            OpCode::PushConst(Value::TrustVector([1.0, 1.0, 1.0, 1.0, 1.0, 1.0])),
            OpCode::PushConst(Value::TrustVector([0.0, 0.0, 0.0, 0.0, 0.0, 0.0])),
            OpCode::TrustDistance,
            OpCode::Return,
        ])
        .unwrap();

        if let Value::Number(n) = result {
            assert!((n - 6.0_f64.sqrt()).abs() < 0.001);
        } else {
            panic!("Expected Number");
        }
    }

    #[test]
    fn test_str_len() {
        let result = run_program(vec![
            OpCode::PushConst(Value::String("Hello".into())),
            OpCode::StrLen,
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_str_contains() {
        let result = run_program(vec![
            OpCode::PushConst(Value::String("Hello World".into())),
            OpCode::PushConst(Value::String("World".into())),
            OpCode::StrContains,
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_math_sqrt() {
        let result = run_program(vec![
            OpCode::PushConst(Value::Number(16.0)),
            OpCode::MathSqrt,
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::Number(4.0));
    }

    #[test]
    fn test_clamp() {
        let result = run_program(vec![
            OpCode::PushConst(Value::Number(150.0)), // value
            OpCode::PushConst(Value::Number(0.0)),   // min
            OpCode::PushConst(Value::Number(100.0)), // max
            OpCode::Clamp,
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::Number(100.0));
    }

    #[test]
    fn test_lerp() {
        // lerp(0, 10, 0.5) = 5
        let result = run_program(vec![
            OpCode::PushConst(Value::Number(0.0)),  // a
            OpCode::PushConst(Value::Number(10.0)), // b
            OpCode::PushConst(Value::Number(0.5)),  // t
            OpCode::Lerp,
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_array_len() {
        let result = run_program(vec![
            OpCode::PushConst(Value::Array(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
            ])),
            OpCode::ArrayLen,
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_array_get() {
        let result = run_program(vec![
            OpCode::PushConst(Value::Array(vec![
                Value::String("a".into()),
                Value::String("b".into()),
                Value::String("c".into()),
            ])),
            OpCode::PushConst(Value::Number(1.0)), // index
            OpCode::ArrayGet,
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::String("b".into()));
    }

    #[test]
    fn test_contains() {
        let result = run_program(vec![
            OpCode::PushConst(Value::String("admin".into())), // needle
            OpCode::PushConst(Value::Array(vec![
                Value::String("user".into()),
                Value::String("admin".into()),
                Value::String("guest".into()),
            ])),
            OpCode::Contains,
            OpCode::Return,
        ])
        .unwrap();

        assert_eq!(result, Value::Bool(true));
    }
}
