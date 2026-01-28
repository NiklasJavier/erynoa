# ◉ ANKER – did:erynoa Spezifikation

> **Schicht:** 0 – Fundament
> **Sphäre:** ERY (DACS-Modul)
> **Bezug:** W3C DID Core Specification

---

## DID-Syntax

```
did:erynoa:<namespace>:<unique-identifier>
```

### Beispiele

| DID                                   | Bedeutung                 |
| ------------------------------------- | ------------------------- |
| `did:erynoa:vehicle:vin-WVW123456789` | Fahrzeug mit VIN          |
| `did:erynoa:charger:loc-munich-001`   | Ladesäule in München      |
| `did:erynoa:org:erynoa-gmbh`          | Organisation              |
| `did:erynoa:agent:seeker-abc123`      | ECHO Seeker-Agent         |
| `did:erynoa:dacs-registry`            | Self-Anchoring System-DID |

---

## Universal DID Namespaces

Das Identity-First Paradigma definiert, dass **jede Entität** im Ökosystem eine DID besitzt:

### Agenten & Akteure

| Namespace            | Beschreibung                  | Beispiel-DID                               |
| -------------------- | ----------------------------- | ------------------------------------------ |
| **agent:seeker**     | Suchende Agenten              | `did:erynoa:agent:seeker:fleet-agent-001`  |
| **agent:provider**   | Anbietende Agenten            | `did:erynoa:agent:provider:swm-charging`   |
| **agent:autonomous** | Autonome AI-Agenten           | `did:erynoa:agent:autonomous:optimizer-1`  |
| **org**              | Organisationen                | `did:erynoa:org:stadtwerke-munich`         |
| **user**             | Natürliche Personen           | `did:erynoa:user:max-mueller-abc123`       |
| **vehicle**          | Fahrzeuge (Real World Assets) | `did:erynoa:vehicle:vin-WVWZZZ3CZWE123456` |

### Objekte & Assets

| Namespace          | Beschreibung                  | Beispiel-DID                                    |
| ------------------ | ----------------------------- | ----------------------------------------------- |
| **amo:material**   | Physische Objekte (AMO)       | `did:erynoa:amo:material:station-munich-001`    |
| **amo:credential** | Soulbound Credentials (AMO)   | `did:erynoa:amo:credential:kyc-verified`        |
| **amo:service**    | Dienstleistungen (AMO)        | `did:erynoa:amo:service:charging-session-xyz`   |
| **blueprint**      | Objekt-Schemata               | `did:erynoa:blueprint:ev-charging-station:v1.2` |
| **standard**       | Normen (ISO, OCPP, Eichrecht) | `did:erynoa:standard:iso:15118:2`               |

### Umgebungen & Governance

| Namespace          | Beschreibung             | Beispiel-DID                                       |
| ------------------ | ------------------------ | -------------------------------------------------- |
| **env:domain**     | Domänen-Umgebungen       | `did:erynoa:env:domain:ev-charging-de`             |
| **env:network**    | Netzwerk-Umgebungen      | `did:erynoa:env:network:hubject-intercharge`       |
| **env:regulatory** | Regulierte Umgebungen    | `did:erynoa:env:regulatory:eichrecht-de`           |
| **legislative:**   | Regelwerk einer Umgebung | `did:erynoa:legislative:env:domain:ev-charging-de` |
| **executive:**     | Durchsetzungsorgan       | `did:erynoa:executive:env:domain:ev-charging-de`   |
| **proposal**       | Governance-Vorschläge    | `did:erynoa:proposal:gp-upgrade-v3`                |
| **dao**            | DAO-Organisationen       | `did:erynoa:dao:ev-charging-governance`            |

### Credentials & Trust

| Namespace       | Beschreibung           | Beispiel-DID                                   |
| --------------- | ---------------------- | ---------------------------------------------- |
| **vc**          | Verifiable Credentials | `did:erynoa:vc:license:fleet-operator-fleetco` |
| **attestation** | Trust-Attestationen    | `did:erynoa:attestation:rating-2025-001`       |

### Intents & Policies

| Namespace  | Beschreibung     | Beispiel-DID                          |
| ---------- | ---------------- | ------------------------------------- |
| **intent** | Agenten-Intents  | `did:erynoa:intent:i-20250128-abc123` |
| **policy** | Agenten-Policies | `did:erynoa:policy:swm-charging-v2`   |

### Wallets & Wirtschaft

| Namespace  | Beschreibung   | Beispiel-DID                     |
| ---------- | -------------- | -------------------------------- |
| **wallet** | Krypto-Wallets | `did:erynoa:wallet:fleetco-main` |

### Infrastruktur

| Namespace     | Beschreibung         | Beispiel-DID                        |
| ------------- | -------------------- | ----------------------------------- |
| **node:dacs** | DACS-Netzwerk-Knoten | `did:erynoa:node:dacs:eu-central-1` |
| **bridge**    | Cross-Chain Bridges  | `did:erynoa:bridge:iota-ethereum`   |

### Sub-Identities

| Namespace          | Beschreibung        | Beispiel-DID                                    |
| ------------------ | ------------------- | ----------------------------------------------- |
| **sub:avatar**     | Umgebungs-Avatar    | `did:erynoa:sub:avatar:a1b2c3d4:hubject`        |
| **sub:delegate**   | Delegierte Befugnis | `did:erynoa:sub:delegate:e5f6g7h8:negotiator`   |
| **sub:ownership**  | Besitz-Anker        | `did:erynoa:sub:ownership:i9j0k1l2:vehicle-123` |
| **sub:session**    | Session-Identity    | `did:erynoa:sub:session:m3n4o5p6:charge-001`    |
| **sub:bundle**     | Asset-Bündel        | `did:erynoa:sub:bundle:q7r8s9t0:fleet-north`    |
| **sub:proxy**      | Temporärer Proxy    | `did:erynoa:sub:proxy:u1v2w3x4:emergency`       |
| **sub:capability** | Capability-Träger   | `did:erynoa:sub:capability:y5z6a7b8:payment`    |
| **sub:persona**    | Kontext-Rolle       | `did:erynoa:sub:persona:c9d0e1f2:business`      |
| **sub:guardian**   | Treuhänder          | `did:erynoa:sub:guardian:g3h4i5j6:iot-custody`  |
| **sub:custodian**  | Asset-Verwahrer     | `did:erynoa:sub:custodian:k7l8m9n0:cold-store`  |

### Testing

| Namespace | Beschreibung   | Beispiel-DID                                    |
| --------- | -------------- | ----------------------------------------------- |
| **test**  | Test-Entitäten | `did:erynoa:test:suite:ev-charging-integration` |
| **mock**  | Mock-Entitäten | `did:erynoa:mock:agent:test-seeker`             |

---

## DID Document Struktur

```json
{
  "@context": ["https://www.w3.org/ns/did/v1", "https://erynoa.io/ns/did/v1"],
  "id": "did:erynoa:org:erynoa-gmbh",
  "controller": "did:erynoa:org:erynoa-gmbh",

  "verificationMethod": [
    {
      "id": "did:erynoa:org:erynoa-gmbh#key-1",
      "type": "Ed25519VerificationKey2020",
      "controller": "did:erynoa:org:erynoa-gmbh",
      "publicKeyMultibase": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
    }
  ],

  "authentication": ["did:erynoa:org:erynoa-gmbh#key-1"],
  "assertionMethod": ["did:erynoa:org:erynoa-gmbh#key-1"],

  "service": [
    {
      "id": "did:erynoa:org:erynoa-gmbh#agent-endpoint",
      "type": "AgentService",
      "serviceEndpoint": "https://agents.erynoa.io/org/erynoa-gmbh"
    }
  ],

  "erynoa": {
    "subIdentities": [
      "did:erynoa:sub:trading:erynoa-gmbh:main",
      "did:erynoa:sub:voting:erynoa-gmbh:governance"
    ],
    "anchors": {
      "iota": { "txHash": "0x...", "timestamp": "2025-01-28T10:00:00Z" },
      "ethereum": { "txHash": "0x...", "timestamp": "2025-01-28T10:00:05Z" }
    },
    "karmaTier": "veteran",
    "trustVector": [0.92, 0.87, 0.78, 0.95]
  }
}
```

---

## Visuelle Übersicht

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   IDENTITY-FIRST ARCHITECTURE                                │
│   ═════════════════════════════                              │
│                                                              │
│   "Alles existiert, weil es identifizierbar ist"            │
│                                                              │
│   ┌────────────────────────────────────────────────────┐    │
│   │                                                    │    │
│   │   🤖 Agents      → did:erynoa:agent:*             │    │
│   │   📦 Objects     → did:erynoa:amo:*               │    │
│   │   📋 Blueprints  → did:erynoa:blueprint:*         │    │
│   │   🌍 Environments→ did:erynoa:env:*               │    │
│   │   📜 Standards   → did:erynoa:standard:*          │    │
│   │   🎫 Credentials → did:erynoa:vc:*                │    │
│   │   💳 Wallets     → did:erynoa:wallet:*            │    │
│   │   📄 Intents     → did:erynoa:intent:*            │    │
│   │   📜 Policies    → did:erynoa:policy:*            │    │
│   │   🗳️ Proposals   → did:erynoa:proposal:*          │    │
│   │   🔗 Sub-IDs     → did:erynoa:sub:*               │    │
│   │                                                    │    │
│   └────────────────────────────────────────────────────┘    │
│                                                              │
│   Jede Entität hat eigene DID → Universal referenzierbar    │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## Weiterführende Dokumente

- [identity-first.md](./identity-first.md) – Das Identity-First Paradigma
- [sub-identities.md](./sub-identities.md) – Hierarchische Identitäten
- [dacs.md](./dacs.md) – Multi-Chain Anchoring
