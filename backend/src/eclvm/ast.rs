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
//!
//! ## Visitor Pattern
//!
//! Der AST unterstützt das Visitor-Pattern für Traversierung:
//!
//! ```rust,ignore
//! use erynoa_api::eclvm::ast::{AstVisitor, Expr, walk_expr};
//!
//! struct MyVisitor;
//! impl AstVisitor for MyVisitor {
//!     fn visit_expr(&mut self, expr: &Expr) {
//!         // Custom logic
//!         walk_expr(self, expr);
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// Source Location & Diagnostics
// ═══════════════════════════════════════════════════════════════════════════

/// Source Location mit Zeilen/Spalten-Information
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Span {
    /// Byte-Offset Start
    pub start: usize,
    /// Byte-Offset Ende
    pub end: usize,
    /// Zeile (1-basiert)
    pub line: u32,
    /// Spalte (1-basiert)
    pub column: u32,
}

impl Span {
    /// Erstelle neuen Span
    pub fn new(start: usize, end: usize, line: u32, column: u32) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    /// Kombiniere zwei Spans
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line.min(other.line),
            column: if self.line <= other.line {
                self.column
            } else {
                other.column
            },
        }
    }

    /// Länge in Bytes
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Ist der Span leer?
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

/// Diagnostic Severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Ein einzelner Diagnose-Eintrag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Severity
    pub severity: DiagnosticSeverity,
    /// Error Code (z.B. "E0001")
    pub code: String,
    /// Nachricht
    pub message: String,
    /// Position im Source
    pub span: Span,
    /// Zusätzliche Labels/Hinweise
    pub labels: Vec<DiagnosticLabel>,
    /// Mögliche Fixes
    pub suggestions: Vec<String>,
}

/// Label für einen Diagnose-Span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticLabel {
    pub span: Span,
    pub message: String,
}

impl Diagnostic {
    /// Erstelle Error
    pub fn error(code: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            code: code.into(),
            message: message.into(),
            span,
            labels: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Erstelle Warning
    pub fn warning(code: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            code: code.into(),
            message: message.into(),
            span,
            labels: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Füge Label hinzu
    pub fn with_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(DiagnosticLabel {
            span,
            message: message.into(),
        });
        self
    }

    /// Füge Suggestion hinzu
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }
}

/// Collector für mehrere Diagnostics
#[derive(Debug, Clone, Default)]
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollector {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn error(&mut self, code: impl Into<String>, message: impl Into<String>, span: Span) {
        self.add(Diagnostic::error(code, message, span));
    }

    pub fn warning(&mut self, code: impl Into<String>, message: impl Into<String>, span: Span) {
        self.add(Diagnostic::warning(code, message, span));
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
    }

    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
    }

    pub fn all(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn take(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AST Nodes
// ═══════════════════════════════════════════════════════════════════════════

/// Ein komplettes ECL-Programm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    /// Liste von Policies
    pub policies: Vec<Policy>,
    /// Globale Konstanten
    pub constants: Vec<ConstDecl>,
    /// Source-Span des gesamten Programms
    pub span: Span,
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

/// Ein Statement mit Span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

/// Statement-Arten
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatementKind {
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

/// Ein Ausdruck mit Span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

/// Ausdrucks-Arten
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExprKind {
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
    pub span: Span,
}

// ═══════════════════════════════════════════════════════════════════════════
// Helper Constructors
// ═══════════════════════════════════════════════════════════════════════════

impl Expr {
    /// Erstelle Expr mit Span
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Erstelle Literal-Expr
    pub fn literal(lit: Literal) -> Self {
        Self {
            kind: ExprKind::Literal(lit),
            span: Span::default(),
        }
    }

    /// Erstelle Identifier-Expr
    pub fn ident(name: impl Into<String>) -> Self {
        Self {
            kind: ExprKind::Identifier(name.into()),
            span: Span::default(),
        }
    }

    /// Erstelle Binary-Expr
    pub fn binary(left: Expr, op: BinaryOp, right: Expr) -> Self {
        let span = left.span.merge(right.span);
        Self {
            kind: ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
            span,
        }
    }

    /// Erstelle Unary-Expr
    pub fn unary(op: UnaryOp, operand: Expr) -> Self {
        let span = operand.span;
        Self {
            kind: ExprKind::Unary {
                op,
                operand: Box::new(operand),
            },
            span,
        }
    }

    /// Erstelle Member-Access
    pub fn member(object: Expr, field: impl Into<String>) -> Self {
        let span = object.span;
        Self {
            kind: ExprKind::Member {
                object: Box::new(object),
                field: field.into(),
            },
            span,
        }
    }

    /// Erstelle Function Call
    pub fn call(function: impl Into<String>, args: Vec<Expr>) -> Self {
        Self {
            kind: ExprKind::Call {
                function: function.into(),
                args,
            },
            span: Span::default(),
        }
    }

    /// Erstelle TrustDim-Access
    pub fn trust_dim(vector: Expr, dimension: TrustDim) -> Self {
        let span = vector.span;
        Self {
            kind: ExprKind::TrustDim {
                vector: Box::new(vector),
                dimension,
            },
            span,
        }
    }
}

impl Statement {
    /// Erstelle Statement mit Span
    pub fn new(kind: StatementKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Erstelle Require-Statement
    pub fn require(expr: Expr, msg: Option<String>) -> Self {
        let span = expr.span;
        Self {
            kind: StatementKind::Require(expr, msg),
            span,
        }
    }

    /// Erstelle Let-Statement
    pub fn let_bind(name: impl Into<String>, expr: Expr) -> Self {
        let span = expr.span;
        Self {
            kind: StatementKind::Let(name.into(), expr),
            span,
        }
    }

    /// Erstelle Emit-Statement
    pub fn emit(event: impl Into<String>) -> Self {
        Self {
            kind: StatementKind::Emit(event.into()),
            span: Span::default(),
        }
    }

    /// Erstelle Return-Statement
    pub fn return_expr(expr: Expr) -> Self {
        let span = expr.span;
        Self {
            kind: StatementKind::Return(expr),
            span,
        }
    }
}

impl Program {
    /// Leeres Programm
    pub fn empty() -> Self {
        Self {
            policies: Vec::new(),
            constants: Vec::new(),
            span: Span::default(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Visitor Pattern
// ═══════════════════════════════════════════════════════════════════════════

/// Visitor Trait für AST Traversierung
///
/// Implementiere diesen Trait für eigene AST-Analysen oder Transformationen.
pub trait AstVisitor {
    /// Besuche Programm
    fn visit_program(&mut self, program: &Program) {
        walk_program(self, program);
    }

    /// Besuche Policy
    fn visit_policy(&mut self, policy: &Policy) {
        walk_policy(self, policy);
    }

    /// Besuche Statement
    fn visit_statement(&mut self, stmt: &Statement) {
        walk_statement(self, stmt);
    }

    /// Besuche Expression
    fn visit_expr(&mut self, expr: &Expr) {
        walk_expr(self, expr);
    }

    /// Besuche Literal
    fn visit_literal(&mut self, _lit: &Literal) {}

    /// Besuche Identifier
    fn visit_identifier(&mut self, _name: &str) {}

    /// Besuche BinaryOp
    fn visit_binary_op(&mut self, _op: BinaryOp) {}

    /// Besuche UnaryOp
    fn visit_unary_op(&mut self, _op: UnaryOp) {}

    /// Besuche TrustDim
    fn visit_trust_dim(&mut self, _dim: TrustDim) {}
}

/// Walk-Funktionen für Default-Traversierung

pub fn walk_program<V: AstVisitor + ?Sized>(visitor: &mut V, program: &Program) {
    for policy in &program.policies {
        visitor.visit_policy(policy);
    }
}

pub fn walk_policy<V: AstVisitor + ?Sized>(visitor: &mut V, policy: &Policy) {
    for stmt in &policy.body {
        visitor.visit_statement(stmt);
    }
}

pub fn walk_statement<V: AstVisitor + ?Sized>(visitor: &mut V, stmt: &Statement) {
    match &stmt.kind {
        StatementKind::Require(expr, _) => visitor.visit_expr(expr),
        StatementKind::Let(_, expr) => visitor.visit_expr(expr),
        StatementKind::Emit(_) => {}
        StatementKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            visitor.visit_expr(condition);
            for s in then_branch {
                visitor.visit_statement(s);
            }
            if let Some(else_stmts) = else_branch {
                for s in else_stmts {
                    visitor.visit_statement(s);
                }
            }
        }
        StatementKind::Return(expr) => visitor.visit_expr(expr),
    }
}

pub fn walk_expr<V: AstVisitor + ?Sized>(visitor: &mut V, expr: &Expr) {
    match &expr.kind {
        ExprKind::Literal(lit) => visitor.visit_literal(lit),
        ExprKind::Identifier(name) => visitor.visit_identifier(name),
        ExprKind::Binary { left, op, right } => {
            visitor.visit_expr(left);
            visitor.visit_binary_op(*op);
            visitor.visit_expr(right);
        }
        ExprKind::Unary { op, operand } => {
            visitor.visit_unary_op(*op);
            visitor.visit_expr(operand);
        }
        ExprKind::Member { object, .. } => visitor.visit_expr(object),
        ExprKind::Index { object, index } => {
            visitor.visit_expr(object);
            visitor.visit_expr(index);
        }
        ExprKind::Call { args, .. } => {
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        ExprKind::TrustDim { vector, dimension } => {
            visitor.visit_expr(vector);
            visitor.visit_trust_dim(*dimension);
        }
    }
}

/// Mutable Visitor für AST-Transformationen
pub trait AstVisitorMut {
    /// Besuche und transformiere Expression
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        walk_expr_mut(self, expr);
    }

    /// Besuche und transformiere Statement
    fn visit_statement_mut(&mut self, stmt: &mut Statement) {
        walk_statement_mut(self, stmt);
    }
}

pub fn walk_expr_mut<V: AstVisitorMut + ?Sized>(visitor: &mut V, expr: &mut Expr) {
    match &mut expr.kind {
        ExprKind::Binary { left, right, .. } => {
            visitor.visit_expr_mut(left);
            visitor.visit_expr_mut(right);
        }
        ExprKind::Unary { operand, .. } => visitor.visit_expr_mut(operand),
        ExprKind::Member { object, .. } => visitor.visit_expr_mut(object),
        ExprKind::Index { object, index } => {
            visitor.visit_expr_mut(object);
            visitor.visit_expr_mut(index);
        }
        ExprKind::Call { args, .. } => {
            for arg in args {
                visitor.visit_expr_mut(arg);
            }
        }
        ExprKind::TrustDim { vector, .. } => visitor.visit_expr_mut(vector),
        _ => {}
    }
}

pub fn walk_statement_mut<V: AstVisitorMut + ?Sized>(visitor: &mut V, stmt: &mut Statement) {
    match &mut stmt.kind {
        StatementKind::Require(expr, _) => visitor.visit_expr_mut(expr),
        StatementKind::Let(_, expr) => visitor.visit_expr_mut(expr),
        StatementKind::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            visitor.visit_expr_mut(condition);
            for s in then_branch {
                visitor.visit_statement_mut(s);
            }
            if let Some(else_stmts) = else_branch {
                for s in else_stmts {
                    visitor.visit_statement_mut(s);
                }
            }
        }
        StatementKind::Return(expr) => visitor.visit_expr_mut(expr),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_construction() {
        // require sender.trust.R >= 0.5
        let stmt = Statement::require(
            Expr::binary(
                Expr::trust_dim(Expr::member(Expr::ident("sender"), "trust"), TrustDim::R),
                BinaryOp::Gte,
                Expr::literal(Literal::Number(0.5)),
            ),
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
            span: Span::default(),
        };

        assert_eq!(program.policies.len(), 1);
        assert_eq!(program.policies[0].name, "transfer_guard");
    }

    #[test]
    fn test_visitor_pattern() {
        struct ExprCounter {
            count: usize,
        }

        impl AstVisitor for ExprCounter {
            fn visit_expr(&mut self, expr: &Expr) {
                self.count += 1;
                walk_expr(self, expr);
            }
        }

        let expr = Expr::binary(
            Expr::literal(Literal::Number(1.0)),
            BinaryOp::Add,
            Expr::binary(
                Expr::literal(Literal::Number(2.0)),
                BinaryOp::Mul,
                Expr::literal(Literal::Number(3.0)),
            ),
        );

        let mut counter = ExprCounter { count: 0 };
        counter.visit_expr(&expr);
        assert_eq!(counter.count, 5); // 1 binary + 2 inner binary + 3 literals
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(0, 10, 1, 1);
        let span2 = Span::new(15, 25, 2, 5);
        let merged = span1.merge(span2);

        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 25);
        assert_eq!(merged.line, 1);
    }

    #[test]
    fn test_diagnostic_collector() {
        let mut collector = DiagnosticCollector::new();
        collector.error("E0001", "Unexpected token", Span::default());
        collector.warning("W0001", "Unused variable", Span::default());

        assert!(collector.has_errors());
        assert_eq!(collector.len(), 2);
        assert_eq!(collector.errors().count(), 1);
    }
}
