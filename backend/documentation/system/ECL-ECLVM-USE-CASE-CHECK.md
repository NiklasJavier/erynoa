# ECL/ECLVM – Prüfung Anwendungsfall vs. Implementierung

Prüfung, ob alles unter ECLVM noch zum dokumentierten ECL-Anwendungsfall passt oder refaktoriert werden muss.

---

## 1. Dokumentierter ECL-Anwendungsfall

### 1.1 ECL Reference (ECL-REFERENCE.md)

- ECL = DSL für **programmatische Access Control** (Policies auf Trust, Credentials, Kontext).
- Policies → Bytecode → **ECLVM** (stack-based, gas-metered, sandboxed).
- Built-ins: `load_trust(did)`, `has_credential(did, schema)`, `resolve_did(did)`, `get_balance`, `get_timestamp`, store_get/put, trust_norm, trust_combine, etc.
- Typen: number, bool, string, did, trust_vector, array, null.

### 1.2 Use-Case-Matrix (concept-v5/03-SYSTEM-ARCHITECTURE.md)

| Use-Case              | ECL-Komponenten                                        |
|-----------------------|--------------------------------------------------------|
| Community-Forum       | structure + ui + api + governance(dao)                 |
| Kollab-Dokument       | structure + ui + datalogic + controller                |
| Marktplatz            | structure + ui + api + governance(optimistic)          |
| DAO Treasury          | controller(multisig) + governance(conviction)         |
| Identity-Provider     | api + policy + attestations                            |
| IoT-Gateway           | api(webhooks) + datalogic + automation                 |
| Event-Streaming       | api + datalogic + ui(reactive)                          |
| Governance-as-a-Service | governance(*) + api + ui                             |
| Supply-Chain          | attestations + api + datalogic                         |
| Multi-Tenant SaaS     | structure(partitions) + api + controller(tenant)      |

**Integration Flow (Dokumentation):**
External System → API → DataLogic → Governance → State Update; UI/Controller/ECLVM Runtime sind verbunden.

### 1.3 StateGraph (state.rs)

- **ECLVM** ist zentrale Komponente: UI, API, DataLogic, Governance, Controller, BlueprintComposer, Gateway, SagaComposer, ECLPolicy, ECLBlueprint **depend on** ECLVM.
- ECLVM **depends on** Identity, Gas, Mana, Trust; **triggers** Event; **aggregates** Execution.

---

## 2. Aktuelle Implementierung

### 2.1 eclvm-Modul

| Komponente           | Status | Anbindung |
|----------------------|--------|-----------|
| Parser / Compiler    | ✅     | ECL-Text → AST → Bytecode |
| Bytecode / OpCode    | ✅     | load_trust, has_credential, store_*, etc. |
| Runtime (VM)         | ✅     | Stack, IP, GasMeter, **HostInterface** |
| **HostInterface**    | ✅     | get_trust_vector, has_credential, resolve_did, get_balance, get_timestamp, store_get/put/nested/append, store_query_by_index, store_list_keys, set_store_context |
| **ErynoaHost**       | ✅     | **Nur DecentralizedStorage**: trust.compute_reputation(), identities.get(), realm.get_*/put_*/... |
| ProgrammableGateway  | ✅     | ECLVM + Host (ErynoaHost); evaluate() für Crossing-Entscheidung |
| ManaManager          | ✅     | Rate-Limiting pro Policy-Ausführung |
| StdLib / PolicyBuilder | ✅   | Convenience für Bytecode-Erstellung |

**Wichtig:** Das eclvm-Modul verwendet **weder** UnifiedState **noch** ECLVMStateContext, StateView oder StateHandle. Es verwendet **ausschließlich** HostInterface → ErynoaHost → DecentralizedStorage.

### 2.2 state.rs – ECLVM-relevante Teile

| Komponente           | Status | Verwendung |
|----------------------|--------|------------|
| **ECLVMState**       | ✅     | Metriken: policies_compiled/executed/passed/denied, blueprints_*, saga_*, crossing_*, total_gas/mana, events_emitted. Wird von **StateIntegrator** (ECLVMObserver) befüllt. |
| **ECLVMStateContext**| ✅     | StateView (Caches) + ECLVMBudget + create_write_handle() → StateHandle. Nur in **state.rs-Tests** und core-Re-Export verwendet. **Nicht** vom eclvm-Modul genutzt. |
| **StateView**        | ✅     | Read-only Caches (trust, realm, identity). Kein Zugriff auf echte Trust-Daten aus Storage/UnifiedState beim ersten Zugriff. |
| **StateHandle**      | ✅     | Write: update_trust, store_put, store_delete, emit_event; commit() → log_and_apply(). Wird nur über ECLVMStateContext erzeugt. |
| **ECLVMBudget**      | ✅     | Gas/Mana-Limits, consume_gas/mana, is_exhausted. Wird in ECLVMStateContext und StateHandle genutzt. |

### 2.3 state_integration – ECLVMObserver

- StateIntegrator implementiert ECLVMObserver: on_policy_executed, on_crossing_policy_evaluated, on_blueprint_*, on_intent_processed, on_saga_step_executed, on_ecl_gas/mana_consumed, etc. → schreiben in **state.eclvm.*** (ECLVMState).
- **Voraussetzung:** Irgendwer muss diese Observer **aufrufen**, nachdem eine Policy läuft oder ein Crossing entschieden wird.

### 2.4 Verbindung ECLVM ↔ State

- **ProgrammableGateway::evaluate()** führt ECLVM mit ErynoaHost aus. Es ruft **nicht** StateIntegrator.on_policy_executed() oder on_crossing_policy_evaluated() auf.
- **peer/gateway.rs** – GatewayGuard (Κ23) nutzt Trust, Rules, Credentials **ohne** ProgrammableGateway oder ECLVM. Es gibt also zwei getrennte Wege: (1) GatewayGuard (regelbasiert), (2) ProgrammableGateway (ECLVM). Sie sind **nicht** zusammengeführt.
- **ErynoaHost** hat keine Referenz auf UnifiedState oder StateIntegrator; nach Policy-Ausführung erfolgt **kein** automatischer Update von ECLVMState.

---

## 3. Lücken: Anwendungsfall vs. Implementierung

| Aspekt | Dokumentation / Use-Case | Implementierung | Bewertung |
|--------|---------------------------|-----------------|-----------|
| Policies (Trust, Credentials) | ECL-Policies, load_trust, has_credential | ErynoaHost → Storage (trust, identities); VM + HostInterface | ✅ **passend** für Policy-Ausführung gegen persistente Daten |
| Crossing (Gateway) | ECL für Realm-Entry/Crossing | ProgrammableGateway + ECLVM + ErynoaHost | ✅ **passend**, aber **nicht** in GatewayGuard (peer) integriert |
| UI / API / DataLogic / Governance / Controller | Sollen ECLVM als Runtime nutzen | Keine ECLVM-Runtime für API-Handler, UI-Logik, DataLogic, Governance, Controller | ❌ **nicht umgesetzt** |
| State-Metriken (ECLVMState) | ECLVM-Metriken im UnifiedState | ECLVMState wird nur befüllt, wenn jemand ECLVMObserver aufruft; ProgrammableGateway tut das **nicht** | ⚠️ **Lücke:** ECLVMState bleibt bei reinem ProgrammableGateway-Einsatz leer |
| ECLVMStateContext / StateView / StateHandle | „State-Abstraktion für ECLVM“ | Nur in state.rs und Tests; eclvm-Modul nutzt sie **nicht** | ⚠️ **Doppelweg:** Zwei Konzepte (Storage-Host vs. State-Context), nicht vereinheitlicht |
| GatewayGuard vs. ProgrammableGateway | Ein Gateway (Κ23) mit ECL-Option | Zwei getrennte Implementierungen, nicht verknüpft | ⚠️ **Lücke:** Kein einheitlicher Gateway-Pfad mit ECL |

---

## 4. Empfehlung

### 4.1 Enger Anwendungsfall: „Nur ECL für Crossing-Policies“

Wenn ECL **nur** für programmierbare Realm-Policies (Crossing) mit persistenter Datenbasis (Trust/Identity/Realm-Storage) genutzt wird:

- **Bewertung:** ECLVM-Pipeline (Parser → Compiler → VM), HostInterface, ErynoaHost, ProgrammableGateway sind **fachlich passend**.
- **Anpassungen (kein Voll-Refactor):**
  1. **Metriken anbinden:** Nach jeder Policy-Ausführung in ProgrammableGateway (oder an der Stelle, die ProgrammableGateway aufruft) StateIntegrator nutzen und z. B. `on_policy_executed(…)` / `on_crossing_policy_evaluated(…)` aufrufen, damit ECLVMState und Health/Invarianten gefüllt werden.
  2. **Gateway vereinheitlichen:** Entweder GatewayGuard (peer) ruft ProgrammableGateway auf (wenn Realm ECL-Policy hat), oder eine gemeinsame Fassade „Gateway“ wählt zwischen Regel- und ECL-basiertem Check. So ist Κ23 einheitlich und ECL optional nutzbar.
  3. **ECLVMStateContext/StateView/StateHandle:** Entweder als „Zukunft/Alternative“ dokumentieren (State-backed ECL-Kontext) oder, falls nicht geplant, schlank halten/entfernen, um Doppelweg zu vermeiden.

### 4.2 Vollständiger Anwendungsfall: „ECLVM als Runtime für UI, API, DataLogic, Governance, Controller“

Wenn die Use-Case-Matrix und der StateGraph so umgesetzt werden sollen, dass UI/API/DataLogic/Governance/Controller **tatsächlich** in ECLVM laufen:

- **Bewertung:** Dafür reicht die aktuelle Implementierung **nicht**; es braucht einen **gezielten Ausbau**, kein komplettes Neuschreiben der ECLVM.
- **Refactor/Erweiterung:**
  1. **Eintrittspunkte:** API-Handler, UI-Logik, DataLogic-Funktionen, Governance-Regeln, Controller-Permission-Rules als ECL/Bytecode definieren und über eine gemeinsame „ECLVM::run_with_host(…)“-Art aufrufen (weiterhin mit HostInterface).
  2. **Host:** Entweder ErynoaHost erweitern (z. B. UnifiedState/StateView für Lese-Metriken, weiterhin Storage für Trust/Identity/Realm), oder ein zweites HostInterface-Implementierung für „State-only“-Kontext (StateView + StateHandle), je nachdem ob Persistenz immer über Storage laufen soll.
  3. **ECLVMStateContext:** Wenn API/UI/DataLogic mit UnifiedState reden sollen, ECLVMStateContext (oder äquivalent) als Kontext an ECLVM übergeben und Host so bauen, dass er aus diesem Kontext (StateView + ggf. StateHandle) liest/schreibt. Dann ErynoaHost und „StateHost“ sauber trennen oder kombinierbar machen.
  4. **Gateway:** Wie oben – eine gemeinsame Gateway-Schicht, die bei Bedarf ProgrammableGateway (ECL) nutzt.

---

## 5. Kurzfassung

| Frage | Antwort |
|-------|--------|
| Ist die ECLVM-Logik (Parser, Compiler, VM, Host, Policies) für den **engen** Anwendungsfall (Crossing-Policies, Trust/Credentials/Realm) passend? | **Ja.** Kein kompletter Refactor nötig. |
| Ist alles unter ECLVM **vollständig** für den **dokumentierten** Use-Case (UI, API, DataLogic, Governance, Controller in ECLVM) passend? | **Nein.** Dafür fehlen Eintrittspunkte, Host-Erweiterung und ggf. Nutzung von ECLVMStateContext. |
| Konkrete nächste Schritte ohne Voll-Refactor | (1) ProgrammableGateway (oder Aufrufer) mit StateIntegrator verbinden (on_policy_executed / on_crossing_policy_evaluated). (2) GatewayGuard und ProgrammableGateway zu einem gemeinsamen Gateway-Pfad zusammenführen. (3) ECLVMStateContext/StateView/StateHandle entweder für künftige State-backed ECL-Nutzung dokumentieren oder abbauen. |
| Wann vollständiger Refactor? | Wenn ihr die Use-Case-Matrix voll umsetzen wollt (ECLVM als Runtime für UI, API, DataLogic, Governance, Controller). Dann gezielter Ausbau (Eintrittspunkte + Host + ggf. ECLVMStateContext), nicht „alles unter eclvm neu bauen“. |

---

**Stand:** Prüfung basierend auf ECL-REFERENCE.md, concept-v5 Use-Case-Matrix, state.rs (ECLVMState, ECLVMStateContext, StateView, StateHandle), eclvm (ErynoaHost, ProgrammableGateway, HostInterface), state_integration (ECLVMObserver), peer/gateway.rs (GatewayGuard).
