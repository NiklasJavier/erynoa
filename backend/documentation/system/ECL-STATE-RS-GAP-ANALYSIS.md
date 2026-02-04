# Gap-Analyse: state.rs-Vorstellungen vs. aktuelle ECL/ECLVM-Implementierung

**Stand:** Nach Umsetzung Gap 1–6 (duration_us, StateBackedHost, policies_by_type, StateHandle-Doku, ECLVMBudget-Doku, StateEvent::PolicyEvaluated).

Vergleich der in **state.rs** dokumentierten Architektur (ECLVMStateContext, StateView, StateHandle, ECLVMState) mit dem **aktuellen** Implementierungsstand. Alle zuvor identifizierten Gaps 1–6 sind adressiert; verbleibende Punkte sind bewusst dokumentierte Einschränkungen oder optionale Erweiterungen.

---

## 1. state.rs-Vorstellungen (Kurz)

### 1.1 Phase 6.4 – Zustandsabstraktion für ECLVM (state.rs ~13801–14720)

- **ECLVMStateContext:** Orchestriert State-Zugriff für ECLVM-Ausführung; kombiniert **StateView** (read), **StateHandle** (write), **ECLVMBudget** (Gas/Mana/Timeout). Doc: *„Verwendung durch ECLVM Host Interface“.*
- **StateView:** Read-only-Snapshot für Policy-Evaluation; `get_trust`, `get_realm`, `get_identity`; Caches aus Snapshot befüllbar (`refresh_from_snapshot`).
- **StateHandle:** Realm-scoped Schreibzugriff; `update_trust`, `store_put`, …; Änderungen über Event-Log; `commit()` → `log_and_apply` auf **UnifiedState** (in-memory), **nicht** auf DecentralizedStorage.
- **ECLVMState:** Metriken-Layer in UnifiedState (policies_executed, crossing_*, total_gas_consumed, avg_evaluation_time_us, policies_by_type, realm_ecl, …).

### 1.2 Erwartungen an ECLVM-Integration

- ECL-Ausführung kann über einen **Host** laufen, der an **ECLVMStateContext** (bzw. StateView/StateHandle) angebunden ist.
- ECLVMState wird bei jeder Policy-Ausführung konsistent befüllt (inkl. duration_us, policy_type, realm_id).
- Health-Score nutzt `eclvm.policy_success_rate()`, `eclvm.crossing_allow_rate()` usw. → diese sollten aus realen ECL-Läufen gespeist werden.

---

## 2. Was aktuell erfüllt ist

| Vorstellung (state.rs) | Status | Umsetzung |
|------------------------|--------|-----------|
| ECLVMState bei Policy-/Crossing-Läufen befüllt | ✅ | StateIntegrator (ECLVMObserver) wird von ProgrammableGateway und EclEntrypoints aufgerufen → `on_policy_executed` / `on_crossing_policy_evaluated` → `state.eclvm.policy_executed` / `crossing_policy_evaluated`. |
| duration_us / avg_evaluation_time_us | ✅ | ExecutionResult.duration_us; Runner setzt Dauer; Observer + StateIntegrator reichen duration_us durch; ECLVMState::policy_executed aktualisiert avg_evaluation_time_us. |
| policies_by_type nach Engine (API, UI, …) | ✅ | ECLPolicyType: Api, Ui, DataLogic, Controller, Governance, Crossing, …; StateIntegrator mappt policy_type-Strings; policy_executed schreibt policies_by_type. |
| StateView aus Snapshot befüllbar | ✅ | StateView::refresh_from_snapshot, from_unified_snapshot; ECLVMStateContext::refresh_view_from_snapshot. |
| Einheitlicher Crossing-Pfad (Regel + optional ECL) | ✅ | GatewayGuard + optional EclCrossingEvaluator; validate_crossing ruft nach Regel-Check optional ProgrammableGateway; Observer → crossing_policy_evaluated. |
| crossing_evaluations / crossings_allowed / crossings_denied | ✅ | StateIntegrator::on_crossing_policy_evaluated ruft eclvm.crossing_policy_evaluated(allowed, from_realm, to_realm); Health nutzt policy_success_rate() und crossing_allow_rate(). |
| Gemeinsame Laufzeit-Hülle für ECL | ✅ | eclvm::runtime::runner::run_policy + PolicyRunContext; ProgrammableGateway und EclEntrypoints nutzen sie. |
| Eintrittspunkte pro Engine (API, UI, DataLogic, Governance, Controller) | ✅ | EclEntrypoints mit run_api, run_ui, run_datalogic, run_governance, run_controller; Observer → ECLVMState. |
| Host an ECLVMStateContext (State-only ECL) | ✅ | StateBackedHost implementiert HostInterface über Arc<ECLVMStateContext> (Lese-Pfad); run_policy(bytecode, &state_host, &context) für State-only ECL möglich. |
| Klarstellung Host vs. StateContext vs. StateHandle vs. Budget | ✅ | ECL-HOST-STATE-CONTEXT.md: Ein-Host-Strategie, StateHandle/Budget-Rollen, ECLVMBudget vs. Runner/VM. |
| StateEvent::PolicyEvaluated aus ECL-Pfad | ✅ | StateIntegrator::on_policy_executed emittiert StateEvent::PolicyEvaluated via state.log_and_apply; apply_state_event → eclvm.policy_executed; Event-Subscriber und CQRS sehen ECL-Läufe. |

---

## 3. Abgeschlossene Gaps (Referenz)

Die folgenden sechs Gaps wurden umgesetzt bzw. dokumentativ geschlossen:

| Gap | Kurzbeschreibung | Lösung |
|-----|------------------|--------|
| **1** | duration_us / avg_evaluation_time_us | ExecutionResult.duration_us; Runner misst Dauer; Observer + policy_executed mit duration_us; GatewayDecision.duration_us. |
| **2** | ECLVMStateContext nicht als Host-Kontext | StateBackedHost (core::eclvm_state_host) implementiert HostInterface über ECLVMStateContext; nur Lese-Pfad, Store-Operationen NotSupported. |
| **3** | policies_by_type nur „Custom“ für API/UI/… | ECLPolicyType um Api, Ui, DataLogic, Controller erweitert; StateIntegrator-Mapping; policy_executed schreibt policies_by_type. |
| **4** | StateHandle in ECL-Laufzeit ungenutzt | In ECL-HOST-STATE-CONTEXT.md dokumentiert: ECL-Schreiben nur über ErynoaHost; StateHandle für künftige State-only-Schreib-Policies. |
| **5** | ECLVMBudget nicht an Runner/VM | In ECL-HOST-STATE-CONTEXT.md dokumentiert: Zwei Budget-Welten; ECLVMBudget nur für State-backed ECL; Produktion nutzt VM-Gas + ManaManager. |
| **6** | Kein StateEvent::PolicyEvaluated aus ECL-Pfad | StateIntegrator emittiert StateEvent::PolicyEvaluated via log_and_apply; einheitlicher Event-Pfad. |

---

## 4. Verbleibende / optionale Lücken

Diese Punkte sind **keine offenen Defizite**, sondern bewusst dokumentierte Einschränkungen oder optionale Erweiterungen für später.

| Thema | Aktueller Stand | Optionale nächste Schritte |
|-------|-----------------|-----------------------------|
| **StateHandle in ECL-Laufzeit** | Nicht genutzt; ECL-Schreiben nur über ErynoaHost → Storage. | StateBackedHost um store_put über ECLVMStateContext::create_write_handle() erweitern; Eintrittspunkt „run mit StateContext + StateHost“ für ephemere State-Mutationen. |
| **ECLVMBudget in Runner/VM** | Runner/ProgrammableGateway nutzen VM-Gas + ManaManager; ECLVMBudget nur bei State-backed ECL (StateBackedHost). | Optional: ECLVMBudget in PolicyRunContext integrieren, sodass VM-Gas und state.rs-Budget synchron laufen; oder Rollen weiter klar trennen. |
| **StateBackedHost store_*** | StateBackedHost gibt für alle store_*-Methoden NotSupported zurück. | Für State-only-Schreib-Policies: StateHandle in StateBackedHost einbinden (create_write_handle pro Run oder pro Kontext). |

---

## 5. Zusammenfassungstabelle

| Bereich | Erfüllt | Anmerkung |
|---------|---------|-----------|
| ECLVMState-Metriken (policies_executed, duration_us, policies_by_type, crossing_*) | ✅ | Vollständig aus ECL-Pipeline befüllt. |
| Event-Pfad (StateEvent::PolicyEvaluated) | ✅ | ECL-Pfad emittiert über log_and_apply. |
| StateView / ECLVMStateContext | ✅ | Snapshot-befüllbar; StateBackedHost nutzt Kontext. |
| StateHandle / ECLVMBudget in Produktion | 📄 | Dokumentiert als „nur State-backed ECL / künftig“. |
| Health/Invarianten (policy_success_rate, crossing_allow_rate) | ✅ | Werte aus realen ECL-Läufen. |

---

## 6. Abgleich mit state.rs-Architektur

- **ECLVMState:** Wird von der ECL-Pipeline (ProgrammableGateway, EclEntrypoints) über StateIntegrator befüllt; zusätzlich wird **StateEvent::PolicyEvaluated** emittiert und von apply_state_event verarbeitet. Metriken (duration_us, policies_by_type, crossing_*, avg_evaluation_time_us) sind konsistent.
- **StateView:** Aus Snapshot befüllbar; in ECLVMStateContext integriert; wird von StateBackedHost für State-only ECL genutzt. Produktion (ErynoaHost) nutzt weiterhin Storage.
- **StateHandle / ECLVMStateContext:** StateHandle in der aktuellen ECL-Laufzeit ungenutzt (dokumentiert); ECLVMStateContext wird von StateBackedHost als Lese-Kontext genutzt. ECLVMBudget gilt nur für State-backed ECL (dokumentiert).

**Stand:** Gap-Analyse nach Umsetzung Gap 1–6. Keine offenen inhaltlichen Lücken; verbleibende Punkte sind optionale Erweiterungen oder bewusste Architekturentscheidungen (ECL-HOST-STATE-CONTEXT.md).
