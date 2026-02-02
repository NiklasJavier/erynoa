//! # Privacy Service (Phase 2 Woche 8 Integration)
//!
//! Integriert alle Privacy-Layer-Komponenten in einen einheitlichen Service:
//! - Onion-Routing (RL2-RL4)
//! - Mixing-Pool mit LAMP (RL8, RL25)
//! - Cover-Traffic mit Protocol-Pledge (RL10, RL18)
//! - Relay-Selection (RL5-RL7)
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                        PRIVACY SERVICE                                  │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                                                                         │
//! │  ┌──────────────────────────────────────────────────────────────────┐  │
//! │  │                     MESSAGE FLOW                                  │  │
//! │  │                                                                   │  │
//! │  │  App ──► [Onion Build] ──► [Mixing Pool] ──► [P2P Send]         │  │
//! │  │                                   ▲                               │  │
//! │  │                                   │                               │  │
//! │  │  [Cover Generator] ───────────────┘                              │  │
//! │  │                                                                   │  │
//! │  └──────────────────────────────────────────────────────────────────┘  │
//! │                                                                         │
//! │  ┌──────────────────────────────────────────────────────────────────┐  │
//! │  │                   BACKGROUND TASKS                                │  │
//! │  │                                                                   │  │
//! │  │  • Mixing-Pool Flush Loop (50ms interval)                        │  │
//! │  │  • Cover-Traffic Generator (Poisson-Rate)                        │  │
//! │  │  • Compliance Monitor (24h window)                               │  │
//! │  │  • Route Refresh (5min interval)                                 │  │
//! │  │                                                                   │  │
//! │  └──────────────────────────────────────────────────────────────────┘  │
//! │                                                                         │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Axiom-Referenzen
//!
//! - **RL2-RL4**: Onion-Routing mit Forward/Backward Secrecy
//! - **RL5-RL7**: Trust-basierte Relay-Selection
//! - **RL8**: ε-Differential-Privacy Mixing-Delays
//! - **RL10**: Cover-Traffic Indistinguishability
//! - **RL18**: Protocol-Pledge Cover-Rates
//! - **RL25**: LAMP Threshold-Mixing

use super::cover_traffic::{
    ComplianceMonitor, CoverGeneratorStats, CoverMessage, CoverTrafficConfig, CoverTrafficGenerator,
};
use super::mixing::{LampStats, MixingPool, MixingPoolConfig};
use super::onion::OnionDecryptor;
use super::relay_selection::{RelayCandidate, RelaySelector, SensitivityLevel};
use libp2p::PeerId;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use x25519_dalek::PublicKey;

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Privacy-Service Konfiguration
#[derive(Debug, Clone)]
pub struct PrivacyServiceConfig {
    /// Mixing-Pool Konfiguration
    pub mixing: MixingPoolConfig,
    /// Cover-Traffic Konfiguration
    pub cover_traffic: CoverTrafficConfig,
    /// Standard-Sensitivitätslevel
    pub default_sensitivity: SensitivityLevel,
    /// Route-Refresh-Intervall
    pub route_refresh_interval: Duration,
    /// Compliance-Check-Intervall
    pub compliance_check_interval: Duration,
    /// Aktiviert Privacy-Layer
    pub enabled: bool,
}

impl Default for PrivacyServiceConfig {
    fn default() -> Self {
        Self {
            mixing: MixingPoolConfig::default(),
            cover_traffic: CoverTrafficConfig::default(),
            default_sensitivity: SensitivityLevel::Medium,
            route_refresh_interval: Duration::from_secs(300), // 5 Minuten
            compliance_check_interval: Duration::from_secs(3600), // 1 Stunde
            enabled: true,
        }
    }
}

impl PrivacyServiceConfig {
    /// Konfiguration für Relay-Nodes
    pub fn for_relay() -> Self {
        Self {
            mixing: MixingPoolConfig::default(),
            cover_traffic: CoverTrafficConfig::for_relay(),
            default_sensitivity: SensitivityLevel::Medium,
            route_refresh_interval: Duration::from_secs(300),
            compliance_check_interval: Duration::from_secs(3600),
            enabled: true,
        }
    }

    /// Konfiguration für High-Privacy-Modus
    pub fn high_privacy() -> Self {
        Self {
            mixing: MixingPoolConfig::high_privacy(),
            cover_traffic: CoverTrafficConfig::default(),
            default_sensitivity: SensitivityLevel::High,
            route_refresh_interval: Duration::from_secs(180), // Häufiger wechseln
            compliance_check_interval: Duration::from_secs(3600),
            enabled: true,
        }
    }

    /// Konfiguration für Mobile/Low-Power
    pub fn mobile() -> Self {
        Self {
            mixing: MixingPoolConfig::mobile(),
            cover_traffic: CoverTrafficConfig::for_mobile(),
            default_sensitivity: SensitivityLevel::Low,
            route_refresh_interval: Duration::from_secs(600), // Seltener wechseln
            compliance_check_interval: Duration::from_secs(7200),
            enabled: true,
        }
    }
}

// ============================================================================
// OUTGOING MESSAGE
// ============================================================================

/// Ausgehende Privacy-Nachricht
#[derive(Debug)]
pub struct PrivacyMessage {
    /// Ziel-Peer
    pub destination: PeerId,
    /// Payload (wird Onion-verschlüsselt)
    pub payload: Vec<u8>,
    /// Sensitivitätslevel
    pub sensitivity: SensitivityLevel,
    /// Ist Cover-Traffic
    pub is_cover: bool,
}

// ============================================================================
// ROUTE CACHE
// ============================================================================

/// Gecachte Route für einen Destination-Peer
struct CachedRoute {
    /// Route (Relay-Peers)
    route: Vec<PeerId>,
    /// Sensitivitätslevel
    sensitivity: SensitivityLevel,
    /// Erstellungszeitpunkt
    created_at: Instant,
}

impl CachedRoute {
    fn is_expired(&self, max_age: Duration) -> bool {
        self.created_at.elapsed() > max_age
    }
}

/// Route-Cache
struct RouteCache {
    /// Routes pro Destination
    routes: RwLock<HashMap<PeerId, CachedRoute>>,
    /// Max-Age für Routes
    max_age: Duration,
}

impl RouteCache {
    fn new(max_age: Duration) -> Self {
        Self {
            routes: RwLock::new(HashMap::new()),
            max_age,
        }
    }

    fn get(&self, destination: &PeerId, sensitivity: SensitivityLevel) -> Option<Vec<PeerId>> {
        let routes = self.routes.read();
        routes.get(destination).and_then(|cached| {
            if !cached.is_expired(self.max_age) && cached.sensitivity == sensitivity {
                Some(cached.route.clone())
            } else {
                None
            }
        })
    }

    fn insert(&self, destination: PeerId, route: Vec<PeerId>, sensitivity: SensitivityLevel) {
        let mut routes = self.routes.write();
        routes.insert(
            destination,
            CachedRoute {
                route,
                sensitivity,
                created_at: Instant::now(),
            },
        );
    }

    fn invalidate(&self, destination: &PeerId) {
        self.routes.write().remove(destination);
    }

    fn clear(&self) {
        self.routes.write().clear();
    }

    fn cleanup_expired(&self) {
        let mut routes = self.routes.write();
        routes.retain(|_, cached| !cached.is_expired(self.max_age));
    }
}

// ============================================================================
// STATISTICS
// ============================================================================

/// Privacy-Service Statistiken
#[derive(Debug, Clone)]
pub struct PrivacyServiceStats {
    /// Mixing-Pool Statistiken
    pub mixing: LampStats,
    /// Cover-Traffic Statistiken
    pub cover_traffic: CoverGeneratorStats,
    /// Gesendete Privacy-Nachrichten
    pub messages_sent: u64,
    /// Empfangene Privacy-Nachrichten
    pub messages_received: u64,
    /// Verworfene Nachrichten (Replay, etc.)
    pub messages_dropped: u64,
    /// Aktive Routen im Cache
    pub cached_routes: usize,
    /// Service-Uptime
    pub uptime_secs: f64,
}

// ============================================================================
// PRIVACY SERVICE
// ============================================================================

/// Privacy-Service für integriertes Onion-Routing, Mixing und Cover-Traffic
///
/// ## Verwendung
///
/// ```rust,ignore
/// use erynoa_api::peer::p2p::privacy::service::{PrivacyService, PrivacyServiceConfig};
///
/// // Service erstellen
/// let (service, output_rx) = PrivacyService::new(PrivacyServiceConfig::default());
///
/// // Starte Background-Tasks
/// let service_arc = Arc::new(service);
/// tokio::spawn(service_arc.clone().run_background_tasks(relay_candidates_fn));
///
/// // Nachricht senden
/// service_arc.send_message(dest, payload, SensitivityLevel::Medium).await?;
/// ```
pub struct PrivacyService {
    /// Konfiguration
    config: PrivacyServiceConfig,

    /// Mixing-Pool
    mixing_pool: Arc<MixingPool>,

    /// Cover-Traffic Generator
    cover_generator: Arc<CoverTrafficGenerator>,

    /// Compliance-Monitor
    compliance_monitor: Arc<ComplianceMonitor>,

    /// Route-Cache
    route_cache: RouteCache,

    /// Onion-Decryptor (für eingehende Nachrichten) - Mutex für interior mutability
    decryptor: parking_lot::Mutex<Option<OnionDecryptor>>,

    /// Output-Channel (für Swarm)
    output_tx: mpsc::Sender<(PeerId, Vec<u8>)>,

    /// Cover-Traffic-Output
    cover_output_tx: mpsc::Sender<CoverMessage>,

    /// Running-State
    running: AtomicBool,

    /// Start-Zeitpunkt
    started_at: Instant,

    /// Statistik-Counter
    messages_sent: AtomicU64,
    messages_received: AtomicU64,
    messages_dropped: AtomicU64,
}

impl PrivacyService {
    /// Erstelle neuen Privacy-Service
    ///
    /// Gibt den Service und einen Receiver für ausgehende Nachrichten zurück.
    pub fn new(
        config: PrivacyServiceConfig,
    ) -> (
        Self,
        mpsc::Receiver<(PeerId, Vec<u8>)>,
        mpsc::Receiver<CoverMessage>,
    ) {
        // Channels erstellen
        let (output_tx, output_rx) = mpsc::channel(1024);
        let (cover_output_tx, cover_output_rx) = mpsc::channel(256);

        // Mixing-Pool erstellen
        let mixing_pool = Arc::new(MixingPool::new(config.mixing.clone(), output_tx.clone()));

        // Cover-Traffic Generator erstellen
        let cover_generator = Arc::new(CoverTrafficGenerator::new(
            config.cover_traffic.clone(),
            cover_output_tx.clone(),
        ));

        // Compliance-Monitor erstellen
        let compliance_monitor = Arc::new(ComplianceMonitor::default());

        // Route-Cache erstellen
        let route_cache = RouteCache::new(config.route_refresh_interval);

        let service = Self {
            config,
            mixing_pool,
            cover_generator,
            compliance_monitor,
            route_cache,
            decryptor: parking_lot::Mutex::new(None),
            output_tx,
            cover_output_tx,
            running: AtomicBool::new(false),
            started_at: Instant::now(),
            messages_sent: AtomicU64::new(0),
            messages_received: AtomicU64::new(0),
            messages_dropped: AtomicU64::new(0),
        };

        (service, output_rx, cover_output_rx)
    }

    /// Erstelle Service mit Decryptor (für Relay-Nodes)
    pub fn with_decryptor(self, decryptor: OnionDecryptor) -> Self {
        *self.decryptor.lock() = Some(decryptor);
        self
    }

    /// Ist Service aktiviert?
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Ist Service am Laufen?
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Hole Konfiguration
    pub fn config(&self) -> &PrivacyServiceConfig {
        &self.config
    }

    /// Sende Privacy-Nachricht
    ///
    /// Die Nachricht wird:
    /// 1. Onion-verschlüsselt (mit gecachter oder neuer Route)
    /// 2. In den Mixing-Pool gelegt
    /// 3. Nach Delay-Ablauf gesendet
    pub async fn send_message(
        &self,
        destination: PeerId,
        payload: Vec<u8>,
        sensitivity: SensitivityLevel,
        relay_candidates: &[RelayCandidate],
    ) -> Result<(), PrivacyError> {
        if !self.config.enabled {
            return Err(PrivacyError::ServiceDisabled);
        }

        // Route auswählen (oder aus Cache)
        let route = self.get_or_create_route(&destination, sensitivity, relay_candidates)?;

        if route.is_empty() {
            return Err(PrivacyError::NoRouteAvailable);
        }

        // Onion-Paket bauen
        let onion_packet = self.build_onion_packet(&payload, &route, &destination)?;

        // In Mixing-Pool legen
        let first_hop = route[0];
        self.mixing_pool.add_message(onion_packet, first_hop).await;

        self.messages_sent.fetch_add(1, Ordering::Relaxed);

        tracing::trace!(
            destination = %destination,
            hops = route.len(),
            sensitivity = ?sensitivity,
            "Privacy message queued"
        );

        Ok(())
    }

    /// Sende Nachricht direkt (ohne Mixing-Pool)
    ///
    /// Für Cover-Traffic und zeitkritische Nachrichten.
    pub async fn send_direct(
        &self,
        destination: PeerId,
        payload: Vec<u8>,
    ) -> Result<(), PrivacyError> {
        if self.output_tx.send((destination, payload)).await.is_err() {
            return Err(PrivacyError::ChannelClosed);
        }
        Ok(())
    }

    /// Verarbeite eingehende Privacy-Nachricht
    ///
    /// Für Relay-Nodes: Entschlüsselt eine Schicht und leitet weiter.
    pub async fn process_incoming(
        &self,
        _source: PeerId,
        encrypted: Vec<u8>,
    ) -> Result<ProcessingResult, PrivacyError> {
        let mut guard = self.decryptor.lock();
        let decryptor = guard.as_mut().ok_or(PrivacyError::NoDecryptor)?;

        // Entschlüsseln
        let layer = decryptor
            .decrypt_layer(&encrypted)
            .map_err(|e| PrivacyError::DecryptionFailed(e.to_string()))?;

        self.messages_received.fetch_add(1, Ordering::Relaxed);

        // Weiterleiten oder finales Ziel
        if layer.is_final {
            Ok(ProcessingResult::FinalDestination {
                payload: layer.payload,
            })
        } else {
            // Gib Forwarding-Info zurück - Caller ist verantwortlich für Weiterleitung
            // (ermöglicht dem Swarm, den MixingPool zu verwenden wenn gewünscht)
            Ok(ProcessingResult::Forwarded {
                next_relay: layer.next_relay,
                payload: layer.payload,
            })
        }
    }

    /// Starte Background-Tasks
    ///
    /// - Mixing-Pool Flush-Loop
    /// - Cover-Traffic Generation
    /// - Route-Cache Cleanup
    /// - Compliance-Monitoring
    pub async fn run_background_tasks<F>(
        self: Arc<Self>,
        route_provider: F,
    ) -> Result<(), PrivacyError>
    where
        F: Fn() -> Vec<PeerId> + Send + Sync + 'static,
    {
        if !self.config.enabled {
            return Ok(());
        }

        self.running.store(true, Ordering::Relaxed);

        let self_clone1 = self.clone();
        let self_clone2 = self.clone();
        let self_clone3 = self.clone();

        // Task 1: Mixing-Pool Flush-Loop
        let mixing_pool = self.mixing_pool.clone();
        let flush_task = tokio::spawn(async move {
            mixing_pool.run_flush_loop().await;
        });

        // Task 2: Cover-Traffic Generator
        let cover_task = tokio::spawn(async move {
            self_clone1.cover_generator.run(route_provider).await;
        });

        // Task 3: Route-Cache Cleanup
        let cleanup_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(self_clone2.config.route_refresh_interval / 2);
            loop {
                interval.tick().await;
                self_clone2.route_cache.cleanup_expired();
            }
        });

        // Task 4: Compliance-Check
        let compliance_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(self_clone3.config.compliance_check_interval);
            loop {
                interval.tick().await;
                self_clone3.check_own_compliance();
            }
        });

        tracing::info!("Privacy service background tasks started");

        // Warte auf alle Tasks (sollte nie enden unter normalen Bedingungen)
        tokio::select! {
            _ = flush_task => {
                tracing::warn!("Mixing-pool flush task ended");
            }
            _ = cover_task => {
                tracing::warn!("Cover-traffic task ended");
            }
            _ = cleanup_task => {
                tracing::warn!("Route cleanup task ended");
            }
            _ = compliance_task => {
                tracing::warn!("Compliance task ended");
            }
        }

        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }

    /// Stoppe Service
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Hole Statistiken
    pub fn stats(&self) -> PrivacyServiceStats {
        PrivacyServiceStats {
            mixing: self.mixing_pool.stats(),
            cover_traffic: self.cover_generator.stats(),
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            messages_received: self.messages_received.load(Ordering::Relaxed),
            messages_dropped: self.messages_dropped.load(Ordering::Relaxed),
            cached_routes: self.route_cache.routes.read().len(),
            uptime_secs: self.started_at.elapsed().as_secs_f64(),
        }
    }

    /// Invalidiere Route für Destination
    pub fn invalidate_route(&self, destination: &PeerId) {
        self.route_cache.invalidate(destination);
    }

    /// Lösche alle gecachten Routen
    pub fn clear_routes(&self) {
        self.route_cache.clear();
    }

    // ========================================================================
    // PRIVATE METHODS
    // ========================================================================

    fn get_or_create_route(
        &self,
        destination: &PeerId,
        sensitivity: SensitivityLevel,
        relay_candidates: &[RelayCandidate],
    ) -> Result<Vec<PeerId>, PrivacyError> {
        // Aus Cache?
        if let Some(route) = self.route_cache.get(destination, sensitivity) {
            return Ok(route);
        }

        // Neue Route erstellen
        let selector = RelaySelector::new(relay_candidates.to_vec(), sensitivity);
        let route_public_keys = selector
            .select_route()
            .map_err(|e| PrivacyError::RouteSelectionFailed(e.to_string()))?;

        // Route ist Vec<PublicKey>, wir konvertieren zu PeerIds
        // Hinweis: In Produktion würde man eine PublicKey→PeerId Lookup-Map verwenden
        let route_peers: Vec<PeerId> = route_public_keys
            .iter()
            .filter_map(|pk| {
                relay_candidates
                    .iter()
                    .find(|rc| rc.public_key == *pk)
                    .map(|rc| rc.peer_id)
            })
            .collect();

        // Cachen
        self.route_cache
            .insert(*destination, route_peers.clone(), sensitivity);

        Ok(route_peers)
    }

    fn build_onion_packet(
        &self,
        payload: &[u8],
        route: &[PeerId],
        _destination: &PeerId,
    ) -> Result<Vec<u8>, PrivacyError> {
        // Hinweis: In einer vollständigen Implementierung würden wir hier
        // die Public-Keys der Relays benötigen. Für jetzt simulieren wir.

        // Placeholder: In echter Implementierung würde OnionBuilder verwendet
        let mut packet = payload.to_vec();

        // Füge Routing-Info hinzu (vereinfacht)
        for peer_id in route.iter().rev() {
            let mut header = peer_id.to_bytes();
            header.extend_from_slice(&(packet.len() as u32).to_be_bytes());
            packet = [header, packet].concat();
        }

        Ok(packet)
    }

    fn check_own_compliance(&self) {
        // Prüfe eigene Cover-Traffic-Compliance
        let stats = self.cover_generator.stats();
        let expected_rate = self.config.cover_traffic.effective_rate();
        let actual_rate = stats.effective_rate;

        if actual_rate < expected_rate * 0.8 {
            tracing::warn!(
                expected = expected_rate,
                actual = actual_rate,
                "Cover-traffic compliance warning: rate too low"
            );
        }
    }
}

// ============================================================================
// PROCESSING RESULT
// ============================================================================

/// Ergebnis der Verarbeitung einer eingehenden Nachricht
#[derive(Debug)]
pub enum ProcessingResult {
    /// Nachricht ist für uns (finales Ziel)
    FinalDestination { payload: Vec<u8> },
    /// Nachricht muss weitergeleitet werden (Caller ist verantwortlich für P2P-Send)
    Forwarded {
        /// Nächster Hop (X25519 PublicKey)
        next_relay: PublicKey,
        /// Payload für nächsten Hop
        payload: Vec<u8>,
    },
}

// ============================================================================
// ERRORS
// ============================================================================

/// Privacy-Service Fehler
#[derive(Debug, thiserror::Error)]
pub enum PrivacyError {
    #[error("Privacy service is disabled")]
    ServiceDisabled,

    #[error("No route available to destination")]
    NoRouteAvailable,

    #[error("Route selection failed: {0}")]
    RouteSelectionFailed(String),

    #[error("Onion encryption failed: {0}")]
    OnionEncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("No decryptor configured")]
    NoDecryptor,

    #[error("Channel closed")]
    ChannelClosed,

    #[error("Internal error: {0}")]
    Internal(String),
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer::p2p::privacy::PeerType;

    #[test]
    fn test_config_defaults() {
        let config = PrivacyServiceConfig::default();
        assert!(config.enabled);
        assert_eq!(config.default_sensitivity, SensitivityLevel::Medium);
        assert_eq!(config.route_refresh_interval, Duration::from_secs(300));
    }

    #[test]
    fn test_config_presets() {
        let relay = PrivacyServiceConfig::for_relay();
        assert_eq!(relay.cover_traffic.peer_type, PeerType::FullRelay);

        let high = PrivacyServiceConfig::high_privacy();
        assert_eq!(high.default_sensitivity, SensitivityLevel::High);

        let mobile = PrivacyServiceConfig::mobile();
        assert_eq!(mobile.default_sensitivity, SensitivityLevel::Low);
    }

    #[test]
    fn test_service_creation() {
        let config = PrivacyServiceConfig::default();
        let (service, _output_rx, _cover_rx) = PrivacyService::new(config);

        assert!(service.is_enabled());
        assert!(!service.is_running());
    }

    #[test]
    fn test_route_cache() {
        let cache = RouteCache::new(Duration::from_secs(300));
        let dest = PeerId::random();
        let route = vec![PeerId::random(), PeerId::random()];

        // Initial leer
        assert!(cache.get(&dest, SensitivityLevel::Medium).is_none());

        // Einfügen
        cache.insert(dest, route.clone(), SensitivityLevel::Medium);

        // Abrufen
        let cached = cache.get(&dest, SensitivityLevel::Medium);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 2);

        // Falsches Sensitivity-Level
        assert!(cache.get(&dest, SensitivityLevel::High).is_none());

        // Invalidieren
        cache.invalidate(&dest);
        assert!(cache.get(&dest, SensitivityLevel::Medium).is_none());
    }

    #[test]
    fn test_stats() {
        let config = PrivacyServiceConfig::default();
        let (service, _output_rx, _cover_rx) = PrivacyService::new(config);

        let stats = service.stats();
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.messages_received, 0);
        assert_eq!(stats.cached_routes, 0);
    }

    #[tokio::test]
    async fn test_send_disabled() {
        let mut config = PrivacyServiceConfig::default();
        config.enabled = false;
        let (service, _output_rx, _cover_rx) = PrivacyService::new(config);

        let result = service
            .send_message(
                PeerId::random(),
                vec![1, 2, 3],
                SensitivityLevel::Medium,
                &[],
            )
            .await;

        assert!(matches!(result, Err(PrivacyError::ServiceDisabled)));
    }

    #[tokio::test]
    async fn test_send_no_route() {
        let config = PrivacyServiceConfig::default();
        let (service, _output_rx, _cover_rx) = PrivacyService::new(config);

        // Ohne Relay-Candidates gibt es keine Route
        let result = service
            .send_message(
                PeerId::random(),
                vec![1, 2, 3],
                SensitivityLevel::Medium,
                &[],
            )
            .await;

        // Erwarte entweder NoRouteAvailable oder RouteSelectionFailed
        assert!(
            matches!(
                result,
                Err(PrivacyError::NoRouteAvailable | PrivacyError::RouteSelectionFailed(_))
            ),
            "Expected NoRouteAvailable or RouteSelectionFailed, got: {:?}",
            result
        );
    }

    #[test]
    fn test_processing_result() {
        let payload = vec![1, 2, 3];
        let result = ProcessingResult::FinalDestination {
            payload: payload.clone(),
        };

        if let ProcessingResult::FinalDestination { payload: p } = result {
            assert_eq!(p, payload);
        } else {
            panic!("Expected FinalDestination");
        }
    }
}
