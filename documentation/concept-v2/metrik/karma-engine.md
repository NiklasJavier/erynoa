# ◊ METRIK – Karma Engine

> **Schicht:** 2 – Vertrauen
> **Sphäre:** ERY (Karmic-Modul)
> **Version:** 2.1 – Tiers, Asymmetrie, Decay

---

## Konzept

Die **Karma Engine** berechnet und verwaltet Trust Vectors. Sie implementiert Karma Tiers, asymmetrische Gewichtung und zeitlichen Verfall.

---

## Karma Tiers

Gestaffelte Vertrauensstufen basierend auf akkumuliertem Karma:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   KARMA TIERS                                                              │
│                                                                             │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │                                                                   │    │
│   │   TIER           KARMA RANGE      PRIVILEGES                     │    │
│   │   ════           ═══════════      ══════════                     │    │
│   │                                                                   │    │
│   │   🌱 Newcomer    0 - 100          Basis-Zugang                   │    │
│   │                                    Rate-Limited                   │    │
│   │                                    Keine Governance               │    │
│   │                                                                   │    │
│   │   ⭐ Established  100 - 500        Voller Zugang                  │    │
│   │                                    Normale Limits                 │    │
│   │                                    Voting: 1× Gewicht             │    │
│   │                                                                   │    │
│   │   🏆 Veteran      500 - 2000       Premium Features               │    │
│   │                                    Erhöhte Limits                 │    │
│   │                                    Voting: 2× Gewicht             │    │
│   │                                                                   │    │
│   │   👑 Elder        2000+            Governance-Rollen              │    │
│   │                                    Keine Limits                   │    │
│   │                                    Voting: 3× Gewicht             │    │
│   │                                    Council-Wählbarkeit            │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Asymmetrie: Negativ wiegt stärker

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ASYMMETRIE-PRINZIP                                                       │
│                                                                             │
│   Positive Events:  reward_weight = 1.0                                    │
│   Negative Events:  penalty_weight = 1.5  ← 50% stärker!                   │
│                                                                             │
│   Beispiel:                                                                │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   ✅ Erfolgreiche Transaktion  →  +0.02 Trust                       │  │
│   │   ❌ Fehlgeschlagene Transaktion →  -0.03 Trust (×1.5)             │  │
│   │                                                                     │  │
│   │   🚫 Betrug erkannt            →  -0.45 Trust (×1.5)               │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│   Grund: Vertrauen ist schwer aufzubauen, leicht zu zerstören.            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Event-Typen und Gewichtung

| Event-Typ          | Trust-Wirkung | Gewicht | Beispiel                  |
| ------------------ | ------------- | ------- | ------------------------- |
| ✅ **Success**     | +0.02         | 1.0     | Transaktion abgeschlossen |
| ⚠️ **Warning**     | -0.005        | 1.5     | Leichte Verzögerung       |
| ❌ **Failure**     | -0.05         | 1.5     | Service abgebrochen       |
| 🚫 **Fraud**       | -0.30         | 1.5     | Betrug erkannt            |
| 🏅 **Attestation** | +0.10         | 1.0     | Zertifizierung erhalten   |
| ⏰ **Inactivity**  | -0.001/Tag    | 1.0     | Decay bei Inaktivität     |

---

## Decay-Mechanismus

Trust verfällt bei Inaktivität:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   DECAY FUNCTION                                                           │
│                                                                             │
│   trust_decayed = trust × (decay_rate ^ days_inactive)                     │
│                                                                             │
│   Default: decay_rate = 0.999                                              │
│                                                                             │
│   Beispiel:                                                                │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   Trust = 0.90                                                      │  │
│   │                                                                     │  │
│   │   Nach 30 Tagen:  0.90 × 0.999^30  = 0.873                         │  │
│   │   Nach 90 Tagen:  0.90 × 0.999^90  = 0.822                         │  │
│   │   Nach 365 Tagen: 0.90 × 0.999^365 = 0.627                         │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│   Aktive Teilnehmer behalten Trust, inaktive verlieren ihn.               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Karma Engine Konfiguration

```yaml
karma_engine {
  # Gewichtung
  reward_weight:  1.0
  penalty_weight: 1.5

  # Decay
  decay_rate:     0.999  # pro Tag
  decay_floor:    0.3    # Minimum Trust

  # Tiers
  tiers: {
    newcomer:    { min: 0,    max: 100 }
    established: { min: 100,  max: 500 }
    veteran:     { min: 500,  max: 2000 }
    elder:       { min: 2000, max: ∞ }
  }

  # Voting-Gewichte
  voting_weights: {
    newcomer:    0
    established: 1
    veteran:     2
    elder:       3
  }
}
```

---

## Karma-Berechnung

```
karma_new = karma_old + Σ(event_impact × event_weight × tier_modifier)

where:
  event_impact = base_impact × (penalty_weight if negative else reward_weight)
  tier_modifier = 1.0 + (tier_level × 0.1)  # Höhere Tiers haben mehr Einfluss
```

---

## Karma → Trust Mapping

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   KARMA TO TRUST CONVERSION                                                │
│                                                                             │
│   Trust-Dimension wird aus Events der Dimension berechnet:                 │
│                                                                             │
│   reliability = f(delivery_events, uptime_events, ...)                     │
│   integrity   = f(accuracy_events, honesty_events, ...)                    │
│   capability  = f(quality_events, capacity_events, ...)                    │
│   reputation  = f(attestation_events, endorsement_events, ...)            │
│                                                                             │
│   Aggregate Karma beeinflusst den Tier:                                    │
│   total_karma = Σ(all positive events) - Σ(all negative events × 1.5)     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Weiterführende Dokumente

- [trust-vectors.md](./trust-vectors.md) – Mehrdimensionales Trust
- [attestations.md](./attestations.md) – Externe Bestätigungen
- [reputation.md](./reputation.md) – Vererbung
