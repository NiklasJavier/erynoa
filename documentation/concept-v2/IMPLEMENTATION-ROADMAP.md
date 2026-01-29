# ERY Implementation Roadmap

> **Typ:** Entwicklungs-Roadmap
> **Methodik:** Vom Abstrakten zum Konkreten
> **Status:** Aktiv

---

## Übersicht: Die drei Phasen

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                         │
│   IMPLEMENTIERUNGS-STRATEGIE                                                           │
│                                                                                         │
│   ┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐               │
│   │                 │      │                 │      │                 │               │
│   │   PHASE 1       │ ───▶ │   PHASE 2       │ ───▶ │   PHASE 3       │               │
│   │   Local Engine  │      │   Simulation    │      │   Networking    │               │
│   │                 │      │                 │      │                 │               │
│   │   "Mathemat.    │      │   "Kybernet.    │      │   "Schnitt-     │               │
│   │    Kern"        │      │    Welt"        │      │    stelle"      │               │
│   │                 │      │                 │      │                 │               │
│   └─────────────────┘      └─────────────────┘      └─────────────────┘               │
│                                                                                         │
│   Formale Logik ───▶ Emergenz-Prüfung ───▶ Praktische Anwendung                        │
│                                                                                         │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

# Phase 1: Local Engine (Der „Mathematische Kern")

> **Fokus:** Erynoa als reine Zustandsmaschine (State Machine)
> **Crates:** `ery-core`, `ery-logic`
> **Ziel:** `cargo test` beweist logische Konsistenz

## TODO: Projekt-Setup

- [ ] Cargo Workspace erstellen mit `ery-core` und `ery-logic` Crates
- [ ] Abhängigkeiten definieren:
  - `serde` + `serde_json` für Serialisierung
  - `thiserror` für Error-Handling
  - `petgraph` für DAG-Struktur (optional)

## TODO: ery-core Implementierung

### Identitäts-Modul (`identity.rs`)

- [ ] `Did` Struct implementieren (Parsing, Validierung)
- [ ] `Namespace` Enum (person, org, device, agent, circle)
- [ ] `Entity` Struct mit Status und Parent-Referenz
- [ ] `EntityStatus` Enum (Active, Suspended, Revoked)
- [ ] `identity_factor(entity) -> f64` (𝕀(s) ∈ {0, 1})
- [ ] `exists(entity) -> bool` (⟨s⟩)

### Trust-Modul (`trust.rs`)

- [ ] `TrustVector` Struct mit (R, I, C, P)
- [ ] Konstanten: `MIN_TRUST = 0.3`, `MAX_TRUST = 1.0`, `INITIAL_TRUST = 0.5`
- [ ] `ASYMMETRY_FACTOR = 1.5` (Δ⁻ = 1.5 · Δ⁺)
- [ ] `aggregate() -> f64` Methode
- [ ] `apply_update(dimension, delta, is_positive)` Methode
- [ ] `apply_decay(lambda)` Methode
- [ ] `constrain_to(parent_trust)` für Delegation (A9)

### Kausalitäts-Modul (`causality.rs`)

- [ ] `Event` Struct mit Hash, Parents, Actor, Timestamp
- [ ] `EventType` Enum (Genesis, Attestation, Transaction, ...)
- [ ] `FinalityLevel` Enum (Nascent, Validated, Witnessed, Anchored, Eternal)
- [ ] `CausalHistory` Struct (DAG-basiert)
  - [ ] In-Memory: `HashMap<Hash, Event>` + `HashMap<Did, Vec<Hash>>`
  - [ ] Optional: `petgraph::DiGraph` für komplexe Queries
- [ ] `depth(did) -> usize` (|ℂ(s)|)
- [ ] `precedes(earlier, later) -> bool` (e ⊲ e')
- [ ] `is_witnessed(hash) -> bool` (⟦e⟧)
- [ ] `is_final(hash) -> bool` (∎e)
- [ ] Zyklen-Prüfung implementieren (A11: ¬(e ⊲ e))

### Weltformel-Modul (`formula.rs`)

- [ ] `sigmoid(x) -> f64` implementieren
- [ ] `attention_score(trust, causal_depth) -> f64`
- [ ] `entity_contribution(entity, trust, causal_depth) -> f64`
- [ ] `WorldFormula::compute(state) -> f64` (𝔼 = Σ 𝕀(s) · σ(𝕋(s) · ln|ℂ(s)|))
- [ ] `WorldFormula::compute_delta(old_state, new_state) -> f64`

### Zustandsmodul (`state.rs`)

- [ ] `SystemState` Struct:
  ```rust
  struct SystemState {
      entities: HashMap<Did, Entity>,
      trust: HashMap<Did, TrustVector>,
      history: CausalHistory,
      system_value: f64,  // 𝔼
  }
  ```
- [ ] CRUD-Methoden für Entities
- [ ] Trust-Lookup und Update

## TODO: ery-logic Implementierung

### Invarianten-Modul (`invariants.rs`)

- [ ] `Invariant` Trait definieren:
  ```rust
  trait Invariant {
      fn name(&self) -> &'static str;
      fn check(&self, state: &SystemState) -> Result<(), InvariantViolation>;
  }
  ```
- [ ] Implementiere alle Invarianten:
  - [ ] `IdentityUniqueness` (A1)
  - [ ] `IdentityPermanence` (A2)
  - [ ] `DelegationExistence` (A3)
  - [ ] `DelegationAcyclicity` (A4)
  - [ ] `TrustBoundedness` (A5: 0 ≤ 𝕋 ≤ 1)
  - [ ] `TrustFloor` (A6: 𝕋 ≥ 0.3)
  - [ ] `DelegationTrustLimit` (A9: 𝕋(child) ≤ 𝕋(parent))
  - [ ] `CausalDagStructure` (A11-A13)
  - [ ] `WorldFormulaCorrectness` (𝔼 korrekt berechnet)
- [ ] `InvariantChecker` Struct:
  ```rust
  struct InvariantChecker {
      invariants: Vec<Box<dyn Invariant>>,
  }
  impl InvariantChecker {
      fn check_all(&self, state: &SystemState) -> Vec<InvariantViolation>;
  }
  ```

### Validierungs-Modul (`validation.rs`)

- [ ] 6-Schichten-Validierung implementieren:
  - [ ] Layer 1: Syntax-Validierung
  - [ ] Layer 2: Identitäts-Validierung
  - [ ] Layer 3: Kausalitäts-Validierung
  - [ ] Layer 4: Trust-Validierung (A23)
  - [ ] Layer 5: Realm-Validierung (A19, A20)
  - [ ] Layer 6: Ressourcen-Validierung
- [ ] `Validator::validate(process, state) -> Result<(), ValidationError>`

### Prozess-Modul (`process.rs`)

- [ ] `Process` Enum definieren:
  ```rust
  enum Process {
      CreateIdentity { ... },
      DelegateIdentity { ... },
      Attest { ... },
      Transfer { ... },
      UpdateTrust { ... },
      // ...
  }
  ```
- [ ] Für jeden Prozess: `apply(state) -> Result<Vec<Event>, ProcessError>`

### Transitions-Modul (`transition.rs`)

- [ ] `TransitionEngine` Struct:
  ```rust
  struct TransitionEngine {
      validator: Validator,
      invariant_checker: InvariantChecker,
  }
  impl TransitionEngine {
      fn apply(&self, state: &mut SystemState, process: Process) 
          -> Result<TransitionResult, TransitionError>;
  }
  ```
- [ ] Workflow:
  1. `validator.validate(process, state)?`
  2. `process.apply(state)?`
  3. `invariant_checker.check_all(state)?`
  4. `WorldFormula::compute(state)` → Update `state.system_value`

## TODO: Unit Tests (Test-Driven Development)

### Identitäts-Tests

- [ ] Test: Identität erstellen → Entity existiert
- [ ] Test: Identität delegieren → Parent-Child-Beziehung korrekt
- [ ] Test: Delegation-Zyklus verhindern → Fehler
- [ ] Test: Revoke → Status = Revoked, aber History bleibt

### Trust-Tests

- [ ] Test: Initial Trust = 0.5
- [ ] Test: Positive Attestation → Trust steigt
- [ ] Test: Negative Attestation → Trust sinkt (1.5× schneller)
- [ ] Test: Trust kann nicht unter 0.3 sinken (Floor)
- [ ] Test: Trust kann nicht über 1.0 steigen (Ceiling)
- [ ] Test: Child-Trust ≤ Parent-Trust

### Kausalitäts-Tests

- [ ] Test: Event mit Parents → korrekt im DAG
- [ ] Test: `precedes(a, b)` korrekt
- [ ] Test: Zyklus-Erkennung funktioniert
- [ ] Test: Witnessing erhöht Finality-Level

### Weltformel-Tests

- [ ] Test: Leerer State → 𝔼 = 0
- [ ] Test: Eine Entity → 𝔼 = σ(0.5 × ln(1)) ≈ 0.5
- [ ] Test: Attestation → 𝔼 steigt
- [ ] Test: Revoke → 𝔼 sinkt
- [ ] Test: Mehr Entities mit hohem Trust → höheres 𝔼

### Invarianten-Tests

- [ ] Test: `InvariantChecker::check_all()` nach jeder Aktion aufrufen
- [ ] Test: Bewusst invaliden State erzeugen → Invariante schlägt fehl
- [ ] Test: Alle 25 Axiome als Unit Tests abbilden

---

# Phase 2: Simulation (Die „Kybernetische Welt")

> **Fokus:** Beweis der Stabilität unter Last und Dynamik
> **Artefakt:** `examples/simulation.rs`
> **Ziel:** System bricht nicht zusammen bei vielen gleichzeitigen Agenten

## TODO: Simulator-Setup

- [ ] `examples/simulation.rs` erstellen
- [ ] Tokio als Runtime für parallele Agenten
- [ ] Konfigurierbare Parameter:
  ```rust
  struct SimulationConfig {
      num_agents: usize,        // z.B. 100
      num_ticks: usize,         // z.B. 10_000
      actions_per_tick: usize,  // z.B. 10
      seed: u64,                // Reproduzierbarkeit
  }
  ```

## TODO: Agent-Generator

- [ ] `generate_agents(n) -> Vec<Did>` implementieren
- [ ] Verschiedene Agent-Typen:
  - [ ] Honest Agents (normale Transaktionen)
  - [ ] High-Activity Agents (viele Aktionen)
  - [ ] Passive Agents (wenige Aktionen)

## TODO: Aktions-Generator

- [ ] Zufällige Aktionen generieren:
  - [ ] `CreateIdentity`
  - [ ] `DelegateIdentity`
  - [ ] `PositiveAttestation`
  - [ ] `NegativeAttestation`
  - [ ] `Transfer`
  - [ ] `CreateRealm`
  - [ ] `JoinRealm`
- [ ] Gewichtete Wahrscheinlichkeiten für realistische Verteilung

## TODO: Simulator-Loop

```rust
fn run_simulation(config: SimulationConfig) {
    let mut state = SystemState::new();
    let engine = TransitionEngine::new();
    let mut rng = StdRng::seed_from_u64(config.seed);
    
    // Agents erstellen
    let agents = generate_agents(config.num_agents, &mut state, &engine);
    
    // Simulation
    for tick in 0..config.num_ticks {
        for _ in 0..config.actions_per_tick {
            let action = generate_random_action(&agents, &mut rng);
            let result = engine.apply(&mut state, action);
            
            // Logging
            log_action(tick, &action, &result, state.system_value);
        }
        
        // Tick-Statistik
        log_tick_stats(tick, &state);
    }
}
```

## TODO: Metriken & Visualisierung

- [ ] CSV-Export implementieren:
  ```csv
  tick,system_value,num_entities,avg_trust,num_events
  0,0.0,0,0.0,0
  1,0.52,5,0.5,5
  ...
  ```
- [ ] Metriken pro Tick:
  - [ ] 𝔼 (Systemwert)
  - [ ] Anzahl Entities
  - [ ] Durchschnittlicher Trust
  - [ ] Anzahl Events
  - [ ] Anzahl Invarianten-Checks (erfolgreich/fehlgeschlagen)
  - [ ] Transaktionen pro Sekunde
- [ ] Optional: Live-Streaming zu Svelte-Dashboard

## TODO: Angriffs-Szenarien (Stress-Tests)

### Sybil-Attacke

- [ ] Ein Agent erstellt viele Sub-Identitäten
- [ ] Erwartung: Trust verteilt sich, kein unfairer Vorteil
- [ ] Prüfung: Invarianten halten, 𝔼 bleibt stabil

### Double-Spending

- [ ] Agent versucht, denselben Wert zweimal zu transferieren
- [ ] Erwartung: Zweiter Transfer wird abgelehnt
- [ ] Prüfung: `TransitionEngine` blockt, Invarianten intakt

### Trust-Manipulation

- [ ] Agents versuchen, sich gegenseitig hochzuvoten (Collusion)
- [ ] Erwartung: Asymmetrie-Faktor und Decay begrenzen Manipulation
- [ ] Prüfung: Trust-Wachstum bleibt logarithmisch

### Kausalitäts-Angriff

- [ ] Agent versucht, auf nicht-existente Events zu referenzieren
- [ ] Erwartung: Validierung schlägt fehl
- [ ] Prüfung: DAG-Struktur bleibt konsistent

## TODO: Ergebnis-Dokumentation

- [ ] Graphen erstellen:
  - [ ] „Entwicklung des Systemwerts über 10.000 Transaktionen"
  - [ ] „Trust-Verteilung nach 1h Simulation"
  - [ ] „Transaktionen pro Sekunde vs. Agenten-Anzahl"
- [ ] Statistiken für Bachelorarbeit:
  - [ ] Durchschnittliche Antwortzeit
  - [ ] Maximale Agenten bevor Performance degradiert
  - [ ] Prozent abgewehrter Angriffe

---

# Phase 3: Networking (Die „Schnittstelle zur Außenwelt")

> **Fokus:** API-Layer und Knoten-Binary
> **Crates:** `ery-api`, `bins/ery-node`, `bins/ery-cli`
> **Voraussetzung:** Phase 1 & 2 sind stabil

## TODO: API-Layer (`ery-api`)

### gRPC-Services definieren

- [ ] Proto-Dateien erweitern/vervollständigen:
  - [ ] `identity.proto` - Identity-Management
  - [ ] `trust.proto` - Trust-Operationen
  - [ ] `transaction.proto` - Transaktionen
  - [ ] `query.proto` - State-Queries
- [ ] `connect-rust` oder `tonic` als gRPC-Framework

### Service-Implementierung

- [ ] `IdentityService`:
  ```rust
  impl IdentityService for EryServer {
      async fn create_identity(&self, req: CreateIdentityRequest) 
          -> Result<CreateIdentityResponse, Status> {
          let process = Process::CreateIdentity { ... };
          self.engine.apply(&mut self.state, process)?;
          Ok(...)
      }
  }
  ```
- [ ] `TrustService`: Attestations, Trust-Queries
- [ ] `TransactionService`: Transfers, Exchanges
- [ ] `QueryService`: State-Abfragen

### Middleware

- [ ] Rate-Limiting
- [ ] Authentication (JWT/DID-Auth)
- [ ] Request-Logging
- [ ] Error-Handling → gRPC-Status-Codes

## TODO: Knoten-Binary (`bins/ery-node`)

### Server-Setup

- [ ] `main.rs`:
  ```rust
  #[tokio::main]
  async fn main() {
      let state = SystemState::new();
      let engine = TransitionEngine::new();
      
      let server = EryServer::new(state, engine);
      
      Server::builder()
          .add_service(IdentityServiceServer::new(server.clone()))
          .add_service(TrustServiceServer::new(server.clone()))
          .serve("[::]:50051").await?;
  }
  ```
- [ ] Graceful Shutdown
- [ ] Health-Check Endpoint
- [ ] Metrics Endpoint (Prometheus)

### State-Persistenz (Optional für Phase 3)

- [ ] Periodischer Snapshot zu Disk
- [ ] WAL (Write-Ahead Log) für Crash-Recovery
- [ ] Später: PostgreSQL/SQLite Integration

### Konfiguration

- [ ] `config/` Dateien nutzen (TOML)
- [ ] Umgebungsvariablen für Secrets
- [ ] CLI-Flags für Override

## TODO: CLI-Client (`bins/ery-cli`)

### Befehle implementieren

- [ ] `ery-cli identity create [--name <name>]`
- [ ] `ery-cli identity list`
- [ ] `ery-cli identity show <did>`
- [ ] `ery-cli trust attest <target-did> --positive/--negative`
- [ ] `ery-cli trust show <did>`
- [ ] `ery-cli tx transfer <from> <to> <amount>`
- [ ] `ery-cli state info` (zeigt 𝔼, Entity-Count, etc.)

### CLI-Framework

- [ ] `clap` für Argument-Parsing
- [ ] Konfigurierbare Server-Adresse
- [ ] Formatierte Ausgabe (Table, JSON)

## TODO: Integration Tests

- [ ] End-to-End Tests mit laufendem Server
- [ ] CLI → gRPC → Engine → State → Response
- [ ] Concurrent Requests testen
- [ ] Error-Cases testen

---

# Zusammenfassung: Checkliste

## Phase 1 Abnahme-Kriterien

- [ ] `cargo test` läuft erfolgreich
- [ ] Alle 25 Axiome als Unit Tests abgebildet
- [ ] Weltformel-Berechnung verifiziert
- [ ] `InvariantChecker` findet keine Verletzungen bei korrekten Aktionen
- [ ] Dokumentation: Jedes Modul hat Rustdoc-Kommentare

## Phase 2 Abnahme-Kriterien

- [ ] Simulation läuft 10.000 Ticks ohne Crash
- [ ] Invarianten werden nie verletzt
- [ ] Angriffs-Szenarien werden abgewehrt
- [ ] CSV-Export funktioniert
- [ ] Mindestens 3 Graphen für Bachelorarbeit erstellt

## Phase 3 Abnahme-Kriterien

- [ ] gRPC-Server startet und antwortet
- [ ] CLI kann alle Basis-Operationen ausführen
- [ ] Integration Tests bestehen
- [ ] Performance: > 100 req/s auf Single-Node

---

# Werkzeuge & Empfehlungen

## Rust-Crates

| Crate | Verwendung |
|-------|------------|
| `serde` | Serialisierung |
| `thiserror` | Error-Definitionen |
| `petgraph` | DAG-Struktur (optional) |
| `tokio` | Async Runtime |
| `tonic` / `connect-rust` | gRPC |
| `clap` | CLI-Parsing |
| `tracing` | Logging |
| `rand` | Zufallsgenerator für Simulation |
| `csv` | CSV-Export |

## Entwicklungs-Workflow

1. **TDD**: Erst Test schreiben, dann Implementierung
2. **Incremental**: Kleine Commits, häufig testen
3. **Dokumentation**: Rustdoc während des Schreibens
4. **CI**: GitHub Actions für `cargo test` + `cargo clippy`

## Für die Bachelorarbeit

- Phase 1 → Kapitel „Formale Spezifikation und Implementierung"
- Phase 2 → Kapitel „Evaluation und Simulation"
- Phase 3 → Kapitel „Prototypische Anwendung"

---

*Implementation Roadmap Version 1.0 – Vom Abstrakten zum Konkreten.*
