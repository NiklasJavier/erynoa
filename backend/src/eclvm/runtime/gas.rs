//! # Gas Metering
//!
//! Schützt vor Endlosschleifen und DoS-Attacken durch Gas-Limits.
//!
//! Jede Operation kostet Gas. Wenn das Gas-Limit erreicht ist,
//! wird die Ausführung abgebrochen.

use crate::error::{ApiError, Result};
use anyhow::anyhow;

/// Gas Meter - Trackt und limitiert den Gasverbrauch
#[derive(Debug, Clone)]
pub struct GasMeter {
    /// Verbleibendes Gas
    remaining: u64,
    /// Ursprüngliches Limit
    limit: u64,
    /// Verbrauchtes Gas
    consumed: u64,
}

impl GasMeter {
    /// Erstelle neuen GasMeter mit Limit
    pub fn new(limit: u64) -> Self {
        Self {
            remaining: limit,
            limit,
            consumed: 0,
        }
    }

    /// Unbegrenztes Gas (für Tests)
    pub fn unlimited() -> Self {
        Self {
            remaining: u64::MAX,
            limit: u64::MAX,
            consumed: 0,
        }
    }

    /// Verbrauche Gas für eine Operation
    pub fn consume(&mut self, amount: u64) -> Result<()> {
        if amount > self.remaining {
            return Err(ApiError::Internal(anyhow!(
                "Out of gas: needed {} but only {} remaining (consumed {} of {} limit)",
                amount,
                self.remaining,
                self.consumed,
                self.limit
            )));
        }
        self.remaining -= amount;
        self.consumed += amount;
        Ok(())
    }

    /// Verbleibendes Gas
    pub fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Verbrauchtes Gas
    pub fn consumed(&self) -> u64 {
        self.consumed
    }

    /// Ursprüngliches Limit
    pub fn limit(&self) -> u64 {
        self.limit
    }

    /// Prozent verbraucht
    pub fn usage_percent(&self) -> f64 {
        if self.limit == 0 {
            return 0.0;
        }
        (self.consumed as f64 / self.limit as f64) * 100.0
    }

    /// Gas zurückgeben (z.B. bei erfolgreicher Transaktion)
    pub fn refund(&mut self, amount: u64) {
        let refund = amount.min(self.consumed);
        self.remaining += refund;
        self.consumed -= refund;
    }

    /// Reset auf Anfangszustand
    pub fn reset(&mut self) {
        self.remaining = self.limit;
        self.consumed = 0;
    }
}

/// Gas-Kosten Schätzung für ein Programm
pub fn estimate_gas(program: &[crate::eclvm::OpCode]) -> u64 {
    program.iter().map(|op| op.gas_cost()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_consumption() {
        let mut meter = GasMeter::new(100);

        assert_eq!(meter.remaining(), 100);
        assert_eq!(meter.consumed(), 0);

        meter.consume(30).unwrap();
        assert_eq!(meter.remaining(), 70);
        assert_eq!(meter.consumed(), 30);

        meter.consume(50).unwrap();
        assert_eq!(meter.remaining(), 20);
        assert_eq!(meter.consumed(), 80);
    }

    #[test]
    fn test_out_of_gas() {
        let mut meter = GasMeter::new(10);

        meter.consume(5).unwrap();

        let result = meter.consume(10);
        assert!(result.is_err());

        // State sollte unverändert sein nach Fehler
        assert_eq!(meter.remaining(), 5);
    }

    #[test]
    fn test_refund() {
        let mut meter = GasMeter::new(100);

        meter.consume(80).unwrap();
        assert_eq!(meter.remaining(), 20);

        meter.refund(30);
        assert_eq!(meter.remaining(), 50);
        assert_eq!(meter.consumed(), 50);
    }

    #[test]
    fn test_usage_percent() {
        let mut meter = GasMeter::new(100);

        meter.consume(25).unwrap();
        assert!((meter.usage_percent() - 25.0).abs() < 0.001);

        meter.consume(25).unwrap();
        assert!((meter.usage_percent() - 50.0).abs() < 0.001);
    }
}
