# ◈ SCHEMA – Semantic Index

> **Schicht:** 1 – Wissen
> **Sphäre:** ERY (Semantic-Modul)
> **Kernfrage:** _„Was bedeutet etwas?"_

---

## Überblick

Der **Semantic Index** ist das Wissensgedächtnis von Erynoa – ein Qdrant-basierter Vektorindex, der Blueprints, Standards und Ontologien semantisch durchsuchbar macht.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   SEMANTIC INDEX ARCHITEKTUR                                               │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   ┌───────────────┐   ┌───────────────┐   ┌───────────────┐        │  │
│   │   │  Blueprints   │   │   Standards   │   │   Ontologie   │        │  │
│   │   │               │   │               │   │               │        │  │
│   │   │ Objektschemen │   │  ISO, OCPP,   │   │  Begriffsrel. │        │  │
│   │   │ Validierung   │   │  Eichrecht    │   │  Taxonomien   │        │  │
│   │   └───────┬───────┘   └───────┬───────┘   └───────┬───────┘        │  │
│   │           │                   │                   │                 │  │
│   │           └───────────────────┼───────────────────┘                 │  │
│   │                               ▼                                     │  │
│   │                    ┌─────────────────────┐                         │  │
│   │                    │  Vector Embeddings  │                         │  │
│   │                    │     (Qdrant)        │                         │  │
│   │                    └──────────┬──────────┘                         │  │
│   │                               │                                     │  │
│   │                               ▼                                     │  │
│   │                    ┌─────────────────────┐                         │  │
│   │                    │  Semantic Search    │                         │  │
│   │                    │  + Filter (DID)     │                         │  │
│   │                    └─────────────────────┘                         │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Funktionen

| Funktion                | Beschreibung                            |
| ----------------------- | --------------------------------------- |
| **Vektor-Suche**        | Semantische Ähnlichkeit zu Suchanfragen |
| **Blueprint-Registry**  | Zentrale Ablage für Objektdefinitionen  |
| **Standard-Referenzen** | Verknüpfung mit normativen Dokumenten   |
| **Ontologie-Mapping**   | Begriffe und ihre Beziehungen           |
| **Version-History**     | Unveränderliche Versionierung via DIDs  |

---

## Wissensebenen

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   DREI EBENEN DES WISSENS                                                  │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   EBENE 1: Normative Standards                                      │  │
│   │   ════════════════════════════                                      │  │
│   │   ISO 19112 · eCl@ss · AML/KYC · Industrienormen                   │  │
│   │   → Stabile Fundamente, selten geändert                            │  │
│   │                                                                     │  │
│   │                          ▼                                          │  │
│   │                                                                     │  │
│   │   EBENE 2: Domain Blueprints                                        │  │
│   │   ══════════════════════════                                        │  │
│   │   EV-Charging-Station · KYC-Credential · Energy-Asset              │  │
│   │   → Anwendungsspezifische Schablonen                               │  │
│   │                                                                     │  │
│   │                          ▼                                          │  │
│   │                                                                     │  │
│   │   EBENE 3: Fluid Extensions                                         │  │
│   │   ═════════════════════════                                         │  │
│   │   Echtzeit-Status · Spot-Preise · Verfügbarkeit                    │  │
│   │   → Kurzlebige, dynamische Daten (TTL)                             │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Query-Beispiel

```yaml
semantic_query {
  # Natürlichsprachliche Suche
  query: "Ladesäule mit 150kW CCS in München"

  # Filter
  filters: {
    blueprint: "did:erynoa:blueprint:ev-charging-station:*"
    environment: "did:erynoa:env:domain:ev-charging-de"
    trust_min: 0.7
  }

  # Ergebnis-Limit
  limit: 10

  # Inkludiere Fluid Extensions
  include_fluid: true
}
```

---

## Technologie-Stack

| Komponente       | Technologie           | Funktion                          |
| ---------------- | --------------------- | --------------------------------- |
| **Vector Store** | Qdrant                | HNSW-basierte Ähnlichkeitssuche   |
| **Embeddings**   | Sentence Transformers | Text → Vector Transformation      |
| **Index**        | Inverted + Vector     | Hybrid-Suche (Keyword + Semantic) |
| **Storage**      | Persistent            | Blueprints, Standards persistent  |
| **Cache**        | TTL-basiert           | Fluid Extensions temporär         |

---

## Integration mit anderen Schichten

| Schicht      | Integration                                         |
| ------------ | --------------------------------------------------- |
| **◉ ANKER**  | Jeder Eintrag hat DID, Author-DID referenziert      |
| **◊ METRIK** | Trust des Authors beeinflusst Ranking               |
| **▣ SPHÄRE** | Environments referenzieren Blueprints als Standards |
| **◐ IMPULS** | Agenten suchen Blueprints für Intent-Matching       |

---

## Weiterführende Dokumente

- [blueprints.md](./blueprints.md) – Objektdefinitionen
- [standards.md](./standards.md) – Normative Standards
- [ontologie.md](./ontologie.md) – Begriffsrelationen
