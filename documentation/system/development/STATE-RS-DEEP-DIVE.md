# state.rs – Tiefenverständnis und Zusammenhänge

Dieses Dokument erklärt die **Unified State Management**-Datei `backend/src/core/state.rs` in der Tiefe: Architektur, Schichten, Beziehungen und Abläufe.

**Verwandte Spezifikation:** [DID-IDENTITY-SPECIFICATION.md](./DID-IDENTITY-SPECIFICATION.md) – hierarchisches DID-System (Root DID, UTI, Peer/Agent Sub-DIDs, Wallet, WalletConnect) und geplante Integration als DidState in UnifiedState.

---

## 1. Überblick und Design-Prinzipien

`state.rs` ist das **zentrale State-Management** für alle Erynoa-Module. Es ist:

- **Hierarchisch**: State-Layer bauen aufeinander auf (Core → Execution → Protection → Storage → Peer → …).
- **Thread-safe**: Zähler als `AtomicU64`/`AtomicUsize`, komplexe Strukturen unter `RwLock` (oder `Arc<RwLock>`).
- **Snapshot-basiert**: Jeder State hat eine `snapshot()`-Methode → konsistente Reads ohne lange Locks.
- **Event-getrieben**: Änderungen werden als `StateEvent` geloggt und über Observer/`StateDelta` propagiert.
- **Realm-isoliert**: Pro Realm eigener Trust, Regeln, Quotas, Crossing-Metriken.

Die neun dokumentierten Design-Prinzipien (Doc-Kommentar am Anfang der Datei) sind:

1. Hierarchische Komposition
2. Thread-Safety (Atomics + RwLock)
3. Dependency Injection
4. Event-Driven Updates (Observer)
5. Snapshot-Isolation
6. Per-Realm Isolation
7. Event-Inversion (P2P/Core über Queues)
8. Circuit Breaker
9. CQRS light (State-Deltas an Subscriber)

---

## 2. Grobstruktur der Datei (Blöcke)

| Zeilen (ca.) | Thema |
|--------------|--------|
| ~86–162 | **SystemMode** (Circuit Breaker: Normal / Degraded / EmergencyShutdown) |
| ~164–516 | **EventBus** (P2P↔Core), **StateDelta/CQRS**, **StateBroadcaster**, **BroadcasterSnapshot** |
| ~519–591 | **StorageHandle** (Backend: Fjall, RocksDB, IPFS, Cloud, Memory) |
| ~593–739 | **CircuitBreaker** (Modus, kritische Anomalien, Gini-Check, Reset) |
| ~741–1335 | **StateEvent** (alle Event-Varianten), **WrappedStateEvent**, **StateEventLog** (Event-Sourcing) |
| ~1337–1700 | **StateGraph**, **StateComponent**, **StateRelation**, **Hashable**, **MerkleStateTracker** |
| ~2778–3310 | **CoreState**: TrustState, EventState, FormulaState, ConsensusState + Snapshots |
| ~3312–3655 | **ExecutionState**: GasState, ManaState, ExecutionsState + Snapshots |
| ~3657–4255 | **ECLVMState**, **RealmECLState** (Policies, Blueprints, Sagas, Crossing) |
| ~4257–4758 | **ProtectionState**: Anomaly, Diversity, Quadratic, AntiCalcification, Calibration, optional ShardMonitor |
| ~4760–4855 | **StorageState** (KV, EventStore, Archive, Blueprint-Marketplace) |
| ~4857–5230 | **PeerState**: GatewayState, SagaComposerState, IntentParserState, **RealmState** |
| ~5232–5320 | **RealmSpecificState** (pro Realm: Trust, Members, Policies, Quota, Crossing, Sagas) |
| ~5322–6560 | **P2PState**: SwarmState, GossipState, KademliaState, RelayState, PrivacyState |
| ~6562–6910 | **UIState**, **APIState**, **GovernanceState** (+ Realm-Varianten) |
| ~6912–8570 | **ControllerState**, **DataLogicState**, **BlueprintComposerState**, **UnifiedState** |
| ~8815–9475 | **UnifiedState**: `snapshot()`, `log_and_apply()`, `apply_state_event()`, Replay, Checkpoints |
| ~9477–9575 | **UnifiedSnapshot**, **create_unified_state()** |
| ~12930–14150+ | **Sharding**: ShardingConfig, RealmLoadError, RealmStorageLoader, LazyShardedRealmState |

---

## 3. Zentrale Konzepte im Detail

### 3.1 SystemMode und Circuit Breaker

- **SystemMode**: `Normal` (alles erlaubt), `Degraded` (Execution pausiert, Crossings blockiert, Mana-Regeneration 0), `EmergencyShutdown` (nur Admin-Recovery).
- **CircuitBreaker**:
  - Zählt kritische Anomalien in einem 1-Minuten-Fenster.
  - Überschreitung von `degraded_threshold` → Degraded; von `emergency_threshold` → EmergencyShutdown.
  - `record_critical_anomaly()` aktualisiert das Fenster und setzt ggf. den Modus.
  - `check_gini(gini)` kann bei Überschreitung des Gini-Thresholds ebenfalls Anomalie auslösen (Anti-Calcification Κ19).
  - `reset_to_normal()` manuell (Admin).

**Zusammenhang**: ProtectionState (Anomaly) ruft bei kritischen Anomalien den Circuit Breaker auf; UnifiedState delegiert `allows_execution()` / `allows_crossings()` an den Circuit Breaker.

---

### 3.2 EventBus (P2P/Core Entkopplung)

- **Ingress**: P2P schickt `NetworkEvent` in `ingress_tx` (oder `priority_ingress_tx` für Critical).
- **Egress**: Core schickt ausgehende Events in `egress_tx`.
- **Core** nimmt die Receiver (`take_ingress_receiver` / `take_priority_receiver`) und verarbeitet in einer Task; P2P nimmt `take_egress_receiver` und sendet.
- Zähler: `ingress_count`, `egress_count`, `dropped_count`, `processed_count`, `priority_processed`.

**Zusammenhang**: Entkoppelt Netzwerk-I/O von Core-Logik; Backpressure durch bounded Channels.

---

### 3.3 StateDelta und StateBroadcaster (CQRS light)

- **StateDelta**: `sequence`, `component` (StateComponent), `delta_type` (Increment, Snapshot, Insert, Delete, Update, Batch), `data` (Bytes), `timestamp_ms`, optional `realm_id`.
- **StateBroadcaster**: `broadcast::Sender<StateDelta>`. `subscribe()` liefert einen `Receiver<StateDelta>`.
- Bei State-Änderungen kann Core `broadcaster.broadcast(delta)` aufrufen; DataLogic, Monitoring, Metrics-Exporter können subscriben.

**Zusammenhang**: UnifiedState nutzt das in `log_and_apply()`: nach Apply wird ein Delta gebroadcastet. Observer in `state_integration` können ihrerseits Deltas auslösen oder auf State-Änderungen reagieren.

---

### 3.4 StateGraph und StateComponent / StateRelation

- **StateComponent**: Enum aller logischen Komponenten (Trust, Event, WorldFormula, Consensus, Gas, Mana, ECLVM, Gateway, Realm, UI, API, Governance, Controller, DataLogic, BlueprintComposer, …).
- **StateRelation**: z.B. `Triggers`, `DependsOn`, `Validates`, `Aggregates`.
- **StateGraph**: Liste von Kanten `(from, relation, to)`. Enthält die „Soll“-Beziehungen (z.B. Trust → Triggers → Event; Event → Triggers → Trust; Trust/Event → DependsOn → WorldFormula; Gas ← Calibration).

**Zusammenhang**: Wird für Validierung, Abhängigkeitsanalyse und Merkle-Deltas genutzt. Die echten Zähler für „Trust hat Event getriggert“ etc. stehen in den jeweiligen States (z.B. TrustState.triggered_events, EventState.trust_triggered).

---

### 3.5 Event-Sourcing: StateEvent, WrappedStateEvent, StateEventLog

- **StateEvent**: Großes Enum mit allen semantischen Änderungen (TrustUpdate, EventProcessed, FormulaComputed, ConsensusRoundCompleted, ExecutionStarted/Completed, PolicyEvaluated, BlueprintAction, SagaProgress, AnomalyDetected, DiversityMetricUpdate, CalibrationApplied, SystemModeChanged, RealmLifecycle, MembershipChange, CrossingEvaluated, NetworkMetricUpdate, PeerConnectionChange, CheckpointCreated, RecoveryCompleted, ReorgDetected, ProposalCreated, VoteCast, ProposalResolved, QuotaViolation, RealmQuarantineChange).
- **WrappedStateEvent**: `id` (Hash), `timestamp_ms`, `parent_ids`, `component`, `sequence`, `event`, optional `signature`.
- **StateEventLog**:
  - In-Memory-Buffer der letzten N Events, `sequence`, Checkpoint-Intervall.
  - `log(event, parent_ids)` erzeugt WrappedStateEvent, schreibt in Buffer, erhöht Zähler und `events_by_component`.
  - `mark_checkpoint(checkpoint_id, state_hash)` für Recovery.
  - `needs_checkpoint()`, `start_recovery()` / `end_recovery()`.

**Zusammenhang**: `UnifiedState::log_and_apply(event, parent_ids)`:
1. Ruft `event_log.log(event, parent_ids)` auf.
2. Wendet das Event mit `apply_state_event(&wrapped)` auf den lebenden State an.
3. Broadcastet ein StateDelta.
4. Aktualisiert den MerkleStateTracker.
5. Wenn `needs_checkpoint()`, wird ein Checkpoint markiert.

Replay: `replay_events(events)` ruft für jedes Event `apply_state_event` auf (mit `start_recovery`/`end_recovery`). So wird State aus Event-Historie wiederhergestellt.

---

### 3.6 MerkleStateTracker und MultiGas

- **MerkleStateTracker**: Merkle-Baum über State-Komponenten; `root_hash()`, `update_component(component, data)`, `deltas_since(seq)`, `verify_delta(delta)`. Für Light-Clients und Differential Sync.
- **MultiGas**: Mehrstufiges Gas (Network, Compute, Storage, Realm). Pro Layer Verbrauch und optional Preise; Realms können für Realm-Gas registriert werden.

**Zusammenhang**: UnifiedState bietet `merkle_root()`, `update_with_merkle()`, `deltas_since()`, `verify_delta()` sowie `consume_network_gas`, `consume_compute_gas`, `consume_storage_gas`, `consume_realm_gas`, `set_gas_price`, `realm_gas_consumed`.

---

### 3.7 CoreState (Κ2–Κ18)

- **TrustState**: entities_count, relationships_count, updates_total, positive/negative_updates, violations_count, avg_trust, trust_distribution; Relationship-Counter: triggered_events, event_triggered_updates, realm_triggered_updates. `update(positive, from_event)`, `update_triggered_event()`, `asymmetry_ratio()`.
- **EventState**: total, genesis, finalized, witnessed, validation_errors, cycles_detected, max_depth, avg_parents, finality_latency_ms; Trigger-Counter (trust_triggered, consensus_validated, execution_triggered, …). `add(is_genesis, parents_count, depth)`, `finalize(latency_ms)`.
- **FormulaState**: current_e, computations, contributors, human_verified, avg_activity, avg_trust_norm, human_factor, e_history. `update(e, activity, trust_norm, human_factor)`, `trend()`.
- **ConsensusState**: epoch, validators, successful_rounds, failed_rounds, avg_round_time_ms, byzantine_detected, leader_changes, events_validated. `round_completed(success, duration_ms)`, `success_rate()`.
- **CoreState** aggregiert diese vier und liefert **CoreSnapshot**.

**Zusammenhang**: Trust-Event-Kausalität (StateGraph) wird in den Relationship-Countern abgebildet. Observer (state_integration) schreiben in diese States; `apply_state_event` führt die entsprechenden Updates aus (TrustUpdate → trust.update, EventProcessed → events.add/finalize, etc.).

---

### 3.8 ExecutionState (IPS ℳ)

- **GasState**: consumed, refunded, out_of_gas_count, current_price, max_per_block, calibration_adjustments, trust_dependency_updates. `consume()`, `refund()`.
- **ManaState**: consumed, regenerated, rate_limited_count, regen_rate, max_per_entity, calibration_adjustments, trust_dependency_updates.
- **ExecutionsState**: active_contexts, total, successful, failed, events_emitted, saga_triggered, gas_aggregations, mana_aggregations, Lamport/Epoch. `start()`, `complete(success, events, duration_ms)`.
- **ExecutionState** bündelt gas, mana, executions und bietet `start()`, `complete()`, `avg_execution_time()`, `success_rate()`.

**Zusammenhang**: Execution-Engines und Observer aktualisieren diese Werte. Calibration (Protection) kann Gas/Mana-Parameter anpassen (calibration_adjustments). Im Event-Sourcing: ExecutionStarted/ExecutionCompleted werden in `apply_state_event` auf `execution.complete` und gas/mana abgebildet.

---

### 3.9 ECLVMState und RealmECLState

- **ECLVMState**: Policies (compiled, cached, executed, passed, denied, runtime_errors), Blueprints (published, deployed, instantiated, verified, downloaded), Intents/Sagas (processed, successful, saga_steps, cross_realm_steps, compensations), total_gas_consumed, total_mana_consumed, out_of_gas_aborts, mana_rate_limited, realm_ecl (HashMap<String, RealmECLState>), crossing_evaluations/allowed/denied, avg_evaluation_time_us, events_emitted.
- **RealmECLState** (pro Realm): policies_executed/passed/denied, gas_consumed, mana_consumed, crossing_policies, membership_policies, active_policies, instantiated_blueprints. `policy_executed()`, `register_policy()`, `success_rate()`.
- ECLVMState: `policy_compiled()`, `policy_executed()` (inkl. Per-Realm-Update), `blueprint_*()`, `intent_processed()`, `saga_step_executed()`, `compensation_triggered()`, `get_or_create_realm_ecl(realm_id)`.

**Zusammenhang**: ECL/ECLVM-Execution und Gateway (Crossing-Policies) schreiben hier. `apply_state_event` mappt PolicyEvaluated, BlueprintAction, SagaProgress auf diese Strukturen.

---

### 3.10 ProtectionState (Κ19–Κ21)

- **AnomalyState**: Zähler pro Severity (critical, high, medium, low). `record(severity)`.
- **DiversityState**: entropy pro dimension, monoculture_warnings, trust_distribution_checks. `set_entropy(dimension, value)`.
- **QuadraticState**: votes_cast, proposals_created, governance_actions.
- **AntiCalcificationState**: interventions, threshold_violations, gini_readings.
- **CalibrationState**: updates_total, params_map (RwLock<HashMap<String, f64>>). Kalibrierung beeinflusst Gas/Mana (Relationship Calibration → Gas/Mana).
- Optional: **ShardMonitor** (Sharding Phase 7): pro Shard Reputation, Cross-Shard-Success/Failure, Quarantine, `get_cross_shard_penalty()`, `contribute_to_veto()`.

**Zusammenhang**: Protection-Engines und Observer füllen diese States. `protection.anomaly_with_circuit_breaker(severity, &circuit_breaker)` verbindet mit dem Circuit Breaker. Health-Score und Shard-Monitor fließen in `protection.health_score()` und ggf. in UnifiedState `calculate_health()` ein.

---

### 3.11 StorageState

Zähler für KV (keys, bytes, reads, writes), EventStore (count, bytes), Archive (epochs, events, bytes, merkle_roots), Blueprint-Marketplace (published, deployed, downloaded), Realms (realm_count, identities, trust_entries). Keine komplexe Logik, nur Metriken für Persistence.

---

### 3.12 PeerState: Gateway, SagaComposer, IntentParser, RealmState

- **GatewayState**: crossings_total/allowed/denied, policy_checks, min_trust_checks. Erfolgsrate für Crossings.
- **SagaComposerState**: compositions, successful, failed, cross_realm_compositions, compensations. `composition_success_rate()`.
- **IntentParserState**: intents_parsed, parse_errors, intents_executed.
- **RealmState**: `realms: RwLock<HashMap<String, RealmSpecificState>>`, total_realms, identity_joined_realm/identity_left_realm, get_realm(), get_or_create_realm().

**Zusammenhang**: Gateway prüft Crossings und aktualisiert GatewayState; SagaComposer schreibt in SagaComposerState; RealmState hält alle RealmSpecificState-Instanzen. Bei Sharding (Phase 7) kann RealmState durch LazyShardedRealmState ersetzt werden (lazy load, LRU, Shards).

---

### 3.13 RealmSpecificState (Per-Realm Isolation)

Pro Realm:

- **Trust & Governance**: trust (TrustVector6D), min_trust, governance_type.
- **Membership**: members, identity_count, pending_members, banned_members, admins.
- **ECL**: active_policies, active_rules.
- **Isolation**: isolation_level, leak_attempts, leaks_blocked.
- **Crossing**: crossings_in/out/denied, active_crossings, crossing_allowlist/blocklist.
- **Saga**: sagas_initiated, cross_realm_sagas_involved, sagas_failed, compensations_executed.
- **Activity**: events_total, events_today, last_event_at, created_at.
- **RealmQuota** (Self-Healing): Limits und Verbrauch pro ResourceType (QueueSlots, StorageBytes, ComputeGas, …), quarantine. `check_operation()`, `consume_resource()`, `quarantine()` / `unquarantine()`.

**Zusammenhang**: Realm-Lifecycle und Membership-Events in `apply_state_event` ändern RealmState und RealmSpecificState (total_realms, identity_joined/left). Quota und Quarantine werden von UnifiedState (`check_realm_quota`, `consume_realm_resource`, `quarantine_realm`) genutzt.

---

### 3.14 P2PState

SwarmState (connected_peers, bytes_sent/received, …), GossipState, KademliaState, RelayState, PrivacyState. Metriken für Netzwerk und `p2p.health_score()`.

---

### 3.15 Engine-Layer: UI, API, Governance, Controller, DataLogic, BlueprintComposer

- **UIState**: Komponenten, Bindings, Trust-Gates, Cache-Hits.
- **APIState**: Requests, Success/Error, Rate-Limits, Latenz.
- **GovernanceState**: proposals_created/completed/accepted, votes_cast.
- **ControllerState**: AuthZ-Checks, Erfolgsrate.
- **DataLogicState**: Events verarbeitet, Bindings, Aggregationen.
- **BlueprintComposerState**: Compositions, Cache, Versioning.

**Zusammenhang**: Diese States werden von den jeweiligen Engines und von `apply_state_event` (z.B. ProposalCreated, VoteCast, ProposalResolved) befüllt. Sie fließen in `UnifiedState::calculate_health()` ein.

---

### 3.16 UnifiedState – der Knotenpunkt

**Enthält**: core, execution, eclvm, protection, storage, peer, p2p, ui, api, governance, controller, data_logic, blueprint_composer, graph, warnings, health_score, event_bus, circuit_breaker, broadcaster, storage_handle, merkle_tracker, multi_gas, event_log.

**Wichtige Methoden**:

- `new()`, `snapshot()` → **UnifiedSnapshot** (alle Sub-Snapshots + event_bus, circuit_breaker, broadcaster, system_mode, merkle_tracker, multi_gas, event_log).
- `calculate_health()`: gewichtete Kombination aus Protection, Consensus, Execution, ECLVM, P2P, Peer, Crossing, Event-Errors, UI/API/Governance/Controller/DataLogic/Blueprint-Health.
- `log_and_apply(event, parent_ids)`: Event loggen, anwenden, Delta broadcasten, Merkle updaten, ggf. Checkpoint.
- `apply_state_event(wrapped)`: großes Match über `wrapped.event` und Anwendung auf core, execution, eclvm, protection, peer, p2p, governance (siehe Abschnitt „apply_state_event“ oben).
- `replay_events(events)`, `create_checkpoint()`, `is_recovering()`, `event_log_stats()`.
- Circuit Breaker: `is_operational()`, `allows_execution()`, `allows_crossings()`, `system_mode()`, `record_anomaly()`, `reset_circuit_breaker()`.
- CQRS: `broadcast_delta()`, `subscribe_deltas()`.
- EventBus: `send_network_event()`, `receive_network_event()`.
- Merkle: `merkle_root()`, `update_with_merkle()`, `deltas_since()`, `verify_delta()`.
- MultiGas: `consume_network_gas`, `consume_compute_gas`, `consume_storage_gas`, `consume_realm_gas`, `register_realm_for_gas`, `set_gas_price`, `realm_gas_consumed`.
- Realm: `check_realm_quota`, `consume_realm_resource`, `quarantine_realm`, `unquarantine_realm`, `is_realm_quarantined`.

**Zusammenhang**: Alles läuft in UnifiedState zusammen. Observer (state_integration) halten eine Referenz auf SharedUnifiedState und mutieren die Sub-States; Event-Sourcing und Circuit Breaker sorgen für Konsistenz und Degradation.

---

### 3.17 Sharding (Phase 7): LazyShardedRealmState, RealmStorageLoader

- **ShardingConfig**: num_shards, max_per_shard, eviction_interval_secs, lru_capacity_per_shard, lazy_loading_enabled, event_replay_on_load.
- **RealmLoadError**: NotFound, StorageError, DeserializationError, EventReplayError, LazyLoadingDisabled, ShardOverloaded.
- **RealmStorageLoader** (async trait): `load_realm_base()`, `load_realm_events_since()`, `realm_exists()`, `persist_realm_snapshot()`.
- **LazyShardedRealmState**: Shards als DashMap (oder ähnlich), pro Shard Realm-Store + LRU; `get_or_load(realm_id)` lädt bei Cache-Miss über RealmStorageLoader, optional Event-Replay, und fügt in den Shard ein. Eviction nach Konfiguration.

**Zusammenhang**: Bei vielen Realms wird RealmState durch LazyShardedRealmState ersetzt; die gleiche RealmSpecificState/RealmSpecificSnapshot-Schnittstelle bleibt. Persistence und Recovery laufen über RealmStorageLoader und Event-Replay.

---

## 4. Datenfluss-Zusammenfassung

1. **Eingang**: API/ECL/P2P erzeugen Aktionen → Engines/Observer mutieren UnifiedState (core, execution, eclvm, protection, peer, p2p, …).
2. **Event-Sourcing**: Wichtige Änderungen als StateEvent → `log_and_apply()` → StateEventLog, Apply, Delta, Merkle, ggf. Checkpoint.
3. **CQRS**: StateDelta-Broadcast an Subscriber (DataLogic, Monitoring).
4. **P2P**: NetworkEvent über EventBus (Ingress/Egress) zwischen P2P und Core.
5. **Circuit Breaker**: Protection/Anomaly ruft bei kritischen Anomalien Circuit Breaker → Moduswechsel → `allows_execution()`/`allows_crossings()` schränken ein.
6. **Snapshots**: Alles lesen über `state.snapshot()` (UnifiedSnapshot) oder Sub-State `.snapshot()`; keine Schreib-Locks für konsistente Reads.
7. **Realm**: Alle Realm-differenzierten Daten in RealmState → RealmSpecificState; Quotas und Quarantine über UnifiedState-Helfer und Protection.
8. **Recovery**: Event-Replay aus StateEventLog oder aus Storage (RealmStorageLoader) in LazyShardedRealmState.

---

## 5. Identity-Layer (Κ6-Κ8) – NEU

Mit dem DID-Identity-Refactoring wurde ein dedizierter **Identity-Layer** in `UnifiedState` integriert:

### 5.1 IdentityState

Position: Vor CoreState (erste State-Schicht nach `started_at`)

```rust
pub struct UnifiedState {
    pub started_at: Instant,
    pub identity: IdentityState,  // NEU
    pub core: CoreState,
    // ...
}
```

**Felder (Atomic-Counters):**
- `sub_dids_total`, `addresses_total`, `active_delegations_count`, `revoked_delegations_count`
- `credentials_issued`, `credentials_verified`
- `events_triggered`, `trust_entries_created`, `realm_memberships_changed`
- `gas_consumed`, `mana_consumed`, `signatures_created`, `signatures_verified`
- `root_created_at_ms`, `mode`, `bootstrap_completed`

**Felder (RwLock-geschützt):**
- `root_did`, `root_document`, `current_device_did`
- `sub_dids`, `sub_did_counts`, `wallets`, `delegations`, `realm_memberships`
- `key_store`, `passkey_manager`

**Wichtige Methoden:**
- `bootstrap_*()` (interactive, agent, ephemeral, test)
- `derive_device_did()`, `derive_agent_did()`, `derive_realm_did()`
- `add_delegation()`, `revoke_delegation()`, `is_delegation_valid()`
- `join_realm()`, `leave_realm()`, `is_realm_member()`
- `add_wallet_address()`, `get_wallets_for_chain()`, `get_primary_wallet()`
- `health_score()`, `snapshot()`

### 5.2 Identity-Events (StateEvent)

13 neue Identity-bezogene Events:

| Event | Beschreibung |
|-------|-------------|
| `IdentityBootstrapped` | Root-DID erstellt |
| `IdentityModeChanged` | Modus gewechselt |
| `SubDIDDerived` | Sub-DID abgeleitet |
| `WalletDerived` | Wallet-Adresse erstellt |
| `DelegationCreated` | Delegation erstellt |
| `DelegationRevoked` | Delegation widerrufen |
| `CredentialIssued` | Credential ausgestellt |
| `CredentialVerified` | Credential verifiziert |
| `KeyRotated` | Schlüssel rotiert (kritisch) |
| `RecoveryInitiated` | Recovery gestartet (kritisch) |
| `IdentityAnomalyDetected` | Anomalie erkannt |
| `CrossShardIdentityResolved` | Cross-Shard Identity aufgelöst |
| `RealmMembershipChanged` | Realm-Mitgliedschaft geändert |

### 5.3 StateGraph Identity-Kanten

38 neue Kanten im StateGraph verbinden Identity mit anderen Komponenten:

```
Trust ← DependsOn ← Identity
Identity → Triggers → Trust, Realm, Event, Anomaly
Swarm, Gateway, Controller → Validates → Identity
UI, API, ECLPolicy, Governance → DependsOn → Identity
```

### 5.4 Zusammenhänge

- **Trust ↔ Identity**: Delegationen beeinflussen Trust (Κ8 Trust-Decay)
- **Realm ↔ Identity**: Realm-Memberships über `IdentityState.join_realm()`
- **P2P ↔ Identity**: `SwarmState.peer_universal_id` für Device-DID
- **Events ↔ Identity**: Identity-Events werden wie andere Events geloggt/replayed
- **Protection ↔ Identity**: `IdentityAnomalyDetected` bei verdächtigen Mustern

---

## 6. Namenskonventionen (laut Prüfung)

- **State-Layer**: `{Name}State` (z. B. TrustState).
- **Sub-State**: `{Parent}{Name}State` (z. B. RealmECLState).
- **Snapshot**: `{Name}Snapshot` ohne „State“ im Namen (z. B. TrustSnapshot, BroadcasterSnapshot, EventLogSnapshot, MerkleTrackerSnapshot).
- **Config**: `{Name}Config`; **Metrics**: `{Name}Metrics`; **Error**: `{Name}Error`; **Trait**: beschreibend, ohne Suffix.

---

Dieses Dokument deckt die Architektur, alle großen Blöcke, die Rolle jedes State-Layers und deren Zusammenspiel in `state.rs` ab. Für einzelne Funktionen oder Felder bleibt die Quelle `backend/src/core/state.rs` die Referenz.
