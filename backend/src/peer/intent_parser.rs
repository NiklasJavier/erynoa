//! # Intent Parser
//!
//! Parst Nutzer-Intents in strukturierte Ziele gemäß Κ22.
//!
//! ## Axiom-Referenz
//!
//! - **Κ22 (Saga-Composer)**: `∀ Intent I : ∃! Saga S : resolve(I) = S`
//!
//! ## Intent-Typen
//!
//! Der Parser unterstützt:
//! - Strukturierte Intents (JSON)
//! - Natürlichsprachliche Intents (via Patterns)

use crate::domain::{Constraint, Goal, Intent, RealmId, DID, ROOT_REALM_ID};
use thiserror::Error;

/// Fehler beim Intent-Parsing
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid intent format: {0}")]
    InvalidFormat(String),

    #[error("Unknown goal type: {0}")]
    UnknownGoalType(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid constraint: {0}")]
    InvalidConstraint(String),
}

/// Ergebnis von Parse-Operationen
pub type ParseResult<T> = Result<T, ParseError>;

/// Intent Parser (Κ22)
///
/// ```text
///     User Input                Structured Intent
///          │                          │
///          ▼                          │
///    ┌───────────┐                   │
///    │   Parse   │◀──────────────────┘
///    └─────┬─────┘
///          │
///          ▼
///    ┌───────────┐
///    │ Validate  │
///    └─────┬─────┘
///          │
///          ▼
///    ┌───────────┐
///    │  Intent   │
///    └───────────┘
/// ```
pub struct IntentParser {
    /// Pattern-Matcher für natürlichsprachliche Intents
    patterns: Vec<PatternMatcher>,

    /// Konfiguration
    config: IntentParserConfig,
}

/// Konfiguration für IntentParser
#[derive(Debug, Clone)]
pub struct IntentParserConfig {
    /// Default Timeout in Stunden
    pub default_timeout_hours: u64,

    /// Default Realm
    pub default_realm: RealmId,

    /// Maximale Constraints pro Intent
    pub max_constraints: usize,
}

impl Default for IntentParserConfig {
    fn default() -> Self {
        Self {
            default_timeout_hours: 24,
            default_realm: ROOT_REALM_ID,
            max_constraints: 10,
        }
    }
}

/// Pattern für natürlichsprachliche Intents
struct PatternMatcher {
    /// Regex/Keywords
    keywords: Vec<String>,
    /// Ziel-Goal-Typ
    goal_type: GoalType,
}

#[derive(Debug, Clone, Copy)]
enum GoalType {
    Transfer,
    Attest,
    Delegate,
    Query,
    Create,
}

impl IntentParser {
    /// Erstelle neuen IntentParser
    pub fn new(config: IntentParserConfig) -> Self {
        let patterns = vec![
            PatternMatcher {
                keywords: vec!["send".into(), "transfer".into(), "pay".into()],
                goal_type: GoalType::Transfer,
            },
            PatternMatcher {
                keywords: vec!["attest".into(), "verify".into(), "certify".into()],
                goal_type: GoalType::Attest,
            },
            PatternMatcher {
                keywords: vec!["delegate".into(), "authorize".into(), "grant".into()],
                goal_type: GoalType::Delegate,
            },
            PatternMatcher {
                keywords: vec!["query".into(), "find".into(), "search".into()],
                goal_type: GoalType::Query,
            },
            PatternMatcher {
                keywords: vec!["create".into(), "new".into(), "mint".into()],
                goal_type: GoalType::Create,
            },
        ];

        Self { patterns, config }
    }

    /// Erstelle mit Default-Config
    pub fn default() -> Self {
        Self::new(IntentParserConfig::default())
    }

    /// Parse strukturierten Intent
    pub fn parse_structured(
        &self,
        source: DID,
        goal: Goal,
        constraints: Vec<Constraint>,
    ) -> ParseResult<Intent> {
        // Validiere Constraints
        if constraints.len() > self.config.max_constraints {
            return Err(ParseError::InvalidConstraint(format!(
                "Too many constraints: {} > {}",
                constraints.len(),
                self.config.max_constraints
            )));
        }

        let mut intent = Intent::new(
            source.id.clone(),
            goal,
            self.config.default_realm.clone(),
            0,
        );

        for constraint in constraints {
            intent = intent.with_constraint(constraint);
        }

        intent = intent.with_timeout(self.config.default_timeout_hours * 3600);

        Ok(intent)
    }

    /// Parse Transfer-Intent
    pub fn parse_transfer(
        &self,
        from: DID,
        to: DID,
        amount: u64,
        asset_type: String,
    ) -> ParseResult<Intent> {
        let goal = Goal::Transfer {
            to: to.id,
            amount,
            asset_type,
        };

        self.parse_structured(from, goal, vec![])
    }

    /// Parse Delegation-Intent
    pub fn parse_delegation(
        &self,
        from: DID,
        to: DID,
        capabilities: Vec<String>,
        ttl_seconds: u64,
    ) -> ParseResult<Intent> {
        let goal = Goal::Delegate {
            to: to.id,
            capabilities,
            trust_factor: 1.0,
            ttl_seconds,
        };

        self.parse_structured(from, goal, vec![])
    }

    /// Parse Query-Intent
    pub fn parse_query(&self, source: DID, predicate: String) -> ParseResult<Intent> {
        let goal = Goal::Query { predicate };
        self.parse_structured(source, goal, vec![])
    }

    /// Parse natürlichsprachlichen Intent (vereinfacht)
    pub fn parse_natural(&self, source: DID, text: &str) -> ParseResult<Intent> {
        let text_lower = text.to_lowercase();

        // Finde passendes Pattern
        let goal_type = self
            .patterns
            .iter()
            .find(|p| p.keywords.iter().any(|k| text_lower.contains(k)))
            .map(|p| p.goal_type);

        match goal_type {
            Some(GoalType::Transfer) => {
                // Extrahiere Betrag und Empfänger (vereinfacht)
                let goal = Goal::Complex {
                    description: text.to_string(),
                    sub_goals: vec![Goal::Transfer {
                        to: DID::new_self(b"unknown").id,
                        amount: 0,
                        asset_type: "ERY".to_string(),
                    }],
                };
                self.parse_structured(source, goal, vec![])
            }
            Some(GoalType::Query) => {
                let goal = Goal::Query {
                    predicate: text.to_string(),
                };
                self.parse_structured(source, goal, vec![])
            }
            Some(GoalType::Attest) => {
                let goal = Goal::Complex {
                    description: text.to_string(),
                    sub_goals: vec![Goal::Attest {
                        subject: DID::new_self(b"unknown").id,
                        claim: text.to_string(),
                    }],
                };
                self.parse_structured(source, goal, vec![])
            }
            _ => {
                // Fallback: Complex Goal
                let goal = Goal::Complex {
                    description: text.to_string(),
                    sub_goals: vec![],
                };
                self.parse_structured(source, goal, vec![])
            }
        }
    }

    /// Validiere Intent
    pub fn validate(&self, intent: &Intent) -> ParseResult<()> {
        // Prüfe Goal-Konsistenz
        match &intent.goal {
            Goal::Transfer { amount, .. } => {
                if *amount == 0 {
                    return Err(ParseError::InvalidFormat(
                        "Transfer amount cannot be zero".to_string(),
                    ));
                }
            }
            Goal::Delegate { capabilities, .. } => {
                if capabilities.is_empty() {
                    return Err(ParseError::MissingField("capabilities".to_string()));
                }
            }
            _ => {}
        }

        // Prüfe Constraints
        for constraint in &intent.constraints {
            self.validate_constraint(constraint)?;
        }

        Ok(())
    }

    fn validate_constraint(&self, constraint: &Constraint) -> ParseResult<()> {
        match constraint {
            Constraint::MinTrust { value } => {
                if *value < 0.0 || *value > 1.0 {
                    return Err(ParseError::InvalidConstraint(format!(
                        "MinTrust must be in [0,1], got {}",
                        value
                    )));
                }
            }
            Constraint::MaxCost { cost, .. } => {
                if cost.gas == 0 && cost.mana == 0 {
                    return Err(ParseError::InvalidConstraint(
                        "MaxCost cannot be zero".to_string(),
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DIDNamespace;

    #[test]
    fn test_parse_transfer() {
        let parser = IntentParser::default();
        let alice = DID::new(DIDNamespace::Self_, b"alice");
        let bob = DID::new(DIDNamespace::Self_, b"bob");

        let intent = parser
            .parse_transfer(alice.clone(), bob.clone(), 100, "ERY".to_string())
            .unwrap();

        match intent.goal {
            Goal::Transfer {
                to,
                amount,
                asset_type,
            } => {
                assert_eq!(to, bob.id);
                assert_eq!(amount, 100);
                assert_eq!(asset_type, "ERY");
            }
            _ => panic!("Expected Transfer goal"),
        }
    }

    #[test]
    fn test_parse_delegation() {
        let parser = IntentParser::default();
        let alice = DID::new(DIDNamespace::Self_, b"alice");
        let bob = DID::new(DIDNamespace::Self_, b"bob");

        let intent = parser
            .parse_delegation(
                alice,
                bob.clone(),
                vec!["transfer".to_string(), "attest".to_string()],
                86400,
            )
            .unwrap();

        match intent.goal {
            Goal::Delegate {
                to,
                capabilities,
                ttl_seconds,
                ..
            } => {
                assert_eq!(to, bob.id);
                assert_eq!(capabilities.len(), 2);
                assert_eq!(ttl_seconds, 86400);
            }
            _ => panic!("Expected Delegate goal"),
        }
    }

    #[test]
    fn test_parse_natural_transfer() {
        let parser = IntentParser::default();
        let alice = DID::new(DIDNamespace::Self_, b"alice");

        let intent = parser.parse_natural(alice, "send 100 ERY to Bob").unwrap();

        match intent.goal {
            Goal::Complex { description, .. } => {
                assert!(description.contains("send"));
            }
            _ => panic!("Expected Complex goal"),
        }
    }

    #[test]
    fn test_validate_zero_amount() {
        let parser = IntentParser::default();
        let alice = DID::new(DIDNamespace::Self_, b"alice");
        let bob = DID::new(DIDNamespace::Self_, b"bob");

        let result = parser.parse_transfer(alice, bob, 0, "ERY".to_string());
        let intent = result.unwrap();

        assert!(parser.validate(&intent).is_err());
    }
}
