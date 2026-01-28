# ⬡ NEXUS – Bridges

> **Schicht:** 6 – Vernetzung
> **Sphäre:** NOA (Interoperability Layer)
> **Typ:** Cross-Chain Kommunikation

---

## Konzept

**Bridges** ermöglichen die Kommunikation zwischen Erynoa und externen Systemen – anderen Blockchains, Legacy-Systemen und APIs.

---

## Bridge-Typen

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   BRIDGE TYPES                                                             │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   1. CHAIN BRIDGES                                                  │  │
│   │      ══════════════                                                 │  │
│   │      Erynoa ←→ Andere Blockchains                                  │  │
│   │                                                                     │  │
│   │      ┌──────────┐      ┌──────────┐      ┌──────────┐              │  │
│   │      │  Erynoa  │ ◀──▶ │  Bridge  │ ◀──▶ │ Ethereum │              │  │
│   │      └──────────┘      └──────────┘      └──────────┘              │  │
│   │                                                                     │  │
│   │   2. ORACLE BRIDGES                                                 │  │
│   │      ═══════════════                                                │  │
│   │      Externe Daten → Erynoa                                        │  │
│   │                                                                     │  │
│   │      ┌──────────┐      ┌──────────┐      ┌──────────┐              │  │
│   │      │ Real World│ ──▶ │  Oracle  │ ──▶ │  Erynoa  │              │  │
│   │      │  (Preise) │      │  Bridge  │      │          │              │  │
│   │      └──────────┘      └──────────┘      └──────────┘              │  │
│   │                                                                     │  │
│   │   3. API BRIDGES                                                    │  │
│   │      ═════════════                                                  │  │
│   │      Legacy-Systeme ←→ Erynoa                                      │  │
│   │                                                                     │  │
│   │      ┌──────────┐      ┌──────────┐      ┌──────────┐              │  │
│   │      │   OCPP   │ ◀──▶ │   API    │ ◀──▶ │  Erynoa  │              │  │
│   │      │ Backend  │      │  Bridge  │      │          │              │  │
│   │      └──────────┘      └──────────┘      └──────────┘              │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Chain Bridge: Ethereum

```yaml
bridge "ethereum-bridge" {
  id:   "did:erynoa:bridge:ethereum-mainnet"
  type: chain

  # Ziel-Chain
  target: {
    network:   ethereum
    chain_id:  1
    rpc:       "https://eth-mainnet.g.alchemy.com/v2/..."
  }

  # Contracts
  contracts: {
    anchor:    "0x1234...ErynoaAnchor"
    registry:  "0x5678...ErynoaDIDRegistry"
    escrow:    "0x9abc...ErynoaEscrow"
  }

  # Operationen
  operations: {
    anchor_events:  enabled
    resolve_did:    enabled
    token_bridge:   enabled  # USDC, etc.
  }

  # Trust
  trust: {
    min_confirmations: 6
    validator_set:     @ref("did:erynoa:bridge:validators:eth")
  }
}
```

---

## Oracle Bridge: Preise

```yaml
bridge "price-oracle" {
  id:   "did:erynoa:bridge:oracle:prices"
  type: oracle

  # Datenquellen
  sources: [
    {
      name:     "Chainlink"
      type:     "decentralized"
      endpoint: "0xabc...ChainlinkETHUSD"
      weight:   0.4
    },
    {
      name:     "CoinGecko"
      type:     "centralized"
      endpoint: "https://api.coingecko.com/..."
      weight:   0.3
    },
    {
      name:     "Internal"
      type:     "erynoa"
      endpoint: @ref("did:erynoa:agent:oracle:prices")
      weight:   0.3
    }
  ]

  # Aggregation
  aggregation: {
    method:          "median"
    min_sources:     2
    max_deviation:   0.05  # 5%
  }

  # Update-Frequenz
  frequency: 60  # Sekunden

  # Feeds
  feeds: [
    { pair: "ETH/USD",  decimals: 8 },
    { pair: "EUR/USD",  decimals: 8 },
    { pair: "IOTA/EUR", decimals: 8 }
  ]
}
```

---

## API Bridge: OCPP

```yaml
bridge "ocpp-bridge" {
  id:   "did:erynoa:bridge:api:ocpp"
  type: api

  # Protokoll
  protocol: {
    type:    "OCPP"
    version: "2.0.1"
    transport: "websocket"
  }

  # Mapping: OCPP → Erynoa
  mappings: {
    # OCPP Message → Erynoa Event
    TransactionEvent: {
      erynoa_type: "charging_session_update"
      fields: {
        transactionId:  "amo.id"
        meterValue:     "attributes.energy_delivered"
        timestamp:      "event.timestamp"
      }
    }

    StatusNotification: {
      erynoa_type: "station_status_update"
      fields: {
        connectorId:    "amo.connector_id"
        connectorStatus: "status"
      }
    }
  }

  # Trust
  trust: {
    requires_auth:  true
    auth_method:    "certificate"
    trust_min:      0.7
  }
}
```

---

## Bridge Security

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   BRIDGE SECURITY MODEL                                                    │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   Layer 1: VALIDATORS                                               │  │
│   │   ═════════════════════                                             │  │
│   │   Multi-Sig Validator Set                                          │  │
│   │   Threshold: 3 of 5                                                 │  │
│   │                                                                     │  │
│   │   Layer 2: FRAUD PROOFS                                             │  │
│   │   ═══════════════════════                                           │  │
│   │   Challenge-Period: 24h                                             │  │
│   │   Slashing bei Betrug                                              │  │
│   │                                                                     │  │
│   │   Layer 3: RATE LIMITING                                            │  │
│   │   ════════════════════════                                          │  │
│   │   Max Value/Hour: 100,000 EUR                                      │  │
│   │   Pause bei Anomalie                                               │  │
│   │                                                                     │  │
│   │   Layer 4: CIRCUIT BREAKER                                          │  │
│   │   ══════════════════════════                                        │  │
│   │   Emergency Stop möglich                                           │  │
│   │   Governance-gesteuert                                             │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Weiterführende Dokumente

- [multi-chain.md](./multi-chain.md) – Chain-Übersicht
- [routing.md](./routing.md) – Netzwerk-Routing
