# Erynoa REST-API – Beschreibung

> **Version:** 1.0
> **Basis-URL:** `/api/v1`
> **Protokoll:** REST (JSON)
> **State-getrieben:** UnifiedState, StateEvent, log_and_apply

Diese Beschreibung deckt alle REST-Endpoints der State-getriebenen API (Phasen 1–5) ab.

---

## 1. Übersicht

| Präfix | Domäne | Zweck |
|--------|--------|--------|
| *(root)* | Rest | Health, Ready, Info, Status |
| `/auth` | Auth | Passkey/WebAuthn |
| `/state` | State | Snapshots, Metriken, Warnings, Mode, Circuit Breaker, Event, Merkle, Delta, Proof, Stream |
| `/health` | Health | State-Health, Aggregate |
| `/events` | Events | Event-Log, Checkpoints |
| `/invariants` | Invariants | Invarianten-Checks |
| `/crossing` | Crossing | Realm-Crossing validieren |
| `/trust` | Trust | Trust lesen/aktualisieren |
| `/identity` | Identity | Root-DID, DID-Infos |
| `/realms` | Realms | Realm-Liste, CRUD, Members, ECL |
| `/ecl` | ECL | Policy ausführen, Entrypoints (Stubs) |
| `/governance` | Governance | Proposals, Vote |
| `/controller` | Controller | AuthZ-Check, Permissions |
| `/intent` | Intent | Intent parsen (Stub) |
| `/saga` | Saga | Compose, Execute, Stats |
| `/debug` | Debug | Replay, Checkpoint |

---

## 2. Rest & Probes

| Methode | Endpoint | Beschreibung |
|---------|----------|--------------|
| GET | `/api/v1/health` | Liveness (immer healthy) |
| GET | `/api/v1/ready` | Readiness inkl. Storage |
| GET | `/api/v1/info` | Version, Environment, Auth-Info |
| GET | `/api/v1/status` | Service-Status-Übersicht |

---

## 3. Auth (Passkey/WebAuthn)

| Methode | Endpoint | Beschreibung |
|---------|----------|--------------|
| GET | `/api/v1/auth/challenge` | 32-Byte-Challenge (5 Min gültig) |
| POST | `/api/v1/auth/passkey/register` | Passkey registrieren (credential_id, public_key) |
| POST | `/api/v1/auth/passkey/verify` | Signatur verifizieren (credential_id, signature, …) |

---

## 4. State (Introspection, Mode, Merkle, Stream)

Basis: `UnifiedState` / `UnifiedSnapshot`. Keine Mutationen außer explizit genannte Endpoints.

### 4.1 Snapshots

| Methode | Endpoint | Query | Beschreibung |
|---------|----------|-------|--------------|
| GET | `/api/v1/state/snapshot` | `?components=core,eclvm` (optional), `?realm_id=` (optional) | Vollständiger **UnifiedSnapshot** oder gefiltert nach Komponenten |
| GET | `/api/v1/state/:component_name` | – | Einzelner Komponenten-Snapshot; `component_name`: identity, core, execution, eclvm, protection, storage, peer, p2p, ui, api, governance, controller, data_logic, blueprint_composer, event_log, event_bus, circuit_breaker, broadcaster, merkle_tracker, multi_gas |

### 4.2 Metriken

| Methode | Endpoint | Beschreibung |
|---------|----------|--------------|
| GET | `/api/v1/state/metrics` | Key-Value-Metriken (Prometheus-Format): health_score, timestamp_ms, uptime_secs, events_sequence, events_buffer_size |
| GET | `/api/v1/state/metrics/eclvm` | ECLVM-Metriken (policies_compiled, policies_executed, policy_success_rate, gas, mana, out_of_gas_aborts) |
| GET | `/api/v1/state/metrics/health` | Health-Metriken (overall_score, status, invariant_score, module_scores) |

### 4.3 Warnings

| Methode | Endpoint | Beschreibung |
|---------|----------|--------------|
| GET | `/api/v1/state/warnings` | Liste aktiver Warnings |
| DELETE | `/api/v1/state/warnings` | Alle Warnings löschen |
| DELETE | `/api/v1/state/warnings/:key` | Warnings mit Key-Prefix löschen |

### 4.4 System Mode & Circuit Breaker

| Methode | Endpoint | Body (POST) | Beschreibung |
|---------|----------|------------|--------------|
| GET | `/api/v1/state/mode` | – | Aktueller **SystemMode** (Normal, Degraded, EmergencyShutdown) |
| POST | `/api/v1/state/mode` | `{ "mode": "Normal" \| "Degraded" \| "EmergencyShutdown" }` | Mode setzen (Ops/Admin) |
| POST | `/api/v1/state/mode/reset` | – | Zurück auf Normal |
| GET | `/api/v1/state/circuit_breaker` | – | **CircuitBreakerSnapshot** (mode, transitions, last_anomaly, …) |

### 4.5 State-Event (Mutation)

| Methode | Endpoint | Body | Beschreibung |
|---------|----------|------|--------------|
| POST | `/api/v1/state/event` | **StateEvent** (JSON, typisiert) | `log_and_apply(event)`; Response: WrappedStateEvent (id, sequence, component). Admin/Debug. |

### 4.6 Merkle & Delta (Phase 5 – Light-Clients)

| Methode | Endpoint | Query/Path | Beschreibung |
|---------|----------|------------|--------------|
| GET | `/api/v1/state/merkle/root` | – | Aktueller Merkle-Root (hex) |
| GET | `/api/v1/state/merkle/component/:component` | – | Merkle-Hash einer Komponente (z. B. trust, eclvm, core, peer). Response: `{ "component", "hash" }` |
| GET | `/api/v1/state/delta` | `?since_root=<hex>`, `?since_sequence=<n>` | Deltas seit angegebenem Root oder Sequenz; Response: `{ "root", "deltas": [ { old_root, new_root, component, proof_path, data_base64, timestamp_ms, sequence } ] }` |
| GET | `/api/v1/state/proof/:component` | – | State-Proof für Komponente: `{ "root", "component", "component_hash", "proof_path" }` (gegen Root verifizierbar) |

### 4.7 State-Stream (CQRS / SSE)

| Methode | Endpoint | Query | Beschreibung |
|---------|----------|-------|--------------|
| GET | `/api/v1/state/stream` | `?components=trust,eclvm` (optional) | **Server-Sent Events**: Subscription auf State-Deltas; jede Änderung (log_and_apply) liefert ein **StateDelta** (JSON). Optional nur bestimmte Komponenten. |

---

## 5. Health

| Methode | Endpoint | Beschreibung |
|---------|----------|--------------|
| GET | `/api/v1/health/state` | State-Health-Score (0–100) aus `state.calculate_health()` |
| GET | `/api/v1/health/state/detail` | Detaillierte Aufschlüsselung (module_scores, invariant_summary) |
| GET | `/api/v1/health/aggregate` | **HealthReport**: score, status, invariant_results, module_scores, can_transact, recommendation |

---

## 6. Invariants

| Methode | Endpoint | Query | Beschreibung |
|---------|----------|-------|--------------|
| GET | `/api/v1/invariants` | `?severity=critical` (optional) | Liste **InvariantResult** (invariant, passed, current_value, threshold, message). Optional Filter nach Severity. |

---

## 7. Events (Event-Log)

| Methode | Endpoint | Query/Path | Beschreibung |
|---------|----------|-------------|---------------|
| GET | `/api/v1/events` | `?limit=100`, `?since_sequence=12345`, `?component=Trust`, `?realm_id=` | Letzte N Events (WrappedStateEvent[]); Filter nach Komponente/Realm |
| GET | `/api/v1/events/:sequence` | – | Einzelnes Event nach Sequenznummer |
| GET | `/api/v1/events/log/snapshot` | – | **EventLogSnapshot** (sequence, buffer_size, total_events, …) |
| GET | `/api/v1/events/checkpoints` | – | Letzter Checkpoint: last_checkpoint_sequence, current_sequence, id, state_hash_hex |

---

## 8. Crossing (Gateway)

| Methode | Endpoint | Body | Beschreibung |
|---------|----------|------|---------------|
| POST | `/api/v1/crossing/validate` | `{ "caller_did", "from_realm", "to_realm" }` | **GatewayGuard.validate_crossing**; Response: allowed, reason, trust_score, gas_used, duration_us, policy_name. 503 wenn Gateway nicht konfiguriert. |
| GET | `/api/v1/crossing/stats` | – | **GatewaySnapshot**: crossings_total, crossings_allowed, crossings_denied, success_rate, avg_crossing_trust, trust_violations, credential_violations |

---

## 9. Trust

| Methode | Endpoint | Body (POST) | Beschreibung |
|---------|----------|-------------|--------------|
| GET | `/api/v1/trust/:did` | – | Trust-Vektor (6D) oder TrustRecord für DID |
| POST | `/api/v1/trust/update` | TrustUpdate-Parameter (entity_id, delta, reason, from_realm, …) | Erzeugt **StateEvent::TrustUpdate** und log_and_apply |

---

## 10. Identity

| Methode | Endpoint | Beschreibung |
|---------|----------|--------------|
| GET | `/api/v1/identity/root` | Root-DID und Basis-Infos |
| GET | `/api/v1/identity/:did` | Identity-Infos für DID (DID Document, realms, delegations) |

---

## 11. Realms

| Methode | Endpoint | Body (POST) | Beschreibung |
|---------|----------|-------------|---------------|
| GET | `/api/v1/realms` | – | Realm-Liste (RealmSnapshot-ähnlich) |
| GET | `/api/v1/realms/:realm_id` | – | Einzelnes Realm inkl. Rules, Members, ECL-Info |
| POST | `/api/v1/realms` | Realm-Lifecycle-Parameter | Realm anlegen; **StateEvent::RealmLifecycle** |
| POST | `/api/v1/realms/:realm_id/members` | Membership-Parameter | Membership ändern; **StateEvent::MembershipChange** |
| GET | `/api/v1/realms/:realm_id/ecl` | – | **RealmECLSnapshot** für dieses Realm |

---

## 12. ECL/ECLVM

| Methode | Endpoint | Body | Beschreibung |
|---------|----------|------|--------------|
| POST | `/api/v1/ecl/run` | bytecode_base64, caller_did, realm_id, gas_limit, policy_type | Policy ausführen (Stub: 501) |
| POST | `/api/v1/ecl/api/:route_id` | caller_did, realm_id, gas_limit (optional) | ECL API-Entrypoint (Stub: 501) |
| POST | `/api/v1/ecl/ui/:component_id` | caller_did, realm_id, gas_limit (optional) | ECL UI-Entrypoint (Stub: 501) |
| POST | `/api/v1/ecl/controller/:key` | caller_did, realm_id, gas_limit (optional) | ECL Controller-Entrypoint (Stub: 501) |

---

## 13. Governance

| Methode | Endpoint | Body | Beschreibung |
|---------|----------|------|--------------|
| POST | `/api/v1/governance/proposals` | Proposal-Parameter | **StateEvent::ProposalCreated** + log_and_apply |
| POST | `/api/v1/governance/proposals/:id/vote` | Vote-Parameter | **StateEvent::VoteCast** |
| GET | `/api/v1/governance/proposals` | – | Liste Proposals (aggregiert aus **GovernanceSnapshot**) |

---

## 14. Controller

| Methode | Endpoint | Body (POST) | Beschreibung |
|---------|----------|-------------|--------------|
| POST | `/api/v1/controller/check` | permission, resource, caller_did, realm_id | AuthZ-Check (zählt Check, Standard: erlaubt; bis Policy-Engine angebunden) |
| GET | `/api/v1/controller/permissions` | – | Permissions für Realm/Caller (**ControllerSnapshot**) |

---

## 15. Intent & Saga

| Methode | Endpoint | Body | Beschreibung |
|---------|----------|------|--------------|
| POST | `/api/v1/intent/parse` | Intent-Text oder Goal | Intent parsen (Stub: 501) |
| POST | `/api/v1/saga/compose` | Goal + Constraints | Saga komponieren (Stub: 501) |
| POST | `/api/v1/saga/execute` | Saga-ID + Kontext | Saga ausführen; **StateEvent::SagaProgress** |
| GET | `/api/v1/saga/stats` | – | **SagaComposerSnapshot** (sagas_composed, successful_compositions, compensations_executed, …) |

---

## 16. Debug (Replay & Checkpoints)

| Methode | Endpoint | Body | Beschreibung |
|---------|----------|------|--------------|
| POST | `/api/v1/debug/replay` | `{ "from_sequence", "to_sequence" }` | Replay von Events auf State-Kopie; Response: angewendete Anzahl, Fehler falls apply fehlschlägt. Admin/Non-Production. |
| POST | `/api/v1/debug/replay/checkpoint` | – | Replay ab letztem Checkpoint bis aktueller Sequenz |
| POST | `/api/v1/debug/checkpoint` | – | Checkpoint manuell auslösen (`state.event_log.mark_checkpoint`) |

---

## 17. Fehlerbehandlung

| HTTP | Bedeutung |
|------|------------|
| 200 | Erfolg |
| 400 | Ungültige Anfrage (z. B. ungültiges JSON/Body) |
| 404 | Ressource nicht gefunden (z. B. unbekannte Komponente, DID, Realm) |
| 501 | Nicht implementiert (z. B. ECL-Entrypoints, Intent/Saga-Stubs) |
| 503 | Service nicht verfügbar (z. B. Gateway nicht konfiguriert) |

JSON-Fehlerantworten enthalten typischerweise `{ "error": "…", "detail": "…" }` oder ähnliche Felder.

---

## 18. Referenzen

- **State-Plan:** `backend/documentation/system/API-PLAN-STATE-DRIVEN.md`
- **Connect-RPC/Proto:** `documentation/system/reference/API-REFERENCE.md`
- **Implementierung:** `backend/src/api/routes.rs`, `backend/src/api/v1/state_handlers.rs`, `production_handlers.rs`, `debug_handlers.rs`

---

_Erstellt: Februar 2026 | Basis: State-getriebene API Phasen 1–5_
