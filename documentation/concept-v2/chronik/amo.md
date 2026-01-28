# ◆ CHRONIK – AMO (Atomic Managed Objects)

> **Schicht:** 5 – Beweis
> **Sphäre:** NOA (Object Layer)
> **Typ:** Universelle Objektrepräsentation

---

## Konzept

**AMO** (Atomic Managed Object) ist die universelle Repräsentation von Assets, Services und Credentials in Erynoa. Jedes handelbare oder referenzierbare "Ding" ist ein AMO.

---

## AMO-Axiome

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   AMO AXIOME                                                               │
│                                                                             │
│   1️⃣ IDENTITÄT                                                              │
│      Jedes AMO hat eine eindeutige DID.                                    │
│      did:erynoa:amo:<type>:<id>                                            │
│                                                                             │
│   2️⃣ BLUEPRINT                                                              │
│      Jedes AMO referenziert ein Blueprint.                                 │
│      Das Blueprint definiert Struktur und Constraints.                     │
│                                                                             │
│   3️⃣ OWNER                                                                  │
│      Jedes AMO hat einen Owner (DID).                                      │
│      Owner kann delegieren, aber nie aufgeben.                             │
│                                                                             │
│   4️⃣ STATE                                                                  │
│      Jedes AMO hat einen definierten Status.                               │
│      Statusänderungen sind Events auf NOA.                                 │
│                                                                             │
│   5️⃣ IMMUTABILITY                                                           │
│      AMO-Kernattribute sind unveränderlich.                                │
│      Änderungen = neue Version (nicht Überschreiben).                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## AMO-Typen

| Typ            | Namespace          | Beispiel                        |
| -------------- | ------------------ | ------------------------------- |
| **Material**   | `amo:material:*`   | Ladesäule, Fahrzeug, Sensor     |
| **Service**    | `amo:service:*`    | Ladevorgang, Wartung, Transport |
| **Credential** | `amo:credential:*` | Zertifikat, Lizenz, KYC         |
| **Data**       | `amo:data:*`       | Messwert, Report, Log           |
| **Contract**   | `amo:contract:*`   | Vertrag, SLA, Agreement         |

---

## AMO-Struktur

```yaml
amo "charging-station-munich-001" {
  # Identität
  id:   "did:erynoa:amo:material:station-munich-001"
  type: material

  # Blueprint-Referenz
  blueprint: @ref("did:erynoa:blueprint:ev-charging-station-de:v1.2")

  # Owner
  owner: @identity("did:erynoa:org:stadtwerke-munich")

  # Attribute (gemäß Blueprint)
  attributes: {
    power_output:    150        # kW
    connector_type:  CCS
    location:        "u281zq5"  # Geohash
    serial_number:   "SWM-2024-00123"
    commissioned:    "2024-06-15"
  }

  # Status
  status: active

  # Credentials
  credentials: [
    @ref("did:erynoa:credential:eichrecht:station-munich-001"),
    @ref("did:erynoa:credential:ocpp-certified:station-munich-001")
  ]

  # Metadata
  created_at:  "2024-06-15T10:00:00Z"
  updated_at:  "2025-01-15T08:00:00Z"
  version:     3
}
```

---

## AMO-Lifecycle

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   AMO LIFECYCLE                                                            │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │       ┌─────────┐                                                   │  │
│   │       │ PENDING │ ◀── Erstellt, aber noch nicht aktiviert          │  │
│   │       └────┬────┘                                                   │  │
│   │            │                                                        │  │
│   │            │ activate()                                             │  │
│   │            │ (requires: credentials verified)                       │  │
│   │            ▼                                                        │  │
│   │       ┌─────────┐                                                   │  │
│   │       │ ACTIVE  │ ◀── Vollständig operativ                         │  │
│   │       └────┬────┘                                                   │  │
│   │            │                                                        │  │
│   │    ┌───────┼───────┐                                               │  │
│   │    │       │       │                                               │  │
│   │    │       │       │ suspend()                                     │  │
│   │    │       │       ▼                                               │  │
│   │    │       │   ┌──────────┐                                        │  │
│   │    │       │   │SUSPENDED │ ◀── Temporär inaktiv                   │  │
│   │    │       │   └────┬─────┘                                        │  │
│   │    │       │        │                                               │  │
│   │    │       │        │ resume()                                      │  │
│   │    │       │        │                                               │  │
│   │    │       │        ▼                                               │  │
│   │    │       │   ┌─────────┐                                         │  │
│   │    │       └───│ ACTIVE  │                                         │  │
│   │    │           └─────────┘                                         │  │
│   │    │                                                                │  │
│   │    │ decommission()                                                 │  │
│   │    ▼                                                                │  │
│   │  ┌──────────────┐                                                   │  │
│   │  │DECOMMISSIONED│ ◀── Permanent inaktiv, historisch               │  │
│   │  └──────────────┘                                                   │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## AMO-Transitionen

```yaml
# AMO aktivieren
amo transition {
  amo:        @ref("did:erynoa:amo:material:station-munich-001")
  action:     activate

  caller:     @identity("did:erynoa:org:stadtwerke-munich")

  # Preconditions (von Logic Guard geprüft)
  preconditions: {
    status:      pending
    credentials: [eichrecht, ocpp-certified]
  }

  # Wird Event auf NOA
}
```

---

## Service-AMO (Temporär)

```yaml
amo "charging-session" {
  id:   "did:erynoa:amo:service:session-2025-001"
  type: service

  # Blueprint
  blueprint: @ref("did:erynoa:blueprint:charging-session:v1")

  # Parteien
  provider: @identity("did:erynoa:agent:provider:swm")
  consumer: @identity("did:erynoa:agent:seeker:vehicle-123")

  # Referenziertes Asset
  asset: @ref("did:erynoa:amo:material:station-munich-001")

  # Service-spezifische Attribute
  attributes: {
    started_at:       "2025-01-28T10:20:00Z"
    ended_at:         "2025-01-28T10:48:00Z"
    energy_delivered: 45  # kWh
    max_power:        142 # kW (peak)
    total_cost:       18.90
  }

  # Status
  status: completed

  # Agreement-Referenz
  agreement: @ref("did:erynoa:agreement:charging-2025-001")
}
```

---

## AMO-Queries

```yaml
# Alle aktiven Ladesäulen eines Betreibers
query amo {
  filter: {
    type:     material
    blueprint: @ref("did:erynoa:blueprint:ev-charging-station-de:*")
    owner:    @identity("did:erynoa:org:stadtwerke-munich")
    status:   active
  }
}

# Alle Services eines Consumers
query amo {
  filter: {
    type:     service
    consumer: @identity("did:erynoa:agent:seeker:vehicle-123")
    timerange: {
      from: "2025-01-01"
      to:   "2025-01-31"
    }
  }
}
```

---

## Weiterführende Dokumente

- [noa-ledger.md](./noa-ledger.md) – Event-Speicherung
- [logic-guards.md](./logic-guards.md) – Transition-Validierung
- [../schema/blueprints.md](../schema/blueprints.md) – Blueprint-Definition
