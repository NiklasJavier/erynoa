//! # Pluggable Transports (RL19)
//!
//! Traffic-Obfuscation zur Umgehung von Deep Packet Inspection (DPI).
//!
//! ## Transport-Modi
//!
//! | Modus           | Beschreibung                                    | Use-Case           |
//! |-----------------|------------------------------------------------|-------------------|
//! | `Direct`        | Kein Obfuscation                               | LOW censorship    |
//! | `Obfs4`         | Randomisiertes Traffic-Shaping                 | MEDIUM censorship |
//! | `Meek`          | HTTP über CDN (Domain-Fronting)                | HIGH censorship   |
//! | `Snowflake`     | WebRTC über Freiwillige Proxies                | HIGH censorship   |
//! | `DomainFronting`| HTTPS mit getarntem Host                       | CRITICAL          |
//!
//! ## Axiom-Referenz
//!
//! - **RL19**: AS-Path Zensur-Resistenz, Pluggable Transports
//! - **Κ20**: Resilience gegen State-Level-Adversaries

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use thiserror::Error;

// ============================================================================
// Constants
// ============================================================================

/// Maximum Reconnect-Versuche pro Transport
pub const MAX_RECONNECT_ATTEMPTS: u32 = 5;

/// Timeout für Transport-Handshake
pub const TRANSPORT_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(30);

/// Obfs4 IAT-Mode Delay (Inter-Arrival Time)
pub const OBFS4_IAT_MODE_DELAY: Duration = Duration::from_millis(50);

/// Bekannte GFW (Great Firewall) ASNs
pub const GFW_ASES: &[u32] = &[
    4134,  // China Telecom
    4837,  // China Unicom
    9808,  // China Mobile
    4812,  // China Telecom (Backbone)
    17816, // China Unicom IP
];

/// Bekannte Zensor-Regionen
pub const KNOWN_HOSTILE_REGIONS: &[&str] = &["CN", "IR", "RU", "BY", "TM", "KP"];

// ============================================================================
// Transport-Typen
// ============================================================================

/// Transport-Typ für Zensur-Resistenz (RL19)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportType {
    /// Direkter TCP/QUIC - kein Obfuscation
    Direct,
    /// obfs4: Randomisiertes Traffic-Shaping mit Scramblesuit-Nachfolger
    Obfs4,
    /// meek: HTTPS über CDN (Cloudflare, Azure, Amazon)
    Meek,
    /// Snowflake: WebRTC über kurzlebige Freiwilligen-Proxies
    Snowflake,
    /// Domain-Fronting: HTTPS mit SNI ≠ Host-Header
    DomainFronting,
    /// Steganographic: Payload in Bildern/Audio versteckt (CRITICAL only)
    Steganographic,
}

impl TransportType {
    /// Gibt Detection-Resistance-Faktor zurück (0.0 = leicht erkennbar, 1.0 = unerkennbar)
    pub fn detection_resistance(&self) -> f64 {
        match self {
            TransportType::Direct => 0.0,
            TransportType::Obfs4 => 0.85,
            TransportType::Meek => 0.95,
            TransportType::Snowflake => 0.92,
            TransportType::DomainFronting => 0.97,
            TransportType::Steganographic => 0.99,
        }
    }

    /// Geschätzte zusätzliche Latenz durch Transport
    pub fn latency_overhead(&self) -> Duration {
        match self {
            TransportType::Direct => Duration::ZERO,
            TransportType::Obfs4 => Duration::from_millis(20),
            TransportType::Meek => Duration::from_millis(150),
            TransportType::Snowflake => Duration::from_millis(100),
            TransportType::DomainFronting => Duration::from_millis(80),
            TransportType::Steganographic => Duration::from_secs(2),
        }
    }

    /// Bandwidth-Overhead-Faktor (1.0 = kein Overhead)
    pub fn bandwidth_overhead(&self) -> f64 {
        match self {
            TransportType::Direct => 1.0,
            TransportType::Obfs4 => 1.15,
            TransportType::Meek => 1.5,
            TransportType::Snowflake => 1.3,
            TransportType::DomainFronting => 1.4,
            TransportType::Steganographic => 10.0, // ~1 bit/pixel
        }
    }

    /// Empfohlener Transport für Censorship-Level
    pub fn for_censorship_level(level: CensorshipLevel) -> Self {
        match level {
            CensorshipLevel::Low => TransportType::Direct,
            CensorshipLevel::Medium => TransportType::Obfs4,
            CensorshipLevel::High => TransportType::Meek,
            CensorshipLevel::Critical => TransportType::Snowflake,
        }
    }
}

impl fmt::Display for TransportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportType::Direct => write!(f, "direct"),
            TransportType::Obfs4 => write!(f, "obfs4"),
            TransportType::Meek => write!(f, "meek"),
            TransportType::Snowflake => write!(f, "snowflake"),
            TransportType::DomainFronting => write!(f, "domain-fronting"),
            TransportType::Steganographic => write!(f, "steganographic"),
        }
    }
}

// ============================================================================
// Censorship-Level
// ============================================================================

/// Zensur-Level basierend auf Region und AS-Path
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CensorshipLevel {
    /// Keine bekannte Zensur
    Low,
    /// Gelegentliches Blocking (z.B. bestimmte Ports)
    Medium,
    /// Aktives DPI und Blocking (z.B. Iran, Russland)
    High,
    /// Staatliche Firewall mit ML-Detection (z.B. China)
    Critical,
}

impl CensorshipLevel {
    /// Bestimme Level aus Region-Code
    pub fn from_region(region: &str) -> Self {
        match region.to_uppercase().as_str() {
            "CN" | "KP" => CensorshipLevel::Critical,
            "IR" | "TM" => CensorshipLevel::High,
            "RU" | "BY" => CensorshipLevel::Medium,
            _ => CensorshipLevel::Low,
        }
    }

    /// Bestimme Level aus ASN
    pub fn from_asn(asn: u32) -> Self {
        if GFW_ASES.contains(&asn) {
            return CensorshipLevel::Critical;
        }
        CensorshipLevel::Low
    }

    /// Kombiniere Region und ASN für finales Level
    pub fn assess(region: &str, source_asn: u32, dest_asn: u32) -> Self {
        let region_level = Self::from_region(region);
        let source_level = Self::from_asn(source_asn);
        let dest_level = Self::from_asn(dest_asn);

        // Höchstes Level gewinnt
        region_level.max(source_level).max(dest_level)
    }
}

impl fmt::Display for CensorshipLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CensorshipLevel::Low => write!(f, "LOW"),
            CensorshipLevel::Medium => write!(f, "MEDIUM"),
            CensorshipLevel::High => write!(f, "HIGH"),
            CensorshipLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

// ============================================================================
// Obfs4 Configuration
// ============================================================================

/// obfs4-Konfiguration
#[derive(Debug, Clone)]
pub struct Obfs4Config {
    /// Node-ID (20 bytes, SHA1 des Public Keys)
    pub node_id: [u8; 20],
    /// Public Key (32 bytes, Curve25519)
    pub public_key: [u8; 32],
    /// IAT-Mode: 0=disabled, 1=enabled, 2=paranoid
    pub iat_mode: u8,
    /// Cert (Base64-encoded Bridge-Line Credential)
    pub cert: String,
}

impl Obfs4Config {
    /// Erstellt neue obfs4-Config aus Bridge-Line
    pub fn from_bridge_line(line: &str) -> Result<Self, TransportError> {
        // Format: obfs4 <IP:Port> <Fingerprint> cert=<cert> iat-mode=<mode>
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return Err(TransportError::InvalidConfig(
                "Invalid obfs4 bridge line format".into(),
            ));
        }

        let mut cert = String::new();
        let mut iat_mode = 0u8;

        for part in &parts[3..] {
            if let Some(c) = part.strip_prefix("cert=") {
                cert = c.to_string();
            } else if let Some(m) = part.strip_prefix("iat-mode=") {
                iat_mode = m.parse().unwrap_or(0);
            }
        }

        if cert.is_empty() {
            return Err(TransportError::InvalidConfig(
                "Missing cert in bridge line".into(),
            ));
        }

        // Decode cert to extract node_id and public_key
        let cert_bytes = base64_decode(&cert)?;
        if cert_bytes.len() < 52 {
            return Err(TransportError::InvalidConfig("Cert too short".into()));
        }

        let mut node_id = [0u8; 20];
        let mut public_key = [0u8; 32];
        node_id.copy_from_slice(&cert_bytes[0..20]);
        public_key.copy_from_slice(&cert_bytes[20..52]);

        Ok(Self {
            node_id,
            public_key,
            iat_mode,
            cert,
        })
    }
}

/// Vereinfachtes Base64-Decoding
fn base64_decode(input: &str) -> Result<Vec<u8>, TransportError> {
    // Vereinfachte Implementierung - in Produktion: base64 crate
    let cleaned = input.replace(['-', '_'], "+");
    let padded = match cleaned.len() % 4 {
        2 => format!("{}==", cleaned),
        3 => format!("{}=", cleaned),
        _ => cleaned,
    };

    // Simpler Decoder für Tests
    let mut result = Vec::new();
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    for chunk in padded.as_bytes().chunks(4) {
        if chunk.len() < 4 {
            break;
        }

        let mut acc = 0u32;
        let mut valid_chars = 0;

        for (i, &c) in chunk.iter().enumerate() {
            if c == b'=' {
                break;
            }
            if let Some(pos) = alphabet.iter().position(|&x| x == c) {
                acc |= (pos as u32) << (6 * (3 - i));
                valid_chars += 1;
            }
        }

        if valid_chars >= 2 {
            result.push((acc >> 16) as u8);
        }
        if valid_chars >= 3 {
            result.push((acc >> 8) as u8);
        }
        if valid_chars >= 4 {
            result.push(acc as u8);
        }
    }

    Ok(result)
}

// ============================================================================
// Meek Configuration
// ============================================================================

/// Meek-Konfiguration (CDN-basiert)
#[derive(Debug, Clone)]
pub struct MeekConfig {
    /// Front-Domain (was der Zensor sieht, z.B. "ajax.aspnetcdn.com")
    pub front_domain: String,
    /// Actual-Host (wo der Traffic hingeht, z.B. "meek.azureedge.net")
    pub url: String,
    /// Optional: Zusätzliche Header
    pub headers: HashMap<String, String>,
}

impl MeekConfig {
    /// Azure CDN Meek
    pub fn azure() -> Self {
        Self {
            front_domain: "ajax.aspnetcdn.com".into(),
            url: "https://meek.azureedge.net/".into(),
            headers: HashMap::new(),
        }
    }

    /// Amazon CloudFront Meek
    pub fn amazon() -> Self {
        Self {
            front_domain: "d2cly7j4zqgua7.cloudfront.net".into(),
            url: "https://d2cly7j4zqgua7.cloudfront.net/".into(),
            headers: HashMap::new(),
        }
    }

    /// Google App Engine Meek
    pub fn google() -> Self {
        Self {
            front_domain: "www.google.com".into(),
            url: "https://meek-reflect.appspot.com/".into(),
            headers: HashMap::new(),
        }
    }
}

// ============================================================================
// Snowflake Configuration
// ============================================================================

/// Snowflake-Konfiguration (WebRTC-basiert)
#[derive(Debug, Clone)]
pub struct SnowflakeConfig {
    /// Broker-URL für Proxy-Vermittlung
    pub broker_url: String,
    /// STUN-Server für NAT-Traversal
    pub stun_servers: Vec<String>,
    /// Maximale Proxy-Anzahl
    pub max_proxies: usize,
    /// Proxy-Rotation-Intervall
    pub proxy_rotation: Duration,
}

impl Default for SnowflakeConfig {
    fn default() -> Self {
        Self {
            broker_url: "https://snowflake-broker.torproject.net/".into(),
            stun_servers: vec![
                "stun:stun.l.google.com:19302".into(),
                "stun:stun.voip.blackberry.com:3478".into(),
            ],
            max_proxies: 3,
            proxy_rotation: Duration::from_secs(600), // 10 Minuten
        }
    }
}

// ============================================================================
// Transport Wrapper
// ============================================================================

/// Transport-Wrapper mit aktuellem Status
#[derive(Debug)]
pub struct TransportWrapper {
    /// Aktiver Transport-Typ
    transport_type: TransportType,
    /// Transport-spezifische Konfiguration
    config: TransportConfig,
    /// Connection-Status
    status: RwLock<TransportStatus>,
    /// Statistiken
    stats: RwLock<TransportStats>,
}

/// Transport-spezifische Konfiguration
#[derive(Debug, Clone)]
pub enum TransportConfig {
    Direct,
    Obfs4(Obfs4Config),
    Meek(MeekConfig),
    Snowflake(SnowflakeConfig),
    DomainFronting { front: String, target: String },
    Steganographic { cover_type: String },
}

/// Transport-Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportStatus {
    /// Nicht verbunden
    Disconnected,
    /// Verbindung wird aufgebaut
    Connecting,
    /// Verbunden und funktionsfähig
    Connected,
    /// Fehler beim Verbinden
    Failed,
    /// Geblockt (Zensor hat erkannt)
    Blocked,
}

/// Transport-Statistiken
#[derive(Debug, Clone, Default)]
pub struct TransportStats {
    /// Bytes gesendet
    pub bytes_sent: u64,
    /// Bytes empfangen
    pub bytes_received: u64,
    /// Erfolgreiche Verbindungen
    pub successful_connections: u32,
    /// Fehlgeschlagene Verbindungen
    pub failed_connections: u32,
    /// Durchschnittliche Latenz
    pub avg_latency_ms: f64,
    /// Letzter Verbindungszeitpunkt
    pub last_connected: Option<Instant>,
}

impl TransportWrapper {
    /// Erstellt neuen Transport-Wrapper
    pub fn new(transport_type: TransportType, config: TransportConfig) -> Self {
        Self {
            transport_type,
            config,
            status: RwLock::new(TransportStatus::Disconnected),
            stats: RwLock::new(TransportStats::default()),
        }
    }

    /// Erstellt Direct-Transport
    pub fn direct() -> Self {
        Self::new(TransportType::Direct, TransportConfig::Direct)
    }

    /// Erstellt obfs4-Transport
    pub fn obfs4(config: Obfs4Config) -> Self {
        Self::new(TransportType::Obfs4, TransportConfig::Obfs4(config))
    }

    /// Erstellt Meek-Transport
    pub fn meek(config: MeekConfig) -> Self {
        Self::new(TransportType::Meek, TransportConfig::Meek(config))
    }

    /// Erstellt Snowflake-Transport
    pub fn snowflake(config: SnowflakeConfig) -> Self {
        Self::new(TransportType::Snowflake, TransportConfig::Snowflake(config))
    }

    /// Gibt Transport-Typ zurück
    pub fn transport_type(&self) -> TransportType {
        self.transport_type
    }

    /// Gibt aktuellen Status zurück
    pub fn status(&self) -> TransportStatus {
        *self.status.read()
    }

    /// Setzt Status
    pub fn set_status(&self, status: TransportStatus) {
        *self.status.write() = status;
    }

    /// Gibt Statistiken zurück
    pub fn stats(&self) -> TransportStats {
        self.stats.read().clone()
    }

    /// Simuliert Verbindungsaufbau (in Produktion: echte Implementierung)
    pub async fn connect(&self, target: &str) -> Result<(), TransportError> {
        self.set_status(TransportStatus::Connecting);

        // Simuliere Verbindungsaufbau basierend auf Transport-Typ
        let delay = self.transport_type.latency_overhead();
        tokio::time::sleep(delay).await;

        // In Produktion: Echter Verbindungsaufbau
        match &self.config {
            TransportConfig::Direct => {
                // Direkte TCP/QUIC Verbindung
            }
            TransportConfig::Obfs4(cfg) => {
                // obfs4 Handshake mit NTor
                if cfg.cert.is_empty() {
                    self.set_status(TransportStatus::Failed);
                    return Err(TransportError::HandshakeFailed("Missing cert".into()));
                }
            }
            TransportConfig::Meek(cfg) => {
                // HTTP CONNECT über CDN
                if cfg.url.is_empty() {
                    self.set_status(TransportStatus::Failed);
                    return Err(TransportError::HandshakeFailed("Missing URL".into()));
                }
            }
            TransportConfig::Snowflake(cfg) => {
                // WebRTC über Broker
                if cfg.broker_url.is_empty() {
                    self.set_status(TransportStatus::Failed);
                    return Err(TransportError::HandshakeFailed("Missing broker".into()));
                }
            }
            TransportConfig::DomainFronting { front, target: _ } => {
                if front.is_empty() {
                    self.set_status(TransportStatus::Failed);
                    return Err(TransportError::HandshakeFailed(
                        "Missing front domain".into(),
                    ));
                }
            }
            TransportConfig::Steganographic { .. } => {
                // Steganographic channel setup
            }
        }

        self.set_status(TransportStatus::Connected);

        // Update Stats
        {
            let mut stats = self.stats.write();
            stats.successful_connections += 1;
            stats.last_connected = Some(Instant::now());
        }

        let _ = target; // Verwendet in echter Implementierung
        Ok(())
    }

    /// Schließt Verbindung
    pub fn disconnect(&self) {
        self.set_status(TransportStatus::Disconnected);
    }

    /// Sendet Daten (wrapped)
    pub async fn send(&self, data: &[u8]) -> Result<usize, TransportError> {
        if self.status() != TransportStatus::Connected {
            return Err(TransportError::NotConnected);
        }

        // In Produktion: Echtes Senden mit Obfuscation
        let sent = data.len();

        {
            let mut stats = self.stats.write();
            stats.bytes_sent += sent as u64;
        }

        Ok(sent)
    }

    /// Empfängt Daten (unwrapped)
    pub async fn receive(&self, _buffer: &mut [u8]) -> Result<usize, TransportError> {
        if self.status() != TransportStatus::Connected {
            return Err(TransportError::NotConnected);
        }

        // In Produktion: Echtes Empfangen mit De-Obfuscation
        Ok(0)
    }
}

// ============================================================================
// Transport Manager
// ============================================================================

/// Transport-Manager für automatische Auswahl und Fallback
pub struct TransportManager {
    /// Verfügbare Transports (sortiert nach Präferenz)
    transports: Vec<Arc<TransportWrapper>>,
    /// Aktiver Transport-Index
    active_index: RwLock<usize>,
    /// Erkanntes Censorship-Level
    censorship_level: RwLock<CensorshipLevel>,
    /// Konfiguration
    config: TransportManagerConfig,
}

/// Transport-Manager-Konfiguration
#[derive(Debug, Clone)]
pub struct TransportManagerConfig {
    /// Automatische Transport-Auswahl
    pub auto_select: bool,
    /// Automatischer Fallback bei Blocking
    pub auto_fallback: bool,
    /// Probe-Intervall für Censorship-Detection
    pub probe_interval: Duration,
    /// Maximale Fallback-Versuche
    pub max_fallback_attempts: u32,
}

impl Default for TransportManagerConfig {
    fn default() -> Self {
        Self {
            auto_select: true,
            auto_fallback: true,
            probe_interval: Duration::from_secs(300), // 5 Minuten
            max_fallback_attempts: 3,
        }
    }
}

impl TransportManager {
    /// Erstellt neuen Transport-Manager
    pub fn new(config: TransportManagerConfig) -> Self {
        Self {
            transports: Vec::new(),
            active_index: RwLock::new(0),
            censorship_level: RwLock::new(CensorshipLevel::Low),
            config,
        }
    }

    /// Fügt Transport hinzu
    pub fn add_transport(&mut self, transport: TransportWrapper) {
        self.transports.push(Arc::new(transport));
    }

    /// Setzt Censorship-Level
    pub fn set_censorship_level(&self, level: CensorshipLevel) {
        *self.censorship_level.write() = level;
    }

    /// Gibt aktuelles Censorship-Level zurück
    pub fn censorship_level(&self) -> CensorshipLevel {
        *self.censorship_level.read()
    }

    /// Wählt besten Transport basierend auf Censorship-Level
    pub fn select_transport(&self) -> Option<Arc<TransportWrapper>> {
        // Verwende config für Auto-Selection
        if !self.config.auto_select {
            return self.active_transport();
        }

        let level = self.censorship_level();
        let preferred_type = TransportType::for_censorship_level(level);

        // Suche passenden Transport
        for transport in &self.transports {
            if transport.transport_type() == preferred_type {
                return Some(Arc::clone(transport));
            }
        }

        // Fallback: Höchsten verfügbaren nehmen
        self.transports
            .iter()
            .max_by_key(|t| t.transport_type().detection_resistance() as u64)
            .cloned()
    }

    /// Gibt aktiven Transport zurück
    pub fn active_transport(&self) -> Option<Arc<TransportWrapper>> {
        let index = *self.active_index.read();
        self.transports.get(index).cloned()
    }

    /// Fallback zum nächsten Transport
    pub fn fallback(&self) -> bool {
        // Prüfe ob Auto-Fallback aktiviert ist
        if !self.config.auto_fallback {
            return false;
        }

        let mut index = self.active_index.write();

        // Prüfe max_fallback_attempts
        if (*index as u32) >= self.config.max_fallback_attempts {
            return false;
        }

        if *index + 1 < self.transports.len() {
            *index += 1;
            true
        } else {
            false
        }
    }

    /// Gibt die Konfiguration zurück
    pub fn config(&self) -> &TransportManagerConfig {
        &self.config
    }

    /// Gibt das Probe-Intervall zurück
    pub fn probe_interval(&self) -> Duration {
        self.config.probe_interval
    }

    /// Gibt alle verfügbaren Transports zurück
    pub fn available_transports(&self) -> Vec<TransportType> {
        self.transports.iter().map(|t| t.transport_type()).collect()
    }

    /// Prüft ob Blocking erkannt wurde
    pub async fn detect_blocking(&self, target: &str) -> bool {
        // In Produktion: Sende Probes über verschiedene Transports
        // und vergleiche Erreichbarkeit
        let _ = target;
        false
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Transport-Fehler
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Invalid transport configuration: {0}")]
    InvalidConfig(String),

    #[error("Transport handshake failed: {0}")]
    HandshakeFailed(String),

    #[error("Transport not connected")]
    NotConnected,

    #[error("Transport blocked by censor")]
    Blocked,

    #[error("Connection timeout")]
    Timeout,

    #[error("IO error: {0}")]
    Io(String),

    #[error("All transports exhausted")]
    AllTransportsExhausted,
}

// ============================================================================
// AS-Path Censorship Detection
// ============================================================================

/// Erkennt ob AS-Path durch zensierte Regionen führt
pub fn detect_censored_as_path(
    source_asn: u32,
    destination_asn: u32,
    known_censors: &[u32],
) -> bool {
    // Vereinfacht: Prüfe ob bekannte Zensor-ASes involviert
    let all_censors: Vec<u32> = known_censors
        .iter()
        .copied()
        .chain(GFW_ASES.iter().copied())
        .collect();

    // Heuristik: Wenn Source oder Destination in Zensor-AS
    all_censors.contains(&source_asn) || all_censors.contains(&destination_asn)
}

/// Empfiehlt Transport basierend auf AS-Path
pub fn recommend_transport(
    source_asn: u32,
    destination_asn: u32,
    region: &str,
    available_bridges: usize,
) -> TransportType {
    let level = CensorshipLevel::assess(region, source_asn, destination_asn);

    match level {
        CensorshipLevel::Low => TransportType::Direct,
        CensorshipLevel::Medium => TransportType::Obfs4,
        CensorshipLevel::High => {
            if available_bridges > 0 {
                TransportType::Meek
            } else {
                TransportType::Snowflake
            }
        }
        CensorshipLevel::Critical => TransportType::Snowflake,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_type_detection_resistance() {
        assert_eq!(TransportType::Direct.detection_resistance(), 0.0);
        assert!(TransportType::Obfs4.detection_resistance() > 0.8);
        assert!(TransportType::Meek.detection_resistance() > 0.9);
        assert!(TransportType::Snowflake.detection_resistance() > 0.9);
        assert!(TransportType::Steganographic.detection_resistance() > 0.95);
    }

    #[test]
    fn test_transport_type_display() {
        assert_eq!(format!("{}", TransportType::Obfs4), "obfs4");
        assert_eq!(format!("{}", TransportType::Meek), "meek");
        assert_eq!(format!("{}", TransportType::Snowflake), "snowflake");
    }

    #[test]
    fn test_censorship_level_from_region() {
        assert_eq!(
            CensorshipLevel::from_region("CN"),
            CensorshipLevel::Critical
        );
        assert_eq!(CensorshipLevel::from_region("IR"), CensorshipLevel::High);
        assert_eq!(CensorshipLevel::from_region("RU"), CensorshipLevel::Medium);
        assert_eq!(CensorshipLevel::from_region("DE"), CensorshipLevel::Low);
        assert_eq!(CensorshipLevel::from_region("US"), CensorshipLevel::Low);
    }

    #[test]
    fn test_censorship_level_from_asn() {
        assert_eq!(CensorshipLevel::from_asn(4134), CensorshipLevel::Critical); // China Telecom
        assert_eq!(CensorshipLevel::from_asn(4837), CensorshipLevel::Critical); // China Unicom
        assert_eq!(CensorshipLevel::from_asn(12345), CensorshipLevel::Low);
    }

    #[test]
    fn test_censorship_level_assess() {
        // China + GFW ASN = Critical
        assert_eq!(
            CensorshipLevel::assess("CN", 4134, 12345),
            CensorshipLevel::Critical
        );

        // Normal region + normal ASN = Low
        assert_eq!(
            CensorshipLevel::assess("DE", 12345, 54321),
            CensorshipLevel::Low
        );

        // Iran region = High
        assert_eq!(
            CensorshipLevel::assess("IR", 12345, 54321),
            CensorshipLevel::High
        );
    }

    #[test]
    fn test_transport_type_for_censorship_level() {
        assert_eq!(
            TransportType::for_censorship_level(CensorshipLevel::Low),
            TransportType::Direct
        );
        assert_eq!(
            TransportType::for_censorship_level(CensorshipLevel::Medium),
            TransportType::Obfs4
        );
        assert_eq!(
            TransportType::for_censorship_level(CensorshipLevel::High),
            TransportType::Meek
        );
        assert_eq!(
            TransportType::for_censorship_level(CensorshipLevel::Critical),
            TransportType::Snowflake
        );
    }

    #[test]
    fn test_detect_censored_as_path() {
        // GFW AS in path
        assert!(detect_censored_as_path(4134, 12345, &[]));
        assert!(detect_censored_as_path(12345, 4837, &[]));

        // Custom censor list
        assert!(detect_censored_as_path(12345, 54321, &[12345]));

        // Clean path
        assert!(!detect_censored_as_path(12345, 54321, &[]));
    }

    #[test]
    fn test_recommend_transport() {
        // China → Snowflake (no bridges)
        assert_eq!(
            recommend_transport(4134, 12345, "CN", 0),
            TransportType::Snowflake
        );

        // Iran → Meek (with bridges)
        assert_eq!(
            recommend_transport(12345, 54321, "IR", 5),
            TransportType::Meek
        );

        // Germany → Direct
        assert_eq!(
            recommend_transport(12345, 54321, "DE", 0),
            TransportType::Direct
        );
    }

    #[test]
    fn test_meek_config_presets() {
        let azure = MeekConfig::azure();
        assert!(azure.front_domain.contains("aspnetcdn"));

        let amazon = MeekConfig::amazon();
        assert!(amazon.front_domain.contains("cloudfront"));

        let google = MeekConfig::google();
        assert!(google.front_domain.contains("google"));
    }

    #[test]
    fn test_snowflake_config_default() {
        let config = SnowflakeConfig::default();
        assert!(!config.broker_url.is_empty());
        assert!(!config.stun_servers.is_empty());
        assert_eq!(config.max_proxies, 3);
    }

    #[test]
    fn test_transport_wrapper_status() {
        let wrapper = TransportWrapper::direct();
        assert_eq!(wrapper.status(), TransportStatus::Disconnected);

        wrapper.set_status(TransportStatus::Connected);
        assert_eq!(wrapper.status(), TransportStatus::Connected);

        wrapper.disconnect();
        assert_eq!(wrapper.status(), TransportStatus::Disconnected);
    }

    #[test]
    fn test_transport_manager_creation() {
        let mut manager = TransportManager::new(TransportManagerConfig::default());
        manager.add_transport(TransportWrapper::direct());
        manager.add_transport(TransportWrapper::meek(MeekConfig::azure()));

        let types = manager.available_transports();
        assert!(types.contains(&TransportType::Direct));
        assert!(types.contains(&TransportType::Meek));
    }

    #[test]
    fn test_transport_manager_select() {
        let mut manager = TransportManager::new(TransportManagerConfig::default());
        manager.add_transport(TransportWrapper::direct());
        manager.add_transport(TransportWrapper::meek(MeekConfig::azure()));

        // Low censorship → Direct
        manager.set_censorship_level(CensorshipLevel::Low);
        let selected = manager.select_transport().unwrap();
        assert_eq!(selected.transport_type(), TransportType::Direct);

        // High censorship → Meek
        manager.set_censorship_level(CensorshipLevel::High);
        let selected = manager.select_transport().unwrap();
        assert_eq!(selected.transport_type(), TransportType::Meek);
    }

    #[test]
    fn test_transport_manager_fallback() {
        let mut manager = TransportManager::new(TransportManagerConfig::default());
        manager.add_transport(TransportWrapper::direct());
        manager.add_transport(TransportWrapper::meek(MeekConfig::azure()));

        assert!(manager.fallback()); // Direct → Meek
        assert!(!manager.fallback()); // No more transports
    }

    #[test]
    fn test_transport_latency_overhead() {
        assert_eq!(TransportType::Direct.latency_overhead(), Duration::ZERO);
        assert!(TransportType::Obfs4.latency_overhead() > Duration::ZERO);
        assert!(TransportType::Meek.latency_overhead() > TransportType::Obfs4.latency_overhead());
        assert!(TransportType::Steganographic.latency_overhead() > Duration::from_secs(1));
    }

    #[test]
    fn test_transport_bandwidth_overhead() {
        assert_eq!(TransportType::Direct.bandwidth_overhead(), 1.0);
        assert!(TransportType::Obfs4.bandwidth_overhead() > 1.0);
        assert!(TransportType::Steganographic.bandwidth_overhead() > 5.0);
    }

    #[test]
    fn test_censorship_level_ordering() {
        assert!(CensorshipLevel::Low < CensorshipLevel::Medium);
        assert!(CensorshipLevel::Medium < CensorshipLevel::High);
        assert!(CensorshipLevel::High < CensorshipLevel::Critical);
    }
}
