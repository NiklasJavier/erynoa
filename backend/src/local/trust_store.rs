//! Trust Store
//!
//! Persistiert Trust-Vektoren zwischen Subjekten.
//!
//! ## Phase 2 Features
//!
//! - Metriken für alle Operationen
//! - Delta-Tracking (positive/negative Updates)
//! - Asymmetry-Ratio gemäß Κ4
//! - Snapshot-Pattern für konsistente Reads

use anyhow::Result;
use fjall::Keyspace;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use super::metrics::{StoreMetrics, StoreMetricsSnapshot};
use super::KvStore;
use crate::domain::{TrustVector6D, DID};

/// Trust-Beziehung zwischen zwei Subjekten
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTrust {
    /// Vertrauender (Trustor)
    pub from: DID,
    /// Vertrauter (Trustee)
    pub to: DID,
    /// Trust-Vektor
    pub trust: TrustVector6D,
    /// Letzte Aktualisierung
    pub updated_at: i64,
    /// Anzahl der Updates
    pub update_count: u64,
}

/// Trust Store für Trust-Vektor-Persistierung
///
/// Jetzt mit integriertem Metriken-Tracking gemäß `state.rs` Patterns.
#[derive(Clone)]
pub struct TrustStore {
    /// Trust-Beziehungen (from:to -> StoredTrust)
    trusts: KvStore,
    /// Ausgehende Trusts Index (from -> vec![to])
    outgoing: KvStore,
    /// Eingehende Trusts Index (to -> vec![from])
    incoming: KvStore,

    // ─────────────────────────────────────────────────────────────────────────
    // METRICS (Phase 2)
    // ─────────────────────────────────────────────────────────────────────────
    /// Gemeinsame Metriken
    metrics: Arc<StoreMetrics>,

    /// Gesamtzahl Updates
    updates_total: Arc<AtomicU64>,

    /// Positive Trust-Updates (omega erhöht)
    positive_updates: Arc<AtomicU64>,

    /// Negative Trust-Updates (omega verringert)
    negative_updates: Arc<AtomicU64>,

    /// Neue Beziehungen erstellt
    relationships_created: Arc<AtomicU64>,

    /// Beziehungen gelöscht
    relationships_deleted: Arc<AtomicU64>,
}

impl TrustStore {
    /// Erstellt einen neuen Trust Store
    pub fn new(keyspace: &Keyspace) -> Result<Self> {
        let store = Self {
            trusts: KvStore::new(keyspace, "trusts")?,
            outgoing: KvStore::new(keyspace, "trusts_outgoing")?,
            incoming: KvStore::new(keyspace, "trusts_incoming")?,
            metrics: Arc::new(StoreMetrics::new()),
            updates_total: Arc::new(AtomicU64::new(0)),
            positive_updates: Arc::new(AtomicU64::new(0)),
            negative_updates: Arc::new(AtomicU64::new(0)),
            relationships_created: Arc::new(AtomicU64::new(0)),
            relationships_deleted: Arc::new(AtomicU64::new(0)),
        };

        // Initial count setzen
        store.metrics.set_count(store.trusts.len() as u64);

        Ok(store)
    }

    /// Erstellt den Key für eine Trust-Beziehung
    fn trust_key(from: &DID, to: &DID) -> String {
        format!("{}:{}", from, to)
    }

    /// Speichert oder aktualisiert einen Trust-Vektor mit Delta-Tracking
    pub fn put(&self, from: DID, to: DID, trust: TrustVector6D) -> Result<()> {
        let start = Instant::now();
        let key = Self::trust_key(&from, &to);

        // Alte Werte für Delta-Berechnung holen
        let old_trust = self.trusts.get::<_, StoredTrust>(&key)?;
        let is_new = old_trust.is_none();

        // Bestehenden Trust holen oder neuen erstellen
        let mut stored = old_trust.unwrap_or(StoredTrust {
            from: from.clone(),
            to: to.clone(),
            trust,
            updated_at: 0,
            update_count: 0,
        });

        // Delta-Tracking vor Update
        let old_omega = stored.trust.omega;

        stored.trust = trust;
        stored.updated_at = chrono::Utc::now().timestamp();
        stored.update_count += 1;

        self.trusts.put(&key, &stored)?;

        // Indizes aktualisieren
        let from_str = from.to_string();
        let to_str = to.to_string();

        // Outgoing Index
        let mut outgoing: Vec<String> = self.outgoing.get(&from_str)?.unwrap_or_default();
        if !outgoing.contains(&to_str) {
            outgoing.push(to_str.clone());
            self.outgoing.put(&from_str, &outgoing)?;
        }

        // Incoming Index
        let mut incoming: Vec<String> = self.incoming.get(&to_str)?.unwrap_or_default();
        if !incoming.contains(&from_str) {
            incoming.push(from_str);
            self.incoming.put(&to_str, &incoming)?;
        }

        // Metriken
        let latency = start.elapsed().as_micros() as u64;
        self.metrics.record_write(latency, 128);
        self.updates_total.fetch_add(1, Ordering::Relaxed);

        // Delta-Tracking
        if is_new {
            self.relationships_created.fetch_add(1, Ordering::Relaxed);
            self.positive_updates.fetch_add(1, Ordering::Relaxed); // Neue Beziehung = positiv
            self.metrics.increment_count();
        } else {
            // Vergleiche omega für positive/negative Klassifikation
            if trust.omega > old_omega {
                self.positive_updates.fetch_add(1, Ordering::Relaxed);
            } else if trust.omega < old_omega {
                self.negative_updates.fetch_add(1, Ordering::Relaxed);
            }
            // Gleichbleibend = neutral, keine Zählung
        }

        Ok(())
    }

    /// Holt einen Trust-Vektor
    pub fn get(&self, from: &DID, to: &DID) -> Result<Option<TrustVector6D>> {
        let start = Instant::now();
        let key = Self::trust_key(from, to);

        let result = self.trusts.get::<_, StoredTrust>(&key)?;

        let latency = start.elapsed().as_micros() as u64;
        self.metrics
            .record_read(latency, if result.is_some() { 128 } else { 0 });

        Ok(result.map(|s| s.trust))
    }

    /// Holt die vollständige Trust-Beziehung
    pub fn get_full(&self, from: &DID, to: &DID) -> Result<Option<StoredTrust>> {
        let key = Self::trust_key(from, to);
        self.trusts.get(&key)
    }

    /// Holt alle ausgehenden Trusts eines Subjekts
    pub fn get_outgoing(&self, from: &DID) -> Result<Vec<StoredTrust>> {
        let from_str = from.to_string();
        let to_list: Vec<String> = self.outgoing.get(&from_str)?.unwrap_or_default();

        let mut trusts = Vec::new();
        for to_str in to_list {
            let key = format!("{}:{}", from_str, to_str);
            if let Some(stored) = self.trusts.get::<_, StoredTrust>(&key)? {
                trusts.push(stored);
            }
        }
        Ok(trusts)
    }

    /// Holt alle eingehenden Trusts eines Subjekts
    pub fn get_incoming(&self, to: &DID) -> Result<Vec<StoredTrust>> {
        let to_str = to.to_string();
        let from_list: Vec<String> = self.incoming.get(&to_str)?.unwrap_or_default();

        let mut trusts = Vec::new();
        for from_str in from_list {
            let key = format!("{}:{}", from_str, to_str);
            if let Some(stored) = self.trusts.get::<_, StoredTrust>(&key)? {
                trusts.push(stored);
            }
        }
        Ok(trusts)
    }

    /// Berechnet den aggregierten eingehenden Trust (Reputation)
    pub fn compute_reputation(&self, subject: &DID) -> Result<TrustVector6D> {
        let incoming = self.get_incoming(subject)?;
        if incoming.is_empty() {
            return Ok(TrustVector6D::default());
        }

        // Summe aller eingehenden Trusts (f32 für unified TrustVector6D)
        let mut sum_r = 0.0f32;
        let mut sum_i = 0.0f32;
        let mut sum_c = 0.0f32;
        let mut sum_p = 0.0f32;
        let mut sum_v = 0.0f32;
        let mut sum_o = 0.0f32;

        for stored in &incoming {
            sum_r += stored.trust.r;
            sum_i += stored.trust.i;
            sum_c += stored.trust.c;
            sum_p += stored.trust.p;
            sum_v += stored.trust.v;
            sum_o += stored.trust.omega;
        }

        // Normalisieren
        let count = incoming.len() as f32;
        Ok(TrustVector6D::new(
            sum_r / count,
            sum_i / count,
            sum_c / count,
            sum_p / count,
            sum_v / count,
            sum_o / count,
        ))
    }

    /// Löscht eine Trust-Beziehung
    pub fn delete(&self, from: &DID, to: &DID) -> Result<bool> {
        let key = Self::trust_key(from, to);
        let existed = self.trusts.delete(&key)?;

        // Indizes bereinigen
        let from_str = from.to_string();
        let to_str = to.to_string();

        if let Some(mut outgoing) = self.outgoing.get::<_, Vec<String>>(&from_str)? {
            outgoing.retain(|s| s != &to_str);
            self.outgoing.put(&from_str, &outgoing)?;
        }

        if let Some(mut incoming) = self.incoming.get::<_, Vec<String>>(&to_str)? {
            incoming.retain(|s| s != &from_str);
            self.incoming.put(&to_str, &incoming)?;
        }

        // Metriken
        if existed {
            self.relationships_deleted.fetch_add(1, Ordering::Relaxed);
            self.metrics.decrement_count();
        }

        Ok(existed)
    }

    /// Zählt alle Trust-Beziehungen
    pub fn count(&self) -> usize {
        self.trusts.len()
    }

    // ─────────────────────────────────────────────────────────────────────────
    // METRICS API (Phase 2)
    // ─────────────────────────────────────────────────────────────────────────

    /// Asymmetry-Ratio (negative/positive Updates)
    ///
    /// Gemäß Κ4 sollte dieses Verhältnis ~2:1 sein (es sollte schwerer sein,
    /// Trust aufzubauen als ihn zu verlieren).
    pub fn asymmetry_ratio(&self) -> f64 {
        let pos = self.positive_updates.load(Ordering::Relaxed) as f64;
        let neg = self.negative_updates.load(Ordering::Relaxed) as f64;

        if pos > 0.0 {
            neg / pos
        } else if neg > 0.0 {
            f64::INFINITY
        } else {
            0.0
        }
    }

    /// Snapshot der TrustStore-Metriken
    pub fn snapshot(&self) -> TrustStoreSnapshot {
        TrustStoreSnapshot {
            relationships: self.count() as u64,
            updates_total: self.updates_total.load(Ordering::Relaxed),
            positive_updates: self.positive_updates.load(Ordering::Relaxed),
            negative_updates: self.negative_updates.load(Ordering::Relaxed),
            relationships_created: self.relationships_created.load(Ordering::Relaxed),
            relationships_deleted: self.relationships_deleted.load(Ordering::Relaxed),
            asymmetry_ratio: self.asymmetry_ratio(),
            metrics: self.metrics.snapshot(),
        }
    }

    /// Health-Score (0.0 - 1.0)
    pub fn health_score(&self) -> f64 {
        self.metrics.health_score()
    }

    /// Ist der Store gesund?
    pub fn is_healthy(&self) -> bool {
        self.health_score() >= 0.9
    }

    /// Zugriff auf die internen Metriken (für Aggregation)
    pub fn metrics(&self) -> &Arc<StoreMetrics> {
        &self.metrics
    }
}

/// Snapshot der TrustStore-Metriken
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustStoreSnapshot {
    /// Anzahl aktiver Trust-Beziehungen
    pub relationships: u64,

    /// Gesamtzahl Updates
    pub updates_total: u64,

    /// Positive Updates (omega erhöht)
    pub positive_updates: u64,

    /// Negative Updates (omega verringert)
    pub negative_updates: u64,

    /// Beziehungen erstellt
    pub relationships_created: u64,

    /// Beziehungen gelöscht
    pub relationships_deleted: u64,

    /// Asymmetrie-Ratio (neg/pos, Ziel ~2:1 gemäß Κ4)
    pub asymmetry_ratio: f64,

    /// Basis-Metriken
    pub metrics: StoreMetricsSnapshot,
}

impl TrustStoreSnapshot {
    /// Ist die Asymmetrie im gesunden Bereich (1.5 - 3.0)?
    pub fn is_asymmetry_healthy(&self) -> bool {
        self.asymmetry_ratio >= 1.5 && self.asymmetry_ratio <= 3.0
    }

    /// Churn-Rate (deletes / creates)
    pub fn churn_rate(&self) -> f64 {
        if self.relationships_created > 0 {
            self.relationships_deleted as f64 / self.relationships_created as f64
        } else {
            0.0
        }
    }

    /// Updates pro Beziehung (Durchschnitt)
    pub fn updates_per_relationship(&self) -> f64 {
        if self.relationships > 0 {
            self.updates_total as f64 / self.relationships as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DIDNamespace;

    fn create_test_store() -> TrustStore {
        let folder = tempfile::tempdir().unwrap();
        let keyspace = fjall::Config::new(folder.path()).open().unwrap();
        TrustStore::new(&keyspace).unwrap()
    }

    #[test]
    fn test_put_get() {
        let store = create_test_store();

        let alice = DID::new(DIDNamespace::Self_, b"alice");
        let bob = DID::new(DIDNamespace::Self_, b"bob");
        let trust = TrustVector6D::new(0.8, 0.9, 0.7, 0.6, 0.5, 0.95);

        store.put(alice.clone(), bob.clone(), trust).unwrap();

        let retrieved = store.get(&alice, &bob).unwrap();
        assert!(retrieved.is_some());
        assert!((retrieved.unwrap().r - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_outgoing_incoming() {
        let store = create_test_store();

        let alice = DID::new(DIDNamespace::Self_, b"alice");
        let bob = DID::new(DIDNamespace::Self_, b"bob");
        let charlie = DID::new(DIDNamespace::Self_, b"charlie");

        let trust = TrustVector6D::new(0.8, 0.9, 0.7, 0.6, 0.5, 0.95);

        store.put(alice.clone(), bob.clone(), trust).unwrap();
        store.put(alice.clone(), charlie.clone(), trust).unwrap();
        store.put(charlie.clone(), bob.clone(), trust).unwrap();

        let alice_outgoing = store.get_outgoing(&alice).unwrap();
        assert_eq!(alice_outgoing.len(), 2);

        let bob_incoming = store.get_incoming(&bob).unwrap();
        assert_eq!(bob_incoming.len(), 2);
    }

    #[test]
    fn test_reputation() {
        let store = create_test_store();

        let alice = DID::new(DIDNamespace::Self_, b"alice");
        let bob = DID::new(DIDNamespace::Self_, b"bob");
        let charlie = DID::new(DIDNamespace::Self_, b"charlie");

        // Bob bekommt Trust von Alice (0.8) und Charlie (0.6)
        store
            .put(
                alice.clone(),
                bob.clone(),
                TrustVector6D::new(0.8, 0.8, 0.8, 0.8, 0.8, 0.8),
            )
            .unwrap();
        store
            .put(
                charlie.clone(),
                bob.clone(),
                TrustVector6D::new(0.6, 0.6, 0.6, 0.6, 0.6, 0.6),
            )
            .unwrap();

        let reputation = store.compute_reputation(&bob).unwrap();
        // Durchschnitt: 0.7
        assert!((reputation.r - 0.7).abs() < 0.001);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Phase 2: Metrics Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_trust_store_snapshot() {
        let store = create_test_store();

        let alice = DID::new(DIDNamespace::Self_, b"alice");
        let bob = DID::new(DIDNamespace::Self_, b"bob");
        let trust = TrustVector6D::new(0.8, 0.9, 0.7, 0.6, 0.5, 0.95);

        store.put(alice, bob, trust).unwrap();

        let snapshot = store.snapshot();

        assert_eq!(snapshot.relationships, 1);
        assert_eq!(snapshot.updates_total, 1);
        assert_eq!(snapshot.positive_updates, 1); // Neue Beziehung = positiv
        assert_eq!(snapshot.relationships_created, 1);
        assert!(store.is_healthy());
    }

    #[test]
    fn test_delta_tracking() {
        let store = create_test_store();

        let alice = DID::new(DIDNamespace::Self_, b"alice");
        let bob = DID::new(DIDNamespace::Self_, b"bob");

        // Erste Trust-Beziehung (omega = 0.5)
        store
            .put(
                alice.clone(),
                bob.clone(),
                TrustVector6D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5),
            )
            .unwrap();

        // Update: omega erhöht auf 0.8 (positiv)
        store
            .put(
                alice.clone(),
                bob.clone(),
                TrustVector6D::new(0.8, 0.8, 0.8, 0.8, 0.8, 0.8),
            )
            .unwrap();

        // Update: omega verringert auf 0.3 (negativ)
        store
            .put(
                alice.clone(),
                bob.clone(),
                TrustVector6D::new(0.3, 0.3, 0.3, 0.3, 0.3, 0.3),
            )
            .unwrap();

        let snapshot = store.snapshot();

        assert_eq!(snapshot.relationships, 1);
        assert_eq!(snapshot.updates_total, 3);
        assert_eq!(snapshot.positive_updates, 2); // Neue Beziehung + Erhöhung
        assert_eq!(snapshot.negative_updates, 1); // Verringerung
    }

    #[test]
    fn test_asymmetry_ratio() {
        let store = create_test_store();

        let alice = DID::new(DIDNamespace::Self_, b"alice");
        let bob = DID::new(DIDNamespace::Self_, b"bob");

        // Neue Beziehung (positiv)
        store
            .put(
                alice.clone(),
                bob.clone(),
                TrustVector6D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5),
            )
            .unwrap();

        // 2 negative Updates
        store
            .put(
                alice.clone(),
                bob.clone(),
                TrustVector6D::new(0.3, 0.3, 0.3, 0.3, 0.3, 0.3),
            )
            .unwrap();
        store
            .put(
                alice.clone(),
                bob.clone(),
                TrustVector6D::new(0.1, 0.1, 0.1, 0.1, 0.1, 0.1),
            )
            .unwrap();

        let ratio = store.asymmetry_ratio();
        // 2 negative / 1 positive = 2.0
        assert!((ratio - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_delete_tracking() {
        let store = create_test_store();

        let alice = DID::new(DIDNamespace::Self_, b"alice");
        let bob = DID::new(DIDNamespace::Self_, b"bob");

        store
            .put(
                alice.clone(),
                bob.clone(),
                TrustVector6D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5),
            )
            .unwrap();

        assert_eq!(store.count(), 1);

        store.delete(&alice, &bob).unwrap();

        let snapshot = store.snapshot();
        assert_eq!(snapshot.relationships, 0);
        assert_eq!(snapshot.relationships_deleted, 1);
    }
}
