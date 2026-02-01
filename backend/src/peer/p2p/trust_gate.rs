//! # Trust-Gate für Verbindungen
//!
//! Trust-basierte Verbindungssteuerung gemäß Κ23 (Gateway).
//!
//! ## Konzept
//!
//! - Eingehende Verbindungen werden gegen Trust-DB geprüft
//! - Niedrig-Trust-Peers: Limitierte Verbindung oder Ablehnung
//! - Hoch-Trust-Peers: Volle Verbindung + Relay-Privileges
//! - Anomaly-Integration: Verdächtige Peers werden temporär gebannt

use crate::peer::p2p::config::TrustGateConfig;
use crate::peer::p2p::identity::SignedPeerInfo;
use anyhow::{anyhow, Result};
use libp2p::PeerId;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Trust-Gate für Verbindungssteuerung
pub struct TrustGate {
    /// Konfiguration
    config: TrustGateConfig,

    /// Bekannte Peers (PeerId → PeerTrustInfo)
    known_peers: RwLock<HashMap<PeerId, PeerTrustInfo>>,

    /// Gebannte Peers (PeerId → Ban-Ende)
    banned_peers: RwLock<HashMap<PeerId, Instant>>,

    /// Verbindungs-Statistiken
    stats: RwLock<ConnectionStats>,
}

/// Trust-Info eines Peers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerTrustInfo {
    /// DID des Peers
    pub did: Option<String>,

    /// Trust-R (Reliability)
    pub trust_r: f64,

    /// Trust-Ω (Influence)
    pub trust_omega: f64,

    /// Wann der Peer zuletzt gesehen wurde
    pub last_seen: u64,

    /// Anzahl erfolgreicher Interaktionen
    pub successful_interactions: u64,

    /// Anzahl fehlgeschlagener Interaktionen
    pub failed_interactions: u64,

    /// Ist Newcomer (noch in Grace-Period)
    pub is_newcomer: bool,

    /// Newcomer-Start
    pub newcomer_since: Option<u64>,

    /// Connection-Level
    pub connection_level: ConnectionLevel,
}

/// Verbindungs-Level basierend auf Trust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionLevel {
    /// Keine Verbindung erlaubt
    Blocked,
    /// Limitierte Verbindung (nur lesen)
    Limited,
    /// Standard-Verbindung
    Standard,
    /// Volle Verbindung mit Relay-Privileges
    Full,
    /// Vertrauenswürdiger Peer (Bootstrap, Validator)
    Trusted,
}

impl ConnectionLevel {
    /// Kann dieser Level Events empfangen?
    pub fn can_receive_events(&self) -> bool {
        !matches!(self, ConnectionLevel::Blocked)
    }

    /// Kann dieser Level Events senden?
    pub fn can_send_events(&self) -> bool {
        matches!(
            self,
            ConnectionLevel::Standard | ConnectionLevel::Full | ConnectionLevel::Trusted
        )
    }

    /// Kann dieser Level als Relay fungieren?
    pub fn can_relay(&self) -> bool {
        matches!(self, ConnectionLevel::Full | ConnectionLevel::Trusted)
    }

    /// Kann dieser Level Sync-Requests stellen?
    pub fn can_sync(&self) -> bool {
        matches!(
            self,
            ConnectionLevel::Standard | ConnectionLevel::Full | ConnectionLevel::Trusted
        )
    }
}

/// Verbindungs-Statistiken
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    /// Gesamte Verbindungen
    pub total_connections: u64,
    /// Abgelehnte Verbindungen
    pub rejected_connections: u64,
    /// Aktive Bans
    pub active_bans: u64,
    /// Connections von Newcomern
    pub newcomer_connections: u64,
}

/// Entscheidung über Verbindungs-Anfrage
#[derive(Debug, Clone)]
pub struct ConnectionDecision {
    /// Erlaubt?
    pub allowed: bool,
    /// Verbindungs-Level
    pub level: ConnectionLevel,
    /// Grund (bei Ablehnung)
    pub reason: Option<String>,
    /// Wie lange bannen (bei Ablehnung mit Ban)
    pub ban_duration: Option<Duration>,
}

impl TrustGate {
    /// Erstelle neuen TrustGate
    pub fn new(config: TrustGateConfig) -> Self {
        Self {
            config,
            known_peers: RwLock::new(HashMap::new()),
            banned_peers: RwLock::new(HashMap::new()),
            stats: RwLock::new(ConnectionStats::default()),
        }
    }

    /// Erstelle als Arc
    pub fn new_arc(config: TrustGateConfig) -> Arc<Self> {
        Arc::new(Self::new(config))
    }

    /// Prüfe ob Verbindung erlaubt ist
    pub fn check_connection(&self, peer_id: &PeerId) -> ConnectionDecision {
        // Prüfe Ban-Status
        if let Some(ban_end) = self.banned_peers.read().get(peer_id) {
            if Instant::now() < *ban_end {
                return ConnectionDecision {
                    allowed: false,
                    level: ConnectionLevel::Blocked,
                    reason: Some("Peer is temporarily banned".to_string()),
                    ban_duration: None,
                };
            }
            // Ban abgelaufen, entferne
            self.banned_peers.write().remove(peer_id);
        }

        // Prüfe bekannten Peer
        if let Some(info) = self.known_peers.read().get(peer_id) {
            return self.decide_for_known_peer(info);
        }

        // Unbekannter Peer
        self.decide_for_unknown_peer(peer_id)
    }

    /// Entscheidung für bekannten Peer
    fn decide_for_known_peer(&self, info: &PeerTrustInfo) -> ConnectionDecision {
        // Newcomer in Grace-Period
        if info.is_newcomer {
            if let Some(since) = info.newcomer_since {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if now - since < self.config.newcomer_grace_period.as_secs() {
                    return ConnectionDecision {
                        allowed: true,
                        level: ConnectionLevel::Limited,
                        reason: Some("Newcomer in grace period".to_string()),
                        ban_duration: None,
                    };
                }
            }
        }

        // Trust-basierte Entscheidung
        let level = self.trust_to_level(info.trust_r, info.trust_omega);

        ConnectionDecision {
            allowed: level != ConnectionLevel::Blocked,
            level,
            reason: if level == ConnectionLevel::Blocked {
                Some(format!(
                    "Trust too low: R={:.2}, Ω={:.2}",
                    info.trust_r, info.trust_omega
                ))
            } else {
                None
            },
            ban_duration: None,
        }
    }

    /// Entscheidung für unbekannten Peer
    fn decide_for_unknown_peer(&self, _peer_id: &PeerId) -> ConnectionDecision {
        if self.config.reject_unknown_peers {
            return ConnectionDecision {
                allowed: false,
                level: ConnectionLevel::Blocked,
                reason: Some("Unknown peers are rejected".to_string()),
                ban_duration: None,
            };
        }

        // Newcomer-Modus
        ConnectionDecision {
            allowed: true,
            level: ConnectionLevel::Limited,
            reason: Some("Unknown peer, limited access".to_string()),
            ban_duration: None,
        }
    }

    /// Konvertiere Trust-Werte zu ConnectionLevel
    fn trust_to_level(&self, trust_r: f64, trust_omega: f64) -> ConnectionLevel {
        if trust_r < self.config.min_incoming_trust_r {
            return ConnectionLevel::Blocked;
        }

        if trust_r >= 0.9 && trust_omega >= 2.0 {
            ConnectionLevel::Trusted
        } else if trust_r >= 0.7 && trust_omega >= self.config.min_relay_trust_omega {
            ConnectionLevel::Full
        } else if trust_r >= 0.5 {
            ConnectionLevel::Standard
        } else {
            ConnectionLevel::Limited
        }
    }

    /// Registriere Peer mit Signed-Info
    pub fn register_peer(&self, peer_id: PeerId, signed_info: &SignedPeerInfo) -> Result<()> {
        // Verifiziere Signatur
        if !signed_info.verify()? {
            return Err(anyhow!("Invalid peer signature"));
        }

        // Prüfe Freshness
        let max_age = self.config.newcomer_grace_period.as_secs() * 2;
        if !signed_info.is_valid(max_age) {
            return Err(anyhow!("Peer info is too old"));
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let info = PeerTrustInfo {
            did: Some(signed_info.did.clone()),
            trust_r: 0.1, // Initial low trust
            trust_omega: 0.0,
            last_seen: now,
            successful_interactions: 0,
            failed_interactions: 0,
            is_newcomer: true,
            newcomer_since: Some(now),
            connection_level: ConnectionLevel::Limited,
        };

        self.known_peers.write().insert(peer_id, info);
        self.stats.write().newcomer_connections += 1;

        tracing::info!(peer_id = %peer_id, did = %signed_info.did, "Registered new peer");
        Ok(())
    }

    /// Update Trust für Peer
    pub fn update_trust(&self, peer_id: &PeerId, trust_r: f64, trust_omega: f64) {
        if let Some(info) = self.known_peers.write().get_mut(peer_id) {
            info.trust_r = trust_r;
            info.trust_omega = trust_omega;
            info.connection_level = self.trust_to_level(trust_r, trust_omega);

            // Newcomer-Status aufheben wenn Trust hoch genug
            if info.is_newcomer && trust_r >= 0.5 {
                info.is_newcomer = false;
                info.newcomer_since = None;
            }
        }
    }

    /// Melde erfolgreiche Interaktion
    pub fn report_success(&self, peer_id: &PeerId) {
        if let Some(info) = self.known_peers.write().get_mut(peer_id) {
            info.successful_interactions += 1;
            info.last_seen = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
    }

    /// Melde fehlgeschlagene Interaktion
    pub fn report_failure(&self, peer_id: &PeerId, severity: FailureSeverity) {
        let mut peers = self.known_peers.write();

        if let Some(info) = peers.get_mut(peer_id) {
            info.failed_interactions += 1;

            // Bei schweren Fehlern: Trust reduzieren
            match severity {
                FailureSeverity::Minor => {
                    // Nichts
                }
                FailureSeverity::Major => {
                    info.trust_r *= 0.9;
                }
                FailureSeverity::Critical => {
                    info.trust_r *= 0.5;
                }
            }

            info.connection_level = self.trust_to_level(info.trust_r, info.trust_omega);
        }

        // Bei kritischen Fehlern: Temporär bannen
        if matches!(severity, FailureSeverity::Critical) {
            drop(peers);
            self.ban_peer(peer_id, Duration::from_secs(300)); // 5 Minuten
        }
    }

    /// Banne Peer temporär
    pub fn ban_peer(&self, peer_id: &PeerId, duration: Duration) {
        let ban_end = Instant::now() + duration;
        self.banned_peers.write().insert(*peer_id, ban_end);
        self.stats.write().active_bans += 1;

        tracing::warn!(peer_id = %peer_id, duration = ?duration, "Peer banned");
    }

    /// Entbanne Peer
    pub fn unban_peer(&self, peer_id: &PeerId) {
        if self.banned_peers.write().remove(peer_id).is_some() {
            let mut stats = self.stats.write();
            if stats.active_bans > 0 {
                stats.active_bans -= 1;
            }
        }
    }

    /// Ist Peer gebannt?
    pub fn is_banned(&self, peer_id: &PeerId) -> bool {
        if let Some(ban_end) = self.banned_peers.read().get(peer_id) {
            Instant::now() < *ban_end
        } else {
            false
        }
    }

    /// Erhalte Peer-Info
    pub fn get_peer_info(&self, peer_id: &PeerId) -> Option<PeerTrustInfo> {
        self.known_peers.read().get(peer_id).cloned()
    }

    /// Anzahl bekannter Peers
    pub fn known_peer_count(&self) -> usize {
        self.known_peers.read().len()
    }

    /// Statistiken
    pub fn stats(&self) -> ConnectionStats {
        self.stats.read().clone()
    }

    /// Aufräumen: Entferne abgelaufene Bans
    pub fn cleanup(&self) {
        let now = Instant::now();
        let mut banned = self.banned_peers.write();
        let before = banned.len();
        banned.retain(|_, ban_end| now < *ban_end);
        let removed = before - banned.len();

        if removed > 0 {
            let mut stats = self.stats.write();
            stats.active_bans = stats.active_bans.saturating_sub(removed as u64);
        }
    }

    /// Alle Peers mit mindestens diesem Level
    pub fn peers_with_level(&self, min_level: ConnectionLevel) -> Vec<PeerId> {
        self.known_peers
            .read()
            .iter()
            .filter(|(_, info)| info.connection_level as u8 >= min_level as u8)
            .map(|(peer_id, _)| *peer_id)
            .collect()
    }
}

/// Schweregrad eines Fehlers
#[derive(Debug, Clone, Copy)]
pub enum FailureSeverity {
    /// Minor: Timeout, temporärer Fehler
    Minor,
    /// Major: Ungültige Daten, Protokoll-Verletzung
    Major,
    /// Critical: Malicious Verhalten, Spam, Angriff
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> TrustGateConfig {
        TrustGateConfig {
            min_incoming_trust_r: 0.1,
            min_relay_trust_omega: 0.5,
            trust_check_timeout: Duration::from_secs(5),
            reject_unknown_peers: false,
            newcomer_grace_period: Duration::from_secs(60),
        }
    }

    #[test]
    fn test_unknown_peer() {
        let gate = TrustGate::new(test_config());
        let peer_id = PeerId::random();

        let decision = gate.check_connection(&peer_id);
        assert!(decision.allowed);
        assert_eq!(decision.level, ConnectionLevel::Limited);
    }

    #[test]
    fn test_reject_unknown_peers() {
        let mut config = test_config();
        config.reject_unknown_peers = true;
        let gate = TrustGate::new(config);

        let peer_id = PeerId::random();
        let decision = gate.check_connection(&peer_id);

        assert!(!decision.allowed);
        assert_eq!(decision.level, ConnectionLevel::Blocked);
    }

    #[test]
    fn test_trust_levels() {
        let gate = TrustGate::new(test_config());

        // Blocked
        assert_eq!(gate.trust_to_level(0.05, 0.0), ConnectionLevel::Blocked);

        // Limited
        assert_eq!(gate.trust_to_level(0.3, 0.2), ConnectionLevel::Limited);

        // Standard
        assert_eq!(gate.trust_to_level(0.6, 0.3), ConnectionLevel::Standard);

        // Full
        assert_eq!(gate.trust_to_level(0.8, 1.0), ConnectionLevel::Full);

        // Trusted
        assert_eq!(gate.trust_to_level(0.95, 2.5), ConnectionLevel::Trusted);
    }

    #[test]
    fn test_ban_peer() {
        let gate = TrustGate::new(test_config());
        let peer_id = PeerId::random();

        assert!(!gate.is_banned(&peer_id));

        gate.ban_peer(&peer_id, Duration::from_secs(60));
        assert!(gate.is_banned(&peer_id));

        let decision = gate.check_connection(&peer_id);
        assert!(!decision.allowed);
    }

    #[test]
    fn test_update_trust() {
        let gate = TrustGate::new(test_config());
        let peer_id = PeerId::random();

        // Registriere als Newcomer
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let info = PeerTrustInfo {
            did: Some("did:erynoa:self:test".to_string()),
            trust_r: 0.1,
            trust_omega: 0.0,
            last_seen: now,
            successful_interactions: 0,
            failed_interactions: 0,
            is_newcomer: true,
            newcomer_since: Some(now),
            connection_level: ConnectionLevel::Limited,
        };

        gate.known_peers.write().insert(peer_id, info);

        // Update Trust
        gate.update_trust(&peer_id, 0.8, 1.5);

        let updated = gate.get_peer_info(&peer_id).unwrap();
        assert_eq!(updated.trust_r, 0.8);
        assert_eq!(updated.trust_omega, 1.5);
        assert!(!updated.is_newcomer);
        assert_eq!(updated.connection_level, ConnectionLevel::Full);
    }

    #[test]
    fn test_failure_reporting() {
        let gate = TrustGate::new(test_config());
        let peer_id = PeerId::random();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let info = PeerTrustInfo {
            did: Some("did:erynoa:self:test".to_string()),
            trust_r: 0.8,
            trust_omega: 1.0,
            last_seen: now,
            successful_interactions: 10,
            failed_interactions: 0,
            is_newcomer: false,
            newcomer_since: None,
            connection_level: ConnectionLevel::Full,
        };

        gate.known_peers.write().insert(peer_id, info);

        // Major failure
        gate.report_failure(&peer_id, FailureSeverity::Major);

        let updated = gate.get_peer_info(&peer_id).unwrap();
        assert!(updated.trust_r < 0.8);
        assert_eq!(updated.failed_interactions, 1);

        // Critical failure → Ban
        gate.report_failure(&peer_id, FailureSeverity::Critical);
        assert!(gate.is_banned(&peer_id));
    }
}
