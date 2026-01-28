# Erynoa – Konzept Navigator

> **Version:** 2.1 – Identity-First + ECLVM
> **Datum:** Januar 2026
> **Status:** Konsolidierte Dokumentation

---

## Das Erynoa-Protokoll in einem Satz

> _„Ein kybernetisches Protokoll, das Maschinen befähigt, eigenständig zu handeln, zu verhandeln und voneinander zu lernen – mit mathematisch fundiertem Vertrauen statt zentraler Autoritäten."_

---

## Die sieben Schichten

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                    E R Y N O A   S C H I C H T E N                         │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │                         ┌─────────────┐                             │  │
│   │                         │  ◉ ANKER    │  Schicht 0                  │  │
│   │                         │  Identität  │  "Wer existiert?"           │  │
│   │                         └──────┬──────┘                             │  │
│   │                                │                                    │  │
│   │              ┌─────────────────┴─────────────────┐                 │  │
│   │              ▼                                   ▼                 │  │
│   │       ┌─────────────┐                     ┌─────────────┐          │  │
│   │       │  ◈ SCHEMA   │                     │  ◊ METRIK   │          │  │
│   │       │   Wissen    │  Schicht 1          │  Vertrauen  │ Schicht 2│  │
│   │       │  "Was ist?" │                     │  "Wie gut?" │          │  │
│   │       └──────┬──────┘                     └──────┬──────┘          │  │
│   │              │                                   │                 │  │
│   │              └─────────────────┬─────────────────┘                 │  │
│   │                                ▼                                   │  │
│   │                         ┌─────────────┐                            │  │
│   │                         │  ▣ SPHÄRE   │  Schicht 3                 │  │
│   │                         │   Ordnung   │  "Wo gilt was?"            │  │
│   │                         └──────┬──────┘                            │  │
│   │                                │                                   │  │
│   │                                ▼                                   │  │
│   │                         ┌─────────────┐                            │  │
│   │                         │  ◐ IMPULS   │  Schicht 4                 │  │
│   │                         │  Handlung   │  "Was geschieht?"          │  │
│   │                         └──────┬──────┘                            │  │
│   │                                │                                   │  │
│   │                                ▼                                   │  │
│   │                         ┌─────────────┐                            │  │
│   │                         │  ◆ CHRONIK  │  Schicht 5                 │  │
│   │                         │  Finalität  │  "Was ist wahr?"           │  │
│   │                         └──────┬──────┘                            │  │
│   │                                │                                   │  │
│   │                                ▼                                   │  │
│   │                         ┌─────────────┐                            │  │
│   │                         │  ◇ NEXUS    │  Schicht 6                 │  │
│   │                         │ Vernetzung  │  "Wie verbunden?"          │  │
│   │                         └─────────────┘                            │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Empfohlene Lesereihenfolge

| Schritt | Schicht   | Dokumente                                                                                 | Fokus                    |
| ------- | --------- | ----------------------------------------------------------------------------------------- | ------------------------ |
| 1       | ◉ ANKER   | [identity-first](./anker/identity-first.md) → [did-erynoa](./anker/did-erynoa.md)         | Existenz verstehen       |
| 2       | ◈ SCHEMA  | [blueprints](./schema/blueprints.md) → [semantic-index](./schema/semantic-index.md)       | Wissen strukturieren     |
| 3       | ◊ METRIK  | [trust-vectors](./metrik/trust-vectors.md) → [karma-engine](./metrik/karma-engine.md)     | Vertrauen quantifizieren |
| 4       | ▣ SPHÄRE  | [environments](./sphaere/environments.md) → [governance](./sphaere/governance.md)         | Ordnung schaffen         |
| 5       | ◐ IMPULS  | [agent-modell](./impuls/agent-modell.md) → [cybernetic-loop](./impuls/cybernetic-loop.md) | Handlung ermöglichen     |
| 6       | ◆ CHRONIK | [noa-ledger](./chronik/noa-ledger.md) → [amo](./chronik/amo.md)                           | Wahrheit finalisieren    |
| 7       | ◇ NEXUS   | [multi-chain](./nexus/multi-chain.md) → [bridges](./nexus/bridges.md)                     | Vernetzung herstellen    |

---

## Die Kybernetische Triade

Die sieben Schichten sind auf drei Sphären verteilt:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   🔮 ERY (Semantic & Identity Lattice)                                      │
│   ════════════════════════════════════                                      │
│   ◉ ANKER    │ DACS-Modul: DIDs, VCs, Sub-Identities, Multi-Chain          │
│   ◈ SCHEMA   │ Semantic-Modul: Blueprints, Standards, Ontologie            │
│   ◊ METRIK   │ Karmic-Modul: Trust Vectors, Karma Tiers, Attestations      │
│   ▣ SPHÄRE   │ Discovery-Modul: Environments, Governance, Search           │
│                                                                             │
│   🤖 ECHO (Emergent Swarm) + Layer 0.5                                      │
│   ════════════════════════════════════                                      │
│   ◐ IMPULS   │ Agenten, ECLVM, Intents, Policies, Negotiation, Wallet      │
│                                                                             │
│   ⚡ NOA (Causal Ledger)                                                     │
│   ══════════════════════                                                    │
│   ◆ CHRONIK  │ AMOs, Logic Guards, MoveVM, Value Streaming, Finality       │
│                                                                             │
│   🔗 Querschnitt                                                            │
│   ══════════════                                                            │
│   ◇ NEXUS    │ Multi-Chain Adapters, Bridges, Network Selection            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Kausalitätsprinzip

> _„Etwas kann nur handeln, wenn es existiert. Es kann nur bewertet werden, wenn es bekannt ist. Es kann nur in einer Ordnung agieren, wenn es vertraut wird. Nur finalisierte Handlungen sind Wahrheit."_

```
ANKER ──▶ SCHEMA ──▶ METRIK ──▶ SPHÄRE ──▶ IMPULS ──▶ CHRONIK ──▶ NEXUS
  │         │          │          │          │           │
  ▼         ▼          ▼          ▼          ▼           ▼
"Wer?"   "Was?"    "Wie gut?"  "Wo?"     "Wie?"      "Wahr?"
```

---

## Feedback-Loops

Das System ist kybernetisch – Ergebnisse fließen zurück:

```
CHRONIK ───────────────────────────────────▶ METRIK
(Transaktion finalisiert)                    (Karma Update)

METRIK ────────────────────────────────────▶ SPHÄRE
(Trust-Änderung)                             (Governance Voting)

SPHÄRE ────────────────────────────────────▶ IMPULS
(Neue Regeln)                                (Agent-Policies)
```

---

## Dokumentenstruktur

```
concept-v2/
├── 00-navigator.md          # Diese Datei
├── anker/                   # ◉ Schicht 0: Identität
├── schema/                  # ◈ Schicht 1: Wissen
├── metrik/                  # ◊ Schicht 2: Vertrauen
├── sphaere/                 # ▣ Schicht 3: Ordnung
├── impuls/                  # ◐ Schicht 4: Handlung
├── chronik/                 # ◆ Schicht 5: Finalität
├── nexus/                   # ◇ Schicht 6: Vernetzung
└── appendix/                # Glossar, ECL-Referenz, Use Cases
```

---

## Verwandte Dokumentation

| Bereich              | Pfad                                                                             |
| -------------------- | -------------------------------------------------------------------------------- |
| System-Dokumentation | [../system/](../system/)                                                         |
| Backend-Architektur  | [../system/reference/architecture.md](../system/reference/architecture.md)       |
| Deployment           | [../system/guides/unified-deployment.md](../system/guides/unified-deployment.md) |
