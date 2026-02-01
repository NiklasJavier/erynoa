# Migration Status: Unified Domain Model

## Übersicht

Die Migration von den alten `domain/*` Modulen zu `domain/unified/*` ist im Gange.

**Status:** 🟡 In Progress
**Letzte Aktualisierung:** Session vom Migrations-Tag
**Kompilierungsfehler:** ~170 (von ursprünglich ~180)

## Abgeschlossene Arbeiten

### Phase 1: Unified Module erweitert ✅

1. **unified/saga.rs**
   - `Intent::source_did()` - Alias für `source` Feld
   - `Goal::Complex.sub_goals` mit `#[serde(alias = "parsed_goals")]`
   - `Constraint::MaxCost` mit optionalen Feldern `amount` und `asset_type`
   - `SagaAction::WaitFor` mit `timeout_seconds` (u64) für Kompatibilität
   - `SagaAction::Lock/Unlock/Mint/Burn/GatewayCheck/ExternalChain` hinzugefügt

2. **unified/formula.rs**
   - `WorldFormulaStatus` - neuer Typ für globalen Status
   - `WorldFormulaContribution::from_subject(subject)` - 1-Arg Factory
   - `WorldFormulaContribution::compute(&self)` - Instance-Methode
   - `WorldFormulaContribution::with_context()` - Builder-Methode

3. **unified/event.rs**
   - `Event::timestamp()` - Alias für `self.coord.wall_time()`
   - `Event::primary_trust_dimension()` - Mapping von Payload zu TrustDimension
   - `Event::is_negative_trust()` - Prüft auf Revoke-Payloads
   - `Event::genesis()` - Factory für Genesis-Events
   - `EventPayload::CredentialIssue/CredentialRevoke/TrustUpdate` - neue Varianten

4. **unified/identity.rs**
   - `DID::parse(s)` und `FromStr` Implementation
   - `DID::generate()` - für Tests
   - `IdentityError::InvalidDIDFormat` - neuer Error-Variant

5. **unified/realm.rs**
   - `VirtualRealm.initial_setup_policy: Option<String>`
   - `VirtualRealm.default_shared_stores/default_personal_stores: Vec<StoreTemplate>`
   - `StoreTemplate` und `StoreType` - neue Typen

6. **unified/trust.rs**
   - `TrustVector6D::min_component()` - Alias für `min()`
   - `ContextType::weights()` - Alias für `default_weights()`

### Teilweise abgeschlossen

1. **peer/saga_composer.rs**
   - `intent.source_did` → `intent.source_did()` ✅
   - `Goal::Complex { parsed_goals }` → `{ sub_goals }` ✅
   - Verbleibend: DID vs UniversalId Parametertypen

## Verbleibende Fehler (Hauptkategorien)

| Fehlertyp                    | Anzahl | Beschreibung                   |
| ---------------------------- | ------ | ------------------------------ |
| `mismatched types`           | ~81    | f32 vs f64, UniversalId vs DID |
| `function takes N arguments` | ~20    | API-Signatur-Änderungen        |
| `field is private`           | ~16    | UniversalId.0 Zugriff          |
| `missing field`              | ~10    | Struct-Initialisierungen       |
| `no method named`            | ~5     | Fehlende Methoden              |

## Nächste Schritte

### Priorität 1: Typ-Inkompatibilitäten

1. **f32 vs f64 in Trust-Berechnungen**
   - TrustVector6D verwendet f32
   - WorldFormula verwendet f64
   - Lösung: Konsistente Typen wählen oder explizite Konvertierungen

2. **UniversalId vs DID**
   - `unified::DID` basiert auf UniversalId
   - Alte Consumer erwarten String-basierte DID
   - Lösung: Consumer schrittweise migrieren

### Priorität 2: API-Anpassungen

1. **WorldFormulaContribution::new()**
   - Alt: `new(DID)` mit Builder-Pattern
   - Neu: `new(UniversalId, lamport)` oder `from_subject(UniversalId)`
   - Lösung: Consumer auf `from_subject()` umstellen

2. **SagaAction-Felder**
   - `Lock.did` → `Lock.owner`
   - `GatewayCheck.did` → `GatewayCheck.subject`

### Priorität 3: Fehlende Felder

1. **WitnessAttestation**
   - Fehlt: `trust_weight`, `timestamp`
   - Hat: `witness`, `coord`, `signature`

2. **Surprisal**
   - Fehlt: `raw_surprisal`
   - Hat: `raw_bits`

3. **Activity**
   - Fehlt: `tau_days`
   - Hat: `tau_seconds`

## Empfohlenes Vorgehen

1. **Kurzfristig:** Alte Module behalten, Consumer schrittweise migrieren
2. **Mittelfristig:** Alle Consumer auf unified umstellen
3. **Langfristig:** Deprecated-Module in v0.3.0 entfernen

## Dateien mit meisten Änderungen erforderlich

1. `src/core/world_formula.rs` - WorldFormula-Engine
2. `src/peer/saga_composer.rs` - Saga-Komposition
3. `src/peer/intent_parser.rs` - Intent-Parsing
4. `src/peer/gateway.rs` - Gateway-Logik
5. `src/local/identity_store.rs` - Identity-Speicher
6. `src/api/v1/*/handlers.rs` - API-Handler

## Testabdeckung

- **Vor Migration:** 384 Tests passing
- **Aktuell:** Kompilierungsfehler, Tests nicht ausführbar
- **Nach Migration:** Alle Tests müssen wieder passen

---

_Dieser Status wird während der Migration aktualisiert._
