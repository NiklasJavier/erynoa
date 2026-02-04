# 🔗 Beziehungsmatrix: Logische Modul-Verbindungen

> **Teil von:** Projekt Pluto
> **Basiert auf:** StateGraph aus `state.rs`

---

## 1. Primäre Beziehungen

### 1.1 Nervensystem → Module

```
                    ┌─────────────────────────────────────────────┐
                    │              UNIFIED STATE                   │
                    │         (nervous_system/)                    │
                    └─────────────────────┬───────────────────────┘
                                          │
                                          │ StateEvent
                                          │ dispatch
                                          ▼
                    ┌─────────────────────────────────────────────┐
                    │              SYNAPSE HUB                     │
                    │            (synapses/)                       │
                    └─────────────────────┬───────────────────────┘
                                          │
           ┌──────────────────────────────┼──────────────────────────────┐
           │                              │                              │
           ▼                              ▼                              ▼
    ┌─────────────┐               ┌─────────────┐               ┌─────────────┐
    │   engines/  │               │   realm/    │               │ protection/ │
    │             │               │             │               │             │
    │ Trust       │◀─────────────▶│ Gateway     │◀─────────────▶│ Anomaly     │
    │ Event       │       ↑       │ Saga        │       ↑       │ Diversity   │
    │ Formula     │       │       │ Sharding    │       │       │ Quadratic   │
    │ Consensus   │       │       │ Quota       │       │       │ Calibration │
    └─────────────┘       │       └─────────────┘       │       └─────────────┘
           │              │              │              │              │
           └──────────────┴──────────────┴──────────────┴──────────────┘
                                          │
                                          ▼
                    ┌─────────────────────────────────────────────┐
                    │               STORAGE                        │
                    │             (storage/)                       │
                    └─────────────────────────────────────────────┘
```

---

## 2. Detaillierte Abhängigkeitsmatrix

### 2.1 Identity-Abhängigkeiten

| Von | Zu | Relation | Axiom | Beschreibung |
|-----|-----|----------|-------|--------------|
| `engines/trust` | `identity/` | DependsOn | Κ6 | Trust basiert auf DID |
| `identity/` | `engines/trust` | Triggers | Κ2 | Neue Identity → Initial Trust |
| `engines/event` | `identity/` | DependsOn | Κ9 | Events haben Autor-DID |
| `realm/gateway` | `identity/` | DependsOn | Κ23 | Crossing prüft Identity |
| `realm/gateway` | `identity/` | Validates | Κ23 | Identity-Verifikation |
| `p2p/swarm` | `identity/` | DependsOn | - | PeerId ist Device-DID |
| `protection/anomaly` | `identity/` | Validates | Κ26 | Identity-Anomalien |
| `synapses/` | `identity/` | Aggregates | - | ControllerObserver |
| `eclvm/` | `identity/` | DependsOn | - | Caller-Identity prüfen |
| `storage/identity_store` | `identity/` | Aggregates | - | DID-Persistenz |

### 2.2 Trust-Abhängigkeiten

| Von | Zu | Relation | Axiom |
|-----|-----|----------|-------|
| `realm/gateway` | `engines/trust` | DependsOn | Κ23 |
| `realm/saga` | `engines/trust` | DependsOn | Κ24 |
| `execution/gas` | `engines/trust` | DependsOn | - |
| `execution/mana` | `engines/trust` | DependsOn | - |
| `engines/formula` | `engines/trust` | DependsOn | Κ15 |
| `engines/consensus` | `engines/trust` | DependsOn | Κ18 |
| `p2p/gossip` | `engines/trust` | DependsOn | - |
| `protection/diversity` | `engines/trust` | Validates | Κ19 |

### 2.3 Realm-Abhängigkeiten

| Von | Zu | Relation | Axiom |
|-----|-----|----------|-------|
| `realm/` | `identity/` | DependsOn | Κ22 |
| `realm/` | `engines/trust` | Bidirectional | Κ22 |
| `realm/gateway` | `eclvm/` | DependsOn | Κ23 |
| `realm/saga` | `eclvm/` | DependsOn | Κ24 |
| `realm/quota` | `protection/` | Triggers | Κ22 |
| `realm/` | `storage/realm` | Aggregates | - |

### 2.4 P2P-Abhängigkeiten

| Von | Zu | Relation |
|-----|-----|----------|
| `p2p/swarm` | `identity/` | DependsOn |
| `p2p/gossip` | `engines/trust` | DependsOn |
| `p2p/gossip` | `engines/event` | Triggers |
| `p2p/dht` | `p2p/swarm` | Aggregates |
| `p2p/relay` | `engines/trust` | DependsOn |
| `p2p/privacy` | `identity/` | DependsOn |

---

## 3. Event-Flow-Diagramm

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              EVENT FLOW                                      │
└─────────────────────────────────────────────────────────────────────────────┘

1. EXTERNAL EVENT
   │
   ▼
┌──────────────┐
│   P2P Swarm  │ ──────▶ NetworkEvent erstellen
└──────────────┘
   │
   ▼
┌──────────────┐
│  EventBus    │ ──────▶ try_send_ingress()
│  (Ingress)   │
└──────────────┘
   │
   ▼
┌──────────────┐
│ UnifiedState │ ──────▶ log_and_apply(StateEvent)
│              │            │
│              │            ├── StateEventLog.log()
│              │            ├── apply_state_event()
│              │            ├── MerkleTracker.update()
│              │            └── StateBroadcaster.broadcast()
└──────────────┘
   │
   ▼
┌──────────────┐
│ SynapseHub   │ ──────▶ dispatch(WrappedStateEvent)
│              │            │
│              │            ├── Direct Observers
│              │            └── Transitive Observers (via StateGraph)
└──────────────┘
   │
   ├──────────────────────────────────────────────────┐
   │                                                  │
   ▼                                                  ▼
┌──────────────┐                              ┌──────────────┐
│TrustObserver │                              │RealmObserver │
│              │                              │              │
│ on_event()   │                              │ on_event()   │
└──────────────┘                              └──────────────┘
```

---

## 4. Cross-Cutting Concerns

### 4.1 Logging & Telemetry

```
Alle Module
    │
    ▼
┌──────────────┐
│  telemetry/  │ ◀──── OpenTelemetry Integration
└──────────────┘
    │
    ├── Traces (Jaeger)
    ├── Metrics (Prometheus)
    └── Logs (structured)
```

### 4.2 Error Propagation

```
domain/unified/error.rs
    │
    └── ErynoaError
            │
            ├── IdentityError   (identity/)
            ├── ExecutionError  (execution/)
            ├── RealmError      (realm/)
            ├── StorageError    (storage/)
            ├── P2PError        (p2p/)
            ├── ECLVMError      (eclvm/)
            └── StateError      (nervous_system/)
```

### 4.3 Config Loading

```
config/
    │
    ├── settings.rs     ◀── Environment Variables
    ├── version.rs      ◀── Build-Time Constants
    │
    └── Module Configs:
        ├── TrustEngineConfig
        ├── EventEngineConfig
        ├── RealmStorageConfig
        ├── P2PConfig
        └── ECLVMConfig
```

---

## 5. Kritische Pfade

### 5.1 Trust-Update (Hot Path)

```
1. TrustEngine.update() ─────────────────────▶ ~50 µs
2. TrustState.update() [Atomic] ─────────────▶ ~1 µs
3. StateEvent::TrustUpdate erstellen ────────▶ ~5 µs
4. log_and_apply() ──────────────────────────▶ ~30 µs
5. SynapseHub.dispatch() ────────────────────▶ ~20 µs
                                    TOTAL: ~106 µs
                                    ZIEL:  < 50 µs
```

### 5.2 Realm-Crossing (Complex Path)

```
1. Intent empfangen ─────────────────────────▶ ~10 µs
2. IntentParser.parse() ─────────────────────▶ ~100 µs
3. SagaComposer.compose() ───────────────────▶ ~200 µs
4. GatewayGuard.evaluate() ──────────────────▶ ~500 µs
   ├── ECLPolicy.validate() [ECLVM] ─────────▶ ~300 µs
   ├── TrustState.get_trust() ───────────────▶ ~5 µs
   └── RealmQuota.check() ───────────────────▶ ~10 µs
5. StateEvent::CrossingEvaluated ────────────▶ ~30 µs
                                    TOTAL: ~1.2 ms
```

---

## 6. Synergy-Matrix

| Modul A | Modul B | Synergy-Score | Grund |
|---------|---------|---------------|-------|
| identity | trust | 10/10 | Fundamental |
| trust | consensus | 9/10 | Κ18 Voting |
| realm | gateway | 9/10 | Κ23 Crossing |
| event | storage | 8/10 | Persistenz |
| eclvm | realm | 8/10 | Policies |
| p2p | identity | 7/10 | PeerId |
| protection | trust | 7/10 | Monitoring |
| formula | trust | 6/10 | Κ15 Input |

---

## 7. Integration-Points

```rust
// Beispiel: Wie realm/gateway.rs mit anderen Modulen interagiert

impl GatewayGuard {
    pub async fn evaluate_crossing(
        &self,
        ctx: &mut ExecutionContext,  // ← execution/
        identity: &UniversalId,       // ← identity/
        from_realm: &RealmId,         // ← realm/
        to_realm: &RealmId,
    ) -> ErynoaResult<CrossingDecision> {
        // 1. Identity validieren
        let did = self.identity_resolver.resolve(identity)?;  // ← identity/

        // 2. Trust prüfen
        let trust = self.trust_state.get_trust(identity)?;    // ← engines/trust

        // 3. Quota prüfen
        self.quota_enforcer.check(to_realm, ResourceType::Crossing)?;  // ← realm/quota

        // 4. Policy evaluieren
        let policy_result = self.eclvm.evaluate_policy(
            "crossing_policy",
            PolicyContext { identity, from: from_realm, to: to_realm, trust }
        ).await?;  // ← eclvm/

        // 5. Event emittieren
        ctx.emit_event(StateEvent::CrossingEvaluated { ... });  // ← nervous_system/

        Ok(policy_result.into())
    }
}
```
