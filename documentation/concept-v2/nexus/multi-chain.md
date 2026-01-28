# ⬡ NEXUS – Multi-Chain

> **Schicht:** 6 – Vernetzung
> **Sphäre:** NOA (Chain Layer)
> **Kernfrage:** _„Wo wird es verankert?"_

---

## Konzept

**Multi-Chain** beschreibt Erynoas Strategie, mehrere Blockchain-Netzwerke für Anchoring zu nutzen – für Redundanz, Kostenoptimierung und Interoperabilität.

---

## Chain-Hierarchie

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   MULTI-CHAIN HIERARCHY                                                    │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   PRIMARY CHAIN: IOTA                                               │  │
│   │   ═══════════════════════                                           │  │
│   │   • Feeless Transactions                                            │  │
│   │   • DAG-basiert (Tangle)                                           │  │
│   │   • Schnelle Finality (~10s)                                       │  │
│   │   • Alle Events werden hier anchored                               │  │
│   │                                                                     │  │
│   │   ─────────────────────────────────────────────────────────────────│  │
│   │                                                                     │  │
│   │   SECONDARY CHAINS                                                  │  │
│   │   ════════════════                                                  │  │
│   │                                                                     │  │
│   │   ┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐  │  │
│   │   │    Ethereum     │   │     Solana      │   │     Polygon     │  │  │
│   │   │                 │   │                 │   │                 │  │  │
│   │   │ • High-Value    │   │ • High-Speed    │   │ • Low-Cost      │  │  │
│   │   │ • DeFi-Bridge   │   │ • High-Freq     │   │ • EVM-Compat    │  │  │
│   │   │ • Max Security  │   │ • Trading       │   │ • Scaling       │  │  │
│   │   └─────────────────┘   └─────────────────┘   └─────────────────┘  │  │
│   │                                                                     │  │
│   │   Optional, für spezifische Use Cases                              │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Chain-Eigenschaften

| Chain        | Kosten          | Speed | Security  | Use Case          |
| ------------ | --------------- | ----- | --------- | ----------------- |
| **IOTA**     | Feeless         | ~10s  | Mittel    | Primary Anchoring |
| **Ethereum** | Hoch ($1-50)    | ~15s  | Sehr hoch | High-Value, DeFi  |
| **Solana**   | Niedrig ($0.01) | ~0.4s | Mittel    | High-Frequency    |
| **Polygon**  | Sehr niedrig    | ~2s   | Mittel    | Volume Scaling    |

---

## Chain-Selection Logic

```yaml
chain_selection {
  # Regeln für Chain-Auswahl
  rules: [
    {
      condition: "transaction.value > 10000"  # EUR
      chains:    [iota, ethereum]              # High-Value = Multi-Anchor
    },
    {
      condition: "transaction.type == 'streaming'"
      chains:    [iota]                        # Streaming = nur IOTA (feeless)
    },
    {
      condition: "transaction.defi_enabled == true"
      chains:    [iota, ethereum]              # DeFi = Ethereum nötig
    },
    {
      condition: "default"
      chains:    [iota]                        # Standard = IOTA only
    }
  ]
}
```

---

## IOTA Integration

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   IOTA INTEGRATION                                                         │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   NOA Events                                                        │  │
│   │        │                                                            │  │
│   │        │ Batch + Merkle Root                                       │  │
│   │        ▼                                                            │  │
│   │   ┌─────────────────────────────────────────────────────────────┐  │  │
│   │   │ IOTA Message                                                │  │  │
│   │   │                                                             │  │  │
│   │   │  {                                                          │  │  │
│   │   │    "type": "erynoa_anchor",                                │  │  │
│   │   │    "version": "2.1",                                       │  │  │
│   │   │    "merkle_root": "0x7f83b165...",                         │  │  │
│   │   │    "event_count": 47,                                      │  │  │
│   │   │    "timestamp": "2025-01-28T10:35:00Z",                    │  │  │
│   │   │    "environment": "did:erynoa:env:domain:ev-charging-de"   │  │  │
│   │   │  }                                                          │  │  │
│   │   │                                                             │  │  │
│   │   └──────────────────────────┬──────────────────────────────────┘  │  │
│   │                              │                                      │  │
│   │                              ▼                                      │  │
│   │   ┌─────────────────────────────────────────────────────────────┐  │  │
│   │   │ IOTA Tangle                                                 │  │  │
│   │   │                                                             │  │  │
│   │   │   ○──○──○──○──●──○──○                                      │  │  │
│   │   │       ╲   ╱       ╲                                        │  │  │
│   │   │        ○──○──○──○──○                                       │  │  │
│   │   │                   ↑                                         │  │  │
│   │   │               Unser Anchor                                  │  │  │
│   │   │                                                             │  │  │
│   │   └─────────────────────────────────────────────────────────────┘  │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Ethereum Integration

```yaml
# Smart Contract für Anchoring
ethereum_anchor {
  contract:     "0x1234...ErynoaAnchor"
  network:      mainnet  # oder goerli für test

  # Funktionen
  functions: {
    anchor: {
      signature: "anchor(bytes32 merkleRoot, uint256 eventCount)"
      gas_limit: 50000
    }

    verify: {
      signature: "verify(bytes32 merkleRoot) returns (bool, uint256)"
      # Returns: (exists, timestamp)
    }
  }

  # Events
  events: {
    Anchored: "Anchored(bytes32 indexed merkleRoot, uint256 eventCount, uint256 timestamp)"
  }
}
```

---

## Cross-Chain Verification

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   CROSS-CHAIN VERIFICATION                                                 │
│                                                                             │
│   Verifier will prüfen: "Ist Event X finalized?"                           │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   1. Get Merkle Proof from NOA                                     │  │
│   │      → {root, path, index}                                          │  │
│   │                                                                     │  │
│   │   2. Verify Event is in Merkle Tree                                │  │
│   │      → hash(event) + path = root ✓                                 │  │
│   │                                                                     │  │
│   │   3. Check IOTA                                                     │  │
│   │      → Find message with root                                       │  │
│   │      → Check confirmations >= 10 ✓                                  │  │
│   │                                                                     │  │
│   │   4. (Optional) Check Ethereum                                      │  │
│   │      → Call verify(root)                                            │  │
│   │      → Returns (true, timestamp) ✓                                  │  │
│   │                                                                     │  │
│   │   Result: Event is cryptographically proven to be finalized.       │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Weiterführende Dokumente

- [bridges.md](./bridges.md) – Cross-Chain Kommunikation
- [routing.md](./routing.md) – Netzwerk-Routing
- [../chronik/finality.md](../chronik/finality.md) – Finality-Levels
