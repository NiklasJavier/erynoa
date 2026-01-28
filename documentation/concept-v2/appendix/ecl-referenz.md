# 📖 APPENDIX – ECL-Referenz

> **Typ:** Referenz
> **Version:** ECL 2.1

---

## Überblick

**ECL** (Erynoa Configuration Language) ist die domänenspezifische Sprache zur Definition von Objekten, Policies, Constraints und Logik in Erynoa.

---

## Syntax-Grundlagen

### Deklarationen

```yaml
# Typ-Deklaration
type_name "identifier" {
  property1: value1
  property2: value2
}

# Beispiel
agent "my-agent" {
  id:   "did:erynoa:agent:seeker:my-agent"
  type: seeker
}
```

### Referenzen

```yaml
# Identitätsreferenz
@identity("did:erynoa:org:example")

# Objektreferenz
@ref("did:erynoa:blueprint:ev-charging:v1")

# Wildcard
@ref("did:erynoa:credential:kyc:*")
```

### Operatoren

```yaml
# Vergleich
{ gte: 50 }       # >= 50
{ lte: 100 }      # <= 100
{ gt: 0 }         # > 0
{ lt: 1000 }      # < 1000
{ eq: "active" }  # == "active"
{ neq: null }     # != null

# Mengen
{ in: [A, B, C] }     # Element von
{ not_in: [X, Y] }    # Nicht Element von

# Logik
{ and: [cond1, cond2] }
{ or: [cond1, cond2] }
{ not: condition }
```

---

## Datentypen

### Primitive

| Typ      | Beispiel        | Beschreibung |
| -------- | --------------- | ------------ |
| `string` | `"hello"`       | Text         |
| `number` | `42`, `3.14`    | Zahl         |
| `bool`   | `true`, `false` | Boolean      |
| `null`   | `null`          | Kein Wert    |

### Erynoa-Typen

| Typ         | Beispiel                 | Beschreibung  |
| ----------- | ------------------------ | ------------- |
| `did`       | `"did:erynoa:..."`       | Dezentrale ID |
| `geohash`   | `"u281zq5"`              | Geo-Lokation  |
| `timestamp` | `"2025-01-28T10:00:00Z"` | ISO 8601 Zeit |
| `duration`  | `300`                    | Sekunden      |
| `currency`  | `EUR`, `USDC`            | Währung       |

### Komplexe Typen

```yaml
# Object
attributes: {
  power: 150
  type: CCS
}

# Array
values: [A, B, C]

# Map
mapping: {
  key1: value1
  key2: value2
}
```

---

## Block-Typen

### Identity

```yaml
identity "org-example" {
  id:     "did:erynoa:org:example"
  type:   organization
  name:   "Example Corp"
  keys:   [{ ... }]
  services: [{ ... }]
}
```

### Blueprint

```yaml
blueprint "ev-charging" {
  id:       "did:erynoa:blueprint:ev-charging:v1"
  version:  "1.0.0"
  based_on: [@ref("...")]

  attributes: {
    power_output: { type: number, min: 0 }
    connector:    { type: enum, values: [...] }
  }

  logic_guard: "0x1::ev::validate"
}
```

### AMO

```yaml
amo "station-001" {
  id:        "did:erynoa:amo:material:station-001"
  type:      material
  blueprint: @ref("...")
  owner:     @identity("...")

  attributes: { ... }
  status:    active
}
```

### Agent

```yaml
agent "seeker-vehicle" {
  id:     "did:erynoa:agent:seeker:vehicle-123"
  type:   seeker
  owner:  @identity("...")
  policy: @ref("...")
  wallet: { ... }
}
```

### Policy

```yaml
policy "charging-policy" {
  id:    "did:erynoa:policy:charging-001"
  scope: { agent_types: [seeker], intent_types: [charge_vehicle] }

  auto_accept: {
    conditions: [{ price: { lte: 0.50 } }]
    combine: AND
  }

  auto_reject: { ... }
  escalate:    { ... }
  limits:      { ... }
}
```

### Intent

```yaml
intent "charge-request" {
  id:          "did:erynoa:intent:charge-001"
  type:        charge_vehicle
  seeker:      @identity("...")
  environment: @ref("...")

  constraints: { ... }
  budget:      { ... }
  trust_requirements: { ... }
}
```

### Environment

```yaml
environment "ev-charging-de" {
  id:         "did:erynoa:env:domain:ev-charging-de"
  type:       domain
  governance: { ... }
  standards:  [...]
  membership: { ... }
}
```

### Constraint

```yaml
constraint "eichrecht-required" {
  name:     "eichrecht_required"
  type:     compliance
  rule:     |
    has_credential(provider, "did:erynoa:credential:eichrecht:*")
  severity: mandatory
}
```

### Logic Guard

```yaml
logic_guard "ev-validate" {
  address:   "0x1::ev_charging::validate"
  version:   "1.0.0"

  input_schema:  { ... }
  output_schema: { ... }

  body: |
    # ECL Logic
    ...

  gas_limit: 50000
}
```

---

## Ausdrücke

### Bedingungen

```yaml
# Einfach
price <= 0.50

# Mit Trust
provider.trust.reliability >= 0.7

# Mit Credential
has_credential(amo, "did:erynoa:credential:eichrecht:*")

# Temporal
now() >= valid_from AND now() <= valid_until

# Geo
distance(seeker.location, provider.location) <= 10km
```

### Match-Ausdrücke

```yaml
match transition {
  Activate => {
    # Logik für Activate
  }
  Suspend => {
    # Logik für Suspend
  }
  _ => {
    # Default
  }
}
```

### Let-Bindungen

```yaml
let owner = amo.owner
let delegates = resolve_delegates(owner)
let is_authorized = caller == owner OR caller in delegates
```

---

## Built-in Funktionen

### Identität

| Funktion                           | Beschreibung          |
| ---------------------------------- | --------------------- |
| `resolve(did)`                     | DID-Dokument auflösen |
| `resolve_delegates(did)`           | Delegierte auflösen   |
| `verify_signature(data, sig, key)` | Signatur prüfen       |

### Trust

| Funktion              | Beschreibung         |
| --------------------- | -------------------- |
| `get_trust(did, env)` | Trust Vector abrufen |
| `get_karma(did)`      | Karma-Punkte abrufen |
| `get_tier(did)`       | Karma-Tier abrufen   |

### Credential

| Funktion                           | Beschreibung          |
| ---------------------------------- | --------------------- |
| `has_credential(subject, pattern)` | Credential prüfen     |
| `verify_credential(vc)`            | Credential validieren |
| `is_valid(credential)`             | Gültigkeit prüfen     |

### Geo

| Funktion                    | Beschreibung       |
| --------------------------- | ------------------ |
| `distance(a, b)`            | Distanz berechnen  |
| `within(point, polygon)`    | In Polygon?        |
| `geohash_prefix(hash, len)` | Prefix extrahieren |

### Zeit

| Funktion             | Beschreibung          |
| -------------------- | --------------------- |
| `now()`              | Aktueller Zeitstempel |
| `duration_since(ts)` | Zeit seit ts          |
| `is_expired(ts)`     | Abgelaufen?           |

---

## Operationen

### AMO-Operationen

```yaml
amo create { ... }
amo transition { amo: ..., action: activate }
amo query { filter: ... }
```

### Agent-Operationen

```yaml
agent create_intent { ... }
agent accept_offer { ... }
agent reject_offer { ... }
```

### Wallet-Operationen

```yaml
wallet topup { ... }
wallet pay { ... }
wallet stream { ... }
```

---

## Weiterführende Dokumente

- [glossar.md](./glossar.md) – Begriffsdefinitionen
- [anwendungen.md](./anwendungen.md) – Use Cases
- [../impuls/eclvm.md](../impuls/eclvm.md) – Runtime
