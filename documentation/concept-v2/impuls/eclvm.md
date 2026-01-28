# ◐ IMPULS – ECLVM

> **Schicht:** 4 – Handlung (Layer 0.5)
> **Sphäre:** ECHO (Runtime)
> **Typ:** Virtual Machine

---

## Konzept

Die **ECLVM** (Erynoa Configuration Language Virtual Machine) ist die Layer-0.5-Runtime, die ECL-Code ausführt. Sie interpretiert Policies, Constraints und Logic Guards.

---

## Architektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ECLVM ARCHITECTURE                                                       │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   ECL SOURCE CODE                                                   │  │
│   │   ═══════════════                                                   │  │
│   │   policy { ... }                                                    │  │
│   │   constraint { ... }                                                │  │
│   │   logic_guard { ... }                                               │  │
│   │        │                                                            │  │
│   │        ▼                                                            │  │
│   │   ┌─────────────────────────────────────────────────────────────┐  │  │
│   │   │ PARSER                                                      │  │  │
│   │   │ Lexer → AST → Semantic Analysis                            │  │  │
│   │   └──────────────────────────┬──────────────────────────────────┘  │  │
│   │                              │                                      │  │
│   │                              ▼                                      │  │
│   │   ┌─────────────────────────────────────────────────────────────┐  │  │
│   │   │ BYTECODE COMPILER                                           │  │  │
│   │   │ AST → Optimized Bytecode                                   │  │  │
│   │   └──────────────────────────┬──────────────────────────────────┘  │  │
│   │                              │                                      │  │
│   │                              ▼                                      │  │
│   │   ┌─────────────────────────────────────────────────────────────┐  │  │
│   │   │ ECLVM RUNTIME                                               │  │  │
│   │   │                                                             │  │  │
│   │   │  ┌────────────┐  ┌────────────┐  ┌────────────┐            │  │  │
│   │   │  │ Interpreter│  │ Sandbox    │  │ Gas Meter  │            │  │  │
│   │   │  │            │  │            │  │            │            │  │  │
│   │   │  │ Stack-based│  │ No I/O     │  │ Limits     │            │  │  │
│   │   │  │ Execution  │  │ No Network │  │ Resources  │            │  │  │
│   │   │  └────────────┘  └────────────┘  └────────────┘            │  │  │
│   │   │                                                             │  │  │
│   │   └─────────────────────────────────────────────────────────────┘  │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## ECLVM-Eigenschaften

| Eigenschaft       | Beschreibung                          |
| ----------------- | ------------------------------------- |
| **Deterministic** | Gleicher Input → Gleicher Output      |
| **Sandboxed**     | Kein Zugriff auf System-Ressourcen    |
| **Gas-Metered**   | Ressourcenverbrauch wird begrenzt     |
| **Pure**          | Keine Seiteneffekte, nur Berechnungen |
| **Verifiable**    | Execution kann reproduziert werden    |

---

## ECL-Module

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ECL MODULE HIERARCHY                                                     │
│                                                                             │
│   ecl/                                                                     │
│   ├── core/           # Basistypen, Operatoren                            │
│   │   ├── types       # DID, AMO, Credential, etc.                        │
│   │   ├── operators   # Vergleiche, Logik, Arithmetik                     │
│   │   └── functions   # Built-in Funktionen                               │
│   │                                                                        │
│   ├── identity/       # Identitäts-Operationen                            │
│   │   ├── did         # DID-Auflösung, Validierung                        │
│   │   ├── credential  # Credential-Prüfung                                │
│   │   └── delegation  # Delegations-Ketten                                │
│   │                                                                        │
│   ├── trust/          # Trust-Berechnungen                                │
│   │   ├── vector      # Trust-Vector-Operationen                          │
│   │   ├── karma       # Karma-Berechnungen                                │
│   │   └── gating      # Trust-Gating-Logik                                │
│   │                                                                        │
│   ├── agent/          # Agent-Logik                                       │
│   │   ├── policy      # Policy-Evaluation                                 │
│   │   ├── intent      # Intent-Matching                                   │
│   │   └── negotiation # Verhandlungs-Logik                                │
│   │                                                                        │
│   └── object/         # AMO-Operationen                                   │
│       ├── blueprint   # Blueprint-Validierung                             │
│       ├── constraint  # Constraint-Prüfung                                │
│       └── transition  # State-Transitionen                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Logic Guards

Logic Guards sind ECLVM-Programme, die AMO-Transitionen validieren:

```yaml
logic_guard "ev_charging_validate" {
  # Registrierung
  address: "0x1::ev_charging::validate"

  # Input-Schema
  input: {
    amo:         AMO
    transition:  Transition
    caller:      DID
    context:     Context
  }

  # Output
  output: {
    valid:   bool
    reason:  string?
  }

  # Logik
  body: |
    # Prüfe Blueprint-Compliance
    let blueprint = resolve(amo.blueprint)
    assert(amo.attributes conforms_to blueprint.attributes)

    # Prüfe Caller-Berechtigung
    let owner = amo.owner
    assert(caller == owner OR caller in owner.delegates)

    # Prüfe Transition-Gültigkeit
    match transition {
      Activate => {
        assert(amo.status == Pending)
        assert(amo has_credential("did:erynoa:credential:eichrecht:*"))
      }
      Deactivate => {
        assert(amo.status == Active)
        assert(no_active_sessions(amo))
      }
      _ => reject("Unknown transition")
    }

    return { valid: true }

  # Gas-Limit
  gas_limit: 10000
}
```

---

## Policy-Evaluation in ECLVM

```yaml
# Policy wird zu ECLVM-Bytecode kompiliert

policy_evaluate {
  # Input
  policy:  @ref("did:erynoa:policy:vehicle-charging-001")
  offer:   @ref("did:erynoa:offer:charging-001")
  context: {
    agent_trust:     0.85
    budget_remaining: 65.00
    current_time:    "2025-01-28T10:15:00Z"
  }

  # ECLVM führt aus:
  # 1. Parse Policy-Regeln
  # 2. Evaluiere auto_reject Conditions
  # 3. Evaluiere auto_accept Conditions
  # 4. Evaluiere escalate Conditions

  # Output
  result: {
    decision:  accept
    reasoning: [
      "price_per_kwh (0.42) <= max (0.50): PASS",
      "trust (0.92) >= min (0.7): PASS",
      "distance (1.2km) <= max (5km): PASS"
    ]
  }
}
```

---

## Gas-System

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ECLVM GAS COSTS                                                          │
│                                                                             │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │                                                                   │    │
│   │   Operation                                Gas Cost               │    │
│   │   ═════════                                ════════               │    │
│   │                                                                   │    │
│   │   Arithmetic (+, -, *, /)                  1                     │    │
│   │   Comparison (==, >, <, >=, <=)            1                     │    │
│   │   Logical (AND, OR, NOT)                   1                     │    │
│   │   Variable Access                          2                     │    │
│   │   Function Call                            10                    │    │
│   │   DID Resolution                           50                    │    │
│   │   Credential Verification                  100                   │    │
│   │   Trust Vector Lookup                      30                    │    │
│   │   Blueprint Validation                     200                   │    │
│   │                                                                   │    │
│   │   Default Gas Limit: 10,000                                      │    │
│   │   Complex Operations: Up to 100,000                              │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│   Gas-Exhaustion führt zu Execution-Abort (keine Seiteneffekte).           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Weiterführende Dokumente

- [policy.md](./policy.md) – Policy-Definition
- [../sphaere/constraints.md](../sphaere/constraints.md) – Constraint-Definition
- [../chronik/logic-guards.md](../chronik/logic-guards.md) – On-Chain Guards
- [../appendix/ecl-referenz.md](../appendix/ecl-referenz.md) – ECL-Sprachspezifikation
