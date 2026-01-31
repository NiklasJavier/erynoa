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

use anyhow::anyhow;
use super::gas::GasMeter;
use super::host::HostInterface;
#[cfg(test)]
use super::host::StubHost;
use crate::eclvm::bytecode::{OpCode, Value};
#[cfg(test)]
use crate::eclvm::bytecode::TrustDimIndex;
use crate::error::{ApiError, Result};

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

            // 3. Operation ausführen
            match op {
                // Stack Manipulation
                OpCode::PushConst(v) => self.stack.push(v),
                OpCode::Pop => { self.pop()?; }
                OpCode::Dup => {
                    let v = self.peek()?.clone();
                    self.stack.push(v);
                }
                OpCode::Swap => {
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.stack.push(a);
                    self.stack.push(b);
                }
                OpCode::Pick(n) => {
                    let idx = self.stack.len().checked_sub(n as usize + 1)
                        .ok_or_else(|| ApiError::Internal(anyhow!("Stack underflow")))?;
                    let v = self.stack[idx].clone();
                    self.stack.push(v);
                }

                // Arithmetik
                OpCode::Add => self.binary_op(|a, b| a + b)?,
                OpCode::Sub => self.binary_op(|a, b| a - b)?,
                OpCode::Mul => self.binary_op(|a, b| a * b)?,
                OpCode::Div => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    if b == 0.0 {
                        return Err(ApiError::Internal(anyhow!("Division by zero")));
                    }
                    self.stack.push(Value::Number(a / b));
                }
                OpCode::Mod => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    if b == 0.0 {
                        return Err(ApiError::Internal(anyhow!("Modulo by zero")));
                    }
                    self.stack.push(Value::Number(a % b));
                }
                OpCode::Neg => {
                    let a = self.pop_number()?;
                    self.stack.push(Value::Number(-a));
                }
                OpCode::Min => self.binary_op(|a, b| a.min(b))?,
                OpCode::Max => self.binary_op(|a, b| a.max(b))?,

                // Vergleiche
                OpCode::Eq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Bool(a == b));
                }
                OpCode::Neq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Bool(a != b));
                }
                OpCode::Gt => self.compare_op(|a, b| a > b)?,
                OpCode::Gte => self.compare_op(|a, b| a >= b)?,
                OpCode::Lt => self.compare_op(|a, b| a < b)?,
                OpCode::Lte => self.compare_op(|a, b| a <= b)?,

                // Logik
                OpCode::And => {
                    let b = self.pop_bool()?;
                    let a = self.pop_bool()?;
                    self.stack.push(Value::Bool(a && b));
                }
                OpCode::Or => {
                    let b = self.pop_bool()?;
                    let a = self.pop_bool()?;
                    self.stack.push(Value::Bool(a || b));
                }
                OpCode::Not => {
                    let a = self.pop_bool()?;
                    self.stack.push(Value::Bool(!a));
                }

                // Control Flow
                OpCode::Jump(addr) => {
                    self.ip = addr;
                }
                OpCode::JumpIfFalse(addr) => {
                    let cond = self.pop_bool()?;
                    if !cond {
                        self.ip = addr;
                    }
                }
                OpCode::JumpIfTrue(addr) => {
                    let cond = self.pop_bool()?;
                    if cond {
                        self.ip = addr;
                    }
                }
                OpCode::Call(addr, _argc) => {
                    self.call_stack.push(self.ip);
                    self.ip = addr;
                }
                OpCode::Return => {
                    if let Some(ret_addr) = self.call_stack.pop() {
                        self.ip = ret_addr;
                    } else {
                        // Return aus main: Programm beenden
                        let result = self.stack.pop().unwrap_or(Value::Null);
                        return Ok(ExecutionResult {
                            value: result,
                            gas_used: self.gas.consumed(),
                            logs: Vec::new(),
                        });
                    }
                }

                // TrustVector Operationen
                OpCode::TrustDim(dim) => {
                    let tv = self.pop_trust_vector()?;
                    let value = tv[dim as usize];
                    self.stack.push(Value::Number(value));
                }
                OpCode::TrustNorm => {
                    let tv = self.pop_trust_vector()?;
                    // Gewichtete Norm (vereinfacht: Durchschnitt)
                    let sum: f64 = tv.iter().sum();
                    let norm = sum / 6.0;
                    self.stack.push(Value::Number(norm));
                }
                OpCode::TrustCombine => {
                    let tv2 = self.pop_trust_vector()?;
                    let tv1 = self.pop_trust_vector()?;
                    // Κ5: t₁ ⊕ t₂ = 1 - (1-t₁)(1-t₂)
                    let mut result = [0.0; 6];
                    for i in 0..6 {
                        result[i] = 1.0 - (1.0 - tv1[i]) * (1.0 - tv2[i]);
                    }
                    self.stack.push(Value::TrustVector(result));
                }
                OpCode::TrustCreate => {
                    let omega = self.pop_number()?;
                    let v = self.pop_number()?;
                    let p = self.pop_number()?;
                    let c = self.pop_number()?;
                    let i = self.pop_number()?;
                    let r = self.pop_number()?;
                    self.stack.push(Value::TrustVector([r, i, c, p, v, omega]));
                }

                // Host Calls
                OpCode::LoadTrust => {
                    let did = self.pop_did()?;
                    let tv = self.host.get_trust_vector(&did)?;
                    self.stack.push(Value::TrustVector(tv));
                }
                OpCode::HasCredential => {
                    let schema = self.pop_string()?;
                    let did = self.pop_did()?;
                    let has = self.host.has_credential(&did, &schema)?;
                    self.stack.push(Value::Bool(has));
                }
                OpCode::ResolveDID => {
                    let did = self.pop_did()?;
                    let exists = self.host.resolve_did(&did)?;
                    self.stack.push(Value::Bool(exists));
                }
                OpCode::GetBalance => {
                    let did = self.pop_did()?;
                    let balance = self.host.get_balance(&did)?;
                    self.stack.push(Value::Number(balance as f64));
                }
                OpCode::GetTimestamp => {
                    let ts = self.host.get_timestamp();
                    self.stack.push(Value::Number(ts as f64));
                }
                OpCode::Log => {
                    let msg = self.pop_string()?;
                    self.host.log(&msg);
                }

                // Assertions
                OpCode::Assert => {
                    let cond = self.pop_bool()?;
                    if !cond {
                        return Err(ApiError::Internal(anyhow!("Assertion failed")));
                    }
                }
                OpCode::Require => {
                    let msg = self.pop_string()?;
                    let cond = self.pop_bool()?;
                    if !cond {
                        return Err(ApiError::Internal(anyhow!("Require failed: {}", msg)));
                    }
                }

                // Ende
                OpCode::Halt => {
                    let result = self.stack.pop().unwrap_or(Value::Null);
                    return Ok(ExecutionResult {
                        value: result,
                        gas_used: self.gas.consumed(),
                        logs: Vec::new(),
                    });
                }
                OpCode::Abort => {
                    return Err(ApiError::Internal(anyhow!("Program aborted")));
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

    // ═══════════════════════════════════════════════════════════════
    // Helper Methods
    // ═══════════════════════════════════════════════════════════════

    fn pop(&mut self) -> Result<Value> {
        self.stack.pop()
            .ok_or_else(|| ApiError::Internal(anyhow!("Stack underflow")))
    }

    fn peek(&self) -> Result<&Value> {
        self.stack.last()
            .ok_or_else(|| ApiError::Internal(anyhow!("Stack underflow")))
    }

    fn pop_number(&mut self) -> Result<f64> {
        let v = self.pop()?;
        v.as_number()
            .ok_or_else(|| ApiError::Internal(anyhow!("Expected number, got {}", v.type_name())))
    }

    fn pop_bool(&mut self) -> Result<bool> {
        let v = self.pop()?;
        v.as_bool()
            .ok_or_else(|| ApiError::Internal(anyhow!("Expected bool, got {}", v.type_name())))
    }

    fn pop_string(&mut self) -> Result<String> {
        let v = self.pop()?;
        v.as_string()
            .map(|s| s.to_string())
            .ok_or_else(|| ApiError::Internal(anyhow!("Expected string, got {}", v.type_name())))
    }

    fn pop_did(&mut self) -> Result<String> {
        let v = self.pop()?;
        match v {
            Value::DID(d) => Ok(d),
            Value::String(s) => Ok(s),
            _ => Err(ApiError::Internal(anyhow!("Expected DID, got {}", v.type_name()))),
        }
    }

    fn pop_trust_vector(&mut self) -> Result<[f64; 6]> {
        let v = self.pop()?;
        v.as_trust_vector()
            .ok_or_else(|| ApiError::Internal(anyhow!("Expected TrustVector, got {}", v.type_name())))
    }

    fn binary_op<F>(&mut self, f: F) -> Result<()>
    where
        F: Fn(f64, f64) -> f64,
    {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        self.stack.push(Value::Number(f(a, b)));
        Ok(())
    }

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
        let result = run_program(vec![
            OpCode::PushConst(Value::Number(42.0)),
            OpCode::Return,
        ]).unwrap();

        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_arithmetic_add() {
        let result = run_program(vec![
            OpCode::PushConst(Value::Number(5.0)),
            OpCode::PushConst(Value::Number(3.0)),
            OpCode::Add,
            OpCode::Return,
        ]).unwrap();

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
        ]).unwrap();

        assert_eq!(result, Value::Number(14.0));
    }

    #[test]
    fn test_comparison() {
        let result = run_program(vec![
            OpCode::PushConst(Value::Number(5.0)),
            OpCode::PushConst(Value::Number(3.0)),
            OpCode::Gt,
            OpCode::Return,
        ]).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_logic_and() {
        let result = run_program(vec![
            OpCode::PushConst(Value::Bool(true)),
            OpCode::PushConst(Value::Bool(false)),
            OpCode::And,
            OpCode::Return,
        ]).unwrap();

        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_conditional_jump() {
        // if (5 > 3) return 100 else return 200
        let result = run_program(vec![
            OpCode::PushConst(Value::Number(5.0)),    // 0
            OpCode::PushConst(Value::Number(3.0)),    // 1
            OpCode::Gt,                               // 2
            OpCode::JumpIfFalse(6),                   // 3 -> springe zu 6 wenn false
            OpCode::PushConst(Value::Number(100.0)), // 4 (true branch)
            OpCode::Return,                           // 5
            OpCode::PushConst(Value::Number(200.0)), // 6 (false branch)
            OpCode::Return,                           // 7
        ]).unwrap();

        assert_eq!(result, Value::Number(100.0));
    }

    #[test]
    fn test_trust_vector_dim() {
        let result = run_program(vec![
            OpCode::PushConst(Value::TrustVector([0.8, 0.7, 0.6, 0.5, 0.4, 0.3])),
            OpCode::TrustDim(TrustDimIndex::R),
            OpCode::Return,
        ]).unwrap();

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
        ]).unwrap();

        if let Value::Number(n) = result {
            assert!((n - 0.75).abs() < 0.001);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_load_trust_from_host() {
        let host = StubHost::new()
            .with_trust("did:erynoa:self:alice", [0.9, 0.8, 0.7, 0.6, 0.5, 0.4]);

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
        ]).unwrap();

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
        let host = StubHost::new()
            .with_credential("did:erynoa:self:alice", "email-verified");

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
}
