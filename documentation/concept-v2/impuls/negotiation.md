# ◐ IMPULS – Negotiation

> **Schicht:** 4 – Handlung
> **Sphäre:** ECHO (Swarm-Modul)
> **Typ:** Verhandlungsprotokolle

---

## Konzept

**Negotiation** ist der Prozess, bei dem Agenten von einem Intent zu einer Vereinbarung kommen. Es unterstützt direkte Annahme, Auktionen und Multi-Round-Verhandlung.

---

## Negotiation-Modelle

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   NEGOTIATION MODELS                                                       │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   1️⃣ DIRECT (Take-it-or-leave-it)                                   │  │
│   │      ══════════════════════════════                                 │  │
│   │      Provider macht Angebot → Seeker akzeptiert/lehnt ab            │  │
│   │      Schnell, einfach, keine Verhandlung                           │  │
│   │                                                                     │  │
│   │   2️⃣ AUCTION (Competitive Bidding)                                  │  │
│   │      ═══════════════════════════════                                │  │
│   │      Seeker schreibt aus → Mehrere Provider bieten                 │  │
│   │      Bestes Angebot gewinnt                                        │  │
│   │                                                                     │  │
│   │   3️⃣ MULTI-ROUND (Haggling)                                         │  │
│   │      ═══════════════════════                                        │  │
│   │      Angebot → Gegenangebot → Angebot → ... → Einigung             │  │
│   │      Komplexer, flexibler                                          │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Direct Negotiation

```yaml
# Seeker Intent
intent {
  type: charge_vehicle
  negotiation_model: direct
  ...
}

# Provider Offer (automatisch generiert basierend auf Policy)
offer {
  id:         "did:erynoa:offer:charging-001"
  in_response_to: @ref("did:erynoa:intent:charge-2025-001")

  provider:   @identity("did:erynoa:agent:provider:swm")

  terms: {
    price_per_kwh:  0.42
    available_at:   "2025-01-28T10:15:00Z"
    estimated_time: 25  # Minuten
  }

  valid_until: "2025-01-28T10:30:00Z"  # 15 Min gültig
}

# Seeker Response
response {
  offer:  @ref("did:erynoa:offer:charging-001")
  action: accept  # oder reject

  # Bei Accept → Transaktion startet
}
```

---

## Auction Negotiation

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   AUCTION FLOW                                                             │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   Seeker                    Providers                               │  │
│   │   ══════                    ═════════                               │  │
│   │                                                                     │  │
│   │   Intent (auction)                                                  │  │
│   │   ────────────────▶         Provider A: 0.45€/kWh                  │  │
│   │                             Provider B: 0.42€/kWh                  │  │
│   │                             Provider C: 0.48€/kWh                  │  │
│   │                                                                     │  │
│   │                     ◀────────────────                               │  │
│   │   Bidding closes                                                    │  │
│   │   ─────────────────                                                │  │
│   │                                                                     │  │
│   │   Evaluate:                                                         │  │
│   │   Score(A) = price(0.3) + trust(0.3) + distance(0.4) = 0.78        │  │
│   │   Score(B) = price(0.35) + trust(0.25) + distance(0.35) = 0.95 ← WIN│ │
│   │   Score(C) = price(0.25) + trust(0.3) + distance(0.3) = 0.85       │  │
│   │                                                                     │  │
│   │   Accept(B)                                                         │  │
│   │   ────────────────▶         Provider B: Akzeptiert                 │  │
│   │                             Provider A, C: Abgelehnt               │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Multi-Round Negotiation

```yaml
negotiation "multi-round" {
  id:     "did:erynoa:negotiation:haggle-001"
  type:   multi_round
  rounds: []  # Wird gefüllt

  # Konfiguration
  config: {
    max_rounds:     5
    round_timeout:  60  # Sekunden
    final_offer:    true  # Letzte Runde ist verbindlich
  }
}

# Runde 1
round {
  number:    1
  from:      provider
  terms:     { price: 0.50 }
  status:    counter_offered
}

# Runde 2
round {
  number:    2
  from:      seeker
  terms:     { price: 0.38 }
  status:    counter_offered
}

# Runde 3
round {
  number:    3
  from:      provider
  terms:     { price: 0.44 }
  status:    counter_offered
}

# Runde 4
round {
  number:    4
  from:      seeker
  terms:     { price: 0.42 }
  status:    accepted  # Provider akzeptiert
}
```

---

## Negotiation-Ergebnis

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   NEGOTIATION OUTCOMES                                                     │
│                                                                             │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │                                                                   │    │
│   │   ✅ AGREED                                                        │    │
│   │      Beide Parteien haben sich geeinigt                           │    │
│   │      → Agreement wird erstellt                                     │    │
│   │      → Transaktion beginnt                                         │    │
│   │                                                                   │    │
│   │   ❌ REJECTED                                                       │    │
│   │      Seeker oder Provider lehnt final ab                          │    │
│   │      → Negotiation endet                                           │    │
│   │      → Keine Trust-Auswirkung (normal)                            │    │
│   │                                                                   │    │
│   │   ⏰ TIMEOUT                                                        │    │
│   │      Keine Einigung innerhalb der Zeit                            │    │
│   │      → Negotiation endet                                           │    │
│   │      → Leichte negative Trust-Auswirkung                          │    │
│   │                                                                   │    │
│   │   🔄 CANCELLED                                                      │    │
│   │      Eine Partei zieht zurück                                     │    │
│   │      → Abhängig vom Zeitpunkt: Trust-Impact                       │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Agreement (Vertrag)

```yaml
agreement {
  id:   "did:erynoa:agreement:charging-2025-001"

  # Parteien
  seeker:   @identity("did:erynoa:agent:seeker:vehicle-123")
  provider: @identity("did:erynoa:agent:provider:swm")

  # Vereinbarte Bedingungen
  terms: {
    service:        @ref("did:erynoa:amo:service:charging-session")
    price_per_kwh:  0.42
    max_amount:     30.00
    currency:       EUR
    start_time:     "2025-01-28T10:20:00Z"
  }

  # Referenzen
  intent:      @ref("did:erynoa:intent:charge-2025-001")
  negotiation: @ref("did:erynoa:negotiation:haggle-001")

  # Status
  status:     active
  created_at: "2025-01-28T10:18:00Z"

  # Wird zu Transaktion (siehe CHRONIK)
}
```

---

## Weiterführende Dokumente

- [intent.md](./intent.md) – Absichten
- [policy.md](./policy.md) – Entscheidungsregeln
- [../chronik/noa-ledger.md](../chronik/noa-ledger.md) – Finalisierung
