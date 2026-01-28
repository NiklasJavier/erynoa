# ⬡ NEXUS – Routing

> **Schicht:** 6 – Vernetzung
> **Sphäre:** ECHO + NOA (Network Layer)
> **Typ:** Peer-to-Peer Kommunikation

---

## Konzept

**Routing** beschreibt, wie Nachrichten und Daten zwischen Erynoa-Knoten fließen. Das Netzwerk ist dezentral und nutzt Content-Addressed Routing.

---

## Netzwerk-Topologie

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ERYNOA NETWORK TOPOLOGY                                                  │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │              ○─────○                     ○─────○                    │  │
│   │             ╱       ╲                   ╱       ╲                   │  │
│   │            ○    A    ○───────────────○    B    ○                   │  │
│   │             ╲       ╱                   ╲       ╱                   │  │
│   │              ○─────○                     ○─────○                    │  │
│   │                   ╲                     ╱                           │  │
│   │                    ╲                   ╱                            │  │
│   │                     ○───────────────○                               │  │
│   │                    ╱                   ╲                            │  │
│   │                   ╱                     ╲                           │  │
│   │              ○─────○                     ○─────○                    │  │
│   │             ╱       ╲                   ╱       ╲                   │  │
│   │            ○    C    ○               ○    D    ○                   │  │
│   │             ╲       ╱                   ╲       ╱                   │  │
│   │              ○─────○                     ○─────○                    │  │
│   │                                                                     │  │
│   │   Mesh-Netzwerk mit redundanten Verbindungen                        │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Node-Typen

| Typ            | Funktion                          | Ressourcen |
| -------------- | --------------------------------- | ---------- |
| **Full Node**  | Kompletter State, alle Events     | Hoch       |
| **Light Node** | Nur relevante Events, verifiziert | Mittel     |
| **Edge Node**  | Agent-Gateway, kein State         | Niedrig    |
| **Validator**  | Anchoring, Konsens                | Sehr hoch  |

---

## Message Routing

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   MESSAGE ROUTING                                                          │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   Agent A will Intent an Agent B senden:                           │  │
│   │                                                                     │  │
│   │   1. DID Resolution                                                 │  │
│   │      did:erynoa:agent:seeker:vehicle-123                           │  │
│   │      → DID Document                                                 │  │
│   │      → Service Endpoint: peer-id-xyz                               │  │
│   │                                                                     │  │
│   │   2. Peer Discovery                                                 │  │
│   │      DHT Lookup: peer-id-xyz                                       │  │
│   │      → Multiaddress: /ip4/1.2.3.4/tcp/4001/p2p/peer-id-xyz        │  │
│   │                                                                     │  │
│   │   3. Connection                                                     │  │
│   │      Direct: A → B (wenn erreichbar)                               │  │
│   │      Relayed: A → Relay → B (wenn NAT/Firewall)                    │  │
│   │                                                                     │  │
│   │   4. Message Delivery                                               │  │
│   │      Encrypted (TLS 1.3)                                           │  │
│   │      Signed (Agent's Key)                                          │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Gossip Protocol

Events werden via Gossip verteilt:

```yaml
gossip_config {
  # Mesh-Konfiguration
  mesh: {
    degree:        6     # Verbindungen pro Node
    degree_low:    4     # Minimum
    degree_high:   12    # Maximum
  }

  # Topics (Environment-basiert)
  topics: [
    "erynoa/global",
    "erynoa/env/ev-charging-de",
    "erynoa/env/energy-trading"
  ]

  # Message-Handling
  messages: {
    max_size:      65536  # Bytes
    ttl:           300    # Sekunden
    seen_cache:    1000   # Deduplizierung
  }

  # Scoring
  scoring: {
    peer_scoring:  true
    invalid_msg:   -100
    valid_msg:     +1
    time_in_mesh:  +0.1/sec
  }
}
```

---

## Content Addressing

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   CONTENT-ADDRESSED DATA                                                   │
│                                                                             │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │                                                                   │    │
│   │   Event/AMO/Blueprint                                             │    │
│   │        │                                                          │    │
│   │        │ Hash                                                     │    │
│   │        ▼                                                          │    │
│   │   CID: bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi │    │
│   │        │                                                          │    │
│   │        │ Distributed Storage                                      │    │
│   │        ▼                                                          │    │
│   │   ┌─────────┐   ┌─────────┐   ┌─────────┐                        │    │
│   │   │ Node A  │   │ Node B  │   │ Node C  │                        │    │
│   │   │ (copy)  │   │ (copy)  │   │ (copy)  │                        │    │
│   │   └─────────┘   └─────────┘   └─────────┘                        │    │
│   │                                                                   │    │
│   │   Jeder mit dem CID kann den Inhalt verifizieren und abrufen.    │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## NAT Traversal

```yaml
nat_traversal {
  # Methoden (in Prioritätsreihenfolge)
  methods: [
    {
      type: "direct"
      description: "Direkte Verbindung wenn möglich"
    },
    {
      type: "hole_punch"
      description: "UDP/TCP Hole Punching"
    },
    {
      type: "relay"
      description: "Über Relay-Server"
      relays: [
        "/dns4/relay.erynoa.io/tcp/4001/p2p/...",
        "/dns4/relay2.erynoa.io/tcp/4001/p2p/..."
      ]
    }
  ]
}
```

---

## Weiterführende Dokumente

- [multi-chain.md](./multi-chain.md) – Chain-Anchoring
- [bridges.md](./bridges.md) – Externe Verbindungen
