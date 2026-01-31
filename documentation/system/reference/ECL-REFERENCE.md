# ECL Reference - Erynoa Configuration Language

> **Version**: 1.0
> **Status**: Production-ready
> **VM**: ECLVM (Stack-based, Gas-metered)

ECL ist eine domänenspezifische Sprache für programmatische Access Control in Erynoa.
Sie ermöglicht deklarative und sichere Policies basierend auf Trust-Vektoren, Credentials und Kontext.

---

## Inhaltsverzeichnis

1. [Grundlagen](#grundlagen)
2. [Syntax](#syntax)
3. [Typen](#typen)
4. [Operatoren](#operatoren)
5. [Built-in Funktionen](#built-in-funktionen)
6. [Trust-Operationen](#trust-operationen)
7. [Policies](#policies)
8. [Beispiele](#beispiele)
9. [Gas-Kosten](#gas-kosten)

---

## Grundlagen

ECL-Policies werden zu Bytecode kompiliert und in der ECLVM ausgeführt.
Jede Ausführung ist:

- **Deterministisch**: Gleiches Input = Gleiches Output
- **Gas-begrenzt**: Schutz vor DoS-Attacken
- **Sandboxed**: Kein direkter Zugriff auf externe Ressourcen

### Hello World

```ecl
policy HelloWorld {
    // Erlaube wenn Trust-R über 0.5
    require trust.r > 0.5, "Insufficient reliability";
    return true;
}
```

---

## Syntax

### Kommentare

```ecl
// Einzeiliger Kommentar

/* Mehrzeiliger
   Kommentar */
```

### Variablen

```ecl
let threshold = 0.6;
let message = "Access denied";
let allowed = trust.r >= threshold;
```

### Konstanten (Policy-Level)

```ecl
const MIN_TRUST = 0.5;
const MAX_ATTEMPTS = 3;
```

### Require-Statements

```ecl
// Einfaches Require
require condition;

// Mit Fehlermeldung
require condition, "Error message";
```

### Kontrollfluss

```ecl
if condition {
    // then block
}

if condition {
    // then block
} else {
    // else block
}

// Ternärer Operator
let result = condition ? value_true : value_false;
```

---

## Typen

| Typ            | Beschreibung          | Beispiel                         |
| -------------- | --------------------- | -------------------------------- |
| `number`       | 64-bit Fließkommazahl | `42.0`, `-3.14`, `1e10`          |
| `bool`         | Wahrheitswert         | `true`, `false`                  |
| `string`       | UTF-8 Text            | `"Hello World"`                  |
| `did`          | Dezentrale Identität  | `did:erynoa:self:alice`          |
| `trust_vector` | 6D Trust-Vektor       | `[0.8, 0.7, 0.9, 0.6, 0.8, 0.7]` |
| `array`        | Liste von Werten      | `[1, 2, 3]`, `["a", "b"]`        |
| `null`         | Leerer Wert           | `null`                           |

### TrustVector6D

Der Trust-Vektor ist zentral für Erynoa. Er hat 6 Dimensionen:

| Dim | Name        | Beschreibung               |
| --- | ----------- | -------------------------- |
| R   | Reliability | Zuverlässigkeit über Zeit  |
| I   | Integrity   | Ehrlichkeit und Konsistenz |
| C   | Competence  | Fachliche Kompetenz        |
| P   | Prestige    | Soziale Reputation         |
| V   | Vigilance   | Sicherheitsbewusstsein     |
| Ω   | Omega       | Systemisches Vertrauen     |

Zugriff:

```ecl
trust.r   // Reliability
trust.i   // Integrity
trust.c   // Competence
trust.p   // Prestige
trust.v   // Vigilance
trust.omega  // Omega
```

---

## Operatoren

### Arithmetik

| Op  | Name           | Beispiel |
| --- | -------------- | -------- |
| `+` | Addition       | `a + b`  |
| `-` | Subtraktion    | `a - b`  |
| `*` | Multiplikation | `a * b`  |
| `/` | Division       | `a / b`  |
| `%` | Modulo         | `a % b`  |
| `-` | Negation       | `-a`     |

### Vergleich

| Op   | Name           | Beispiel |
| ---- | -------------- | -------- |
| `==` | Gleich         | `a == b` |
| `!=` | Ungleich       | `a != b` |
| `>`  | Größer         | `a > b`  |
| `>=` | Größer-Gleich  | `a >= b` |
| `<`  | Kleiner        | `a < b`  |
| `<=` | Kleiner-Gleich | `a <= b` |

### Logik

| Op     | Name  | Beispiel   |
| ------ | ----- | ---------- |
| `&&`   | Und   | `a && b`   |
| `\|\|` | Oder  | `a \|\| b` |
| `!`    | Nicht | `!a`       |

### Präzedenz (höchste zuerst)

1. `!`, `-` (unär)
2. `*`, `/`, `%`
3. `+`, `-`
4. `>`, `>=`, `<`, `<=`
5. `==`, `!=`
6. `&&`
7. `||`

---

## Built-in Funktionen

### Trust-Funktionen

#### `load_trust(did) → TrustVector`

Lädt den Trust-Vektor für eine DID.

```ecl
let tv = load_trust(requester);
require tv.r > 0.5;
```

#### `trust_norm(tv) → number`

Berechnet den gewichteten Durchschnitt eines Trust-Vektors.

```ecl
let overall = trust_norm(trust);
require overall >= 0.6;
```

#### `trust_combine(tv1, tv2) → TrustVector`

Kombiniert zwei Trust-Vektoren nach Κ5: `t₁ ⊕ t₂ = 1 - (1-t₁)(1-t₂)`

```ecl
let combined = trust_combine(self_trust, peer_trust);
```

#### `trust_above_threshold(tv, threshold) → bool`

Prüft ob alle Dimensionen über dem Schwellwert liegen.

```ecl
require trust_above_threshold(trust, 0.5), "Not all dimensions above 0.5";
```

#### `trust_distance(tv1, tv2) → number`

Euklidische Distanz zwischen zwei Trust-Vektoren (für Anomalie-Erkennung).

```ecl
let anomaly = trust_distance(current_trust, baseline_trust);
require anomaly < 0.3, "Trust deviation too high";
```

### Credential-Funktionen

#### `has_credential(did, schema) → bool`

Prüft ob eine DID ein bestimmtes Credential besitzt.

```ecl
require has_credential(requester, "email-verified");
require has_credential(requester, "kyc-level-2");
```

#### `resolve_did(did) → bool`

Prüft ob eine DID existiert und auflösbar ist.

```ecl
require resolve_did(target), "Unknown target DID";
```

### Balance-Funktionen

#### `get_balance(did) → number`

Holt den Kontostand einer DID.

```ecl
require get_balance(sender) >= amount, "Insufficient funds";
```

### Zeit-Funktionen

#### `get_timestamp() → number`

Aktueller Unix-Timestamp in Sekunden.

```ecl
let now = get_timestamp();
require now < expiry, "Offer expired";
```

#### `time_since(timestamp) → number`

Sekunden seit einem Timestamp.

```ecl
let age = time_since(last_activity);
require age < 3600, "Session expired"; // 1 Stunde
```

### Math-Funktionen

#### `abs(n) → number`

Absolutwert.

```ecl
let diff = abs(expected - actual);
```

#### `sqrt(n) → number`

Quadratwurzel.

```ecl
let distance = sqrt(dx*dx + dy*dy);
```

#### `floor(n) → number`, `ceil(n) → number`, `round(n) → number`

Runden.

```ecl
let whole = floor(amount / 100) * 100;
```

#### `min(a, b) → number`, `max(a, b) → number`

Minimum/Maximum.

```ecl
let bounded = min(max(value, 0), 100);
```

#### `clamp(value, min, max) → number`

Begrenzt Wert auf Intervall.

```ecl
let normalized = clamp(score, 0, 1);
```

#### `lerp(a, b, t) → number`

Lineare Interpolation: `a + t * (b - a)`

```ecl
let blended = lerp(old_value, new_value, 0.5);
```

#### `surprisal(p) → number`

Informationstheoretische Überraschung: `S(p) = -log₂(p)`

```ecl
// Wie "überraschend" ist dieses Verhalten?
let s = surprisal(probability);
require s < 5, "Behavior too unusual"; // < 32:1 odds
```

### String-Funktionen

#### `str_len(s) → number`

Länge eines Strings.

```ecl
require str_len(name) > 0, "Name required";
require str_len(name) <= 100, "Name too long";
```

#### `str_contains(haystack, needle) → bool`

Prüft ob String Substring enthält.

```ecl
require !str_contains(message, "spam");
```

#### `str_eq_ignore_case(a, b) → bool`

Case-insensitiver Vergleich.

```ecl
let is_admin = str_eq_ignore_case(role, "ADMIN");
```

### Array-Funktionen

#### `array_len(arr) → number`

Länge eines Arrays.

```ecl
require array_len(recipients) > 0;
require array_len(recipients) <= 100;
```

#### `array_get(arr, index) → value`

Element an Index (0-basiert).

```ecl
let first = array_get(items, 0);
```

#### `contains(value, arr) → bool`

Prüft ob Wert in Array enthalten.

```ecl
let allowed_roles = ["admin", "moderator"];
require contains(role, allowed_roles);
```

### Debug-Funktionen

#### `log(message)`

Gibt Debug-Nachricht aus (nur in Development).

```ecl
log("Processing request from " + requester);
```

---

## Trust-Operationen

### Dimension-Zugriff

```ecl
// Direkter Zugriff auf die 6 Dimensionen
trust.r      // Reliability [0, 1]
trust.i      // Integrity [0, 1]
trust.c      // Competence [0, 1]
trust.p      // Prestige [0, 1]
trust.v      // Vigilance [0, 1]
trust.omega  // Omega (systemisch) [0, 1]
```

### Trust-Formeln aus der World Formula

```ecl
// Κ5: Trust Kombination
// t₁ ⊕ t₂ = 1 - (1-t₁)(1-t₂)
let combined = trust_combine(trust1, trust2);

// Schwellwert-basierte Zugriffskontrolle
// τ(a, r) ≥ θ
require trust_norm(trust) >= threshold;

// Multi-dimensionale Prüfung
// ∀d ∈ D: τd(a, r) ≥ θd
require trust.r >= 0.7;
require trust.i >= 0.8;
require trust.c >= 0.5;
```

---

## Policies

### Struktur

```ecl
policy PolicyName {
    // Konstanten (optional)
    const THRESHOLD = 0.6;

    // Require-Statements
    require condition1, "Error 1";
    require condition2, "Error 2";

    // Lokale Variablen
    let computed = trust.r * 2;

    // Kontrollfluss
    if special_case {
        require extra_condition;
    }

    // Rückgabe
    return true;  // oder false
}
```

### Programme (mehrere Policies)

```ecl
// Konstanten auf Programmebene
const GLOBAL_THRESHOLD = 0.5;

policy Policy1 {
    require trust.r >= GLOBAL_THRESHOLD;
    return true;
}

policy Policy2 {
    require trust.i >= GLOBAL_THRESHOLD;
    return true;
}
```

---

## Beispiele

### 1. Basic Entry Policy

```ecl
policy BasicEntry {
    const MIN_TRUST = 0.3;

    // Newcomer mit sehr niedrigem Trust ablehnen
    require trust_norm(trust) >= MIN_TRUST,
        "Trust too low for entry";

    return true;
}
```

### 2. Verified Users Only

```ecl
policy VerifiedUsersOnly {
    const REQUIRED_SCHEMA = "email-verified";

    // Email muss verifiziert sein
    require has_credential(requester, REQUIRED_SCHEMA),
        "Email verification required";

    // Minimaler Trust
    require trust.i >= 0.5, "Integrity too low";

    return true;
}
```

### 3. Secure Transfer Gateway

```ecl
policy SecureTransferGateway {
    // Konfiguration
    const MIN_SENDER_TRUST = 0.6;
    const MIN_RECEIVER_TRUST = 0.4;
    const MAX_AMOUNT = 10000;
    const REQUIRE_KYC_ABOVE = 5000;

    // 1. Sender validieren
    let sender_trust = load_trust(sender);
    require trust_norm(sender_trust) >= MIN_SENDER_TRUST,
        "Sender trust insufficient";

    // 2. Empfänger validieren
    require resolve_did(receiver), "Unknown receiver";
    let receiver_trust = load_trust(receiver);
    require trust_norm(receiver_trust) >= MIN_RECEIVER_TRUST,
        "Receiver trust too low";

    // 3. Betrag prüfen
    require amount > 0, "Amount must be positive";
    require amount <= MAX_AMOUNT, "Amount exceeds limit";

    // 4. Balance prüfen
    require get_balance(sender) >= amount, "Insufficient funds";

    // 5. KYC für hohe Beträge
    if amount > REQUIRE_KYC_ABOVE {
        require has_credential(sender, "kyc-level-2"),
            "KYC required for large transfers";
    }

    // 6. Anomalie-Erkennung
    let baseline = load_trust(sender);  // Historischer Wert
    let deviation = trust_distance(sender_trust, baseline);
    require deviation < 0.3, "Unusual trust pattern detected";

    return true;
}
```

### 4. Time-Limited Access

```ecl
policy TimeLimitedAccess {
    const SESSION_DURATION = 3600;  // 1 Stunde
    const REFRESH_THRESHOLD = 300;  // 5 Minuten

    // Session noch gültig?
    let elapsed = time_since(session_start);
    require elapsed < SESSION_DURATION, "Session expired";

    // Refresh empfehlen?
    let should_refresh = elapsed > (SESSION_DURATION - REFRESH_THRESHOLD);
    if should_refresh {
        log("Session refresh recommended");
    }

    return true;
}
```

### 5. Role-Based Access Control

```ecl
policy RoleBasedAccess {
    const ADMIN_ROLES = ["admin", "super-admin"];
    const MOD_ROLES = ["moderator", "admin", "super-admin"];

    // Admin-Aktionen
    if action == "delete" || action == "ban" {
        require contains(role, ADMIN_ROLES), "Admin role required";
        require trust.i >= 0.9, "High integrity required for admin actions";
    }

    // Moderator-Aktionen
    if action == "edit" || action == "hide" {
        require contains(role, MOD_ROLES), "Moderator role required";
        require trust.c >= 0.7, "Competence required for moderation";
    }

    return true;
}
```

### 6. Multi-Sig Approval

```ecl
policy MultiSigApproval {
    const REQUIRED_APPROVALS = 2;
    const MIN_APPROVER_TRUST = 0.8;

    // Approvals zählen
    let valid_approvals = 0;

    // Jeden Approver prüfen
    let i = 0;
    let len = array_len(approvers);

    // Vereinfacht (echte Schleifen in v2)
    if len > 0 {
        let approver = array_get(approvers, 0);
        let tv = load_trust(approver);
        if trust_norm(tv) >= MIN_APPROVER_TRUST {
            valid_approvals = valid_approvals + 1;
        }
    }
    if len > 1 {
        let approver = array_get(approvers, 1);
        let tv = load_trust(approver);
        if trust_norm(tv) >= MIN_APPROVER_TRUST {
            valid_approvals = valid_approvals + 1;
        }
    }
    if len > 2 {
        let approver = array_get(approvers, 2);
        let tv = load_trust(approver);
        if trust_norm(tv) >= MIN_APPROVER_TRUST {
            valid_approvals = valid_approvals + 1;
        }
    }

    require valid_approvals >= REQUIRED_APPROVALS,
        "Insufficient approvals";

    return true;
}
```

---

## Gas-Kosten

Jede Operation verbraucht Gas. Das verhindert DoS-Attacken und garantiert deterministische Ausführung.

### Basis-Operationen

| Operation                    | Gas |
| ---------------------------- | --- |
| `push`, `pop`, `dup`         | 1   |
| `+`, `-`                     | 2   |
| `*`                          | 3   |
| `/`, `%`                     | 5   |
| Vergleiche (`==`, `>`, etc.) | 2   |
| `&&`, `\|\|`                 | 2   |
| `!`                          | 1   |

### Trust-Operationen

| Operation               | Gas |
| ----------------------- | --- |
| `trust.dim`             | 3   |
| `trust_norm`            | 10  |
| `trust_combine`         | 15  |
| `trust_create`          | 8   |
| `trust_above_threshold` | 10  |
| `trust_distance`        | 15  |

### Host-Calls (teuer)

| Operation        | Gas |
| ---------------- | --- |
| `load_trust`     | 100 |
| `has_credential` | 50  |
| `resolve_did`    | 50  |
| `get_balance`    | 50  |
| `get_timestamp`  | 5   |
| `log`            | 20  |

### Erweiterte Funktionen

| Operation                | Gas |
| ------------------------ | --- |
| `surprisal`              | 8   |
| `str_len`                | 3   |
| `str_contains`           | 8   |
| `sqrt`                   | 5   |
| `clamp`, `lerp`          | 4-5 |
| `array_len`, `array_get` | 2-3 |
| `contains`               | 10  |

### Gas-Limits

| Kontext         | Typisches Limit |
| --------------- | --------------- |
| Entry Policy    | 1.000           |
| Transfer Policy | 5.000           |
| Complex Policy  | 10.000          |
| Maximum         | 100.000         |

---

## Best Practices

### 1. Fail Fast

```ecl
// ✅ Günstige Checks zuerst
require amount > 0;
require amount <= MAX;
require get_balance(sender) >= amount;  // Teuer - am Ende

// ❌ Vermeiden
require get_balance(sender) >= amount;  // Teuer als erstes
require amount > 0;
```

### 2. Klare Fehlermeldungen

```ecl
// ✅ Spezifische Nachrichten
require trust.r >= 0.7, "Reliability must be at least 0.7";

// ❌ Generisch
require trust.r >= 0.7, "Error";
```

### 3. Konstanten verwenden

```ecl
// ✅ Wartbar
const MIN_TRUST = 0.6;
require trust.r >= MIN_TRUST;

// ❌ Magic Numbers
require trust.r >= 0.6;
```

### 4. Defensive Programming

```ecl
// ✅ Bounds checken
require array_len(items) > 0, "Empty array";
let first = array_get(items, 0);

// ❌ Unchecked Access
let first = array_get(items, 0);  // Kann crashen
```

---

## Anhang: Vollständige OpCode-Referenz

| OpCode              | Stack Effect              | Description           |
| ------------------- | ------------------------- | --------------------- |
| `PushConst(v)`      | `[] → [v]`                | Konstante pushen      |
| `Pop`               | `[a] → []`                | Top entfernen         |
| `Dup`               | `[a] → [a, a]`            | Top duplizieren       |
| `Swap`              | `[a, b] → [b, a]`         | Top zwei tauschen     |
| `Add`               | `[a, b] → [a+b]`          | Addition              |
| `Sub`               | `[a, b] → [a-b]`          | Subtraktion           |
| `Mul`               | `[a, b] → [a*b]`          | Multiplikation        |
| `Div`               | `[a, b] → [a/b]`          | Division              |
| `Mod`               | `[a, b] → [a%b]`          | Modulo                |
| `Neg`               | `[a] → [-a]`              | Negation              |
| `Eq`                | `[a, b] → [a==b]`         | Gleichheit            |
| `Neq`               | `[a, b] → [a!=b]`         | Ungleichheit          |
| `Gt`                | `[a, b] → [a>b]`          | Größer                |
| `Gte`               | `[a, b] → [a>=b]`         | Größer-Gleich         |
| `Lt`                | `[a, b] → [a<b]`          | Kleiner               |
| `Lte`               | `[a, b] → [a<=b]`         | Kleiner-Gleich        |
| `And`               | `[a, b] → [a&&b]`         | Logisches UND         |
| `Or`                | `[a, b] → [a\|\|b]`       | Logisches ODER        |
| `Not`               | `[a] → [!a]`              | Logisches NICHT       |
| `Jump(addr)`        | `[]`                      | Sprung                |
| `JumpIfFalse(addr)` | `[cond]`                  | Bedingter Sprung      |
| `Return`            | `[v] → return v`          | Rückgabe              |
| `TrustDim(d)`       | `[tv] → [tv[d]]`          | Dimension extrahieren |
| `TrustNorm`         | `[tv] → [norm]`           | Norm berechnen        |
| `TrustCombine`      | `[tv1, tv2] → [combined]` | Kombinieren           |
| `LoadTrust`         | `[did] → [tv]`            | Trust laden           |
| `HasCredential`     | `[did, schema] → [bool]`  | Credential prüfen     |
| `Require`           | `[cond, msg] → []`        | Assertion             |
| `Halt`              | `[v]`                     | Programm beenden      |

---

_ECL Reference v1.0 - © 2026 Erynoa Project_
