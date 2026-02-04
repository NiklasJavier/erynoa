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
//!
//! ## UniversalId-Integration (v0.4.0)
//!
//! Jeder `PeerTrustInfo` hat nun eine `universal_id`, die konsistente
//! Identifikation über alle Erynoa-Subsysteme ermöglicht.
//! Der `TrustGate` unterstützt nun Lookups sowohl über `PeerId` als auch `UniversalId`.
//!
//! ## StateEvent-Integration (v0.4.0)
//!
//! Der TrustGate emittiert nun StateEvents für:
//! - Trust-Updates (TrustUpdated)
//! - Peer-Banning/Unbanning (PeerBanned/PeerUnbanned)
//! - Verbindungs-Entscheidungen (ConnectionDecision)

use crate::core::state::{StateEvent, StateEventEmitter, NoOpEmitter};
use crate::domain::UniversalId;
use crate::peer::p2p::config::TrustGateConfig;
use crate::peer::p2p::identity::SignedPeerInfo;
use anyhow::{anyhow, Result};
use libp2p::PeerId;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Trust-Gate für Verbindungssteuerung
///
/// Unterstützt nun Lookups sowohl über `PeerId` als auch `UniversalId`.
/// Emittiert StateEvents für Trust-Änderungen und Banning (v0.4.0).
pub struct TrustGate {
    /// Konfiguration
    config: TrustGateConfig,

    /// Bekannte Peers (PeerId → PeerTrustInfo)
    known_peers: RwLock<HashMap<PeerId, PeerTrustInfo>>,

    /// Reverse-Lookup: UniversalId → PeerId
    ///
    /// Ermöglicht schnelle Lookups über UniversalId für Integration
    /// mit StateEvents und anderen Subsystemen.
    universal_id_to_peer: RwLock<HashMap<UniversalId, PeerId>>,

    /// Gebannte Peers (PeerId → Ban-Ende)
    banned_peers: RwLock<HashMap<PeerId, Instant>>,

    /// Gebannte UniversalIds (für persistente Bans)
    banned_universal_ids: RwLock<HashMap<UniversalId, Instant>>,

    /// Verbindungs-Statistiken
    stats: RwLock<ConnectionStats>,

    // ========================================================================
    // StateEvent-Integration (v0.4.0)
    // ========================================================================
    /// StateEvent-Emitter für Integration mit UnifiedState
    state_event_emitter: Arc<dyn StateEventEmitter>,

    /// Counter: Trust-Updates
    trust_updates_count: AtomicU64,
    /// Counter: Bans
    bans_count: AtomicU64,
    /// Counter: Verbindungs-Entscheidungen
    connection_decisions_count: AtomicU64,
}

/// Trust-Info eines Peers
///
/// Enthält nun `universal_id` für systemweite Identifikation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerTrustInfo {
    /// UniversalId des Peers (Primärer Identifier)
    ///
    /// Content-addressed 32-byte Identifier aus der DID abgeleitet.
    /// Ermöglicht konsistente Lookups über alle Erynoa-Subsysteme.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub universal_id: Option<UniversalId>,

    /// DID des Peers (URI-Format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did: Option<String>,

    /// Trust-R (Reliability) - 0.0 bis 1.0
    pub trust_r: f64,

    /// Trust-Ω (Influence) - 0.0 bis unbegrenzt
    pub trust_omega: f64,

    /// Wann der Peer zuletzt gesehen wurde (Unix-Timestamp)
    pub last_seen: u64,

    /// Anzahl erfolgreicher Interaktionen
    pub successful_interactions: u64,

    /// Anzahl fehlgeschlagener Interaktionen
    pub failed_interactions: u64,

    /// Ist Newcomer (noch in Grace-Period)
    pub is_newcomer: bool,

    /// Newcomer-Start (Unix-Timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newcomer_since: Option<u64>,

    /// Connection-Level basierend auf Trust
    pub connection_level: ConnectionLevel,
}

impl PeerTrustInfo {
    /// Erstelle neue PeerTrustInfo mit UniversalId
    pub fn new_with_universal_id(universal_id: UniversalId, did: Option<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            universal_id: Some(universal_id),
            did,
            trust_r: 0.1, // Initial low trust
            trust_omega: 0.0,
            last_seen: now,
            successful_interactions: 0,
            failed_interactions: 0,
            is_newcomer: true,
            newcomer_since: Some(now),
            connection_level: ConnectionLevel::Limited,
        }
    }

    /// Hat diese Info eine gültige UniversalId?
    pub fn has_universal_id(&self) -> bool {
        self.universal_id.is_some()
    }

    /// Berechne kombinierte Trust-Score
    ///
    /// Kombiniert Trust-R und Trust-Ω zu einem einzelnen Wert.
    pub fn combined_trust_score(&self) -> f64 {
        // R ist wichtiger (70%), Ω für Einfluss (30%)
        self.trust_r * 0.7 + (self.trust_omega / (1.0 + self.trust_omega)) * 0.3
    }
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
            universal_id_to_peer: RwLock::new(HashMap::new()),
            banned_peers: RwLock::new(HashMap::new()),
            banned_universal_ids: RwLock::new(HashMap::new()),
            stats: RwLock::new(ConnectionStats::default()),
            state_event_emitter: Arc::new(NoOpEmitter),
            trust_updates_count: AtomicU64::new(0),
            bans_count: AtomicU64::new(0),
            connection_decisions_count: AtomicU64::new(0),
        }
    }

    /// Erstelle als Arc
    pub fn new_arc(config: TrustGateConfig) -> Arc<Self> {
        Arc::new(Self::new(config))
    }

    /// Erstelle mit StateEventEmitter (v0.4.0)
    pub fn new_with_emitter(config: TrustGateConfig, emitter: Arc<dyn StateEventEmitter>) -> Self {
        let mut gate = Self::new(config);
        gate.state_event_emitter = emitter;
        gate
    }

    /// Setze StateEventEmitter nachträglich
    pub fn set_state_event_emitter(&mut self, emitter: Arc<dyn StateEventEmitter>) {
        self.state_event_emitter = emitter;
    }

    /// Statistiken: Trust-Updates, Bans, Connection-Decisions (v0.4.0)
    pub fn event_counts(&self) -> (u64, u64, u64) {
        (
            self.trust_updates_count.load(Ordering::Relaxed),
            self.bans_count.load(Ordering::Relaxed),
            self.connection_decisions_count.load(Ordering::Relaxed),
        )
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
    ///
    /// Extrahiert und speichert die UniversalId für konsistente Lookups.
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

        // Parse UniversalId aus SignedPeerInfo
        let universal_id = signed_info.parse_universal_id().ok();

        // Prüfe ob UniversalId gebannt ist
        if let Some(ref uid) = universal_id {
            if let Some(ban_end) = self.banned_universal_ids.read().get(uid) {
                if Instant::now() < *ban_end {
                    return Err(anyhow!("UniversalId is banned"));
                }
            }
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let info = PeerTrustInfo {
            universal_id: universal_id.clone(),
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

        // Speichere in beiden Maps
        self.known_peers.write().insert(peer_id, info);

        // Reverse-Lookup registrieren
        if let Some(uid) = universal_id {
            self.universal_id_to_peer.write().insert(uid.clone(), peer_id);
            tracing::info!(
                peer_id = %peer_id,
                universal_id = %uid.to_hex(),
                did = %signed_info.did,
                "Registered new peer with UniversalId"
            );
        } else {
            tracing::info!(peer_id = %peer_id, did = %signed_info.did, "Registered new peer (no UniversalId)");
        }

        self.stats.write().newcomer_connections += 1;
        Ok(())
    }

    /// Registriere Peer direkt mit UniversalId
    ///
    /// Für programmatische Registrierung ohne SignedPeerInfo.
    pub fn register_peer_with_universal_id(
        &self,
        peer_id: PeerId,
        universal_id: UniversalId,
        did: Option<String>,
    ) -> Result<()> {
        // Prüfe ob UniversalId gebannt ist
        if let Some(ban_end) = self.banned_universal_ids.read().get(&universal_id) {
            if Instant::now() < *ban_end {
                return Err(anyhow!("UniversalId is banned"));
            }
        }

        let info = PeerTrustInfo::new_with_universal_id(universal_id.clone(), did);

        self.known_peers.write().insert(peer_id, info);
        self.universal_id_to_peer.write().insert(universal_id.clone(), peer_id);
        self.stats.write().newcomer_connections += 1;

        tracing::info!(
            peer_id = %peer_id,
            universal_id = %universal_id.to_hex(),
            "Registered peer with UniversalId"
        );

        Ok(())
    }

    /// Update Trust für Peer (v0.4.0: Mit StateEvent)
    pub fn update_trust(&self, peer_id: &PeerId, trust_r: f64, trust_omega: f64) {
        self.update_trust_with_reason(peer_id, trust_r, trust_omega, None);
    }

    /// Update Trust für Peer mit Grund (v0.4.0)
    pub fn update_trust_with_reason(
        &self,
        peer_id: &PeerId,
        trust_r: f64,
        trust_omega: f64,
        reason: Option<&str>,
    ) {
        let (universal_id, old_trust_r, old_trust_omega, new_level) = {
            let mut peers = self.known_peers.write();
            if let Some(info) = peers.get_mut(peer_id) {
                let old_r = info.trust_r;
                let old_omega = info.trust_omega;

                info.trust_r = trust_r;
                info.trust_omega = trust_omega;
                info.connection_level = self.trust_to_level(trust_r, trust_omega);

                // Newcomer-Status aufheben wenn Trust hoch genug
                if info.is_newcomer && trust_r >= 0.5 {
                    info.is_newcomer = false;
                    info.newcomer_since = None;
                }

                (info.universal_id, old_r, old_omega, info.connection_level)
            } else {
                return;
            }
        };

        // StateEvent emittieren (v0.4.0)
        self.trust_updates_count.fetch_add(1, Ordering::Relaxed);
        self.state_event_emitter.emit(StateEvent::TrustUpdated {
            peer_id: peer_id.to_string(),
            peer_universal_id: universal_id,
            old_trust_r,
            old_trust_omega,
            new_trust_r: trust_r,
            new_trust_omega: trust_omega,
            reason: reason.map(|s| s.to_string()),
            new_level: format!("{:?}", new_level),
        });
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
            self.ban_peer_with_reason(
                peer_id,
                Duration::from_secs(300), // 5 Minuten
                "critical_failure",
            );
        }
    }

    /// Banne Peer temporär (v0.4.0: Mit StateEvent)
    pub fn ban_peer(&self, peer_id: &PeerId, duration: Duration) {
        self.ban_peer_with_reason(peer_id, duration, "unspecified");
    }

    /// Banne Peer mit Grund (v0.4.0)
    pub fn ban_peer_with_reason(&self, peer_id: &PeerId, duration: Duration, reason: &str) {
        let ban_end = Instant::now() + duration;
        self.banned_peers.write().insert(*peer_id, ban_end);
        self.stats.write().active_bans += 1;

        // Hole UniversalId falls vorhanden
        let universal_id = self.known_peers.read()
            .get(peer_id)
            .and_then(|info| info.universal_id);

        // StateEvent emittieren (v0.4.0)
        self.bans_count.fetch_add(1, Ordering::Relaxed);
        self.state_event_emitter.emit(StateEvent::PeerBanned {
            peer_id: peer_id.to_string(),
            peer_universal_id: universal_id,
            duration_secs: duration.as_secs(),
            reason: reason.to_string(),
        });

        tracing::warn!(
            peer_id = %peer_id,
            duration = ?duration,
            reason = reason,
            "Peer banned"
        );
    }

    /// Entbanne Peer (v0.4.0: Mit StateEvent)
    pub fn unban_peer(&self, peer_id: &PeerId) {
        self.unban_peer_internal(peer_id, true);
    }

    /// Entbanne Peer mit manuell/automatisch Flag
    fn unban_peer_internal(&self, peer_id: &PeerId, manual: bool) {
        if self.banned_peers.write().remove(peer_id).is_some() {
            let mut stats = self.stats.write();
            if stats.active_bans > 0 {
                stats.active_bans -= 1;
            }
            drop(stats);

            // Hole UniversalId falls vorhanden
            let universal_id = self.known_peers.read()
                .get(peer_id)
                .and_then(|info| info.universal_id);

            // StateEvent emittieren (v0.4.0)
            self.state_event_emitter.emit(StateEvent::PeerUnbanned {
                peer_id: peer_id.to_string(),
                peer_universal_id: universal_id,
                manual,
            });

            tracing::info!(
                peer_id = %peer_id,
                manual = manual,
                "Peer unbanned"
            );
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

    /// Erhalte Peer-Info über PeerId
    pub fn get_peer_info(&self, peer_id: &PeerId) -> Option<PeerTrustInfo> {
        self.known_peers.read().get(peer_id).cloned()
    }

    /// Erhalte Peer-Info über UniversalId
    ///
    /// Nutzt den Reverse-Lookup Index für schnellen Zugriff.
    pub fn get_peer_info_by_universal_id(&self, universal_id: &UniversalId) -> Option<PeerTrustInfo> {
        let peer_id = self.universal_id_to_peer.read().get(universal_id).copied()?;
        self.known_peers.read().get(&peer_id).cloned()
    }

    /// Erhalte PeerId für eine UniversalId
    pub fn get_peer_id_by_universal_id(&self, universal_id: &UniversalId) -> Option<PeerId> {
        self.universal_id_to_peer.read().get(universal_id).copied()
    }

    /// Erhalte UniversalId für eine PeerId
    pub fn get_universal_id_by_peer_id(&self, peer_id: &PeerId) -> Option<UniversalId> {
        self.known_peers
            .read()
            .get(peer_id)
            .and_then(|info| info.universal_id.clone())
    }

    /// Prüfe Verbindung für UniversalId
    pub fn check_connection_by_universal_id(&self, universal_id: &UniversalId) -> ConnectionDecision {
        // Prüfe Ban-Status der UniversalId
        if let Some(ban_end) = self.banned_universal_ids.read().get(universal_id) {
            if Instant::now() < *ban_end {
                return ConnectionDecision {
                    allowed: false,
                    level: ConnectionLevel::Blocked,
                    reason: Some("UniversalId is banned".to_string()),
                    ban_duration: None,
                };
            }
        }

        // Lookup PeerId und delegate
        if let Some(peer_id) = self.get_peer_id_by_universal_id(universal_id) {
            return self.check_connection(&peer_id);
        }

        // Unbekannte UniversalId
        if self.config.reject_unknown_peers {
            ConnectionDecision {
                allowed: false,
                level: ConnectionLevel::Blocked,
                reason: Some("Unknown UniversalId, peers are rejected".to_string()),
                ban_duration: None,
            }
        } else {
            ConnectionDecision {
                allowed: true,
                level: ConnectionLevel::Limited,
                reason: Some("Unknown UniversalId, limited access".to_string()),
                ban_duration: None,
            }
        }
    }

    /// Banne UniversalId (persistenter als PeerId-Ban)
    ///
    /// Blockt alle PeerIds die diese UniversalId nutzen.
    pub fn ban_universal_id(&self, universal_id: &UniversalId, duration: Duration) {
        let ban_end = Instant::now() + duration;
        self.banned_universal_ids.write().insert(universal_id.clone(), ban_end);

        // Auch zugehörige PeerId bannen falls bekannt
        if let Some(peer_id) = self.get_peer_id_by_universal_id(universal_id) {
            self.banned_peers.write().insert(peer_id, ban_end);
        }

        self.stats.write().active_bans += 1;
        tracing::warn!(
            universal_id = %universal_id.to_hex(),
            duration = ?duration,
            "UniversalId banned"
        );
    }

    /// Entbanne UniversalId
    pub fn unban_universal_id(&self, universal_id: &UniversalId) {
        let removed_uid = self.banned_universal_ids.write().remove(universal_id).is_some();

        // Auch zugehörige PeerId entbannen
        if let Some(peer_id) = self.get_peer_id_by_universal_id(universal_id) {
            self.banned_peers.write().remove(&peer_id);
        }

        if removed_uid {
            let mut stats = self.stats.write();
            if stats.active_bans > 0 {
                stats.active_bans -= 1;
            }
        }
    }

    /// Ist UniversalId gebannt?
    pub fn is_universal_id_banned(&self, universal_id: &UniversalId) -> bool {
        if let Some(ban_end) = self.banned_universal_ids.read().get(universal_id) {
            Instant::now() < *ban_end
        } else {
            false
        }
    }

    /// Anzahl bekannter Peers
    pub fn known_peer_count(&self) -> usize {
        self.known_peers.read().len()
    }

    /// Anzahl Peers mit UniversalId
    pub fn peers_with_universal_id_count(&self) -> usize {
        self.universal_id_to_peer.read().len()
    }

    /// Statistiken
    pub fn stats(&self) -> ConnectionStats {
        self.stats.read().clone()
    }

    /// Aufräumen: Entferne abgelaufene Bans
    pub fn cleanup(&self) {
        let now = Instant::now();

        // PeerId-Bans
        let mut banned_peers = self.banned_peers.write();
        let before_peers = banned_peers.len();
        banned_peers.retain(|_, ban_end| now < *ban_end);
        let removed_peers = before_peers - banned_peers.len();
        drop(banned_peers);

        // UniversalId-Bans
        let mut banned_uids = self.banned_universal_ids.write();
        let before_uids = banned_uids.len();
        banned_uids.retain(|_, ban_end| now < *ban_end);
        let removed_uids = before_uids - banned_uids.len();
        drop(banned_uids);

        let total_removed = removed_peers.max(removed_uids);
        if total_removed > 0 {
            let mut stats = self.stats.write();
            stats.active_bans = stats.active_bans.saturating_sub(total_removed as u64);
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

    /// Alle UniversalIds mit mindestens diesem Level
    pub fn universal_ids_with_level(&self, min_level: ConnectionLevel) -> Vec<UniversalId> {
        self.known_peers
            .read()
            .values()
            .filter(|info| info.connection_level as u8 >= min_level as u8)
            .filter_map(|info| info.universal_id.clone())
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

    fn test_universal_id() -> UniversalId {
        // Erstelle Test-UniversalId
        UniversalId::new(UniversalId::TAG_DID, 1, b"test-peer-identity-data")
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

        // Registriere als Newcomer mit UniversalId
        let universal_id = test_universal_id();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let info = PeerTrustInfo {
            universal_id: Some(universal_id.clone()),
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
        gate.universal_id_to_peer.write().insert(universal_id.clone(), peer_id);

        // Update Trust
        gate.update_trust(&peer_id, 0.8, 1.5);

        let updated = gate.get_peer_info(&peer_id).unwrap();
        assert_eq!(updated.trust_r, 0.8);
        assert_eq!(updated.trust_omega, 1.5);
        assert!(!updated.is_newcomer);
        assert_eq!(updated.connection_level, ConnectionLevel::Full);

        // UniversalId-Lookup sollte auch funktionieren
        let by_uid = gate.get_peer_info_by_universal_id(&universal_id).unwrap();
        assert_eq!(by_uid.trust_r, 0.8);
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
            universal_id: Some(test_universal_id()),
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

    #[test]
    fn test_register_peer_with_universal_id() {
        let gate = TrustGate::new(test_config());
        let peer_id = PeerId::random();
        let universal_id = test_universal_id();

        gate.register_peer_with_universal_id(
            peer_id,
            universal_id.clone(),
            Some("did:erynoa:self:test".to_string()),
        )
        .unwrap();

        // Verifiziere Registrierung
        assert_eq!(gate.known_peer_count(), 1);
        assert_eq!(gate.peers_with_universal_id_count(), 1);

        // Lookup über PeerId
        let info = gate.get_peer_info(&peer_id).unwrap();
        assert!(info.universal_id.is_some());
        assert_eq!(info.universal_id.unwrap(), universal_id);

        // Lookup über UniversalId
        let by_uid = gate.get_peer_info_by_universal_id(&universal_id).unwrap();
        assert_eq!(by_uid.did, Some("did:erynoa:self:test".to_string()));
    }

    #[test]
    fn test_ban_universal_id() {
        let gate = TrustGate::new(test_config());
        let peer_id = PeerId::random();
        let universal_id = test_universal_id();

        // Registriere Peer
        gate.register_peer_with_universal_id(
            peer_id,
            universal_id.clone(),
            None,
        )
        .unwrap();

        // Ban via UniversalId
        gate.ban_universal_id(&universal_id, Duration::from_secs(60));

        assert!(gate.is_universal_id_banned(&universal_id));
        assert!(gate.is_banned(&peer_id)); // PeerId sollte auch gebannt sein

        // Neue Registrierung mit gleicher UniversalId sollte fehlschlagen
        let new_peer_id = PeerId::random();
        let result = gate.register_peer_with_universal_id(
            new_peer_id,
            universal_id.clone(),
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_check_connection_by_universal_id() {
        let gate = TrustGate::new(test_config());
        let universal_id = test_universal_id();

        // Unbekannte UniversalId (nicht registriert)
        let decision = gate.check_connection_by_universal_id(&universal_id);
        assert!(decision.allowed);
        assert_eq!(decision.level, ConnectionLevel::Limited);

        // Registriere und update Trust
        let peer_id = PeerId::random();
        gate.register_peer_with_universal_id(peer_id, universal_id.clone(), None).unwrap();
        gate.update_trust(&peer_id, 0.85, 1.2);

        // Jetzt sollte Full-Level sein
        let decision = gate.check_connection_by_universal_id(&universal_id);
        assert!(decision.allowed);
        assert_eq!(decision.level, ConnectionLevel::Full);
    }

    #[test]
    fn test_peer_trust_info_combined_score() {
        let info = PeerTrustInfo::new_with_universal_id(
            test_universal_id(),
            Some("did:erynoa:self:test".to_string()),
        );

        // Initial trust should give low combined score
        let initial_score = info.combined_trust_score();
        assert!(initial_score > 0.0 && initial_score < 0.5);

        // Higher trust values
        let mut high_trust = info.clone();
        high_trust.trust_r = 0.9;
        high_trust.trust_omega = 2.0;

        let high_score = high_trust.combined_trust_score();
        assert!(high_score > 0.7);
    }
}
