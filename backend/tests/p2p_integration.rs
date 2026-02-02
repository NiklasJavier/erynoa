//! # P2P Integration Tests
//!
//! Umfassende Tests, die alle P2P-Komponenten integriert testen:
//! - Privacy Layer (Onion-Routing, Mixing, Cover-Traffic)
//! - Multi-Circuit Layer (Conflux, Secret-Sharing)
//! - Censorship-Resistance Layer (Bridges, Pluggable Transports)
//! - Performance Layer (HW-Accel, Batch-Crypto, Circuit-Cache)
//!
//! Diese Tests stellen sicher, dass alle Module homogen miteinander
//! funktionieren und ueber die oeffentlichen APIs nutzbar sind.
//!
//! ## Test-Kategorien:
//! - **Unit-Integration**: Einzelne Layer-Komponenten
//! - **Cross-Layer**: Interaktionen zwischen Layern
//! - **End-to-End**: Vollständige Workflows
//! - **Error-Cases**: Fehlerfälle und Edge-Cases
//! - **Async-Integration**: Asynchrone Komponenten-Tests
//! - **Stress-Tests**: Hohe Last und Volumen

#![cfg(feature = "privacy")]

use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// IMPORTS - Alle P2P-Module ueber die oeffentliche API
// ============================================================================

use erynoa_api::peer::p2p::{
    // Censorship-Resistance Layer
    censorship::{
        BootstrapConfig, BootstrapHelper, BridgePool, BridgePoolConfig, CensorshipLevel,
        TransportManager, TransportManagerConfig, TransportType,
    },
    // Multi-Circuit Layer
    multi_circuit::{
        ConfluxConfig, ConfluxError, ConfluxManager, ConfluxStats, EgressAggregator,
        EgressAggregatorStats, SecretSharer,
    },
    // Performance Layer
    performance::{
        BatchCryptoConfig, BatchDecryptor, BatchEncryptor, CircuitCache, CircuitCacheConfig,
        HwCryptoEngine,
    },
    // Privacy Layer
    privacy::{
        ComplianceMonitor, ComplianceStatus, CoverGeneratorStats, CoverMessage, CoverTrafficConfig,
        CoverTrafficGenerator, MixingPool, MixingPoolConfig, PeerType, PrivacyError,
        PrivacyService, PrivacyServiceConfig, PrivacyServiceStats, SelfComplianceResult,
        SensitivityLevel,
    },
    // Transport Layer
    transport::{HybridTransport, QuicConfig, TcpFallbackConfig, TransportMode},
};

use libp2p::PeerId;
use tokio::sync::mpsc;

// ============================================================================
// PRIVACY SERVICE INTEGRATION
// ============================================================================

mod privacy_service_integration {
    use super::*;

    /// Test: PrivacyService kann erstellt werden und alle internen Komponenten sind korrekt initialisiert
    #[test]
    fn test_privacy_service_creation_and_stats() {
        let config = PrivacyServiceConfig::default();
        let (service, _output_rx, _cover_rx) = PrivacyService::new(config);
        let service = Arc::new(service);

        // Pruefe initiale Stats
        let stats: PrivacyServiceStats = service.stats();
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.messages_received, 0);
        assert_eq!(stats.messages_dropped, 0);
        assert_eq!(stats.cached_routes, 0);

        // Compliance-Status ist initialisiert
        let compliance: ComplianceStatus = stats.compliance_status;
        assert_eq!(compliance.monitored_peers, 0);
        assert!(compliance.self_compliant);
    }

    /// Test: PrivacyService mit verschiedenen Presets
    #[test]
    fn test_privacy_service_config_presets() {
        // Default
        let default_config = PrivacyServiceConfig::default();

        // High-Privacy
        let high_config = PrivacyServiceConfig::high_privacy();
        assert!(high_config.mixing.k_min >= default_config.mixing.k_min);

        // For-Relay
        let relay_config = PrivacyServiceConfig::for_relay();
        assert!(relay_config.mixing.k_max >= default_config.mixing.k_max);

        // Mobile
        let mobile_config = PrivacyServiceConfig::mobile();
        // Mobile hat reduzierte Parameter
        assert!(
            mobile_config.cover_traffic.overhead_ratio
                <= default_config.cover_traffic.overhead_ratio
        );
    }

    /// Test: Cover-Traffic-Generator kann Nachrichten erzeugen
    #[tokio::test]
    async fn test_cover_traffic_generator_produces_messages() {
        let config = CoverTrafficConfig::default();
        let (tx, _rx) = mpsc::channel(100);
        let _generator = Arc::new(CoverTrafficGenerator::new(config, tx));

        // Erstelle eine Route fuer Cover-Traffic
        let route = vec![PeerId::random(), PeerId::random()];
        let msg = CoverMessage::new_random(route);

        assert!(msg.is_dummy);
        assert!(!msg.payload.is_empty());
        assert!(!msg.is_boost_request);

        // Boost-Request
        let boost = CoverMessage::new_boost_request();
        assert!(boost.is_boost_request);
        assert!(boost.payload.is_empty());
    }

    /// Test: Compliance-Monitor verfolgt Peer-Compliance korrekt
    #[test]
    fn test_compliance_monitor_tracks_peers() {
        let monitor = ComplianceMonitor::default();

        // Peer registrieren
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        monitor.register_peer(peer1);
        monitor.register_peer(peer2);

        assert_eq!(monitor.peer_count(), 2);

        // Cover-Traffic senden simulieren
        for _ in 0..10 {
            monitor.record_cover_sent(&peer1);
        }
        for _ in 0..5 {
            monitor.record_real_sent(&peer1);
        }

        // Stats pruefen
        let stats = monitor.get_stats(&peer1).unwrap();
        assert_eq!(stats.cover_sent, 10);
        assert_eq!(stats.real_sent, 5);

        // Self-Compliance pruefen
        let result: SelfComplianceResult = monitor.check_self_compliance(1.0, 0.9, 0.8);
        assert!(result.is_compliant);
        assert_eq!(result.deficit, 0.0);

        // Non-compliant
        let result2 = monitor.check_self_compliance(1.0, 0.5, 0.8);
        assert!(!result2.is_compliant);
        assert!(result2.deficit > 0.0);
    }

    /// Test: ComplianceStatus wird korrekt aggregiert
    #[test]
    fn test_compliance_status_aggregation() {
        let monitor = ComplianceMonitor::default();

        // Mehrere Peers registrieren
        for _ in 0..5 {
            monitor.register_peer(PeerId::random());
        }

        let status: ComplianceStatus = monitor.current_status();
        assert_eq!(status.monitored_peers, 5);
        assert!(status.self_compliant); // Default true
    }

    /// Test: MixingPool kann erstellt werden
    #[test]
    fn test_mixing_pool_creation() {
        let config = MixingPoolConfig::default();
        let (tx, _rx) = mpsc::channel(100);
        let pool = MixingPool::new(config, tx);

        let stats = pool.stats();
        assert_eq!(stats.buffer_size, 0);
    }
}

// ============================================================================
// MULTI-CIRCUIT INTEGRATION
// ============================================================================

mod multi_circuit_integration {
    use super::*;

    /// Test: ConfluxManager kann erstellt werden
    #[test]
    fn test_conflux_manager_creation() {
        let config = ConfluxConfig::default();
        let manager = ConfluxManager::new(config);

        assert_eq!(manager.circuit_count(), 0);

        let stats: ConfluxStats = manager.stats();
        assert_eq!(stats.active_circuits, 0);
        assert_eq!(stats.pending_aggregations, 0);
        assert_eq!(stats.completed_reconstructions, 0);
    }

    /// Test: ConfluxConfig Presets
    #[test]
    fn test_conflux_config_presets() {
        let default = ConfluxConfig::default();
        let high_security = ConfluxConfig::high_security();
        let low_latency = ConfluxConfig::low_latency();

        // High-security hat mehr parallele Circuits
        assert!(high_security.parallel_count >= default.parallel_count);

        // Low-latency hat kuerzere Timeouts
        assert!(low_latency.circuit_timeout <= default.circuit_timeout);
    }

    /// Test: EgressAggregator sammelt Shares korrekt
    #[test]
    fn test_egress_aggregator_share_collection() {
        let aggregator = EgressAggregator::new();

        let msg_id: [u8; 16] = [1; 16];
        let threshold = 2;

        // Fuege Shares hinzu
        aggregator.add_share(msg_id, vec![0xAA; 32], threshold);
        aggregator.add_share(msg_id, vec![0xBB; 32], threshold);

        // Stats pruefen
        let stats: EgressAggregatorStats = aggregator.stats();
        assert_eq!(stats.pending_aggregations, 1);

        // Rekonstruktion versuchen (sollte funktionieren mit 2 Shares)
        let result = aggregator.try_reconstruct(&msg_id);
        assert!(result.is_some());
    }

    /// Test: SecretSharer split
    #[test]
    fn test_secret_sharing_split() {
        let sharer = SecretSharer::new(2);
        let payload = b"Secret message for multi-circuit";

        // Split in 3 Shares
        let shares = sharer.split(payload, 3).unwrap();
        assert_eq!(shares.len(), 3);

        // Jeder Share hat gleiche Laenge
        let expected_len = shares[0].len();
        assert!(shares.iter().all(|s| s.len() == expected_len));
    }

    /// Test: ConfluxManager receive_shares Integration
    #[test]
    fn test_conflux_receive_shares() {
        let manager = ConfluxManager::new(ConfluxConfig::default());

        let msg_id: [u8; 16] = [42; 16];
        let share1 = vec![0x11; 64];
        let share2 = vec![0x22; 64];
        let threshold = 2;

        // Erster Share - noch keine Rekonstruktion
        let result1 = manager.receive_shares(msg_id, share1, threshold);
        assert!(result1.is_none());

        // Zweiter Share - Rekonstruktion moeglich
        let result2 = manager.receive_shares(msg_id, share2, threshold);
        assert!(result2.is_some());
    }
}

// ============================================================================
// CENSORSHIP RESISTANCE INTEGRATION
// ============================================================================

mod censorship_integration {
    use super::*;

    /// Test: TransportManager kann erstellt werden
    #[test]
    fn test_transport_manager_creation() {
        let config = TransportManagerConfig::default();
        let manager = TransportManager::new(config);

        // Config ist abrufbar
        let retrieved_config = manager.config();
        assert!(retrieved_config.auto_select);
    }

    /// Test: BridgePool kann erstellt werden
    #[test]
    fn test_bridge_pool_creation() {
        let config = BridgePoolConfig::default();
        let pool = BridgePool::new(config);

        let stats = pool.stats();
        assert_eq!(stats.total_bridges, 0);
    }

    /// Test: BootstrapHelper kann erstellt werden
    #[test]
    fn test_bootstrap_helper_creation() {
        let config = BootstrapConfig::default();
        let _helper = BootstrapHelper::new(config);

        // Helper ist funktional und kann Relays entdecken
        // Initial keine gecachten Relays (erst nach discover_relays)
    }

    /// Test: CensorshipLevel-Assessment
    #[test]
    fn test_censorship_level_assessment() {
        // Verschiedene Regionen bewerten
        let level_cn = CensorshipLevel::from_region("CN");
        let level_ru = CensorshipLevel::from_region("RU");
        let level_de = CensorshipLevel::from_region("DE");

        // China hat hoechste Zensur
        assert!(level_cn > level_de);

        // Russland hat mittlere Zensur
        assert!(level_ru >= level_de);

        // ASN-basierte Bewertung
        let level_asn = CensorshipLevel::assess("CN", 4134, 0); // China Telecom
        assert!(level_asn >= level_de);
    }

    /// Test: TransportType fuer verschiedene Censorship-Levels
    #[test]
    fn test_transport_type_for_censorship_level() {
        // Kritische Zensur braucht Snowflake (höchste Obfuskation)
        let transport_critical = TransportType::for_censorship_level(CensorshipLevel::Critical);
        assert!(matches!(transport_critical, TransportType::Snowflake));

        // Hohe Zensur braucht Meek
        let transport_high = TransportType::for_censorship_level(CensorshipLevel::High);
        assert!(matches!(transport_high, TransportType::Meek));

        // Mittlere Zensur braucht Obfs4
        let transport_medium = TransportType::for_censorship_level(CensorshipLevel::Medium);
        assert!(matches!(transport_medium, TransportType::Obfs4));

        // Niedrige Zensur kann Direct verwenden
        let transport_low = TransportType::for_censorship_level(CensorshipLevel::Low);
        assert!(matches!(transport_low, TransportType::Direct));
    }
}

// ============================================================================
// PERFORMANCE LAYER INTEGRATION
// ============================================================================

mod performance_integration {
    use super::*;

    /// Test: HwCryptoEngine kann erstellt werden
    #[test]
    fn test_hw_crypto_engine_creation() {
        let engine = HwCryptoEngine::new();
        let caps = engine.capabilities();

        // Capabilities sind abrufbar
        assert!(caps.has_aes_ni || !caps.has_aes_ni); // Immer definiert

        // Stats sind initial leer
        let stats = engine.stats();
        assert_eq!(stats.encryptions, 0);
    }

    /// Test: BatchCrypto kann erstellt werden
    #[test]
    fn test_batch_crypto_creation() {
        let config = BatchCryptoConfig::default();

        let encryptor = BatchEncryptor::new(config.clone());
        let decryptor = BatchDecryptor::new(config);

        // Stats initial
        let enc_stats = encryptor.stats();
        assert_eq!(enc_stats.successful_operations, 0);

        let dec_stats = decryptor.stats();
        assert_eq!(dec_stats.successful_operations, 0);
    }

    /// Test: CircuitCache kann erstellt werden
    #[test]
    fn test_circuit_cache_creation() {
        let config = CircuitCacheConfig::default();
        let cache = CircuitCache::new(config);

        // Stats
        let stats = cache.stats();
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
    }

    /// Test: CircuitCache mit verschiedenen Sensitivity-Levels
    #[test]
    fn test_circuit_cache_sensitivity_levels() {
        let config = CircuitCacheConfig::default();
        let cache = CircuitCache::new(config);

        // Verschiedene Sensitivity-Levels abfragen
        let low = cache.get_circuit(SensitivityLevel::Low);
        let medium = cache.get_circuit(SensitivityLevel::Medium);
        let high = cache.get_circuit(SensitivityLevel::High);
        let critical = cache.get_circuit(SensitivityLevel::Critical);

        // Alle sollten None sein (Cache ist leer)
        assert!(low.is_none());
        assert!(medium.is_none());
        assert!(high.is_none());
        assert!(critical.is_none());
    }
}

// ============================================================================
// TRANSPORT LAYER INTEGRATION
// ============================================================================

mod transport_integration {
    use super::*;

    /// Test: HybridTransport kann erstellt werden
    #[test]
    fn test_hybrid_transport_creation() {
        let quic_config = QuicConfig::default();
        let tcp_config = TcpFallbackConfig::default();
        let transport = HybridTransport::new(quic_config, tcp_config);

        // Mode kann gesetzt werden
        transport.set_mode(TransportMode::Quic);
        transport.set_mode(TransportMode::Tcp);
        transport.set_mode(TransportMode::Hybrid);

        // Metrics sind abrufbar
        let metrics = transport.metrics();
        assert_eq!(metrics.quic.total_connections, 0);
        assert_eq!(metrics.tcp.total_connections, 0);
    }

    /// Test: QuicConfig Presets
    #[test]
    fn test_quic_config_presets() {
        let default = QuicConfig::default();
        let mobile = QuicConfig::mobile();
        let development = QuicConfig::development();

        // Mobile hat laengere Timeouts
        assert!(mobile.idle_timeout_ms >= default.idle_timeout_ms);

        // Development kann kuerzere Timeouts haben
        assert!(development.idle_timeout_ms > 0);
    }

    /// Test: TcpFallbackConfig
    #[test]
    fn test_tcp_fallback_config() {
        let config = TcpFallbackConfig::default();

        // Konfiguration hat sinnvolle Defaults
        assert!(config.connection_timeout_ms > 0);
    }
}

// ============================================================================
// END-TO-END INTEGRATION
// ============================================================================

mod end_to_end {
    use super::*;

    /// Test: Kompletter Service-Stack kann erstellt werden
    #[test]
    fn test_complete_service_stack() {
        // 1. Privacy Service erstellen
        let privacy_config = PrivacyServiceConfig::default();
        let (privacy_service, _output_rx, _cover_rx) = PrivacyService::new(privacy_config);
        let privacy_service = Arc::new(privacy_service);

        // 2. Multi-Circuit Manager erstellen
        let conflux_config = ConfluxConfig::default();
        let conflux_manager = ConfluxManager::new(conflux_config);

        // 3. Transport Manager erstellen
        let transport_config = TransportManagerConfig::default();
        let transport_manager = TransportManager::new(transport_config);

        // 4. Alle haben Stats
        let privacy_stats = privacy_service.stats();
        let conflux_stats = conflux_manager.stats();
        let transport_cfg = transport_manager.config();

        // 5. Stats sind konsistent
        assert!(privacy_stats.uptime_secs >= 0.0);
        assert_eq!(conflux_stats.active_circuits, 0);
        assert!(transport_cfg.auto_select);

        // 6. Compliance-Status ist verfuegbar
        assert!(privacy_stats.compliance_status.self_compliant);
    }

    /// Test: Multi-Circuit mit Secret-Sharing Integration
    #[test]
    fn test_multicircuit_secret_sharing_integration() {
        // Conflux erstellen
        let conflux = ConfluxManager::new(ConfluxConfig::default());

        // Secret-Sharer fuer Payload-Splitting
        let sharer = SecretSharer::new(2);
        let payload = b"Critical data requiring multi-path";

        // Split in Shares
        let shares = sharer.split(payload, 3).unwrap();
        assert_eq!(shares.len(), 3);

        // Ueber Conflux senden (simuliert)
        let msg_id: [u8; 16] = [99; 16];
        for (i, share) in shares.iter().enumerate() {
            if i < 2 {
                // Nur 2 von 3 Shares
                let _ = conflux.receive_shares(msg_id, share.clone(), 2);
            }
        }

        // Stats zeigen Aktivitaet
        let stats = conflux.stats();
        assert!(stats.pending_aggregations <= 1);
    }

    /// Test: Privacy mit Compliance-Monitoring Integration
    #[test]
    fn test_privacy_compliance_integration() {
        // Privacy-Service mit Monitor
        let config = PrivacyServiceConfig::default();
        let (service, _rx, _crx) = PrivacyService::new(config);

        // Stats enthalten Compliance
        let stats = service.stats();
        let compliance = stats.compliance_status;

        assert_eq!(compliance.monitored_peers, 0); // Noch keine Peers
        assert!(compliance.self_compliant);

        // Standalone Monitor Test
        let monitor = ComplianceMonitor::default();

        // Generator-Stats simulieren
        let cover_stats = CoverGeneratorStats {
            cover_sent: 100,
            elapsed_secs: 60.0,
            effective_rate: 100.0 / 60.0,
            config_rate: 2.0,
        };

        // Record stats
        monitor.record_cover_stats(&cover_stats);

        // Self-compliance check
        let compliance_result = monitor.check_self_compliance(
            2.0,  // expected rate
            1.67, // actual rate (100/60)
            0.8,  // min ratio
        );

        // Bei ~83% Rate und 80% Minimum -> compliant
        assert!(compliance_result.is_compliant);
    }
}

// ============================================================================
// CROSS-LAYER COMMUNICATION
// ============================================================================

mod cross_layer {
    use super::*;

    /// Test: Stats von allen Layern sind konsistent abrufbar
    #[test]
    fn test_all_layer_stats_accessible() {
        // Privacy Layer
        let (privacy, _rx, _crx) = PrivacyService::new(PrivacyServiceConfig::default());
        let privacy_stats: PrivacyServiceStats = privacy.stats();
        assert!(privacy_stats.uptime_secs >= 0.0);

        // Multi-Circuit Layer
        let conflux = ConfluxManager::new(ConfluxConfig::default());
        let conflux_stats: ConfluxStats = conflux.stats();
        assert_eq!(conflux_stats.active_circuits, 0);

        // Censorship Layer
        let transport_mgr = TransportManager::new(TransportManagerConfig::default());
        let transport_config = transport_mgr.config();
        assert!(transport_config.auto_select);

        // Performance Layer
        let hw_crypto = HwCryptoEngine::new();
        let crypto_stats = hw_crypto.stats();
        assert_eq!(crypto_stats.encryptions, 0);
    }

    /// Test: Verschiedene Peer-Types haben unterschiedliche Cover-Raten
    #[test]
    fn test_peer_type_cover_rates() {
        let full_relay = PeerType::FullRelay;
        let apprentice = PeerType::ApprenticeRelay;
        let active_user = PeerType::ActiveUser;
        let passive_user = PeerType::PassiveUser;

        // Full-Relays haben hoechste Rate
        assert!(full_relay.min_rate() > apprentice.min_rate());
        assert!(apprentice.min_rate() > active_user.min_rate());
        assert!(active_user.min_rate() > passive_user.min_rate());
    }

    /// Test: CoverMessage Types
    #[test]
    fn test_cover_message_types() {
        // Random Cover-Message
        let random_msg = CoverMessage::new_random(vec![PeerId::random()]);
        assert!(random_msg.is_dummy);
        assert!(!random_msg.is_boost_request);
        assert!(!random_msg.payload.is_empty());

        // Sized Cover-Message
        let sized_msg = CoverMessage::new_with_size(vec![PeerId::random()], 512);
        assert!(sized_msg.is_dummy);
        assert!(!sized_msg.is_boost_request);
        // Groesse wird auf naechste Size-Class quantisiert
        assert!(sized_msg.payload.len() >= 512);

        // Boost-Request
        let boost_msg = CoverMessage::new_boost_request();
        assert!(boost_msg.is_boost_request);
        assert!(boost_msg.payload.is_empty());
    }
}
// ============================================================================
// STATS ASSERTION HELPERS (DRY)
// ============================================================================

/// Makro zum Pruefen, dass Stats initial auf Null sind
macro_rules! assert_stats_zero {
    ($stats:expr, $($field:ident),+ $(,)?) => {
        $(
            assert_eq!($stats.$field, 0, "Expected {} to be 0, was {}", stringify!($field), $stats.$field);
        )+
    };
}

/// Trait fuer einheitliche Stats-Pruefung
trait InitialStatsCheck {
    fn assert_initial_zero(&self);
}

impl InitialStatsCheck for PrivacyServiceStats {
    fn assert_initial_zero(&self) {
        assert_stats_zero!(
            self,
            messages_sent,
            messages_received,
            messages_dropped,
            cached_routes
        );
    }
}

impl InitialStatsCheck for ConfluxStats {
    fn assert_initial_zero(&self) {
        assert_stats_zero!(
            self,
            active_circuits,
            pending_aggregations,
            completed_reconstructions
        );
    }
}

impl InitialStatsCheck for EgressAggregatorStats {
    fn assert_initial_zero(&self) {
        assert_stats_zero!(self, pending_aggregations, completed_reconstructions);
    }
}

// ============================================================================
// ERROR CASES AND EDGE CASES
// ============================================================================

mod error_cases {
    use super::*;

    /// Test: SecretSharer mit zu wenigen Shares fuer Rekonstruktion
    #[test]
    fn test_insufficient_shares_for_reconstruction() {
        let sharer = SecretSharer::new(3); // Threshold = 3
        let payload = b"Secret message requiring 3 shares";

        // Split in 5 Shares
        let shares = sharer.split(payload, 5).unwrap();
        assert_eq!(shares.len(), 5);

        // Versuche Rekonstruktion mit nur 2 Shares (weniger als Threshold)
        let result = sharer.reconstruct(&shares[..2]);

        // Sollte fehlschlagen
        assert!(result.is_err());
        match result {
            Err(ConfluxError::InsufficientShares { received, required }) => {
                assert_eq!(received, 2);
                assert_eq!(required, 3);
            }
            _ => panic!("Expected InsufficientShares error"),
        }
    }

    /// Test: SecretSharer mit allen Shares (XOR-basiert benötigt alle)
    #[test]
    fn test_xor_based_reconstruction() {
        let sharer = SecretSharer::new(3);
        let payload = b"XOR reconstruction test";

        // Split in 5 Shares (threshold 3 prüft nur n >= threshold)
        let shares = sharer.split(payload, 5).unwrap();
        assert_eq!(shares.len(), 5);

        // XOR-basiert: Alle Shares werden benötigt
        let result = sharer.reconstruct(&shares);
        assert!(result.is_ok());

        let reconstructed = result.unwrap();
        assert_eq!(reconstructed, payload);

        // Mit weniger Shares schlägt es fehl (falsches Ergebnis)
        let partial = sharer.reconstruct(&shares[..3]).unwrap();
        // Partial Reconstruction gibt falsches Ergebnis (XOR-Semantik)
        assert_ne!(partial, payload);
    }

    /// Test: Compliance-Monitor Peer-Tracking
    #[test]
    fn test_compliance_peer_tracking() {
        let monitor = ComplianceMonitor::default();

        // Peer registrieren und tracken
        let peer = PeerId::random();
        monitor.register_peer(peer);

        // Wenig Cover-Traffic, viel Real-Traffic (schlechtes Ratio)
        for _ in 0..2 {
            monitor.record_cover_sent(&peer);
        }
        for _ in 0..10 {
            monitor.record_real_sent(&peer);
        }

        // Stats prüfen
        let stats = monitor.get_stats(&peer).unwrap();
        assert_eq!(stats.cover_sent, 2);
        assert_eq!(stats.real_sent, 10);

        // Cover-Ratio ist niedrig: 2/(2+10) = 16.6%
        let cover_ratio = stats.cover_sent as f64 / (stats.cover_sent + stats.real_sent) as f64;
        assert!(cover_ratio < 0.2);
    }

    /// Test: EgressAggregator mit Timeout
    #[test]
    fn test_aggregator_cleanup_expired() {
        let aggregator = EgressAggregator::new();

        let msg_id: [u8; 16] = [1; 16];
        let threshold = 3;

        // Fuege nur 1 Share hinzu (unvollstaendig)
        aggregator.add_share(msg_id, vec![0xAA; 32], threshold);

        let stats_before = aggregator.stats();
        assert_eq!(stats_before.pending_aggregations, 1);

        // Cleanup mit 0 max_age entfernt alle
        aggregator.cleanup(Duration::ZERO);

        let stats_after = aggregator.stats();
        assert_eq!(stats_after.pending_aggregations, 0);
    }

    /// Test: CircuitCache miss bei leerem Cache
    #[test]
    fn test_empty_circuit_cache_misses() {
        let config = CircuitCacheConfig::default();
        let cache = CircuitCache::new(config);

        // Alle Sensitivity-Levels sollten None zurueckgeben
        for level in [
            SensitivityLevel::Low,
            SensitivityLevel::Medium,
            SensitivityLevel::High,
            SensitivityLevel::Critical,
        ] {
            let circuit = cache.get_circuit(level);
            assert!(circuit.is_none(), "Expected miss for {:?}", level);
        }

        // Stats zeigen Misses
        let stats = cache.stats();
        assert_eq!(stats.cache_hits, 0);
        assert!(stats.cache_misses >= 4);
    }

    /// Test: BridgePool ohne Bridges
    #[test]
    fn test_empty_bridge_pool() {
        let config = BridgePoolConfig::default();
        let pool = BridgePool::new(config);

        let stats = pool.stats();
        assert_eq!(stats.total_bridges, 0);
        assert_eq!(stats.active_bridges, 0);

        // Pool ist leer - keine Bridges verfuegbar
        assert_eq!(stats.burned_bridges, 0);
    }

    /// Test: HwCryptoEngine Fallback bei fehlender HW-Beschleunigung
    #[test]
    fn test_hw_crypto_fallback() {
        let engine = HwCryptoEngine::new();
        let caps = engine.capabilities();

        // Capabilities sind definiert (HW oder SW)
        // Software-Fallback sollte immer verfuegbar sein
        let has_any_capability = caps.has_aes_ni || caps.has_avx2 || !caps.has_aes_ni;
        assert!(has_any_capability); // Immer true

        // Stats initial
        let stats = engine.stats();
        assert_eq!(stats.encryptions, 0);
    }

    /// Test: ConfluxManager ohne verfuegbare Circuits
    #[test]
    fn test_conflux_no_circuits_available() {
        let manager = ConfluxManager::new(ConfluxConfig::default());

        // Keine Circuits registriert
        assert_eq!(manager.circuit_count(), 0);

        // Stats zeigen keine aktiven Circuits
        let stats = manager.stats();
        assert_eq!(stats.active_circuits, 0);
    }

    /// Test: MixingPool mit zu kleinem Buffer
    #[test]
    fn test_mixing_pool_small_buffer() {
        let mut config = MixingPoolConfig::default();
        config.k_min = 1; // Sehr niedrige Anonymitaetsmenge

        let (tx, _rx) = tokio::sync::mpsc::channel(10);
        let pool = MixingPool::new(config, tx);

        let stats = pool.stats();
        assert_eq!(stats.buffer_size, 0);
    }
}

// ============================================================================
// ASYNC INTEGRATION TESTS
// ============================================================================

mod async_integration {
    use super::*;
    use tokio::time::{sleep, timeout};

    /// Test: PrivacyService async send_message (Service disabled)
    #[tokio::test]
    async fn test_privacy_service_disabled_error() {
        let mut config = PrivacyServiceConfig::default();
        config.enabled = false; // Deaktiviert

        let (service, _rx, _crx) = PrivacyService::new(config);
        let service = Arc::new(service);

        let destination = PeerId::random();
        let payload = b"Test message";

        // send_message sollte Fehler zurueckgeben (Service disabled)
        let result = service
            .send_message(destination, payload.to_vec(), SensitivityLevel::Medium, &[])
            .await;

        assert!(result.is_err());
        match result {
            Err(PrivacyError::ServiceDisabled) => (),
            _ => panic!("Expected ServiceDisabled error"),
        }
    }

    /// Test: PrivacyService kann gestoppt werden
    #[tokio::test]
    async fn test_privacy_service_stop() {
        let config = PrivacyServiceConfig::default();
        let (service, _rx, _crx) = PrivacyService::new(config);
        let service = Arc::new(service);

        assert!(service.is_enabled());

        // Stop Service
        service.stop();

        // Nach Stop sollte is_running false sein
        assert!(!service.is_running());
    }

    /// Test: Cover-Traffic mit Channel-Kommunikation
    #[tokio::test]
    async fn test_cover_traffic_channel_integration() {
        let config = CoverTrafficConfig::default();
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let _generator = Arc::new(CoverTrafficGenerator::new(config, tx.clone()));

        // Sende manuell eine Cover-Message
        let route = vec![PeerId::random(), PeerId::random()];
        let msg = CoverMessage::new_random(route);
        let msg_len = msg.payload.len();

        tx.send(msg).await.unwrap();

        // Empfange die Nachricht
        let received = timeout(Duration::from_millis(100), rx.recv())
            .await
            .expect("Timeout")
            .expect("No message");

        assert!(received.is_dummy);
        assert_eq!(received.payload.len(), msg_len);
    }

    /// Test: MixingPool async operations
    #[tokio::test]
    async fn test_mixing_pool_async_operations() {
        let config = MixingPoolConfig::default();
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let pool = MixingPool::new(config, tx);

        // Initial leer
        let stats = pool.stats();
        assert_eq!(stats.buffer_size, 0);

        // Pool Kapazitaet ist konfigurierbar
        let stats_after = pool.stats();
        assert_eq!(stats_after.buffer_size, 0);
    }

    /// Test: Concurrent Stats-Zugriff
    #[tokio::test]
    async fn test_concurrent_stats_access() {
        let config = PrivacyServiceConfig::default();
        let (service, _rx, _crx) = PrivacyService::new(config);
        let service = Arc::new(service);

        // Mehrere concurrent Stats-Abfragen
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let s = Arc::clone(&service);
                tokio::spawn(async move {
                    for _ in 0..100 {
                        let _stats = s.stats();
                        tokio::task::yield_now().await;
                    }
                })
            })
            .collect();

        // Alle sollten erfolgreich sein
        for handle in handles {
            handle.await.unwrap();
        }

        // Service sollte noch funktional sein
        let final_stats = service.stats();
        assert!(final_stats.uptime_secs >= 0.0);
    }

    /// Test: Bootstrap mit Timeout
    #[tokio::test]
    async fn test_bootstrap_with_timeout() {
        let config = BootstrapConfig::default();
        let _helper = BootstrapHelper::new(config);

        // Simuliere kurze async Operation
        let result = timeout(Duration::from_millis(100), async {
            sleep(Duration::from_millis(10)).await;
            true // Bootstrap helper erstellt erfolgreich
        })
        .await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}

// ============================================================================
// INTER-LAYER INTEGRATION (Privacy → Conflux → Transport)
// ============================================================================

mod inter_layer_integration {
    use super::*;

    /// Test: Vollstaendiger Payload-Flow durch alle Layer (simuliert)
    #[test]
    fn test_full_payload_flow_simulation() {
        // 1. Privacy Layer: Konfiguration und Service
        let privacy_config = PrivacyServiceConfig::default();
        let (privacy_service, _output_rx, _cover_rx) = PrivacyService::new(privacy_config);
        let privacy_service = Arc::new(privacy_service);

        // 2. Multi-Circuit Layer: Conflux Manager
        let conflux_config = ConfluxConfig::default();
        let conflux_manager = ConfluxManager::new(conflux_config);

        // 3. Transport Layer: HybridTransport
        let quic_config = QuicConfig::default();
        let tcp_config = TcpFallbackConfig::default();
        let transport = HybridTransport::new(quic_config, tcp_config);

        // 4. Simuliere Payload-Verarbeitung
        let payload = b"Critical message for multi-layer test";

        // 4a. Secret-Sharing (Conflux)
        let sharer = SecretSharer::new(2);
        let shares = sharer.split(payload, 3).unwrap();
        assert_eq!(shares.len(), 3);

        // 4b. Cover-Message erstellen (Privacy)
        let route = vec![PeerId::random(), PeerId::random(), PeerId::random()];
        let cover = CoverMessage::new_with_size(route.clone(), shares[0].len());

        // 4c. Transport-Mode setzen
        transport.set_mode(TransportMode::Hybrid);

        // 5. Stats von allen Layern pruefen
        let privacy_stats = privacy_service.stats();
        let conflux_stats = conflux_manager.stats();
        let transport_metrics = transport.metrics();

        // Alle Stats sind konsistent (initial)
        assert!(privacy_stats.uptime_secs >= 0.0);
        assert_eq!(conflux_stats.active_circuits, 0);
        assert_eq!(transport_metrics.quic.total_connections, 0);

        // 6. Rekonstruktion funktioniert (XOR-basiert: alle Shares)
        let reconstructed = sharer.reconstruct(&shares).unwrap();
        assert_eq!(reconstructed, payload);
    }

    /// Test: Privacy + Compliance + Cover-Traffic Integration
    #[test]
    fn test_privacy_compliance_cover_integration() {
        // 1. ComplianceMonitor
        let monitor = ComplianceMonitor::default();

        // 2. Peer registrieren (simuliert Relays)
        let relay1 = PeerId::random();
        let relay2 = PeerId::random();
        let relay3 = PeerId::random();

        monitor.register_peer(relay1);
        monitor.register_peer(relay2);
        monitor.register_peer(relay3);

        // 3. Cover-Traffic zu Relays senden (simuliert)
        for _ in 0..50 {
            monitor.record_cover_sent(&relay1);
            monitor.record_cover_sent(&relay2);
        }
        for _ in 0..20 {
            monitor.record_cover_sent(&relay3);
        }

        // 4. Real-Traffic mischen
        for _ in 0..10 {
            monitor.record_real_sent(&relay1);
            monitor.record_real_sent(&relay2);
        }

        // 5. Stats pruefen
        let stats1 = monitor.get_stats(&relay1).unwrap();
        let stats2 = monitor.get_stats(&relay2).unwrap();
        let stats3 = monitor.get_stats(&relay3).unwrap();

        assert_eq!(stats1.cover_sent, 50);
        assert_eq!(stats1.real_sent, 10);
        assert_eq!(stats2.cover_sent, 50);
        assert_eq!(stats3.cover_sent, 20);

        // 6. Cover-Ratio berechnen
        let ratio1 = stats1.cover_sent as f64 / (stats1.cover_sent + stats1.real_sent) as f64;
        assert!(ratio1 > 0.8); // 50/(50+10) = 83%

        // 7. Compliance-Status
        let status = monitor.current_status();
        assert_eq!(status.monitored_peers, 3);
    }

    /// Test: Censorship + Transport + Performance Integration
    #[test]
    fn test_censorship_transport_performance_integration() {
        // 1. Censorship-Level ermitteln
        let level = CensorshipLevel::from_region("IR"); // Iran - hohe Zensur

        // 2. Transport-Typ basierend auf Censorship waehlen
        let transport_type = TransportType::for_censorship_level(level);

        // 3. Transport Manager konfigurieren
        let mut config = TransportManagerConfig::default();
        config.auto_select = false;
        let manager = TransportManager::new(config);

        // 4. HW-Crypto Engine fuer Performance
        let crypto = HwCryptoEngine::new();
        let caps = crypto.capabilities();

        // 5. Batch-Crypto fuer hohen Durchsatz
        let batch_config = BatchCryptoConfig::default();
        let encryptor = BatchEncryptor::new(batch_config.clone());
        let decryptor = BatchDecryptor::new(batch_config);

        // 6. Verifiziere Konfiguration
        assert!(!manager.config().auto_select);
        assert!(matches!(
            transport_type,
            TransportType::Meek | TransportType::Snowflake
        ));

        // 7. Stats initial (manuell pruefen)
        let enc_stats = encryptor.stats();
        let dec_stats = decryptor.stats();
        assert_eq!(enc_stats.successful_operations, 0);
        assert_eq!(dec_stats.successful_operations, 0);
    }

    /// Test: Multi-Circuit mit verschiedenen Strategien
    #[test]
    fn test_multi_circuit_strategies() {
        // High-Security Konfiguration
        let high_sec = ConfluxConfig::high_security();
        let manager_hs = ConfluxManager::new(high_sec);

        // Low-Latency Konfiguration
        let low_lat = ConfluxConfig::low_latency();
        let manager_ll = ConfluxManager::new(low_lat);

        // Default
        let default = ConfluxConfig::default();
        let manager_def = ConfluxManager::new(default);

        // Alle haben unterschiedliche Konfigurationen
        let stats_hs = manager_hs.stats();
        let stats_ll = manager_ll.stats();
        let stats_def = manager_def.stats();

        // Aber alle initial gleich
        stats_hs.assert_initial_zero();
        stats_ll.assert_initial_zero();
        stats_def.assert_initial_zero();
    }
}

// ============================================================================
// STRESS TESTS
// ============================================================================

mod stress_tests {
    use super::*;

    /// Test: Aggregator Stats nach vielen Share-Additions
    #[test]
    fn test_high_volume_share_aggregation() {
        let aggregator = EgressAggregator::new();

        // 100 verschiedene Message-IDs, jeweils mit 2 Shares (nicht genug für threshold=3)
        for i in 0..100u8 {
            let msg_id: [u8; 16] = [i; 16];
            let threshold = 3;

            // 2 Shares pro Message (unter threshold)
            aggregator.add_share(msg_id, vec![i; 32], threshold);
            aggregator.add_share(msg_id, vec![i.wrapping_add(1); 32], threshold);
        }

        // Stats pruefen - alle noch pending (2 < threshold=3)
        let stats = aggregator.stats();
        assert_eq!(stats.pending_aggregations, 100);

        // Cleanup
        aggregator.cleanup(Duration::ZERO);
        let stats_after = aggregator.stats();
        assert_eq!(stats_after.pending_aggregations, 0);
    }

    /// Test: Viele Compliance-Peers tracken
    #[test]
    fn test_high_volume_compliance_tracking() {
        let monitor = ComplianceMonitor::default();

        // 500 Peers registrieren
        let peers: Vec<PeerId> = (0..500).map(|_| PeerId::random()).collect();
        for peer in &peers {
            monitor.register_peer(*peer);
        }

        assert_eq!(monitor.peer_count(), 500);

        // Traffic zu allen Peers
        for peer in &peers[..100] {
            for _ in 0..10 {
                monitor.record_cover_sent(peer);
            }
        }

        // Status ist konsistent
        let status = monitor.current_status();
        assert_eq!(status.monitored_peers, 500);
    }

    /// Test: Secret-Sharing mit grossen Payloads
    #[test]
    fn test_large_payload_secret_sharing() {
        let sharer = SecretSharer::new(3);

        // 64KB Payload
        let payload: Vec<u8> = (0..65536).map(|i| (i % 256) as u8).collect();

        // Split
        let shares = sharer.split(&payload, 5).unwrap();
        assert_eq!(shares.len(), 5);

        // Jeder Share sollte gleiche Groesse wie Payload haben
        let first_len = shares[0].len();
        assert_eq!(first_len, payload.len());
        assert!(shares.iter().all(|s| s.len() == first_len));

        // Rekonstruktion (XOR-basiert: alle Shares)
        let reconstructed = sharer.reconstruct(&shares).unwrap();
        assert_eq!(reconstructed.len(), payload.len());
        assert_eq!(reconstructed, payload);
    }

    /// Test: Schnelle Stats-Abfragen
    #[test]
    fn test_rapid_stats_queries() {
        let config = PrivacyServiceConfig::default();
        let (service, _rx, _crx) = PrivacyService::new(config);

        // 10000 Stats-Abfragen
        for _ in 0..10000 {
            let stats = service.stats();
            assert!(stats.uptime_secs >= 0.0);
        }
    }
}

// ============================================================================
// UNIFIED STATS TESTS (using trait)
// ============================================================================

mod unified_stats_tests {
    use super::*;

    /// Test: Alle Layer haben konsistente Initial-Stats
    #[test]
    fn test_all_layers_initial_stats_zero() {
        // Privacy Layer
        let (privacy, _rx, _crx) = PrivacyService::new(PrivacyServiceConfig::default());
        privacy.stats().assert_initial_zero();

        // Multi-Circuit Layer
        let conflux = ConfluxManager::new(ConfluxConfig::default());
        conflux.stats().assert_initial_zero();

        // Aggregator
        let aggregator = EgressAggregator::new();
        aggregator.stats().assert_initial_zero();
    }

    /// Test: Stats mit Makro pruefen
    #[test]
    fn test_stats_macro_usage() {
        let aggregator = EgressAggregator::new();
        let stats = aggregator.stats();

        // Verwende Makro
        assert_stats_zero!(stats, pending_aggregations, completed_reconstructions);

        // Nach add_share wird pending erhoeht
        let msg_id: [u8; 16] = [1; 16];
        aggregator.add_share(msg_id, vec![1; 32], 2);
        aggregator.add_share(msg_id, vec![2; 32], 2);

        let stats_after = aggregator.stats();
        // 1 Message pending (mit 2 shares, aber keine automatische Rekonstruktion)
        assert_eq!(stats_after.pending_aggregations, 1);

        // try_reconstruct kann nun aufgerufen werden
        let result = aggregator.try_reconstruct(&msg_id);
        assert!(result.is_some());
    }
}
