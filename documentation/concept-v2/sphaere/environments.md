# ▣ SPHÄRE – Environments

> **Schicht:** 3 – Räume
> **Sphäre:** ERY (Semantic-Modul)
> **Kernfrage:** _„In welchem Kontext?"_

---

## Konzept

**Environments** sind abgegrenzte Kontextblasen, in denen spezifische Regeln, Standards und Governance gelten. Sie definieren die "Spielfelder" für Agenten-Interaktionen.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ENVIRONMENTS = ABGEGRENZTE REGELRÄUME                                    │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   🌍 Global Environment (Basis-Protokoll)                          │  │
│   │   ═══════════════════════════════════════                          │  │
│   │                                                                     │  │
│   │   ┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐  │  │
│   │   │                 │   │                 │   │                 │  │  │
│   │   │   🇩🇪 DE Domain  │   │   🇫🇷 FR Domain  │   │   🇪🇺 EU Domain  │  │  │
│   │   │   EV-Charging   │   │   EV-Charging   │   │   Energy Trading│  │  │
│   │   │                 │   │                 │   │                 │  │  │
│   │   │   + Eichrecht   │   │   + AFIR        │   │   + MiFID II    │  │  │
│   │   │   + PTB         │   │   + French NF   │   │   + REMIT       │  │  │
│   │   │                 │   │                 │   │                 │  │  │
│   │   └─────────────────┘   └─────────────────┘   └─────────────────┘  │  │
│   │                                                                     │  │
│   │   Jedes Environment hat eigene Blueprints, Standards, Governance   │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Environment-Typen

| Typ         | Scope             | Beispiel                    | Governance      |
| ----------- | ----------------- | --------------------------- | --------------- |
| **Domain**  | Branche/Industrie | `env:domain:ev-charging-de` | Branchenverband |
| **Geo**     | Geographisch      | `env:geo:europe`            | Regulatoren     |
| **Private** | Unternehmen       | `env:private:swm`           | Eigentümer      |
| **Testnet** | Entwicklung       | `env:test:dev-staging`      | Entwickler      |

---

## Environment-Struktur

```yaml
environment "ev-charging-de" {
  id:   "did:erynoa:env:domain:ev-charging-de"
  name: "EV Charging Germany"
  type: domain

  # Governance
  governance: {
    owner:     @identity("did:erynoa:org:bdew")  # Bundesverband
    council:   @ref("did:erynoa:gov:ev-de-council")
    proposals: @ref("did:erynoa:gov:ev-de-proposals")
  }

  # Geltende Standards
  standards: [
    @ref("did:erynoa:standard:ocpp:2.0.1"),
    @ref("did:erynoa:standard:eichrecht:ptb"),
    @ref("did:erynoa:standard:iso:15118")
  ]

  # Erforderliche Blueprints
  required_blueprints: [
    @ref("did:erynoa:blueprint:ev-charging-station-de:*")
  ]

  # Trust-Anforderungen
  trust_requirements: {
    min_operator_trust:  0.7
    min_provider_trust:  0.6
  }

  # Mitgliedschaft
  membership: {
    type:         "open"  # oder "invite", "approval"
    kyc_required: true
    blueprint:    @ref("did:erynoa:blueprint:operator-credential-de:*")
  }
}
```

---

## Environment-Hierarchie

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ENVIRONMENT NESTING                                                      │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   env:global (Erynoa Protocol)                                     │  │
│   │   ║                                                                 │  │
│   │   ╠══ env:geo:europe                                               │  │
│   │   ║   ║                                                             │  │
│   │   ║   ╠══ env:domain:ev-charging-eu                                │  │
│   │   ║   ║   ║                                                         │  │
│   │   ║   ║   ╠══ env:domain:ev-charging-de (+ Eichrecht)              │  │
│   │   ║   ║   ╠══ env:domain:ev-charging-fr (+ AFIR-FR)                │  │
│   │   ║   ║   └══ env:domain:ev-charging-nl                            │  │
│   │   ║   ║                                                             │  │
│   │   ║   └══ env:domain:energy-trading-eu                             │  │
│   │   ║                                                                 │  │
│   │   ╠══ env:geo:north-america                                        │  │
│   │   ║   └══ ...                                                       │  │
│   │   ║                                                                 │  │
│   │   └══ env:private:swm (Stadtwerke München)                         │  │
│   │       └══ env:private:swm-internal-fleet                           │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│   Regeln vererben sich nach unten, können aber überschrieben werden.       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Environment-Beitritt

```yaml
environment join {
  environment: @ref("did:erynoa:env:domain:ev-charging-de")

  # Antragsteller
  applicant: @identity("did:erynoa:org:new-operator")

  # Erforderliche Credentials
  credentials: [
    @ref("did:erynoa:credential:kyc-verified"),
    @ref("did:erynoa:credential:operator-license-de")
  ]

  # Wird geprüft gegen Environment-Membership-Regeln
}
```

---

## Cross-Environment Operationen

Agenten können in mehreren Environments aktiv sein:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   MULTI-ENVIRONMENT AGENT                                                  │
│                                                                             │
│   did:erynoa:agent:provider:swm                                            │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │                                                                   │    │
│   │   Aktiv in:                                                       │    │
│   │                                                                   │    │
│   │   ┌─────────────────┐   ┌─────────────────┐                      │    │
│   │   │ ev-charging-de  │   │ energy-trading  │                      │    │
│   │   │                 │   │                 │                      │    │
│   │   │ Trust: 0.92     │   │ Trust: 0.78     │ ← Separate Trusts   │    │
│   │   │ Role: Provider  │   │ Role: Trader    │                      │    │
│   │   └─────────────────┘   └─────────────────┘                      │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│   Trust ist Environment-spezifisch!                                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Weiterführende Dokumente

- [governance.md](./governance.md) – Regelwerke
- [discovery.md](./discovery.md) – Objekt-Suche
- [constraints.md](./constraints.md) – Einschränkungen
