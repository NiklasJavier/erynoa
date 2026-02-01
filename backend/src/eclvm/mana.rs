//! # Mana System - Reputation-based Bandwidth
//!
//! Trust kauft Rechenzeit: Je höher der Trust-Score, desto mehr Gas-Budget.
//!
//! ## Konzept
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    MANA FLOW                                │
//! │                                                             │
//! │   TrustVector ──► calculate_bandwidth() ──► max_mana       │
//! │                                                             │
//! │   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐  │
//! │   │ Trust: 0.1  │     │ Trust: 0.5  │     │ Trust: 0.9  │  │
//! │   │ Mana: 11k   │     │ Mana: 60k   │     │ Mana: 110k  │  │
//! │   │ (Newcomer)  │     │ (Regular)   │     │ (Veteran)   │  │
//! │   └─────────────┘     └─────────────┘     └─────────────┘  │
//! │                                                             │
//! │   Regeneration: mana += regen_rate * elapsed_seconds       │
//! │   Regen Rate:   base_regen * (1 + trust.R)                 │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Anti-Spam Mechanik
//!
//! - Sybil-Attacke: 1000 Fake-Accounts mit Trust 0.1 haben je nur 11k Mana
//! - Nach 2 teuren Requests ist ihr Budget leer
//! - Regeneration ist langsam → Spam wird unökonomisch
//! - Trust sinkt bei Spam → negative Feedback Loop

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::domain::TrustVector6D;
use crate::error::Result;

/// Mana-Konfiguration
#[derive(Debug, Clone)]
pub struct ManaConfig {
    /// Basis-Mana für alle Nutzer (auch Trust 0)
    pub base_allowance: u64,

    /// Maximaler Multiplikator bei Trust 1.0
    /// Formel: max_mana = base * (1 + trust.R * max_multiplier)
    pub max_multiplier: f64,

    /// Basis-Regenerationsrate pro Sekunde
    pub base_regen_per_sec: u64,

    /// Regenerations-Multiplikator basierend auf Trust
    pub regen_trust_factor: f64,

    /// Minimale Zeit zwischen Regenerations-Updates
    pub regen_interval: Duration,
}

impl Default for ManaConfig {
    fn default() -> Self {
        Self {
            base_allowance: 10_000,   // Genug für ~10 einfache Ops
            max_multiplier: 100.0,    // Trust 1.0 = 100x mehr
            base_regen_per_sec: 100,  // 100 Gas/sec Basis
            regen_trust_factor: 10.0, // Trust 1.0 = 10x schnellere Regen
            regen_interval: Duration::from_secs(1),
        }
    }
}

/// Mana-Status eines einzelnen Nutzers
#[derive(Debug, Clone)]
pub struct ManaAccount {
    /// Aktuelles Mana-Guthaben
    current: u64,
    /// Maximales Mana (basierend auf Trust)
    max: u64,
    /// Regenerationsrate pro Sekunde
    regen_rate: u64,
    /// Letztes Update
    last_update: Instant,
    /// Trust-Snapshot bei letzter Berechnung (f32 für unified TrustVector6D Kompatibilität)
    trust_snapshot: f32,
}

impl ManaAccount {
    /// Erstelle neuen Account basierend auf Trust
    pub fn new(trust: &TrustVector6D, config: &ManaConfig) -> Self {
        let reliability = trust.r; // R-Dimension für Bandwidth (f32)
        let max = calculate_max_mana(reliability as f64, config);
        let regen_rate = calculate_regen_rate(reliability as f64, config);

        Self {
            current: max, // Starte voll aufgeladen
            max,
            regen_rate,
            last_update: Instant::now(),
            trust_snapshot: reliability,
        }
    }

    /// Aktualisiere Mana (Regeneration + Trust-Update)
    pub fn update(&mut self, trust: &TrustVector6D, config: &ManaConfig) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);

        // 1. Regeneration anwenden
        if elapsed >= config.regen_interval {
            let secs = elapsed.as_secs_f64();
            let regen_amount = (self.regen_rate as f64 * secs) as u64;
            self.current = self.current.saturating_add(regen_amount).min(self.max);
            self.last_update = now;
        }

        // 2. Trust-Änderung prüfen (nur bei signifikanter Änderung)
        let reliability = trust.r;
        if (reliability - self.trust_snapshot).abs() > 0.05 {
            self.max = calculate_max_mana(reliability as f64, config);
            self.regen_rate = calculate_regen_rate(reliability as f64, config);
            self.trust_snapshot = reliability;

            // Bei Trust-Verlust: current auf neues max cappen
            self.current = self.current.min(self.max);
        }
    }

    /// Prüfe ob genug Mana für eine Operation vorhanden ist
    pub fn can_afford(&self, cost: u64) -> bool {
        self.current >= cost
    }

    /// Verbrauche Mana für eine Operation
    pub fn consume(&mut self, amount: u64) -> Result<()> {
        if amount > self.current {
            return Err(crate::error::ApiError::RateLimited {
                retry_after: self.time_to_regenerate(amount),
            });
        }
        self.current = self.current.saturating_sub(amount);
        Ok(())
    }

    /// Zeit bis genug Mana regeneriert ist
    pub fn time_to_regenerate(&self, needed: u64) -> Duration {
        if needed <= self.current {
            return Duration::ZERO;
        }
        let deficit = needed - self.current;
        let secs = deficit as f64 / self.regen_rate as f64;
        Duration::from_secs_f64(secs)
    }

    /// Aktuelles Mana
    pub fn current(&self) -> u64 {
        self.current
    }

    /// Maximales Mana
    pub fn max(&self) -> u64 {
        self.max
    }

    /// Regenerationsrate pro Sekunde
    pub fn regen_rate(&self) -> u64 {
        self.regen_rate
    }

    /// Prozent gefüllt
    pub fn fill_percent(&self) -> f64 {
        if self.max == 0 {
            return 0.0;
        }
        (self.current as f64 / self.max as f64) * 100.0
    }
}

/// Berechne maximales Mana basierend auf Trust
fn calculate_max_mana(reliability: f64, config: &ManaConfig) -> u64 {
    let multiplier = 1.0 + (reliability * config.max_multiplier);
    (config.base_allowance as f64 * multiplier) as u64
}

/// Berechne Regenerationsrate basierend auf Trust
fn calculate_regen_rate(reliability: f64, config: &ManaConfig) -> u64 {
    let multiplier = 1.0 + (reliability * config.regen_trust_factor);
    (config.base_regen_per_sec as f64 * multiplier) as u64
}

/// Mana-Manager für alle Nutzer
pub struct ManaManager {
    /// Konfiguration
    config: ManaConfig,
    /// Accounts nach DID
    accounts: RwLock<HashMap<String, ManaAccount>>,
}

impl ManaManager {
    /// Erstelle neuen Manager
    pub fn new(config: ManaConfig) -> Self {
        Self {
            config,
            accounts: RwLock::new(HashMap::new()),
        }
    }

    /// Erstelle Manager mit Default-Config
    pub fn default_config() -> Self {
        Self::new(ManaConfig::default())
    }

    /// Hole oder erstelle Account für DID
    pub fn get_or_create(&self, did: &str, trust: &TrustVector6D) -> ManaAccount {
        let mut accounts = self.accounts.write().unwrap();

        accounts
            .entry(did.to_string())
            .and_modify(|acc| acc.update(trust, &self.config))
            .or_insert_with(|| ManaAccount::new(trust, &self.config))
            .clone()
    }

    /// Pre-Flight Check: Kann der User diese Operation ausführen?
    ///
    /// Gibt `Ok(())` zurück wenn genug Mana, sonst `Err(RateLimited)`
    pub fn preflight_check(
        &self,
        did: &str,
        trust: &TrustVector6D,
        estimated_gas: u64,
    ) -> Result<()> {
        let account = self.get_or_create(did, trust);

        if !account.can_afford(estimated_gas) {
            return Err(crate::error::ApiError::RateLimited {
                retry_after: account.time_to_regenerate(estimated_gas),
            });
        }

        Ok(())
    }

    /// Verbrauche Mana nach erfolgreicher Ausführung
    pub fn deduct(&self, did: &str, trust: &TrustVector6D, actual_gas: u64) -> Result<()> {
        let mut accounts = self.accounts.write().unwrap();

        let account = accounts
            .entry(did.to_string())
            .and_modify(|acc| acc.update(trust, &self.config))
            .or_insert_with(|| ManaAccount::new(trust, &self.config));

        account.consume(actual_gas)
    }

    /// Hole Status für einen User (für API/Debugging)
    pub fn get_status(&self, did: &str, trust: &TrustVector6D) -> ManaStatus {
        let account = self.get_or_create(did, trust);
        ManaStatus {
            current_mana: account.current(),
            max_mana: account.max(),
            regen_rate: account.regen_rate(),
            fill_percent: account.fill_percent(),
        }
    }

    /// Cleanup: Entferne inaktive Accounts (für Memory-Management)
    pub fn cleanup_inactive(&self, max_age: Duration) {
        let mut accounts = self.accounts.write().unwrap();
        let now = Instant::now();

        accounts.retain(|_, acc| now.duration_since(acc.last_update) < max_age);
    }
}

/// Mana-Status für API-Responses
#[derive(Debug, Clone, serde::Serialize)]
pub struct ManaStatus {
    pub current_mana: u64,
    pub max_mana: u64,
    pub regen_rate: u64,
    pub fill_percent: f64,
}

// ═══════════════════════════════════════════════════════════════
// Vorberechnete Bandwidth-Tabelle (für schnelle Lookups)
// ═══════════════════════════════════════════════════════════════

/// Bandwidth-Tier basierend auf Trust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandwidthTier {
    /// Trust 0.0-0.1: Minimale Bandbreite (Sybil-Schutz)
    Newcomer,
    /// Trust 0.1-0.3: Eingeschränkt
    Limited,
    /// Trust 0.3-0.6: Normal
    Standard,
    /// Trust 0.6-0.8: Erhöht
    Elevated,
    /// Trust 0.8-1.0: Maximum
    Veteran,
}

impl BandwidthTier {
    /// Bestimme Tier aus Trust-Score
    pub fn from_trust(reliability: f64) -> Self {
        match reliability {
            r if r < 0.1 => Self::Newcomer,
            r if r < 0.3 => Self::Limited,
            r if r < 0.6 => Self::Standard,
            r if r < 0.8 => Self::Elevated,
            _ => Self::Veteran,
        }
    }

    /// Typisches Max-Mana für diesen Tier
    pub fn typical_max_mana(&self) -> u64 {
        match self {
            Self::Newcomer => 11_000, // 10k * 1.1
            Self::Limited => 30_000,  // 10k * 3
            Self::Standard => 60_000, // 10k * 6
            Self::Elevated => 80_000, // 10k * 8
            Self::Veteran => 110_000, // 10k * 11
        }
    }

    /// Beschreibung
    pub fn description(&self) -> &'static str {
        match self {
            Self::Newcomer => "New user with minimal bandwidth",
            Self::Limited => "Limited bandwidth, build trust to increase",
            Self::Standard => "Standard bandwidth for regular users",
            Self::Elevated => "Elevated bandwidth for trusted users",
            Self::Veteran => "Maximum bandwidth for veterans",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trust_with_r(r: f32) -> TrustVector6D {
        TrustVector6D::new(r, 0.5, 0.5, 0.5, 0.5, 0.5)
    }

    #[test]
    fn test_newcomer_low_mana() {
        let config = ManaConfig::default();
        let trust = trust_with_r(0.1);
        let account = ManaAccount::new(&trust, &config);

        // Trust 0.1 → multiplier 1 + 0.1*100 = 11
        // max_mana = 10_000 * 11 = 110_000
        assert_eq!(account.max(), 110_000);
    }

    #[test]
    fn test_veteran_high_mana() {
        let config = ManaConfig::default();
        let trust = trust_with_r(0.9);
        let account = ManaAccount::new(&trust, &config);

        // Trust 0.9 → multiplier 1 + 0.9*100 = 91
        // max_mana = 10_000 * 91 = 910_000 (mit Fließkomma-Toleranz)
        let max = account.max();
        assert!(
            (max as i64 - 910_000i64).abs() < 10,
            "Expected ~910_000, got {}",
            max
        );
    }

    #[test]
    fn test_zero_trust_baseline() {
        let config = ManaConfig::default();
        let trust = trust_with_r(0.0);
        let account = ManaAccount::new(&trust, &config);

        // Trust 0.0 → multiplier 1
        // max_mana = 10_000 * 1 = 10_000
        assert_eq!(account.max(), 10_000);
    }

    #[test]
    fn test_consume_mana() {
        let config = ManaConfig::default();
        let trust = trust_with_r(0.5);
        let mut account = ManaAccount::new(&trust, &config);

        let initial = account.current();
        account.consume(5000).unwrap();

        assert_eq!(account.current(), initial - 5000);
    }

    #[test]
    fn test_consume_insufficient_mana() {
        let config = ManaConfig::default();
        let trust = trust_with_r(0.0); // Nur 10k Mana
        let mut account = ManaAccount::new(&trust, &config);

        // Versuche mehr zu verbrauchen als vorhanden
        let result = account.consume(20_000);

        assert!(result.is_err());
    }

    #[test]
    fn test_regen_rate_by_trust() {
        let config = ManaConfig::default();

        let low_trust = trust_with_r(0.1);
        let high_trust = trust_with_r(0.9);

        let low_acc = ManaAccount::new(&low_trust, &config);
        let high_acc = ManaAccount::new(&high_trust, &config);

        // Higher trust = faster regen
        assert!(high_acc.regen_rate() > low_acc.regen_rate());
    }

    #[test]
    fn test_bandwidth_tier() {
        assert_eq!(BandwidthTier::from_trust(0.05), BandwidthTier::Newcomer);
        assert_eq!(BandwidthTier::from_trust(0.2), BandwidthTier::Limited);
        assert_eq!(BandwidthTier::from_trust(0.5), BandwidthTier::Standard);
        assert_eq!(BandwidthTier::from_trust(0.7), BandwidthTier::Elevated);
        assert_eq!(BandwidthTier::from_trust(0.95), BandwidthTier::Veteran);
    }

    #[test]
    fn test_manager_preflight_check() {
        let manager = ManaManager::default_config();
        let trust = trust_with_r(0.5);

        // Sollte für normale Ops funktionieren
        let result = manager.preflight_check("did:erynoa:self:alice", &trust, 1000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_manager_deduct() {
        let manager = ManaManager::default_config();
        let trust = trust_with_r(0.5);
        let did = "did:erynoa:self:bob";

        let before = manager.get_status(did, &trust);
        manager.deduct(did, &trust, 5000).unwrap();
        let after = manager.get_status(did, &trust);

        assert_eq!(after.current_mana, before.current_mana - 5000);
    }

    #[test]
    fn test_sybil_attack_limited() {
        // Simuliere 10 Sybil-Accounts mit Trust 0.0
        let manager = ManaManager::default_config();
        let sybil_trust = trust_with_r(0.0);

        // Jeder Sybil hat nur 10k Mana
        for i in 0..10 {
            let did = format!("did:erynoa:self:sybil{}", i);
            let status = manager.get_status(&did, &sybil_trust);
            assert_eq!(status.max_mana, 10_000);
        }

        // Ein legitimer User mit Trust 0.8 hat viel mehr
        let legit_trust = trust_with_r(0.8);
        let legit_status = manager.get_status("did:erynoa:self:legit", &legit_trust);

        // 10k * (1 + 0.8*100) = 10k * 81 = 810k
        assert_eq!(legit_status.max_mana, 810_000);

        // Ein legitimer User hat mehr Mana als alle 10 Sybils zusammen!
        assert!(legit_status.max_mana > 10 * 10_000);
    }
}
