# ◉ ANKER – DACS (Decentralized Anchor Control System)

> **Schicht:** 0 – Fundament
> **Sphäre:** ERY (Identity-Modul)
> **Typ:** Multi-Chain DID-System

---

## Überblick

**DACS** ist das Identity-Modul innerhalb von ERY – ein selbst-ankerndes, Multi-Chain DID-System, das `did:erynoa` Identifikatoren über mehrere Blockchains hinweg verankert und verifiziert.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   "Ein dezentrales Netzwerk von Validatoren, das Identitäten               │
│    auf mehreren Blockchains gleichzeitig verankert – und sich              │
│    selbst durch genau diesen Mechanismus absichert."                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Das Problem: Single-Chain Identity

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ❌ Chain-Lock-in                                                          │
│      User auf Ethereum kann nicht mit User auf IOTA interagieren           │
│                                                                             │
│   ❌ Single Point of Failure                                                │
│      Chain down = Identität nicht verifizierbar                            │
│                                                                             │
│   ❌ Zentralisiertes Risiko                                                 │
│      Angreifer muss nur eine Chain kompromittieren                         │
│                                                                             │
│   ❌ Ökosystem-Fragmentierung                                               │
│      Jede Chain hat eigene DID-Methode, keine Interoperabilität            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Die DACS-Lösung

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ✅ Chain-Agnostisch                                                       │
│      Eine DID, verifizierbar von jeder Chain                               │
│                                                                             │
│   ✅ Maximale Resilience                                                    │
│      N-1 Chains können ausfallen, DID bleibt gültig                        │
│                                                                             │
│   ✅ Höchste Sicherheit                                                     │
│      Angreifer müsste mehrere Chains + DACS kompromittieren                │
│                                                                             │
│   ✅ Universelle Interoperabilität                                          │
│      Ethereum-Agent kann mit IOTA-Agent handeln                            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Architektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      DACS Node Network                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                   │
│   │  DACS Node  │◀──▶│  DACS Node  │◀──▶│  DACS Node  │                   │
│   │   (BFT)     │    │   (BFT)     │    │   (BFT)     │                   │
│   └──────┬──────┘    └──────┬──────┘    └──────┬──────┘                   │
│          │                  │                  │                           │
│          └──────────────────┼──────────────────┘                           │
│                             │                                              │
│              ┌──────────────┼──────────────┐                               │
│              ▼              ▼              ▼                               │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                    │
│   │    IOTA      │  │   Ethereum   │  │    Solana    │                    │
│   │  (Primary)   │  │    L2        │  │              │                    │
│   │              │  │ (Secondary)  │  │ (Secondary)  │                    │
│   │  Full DID    │  │  Hash Only   │  │  Hash Only   │                    │
│   │  Document    │  │  + Timestamp │  │  + Timestamp │                    │
│   └──────────────┘  └──────────────┘  └──────────────┘                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Komponenten

| Komponente         | Technologie    | Funktion                                       |
| ------------------ | -------------- | ---------------------------------------------- |
| **DACS Node**      | Rust, libp2p   | Validatoren im BFT-Netzwerk                    |
| **BFT Consensus**  | PBFT/HotStuff  | Finalisierung von DID-Operationen              |
| **Threshold Sigs** | BLS t-of-n     | Kollektive Signaturen (67% Threshold)          |
| **Chain Adapters** | Multi-Chain    | IOTA, Ethereum L2, Solana Anbindung            |
| **DID Registry**   | Self-Anchoring | did:erynoa:dacs-registry verankert sich selbst |

---

## Multi-Chain Anchoring Strategie

| Chain           | Rolle     | Speicherung       | Zweck                         |
| --------------- | --------- | ----------------- | ----------------------------- |
| **IOTA**        | Primary   | Full DID Document | Haupt-Identitätsspeicher      |
| **Ethereum L2** | Secondary | Hash + Timestamp  | Redundanz, Interoperabilität  |
| **Solana**      | Secondary | Hash + Timestamp  | Performance, Ecosystem-Access |

---

## Self-Anchoring

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   SELF-ANCHORING MECHANISMUS                                               │
│                                                                             │
│   Das DACS-System verankert seine eigene Registry-DID:                     │
│   did:erynoa:dacs-registry                                                 │
│                                                                             │
│   ┌───────────────────────────────────────────────────────────────────┐   │
│   │                                                                   │   │
│   │   1. Genesis: Initiale Validatoren signieren Registry-DID        │   │
│   │   2. Anchoring: Registry wird auf allen Chains verankert         │   │
│   │   3. Bootstrap: Neue Nodes verifizieren gegen Registry           │   │
│   │   4. Rotation: Validatoren-Updates werden selbst signiert        │   │
│   │                                                                   │   │
│   │   → Keine externe Abhängigkeit für Bootstrapping                 │   │
│   │                                                                   │   │
│   └───────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## DID-Operationen

### Create

```yaml
dacs create_did {
  namespace:  "vehicle"
  identifier: "vin-WVW123456789"

  keys: [{
    type:    Ed25519
    purpose: [authentication, assertionMethod]
  }]

  anchors: [IOTA, Ethereum, Solana]
}
```

### Resolve

```yaml
dacs resolve {
  did: "did:erynoa:vehicle:vin-WVW123456789"

  # Response enthält:
  # - DID Document
  # - Anchor-Status pro Chain
  # - Verification Methods
  # - Service Endpoints
}
```

### Update

```yaml
dacs update_did {
  did:       "did:erynoa:vehicle:vin-WVW123456789"
  operation: "add_service"

  service: {
    id:   "#agent-endpoint"
    type: "AgentService"
    endpoint: "https://agents.erynoa.io/..."
  }

  # Erfordert BFT-Konsens (67% der Validatoren)
}
```

### Deactivate

```yaml
dacs deactivate_did {
  did:    "did:erynoa:vehicle:vin-WVW123456789"
  reason: "Vehicle decommissioned"

  # DID wird auf allen Chains als deaktiviert markiert
  # Historische Referenzen bleiben verifizierbar
}
```

---

## Progressive Decentralization

| Phase       | Validatoren                 | Threshold |
| ----------- | --------------------------- | --------- |
| **Phase 1** | Team (5 Nodes)              | 3-of-5    |
| **Phase 2** | Community (21 Nodes)        | 14-of-21  |
| **Phase 3** | Permissionless (100+ Nodes) | 67%       |

---

## Integration mit ERY

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ERY (Semantic & Identity Lattice)                                        │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   🔐 DACS ◀────▶ 📚 Semantic ◀────▶ ⚖️ Karmic ◀────▶ 🌍 Discovery  │  │
│   │   Identity        Index           Engine         DHT+Geo            │  │
│   │                                                                     │  │
│   │   DIDs werden in allen ERY-Modulen referenziert:                   │  │
│   │   - Blueprints haben DID-Author                                     │  │
│   │   - Trust Vectors sind an DIDs gebunden                            │  │
│   │   - Discovery indiziert nach DID-Namespaces                        │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Kernmerkmale

| Merkmal                          | Beschreibung                                         |
| -------------------------------- | ---------------------------------------------------- |
| **Multi-Chain Anchoring**        | DIDs werden auf N Blockchains gleichzeitig verankert |
| **Sub-Identities (16 Typen)**    | Spezialisierte Identitäten für verschiedene Zwecke   |
| **Dezentrale Validatoren**       | DACS Nodes koordinieren via BFT-Konsens              |
| **Self-Anchoring**               | Das DACS-System verankert seine eigene Registry      |
| **Chain-Agnostisch**             | Funktioniert über IOTA, Ethereum, Solana, etc.       |
| **Progressive Decentralization** | Von Team → Community → Permissionless                |

---

## Weiterführende Dokumente

- [identity-first.md](./identity-first.md) – Das Paradigma
- [did-erynoa.md](./did-erynoa.md) – DID-Namespaces
- [sub-identities.md](./sub-identities.md) – Spezialisierte Identitäten
- [../nexus/multi-chain.md](../nexus/multi-chain.md) – Chain-Adapter Details
