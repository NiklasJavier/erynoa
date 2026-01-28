# ◐ IMPULS – Agent-Modell

> **Schicht:** 4 – Handlung
> **Sphäre:** ECHO (Swarm-Modul)
> **Kernfrage:** _„Wer handelt?"_

---

## Konzept

**Agenten** sind autonome digitale Einheiten, die Interessen vertreten und Transaktionen durchführen. Sie sind die "Hände" im Erynoa-Ökosystem.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   AGENT = AUTONOME HANDLUNGSEINHEIT                                        │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │      🤖                                                             │  │
│   │      AGENT                                                          │  │
│   │      did:erynoa:agent:*                                            │  │
│   │                                                                     │  │
│   │      ┌────────────────────────────────────────────────────────┐    │  │
│   │      │                                                        │    │  │
│   │      │  • Eigene DID (Identität)                              │    │  │
│   │      │  • Eigener Trust Vector (Reputation)                   │    │  │
│   │      │  • Eigene Credentials (Berechtigungen)                 │    │  │
│   │      │  • Eigener Wallet (Vermögen)                           │    │  │
│   │      │  • Eigene Policies (Entscheidungsregeln)               │    │  │
│   │      │  • Eigene Intents (Ziele)                              │    │  │
│   │      │                                                        │    │  │
│   │      └────────────────────────────────────────────────────────┘    │  │
│   │                                                                     │  │
│   │   Agent handelt autonom innerhalb seiner Policy-Grenzen.           │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Agent-Typen

| Typ           | Namespace           | Beschreibung                 | Beispiel                   |
| ------------- | ------------------- | ---------------------------- | -------------------------- |
| **Seeker**    | `agent:seeker:*`    | Sucht Ressourcen/Dienste     | Fahrzeug sucht Ladestation |
| **Provider**  | `agent:provider:*`  | Bietet Ressourcen/Dienste    | Ladesäulen-Betreiber       |
| **Broker**    | `agent:broker:*`    | Vermittelt zwischen Parteien | Roaming-Plattform          |
| **Oracle**    | `agent:oracle:*`    | Liefert externe Daten        | Wetter-Service, Preisfeed  |
| **Validator** | `agent:validator:*` | Prüft und bestätigt          | Eichamt, Zertifizierer     |

---

## Agent-Definition

```yaml
agent "vehicle-charging-agent" {
  id:    "did:erynoa:agent:seeker:vehicle-123"
  type:  seeker

  # Zugehörigkeit
  owner: @identity("did:erynoa:person:alice")

  # Credentials
  credentials: [
    @ref("did:erynoa:credential:payment-method:cc-visa"),
    @ref("did:erynoa:credential:roaming:plugsurfing")
  ]

  # Wallet
  wallet: {
    balance:    50.00  # EUR
    currencies: [EUR, USDC]
    limit:      100.00 # EUR pro Tag
  }

  # Policy (Entscheidungsregeln)
  policy: @ref("did:erynoa:policy:vehicle-charging-default")

  # Environments
  active_environments: [
    @ref("did:erynoa:env:domain:ev-charging-de"),
    @ref("did:erynoa:env:domain:ev-charging-at")
  ]
}
```

---

## Agent-Lifecycle

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   AGENT LIFECYCLE                                                          │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   1. CREATION                                                       │  │
│   │      ══════════                                                     │  │
│   │      Owner erstellt Agent mit DID                                   │  │
│   │      Initial Trust = Owner Trust × 0.5                              │  │
│   │           │                                                         │  │
│   │           ▼                                                         │  │
│   │   2. CONFIGURATION                                                  │  │
│   │      ═════════════                                                  │  │
│   │      Policy zuweisen, Wallet auffüllen                              │  │
│   │      Credentials delegieren                                         │  │
│   │           │                                                         │  │
│   │           ▼                                                         │  │
│   │   3. ACTIVATION                                                     │  │
│   │      ══════════                                                     │  │
│   │      Agent wird in Environments registriert                         │  │
│   │      Kann Intents erstellen und verhandeln                         │  │
│   │           │                                                         │  │
│   │           ▼                                                         │  │
│   │   4. OPERATION                                                      │  │
│   │      ═════════                                                      │  │
│   │      Autonomer Betrieb gemäß Policy                                │  │
│   │      Trust baut sich auf/ab                                        │  │
│   │           │                                                         │  │
│   │           ▼                                                         │  │
│   │   5. TERMINATION                                                    │  │
│   │      ═══════════                                                    │  │
│   │      Owner kann Agent deaktivieren                                  │  │
│   │      Wallet wird zurückgeführt                                      │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Agent-Hierarchie

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   AGENT HIERARCHIE                                                         │
│                                                                             │
│   did:erynoa:org:stadtwerke-munich                                         │
│   │                                                                        │
│   ├── did:erynoa:agent:provider:swm-charging                               │
│   │   (vertritt alle Ladesäulen)                                           │
│   │                                                                        │
│   ├── did:erynoa:agent:broker:swm-roaming                                  │
│   │   (vermittelt Roaming-Anfragen)                                        │
│   │                                                                        │
│   └── did:erynoa:agent:oracle:swm-pricing                                  │
│       (liefert dynamische Preise)                                          │
│                                                                             │
│   Agent-Aktionen propagieren Trust zum Owner (gedämpft).                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Agent Operations

```yaml
# Intent erstellen
agent create_intent {
  agent:  @identity("did:erynoa:agent:seeker:vehicle-123")

  intent: {
    type:        charge_vehicle
    constraints: { power_min: 50, connector: CCS }
    budget:      { max: 30.00, currency: EUR }
    deadline:    "2025-01-28T12:00:00Z"
  }
}

# Angebot annehmen
agent accept_offer {
  agent:  @identity("did:erynoa:agent:seeker:vehicle-123")
  offer:  @ref("did:erynoa:offer:charging-001")

  # Policy wird automatisch geprüft
}
```

---

## Weiterführende Dokumente

- [intent.md](./intent.md) – Absichtserklärungen
- [policy.md](./policy.md) – Entscheidungsregeln
- [negotiation.md](./negotiation.md) – Verhandlung
- [wallet.md](./wallet.md) – Vermögensverwaltung
- [eclvm.md](./eclvm.md) – Runtime
