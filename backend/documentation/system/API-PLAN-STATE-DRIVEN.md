# API-Plan: State-getriebene System-Integration

**Basis:** `state.rs` (UnifiedState, Snapshots, Event-Log, Health, Invarianten, Circuit Breaker, Merkle, CQRS).
**Ziel:** System vollständig integrierbar machen – **Debug/Observability** und **produktive Interaktionen** – logisch abgestimmt, präzise, erweiterbar.

**Protokoll:** Connect-RPC (primär) + REST-Fallbacks; Streaming wo sinnvoll.

---

## 1. Übersicht: API-Domänen

| Domäne | Zweck | Zielgruppe |
|--------|--------|------------|
| **State Introspection** | Snapshots, Teil-Snapshots, Komponenten-Metriken | Debug, Monitoring, Dashboards |
| **Health & Invariants** | Aggregierter Health, Invarianten-Checks, Warnings | Probes, SRE, Alerting |
| **Event Log & Replay** | Event-Historie, Filter, Replay, Checkpoints | Debug, Audit, Recovery |
| **Circuit Breaker & Mode** | SystemMode lesen/setzen, Degradation steuern | Ops, Notfall |
| **Merkle & Delta Sync** | State-Proofs, inkrementelle Deltas | Light-Clients, Sync |
| **Streaming / CQRS** | State-Delta-Subscription, Echtzeit-Feed | UIs, DataLogic |
| **Crossing & Gateway** | Realm-Crossing prüfen, ECL-Entry | Produktion |
| **ECL/ECLVM** | Policy ausführen, Entrypoints (API/UI/…), Metriken | Produktion, Admin |
| **Trust & Identity** | Trust lesen/aktualisieren, Identity/Realm | Produktion |
| **Governance & Controller** | Proposals, Votes, AuthZ | Produktion |
| **Realm & Membership** | Realm-Lifecycle, Membership | Produktion |
| **Intent & Saga** | Intent parsen, Saga komponieren/ausführen | Produktion |

---

## 2. State Introspection (Debug & Monitoring)

Alle Endpoints lesen ausschließlich aus UnifiedState/UnifiedSnapshot; keine Mutationen.

### 2.1 Voll-Snapshot

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| GET | `/api/v1/state/snapshot` | Liefert **UnifiedSnapshot** (JSON): identity, core, execution, eclvm, protection, storage, peer, p2p, ui, api, governance, controller, data_logic, blueprint_composer, health_score, warnings, event_bus, circuit_breaker, broadcaster, system_mode, merkle_tracker, multi_gas, event_log. |
| Query | `?components=core,eclvm,peer` | Optional: nur angegebene Komponenten im Snapshot (reduziert Payload). |
| Query | `?realm=<realm_id>` | Optional: bei Realm-Komponenten nur diesen Realm (peer.realm, realm_ecl, etc.). |

**Response (Auszug):**
`UnifiedSnapshot` mit `timestamp_ms`, `uptime_secs`, allen Sub-Snapshots, `health_score`, `warnings`, `system_mode`, `event_log.sequence`, etc.

### 2.2 Komponenten-Snapshots (granular)

| Methode | Endpoint | state.rs-Referenz |
|---------|----------|-------------------|
| GET | `/api/v1/state/core` | CoreSnapshot (trust, events, formula, consensus) |
| GET | `/api/v1/state/execution` | ExecutionSnapshot (gas, mana, executions) |
| GET | `/api/v1/state/eclvm` | ECLVMSnapshot (policies_*, crossing_*, policies_by_type, avg_evaluation_time_us, realm_ecl) |
| GET | `/api/v1/state/protection` | ProtectionSnapshot (anomaly, diversity, quadratic, anti_calc, calibration) |
| GET | `/api/v1/state/storage` | StorageSnapshot (kv_*, event_store, archive, blueprint_marketplace) |
| GET | `/api/v1/state/peer` | PeerSnapshot (gateway, saga, intent_parser, realm) |
| GET | `/api/v1/state/p2p` | P2PSnapshot (swarm, gossip, kademlia, relay, privacy) |
| GET | `/api/v1/state/ui` | UISnapshot (components, bindings, trust_gates) |
| GET | `/api/v1/state/api` | APISnapshot (endpoints, rate_limits, requests) |
| GET | `/api/v1/state/governance` | GovernanceSnapshot (proposals, votes, delegation) |
| GET | `/api/v1/state/controller` | ControllerSnapshot (permissions, authz, audit) |
| GET | `/api/v1/state/data_logic` | DataLogicSnapshot (streams, aggregations, events) |
| GET | `/api/v1/state/blueprint_composer` | BlueprintComposerSnapshot (compositions, cache) |
| GET | `/api/v1/state/identity` | IdentitySnapshot (root_did, sub_dids, delegations, health) |
| GET | `/api/v1/state/event_bus` | EventBusSnapshot (ingress/egress queues) |
| GET | `/api/v1/state/merkle` | MerkleTrackerSnapshot (root, component_hashes) |
| GET | `/api/v1/state/multi_gas` | MultiGasSnapshot (L1–L4 Gas/Mana) |
| GET | `/api/v1/state/event_log` | EventLogSnapshot (sequence, buffer_size, total_events, checkpoints) |

**Query-Parameter:**
`realm=<realm_id>` wo sinnvoll (z. B. peer, eclvm.realm_ecl, governance, controller).

### 2.3 Einzelmetriken (leichtgewichtig)

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| GET | `/api/v1/state/metrics` | Key-Value-Metriken für Prometheus/Scraping: z. B. `eclvm.policies_executed`, `eclvm.crossing_allow_rate`, `core.trust.updates_total`, `peer.gateway.crossings_total`, `p2p.swarm.connected_peers`, `health_score`, `system_mode`. |
| GET | `/api/v1/state/metrics/eclvm` | Nur ECLVM-Metriken (policies_executed, policies_by_type, avg_evaluation_time_us, crossing_*, policy_success_rate). |
| GET | `/api/v1/state/metrics/health` | Nur health_score, warnings, system_mode. |

---

## 3. Health & Invariants

### 3.1 Health (aggregiert)

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| GET | `/api/v1/health` | Liveness (bereits vorhanden). |
| GET | `/api/v1/ready` | Readiness inkl. Storage (bereits vorhanden). |
| GET | `/api/v1/health/state` | **State-Health:** `state.calculate_health()` (0–100), Gewichte aus state.rs (Identity, Protection, Consensus, Execution, ECLVM, P2P, Peer, Crossing, Event-Errors, UI/API/Governance/Controller/DataLogic/Blueprint). |
| GET | `/api/v1/health/state/detail` | Wie oben + Aufschlüsselung pro Layer (identity_health, protection_health, consensus_success_rate, execution_success_rate, eclvm_policy_success_rate, eclvm_crossing_allow_rate, p2p_health, peer_health, etc.). |

### 3.2 Invarianten

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| GET | `/api/v1/invariants` | **StateCoordinator.check_invariants():** Liste aller InvariantResult (invariant, passed, current_value, threshold, message). Invarianten: TrustAsymmetry, EventFinality, ConsensusSuccessRate, ExecutionSuccessRate, ProtectionHealth, StorageGrowthRate, DiversityEntropy, WorldFormulaPositive. |
| GET | `/api/v1/invariants?severity=critical` | Nur Critical/Error (für Alerting). |
| GET | `/api/v1/health/aggregate` | **StateCoordinator.aggregate_health():** HealthReport (score, status, invariant_results, module_scores, can_transact, recommendation). |

### 3.3 Warnings

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| GET | `/api/v1/state/warnings` | Aktive Warnings aus `state.warnings` (z. B. Trust-Asymmetrie, niedrige Peer-Count, Event-Validation-Errors). |
| DELETE | `/api/v1/state/warnings` | Alle Warnings löschen (Debug). |
| DELETE | `/api/v1/state/warnings/<key>` | Einzelne Warning nach Key löschen. |

---

## 4. Event Log & Replay (Debug & Audit)

### 4.1 Event-Historie

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| GET | `/api/v1/events` | Letzte N Events aus Event-Log-Buffer (z. B. `?limit=100`, `?since_sequence=12345`). Liefert WrappedStateEvent[] (id, timestamp_ms, parent_ids, component, sequence, event). |
| GET | `/api/v1/events?component=ECLPolicy` | Events gefiltert nach StateComponent (Trust, Event, Execution, ECLPolicy, Gateway, Realm, Identity, …). |
| GET | `/api/v1/events?realm=<realm_id>` | Events mit Realm-Kontext (realm_context() != null). |
| GET | `/api/v1/events/<sequence>` | Einzelnes Event nach Sequenznummer. |
| GET | `/api/v1/events/log/snapshot` | EventLogSnapshot (sequence, buffer_size, total_events, critical_events, events_since_checkpoint, last_checkpoint_sequence, is_recovering). |

### 4.2 Replay (Debug)

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| POST | `/api/v1/debug/replay` | **Body:** `{ "from_sequence": 0, "to_sequence": 1000 }`. Replay von Events auf eine frische State-Kopie (oder Dry-Run); Response: Anzahl angewendeter Events, letzter Snapshot-Hash, Fehler falls apply_state_event fehlschlägt. Nur in Non-Production oder mit Admin-Role. |
| POST | `/api/v1/debug/replay/checkpoint` | Replay ab letztem Checkpoint bis aktueller Sequenz (Recovery-Simulation). |

### 4.3 Checkpoints

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| GET | `/api/v1/events/checkpoints` | Letzter Checkpoint (id, sequence, state_hash). |
| POST | `/api/v1/debug/checkpoint` | Checkpoint manuell auslösen (state.event_log.mark_checkpoint). Nur Debug/Admin. |

---

## 5. Circuit Breaker & System Mode

### 5.1 Lesen

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| GET | `/api/v1/state/mode` | Aktueller **SystemMode** (Normal, Degraded, EmergencyShutdown) + Beschreibung. Aus state.circuit_breaker. |
| GET | `/api/v1/state/circuit_breaker` | CircuitBreakerSnapshot (mode, transitions, last_anomaly, etc.). |

### 5.2 Setzen (Ops/Notfall)

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| POST | `/api/v1/state/mode` | **Body:** `{ "mode": "Degraded" | "Normal" | "EmergencyShutdown" }`. Setzt SystemMode (nur mit Admin/Service-Role). Bei EmergencyShutdown: nur Admin-Recovery-Endpoint aktiv. |
| POST | `/api/v1/state/mode/reset` | Zurück auf Normal (nach manueller Prüfung). |

---

## 6. Merkle & Delta Sync (Light-Clients)

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| GET | `/api/v1/state/merkle/root` | Merkle-Root des aktuellen State (MerkleHash). |
| GET | `/api/v1/state/merkle/component/<component>` | Merkle-Hash einer Komponente (z. B. core, eclvm, peer). |
| GET | `/api/v1/state/delta?since_root=<hash>` | Delta seit angegebenem Root: geänderte Komponenten + neue Hashes + optionale Snapshot-Deltas (strukturiert für inkrementelle Sync). |
| GET | `/api/v1/state/proof/<component>` | State-Proof für eine Komponente (für Verifizierung gegen Root). |

---

## 7. Streaming / CQRS

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| GET (SSE) oder Connect Streaming | `/api/v1/state/stream` | **State-Delta-Stream:** Subscription auf state.broadcaster; jede State-Änderung (log_and_apply) liefert StateDelta (component, DeltaType, payload). Client erhält Echtzeit-Updates (z. B. für Dashboards). |
| Query | `?components=ECLPolicy,Gateway` | Optional: nur bestimmte Komponenten streamen. |

**Alternative:** Connect-RPC Server-Stream `StateService/SubscribeDeltas(SubscribeDeltasRequest) returns (stream StateDelta)`.

---

## 8. Crossing & Gateway (Produktion)

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| POST | `/api/v1/crossing/validate` | **Body:** `{ "caller_did": "...", "from_realm": "...", "to_realm": "..." }`. Ruft GatewayGuard.validate_crossing (Regel + optional ECL) auf. Response: allowed, reason, trust_score, gas_used, duration_us, policy_name. Entspricht state.rs Gateway + EclCrossingEvaluator. |
| GET | `/api/v1/crossing/stats` | GatewaySnapshot (crossings_total, crossings_allowed, crossings_denied, success_rate, avg_crossing_trust, trust_violations, credential_violations). |

---

## 9. ECL/ECLVM (Produktion & Admin)

### 9.1 Policy ausführen (generisch)

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| POST | `/api/v1/ecl/run` | **Body:** `{ "bytecode_base64": "...", "caller_did": "...", "realm_id": "...", "gas_limit": 50000, "policy_type": "api" }`. Führt run_policy aus (Host = ErynoaHost oder StateBackedHost); Response: value, gas_used, duration_us, passed (wenn Bool). |

### 9.2 Entrypoints (Engine-spezifisch)

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| POST | `/api/v1/ecl/api/<route_id>` | EclEntrypoints.run_api(route_id, caller_did, realm_id, gas_limit). |
| POST | `/api/v1/ecl/ui/<component_id>` | EclEntrypoints.run_ui(component_id, caller_did, realm_id, gas_limit). |
| POST | `/api/v1/ecl/datalogic/<stream_or_agg_id>` | EclEntrypoints.run_datalogic(...). |
| POST | `/api/v1/ecl/governance/<proposal_type_or_realm>` | EclEntrypoints.run_governance(...). |
| POST | `/api/v1/ecl/controller/<permission_or_resource>` | EclEntrypoints.run_controller(...). |

**Body (gemeinsam):** `{ "caller_did": "...", "realm_id": "...", "gas_limit": optional }`.
**Response:** value (JSON-Interpretation des ECL-Rückgabewerts), gas_used, duration_us.

### 9.3 ECL-Metriken & Admin

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| GET | `/api/v1/ecl/metrics` | ECLVMSnapshot (policies_executed, policies_by_type, crossing_*, avg_evaluation_time_us, policy_success_rate, crossing_allow_rate). |
| GET | `/api/v1/ecl/policies` | Registrierte ECL-Policies pro Realm (aus ProgrammableGateway/EclEntrypoints), nur Metadaten (name, realm, type). |
| POST | `/api/v1/ecl/compile` | **Body:** ECL-Quelltext oder AST. Response: bytecode_base64 oder Fehler (Debug/Admin). |

---

## 10. Trust & Identity (Produktion)

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| GET | `/api/v1/trust/<did>` | Trust-Vektor (6D) oder TrustRecord für DID. Aus StateView/Storage. |
| POST | `/api/v1/trust/update` | **Body:** TrustUpdate-Event-Parameter (entity_id, delta, reason, from_realm, …). Erzeugt StateEvent::TrustUpdate und log_and_apply. |
| GET | `/api/v1/identity/<did>` | Identity-Infos (DID Document, realms, delegations) aus state.identity. |
| GET | `/api/v1/identity/root` | Root-DID und Basis-Infos. |

---

## 11. Governance & Controller (Produktion)

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| POST | `/api/v1/governance/proposals` | Proposal erstellen; StateEvent::ProposalCreated + log_and_apply. |
| POST | `/api/v1/governance/proposals/<id>/vote` | Vote abgeben; StateEvent::VoteCast. |
| GET | `/api/v1/governance/proposals` | Liste Proposals (aus state.governance). |
| POST | `/api/v1/controller/check` | AuthZ-Check (permission, resource, caller_did, realm_id); nutzt state.controller + optional ECL run_controller. |
| GET | `/api/v1/controller/permissions` | Permissions für Realm/Caller (Read-Only). |

---

## 12. Realm & Membership (Produktion)

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| GET | `/api/v1/realms` | Realm-Liste (RealmSnapshot-ähnlich). |
| GET | `/api/v1/realms/<realm_id>` | Einzelnes Realm inkl. Rules, Members, ECL-Entry-Policy-Info. |
| POST | `/api/v1/realms` | Realm anlegen; StateEvent::RealmLifecycle. |
| POST | `/api/v1/realms/<realm_id>/members` | Membership ändern; StateEvent::MembershipChange / RealmMembershipChanged. |
| GET | `/api/v1/realms/<realm_id>/ecl` | RealmECLSnapshot für dieses Realm (policies_executed, crossing_policies, etc.). |

---

## 13. Intent & Saga (Produktion)

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| POST | `/api/v1/intent/parse` | **Body:** Intent-Text oder strukturiertes Goal. Response: geparster Intent, Validierung, geschätzte Saga-Steps. |
| POST | `/api/v1/saga/compose` | **Body:** Goal (Transfer, Attest, Delegate, …) + Constraints. Response: komponierte Saga (Steps), gas/mana-Schätzung. |
| POST | `/api/v1/saga/execute` | **Body:** Saga-ID + Kontext. Führt Saga aus (Crossing-Checks, Steps); StateEvents (SagaProgress, CrossingEvaluated, …). |
| GET | `/api/v1/saga/stats` | SagaComposerSnapshot (sagas_composed, successful_compositions, compensations_executed, …). |

---

## 14. State-Mutation über Events (einheitlicher Pfad)

Für alle Mutationen, die state.rs abbilden, gilt: **Einmaliger Eintritt über StateEvent + log_and_apply.**

| Methode | Endpoint | Beschreibung |
|---------|----------|---------------|
| POST | `/api/v1/state/event` | **Body:** StateEvent (typisiert, JSON). Führt state.log_and_apply(event, parent_ids) aus. parent_ids optional (default []). Response: WrappedStateEvent (id, sequence, component). **Einschränkung:** Nur für vertrauenswürdige Clients/Admin; oder nur bestimmte Event-Typen erlauben (z. B. TrustUpdate, MembershipChange, ProposalCreated). |

**Sicherheit:**
- In Produktion: nur ausgewählte StateEvent-Varianten freigeben; Rest über fachliche Endpoints (z. B. POST /trust/update, POST /governance/proposals).
- Debug/Admin: volle StateEvent-Palette für Replay/Tests.

---

## 15. Connect-RPC Service-Entwurf (Kurz)

| Service | RPCs | Zweck |
|---------|------|--------|
| **StateService** | GetSnapshot, GetComponentSnapshot, GetMetrics, SubscribeDeltas | Introspection, Streaming |
| **HealthService** | Check, Ready, StateHealth, StateHealthDetail | Health (erweitert) |
| **InvariantService** | CheckInvariants, GetAggregateHealth | Invarianten, HealthReport |
| **EventLogService** | ListEvents, GetEvent, GetLogSnapshot, Replay | Event-Log, Replay |
| **CircuitBreakerService** | GetMode, SetMode, Reset | SystemMode |
| **CrossingService** | ValidateCrossing, GetStats | Gateway |
| **ECLService** | RunPolicy, RunApi, RunUi, RunDatalogic, RunGovernance, RunController, GetMetrics, Compile | ECL/ECLVM |
| **TrustService** | GetTrust, UpdateTrust | Trust |
| **GovernanceService** | CreateProposal, Vote, ListProposals | Governance |
| **RealmService** | ListRealms, GetRealm, CreateRealm, UpdateMembership | Realm |
| **SagaService** | ParseIntent, ComposeSaga, ExecuteSaga | Intent/Saga |
| **StateMutationService** | ApplyEvent | Generisches Event (Admin) |

---

## 16. Priorisierung & Phasen

| Phase | Fokus | Endpoints |
|-------|--------|-----------|
| **1** | Debug & Observability | State Snapshots (voll + Komponenten), Health/State, Invarianten, Event-Log (Lesen), Metrics, Warnings |
| **2** | Produktion Kern | Crossing/validate, ECL Entrypoints (api, ui, controller), Trust (read/update), Realm (read, membership) |
| **3** | Erweiterte Produktion | Governance (proposals, vote), Saga (parse, compose, execute), Intent |
| **4** | Ops & Recovery | Circuit Breaker (mode setzen), Replay, Checkpoints, State/event (Mutation) |
| **5** | Skalierung & Sync | Merkle/Delta, State-Stream (CQRS), Light-Client-Unterstützung |

---

## 17. Zusammenfassung

- **State als Single Source of Truth:** Alle Lese-APIs leiten aus UnifiedState/UnifiedSnapshot ab; alle Schreibpfade laufen über StateEvent + log_and_apply (direkt oder über fachliche Endpoints).
- **Debug:** Vollständige Introspection (Snapshots, Events, Replay, Invarianten, Health-Detail, Mode), ohne Produktionsdaten zu verändern.
- **Produktion:** Crossing, ECL (alle Entrypoints), Trust, Identity, Realm, Governance, Controller, Saga – einheitlich an state.rs und ECLVM angekoppelt.
- **Streaming:** State-Delta-Subscription für Echtzeit-UIs und DataLogic.
- **Sicherheit:** Admin-only für Mode, Replay, generisches ApplyEvent; fachliche Endpoints mit AuthZ (Controller/ECL).

**Referenzen:** state.rs (UnifiedState, UnifiedSnapshot, StateEvent, StateCoordinator, HealthReport, Invariant); ECL-HOST-STATE-CONTEXT.md; API-REFERENCE.md (Connect-RPC, REST-Fallbacks).
