//! # Bootstrap-Helpers (V2.6)
//!
//! Automatische "Recommended Relay Lists" aus DHT für Newcomer.
//! Schneller Einstieg ohne manuelle Konfiguration.
//!
//! ## Discovery-Strategie
//!
//! 1. Check lokalen Cache
//! 2. DHT-Query für "/erynoa/relays/v1/{region}"
//! 3. Filter nach Trust + Region
//! 4. Sortiere nach (Trust × Capacity / Latency)
//!
//! ## Axiom-Referenz
//!
//! - **RL5-RL7**: Trust-basierte Relay-Auswahl
//! - **Κ23**: Gateway - Realm-Join via P2P

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use parking_lot::RwLock;
use thiserror::Error;

// ============================================================================
// Constants
// ============================================================================

/// Minimum Relays für Quick-Start
pub const MIN_RELAYS_QUICK_START: usize = 5;

/// Maximum Relays im Cache
pub const MAX_CACHED_RELAYS: usize = 100;

/// Cache-TTL (1 Stunde)
pub const CACHE_TTL: Duration = Duration::from_secs(3600);

/// DHT-Query-Timeout
pub const DHT_QUERY_TIMEOUT: Duration = Duration::from_secs(10);

/// Minimum Trust-Score für Empfehlungen
pub const MIN_TRUST_SCORE: f64 = 0.5;

/// DHT-Key-Prefix für Relay-Listen
pub const DHT_RELAY_PREFIX: &str = "/erynoa/relays/v1";

// ============================================================================
// Recommended Relay
// ============================================================================

/// Empfohlener Relay mit Metadaten
#[derive(Debug, Clone)]
pub struct RecommendedRelay {
    /// Peer-ID (libp2p-kompatibel, 32 Bytes)
    pub peer_id: [u8; 32],
    /// Multiaddresses (serialisiert)
    pub multiaddrs: Vec<String>,
    /// Trust-Score (aus DHT aggregiert)
    pub trust_score: f64,
    /// Region/Jurisdiction
    pub region: String,
    /// Latenz-Schätzung (ms)
    pub estimated_latency_ms: u32,
    /// Kapazität (Messages/s)
    pub capacity: u32,
    /// Letztes Seen (Unix-Timestamp)
    pub last_seen: u64,
    /// Unterstützte Transports
    pub transports: Vec<String>,
    /// ASN (Autonomous System Number)
    pub asn: Option<u32>,
    /// Zusätzliche Metadaten
    pub metadata: HashMap<String, String>,
}

impl RecommendedRelay {
    /// Erstellt neuen empfohlenen Relay
    pub fn new(peer_id: [u8; 32], region: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            peer_id,
            multiaddrs: Vec::new(),
            trust_score: 0.5,
            region: region.to_string(),
            estimated_latency_ms: 100,
            capacity: 1000,
            last_seen: now,
            transports: vec!["quic".into(), "tcp".into()],
            asn: None,
            metadata: HashMap::new(),
        }
    }

    /// Setzt Trust-Score
    pub fn with_trust(mut self, score: f64) -> Self {
        self.trust_score = score.clamp(0.0, 1.0);
        self
    }

    /// Setzt Latenz
    pub fn with_latency(mut self, latency_ms: u32) -> Self {
        self.estimated_latency_ms = latency_ms;
        self
    }

    /// Setzt Kapazität
    pub fn with_capacity(mut self, capacity: u32) -> Self {
        self.capacity = capacity;
        self
    }

    /// Fügt Multiaddr hinzu
    pub fn with_multiaddr(mut self, addr: &str) -> Self {
        self.multiaddrs.push(addr.to_string());
        self
    }

    /// Setzt ASN
    pub fn with_asn(mut self, asn: u32) -> Self {
        self.asn = Some(asn);
        self
    }

    /// Berechnet Composite-Score für Ranking
    ///
    /// Score = Trust × Capacity / (1 + Latency/100)
    pub fn composite_score(&self) -> f64 {
        self.trust_score * self.capacity as f64 / (1.0 + self.estimated_latency_ms as f64 / 100.0)
    }

    /// Prüft ob Relay noch frisch ist (innerhalb TTL)
    pub fn is_fresh(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        now - self.last_seen < CACHE_TTL.as_secs()
    }

    /// Gibt Peer-ID als Hex zurück
    pub fn peer_id_hex(&self) -> String {
        hex::encode(self.peer_id)
    }
}

impl PartialEq for RecommendedRelay {
    fn eq(&self, other: &Self) -> bool {
        self.peer_id == other.peer_id
    }
}

impl Eq for RecommendedRelay {}

// ============================================================================
// Bootstrap Config
// ============================================================================

/// Bootstrap-Helper-Konfiguration
#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    /// Minimum Relays zum Starten
    pub min_relays: usize,
    /// Maximum Relays zu cachen
    pub max_cached_relays: usize,
    /// Cache-TTL (Sekunden)
    pub cache_ttl: Duration,
    /// Bevorzugte Regionen (optional)
    pub preferred_regions: Vec<String>,
    /// Trust-Minimum für Empfehlungen
    pub min_trust_score: f64,
    /// DHT-Query-Timeout
    pub dht_timeout: Duration,
    /// Fallback-Relays (hardcoded)
    pub fallback_relays: Vec<RecommendedRelay>,
    /// Diversitäts-Anforderung (verschiedene ASNs)
    pub require_diversity: bool,
    /// Minimum verschiedene ASNs
    pub min_diverse_asns: usize,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            min_relays: MIN_RELAYS_QUICK_START,
            max_cached_relays: MAX_CACHED_RELAYS,
            cache_ttl: CACHE_TTL,
            preferred_regions: Vec::new(),
            min_trust_score: MIN_TRUST_SCORE,
            dht_timeout: DHT_QUERY_TIMEOUT,
            fallback_relays: Vec::new(),
            require_diversity: true,
            min_diverse_asns: 3,
        }
    }
}

impl BootstrapConfig {
    /// Erstellt Config mit bevorzugter Region
    pub fn with_region(mut self, region: &str) -> Self {
        self.preferred_regions.push(region.to_uppercase());
        self
    }

    /// Erstellt Config mit mehreren Regionen
    pub fn with_regions(mut self, regions: &[&str]) -> Self {
        for region in regions {
            self.preferred_regions.push(region.to_uppercase());
        }
        self
    }

    /// Setzt Minimum Trust-Score
    pub fn with_min_trust(mut self, score: f64) -> Self {
        self.min_trust_score = score.clamp(0.0, 1.0);
        self
    }

    /// Fügt Fallback-Relay hinzu
    pub fn with_fallback(mut self, relay: RecommendedRelay) -> Self {
        self.fallback_relays.push(relay);
        self
    }
}

// ============================================================================
// Cache Entry
// ============================================================================

/// Cache-Eintrag mit Zeitstempel
#[derive(Debug, Clone)]
struct CacheEntry {
    relays: Vec<RecommendedRelay>,
    cached_at: Instant,
}

impl CacheEntry {
    fn new(relays: Vec<RecommendedRelay>) -> Self {
        Self {
            relays,
            cached_at: Instant::now(),
        }
    }

    fn is_valid(&self, ttl: Duration) -> bool {
        self.cached_at.elapsed() < ttl
    }
}

// ============================================================================
// DHT Client (Mock)
// ============================================================================

/// Mock DHT-Client für Tests
/// In Produktion: libp2p Kademlia DHT
#[derive(Debug)]
pub struct MockDhtClient {
    /// Gespeicherte Relay-Listen
    data: RwLock<HashMap<String, Vec<u8>>>,
}

impl MockDhtClient {
    /// Erstellt neuen Mock-Client
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }

    /// Speichert Daten im DHT
    pub fn put(&self, key: &str, value: Vec<u8>) {
        self.data.write().insert(key.to_string(), value);
    }

    /// Liest Daten aus DHT
    pub async fn get(&self, key: &str) -> Result<Vec<u8>, BootstrapError> {
        self.data
            .read()
            .get(key)
            .cloned()
            .ok_or(BootstrapError::DhtError("Key not found".into()))
    }
}

impl Default for MockDhtClient {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Bootstrap Helper
// ============================================================================

/// Bootstrap-Helper für Newcomer
pub struct BootstrapHelper {
    /// DHT-Client für Relay-Discovery
    dht_client: Arc<MockDhtClient>,
    /// Lokaler Cache (Region → Relays)
    relay_cache: RwLock<HashMap<String, CacheEntry>>,
    /// Discovery-Config
    config: BootstrapConfig,
    /// Discovery-History (für Debugging)
    discovery_history: RwLock<VecDeque<DiscoveryEvent>>,
    /// Statistiken
    stats: RwLock<BootstrapStats>,
}

/// Discovery-Event für History
#[derive(Debug, Clone)]
pub struct DiscoveryEvent {
    /// Zeitpunkt
    pub timestamp: Instant,
    /// Aktion
    pub action: String,
    /// Gefundene Relays
    pub relay_count: usize,
    /// Region
    pub region: Option<String>,
}

/// Bootstrap-Statistiken
#[derive(Debug, Clone, Default)]
pub struct BootstrapStats {
    /// Cache-Hits
    pub cache_hits: u64,
    /// Cache-Misses
    pub cache_misses: u64,
    /// DHT-Queries
    pub dht_queries: u64,
    /// Erfolgreiche Quick-Starts
    pub successful_bootstraps: u64,
    /// Fehlgeschlagene Quick-Starts
    pub failed_bootstraps: u64,
}

impl BootstrapHelper {
    /// Erstellt neuen Bootstrap-Helper
    pub fn new(config: BootstrapConfig) -> Self {
        Self {
            dht_client: Arc::new(MockDhtClient::new()),
            relay_cache: RwLock::new(HashMap::new()),
            config,
            discovery_history: RwLock::new(VecDeque::with_capacity(100)),
            stats: RwLock::new(BootstrapStats::default()),
        }
    }

    /// Erstellt Helper mit custom DHT-Client
    pub fn with_dht_client(mut self, client: Arc<MockDhtClient>) -> Self {
        self.dht_client = client;
        self
    }

    /// Entdeckt und empfiehlt Relays für Newcomer
    ///
    /// ## Discovery-Strategie
    /// 1. Check lokalen Cache
    /// 2. DHT-Query für "/erynoa/relays/v1/{region}"
    /// 3. Filter nach Trust + Region
    /// 4. Sortiere nach (Trust × Capacity / Latency)
    pub async fn discover_relays(&self) -> Result<Vec<RecommendedRelay>, BootstrapError> {
        // 1. Cache prüfen
        let cache_key = if self.config.preferred_regions.is_empty() {
            "global".to_string()
        } else {
            self.config.preferred_regions.join(",")
        };

        {
            let cache = self.relay_cache.read();
            if let Some(entry) = cache.get(&cache_key) {
                if entry.is_valid(self.config.cache_ttl)
                    && entry.relays.len() >= self.config.min_relays
                {
                    self.stats.write().cache_hits += 1;
                    self.log_event("cache_hit", entry.relays.len(), Some(&cache_key));
                    return Ok(entry.relays.clone());
                }
            }
        }

        self.stats.write().cache_misses += 1;

        // 2. DHT-Discovery
        let dht_key = format!(
            "{}/{}",
            DHT_RELAY_PREFIX,
            if self.config.preferred_regions.is_empty() {
                "global"
            } else {
                &self.config.preferred_regions[0]
            }
        );

        self.stats.write().dht_queries += 1;

        let relays = match self.dht_client.get(&dht_key).await {
            Ok(raw_data) => self.parse_relay_list(&raw_data)?,
            Err(_) => {
                // Fallback zu hardcoded Relays
                self.log_event(
                    "dht_failed_using_fallback",
                    self.config.fallback_relays.len(),
                    None,
                );
                self.config.fallback_relays.clone()
            }
        };

        // 3. Filter + Sort
        let filtered = self.filter_and_sort(relays);

        // 4. Cache aktualisieren
        {
            let mut cache = self.relay_cache.write();
            cache.insert(cache_key.clone(), CacheEntry::new(filtered.clone()));
        }

        self.log_event("discovery_complete", filtered.len(), Some(&cache_key));

        Ok(filtered)
    }

    /// Schnell-Start für Newcomer
    ///
    /// Liefert minimale, diverse Relay-Liste für sofortigen Start
    pub async fn quick_start(&self) -> Result<Vec<RecommendedRelay>, BootstrapError> {
        let relays = self.discover_relays().await?;

        // Wähle diverse Subset (verschiedene Regionen/AS)
        let diverse = self.select_diverse(&relays, self.config.min_relays);

        if diverse.len() < self.config.min_relays {
            self.stats.write().failed_bootstraps += 1;
            return Err(BootstrapError::InsufficientRelays {
                found: diverse.len(),
                required: self.config.min_relays,
            });
        }

        self.stats.write().successful_bootstraps += 1;
        self.log_event("quick_start_success", diverse.len(), None);

        Ok(diverse)
    }

    /// Filtert und sortiert Relays nach Konfiguration
    fn filter_and_sort(&self, relays: Vec<RecommendedRelay>) -> Vec<RecommendedRelay> {
        let mut filtered: Vec<_> = relays
            .into_iter()
            .filter(|r| r.trust_score >= self.config.min_trust_score)
            .filter(|r| r.is_fresh())
            .filter(|r| {
                self.config.preferred_regions.is_empty()
                    || self
                        .config
                        .preferred_regions
                        .iter()
                        .any(|pr| pr.eq_ignore_ascii_case(&r.region))
            })
            .collect();

        // Sortiere nach Composite-Score (absteigend)
        filtered.sort_by(|a, b| {
            b.composite_score()
                .partial_cmp(&a.composite_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        filtered.truncate(self.config.max_cached_relays);
        filtered
    }

    /// Wählt diverse Relays (verschiedene Regionen/ASNs)
    fn select_diverse(&self, relays: &[RecommendedRelay], count: usize) -> Vec<RecommendedRelay> {
        let mut result = Vec::with_capacity(count);
        let mut used_regions = std::collections::HashSet::new();
        let mut used_asns = std::collections::HashSet::new();

        // Phase 1: Maximale Diversität
        for relay in relays {
            let region_new = !used_regions.contains(&relay.region);
            let asn_new = relay.asn.map_or(true, |asn| !used_asns.contains(&asn));

            if region_new || asn_new {
                result.push(relay.clone());
                used_regions.insert(relay.region.clone());
                if let Some(asn) = relay.asn {
                    used_asns.insert(asn);
                }

                if result.len() >= count {
                    return result;
                }
            }
        }

        // Phase 2: Auffüllen mit besten verbleibenden
        for relay in relays {
            if !result.iter().any(|r| r.peer_id == relay.peer_id) {
                result.push(relay.clone());
                if result.len() >= count {
                    break;
                }
            }
        }

        result
    }

    /// Parsed Relay-Liste aus DHT-Bytes
    fn parse_relay_list(&self, data: &[u8]) -> Result<Vec<RecommendedRelay>, BootstrapError> {
        // Vereinfachtes JSON-Parsing
        // In Produktion: Protobuf oder CBOR
        let json_str =
            std::str::from_utf8(data).map_err(|e| BootstrapError::ParseError(e.to_string()))?;

        // Simples JSON-Array-Parsing für Demo
        if json_str.trim().is_empty() || json_str == "[]" {
            return Ok(Vec::new());
        }

        // Für echte Implementierung: serde_json
        Ok(Vec::new())
    }

    /// Fügt Relay zum Cache hinzu (für Tests)
    pub fn add_to_cache(&self, region: &str, relays: Vec<RecommendedRelay>) {
        let mut cache = self.relay_cache.write();
        cache.insert(region.to_string(), CacheEntry::new(relays));
    }

    /// Invalidiert Cache
    pub fn invalidate_cache(&self) {
        self.relay_cache.write().clear();
    }

    /// Gibt Statistiken zurück
    pub fn stats(&self) -> BootstrapStats {
        self.stats.read().clone()
    }

    /// Gibt Discovery-History zurück
    pub fn history(&self) -> Vec<DiscoveryEvent> {
        self.discovery_history.read().iter().cloned().collect()
    }

    /// Loggt Discovery-Event
    fn log_event(&self, action: &str, relay_count: usize, region: Option<&str>) {
        let mut history = self.discovery_history.write();
        if history.len() >= 100 {
            history.pop_front();
        }
        history.push_back(DiscoveryEvent {
            timestamp: Instant::now(),
            action: action.to_string(),
            relay_count,
            region: region.map(String::from),
        });
    }

    /// Gibt Konfiguration zurück
    pub fn config(&self) -> &BootstrapConfig {
        &self.config
    }
}

// ============================================================================
// DHT Relay Publisher
// ============================================================================

/// Publisher für Relay-Listen ins DHT
pub struct RelayPublisher {
    /// DHT-Client
    dht_client: Arc<MockDhtClient>,
    /// Publish-Intervall
    publish_interval: Duration,
    /// Letzte Veröffentlichung
    last_publish: RwLock<Option<Instant>>,
}

impl RelayPublisher {
    /// Erstellt neuen Publisher
    pub fn new(dht_client: Arc<MockDhtClient>) -> Self {
        Self {
            dht_client,
            publish_interval: Duration::from_secs(300), // 5 Minuten
            last_publish: RwLock::new(None),
        }
    }

    /// Prüft ob Publish fällig ist
    pub fn should_publish(&self) -> bool {
        self.last_publish
            .read()
            .map_or(true, |t| t.elapsed() >= self.publish_interval)
    }

    /// Veröffentlicht Relay-Liste
    pub async fn publish(
        &self,
        region: &str,
        relays: &[RecommendedRelay],
    ) -> Result<(), BootstrapError> {
        let key = format!("{}/{}", DHT_RELAY_PREFIX, region);

        // Serialize (vereinfacht - in Produktion: Protobuf/CBOR)
        let data = self.serialize_relays(relays)?;

        self.dht_client.put(&key, data);
        *self.last_publish.write() = Some(Instant::now());

        Ok(())
    }

    /// Serialisiert Relay-Liste
    fn serialize_relays(&self, relays: &[RecommendedRelay]) -> Result<Vec<u8>, BootstrapError> {
        // Vereinfachte Serialisierung
        // In Produktion: serde + protobuf
        let mut data = Vec::new();

        // Version Byte
        data.push(1);

        // Relay Count (u16)
        let count = relays.len().min(u16::MAX as usize) as u16;
        data.extend_from_slice(&count.to_be_bytes());

        for relay in relays.iter().take(count as usize) {
            // Peer ID (32 bytes)
            data.extend_from_slice(&relay.peer_id);

            // Trust Score (f64 als u64 bits)
            data.extend_from_slice(&relay.trust_score.to_bits().to_be_bytes());

            // Latency (u32)
            data.extend_from_slice(&relay.estimated_latency_ms.to_be_bytes());

            // Capacity (u32)
            data.extend_from_slice(&relay.capacity.to_be_bytes());

            // Region (length + bytes)
            let region_bytes = relay.region.as_bytes();
            data.push(region_bytes.len().min(255) as u8);
            data.extend_from_slice(&region_bytes[..region_bytes.len().min(255)]);
        }

        Ok(data)
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Bootstrap-Fehler
#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error("DHT error: {0}")]
    DhtError(String),

    #[error("Insufficient relays: found {found}, required {required}")]
    InsufficientRelays { found: usize, required: usize },

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Timeout during discovery")]
    Timeout,

    #[error("No fallback relays configured")]
    NoFallback,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_peer_id(n: u8) -> [u8; 32] {
        let mut id = [0u8; 32];
        id[0] = n;
        id[31] = n;
        id
    }

    fn test_relay(n: u8, region: &str) -> RecommendedRelay {
        RecommendedRelay::new(test_peer_id(n), region)
            .with_trust(0.7 + n as f64 * 0.01)
            .with_latency(50 + n as u32 * 10)
            .with_capacity(1000 - n as u32 * 50)
            .with_multiaddr(&format!("/ip4/192.168.1.{}/tcp/4001", n))
    }

    #[test]
    fn test_recommended_relay_creation() {
        let relay = RecommendedRelay::new(test_peer_id(1), "DE")
            .with_trust(0.8)
            .with_latency(50)
            .with_capacity(2000);

        assert_eq!(relay.region, "DE");
        assert_eq!(relay.trust_score, 0.8);
        assert_eq!(relay.estimated_latency_ms, 50);
        assert_eq!(relay.capacity, 2000);
        assert!(relay.is_fresh());
    }

    #[test]
    fn test_recommended_relay_composite_score() {
        let relay1 = RecommendedRelay::new(test_peer_id(1), "DE")
            .with_trust(0.8)
            .with_latency(50)
            .with_capacity(1000);

        let relay2 = RecommendedRelay::new(test_peer_id(2), "DE")
            .with_trust(0.6)
            .with_latency(100)
            .with_capacity(1000);

        // Higher trust + lower latency = higher score
        assert!(relay1.composite_score() > relay2.composite_score());
    }

    #[test]
    fn test_bootstrap_config_default() {
        let config = BootstrapConfig::default();

        assert_eq!(config.min_relays, MIN_RELAYS_QUICK_START);
        assert_eq!(config.max_cached_relays, MAX_CACHED_RELAYS);
        assert_eq!(config.min_trust_score, MIN_TRUST_SCORE);
        assert!(config.preferred_regions.is_empty());
    }

    #[test]
    fn test_bootstrap_config_builder() {
        let config = BootstrapConfig::default()
            .with_region("DE")
            .with_region("FR")
            .with_min_trust(0.7);

        assert_eq!(config.preferred_regions.len(), 2);
        assert!(config.preferred_regions.contains(&"DE".to_string()));
        assert_eq!(config.min_trust_score, 0.7);
    }

    #[tokio::test]
    async fn test_bootstrap_helper_cache() {
        let config = BootstrapConfig::default();
        let helper = BootstrapHelper::new(config);

        // Pre-populate cache
        let relays: Vec<_> = (1..=10).map(|n| test_relay(n, "DE")).collect();
        helper.add_to_cache("global", relays.clone());

        // Should hit cache
        let result = helper.discover_relays().await.unwrap();
        assert!(!result.is_empty());

        let stats = helper.stats();
        assert_eq!(stats.cache_hits, 1);
    }

    #[tokio::test]
    async fn test_bootstrap_helper_quick_start() {
        let config = BootstrapConfig::default();
        let helper = BootstrapHelper::new(config);

        // Pre-populate cache with enough relays
        let relays: Vec<_> = (1..=10).map(|n| test_relay(n, "DE")).collect();
        helper.add_to_cache("global", relays);

        let result = helper.quick_start().await.unwrap();
        assert_eq!(result.len(), MIN_RELAYS_QUICK_START);
    }

    #[tokio::test]
    async fn test_bootstrap_helper_insufficient_relays() {
        let config = BootstrapConfig::default();
        let helper = BootstrapHelper::new(config);

        // Pre-populate cache with too few relays
        let relays = vec![test_relay(1, "DE")];
        helper.add_to_cache("global", relays);

        let result = helper.quick_start().await;
        assert!(matches!(
            result,
            Err(BootstrapError::InsufficientRelays { .. })
        ));
    }

    #[test]
    fn test_bootstrap_helper_filter_by_trust() {
        let config = BootstrapConfig::default().with_min_trust(0.75);
        let helper = BootstrapHelper::new(config);

        let relays = vec![
            RecommendedRelay::new(test_peer_id(1), "DE").with_trust(0.8),
            RecommendedRelay::new(test_peer_id(2), "DE").with_trust(0.5), // Below threshold
            RecommendedRelay::new(test_peer_id(3), "DE").with_trust(0.9),
        ];

        let filtered = helper.filter_and_sort(relays);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.trust_score >= 0.75));
    }

    #[test]
    fn test_bootstrap_helper_filter_by_region() {
        let config = BootstrapConfig::default().with_region("DE");
        let helper = BootstrapHelper::new(config);

        let relays = vec![
            RecommendedRelay::new(test_peer_id(1), "DE").with_trust(0.8),
            RecommendedRelay::new(test_peer_id(2), "FR").with_trust(0.9), // Wrong region
            RecommendedRelay::new(test_peer_id(3), "de").with_trust(0.7), // Case-insensitive
        ];

        let filtered = helper.filter_and_sort(relays);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.region.eq_ignore_ascii_case("DE")));
    }

    #[test]
    fn test_bootstrap_helper_sort_by_score() {
        let config = BootstrapConfig::default();
        let helper = BootstrapHelper::new(config);

        let relays = vec![
            RecommendedRelay::new(test_peer_id(1), "DE")
                .with_trust(0.6)
                .with_capacity(1000)
                .with_latency(100),
            RecommendedRelay::new(test_peer_id(2), "DE")
                .with_trust(0.9)
                .with_capacity(2000)
                .with_latency(50),
            RecommendedRelay::new(test_peer_id(3), "DE")
                .with_trust(0.7)
                .with_capacity(1500)
                .with_latency(75),
        ];

        let sorted = helper.filter_and_sort(relays);

        // Relay 2 should be first (highest score)
        assert_eq!(sorted[0].peer_id, test_peer_id(2));

        // Verify descending order
        for i in 1..sorted.len() {
            assert!(sorted[i - 1].composite_score() >= sorted[i].composite_score());
        }
    }

    #[test]
    fn test_bootstrap_helper_select_diverse() {
        let config = BootstrapConfig::default();
        let helper = BootstrapHelper::new(config);

        let relays = vec![
            RecommendedRelay::new(test_peer_id(1), "DE").with_asn(1234),
            RecommendedRelay::new(test_peer_id(2), "DE").with_asn(1234), // Same ASN
            RecommendedRelay::new(test_peer_id(3), "FR").with_asn(5678),
            RecommendedRelay::new(test_peer_id(4), "US").with_asn(9012),
        ];

        let diverse = helper.select_diverse(&relays, 3);
        assert_eq!(diverse.len(), 3);

        // Should have different regions/ASNs
        let regions: std::collections::HashSet<_> = diverse.iter().map(|r| &r.region).collect();
        assert!(regions.len() >= 2);
    }

    #[test]
    fn test_bootstrap_helper_invalidate_cache() {
        let config = BootstrapConfig::default();
        let helper = BootstrapHelper::new(config);

        let relays = vec![test_relay(1, "DE")];
        helper.add_to_cache("global", relays);

        helper.invalidate_cache();

        // Cache should be empty now
        let cache = helper.relay_cache.read();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_entry_validity() {
        let relays = vec![test_relay(1, "DE")];
        let entry = CacheEntry::new(relays);

        assert!(entry.is_valid(Duration::from_secs(60)));

        // Note: Can't easily test expiration without time manipulation
    }

    #[test]
    fn test_relay_publisher_serialize() {
        let dht = Arc::new(MockDhtClient::new());
        let publisher = RelayPublisher::new(dht);

        let relays = vec![RecommendedRelay::new(test_peer_id(1), "DE")
            .with_trust(0.8)
            .with_latency(50)
            .with_capacity(1000)];

        let data = publisher.serialize_relays(&relays).unwrap();

        // Check format
        assert_eq!(data[0], 1); // Version
        assert_eq!(u16::from_be_bytes([data[1], data[2]]), 1); // Count
    }

    #[tokio::test]
    async fn test_relay_publisher_publish() {
        let dht = Arc::new(MockDhtClient::new());
        let publisher = RelayPublisher::new(Arc::clone(&dht));

        let relays = vec![test_relay(1, "DE")];

        publisher.publish("DE", &relays).await.unwrap();

        // Verify data in DHT
        let key = format!("{}/DE", DHT_RELAY_PREFIX);
        let stored = dht.get(&key).await.unwrap();
        assert!(!stored.is_empty());
    }

    #[test]
    fn test_relay_publisher_should_publish() {
        let dht = Arc::new(MockDhtClient::new());
        let publisher = RelayPublisher::new(dht);

        // Initially should publish
        assert!(publisher.should_publish());

        // After marking, should not publish immediately
        *publisher.last_publish.write() = Some(Instant::now());
        assert!(!publisher.should_publish());
    }

    #[test]
    fn test_bootstrap_error_display() {
        let err = BootstrapError::InsufficientRelays {
            found: 2,
            required: 5,
        };
        assert!(err.to_string().contains("2"));
        assert!(err.to_string().contains("5"));
    }

    #[test]
    fn test_recommended_relay_peer_id_hex() {
        let relay = RecommendedRelay::new(test_peer_id(0xAB), "DE");
        let hex = relay.peer_id_hex();
        assert!(hex.starts_with("ab"));
        assert!(hex.ends_with("ab"));
    }

    #[tokio::test]
    async fn test_bootstrap_helper_history() {
        let config = BootstrapConfig::default();
        let helper = BootstrapHelper::new(config);

        // Trigger some events
        let relays: Vec<_> = (1..=10).map(|n| test_relay(n, "DE")).collect();
        helper.add_to_cache("global", relays);
        let _ = helper.discover_relays().await;

        let history = helper.history();
        assert!(!history.is_empty());
    }

    #[test]
    fn test_recommended_relay_equality() {
        let relay1 = RecommendedRelay::new(test_peer_id(1), "DE");
        let relay2 = RecommendedRelay::new(test_peer_id(1), "FR"); // Same peer_id, different region
        let relay3 = RecommendedRelay::new(test_peer_id(2), "DE");

        assert_eq!(relay1, relay2); // Same peer_id
        assert_ne!(relay1, relay3); // Different peer_id
    }
}
