# ◈ SCHEMA – Ontologie

> **Schicht:** 1 – Wissen
> **Sphäre:** ERY (Semantic-Modul)
> **Typ:** Begriffsrelationen und Taxonomien

---

## Konzept

Die **Ontologie** definiert die Beziehungen zwischen Begriffen im Erynoa-Ökosystem. Sie ermöglicht semantisches Reasoning und intelligente Suche.

---

## Relationstypen

| Relation          | Bedeutung            | Beispiel                          |
| ----------------- | -------------------- | --------------------------------- |
| **is_a**          | Klassenzugehörigkeit | CCS `is_a` Connector              |
| **has_part**      | Komposition          | Station `has_part` Connector      |
| **uses**          | Nutzungsbeziehung    | Station `uses` OCPP               |
| **complies_with** | Normerfüllung        | Station `complies_with` ISO_15118 |
| **located_in**    | Räumliche Einordnung | Station `located_in` Munich       |
| **owned_by**      | Besitzverhältnis     | Station `owned_by` Operator       |
| **provides**      | Dienstleistung       | Station `provides` Charging       |
| **requires**      | Abhängigkeit         | Charging `requires` Energy        |

---

## Taxonomie-Beispiel

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   CHARGING INFRASTRUCTURE TAXONOMIE                                        │
│                                                                             │
│   Asset                                                                     │
│   ├── Infrastructure                                                        │
│   │   ├── ChargingStation                                                  │
│   │   │   ├── ACStation                                                    │
│   │   │   │   ├── Type2Station                                             │
│   │   │   │   └── SchukoStation                                            │
│   │   │   └── DCStation                                                    │
│   │   │       ├── CCSStation                                               │
│   │   │       ├── CHAdeMOStation                                           │
│   │   │       └── TeslaStation                                             │
│   │   └── GridConnection                                                   │
│   │       ├── LowVoltage                                                   │
│   │       └── MediumVoltage                                                │
│   └── Vehicle                                                               │
│       └── ElectricVehicle                                                  │
│           ├── BEV                                                          │
│           └── PHEV                                                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Ontologie in ECL

```yaml
ontology "ev-charging" {
  version: "1.0"

  classes: {
    ChargingStation: {
      parent: Infrastructure
      properties: [power_output, connector_type, location]
    }

    DCStation: {
      parent: ChargingStation
      constraints: {
        power_output: { min: 20 }  # kW
      }
    }

    CCSStation: {
      parent: DCStation
      fixed: {
        connector_type: CCS
      }
    }
  }

  relations: {
    provides: {
      domain: ChargingStation
      range:  ChargingService
    }

    located_in: {
      domain: Asset
      range:  Location
    }
  }
}
```

---

## Semantic Reasoning

Die Ontologie ermöglicht intelligente Schlussfolgerungen:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   REASONING BEISPIEL                                                       │
│                                                                             │
│   Gegeben:                                                                 │
│   - Station-001 is_a CCSStation                                            │
│   - CCSStation is_a DCStation                                              │
│   - DCStation is_a ChargingStation                                         │
│   - DCStation requires HighPowerConnection                                 │
│                                                                             │
│   Abgeleitet:                                                              │
│   - Station-001 is_a ChargingStation ✓                                     │
│   - Station-001 requires HighPowerConnection ✓                             │
│   - Station-001 has connector_type CCS ✓                                   │
│                                                                             │
│   → Agent kann nach "ChargingStation" suchen und findet Station-001        │
│   → System weiß, dass HighPowerConnection nötig ist                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Integration mit Suche

```yaml
semantic_query {
  # Query: "Schnelllader in der Nähe"

  # Ontologie expandiert automatisch:
  # - "Schnelllader" → DCStation (power > 50kW)
  # - Inkludiert: CCSStation, CHAdeMOStation, TeslaStation

  expanded_types: [DCStation, CCSStation, CHAdeMOStation, TeslaStation]

  # Ergebnis enthält alle DC-Stationen,
  # auch wenn Query nur "Schnelllader" enthielt
}
```

---

## Weiterführende Dokumente

- [semantic-index.md](./semantic-index.md) – Vektor-Suche
- [blueprints.md](./blueprints.md) – Objektdefinitionen
- [standards.md](./standards.md) – Normative Basis
