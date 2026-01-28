# ◆ CHRONIK – NOA Ledger

> **Schicht:** 5 – Beweis
> **Sphäre:** NOA (Causal Ledger)
> **Kernfrage:** _„Was ist bewiesen geschehen?"_

---

## Konzept

Der **NOA Ledger** ist das kausale Beweissystem von Erynoa. Er speichert nicht nur _was_ passiert ist, sondern _warum_ und _in welcher Reihenfolge_.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   NOA = CAUSALLY ORDERED PROOF LAYER                                       │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   Traditionaler Ledger:                                             │  │
│   │   ═══════════════════════                                           │  │
│   │   Block N: [Tx1, Tx2, Tx3]  →  Block N+1: [Tx4, Tx5]               │  │
│   │   (Reihenfolge, aber keine Kausalität)                              │  │
│   │                                                                     │  │
│   │   NOA Ledger:                                                       │  │
│   │   ═══════════                                                       │  │
│   │   ┌─────────────────────────────────────────────────────────────┐  │  │
│   │   │                                                             │  │  │
│   │   │        Intent         Agreement       Execution             │  │  │
│   │   │           │               │               │                 │  │  │
│   │   │           ▼               ▼               ▼                 │  │  │
│   │   │        ┌─────┐        ┌─────┐        ┌─────┐               │  │  │
│   │   │        │ E1  │───────▶│ E2  │───────▶│ E3  │               │  │  │
│   │   │        └─────┘        └─────┘        └─────┘               │  │  │
│   │   │           │               │               │                 │  │  │
│   │   │           │               │               ▼                 │  │  │
│   │   │           │               │           ┌─────┐               │  │  │
│   │   │           │               └──────────▶│ E4  │ Finalization  │  │  │
│   │   │           │                           └─────┘               │  │  │
│   │   │           │                               │                 │  │  │
│   │   │           │                               ▼                 │  │  │
│   │   │           └──────────────────────────▶Trust Event           │  │  │
│   │   │                                                             │  │  │
│   │   └─────────────────────────────────────────────────────────────┘  │  │
│   │                                                                     │  │
│   │   Jedes Event zeigt auf seine Ursachen (DAG, nicht Chain).         │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Event-Struktur

```yaml
event {
  id:   "did:erynoa:event:tx-2025-001-003"
  type: transaction_execution

  # Kausale Referenzen
  causes: [
    @ref("did:erynoa:event:tx-2025-001-001"),  # Intent
    @ref("did:erynoa:event:tx-2025-001-002")   # Agreement
  ]

  # Zeitstempel
  timestamp: "2025-01-28T10:30:00Z"

  # Inhalt
  payload: {
    transaction: @ref("did:erynoa:transaction:charging-2025-001")
    action:      "execute"
    metrics: {
      energy_delivered: 45  # kWh
      duration:         28  # min
      amount_paid:      18.90  # EUR
    }
  }

  # Teilnehmer
  participants: [
    @identity("did:erynoa:agent:seeker:vehicle-123"),
    @identity("did:erynoa:agent:provider:swm")
  ]

  # Proof (IOTA Anchor)
  anchor: {
    chain:    "iota"
    block:    "0xabc123..."
    index:    42
    verified: true
  }
}
```

---

## Event-Typen

| Typ                     | Beschreibung           | Triggers          |
| ----------------------- | ---------------------- | ----------------- |
| **intent_created**      | Agent erstellt Absicht | Discovery         |
| **offer_made**          | Provider macht Angebot | Negotiation       |
| **agreement_reached**   | Parteien einigen sich  | Transaction Start |
| **execution_started**   | Dienstleistung beginnt | Streaming Start   |
| **execution_completed** | Dienstleistung endet   | Finalization      |
| **payment_streamed**    | Zahlung fließt         | Wallet Update     |
| **trust_updated**       | Trust-Wert ändert sich | Karma Update      |
| **amo_transitioned**    | AMO wechselt Status    | State Change      |

---

## Kausale Ordnung

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   CAUSAL ORDERING (DAG)                                                    │
│                                                                             │
│   Intent A        Intent B                                                  │
│      │               │                                                      │
│      ▼               ▼                                                      │
│   Offer A1       Offer B1                                                   │
│      │               │                                                      │
│      ▼               ▼                                                      │
│   Agree A        Agree B                                                    │
│      │               │                                                      │
│      │               │                                                      │
│      ▼               │                                                      │
│   Execute A          │                                                      │
│      │               │                                                      │
│      │               │                                                      │
│      └───────┬───────┘                                                      │
│              │                                                              │
│              ▼                                                              │
│         Trust Update                                                        │
│         (depends on both A and B outcomes)                                  │
│                                                                             │
│   Events können parallel sein, solange sie nicht kausal abhängen.          │
│   Finale Ordnung wird beim Anchoring festgelegt.                           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Finality

Events erreichen Finality durch Anchoring:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   FINALITY LEVELS                                                          │
│                                                                             │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │                                                                   │    │
│   │   Level 0: PENDING                                                │    │
│   │   ═══════════════════                                             │    │
│   │   Event erstellt, noch nicht verteilt                             │    │
│   │   Kann noch geändert/storniert werden                             │    │
│   │                                                                   │    │
│   │   Level 1: DISTRIBUTED                                            │    │
│   │   ════════════════════                                            │    │
│   │   Event an Netzwerk verteilt                                      │    │
│   │   Wird von Peers validiert                                        │    │
│   │                                                                   │    │
│   │   Level 2: ANCHORED                                               │    │
│   │   ═════════════════                                               │    │
│   │   Event auf IOTA/Ethereum anchored                                │    │
│   │   Unveränderlich, aber Anchor könnte theoretisch reorg'd werden   │    │
│   │                                                                   │    │
│   │   Level 3: FINAL                                                  │    │
│   │   ═══════════════                                                 │    │
│   │   Anchor hat genug Confirmations                                  │    │
│   │   Praktisch irreversibel                                          │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Query-Beispiele

```yaml
# Alle Events einer Transaktion
query events {
  filter: {
    transaction: @ref("did:erynoa:transaction:charging-2025-001")
  }
  order: causal  # Kausale Reihenfolge
}

# Trust-History einer Identität
query events {
  filter: {
    type: trust_updated
    subject: @identity("did:erynoa:agent:provider:swm")
    timerange: {
      from: "2025-01-01"
      to:   "2025-01-31"
    }
  }
  order: chronological
}
```

---

## Weiterführende Dokumente

- [amo.md](./amo.md) – Objekte
- [logic-guards.md](./logic-guards.md) – Validierung
- [streaming.md](./streaming.md) – Value Streaming
- [finality.md](./finality.md) – Anchoring
