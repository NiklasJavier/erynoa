# ◈ SCHEMA – Blueprints

> **Schicht:** 1 – Wissen
> **Sphäre:** ERY (Semantic-Modul)
> **Typ:** Domain-spezifische Objektdefinitionen

---

## Konzept

**Blueprints** sind anwendungsspezifische Schablonen, die definieren, _wie_ ein Objekt beschaffen sein soll. Sie basieren auf Normativen Standards und bilden die Brücke zwischen abstrakten Normen und konkreten AMO-Instanzen.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   BLUEPRINT = SCHABLONE FÜR OBJEKTE                                        │
│                                                                             │
│   📐 DEFINITION (ERY)                     📦 INSTANZ (NOA)                  │
│   ─────────────────────                   ────────────────                  │
│                                                                             │
│   "Wie soll eine Ladesäule              "Diese konkrete Ladesäule          │
│    beschaffen sein?"                      in München, Betreiber X"          │
│                                                                             │
│   ┌─────────────────┐                    ┌─────────────────┐                │
│   │    Blueprint    │──────────────────▶│      AMO        │                │
│   │   (Schablone)   │   instantiiert    │   (Objekt)      │                │
│   └─────────────────┘                    └─────────────────┘                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Blueprint-Struktur

```yaml
blueprint {
  # Identität
  id: "did:erynoa:blueprint:ev-charging-station:v1.2"
  name: "EV Charging Station"
  version: "1.2.0"
  author: @identity("did:erynoa:org:erynoa-foundation")

  # Normative Wurzeln
  based_on: [
    @ref("did:erynoa:standard:iso:19112:2019"),      # Geo-Kontext
    @ref("did:erynoa:standard:eclass:27-27-90-01"), # Ladestationen
    @ref("did:erynoa:standard:ocpp:2.0.1")          # Protokoll
  ]

  # Attribute mit Validierung
  attributes: {
    power_output: {
      type:     number
      unit:     "kW"
      min:      3.7
      max:      350
      required: true
    }

    connector_type: {
      type:     enum
      values:   [Type2, CCS, CHAdeMO, Tesla]
      required: true
    }

    location: {
      type:      geo
      format:    geohash
      precision: 8
      required:  true
    }

    operator: {
      type:      did
      namespace: "org"
      required:  true
      trust_min: 0.7  # Trust-Gating auf Blueprint-Ebene
    }
  }

  # Validierungslogik (referenziert MoveScript)
  logic_guard: "0x1::ev_charging::validate"

  # Erlaubte AMO-Typen
  amo_types: [material, service]
}
```

---

## Beispiele für Domain Blueprints

| Blueprint               | Basiert auf         | Definiert                              |
| ----------------------- | ------------------- | -------------------------------------- |
| **EV-Charging-Station** | ISO, eCl@ss, OCPP   | Ladeleistung, Steckertyp, Standort     |
| **KYC-Credential**      | AML/KYC-Richtlinien | Identitätsattribute, Verifizierung     |
| **Energy-Certificate**  | Herkunftsnachweise  | Energiequelle, Zeitraum, Menge         |
| **Maintenance-Record**  | DIN, ISO            | Wartungstyp, Intervalle, Zertifizierer |
| **Fleet-Vehicle**       | ISO 55000, VIN      | Fahrzeugdaten, Wartungsstatus          |
| **Service-Contract**    | ETSI, ISO           | SLA-Parameter, Laufzeiten              |

---

## Blueprint-Hierarchie

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   BLUEPRINT HIERARCHIE                                                     │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   Normative Standards (Ebene 1)                                     │  │
│   │   ─────────────────────────────                                     │  │
│   │   ISO 19112 · eCl@ss · OCPP                                        │  │
│   │        │                                                            │  │
│   │        │ referenziert von                                           │  │
│   │        ▼                                                            │  │
│   │   Generic Blueprint (Ebene 2a)                                      │  │
│   │   ─────────────────────────────                                     │  │
│   │   ev-charging-station:v1 (abstrakt)                                │  │
│   │        │                                                            │  │
│   │        │ spezialisiert zu                                           │  │
│   │        ▼                                                            │  │
│   │   Domain Blueprint (Ebene 2b)                                       │  │
│   │   ────────────────────────────                                      │  │
│   │   ev-charging-station-de:v1 (Deutschland-spezifisch)               │  │
│   │   + Eichrecht-Anforderungen                                        │  │
│   │   + PTB-Zertifizierung                                             │  │
│   │        │                                                            │  │
│   │        │ instanziiert zu                                            │  │
│   │        ▼                                                            │  │
│   │   AMO (Ebene 3)                                                     │  │
│   │   ─────────────                                                     │  │
│   │   station-munich-001 (konkrete Säule)                              │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Drei Funktionen von Blueprints

```
┌─────────────────────────────────────────────────────────────────┐
│                    Blueprint-Funktionen                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   1️⃣ Ontologische Verankerung                                   │
│      "Was IST eine Ladesäule?"                                  │
│      → Definition, Parameter, Messverfahren                     │
│                                                                 │
│   2️⃣ Normative Referenz                                         │
│      "Entspricht dieses Objekt dem Standard?"                   │
│      → Compliance, Zertifizierung, Audit                        │
│                                                                 │
│   3️⃣ Vertrauensanker                                            │
│      "Wie vertrauenswürdig ist dieser Standard?"                │
│      → Trust propagiert zu allen abgeleiteten Objekten          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Evolution ohne Bruch

Blueprints unterstützen versionierte Evolution:

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│   Blueprint v1.0                        Blueprint v2.0          │
│   ══════════════                        ══════════════          │
│   (immutabel)         ───────▶          (immutabel)             │
│        │                                      │                 │
│        │              Migration               │                 │
│        ▼              Governance              ▼                 │
│   ┌─────────┐         ─────────▶        ┌─────────┐            │
│   │ AMOs    │                           │ AMOs    │             │
│   │ v1.x    │                           │ v2.x    │             │
│   └─────────┘                           └─────────┘             │
│                                                                 │
│   Stabil genug für Rechtssicherheit                             │
│   Flexibel genug für Weiterentwicklung                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Blueprint-Operationen

### Registrierung

```yaml
blueprint register {
  definition: { ... }  # Blueprint-Definition

  # Wird im Semantic Index gespeichert
  # Erhält DID: did:erynoa:blueprint:<name>:<version>
  # Author-Trust wird geprüft
}
```

### Instanziierung (via ECLVM)

```yaml
amo create {
  blueprint: @ref("did:erynoa:blueprint:ev-charging-station:v1.2")

  values: {
    power_output: 150
    connector_type: CCS
    location: "u281zq"
    operator: @identity("did:erynoa:org:swm")
  }

  # Logic Guard wird ausgeführt zur Validierung
}
```

---

## Weiterführende Dokumente

- [semantic-index.md](./semantic-index.md) – Speicherung und Suche
- [standards.md](./standards.md) – Normative Basis
- [../chronik/amo.md](../chronik/amo.md) – Instanziierte Objekte
