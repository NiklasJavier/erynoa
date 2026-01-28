# ▣ SPHÄRE – Discovery

> **Schicht:** 3 – Räume
> **Sphäre:** ERY (Semantic-Modul)
> **Typ:** Objekt-Suche und Matching

---

## Konzept

**Discovery** ist der Mechanismus, mit dem Agenten passende Objekte, Dienste und Gegenparteien finden. Es kombiniert semantische Suche mit Trust-Gating und Environment-Constraints.

---

## Discovery Pipeline

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   DISCOVERY PIPELINE                                                       │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   INTENT                                                            │  │
│   │   ══════                                                            │  │
│   │   "Ich suche eine 150kW Ladesäule in München"                      │  │
│   │        │                                                            │  │
│   │        ▼                                                            │  │
│   │   1. SEMANTIC SEARCH                                                │  │
│   │      ════════════════                                               │  │
│   │      Qdrant-Vektorsuche                                            │  │
│   │      → 500 Kandidaten                                               │  │
│   │        │                                                            │  │
│   │        ▼                                                            │  │
│   │   2. ENVIRONMENT FILTER                                             │  │
│   │      ═══════════════════                                            │  │
│   │      Nur Objekte im passenden Environment                          │  │
│   │      → 320 Kandidaten                                               │  │
│   │        │                                                            │  │
│   │        ▼                                                            │  │
│   │   3. TRUST GATING                                                   │  │
│   │      ═════════════                                                  │  │
│   │      Min-Trust aus Intent prüfen                                   │  │
│   │      → 180 Kandidaten                                               │  │
│   │        │                                                            │  │
│   │        ▼                                                            │  │
│   │   4. CONSTRAINT MATCHING                                            │  │
│   │      ═══════════════════                                            │  │
│   │      Blueprint-Attribute prüfen (power ≥ 150kW)                    │  │
│   │      → 45 Kandidaten                                                │  │
│   │        │                                                            │  │
│   │        ▼                                                            │  │
│   │   5. RANKING                                                        │  │
│   │      ═══════                                                        │  │
│   │      Score = f(semantic_similarity, trust, distance, price)        │  │
│   │      → Top 10 Ergebnisse                                            │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Discovery Query

```yaml
discover {
  # Natürlichsprachlich oder strukturiert
  query: "150kW CCS Ladesäule"

  # Oder strukturiert
  blueprint: @ref("did:erynoa:blueprint:ev-charging-station-de:*")

  # Environment-Scope
  environment: @ref("did:erynoa:env:domain:ev-charging-de")

  # Constraints
  constraints: {
    power_output: { gte: 150 }
    connector_type: CCS
    location: {
      geohash: "u281"  # München-Area
      radius:  10km
    }
  }

  # Trust-Requirements
  trust: {
    min_aggregate:   0.7
    min_reliability: 0.8
  }

  # Ranking-Gewichte
  ranking: {
    semantic_similarity: 0.3
    trust:               0.3
    distance:            0.2
    price:               0.2
  }

  # Ergebnisse
  limit: 10
  include_fluid: true  # Echtzeit-Daten (Verfügbarkeit)
}
```

---

## Fluid Extensions

Echtzeit-Daten werden separat gehalten (TTL-basiert):

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   STATIC vs. FLUID DATA                                                    │
│                                                                             │
│   ┌─────────────────────────────────┐   ┌─────────────────────────────────┐│
│   │                                 │   │                                 ││
│   │   STATIC (AMO)                  │   │   FLUID (Extensions)            ││
│   │   ════════════                  │   │   ═══════════════════           ││
│   │                                 │   │                                 ││
│   │   • Standort: u281zq5           │   │   • Verfügbar: true            ││
│   │   • Leistung: 150kW             │   │   • Wartezeit: 0 min           ││
│   │   • Stecker: CCS                │   │   • Spot-Preis: 0.45 €/kWh     ││
│   │   • Betreiber: SWM              │   │   • Letzte Nutzung: vor 12min  ││
│   │                                 │   │                                 ││
│   │   Persistent, On-Chain          │   │   Temporär, TTL: 60s           ││
│   │                                 │   │                                 ││
│   └─────────────────────────────────┘   └─────────────────────────────────┘│
│                                                                             │
│   Discovery kombiniert beide Datenquellen.                                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Discovery Result

```yaml
discovery_result {
  query_id: "query-2025-001"
  timestamp: "2025-01-28T10:15:00Z"

  results: [
    {
      amo:   @ref("did:erynoa:amo:material:station-munich-001")
      score: 0.94

      match_details: {
        semantic_similarity: 0.95
        trust:               0.92
        distance:            1.2  # km
        price:               0.42 # €/kWh
      }

      fluid: {
        available:    true
        wait_time:    0
        spot_price:   0.42
        last_update:  "2025-01-28T10:14:50Z"
      }
    },
    # ... weitere Ergebnisse
  ]

  total_candidates:  500
  after_environment: 320
  after_trust:       180
  after_constraints: 45
  returned:          10
}
```

---

## Proaktives Discovery

Agenten können Discovery-Subscriptions erstellen:

```yaml
discovery_subscription {
  id:    "sub-2025-001"
  owner: @identity("did:erynoa:vehicle:123")

  # Trigger-Bedingungen
  trigger: {
    type: "location_change"
    params: {
      geohash_prefix: "u28"  # Wenn in München-Area
    }
  }

  # Discovery-Query (wird bei Trigger ausgeführt)
  query: { ... }

  # Notification
  notify: {
    method: "push"
    endpoint: "did:erynoa:vehicle:123#inbox"
  }
}
```

---

## Weiterführende Dokumente

- [environments.md](./environments.md) – Kontexte
- [governance.md](./governance.md) – Regelwerke
- [constraints.md](./constraints.md) – Einschränkungen
- [../schema/semantic-index.md](../schema/semantic-index.md) – Vector Search
