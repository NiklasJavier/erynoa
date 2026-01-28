# ◆ CHRONIK – Value Streaming

> **Schicht:** 5 – Beweis
> **Sphäre:** NOA (Payment Layer)
> **Typ:** Kontinuierlicher Werttransfer

---

## Konzept

**Value Streaming** ermöglicht kontinuierlichen Werttransfer während laufender Dienste. Statt Einmalzahlungen fließt Wert in Echtzeit.

---

## Warum Streaming?

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   TRADITIONAL vs. STREAMING                                                │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   TRADITIONAL (Einmalzahlung)                                       │  │
│   │   ═══════════════════════════                                       │  │
│   │                                                                     │  │
│   │   Dienst startet        Dienst endet         Zahlung               │  │
│   │        │                     │                  │                  │  │
│   │        ▼                     ▼                  ▼                  │  │
│   │   ┌─────────────────────────────────────┐  ┌─────┐                │  │
│   │   │         45 kWh Laden                │  │21€  │                │  │
│   │   └─────────────────────────────────────┘  └─────┘                │  │
│   │                                                                     │  │
│   │   Risiken:                                                         │  │
│   │   • Seeker könnte nicht zahlen                                     │  │
│   │   • Provider muss vorleisten                                       │  │
│   │   • Abbruch ist kompliziert                                        │  │
│   │                                                                     │  │
│   │   ─────────────────────────────────────────────────────────────   │  │
│   │                                                                     │  │
│   │   STREAMING (Kontinuierlich)                                        │  │
│   │   ══════════════════════════                                        │  │
│   │                                                                     │  │
│   │   ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐        │  │
│   │   │     │     │     │     │     │     │     │     │     │        │  │
│   │   │ 2€  │ 2€  │ 2€  │ 2€  │ 3€  │ 3€  │ 3€  │ 2€  │ 2€  │ = 21€ │  │
│   │   │     │     │     │     │     │     │     │     │     │        │  │
│   │   └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘        │  │
│   │     ▲                                               ▲             │  │
│   │   Start                                           Ende             │  │
│   │                                                                     │  │
│   │   Vorteile:                                                        │  │
│   │   • Kein Vorschuss nötig                                          │  │
│   │   • Provider hat Sicherheit                                        │  │
│   │   • Abbruch jederzeit fair                                        │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Stream-Definition

```yaml
payment_stream {
  id: "did:erynoa:stream:charging-2025-001"

  # Parteien
  sender: {
    wallet:  @ref("did:erynoa:wallet:vehicle-123")
    agent:   @identity("did:erynoa:agent:seeker:vehicle-123")
  }

  receiver: {
    wallet:  @ref("did:erynoa:wallet:provider-swm")
    agent:   @identity("did:erynoa:agent:provider:swm")
  }

  # Rate-Konfiguration
  rate: {
    type:     "usage"  # oder "time"
    amount:   0.42     # EUR
    per:      "kWh"    # Einheit
  }

  # Alternativ: zeitbasiert
  # rate: {
  #   type:   "time"
  #   amount: 0.10    # EUR
  #   per:    "minute"
  # }

  # Limits
  limits: {
    max_amount:    50.00   # EUR
    max_duration:  3600    # Sekunden
  }

  # Referenzen
  agreement: @ref("did:erynoa:agreement:charging-2025-001")
  service:   @ref("did:erynoa:amo:service:session-2025-001")

  # Status
  status:     active
  started_at: "2025-01-28T10:20:00Z"

  # Akkumuliert
  transferred: {
    amount:   14.50
    units:    34.5  # kWh
    at:       "2025-01-28T10:35:00Z"
  }
}
```

---

## Stream-Lifecycle

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   STREAM LIFECYCLE                                                         │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   1. OPEN                                                           │  │
│   │      ════════                                                       │  │
│   │      Stream wird erstellt nach Agreement                            │  │
│   │      Sender-Wallet wird reserviert (Escrow)                        │  │
│   │           │                                                         │  │
│   │           ▼                                                         │  │
│   │   2. STREAMING                                                      │  │
│   │      ═══════════                                                    │  │
│   │      Wert fließt kontinuierlich                                    │  │
│   │      Intervall: typisch alle 10-60 Sekunden                        │  │
│   │      Jeder Transfer = Event auf NOA                                │  │
│   │           │                                                         │  │
│   │           │                                                         │  │
│   │    ┌──────┴──────┐                                                 │  │
│   │    │             │                                                 │  │
│   │    ▼             ▼                                                 │  │
│   │ 3a. COMPLETE  3b. ABORT                                            │  │
│   │ ════════════  ═══════════                                          │  │
│   │ Dienst endet  Vorzeitiger                                          │  │
│   │ normal        Abbruch                                              │  │
│   │    │             │                                                 │  │
│   │    │             │                                                 │  │
│   │    └──────┬──────┘                                                 │  │
│   │           │                                                         │  │
│   │           ▼                                                         │  │
│   │   4. SETTLE                                                         │  │
│   │      ════════                                                       │  │
│   │      Finale Abrechnung                                             │  │
│   │      Unreservierter Betrag zurück an Sender                        │  │
│   │      Event: stream_settled                                         │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Stream-Events

```yaml
# Stream-Start
event {
  type:   stream_opened
  stream: @ref("did:erynoa:stream:charging-2025-001")

  reserved: 50.00  # Escrow
}

# Transfer (alle 30 Sekunden)
event {
  type:   stream_transfer
  stream: @ref("did:erynoa:stream:charging-2025-001")

  delta: {
    amount: 2.10
    units:  5.0  # kWh
  }

  cumulative: {
    amount: 14.50
    units:  34.5
  }
}

# Stream-Ende
event {
  type:   stream_settled
  stream: @ref("did:erynoa:stream:charging-2025-001")

  final: {
    total_amount:   18.90
    total_units:    45.0
    duration:       1680  # Sekunden
    returned:       31.10  # Unreserviert
  }

  outcome: completed  # oder aborted
}
```

---

## Abbruch-Handling

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ABORT SCENARIOS                                                          │
│                                                                             │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │                                                                   │    │
│   │   Scenario: Seeker bricht ab                                      │    │
│   │   ──────────────────────────                                      │    │
│   │   • Stream wird gestoppt                                          │    │
│   │   • Bereits gelieferter Wert wird bezahlt                         │    │
│   │   • Kein Penalty (normaler Abbruch)                               │    │
│   │                                                                   │    │
│   │   Scenario: Provider bricht ab                                    │    │
│   │   ────────────────────────────                                    │    │
│   │   • Stream wird gestoppt                                          │    │
│   │   • Seeker zahlt nur für gelieferten Wert                         │    │
│   │   • Möglicher Trust-Impact für Provider                           │    │
│   │                                                                   │    │
│   │   Scenario: Wallet insufficient                                   │    │
│   │   ─────────────────────────────                                   │    │
│   │   • Stream pausiert                                               │    │
│   │   • Seeker hat Grace Period zum Aufladen                          │    │
│   │   • Nach Timeout: Abbruch mit Penalty                             │    │
│   │                                                                   │    │
│   │   Scenario: Limit erreicht                                        │    │
│   │   ───────────────────────────                                     │    │
│   │   • Stream stoppt automatisch                                     │    │
│   │   • Settlement mit max_amount                                     │    │
│   │   • Kein Penalty                                                  │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Weiterführende Dokumente

- [noa-ledger.md](./noa-ledger.md) – Event-Speicherung
- [finality.md](./finality.md) – Settlement-Finalisierung
- [../impuls/wallet.md](../impuls/wallet.md) – Wallet-Management
