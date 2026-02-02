//! Network Metrics - Echtzeit-Statistiken

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

// ============================================================================
// NETWORK METRICS
// ============================================================================

/// Aggregierte Netzwerk-Metriken
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkMetrics {
    // Peer-Statistiken
    pub connected_peers: usize,
    pub discovered_peers: usize,
    pub failed_connections: usize,

    // Traffic (Bytes)
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub bytes_per_second_in: f64,
    pub bytes_per_second_out: f64,

    // Messages
    pub messages_sent: u64,
    pub messages_received: u64,
    pub messages_per_second: f64,
    pub gossip_messages: u64,
    pub request_response_messages: u64,

    // Latenz (ms)
    pub avg_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub p95_latency_ms: f64,

    // Protokoll-spezifisch
    pub kademlia_queries: u64,
    pub dht_records_stored: u64,
    pub relay_circuits_active: usize,
    pub dcutr_attempts: u64,
    pub dcutr_successes: u64,

    // Privacy Layer
    pub onion_circuits_built: u64,
    pub cover_traffic_sent: u64,
    pub messages_mixed: u64,

    // Uptime
    pub uptime_seconds: u64,
}

// ============================================================================
// ATOMIC METRICS COLLECTOR
// ============================================================================

/// Thread-safe Metriken-Sammler mit atomaren Countern
pub struct MetricsCollector {
    start_time: Instant,

    // Peer Stats
    pub connected_peers: AtomicUsize,
    pub discovered_peers: AtomicUsize,
    pub failed_connections: AtomicUsize,

    // Traffic
    pub bytes_sent: AtomicU64,
    pub bytes_received: AtomicU64,
    last_bytes_sent: AtomicU64,
    last_bytes_received: AtomicU64,
    last_rate_check: std::sync::RwLock<Instant>,

    // Messages
    pub messages_sent: AtomicU64,
    pub messages_received: AtomicU64,
    pub gossip_messages: AtomicU64,
    pub request_response_messages: AtomicU64,
    last_messages_total: AtomicU64,

    // Latency Samples (Ring-Buffer)
    latency_samples: std::sync::RwLock<Vec<u64>>,
    latency_index: AtomicUsize,

    // Protocol Stats
    pub kademlia_queries: AtomicU64,
    pub dht_records_stored: AtomicU64,
    pub relay_circuits_active: AtomicUsize,
    pub dcutr_attempts: AtomicU64,
    pub dcutr_successes: AtomicU64,

    // Privacy Stats
    pub onion_circuits_built: AtomicU64,
    pub cover_traffic_sent: AtomicU64,
    pub messages_mixed: AtomicU64,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            connected_peers: AtomicUsize::new(0),
            discovered_peers: AtomicUsize::new(0),
            failed_connections: AtomicUsize::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            last_bytes_sent: AtomicU64::new(0),
            last_bytes_received: AtomicU64::new(0),
            last_rate_check: std::sync::RwLock::new(Instant::now()),
            messages_sent: AtomicU64::new(0),
            messages_received: AtomicU64::new(0),
            gossip_messages: AtomicU64::new(0),
            request_response_messages: AtomicU64::new(0),
            last_messages_total: AtomicU64::new(0),
            latency_samples: std::sync::RwLock::new(vec![0; 100]), // 100 samples
            latency_index: AtomicUsize::new(0),
            kademlia_queries: AtomicU64::new(0),
            dht_records_stored: AtomicU64::new(0),
            relay_circuits_active: AtomicUsize::new(0),
            dcutr_attempts: AtomicU64::new(0),
            dcutr_successes: AtomicU64::new(0),
            onion_circuits_built: AtomicU64::new(0),
            cover_traffic_sent: AtomicU64::new(0),
            messages_mixed: AtomicU64::new(0),
        }
    }

    // ========================================================================
    // RECORDING METHODS
    // ========================================================================

    /// Peer verbunden
    pub fn record_peer_connected(&self) {
        self.connected_peers.fetch_add(1, Ordering::Relaxed);
    }

    /// Peer getrennt
    pub fn record_peer_disconnected(&self) {
        self.connected_peers.fetch_sub(1, Ordering::Relaxed);
    }

    /// Peer entdeckt
    pub fn record_peer_discovered(&self) {
        self.discovered_peers.fetch_add(1, Ordering::Relaxed);
    }

    /// Verbindungsfehler
    pub fn record_connection_failed(&self) {
        self.failed_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// Bytes gesendet
    pub fn record_bytes_sent(&self, bytes: u64) {
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Bytes empfangen
    pub fn record_bytes_received(&self, bytes: u64) {
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Message gesendet
    pub fn record_message_sent(&self) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    /// Message empfangen
    pub fn record_message_received(&self) {
        self.messages_received.fetch_add(1, Ordering::Relaxed);
    }

    /// Gossip-Message
    pub fn record_gossip_message(&self) {
        self.gossip_messages.fetch_add(1, Ordering::Relaxed);
        self.record_message_received();
    }

    /// Request-Response Message
    pub fn record_request_response(&self) {
        self.request_response_messages.fetch_add(1, Ordering::Relaxed);
    }

    /// Latenz-Sample aufzeichnen
    pub fn record_latency(&self, latency: Duration) {
        let ms = latency.as_millis() as u64;
        let index = self.latency_index.fetch_add(1, Ordering::Relaxed) % 100;
        if let Ok(mut samples) = self.latency_samples.write() {
            samples[index] = ms;
        }
    }

    /// Kademlia-Query
    pub fn record_kademlia_query(&self) {
        self.kademlia_queries.fetch_add(1, Ordering::Relaxed);
    }

    /// DHT Record gespeichert
    pub fn record_dht_store(&self) {
        self.dht_records_stored.fetch_add(1, Ordering::Relaxed);
    }

    /// Relay Circuit aktiv
    pub fn record_relay_circuit_opened(&self) {
        self.relay_circuits_active.fetch_add(1, Ordering::Relaxed);
    }

    /// Relay Circuit geschlossen
    pub fn record_relay_circuit_closed(&self) {
        self.relay_circuits_active.fetch_sub(1, Ordering::Relaxed);
    }

    /// DCUTR Versuch
    pub fn record_dcutr_attempt(&self) {
        self.dcutr_attempts.fetch_add(1, Ordering::Relaxed);
    }

    /// DCUTR Erfolg
    pub fn record_dcutr_success(&self) {
        self.dcutr_successes.fetch_add(1, Ordering::Relaxed);
    }

    /// Onion Circuit gebaut
    pub fn record_onion_circuit(&self) {
        self.onion_circuits_built.fetch_add(1, Ordering::Relaxed);
    }

    /// Cover Traffic gesendet
    pub fn record_cover_traffic(&self) {
        self.cover_traffic_sent.fetch_add(1, Ordering::Relaxed);
    }

    /// Message gemischt
    pub fn record_message_mixed(&self) {
        self.messages_mixed.fetch_add(1, Ordering::Relaxed);
    }

    // ========================================================================
    // SNAPSHOT
    // ========================================================================

    /// Erstelle Snapshot der aktuellen Metriken
    pub fn snapshot(&self) -> NetworkMetrics {
        let now = Instant::now();
        let uptime = now.duration_since(self.start_time);

        // Rate-Berechnung
        let (bps_in, bps_out, mps) = self.calculate_rates();

        // Latenz-Statistiken
        let (avg_lat, min_lat, max_lat, p95_lat) = self.calculate_latency_stats();

        NetworkMetrics {
            connected_peers: self.connected_peers.load(Ordering::Relaxed),
            discovered_peers: self.discovered_peers.load(Ordering::Relaxed),
            failed_connections: self.failed_connections.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            bytes_per_second_in: bps_in,
            bytes_per_second_out: bps_out,
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            messages_received: self.messages_received.load(Ordering::Relaxed),
            messages_per_second: mps,
            gossip_messages: self.gossip_messages.load(Ordering::Relaxed),
            request_response_messages: self.request_response_messages.load(Ordering::Relaxed),
            avg_latency_ms: avg_lat,
            min_latency_ms: min_lat,
            max_latency_ms: max_lat,
            p95_latency_ms: p95_lat,
            kademlia_queries: self.kademlia_queries.load(Ordering::Relaxed),
            dht_records_stored: self.dht_records_stored.load(Ordering::Relaxed),
            relay_circuits_active: self.relay_circuits_active.load(Ordering::Relaxed),
            dcutr_attempts: self.dcutr_attempts.load(Ordering::Relaxed),
            dcutr_successes: self.dcutr_successes.load(Ordering::Relaxed),
            onion_circuits_built: self.onion_circuits_built.load(Ordering::Relaxed),
            cover_traffic_sent: self.cover_traffic_sent.load(Ordering::Relaxed),
            messages_mixed: self.messages_mixed.load(Ordering::Relaxed),
            uptime_seconds: uptime.as_secs(),
        }
    }

    fn calculate_rates(&self) -> (f64, f64, f64) {
        let now = Instant::now();

        let elapsed = {
            let last = self.last_rate_check.read().unwrap();
            now.duration_since(*last).as_secs_f64()
        };

        if elapsed < 0.1 {
            return (0.0, 0.0, 0.0);
        }

        let current_in = self.bytes_received.load(Ordering::Relaxed);
        let current_out = self.bytes_sent.load(Ordering::Relaxed);
        let current_msgs =
            self.messages_sent.load(Ordering::Relaxed) +
            self.messages_received.load(Ordering::Relaxed);

        let last_in = self.last_bytes_received.swap(current_in, Ordering::Relaxed);
        let last_out = self.last_bytes_sent.swap(current_out, Ordering::Relaxed);
        let last_msgs = self.last_messages_total.swap(current_msgs, Ordering::Relaxed);

        // Update timestamp
        if let Ok(mut last_check) = self.last_rate_check.write() {
            *last_check = now;
        }

        let bps_in = (current_in.saturating_sub(last_in)) as f64 / elapsed;
        let bps_out = (current_out.saturating_sub(last_out)) as f64 / elapsed;
        let mps = (current_msgs.saturating_sub(last_msgs)) as f64 / elapsed;

        (bps_in, bps_out, mps)
    }

    fn calculate_latency_stats(&self) -> (f64, f64, f64, f64) {
        let samples: Vec<u64> = if let Ok(s) = self.latency_samples.read() {
            s.iter().cloned().filter(|&x| x > 0).collect()
        } else {
            return (0.0, 0.0, 0.0, 0.0);
        };

        if samples.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let sum: u64 = samples.iter().sum();
        let avg = sum as f64 / samples.len() as f64;
        let min = *samples.iter().min().unwrap_or(&0) as f64;
        let max = *samples.iter().max().unwrap_or(&0) as f64;

        // P95
        let mut sorted = samples.clone();
        sorted.sort_unstable();
        let p95_idx = (sorted.len() as f64 * 0.95) as usize;
        let p95 = sorted.get(p95_idx.min(sorted.len() - 1)).copied().unwrap_or(0) as f64;

        (avg, min, max, p95)
    }
}

// ============================================================================
// TRAFFIC FORMATTER
// ============================================================================

/// Human-readable Byte-Formatierung
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Human-readable Rate-Formatierung
pub fn format_rate(bytes_per_sec: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;

    if bytes_per_sec >= MB {
        format!("{:.2} MB/s", bytes_per_sec / MB)
    } else if bytes_per_sec >= KB {
        format!("{:.2} KB/s", bytes_per_sec / KB)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        collector.record_peer_connected();
        collector.record_peer_connected();
        collector.record_bytes_sent(1000);
        collector.record_message_sent();

        let snapshot = collector.snapshot();
        assert_eq!(snapshot.connected_peers, 2);
        assert_eq!(snapshot.bytes_sent, 1000);
        assert_eq!(snapshot.messages_sent, 1);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1500), "1.46 KB");
        assert_eq!(format_bytes(1_500_000), "1.43 MB");
    }
}
