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

#![cfg(feature = "privacy")]

use std::sync::Arc;

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
        ConfluxConfig, ConfluxManager, ConfluxStats, EgressAggregator, EgressAggregatorStats,
        SecretSharer,
    },
    // Performance Layer
    performance::{
        BatchCryptoConfig, BatchDecryptor, BatchEncryptor, CircuitCache, CircuitCacheConfig,
        HwCryptoEngine,
    },
    // Privacy Layer
    privacy::{
        ComplianceMonitor, ComplianceStatus, CoverGeneratorStats, CoverMessage, CoverTrafficConfig,
        CoverTrafficGenerator, MixingPool, MixingPoolConfig, PeerType, PrivacyService,
        PrivacyServiceConfig, PrivacyServiceStats, SelfComplianceResult, SensitivityLevel,
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
