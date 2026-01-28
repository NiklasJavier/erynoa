# ◆ CHRONIK – Logic Guards

> **Schicht:** 5 – Beweis
> **Sphäre:** NOA (Validation Layer)
> **Typ:** On-Chain Validierung

---

## Konzept

**Logic Guards** sind deterministische Programme, die AMO-Transitionen und Transaktionen validieren. Sie laufen in der ECLVM und stellen sicher, dass nur gültige Zustandsänderungen stattfinden.

---

## Architektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   LOGIC GUARD FLOW                                                         │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   Transition Request                                                │  │
│   │   ══════════════════                                                │  │
│   │   "Aktiviere AMO station-munich-001"                               │  │
│   │        │                                                            │  │
│   │        ▼                                                            │  │
│   │   ┌─────────────────────────────────────────────────────────────┐  │  │
│   │   │ Blueprint Lookup                                            │  │  │
│   │   │ → logic_guard: "0x1::ev_charging::validate"                │  │  │
│   │   └──────────────────────────┬──────────────────────────────────┘  │  │
│   │                              │                                      │  │
│   │                              ▼                                      │  │
│   │   ┌─────────────────────────────────────────────────────────────┐  │  │
│   │   │ ECLVM Execution                                             │  │  │
│   │   │                                                             │  │  │
│   │   │  Input:                                                     │  │  │
│   │   │  - amo: current state                                       │  │  │
│   │   │  - transition: activate                                     │  │  │
│   │   │  - caller: did:erynoa:org:stadtwerke-munich                │  │  │
│   │   │  - context: environment, timestamp, etc.                    │  │  │
│   │   │                                                             │  │  │
│   │   │  Logic:                                                     │  │  │
│   │   │  - Check caller authorization                               │  │  │
│   │   │  - Check preconditions (status == pending)                  │  │  │
│   │   │  - Check credentials (eichrecht, ocpp)                      │  │  │
│   │   │  - Check environment constraints                            │  │  │
│   │   │                                                             │  │  │
│   │   │  Output: { valid: true/false, reason: string }              │  │  │
│   │   │                                                             │  │  │
│   │   └──────────────────────────┬──────────────────────────────────┘  │  │
│   │                              │                                      │  │
│   │              ┌───────────────┴───────────────┐                     │  │
│   │              │                               │                     │  │
│   │              ▼                               ▼                     │  │
│   │         valid: true                     valid: false              │  │
│   │         → Execute                       → Reject                   │  │
│   │         → Create Event                  → Return Error            │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Logic Guard Definition

```yaml
logic_guard "ev_charging_validate" {
  # Adresse (für ECLVM)
  address: "0x1::ev_charging::validate"

  # Versioning
  version: "1.0.0"

  # Input-Schema
  input_schema: {
    amo:        AMO
    transition: Transition
    caller:     DID
    context:    Context
  }

  # Output-Schema
  output_schema: {
    valid:   bool
    reason:  string?
    effects: Effect[]?  # Optional: Seiteneffekte
  }

  # Logik (ECL)
  body: |
    # 1. Caller-Berechtigung prüfen
    let owner = amo.owner
    let delegates = resolve_delegates(owner)

    if not (caller == owner or caller in delegates) {
      return { valid: false, reason: "Unauthorized caller" }
    }

    # 2. Transition-spezifische Prüfungen
    match transition {
      Activate => {
        # Status muss Pending sein
        if amo.status != Pending {
          return { valid: false, reason: "Can only activate pending AMOs" }
        }

        # Credentials prüfen
        let required = ["eichrecht", "ocpp-certified"]
        for cred in required {
          if not has_valid_credential(amo, cred) {
            return { valid: false, reason: "Missing credential: " + cred }
          }
        }
      }

      Suspend => {
        if amo.status != Active {
          return { valid: false, reason: "Can only suspend active AMOs" }
        }
      }

      Decommission => {
        # Prüfe keine aktiven Sessions
        let active_sessions = query_active_sessions(amo.id)
        if active_sessions.count > 0 {
          return { valid: false, reason: "Cannot decommission with active sessions" }
        }
      }

      _ => {
        return { valid: false, reason: "Unknown transition" }
      }
    }

    # 3. Environment-Constraints prüfen
    let env = context.environment
    let constraints = resolve_constraints(env)

    for constraint in constraints {
      if not evaluate(constraint, amo, transition) {
        return { valid: false, reason: "Constraint violation: " + constraint.name }
      }
    }

    # Alles OK
    return { valid: true }

  # Ressourcen-Limits
  gas_limit: 50000
}
```

---

## Guard-Typen

| Typ                   | Anwendung                | Beispiel                        |
| --------------------- | ------------------------ | ------------------------------- |
| **AMO Guard**         | AMO-Transitionen         | Aktivierung, Dekommissionierung |
| **Transaction Guard** | Transaktions-Validierung | Payment-Prüfung                 |
| **Agreement Guard**   | Vertragsschluss          | Terms-Validierung               |
| **Credential Guard**  | Credential-Issuance      | Berechtigungs-Prüfung           |

---

## Guard-Kette

Mehrere Guards können sequentiell ausgeführt werden:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   GUARD CHAIN                                                              │
│                                                                             │
│   Transaction Request                                                       │
│        │                                                                    │
│        ▼                                                                    │
│   ┌─────────────────┐                                                      │
│   │ Guard 1:        │                                                      │
│   │ Authorization   │──▶ PASS                                              │
│   └────────┬────────┘                                                      │
│            │                                                                │
│            ▼                                                                │
│   ┌─────────────────┐                                                      │
│   │ Guard 2:        │                                                      │
│   │ Trust Check     │──▶ PASS                                              │
│   └────────┬────────┘                                                      │
│            │                                                                │
│            ▼                                                                │
│   ┌─────────────────┐                                                      │
│   │ Guard 3:        │                                                      │
│   │ Budget Check    │──▶ PASS                                              │
│   └────────┬────────┘                                                      │
│            │                                                                │
│            ▼                                                                │
│   ┌─────────────────┐                                                      │
│   │ Guard 4:        │                                                      │
│   │ Domain Rules    │──▶ PASS                                              │
│   └────────┬────────┘                                                      │
│            │                                                                │
│            ▼                                                                │
│       EXECUTE                                                               │
│                                                                             │
│   Alle Guards müssen PASS zurückgeben.                                     │
│   Erstes FAIL stoppt die Kette.                                            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Determinismus

Logic Guards müssen deterministisch sein:

| Erlaubt                   | Verboten                       |
| ------------------------- | ------------------------------ |
| Arithmetik                | Random                         |
| Logik                     | Network I/O                    |
| String-Ops                | File I/O                       |
| DID-Resolution (cached)   | Time (außer context.timestamp) |
| Credential-Check (cached) | External Calls                 |

---

## Weiterführende Dokumente

- [noa-ledger.md](./noa-ledger.md) – Event-Speicherung
- [amo.md](./amo.md) – Objekte
- [../impuls/eclvm.md](../impuls/eclvm.md) – Runtime
