//! # ECLVM Compiler
//!
//! Kompiliert AST zu Bytecode.
//!
//! ## Status: Level 2 (geplant)
//!
//! Für Level 1 verwenden wir handgeschriebene Bytecode-Programme.
//!
//! ## Beispiel
//!
//! ```rust,ignore
//! // AST:
//! // require sender.trust.R >= 0.5
//!
//! // Wird kompiliert zu:
//! vec![
//!     OpCode::PushConst(Value::DID("sender".into())),  // sender
//!     OpCode::LoadTrust,                               // sender.trust
//!     OpCode::TrustDim(TrustDimIndex::R),              // sender.trust.R
//!     OpCode::PushConst(Value::Number(0.5)),           // 0.5
//!     OpCode::Gte,                                     // >=
//!     OpCode::Assert,                                  // require
//! ]
//! ```

use anyhow::anyhow;
use crate::eclvm::ast::{BinaryOp, Expr, Literal, Policy, Program, Statement, TrustDim, UnaryOp};
use crate::eclvm::bytecode::{OpCode, TrustDimIndex, Value};
use crate::error::{ApiError, Result};

/// Compiler für ECL zu Bytecode
pub struct Compiler {
    /// Emittierter Bytecode
    bytecode: Vec<OpCode>,
    /// Symbol-Tabelle (Variable -> Stack-Offset)
    symbols: std::collections::HashMap<String, usize>,
    /// Aktueller Stack-Offset
    stack_offset: usize,
}

impl Compiler {
    /// Neuer Compiler
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            symbols: std::collections::HashMap::new(),
            stack_offset: 0,
        }
    }

    /// Kompiliere Programm zu Bytecode
    pub fn compile(mut self, program: &Program) -> Result<Vec<OpCode>> {
        for policy in &program.policies {
            self.compile_policy(policy)?;
        }
        Ok(self.bytecode)
    }

    /// Kompiliere einzelne Policy
    pub fn compile_policy(&mut self, policy: &Policy) -> Result<()> {
        for stmt in &policy.body {
            self.compile_statement(stmt)?;
        }
        // Implizites Return true am Ende
        self.emit(OpCode::PushConst(Value::Bool(true)));
        self.emit(OpCode::Return);
        Ok(())
    }

    /// Kompiliere Statement
    fn compile_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Require(expr, msg) => {
                self.compile_expr(expr)?;
                if let Some(message) = msg {
                    self.emit(OpCode::PushConst(Value::String(message.clone())));
                    self.emit(OpCode::Require);
                } else {
                    self.emit(OpCode::Assert);
                }
            }
            Statement::Let(name, expr) => {
                self.compile_expr(expr)?;
                self.symbols.insert(name.clone(), self.stack_offset);
                self.stack_offset += 1;
            }
            Statement::Emit(event) => {
                self.emit(OpCode::PushConst(Value::String(format!("emit:{}", event))));
                self.emit(OpCode::Log);
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.compile_expr(condition)?;
                let jump_false = self.bytecode.len();
                self.emit(OpCode::JumpIfFalse(0)); // Placeholder

                for s in then_branch {
                    self.compile_statement(s)?;
                }

                if let Some(else_stmts) = else_branch {
                    let jump_end = self.bytecode.len();
                    self.emit(OpCode::Jump(0)); // Placeholder

                    // Patch jump_false
                    self.bytecode[jump_false] = OpCode::JumpIfFalse(self.bytecode.len());

                    for s in else_stmts {
                        self.compile_statement(s)?;
                    }

                    // Patch jump_end
                    self.bytecode[jump_end] = OpCode::Jump(self.bytecode.len());
                } else {
                    // Patch jump_false
                    self.bytecode[jump_false] = OpCode::JumpIfFalse(self.bytecode.len());
                }
            }
            Statement::Return(expr) => {
                self.compile_expr(expr)?;
                self.emit(OpCode::Return);
            }
        }
        Ok(())
    }

    /// Kompiliere Expression
    fn compile_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Literal(lit) => {
                let value = self.literal_to_value(lit);
                self.emit(OpCode::PushConst(value));
            }
            Expr::Identifier(name) => {
                if let Some(&offset) = self.symbols.get(name) {
                    let pick_idx = (self.stack_offset - offset - 1) as u8;
                    self.emit(OpCode::Pick(pick_idx));
                } else {
                    // Globale Variable oder Built-in
                    self.emit(OpCode::PushConst(Value::DID(name.clone())));
                }
            }
            Expr::Binary { left, op, right } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                self.emit(self.binary_op_to_opcode(*op));
            }
            Expr::Unary { op, operand } => {
                self.compile_expr(operand)?;
                match op {
                    UnaryOp::Neg => self.emit(OpCode::Neg),
                    UnaryOp::Not => self.emit(OpCode::Not),
                }
            }
            Expr::Member { object, field } => {
                self.compile_expr(object)?;
                if field == "trust" {
                    self.emit(OpCode::LoadTrust);
                } else {
                    return Err(ApiError::Internal(anyhow!("Unknown field: {}", field)));
                }
            }
            Expr::TrustDim { vector, dimension } => {
                self.compile_expr(vector)?;
                let idx = match dimension {
                    TrustDim::R => TrustDimIndex::R,
                    TrustDim::I => TrustDimIndex::I,
                    TrustDim::C => TrustDimIndex::C,
                    TrustDim::P => TrustDimIndex::P,
                    TrustDim::V => TrustDimIndex::V,
                    TrustDim::Omega => TrustDimIndex::Omega,
                };
                self.emit(OpCode::TrustDim(idx));
            }
            Expr::Call { function, args } => {
                for arg in args {
                    self.compile_expr(arg)?;
                }
                match function.as_str() {
                    "credential" => self.emit(OpCode::HasCredential),
                    "balance" => self.emit(OpCode::GetBalance),
                    "timestamp" => self.emit(OpCode::GetTimestamp),
                    _ => return Err(ApiError::Internal(anyhow!("Unknown function: {}", function))),
                }
            }
            Expr::Index { .. } => {
                return Err(ApiError::Internal(anyhow!(
                    "Index expressions not yet supported"
                )));
            }
        }
        Ok(())
    }

    fn emit(&mut self, op: OpCode) {
        self.bytecode.push(op);
    }

    fn literal_to_value(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Null => Value::Null,
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Number(n) => Value::Number(*n),
            Literal::String(s) => Value::String(s.clone()),
            Literal::DID(d) => Value::DID(d.clone()),
            Literal::TrustVector(tv) => Value::TrustVector(*tv),
        }
    }

    fn binary_op_to_opcode(&self, op: BinaryOp) -> OpCode {
        match op {
            BinaryOp::Add => OpCode::Add,
            BinaryOp::Sub => OpCode::Sub,
            BinaryOp::Mul => OpCode::Mul,
            BinaryOp::Div => OpCode::Div,
            BinaryOp::Mod => OpCode::Mod,
            BinaryOp::Eq => OpCode::Eq,
            BinaryOp::Neq => OpCode::Neq,
            BinaryOp::Lt => OpCode::Lt,
            BinaryOp::Lte => OpCode::Lte,
            BinaryOp::Gt => OpCode::Gt,
            BinaryOp::Gte => OpCode::Gte,
            BinaryOp::And => OpCode::And,
            BinaryOp::Or => OpCode::Or,
        }
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eclvm::ast::{Policy, Program, Span};
    use crate::eclvm::runtime::{host::StubHost, vm::ECLVM};

    #[test]
    fn test_compile_simple_require() {
        // require true
        let policy = Policy {
            name: "test".into(),
            body: vec![Statement::Require(Expr::Literal(Literal::Bool(true)), None)],
            span: Span::default(),
        };

        let program = Program {
            policies: vec![policy],
            constants: Vec::new(),
        };

        let bytecode = Compiler::new().compile(&program).unwrap();

        // Sollte: PushConst(true), Assert, PushConst(true), Return
        assert!(bytecode.len() >= 2);

        // Ausführen
        let host = StubHost::new();
        let mut vm = ECLVM::new_unlimited(bytecode, &host);
        let result = vm.run().unwrap();

        assert_eq!(result.value, Value::Bool(true));
    }

    #[test]
    fn test_compile_binary_expr() {
        // require 5 > 3
        let policy = Policy {
            name: "test".into(),
            body: vec![Statement::Require(
                Expr::Binary {
                    left: Box::new(Expr::Literal(Literal::Number(5.0))),
                    op: BinaryOp::Gt,
                    right: Box::new(Expr::Literal(Literal::Number(3.0))),
                },
                None,
            )],
            span: Span::default(),
        };

        let program = Program {
            policies: vec![policy],
            constants: Vec::new(),
        };

        let bytecode = Compiler::new().compile(&program).unwrap();

        let host = StubHost::new();
        let mut vm = ECLVM::new_unlimited(bytecode, &host);
        let result = vm.run().unwrap();

        assert_eq!(result.value, Value::Bool(true));
    }
}
