# Pluto::RealmArchitecture ≡ Isolierte Welten

> **Notation:** Pluto (komprimiert-formal)
> **Version:** 6.0 | **Datum:** 2026-02

---

## §1 Realm – Formaldefinition

### Δ1.1 Hauptdefinition

$$\boxed{\mathfrak{R} = \langle \mathcal{I}_R, \mathcal{M}, \mathcal{R}, \mathcal{T}_{\text{local}}, \mathcal{S}, \mathcal{P}, \mathcal{Q} \rangle}$$

**Bedeutung:** Ein Realm $\mathfrak{R}$ ist ein 7-Tupel souveräner Komponenten.

### Δ1.2 Symboltafel

| Symbol                       | Definition                | Domäne                                 |
| ---------------------------- | ------------------------- | -------------------------------------- |
| $\mathfrak{R}$               | Realm (souveräne Einheit) | $\mathfrak{R} \subseteq \mathcal{U}$   |
| $\mathcal{I}_R$              | Realm-Identifikator       | $\text{DID}$                           |
| $\mathcal{M}$                | Mitgliedermenge           | $\mathcal{P}(\text{UniversalId})$      |
| $\mathcal{R}$                | Regelmenge                | $\mathcal{P}(\text{Rule})$             |
| $\mathcal{T}_{\text{local}}$ | Lokaler Trust-Raum        | $\mathbb{R}^6$                         |
| $\mathcal{S}$                | Speicherpartitionen       | $\text{Map}(\text{Key}, \text{Value})$ |
| $\mathcal{P}$                | Policies (ECL)            | $\text{Set}(\text{Policy})$            |
| $\mathcal{Q}$                | Quota (Mana/Storage)      | $\mathbb{N}^2$                         |

---

## §2 Hierarchie-Struktur

### Η2.1 Realm-Baum

$$\mathcal{H}_{\mathfrak{R}} : \mathfrak{R}_{\text{root}} \to \mathfrak{R}_{\text{virtual}} \to \mathfrak{R}_{\text{partition}}$$

```text
                    ┌───────────────────┐
                    │   RootRealm       │
                    │   Κ1 ... Κ28      │
                    └─────────┬─────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
    ┌──────────┐        ┌──────────┐        ┌──────────┐
    │ EU-Realm │        │ Gaming   │        │ DAO      │
    │ +GDPR    │        │ +FairPlay│        │ +Token   │
    └────┬─────┘        └────┬─────┘        └──────────┘
         │                   │
    ┌────┴────┐         ┌────┴────┐
    │ DE-Part │         │ Shard-0 │
    └─────────┘         └─────────┘
```

### Η2.2 Typ-Definitionen

$$\text{RealmType} \in \{\text{Root}, \text{Virtual}, \text{Partition}\}$$

| Typ                               | Eigenschaften                                       |
| --------------------------------- | --------------------------------------------------- |
| $\mathfrak{R}_{\text{root}}$      | 28 Kern-Axiome, $\min_{\tau} = 0$                   |
| $\mathfrak{R}_{\text{virtual}}$   | $\mathcal{R} \supseteq \mathcal{R}_{\text{parent}}$ |
| $\mathfrak{R}_{\text{partition}}$ | Sharding, Read-Replicas                             |

---

## §3 Axiome

### Κ1: Monotone Regelvererbung

$$\boxed{\forall \, \mathfrak{R}_c \subset \mathfrak{R}_p : \mathcal{R}(\mathfrak{R}_c) \supseteq \mathcal{R}(\mathfrak{R}_p)}$$

**Invariante:** Regeln können nur hinzugefügt, nie entfernt werden.

**Beispiel:**
$$\mathcal{R}_{\text{Root}} = \{K_1, ..., K_{28}\}$$
$$\mathcal{R}_{\text{EU}} = \{K_1, ..., K_{28}, \text{GDPR}, \text{MiCA}\}$$
$$\mathcal{R}_{\text{DE}} = \{K_1, ..., K_{28}, \text{GDPR}, \text{MiCA}, \text{BaFin}\}$$

**Validierung:**
$$\text{validate}_{\text{K1}} : \mathcal{R}_c \to \mathcal{R}_p \to \text{Result}\langle(), K1\text{Violation}\rangle$$

---

### Κ21: Quadratisches Voting

$$\boxed{v(s) = \sqrt{\text{tokens}(s)}}$$

**Eigenschaften:**

$$
\begin{aligned}
\text{(i)}\quad   & v : \mathbb{N} \to \mathbb{R}^+ \\
\text{(ii)}\quad  & \frac{dv}{d\text{tokens}} = \frac{1}{2\sqrt{\text{tokens}}} > 0 \\
\text{(iii)}\quad & \frac{d^2v}{d\text{tokens}^2} < 0 \quad\text{(konkav → Plutokratie-Dämpfung)}
\end{aligned}
$$

**Governance-Typen:**
$$\mathcal{G} \in \{\text{Quadratic}, \text{Token}, \text{Reputation}, \text{Delegated}\}$$

---

### Κ22: Saga-Pattern

$$\boxed{\text{Saga} = \langle S_1, ..., S_n, C_1, ..., C_n \rangle}$$

**wobei:**

- $S_i$ = Schritt $i$ (forward action)
- $C_i$ = Compensation für $S_i$ (rollback action)

**Atomarität:**

$$
\text{execute}(\text{Saga}) \Rightarrow \begin{cases}
\text{commit all } S_i & \text{bei Erfolg} \\
\text{apply } C_k, ..., C_1 & \text{bei Fehler in } S_k
\end{cases}
$$

**Cross-Realm-Saga:**
$$\text{Saga}_{\text{cross}} : \mathfrak{R}_A \times \mathfrak{R}_B \to \text{Result}$$

---

### Κ23: Realm-Crossing Trust-Dämpfung

$$\boxed{\tau_{\text{eff}}(\mathfrak{R}_B) = \tau(\mathfrak{R}_A) \cdot \gamma_{A \to B}}$$

**wobei Crossing-Faktor:**
$$\gamma_{A \to B} \in (0, 1] \quad\text{abhängig von:}$$

| Bedingung      | $\gamma$        |
| -------------- | --------------- |
| Sibling-Realms | $0.8$           |
| Parent-Child   | $0.9$           |
| Allowlist      | $1.0$           |
| Blocklist      | $0.0$           |
| Neutral        | Policy-abhängig |

**Beispiel:**
$$\tau_{\text{Alice}}(\text{EU}) = 0.9 \implies \tau_{\text{Alice}}(\text{Gaming}) = 0.9 \times 0.8 = 0.72$$

---

### Κ24: Realm-lokaler Trust

$$\boxed{\mathcal{T}_{\mathfrak{R}} : \text{DID} \to \mathbb{W}^6}$$

**Trust-Dimensionen pro Realm:**
$$\mathbb{W}_{\mathfrak{R}} = (R, I, C, P, V, \Omega)_{\mathfrak{R}}$$

**Isolation:**
$$\mathcal{T}_{\mathfrak{R}_A}(s) \perp\!\!\!\perp \mathcal{T}_{\mathfrak{R}_B}(s)$$

**Bedeutung:** Trust-Aktionen in Realm A beeinflussen nicht Trust in Realm B.

**Ausnahme (Leak bei schweren Verstößen):**
$$\tau(s) < 0.1 \Rightarrow \text{Warning}(\forall \mathfrak{R})$$
$$\tau(s) = 0 \Rightarrow \text{CrossRealm-Mark}(s)$$

---

## §4 Membership-Modell

### Μ4.1 Realm-Sub-DIDs

$$\text{DID}_{\text{root}} \xrightarrow{\text{derive}} \text{DID}_{\mathfrak{R}}$$

**HD-Wallet-Derivation:**
$$\text{path} = m/44'/\text{erynoa}'/0'/\text{realm}/\langle\text{realm-id}\rangle/0$$

**Privacy-Eigenschaft:**
$$\text{Aktivitäten}(\mathfrak{R}_A) \perp\!\!\!\perp \text{Aktivitäten}(\mathfrak{R}_B)$$

### Μ4.2 Rollen

$$\text{Role} \in \{\text{Member}, \text{Mod}, \text{Admin}, \text{Owner}\}$$

**Ordnung:**
$$\text{Member} < \text{Mod} < \text{Admin} < \text{Owner}$$

### Μ4.3 Membership-Zustände

$$\mathcal{M}_{\mathfrak{R}} = \mathcal{M}_{\text{active}} \uplus \mathcal{M}_{\text{pending}} \uplus \mathcal{M}_{\text{banned}}$$

---

## §5 Isolation-Levels

$$\text{IsoLevel} \in \{0, 1, 2\}$$

| Level | Name    | Eigenschaften                                   |
| ----- | ------- | ----------------------------------------------- |
| $0$   | Public  | $\forall$ read, open join, crossing erlaubt     |
| $1$   | Members | Members read, invite-only, crossing mit Status  |
| $2$   | Strict  | E2E-verschlüsselt, Multi-Vouch, keine Crossings |

**Formal:**

$$
\text{canRead}(s, \mathfrak{R}) = \begin{cases}
\top & \text{if } \text{level} = 0 \\
s \in \mathcal{M}_{\mathfrak{R}} & \text{if } \text{level} \geq 1
\end{cases}
$$

$$
\text{canCross}(s, \mathfrak{R}_A, \mathfrak{R}_B) = \begin{cases}
\top & \text{if } \text{level}(\mathfrak{R}_B) = 0 \\
s \in \mathcal{M}_{\mathfrak{R}_B} & \text{if } \text{level}(\mathfrak{R}_B) = 1 \\
\bot & \text{if } \text{level}(\mathfrak{R}_B) = 2
\end{cases}
$$

---

## §6 Quota & Self-Healing

### Θ6.1 Quota-Definition

$$\mathcal{Q}_{\mathfrak{R}} = \langle q_{\text{mana}}, q_{\text{storage}}, \dot{q}_{\text{regen}} \rangle$$

**Quota-Health:**
$$h_{\mathcal{Q}} = 1 - \frac{q_{\text{used}}}{q_{\text{limit}}} \in [0, 1]$$

### Θ6.2 Self-Healing-Mechanismus

$$
\begin{aligned}
h_{\mathcal{Q}} < 0.2 &\Rightarrow \text{Throttling} + \text{Alert} \\
h_{\mathcal{Q}} = 0 &\Rightarrow \text{ReadOnly-Mode}
\end{aligned}
$$

**Regeneration:**
$$q_{\text{mana}}(t+\Delta t) = \min(q_{\text{mana}}(t) + \dot{q}_{\text{regen}} \cdot \Delta t, q_{\text{limit}})$$

---

## §7 Gateway-Policies

### Γ7.1 ECL-Policy-Struktur

$$\text{GatewayPolicy} = \langle \mathcal{R}_{\text{eq}}, \mathcal{V}, \mathcal{A}_{\text{join}}, c_{\text{join}} \rangle$$

**wobei:**

- $\mathcal{R}_{\text{eq}}$ = Trust-Requirements: $\min(\tau_R), \min(\tau_\Omega)$
- $\mathcal{V}$ = Verification: KYC oder Vouching
- $\mathcal{A}_{\text{join}}$ = Actions bei Join
- $c_{\text{join}}$ = Mana-Cost

### Γ7.2 Vouching-Requirement

$$\text{vouching}_{\text{valid}} \iff |\{v \in \mathcal{V} : \tau(v) \geq \tau_{\min}\}| \geq n_{\min}$$

---

## §8 Governance-Modell

### Π8.1 Proposal-Lifecycle

$$\text{Proposal} : \text{Draft} \to \text{Discussion} \to \text{Voting} \to \text{Execution}$$

**Timing-Constraints:**
$$t_{\text{discussion}} \geq 48h, \quad t_{\text{voting}} \geq 72h, \quad t_{\text{timelock}} \geq 24h$$

### Π8.2 Quorum

$$\text{quorum}_{\text{valid}} \iff \frac{\sum v_i}{\sum \text{tokens}} \geq q_{\min} \land \frac{v_{\text{yes}}}{v_{\text{total}}} \geq a_{\min}$$

**Default:** $q_{\min} = 0.1$ (10%), $a_{\min} = 0.5$ (50%)

### Π8.3 Veto

$$\text{veto}_{\text{triggered}} \iff \frac{v_{\text{no}}}{v_{\text{total}}} \geq v_{\text{threshold}}$$

**Default:** $v_{\text{threshold}} = 0.33$

---

## §9 P2P-Integration

### Ρ9.1 Realm-Scoped Gossip

$$\text{Topic}_{\mathfrak{R}} = \texttt{/erynoa/realm/}\langle\mathfrak{R}_{\text{id}}\rangle\texttt{/events}$$

**Subscriber-Set:**
$$\text{Subscribers}(\text{Topic}_{\mathfrak{R}}) \subseteq \mathcal{M}_{\mathfrak{R}}$$

### Ρ9.2 Cross-Realm-Topics

$$
\begin{aligned}
\text{Topic}_{\text{saga}} &= \texttt{/erynoa/cross-realm/sagas} \\
\text{Topic}_{\text{announce}} &= \texttt{/erynoa/cross-realm/announcements}
\end{aligned}
$$

### Ρ9.3 Peer-Discovery

$$\text{DHT-Key}_{\mathfrak{R}} = \texttt{/realm/}\langle\mathfrak{R}_{\text{id}}\rangle\texttt{/peers}$$

---

## §10 StateEvent-Algebra

### Ε10.1 Lifecycle-Events

$$
\begin{aligned}
\text{RealmCreated} &: \mathfrak{R}_{\text{id}} \times \text{Name} \times \mathfrak{R}_{\text{parent}}? \times \text{DID} \times \mathcal{G} \\
\text{MemberJoined} &: \mathfrak{R}_{\text{id}} \times \text{DID} \times \text{DID}_{\mathfrak{R}}? \times \text{Role} \times \mathbb{N} \\
\text{MemberBanned} &: \mathfrak{R}_{\text{id}} \times \text{DID} \times \text{DID} \times \text{String}
\end{aligned}
$$

### Ε10.2 Crossing-Events

$$
\begin{aligned}
\text{CrossingAttempted} &: \mathfrak{R}_{\text{from}} \times \mathfrak{R}_{\text{to}} \times \text{DID} \times \gamma \\
\text{CrossingSucceeded} &: \mathfrak{R}_{\text{from}} \times \mathfrak{R}_{\text{to}} \times \text{DID} \times \tau_{\text{eff}} \\
\text{CrossingDenied} &: \mathfrak{R}_{\text{from}} \times \mathfrak{R}_{\text{to}} \times \text{DID} \times \text{Reason}
\end{aligned}
$$

### Ε10.3 Saga-Events

$$
\begin{aligned}
\text{SagaStarted} &: \text{SagaId} \times \text{Type} \times [\mathfrak{R}] \times \text{DID} \\
\text{SagaStepCompleted} &: \text{SagaId} \times \text{StepId} \times \mathfrak{R} \\
\text{SagaCompleted} &: \text{SagaId} \times \mathbb{B} \times \mathbb{N}
\end{aligned}
$$

---

## §11 StateGraph-Integration

### Σ11.1 Dependency-Graph

$$
\begin{aligned}
\mathfrak{R} &\xrightarrow{\text{DependsOn}} \text{Identity} \\
\mathfrak{R} &\xrightarrow{\text{DependsOn}} \text{Trust} \\
\mathfrak{R} &\xrightarrow{\text{DependsOn}} \text{Mana} \\
\mathfrak{R} &\xrightarrow{\text{Aggregates}} \text{Storage} \\
\mathfrak{R} &\xrightarrow{\text{Aggregates}} \text{Packages} \\
\mathfrak{R} &\xrightarrow{\text{Triggers}} \text{Event} \\
\mathfrak{R} &\xrightarrow{\text{Validates}} \text{Rules} \\
\mathfrak{R} &\xleftrightarrow{\text{Bidirectional}} \text{P2P} \\
\mathfrak{R} &\xleftrightarrow{\text{Bidirectional}} \text{ECLVM}
\end{aligned}
$$

---

## §12 Theoreme

### Τ12.1 Regel-Monotonie

$$\forall \mathfrak{R}_c, \mathfrak{R}_p : \mathfrak{R}_c \subset \mathfrak{R}_p \Rightarrow |\mathcal{R}_c| \geq |\mathcal{R}_p|$$

**Beweis:** Direkt aus Κ1. $\square$

### Τ12.2 Crossing-Dämpfung

$$\tau_{\text{eff}} \leq \tau_{\text{origin}}$$

**Beweis:** $\gamma \in (0, 1] \Rightarrow \tau \cdot \gamma \leq \tau$. $\square$

### Τ12.3 Trust-Isolation

$$\text{Update}(\mathcal{T}_{\mathfrak{R}_A}) \not\Rightarrow \text{Change}(\mathcal{T}_{\mathfrak{R}_B})$$

**Beweis:** Folgt aus unabhängigen Trust-Räumen per Κ24. $\square$

### Τ12.4 Saga-Atomarität

$$\text{Saga}(S_1, ..., S_n) \in \{\text{All-Commit}, \text{All-Rollback}\}$$

**Beweis:** Per Konstruktion: Fehler in $S_k$ triggert $C_{k-1}, ..., C_1$. $\square$

---

## §13 Zusammenfassung

$$
\begin{array}{|l|c|l|}
\hline
\textbf{Komponente} & \textbf{Axiom} & \textbf{Eigenschaft} \\
\hline
\text{Regelvererbung} & K1 & \mathcal{R}_c \supseteq \mathcal{R}_p \\
\text{Quadratic Voting} & K21 & v = \sqrt{\text{tokens}} \\
\text{Saga-Pattern} & K22 & \text{Atomare Cross-Realm-Ops} \\
\text{Crossing-Dämpfung} & K23 & \tau_{\text{eff}} = \tau \cdot \gamma \\
\text{Lokaler Trust} & K24 & \mathcal{T}_A \perp\!\!\!\perp \mathcal{T}_B \\
\hline
\end{array}
$$

**Kernaussage:**
$$\mathfrak{R} = \text{Souveräne Einheit mit isolierter Governance, Trust und Storage}$$

---

**∎ QED**
