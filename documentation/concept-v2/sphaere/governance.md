# ▣ SPHÄRE – Governance

> **Schicht:** 3 – Räume
> **Sphäre:** ERY (Governance-Modul)
> **Typ:** Regelwerke und Entscheidungsfindung

---

## Konzept

**Governance** definiert, wie Environments verwaltet werden: Wer darf Regeln ändern, wie werden Entscheidungen getroffen, welche Prozesse gelten.

---

## Governance-Modelle

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   GOVERNANCE SPECTRUM                                                      │
│                                                                             │
│   Zentralisiert                                    Dezentralisiert         │
│   ─────────────────────────────────────────────────────────────────        │
│   │                                                               │        │
│   │   👤 Single Owner    👥 Council    🗳️ DAO    🌐 Protocol     │        │
│   │   (Private Env)      (Domain)     (Public)  (Global)         │        │
│   │                                                               │        │
│   ─────────────────────────────────────────────────────────────────        │
│                                                                             │
│   Unterschiedliche Environments nutzen unterschiedliche Modelle.           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Council Governance

Für Domain-Environments typisch:

```yaml
governance "ev-charging-de-council" {
  id:   "did:erynoa:gov:ev-de-council"
  type: council

  # Council-Mitglieder
  members: [
    { did: "did:erynoa:org:bdew",        role: chair,    weight: 2 },
    { did: "did:erynoa:org:vde",         role: member,   weight: 1 },
    { did: "did:erynoa:org:bne",         role: member,   weight: 1 },
    { did: "did:erynoa:org:tuev-sued",   role: observer, weight: 0 }
  ]

  # Entscheidungsregeln
  voting: {
    quorum:            0.6   # 60% müssen abstimmen
    approval_threshold: 0.66  # 2/3 Mehrheit
    voting_period:     7     # Tage
  }

  # Karma-basiertes Voting-Gewicht
  karma_weighted: true
  karma_tiers: {
    elder:       3
    veteran:     2
    established: 1
    newcomer:    0  # Kein Stimmrecht
  }
}
```

---

## Proposal-Prozess

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   GOVERNANCE PROPOSAL LIFECYCLE                                            │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   1. DRAFT                                                          │  │
│   │      Proposal wird erstellt                                         │  │
│   │      - Änderung beschreiben                                         │  │
│   │      - Begründung liefern                                           │  │
│   │           │                                                         │  │
│   │           ▼                                                         │  │
│   │   2. DISCUSSION (7 Tage)                                            │  │
│   │      Community diskutiert                                           │  │
│   │      - Feedback sammeln                                             │  │
│   │      - Proposal anpassen                                            │  │
│   │           │                                                         │  │
│   │           ▼                                                         │  │
│   │   3. VOTING (7 Tage)                                                │  │
│   │      Council stimmt ab                                              │  │
│   │      - Karma-gewichtete Stimmen                                     │  │
│   │      - Quorum prüfen                                                │  │
│   │           │                                                         │  │
│   │           ▼                                                         │  │
│   │   4. EXECUTION / REJECTION                                          │  │
│   │      Bei Annahme: Änderung wird aktiviert                          │  │
│   │      Bei Ablehnung: Begründung dokumentiert                        │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Proposal-Typen

| Typ                   | Beschreibung               | Approval-Threshold |
| --------------------- | -------------------------- | ------------------ |
| **Standard-Addition** | Neuen Standard hinzufügen  | 66%                |
| **Standard-Removal**  | Standard entfernen         | 75%                |
| **Blueprint-Change**  | Blueprint ändern           | 66%                |
| **Trust-Threshold**   | Trust-Anforderungen ändern | 75%                |
| **Membership-Rule**   | Beitrittsregeln ändern     | 66%                |
| **Council-Change**    | Council-Mitglieder ändern  | 80%                |
| **Emergency**         | Sofortmaßnahme             | Chair + 50%        |

---

## Governance-Struktur in ECL

```yaml
proposal {
  id:   "did:erynoa:proposal:ev-de-2025-001"
  type: standard_addition

  # Antragsteller
  proposer: @identity("did:erynoa:org:ionity")

  # Betroffenes Environment
  environment: @ref("did:erynoa:env:domain:ev-charging-de")

  # Änderung
  change: {
    action: "add_standard"
    target: @ref("did:erynoa:standard:megawatt-charging:v1")
    rationale: "Megawatt Charging System für Nutzfahrzeuge"
  }

  # Status
  status: voting

  # Stimmen
  votes: {
    for:     [did1, did2, did3]   # 3 × weight
    against: [did4]               # 1 × weight
    abstain: [did5]               # 1 × weight
  }
}
```

---

## On-Chain vs. Off-Chain Governance

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   HYBRID GOVERNANCE                                                        │
│                                                                             │
│   ┌─────────────────────────────────┐   ┌─────────────────────────────────┐│
│   │                                 │   │                                 ││
│   │   OFF-CHAIN                     │   │   ON-CHAIN                      ││
│   │   ═════════                     │   │   ════════                      ││
│   │                                 │   │                                 ││
│   │   • Diskussionen                │   │   • Finale Abstimmungen        ││
│   │   • Draft-Phase                 │   │   • Execution                   ││
│   │   • Feedback                    │   │   • Unveränderlicher Record    ││
│   │   • Informelle Absprachen       │   │   • Rechtssicherheit            ││
│   │                                 │   │                                 ││
│   │   Schnell, flexibel             │   │   Permanent, auditierbar       ││
│   │                                 │   │                                 ││
│   └─────────────────────────────────┘   └─────────────────────────────────┘│
│                                                                             │
│   Kombination aus Agilität und Verbindlichkeit.                            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Weiterführende Dokumente

- [environments.md](./environments.md) – Kontexte
- [discovery.md](./discovery.md) – Suche
- [constraints.md](./constraints.md) – Einschränkungen
