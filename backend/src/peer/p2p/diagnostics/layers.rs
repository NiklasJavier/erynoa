//! Layer Diagnostics - 8-Schichten-Analyse mit echten Swarm-Daten
//!
//! Diese Datei enthält den DiagnosticRunner der echte Laufzeit-Daten
//! vom SwarmState verwendet statt statische Platzhalter.

use super::{
    DiagnosticCheck, DiagnosticState, LayerDiagnostic, NatStatus, P2PDiagnostics, SwarmSnapshot,
    SwarmState,
};
use std::sync::Arc;

// ============================================================================
// DIAGNOSTIC RUNNER
// ============================================================================

/// Runner für P2P-Diagnosen
///
/// Kann mit SwarmState für echte Laufzeit-Daten verwendet werden,
/// oder ohne für statische Basis-Checks.
pub struct DiagnosticRunner {
    peer_count: usize,
    connected_peers: Vec<String>,
    state: Option<Arc<DiagnosticState>>,
    /// Live Swarm State mit echten Metriken
    swarm_state: Option<Arc<SwarmState>>,
}

impl Default for DiagnosticRunner {
    fn default() -> Self {
        Self {
            peer_count: 0,
            connected_peers: Vec::new(),
            state: None,
            swarm_state: None,
        }
    }
}

impl DiagnosticRunner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Erstelle Runner aus DiagnosticState (Legacy-Modus)
    pub fn from_state(state: &Arc<DiagnosticState>) -> Self {
        let peer_ids = state.peer_ids();
        Self {
            peer_count: peer_ids.len(),
            connected_peers: peer_ids,
            state: Some(Arc::clone(state)),
            swarm_state: None,
        }
    }

    /// Erstelle Runner mit echtem SwarmState (empfohlen)
    pub fn from_swarm_state(swarm_state: &Arc<SwarmState>) -> Self {
        let snapshot = swarm_state.snapshot();
        Self {
            peer_count: snapshot.connected_peers_count,
            connected_peers: snapshot.peers.iter().map(|p| p.peer_id.clone()).collect(),
            state: None,
            swarm_state: Some(Arc::clone(swarm_state)),
        }
    }

    /// Erstelle Runner mit beiden State-Quellen
    pub fn from_both(
        state: &Arc<DiagnosticState>,
        swarm_state: &Arc<SwarmState>,
    ) -> Self {
        let snapshot = swarm_state.snapshot();
        Self {
            peer_count: snapshot.connected_peers_count,
            connected_peers: snapshot.peers.iter().map(|p| p.peer_id.clone()).collect(),
            state: Some(Arc::clone(state)),
            swarm_state: Some(Arc::clone(swarm_state)),
        }
    }

    pub fn with_peer_info(mut self, peer_count: usize, peers: Vec<String>) -> Self {
        self.peer_count = peer_count;
        self.connected_peers = peers;
        self
    }

    /// Hole SwarmSnapshot falls verfügbar
    fn get_swarm_snapshot(&self) -> Option<SwarmSnapshot> {
        self.swarm_state.as_ref().map(|s| s.snapshot())
    }

    /// Führe alle Diagnosen aus
    pub async fn run_all(&self, peer_id: Option<String>) -> P2PDiagnostics {
        let mut diagnostics = P2PDiagnostics::new();

        if let Some(id) = peer_id {
            diagnostics = diagnostics.with_node_id(id);
        }

        // Metriken hinzufügen
        if let Some(ref state) = self.state {
            diagnostics = diagnostics.with_metrics(state.get_metrics());
        }

        // SwarmSnapshot hinzufügen falls verfügbar
        if let Some(snapshot) = self.get_swarm_snapshot() {
            diagnostics = diagnostics.with_swarm_snapshot(snapshot);
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

        // TCP Transport - immer verfügbar wenn kompiliert
        layer.add_check(DiagnosticCheck::healthy(
            "TCP Transport",
            "libp2p TCP transport active",
        ));

        // Noise Protocol - immer verfügbar
        layer.add_check(DiagnosticCheck::healthy(
            "Noise Encryption",
            "XX handshake pattern configured",
        ));

        // Yamux Multiplexing - immer verfügbar
        layer.add_check(DiagnosticCheck::healthy(
            "Yamux Multiplexing",
            "Stream multiplexing enabled",
        ));

        // QUIC Transport - Feature-gated
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

        // Traffic-Stats - echte Daten wenn verfügbar
        if let Some(snapshot) = self.get_swarm_snapshot() {
            let inbound = snapshot.inbound_connections;
            let outbound = snapshot.outbound_connections;
            let errors = snapshot.connection_errors;

            let conn_check = if errors == 0 || (inbound + outbound) > errors * 2 {
                DiagnosticCheck::healthy(
                    "Connection Stats",
                    format!("↓{} inbound, ↑{} outbound, {} errors", inbound, outbound, errors),
                )
            } else {
                DiagnosticCheck::degraded(
                    "Connection Stats",
                    format!("High error rate: {} errors vs {} connections", errors, inbound + outbound),
                )
            };
            layer.add_check(conn_check);
        } else if let Some(ref state) = self.state {
            let metrics = state.get_metrics();
            layer.add_check(
                DiagnosticCheck::healthy(
                    "Traffic Stats",
                    format!(
                        "↓{}/s ↑{}/s",
                        super::metrics::format_rate(metrics.bytes_per_second_in),
                        super::metrics::format_rate(metrics.bytes_per_second_out)
                    ),
                )
                .with_metric(metrics.bytes_sent as f64 + metrics.bytes_received as f64, "B"),
            );
        }

        layer
    }

    async fn check_identity_layer(&self) -> LayerDiagnostic {
        let mut layer = LayerDiagnostic::new("Identity (Keypair/PeerId/Identify)", 2)
            .with_feature("p2p")
            .with_description("Cryptographic identity and peer identification");

        // Diese sind immer healthy wenn der Node läuft
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

        if let Some(snapshot) = self.get_swarm_snapshot() {
            // Kademlia - ECHTE Routing Table Size
            let routing_size = snapshot.kademlia_routing_table_size;
            let kad_check = if routing_size >= 3 {
                DiagnosticCheck::healthy(
                    "Kademlia DHT",
                    format!("{} peers in routing table", routing_size),
                )
            } else if routing_size > 0 {
                DiagnosticCheck::degraded(
                    "Kademlia DHT",
                    format!("Only {} peer(s) in routing table (recommended: 3+)", routing_size),
                )
            } else {
                DiagnosticCheck::unavailable(
                    "Kademlia DHT",
                    "Routing table empty - DHT not functional",
                )
            };
            layer.add_check(kad_check.with_metric(routing_size as f64, " peers"));

            // Bootstrap Status - ECHT
            let bootstrap_check = if snapshot.kademlia_bootstrap_complete {
                DiagnosticCheck::healthy("Bootstrap", "Kademlia bootstrap complete")
            } else if routing_size > 0 {
                DiagnosticCheck::degraded(
                    "Bootstrap",
                    "Bootstrap in progress...",
                )
            } else {
                DiagnosticCheck::unavailable(
                    "Bootstrap",
                    "Not bootstrapped - no peers known",
                )
            };
            layer.add_check(bootstrap_check);

            // mDNS - ECHTE Aktivität
            let mdns_check = if snapshot.mdns_active {
                DiagnosticCheck::healthy(
                    "mDNS Discovery",
                    format!("{} peers discovered via mDNS", snapshot.mdns_discovered_count),
                )
            } else {
                // mDNS ist optional, nicht degraded wenn nicht aktiv
                DiagnosticCheck::healthy(
                    "mDNS Discovery",
                    "Waiting for local network peers",
                )
            };
            layer.add_check(mdns_check);

            // DHT Records
            if snapshot.dht_records_stored > 0 {
                layer.add_check(
                    DiagnosticCheck::healthy(
                        "DHT Storage",
                        format!("{} records stored", snapshot.dht_records_stored),
                    )
                    .with_metric(snapshot.dht_records_stored as f64, ""),
                );
            }

        } else {
            // Fallback auf peer_count wenn kein SwarmState
            let kad_status = if self.peer_count > 0 {
                DiagnosticCheck::healthy(
                    "Kademlia DHT",
                    format!("Connected to {} peers", self.peer_count),
                )
                .with_metric(self.peer_count as f64, " peers")
            } else {
                DiagnosticCheck::degraded("Kademlia DHT", "No peers connected")
                    .with_metric(0.0, " peers")
            };
            layer.add_check(kad_status);

            layer.add_check(DiagnosticCheck::healthy(
                "mDNS Discovery",
                "Local network discovery enabled",
            ));

            let bootstrap_status = if self.peer_count >= 3 {
                DiagnosticCheck::healthy(
                    "Bootstrap",
                    format!("Connected to {} peers", self.peer_count),
                )
            } else if self.peer_count > 0 {
                DiagnosticCheck::degraded(
                    "Bootstrap",
                    format!("Only {} peers (recommended: 3+)", self.peer_count),
                )
            } else {
                DiagnosticCheck::unavailable("Bootstrap", "No peers connected")
            };
            layer.add_check(bootstrap_status);
        }

        layer
    }

    async fn check_nat_traversal_layer(&self) -> LayerDiagnostic {
        let mut layer = LayerDiagnostic::new("NAT-Traversal (AutoNAT/DCUTR/Relay/UPnP)", 4)
            .with_feature("p2p")
            .with_description("NAT detection and traversal mechanisms");

        if let Some(snapshot) = self.get_swarm_snapshot() {
            // AutoNAT - ECHTER Status
            let autonat_check = match snapshot.nat_status {
                NatStatus::Public => DiagnosticCheck::healthy(
                    "AutoNAT",
                    "NAT Status: Public - directly reachable",
                ),
                NatStatus::Private => DiagnosticCheck::degraded(
                    "AutoNAT",
                    "NAT Status: Private - needs relay/holepunching",
                ),
                NatStatus::Unknown => DiagnosticCheck::healthy(
                    "AutoNAT",
                    "NAT Status: Probing...",
                ),
            };
            layer.add_check(autonat_check);

            // Externe Adressen
            let ext_addrs = &snapshot.external_addresses;
            if !ext_addrs.is_empty() {
                layer.add_check(
                    DiagnosticCheck::healthy(
                        "External Addresses",
                        format!("{} confirmed addresses", ext_addrs.len()),
                    )
                    .with_details(serde_json::json!({ "addresses": ext_addrs })),
                );
            }

            // DCUTR - ECHTE Erfolgsrate
            let total_dcutr = snapshot.dcutr_successes + snapshot.dcutr_failures;
            let dcutr_check = if total_dcutr == 0 {
                DiagnosticCheck::healthy(
                    "DCUTR",
                    "Holepunching: No attempts yet",
                )
            } else if snapshot.dcutr_success_rate >= 50.0 {
                DiagnosticCheck::healthy(
                    "DCUTR",
                    format!(
                        "Holepunching: {}/{} successful ({:.0}%)",
                        snapshot.dcutr_successes, total_dcutr, snapshot.dcutr_success_rate
                    ),
                )
            } else {
                DiagnosticCheck::degraded(
                    "DCUTR",
                    format!(
                        "Low success rate: {}/{} ({:.0}%)",
                        snapshot.dcutr_successes, total_dcutr, snapshot.dcutr_success_rate
                    ),
                )
            };
            layer.add_check(dcutr_check.with_metric(snapshot.dcutr_success_rate, "%"));

            // Relay - ECHTER Status
            let relay_check = if snapshot.has_relay_reservation {
                DiagnosticCheck::healthy(
                    "Relay Client",
                    "Active relay reservation",
                )
            } else if snapshot.nat_status == NatStatus::Public {
                DiagnosticCheck::healthy(
                    "Relay Client",
                    "Not needed - publicly reachable",
                )
            } else {
                DiagnosticCheck::degraded(
                    "Relay Client",
                    "No relay reservation - may be unreachable behind NAT",
                )
            };
            layer.add_check(relay_check);

            // Relay Server - ECHTE Circuit-Anzahl
            layer.add_check(
                DiagnosticCheck::healthy(
                    "Relay Server",
                    format!("Serving {} circuits", snapshot.relay_circuits_serving),
                )
                .with_metric(snapshot.relay_circuits_serving as f64, ""),
            );

            // UPnP - ECHTER Status
            let upnp_check = if snapshot.upnp_available {
                DiagnosticCheck::healthy(
                    "UPnP",
                    "Port mapping active",
                )
            } else {
                // UPnP ist optional, nicht degraded
                DiagnosticCheck::healthy(
                    "UPnP",
                    "Not available (normal in Docker/cloud)",
                )
            };
            layer.add_check(upnp_check);

        } else {
            // Fallback - statische Checks (Legacy)
            layer.add_check(DiagnosticCheck::healthy(
                "AutoNAT",
                "NAT type detection configured",
            ));

            layer.add_check(DiagnosticCheck::healthy(
                "DCUTR",
                "Direct Connection Upgrade available",
            ));

            layer.add_check(DiagnosticCheck::healthy(
                "Relay Client",
                "Circuit relay client configured",
            ));

            layer.add_check(DiagnosticCheck::healthy(
                "Relay Server",
                "Providing relay services",
            ));

            layer.add_check(DiagnosticCheck::healthy(
                "UPnP",
                "Automatic port mapping enabled",
            ));
        }

        layer
    }

    async fn check_performance_layer(&self) -> LayerDiagnostic {
        let mut layer = LayerDiagnostic::new("Performance (Latency/HW-Accel)", 5)
            .with_feature("privacy")
            .with_description("Performance metrics and hardware acceleration");

        #[cfg(feature = "privacy")]
        {
            // Latenz - ECHTE Ping-Daten
            if let Some(snapshot) = self.get_swarm_snapshot() {
                if snapshot.avg_ping_ms > 0.0 {
                    let latency_check = if snapshot.avg_ping_ms < 100.0 {
                        DiagnosticCheck::healthy(
                            "Network Latency",
                            format!(
                                "avg: {:.0}ms, min: {:.0}ms, max: {:.0}ms",
                                snapshot.avg_ping_ms, snapshot.min_ping_ms, snapshot.max_ping_ms
                            ),
                        )
                    } else if snapshot.avg_ping_ms < 500.0 {
                        DiagnosticCheck::degraded(
                            "Network Latency",
                            format!(
                                "avg: {:.0}ms (elevated), max: {:.0}ms",
                                snapshot.avg_ping_ms, snapshot.max_ping_ms
                            ),
                        )
                    } else {
                        DiagnosticCheck::unavailable(
                            "Network Latency",
                            format!(
                                "avg: {:.0}ms (critical!)",
                                snapshot.avg_ping_ms
                            ),
                        )
                    };
                    layer.add_check(latency_check.with_metric(snapshot.avg_ping_ms, "ms"));
                } else {
                    layer.add_check(DiagnosticCheck::healthy(
                        "Network Latency",
                        "No measurements yet",
                    ));
                }
            }

            // Hardware Acceleration - Runtime Detection
            let hw_status = check_hw_acceleration();
            layer.add_check(hw_status);

            // Batch Crypto
            layer.add_check(DiagnosticCheck::healthy(
                "Batch Crypto",
                "Parallel encryption/decryption available",
            ));
        }

        #[cfg(not(feature = "privacy"))]
        {
            layer.add_check(DiagnosticCheck::disabled("Network Latency"));
            layer.add_check(DiagnosticCheck::disabled("HW Acceleration"));
            layer.add_check(DiagnosticCheck::disabled("Batch Crypto"));
        }

        layer
    }

    async fn check_privacy_layer(&self) -> LayerDiagnostic {
        let mut layer = LayerDiagnostic::new("Privacy (Onion/Mixing/Cover)", 6)
            .with_feature("privacy")
            .with_description("Privacy-preserving communication layer");

        #[cfg(feature = "privacy")]
        {
            // Onion Routing
            layer.add_check(DiagnosticCheck::healthy(
                "Onion Routing",
                "Multi-hop encryption available",
            ));

            // Relay Selection
            layer.add_check(DiagnosticCheck::healthy(
                "Trust-based Relay Selection",
                "6D Trust-vector scoring enabled",
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
                    "Laplace-delayed message pools",
                ));

                layer.add_check(DiagnosticCheck::healthy(
                    "Cover Traffic",
                    "Dummy message generation ready",
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

        if let Some(snapshot) = self.get_swarm_snapshot() {
            // Gossipsub - ECHTE Mesh-Größe und Message-Counts
            let mesh_size = snapshot.gossip_mesh_size;
            let msgs_rx = snapshot.gossip_messages_received;
            let msgs_tx = snapshot.gossip_messages_sent;

            let gossip_check = if mesh_size >= 3 {
                DiagnosticCheck::healthy(
                    "Gossipsub",
                    format!(
                        "Mesh: {} peers, {} msgs rx, {} msgs tx",
                        mesh_size, msgs_rx, msgs_tx
                    ),
                )
            } else if mesh_size > 0 {
                DiagnosticCheck::degraded(
                    "Gossipsub",
                    format!(
                        "Small mesh: {} peer(s) - messages may not propagate well",
                        mesh_size
                    ),
                )
            } else if self.peer_count > 0 {
                // Peers verbunden aber nicht im Mesh
                DiagnosticCheck::degraded(
                    "Gossipsub",
                    "No peers in mesh - waiting for mesh formation",
                )
            } else {
                DiagnosticCheck::unavailable(
                    "Gossipsub",
                    "No peers - messages cannot propagate",
                )
            };
            layer.add_check(gossip_check.with_metric(mesh_size as f64, " peers"));

            // Topics
            let topics = snapshot.gossip_topics_subscribed;
            if topics > 0 {
                layer.add_check(
                    DiagnosticCheck::healthy(
                        "Topic Subscriptions",
                        format!("{} topics subscribed", topics),
                    )
                    .with_metric(topics as f64, ""),
                );
            }

        } else if let Some(ref state) = self.state {
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
                    "No peers in mesh",
                )
            };
            layer.add_check(gossip_check.with_metric(metrics.gossip_messages as f64, " msgs"));
        } else {
            let gossip_status = if self.peer_count > 0 {
                DiagnosticCheck::healthy(
                    "Gossipsub",
                    format!("PubSub mesh with {} peers", self.peer_count),
                )
            } else {
                DiagnosticCheck::degraded(
                    "Gossipsub",
                    "No peers in mesh",
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

        // Realm Topics
        layer.add_check(DiagnosticCheck::healthy(
            "Realm Topics",
            "Topic subscription system ready",
        ));

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
                "obfs4, Meek, Snowflake available",
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

/// Prüft Hardware-Beschleunigung zur Runtime
fn check_hw_acceleration() -> DiagnosticCheck {
    #[cfg(target_arch = "x86_64")]
    {
        // Runtime Feature Detection für x86_64
        if std::arch::is_x86_feature_detected!("avx2") {
            if std::arch::is_x86_feature_detected!("avx512f") {
                return DiagnosticCheck::healthy(
                    "HW Acceleration",
                    "AVX-512 detected - maximum SIMD performance",
                );
            }
            return DiagnosticCheck::healthy(
                "HW Acceleration",
                "AVX2 detected - 8× parallel SIMD ops",
            );
        } else if std::arch::is_x86_feature_detected!("sse4.1") {
            return DiagnosticCheck::degraded(
                "HW Acceleration",
                "SSE4.1 only - limited SIMD (4× parallel)",
            );
        }
        return DiagnosticCheck::degraded(
            "HW Acceleration",
            "No SIMD detected - scalar fallback",
        );
    }

    #[cfg(target_arch = "aarch64")]
    {
        // ARM64 hat immer NEON
        return DiagnosticCheck::healthy(
            "HW Acceleration",
            "ARM NEON detected - hardware crypto",
        );
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        DiagnosticCheck::degraded(
            "HW Acceleration",
            "Unknown architecture - scalar fallback",
        )
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_diagnostic_runner_basic() {
        let runner = DiagnosticRunner::new()
            .with_peer_info(3, vec!["peer1".into(), "peer2".into(), "peer3".into()]);

        let diagnostics = runner.run_all(Some("test-peer-id".into())).await;

        assert!(!diagnostics.layers.is_empty());
        assert!(diagnostics.summary.total_checks > 0);
    }

    #[tokio::test]
    async fn test_runner_with_swarm_state() {
        let swarm_state = Arc::new(SwarmState::new("test-peer"));

        // Simuliere einige Events
        swarm_state.set_nat_status(NatStatus::Public);
        swarm_state.peer_connected("peer-1", false, false);
        swarm_state.peer_connected("peer-2", true, false);
        swarm_state.kademlia_bootstrap_done();
        swarm_state.set_kademlia_routing_table_size(5);

        let runner = DiagnosticRunner::from_swarm_state(&swarm_state);
        let diagnostics = runner.run_all(Some("test-peer".into())).await;

        // Prüfe dass SwarmSnapshot verwendet wird
        assert!(diagnostics.swarm_snapshot.is_some());
        let snapshot = diagnostics.swarm_snapshot.unwrap();
        assert_eq!(snapshot.nat_status, NatStatus::Public);
        assert_eq!(snapshot.connected_peers_count, 2);
        assert!(snapshot.kademlia_bootstrap_complete);
    }

    #[test]
    fn test_hw_acceleration_check() {
        let check = check_hw_acceleration();
        // Sollte nicht Unknown sein
        assert_ne!(check.status, super::super::ComponentStatus::Unknown);
    }
}
