# ECL Host vs. StateContext vs. StateHandle (Phase 4)

Klarstellung: Wann **ErynoaHost**, wann **ECLVMStateContext/StateView**, wann **StateHandle** nutzen.

---

## Übersicht

| Komponente | Rolle | Wann nutzen |
|------------|--------|-------------|
| **ErynoaHost** | Produktion: Storage (Trust, Identity, Realm-Storage) | Crossing, API, UI, DataLogic, Governance, Controller – immer wenn persistente Daten (Trust, Credentials, Realm-Storage) gelesen/geschrieben werden. |
| **ECLVMStateContext + StateView** | Optional: State-only oder Tests | ECL gegen UnifiedState-Snapshot ohne Storage; Tests; Lese-Metriken aus State. StateView per `refresh_from_snapshot` befüllbar. |
| **StateHandle** | Ephemere Schreibzugriffe auf UnifiedState | Nur für **Laufzeit-State-Mutation** (in-memory), **nicht** für persistente Realm-Daten. `store_put` → dirty keys; `commit()` → log_and_apply auf UnifiedState, **nicht** auf DecentralizedStorage. |

---

## Ein-Host-Strategie (Empfehlung)

- **Ein Host:** ErynoaHost bleibt der Standard-Host für alle ECL-Eintrittspunkte (Crossing, API, UI, DataLogic, Governance, Controller). Er spricht mit **DecentralizedStorage** (Trust, Identities, Realm-Storage).
- **StateView optional:** StateView aus UnifiedSnapshot befüllbar (Phase 2.3); kann künftig an ErynoaHost angebunden werden für Lese-Metriken aus State (z. B. `get_metric("trust.avg")`), ohne einen zweiten „StateHost“ einzuführen.
- **ECLVMStateContext:** Wird als **Kontext-Container** (caller_did, realm_id, Budget, StateView) genutzt, wenn ECL gegen State-only laufen soll (Tests oder spezielle Policies ohne Storage). In der Produktion bleibt **ErynoaHost** der Host; ECLVMStateContext kann trotzdem für Budget und Caller/Realm-Kontext verwendet werden, wobei der Host getrennt übergeben wird (z. B. in `run_policy(bytecode, host, context)`).

---

## Realm-Daten schreiben

- **Persistent (Realm-Storage):** Immer über **ErynoaHost.store_put** – schreibt in DecentralizedStorage/Realm-Storage.
- **Ephemer (nur UnifiedState):** **StateHandle.store_put** + am Ende **commit()** – aktualisiert nur den In-Memory-UnifiedState (log_and_apply), nicht den persistenten Storage.

---

## StateHandle in der ECL-Laufzeit (Gap 4)

- **Aktuell:** Kein ECL-Pfad (ProgrammableGateway, EclEntrypoints, Runner) erstellt oder nutzt einen **StateHandle**. Schreibzugriffe laufen ausschließlich über **ErynoaHost** (Storage).
- **Konsequenz:** ECL-Schreiben in der Produktion nur über ErynoaHost → DecentralizedStorage. **StateHandle** und „ephemere State-Mutationen“ über ECL (nur UnifiedState, ohne Storage) sind in der aktuellen Pipeline **nicht** angebunden.
- **Vorgesehen:** StateHandle ist für **künftige State-only-Schreib-Policies** gedacht (z. B. mit StateBackedHost erweitert um store_put über `ECLVMStateContext::create_write_handle()`). Bis dahin gilt: ECL-Schreiben nur über ErynoaHost → Storage; StateHandle optional dokumentiert und für spätere Integration vorbereitet.

---

## ECLVMBudget vs. Runner/VM (Gap 5)

- **Zwei Budget-Welten:** (1) **Runner/VM:** `PolicyRunContext.gas_limit` → VM-interner **GasMeter**; Mana wird in ProgrammableGateway separat über **ManaManager** abgezogen. (2) **state.rs:** **ECLVMBudget** in ECLVMStateContext (Gas/Mana/Timeout); StateView-Reads und StateHandle verbrauchen darüber.
- **Aktuell:** Runner, ProgrammableGateway und EclEntrypoints nutzen **nicht** ECLVMBudget. Produktion (ErynoaHost + run_policy) führt eigenes Gas (VM) und Mana (ManaManager).
- **Konsequenz:** ECLVMBudget gilt nur für **State-backed ECL** (ECLVMStateContext, z. B. StateBackedHost). Health/Invarianten, die auf ECLVMBudget in state.rs bauen, reflektieren die Produktions-ECL-Läufe nicht. Langfristig optional: ECLVMBudget in Runner-Kontext integrieren oder Rollen klar trennen (dokumentiert).

---

## Kurzfassung

- **ErynoaHost** = Produktion (Storage); Standard für alle ECL-Eintrittspunkte.
- **ECLVMStateContext + StateView** = optional State-only oder Tests; StateView aus Snapshot befüllbar.
- **StateHandle** = nur für ephemere Laufzeit-State-Änderungen; Realm-Daten dauerhaft → ErynoaHost.

**Stand:** Phase 4 (ECL-ECLVM-REFACTORING-PLAN).
