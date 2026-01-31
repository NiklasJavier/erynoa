//! # ECLVM Abstract Syntax Tree
//!
//! Der AST repräsentiert ein geparstes ECL-Programm.
//! Wird vom Parser erzeugt und vom Compiler in Bytecode übersetzt.
//!
//! ## Beispiel ECL Syntax (geplant)
//!
//! ```text
//! policy "transfer_guard" {
//!     require sender.trust.R >= 0.5
//!     require sender.credential("kyc-verified")
//!
//!     let max_amount = sender.trust.R * 1000
//!     require amount <= max_amount
//!
//!     emit "transfer_approved"
//! }
//! ```

use serde::{Deserialize, Serialize};

/// Ein komplettes ECL-Programm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    /// Liste von Policies
    pub policies: Vec<Policy>,
    /// Globale Konstanten
    pub constants: Vec<ConstDecl>,
}

/// Eine Policy-Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Name der Policy
    pub name: String,
    /// Body der Policy
    pub body: Vec<Statement>,
    /// Location im Source
    pub span: Span,
}

/// Ein Statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    /// require <expr>
    Require(Expr, Option<String>),
    /// let <name> = <expr>
    Let(String, Expr),
    /// emit <event>
    Emit(String),
    /// if <cond> { <body> } else { <body> }
    If {
        condition: Expr,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    /// return <expr>
    Return(Expr),
}

/// Ein Ausdruck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    /// Literal-Wert
    Literal(Literal),
    /// Variable
    Identifier(String),
    /// Binary Operation: a + b, a > b, etc.
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    /// Unary Operation: !a, -a
    Unary { op: UnaryOp, operand: Box<Expr> },
    /// Member Access: sender.trust
    Member { object: Box<Expr>, field: String },
    /// Index Access: trust[0]
    Index { object: Box<Expr>, index: Box<Expr> },
    /// Function Call: credential("kyc")
    Call { function: String, args: Vec<Expr> },
    /// Trust Dimension: trust.R
    TrustDim {
        vector: Box<Expr>,
        dimension: TrustDim,
    },
}

/// Literal-Werte
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    DID(String),
    TrustVector([f64; 6]),
}

/// Binary Operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetik
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Vergleiche
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,

    // Logik
    And,
    Or,
}

/// Unary Operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// Trust-Dimensionen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustDim {
    R,
    I,
    C,
    P,
    V,
    Omega,
}

/// Konstanten-Deklaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstDecl {
    pub name: String,
    pub value: Literal,
}

/// Source Location
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Program {
    /// Leeres Programm
    pub fn empty() -> Self {
        Self {
            policies: Vec::new(),
            constants: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_construction() {
        // require sender.trust.R >= 0.5
        let stmt = Statement::Require(
            Expr::Binary {
                left: Box::new(Expr::TrustDim {
                    vector: Box::new(Expr::Member {
                        object: Box::new(Expr::Identifier("sender".into())),
                        field: "trust".into(),
                    }),
                    dimension: TrustDim::R,
                }),
                op: BinaryOp::Gte,
                right: Box::new(Expr::Literal(Literal::Number(0.5))),
            },
            None,
        );

        let policy = Policy {
            name: "transfer_guard".into(),
            body: vec![stmt],
            span: Span::default(),
        };

        let program = Program {
            policies: vec![policy],
            constants: Vec::new(),
        };

        assert_eq!(program.policies.len(), 1);
        assert_eq!(program.policies[0].name, "transfer_guard");
    }
}
