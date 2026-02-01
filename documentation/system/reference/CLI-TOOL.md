# Erynoa CLI Tool – Vollständige Referenz

> **Version:** 1.0.0
> **Datum:** Februar 2026
> **Status:** Production-Ready
> **Basis:** IPS-01-imp.md v1.2.0 + UNIFIED-DATA-MODEL.md v1.1.0

---

## Executive Summary

Das **Erynoa CLI Tool** (`ecl`) ist das primäre Kommandozeilen-Interface für die Interaktion mit dem **Erynoa Configuration Language (ECL)** System. Es implementiert die ECLVM-Schicht des Integrated Processing System (IPS) und ermöglicht:

- **Policy-Entwicklung**: Kompilieren, Testen und Ausführen von ECL-Policies
- **Deterministische Ausführung**: Gas-gemessene, stack-basierte VM gemäß IPS §II
- **Trust-Integration**: Zugriff auf 6D-Trust-Vektoren und Credential-Prüfungen
- **Debugging**: REPL-Umgebung mit Bytecode-Inspection und Variablen-Tracking

---

## I. Installation

### 1.1 Voraussetzungen

| Komponente | Mindestversion | Empfohlen |
| ---------- | -------------- | --------- |
| Rust       | 1.75.0         | 1.82.0+   |
| Cargo      | 1.75.0         | 1.82.0+   |

### 1.2 Build & Installation

```bash
# Aus dem Backend-Verzeichnis
cd backend

# Mit CLI-Feature kompilieren und installieren
cargo install --path . --features cli --bin ecl

# Oder nur lokal bauen
cargo build --release --features cli --bin ecl
```

### 1.3 Überprüfung

```bash
# Version anzeigen
ecl --version
# Ausgabe: ecl 0.1.0

# Hilfe anzeigen
ecl --help
```

---

## II. Kommandoübersicht

```
┌────────────────────────────────────────────────────────────────────────────────┐
│                              ECL CLI COMMANDS                                   │
├──────────┬─────────────────────────────────────────────────────────────────────┤
│ Command  │ Beschreibung                                                        │
├──────────┼─────────────────────────────────────────────────────────────────────┤
│ repl     │ Interaktive REPL-Umgebung für ECL-Ausdrücke                        │
│ eval     │ Einzelnen ECL-Ausdruck evaluieren                                   │
│ compile  │ ECL-Datei zu Bytecode kompilieren                                   │
│ run      │ ECL-Policy mit Kontext ausführen                                    │
│ check    │ Syntax-Check ohne Ausführung                                        │
│ fmt      │ ECL-Datei formatieren (Preview)                                     │
└──────────┴─────────────────────────────────────────────────────────────────────┘
```

---

## III. Kommando-Referenz

### 3.1 `ecl repl` – Interaktive REPL

Startet eine interaktive Read-Eval-Print-Loop für ECL-Ausdrücke.

#### Syntax

```bash
ecl repl [OPTIONS]
```

#### Optionen

| Option      | Kurzform | Beschreibung                                          |
| ----------- | -------- | ----------------------------------------------------- |
| `--verbose` | `-v`     | Aktiviert ausführliche Ausgabe (Gas-Metering, Timing) |

#### REPL-Interne Befehle

| Befehl             | Alias       | Beschreibung                   |
| ------------------ | ----------- | ------------------------------ |
| `quit`             | `exit`, `q` | REPL beenden                   |
| `help`             | `h`, `?`    | Hilfe anzeigen                 |
| `clear`            |             | Terminal leeren                |
| `history`          |             | Befehls-Historie anzeigen      |
| `vars`             |             | Definierte Variablen anzeigen  |
| `:type <expr>`     |             | Typ eines Ausdrucks anzeigen   |
| `:bytecode <expr>` |             | Kompilierten Bytecode anzeigen |
| `:load <file>`     |             | ECL-Datei laden und ausführen  |

#### Beispiel

```bash
$ ecl repl --verbose

ECL REPL v0.1.0
Type 'help' for commands, 'quit' to exit

ecl> 2 + 3 * 4
=> 14
   Gas used: 5

ecl> let x = trust.reliability
=> 0.5
   Gas used: 10

ecl> x > 0.3
=> true
   Gas used: 3

ecl> :type x
Number

ecl> :bytecode 1 + 1
PUSH 1
PUSH 1
ADD
```

#### IPS-Mapping

Die REPL implementiert die **Prozess-Monade ℳ** (IPS §II) mit:

- **Gas-Metering**: Jede Operation verbraucht Gas (Standard: 10.000)
- **Trust-Context**: Stub-Host mit konfigurierbaren Trust-Vektoren
- **Event-Emission**: Befehle werden als Events geloggt

---

### 3.2 `ecl eval` – Ausdruck Evaluieren

Evaluiert einen einzelnen ECL-Ausdruck und gibt das Ergebnis aus.

#### Syntax

```bash
ecl eval <EXPRESSION> [OPTIONS]
```

#### Argumente

| Argument       | Beschreibung                                            |
| -------------- | ------------------------------------------------------- |
| `<EXPRESSION>` | Der zu evaluierende ECL-Ausdruck (in Anführungszeichen) |

#### Optionen

| Option       | Kurzform | Beschreibung                               |
| ------------ | -------- | ------------------------------------------ |
| `--bytecode` | `-b`     | Zeigt kompilierten Bytecode vor Ausführung |

#### Beispiel

```bash
# Einfache Berechnung
$ ecl eval "2 + 2"
=> 4

# Mit Bytecode-Anzeige
$ ecl eval "trust.reliability >= 0.7" --bytecode
Bytecode:
  LOAD_TRUST
  PUSH 0
  INDEX
  PUSH 0.7
  GTE

=> true
```

---

### 3.3 `ecl compile` – Kompilieren

Kompiliert eine ECL-Quelldatei zu Bytecode.

#### Syntax

```bash
ecl compile <INPUT> [OPTIONS]
```

#### Argumente

| Argument  | Beschreibung                     |
| --------- | -------------------------------- |
| `<INPUT>` | Pfad zur ECL-Quelldatei (`.ecl`) |

#### Optionen

| Option            | Kurzform | Beschreibung                        |
| ----------------- | -------- | ----------------------------------- |
| `--output <FILE>` | `-o`     | Ausgabedatei für Bytecode (`.eclc`) |
| `--optimize`      | `-O`     | Bytecode-Optimierung aktivieren     |
| `--disasm`        | `-d`     | Disassembly ausgeben                |

#### Beispiel

```bash
# Kompilieren mit Optimierung
$ ecl compile policy.ecl -o policy.eclc -O
Compiling: policy.ecl
  Optimizing...
  Reduced: 45 → 32 instructions
Written: policy.eclc (256 bytes)
✓ Compilation successful

# Mit Disassembly
$ ecl compile policy.ecl --disasm
Compiling: policy.ecl

Disassembly:
  0000: PUSH 0.7
  0001: LOAD_TRUST
  0002: PUSH 0
  0003: INDEX
  0004: GTE
  ...

✓ Compilation successful
```

#### IPS-Mapping: Kompilierungspfad

```
φ_compile : AST → Bytecode Β

Entspricht dem IPS-Fundamentaldiagramm (§I.2):
  Intent → φ_parse → AST → φ_compile → Bytecode
```

---

### 3.4 `ecl run` – Policy Ausführen

Führt eine kompilierte oder Quell-ECL-Datei mit Kontext aus.

#### Syntax

```bash
ecl run <INPUT> [OPTIONS]
```

#### Argumente

| Argument  | Beschreibung                             |
| --------- | ---------------------------------------- |
| `<INPUT>` | Pfad zur ECL-Datei (`.ecl` oder `.eclc`) |

#### Optionen

| Option             | Kurzform | Beschreibung               | Standard |
| ------------------ | -------- | -------------------------- | -------- |
| `--context <FILE>` | `-c`     | JSON-Kontext-Datei         | -        |
| `--gas-limit <N>`  | `-g`     | Maximales Gas              | 10.000   |
| `--trace`          | `-t`     | Execution-Trace aktivieren | false    |

#### Kontext-Format (JSON)

Das Kontext-JSON implementiert den **Trust-Context** aus UDM §II:

```json
{
  "trust": [0.8, 0.7, 0.9, 0.6, 0.8, 0.7],
  "balance": 1000,
  "credentials": ["kyc-verified", "premium-user"]
}
```

| Feld          | Typ        | Beschreibung                       | UDM-Referenz |
| ------------- | ---------- | ---------------------------------- | ------------ |
| `trust`       | `[f64; 6]` | 6D-Trust-Vektor [R, I, C, P, V, Ω] | §II.1        |
| `balance`     | `u64`      | Mana/Token-Balance                 | §III.2       |
| `credentials` | `string[]` | Attestation-Schemas                | §I.4         |

#### Trust-Vektor Dimensionen

| Index | Symbol | Dimension   | Beschreibung                           |
| ----- | ------ | ----------- | -------------------------------------- |
| 0     | R      | Reliability | Zuverlässigkeit (Promises eingehalten) |
| 1     | I      | Integrity   | Integrität (Datenqualität)             |
| 2     | C      | Competence  | Kompetenz (Fähigkeiten)                |
| 3     | P      | Performance | Leistung (Geschwindigkeit, Effizienz)  |
| 4     | V      | Values      | Werte-Alignment                        |
| 5     | Ω      | Omega       | Aggregierter Gesamtwert                |

#### Beispiel

```bash
# Mit Kontext-Datei
$ ecl run access-policy.ecl -c user-context.json
Running: access-policy.ecl
  Gas limit: 10000

Result: true
  Gas: 245 / 10000

# Mit erhöhtem Gas-Limit
$ ecl run complex-policy.ecl -g 100000 --trace
Running: complex-policy.ecl
  Gas limit: 100000

[TRACE] PUSH 0.7
[TRACE] LOAD_TRUST
[TRACE] INDEX (R)
...

Result: {"access": "granted", "level": 3}
  Gas: 1234 / 100000
```

#### IPS-Mapping: Ausführungspfad

```
φ_exec : Program × State × Gas → (Value, State', Gas')

Die Ausführung folgt dem IPS-Fundamentaldiagramm:
  Bytecode Β → ECLVM Runtime → Result + Events
```

---

### 3.5 `ecl check` – Syntax-Prüfung

Prüft die Syntax einer ECL-Datei ohne Ausführung.

#### Syntax

```bash
ecl check <INPUT>
```

#### Argumente

| Argument  | Beschreibung            |
| --------- | ----------------------- |
| `<INPUT>` | Pfad zur ECL-Quelldatei |

#### Beispiel

```bash
# Erfolgreiche Prüfung
$ ecl check valid-policy.ecl
Checking: valid-policy.ecl
✓ No errors found

# Fehlerhafte Datei
$ ecl check broken-policy.ecl
Checking: broken-policy.ecl
  ✗ Unexpected token 'if' (line 5)
  ✗ Missing closing brace (line 12)
Error: Errors found
```

---

### 3.6 `ecl fmt` – Formatieren

Formatiert eine ECL-Datei (Preview-Feature).

#### Syntax

```bash
ecl fmt <INPUT> [OPTIONS]
```

#### Argumente

| Argument  | Beschreibung            |
| --------- | ----------------------- |
| `<INPUT>` | Pfad zur ECL-Quelldatei |

#### Optionen

| Option    | Kurzform | Beschreibung               |
| --------- | -------- | -------------------------- |
| `--write` | `-w`     | Datei direkt überschreiben |

#### Hinweis

Dieses Feature ist derzeit im Preview-Status. Vollständiges Pretty-Printing wird in einer zukünftigen Version implementiert.

---

## IV. ECL Sprachübersicht

### 4.1 Datentypen (UDM-konform)

| Typ            | Syntax           | UDM-Referenz | Beispiel               |
| -------------- | ---------------- | ------------ | ---------------------- |
| `null`         | `null`           | -            | `null`                 |
| `bool`         | `true`, `false`  | -            | `true`                 |
| `number`       | Dezimal          | -            | `42`, `3.14`           |
| `string`       | `"..."`          | -            | `"hello"`              |
| `did`          | `did:erynoa:...` | §I.3         | `did:erynoa:abc123`    |
| `trust_vector` | `[R,I,C,P,V,Ω]`  | §II.1        | `[0.8, 0.7, 0.9, ...]` |
| `array`        | `[...]`          | -            | `[1, 2, 3]`            |

### 4.2 Operatoren

```ecl
// Arithmetik
+ - * / %

// Vergleich
== != < <= > >=

// Logik
&& || !

// Trust-Zugriff
trust.reliability    // R-Dimension
trust.integrity      // I-Dimension
trust.competence     // C-Dimension
trust.performance    // P-Dimension
trust.values         // V-Dimension
trust.omega          // Ω (aggregiert)
```

### 4.3 Kontrollstrukturen

```ecl
// Bedingte Ausdrücke
if condition then value_true else value_false

// Let-Bindings
let x = expression

// Funktionsaufrufe
has_credential("kyc-verified")
get_balance()
```

### 4.4 Built-in Funktionen (Stdlib)

| Funktion                 | Beschreibung         | IPS-Referenz     |
| ------------------------ | -------------------- | ---------------- |
| `has_credential(schema)` | Prüft Attestation    | Κ6 (Attestation) |
| `get_balance()`          | Mana-Balance abrufen | §III.2           |
| `trust_gate(threshold)`  | Trust-Check          | §IV.2            |
| `emit_event(type, data)` | Event emittieren     | §I.2             |

---

## V. ECLVM Architektur

### 5.1 Stack-basierte VM

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                              ECLVM ARCHITECTURE                                 ║
╠════════════════════════════════════════════════════════════════════════════════╣
║                                                                                ║
║   ┌─────────────────────────────────────────────────────────────────────────┐  ║
║   │                           Host Interface                                │  ║
║   │  (Trust, Balance, Credentials, Storage, Events)                        │  ║
║   └─────────────────────────────────────────────────────────────────────────┘  ║
║                                    ▲                                           ║
║                                    │                                           ║
║   ┌─────────────────────────────────────────────────────────────────────────┐  ║
║   │                              ECLVM Core                                 │  ║
║   ├─────────────────────────────────────────────────────────────────────────┤  ║
║   │                                                                         │  ║
║   │   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐        │  ║
║   │   │  Stack   │    │  Locals  │    │   Gas    │    │ Program  │        │  ║
║   │   │ (Values) │    │  (Vars)  │    │ Counter  │    │ Counter  │        │  ║
║   │   └──────────┘    └──────────┘    └──────────┘    └──────────┘        │  ║
║   │                                                                         │  ║
║   └─────────────────────────────────────────────────────────────────────────┘  ║
║                                    ▲                                           ║
║                                    │                                           ║
║   ┌─────────────────────────────────────────────────────────────────────────┐  ║
║   │                           Bytecode Β                                    │  ║
║   │  [PUSH] [LOAD] [ADD] [CALL] [JMP] [RET] ...                            │  ║
║   └─────────────────────────────────────────────────────────────────────────┘  ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### 5.2 Instruction Set

| Opcode       | Argumente | Stack-Effekt    | Gas | Beschreibung       |
| ------------ | --------- | --------------- | --- | ------------------ |
| `PUSH`       | value     | → value         | 1   | Wert auf Stack     |
| `POP`        | -         | value →         | 1   | Wert vom Stack     |
| `ADD`        | -         | a, b → (a+b)    | 1   | Addition           |
| `SUB`        | -         | a, b → (a-b)    | 1   | Subtraktion        |
| `MUL`        | -         | a, b → (a\*b)   | 1   | Multiplikation     |
| `DIV`        | -         | a, b → (a/b)    | 1   | Division           |
| `EQ`         | -         | a, b → (a==b)   | 1   | Gleichheit         |
| `LT`         | -         | a, b → (a<b)    | 1   | Kleiner            |
| `GT`         | -         | a, b → (a>b)    | 1   | Größer             |
| `AND`        | -         | a, b → (a&&b)   | 1   | Logisches UND      |
| `OR`         | -         | a, b → (a\|\|b) | 1   | Logisches ODER     |
| `NOT`        | -         | a → (!a)        | 1   | Negation           |
| `LOAD`       | index     | → value         | 2   | Local laden        |
| `STORE`      | index     | value →         | 2   | Local speichern    |
| `LOAD_TRUST` | -         | → trust_vec     | 5   | Trust-Vektor laden |
| `INDEX`      | -         | arr, i → arr[i] | 2   | Array-Index        |
| `CALL`       | func_id   | args → result   | 10+ | Funktion aufrufen  |
| `JMP`        | offset    | -               | 1   | Unbedingter Sprung |
| `JZ`         | offset    | cond →          | 1   | Sprung wenn false  |
| `RET`        | -         | -               | 1   | Rückgabe           |

### 5.3 Gas-Kosten (Kosten-Algebra 𝒦)

Die Gas-Kosten entsprechen der **Kosten-Algebra 𝒦** aus IPS §III:

```
𝒦 : Instruction → ℕ⁺

Mit:
  𝒦(simple_op)  = 1      (ADD, SUB, MUL, ...)
  𝒦(memory_op)  = 2      (LOAD, STORE, INDEX)
  𝒦(trust_op)   = 5      (LOAD_TRUST)
  𝒦(host_call)  = 10+    (CALL, variabel)
  𝒦(crypto_op)  = 100+   (VERIFY, SIGN)
```

---

## VI. Integration mit IPS & UDM

### 6.1 Adjunktion Core ↔ ECLVM

Die CLI nutzt die **Adjunktion F ⊣ G** (IPS §VII.2) für verlustfreie Übersetzung:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   Core Domain (UDM)              ECLVM Domain                              │
│                                                                             │
│   UniversalId    ──────F──────▶  Value::DID                                │
│   Trust6D        ──────F──────▶  Value::TrustVector                        │
│   Event          ──────F──────▶  Value::Array (serialized)                 │
│                                                                             │
│   Value::DID     ◀─────G──────   UniversalId                               │
│   Value::Trust   ◀─────G──────   Trust6D                                   │
│   Value::Array   ◀─────G──────   Event                                     │
│                                                                             │
│   Invariante: G(F(x)) ≅ x  (Zig-Zag Identity)                              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.2 Event-DAG Integration

Jede Policy-Ausführung kann Events emittieren, die in den **Event-DAG** (UDM §III) eingefügt werden:

```ecl
// In ECL-Policy
if access_granted then
  emit_event("access", { resource: "document-123", level: 3 })
```

### 6.3 Trust-Gate Pattern

Das **Trust-Gate** (IPS §IV.2) schützt sensible Operationen:

```ecl
// Nur bei ausreichendem Trust ausführen
if trust.omega >= 0.7 then
  sensitive_operation()
else
  error("Insufficient trust")
```

---

## VII. Fehlerbehandlung

### 7.1 Exit-Codes

| Code | Bedeutung                                            |
| ---- | ---------------------------------------------------- |
| 0    | Erfolg                                               |
| 1    | Allgemeiner Fehler                                   |
| 2    | Syntax-/Parse-Fehler                                 |
| 3    | Kompilierungsfehler                                  |
| 4    | Laufzeitfehler (Gas exhausted, Stack overflow, etc.) |
| 5    | Datei nicht gefunden                                 |

### 7.2 Error-Hierarchie (UDM §0.2)

```
ExecutionError
├── GasExhausted        # Gas-Limit erreicht
├── StackOverflow       # Stack-Limit überschritten
├── PolicyViolation     # Policy-Regel verletzt
├── SchemaViolation     # Daten entsprechen nicht Schema
├── AccessDenied        # Unzureichende Berechtigung
└── TrustGateBlocked    # Trust-Schwelle nicht erreicht
```

---

## VIII. Beispiele

### 8.1 Access-Control Policy

```ecl
// access-policy.ecl
// Zugriffskontrolle basierend auf Trust und Credentials

let min_trust = 0.6
let has_kyc = has_credential("kyc-verified")
let trust_ok = trust.omega >= min_trust

if has_kyc && trust_ok then
  { access: "granted", level: 2 }
else if trust_ok then
  { access: "granted", level: 1 }
else
  { access: "denied", reason: "insufficient_trust" }
```

### 8.2 Rate-Limiting Policy

```ecl
// rate-limit.ecl
// Kostenbasiertes Rate-Limiting

let balance = get_balance()
let cost = 10  // Mana pro Request

if balance >= cost then
  { allowed: true, new_balance: balance - cost }
else
  { allowed: false, reason: "insufficient_mana" }
```

### 8.3 Multi-Factor Trust Check

```ecl
// trust-gate.ecl
// Mehrfaktor Trust-Prüfung

let r = trust.reliability
let i = trust.integrity
let c = trust.competence

// Gewichtete Bewertung
let score = r * 0.3 + i * 0.4 + c * 0.3

if score >= 0.75 then
  { status: "high_trust", score: score }
else if score >= 0.5 then
  { status: "medium_trust", score: score }
else
  { status: "low_trust", score: score }
```

---

## IX. Konfiguration

### 9.1 Umgebungsvariablen

| Variable         | Beschreibung                                | Standard |
| ---------------- | ------------------------------------------- | -------- |
| `ECL_GAS_LIMIT`  | Standard Gas-Limit                          | 10000    |
| `ECL_STACK_SIZE` | Stack-Größe                                 | 1024     |
| `ECL_LOG_LEVEL`  | Log-Level (trace, debug, info, warn, error) | info     |

### 9.2 Config-Datei

Die CLI sucht nach `~/.ecl/config.toml`:

```toml
[execution]
gas_limit = 10000
stack_size = 1024
trace_enabled = false

[repl]
history_file = "~/.ecl/history"
prompt_style = "arrow"  # "arrow" | "minimal" | "verbose"

[output]
color = true
format = "pretty"  # "pretty" | "json" | "minimal"
```

---

## X. Debugging & Troubleshooting

### 10.1 Verbose-Modus

```bash
# Maximale Ausgabe
ecl repl --verbose

# Oder per Umgebungsvariable
ECL_LOG_LEVEL=trace ecl run policy.ecl
```

### 10.2 Bytecode-Inspection

```bash
# Bytecode vor Ausführung anzeigen
ecl compile policy.ecl --disasm

# In REPL
ecl> :bytecode 1 + 2 * 3
```

### 10.3 Häufige Probleme

| Problem            | Ursache             | Lösung                   |
| ------------------ | ------------------- | ------------------------ |
| `GasExhausted`     | Komplexe Berechnung | `--gas-limit` erhöhen    |
| `StackOverflow`    | Tiefe Rekursion     | Iterativ umschreiben     |
| `TrustGateBlocked` | Trust zu niedrig    | Context-Trust anpassen   |
| `Parse error`      | Syntax-Fehler       | `ecl check` zur Diagnose |

---

## XI. Weitere Ressourcen

### 11.1 Dokumentation

| Dokument                                                          | Beschreibung                |
| ----------------------------------------------------------------- | --------------------------- |
| [IPS-01-imp.md](../development/IPS-01-imp.md)                     | Mathematisches Logik-Modell |
| [UNIFIED-DATA-MODEL.md](../development/UNIFIED-DATA-MODEL.md)     | UDM Spezifikation           |
| [BACKEND-ARCHITECTURE.md](./BACKEND-ARCHITECTURE.md)              | Backend-Architektur         |
| [IPS-UDM-GAP-ANALYSIS.md](../development/IPS-UDM-GAP-ANALYSIS.md) | Implementierungs-Status     |

### 11.2 Quellcode-Referenz

| Datei                    | Beschreibung       |
| ------------------------ | ------------------ |
| `src/bin/ecl.rs`         | CLI Entry Point    |
| `src/eclvm/cli.rs`       | CLI Implementation |
| `src/eclvm/parser.rs`    | ECL Parser         |
| `src/eclvm/compiler.rs`  | Bytecode Compiler  |
| `src/eclvm/runtime/`     | VM Runtime         |
| `src/eclvm/optimizer.rs` | Bytecode Optimizer |
| `src/eclvm/stdlib.rs`    | Standard Library   |

---

## XII. Axiom-Mapping

Das ECL CLI implementiert folgende Erynoa-Axiome (Κ1-Κ28):

| Axiom                | Implementierung             | CLI-Kommando     |
| -------------------- | --------------------------- | ---------------- |
| Κ5 (Trust6D)         | `LOAD_TRUST` Instruction    | `run`, `repl`    |
| Κ6 (Attestation)     | `has_credential()` Funktion | `run`, `repl`    |
| Κ10 (DID-Auth)       | DID-Werte in Context        | `run`            |
| Κ11 (Event-DAG)      | `emit_event()` Funktion     | `run`            |
| Κ15 (Policy)         | ECL Policy-Sprache          | `compile`, `run` |
| Κ16 (Access-Control) | Trust-Gate Pattern          | `run`            |
| Κ17 (Gas-Metering)   | Gas-Counter                 | alle             |
| Κ18 (Determinismus)  | Stack-basierte VM           | alle             |

---

## Changelog

### v1.0.0 (Februar 2026)

- Initial Release basierend auf IPS v1.2.0 und UDM v1.1.0
- Vollständige CLI mit 6 Kommandos
- REPL mit interaktiven Features
- Gas-Metering und Trust-Integration
- Dokumentation mit IPS/UDM-Mapping

---

_Erstellt: Februar 2026 | Basis: IPS v1.2.0 + UDM v1.1.0_
