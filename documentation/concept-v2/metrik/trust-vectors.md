# ◊ METRIK – Trust Vectors

> **Schicht:** 2 – Vertrauen
> **Sphäre:** ERY (Karmic-Modul)
> **Kernfrage:** _„Wie vertrauenswürdig?"_

---

## Konzept

**Trust Vectors** sind mehrdimensionale Vertrauenswerte, die an DIDs gebunden sind. Sie quantifizieren Vertrauen maschinenlesbar und ermöglichen automatisiertes Trust-Gating.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   In der Maschinenökonomie:                                                 │
│                                                                             │
│       🤖 Agent A                    🤖 Agent B                              │
│       did:erynoa:agent:a            did:erynoa:agent:b                      │
│           │                             │                                   │
│           │  "Kann ich dir trauen?"     │                                   │
│           └─────────────────────────────┘                                   │
│                        │                                                    │
│                        ▼                                                    │
│              ┌─────────────────┐                                            │
│              │  Trust Vector   │  ◀── Mathematisch berechenbar              │
│              │  [0.92, 0.87,   │      Maschinenlesbar                       │
│              │   0.78, 0.95]   │      Mehrdimensional                       │
│              └─────────────────┘      An DID gebunden                       │
│                                                                             │
│   Vertrauen ist kein Gefühl – es ist ein DATENTYP.                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Die vier Trust-Dimensionen

| Dimension       | Symbol | Misst               | Beispiel-Events                     |
| --------------- | ------ | ------------------- | ----------------------------------- |
| **Reliability** | 🎯     | Zuverlässigkeit     | Liefertreue, Uptime, Pünktlichkeit  |
| **Integrity**   | 🛡️     | Ehrlichkeit         | Keine Falschangaben, korrekte Daten |
| **Capability**  | ⚡     | Leistungsfähigkeit  | Technische Qualität, Kapazität      |
| **Reputation**  | 🌟     | Externe Wahrnehmung | Attestations, Endorsements          |

---

## Trust Vector Struktur

```yaml
trust_vector {
  subject: @identity("did:erynoa:agent:provider:swm-charging")

  dimensions: {
    reliability:  0.92   # Zuverlässigkeit
    integrity:    0.87   # Ehrlichkeit
    capability:   0.78   # Leistungsfähigkeit
    reputation:   0.95   # Externe Wahrnehmung
  }

  # Aggregierter Wert (gewichteter Durchschnitt)
  aggregate: 0.88

  # Kontext
  environment: "did:erynoa:env:domain:ev-charging-de"
  last_update: "2025-01-28T10:00:00Z"
  event_count: 1247
}
```

---

## Trust-Gating

Trust Vectors werden für automatisiertes Gating verwendet:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   TRUST-GATING FLOW                                                        │
│                                                                             │
│   Intent definiert Mindest-Trust:                                          │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │  trust_requirements: {                                              │  │
│   │    min_aggregate: 0.7                                               │  │
│   │    min_reliability: 0.8                                             │  │
│   │    min_integrity: 0.6                                               │  │
│   │  }                                                                  │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│   Kandidaten-Prüfung:                                                      │
│                                                                             │
│   Provider A: [0.92, 0.87, 0.78, 0.95] → aggregate: 0.88 → ✅ PASS         │
│   Provider B: [0.65, 0.90, 0.70, 0.80] → reliability: 0.65 → ❌ FAIL       │
│   Provider C: [0.85, 0.55, 0.80, 0.75] → integrity: 0.55 → ❌ FAIL         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Trust-Berechnung

Der Trust Vector wird kontinuierlich aus Events berechnet:

```
trust[dim] = Σ(event_weight × event_value × decay(age)) / normalization
```

| Faktor            | Beschreibung                                      |
| ----------------- | ------------------------------------------------- |
| **event_weight**  | Gewicht des Event-Typs (0.0 - 1.0)                |
| **event_value**   | Positiv (+) oder Negativ (-)                      |
| **decay(age)**    | Zeitlicher Verfall (ältere Events zählen weniger) |
| **normalization** | Normierung auf [0, 1]                             |

---

## Environment-Spezifität

Trust Vectors gelten pro Environment-Kontext:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   SAME DID, DIFFERENT CONTEXTS                                             │
│                                                                             │
│   did:erynoa:agent:provider:swm                                            │
│                                                                             │
│   ┌─────────────────────┐    ┌─────────────────────┐                      │
│   │ env:ev-charging-de  │    │ env:energy-trading  │                      │
│   │                     │    │                     │                      │
│   │ Trust: [0.92, 0.87, │    │ Trust: [0.78, 0.90, │                      │
│   │         0.78, 0.95] │    │         0.65, 0.82] │                      │
│   │                     │    │                     │                      │
│   │ Fokus: Ladesäulen   │    │ Fokus: Energiehandel│                      │
│   │ Events: 1247        │    │ Events: 89          │                      │
│   └─────────────────────┘    └─────────────────────┘                      │
│                                                                             │
│   Gleiche Identität kann unterschiedliche Trust-Profile haben.             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Weiterführende Dokumente

- [karma-engine.md](./karma-engine.md) – Tiers und Asymmetrie
- [attestations.md](./attestations.md) – Externe Bestätigungen
- [reputation.md](./reputation.md) – Vererbung und Events
