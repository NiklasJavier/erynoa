# ▣ SPHÄRE – Constraints

> **Schicht:** 3 – Räume
> **Sphäre:** ERY (Semantic-Modul)
> **Typ:** Einschränkungen und Policies

---

## Konzept

**Constraints** sind Regeln, die innerhalb eines Environments gelten. Sie definieren, was erlaubt und was verboten ist, und werden bei Discovery und Transaktion geprüft.

---

## Constraint-Typen

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   CONSTRAINT CATEGORIES                                                    │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   📊 ATTRIBUTE CONSTRAINTS                                          │  │
│   │      "power_output >= 22kW"                                         │  │
│   │      "connector_type in [CCS, Type2]"                              │  │
│   │                                                                     │  │
│   │   🎯 TRUST CONSTRAINTS                                              │  │
│   │      "operator.trust.reliability >= 0.8"                           │  │
│   │      "provider.karma_tier >= veteran"                              │  │
│   │                                                                     │  │
│   │   🌍 GEO CONSTRAINTS                                                │  │
│   │      "location.country == 'DE'"                                    │  │
│   │      "distance(seeker, provider) <= 50km"                          │  │
│   │                                                                     │  │
│   │   📜 COMPLIANCE CONSTRAINTS                                         │  │
│   │      "has_credential('eichrecht-certificate')"                     │  │
│   │      "complies_with('ISO-15118')"                                  │  │
│   │                                                                     │  │
│   │   ⏰ TEMPORAL CONSTRAINTS                                           │  │
│   │      "valid_from <= now() <= valid_until"                          │  │
│   │      "operating_hours.includes(now().time())"                      │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Constraint-Definition in ECL

```yaml
constraints "ev-charging-de-rules" {
  id:          "did:erynoa:constraints:ev-de-001"
  environment: @ref("did:erynoa:env:domain:ev-charging-de")

  rules: [
    # Eichrecht-Pflicht
    {
      name: "eichrecht_required"
      type: compliance
      rule: |
        has_credential(provider, "did:erynoa:credential:eichrecht:*")
      severity: mandatory
      message: "Eichrecht-Zertifikat erforderlich"
    },

    # Mindest-Trust für Betreiber
    {
      name: "operator_trust_minimum"
      type: trust
      rule: |
        provider.trust.reliability >= 0.7 AND
        provider.trust.integrity >= 0.6
      severity: mandatory
    },

    # AC-Stationen mindestens 11kW
    {
      name: "min_ac_power"
      type: attribute
      rule: |
        IF amo.connector_type IN [Type2, Schuko]
        THEN amo.power_output >= 11
      severity: warning
      message: "AC-Stationen sollten mindestens 11kW bieten"
    },

    # Geo-Einschränkung
    {
      name: "germany_only"
      type: geo
      rule: |
        amo.location.country == "DE"
      severity: mandatory
    }
  ]
}
```

---

## Severity Levels

| Level         | Bedeutung     | Bei Verletzung               |
| ------------- | ------------- | ---------------------------- |
| **mandatory** | Verpflichtend | Transaktion abgelehnt        |
| **warning**   | Empfohlen     | Warnung, Transaktion möglich |
| **info**      | Hinweis       | Nur Logging                  |

---

## Constraint-Prüfung

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   CONSTRAINT EVALUATION FLOW                                               │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   Intent eingereicht                                                │  │
│   │        │                                                            │  │
│   │        ▼                                                            │  │
│   │   ┌─────────────────────────────────────────────────────────────┐  │  │
│   │   │ Constraint Engine                                           │  │  │
│   │   │                                                             │  │  │
│   │   │  FOR each rule IN environment.constraints:                  │  │  │
│   │   │    result = evaluate(rule, intent, amo, participants)       │  │  │
│   │   │                                                             │  │  │
│   │   │    IF result == FAIL AND rule.severity == mandatory:        │  │  │
│   │   │      REJECT intent                                          │  │  │
│   │   │                                                             │  │  │
│   │   │    IF result == FAIL AND rule.severity == warning:          │  │  │
│   │   │      ADD warning to response                                │  │  │
│   │   │                                                             │  │  │
│   │   └─────────────────────────────────────────────────────────────┘  │  │
│   │        │                                                            │  │
│   │        ▼                                                            │  │
│   │   Constraint Report                                                 │  │
│   │   {                                                                 │  │
│   │     passed: 5,                                                      │  │
│   │     warnings: 1,                                                    │  │
│   │     failed: 0,                                                      │  │
│   │     details: [...]                                                  │  │
│   │   }                                                                 │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Dynamic Constraints

Constraints können dynamisch sein (basierend auf Kontext):

```yaml
dynamic_constraint {
  name: "peak_hour_premium_required"
  type: temporal

  condition: |
    now().hour() >= 17 AND now().hour() <= 20  # Rush Hour

  # Wenn Bedingung erfüllt:
  activated_rule: |
    provider.karma_tier >= veteran OR
    seeker.has_premium_subscription

  message: "Zu Stoßzeiten nur für Premium-Nutzer oder Veteran-Provider"
}
```

---

## Constraint Inheritance

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   CONSTRAINT INHERITANCE                                                   │
│                                                                             │
│   env:global                                                               │
│   └── constraint: "valid_did_required" (mandatory)                         │
│       │                                                                    │
│       ├── env:geo:europe                                                   │
│       │   └── constraint: "gdpr_compliance" (mandatory)                    │
│       │       │                                                            │
│       │       └── env:domain:ev-charging-de                                │
│       │           └── constraint: "eichrecht_required" (mandatory)         │
│       │           └── constraint: "min_ac_power" (warning)                 │
│                                                                             │
│   Constraints akkumulieren sich nach unten.                                │
│   Child-Environments können NICHT Parent-Constraints aufweichen.           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Weiterführende Dokumente

- [environments.md](./environments.md) – Kontexte
- [governance.md](./governance.md) – Regeländerungen
- [discovery.md](./discovery.md) – Constraint-Filtering
