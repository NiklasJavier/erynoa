//! Trust Store
//!
//! Persistiert Trust-Vektoren zwischen Subjekten.

use anyhow::Result;
use fjall::Keyspace;
use serde::{Deserialize, Serialize};

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
#[derive(Clone)]
pub struct TrustStore {
    /// Trust-Beziehungen (from:to -> StoredTrust)
    trusts: KvStore,
    /// Ausgehende Trusts Index (from -> vec![to])
    outgoing: KvStore,
    /// Eingehende Trusts Index (to -> vec![from])
    incoming: KvStore,
}

impl TrustStore {
    /// Erstellt einen neuen Trust Store
    pub fn new(keyspace: &Keyspace) -> Result<Self> {
        Ok(Self {
            trusts: KvStore::new(keyspace, "trusts")?,
            outgoing: KvStore::new(keyspace, "trusts_outgoing")?,
            incoming: KvStore::new(keyspace, "trusts_incoming")?,
        })
    }

    /// Erstellt den Key für eine Trust-Beziehung
    fn trust_key(from: &DID, to: &DID) -> String {
        format!("{}:{}", from, to)
    }

    /// Speichert oder aktualisiert einen Trust-Vektor
    pub fn put(&self, from: DID, to: DID, trust: TrustVector6D) -> Result<()> {
        let key = Self::trust_key(&from, &to);

        // Bestehenden Trust holen oder neuen erstellen
        let mut stored = self
            .trusts
            .get::<_, StoredTrust>(&key)?
            .unwrap_or(StoredTrust {
                from: from.clone(),
                to: to.clone(),
                trust,
                updated_at: 0,
                update_count: 0,
            });

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

        Ok(())
    }

    /// Holt einen Trust-Vektor
    pub fn get(&self, from: &DID, to: &DID) -> Result<Option<TrustVector6D>> {
        let key = Self::trust_key(from, to);
        Ok(self.trusts.get::<_, StoredTrust>(&key)?.map(|s| s.trust))
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

        Ok(existed)
    }

    /// Zählt alle Trust-Beziehungen
    pub fn count(&self) -> usize {
        self.trusts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
