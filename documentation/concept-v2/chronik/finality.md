# ◆ CHRONIK – Finality

> **Schicht:** 5 – Beweis
> **Sphäre:** NOA (Anchoring Layer)
> **Typ:** Unveränderlichkeit und Beweissicherung

---

## Konzept

**Finality** ist der Zustand, in dem ein Event oder eine Transaktion unveränderlich und nachweisbar wird. Erynoa nutzt Multi-Chain-Anchoring für robuste Finality.

---

## Finality-Stufen

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   FINALITY PROGRESSION                                                     │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   Zeit ────────────────────────────────────────────────────────▶   │  │
│   │                                                                     │  │
│   │   Level 0          Level 1         Level 2         Level 3         │  │
│   │   PENDING          DISTRIBUTED     ANCHORED        FINAL            │  │
│   │   ═══════          ═══════════     ════════        ═════            │  │
│   │                                                                     │  │
│   │   ┌─────┐          ┌─────┐         ┌─────┐         ┌─────┐         │  │
│   │   │     │ ───────▶ │     │ ──────▶ │     │ ──────▶ │ ✓✓✓ │         │  │
│   │   └─────┘          └─────┘         └─────┘         └─────┘         │  │
│   │                                                                     │  │
│   │   Erstellt         An Netzwerk     Auf Chain       Genug           │  │
│   │   lokal            verteilt        geschrieben     Confirmations   │  │
│   │                                                                     │  │
│   │   Reversibel ──────────────────────────────────▶ Irreversibel      │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Anchoring-Prozess

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ANCHORING ARCHITECTURE                                                   │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   Events auf NOA                                                    │  │
│   │        │                                                            │  │
│   │        │ Batch (alle N Sekunden oder M Events)                     │  │
│   │        ▼                                                            │  │
│   │   ┌─────────────────────────────────────────────────────────────┐  │  │
│   │   │ Merkle Tree Builder                                         │  │  │
│   │   │                                                             │  │  │
│   │   │  Event 1 ──┐                                                │  │  │
│   │   │            ├──▶ Hash A ──┐                                  │  │  │
│   │   │  Event 2 ──┘             │                                  │  │  │
│   │   │                          ├──▶ Merkle Root                   │  │  │
│   │   │  Event 3 ──┐             │                                  │  │  │
│   │   │            ├──▶ Hash B ──┘                                  │  │  │
│   │   │  Event 4 ──┘                                                │  │  │
│   │   │                                                             │  │  │
│   │   └──────────────────────────┬──────────────────────────────────┘  │  │
│   │                              │                                      │  │
│   │                              ▼                                      │  │
│   │   ┌─────────────────────────────────────────────────────────────┐  │  │
│   │   │ Multi-Chain Anchoring                                       │  │  │
│   │   │                                                             │  │  │
│   │   │  Merkle Root ─────┬─────────────▶ IOTA (Primary)           │  │  │
│   │   │                   │                                         │  │  │
│   │   │                   ├─────────────▶ Ethereum (Secondary)     │  │  │
│   │   │                   │                                         │  │  │
│   │   │                   └─────────────▶ Solana (Optional)        │  │  │
│   │   │                                                             │  │  │
│   │   └─────────────────────────────────────────────────────────────┘  │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Anchor-Record

```yaml
anchor {
  id:          "did:erynoa:anchor:2025-01-28-001"
  merkle_root: "0x7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069"

  # Events in diesem Anchor
  events: [
    @ref("did:erynoa:event:tx-001"),
    @ref("did:erynoa:event:tx-002"),
    @ref("did:erynoa:event:tx-003"),
    @ref("did:erynoa:event:tx-004")
  ]

  # Chain-Anchors
  chains: {
    iota: {
      block:        "0xabc123..."
      index:        42
      timestamp:    "2025-01-28T10:35:00Z"
      confirmations: 12
      status:       final
    }

    ethereum: {
      block:        18234567
      tx_hash:      "0xdef456..."
      timestamp:    "2025-01-28T10:35:12Z"
      confirmations: 6
      status:       final
    }
  }

  # Gesamt-Status
  status: final
  finalized_at: "2025-01-28T10:37:00Z"
}
```

---

## Finality-Konfiguration

```yaml
finality_config {
  # Batch-Settings
  batch: {
    max_events:      100
    max_wait:        30   # Sekunden
  }

  # Chain-Anforderungen
  chains: {
    iota: {
      enabled:      true
      required:     true   # Muss erfolgreich sein
      confirmations: 10    # Für Finality
    }

    ethereum: {
      enabled:      true
      required:     false  # Nice-to-have
      confirmations: 6
    }
  }

  # Finality-Kriterien
  finality: {
    mode:            "primary"   # Nur IOTA reicht
    # mode:          "all"       # Alle Chains müssen bestätigen
    timeout:         300         # Sekunden bis Fallback
  }
}
```

---

## Proof-Verification

```yaml
# Proof dass ein Event finalized ist
proof_request {
  event: @ref("did:erynoa:event:tx-001")
}

proof_response {
  event:       @ref("did:erynoa:event:tx-001")
  finalized:   true

  # Merkle-Proof
  merkle_proof: {
    root:   "0x7f83b1657..."
    path:   ["0xaaa...", "0xbbb..."]
    index:  0
  }

  # Chain-Proofs
  chain_proofs: {
    iota: {
      block:        "0xabc123..."
      verified:     true
      verified_at:  "2025-01-28T10:40:00Z"
    }
  }
}
```

---

## Rechtliche Bedeutung

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   FINALITY = RECHTSSICHERHEIT                                              │
│                                                                             │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │                                                                   │    │
│   │   Level 3 (FINAL) bedeutet:                                       │    │
│   │                                                                   │    │
│   │   ✓ Unveränderlich – Event kann nicht mehr geändert werden       │    │
│   │   ✓ Nachweisbar    – Kryptographischer Beweis existiert          │    │
│   │   ✓ Auditierbar    – Dritte können unabhängig prüfen             │    │
│   │   ✓ Rechtsgültig   – Kann als Beweis in Verfahren dienen         │    │
│   │                                                                   │    │
│   │   Anwendungsfälle:                                                │    │
│   │   • Eichrechtskonforme Abrechnung                                 │    │
│   │   • Audit-Trail für Compliance                                    │    │
│   │   • Streitbeilegung                                               │    │
│   │   • Regulatorische Nachweise                                      │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Weiterführende Dokumente

- [noa-ledger.md](./noa-ledger.md) – Event-Struktur
- [streaming.md](./streaming.md) – Payment Settlement
- [../nexus/multi-chain.md](../nexus/multi-chain.md) – Chain-Details
