//! Event-System für Echtzeit-Diagnose

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::RwLock;
use std::time::Instant;

// ============================================================================
// DIAGNOSTIC EVENT
// ============================================================================

/// Typ eines Diagnostic-Events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    // Peer Events
    PeerConnected,
    PeerDisconnected,
    PeerDiscovered,
    ConnectionFailed,

    // Protocol Events
    GossipReceived,
    GossipSent,
    RequestReceived,
    ResponseSent,
    KademliaQuery,
    KademliaResult,

    // NAT Events
    AutoNatStatus,
    DcutrAttempt,
    DcutrSuccess,
    DcutrFailed,
    RelayReservation,
    UpnpMapping,

    // Privacy Events
    OnionCircuitBuilt,
    OnionCircuitFailed,
    CoverTrafficSent,
    MessageMixed,

    // System Events
    SwarmStarted,
    SwarmStopped,
    ConfigReloaded,
    Error,
    Warning,
    Info,
}

impl EventType {
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::PeerConnected => "🟢",
            Self::PeerDisconnected => "🔴",
            Self::PeerDiscovered => "🔍",
            Self::ConnectionFailed => "❌",
            Self::GossipReceived => "📨",
            Self::GossipSent => "📤",
            Self::RequestReceived => "📥",
            Self::ResponseSent => "📤",
            Self::KademliaQuery => "🔎",
            Self::KademliaResult => "📋",
            Self::AutoNatStatus => "🌐",
            Self::DcutrAttempt => "🔄",
            Self::DcutrSuccess => "✅",
            Self::DcutrFailed => "❌",
            Self::RelayReservation => "🔗",
            Self::UpnpMapping => "🗺️",
            Self::OnionCircuitBuilt => "🧅",
            Self::OnionCircuitFailed => "💔",
            Self::CoverTrafficSent => "🎭",
            Self::MessageMixed => "🔀",
            Self::SwarmStarted => "🚀",
            Self::SwarmStopped => "🛑",
            Self::ConfigReloaded => "⚙️",
            Self::Error => "🔥",
            Self::Warning => "⚠️",
            Self::Info => "ℹ️",
        }
    }

    pub fn severity(&self) -> EventSeverity {
        match self {
            Self::Error | Self::ConnectionFailed | Self::OnionCircuitFailed | Self::DcutrFailed => {
                EventSeverity::Error
            }
            Self::Warning | Self::PeerDisconnected => EventSeverity::Warning,
            Self::Info
            | Self::SwarmStarted
            | Self::SwarmStopped
            | Self::ConfigReloaded
            | Self::AutoNatStatus => EventSeverity::Info,
            _ => EventSeverity::Debug,
        }
    }
}

/// Severity-Level eines Events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum EventSeverity {
    Debug,
    Info,
    Warning,
    Error,
}

impl EventSeverity {
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Debug => "severity-debug",
            Self::Info => "severity-info",
            Self::Warning => "severity-warning",
            Self::Error => "severity-error",
        }
    }
}

/// Ein einzelnes Diagnostic-Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticEvent {
    /// Eindeutige Event-ID
    pub id: u64,
    /// Zeitstempel (RFC3339)
    pub timestamp: String,
    /// Millisekunden seit Start
    pub uptime_ms: u64,
    /// Event-Typ
    pub event_type: EventType,
    /// Severity
    pub severity: EventSeverity,
    /// Kurze Beschreibung
    pub message: String,
    /// Betroffene Peer-ID (falls relevant)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_id: Option<String>,
    /// Topic (für Gossip-Events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    /// Zusätzliche Details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl DiagnosticEvent {
    /// Formatiere für CLI-Ausgabe
    pub fn to_cli_line(&self) -> String {
        let peer_str = self
            .peer_id
            .as_ref()
            .map(|p| format!(" [{}]", &p[..p.len().min(12)]))
            .unwrap_or_default();

        format!(
            "{} {} {:?}{} - {}",
            self.event_type.emoji(),
            &self.timestamp[11..23], // HH:MM:SS.mmm
            self.event_type,
            peer_str,
            self.message
        )
    }
}

// ============================================================================
// EVENT BUFFER
// ============================================================================

/// Ring-Buffer für Events mit begrenzter Kapazität
pub struct EventBuffer {
    events: RwLock<VecDeque<DiagnosticEvent>>,
    capacity: usize,
    next_id: std::sync::atomic::AtomicU64,
    start_time: Instant,
}

impl EventBuffer {
    /// Erstelle neuen Buffer mit gegebener Kapazität
    pub fn new(capacity: usize) -> Self {
        Self {
            events: RwLock::new(VecDeque::with_capacity(capacity)),
            capacity,
            next_id: std::sync::atomic::AtomicU64::new(1),
            start_time: Instant::now(),
        }
    }

    /// Event hinzufügen
    pub fn push(&self, event_type: EventType, message: impl Into<String>) -> u64 {
        self.push_with_details(event_type, message, None, None, None)
    }

    /// Event mit Peer-ID hinzufügen
    pub fn push_with_peer(
        &self,
        event_type: EventType,
        message: impl Into<String>,
        peer_id: impl Into<String>,
    ) -> u64 {
        self.push_with_details(event_type, message, Some(peer_id.into()), None, None)
    }

    /// Event mit allen Details hinzufügen
    pub fn push_with_details(
        &self,
        event_type: EventType,
        message: impl Into<String>,
        peer_id: Option<String>,
        topic: Option<String>,
        details: Option<serde_json::Value>,
    ) -> u64 {
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let uptime = self.start_time.elapsed();

        let event = DiagnosticEvent {
            id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            uptime_ms: uptime.as_millis() as u64,
            severity: event_type.severity(),
            event_type,
            message: message.into(),
            peer_id,
            topic,
            details,
        };

        if let Ok(mut buffer) = self.events.write() {
            if buffer.len() >= self.capacity {
                buffer.pop_front();
            }
            buffer.push_back(event);
        }

        id
    }

    /// Letzte N Events abrufen
    pub fn get_recent(&self, count: usize) -> Vec<DiagnosticEvent> {
        if let Ok(buffer) = self.events.read() {
            buffer.iter().rev().take(count).cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Alle Events abrufen
    pub fn get_all(&self) -> Vec<DiagnosticEvent> {
        if let Ok(buffer) = self.events.read() {
            buffer.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Events nach Typ filtern
    pub fn get_by_type(&self, event_type: EventType, limit: usize) -> Vec<DiagnosticEvent> {
        if let Ok(buffer) = self.events.read() {
            buffer
                .iter()
                .rev()
                .filter(|e| e.event_type == event_type)
                .take(limit)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Events nach Severity filtern
    pub fn get_by_severity(&self, min_severity: EventSeverity, limit: usize) -> Vec<DiagnosticEvent> {
        if let Ok(buffer) = self.events.read() {
            buffer
                .iter()
                .rev()
                .filter(|e| e.severity >= min_severity)
                .take(limit)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Events seit ID abrufen (für Polling)
    pub fn get_since(&self, last_id: u64) -> Vec<DiagnosticEvent> {
        if let Ok(buffer) = self.events.read() {
            buffer.iter().filter(|e| e.id > last_id).cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Anzahl der Events
    pub fn len(&self) -> usize {
        self.events.read().map(|b| b.len()).unwrap_or(0)
    }

    /// Buffer leer?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Buffer leeren
    pub fn clear(&self) {
        if let Ok(mut buffer) = self.events.write() {
            buffer.clear();
        }
    }
}

impl Default for EventBuffer {
    fn default() -> Self {
        Self::new(1000) // Default: 1000 Events
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_buffer() {
        let buffer = EventBuffer::new(5);

        for i in 0..10 {
            buffer.push(EventType::Info, format!("Event {}", i));
        }

        // Nur die letzten 5 sollten vorhanden sein
        assert_eq!(buffer.len(), 5);

        let events = buffer.get_all();
        assert_eq!(events[0].message, "Event 5");
        assert_eq!(events[4].message, "Event 9");
    }

    #[test]
    fn test_event_filtering() {
        let buffer = EventBuffer::new(100);

        buffer.push(EventType::PeerConnected, "Peer 1 connected");
        buffer.push(EventType::GossipReceived, "Message received");
        buffer.push(EventType::PeerConnected, "Peer 2 connected");
        buffer.push(EventType::Error, "Something went wrong");

        let connects = buffer.get_by_type(EventType::PeerConnected, 10);
        assert_eq!(connects.len(), 2);

        let errors = buffer.get_by_severity(EventSeverity::Error, 10);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_event_since() {
        let buffer = EventBuffer::new(100);

        let id1 = buffer.push(EventType::Info, "First");
        let _id2 = buffer.push(EventType::Info, "Second");
        let _id3 = buffer.push(EventType::Info, "Third");

        let new_events = buffer.get_since(id1);
        assert_eq!(new_events.len(), 2);
        assert_eq!(new_events[0].message, "Second");
    }
}
