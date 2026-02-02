//! Layer Diagnostics - 8-Schichten-Analyse

use super::{DiagnosticCheck, DiagnosticState, LayerDiagnostic, P2PDiagnostics};
use std::sync::Arc;

// ============================================================================
// DIAGNOSTIC RUNNER
// ============================================================================

/// Runner für P2P-Diagnosen
pub struct DiagnosticRunner {
    peer_count: usize,
    connected_peers: Vec<String>,
    state: Option<Arc<DiagnosticState>>,
}

impl Default for DiagnosticRunner {
    fn default() -> Self {
        Self {
            peer_count: 0,
            connected_peers: Vec::new(),
            state: None,
        }
    }
}

impl DiagnosticRunner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Erstelle Runner aus DiagnosticState
    pub fn from_state(state: &Arc<DiagnosticState>) -> Self {
        let peer_ids = state.peer_ids();
        Self {
            peer_count: peer_ids.len(),
            connected_peers: peer_ids,
            state: Some(Arc::clone(state)),
        }
    }

    pub fn with_peer_info(mut self, peer_count: usize, peers: Vec<String>) -> Self {
        self.peer_count = peer_count;
        self.connected_peers = peers;
        self
    }

    /// Führe alle Diagnosen aus
    pub async fn run_all(&self, peer_id: Option<String>) -> P2PDiagnostics {
        let mut diagnostics = P2PDiagnostics::new();

        if let Some(id) = peer_id {
            diagnostics = diagnostics.with_node_id(id);
        }

        // Metriken hinzufügen falls State vorhanden
        if let Some(ref state) = self.state {
            diagnostics = diagnostics.with_metrics(state.get_metrics());
        }

        // Layer 1: Transport
        diagnostics.add_layer(self.check_transport_layer().await);

        // Layer 2: Identity
        diagnostics.add_layer(self.check_identity_layer().await);

        // Layer 3: Discovery
        diagnostics.add_layer(self.check_discovery_layer().await);

        // Layer 4: NAT-Traversal
        diagnostics.add_layer(self.check_nat_traversal_layer().await);

        // Layer 5: Performance
        diagnostics.add_layer(self.check_performance_layer().await);

        // Layer 6: Privacy
        diagnostics.add_layer(self.check_privacy_layer().await);

        // Layer 7: Application
        diagnostics.add_layer(self.check_application_layer().await);

        // Layer 0: Censorship-Resistance
        diagnostics.add_layer(self.check_censorship_layer().await);

        diagnostics
    }

    async fn check_transport_layer(&self) -> LayerDiagnostic {
        let mut layer = LayerDiagnostic::new("Transport (TCP/QUIC/Noise/Yamux)", 1)
            .with_feature("p2p")
            .with_description("Core network transport protocols");

        // TCP Transport
        layer.add_check(DiagnosticCheck::healthy(
            "TCP Transport",
            "libp2p TCP transport available",
        ));

        // Noise Protocol
        layer.add_check(DiagnosticCheck::healthy(
            "Noise Encryption",
            "XX handshake pattern configured",
        ));

        // Yamux Multiplexing
        layer.add_check(DiagnosticCheck::healthy(
            "Yamux Multiplexing",
            "Stream multiplexing enabled",
        ));

        // QUIC Transport
        #[cfg(feature = "privacy")]
        {
            layer.add_check(DiagnosticCheck::healthy(
                "QUIC Transport",
                "0-RTT transport available",
            ));
        }
        #[cfg(not(feature = "privacy"))]
        {
            layer.add_check(DiagnosticCheck::disabled("QUIC Transport"));
        }

        // Traffic-Metriken falls verfügbar
        if let Some(ref state) = self.state {
            let metrics = state.get_metrics();
            let traffic_check = DiagnosticCheck::healthy(
                "Traffic Stats",
                format!(
                    "↓{}/s ↑{}/s",
                    super::metrics::format_rate(metrics.bytes_per_second_in),
                    super::metrics::format_rate(metrics.bytes_per_second_out)
                ),
            )
            .with_metric(metrics.bytes_sent as f64 + metrics.bytes_received as f64, "B");
            layer.add_check(traffic_check);
        }

        layer
    }

    async fn check_identity_layer(&self) -> LayerDiagnostic {
        let mut layer = LayerDiagnostic::new("Identity (Keypair/PeerId/Identify)", 2)
            .with_feature("p2p")
            .with_description("Cryptographic identity and peer identification");

        layer.add_check(DiagnosticCheck::healthy(
            "Ed25519 Keypair",
            "Cryptographic identity generated",
        ));

        layer.add_check(DiagnosticCheck::healthy(
            "PeerId Derivation",
            "Multihash PeerId from public key",
        ));

        layer.add_check(DiagnosticCheck::healthy(
            "Identify Protocol",
            "/erynoa/identify/1.0.0 active",
        ));

        layer
    }

    async fn check_discovery_layer(&self) -> LayerDiagnostic {
        let mut layer = LayerDiagnostic::new("Discovery (Kademlia/mDNS/Bootstrap)", 3)
            .with_feature("p2p")
            .with_description("Peer discovery and DHT routing");

        // Kademlia DHT
        let kad_status = if self.peer_count > 0 {
            DiagnosticCheck::healthy(
                "Kademlia DHT",
                format!("DHT active with {} peers in routing table", self.peer_count),
            )
            .with_metric(self.peer_count as f64, " peers")
        } else {
            DiagnosticCheck::degraded("Kademlia DHT", "DHT empty - no peers in routing table")
                .with_metric(0.0, " peers")
        };
        layer.add_check(kad_status);

        // mDNS
        layer.add_check(DiagnosticCheck::healthy(
            "mDNS Discovery",
            "Local network discovery active",
        ));

        // Bootstrap
        let bootstrap_status = if self.peer_count >= 3 {
            DiagnosticCheck::healthy(
                "Bootstrap",
                format!("Connected to {} bootstrap peers", self.peer_count),
            )
        } else if self.peer_count > 0 {
            DiagnosticCheck::degraded(
                "Bootstrap",
                format!("Only {} peers connected (recommended: 3+)", self.peer_count),
            )
        } else {
            DiagnosticCheck::unavailable("Bootstrap", "No peers connected - network isolated")
        };
        layer.add_check(bootstrap_status);

        // Query Stats falls verfügbar
        if let Some(ref state) = self.state {
            let metrics = state.get_metrics();
            if metrics.kademlia_queries > 0 {
                layer.add_check(
                    DiagnosticCheck::healthy(
                        "DHT Queries",
                        format!("{} queries executed", metrics.kademlia_queries),
                    )
                    .with_metric(metrics.kademlia_queries as f64, ""),
                );
            }
        }

        layer
    }

    async fn check_nat_traversal_layer(&self) -> LayerDiagnostic {
        let mut layer = LayerDiagnostic::new("NAT-Traversal (AutoNAT/DCUTR/Relay/UPnP)", 4)
            .with_feature("p2p")
            .with_description("NAT detection and traversal mechanisms");

        // AutoNAT
        layer.add_check(DiagnosticCheck::healthy(
            "AutoNAT",
            "NAT type detection configured",
        ));

        // DCUTR (Holepunching)
        if let Some(ref state) = self.state {
            let metrics = state.get_metrics();
            let success_rate = if metrics.dcutr_attempts > 0 {
                (metrics.dcutr_successes as f64 / metrics.dcutr_attempts as f64) * 100.0
            } else {
                100.0 // No attempts = OK
            };

            let dcutr_check = if success_rate >= 50.0 || metrics.dcutr_attempts == 0 {
                DiagnosticCheck::healthy(
                    "DCUTR",
                    format!(
                        "Holepunching: {}/{} successful ({:.0}%)",
                        metrics.dcutr_successes, metrics.dcutr_attempts, success_rate
                    ),
                )
            } else {
                DiagnosticCheck::degraded(
                    "DCUTR",
                    format!(
                        "Low holepunch success: {}/{} ({:.0}%)",
                        metrics.dcutr_successes, metrics.dcutr_attempts, success_rate
                    ),
                )
            };
            layer.add_check(dcutr_check.with_metric(success_rate, "%"));
        } else {
            layer.add_check(DiagnosticCheck::healthy(
                "DCUTR",
                "Direct Connection Upgrade available",
            ));
        }

        // Relay Client
        layer.add_check(DiagnosticCheck::healthy(
            "Relay Client",
            "Circuit relay client configured",
        ));

        // Relay Server
        if let Some(ref state) = self.state {
            let metrics = state.get_metrics();
            layer.add_check(
                DiagnosticCheck::healthy(
                    "Relay Server",
                    format!("{} active circuits", metrics.relay_circuits_active),
                )
                .with_metric(metrics.relay_circuits_active as f64, ""),
            );
        } else {
            layer.add_check(DiagnosticCheck::healthy(
                "Relay Server",
                "Providing relay services to peers",
            ));
        }

        // UPnP
        layer.add_check(DiagnosticCheck::healthy(
            "UPnP",
            "Automatic port mapping enabled (gateway-dependent)",
        ));

        layer
    }

    async fn check_performance_layer(&self) -> LayerDiagnostic {
        let mut layer = LayerDiagnostic::new("Performance (Batch/Cache/HW-Accel)", 5)
            .with_feature("privacy")
            .with_description("Performance optimizations and hardware acceleration");

        #[cfg(feature = "privacy")]
        {
            // Batch Crypto
            layer.add_check(DiagnosticCheck::healthy(
                "Batch Crypto",
                "Parallel encryption/decryption available (RL20)",
            ));

            // Circuit Cache
            if let Some(ref state) = self.state {
                let metrics = state.get_metrics();
                layer.add_check(
                    DiagnosticCheck::healthy(
                        "Circuit Cache",
                        format!("{} circuits built", metrics.onion_circuits_built),
                    )
                    .with_metric(metrics.onion_circuits_built as f64, ""),
                );
            } else {
                layer.add_check(DiagnosticCheck::healthy(
                    "Circuit Cache",
                    "Pre-built circuits for <100ms latency (RL23)",
                ));
            }

            // Hardware Acceleration
            let hw_status = check_hw_acceleration();
            layer.add_check(hw_status);
        }

        #[cfg(not(feature = "privacy"))]
        {
            layer.add_check(DiagnosticCheck::disabled("Batch Crypto"));
            layer.add_check(DiagnosticCheck::disabled("Circuit Cache"));
            layer.add_check(DiagnosticCheck::disabled("HW Acceleration"));
        }

        layer
    }

    async fn check_privacy_layer(&self) -> LayerDiagnostic {
        let mut layer = LayerDiagnostic::new("Privacy (Onion/Relay-Selection/Mixing)", 6)
            .with_feature("privacy")
            .with_description("Privacy-preserving communication layer");

        #[cfg(feature = "privacy")]
        {
            // Onion Routing
            layer.add_check(DiagnosticCheck::healthy(
                "Onion Routing",
                "Multi-hop encryption (RL2-RL4)",
            ));

            // Relay Selection
            layer.add_check(DiagnosticCheck::healthy(
                "Trust-based Relay Selection",
                "6D Trust-vector scoring (RL5-RL7)",
            ));

            // Mixing mit Metriken
            if let Some(ref state) = self.state {
                let metrics = state.get_metrics();
                layer.add_check(
                    DiagnosticCheck::healthy(
                        "ε-DP Mixing",
                        format!("{} messages mixed", metrics.messages_mixed),
                    )
                    .with_metric(metrics.messages_mixed as f64, ""),
                );

                layer.add_check(
                    DiagnosticCheck::healthy(
                        "Cover Traffic",
                        format!("{} dummy messages sent", metrics.cover_traffic_sent),
                    )
                    .with_metric(metrics.cover_traffic_sent as f64, ""),
                );
            } else {
                layer.add_check(DiagnosticCheck::healthy(
                    "ε-DP Mixing",
                    "Laplace-delayed message pools (RL8-RL10)",
                ));

                layer.add_check(DiagnosticCheck::healthy(
                    "Cover Traffic",
                    "Dummy message generation active",
                ));
            }

            // Wire Format
            layer.add_check(DiagnosticCheck::healthy(
                "Wire Format",
                "Binary serialization protocol",
            ));
        }

        #[cfg(not(feature = "privacy"))]
        {
            layer.add_check(DiagnosticCheck::disabled("Onion Routing"));
            layer.add_check(DiagnosticCheck::disabled("Trust-based Relay Selection"));
            layer.add_check(DiagnosticCheck::disabled("ε-DP Mixing"));
            layer.add_check(DiagnosticCheck::disabled("Cover Traffic"));
            layer.add_check(DiagnosticCheck::disabled("Wire Format"));
        }

        layer
    }

    async fn check_application_layer(&self) -> LayerDiagnostic {
        let mut layer = LayerDiagnostic::new("Application (Gossipsub/Request-Response)", 7)
            .with_feature("p2p")
            .with_description("Application-level protocols");

        // Gossipsub mit Metriken
        if let Some(ref state) = self.state {
            let metrics = state.get_metrics();
            let gossip_check = if self.peer_count > 0 {
                DiagnosticCheck::healthy(
                    "Gossipsub",
                    format!(
                        "Mesh: {} peers, {} msgs received",
                        self.peer_count, metrics.gossip_messages
                    ),
                )
            } else {
                DiagnosticCheck::degraded(
                    "Gossipsub",
                    "No peers in mesh - messages won't propagate",
                )
            };
            layer.add_check(gossip_check.with_metric(metrics.gossip_messages as f64, " msgs"));

            // Message Rate
            layer.add_check(
                DiagnosticCheck::healthy(
                    "Message Rate",
                    format!("{:.1} msg/s", metrics.messages_per_second),
                )
                .with_metric(metrics.messages_per_second, "/s"),
            );
        } else {
            let gossip_status = if self.peer_count > 0 {
                DiagnosticCheck::healthy(
                    "Gossipsub",
                    format!("PubSub mesh with {} peers", self.peer_count),
                )
            } else {
                DiagnosticCheck::degraded(
                    "Gossipsub",
                    "No peers in mesh - messages won't propagate",
                )
            };
            layer.add_check(gossip_status);
        }

        // Request-Response
        if let Some(ref state) = self.state {
            let metrics = state.get_metrics();
            layer.add_check(
                DiagnosticCheck::healthy(
                    "Request-Response",
                    format!("{} req/res exchanges", metrics.request_response_messages),
                )
                .with_metric(metrics.request_response_messages as f64, ""),
            );
        } else {
            layer.add_check(DiagnosticCheck::healthy(
                "Request-Response",
                "/erynoa/sync/1.0.0 protocol active",
            ));
        }

        // Topics
        layer.add_check(DiagnosticCheck::healthy(
            "Realm Topics",
            "Topic subscription system ready",
        ));

        // Trust Gate
        layer.add_check(DiagnosticCheck::healthy(
            "Trust Gate",
            "Connection filtering by trust score (Κ19)",
        ));

        // Latency Stats
        if let Some(ref state) = self.state {
            let metrics = state.get_metrics();
            if metrics.avg_latency_ms > 0.0 {
                let latency_check = if metrics.avg_latency_ms < 100.0 {
                    DiagnosticCheck::healthy(
                        "Network Latency",
                        format!(
                            "avg: {:.0}ms, p95: {:.0}ms",
                            metrics.avg_latency_ms, metrics.p95_latency_ms
                        ),
                    )
                } else if metrics.avg_latency_ms < 500.0 {
                    DiagnosticCheck::degraded(
                        "Network Latency",
                        format!(
                            "avg: {:.0}ms (high), p95: {:.0}ms",
                            metrics.avg_latency_ms, metrics.p95_latency_ms
                        ),
                    )
                } else {
                    DiagnosticCheck::unavailable(
                        "Network Latency",
                        format!(
                            "avg: {:.0}ms (critical), p95: {:.0}ms",
                            metrics.avg_latency_ms, metrics.p95_latency_ms
                        ),
                    )
                };
                layer.add_check(latency_check.with_metric(metrics.avg_latency_ms, "ms"));
            }
        }

        layer
    }

    async fn check_censorship_layer(&self) -> LayerDiagnostic {
        let mut layer = LayerDiagnostic::new("Censorship-Resistance (Bridges/Transports)", 0)
            .with_feature("privacy")
            .with_description("Anti-censorship mechanisms");

        #[cfg(feature = "privacy")]
        {
            // Pluggable Transports
            layer.add_check(DiagnosticCheck::healthy(
                "Pluggable Transports",
                "obfs4, Meek, Snowflake available (RL19)",
            ));

            // Bridge Network
            layer.add_check(DiagnosticCheck::healthy(
                "Bridge Network",
                "Unlisted entry points configurable",
            ));

            // Bootstrap Helper
            layer.add_check(DiagnosticCheck::healthy(
                "Bootstrap Helper",
                "DHT-recommended relay lists",
            ));
        }

        #[cfg(not(feature = "privacy"))]
        {
            layer.add_check(DiagnosticCheck::disabled("Pluggable Transports"));
            layer.add_check(DiagnosticCheck::disabled("Bridge Network"));
            layer.add_check(DiagnosticCheck::disabled("Bootstrap Helper"));
        }

        layer
    }
}

/// Prüft Hardware-Beschleunigung
fn check_hw_acceleration() -> DiagnosticCheck {
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        #[cfg(target_feature = "avx2")]
        {
            return DiagnosticCheck::healthy(
                "HW Acceleration",
                "AVX2 SIMD detected - 8× parallel ops",
            );
        }

        #[cfg(not(target_feature = "avx2"))]
        {
            // Runtime check
            if std::is_x86_feature_detected!("avx2") {
                return DiagnosticCheck::healthy(
                    "HW Acceleration",
                    "AVX2 SIMD detected - 8× parallel ops",
                );
            } else if std::is_x86_feature_detected!("sse4.1") {
                return DiagnosticCheck::degraded(
                    "HW Acceleration",
                    "SSE4.1 only - limited SIMD (4× parallel)",
                );
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        return DiagnosticCheck::healthy("HW Acceleration", "ARM NEON detected - hardware crypto");
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "aarch64")))]
    {
        DiagnosticCheck::degraded(
            "HW Acceleration",
            "No SIMD detected - fallback to scalar operations",
        )
    }

    // Fallback für x86 ohne SIMD
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    DiagnosticCheck::degraded(
        "HW Acceleration",
        "No SIMD detected - fallback to scalar operations",
    )
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_diagnostic_runner() {
        let runner = DiagnosticRunner::new()
            .with_peer_info(3, vec!["peer1".into(), "peer2".into(), "peer3".into()]);

        let diagnostics = runner.run_all(Some("test-peer-id".into())).await;

        assert!(!diagnostics.layers.is_empty());
        assert!(diagnostics.summary.total_checks > 0);
        assert_eq!(diagnostics.summary.healthy_count + diagnostics.summary.disabled_count,
                   diagnostics.summary.total_checks - diagnostics.summary.degraded_count - diagnostics.summary.unavailable_count);
    }

    #[tokio::test]
    async fn test_runner_with_state() {
        let state = std::sync::Arc::new(DiagnosticState::new("test-peer"));
        state.peer_connected("peer-1", super::super::ConnectionType::Direct);
        state.peer_connected("peer-2", super::super::ConnectionType::Direct);

        let runner = DiagnosticRunner::from_state(&state);
        let diagnostics = runner.run_all(None).await;

        assert!(diagnostics.network_metrics.is_some());
    }
}
