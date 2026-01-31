//! # Dezentrale Identifikatoren (DID)
//!
//! Implementierung gemäß Axiome Κ6-Κ8.
//!
//! ## Axiom-Referenz
//!
//! - **Κ6 (Existenz-Eindeutigkeit)**: `∀ entity e : ∃! did ∈ DID : identity(e) = did`
//! - **Κ7 (Permanenz)**: `⟨s⟩ ∧ ⟦create(s)⟧ ⟹ □⟨s⟩`
//! - **Κ8 (Delegations-Struktur)**: `s ⊳ s' → 𝕋(s') ≤ 𝕋(s)`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// DID Namespace gemäß Erynoa-Spezifikation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DIDNamespace {
    /// Natürliche Personen
    Self_,
    /// Organisationen, Firmen, DAOs
    Guild,
    /// KI-Agenten, autonome Systeme
    Spirit,
    /// IoT-Geräte, physische Assets
    Thing,
    /// Container, Transportmittel
    Vessel,
    /// Datenquellen, APIs
    Source,
    /// Dienstleistungen, Handwerke
    Craft,
    /// Speicher, Safes
    Vault,
    /// Verträge, Vereinbarungen
    Pact,
    /// Gruppen, Communities
    Circle,
}

impl fmt::Display for DIDNamespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DIDNamespace::Self_ => write!(f, "self"),
            DIDNamespace::Guild => write!(f, "guild"),
            DIDNamespace::Spirit => write!(f, "spirit"),
            DIDNamespace::Thing => write!(f, "thing"),
            DIDNamespace::Vessel => write!(f, "vessel"),
            DIDNamespace::Source => write!(f, "source"),
            DIDNamespace::Craft => write!(f, "craft"),
            DIDNamespace::Vault => write!(f, "vault"),
            DIDNamespace::Pact => write!(f, "pact"),
            DIDNamespace::Circle => write!(f, "circle"),
        }
    }
}

impl FromStr for DIDNamespace {
    type Err = DIDError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "self" => Ok(DIDNamespace::Self_),
            "guild" => Ok(DIDNamespace::Guild),
            "spirit" => Ok(DIDNamespace::Spirit),
            "thing" => Ok(DIDNamespace::Thing),
            "vessel" => Ok(DIDNamespace::Vessel),
            "source" => Ok(DIDNamespace::Source),
            "craft" => Ok(DIDNamespace::Craft),
            "vault" => Ok(DIDNamespace::Vault),
            "pact" => Ok(DIDNamespace::Pact),
            "circle" => Ok(DIDNamespace::Circle),
            _ => Err(DIDError::InvalidNamespace(s.to_string())),
        }
    }
}

/// Dezentraler Identifikator (DID)
///
/// Format: `did:erynoa:<namespace>:<unique-id>`
///
/// # Beispiel
/// ```
/// use erynoa_api::domain::DID;
///
/// let did = DID::new_self("alice123");
/// assert_eq!(did.to_string(), "did:erynoa:self:alice123");
/// ```
///
/// # Hinweis
/// PartialEq, Eq und Hash werden nur anhand von `namespace` und `unique_id` berechnet,
/// NICHT anhand von `created_at`. Das bedeutet, dass zwei DIDs mit demselben Namespace
/// und derselben unique_id als gleich gelten, auch wenn sie zu unterschiedlichen Zeiten erstellt wurden.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DID {
    /// Der Namespace dieser DID
    pub namespace: DIDNamespace,
    /// Eindeutige ID innerhalb des Namespace (typisch: Base58-encoded public key hash)
    pub unique_id: String,
    /// Zeitpunkt der Erstellung (Κ7: Permanenz)
    pub created_at: DateTime<Utc>,
}

impl PartialEq for DID {
    fn eq(&self, other: &Self) -> bool {
        self.namespace == other.namespace && self.unique_id == other.unique_id
    }
}

impl Eq for DID {}

impl std::hash::Hash for DID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.namespace.hash(state);
        self.unique_id.hash(state);
    }
}

impl DID {
    /// Erstelle neue DID mit gegebenem Namespace
    pub fn new(namespace: DIDNamespace, unique_id: impl Into<String>) -> Self {
        Self {
            namespace,
            unique_id: unique_id.into(),
            created_at: Utc::now(),
        }
    }

    /// Kurzform für `did:erynoa:self:<id>`
    pub fn new_self(unique_id: impl Into<String>) -> Self {
        Self::new(DIDNamespace::Self_, unique_id)
    }

    /// Kurzform für `did:erynoa:guild:<id>`
    pub fn new_guild(unique_id: impl Into<String>) -> Self {
        Self::new(DIDNamespace::Guild, unique_id)
    }

    /// Kurzform für `did:erynoa:spirit:<id>`
    pub fn new_spirit(unique_id: impl Into<String>) -> Self {
        Self::new(DIDNamespace::Spirit, unique_id)
    }

    /// Prüft ob diese DID eine menschliche Entität repräsentiert
    pub fn is_human_capable(&self) -> bool {
        matches!(self.namespace, DIDNamespace::Self_ | DIDNamespace::Guild)
    }

    /// Prüft ob diese DID ein KI-Agent ist
    pub fn is_agent(&self) -> bool {
        matches!(self.namespace, DIDNamespace::Spirit)
    }

    /// Gibt die vollständige DID-URI zurück
    pub fn to_uri(&self) -> String {
        format!("did:erynoa:{}:{}", self.namespace, self.unique_id)
    }
}

impl fmt::Display for DID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "did:erynoa:{}:{}", self.namespace, self.unique_id)
    }
}

impl FromStr for DID {
    type Err = DIDError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 4 {
            return Err(DIDError::InvalidFormat(s.to_string()));
        }
        if parts[0] != "did" || parts[1] != "erynoa" {
            return Err(DIDError::InvalidMethod(s.to_string()));
        }

        let namespace = DIDNamespace::from_str(parts[2])?;
        let unique_id = parts[3].to_string();

        Ok(Self {
            namespace,
            unique_id,
            created_at: Utc::now(), // Wird bei Deserialisierung überschrieben
        })
    }
}

/// Delegation zwischen DIDs (Κ8)
///
/// Κ8: `s ⊳ s' → 𝕋(s') ≤ 𝕋(s)` - Delegierter kann nie mehr Trust haben
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    /// Delegierender (Source)
    pub from: DID,
    /// Delegierter (Target)
    pub to: DID,
    /// Trust-Vererbungsfaktor (0.0-1.0)
    /// Gemäß Κ8: 𝕋(to) ≤ factor × 𝕋(from)
    pub trust_factor: f64,
    /// Erlaubte Capabilities
    pub capabilities: Vec<Capability>,
    /// Ablaufzeit (optional)
    pub expires_at: Option<DateTime<Utc>>,
    /// Ist diese Delegation widerrufbar?
    pub revocable: bool,
    /// Erstellungszeitpunkt
    pub created_at: DateTime<Utc>,
}

impl Delegation {
    /// Erstelle neue Delegation mit Default-Werten
    pub fn new(from: DID, to: DID, trust_factor: f64) -> Self {
        Self {
            from,
            to,
            trust_factor: trust_factor.clamp(0.0, 1.0),
            capabilities: vec![Capability::All],
            expires_at: None,
            revocable: true,
            created_at: Utc::now(),
        }
    }

    /// Prüft ob die Delegation noch gültig ist
    pub fn is_valid(&self) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() < expires,
            None => true,
        }
    }

    /// Berechnet den effektiven Trust-Faktor unter Berücksichtigung
    /// der Ketten-Dämpfung (Theorem Τ1)
    pub fn effective_trust_factor(&self, chain_length: usize) -> f64 {
        // Τ1: t_chain = exp(Σᵢ ln(tᵢ) / √n)
        // Für einzelne Delegation: ln(factor) / √1 = ln(factor)
        // Bei Ketten: Dämpfung durch √n
        let dampening = (chain_length as f64).sqrt();
        (self.trust_factor.ln() / dampening).exp()
    }
}

/// Capabilities die delegiert werden können
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Capability {
    /// Alle Capabilities
    All,
    /// Transfers durchführen
    Transfer,
    /// Attestationen erstellen
    Attest,
    /// Claims machen
    Claim,
    /// An Governance teilnehmen
    Governance,
    /// Weitere Delegationen erstellen
    Delegate,
}

/// Fehler bei DID-Operationen
#[derive(Debug, Clone, thiserror::Error)]
pub enum DIDError {
    #[error("Ungültiges DID-Format: {0}")]
    InvalidFormat(String),

    #[error("Ungültige DID-Methode (erwartet 'erynoa'): {0}")]
    InvalidMethod(String),

    #[error("Ungültiger Namespace: {0}")]
    InvalidNamespace(String),

    #[error("DID existiert nicht: {0}")]
    NotFound(String),

    #[error("Delegation verletzt Κ8 (Trust-Beschränkung)")]
    TrustViolation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_did_creation() {
        let did = DID::new_self("alice123");
        assert_eq!(did.namespace, DIDNamespace::Self_);
        assert_eq!(did.unique_id, "alice123");
        assert_eq!(did.to_string(), "did:erynoa:self:alice123");
    }

    #[test]
    fn test_did_parsing() {
        let did: DID = "did:erynoa:guild:mycompany".parse().unwrap();
        assert_eq!(did.namespace, DIDNamespace::Guild);
        assert_eq!(did.unique_id, "mycompany");
    }

    #[test]
    fn test_delegation_trust_factor() {
        let from = DID::new_self("alice");
        let to = DID::new_self("bob");
        let delegation = Delegation::new(from, to, 0.7);

        // Ketten-Dämpfung bei Länge 1
        assert!((delegation.effective_trust_factor(1) - 0.7).abs() < 0.001);

        // Ketten-Dämpfung bei Länge 4: exp(ln(0.7) / √4) = exp(ln(0.7) / 2) ≈ 0.837
        let factor_chain_4 = delegation.effective_trust_factor(4);
        assert!(factor_chain_4 > delegation.trust_factor);
    }
}
