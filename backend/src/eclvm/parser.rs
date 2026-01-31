//! # ECLVM Parser
//!
//! Parst ECL-Text in einen AST mit dem chumsky Parser-Combinator.
//!
//! ## Syntax
//!
//! ```text
//! // Kommentare
//! const MAX_TRANSFER = 10000
//!
//! policy "transfer_guard" {
//!     // Sender muss verifiziert sein
//!     require sender.trust.R >= 0.5
//!     require sender.credential("kyc-verified")
//!
//!     // Maximaler Betrag basierend auf Trust
//!     let max_amount = sender.trust.R * MAX_TRANSFER
//!     require amount <= max_amount, "Amount exceeds trust-based limit"
//!
//!     // Logging
//!     emit "transfer_approved"
//! }
//! ```

use crate::eclvm::ast::{
    BinaryOp, ConstDecl, Diagnostic, DiagnosticCollector, Expr, ExprKind, Literal, Policy, Program,
    Span, Statement, StatementKind, TrustDim, UnaryOp,
};
use crate::error::{ApiError, Result};
use chumsky::prelude::*;

// Alias for the chumsky Parser trait to avoid conflict with our Parser struct
use chumsky::Parser as ChumskyParser;

// ═══════════════════════════════════════════════════════════════════════════
// Token Types
// ═══════════════════════════════════════════════════════════════════════════

/// Lexer Token
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Policy,
    Require,
    Let,
    If,
    Else,
    Return,
    Emit,
    Const,
    True,
    False,
    Null,

    // Trust dimensions
    TrustR,
    TrustI,
    TrustC,
    TrustP,
    TrustV,
    TrustOmega,

    // Identifiers & Literals
    Ident(String),
    Number(f64),
    String(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    BangEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    AmpAmp,
    PipePipe,
    Bang,
    Dot,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Newline,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Policy => write!(f, "policy"),
            Token::Require => write!(f, "require"),
            Token::Let => write!(f, "let"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Return => write!(f, "return"),
            Token::Emit => write!(f, "emit"),
            Token::Const => write!(f, "const"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Null => write!(f, "null"),
            Token::TrustR => write!(f, "R"),
            Token::TrustI => write!(f, "I"),
            Token::TrustC => write!(f, "C"),
            Token::TrustP => write!(f, "P"),
            Token::TrustV => write!(f, "V"),
            Token::TrustOmega => write!(f, "Ω"),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Number(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Eq => write!(f, "="),
            Token::EqEq => write!(f, "=="),
            Token::BangEq => write!(f, "!="),
            Token::Lt => write!(f, "<"),
            Token::LtEq => write!(f, "<="),
            Token::Gt => write!(f, ">"),
            Token::GtEq => write!(f, ">="),
            Token::AmpAmp => write!(f, "&&"),
            Token::PipePipe => write!(f, "||"),
            Token::Bang => write!(f, "!"),
            Token::Dot => write!(f, "."),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::Newline => write!(f, "\\n"),
        }
    }
}

impl std::hash::Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl Eq for Token {}

// ═══════════════════════════════════════════════════════════════════════════
// Lexer
// ═══════════════════════════════════════════════════════════════════════════

type SimpleSpan = std::ops::Range<usize>;

/// Erstellt den Lexer für ECL
fn lexer() -> impl ChumskyParser<char, Vec<(Token, SimpleSpan)>, Error = Simple<char>> {
    // Kommentare
    let comment = just("//").then(take_until(just('\n'))).padded();

    // Whitespace (ohne Newlines für Statement-Trennung)
    let ws = filter(|c: &char| c.is_whitespace() && *c != '\n').repeated();

    // Strings
    let string = just('"')
        .ignore_then(filter(|c| *c != '"').repeated())
        .then_ignore(just('"'))
        .collect::<String>()
        .map(Token::String);

    // Numbers
    let number = text::int(10)
        .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
        .collect::<String>()
        .map(|s| Token::Number(s.parse().unwrap()));

    // Identifiers and Keywords
    let ident = text::ident().map(|s: String| match s.as_str() {
        "policy" => Token::Policy,
        "require" => Token::Require,
        "let" => Token::Let,
        "if" => Token::If,
        "else" => Token::Else,
        "return" => Token::Return,
        "emit" => Token::Emit,
        "const" => Token::Const,
        "true" => Token::True,
        "false" => Token::False,
        "null" => Token::Null,
        "R" => Token::TrustR,
        "I" => Token::TrustI,
        "C" => Token::TrustC,
        "P" => Token::TrustP,
        "V" => Token::TrustV,
        "omega" => Token::TrustOmega,
        _ => Token::Ident(s),
    });

    // Omega (Ω)
    let omega = just('Ω').to(Token::TrustOmega);

    // Operators (longer first!)
    let op = choice((
        just("==").to(Token::EqEq),
        just("!=").to(Token::BangEq),
        just("<=").to(Token::LtEq),
        just(">=").to(Token::GtEq),
        just("&&").to(Token::AmpAmp),
        just("||").to(Token::PipePipe),
        just('+').to(Token::Plus),
        just('-').to(Token::Minus),
        just('*').to(Token::Star),
        just('/').to(Token::Slash),
        just('%').to(Token::Percent),
        just('=').to(Token::Eq),
        just('<').to(Token::Lt),
        just('>').to(Token::Gt),
        just('!').to(Token::Bang),
        just('.').to(Token::Dot),
    ));

    // Delimiters
    let delim = choice((
        just('(').to(Token::LParen),
        just(')').to(Token::RParen),
        just('{').to(Token::LBrace),
        just('}').to(Token::RBrace),
        just('[').to(Token::LBracket),
        just(']').to(Token::RBracket),
        just(',').to(Token::Comma),
        just(':').to(Token::Colon),
    ));

    // Newline as token (for statement separation)
    let newline = just('\n').to(Token::Newline);

    // Combine all tokens
    let token = choice((string, number, omega, op, delim, ident, newline));

    token
        .map_with_span(|tok, span| (tok, span))
        .padded_by(comment.repeated())
        .padded_by(ws)
        .repeated()
        .then_ignore(end())
}

// ═══════════════════════════════════════════════════════════════════════════
// Parser Helpers
// ═══════════════════════════════════════════════════════════════════════════

fn to_span(range: SimpleSpan) -> Span {
    Span::new(range.start, range.end, 0, 0)
}

// ═══════════════════════════════════════════════════════════════════════════
// Expression Parser
// ═══════════════════════════════════════════════════════════════════════════

fn expr_parser() -> impl ChumskyParser<Token, Expr, Error = Simple<Token>> + Clone {
    recursive(|expr| {
        // Literals
        let literal = select! {
            Token::Number(n) => Literal::Number(n),
            Token::String(s) => Literal::String(s),
            Token::True => Literal::Bool(true),
            Token::False => Literal::Bool(false),
            Token::Null => Literal::Null,
        }
        .map_with_span(|lit, span| Expr::new(ExprKind::Literal(lit), to_span(span)));

        // Identifiers
        let ident = select! {
            Token::Ident(s) => s,
        }
        .map_with_span(|name, span| Expr::new(ExprKind::Identifier(name), to_span(span)));

        // Trust dimensions
        let trust_dim = select! {
            Token::TrustR => TrustDim::R,
            Token::TrustI => TrustDim::I,
            Token::TrustC => TrustDim::C,
            Token::TrustP => TrustDim::P,
            Token::TrustV => TrustDim::V,
            Token::TrustOmega => TrustDim::Omega,
        };

        // Parenthesized expression
        let paren = expr
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen));

        // Atom (base expressions)
        let atom = choice((literal, ident, paren));

        // Postfix operations: function calls, member access, trust dimension
        let postfix = atom
            .then(
                choice((
                    // Function call: foo(args)
                    expr.clone()
                        .separated_by(just(Token::Comma))
                        .allow_trailing()
                        .delimited_by(just(Token::LParen), just(Token::RParen))
                        .map(|args| PostfixOp::Call(args)),
                    // Trust dimension access: .R, .I, etc.
                    just(Token::Dot)
                        .ignore_then(trust_dim)
                        .map(PostfixOp::TrustDim),
                    // Member access: .field
                    just(Token::Dot)
                        .ignore_then(select! { Token::Ident(s) => s })
                        .map(PostfixOp::Member),
                ))
                .repeated(),
            )
            .foldl(|e, op| {
                let span = e.span;
                match op {
                    PostfixOp::Call(args) => {
                        if let ExprKind::Identifier(func) = &e.kind {
                            Expr::new(
                                ExprKind::Call {
                                    function: func.clone(),
                                    args,
                                },
                                span,
                            )
                        } else {
                            e // Error: can't call non-identifier
                        }
                    }
                    PostfixOp::Member(field) => Expr::new(
                        ExprKind::Member {
                            object: Box::new(e),
                            field,
                        },
                        span,
                    ),
                    PostfixOp::TrustDim(dim) => Expr::new(
                        ExprKind::TrustDim {
                            vector: Box::new(e),
                            dimension: dim,
                        },
                        span,
                    ),
                }
            });

        // Unary operators
        let unary = just(Token::Bang)
            .or(just(Token::Minus))
            .repeated()
            .then(postfix)
            .foldr(|op, e| {
                let unary_op = match op {
                    Token::Bang => UnaryOp::Not,
                    Token::Minus => UnaryOp::Neg,
                    _ => unreachable!(),
                };
                Expr::unary(unary_op, e)
            });

        // Binary operators with precedence

        // Multiplication/Division (highest precedence)
        let product = unary
            .clone()
            .then(
                choice((
                    just(Token::Star).to(BinaryOp::Mul),
                    just(Token::Slash).to(BinaryOp::Div),
                    just(Token::Percent).to(BinaryOp::Mod),
                ))
                .then(unary)
                .repeated(),
            )
            .foldl(|a, (op, b)| Expr::binary(a, op, b));

        // Addition/Subtraction
        let sum = product
            .clone()
            .then(
                choice((
                    just(Token::Plus).to(BinaryOp::Add),
                    just(Token::Minus).to(BinaryOp::Sub),
                ))
                .then(product)
                .repeated(),
            )
            .foldl(|a, (op, b)| Expr::binary(a, op, b));

        // Comparisons
        let comparison = sum
            .clone()
            .then(
                choice((
                    just(Token::EqEq).to(BinaryOp::Eq),
                    just(Token::BangEq).to(BinaryOp::Neq),
                    just(Token::LtEq).to(BinaryOp::Lte),
                    just(Token::GtEq).to(BinaryOp::Gte),
                    just(Token::Lt).to(BinaryOp::Lt),
                    just(Token::Gt).to(BinaryOp::Gt),
                ))
                .then(sum)
                .repeated(),
            )
            .foldl(|a, (op, b)| Expr::binary(a, op, b));

        // Logical AND
        let logical_and = comparison
            .clone()
            .then(
                just(Token::AmpAmp)
                    .to(BinaryOp::And)
                    .then(comparison)
                    .repeated(),
            )
            .foldl(|a, (op, b)| Expr::binary(a, op, b));

        // Logical OR (lowest precedence)
        logical_and
            .clone()
            .then(
                just(Token::PipePipe)
                    .to(BinaryOp::Or)
                    .then(logical_and)
                    .repeated(),
            )
            .foldl(|a, (op, b)| Expr::binary(a, op, b))
    })
}

/// Postfix operation helper
enum PostfixOp {
    Call(Vec<Expr>),
    Member(String),
    TrustDim(TrustDim),
}

// ═══════════════════════════════════════════════════════════════════════════
// Statement Parser
// ═══════════════════════════════════════════════════════════════════════════

fn statement_parser() -> impl ChumskyParser<Token, Statement, Error = Simple<Token>> + Clone {
    let expr = expr_parser();

    // require <expr> [, "message"]
    let require = just(Token::Require)
        .ignore_then(expr.clone())
        .then(
            just(Token::Comma)
                .ignore_then(select! { Token::String(s) => s })
                .or_not(),
        )
        .map_with_span(|(e, msg), span| {
            Statement::new(StatementKind::Require(e, msg), to_span(span))
        });

    // let <name> = <expr>
    let let_stmt = just(Token::Let)
        .ignore_then(select! { Token::Ident(s) => s })
        .then_ignore(just(Token::Eq))
        .then(expr.clone())
        .map_with_span(|(name, e), span| {
            Statement::new(StatementKind::Let(name, e), to_span(span))
        });

    // emit <string>
    let emit = just(Token::Emit)
        .ignore_then(select! { Token::String(s) => s })
        .map_with_span(|event, span| Statement::new(StatementKind::Emit(event), to_span(span)));

    // return <expr>
    let return_stmt = just(Token::Return)
        .ignore_then(expr.clone())
        .map_with_span(|e, span| Statement::new(StatementKind::Return(e), to_span(span)));

    // Newlines between statements
    let newlines = just(Token::Newline).repeated();

    // if <cond> { <body> } [else { <body> }]
    let if_stmt = recursive(|if_stmt| {
        let stmt = choice((
            require.clone(),
            let_stmt.clone(),
            emit.clone(),
            return_stmt.clone(),
            if_stmt,
        ));

        let block = stmt
            .padded_by(newlines.clone())
            .repeated()
            .delimited_by(just(Token::LBrace), just(Token::RBrace));

        just(Token::If)
            .ignore_then(expr.clone())
            .then(block.clone())
            .then(just(Token::Else).ignore_then(block).or_not())
            .map_with_span(|((cond, then_branch), else_branch), span| {
                Statement::new(
                    StatementKind::If {
                        condition: cond,
                        then_branch,
                        else_branch,
                    },
                    to_span(span),
                )
            })
    });

    choice((require, let_stmt, emit, return_stmt, if_stmt))
}

// ═══════════════════════════════════════════════════════════════════════════
// Policy Parser
// ═══════════════════════════════════════════════════════════════════════════

fn policy_parser() -> impl ChumskyParser<Token, Policy, Error = Simple<Token>> {
    let newlines = just(Token::Newline).repeated();
    let stmt = statement_parser();

    just(Token::Policy)
        .ignore_then(select! { Token::String(s) => s })
        .then(
            stmt.padded_by(newlines.clone())
                .repeated()
                .delimited_by(just(Token::LBrace), just(Token::RBrace)),
        )
        .map_with_span(|(name, body), span| Policy {
            name,
            body,
            span: to_span(span),
        })
}

// ═══════════════════════════════════════════════════════════════════════════
// Program Parser
// ═══════════════════════════════════════════════════════════════════════════

fn const_parser() -> impl ChumskyParser<Token, ConstDecl, Error = Simple<Token>> {
    just(Token::Const)
        .ignore_then(select! { Token::Ident(s) => s })
        .then_ignore(just(Token::Eq))
        .then(select! {
            Token::Number(n) => Literal::Number(n),
            Token::String(s) => Literal::String(s),
            Token::True => Literal::Bool(true),
            Token::False => Literal::Bool(false),
        })
        .map_with_span(|(name, value), span| ConstDecl {
            name,
            value,
            span: to_span(span),
        })
}

/// Item in einem Programm (Const oder Policy)
enum ProgramItem {
    Const(ConstDecl),
    Policy(Policy),
}

fn program_parser() -> impl ChumskyParser<Token, Program, Error = Simple<Token>> {
    let newlines = just(Token::Newline).repeated();

    let item = choice((
        const_parser().map(ProgramItem::Const),
        policy_parser().map(ProgramItem::Policy),
    ));

    item.padded_by(newlines)
        .repeated()
        .then_ignore(end())
        .map_with_span(|items, span| {
            let mut constants = Vec::new();
            let mut policies = Vec::new();

            for item in items {
                match item {
                    ProgramItem::Const(c) => constants.push(c),
                    ProgramItem::Policy(p) => policies.push(p),
                }
            }

            Program {
                policies,
                constants,
                span: to_span(span),
            }
        })
}

// ═══════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════

/// Parser für ECL-Programme
pub struct Parser;

impl Parser {
    /// Parse ECL-Text in AST
    pub fn parse(source: &str) -> Result<Program> {
        // Step 1: Lexing
        let (tokens, lex_errors) = lexer().parse_recovery(source);

        if !lex_errors.is_empty() {
            let errors: Vec<String> = lex_errors
                .iter()
                .map(|e| format!("Lexer error at {}: {:?}", e.span().start, e))
                .collect();
            return Err(ApiError::Validation(errors.join("; ")));
        }

        let tokens =
            tokens.ok_or_else(|| ApiError::Validation("Lexer produced no tokens".to_string()))?;

        // Step 2: Parsing - create proper stream from tokens
        let len = source.len();
        let stream = chumsky::Stream::from_iter(len..len + 1, tokens.into_iter());

        let (program, parse_errors) = program_parser().parse_recovery(stream);

        if !parse_errors.is_empty() {
            let errors: Vec<String> = parse_errors
                .iter()
                .map(|e| format!("Parse error: {:?}", e))
                .collect();
            return Err(ApiError::Validation(errors.join("; ")));
        }

        program.ok_or_else(|| ApiError::Validation("Parser produced no output".to_string()))
    }

    /// Parse mit detaillierten Diagnostics
    pub fn parse_with_diagnostics(source: &str) -> (Option<Program>, DiagnosticCollector) {
        let mut diagnostics = DiagnosticCollector::new();

        // Step 1: Lexing
        let (tokens, lex_errors) = lexer().parse_recovery(source);

        for error in &lex_errors {
            let span = Span::new(error.span().start, error.span().end, 0, 0);
            diagnostics.add(Diagnostic::error(
                "E1001",
                format!("Unexpected character: {:?}", error),
                span,
            ));
        }

        let Some(tokens) = tokens else {
            diagnostics.error("E1002", "Lexer failed completely", Span::default());
            return (None, diagnostics);
        };

        // Step 2: Parsing
        let len = source.len();
        let stream = chumsky::Stream::from_iter(len..len + 1, tokens.into_iter());

        let (program, parse_errors) = program_parser().parse_recovery(stream);

        for error in &parse_errors {
            diagnostics.add(Diagnostic::error(
                "E2001",
                format!("Syntax error: {:?}", error),
                Span::default(),
            ));
        }

        (program, diagnostics)
    }

    /// Parse einzelne Expression (für REPL)
    pub fn parse_expr(source: &str) -> Result<Expr> {
        let (tokens, lex_errors) = lexer().parse_recovery(source);

        if !lex_errors.is_empty() {
            return Err(ApiError::Validation("Lexer error".to_string()));
        }

        let tokens = tokens.ok_or_else(|| ApiError::Validation("No tokens".to_string()))?;
        let filtered: Vec<_> = tokens
            .into_iter()
            .filter(|(t, _)| *t != Token::Newline)
            .collect();

        let len = source.len();
        let stream = chumsky::Stream::from_iter(len..len + 1, filtered.into_iter());

        expr_parser()
            .then_ignore(end())
            .parse(stream)
            .map_err(|e| ApiError::Validation(format!("Parse error: {:?}", e)))
    }

    /// Parse einzelne Policy
    pub fn parse_policy(source: &str) -> Result<Policy> {
        let full_source = if source.trim().starts_with("policy") {
            source.to_string()
        } else {
            format!("policy \"unnamed\" {{ {} }}", source)
        };

        let program = Self::parse(&full_source)?;
        program
            .policies
            .into_iter()
            .next()
            .ok_or_else(|| ApiError::Validation("No policy found".to_string()))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_policy() {
        let source = r#"
policy "test" {
    require true
}
"#;

        let program = Parser::parse(source).unwrap();
        assert_eq!(program.policies.len(), 1);
        assert_eq!(program.policies[0].name, "test");
    }

    #[test]
    fn test_parse_require_with_message() {
        let source = r#"
policy "test" {
    require x > 5, "x must be greater than 5"
}
"#;

        let program = Parser::parse(source).unwrap();
        assert_eq!(program.policies.len(), 1);

        if let StatementKind::Require(_, msg) = &program.policies[0].body[0].kind {
            assert_eq!(msg.as_deref(), Some("x must be greater than 5"));
        } else {
            panic!("Expected Require statement");
        }
    }

    #[test]
    fn test_parse_binary_expression() {
        let source = "5 + 3 * 2";
        let expr = Parser::parse_expr(source).unwrap();

        // Should parse as 5 + (3 * 2) due to precedence
        if let ExprKind::Binary { op, .. } = &expr.kind {
            assert_eq!(*op, BinaryOp::Add);
        } else {
            panic!("Expected Binary expression");
        }
    }

    #[test]
    fn test_parse_trust_access() {
        let source = r#"
policy "trust_check" {
    require sender.trust.R >= 0.5
}
"#;

        let program = Parser::parse(source).unwrap();
        assert_eq!(program.policies.len(), 1);
    }

    #[test]
    fn test_parse_function_call() {
        let source = r#"
policy "credential_check" {
    require credential("kyc-verified")
}
"#;

        let program = Parser::parse(source).unwrap();
        assert_eq!(program.policies.len(), 1);
    }

    #[test]
    fn test_parse_let_statement() {
        let source = r#"
policy "calculation" {
    let max_amount = 1000 * 0.5
    require amount <= max_amount
}
"#;

        let program = Parser::parse(source).unwrap();
        assert_eq!(program.policies[0].body.len(), 2);

        if let StatementKind::Let(name, _) = &program.policies[0].body[0].kind {
            assert_eq!(name, "max_amount");
        } else {
            panic!("Expected Let statement");
        }
    }

    #[test]
    fn test_parse_const_declaration() {
        let source = r#"
const MAX_TRANSFER = 10000

policy "test" {
    require true
}
"#;

        let program = Parser::parse(source).unwrap();
        assert_eq!(program.constants.len(), 1);
        assert_eq!(program.constants[0].name, "MAX_TRANSFER");
    }

    #[test]
    fn test_parse_if_statement() {
        let source = r#"
policy "conditional" {
    if x > 10 {
        emit "large"
    } else {
        emit "small"
    }
}
"#;

        let program = Parser::parse(source).unwrap();
        assert_eq!(program.policies[0].body.len(), 1);

        if let StatementKind::If {
            then_branch,
            else_branch,
            ..
        } = &program.policies[0].body[0].kind
        {
            assert_eq!(then_branch.len(), 1);
            assert!(else_branch.is_some());
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_parse_with_diagnostics() {
        let source = "policy \"test\" { require true }";
        let (program, diagnostics) = Parser::parse_with_diagnostics(source);

        assert!(program.is_some());
        assert!(!diagnostics.has_errors());
    }

    #[test]
    fn test_lexer_tokens() {
        let source = "policy \"test\" { require x >= 0.5 }";
        let (tokens, errors) = lexer().parse_recovery(source);

        assert!(errors.is_empty());
        let tokens = tokens.unwrap();

        // Check we got expected tokens
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Policy)));
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Require)));
        assert!(tokens.iter().any(|(t, _)| matches!(t, Token::GtEq)));
    }

    #[test]
    fn test_parser_not_implemented_fallback() {
        // Der alte Test sollte jetzt funktionieren
        let result = Parser::parse("policy \"test\" { require true }");
        assert!(result.is_ok());
    }
}
