//! # Bridge Network (RL19)
//!
//! Unlisted Entry Points für zensierte Regionen.
//!
//! ## Bridge-Discovery-Methoden
//!
//! | Methode           | Sicherheit    | Use-Case                     |
//! |-------------------|---------------|------------------------------|
//! | MOAT              | Medium        | CAPTCHA-geschützt            |
//! | Email-Responder   | Medium-High   | Unique Bridge pro Email      |
//! | Social-Graph      | High          | Web-of-Trust Einladungen     |
//! | Physical-Exchange | Highest       | QR-Code bei physischem Treffen|
//!
//! ## Axiom-Referenz
//!
//! - **RL19**: Bridge-Relay Network, unlisted entry points
//! - **Κ20**: Resilience gegen Blocking

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use parking_lot::RwLock;
use thiserror::Error;

use super::pluggable_transports::{CensorshipLevel, TransportType};

// ============================================================================
// Constants
// ============================================================================

/// Standard Bridge-Rotation-Intervall
pub const BRIDGE_ROTATION_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60); // 24h

/// Maximum Burn-Reports bis Bridge als kompromittiert gilt
pub const BRIDGE_BURN_THRESHOLD: u32 = 3;

/// Bridge-Test-Timeout
pub const BRIDGE_TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Maximum Bridges pro Pool
pub const MAX_BRIDGES_PER_POOL: usize = 100;

// ============================================================================
// Bridge Types
// ============================================================================

/// Bridge-Discovery-Methode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiscoveryMethod {
    /// MOAT: CAPTCHA-geschützte Web-Anfrage
    Moat,
    /// Email-Responder: Bridge per Email anfordern
    EmailResponder,
    /// Social-Graph: Einladung über vertrauenswürdige Kontakte
    SocialGraph,
    /// Physical-Exchange: QR-Code bei physischem Treffen
    PhysicalExchange,
    /// DHT: Aus DHT (für Erynoa-eigene Bridges)
    Dht,
    /// Hardcoded: Vorkonfigurierte Backup-Bridges
    Hardcoded,
}

impl DiscoveryMethod {
    /// Sicherheitslevel der Discovery-Methode (höher = sicherer)
    pub fn security_level(&self) -> u8 {
        match self {
            DiscoveryMethod::Hardcoded => 1,
            DiscoveryMethod::Dht => 2,
            DiscoveryMethod::Moat => 3,
            DiscoveryMethod::EmailResponder => 4,
            DiscoveryMethod::SocialGraph => 5,
            DiscoveryMethod::PhysicalExchange => 6,
        }
    }
}

impl fmt::Display for DiscoveryMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscoveryMethod::Moat => write!(f, "moat"),
            DiscoveryMethod::EmailResponder => write!(f, "email"),
            DiscoveryMethod::SocialGraph => write!(f, "social"),
            DiscoveryMethod::PhysicalExchange => write!(f, "physical"),
            DiscoveryMethod::Dht => write!(f, "dht"),
            DiscoveryMethod::Hardcoded => write!(f, "hardcoded"),
        }
    }
}

/// Bridge-Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeStatus {
    /// Ungetestet
    Unknown,
    /// Funktioniert
    Active,
    /// Temporär nicht erreichbar
    Unreachable,
    /// Geblockt (Burn-Reports überschritten)
    Burned,
    /// Abgelaufen (Rotation fällig)
    Expired,
}

impl fmt::Display for BridgeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BridgeStatus::Unknown => write!(f, "unknown"),
            BridgeStatus::Active => write!(f, "active"),
            BridgeStatus::Unreachable => write!(f, "unreachable"),
            BridgeStatus::Burned => write!(f, "burned"),
            BridgeStatus::Expired => write!(f, "expired"),
        }
    }
}

// ============================================================================
// Bridge Info
// ============================================================================

/// Bridge-Information
#[derive(Debug, Clone)]
pub struct BridgeInfo {
    /// Eindeutige Bridge-ID
    pub id: [u8; 20],
    /// Adresse (IP:Port oder Domain)
    pub address: String,
    /// Fingerprint (SHA1 des Public Keys)
    pub fingerprint: [u8; 20],
    /// Transport-Typ
    pub transport: TransportType,
    /// Unterstützte Regionen
    pub supported_regions: Vec<String>,
    /// Discovery-Methode
    pub discovery_method: DiscoveryMethod,
    /// Hinzugefügt am (Unix-Timestamp)
    pub added_at: u64,
    /// Zuletzt getestet (Unix-Timestamp)
    pub last_tested: Option<u64>,
    /// Aktueller Status
    pub status: BridgeStatus,
    /// Burn-Report-Zähler
    pub burn_reports: u32,
    /// Transport-spezifische Optionen (z.B. obfs4 cert)
    pub options: HashMap<String, String>,
}

impl BridgeInfo {
    /// Erstellt neue Bridge-Info
    pub fn new(
        address: String,
        fingerprint: [u8; 20],
        transport: TransportType,
        discovery_method: DiscoveryMethod,
    ) -> Self {
        // ID = Hash aus Address + Fingerprint
        let mut id_input = address.as_bytes().to_vec();
        id_input.extend_from_slice(&fingerprint);
        let id = blake3_hash_20(&id_input);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            address,
            fingerprint,
            transport,
            supported_regions: Vec::new(),
            discovery_method,
            added_at: now,
            last_tested: None,
            status: BridgeStatus::Unknown,
            burn_reports: 0,
            options: HashMap::new(),
        }
    }

    /// Fügt Region hinzu
    pub fn with_region(mut self, region: &str) -> Self {
        self.supported_regions.push(region.to_uppercase());
        self
    }

    /// Fügt mehrere Regionen hinzu
    pub fn with_regions(mut self, regions: &[&str]) -> Self {
        for region in regions {
            self.supported_regions.push(region.to_uppercase());
        }
        self
    }

    /// Fügt Option hinzu (z.B. obfs4 cert)
    pub fn with_option(mut self, key: &str, value: &str) -> Self {
        self.options.insert(key.to_string(), value.to_string());
        self
    }

    /// Prüft ob Bridge für Region geeignet ist
    pub fn supports_region(&self, region: &str) -> bool {
        if self.supported_regions.is_empty() {
            return true; // Keine Einschränkung = überall
        }
        self.supported_regions
            .iter()
            .any(|r| r.eq_ignore_ascii_case(region))
    }

    /// Prüft ob Bridge verwendbar ist
    pub fn is_usable(&self) -> bool {
        matches!(self.status, BridgeStatus::Active | BridgeStatus::Unknown)
            && self.burn_reports < BRIDGE_BURN_THRESHOLD
    }

    /// Markiert als getestet
    pub fn mark_tested(&mut self, success: bool) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.last_tested = Some(now);
        self.status = if success {
            BridgeStatus::Active
        } else {
            BridgeStatus::Unreachable
        };
    }

    /// Fügt Burn-Report hinzu
    pub fn report_burned(&mut self) {
        self.burn_reports += 1;
        if self.burn_reports >= BRIDGE_BURN_THRESHOLD {
            self.status = BridgeStatus::Burned;
        }
    }

    /// Prüft ob Rotation fällig
    pub fn needs_rotation(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        now - self.added_at > BRIDGE_ROTATION_INTERVAL.as_secs()
    }

    /// Generiert Bridge-Line (für obfs4)
    pub fn to_bridge_line(&self) -> String {
        let mut line = format!(
            "{} {} {}",
            self.transport,
            self.address,
            hex::encode(self.fingerprint)
        );

        for (key, value) in &self.options {
            line.push_str(&format!(" {}={}", key, value));
        }

        line
    }
}

/// Vereinfachtes Blake3-Hash auf 20 Bytes
fn blake3_hash_20(data: &[u8]) -> [u8; 20] {
    let hash = blake3::hash(data);
    let mut result = [0u8; 20];
    result.copy_from_slice(&hash.as_bytes()[..20]);
    result
}

// ============================================================================
// Bridge Pool
// ============================================================================

/// Bridge-Pool mit automatischer Verwaltung
pub struct BridgePool {
    /// Bridges nach ID
    bridges: RwLock<HashMap<[u8; 20], BridgeInfo>>,
    /// Bridges nach Region
    by_region: RwLock<HashMap<String, HashSet<[u8; 20]>>>,
    /// Konfiguration
    config: BridgePoolConfig,
    /// Statistiken
    stats: RwLock<BridgePoolStats>,
}

/// Bridge-Pool-Konfiguration
#[derive(Debug, Clone)]
pub struct BridgePoolConfig {
    /// Maximale Bridges im Pool
    pub max_bridges: usize,
    /// Automatische Rotation aktivieren
    pub auto_rotation: bool,
    /// Rotation-Intervall
    pub rotation_interval: Duration,
    /// Burn-Threshold
    pub burn_threshold: u32,
    /// Bevorzugte Discovery-Methoden (in Reihenfolge)
    pub preferred_discovery: Vec<DiscoveryMethod>,
}

impl Default for BridgePoolConfig {
    fn default() -> Self {
        Self {
            max_bridges: MAX_BRIDGES_PER_POOL,
            auto_rotation: true,
            rotation_interval: BRIDGE_ROTATION_INTERVAL,
            burn_threshold: BRIDGE_BURN_THRESHOLD,
            preferred_discovery: vec![
                DiscoveryMethod::SocialGraph,
                DiscoveryMethod::EmailResponder,
                DiscoveryMethod::Moat,
                DiscoveryMethod::Dht,
            ],
        }
    }
}

/// Bridge-Pool-Statistiken
#[derive(Debug, Clone, Default)]
pub struct BridgePoolStats {
    /// Gesamtzahl Bridges
    pub total_bridges: usize,
    /// Aktive Bridges
    pub active_bridges: usize,
    /// Gebrannte Bridges
    pub burned_bridges: usize,
    /// Erfolgreich verwendete Bridges
    pub successful_uses: u64,
    /// Fehlgeschlagene Verwendungen
    pub failed_uses: u64,
}

impl BridgePool {
    /// Erstellt neuen Bridge-Pool
    pub fn new(config: BridgePoolConfig) -> Self {
        Self {
            bridges: RwLock::new(HashMap::new()),
            by_region: RwLock::new(HashMap::new()),
            config,
            stats: RwLock::new(BridgePoolStats::default()),
        }
    }

    /// Fügt Bridge hinzu
    pub fn add_bridge(&self, bridge: BridgeInfo) -> Result<(), BridgeError> {
        let mut bridges = self.bridges.write();

        if bridges.len() >= self.config.max_bridges {
            return Err(BridgeError::PoolFull);
        }

        let id = bridge.id;
        let regions = bridge.supported_regions.clone();

        bridges.insert(id, bridge);

        // Update Region-Index
        let mut by_region = self.by_region.write();
        for region in &regions {
            by_region
                .entry(region.clone())
                .or_insert_with(HashSet::new)
                .insert(id);
        }

        // Update Stats
        self.update_stats();

        Ok(())
    }

    /// Entfernt Bridge
    pub fn remove_bridge(&self, id: &[u8; 20]) -> Option<BridgeInfo> {
        let mut bridges = self.bridges.write();

        if let Some(bridge) = bridges.remove(id) {
            // Update Region-Index
            let mut by_region = self.by_region.write();
            for region in &bridge.supported_regions {
                if let Some(ids) = by_region.get_mut(region) {
                    ids.remove(id);
                }
            }

            self.update_stats();
            return Some(bridge);
        }

        None
    }

    /// Gibt Bridge nach ID zurück
    pub fn get_bridge(&self, id: &[u8; 20]) -> Option<BridgeInfo> {
        self.bridges.read().get(id).cloned()
    }

    /// Wählt beste Bridge für Region
    pub fn select_bridge(
        &self,
        region: &str,
        transport: Option<TransportType>,
    ) -> Option<BridgeInfo> {
        let bridges = self.bridges.read();

        let mut candidates: Vec<_> = bridges
            .values()
            .filter(|b| b.is_usable())
            .filter(|b| b.supports_region(region))
            .filter(|b| transport.map_or(true, |t| b.transport == t))
            .cloned()
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // Sortiere nach: Discovery-Security × (1 / burn_reports) × recency
        candidates.sort_by(|a, b| {
            let score_a = self.bridge_score(a);
            let score_b = self.bridge_score(b);
            score_b.partial_cmp(&score_a).unwrap()
        });

        candidates.into_iter().next()
    }

    /// Berechnet Bridge-Score
    fn bridge_score(&self, bridge: &BridgeInfo) -> f64 {
        let security = bridge.discovery_method.security_level() as f64;
        let burn_penalty = 1.0 / (1.0 + bridge.burn_reports as f64);

        let recency = if let Some(tested) = bridge.last_tested {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let age_hours = (now - tested) / 3600;
            1.0 / (1.0 + age_hours as f64 / 24.0)
        } else {
            0.5 // Ungetestete Bridges bekommen mittleren Wert
        };

        security * burn_penalty * recency
    }

    /// Gibt alle Bridges für Region zurück
    pub fn bridges_for_region(&self, region: &str) -> Vec<BridgeInfo> {
        let by_region = self.by_region.read();
        let bridges = self.bridges.read();

        let region_upper = region.to_uppercase();

        if let Some(ids) = by_region.get(&region_upper) {
            ids.iter()
                .filter_map(|id| bridges.get(id).cloned())
                .filter(|b| b.is_usable())
                .collect()
        } else {
            // Fallback: Alle Bridges ohne Region-Einschränkung
            bridges
                .values()
                .filter(|b| b.is_usable() && b.supported_regions.is_empty())
                .cloned()
                .collect()
        }
    }

    /// Markiert Bridge als verwendet (erfolgreich/fehlgeschlagen)
    pub fn mark_used(&self, id: &[u8; 20], success: bool) {
        let mut bridges = self.bridges.write();
        let mut stats = self.stats.write();

        if let Some(bridge) = bridges.get_mut(id) {
            bridge.mark_tested(success);

            if success {
                stats.successful_uses += 1;
            } else {
                stats.failed_uses += 1;
            }
        }
    }

    /// Meldet Bridge als gebrannt
    pub fn report_burned(&self, id: &[u8; 20]) {
        let mut bridges = self.bridges.write();

        if let Some(bridge) = bridges.get_mut(id) {
            bridge.report_burned();
        }

        self.update_stats();
    }

    /// Gibt Statistiken zurück
    pub fn stats(&self) -> BridgePoolStats {
        self.stats.read().clone()
    }

    /// Aktualisiert Statistiken
    fn update_stats(&self) {
        let bridges = self.bridges.read();
        let mut stats = self.stats.write();

        stats.total_bridges = bridges.len();
        stats.active_bridges = bridges
            .values()
            .filter(|b| b.status == BridgeStatus::Active)
            .count();
        stats.burned_bridges = bridges
            .values()
            .filter(|b| b.status == BridgeStatus::Burned)
            .count();
    }

    /// Rotiert abgelaufene Bridges
    pub fn rotate_expired(&self) -> Vec<[u8; 20]> {
        let bridges = self.bridges.read();

        let expired: Vec<_> = bridges
            .iter()
            .filter(|(_, b)| b.needs_rotation())
            .map(|(id, _)| *id)
            .collect();

        drop(bridges);

        for id in &expired {
            let mut bridges = self.bridges.write();
            if let Some(bridge) = bridges.get_mut(id) {
                bridge.status = BridgeStatus::Expired;
            }
        }

        self.update_stats();
        expired
    }

    /// Entfernt alle gebrannten und abgelaufenen Bridges
    pub fn cleanup(&self) -> usize {
        let bridges = self.bridges.read();

        let to_remove: Vec<_> = bridges
            .iter()
            .filter(|(_, b)| matches!(b.status, BridgeStatus::Burned | BridgeStatus::Expired))
            .map(|(id, _)| *id)
            .collect();

        drop(bridges);

        let count = to_remove.len();
        for id in to_remove {
            self.remove_bridge(&id);
        }

        count
    }

    /// Gibt Pool-Größe zurück
    pub fn len(&self) -> usize {
        self.bridges.read().len()
    }

    /// Prüft ob Pool leer ist
    pub fn is_empty(&self) -> bool {
        self.bridges.read().is_empty()
    }
}

// ============================================================================
// Bridge Distributor
// ============================================================================

/// Bridge-Distributor für verschiedene Discovery-Methoden
pub struct BridgeDistributor {
    /// MOAT-Endpoint
    moat_url: Option<String>,
    /// Email-Responder-Adresse
    email_responder: Option<String>,
    /// Rate-Limiting
    last_request: RwLock<HashMap<DiscoveryMethod, Instant>>,
    /// Request-Cooldown
    cooldown: Duration,
}

impl BridgeDistributor {
    /// Erstellt neuen Distributor
    pub fn new() -> Self {
        Self {
            moat_url: Some("https://bridges.torproject.org/moat".into()),
            email_responder: Some("bridges@torproject.org".into()),
            last_request: RwLock::new(HashMap::new()),
            cooldown: Duration::from_secs(300), // 5 Minuten
        }
    }

    /// Setzt MOAT-URL
    pub fn with_moat_url(mut self, url: &str) -> Self {
        self.moat_url = Some(url.to_string());
        self
    }

    /// Setzt Email-Responder
    pub fn with_email_responder(mut self, email: &str) -> Self {
        self.email_responder = Some(email.to_string());
        self
    }

    /// Prüft ob Request erlaubt ist (Rate-Limiting)
    pub fn can_request(&self, method: DiscoveryMethod) -> bool {
        let last = self.last_request.read();
        if let Some(time) = last.get(&method) {
            time.elapsed() >= self.cooldown
        } else {
            true
        }
    }

    /// Markiert Request als durchgeführt
    fn mark_requested(&self, method: DiscoveryMethod) {
        self.last_request.write().insert(method, Instant::now());
    }

    /// Fordert Bridge via MOAT an
    pub async fn request_moat(
        &self,
        transport: TransportType,
        _region: &str,
    ) -> Result<Vec<BridgeInfo>, BridgeError> {
        if !self.can_request(DiscoveryMethod::Moat) {
            return Err(BridgeError::RateLimited);
        }

        let _url = self
            .moat_url
            .as_ref()
            .ok_or(BridgeError::DiscoveryUnavailable)?;

        self.mark_requested(DiscoveryMethod::Moat);

        // In Produktion: HTTP POST mit CAPTCHA-Lösung
        // Simuliere Antwort
        let bridge = BridgeInfo::new(
            "192.0.2.1:443".into(),
            [0u8; 20],
            transport,
            DiscoveryMethod::Moat,
        );

        Ok(vec![bridge])
    }

    /// Fordert Bridge via Email an
    pub async fn request_email(
        &self,
        transport: TransportType,
        _user_email: &str,
    ) -> Result<(), BridgeError> {
        if !self.can_request(DiscoveryMethod::EmailResponder) {
            return Err(BridgeError::RateLimited);
        }

        let _responder = self
            .email_responder
            .as_ref()
            .ok_or(BridgeError::DiscoveryUnavailable)?;

        self.mark_requested(DiscoveryMethod::EmailResponder);

        // In Produktion: Email senden
        // Bridge wird asynchron per Email zurückgesendet
        let _ = transport;

        Ok(())
    }
}

impl Default for BridgeDistributor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Bridge-Fehler
#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("Bridge pool is full")]
    PoolFull,

    #[error("No bridges available for region")]
    NoBridgesAvailable,

    #[error("Bridge discovery method unavailable")]
    DiscoveryUnavailable,

    #[error("Rate limited, please wait")]
    RateLimited,

    #[error("Bridge test failed: {0}")]
    TestFailed(String),

    #[error("Invalid bridge line format")]
    InvalidFormat,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_fingerprint() -> [u8; 20] {
        let mut fp = [0u8; 20];
        fp[0] = 0xDE;
        fp[1] = 0xAD;
        fp[19] = 0xBE;
        fp
    }

    #[test]
    fn test_bridge_info_creation() {
        let bridge = BridgeInfo::new(
            "192.168.1.1:443".into(),
            test_fingerprint(),
            TransportType::Obfs4,
            DiscoveryMethod::Moat,
        );

        assert_eq!(bridge.address, "192.168.1.1:443");
        assert_eq!(bridge.transport, TransportType::Obfs4);
        assert_eq!(bridge.discovery_method, DiscoveryMethod::Moat);
        assert_eq!(bridge.status, BridgeStatus::Unknown);
        assert!(bridge.is_usable());
    }

    #[test]
    fn test_bridge_info_regions() {
        let bridge = BridgeInfo::new(
            "192.168.1.1:443".into(),
            test_fingerprint(),
            TransportType::Obfs4,
            DiscoveryMethod::Moat,
        )
        .with_region("CN")
        .with_region("IR");

        assert!(bridge.supports_region("CN"));
        assert!(bridge.supports_region("cn")); // Case-insensitive
        assert!(bridge.supports_region("IR"));
        assert!(!bridge.supports_region("DE"));
    }

    #[test]
    fn test_bridge_info_options() {
        let bridge = BridgeInfo::new(
            "192.168.1.1:443".into(),
            test_fingerprint(),
            TransportType::Obfs4,
            DiscoveryMethod::Moat,
        )
        .with_option("cert", "ABC123")
        .with_option("iat-mode", "1");

        assert_eq!(bridge.options.get("cert"), Some(&"ABC123".to_string()));
        assert_eq!(bridge.options.get("iat-mode"), Some(&"1".to_string()));
    }

    #[test]
    fn test_bridge_burn_reports() {
        let mut bridge = BridgeInfo::new(
            "192.168.1.1:443".into(),
            test_fingerprint(),
            TransportType::Obfs4,
            DiscoveryMethod::Moat,
        );

        assert!(bridge.is_usable());

        bridge.report_burned();
        bridge.report_burned();
        assert!(bridge.is_usable()); // Still under threshold

        bridge.report_burned();
        assert!(!bridge.is_usable()); // Now burned
        assert_eq!(bridge.status, BridgeStatus::Burned);
    }

    #[test]
    fn test_bridge_tested() {
        let mut bridge = BridgeInfo::new(
            "192.168.1.1:443".into(),
            test_fingerprint(),
            TransportType::Obfs4,
            DiscoveryMethod::Moat,
        );

        assert_eq!(bridge.status, BridgeStatus::Unknown);

        bridge.mark_tested(true);
        assert_eq!(bridge.status, BridgeStatus::Active);
        assert!(bridge.last_tested.is_some());

        bridge.mark_tested(false);
        assert_eq!(bridge.status, BridgeStatus::Unreachable);
    }

    #[test]
    fn test_bridge_to_line() {
        let bridge = BridgeInfo::new(
            "192.168.1.1:443".into(),
            test_fingerprint(),
            TransportType::Obfs4,
            DiscoveryMethod::Moat,
        )
        .with_option("cert", "ABC123");

        let line = bridge.to_bridge_line();
        assert!(line.contains("obfs4"));
        assert!(line.contains("192.168.1.1:443"));
        assert!(line.contains("cert=ABC123"));
    }

    #[test]
    fn test_discovery_method_security() {
        assert!(
            DiscoveryMethod::PhysicalExchange.security_level()
                > DiscoveryMethod::SocialGraph.security_level()
        );
        assert!(
            DiscoveryMethod::SocialGraph.security_level()
                > DiscoveryMethod::EmailResponder.security_level()
        );
        assert!(DiscoveryMethod::Moat.security_level() > DiscoveryMethod::Dht.security_level());
    }

    #[test]
    fn test_bridge_pool_add() {
        let pool = BridgePool::new(BridgePoolConfig::default());

        let bridge = BridgeInfo::new(
            "192.168.1.1:443".into(),
            test_fingerprint(),
            TransportType::Obfs4,
            DiscoveryMethod::Moat,
        )
        .with_region("CN");

        assert!(pool.add_bridge(bridge).is_ok());
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_bridge_pool_select() {
        let pool = BridgePool::new(BridgePoolConfig::default());

        // Add bridge for China
        let bridge1 = BridgeInfo::new(
            "192.168.1.1:443".into(),
            test_fingerprint(),
            TransportType::Obfs4,
            DiscoveryMethod::Moat,
        )
        .with_region("CN");

        // Add bridge for Iran
        let mut fp2 = test_fingerprint();
        fp2[0] = 0xCA;
        let bridge2 = BridgeInfo::new(
            "192.168.1.2:443".into(),
            fp2,
            TransportType::Meek,
            DiscoveryMethod::SocialGraph,
        )
        .with_region("IR");

        pool.add_bridge(bridge1).unwrap();
        pool.add_bridge(bridge2).unwrap();

        // Select for China
        let selected = pool.select_bridge("CN", None);
        assert!(selected.is_some());
        assert!(selected.unwrap().supports_region("CN"));

        // Select for Iran with specific transport
        let selected = pool.select_bridge("IR", Some(TransportType::Meek));
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().transport, TransportType::Meek);
    }

    #[test]
    fn test_bridge_pool_mark_used() {
        let pool = BridgePool::new(BridgePoolConfig::default());

        let bridge = BridgeInfo::new(
            "192.168.1.1:443".into(),
            test_fingerprint(),
            TransportType::Obfs4,
            DiscoveryMethod::Moat,
        );
        let id = bridge.id;

        pool.add_bridge(bridge).unwrap();
        pool.mark_used(&id, true);

        let stats = pool.stats();
        assert_eq!(stats.successful_uses, 1);

        let updated = pool.get_bridge(&id).unwrap();
        assert_eq!(updated.status, BridgeStatus::Active);
    }

    #[test]
    fn test_bridge_pool_report_burned() {
        let pool = BridgePool::new(BridgePoolConfig::default());

        let bridge = BridgeInfo::new(
            "192.168.1.1:443".into(),
            test_fingerprint(),
            TransportType::Obfs4,
            DiscoveryMethod::Moat,
        );
        let id = bridge.id;

        pool.add_bridge(bridge).unwrap();

        for _ in 0..BRIDGE_BURN_THRESHOLD {
            pool.report_burned(&id);
        }

        let stats = pool.stats();
        assert_eq!(stats.burned_bridges, 1);
    }

    #[test]
    fn test_bridge_pool_cleanup() {
        let pool = BridgePool::new(BridgePoolConfig::default());

        let mut bridge = BridgeInfo::new(
            "192.168.1.1:443".into(),
            test_fingerprint(),
            TransportType::Obfs4,
            DiscoveryMethod::Moat,
        );
        bridge.status = BridgeStatus::Burned;

        pool.add_bridge(bridge).unwrap();
        assert_eq!(pool.len(), 1);

        let cleaned = pool.cleanup();
        assert_eq!(cleaned, 1);
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn test_bridge_pool_full() {
        let config = BridgePoolConfig {
            max_bridges: 2,
            ..Default::default()
        };
        let pool = BridgePool::new(config);

        let bridge1 = BridgeInfo::new(
            "192.168.1.1:443".into(),
            test_fingerprint(),
            TransportType::Obfs4,
            DiscoveryMethod::Moat,
        );

        let mut fp2 = test_fingerprint();
        fp2[0] = 0xCA;
        let bridge2 = BridgeInfo::new(
            "192.168.1.2:443".into(),
            fp2,
            TransportType::Obfs4,
            DiscoveryMethod::Moat,
        );

        let mut fp3 = test_fingerprint();
        fp3[0] = 0xFE;
        let bridge3 = BridgeInfo::new(
            "192.168.1.3:443".into(),
            fp3,
            TransportType::Obfs4,
            DiscoveryMethod::Moat,
        );

        assert!(pool.add_bridge(bridge1).is_ok());
        assert!(pool.add_bridge(bridge2).is_ok());
        assert!(matches!(
            pool.add_bridge(bridge3),
            Err(BridgeError::PoolFull)
        ));
    }

    #[test]
    fn test_bridge_distributor_rate_limiting() {
        let distributor = BridgeDistributor::new();

        assert!(distributor.can_request(DiscoveryMethod::Moat));
        distributor.mark_requested(DiscoveryMethod::Moat);
        assert!(!distributor.can_request(DiscoveryMethod::Moat));

        // Other methods should still work
        assert!(distributor.can_request(DiscoveryMethod::EmailResponder));
    }

    #[test]
    fn test_bridge_status_display() {
        assert_eq!(format!("{}", BridgeStatus::Active), "active");
        assert_eq!(format!("{}", BridgeStatus::Burned), "burned");
        assert_eq!(format!("{}", BridgeStatus::Unreachable), "unreachable");
    }

    #[test]
    fn test_discovery_method_display() {
        assert_eq!(format!("{}", DiscoveryMethod::Moat), "moat");
        assert_eq!(format!("{}", DiscoveryMethod::SocialGraph), "social");
        assert_eq!(format!("{}", DiscoveryMethod::PhysicalExchange), "physical");
    }
}
