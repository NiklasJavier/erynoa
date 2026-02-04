# ECL/ECLVM – Vollständiger Refactoring-Plan

Plan, um den **vollständigen ECL-Anwendungsfall** umzusetzen: ECLVM als Runtime für Crossing-Policies, API, UI, DataLogic, Governance und Controller; einheitlicher Gateway-Pfad; State-Metriken und State-backed ECL konsistent angebunden.

Basis: [ECL-ECLVM-USE-CASE-CHECK.md](./ECL-ECLVM-USE-CASE-CHECK.md).

---

## Übersicht Phasen

| Phase | Inhalt | Abhängigkeiten | Grober Aufwand |
|-------|--------|-----------------|----------------|
| **1** | Metriken + Gateway vereinheitlichen | – | klein |
| **2** | Host + State-Integration (Observer, optional StateView) | Phase 1 | mittel |
| **3** | Eintrittspunkte ECLVM (API, UI, DataLogic, Governance, Controller) | Phase 2 | groß |
| **4** | ECLVMStateContext / StateView / StateHandle klären | Phase 2, 3 | mittel |

---

## Phase 1: Metriken + Gateway vereinheitlichen

**Ziel:** ECLVMState wird bei jeder Policy-/Crossing-Ausführung befüllt; Κ23 hat einen einheitlichen Pfad (regelbasiert oder ECL).

### 1.1 ProgrammableGateway → StateIntegrator

- **Wo:** Aufrufer von `ProgrammableGateway::evaluate()` oder direkt in `ProgrammableGateway::evaluate()` (wenn Host/Callback übergeben wird).
- **Was:**
  - Nach jeder Policy-Ausführung: `StateIntegrator::on_policy_executed(passed, policy_type, gas_used, mana_used, duration_us, realm_id)` aufrufen.
  - Nach jeder Crossing-Entscheidung: `StateIntegrator::on_crossing_policy_evaluated(from_realm, to_realm, entity, allowed, trust_score, policy_id)` aufrufen.
- **Option A:** ProgrammableGateway bekommt optional `observer: Option<Arc<dyn ECLVMObserver>>`; nach `evaluate()` ruft es observer.on_policy_executed/on_crossing_policy_evaluated auf.
- **Option B:** Die Stelle, die ProgrammableGateway nutzt (z. B. eine zentrale „CrossingService“-Fassade), ruft nach evaluate() den StateIntegrator auf.
- **Ergebnis:** ECLVMState (policies_executed, crossing_*, etc.) und Health/Invarianten reflektieren echte Policy-Läufe.

### 1.2 Gateway vereinheitlichen (Κ23)

- **Ausgangslage:** `peer/gateway.rs` = GatewayGuard (regelbasiert: min_trust, rules, credentials). `eclvm/programmable_gateway.rs` = ProgrammableGateway (ECLVM + ErynoaHost). Getrennt, nicht verknüpft.
- **Ziel:** Ein Crossing-Pfad: „Gateway“ prüft zuerst Regel-Basis (min_trust, credentials); wenn Realm eine ECL-Entry-Policy hat, zusätzlich ProgrammableGateway ausführen.
- **Vorgehen:**
  1. **GatewayGuard erweitern** (oder neue Fassade `UnifiedGateway`):
     - Optional: `programmable: Option<Arc<ProgrammableGateway<ErynoaHost>>>` + `realm_entry_policies: HashMap<RealmId, String>` (Policy-Name pro Realm).
     - `validate_crossing(did, from_realm, to_realm)`:
       1. Wie bisher: min_trust, rules, credentials prüfen.
       2. Wenn `programmable` gesetzt und für `to_realm` eine Entry-Policy existiert: `ProgrammableGateway::evaluate(sender_did, from_realm, to_realm)` aufrufen; nur bei allow durchlassen.
  2. **ErynoaHost/Storage-Kontext:** UnifiedGateway bzw. GatewayGuard braucht für ProgrammableGateway Zugriff auf Host (ErynoaHost mit DecentralizedStorage) oder eine Factory, die Host mit realm_id + caller_did erzeugt. AppState (unified_state + storage) ist der natürliche Ort; beim Start UnifiedGateway mit storage + optional programmable bauen.
  3. **SagaComposer / andere Aufrufer:** Statt nur GatewayGuard zu nutzen, die vereinheitlichte Gateway-Fassade nutzen (eine Methode `validate_crossing` / `allow_crossing`, die intern Regel-Check + ggf. ECL kombiniert).
- **Ergebnis:** Κ23 einheitlich; ECL optional pro Realm; eine Stelle ruft ProgrammableGateway auf und kann dort auch Observer aufrufen (siehe 1.1).

### 1.3 Abnahmekriterien Phase 1

- [x] Jeder Aufruf von ProgrammableGateway.evaluate (bzw. der vereinheitlichten Gateway-Validierung) führt zu on_policy_executed / on_crossing_policy_evaluated.
- [x] ECLVMState-Zähler (policies_executed, crossing_*, etc.) steigen bei echten Crossing-Checks mit ECL.
- [x] GatewayGuard und ProgrammableGateway sind über eine gemeinsame Fassade oder Erweiterung des GatewayGuard genutzt; kein doppelter „Crossing“-Pfad mehr.

### 1.4 Umsetzung (Stand)

- **1.1** `PolicyExecutionObserver` in eclvm; `ProgrammableGateway::with_observer()`, `GatewayDecision` um `gas_used`/`mana_used`; nach `validate_crossing` Aufruf von `on_policy_executed` und `on_crossing_policy_evaluated`. `ECLVMObserverAdapter` in core verbindet StateIntegrator mit dem Trait.
- **1.2** `EclCrossingEvaluator` in eclvm; `ProgrammableGateway` implementiert es. `GatewayGuard::with_ecl_evaluator()`, `register_realm_ecl_entry()`; `validate_crossing` ruft nach Regel-Check optional ECL-Evaluator auf.
- **Wiring in App:** Beim Aufbau des Gateways (z. B. in Server/SagaComposer): `ProgrammableGateway::new(Arc::new(ErynoaHost::new(storage))).with_observer(Arc::new(ECLVMObserverAdapter::new(integrator)))`; `GatewayGuard::default().with_ecl_evaluator(Arc::new(programmable_gateway))`; für Realms mit ECL-Entry `guard.register_realm_ecl_entry(realm_id)`.

---

## Phase 2: Host + State-Integration

**Ziel:** ECLVM-Host kann optional UnifiedState/StateIntegrator einbeziehen (Metriken, Lese-Metriken aus State); klare Verantwortung Storage vs. State.

### 2.1 ErynoaHost + StateIntegrator (Metriken-Callback)

- **Option A (minimal):** ErynoaHost bekommt optional `observer: Option<Arc<dyn ECLVMObserver>>`. In jeder Host-Methode, die im Policy-Pfad genutzt wird (z. B. nach get_trust_vector, has_credential), wird nicht getriggert; stattdessen nur **einmal** nach Policy-Ende in ProgrammableGateway (wie Phase 1.1). Keine Änderung an ErynoaHost nötig, wenn Phase 1.1 am Aufrufer umgesetzt ist.
- **Option B (zentral im Host):** ErynoaHost erhält `observer: Option<Arc<dyn ECLVMObserver>>` und eine Methode `notify_policy_finished(passed, policy_type, gas, mana, duration_us, realm_id)`, die von ProgrammableGateway nach VM.run() aufgerufen wird (ProgrammableGateway braucht dann Referenz auf Observer). Dann bleibt „Wer führt Policy aus?“ die einzige Stelle, die den Observer aufruft – entweder Gateway-Fassade oder ProgrammableGateway selbst.
- **Empfehlung:** Phase 1.1 umsetzen (Observer-Aufruf nach evaluate()); ErynoaHost vorerst unverändert. Falls später ECLVM auch an anderen Stellen (API, UI, …) läuft, kann eine gemeinsame „run_policy_and_notify(…)“-Hülle den Observer zentral aufrufen.

### 2.2 Host-Erweiterung: Lese-Metriken aus UnifiedState (optional)

- **Wenn** ECL-Policies nicht nur Storage (Trust, Identity, Realm) lesen, sondern auch Laufzeit-Metriken (z. B. aus UnifiedState) brauchen:
  - HostInterface um optionale Methoden erweitern (z. B. `get_metric(name) -> Option<f64>`) oder ErynoaHost um ein `state_snapshot: Option<UnifiedSnapshot>` (periodisch/on-demand gefüllt) erweitern.
  - ErynoaHost bei Erstellung mit `SharedUnifiedState` versorgen; vor Policy-Run oder in get_metric Snapshot holen und Metrik aus Snapshot liefern.
- **Wenn** nur Storage reicht: entfällt.

### 2.3 StateView mit echten Daten füllen (für State-backed ECL)

- **Problem:** StateView (state.rs) hält nur Caches; get_trust/get_realm/get_identity liefern nur vorher per set_trust_cached/set_realm_cached gesetzte Werte. Kein automatisches Befüllen aus Storage oder UnifiedState.
- **Ziel (falls State-backed ECL genutzt werden soll):** ECLVMStateContext/StateView als Lese-Kontext für ECL, der aus einer autoritativen Quelle kommt (Storage oder UnifiedState).
- **Vorgehen:**
  1. **StateView.from_unified_snapshot** erweitern: Aus UnifiedSnapshot trust/realm/identity-relevante Felder in die Caches von StateView übernehmen (z. B. aus snapshot.core.trust, snapshot.peer.realm, identity-Snapshot), sodass get_trust/get_realm/get_identity ohne manuelles set_* sinnvolle Werte liefern.
  2. **Oder:** StateView erhält eine Methode `refresh_from(state: &UnifiedState)` bzw. `refresh_from_snapshot(snapshot: &UnifiedSnapshot)`, die Caches aus State/Snapshot befüllt. ECLVMStateContext ruft refresh vor der ersten ECL-Ausführung auf.
- **Ergebnis:** StateView wird zur echten „Read-View“ auf State (oder Snapshot); ECLVMStateContext kann für State-only-Evaluation genutzt werden (ohne Storage), wenn gewünscht.

### 2.4 Abnahmekriterien Phase 2

- [x] Nach jeder Policy-Ausführung (Crossing und später andere Eintrittspunkte) ist ECLVMState konsistent befüllt (Phase 1 + ggf. zentrale run_policy_and_notify).
- [x] Optional: ErynoaHost kann bei Bedarf Laufzeit-Metriken aus UnifiedState/Snapshot liefern (HostInterface um `get_metric(name) -> Option<f64>` mit Default `None` erweitert; Wrapper mit State später in core möglich).
- [x] Optional: StateView kann aus UnifiedState/Snapshot befüllt werden; ECLVMStateContext liefert dann sinnvolle Read-Values ohne manuelles Caching.

### 2.5 Umsetzung (Stand)

- **2.1** ErynoaHost unverändert; Metriken-Callback wie in Phase 1.1 über ProgrammableGateway/Observer.
- **2.2** HostInterface um optionale Methode `get_metric(name: &str) -> Option<f64>` erweitert (Default: `None`). Später: Wrapper in core mit SharedUnifiedState für State-Metriken.
- **2.3** StateView: `refresh_from_snapshot(&mut self, snapshot: &UnifiedSnapshot)` befüllt trust_cache (Caller + avg_trust), realm_cache (aus snapshot.peer.realm.realms), identity_cache (root_did + avg_trust). `from_unified_snapshot` ruft refresh_from_snapshot auf. ECLVMStateContext: `refresh_view_from_snapshot(&mut self, snapshot: &UnifiedSnapshot)` ruft view.refresh_from_snapshot auf.

---

## Phase 3: Eintrittspunkte ECLVM (API, UI, DataLogic, Governance, Controller)

**Ziel:** API-Handler, UI-Logik, DataLogic-Funktionen, Governance-Regeln und Controller-Permission-Rules können als ECL/Bytecode definiert und in der ECLVM ausgeführt werden.

### 3.1 Gemeinsame Laufzeit-Hülle

- **Komponente:** `eclvm::runtime::Runner` (oder ähnlich) mit Signatur in der Art:
  - `run_policy(bytecode, host, context: PolicyContext) -> Result<Value, InterpretError>`
  - `PolicyContext`: caller_did, realm_id, gas_limit, mana_limit, optional request_id / component_id.
- **Verhalten:** VM mit bytecode + host starten; StoreContext setzen (realm_id, caller_did); Gas/Mana aus context; nach Run Observer aufrufen (on_policy_executed mit passed, gas, mana, duration_us, realm_id). So wird jede ECL-Ausführung einheitlich gemessen und geloggt.
- **Wiederverwendung:** ProgrammableGateway nutzt diese Hülle intern; API/UI/DataLogic/Governance/Controller rufen sie mit ihrem Bytecode und Context auf.

### 3.2 API-Engine

- **Ziel:** Bestimmte API-Endpoints können als ECL-Programm (Bytecode) registriert werden; Request wird in VM-Kontext (caller_did, realm_id, body als Input) übergeben, Ergebnis als Response.
- **Schritte:**
  1. API-Layer (z. B. axum) erlaubt Registrierung von „ECL-Handler“ pro Route (z. B. POST /realm/:realm_id/action).
  2. Bei Request: caller aus Auth, realm_id aus Pfad/Header; Bytecode für diese Route laden; Runner.run_policy(bytecode, host, context) mit gas/mana-Limits; Ergebnis serialisieren und als Response zurückgeben.
  3. Host = ErynoaHost (Storage); ggf. erweiterter Host mit get_request_body()/set_response() für ECL-seitigen Zugriff auf Request/Response.
- **Abnahme:** Mindestens ein Endpoint wird als ECL-Bytecode betrieben; Metriken (ECLVMState, API-State) steigen.

### 3.3 UI-Engine

- **Ziel:** Trust-Gates, Sichtbarkeitsregeln oder einfache UI-Logik als ECL (z. B. „Zeige Komponente wenn trust.r > 0.5“).
- **Schritte:**
  1. UI-Layer (oder state_integration UIObserver) bei „Soll Komponente X angezeigt werden?“ ECL-Bytecode für Komponente X aufrufen (caller_did, realm_id, component_id).
  2. Runner.run_policy(bytecode, host, context); Rückgabe bool oder Zahl → Sichtbarkeit/Reihenfolge.
  3. Host = ErynoaHost (oder StateView-basiert, wenn nur State-Metriken nötig).
- **Abnahme:** Mindestens eine Sichtbarkeitsentscheidung läuft über ECL; ECLVMState/UI-State werden aktualisiert.

### 3.4 DataLogic-Engine

- **Ziel:** Aggregationen, Filter oder Transformationen als ECL (z. B. „Filter Events wo trust_norm(creator) > 0.6“).
- **Schritte:**
  1. DataLogic-Layer bei Stream-Filter/Aggregation: optional ECL-Bytecode pro Stream oder pro Aggregation; Input = Event-Payload (oder Referenz), Context = caller_did, realm_id.
  2. Runner.run_policy(bytecode, host, context); Rückgabe bool (Filter) oder number (Aggregation).
  3. Host = ErynoaHost (store_get für Zwischenergebnisse möglich).
- **Abnahme:** Mindestens ein Filter oder eine Aggregation nutzt ECL; ECLVMState/DataLogic-State konsistent.

### 3.5 Governance-Engine

- **Ziel:** Abstimmungsregeln, Quorum, Delegation als ECL (z. B. „Proposal annehmen wenn yes_votes / total > 0.5 und trust_norm(voter) > 0.3“).
- **Schritte:**
  1. Governance-Layer bei Vote-Auswertung / Proposal-Transition: ECL-Bytecode für Realm oder Proposal-Type laden; Context = voter_did, realm_id, proposal_id, current_yes/no.
  2. Runner.run_policy(bytecode, host, context); Rückgabe bool (angenommen/abgelehnt) oder nächster Status.
  3. Host = ErynoaHost (Trust, Identity, ggf. store für Proposal-Daten).
- **Abnahme:** Mindestens eine Governance-Entscheidung (Vote-Auswertung oder Status-Übergang) läuft über ECL.

### 3.6 Controller-Engine

- **Ziel:** Permission-Checks oder Delegation-Rules als ECL (z. B. „Erlaube Aktion X wenn has_credential(caller, 'admin') oder trust.p > 0.7“).
- **Schritte:**
  1. Controller bei authz_check: optional ECL-Bytecode pro Permission oder Resource; Context = caller_did, realm_id, permission_id, resource.
  2. Runner.run_policy(bytecode, host, context); Rückgabe bool (erlaubt/verweigert).
  3. Host = ErynoaHost (has_credential, get_trust_vector, resolve_did).
- **Abnahme:** Mindestens eine AuthZ-Entscheidung läuft über ECL.

### 3.7 Abnahmekriterien Phase 3

- [x] Eine gemeinsame Runner-/run_policy-Hülle existiert; ProgrammableGateway und alle neuen Eintrittspunkte nutzen sie.
- [x] Pro Engine (API, UI, DataLogic, Governance, Controller) ist mindestens ein ECL-Eintrittspunkt implementiert und getestet.
- [x] ECLVMState und bestehende State-Engines (API, UI, DataLogic, Governance, Controller) werden bei ECL-Läufen konsistent aktualisiert (Observer-Aufruf in Runner).

### 3.8 Umsetzung (Stand)

- **3.1** `eclvm::runtime::runner`: `PolicyRunContext`, `run_policy(bytecode, host, context) -> Result<ExecutionResult>`. ProgrammableGateway nutzt `run_policy` intern.
- **3.2–3.6** `eclvm::entrypoints::EclEntrypoints`: Registrierung pro Engine (api_handlers, ui_handlers, datalogic_handlers, governance_handlers, controller_handlers); `run_api`, `run_ui`, `run_datalogic`, `run_governance`, `run_controller` mit optionalem Observer; einheitlich `run_policy` + on_policy_executed.

---

## Phase 4: ECLVMStateContext / StateView / StateHandle klären

**Ziel:** Kein Doppelweg „Storage-Host vs. State-Context“; klare Empfehlung, wann welcher Pfad genutzt wird.

### 4.1 Ein-Host- vs. Zwei-Host-Strategie

- **Ein-Host:** ErynoaHost wird so erweitert, dass er sowohl Storage als auch optional UnifiedState/StateView nutzt (z. B. get_trust zuerst aus StateView, fallback Storage; oder nur Storage, Metriken nur über Observer). ECLVMStateContext wird dann nur noch als „Kontext-Container“ (caller_did, realm_id, Budget) genutzt; Host bleibt ErynoaHost.
- **Zwei-Host:** ErynoaHost = Storage-only (wie heute). Zusätzlich `StateHost` implementiert HostInterface und liest/schreibt nur über StateView/StateHandle (UnifiedState). ECLVM kann mit Host = StateHost laufen, wenn keine Persistenz nötig ist (z. B. reine Metrik-Policies). ECLVMStateContext liefert StateView + StateHandle; StateHost wird aus diesem Kontext erzeugt.
- **Empfehlung:** Ein-Host (ErynoaHost erweitern) reduziert Duplikation; StateView aus Snapshot befüllbar machen (Phase 2.3), ErynoaHost optional mit StateView füttern für Lese-Metriken. StateHandle/commit für Schreib-Policies erst in Phase 3+ wenn nötig (dann Host-Methode store_put → StateHandle.store_put + am Ende commit).

### 4.2 ECLVMStateContext Rolle

- **Option A:** ECLVMStateContext bleibt „State-backed Execution Context“ für künftige oder spezielle Use-Cases (z. B. Tests, oder ECL die nur gegen UnifiedState laufen soll). ErynoaHost bleibt Standard-Host für Produktion (Storage).
- **Option B:** ECLVMStateContext wird zur Standard-Erstellung von „Execution Context“: er liefert caller_did, realm_id, Budget, und eine Referenz auf den zu nutzenden Host (ErynoaHost oder StateHost). Runner.run_policy nimmt dann (bytecode, context: ECLVMStateContext, host: Arc<dyn HostInterface>).
- **Dokumentation:** In state.rs und eclvm/ klar dokumentieren: „ErynoaHost = Produktion (Storage); ECLVMStateContext + StateView = optional State-only oder Tests.“

### 4.3 StateHandle und Persistenz

- **Heute:** StateHandle.store_put speichert nur „dirty keys“ und bei commit() wird log_and_apply aufgerufen – also UnifiedState (in-memory) aktualisiert, **nicht** DecentralizedStorage. Für ECL-seitige Realm-Daten ist aber Storage die Quelle (ErynoaHost.store_put → RealmStorage).
- **Konsequenz:** Wenn ECL Realm-Daten schreiben soll, bleibt ErynoaHost.store_put der richtige Weg (persistiert in RealmStorage). StateHandle.store_put kann für „ephemere“ State-Änderungen genutzt werden (nur UnifiedState). In der Doku trennen: „Realm-Daten schreiben → Host (ErynoaHost); reine Laufzeit-State-Mutation → StateHandle + log_and_apply“.

### 4.4 Abnahmekriterien Phase 4

- [x] Dokumentation beschreibt einheitlich: wann ErynoaHost, wann StateView/ECLVMStateContext, wann StateHandle.
- [x] Entweder Ein-Host (erweiterter ErynoaHost) oder Zwei-Host (ErynoaHost + StateHost) umgesetzt; keine unklare Doppelung mehr.
- [x] ECLVMStateContext ist entweder in Runner/ECLVM-Eintrittspunkte integriert oder explizit als „State-only/Test“-Pfad dokumentiert.

### 4.5 Umsetzung (Stand)

- **Dokumentation:** [ECL-HOST-STATE-CONTEXT.md](./ECL-HOST-STATE-CONTEXT.md) – ErynoaHost = Produktion (Storage); ECLVMStateContext + StateView = optional State-only/Tests; StateHandle = nur ephemere Laufzeit-State-Änderungen; Realm-Daten schreiben → ErynoaHost.
- **Ein-Host-Strategie:** ErynoaHost bleibt Standard-Host; StateView aus Snapshot befüllbar (Phase 2.3); künftig optional get_metric aus State ohne zweiten StateHost.

---

## Abhängigkeiten und Reihenfolge

```
Phase 1 (Metriken + Gateway)
    ↓
Phase 2 (Host + State-Integration)
    ↓
Phase 3 (Eintrittspunkte) ← kann pro Engine inkrementell erfolgen
    ↓
Phase 4 (StateContext/StateView/StateHandle klären) ← parallel zu Phase 3 oder danach
```

- **Phase 1** kann sofort umgesetzt werden; keine Abhängigkeit von 2/3/4.
- **Phase 2** baut auf 1 auf (Observer-Aufruf existiert bereits); 2.3 (StateView befüllen) ist Voraussetzung für einen sinnvollen StateHost oder erweiterten ErynoaHost mit StateView.
- **Phase 3** nutzt die Runner-Hülle und Observer aus Phase 1/2; jede Engine (API, UI, DataLogic, Governance, Controller) kann nacheinander ergänzt werden.
- **Phase 4** kann nach 2 und teilweise parallel zu 3 erfolgen (Dokumentation + Entscheidung Ein-Host vs. Zwei-Host).

---

## Kurz-Checkliste (Vollständiger Refactor)

| # | Task | Phase |
|---|------|--------|
| 1 | ProgrammableGateway (oder Aufrufer) ruft nach evaluate() StateIntegrator on_policy_executed / on_crossing_policy_evaluated auf | 1 |
| 2 | Gateway vereinheitlichen: GatewayGuard + ProgrammableGateway in einem Pfad (UnifiedGateway oder Erweiterung GatewayGuard) | 1 |
| 3 | StateView aus UnifiedState/Snapshot befüllbar machen (refresh_from / from_unified_snapshot erweitern) | 2 |
| 4 | Optional: ErynoaHost um StateView oder Snapshot-Metriken erweitern | 2 |
| 5 | Runner / run_policy_and_notify-Hülle einführen; ProgrammableGateway darauf umstellen | 3 |
| 6 | API: ECL-Handler pro Route; Runner + ErynoaHost; Observer in Runner | 3 |
| 7 | UI: ECL für Sichtbarkeit/Trust-Gate; Runner + Host | 3 |
| 8 | DataLogic: ECL für Filter/Aggregation; Runner + Host | 3 |
| 9 | Governance: ECL für Vote/Proposal-Entscheidung; Runner + Host | 3 |
| 10 | Controller: ECL für AuthZ; Runner + Host | 3 |
| 11 | Ein-Host vs. Zwei-Host entscheiden; ECLVMStateContext/StateHandle Rolle dokumentieren und ggf. anbinden | 4 |

---

**Stand:** Basierend auf ECL-ECLVM-USE-CASE-CHECK.md und aktueller Codebasis (eclvm, state.rs, state_integration, peer/gateway).

---

## Gap-Analyse (state.rs vs. Implementierung)

Abgleich der **state.rs-Vorstellungen** (ECLVMStateContext, StateView, StateHandle, ECLVMState) mit der umgesetzten ECL/ECLVM-Integration: [ECL-STATE-RS-GAP-ANALYSIS.md](./ECL-STATE-RS-GAP-ANALYSIS.md). Enthält Lücken (duration_us, ECLVMStateContext nicht an Host angebunden, policies_by_type, ECLVMBudget, StateEvent) und Empfehlungen.
