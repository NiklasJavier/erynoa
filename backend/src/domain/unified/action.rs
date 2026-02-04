//! # Lifecycle-Actions und Metriken
//!
//! Definiert Lifecycle-Aktionen für Realms, Memberships, Blueprints und Netzwerk-Metriken.
//!
//! ## Typen
//!
//! - [`RealmAction`]: Realm-Lifecycle-Aktionen
//! - [`MembershipAction`]: Mitgliedschafts-Aktionen
//! - [`BlueprintAction`]: Blueprint-Lifecycle-Aktionen
//! - [`NetworkMetric`]: Netzwerk-Metrik-Typen
//!
//! ## Verwendung
//!
//! ```rust
//! use erynoa_api::domain::unified::action::{RealmAction, MembershipAction};
//!
//! let action = RealmAction::Created;
//! assert!(action.is_lifecycle_change());
//!
//! let membership = MembershipAction::Joined;
//! assert!(membership.affects_member_count());
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// RealmAction - Realm-Lifecycle-Aktionen
// ============================================================================

/// Realm-Lifecycle-Aktion
///
/// Beschreibt Aktionen, die den Zustand eines Realms verändern.
/// Verwendet für Event-Sourcing und Audit-Trails.
///
/// ## Lebenszyklus
///
/// ```text
/// Created → ConfigChanged* → Paused ↔ Resumed → Destroyed
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealmAction {
    /// Realm erstellt
    Created,
    /// Konfiguration geändert (MinTrust, Governance, etc.)
    ConfigChanged,
    /// Realm gelöscht/archiviert
    Destroyed,
    /// Realm pausiert (Admin-Aktion, temporär deaktiviert)
    Paused,
    /// Realm wiederhergestellt (nach Pause)
    Resumed,
}

impl RealmAction {
    /// Prüfe ob Aktion eine Lifecycle-Änderung ist (Created/Destroyed)
    #[inline]
    pub fn is_lifecycle_change(&self) -> bool {
        matches!(self, RealmAction::Created | RealmAction::Destroyed)
    }

    /// Prüfe ob Aktion den Realm deaktiviert
    #[inline]
    pub fn is_deactivating(&self) -> bool {
        matches!(self, RealmAction::Destroyed | RealmAction::Paused)
    }

    /// Prüfe ob Aktion den Realm aktiviert/reaktiviert
    #[inline]
    pub fn is_activating(&self) -> bool {
        matches!(self, RealmAction::Created | RealmAction::Resumed)
    }

    /// Prüfe ob Aktion reversibel ist
    #[inline]
    pub fn is_reversible(&self) -> bool {
        matches!(
            self,
            RealmAction::ConfigChanged | RealmAction::Paused | RealmAction::Resumed
        )
    }

    /// Human-readable Beschreibung
    pub fn description(&self) -> &'static str {
        match self {
            RealmAction::Created => "Realm created",
            RealmAction::ConfigChanged => "Realm configuration changed",
            RealmAction::Destroyed => "Realm destroyed/archived",
            RealmAction::Paused => "Realm paused (temporarily disabled)",
            RealmAction::Resumed => "Realm resumed (reactivated)",
        }
    }
}

impl std::fmt::Display for RealmAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RealmAction::Created => write!(f, "Created"),
            RealmAction::ConfigChanged => write!(f, "ConfigChanged"),
            RealmAction::Destroyed => write!(f, "Destroyed"),
            RealmAction::Paused => write!(f, "Paused"),
            RealmAction::Resumed => write!(f, "Resumed"),
        }
    }
}

// ============================================================================
// MembershipAction - Mitgliedschafts-Aktionen
// ============================================================================

/// Mitgliedschafts-Aktion
///
/// Beschreibt Aktionen, die die Mitgliedschaft in einem Realm verändern.
///
/// ## Axiom-Referenz
///
/// - **Κ1**: Realm-Hierarchie und Mitgliedschaftsvererbung
/// - **Κ6**: Identity-basierte Mitgliedschaft
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipAction {
    /// Mitglied beigetreten
    Joined,
    /// Mitglied ausgetreten
    Left,
    /// Mitglied gebannt
    Banned,
    /// Rolle geändert (z.B. Member → Admin)
    RoleChanged,
    /// Einladung gesendet
    Invited,
    /// Einladung angenommen
    InviteAccepted,
}

impl MembershipAction {
    /// Prüfe ob Aktion die Mitgliederzahl ändert
    #[inline]
    pub fn affects_member_count(&self) -> bool {
        matches!(
            self,
            MembershipAction::Joined
                | MembershipAction::Left
                | MembershipAction::Banned
                | MembershipAction::InviteAccepted
        )
    }

    /// Prüfe ob Aktion einen neuen Member hinzufügt
    #[inline]
    pub fn is_joining(&self) -> bool {
        matches!(
            self,
            MembershipAction::Joined | MembershipAction::InviteAccepted
        )
    }

    /// Prüfe ob Aktion einen Member entfernt
    #[inline]
    pub fn is_leaving(&self) -> bool {
        matches!(self, MembershipAction::Left | MembershipAction::Banned)
    }

    /// Prüfe ob Aktion administrative Rechte erfordert
    #[inline]
    pub fn requires_admin(&self) -> bool {
        matches!(
            self,
            MembershipAction::Banned | MembershipAction::RoleChanged | MembershipAction::Invited
        )
    }

    /// Prüfe ob Aktion negativ ist (Entfernung/Ban)
    #[inline]
    pub fn is_negative(&self) -> bool {
        matches!(self, MembershipAction::Left | MembershipAction::Banned)
    }

    /// Human-readable Beschreibung
    pub fn description(&self) -> &'static str {
        match self {
            MembershipAction::Joined => "Member joined realm",
            MembershipAction::Left => "Member left realm",
            MembershipAction::Banned => "Member banned from realm",
            MembershipAction::RoleChanged => "Member role changed",
            MembershipAction::Invited => "Member invited to realm",
            MembershipAction::InviteAccepted => "Invitation accepted",
        }
    }
}

impl std::fmt::Display for MembershipAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MembershipAction::Joined => write!(f, "Joined"),
            MembershipAction::Left => write!(f, "Left"),
            MembershipAction::Banned => write!(f, "Banned"),
            MembershipAction::RoleChanged => write!(f, "RoleChanged"),
            MembershipAction::Invited => write!(f, "Invited"),
            MembershipAction::InviteAccepted => write!(f, "InviteAccepted"),
        }
    }
}

// ============================================================================
// BlueprintAction - Blueprint-Lifecycle-Aktionen
// ============================================================================

/// Blueprint-Lifecycle-Aktion
///
/// Beschreibt Aktionen im Lebenszyklus eines ECL-Blueprints.
///
/// ## Lebenszyklus
///
/// ```text
/// Published → Verified? → Deployed → Instantiated*
///                           ↓
///                      Deprecated
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlueprintAction {
    /// Blueprint veröffentlicht (im Marketplace)
    Published,
    /// Blueprint deployed in einem Realm
    Deployed,
    /// Blueprint instanziiert (konkrete Instanz erstellt)
    Instantiated,
    /// Blueprint verifiziert (Community-Review, Audit)
    Verified,
    /// Blueprint deprecated (veraltet, nicht mehr empfohlen)
    Deprecated,
}

impl BlueprintAction {
    /// Prüfe ob Aktion Vertrauen signalisiert
    #[inline]
    pub fn is_trust_positive(&self) -> bool {
        matches!(
            self,
            BlueprintAction::Verified | BlueprintAction::Deployed | BlueprintAction::Instantiated
        )
    }

    /// Prüfe ob Aktion Veralterung signalisiert
    #[inline]
    pub fn is_deprecating(&self) -> bool {
        matches!(self, BlueprintAction::Deprecated)
    }

    /// Prüfe ob Aktion den Blueprint verfügbar macht
    #[inline]
    pub fn is_publishing(&self) -> bool {
        matches!(self, BlueprintAction::Published | BlueprintAction::Deployed)
    }

    /// Human-readable Beschreibung
    pub fn description(&self) -> &'static str {
        match self {
            BlueprintAction::Published => "Blueprint published to marketplace",
            BlueprintAction::Deployed => "Blueprint deployed to realm",
            BlueprintAction::Instantiated => "Blueprint instantiated",
            BlueprintAction::Verified => "Blueprint verified by community",
            BlueprintAction::Deprecated => "Blueprint marked as deprecated",
        }
    }
}

impl std::fmt::Display for BlueprintAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlueprintAction::Published => write!(f, "Published"),
            BlueprintAction::Deployed => write!(f, "Deployed"),
            BlueprintAction::Instantiated => write!(f, "Instantiated"),
            BlueprintAction::Verified => write!(f, "Verified"),
            BlueprintAction::Deprecated => write!(f, "Deprecated"),
        }
    }
}

// ============================================================================
// NetworkMetric - Netzwerk-Metrik-Typen
// ============================================================================

/// Netzwerk-Metrik-Typ
///
/// Identifiziert verschiedene Netzwerk-Metriken für Monitoring und Telemetrie.
///
/// ## Kategorien
///
/// - **Connectivity**: ConnectedPeers
/// - **Bandwidth**: BytesSent, BytesReceived
/// - **Latency**: LatencyAvg
/// - **Gossip**: GossipMessages, GossipMessagesReceived
/// - **DHT**: DHTLookups
/// - **Privacy**: PrivacyMessagesSent, PrivacyCircuitsCreated, CoverTrafficMessages
/// - **Transport**: TransportFallbacks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkMetric {
    /// Anzahl verbundener Peers
    ConnectedPeers,
    /// Gesendete Bytes (total)
    BytesSent,
    /// Empfangene Bytes (total)
    BytesReceived,
    /// Durchschnittliche Latenz in Millisekunden
    LatencyAvg,
    /// Gossip-Nachrichten propagiert (gesendet)
    GossipMessages,
    /// Gossip-Nachrichten empfangen
    GossipMessagesReceived,
    /// DHT-Lookups durchgeführt
    DHTLookups,
    /// Privacy-Nachrichten gesendet (Onion-Routing)
    PrivacyMessagesSent,
    /// Privacy-Circuits erstellt
    PrivacyCircuitsCreated,
    /// Cover-Traffic-Nachrichten (für Privacy)
    CoverTrafficMessages,
    /// Transport-Fallback-Events (QUIC → TCP)
    TransportFallbacks,
}

impl NetworkMetric {
    /// Prüfe ob Metrik Bandwidth-bezogen ist
    #[inline]
    pub fn is_bandwidth(&self) -> bool {
        matches!(self, NetworkMetric::BytesSent | NetworkMetric::BytesReceived)
    }

    /// Prüfe ob Metrik Privacy-bezogen ist
    #[inline]
    pub fn is_privacy(&self) -> bool {
        matches!(
            self,
            NetworkMetric::PrivacyMessagesSent
                | NetworkMetric::PrivacyCircuitsCreated
                | NetworkMetric::CoverTrafficMessages
        )
    }

    /// Prüfe ob Metrik Gossip-bezogen ist
    #[inline]
    pub fn is_gossip(&self) -> bool {
        matches!(
            self,
            NetworkMetric::GossipMessages | NetworkMetric::GossipMessagesReceived
        )
    }

    /// Prüfe ob Metrik ein Counter ist (monoton steigend)
    #[inline]
    pub fn is_counter(&self) -> bool {
        !matches!(
            self,
            NetworkMetric::ConnectedPeers | NetworkMetric::LatencyAvg
        )
    }

    /// Prüfe ob Metrik ein Gauge ist (kann steigen/fallen)
    #[inline]
    pub fn is_gauge(&self) -> bool {
        matches!(
            self,
            NetworkMetric::ConnectedPeers | NetworkMetric::LatencyAvg
        )
    }

    /// Einheit der Metrik
    pub fn unit(&self) -> &'static str {
        match self {
            NetworkMetric::ConnectedPeers => "peers",
            NetworkMetric::BytesSent | NetworkMetric::BytesReceived => "bytes",
            NetworkMetric::LatencyAvg => "ms",
            NetworkMetric::GossipMessages
            | NetworkMetric::GossipMessagesReceived
            | NetworkMetric::DHTLookups
            | NetworkMetric::PrivacyMessagesSent
            | NetworkMetric::PrivacyCircuitsCreated
            | NetworkMetric::CoverTrafficMessages
            | NetworkMetric::TransportFallbacks => "count",
        }
    }

    /// Human-readable Beschreibung
    pub fn description(&self) -> &'static str {
        match self {
            NetworkMetric::ConnectedPeers => "Number of connected peers",
            NetworkMetric::BytesSent => "Total bytes sent",
            NetworkMetric::BytesReceived => "Total bytes received",
            NetworkMetric::LatencyAvg => "Average latency in milliseconds",
            NetworkMetric::GossipMessages => "Gossip messages sent",
            NetworkMetric::GossipMessagesReceived => "Gossip messages received",
            NetworkMetric::DHTLookups => "DHT lookups performed",
            NetworkMetric::PrivacyMessagesSent => "Privacy messages sent (onion)",
            NetworkMetric::PrivacyCircuitsCreated => "Privacy circuits created",
            NetworkMetric::CoverTrafficMessages => "Cover traffic messages",
            NetworkMetric::TransportFallbacks => "Transport fallbacks (QUIC→TCP)",
        }
    }
}

impl std::fmt::Display for NetworkMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ============================================================================
// Compile-Time Size Checks
// ============================================================================

const _: () = {
    assert!(std::mem::size_of::<RealmAction>() == 1);
    assert!(std::mem::size_of::<MembershipAction>() == 1);
    assert!(std::mem::size_of::<BlueprintAction>() == 1);
    assert!(std::mem::size_of::<NetworkMetric>() == 1);
};

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // RealmAction Tests
    // ========================================================================

    #[test]
    fn test_realm_action_lifecycle() {
        assert!(RealmAction::Created.is_lifecycle_change());
        assert!(RealmAction::Destroyed.is_lifecycle_change());
        assert!(!RealmAction::ConfigChanged.is_lifecycle_change());
    }

    #[test]
    fn test_realm_action_activation() {
        assert!(RealmAction::Created.is_activating());
        assert!(RealmAction::Resumed.is_activating());
        assert!(!RealmAction::Paused.is_activating());

        assert!(RealmAction::Destroyed.is_deactivating());
        assert!(RealmAction::Paused.is_deactivating());
        assert!(!RealmAction::Created.is_deactivating());
    }

    #[test]
    fn test_realm_action_reversible() {
        assert!(RealmAction::Paused.is_reversible());
        assert!(!RealmAction::Destroyed.is_reversible());
    }

    // ========================================================================
    // MembershipAction Tests
    // ========================================================================

    #[test]
    fn test_membership_action_count() {
        assert!(MembershipAction::Joined.affects_member_count());
        assert!(MembershipAction::Left.affects_member_count());
        assert!(MembershipAction::Banned.affects_member_count());
        assert!(!MembershipAction::Invited.affects_member_count());
        assert!(!MembershipAction::RoleChanged.affects_member_count());
    }

    #[test]
    fn test_membership_action_joining_leaving() {
        assert!(MembershipAction::Joined.is_joining());
        assert!(MembershipAction::InviteAccepted.is_joining());
        assert!(!MembershipAction::Left.is_joining());

        assert!(MembershipAction::Left.is_leaving());
        assert!(MembershipAction::Banned.is_leaving());
        assert!(!MembershipAction::Joined.is_leaving());
    }

    #[test]
    fn test_membership_action_admin() {
        assert!(MembershipAction::Banned.requires_admin());
        assert!(MembershipAction::RoleChanged.requires_admin());
        assert!(MembershipAction::Invited.requires_admin());
        assert!(!MembershipAction::Joined.requires_admin());
    }

    // ========================================================================
    // BlueprintAction Tests
    // ========================================================================

    #[test]
    fn test_blueprint_action_trust() {
        assert!(BlueprintAction::Verified.is_trust_positive());
        assert!(BlueprintAction::Deployed.is_trust_positive());
        assert!(!BlueprintAction::Deprecated.is_trust_positive());
    }

    #[test]
    fn test_blueprint_action_publishing() {
        assert!(BlueprintAction::Published.is_publishing());
        assert!(BlueprintAction::Deployed.is_publishing());
        assert!(!BlueprintAction::Instantiated.is_publishing());
    }

    // ========================================================================
    // NetworkMetric Tests
    // ========================================================================

    #[test]
    fn test_network_metric_categories() {
        assert!(NetworkMetric::BytesSent.is_bandwidth());
        assert!(NetworkMetric::BytesReceived.is_bandwidth());
        assert!(!NetworkMetric::ConnectedPeers.is_bandwidth());

        assert!(NetworkMetric::PrivacyMessagesSent.is_privacy());
        assert!(NetworkMetric::PrivacyCircuitsCreated.is_privacy());
        assert!(!NetworkMetric::GossipMessages.is_privacy());

        assert!(NetworkMetric::GossipMessages.is_gossip());
        assert!(!NetworkMetric::DHTLookups.is_gossip());
    }

    #[test]
    fn test_network_metric_type() {
        assert!(NetworkMetric::ConnectedPeers.is_gauge());
        assert!(NetworkMetric::LatencyAvg.is_gauge());
        assert!(!NetworkMetric::BytesSent.is_gauge());

        assert!(NetworkMetric::BytesSent.is_counter());
        assert!(!NetworkMetric::ConnectedPeers.is_counter());
    }

    #[test]
    fn test_network_metric_units() {
        assert_eq!(NetworkMetric::ConnectedPeers.unit(), "peers");
        assert_eq!(NetworkMetric::BytesSent.unit(), "bytes");
        assert_eq!(NetworkMetric::LatencyAvg.unit(), "ms");
        assert_eq!(NetworkMetric::GossipMessages.unit(), "count");
    }

    // ========================================================================
    // Serde Tests
    // ========================================================================

    #[test]
    fn test_serde_roundtrip() {
        let action = RealmAction::ConfigChanged;
        let json = serde_json::to_string(&action).unwrap();
        let parsed: RealmAction = serde_json::from_str(&json).unwrap();
        assert_eq!(action, parsed);

        let membership = MembershipAction::Banned;
        let json = serde_json::to_string(&membership).unwrap();
        let parsed: MembershipAction = serde_json::from_str(&json).unwrap();
        assert_eq!(membership, parsed);

        let blueprint = BlueprintAction::Verified;
        let json = serde_json::to_string(&blueprint).unwrap();
        let parsed: BlueprintAction = serde_json::from_str(&json).unwrap();
        assert_eq!(blueprint, parsed);

        let metric = NetworkMetric::PrivacyCircuitsCreated;
        let json = serde_json::to_string(&metric).unwrap();
        let parsed: NetworkMetric = serde_json::from_str(&json).unwrap();
        assert_eq!(metric, parsed);
    }

    #[test]
    fn test_display_impls() {
        assert_eq!(format!("{}", RealmAction::Created), "Created");
        assert_eq!(format!("{}", MembershipAction::Joined), "Joined");
        assert_eq!(format!("{}", BlueprintAction::Published), "Published");
    }
}
