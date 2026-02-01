//! # Quadratic Governance
//!
//! Quadratisches Voting gemäß Κ21.
//!
//! ## Axiom-Referenz
//!
//! - **Κ21 (Quadratisches Governance)**: `vote_cost(n) = n²`
//!
//! ## Mechanik
//!
//! ```text
//! Stimmen │ Kosten
//! ────────┼───────
//!    1    │    1
//!    2    │    4
//!    3    │    9
//!    4    │   16
//!    5    │   25
//! ```
//!
//! Dies reduziert die Macht von Whales und fördert breite Partizipation.

use crate::domain::DID;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use thiserror::Error;

/// Fehler bei Governance-Operationen
#[derive(Debug, Error)]
pub enum GovernanceError {
    #[error("Proposal not found: {0}")]
    ProposalNotFound(String),

    #[error("Insufficient credits: have {have}, need {need}")]
    InsufficientCredits { have: u64, need: u64 },

    #[error("Proposal already closed")]
    ProposalClosed,

    #[error("Already voted on this proposal")]
    AlreadyVoted,

    #[error("Invalid vote count: {0}")]
    InvalidVoteCount(String),
}

/// Ergebnis von Governance-Operationen
pub type GovernanceResult<T> = Result<T, GovernanceError>;

/// Quadratic Governance Engine (Κ21)
///
/// ```text
///                    Κ21: vote_cost(n) = n²
///                              │
///                              ▼
///     ┌──────────────────────────────────────────────┐
///     │                 Proposal                      │
///     │                                              │
///     │   Voter A: 2 votes → 4 credits              │
///     │   Voter B: 1 vote  → 1 credit               │
///     │   Voter C: 3 votes → 9 credits              │
///     │                                              │
///     │   Total For:  5 votes                        │
///     │   Total Against: 1 vote                      │
///     └──────────────────────────────────────────────┘
/// ```
pub struct QuadraticGovernance {
    /// Proposals
    proposals: HashMap<String, Proposal>,

    /// Credits pro DID
    credits: HashMap<DID, u64>,

    /// Stimmen pro Proposal pro DID
    votes: HashMap<String, HashMap<DID, Vote>>,

    /// Konfiguration
    config: QuadraticConfig,
}

/// Konfiguration für QuadraticGovernance
#[derive(Debug, Clone)]
pub struct QuadraticConfig {
    /// Initiale Credits für neue Voter
    pub initial_credits: u64,

    /// Minimum Quorum (Anteil der Voter)
    pub quorum_ratio: f64,

    /// Minimum Approval für Annahme
    pub approval_threshold: f64,

    /// Proposal-Dauer in Stunden
    pub proposal_duration_hours: u64,
}

impl Default for QuadraticConfig {
    fn default() -> Self {
        Self {
            initial_credits: 100,
            quorum_ratio: 0.1,            // 10% müssen teilnehmen
            approval_threshold: 0.5,      // 50% Zustimmung
            proposal_duration_hours: 168, // 1 Woche
        }
    }
}

/// Ein Governance-Proposal
#[derive(Debug, Clone)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposer: DID,
    pub created_at: DateTime<Utc>,
    pub closes_at: DateTime<Utc>,
    pub status: ProposalStatus,
}

/// Status eines Proposals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Expired,
}

/// Eine Stimme
#[derive(Debug, Clone)]
pub struct Vote {
    pub voter: DID,
    pub direction: VoteDirection,
    pub vote_count: u64,
    pub credit_cost: u64,
    pub timestamp: DateTime<Utc>,
}

/// Abstimmungsrichtung
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoteDirection {
    For,
    Against,
    Abstain,
}

impl QuadraticGovernance {
    /// Erstelle neue QuadraticGovernance Engine
    pub fn new(config: QuadraticConfig) -> Self {
        Self {
            proposals: HashMap::new(),
            credits: HashMap::new(),
            votes: HashMap::new(),
            config,
        }
    }

    /// Erstelle mit Default-Config
    pub fn default() -> Self {
        Self::new(QuadraticConfig::default())
    }

    /// Κ21: Berechne Kosten für n Stimmen
    ///
    /// ```text
    /// cost(n) = n²
    /// ```
    pub fn vote_cost(vote_count: u64) -> u64 {
        vote_count.saturating_mul(vote_count)
    }

    /// Berechne wie viele Stimmen mit gegebenen Credits möglich
    pub fn max_votes_for_credits(credits: u64) -> u64 {
        (credits as f64).sqrt().floor() as u64
    }

    /// Registriere neuen Voter
    pub fn register_voter(&mut self, did: DID) {
        self.credits
            .entry(did)
            .or_insert(self.config.initial_credits);
    }

    /// Hole Credits für DID
    pub fn get_credits(&self, did: &DID) -> u64 {
        self.credits.get(did).copied().unwrap_or(0)
    }

    /// Füge Credits hinzu
    pub fn add_credits(&mut self, did: &DID, amount: u64) {
        *self.credits.entry(did.clone()).or_default() += amount;
    }

    /// Erstelle neues Proposal
    pub fn create_proposal(
        &mut self,
        id: String,
        title: String,
        description: String,
        proposer: DID,
    ) -> Proposal {
        let now = Utc::now();
        let closes_at = now + chrono::Duration::hours(self.config.proposal_duration_hours as i64);

        let proposal = Proposal {
            id: id.clone(),
            title,
            description,
            proposer,
            created_at: now,
            closes_at,
            status: ProposalStatus::Active,
        };

        self.proposals.insert(id.clone(), proposal.clone());
        self.votes.insert(id, HashMap::new());

        proposal
    }

    /// Abstimmen auf Proposal
    pub fn vote(
        &mut self,
        proposal_id: &str,
        voter: DID,
        direction: VoteDirection,
        vote_count: u64,
    ) -> GovernanceResult<Vote> {
        // Prüfe Proposal existiert
        let proposal = self
            .proposals
            .get(proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.to_string()))?;

        // Prüfe Proposal ist aktiv
        if proposal.status != ProposalStatus::Active {
            return Err(GovernanceError::ProposalClosed);
        }

        // Prüfe nicht bereits abgestimmt
        if self
            .votes
            .get(proposal_id)
            .map(|v| v.contains_key(&voter))
            .unwrap_or(false)
        {
            return Err(GovernanceError::AlreadyVoted);
        }

        // Κ21: Berechne Kosten
        let cost = Self::vote_cost(vote_count);

        // Prüfe Credits
        let available = self.get_credits(&voter);
        if available < cost {
            return Err(GovernanceError::InsufficientCredits {
                have: available,
                need: cost,
            });
        }

        // Registriere Voter falls nicht existiert
        self.register_voter(voter.clone());

        // Ziehe Credits ab
        *self.credits.get_mut(&voter).unwrap() -= cost;

        // Erstelle Vote
        let vote = Vote {
            voter: voter.clone(),
            direction,
            vote_count,
            credit_cost: cost,
            timestamp: Utc::now(),
        };

        // Speichere Vote
        self.votes
            .get_mut(proposal_id)
            .unwrap()
            .insert(voter, vote.clone());

        Ok(vote)
    }

    /// Berechne Ergebnis eines Proposals
    pub fn tally(&self, proposal_id: &str) -> GovernanceResult<ProposalTally> {
        let votes = self
            .votes
            .get(proposal_id)
            .ok_or_else(|| GovernanceError::ProposalNotFound(proposal_id.to_string()))?;

        let mut votes_for = 0u64;
        let mut votes_against = 0u64;
        let mut votes_abstain = 0u64;
        let mut credits_spent = 0u64;

        for vote in votes.values() {
            credits_spent += vote.credit_cost;

            match vote.direction {
                VoteDirection::For => votes_for += vote.vote_count,
                VoteDirection::Against => votes_against += vote.vote_count,
                VoteDirection::Abstain => votes_abstain += vote.vote_count,
            }
        }

        let total_votes = votes_for + votes_against;
        let approval_ratio = if total_votes > 0 {
            votes_for as f64 / total_votes as f64
        } else {
            0.0
        };

        let unique_voters = votes.len();
        let total_voters = self.credits.len();
        let quorum_reached = total_voters == 0
            || (unique_voters as f64 / total_voters as f64) >= self.config.quorum_ratio;

        let passed = quorum_reached && approval_ratio >= self.config.approval_threshold;

        Ok(ProposalTally {
            proposal_id: proposal_id.to_string(),
            votes_for,
            votes_against,
            votes_abstain,
            credits_spent,
            unique_voters,
            approval_ratio,
            quorum_reached,
            passed,
        })
    }

    /// Schließe Proposal und setze finalen Status
    pub fn finalize(&mut self, proposal_id: &str) -> GovernanceResult<ProposalStatus> {
        let tally = self.tally(proposal_id)?;

        let status = if tally.passed {
            ProposalStatus::Passed
        } else {
            ProposalStatus::Rejected
        };

        if let Some(proposal) = self.proposals.get_mut(proposal_id) {
            proposal.status = status;
        }

        Ok(status)
    }

    /// Statistiken
    pub fn stats(&self) -> QuadraticGovernanceStats {
        let active = self
            .proposals
            .values()
            .filter(|p| p.status == ProposalStatus::Active)
            .count();
        let passed = self
            .proposals
            .values()
            .filter(|p| p.status == ProposalStatus::Passed)
            .count();

        let total_credits: u64 = self.credits.values().sum();

        QuadraticGovernanceStats {
            total_proposals: self.proposals.len(),
            active_proposals: active,
            passed_proposals: passed,
            total_voters: self.credits.len(),
            total_credits_available: total_credits,
        }
    }
}

/// Auszählungsergebnis eines Proposals
#[derive(Debug, Clone)]
pub struct ProposalTally {
    pub proposal_id: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub votes_abstain: u64,
    pub credits_spent: u64,
    pub unique_voters: usize,
    pub approval_ratio: f64,
    pub quorum_reached: bool,
    pub passed: bool,
}

/// Statistiken der QuadraticGovernance
#[derive(Debug, Clone)]
pub struct QuadraticGovernanceStats {
    pub total_proposals: usize,
    pub active_proposals: usize,
    pub passed_proposals: usize,
    pub total_voters: usize,
    pub total_credits_available: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vote_cost() {
        assert_eq!(QuadraticGovernance::vote_cost(1), 1);
        assert_eq!(QuadraticGovernance::vote_cost(2), 4);
        assert_eq!(QuadraticGovernance::vote_cost(3), 9);
        assert_eq!(QuadraticGovernance::vote_cost(10), 100);
    }

    #[test]
    fn test_max_votes_for_credits() {
        assert_eq!(QuadraticGovernance::max_votes_for_credits(100), 10);
        assert_eq!(QuadraticGovernance::max_votes_for_credits(9), 3);
        assert_eq!(QuadraticGovernance::max_votes_for_credits(8), 2);
    }

    #[test]
    fn test_voting() {
        let mut gov = QuadraticGovernance::default();

        let alice = DID::new_self(b"alice");
        let bob = DID::new_self(b"bob");

        gov.register_voter(alice.clone());
        gov.register_voter(bob.clone());

        let proposal = gov.create_proposal(
            "prop1".to_string(),
            "Test Proposal".to_string(),
            "Description".to_string(),
            alice.clone(),
        );

        // Alice: 5 Stimmen = 25 Credits
        gov.vote(&proposal.id, alice.clone(), VoteDirection::For, 5)
            .unwrap();

        // Bob: 3 Stimmen = 9 Credits
        gov.vote(&proposal.id, bob.clone(), VoteDirection::Against, 3)
            .unwrap();

        let tally = gov.tally(&proposal.id).unwrap();

        assert_eq!(tally.votes_for, 5);
        assert_eq!(tally.votes_against, 3);
        assert_eq!(tally.credits_spent, 34); // 25 + 9
        assert_eq!(tally.unique_voters, 2);
    }

    #[test]
    fn test_insufficient_credits() {
        let mut gov = QuadraticGovernance::default();

        let alice = DID::new_self(b"alice");
        gov.register_voter(alice.clone());

        let proposal = gov.create_proposal(
            "prop1".to_string(),
            "Test".to_string(),
            "Desc".to_string(),
            alice.clone(),
        );

        // Alice hat 100 Credits, versucht 11 Stimmen (121 Credits)
        let result = gov.vote(&proposal.id, alice, VoteDirection::For, 11);

        assert!(matches!(
            result,
            Err(GovernanceError::InsufficientCredits { .. })
        ));
    }

    #[test]
    fn test_quadratic_advantage() {
        // Whale vs. viele kleine Voter
        let mut gov = QuadraticGovernance::default();

        let whale = DID::new_self(b"whale");
        gov.register_voter(whale.clone());
        gov.add_credits(&whale, 900); // 900 + 100 (initial) = 1000 total

        // 10 kleine Voter mit je 100 Credits
        for i in 0..10 {
            let did = DID::new_self(format!("small{}", i).as_bytes());
            gov.register_voter(did.clone());
            // Jeder kleine Voter bekommt 100 Credits (100 initial + 0 extra)
            // Also 100 Credits = 10 Stimmen möglich
        }

        let proposal = gov.create_proposal(
            "prop1".to_string(),
            "Test".to_string(),
            "Desc".to_string(),
            whale.clone(),
        );

        // Whale: √1000 ≈ 31 Stimmen für 961 Credits (31² = 961)
        gov.vote(&proposal.id, whale, VoteDirection::For, 31)
            .unwrap();

        // Jeder kleine Voter: √100 = 10 Stimmen für 100 Credits
        for i in 0..10 {
            let did = DID::new_self(format!("small{}", i).as_bytes());
            gov.vote(&proposal.id, did, VoteDirection::Against, 10)
                .unwrap();
        }

        let tally = gov.tally(&proposal.id).unwrap();

        // 31 für, 100 gegen
        assert_eq!(tally.votes_for, 31);
        assert_eq!(tally.votes_against, 100);
        assert!(!tally.passed); // Die kleinen gewinnen!
    }
}
