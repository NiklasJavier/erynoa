# 🧮 State-Kerngedanken: Formale Spezifikation

> **Komprimierung von:** `08-STATE-KERNGEDANKEN.md`
> **Notation:** Pluto-DNA (mathematisch-logisch)

---

## §1 Design-Prinzipien $\mathcal{P}$

Das State-System folgt 9 fundamentalen Prinzipien:

$$
\mathcal{P} = \{ P_1, P_2, \ldots, P_9 \}
$$

| $P_i$ | Name                      | Formale Definition                                                                             |
| ----- | ------------------------- | ---------------------------------------------------------------------------------------------- |
| $P_1$ | Hierarchische Komposition | $\forall L_i, L_j \in \mathcal{L}: i < j \Rightarrow L_i \prec L_j$                            |
| $P_2$ | Thread-Safety             | $\forall s \in \mathcal{S}: \text{atomic}(s) \lor \text{rwlock}(s)$                            |
| $P_3$ | Dependency Injection      | $\forall m \in \mathcal{M}: \text{deps}(m) \subseteq \text{inject}(\text{Hub})$                |
| $P_4$ | Event-Driven              | $\Delta s \Rightarrow \exists e \in \mathcal{E}: \text{emit}(e)$                               |
| $P_5$ | Snapshot-Isolation        | $\text{read}(s) \cap \text{lock}(s) = \emptyset$                                               |
| $P_6$ | Per-Realm Isolation       | $\forall r \in \mathcal{R}: \text{State}(r) \cap \text{State}(r') = \emptyset$ für $r \neq r'$ |
| $P_7$ | Event-Inversion           | $\text{P2P} \xleftrightarrow{\text{Queue}} \text{Core}$                                        |
| $P_8$ | Circuit Breaker           | $\text{anomaly}(t) > \theta \Rightarrow \text{degrade}()$                                      |
| $P_9$ | CQRS Light                | $\Delta s \xrightarrow{\text{broadcast}} \text{Subscribers}$                                   |

---

## §2 Kern-Strukturen $\mathcal{K}$

### 2.1 EventBus $\mathbb{B}$

$$
\mathbb{B} = \langle I, E, P, \mu \rangle
$$

wobei:

- $I = \langle I_{tx}, I_{rx} \rangle$ — Ingress-Kanäle (P2P → Core)
- $E = \langle E_{tx}, E_{rx} \rangle$ — Egress-Kanäle (Core → P2P)
- $P$ — Priority-Queue für Consensus-kritische Events
- $\mu = \langle \mu_I, \mu_E, \mu_D \rangle$ — Metriken (Ingress, Egress, Dropped)

**Axiom (Event-Flow):**

$$
\forall e \in \text{NetworkEvent}: e \in I \oplus e \in E
$$

### 2.2 StateDelta $\Delta$

$$
\Delta = \langle \text{seq}, \kappa, \tau, \text{data}, t, r? \rangle
$$

| Symbol        | Typ                                         | Beschreibung                     |
| ------------- | ------------------------------------------- | -------------------------------- |
| $\text{seq}$  | $\mathbb{N}$                                | Sequenznummer (monoton steigend) |
| $\kappa$      | $\text{StateComponent}$                     | Betroffene Komponente            |
| $\tau$        | $\text{DeltaType}$                          | Art der Änderung                 |
| $\text{data}$ | $\text{Vec}\langle u8 \rangle$              | Serialisierte Daten              |
| $t$           | $\mathbb{N}$                                | Zeitstempel (ms)                 |
| $r?$          | $\text{Option}\langle\text{RealmId}\rangle$ | Optionale Realm-Zuordnung        |

**Broadcaster-Invariante:**

$$
\text{seq}(t+1) = \text{seq}(t) + 1 \quad \land \quad \text{seq}(0) = 0
$$

### 2.3 CircuitBreaker $\mathbb{C}$

$$
\mathbb{C} = \langle \sigma, W, \Theta \rangle
$$

wobei:

- $\sigma \in \{ \text{Normal}, \text{Degraded}, \text{Emergency} \}$ — SystemMode
- $W \subseteq \mathbb{N}^{60}$ — Critical-Window (Anomalien/Minute)
- $\Theta = \langle \theta_D, \theta_E, \theta_G \rangle$ — Schwellwerte

**Transition-Regeln:**

$$
\sigma \xrightarrow{|W| > \theta_D} \text{Degraded} \xrightarrow{|W| > \theta_E} \text{Emergency}
$$

$$
\theta_G = \text{Gini-Threshold} \quad (\text{Anti-Calcification } \kappa_{19})
$$

---

## §3 StateEvent-Taxonomie $\mathcal{E}$

Die 42 Event-Varianten partitionieren sich in 8 Kategorien:

$$
\mathcal{E} = \bigcup_{i=1}^{8} \mathcal{E}_i
$$

| $\mathcal{E}_i$ | Kategorie  | Events                                                                               | Kardinalität |
| --------------- | ---------- | ------------------------------------------------------------------------------------ | ------------ |
| $\mathcal{E}_1$ | Core       | TrustUpdate, EventProcessed, FormulaComputed, ConsensusRound                         | 4            |
| $\mathcal{E}_2$ | Execution  | ExecutionStarted, ExecutionCompleted, PolicyEvaluated, BlueprintAction, SagaProgress | 5            |
| $\mathcal{E}_3$ | Protection | AnomalyDetected, DiversityMetricUpdate, CalibrationApplied, SystemModeChanged        | 4            |
| $\mathcal{E}_4$ | Realm      | RealmLifecycle, MembershipChange, CrossingEvaluated                                  | 3            |
| $\mathcal{E}_5$ | P2P        | NetworkMetricUpdate, PeerConnectionChange, TrustUpdated, PeerBanned                  | 4            |
| $\mathcal{E}_6$ | Privacy    | CircuitCreated, CircuitClosed, MessageSent, CoverTraffic, MixingPool, RelaySelection | 6            |
| $\mathcal{E}_7$ | Recovery   | CheckpointCreated, RecoveryCompleted, ReorgDetected                                  | 3            |
| $\mathcal{E}_8$ | Identity   | IdentityBootstrapped, SubDIDDerived, Delegation, Credential, KeyRotated              | 5+           |

**Gesamt:** $|\mathcal{E}| = 42$

### Emitter-Trait $\phi$

$$
\phi: \mathcal{E} \rightarrow \text{Channel} \cup \text{Callback} \cup \emptyset
$$

| Implementierung  | Semantik              |
| ---------------- | --------------------- |
| $\phi_\emptyset$ | NoOpEmitter (Tests)   |
| $\phi_C$         | ChannelEmitter (mpsc) |
| $\phi_F$         | CallbackEmitter (Fn)  |

---

## §4 Wrapped Event & Log $\mathcal{W}$

### 4.1 WrappedStateEvent

$$
\mathcal{W} = \langle \text{id}, t, \pi, \kappa, \text{seq}, e, \sigma? \rangle
$$

| Symbol       | Typ                                     | Invariante                                            |
| ------------ | --------------------------------------- | ----------------------------------------------------- |
| $\text{id}$  | $\text{Blake3}$                         | $\text{id} = H_{\text{Blake3}}(e \| t \| \text{seq})$ |
| $t$          | $\mathbb{N}_{128}$                      | Timestamp (ms)                                        |
| $\pi$        | $\text{Vec}\langle\text{id}\rangle$     | Parent-IDs (Kausalität)                               |
| $\kappa$     | $\text{StateComponent}$                 | Komponenten-Zuordnung                                 |
| $\text{seq}$ | $\mathbb{N}_{64}$                       | Sequenznummer                                         |
| $e$          | $\text{StateEvent}$                     | Das eigentliche Event                                 |
| $\sigma?$    | $\text{Option}\langle\text{Sig}\rangle$ | Optionale Signatur                                    |

**Kausalitäts-Invariante ($\kappa_9$):**

$$
\forall w \in \mathcal{W}: \forall p \in \pi(w): \text{seq}(p) < \text{seq}(w)
$$

### 4.2 StateEventLog

$$
\mathcal{L} = \langle \text{seq}, B, c, \iota \rangle
$$

- $B$: Ring-Buffer mit $|B| = 10.000$
- $c$: Letzter Checkpoint-ID
- $\iota = 5.000$: Checkpoint-Intervall

**Checkpoint-Regel:**

$$
\text{seq} \mod \iota = 0 \Rightarrow \text{checkpoint}()
$$

---

## §5 Merkle State Tracking $\mathcal{M}$

$$
\mathcal{M} = \langle \rho, H_\kappa, \Delta_H \rangle
$$

| Symbol     | Beschreibung                                          |
| ---------- | ----------------------------------------------------- |
| $\rho$     | Root-Hash (aktueller State)                           |
| $H_\kappa$ | $\text{StateComponent} \rightarrow \text{MerkleHash}$ |
| $\Delta_H$ | History von MerkleDeltas                              |

### MerkleDelta

$$
\delta_M = \langle \rho_{\text{old}}, \rho_{\text{new}}, \kappa, \pi, \text{data} \rangle
$$

**Verifikations-Axiom:**

$$
\text{verify}(\pi, \rho_{\text{old}}, \text{data}) \Rightarrow \rho_{\text{new}} = \text{apply}(\rho_{\text{old}}, \text{data})
$$

---

## §6 StateGraph: Relationsalgebra $\mathcal{G}$

### 6.1 Definition

$$
\mathcal{G} = \langle V, E, \lambda \rangle
$$

wobei:

- $V$: 40 StateComponents
- $E \subseteq V \times V$: 110+ Kanten
- $\lambda: E \rightarrow \mathcal{R}$: Relationstyp-Funktion

### 6.2 Relationstypen $\mathcal{R}$

$$
\mathcal{R} = \{ \rightarrow_D, \rightarrow_T, \rightarrow_A, \rightarrow_V, \leftrightarrow_B \}
$$

| Symbol                  | Name          | Semantik                             |
| ----------------------- | ------------- | ------------------------------------ |
| $A \rightarrow_D B$     | DependsOn     | $B$ muss vor $A$ initialisiert sein  |
| $A \rightarrow_T B$     | Triggers      | Update in $A$ triggert Update in $B$ |
| $A \rightarrow_A B$     | Aggregates    | $A$ enthält/aggregiert $B$           |
| $A \rightarrow_V B$     | Validates     | $A$ validiert $B$                    |
| $A \leftrightarrow_B B$ | Bidirectional | Wechselseitige Abhängigkeit          |

### 6.3 Schlüssel-Relationen (partitioniert nach Layer)

#### Identity-Layer ($\kappa_6$–$\kappa_8$)

$$
\begin{aligned}
&\text{Trust} \rightarrow_D \text{Identity} \\
&\text{Identity} \rightarrow_T \text{Trust} \\
&\text{Event} \rightarrow_D \text{Identity} \\
&\text{Realm} \rightarrow_D \text{Identity} \\
&\text{Gateway} \rightarrow_V \text{Identity} \\
&\text{Swarm} \rightarrow_V \text{Identity}
\end{aligned}
$$

#### Core-Layer ($\kappa_2$–$\kappa_{18}$)

$$
\begin{aligned}
&\text{Trust} \leftrightarrow_B \text{Event} \\
&\text{Trust} \rightarrow_D \text{WorldFormula} \\
&\text{WorldFormula} \rightarrow_T \text{Consensus} \\
&\text{Consensus} \rightarrow_V \text{Event}
\end{aligned}
$$

#### Execution-Layer

$$
\begin{aligned}
&\text{Gas} \rightarrow_D \text{Trust} \\
&\text{Mana} \rightarrow_D \text{Trust} \\
&\text{Execution} \rightarrow_A \{ \text{Gas}, \text{Mana} \} \\
&\text{Execution} \rightarrow_T \text{Event}
\end{aligned}
$$

#### ECLVM-Layer

$$
\begin{aligned}
&\text{ECLVM} \rightarrow_D \{ \text{Gas}, \text{Mana} \} \\
&\text{ECLVM} \rightarrow_T \text{Event} \\
&\text{ECLPolicy} \rightarrow_V \{ \text{Gateway}, \text{Realm} \}
\end{aligned}
$$

#### Realm-Layer ($\kappa_{22}$–$\kappa_{24}$)

$$
\begin{aligned}
&\text{Realm} \rightarrow_A \text{Gateway} \\
&\text{Realm} \rightarrow_T \text{SagaComposer} \\
&\text{Gateway} \rightarrow_D \text{ECLPolicy} \\
&\text{SagaComposer} \rightarrow_D \text{ECLVM}
\end{aligned}
$$

### 6.4 Graph-Operationen

$$
\begin{aligned}
\text{deps}(v) &= \{ u \mid (v, u) \in E \land \lambda(v,u) = \rightarrow_D \} \\
\text{deps}^*(v) &= \text{transitive\_closure}(\text{deps}(v)) \\
\text{triggers}(v) &= \{ u \mid (v, u) \in E \land \lambda(v,u) = \rightarrow_T \} \\
\text{validators}(v) &= \{ u \mid (u, v) \in E \land \lambda(u,v) = \rightarrow_V \} \\
\text{crit}(v) &= |\text{deps}^{-1}(v)| + |\text{triggers}(v)|
\end{aligned}
$$

---

## §7 Sub-States $\mathcal{S}$

### 7.1 TrustState $\mathcal{S}_T$

$$
\mathcal{S}_T = \langle N_e, N_r, \mu, T_\text{avg}, \mathcal{D}, \mathcal{T}_\text{id} \rangle
$$

| Symbol                  | Typ                                                | Beschreibung                           |
| ----------------------- | -------------------------------------------------- | -------------------------------------- |
| $N_e$                   | $\mathbb{N}$                                       | Anzahl Entities                        |
| $N_r$                   | $\mathbb{N}$                                       | Anzahl Relationships                   |
| $\mu$                   | $\langle +, -, v \rangle$                          | Updates (positiv, negativ, Violations) |
| $T_\text{avg}$          | $[0,1]$                                            | Durchschnittliches Trust               |
| $\mathcal{D}$           | TrustDistribution                                  | Verteilung                             |
| $\mathcal{T}_\text{id}$ | $\text{UniversalId} \rightarrow \text{TrustEntry}$ | Trust pro ID                           |

**Asymmetrie-Invariante ($\kappa_4$):**

$$
\frac{\mu_-}{\mu_+} \approx 2:1
$$

**Trust-Wertebereich ($\kappa_2$):**

$$
\forall e \in \mathcal{T}_\text{id}: e.\text{trust} \in [0, 1]
$$

### 7.2 IdentityState $\mathcal{S}_I$

$$
\mathcal{S}_I = \langle \beta, \text{DID}_\text{root}, \mathcal{M}, \mathcal{S}_\text{sub}, \mathcal{A}, \mathcal{D}_\text{del}, \mathcal{C} \rangle
$$

| Symbol                   | Beschreibung            | Invariante                                         |
| ------------------------ | ----------------------- | -------------------------------------------------- |
| $\beta$                  | Bootstrap abgeschlossen | $\beta \in \{\top, \bot\}$                         |
| $\text{DID}_\text{root}$ | Root-DID                | Format: `did:erynoa:*` ($\kappa_6$)                |
| $\mathcal{M}$            | IdentityMode            |                                                    |
| $\mathcal{S}_\text{sub}$ | Sub-DIDs                | $\text{UniversalId} \rightarrow \text{SubDIDInfo}$ |
| $\mathcal{A}$            | Wallet-Adressen         |                                                    |
| $\mathcal{D}_\text{del}$ | Delegations             | Trust-Decay ($\kappa_8$)                           |
| $\mathcal{C}$            | Credentials             | Issued/Verified Counters                           |

---

## §8 Komponenten-Klassifikation $\mathcal{K}_{40}$

Die 40 StateComponents partitionieren sich:

$$
\mathcal{K}_{40} = \bigcup_{i=1}^{9} L_i
$$

| Layer $L_i$       | Komponenten                                                                                                     |
| ----------------- | --------------------------------------------------------------------------------------------------------------- |
| $L_\text{Core}$   | $\{ \text{Trust}, \text{Event}, \text{WorldFormula}, \text{Consensus} \}$                                       |
| $L_\text{Exec}$   | $\{ \text{Gas}, \text{Mana}, \text{Execution} \}$                                                               |
| $L_\text{Engine}$ | $\{ \text{ECLVM}, \text{ECLPolicy}, \text{ECLBlueprint} \}$                                                     |
| $L_\text{Prot}$   | $\{ \text{Anomaly}, \text{Diversity}, \text{Quadratic}, \text{AntiCalcification}, \text{Calibration} \}$        |
| $L_\text{Peer}$   | $\{ \text{Realm}, \text{Gateway}, \text{SagaComposer}, \text{IntentParser}, \text{Room}, \text{Partition} \}$   |
| $L_\text{P2P}$    | $\{ \text{Swarm}, \text{Gossip}, \text{Kademlia}, \text{Relay}, \text{NatTraversal}, \text{Privacy} \}$         |
| $L_\text{Store}$  | $\{ \text{EventStore}, \text{Archive}, \text{KvStore}, \text{Blueprint} \}$                                     |
| $L_\text{ID}$     | $\{ \text{Identity}, \text{Credential}, \text{KeyManagement} \}$                                                |
| $L_\text{UI}$     | $\{ \text{Controller}, \text{UI}, \text{DataLogic}, \text{API}, \text{Governance}, \text{BlueprintComposer} \}$ |

---

## §9 Kritikalitäts-Funktion $\text{crit}: V \rightarrow \mathbb{N}$

$$
\text{crit}(v) = |\{ u \mid u \rightarrow_D v \}| + |\text{triggers}(v)|
$$

| $v$      | $   | \text{deps}^{-1} | $   | $     | \text{trig} | $   | $\text{crit}$ | Priorität |
| -------- | --- | ---------------- | --- | ----- | ----------- | --- | ------------- | --------- |
| Identity | 18  | 6                | 24  | $P_0$ |
| Trust    | 15  | 5                | 20  | $P_0$ |
| Event    | 10  | 6                | 16  | $P_0$ |
| ECLVM    | 8   | 4                | 12  | $P_1$ |
| Gateway  | 6   | 3                | 9   | $P_1$ |
| Realm    | 5   | 4                | 9   | $P_1$ |
| Gas      | 8   | 0                | 8   | $P_2$ |
| Swarm    | 4   | 3                | 7   | $P_2$ |
| Privacy  | 2   | 1                | 3   | $P_2$ |

---

## §10 Invarianten-Mapping $\kappa_i \mapsto \text{Impl}$

| $\kappa_i$    | Beschreibung                              | State-Implementierung                     |
| ------------- | ----------------------------------------- | ----------------------------------------- |
| $\kappa_2$    | $T \in [0,1]$                             | `TrustEntry.global_trust.clamp(0.0, 1.0)` |
| $\kappa_4$    | $\frac{\Delta T^-}{\Delta T^+} \approx 2$ | `TrustState.asymmetry_ratio()`            |
| $\kappa_6$    | DID-Format                                | `IdentityState.root_did`                  |
| $\kappa_8$    | Delegation Decay                          | `TrustEntry.decay_factor`                 |
| $\kappa_9$    | Event-Kausalität                          | `WrappedStateEvent.parent_ids`            |
| $\kappa_{11}$ | Gas-Monotonie                             | `ExecutionState.gas_consumed`             |
| $\kappa_{13}$ | Mana $\geq 0$                             | `ManaState`                               |
| $\kappa_{19}$ | Gini $< \theta_G$                         | `CircuitBreaker.gini_threshold`           |
| $\kappa_{22}$ | Realm-Inheritance                         | `RealmState.parent_realm`                 |
| $\kappa_{23}$ | Crossing-Policy                           | `GatewayState` + `ECLPolicy`              |
| $\kappa_{24}$ | Saga-Atomicity                            | `SagaState.compensation_triggered`        |

---

## §11 Nervensystem-Mapping $\mathcal{N}$

Finale Struktur für Pluto:

$$
\mathcal{N} = \langle \mathcal{S}, \mathcal{E}, \mathcal{G}, \text{Hub}, \phi, \mathcal{L}, \mathcal{M}, \mathbb{B}, \mathbb{C} \rangle
$$

| Symbol        | Modul         | Beschreibung                 |
| ------------- | ------------- | ---------------------------- |
| $\mathcal{S}$ | `state/`      | UnifiedState (partitioniert) |
| $\mathcal{E}$ | `events/`     | StateEvent-Taxonomie         |
| $\mathcal{G}$ | `graph/`      | StateGraph + Traversal       |
| Hub           | `synapse/`    | SynapseHub (Dispatch)        |
| $\phi$        | `emitters/`   | StateEventEmitter            |
| $\mathcal{L}$ | `log/`        | StateEventLog                |
| $\mathcal{M}$ | `merkle/`     | MerkleStateTracker           |
| $\mathbb{B}$  | `bus/`        | EventBus                     |
| $\mathbb{C}$  | `protection/` | CircuitBreaker               |

---

## 📐 Zusammenfassung

Das State-System ist formal definiert als:

$$
\boxed{
\text{State}_\text{Pluto} = \langle \mathcal{P}_9, \mathcal{K}_{40}, \mathcal{E}_{42}, \mathcal{G}_{110+}, \mathcal{S}_T, \mathcal{S}_I, \kappa_{1..24} \rangle
}
$$

Mit den Haupt-Invarianten:

1. **Hierarchie:** $L_i \prec L_j \Leftrightarrow i < j$
2. **Kausalität:** $\text{seq}(p) < \text{seq}(e) \; \forall p \in \text{parents}(e)$
3. **Trust-Bound:** $T \in [0,1]$
4. **Asymmetrie:** $\Delta T^- / \Delta T^+ \approx 2$
5. **Snapshot-Isolation:** Lesezugriffe blockieren nie
