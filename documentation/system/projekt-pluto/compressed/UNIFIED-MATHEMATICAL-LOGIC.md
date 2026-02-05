# 🧮 Erynoa/Pluto: Unified Mathematical Logic

> **Version:** 1.9.0 (FINAL)
> **Datum:** 2026-02-04
> **Status:** ✅ Phase 6 COMPLETE + state.rs + concept-v4 + **Κ0 Passkey-Primacy**
> **Quellen:** entities.json, relations.json, formulas.json, constraints.json, integration.json, migrations.json, 06-eclvm-wasm-migration.json, 08-STATE-KERNGEDANKEN.md, 09-TRUST-GAS-MANA-DREIEINIGKEIT.pluto.md, 10-IDENTITY-MULTI-DID-ARCHITEKTUR.pluto.md, 14-SHARDING-ARCHITEKTUR.pluto.md, 16-REALM-GOVERNANCE.pluto.md, 17-REALM-URL-RESOURCE-ADDRESSING.pluto.md, **00-OVERVIEW.md**, **02-ZIEL-ARCHITEKTUR.md**, **03-BEZIEHUNGSMATRIX.md**, **11-PACKAGEMANAGER-BLUEPRINT-TRANSFORMATION.md**, **12-PACKAGEMANAGER-SYNERGIEN-FEATURES.md**, **13-REALM-ARCHITEKTUR-ISOLATION.md**, **18-AGENT-SHELL-ZUGRIFF.md**, **07-SYNERGISTISCHE-INTEGRATION.md**, **19-USE-CASES-DEZENTRALER-STORAGE.md**, **01-IST-ANALYSE.md**, **04-PHASENPLAN.md**, **05-MIGRATION-SCRIPTS.md**, **06-ECLVM-WASM-MIGRATION.md**, **16.1-LEGACY-MEGA-REFACTORING-PLAN.md**, **16.2-LEGACY-PHASE1-QUICKSTART.md**, **backend/src/core/state.rs** (21,495 LOC), **concept-v4/LOGIC.md**, **concept-v4/P2P-PRIVATE-RELAY-LOGIC.md**, **concept-v4/LOGIC-GENERATIVE-REALMS.md**

---

## Inhaltsverzeichnis

1. [Das Pluto-Universum](#i-das-pluto-universum)
2. [Entitäten-Ontologie](#ii-entitäten-ontologie)
3. [Trust-Gas-Mana Dreieinigkeit](#iii-trust-gas-mana-dreieinigkeit)
4. [Identity-Architektur](#iv-identity-architektur)
5. [Relationsalgebra](#v-relationsalgebra)
6. [Nervensystem-Integration](#vi-nervensystem-integration)
7. [State-Kerngedanken](#vii-state-kerngedanken)
8. [Execution Engine (ECLVM/WASM)](#viii-execution-engine-eclvmwasm)
9. [Sharding-Architektur](#ix-sharding-architektur)
10. [Realm-Governance](#x-realm-governance)
11. [URL-Resource-Addressing](#xi-url-resource-addressing)
12. [Migrations-Algebra](#xii-migrations-algebra)
13. [Operations & Synergien](#xiii-operations--synergien)
14. [Axiom-Katalog (Constraints)](#xiv-axiom-katalog-constraints)
15. [Kritische Pfade & Performance](#xv-kritische-pfade--performance)
16. [Globale Verbindungsanalyse](#xvi-globale-verbindungsanalyse)
17. [Zusammenfassung & Haupttheorem](#xvii-zusammenfassung--haupttheorem)
18. [**Ziel-Architektur (Target-State)**](#xviii-ziel-architektur-target-state) 🆕
19. [**Erweiterte Beziehungsmatrix**](#xix-erweiterte-beziehungsmatrix) 🆕
20. [**Implementierungs-Roadmap**](#xx-implementierungs-roadmap) 🆕
21. [**Package-Manager-Algebra**](#xxi-package-manager-algebra) 📦
22. [**Package-Synergien & Emergente Features**](#xxii-package-synergien--emergente-features) 📦
23. [**Realm-Isolation-Algebra**](#xxiii-realm-isolation-algebra) 🏰
24. [**Realm-Synergien & Cross-Realm-Operationen**](#xxiv-realm-synergien--cross-realm-operationen) 🏰
25. [**Agent-Shell-Algebra**](#xxv-agent-shell-algebra) 🤖
26. [**Agent-Synergien und Compute-Algebra**](#xxvi-agent-synergien-und-compute-algebra) 🤖
27. [**Synergistische System-Integration**](#xxvii-synergistische-system-integration) 🧬
28. [**Dezentraler Storage & Use-Case-Algebra**](#xxviii-dezentraler-storage--use-case-algebra) 📦
29. [**Appendix C: IST-Zustand-Defizite**](#appendix-c-ist-zustand-defizite) 🔴
30. [**Appendix D: Phasenplan-Timeline**](#appendix-d-phasenplan-timeline) 📅
31. [**Appendix E: Legacy-Refactoring-Algebra**](#appendix-e-legacy-refactoring-algebra) 🔧
32. [**Appendix F: WASM-Migrations-Algebra (Extended)**](#appendix-f-wasm-migrations-algebra-extended) 🔄
33. [**Appendix G: Code-Mapping (state.rs ↔ Formalisierung)**](#appendix-g-code-mapping-staters--formalisierung) 🔗
34. [**Appendix H: Concept-V4 Integration (Kategorientheorie + P2P-Relay + Generative Realms)**](#appendix-h-concept-v4-integration-kategorientheorie--p2p-relay--generative-realms) 🆕

---

## I. Das Pluto-Universum

### I.1 Fundamentaldefinition

Das Erynoa-System ist formal definiert als 8-Tupel:

$$\boxed{\mathbb{U}_{\text{Pluto}} = \langle \mathcal{E}, \mathcal{R}, \mathcal{O}, \mathcal{K}, \mathcal{S}, \mathcal{N}, \Psi, \Phi \rangle}$$

| Symbol        | Menge          | Kardinalität         | Bedeutung                   |
| ------------- | -------------- | -------------------- | --------------------------- |
| $\mathcal{E}$ | Entities       | 8 Haupttypen         | Ontologische Grundbausteine |
| $\mathcal{R}$ | Relations      | 6 Typen, 110+ Kanten | Topologische Verbindungen   |
| $\mathcal{O}$ | Operations     | 20+                  | State-Transitionen          |
| $\mathcal{K}$ | Constraints    | 26+ Axiome           | Invarianten                 |
| $\mathcal{S}$ | Synergies      | 11+                  | Emergente Kopplungen        |
| $\mathcal{N}$ | Nervous System | 5 Komponenten        | Event-Dispatch-System       |
| $\Psi$        | Execution      | 2 Modi               | ECLVM (Legacy ↔ WASM)       |
| $\Phi$        | Migrations     | 7 Operationen        | Code-Transformationen       |

### I.2 Schichten-Architektur

$$\mathcal{L} = \{L_0, L_1, L_2, L_3, L_4\}$$

```
L₀ (Core):           {Σ, Hub}                          — Nervensystem-Kern
L₁ (Engines):        {E_τ, E_event, E_formula, E_consensus}  — Verarbeitung
L₂ (Domain):         {ι, ρ, G, S, Q}                   — Domänen-Logik
L₃ (Infrastructure): {Ω, P2P, Ψ}                       — Infrastruktur
L₄ (Protection):     {A, D, Q, C} = 🛡️                 — Schutz
```

**Datenfluss:**
$$\text{Primary:} \quad \Sigma \rightarrow \text{Hub} \rightarrow L_1 \rightarrow L_2 \rightarrow L_3$$
$$\text{Feedback:} \quad L_4 \vdash (L_1 \cup L_2 \cup L_3)$$

---

## II. Entitäten-Ontologie

> **Quelle:** entities.json

### II.1 UnifiedState $\Sigma$ — Das Gehirn

$$\boxed{\Sigma = \Sigma_{\text{Trust}} \times \Sigma_{\text{Identity}} \times \Sigma_{\text{Realm}} \times \Sigma_{\text{Event}} \times \Sigma_{\text{Storage}} \times \Sigma_{\text{Protection}}}$$

| Komponente                   | Inhalt                                     |
| ---------------------------- | ------------------------------------------ |
| $\Sigma_{\text{Trust}}$      | Atomic Counters, TrustVectors, Relations   |
| $\Sigma_{\text{Identity}}$   | DIDs, Credentials, Delegations             |
| $\Sigma_{\text{Realm}}$      | Realm-Graph, Policies, Memberships         |
| $\Sigma_{\text{Event}}$      | EventLog, Checkpoints, MerkleTrees         |
| $\Sigma_{\text{Storage}}$    | Blob-Indices, Quotas                       |
| $\Sigma_{\text{Protection}}$ | CircuitBreaker, AnomalyMetrics, SystemMode |

**Axiome:** K9, K_StateConsistency

### II.2 SynapseHub — Die Synapsen

$$\text{Hub} = (\text{Observers}: \text{Map}\langle\text{Component}, \text{List}\langle\text{Obs}\rangle\rangle, \text{Graph}: \text{StateGraph})$$

**Komponenten:**

- EventBus: Ingress (P2P→Core), Egress (Core→P2P)
- Observers: TrustObserver, RealmObserver, etc.
- StateGraph: DependsOn, Triggers, Validates

### II.3 Identity $\iota$ — Der Akteur

$$\boxed{\iota = (\text{did}: H_{256}, \text{ns}: \mathcal{N}, \vec{\tau}: \mathbb{T}^6, \nu: \mathbb{N})}$$

| Komponente   | Beschreibung                                   |
| ------------ | ---------------------------------------------- |
| did          | Format: `did:erynoa:...`                       |
| trust_vector | TrustVector6D — Emergentes Vertrauen           |
| credentials  | Set⟨VerifiableCredential⟩                      |
| resources    | {mana: Balance, gas: Budget}                   |
| mode         | ∈ {Interactive, AgentManaged, Ephemeral, Test} |

**Axiome:** K2, K3, K6, K7, K8

### II.4 Realm $\rho$ — Das Organ

$$\boxed{\rho = (\text{id}: H_{256}, \text{parent}: \rho?, \text{rules}: \mathcal{P}(\text{Rule}), M: \mathcal{P}(\iota))}$$

| Komponente | Typ                                                |
| ---------- | -------------------------------------------------- |
| governance | GovernanceConfig ∈ {Quadratic, Reputation, ...}    |
| resources  | {stores: Map⟨Name, StoreSchema⟩, blobs: BlobStore} |
| policies   | Set⟨ECLPolicy⟩                                     |
| isolation  | IsolationLevel                                     |

**Axiome:** K1, K22, K23, K24

### II.5 ECLVM $\Psi$ — Die Execution Engine

$$\Psi: \text{State} \times \text{Policy} \rightarrow \text{Result}$$

| Komponente | Beschreibung                     |
| ---------- | -------------------------------- |
| engine     | WasmPolicyEngine (Wasmtime)      |
| bridge     | WasmStateBridge (Host-Functions) |
| security   | Sandboxing, Fuel-Metering (Gas)  |
| mode       | ∈ {Legacy, Wasm, Auto}           |

### II.6 Storage $\Omega$ — Das Gedächtnis

$$\Omega_{\text{Store}} = (\text{blobs}: \text{Map}\langle\text{Hash}, \text{Blob}\rangle, \text{indices}: \text{Map}\langle\text{Key}, \text{Value}\rangle)$$

| Komponente | Beschreibung                                 |
| ---------- | -------------------------------------------- |
| blob_store | Content-Addressed (Blake3), Dynamic Chunking |
| kv_store   | Fjall/LSM-Tree                               |
| tiers      | [Hot, Warm, Cold]                            |
| cost_model | Trust/Gas/Mana-based                         |

**Axiome:** K10, K26

### II.7 Package $\pi$ — Die Blueprints

$$\pi = (\text{cid}: H_{256}, \text{meta}: \text{Manifest}, \text{content}: \text{BlobId})$$

| Komponente | Beschreibung                                 |
| ---------- | -------------------------------------------- |
| manifest   | {license: LicenseType, features: FeatureMap} |
| lifecycle  | ∈ {Draft, Published, Deprecated, Revoked}    |
| resolution | DependencySolver (SAT)                       |

**Axiome:** K_PkgIntegrity

### II.8 Protection 🛡️ — Das Immunsystem

$$\text{🛡️} = (\text{mode}: \text{SystemMode}, \text{metrics}: \text{AnomalyVector})$$

| Komponente      | Beschreibung                              |
| --------------- | ----------------------------------------- |
| circuit_breaker | Threshold-based triggers                  |
| system_mode     | ∈ {Normal, Degraded, Emergency, Recovery} |
| detectors       | [Anomaly, Diversity, AntiCalcification]   |

**Axiome:** K4, K19

---

## III. Trust-Gas-Mana Dreieinigkeit

> **Quelle:** 09-TRUST-GAS-MANA-DREIEINIGKEIT.pluto.md, formulas.json

### III.1 Negation der Token-Metapher

$$\boxed{\neg(\text{Trust} \equiv \text{Token}) \land \neg(\text{Gas} \equiv \text{Coin}) \land \neg(\text{Mana} \equiv \text{Credit})}$$

**Theorem (Emergenz):**
$$\text{Trust}, \text{Gas}, \text{Mana} \in \text{EmergentProperties}(\mathcal{S})$$

### III.2 Organische Triaden-Metapher

| Symbol         | Metapher       | Funktion            | Regeneration    |
| -------------- | -------------- | ------------------- | --------------- |
| $\tau$ (Trust) | 🫀 Immunsystem | Entitäts-Bewertung  | Emergent        |
| $\gamma$ (Gas) | ⚡ Muskelkraft | Compute-Budget      | ∅ (erschöpfend) |
| $\mu$ (Mana)   | 🌊 Atem        | Bandwidth-Kapazität | Kontinuierlich  |

### III.3 Kausalkette

$$\text{Existenz} \xrightarrow{\text{Handlung}} \text{Beobachtung} \xrightarrow{\text{Bewertung}} \tau \xrightarrow{\text{Skalierung}} (\gamma, \mu)$$

**Initialzustand (Newcomer):**
$$\tau_0 = 0.1, \quad \mu_0 = 10{,}000, \quad \gamma_{\text{cost}} = 1.9 \cdot \gamma_{\text{base}}$$

### III.4 Trust-Vektor $\mathbb{T}$ (6 Dimensionen)

$$\boxed{\mathbb{T} = (R, I, C, P, V, \Omega) \in [0,1]^6}$$

| Dim      | Name           | Semantik                        | Systemeffekt                       |
| -------- | -------------- | ------------------------------- | ---------------------------------- |
| $R$      | Reliability    | Versprechen-Einhaltung          | $\gamma_{\text{budget}}$           |
| $I$      | Integrity      | Konsistenz                      | $w_{\text{vote}}$                  |
| $C$      | Competence     | Fähigkeitsnachweis              | $\text{Access}_{\text{complex}}$   |
| $P$      | Prestige       | Externe Attestation             | $\text{Influence}_{\text{social}}$ |
| $V$      | Vigilance      | Anomalie-Erkennung              | $w_{\text{protection}}$            |
| $\Omega$ | Omega (Wisdom) | $\int_{\text{past}}$ Handlungen | $\mu_{\text{regen}}$               |

**Gewichtete Norm:**
$$\|\mathbb{T}\|_w = \sqrt{\sum_{d \in \{R,I,C,P,V,\Omega\}} w_d \cdot T_d^2}, \quad \sum_d w_d = 1$$

### III.5 Asymmetrie-Axiom (K4)

$$\boxed{\Delta^-(d) = \lambda_d \cdot \Delta^+(d)}$$

| Dimension    | $\lambda$ | Interpretation        |
| ------------ | --------- | --------------------- |
| $R, I, C, P$ | 1.5       | Negative 50% stärker  |
| $V, \Omega$  | 2.0       | Negative 100% stärker |

### III.6 Trust-Update-Formel

$$\vec{\tau}' = \text{clamp}(\vec{\tau} + \delta \times \text{AsymmetryMatrix}, 0, 1)$$

### III.7 Trust-Level-Klassen

$$[0,1] = \bigcup_k \mathcal{T}_k$$

| Klasse      | $\tau$-Bereich | $\mu_{\max}$ | $\gamma_{\text{cost}}$ | Rechte                     |
| ----------- | -------------- | ------------ | ---------------------- | -------------------------- |
| Newcomer    | $[0.0, 0.2)$   | 10k–30k      | $1.8\text{–}2.0\times$ | Basic only                 |
| Established | $[0.2, 0.5)$   | 30k–500k     | $1.5\text{–}1.8\times$ | Voting, Packages           |
| Trusted     | $[0.5, 0.8)$   | 500k–800k    | $1.2\text{–}1.5\times$ | Publish, Realm-Gründung    |
| Veteran     | $[0.8, 1.0]$   | 800k–1M+     | $1.0\text{–}1.2\times$ | Publish (No-Review), Admin |

### III.8 Gas $\gamma$ — Metriken

**Budget-Emergenz:**
$$\gamma_{\text{budget}} = \gamma_{\text{base}} \cdot (1 + \tau_R \cdot \phi_\gamma), \quad \phi_\gamma = 2.0$$

**Kosten-Skalierung:**
$$\gamma_{\text{cost}}(\text{op}) = \gamma_{\text{base}}(\text{op}) \cdot (2 - \tau_R)$$

**Erschöpfungs-Invariante (K11):**
$$\boxed{\gamma(t+1) \leq \gamma(t) \quad \text{während Execution}}$$

**OpCode-Kostentabelle:**

| OpCode        | $\gamma_{\text{base}}$ |
| ------------- | ---------------------- |
| PUSH/CONST    | 1                      |
| ADD/SUB       | 2                      |
| MUL           | 3                      |
| DIV/MOD       | 5                      |
| LOAD/STORE    | 5/10                   |
| HOST_CALL     | 50                     |
| CRYPTO_VERIFY | 500                    |
| ZK_VERIFY     | 10,000                 |

### III.9 Mana $\mu$ — Metriken

**Kapazitäts-Emergenz:**
$$\mu_{\max} = \mu_{\text{base}} \cdot (1 + \tau_\Omega \cdot \phi_\mu), \quad \phi_\mu = 100$$

**Regenerations-Funktion:**
$$\frac{d\mu}{dt} = r_{\text{base}} \cdot (1 + \tau_\Omega \cdot \psi_\mu), \quad r_{\text{base}} = 100/\text{sec}, \psi_\mu = 10$$

**Regenerations-Invariante (K13):**
$$\mu(t) = \min(\mu_{\max}, \mu(t-1) + r)$$

**I/O-Kostentabelle:**

| Operation       | $\mu_{\text{base}}$ |
| --------------- | ------------------- |
| STORAGE_GET/PUT | 5/10                |
| P2P_PUBLISH     | 10                  |
| P2P_CONNECT     | 20                  |
| REALM_CROSSING  | 50                  |
| SAGA_STEP       | 30                  |

### III.10 Kostenalgebra $\kappa$

**Kostenvektor:**
$$\kappa = (\gamma, \mu, \varrho) \in \mathbb{R}^+ \times \mathbb{R}^+ \times [0,1]$$

**Sequentielle Komposition ($\oplus$):**
$$\kappa_1 \oplus \kappa_2 = (\gamma_1 + \gamma_2, \mu_1 + \mu_2, 1 - (1-\varrho_1)(1-\varrho_2))$$

**Parallele Komposition ($\otimes$):**
$$\kappa_1 \otimes \kappa_2 = (\max(\gamma_1, \gamma_2), \mu_1 + \mu_2, \max(\varrho_1, \varrho_2))$$

**Trust-Adjustierung:**
$$\kappa_{\text{eff}} = (\gamma \cdot (2 - \tau_R), \mu, \varrho)$$

### III.11 Feedback-Loops

**Selbstverstärkung (Matthäus-Effekt):**
$$\tau \uparrow \implies (\mu, \gamma) \uparrow \implies P(\text{Erfolg}) \uparrow \implies \tau \uparrow$$

**Selbstauslöschung (Angreifer):**
$$\tau \downarrow \implies (\mu, \gamma) \downarrow \implies P(\text{Erschöpfung}) \uparrow \implies \text{Isolation}$$

### III.12 Sybil-Unmöglichkeitsbeweis

| Metrik              | 1 Veteran ($\tau=0.9$) | 100 Sybils ($\tau=0.0$) |
| ------------------- | ---------------------- | ----------------------- |
| $\mu_{\text{init}}$ | 910,000                | 1,000,000               |
| $r_{\text{total}}$  | 60,000/min             | 6,000/min               |
| Sustained Rate      | 1,000/min              | 100/min                 |

$$\boxed{\text{Rate}_{\text{Veteran}} = 10 \times \text{Rate}_{\text{Sybil-Cluster}}}$$

---

## IV. Identity-Architektur

> **Quelle:** 10-IDENTITY-MULTI-DID-ARCHITEKTUR.pluto.md

### IV.0 Axiom Κ0 — Passkey-Primacy (Authentifizierungs-Fundament)

> **🔑 FUNDAMENTAL:** Passkey/WebAuthn ist die **einzige** Hardware-gebundene Authentifizierungsmethode. Alle Identitäten, DIDs und Ableitungen basieren auf dieser Wurzel.

$$\boxed{\forall \iota \in \mathcal{I}_{\text{Human}}: \exists! \, pk \in \text{Passkey}_{\text{HW}} : \text{auth}(\iota) = \text{verify}(pk)}$$

**Aussage:** Jede menschliche Identität ($\text{Self}\_$) erfordert **genau einen** Hardware-gebundenen Passkey zur Authentifizierung.

**Formal:**
$$\text{Passkey}_{\text{HW}} = \langle K_{priv}^{\text{TPM/SE}}, K_{pub}, \text{RP-ID}, \text{CredentialId}, \text{Counter} \rangle$$

**Ableitungskette (Top-Down):**

```
┌─────────────────────────────────────────────────────────────────┐
│                    PASSKEY (Hardware-Bound)                     │
│           Einzige Authentifizierungs-Wurzel (Κ0)                │
│                  ══════════════════════                         │
│                          │                                      │
│                          ▼                                      │
│                 ┌─────────────────┐                             │
│                 │    Root-DID     │ M_0 Interactive Mode        │
│                 │ (Self_ Namespace)│ τ = 1.0 (voller Trust)     │
│                 └────────┬────────┘                             │
│                          │                                      │
│        ┌─────────────────┼─────────────────┐                    │
│        ▼                 ▼                 ▼                    │
│  ┌──────────┐     ┌──────────┐      ┌──────────┐                │
│  │Device-DID│     │Agent-DID │      │Realm-DID │                │
│  │(Self_)   │     │(Spirit)  │      │(Circle)  │                │
│  │τ = 1.0   │     │τ = 0.8   │      │τ = 1.0   │                │
│  └──────────┘     └──────────┘      └──────────┘                │
│        │                │                 │                     │
│        ▼                ▼                 ▼                     │
│   Wallet-Adr.     Sub-Agents       Realm-Members                │
└─────────────────────────────────────────────────────────────────┘
```

**Implikationen:**

| Eigenschaft               | Aussage                                                       |
| ------------------------- | ------------------------------------------------------------- |
| **Uniqueness**            | $\|\text{Passkey}(\iota)\| = 1$ (genau ein HW-Passkey pro ID) |
| **Hardware-Binding**      | $K_{priv} \in \text{TPM} \cup \text{SecureEnclave}$           |
| **Non-Exportability**     | $\nexists \, f: K_{priv} \rightarrow \text{Plaintext}$        |
| **Bootstrap-Requirement** | $\text{bootstrap}(\iota) \Rightarrow pk \neq \bot$            |
| **Mode-Derivation**       | $M_0 = \text{Interactive} \Leftrightarrow pk \neq \bot$       |

**Mode-Abhängigkeit:**

$$
\mathcal{M}(\iota) = \begin{cases}
M_0 \, (\text{Interactive}) & \text{if } pk \neq \bot \land \text{HW-Present} \\
M_1 \, (\text{AgentManaged}) & \text{if } \text{delegated-from}(M_0) \\
M_2 \, (\text{Ephemeral}) & \text{if } \text{no-persist} \\
M_3 \, (\text{Test}) & \text{if } \text{deterministic-seed}
\end{cases}
$$

**Trust-Konsequenz:**
$$\tau(M_i) = \tau(M_0) \cdot \text{penalty}(M_i), \quad \text{penalty}(M_0) = 1.0$$

**Code-Entsprechung (state.rs):**

```rust
pub struct IdentityState {
    passkey_manager: Option<SharedPasskeyManager>,  // Κ0: MUST be Some for Interactive
    // ...
}

// Bootstrap erfordert Passkey
fn bootstrap_interactive(&self) -> Result<()> {
    let pk = self.passkey_manager.as_ref()
        .ok_or(Error::PasskeyNotAvailable)?;  // Κ0 enforced
    // ...
}
```

**Sicherheitsgarantie:**
$$\boxed{\text{Compromise}(\text{Software-Key}) \not\Rightarrow \text{Compromise}(\text{Identity})}$$

Da $K_{priv}$ nie die Hardware verlässt, kann ein kompromittiertes Gerät keine Identität permanent übernehmen.

---

### IV.1 DID-Definition

$$\boxed{\text{DID} = \langle \mathcal{N}, \mathcal{U}, K_{pub} \rangle}$$

wobei:

- $\mathcal{N} \in \{\text{Self}, \text{Guild}, \text{Spirit}, \text{Thing}, \text{Vessel}, \text{Source}, \text{Craft}, \text{Vault}, \text{Pact}, \text{Circle}\}$
- $\mathcal{U} = H_{\text{Blake3}}(\mathcal{N} \| K_{pub})$ — UniversalId (32 Bytes)
- $K_{pub}$ — Ed25519 Public Key (32 Bytes)

**Format:**
$$\texttt{did:erynoa:}\langle\text{namespace}\rangle\texttt{:}\langle\text{universal-id-hex}\rangle$$

### IV.2 Namespace-Kodierung

| Byte   | Namespace | Semantik               | Entitäts-Typ |
| ------ | --------- | ---------------------- | ------------ |
| `0x01` | Self      | Natürliche Person      | Mensch       |
| `0x02` | Guild     | Organisation/DAO       | Kollektiv    |
| `0x03` | Spirit    | KI-Agent               | Autonom      |
| `0x04` | Thing     | IoT-Gerät              | Physisch     |
| `0x05` | Vessel    | Container/Transport    | Mobil        |
| `0x06` | Source    | Datenquelle/API        | Feed         |
| `0x07` | Craft     | Service/Dienstleistung | Funktion     |
| `0x08` | Vault     | Speicher/Safe          | Persistent   |
| `0x09` | Pact      | Vertrag/Vereinbarung   | Bindend      |
| `0x0A` | Circle    | Gruppe/Realm           | Kollaborativ |

### IV.3 Hierarchische Ableitung

**DID-Baum:**
$$\mathcal{T} = \langle \text{Root}, \mathcal{D}, \mathcal{A}, \mathcal{R} \rangle$$

**Ableitungsfunktionen $\partial$:**
$$\partial_{\text{device}}(\text{Root}, i) = \text{DID}(\text{Self}, H_{\text{Blake3}}(K_{pub}^{\text{Root}} \| \texttt{"device"} \| i))$$
$$\partial_{\text{agent}}(\text{Root}, i) = \text{DID}(\text{Spirit}, H_{\text{Blake3}}(K_{pub}^{\text{Root}} \| \texttt{"agent"} \| i))$$
$$\partial_{\text{realm}}(\text{Root}, \mathcal{U}_r) = \text{DID}(\text{Circle}, H_{\text{Blake3}}(K_{pub}^{\text{Root}} \| \texttt{"realm"} \| \mathcal{U}_r))$$

**BIP44-Pfade:**
$$m / 44' / \text{erynoa}' / 0' / \langle\text{zweck}\rangle / \langle\text{index}\rangle$$

### IV.4 Betriebsmodi $\mathcal{M}$

| $M_i$ | Name         | Signatur-Typ            | Trust-Penalty $\tau$ | Realm-fähig |
| ----- | ------------ | ----------------------- | -------------------- | ----------- |
| $M_0$ | Interactive  | WebAuthn (HW-bound)     | 1.0                  | ✓           |
| $M_1$ | AgentManaged | Software-Key (autonom)  | 0.8                  | ✓           |
| $M_2$ | Ephemeral    | Flüchtig (kein Persist) | 0.5                  | ✗           |
| $M_3$ | Test         | Deterministisch (Fake)  | 1.0                  | ✓           |

**Effektiver Trust:**
$$\mathbb{T}_{\text{eff}}(s) = \mathbb{T}_{\text{raw}}(s) \cdot \tau(\mathcal{M}(s))$$

### IV.5 Delegation $\Delta$ (K8)

**Trust-Vererbung:**
$$\boxed{s \rhd s' \Rightarrow \mathbb{T}(s') \leq \tau_{\text{factor}} \cdot \mathbb{T}(s)}$$

**Delegation-Struktur:**
$$\Delta = \langle \text{id}, s, s', \tau, \mathcal{C}, t_{\text{exp}}?, t_{\text{create}}, \rho \rangle$$

**Capability-Algebra:**
$$\mathcal{C} = \{\star, \text{read}:r, \text{write}:r, \text{execute}:a, \text{delegate}:n, \text{attest}:\vec{t}, \text{custom}:k:p\}$$

**Ketten-Trust-Propagation:**
$$\mathbb{T}_{\text{eff}}(s_n) = \mathbb{T}(s_0) \cdot \prod_{i=0}^{n-1} \tau_i$$

**Tiefenbegrenzung:**
$$\text{depth}(\Delta) \leq n_{\max}$$

### IV.6 Wallet-Integration

**Chain-Derivation-Regeln:**
$$\partial_{\text{EVM}}(\text{DID}) = \text{keccak256}(K_{\text{secp256k1}})_{[12..32]}$$
$$\partial_{\text{Solana}}(\text{DID}) = \text{base58}(K_{\text{Ed25519}})$$
$$\partial_{\text{Cosmos}}(\text{DID}) = \text{bech32}(\texttt{"cosmos"}, \text{ripemd160}(\text{sha256}(K_{\text{secp256k1}})))$$

### IV.7 P2P-Konvertierung

$$\text{DID} \xleftrightarrow{K_{pub}} \text{PeerId} \xleftrightarrow{\mathcal{U}} \text{UniversalId}$$

**Alle drei teilen denselben Ed25519 Public Key als Fundament.**

### IV.8 Realm-Membership

**Isolationsprinzip:**
$$\forall r, r' \in \mathcal{R}: r \neq r' \Rightarrow \text{State}(r) \cap \text{State}(r') = \emptyset$$

**Rollen-Multiplikatoren:**

| $\rho$    | $\mu_\rho$ |
| --------- | ---------- |
| Member    | 1.0        |
| Moderator | 1.1        |
| Admin     | 1.2        |
| Owner     | 1.3        |

$$\mathbb{T}_{\text{eff}}^{\mathcal{R}} = \min(1.0, \mathbb{T}_{\text{local}} \cdot \mu_\rho)$$

---

## V. Relationsalgebra

> **Quelle:** relations.json

### V.1 Relationstypen

| Symbol             | Name          | Semantik                                              | Transitivität |
| ------------------ | ------------- | ----------------------------------------------------- | ------------- |
| $\triangleright$   | DependsOn     | $A \triangleright B \equiv A$ requires $B$            | ✓             |
| $\rightarrow$      | Triggers      | $\text{state\_change}(A) \implies \text{event}(B)$    | ✗             |
| $\vdash$           | Validates     | $A \vdash B \equiv A$ asserts invariants on $B$       | ✗             |
| $\uplus$           | Aggregates    | $A \uplus B \equiv A$ contains/manages $B$            | ✓             |
| $\leftrightarrow$  | Bidirectional | $(A \triangleright B) \land (B \triangleright A)$     | ✗             |
| $\rightsquigarrow$ | Updates       | $A \rightsquigarrow B \equiv A$ modifies state of $B$ | ✗             |

### V.2 Identity-Relationen

| From      | To      | Type                       | Axiom | Formalism                                                                             |
| --------- | ------- | -------------------------- | ----- | ------------------------------------------------------------------------------------- |
| $\tau$    | $\iota$ | $\triangleright$           | K6    | $\tau(x) \triangleright \iota(x) \equiv$ trust requires identity                      |
| $\iota$   | $\tau$  | $\rightarrow$              | K2    | $\forall\iota_{\text{new}}: \text{spawn}(\iota) \rightarrow \text{init}(\tau(\iota))$ |
| Event     | $\iota$ | $\triangleright$           | K9    | $\forall e: \text{author}(e) \in \iota$                                               |
| Gateway   | $\iota$ | $\triangleright$, $\vdash$ | K23   | crossing $\implies$ verify($\iota$)                                                   |
| P2P.Swarm | $\iota$ | $\triangleright$           | —     | PeerId $\equiv$ DeviceDID $\in \iota$                                                 |
| Anomaly   | $\iota$ | $\vdash$                   | K26   | 🛡️.A $\vdash \neg$anomalous($\iota$)                                                  |
| Hub       | $\iota$ | $\uplus$                   | —     | Hub $\uplus$ Observer($\iota$)                                                        |
| $\Psi$    | $\iota$ | $\triangleright$           | —     | $\Psi$.call $\triangleright$ caller $\in \iota$                                       |

### V.3 Trust-Relationen

| From      | To     | Type             | Axiom | Formalism                                     |
| --------- | ------ | ---------------- | ----- | --------------------------------------------- |
| Gateway   | $\tau$ | $\triangleright$ | K23   | crossing $\implies \tau > \theta_{\min}$      |
| Saga      | $\tau$ | $\triangleright$ | K24   | saga.exec $\triangleright \tau$(participants) |
| Gas       | $\tau$ | $\triangleright$ | —     | $\gamma_{\text{limit}} = f(\tau)$             |
| Mana      | $\tau$ | $\triangleright$ | —     | $\mu_{\text{regen}} = g(\tau)$                |
| Formula   | $\tau$ | $\triangleright$ | K15   | F.input $\triangleright \tau$                 |
| Consensus | $\tau$ | $\triangleright$ | K18   | vote_weight $= h(\tau)$                       |
| Diversity | $\tau$ | $\vdash$         | K19   | 🛡️.D $\vdash$ Gini($\tau$) $< \theta$         |

### V.4 Realm-Relationen

| From    | To      | Type              | Axiom | Formalism                                |
| ------- | ------- | ----------------- | ----- | ---------------------------------------- |
| $\rho$  | $\iota$ | $\triangleright$  | K22   | $\rho$.members $\subseteq \iota$         |
| $\rho$  | $\tau$  | $\leftrightarrow$ | K22   | $\rho \leftrightarrow \tau$              |
| Gateway | $\Psi$  | $\triangleright$  | K23   | G.policy $\rightarrow \Psi$.eval         |
| Saga    | $\Psi$  | $\triangleright$  | K24   | S.step $\rightarrow \Psi$.exec           |
| Quota   | 🛡️      | $\rightarrow$     | K22   | quota_exceeded $\rightarrow$ trigger(🛡️) |

### V.5 Shell-Relationen

| From  | To         | Type                                 | Axiom | Formalism                                                                                                |
| ----- | ---------- | ------------------------------------ | ----- | -------------------------------------------------------------------------------------------------------- |
| Shell | $\iota$    | $\triangleright$                     | K8    | shell.access $\triangleright$ valid(agent_did)                                                           |
| Shell | $\tau$     | $\triangleright$, $\rightsquigarrow$ | K6    | shell.action $\triangleright \tau > \theta_{\text{shell}}$; result(action) $\rightsquigarrow \Delta\tau$ |
| Shell | $\Psi$     | $\triangleright$                     | K23   | shell $\rightarrow \Psi$.eval(ShellPolicy)                                                               |
| Shell | Capability | $\triangleright$                     | K8    | action $\in$ capabilities(agent)                                                                         |
| Shell | Event      | $\rightarrow$                        | K9    | shell.action $\rightarrow$ audit_event                                                                   |
| Shell | Anomaly    | $\rightarrow$                        | K26   | shell.action $\rightarrow$ check(🛡️.A)                                                                   |

### V.6 P2P-Relationen

| From        | To        | Type             | Formalism                                |
| ----------- | --------- | ---------------- | ---------------------------------------- |
| P2P.Swarm   | $\iota$   | $\triangleright$ | peer $\equiv$ device_did                 |
| P2P.Gossip  | $\tau$    | $\triangleright$ | gossip_priority $= f(\tau)$              |
| P2P.Gossip  | Event     | $\rightarrow$    | recv(msg) $\rightarrow$ emit(event)      |
| P2P.DHT     | P2P.Swarm | $\uplus$         | DHT $\uplus$ Swarm                       |
| P2P.Relay   | $\tau$    | $\triangleright$ | relay $\triangleright \tau$(destination) |
| P2P.Privacy | $\iota$   | $\triangleright$ | privacy $\triangleright \iota$.mode      |

### V.7 Synergie-Matrix

| A       | B         | Score | Reason               | Formalism                           |
| ------- | --------- | ----- | -------------------- | ----------------------------------- |
| $\iota$ | $\tau$    | 10    | Fundamental coupling | $\iota \leftrightarrow \tau$        |
| $\tau$  | Consensus | 9     | K18 Voting weight    | vote $= f(\tau)$                    |
| $\rho$  | Gateway   | 9     | K23 Crossing         | cross $\implies$ G.eval             |
| Event   | $\Omega$  | 8     | Persistence          | $e \rightarrow \Omega$.persist($e$) |
| $\Psi$  | $\rho$    | 8     | Policy execution     | $\rho$.policy $\rightarrow \Psi$    |
| Shell   | $\iota$   | 8     | K8 Capabilities      | shell $\triangleright \iota$        |
| Shell   | $\tau$    | 8     | Trust thresholds     | shell $\triangleright \tau$         |
| P2P     | $\iota$   | 7     | PeerId mapping       | peer $\equiv$ did                   |
| 🛡️      | $\tau$    | 7     | Monitoring           | 🛡️ $\vdash \tau$                    |
| Formula | $\tau$    | 6     | K15 Input            | F $\triangleright \tau$             |

### V.8 Invarianten

**Layer-Isolation:**
$$\forall(a,b) \in L_i \times L_j: i < j \implies \neg(b \triangleright a)$$

**Event-Causality:**
$$\forall e_1, e_2: e_1 \rightarrow e_2 \implies t(e_1) < t(e_2)$$

**Observer-Independence:**
$$\forall \text{obs}_1, \text{obs}_2 \in \text{Observers}: \text{obs}_1.\text{effect} \cap \text{obs}_2.\text{effect} = \emptyset$$

---

## VI. Nervensystem-Integration

> **Quelle:** integration.json

### VI.1 Nervensystem-Definition

$$\boxed{\mathcal{N} = \langle \Sigma, \text{Hub}, \mathcal{E}_{\text{ng}}, \mathcal{O}_{\text{bs}}, \mathcal{A}_{\text{dapter}} \rangle}$$

**Biologische Metapher:**

| Komponente   | Metapher     | Funktion                    |
| ------------ | ------------ | --------------------------- |
| UnifiedState | Gehirn       | Zentrale Koordination       |
| SynapseHub   | Synapsen     | Signalübertragung           |
| Engines      | Muskeln      | Ausführung                  |
| Protection   | Immunsystem  | Schutz                      |
| Storage      | Gedächtnis   | Persistenz                  |
| P2P          | Nervenbahnen | Kommunikation               |
| Realm        | Organe       | Isolation & Spezialisierung |
| Domain       | DNA          | Typen & Invarianten         |

### VI.2 State-Relations

| Typ           | Symbol            | Formalism                                                            |
| ------------- | ----------------- | -------------------------------------------------------------------- |
| DependsOn     | $\rightarrow_a$   | $A \rightarrow_a B \iff$ init($A$) requires init($B$)                |
| Triggers      | $\rightarrow_t$   | $A \rightarrow_t B \iff$ update($A$) $\implies$ notify($B$)          |
| Aggregates    | $\supseteq$       | $A \supseteq B \iff$ state($A$) contains state($B$)                  |
| Validates     | $\vdash$          | $A \vdash B \iff$ invariants($A$) checked_by $B$                     |
| Bidirectional | $\leftrightarrow$ | $A \leftrightarrow B \iff A \rightarrow_t B \land B \rightarrow_t A$ |

### VI.3 StateEvents (42 Varianten)

$$\mathcal{SE} = \sum_{c \in \text{Components}} \text{Events}(c)$$

**Trust-Events:**

- TrustUpdate: $\Delta T(\iota, \vec{\tau}, \vec{\tau}', \text{reason})$
- TrustViolation: $\text{⚠}T(\iota, \text{violation\_type})$
- IdentityBootstrapped: $\exists\iota: \Sigma' = \Sigma \cup \{\iota\}$

**Event-Events:**

- EventAdded: $e \in \text{Log} \implies \text{Log}' = \text{Log} \cup \{e\}$
- EventFinalized: finalized($e$) $\land$ merkle_root'
- EventPersisted: storage($e$) = committed

**Protection-Events:**

- AnomalyDetected: $\alpha$: (type, severity, subject)
- EntropyUpdate: $H(\Sigma) \rightarrow H'$
- SystemModeChange: Mode: $M \rightarrow M'$ (reason)

**Realm-Events:**

- CrossingEvaluated: $\iota: \rho_1 \rightarrow? \rho_2 \implies$ {Allow, Deny}
- RealmRegistered: $\exists\rho$: Realms' = Realms $\cup \{\rho\}$

### VI.4 SynapseHub

$$\text{Hub} = \langle \mathcal{O}_{\text{reg}}, \text{Graph}, \text{Queue} \rangle$$

| Komponente                 | Typ                                 | Beschreibung     |
| -------------------------- | ----------------------------------- | ---------------- | ------------------------- | --------------- |
| $\mathcal{O}_{\text{reg}}$ | Map⟨StateComponent, List⟨Observer⟩⟩ | $                | \mathcal{O}\_{\text{reg}} | \geq 30$ traits |
| Graph                      | DAG⟨StateComponent, StateRelation⟩  | Dependency Graph |
| Queue                      | MPSC⟨WrappedStateEvent⟩             | async dispatch   |

**Dispatch-Algorithmus:**

1. direct*observers = $\mathcal{O}*{\text{reg}}$[event.component]
2. transitive*observers = $\bigcup*{c \in \text{triggered_by}(\text{event.component})} \mathcal{O}\_{\text{reg}}[c]$
3. $\forall$ obs $\in$ direct_observers $\cup$ transitive_observers: obs.on_event(event)

**Komplexität:** $O(|\text{observers}| + |\text{edges}|)$

### VI.5 Observer-Traits

| Trait              | Events                                            | Signature                                              |
| ------------------ | ------------------------------------------------- | ------------------------------------------------------ |
| TrustObserver      | TrustUpdate, IdentityBootstrapped, TrustViolation | fn on_trust_update(entity, old, new, reason)           |
| EventObserver      | EventAdded, EventFinalized                        | fn on_event_added(event), fn on_event_finalized(event) |
| ProtectionObserver | AnomalyDetected, EntropyUpdate                    | fn on_anomaly_detected(anomaly)                        |
| RealmObserver      | CrossingEvaluated, RealmRegistered                | fn on_crossing_succeeded(from, to)                     |
| StorageObserver    | EventPersisted, ArchiveCompleted                  | fn on_event_persisted(event_id, size, ts)              |
| P2PObserver        | PeerConnectionChange, NetworkMetricUpdate         | fn on_peer_connected(peer_id, addr)                    |

### VI.6 Cascade-Pattern

**Trust-Update-Cascade:**
$$\Delta\tau(\iota) \rightarrow_t \Delta\text{access}(\iota) \rightarrow_t \Delta\text{quota}(\iota) \rightarrow_t \Delta\text{budget}(\iota)$$

**Kette:**

1. StateEvent::TrustUpdate{ι, τ, τ', reason}
2. TrustState.apply_event() [Direct Owner]
3. RealmState.apply_event() [DependsOn] → recalc_access()
4. GatewayState.recalc_access() [Triggers] → neue Realms zugänglich
5. QuotaEnforcer.update_limits() [Triggers] → mehr Quota
6. ECLVMState.update_budget() [Triggers] → mehr Gas/Mana

### VI.7 State-Components (37 Komponenten)

| Layer      | Komponenten                                                           | Criticality |
| ---------- | --------------------------------------------------------------------- | ----------- |
| Core       | Identity, Trust, Event, Formula, Consensus                            | 🔴 Kritisch |
| Execution  | Gas, Mana, Execution                                                  | 🟡 Hoch     |
| Engine     | ECLVM, ECLPolicy, ECLBlueprint, UI, API, Governance, Controller       | 🟡-🟢       |
| Protection | Anomaly, Diversity, Quadratic, AntiCalcification, Calibration         | 🔴-🟢       |
| Peer       | Realm, Gateway, SagaComposer, IntentParser, Room                      | 🔴-🟢       |
| P2P        | Swarm, Gossip, DHT, Relay, Privacy, TrustGate                         | 🔴-🟢       |
| Storage    | Storage, EventStore, IdentityStore, TrustStore, ContentStore, Archive | 🔴-🟢       |

### VI.8 Module-Integrationen

**P2P-Integration:**

```
on_connection:
  trust_gate.check_connection(peer_id) → decision
  emit(PeerConnectionChange{peer_id, connected: true, level})
  emit(NetworkMetricUpdate{ConnectedPeers, +1})
```

**Protection-Integration:**

```
analyze_with_state:
  emit(AnomalyDetected{type, severity, subject})
  emit(SystemModeChange if severity = Critical)
```

**Realm-Integration:**

```
crossing_evaluation:
  1. trust = Σ.trust.get_trust(ι)
  2. policy = ECLVM.get_crossing_policy(ρ_target)
  3. result = ECLVM.evaluate_policy(policy, ctx) [gas-metered]
  4. emit(CrossingSucceeded|CrossingFailed)
```

### VI.9 Domain-Primitives

**UniversalId:**
$$\text{structure} = (\text{tag}: u8, \text{version}: u8, \text{hash}: [u8; 32])$$
$$\text{encoding} = \text{tag} \| \text{version} \| \text{blake3}(\text{content})$$

**TemporalCoord:**
$$\text{structure} = (\text{unix\_ts}: u128, \text{lamport}: u64, \text{node\_id}: \text{UniversalId})$$
$$\text{ordering} = \text{lexicographic}(\text{unix\_ts}, \text{lamport}, \text{node\_id})$$

**TrustVector6D:**
$$\text{structure} = (r, i, c, p, v, \omega) \in [0,1]^6$$

| Preset   | Values                         |
| -------- | ------------------------------ |
| NEWCOMER | (0.1, 0.1, 0.1, 0.1, 0.1, 0.1) |
| TRUSTED  | (0.7, 0.7, 0.7, 0.7, 0.7, 0.7) |
| VERIFIED | (0.9, 0.9, 0.9, 0.9, 0.9, 0.9) |

---

## VII. State-Kerngedanken

> **Quelle:** 08-STATE-KERNGEDANKEN.md

### VII.1 Design-Prinzipien $\mathcal{P}$

| $P_i$ | Name                      | Formale Definition                                                                             |
| ----- | ------------------------- | ---------------------------------------------------------------------------------------------- |
| $P_1$ | Hierarchische Komposition | $\forall L_i, L_j \in \mathcal{L}: i < j \Rightarrow L_i \prec L_j$                            |
| $P_2$ | Thread-Safety             | $\forall s \in \mathcal{S}: \text{atomic}(s) \lor \text{rwlock}(s)$                            |
| $P_3$ | Dependency Injection      | $\forall m \in \mathcal{M}: \text{deps}(m) \subseteq \text{inject}(\text{Hub})$                |
| $P_4$ | Event-Driven              | $\Delta s \Rightarrow \exists e \in \mathcal{E}: \text{emit}(e)$                               |
| $P_5$ | Snapshot-Isolation        | $\text{read}(s) \cap \text{lock}(s) = \emptyset$                                               |
| $P_6$ | Per-Realm Isolation       | $\forall r \in \mathcal{R}: \text{State}(r) \cap \text{State}(r') = \emptyset$ für $r \neq r'$ |
| $P_7$ | Event-Inversion           | P2P $\xleftrightarrow{\text{Queue}}$ Core                                                      |
| $P_8$ | Circuit Breaker           | anomaly($t$) $> \theta \Rightarrow$ degrade()                                                  |
| $P_9$ | CQRS Light                | $\Delta s \xrightarrow{\text{broadcast}}$ Subscribers                                          |

### VII.2 EventBus $\mathbb{B}$

$$\mathbb{B} = \langle I, E, P, \mu \rangle$$

- $I = \langle I_{tx}, I_{rx} \rangle$ — Ingress-Kanäle (P2P → Core)
- $E = \langle E_{tx}, E_{rx} \rangle$ — Egress-Kanäle (Core → P2P)
- $P$ — Priority-Queue für Consensus-kritische Events
- $\mu = \langle \mu_I, \mu_E, \mu_D \rangle$ — Metriken (Ingress, Egress, Dropped)

**Axiom (Event-Flow):**
$$\forall e \in \text{NetworkEvent}: e \in I \oplus e \in E$$

### VII.3 StateDelta $\Delta$

$$\Delta = \langle \text{seq}, \kappa, \tau, \text{data}, t, r? \rangle$$

| Symbol   | Typ             | Beschreibung                     |
| -------- | --------------- | -------------------------------- |
| seq      | $\mathbb{N}$    | Sequenznummer (monoton steigend) |
| $\kappa$ | StateComponent  | Betroffene Komponente            |
| $\tau$   | DeltaType       | Art der Änderung                 |
| data     | Vec⟨u8⟩         | Serialisierte Daten              |
| $t$      | $\mathbb{N}$    | Zeitstempel (ms)                 |
| $r?$     | Option⟨RealmId⟩ | Optionale Realm-Zuordnung        |

**Broadcaster-Invariante:**
$$\text{seq}(t+1) = \text{seq}(t) + 1 \land \text{seq}(0) = 0$$

### VII.4 CircuitBreaker $\mathbb{C}$

$$\mathbb{C} = \langle \sigma, W, \Theta \rangle$$

- $\sigma \in \{\text{Normal}, \text{Degraded}, \text{Emergency}\}$ — SystemMode
- $W \subseteq \mathbb{N}^{60}$ — Critical-Window (Anomalien/Minute)
- $\Theta = \langle \theta_D, \theta_E, \theta_G \rangle$ — Schwellwerte

**Transition-Regeln:**
$$\sigma \xrightarrow{|W| > \theta_D} \text{Degraded} \xrightarrow{|W| > \theta_E} \text{Emergency}$$

### VII.5 WrappedStateEvent

$$\mathcal{W} = \langle \text{id}, t, \pi, \kappa, \text{seq}, e, \sigma? \rangle$$

| Symbol | Invariante                                            |
| ------ | ----------------------------------------------------- |
| id     | $\text{id} = H_{\text{Blake3}}(e \| t \| \text{seq})$ |
| $\pi$  | Parent-IDs (Kausalität)                               |

**Kausalitäts-Invariante (K9):**
$$\forall w \in \mathcal{W}: \forall p \in \pi(w): \text{seq}(p) < \text{seq}(w)$$

### VII.6 StateEventLog

$$\mathcal{L} = \langle \text{seq}, B, c, \iota \rangle$$

- $B$: Ring-Buffer mit $|B| = 10.000$
- $c$: Letzter Checkpoint-ID
- $\iota = 5.000$: Checkpoint-Intervall

**Checkpoint-Regel:**
$$\text{seq} \mod \iota = 0 \Rightarrow \text{checkpoint}()$$

### VII.7 Merkle State Tracking

$$\mathcal{M} = \langle \rho, H_\kappa, \Delta_H \rangle$$

- $\rho$: Root-Hash (aktueller State)
- $H_\kappa$: StateComponent $\rightarrow$ MerkleHash
- $\Delta_H$: History von MerkleDeltas

**Verifikations-Axiom:**
$$\text{verify}(\pi, \rho_{\text{old}}, \text{data}) \Rightarrow \rho_{\text{new}} = \text{apply}(\rho_{\text{old}}, \text{data})$$

### VII.8 StateGraph $\mathcal{G}$

$$\mathcal{G} = \langle V, E, \lambda \rangle$$

- $V$: 40 StateComponents
- $E \subseteq V \times V$: 110+ Kanten
- $\lambda: E \rightarrow \mathcal{R}$: Relationstyp-Funktion

**Graph-Operationen:**
$$\text{deps}(v) = \{u \mid (v, u) \in E \land \lambda(v,u) = \rightarrow_D\}$$
$$\text{deps}^*(v) = \text{transitive\_closure}(\text{deps}(v))$$
$$\text{triggers}(v) = \{u \mid (v, u) \in E \land \lambda(v,u) = \rightarrow_T\}$$
$$\text{validators}(v) = \{u \mid (u, v) \in E \land \lambda(u,v) = \rightarrow_V\}$$
$$\text{crit}(v) = |\text{deps}^{-1}(v)| + |\text{triggers}(v)|$$

### VII.9 TrustState $\mathcal{S}_T$

$$\mathcal{S}_T = \langle N_e, N_r, \mu, T_{\text{avg}}, \mathcal{D}, \mathcal{T}_{\text{id}} \rangle$$

| Symbol                    | Beschreibung                           |
| ------------------------- | -------------------------------------- |
| $N_e$                     | Anzahl Entities                        |
| $N_r$                     | Anzahl Relationships                   |
| $\mu$                     | Updates (positiv, negativ, Violations) |
| $T_{\text{avg}}$          | Durchschnittliches Trust               |
| $\mathcal{D}$             | TrustDistribution                      |
| $\mathcal{T}_{\text{id}}$ | UniversalId $\rightarrow$ TrustEntry   |

**Asymmetrie-Invariante (K4):**
$$\frac{\mu_-}{\mu_+} \approx 2:1$$

### VII.10 Kritikalitäts-Funktion

$$\text{crit}(v) = |\{u \mid u \rightarrow_D v\}| + |\text{triggers}(v)|$$

| $v$      | $   | \text{deps}^{-1} | $      | $     | \text{trig} | $   | crit | Priorität |
| -------- | --- | ---------------- | ------ | ----- | ----------- | --- | ---- | --------- |
| Identity | 18  | 6                | **24** | $P_0$ |
| Trust    | 15  | 5                | **20** | $P_0$ |
| Event    | 10  | 6                | **16** | $P_0$ |
| ECLVM    | 8   | 4                | 12     | $P_1$ |
| Gateway  | 6   | 3                | 9      | $P_1$ |
| Realm    | 5   | 4                | 9      | $P_1$ |
| Gas      | 8   | 0                | 8      | $P_2$ |
| Swarm    | 4   | 3                | 7      | $P_2$ |

---

## VIII. Execution Engine (ECLVM/WASM)

> **Quelle:** 06-eclvm-wasm-migration.json

### VIII.1 Symbole

| Symbol                 | Bedeutung                   |
| ---------------------- | --------------------------- |
| $\Psi$                 | ECLVM (Execution Engine)    |
| $\Psi_{\text{legacy}}$ | Stack-VM Interpreter        |
| $\Psi_{\text{wasm}}$   | WebAssembly Runtime         |
| $\sigma$               | SystemState                 |
| $\pi$                  | Policy (ECL-Programm)       |
| $\vec{\tau}$           | TrustVector6D $\in [0,1]^6$ |
| $\gamma$               | Gas (computational cost)    |
| $\mu$                  | Mana (resource credits)     |
| $\Phi$                 | Host-Function Interface     |

### VIII.2 Algebraische Typen

**ExecutionMode:**
$$\text{Mode} ::= \text{Legacy} \mid \text{Wasm} \mid \text{Auto}$$
$$\text{Auto}(\pi) = \begin{cases} \text{Wasm} & \text{if } |\text{opcodes}(\pi)| > \theta \\ \text{Legacy} & \text{otherwise} \end{cases}$$

**WasmPolicyEngine:**
$$\text{Engine} = \langle E_{\text{wasm}}, \text{Cache}, \text{Linker}, \text{Config} \rangle$$

| Komponente        | Typ                                                             |
| ----------------- | --------------------------------------------------------------- |
| $E_{\text{wasm}}$ | Wasmtime Engine (shared)                                        |
| Cache             | Map⟨PolicyId, CompiledModule⟩                                   |
| Linker            | HostFunctions $\rightarrow$ WasmImports                         |
| Config            | ⟨fuel_limit: $\mathbb{N}$, mem_pages: $\mathbb{N}$, simd: Bool⟩ |

**GasLayer:**
$$\text{Layer} ::= \text{Network} \mid \text{Compute} \mid \text{Storage} \mid \text{Realm}$$
$$\text{MultiGas} = \text{Map}\langle\text{Layer}, (\text{used}: \mathbb{N}, \text{limit}: \mathbb{N})\rangle$$

### VIII.3 Operationen

**Compile:**
$$\text{compile}: \text{ECL\_Source} \rightarrow \text{Result}\langle\text{CompiledWasmPolicy}, \text{Error}\rangle$$
$$\text{Source} \xrightarrow{\text{parse}} \text{AST} \xrightarrow{\text{optimize}} \text{AST}' \xrightarrow{\text{codegen}} \text{WASM\_Bytes} \xrightarrow{\text{wasmtime}} \text{Module}$$

**Execute:**
$$\text{execute}: (\text{Policy} \times \text{Context}) \xrightarrow{\text{async}} \text{Result}\langle\text{Value}, \text{Error}\rangle$$
$$\pi(\sigma) = \Psi_{\text{mode}}(\pi, \sigma) \text{ where mode} \in \{\text{Legacy}, \text{Wasm}, \text{Auto}\}$$

### VIII.4 Host-Functions

| Kategorie | Funktion       | Signatur                                                                       |
| --------- | -------------- | ------------------------------------------------------------------------------ |
| Trust     | get_trust      | DID $\rightarrow$ Result⟨$\vec{\tau}$, Error⟩                                  |
| Trust     | trust_norm     | $\vec{\tau} \rightarrow \mathbb{R}$, $\|\vec{\tau}\| = \sqrt{\sum_i \tau_i^2}$ |
| Identity  | has_credential | (DID × Schema) $\rightarrow$ Result⟨Bool, Error⟩                               |
| Identity  | resolve_did    | DID $\rightarrow$ Result⟨Bool, Error⟩                                          |
| State     | store_get      | (Store × Key) $\rightarrow$ Result⟨Option⟨Value⟩, Error⟩                       |
| State     | store_put      | (Store × Key × Value) $\rightarrow$ Result⟨(), Error⟩                          |
| Budget    | consume_gas    | (Layer × Amount) $\rightarrow$ Result⟨(), Error⟩                               |
| Budget    | get_budget     | () $\rightarrow$ (gas_used, gas_limit, mana_used, mana_limit)                  |
| Events    | emit_event     | (EventType × Payload) $\rightarrow$ Result⟨(), Error⟩                          |

### VIII.5 OpCode-Mappings

| ECL OpCode      | WASM Equivalent                         |
| --------------- | --------------------------------------- |
| Push(f64)       | f64.const                               |
| Add/Sub/Mul/Div | f64.add/sub/mul/div                     |
| Eq/Lt           | f64.eq/lt                               |
| And/Or/Not      | i32.and/or/eqz (nach bool-conversion)   |
| LoadTrust(dim)  | call $erynoa.get_trust                  |
| HasCredential   | call $erynoa.has_credential             |
| StoreGet/Put    | call $erynoa.store_get/put + Mana       |
| Require         | br_if + unreachable (conditional abort) |

**Invariante:**
$$\forall \text{op} \in \text{ECL\_OpCodes}: \exists \text{wasm\_equiv}(\text{op}) \in \text{WASM\_Instrs}$$

### VIII.6 Performance-Modell

| Metrik                    | Legacy | WASM  | Faktor |
| ------------------------- | ------ | ----- | ------ |
| Latenz (avg)              | 2ms    | 0.2ms | 10×    |
| Throughput (trust_ops/ms) | 50     | 500   | 10×    |
| Throughput (policies/s)   | 500    | 5000  | 10×    |
| Startup (cold)            | 1ms    | 1ms   | 1×     |
| Startup (hot)             | 1ms    | 0.1ms | 10×    |
| Memory per Policy         | 1MB    | 2MB   | 0.5×   |

$$T_{\text{wasm}} \approx \frac{T_{\text{legacy}}}{10}$$

### VIII.7 Constraints (WASM)

| ID          | Name              | Formalism                                                                                      |
| ----------- | ----------------- | ---------------------------------------------------------------------------------------------- |
| K_WASM_Det  | WASM Determinism  | $\forall\pi, \sigma: \Psi_{\text{wasm}}(\pi, \sigma) = \Psi_{\text{wasm}}(\pi, \sigma)$        |
| K_WASM_Iso  | Sandbox Isolation | $\forall\pi: \text{effects}(\pi) \subseteq \Phi(\text{Bridge})$                                |
| K_WASM_Fuel | Fuel Boundedness  | fuel_consumed($\pi$) $\leq$ fuel_limit $\implies$ terminates($\pi$)                            |
| K_Mode_Eq   | Mode Equivalence  | $\forall\pi, \sigma: \Psi_{\text{legacy}}(\pi, \sigma) \equiv \Psi_{\text{wasm}}(\pi, \sigma)$ |
| K_Host_Con  | Host Consistency  | commit(Bridge) $\implies$ consistent(UnifiedState)                                             |

### VIII.8 Synergien (WASM)

| Synergie      | Formel                                                                                       | Beschreibung                        |
| ------------- | -------------------------------------------------------------------------------------------- | ----------------------------------- |
| WASM→Trust    | $\tau_{\text{compute\_time}}(\text{wasm}) = \tau_{\text{compute\_time}}(\text{legacy}) / 10$ | Realm-Crossings 10× schneller       |
| WASM→Gateway  | Gateway*latency = min($T*{\text{wasm}}$, $T_{\text{threshold}}$)                             | ProgrammableGateway 0.2ms statt 2ms |
| WASM→Realm    | enforce(quota) via Fuel $\implies$ Realm_isolation                                           | WASM-Fuel mapped auf RealmQuota     |
| Cache→Startup | $T_{\text{exec}} = $ if cached($\pi$) then $T_{\text{hot}}$ else $T_{\text{cold}}$           | Pre-compiled Cache                  |

---

## IX. Sharding-Architektur

> **Quelle:** 14-SHARDING-ARCHITEKTUR.pluto.md

### IX.1 Formaldefinition

$$\boxed{\mathcal{L} = \langle \mathcal{S}, h, \mathcal{C}, \mathcal{E}, \mathcal{M} \rangle}$$

| Symbol        | Definition        | Domäne               |
| ------------- | ----------------- | -------------------- | ----------- | ------------------------ |
| $\mathcal{S}$ | Shard-Menge       | $                    | \mathcal{S} | \in \{4, 64, 128, 256\}$ |
| $h$           | FxHash-Funktion   | $\mathbb{Z}_n$       |
| $\mathcal{C}$ | Cache (DashMap)   | lock-free Map        |
| $\mathcal{E}$ | LRU-Eviction      | time-based           |
| $\rho$        | Shard-Reputation  | $[0, 1]$             |
| $\eta$        | Shard-Entropy     | $[0, 1]$             |
| $\gamma$      | Gas-Multiplikator | $[1, \gamma_{\max}]$ |

### IX.2 Shard-Selektion

$$h(r) := \text{FxHash}(r) \mod n$$

**Eigenschaften:**

- $h: \text{RealmID} \rightarrow \mathbb{Z}_n$ (deterministisch)
- $\mathbb{E}[|S_i|] = \frac{|\mathcal{R}|}{n}$ (gleichverteilt)
- $O(1)$ Berechnung

$$\forall r \in \mathcal{R}: \text{shard}(r) = S_{h(r)}$$

### IX.3 Cache-Operationen

**Lookup (synchron):**
$$\text{get\_cached}(r) = \begin{cases} \mathcal{C}(S_{h(r)})[r] & \text{if } r \in \text{dom}(\mathcal{C}(S_{h(r)})) \\ \bot & \text{otherwise} \end{cases}$$

**Lazy Loading (asynchron):**
$$\text{get\_or\_load}(r) = \begin{cases} \mathcal{C}(S_{h(r)})[r] & \text{cache-hit} \\ \text{load}(r) \circ \text{replay}(r) \circ \text{insert}(r) & \text{cache-miss} \end{cases}$$

**Pipeline:**
$$\text{Storage} \xrightarrow{\text{load}} \text{Snapshot} \xrightarrow{\text{replay}} \text{State} \xrightarrow{\text{insert}} \mathcal{C}$$

**LRU-Eviction:**
$$\text{evict}(S_i) = \{r \in \mathcal{C}(S_i) : \text{access\_time}(r) < t_{\text{threshold}}\}$$
$$|\mathcal{C}(S_i)| \leq \kappa_{\max} \quad \forall S_i \in \mathcal{S}$$

### IX.4 ShardMonitor — Sicherheitsmodell

**Entropy-Metrik:**
$$\eta(S_i) := -\sum_{s \in \text{sources}(S_i)} p_s \cdot \log_2(p_s)$$
$$\hat{\eta}(S_i) = \frac{\eta(S_i)}{\log_2(|\text{sources}(S_i)|)} \in [0, 1]$$

**Bias-Detektion:**
$$\text{bias}(S_i) \iff \hat{\eta}(S_i) < \theta_{\text{bias}}$$

**Reputation-Funktion:**
$$\rho(S_i) := \frac{\text{success}(S_i)}{\text{success}(S_i) + \text{fail}(S_i)}$$

**EWMA-Update:**
$$\rho_{t+1} = \alpha \cdot \rho_t + (1 - \alpha) \cdot \rho_{\text{new}}$$

**Quarantäne-Prädikat:**
$$Q(S_i) \iff \text{fail}(S_i) > \phi_Q \lor \rho(S_i) < \rho_{\min}$$

### IX.5 Cross-Shard-Interaktion

**Gas-Penalty:**
$$\boxed{\gamma(S_i) = 1 + (1 - \rho(S_i)) \cdot \gamma_{\max}}$$

| $\rho$ | $\gamma$ (bei $\gamma_{\max}=5$) |
| ------ | -------------------------------- |
| 1.0    | 1.0×                             |
| 0.5    | 3.0×                             |
| 0.0    | 5.0×                             |

**Trust-Dämpfung:**
$$\Delta T_{\text{eff}} = \Delta T \cdot \rho(S_{\text{source}})$$
$$Q(S_{\text{source}}) \implies \Delta T_{\text{eff}} = 0$$

### IX.6 Konfigurationsprofile

| Profil     | $n$           | $\kappa_{\max}$ | $\tau_{\text{evict}}$ | Use Case |
| ---------- | ------------- | --------------- | --------------------- | -------- |
| minimal    | 4             | 100             | 60s                   | Tests    |
| default    | 64            | 20.000          | 600s                  | Dev      |
| production | 128           | 50.000          | 300s                  | Prod     |
| auto       | $4 \cdot$ CPU | 30.000          | 600s                  | Auto     |

### IX.7 Theoreme

**Lookup-Komplexität:**
$$\text{get\_cached}(r) \in O(1)$$

**Load-Balancing:**
$$\text{stddev}\left(\frac{|S_i|}{|\mathcal{R}|/n}\right) \xrightarrow{|\mathcal{R}| \to \infty} 0$$

**Quarantäne-Sicherheit:**
$$Q(S_i) \implies \forall r \in \mathcal{C}(S_i): \text{cross-shard}(r) = \bot$$

**Reputation-Konvergenz:**
$$\lim_{t \to \infty} \rho(S_i) = \frac{\lambda_{\text{success}}}{\lambda_{\text{success}} + \lambda_{\text{fail}}}$$

### IX.8 Relationen

$$\text{Sharding} \xrightarrow{\text{DependsOn}} \text{Realm}$$
$$\text{Sharding} \xrightarrow{\text{DependsOn}} \text{Storage}$$
$$\text{Sharding} \xrightarrow{\text{Aggregates}} \text{Trust}$$
$$\text{Sharding} \xrightarrow{\text{Aggregates}} \text{Gas}$$
$$\text{Sharding} \xrightarrow{\text{Triggers}} \text{Event}$$
$$\text{Sharding} \xrightarrow{\text{Validates}} \text{Protection}$$
$$\text{Sharding} \xleftrightarrow{\text{Bidir}} \text{P2P}$$

---

## X. Realm-Governance

> **Quelle:** 16-REALM-GOVERNANCE.pluto.md

### X.1 Axiom: Realm-Exklusivität

$$\boxed{\mathcal{G} \iff \exists\, \mathcal{R} : \mathcal{G} \subseteq \mathcal{R}}$$

**Negationen:**

- $\neg\mathcal{G}(\text{Identity})$ — keine Identitäts-Governance
- $\neg\mathcal{G}(\text{Package})$ — keine Package-Governance
- $\neg\mathcal{G}(\text{Global})$ — keine globale direktdemokratische Governance

### X.2 Stimmgewicht-Hauptformel

$$\boxed{W(m) = G(m) \cdot \left(1 + \alpha \cdot T_{\text{rel}}(m)\right)}$$

| Komponente          | Bedeutung                                     |
| ------------------- | --------------------------------------------- |
| $W(m)$              | Finales Stimmgewicht des Members $m$          |
| $G(m)$              | Governance-Basis-Gewicht (aus GovernanceType) |
| $\alpha \in [0, 1]$ | Trust-Einfluss-Faktor (Realm-konfiguriert)    |
| $T_{\text{rel}}(m)$ | Relativer Trust im Realm                      |

### X.3 Relativer Trust

$$\boxed{T_{\text{rel}}(m) = \frac{T(m) - T_{\text{avg}}}{T_{\text{avg}}}}$$

| Bedingung               | $T_{\text{rel}}$ | Effekt  |
| ----------------------- | ---------------- | ------- |
| $T(m) > T_{\text{avg}}$ | $> 0$            | Bonus   |
| $T(m) = T_{\text{avg}}$ | $= 0$            | Neutral |
| $T(m) < T_{\text{avg}}$ | $< 0$            | Malus   |

**Aggregierter Trust:**
$$T(m) = \frac{\sum_{d \in \mathcal{D}} w_d \cdot T_d(m)}{\sum_{d \in \mathcal{D}} w_d}$$

**Standard-Gewichtung:**
$$\mathbf{w} = (1.0, 1.0, 1.0, 1.0, 1.0, 2.0)^T$$
$\Omega$ (Axiom-Treue) ist doppelt gewichtet.

### X.4 Governance-Typen

$$G(m) = \begin{cases} \sqrt{\tau(m)} & \text{Quadratic} \\ \tau(m) & \text{Token} \\ T(m) & \text{Reputation} \\ 1 & \text{MemberEqual} \\ G_{\text{base}}(m) + \sum_{d \in D(m)} G(d) \cdot \delta^{\text{depth}(d)} & \text{Delegated} \end{cases}$$

| Typ         | $G(m)$        | $\alpha$ | Anwendung         |
| ----------- | ------------- | -------- | ----------------- |
| Quadratic   | $\sqrt{\tau}$ | Optional | DAOs              |
| Token       | $\tau$        | Optional | Investment-DAOs   |
| Reputation  | $T$           | 1.0      | Merit-Guilds      |
| Delegated   | rekursiv      | Via Base | Große Communities |
| MemberEqual | 1             | Optional | Cooperatives      |

### X.5 Liquid Democracy (K8)

**Delegation-Relation:**
$$\mathcal{D} \subseteq \mathcal{M} \times \mathcal{M}$$
$$(m_1, m_2) \in \mathcal{D} \iff m_1 \text{ delegiert an } m_2$$

**Delegation mit Trust-Decay:**
$$\boxed{W_{\text{del}}(m) = G(m) + \sum_{d \in D(m)} G(d) \cdot t_d^{\text{depth}(d)}}$$

**Invarianten:**
$$\text{depth}(d) \leq \text{depth}_{\max}$$
$$t_d^{\text{depth}(d)} \geq \delta_{\min}$$

### X.6 Proposal-Lifecycle

**Zustandsautomat:**
$$\mathcal{S}_\mathcal{P} = \{\text{Draft}, \text{Discussion}, \text{Voting}, \text{Timelock}, \text{Executed}, \text{Defeated}, \text{Vetoed}\}$$

**Transitionen:**
$$\text{Draft} \xrightarrow{\text{submit}} \text{Discussion}$$
$$\text{Discussion} \xrightarrow{t \geq t_{\text{disc}}} \text{Voting}$$
$$\text{Voting} \xrightarrow{v \geq q \land a \geq \theta} \text{Timelock}$$
$$\text{Voting} \xrightarrow{v < q \lor a < \theta} \text{Defeated}$$
$$\text{Timelock} \xrightarrow{t \geq t_{\text{lock}} \land \neg\text{veto}} \text{Executed}$$
$$\text{Timelock} \xrightarrow{\text{veto} \geq \theta_v} \text{Vetoed}$$

### X.7 Quorum & Approval

$$\boxed{\text{accepted} \iff \left(\frac{\sum W_{\text{voted}}}{\sum W_{\text{total}}} \geq q\right) \land \left(\frac{\sum W_{\text{for}}}{\sum W_{\text{voted}}} \geq \theta\right)}$$

**Dynamisches Quorum:**
$$q_{\text{dyn}} = \min\left(q_{\text{base}} + \beta \cdot \text{participation}_{\text{history}}, q_{\max}\right)$$

### X.8 Proposal-Kategorien

| Kategorie        | $\theta$ | Timelock | Supermajority |
| ---------------- | -------- | -------- | ------------- |
| ParameterChange  | 0.50     | 24h      | ✗             |
| TreasurySpend    | 0.60     | 48h      | ✗             |
| RuleChange       | 0.67     | 72h      | ✓             |
| MemberAction     | 0.75     | 24h      | ✓             |
| GovernanceChange | 0.80     | 7d       | ✓             |

### X.9 Veto-Mechanismus

$$\boxed{\text{vetoed} \iff \frac{\sum W_{\text{veto}}}{\sum W_{\text{total}}} \geq \theta_v}$$

**Typisch:** $\theta_v = 0.33$

### X.10 Anti-Sybil durch Trust

**Newcomer-Dämpfung:**
$$\text{newcomer: } T(m) = 0.1 \implies T_{\text{rel}} = \frac{0.1 - 0.6}{0.6} = -0.83$$
$$\text{Mit } \alpha = 0.5: W(m) = G(m) \cdot (1 + 0.5 \cdot (-0.83)) = 0.58 \cdot G(m)$$

### X.11 Trust-Bidirektionale Kopplung

**Governance → Trust:**

| Event               | $\Delta T$ |
| ------------------- | ---------- |
| ProposalAccepted    | +0.02      |
| ProposalRejected    | -0.01      |
| ProposalSpam        | -0.10      |
| VotingParticipation | +0.005     |
| DelegationReceived  | +0.01      |
| SuccessfulVeto      | +0.02      |

**Trust → Governance:**
$$W(m) = G(m) \cdot (1 + \alpha \cdot T_{\text{rel}}(m))$$

---

## XI. URL-Resource-Addressing

> **Quelle:** 17-REALM-URL-RESOURCE-ADDRESSING.pluto.md

### XI.1 URL-Schema (K26)

$$\boxed{\text{URL} = \texttt{erynoa://}\langle\text{authority}\rangle/\langle\text{type}\rangle/\langle\text{path}\rangle\,[?\langle\text{params}\rangle]\,[\#\langle\text{fragment}\rangle]}$$

**Komponenten-Algebra:**
$$\text{URL} := \langle \mathcal{A}, \tau, \pi, \phi, \psi \rangle$$

| Symbol        | Domäne                                                            |
| ------------- | ----------------------------------------------------------------- |
| $\mathcal{A}$ | DID $\cup$ Alias                                                  |
| $\tau$        | {store, profile, contract, asset, event, meta, governance, trust} |
| $\pi$         | [String] — Path                                                   |
| $\phi$        | Map⟨String, String⟩ — Query-Params                                |

### XI.2 Authority-Resolution

$$\boxed{\text{resolve}(\mathcal{A}) = \begin{cases} \mathcal{A} & \text{if } \mathcal{A} \in \text{DID} \\ \text{Registry}(\mathcal{A}) & \text{if } \mathcal{A} \in \text{Alias} \end{cases}}$$

**Alias-Registrierung:**
$$\text{register}(\text{alias}): \text{Alias} \times \text{DID} \rightarrow \text{Registry}$$
**Kosten:** Mana = 10000 (Anti-Squatting)

### XI.3 Resource-Schema (K27)

$$\boxed{\mathcal{S} = \langle \text{version}, \mathcal{T}, \text{fallback}, \text{inheritance} \rangle}$$

**Standard-Types:**

| Type       | Pattern                    | Resolver   | Access            |
| ---------- | -------------------------- | ---------- | ----------------- |
| store      | `store/<name>/<key>`       | Storage    | realm-policy      |
| profile    | `profile/<did>`            | Identity   | owner-or-public   |
| contract   | `contract/<name>/<method>` | ECLVM      | contract-policy   |
| asset      | `asset/<category>/<id>`    | Storage    | policy-controlled |
| event      | `event/<type>/<ts>`        | EventLog   | members-only      |
| meta       | `meta/<key>`               | Metadata   | public            |
| governance | `governance/<proposal-id>` | Governance | members-only      |
| trust      | `trust/<did>`              | TrustCore  | members-only      |

**Schema-Vererbung (K1):**
$$\boxed{\mathcal{S}_{\text{child}} \supseteq \mathcal{S}_{\text{parent}}}$$

### XI.4 Resolution-Engine

$$\boxed{\text{resolve}: \text{URL} \times \text{DID} \rightarrow \text{Resource} \cup \{\bot\}}$$

**Pipeline:**
$$\text{URL} \xrightarrow{\text{parse}} \langle \mathcal{A}, \tau, \pi \rangle \xrightarrow{\text{schema}} \text{TypeDef} \xrightarrow{\text{access}} \text{Policy} \xrightarrow{\mathcal{R}} \text{Resource}$$

**Resolver-Dispatch:**
$$\mathcal{R}(\tau) = \begin{cases} \text{StorageResolver} & \tau \in \{\text{store}, \text{asset}\} \\ \text{IdentityResolver} & \tau = \text{profile} \\ \text{ECLVMResolver} & \tau = \text{contract} \\ \text{EventLogResolver} & \tau = \text{event} \\ \text{MetadataResolver} & \tau = \text{meta} \end{cases}$$

**Storage-Mapping:**
$$\texttt{erynoa://R/store/inventory/items} \mapsto \texttt{realm:\{R\}:shared:store:inventory:items}$$

### XI.5 Open-Access-Policy (K28)

$$\boxed{\text{access}(\text{url}, \text{req}) = \begin{cases} \text{Allow}(\mathcal{F}) & \text{if policy} \vdash \text{requester} \\ \text{Deny} & \text{otherwise} \end{cases}}$$

**Member vs. Non-Member:**
$$\text{eval}(r, \text{req}) = \begin{cases} \text{member-access}(\tau, \pi) & \text{if req} \in \mathcal{M}(\mathcal{R}) \\ \text{open-access}(\tau, \text{req}) & \text{if } \tau \in \mathcal{T}_{\text{public}} \land \text{policy}(\tau) \\ \text{Deny} & \text{otherwise} \end{cases}$$

**Trust-Requirements für Non-Members:**
$$\text{non-member-access} \iff T_\Omega(\text{req}) \geq T_{\Omega,\min} \lor \text{req} \in \bigcup_{R \in \mathcal{R}_{\text{trusted}}} \mathcal{M}(R)$$

### XI.6 Access-Evaluation-Matrix

$$\boxed{\text{access}(\text{req}, \tau, \pi) = \bigvee_{p \in \mathcal{P}} \text{eval}_p(\text{req}, \tau, \pi)}$$

**Prioritätsordnung:**
$$\text{Member} \succ \text{Open-Policy} \succ \text{Crossing-Eval} \succ \text{Deny}$$

| Requester   | Type    | Policy  | Result            |
| ----------- | ------- | ------- | ----------------- |
| Member      | private | any     | ✓ Allow           |
| Member      | public  | any     | ✓ Allow           |
| Non-Member  | public  | K28     | ✓ Allow(filtered) |
| Non-Member  | private | any     | ✗ Deny            |
| Cross-Realm | any     | K23+K28 | ⚖ Crossing-Eval   |

### XI.7 Cross-Realm Resolution (K23)

**Crossing-Dampening:**
$$\boxed{T_{\text{cross}} = T_{\text{local}} \cdot (1 - \kappa_{23})}$$
**Typisch:** $\kappa_{23} = 0.3$

### XI.8 Query-Parameter

| Param        | Typ     | Beschreibung           |
| ------------ | ------- | ---------------------- |
| view         | enum    | {public, full, raw}    |
| fields       | list    | Feld-Selektion         |
| version      | semver  | Spezifische Version    |
| at           | ISO8601 | Historischer Zeitpunkt |
| limit/offset | int     | Pagination             |
| sort         | expr    | field:asc\|desc        |
| filter       | expr    | field:value            |

### XI.9 URL-Operationen

$$\mathcal{O} = \{\text{Read}, \text{Write}, \text{Subscribe}, \text{Execute}\}$$
$$\boxed{\text{sig-required}(o) \iff o \in \{\text{Write}, \text{Execute}\}}$$

---

## XII. Migrations-Algebra

> **Quelle:** migrations.json

### XII.1 Domäne

$$\text{FileSystem: FS} = (\text{Paths}: \mathcal{P}(\text{String}), \text{Contents}: \text{Map}\langle\text{Path}, \text{Bytes}\rangle)$$
$$\text{GitState: G} = (\text{Commits}: \text{List}\langle\text{Hash}\rangle, \text{Branches}: \text{Map}\langle\text{Name}, \text{Hash}\rangle, \text{WorkTree}: \text{FS})$$
$$\text{CompilationState: C} \in \{\text{Success}, \text{Failure}(\text{Errors})\}$$
$$\text{TestState: T} = (\text{Passed}: \mathbb{N}, \text{Failed}: \mathbb{N}, \text{Skipped}: \mathbb{N})$$
$$\text{State-Tuple: M} = (\text{FS}, G, C, T)$$

### XII.2 Operationen

| Symbol                    | Signatur                                          | Beschreibung                                   |
| ------------------------- | ------------------------------------------------- | ---------------------------------------------- |
| $\Phi_{\text{setup}}$     | FS $\rightarrow$ FS'                              | Verzeichnisstruktur erstellen (idempotent)     |
| $\Phi_{\text{extract}}$   | (FS, Source, Target, LineRange) $\rightarrow$ FS' | Code-Segmente mit Header-Injektion extrahieren |
| $\Phi_{\text{backup}}$    | (FS, G) $\rightarrow$ (FS', BackupRef)            | Timestamped Backup erstellen                   |
| $\Phi_{\text{check}}$     | FS $\rightarrow$ (C, T)                           | Kompilierung & Testintegrität validieren       |
| $\Phi_{\text{imports}}$   | FS $\rightarrow$ FS'                              | Globale Import-Pfad-Transformation             |
| $\Phi_{\text{deprecate}}$ | (OldPath, NewPath) $\rightarrow$ ReExportDecl     | deprecated Re-Exports generieren               |
| $\Phi_{\text{rollback}}$  | (FS, BackupRef) $\rightarrow$ FS'                 | Vorherigen Zustand wiederherstellen            |

**Import-Transformationen:**

| Pattern                | Replacement               |
| ---------------------- | ------------------------- |
| `crate::core::state::` | `crate::nervous_system::` |
| `crate::peer::p2p::`   | `crate::p2p::`            |
| `crate::local::`       | `crate::storage::`        |

### XII.3 Execution-Graph

$$\text{Pipeline} = \Phi_{\text{backup}} ; \Phi_{\text{setup}} ; (\Phi_{\text{extract}} ; \Phi_{\text{check}})^* ; \Phi_{\text{imports}} ; \Phi_{\text{check}} ; \Phi_{\text{deprecate}}$$

**Kanten:**

```
START → Φ_backup → Φ_setup → Φ_extract → Φ_check_1
Φ_check_1 [C=Success] → Φ_imports
Φ_check_1 [C=Failure] → Φ_rollback
Φ_imports → Φ_check_2
Φ_check_2 [C=Success] → Φ_deprecate → END
Φ_check_2 [C=Failure] → Φ_rollback
```

### XII.4 Migrations-Constraints

| ID  | Name                    | Formalism                                                                                                                                 |
| --- | ----------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| M1  | BackupBeforeDestruction | $\forall \Phi_{\text{destructive}}: \exists \Phi_{\text{backup}} < \Phi_{\text{destructive}}$                                             |
| M2  | CheckAfterMutation      | $\forall \Phi_{\text{mutate}} \in \{\Phi_{\text{extract}}, \Phi_{\text{imports}}\}: \Phi_{\text{mutate}} \rightarrow \Phi_{\text{check}}$ |
| M3  | RollbackSafety          | $\Phi_{\text{rollback}}(\Phi_{\text{rollback}}(s, \text{ref}), \text{ref}) = \Phi_{\text{rollback}}(s, \text{ref})$                       |
| M4  | AtomicPhase             | Phase_complete $\iff$ ($C$ = Success $\land$ $T$.Failed = 0)                                                                              |
| M5  | GitTrace                | $\forall$ Phase$_i$: $\exists$ commit$_i \in G$.Commits                                                                                   |

### XII.5 Git-Workflow

$$\text{Branch} = \texttt{'refactor/projekt-pluto'}$$
$$\text{Commit-Pattern} = \texttt{"Phase \{i\}: \{description\}"}$$
$$\text{Merge-Condition} = \forall \Phi_{\text{check}}: C = \text{Success}$$

### XII.6 Synergien

| Synergie        | Formel                                                          | Beschreibung                                         |
| --------------- | --------------------------------------------------------------- | ---------------------------------------------------- |
| Migration→Git   | Phase_complete $\implies$ Commit                                | Jede erfolgreich abgeschlossene Phase wird committet |
| Backup→Rollback | $\Phi_{\text{rollback}} \circ \Phi_{\text{backup}} = \text{id}$ | Backup + Rollback sind inverse Operationen           |
| Check→Pipeline  | $C$ = Failure $\implies \neg$continue(Pipeline)                 | Fehler stoppen die Pipeline sofort                   |

---

## XIII. Operations & Synergien

> **Quelle:** formulas.json

### XIII.1 Storage-Operationen

| Operation | Kosten-Formel                                   | Beschreibung                             |
| --------- | ----------------------------------------------- | ---------------------------------------- |
| upload    | Cost = (SizeMB × 1.0 Mana) + (Chunks × 0.1 Gas) | Mana für Bandbreite, Gas für Indexierung |
| download  | Cost = SizeMB × 0.1 Mana                        | Mana für Bandbreite                      |
| pin       | Cost = SizeMB × 0.01 Mana / Day                 | Laufende Kosten für Pinned Content       |

**Trust-Requirement:** min_upload_trust (default: 0.3)

### XIII.2 Protection-Operationen

**CircuitBreaker:**

- Trigger: Anomalies/Minute > CriticalThreshold
- Effect: SystemMode → Degraded
- Recovery: Anomalies/Minute < SafeThreshold for 5 Minutes

**Anti-Calcification:**

- Trigger: Gini(TrustDistribution) > 0.8
- Effect: TrustDecayFactor \*= 1.1 (globally)

### XIII.3 Package-Resolution

$$\text{Maximize}(\text{Trust}) \land \text{Minimize}(\text{Conflicts}) \land \text{VersionMatch}(\text{SemVer})$$

**Heuristics:** PreferHighestTrust, PreferPinned

### XIII.4 Quadratic Voting

$$\text{Votes} = \lfloor\sqrt{\text{Tokens}}\rfloor$$
$$\text{Weight} = \text{Votes} \times (1 + \alpha \cdot \text{relative\_trust})$$

### XIII.5 P2P-Propagation

$$\text{TargetPeers} = \log_2(\text{NetworkSize}) \times K$$
$$\text{TTL} = 5 \text{ hops (default)}$$

### XIII.6 Synergieformeln

| Synergie              | Name                 | Formel                                             |
| --------------------- | -------------------- | -------------------------------------------------- |
| Trust→Storage         | Trusted Seeding      | Seed_Bonus = min(0.01, HostedGB × 0.0001)          |
| Trust→Identity        | Verified Badges      | Badge = Trust > 0.9 + ID_Age > 30d                 |
| Mana→Storage          | Storage Rent         | Quota = Trust × BaseQuota × (1 + ManaBalance/1000) |
| Protection→Governance | Emergency Governance | Mode=Degraded ⟹ ProposalTime / 2                   |

---

## XIV. Axiom-Katalog (Constraints)

> **Quelle:** constraints.json + alle Dokumente

### XIV.0 🔑 Fundamentales Axiom (Passkey-Primacy)

| ID     | Name                | Formalism                                                                                                                              |
| ------ | ------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| **Κ0** | **Passkey-Primacy** | $\forall \iota \in \mathcal{I}_{\text{Human}}: \exists! \, pk \in \text{Passkey}_{\text{HW}} : \text{auth}(\iota) = \text{verify}(pk)$ |

> **Bedeutung:** Passkey/WebAuthn ist die **einzige** Hardware-gebundene Authentifizierungsmethode. Alle Identitäten, DIDs und Ableitungen basieren auf dieser Wurzel. Κ0 ist das fundamentale Axiom, von dem alle anderen abhängen.

**Implikationen:**

- $K_{priv} \in \text{TPM} \cup \text{SecureEnclave}$ (Hardware-Binding)
- $\nexists \, f: K_{priv} \rightarrow \text{Plaintext}$ (Non-Exportability)
- $M_0 = \text{Interactive} \Leftrightarrow pk \neq \bot$ (Mode-Voraussetzung)
- Alle anderen Modi ($M_1, M_2, M_3$) delegieren von oder leiten ab von $M_0$

### XIV.1 Core-Axiome

| ID      | Name                | Formalism                                                                             |
| ------- | ------------------- | ------------------------------------------------------------------------------------- |
| **K1**  | MonotoneInheritance | $\rho' \triangleright \rho \implies \text{rules}(\rho) \subseteq \text{rules}(\rho')$ |
| **K9**  | EventCausality      | $\forall e \in \text{Log}: \text{timestamp}(e) < \text{timestamp}(\text{parent}(e))$  |
| **K10** | ContentAddressing   | ID(blob) = Hash(content(blob))                                                        |

### XIV.2 Trust-Axiome

| ID     | Name            | Formalism                                                                                           |
| ------ | --------------- | --------------------------------------------------------------------------------------------------- | ------ | --------- |
| **K2** | BoundedTrust    | $\tau \in [0,1]^6$                                                                                  |
| **K3** | DeltaBounded    | $\forall\delta:                                                                                     | \delta | \leq 0.1$ |
| **K4** | AsymmetricDecay | $\Delta^- = \lambda \times \Delta^+$, $\lambda \in \{1.5, 2.0\}$                                    |
| **K8** | DelegationDecay | $\tau_{\text{eff}}(\text{depth}) = \tau_{\text{base}} \times (\text{decay\_factor})^{\text{depth}}$ |

### XIV.3 Protection-Axiome

| ID           | Name              | Formalism                                         |
| ------------ | ----------------- | ------------------------------------------------- |
| **K19**      | AntiCalcification | Gini(Trust) > Threshold ⟹ Trigger(Redistribution) |
| **K_Stable** | SystemStability   | Anomalies(t) > Threshold ⟹ Mode = Degraded        |

### XIV.4 Resource-Axiome

| ID      | Name         | Formalism                                           |
| ------- | ------------ | --------------------------------------------------- |
| **K11** | GasMonotonic | $\gamma(t+1) \leq \gamma(t)$                        |
| **K13** | ManaRegen    | $\mu(t) = \min(\text{Cap}, \mu(t-1) + \text{Rate})$ |

### XIV.5 Identity-Axiome

| ID     | Name                 | Formalism                                         |
| ------ | -------------------- | ------------------------------------------------- |
| **K6** | SelfSovereign        | Keys $\subset$ Control(User)                      |
| **K7** | UniversalIdImmutable | created($\mathcal{U}$) ⟹ immutable($\mathcal{U}$) |

### XIV.6 Realm-Axiome

| ID      | Name            | Formalism                                                                                      |
| ------- | --------------- | ---------------------------------------------------------------------------------------------- |
| **K22** | RealmMembership | $\rho$.members $\subseteq \iota$                                                               |
| **K23** | CrossingPolicy  | crossing ⟹ verify($\iota$) $\land$ $\tau > \theta_{\min}$                                      |
| **K24** | SagaAtomicity   | saga.exec $\triangleright \tau$(participants); $\tau(\iota, \rho_1) \perp \tau(\iota, \rho_2)$ |

### XIV.7 Storage/URL-Axiome

| ID      | Name               | Formalism                                                                         |
| ------- | ------------------ | --------------------------------------------------------------------------------- |
| **K26** | URLSchema          | Unique Mapping: URL $\leftrightarrow$ Resource                                    |
| **K27** | ResourceResolution | resolve(url, ctx) → resource $\iff \mathcal{S}(\mathcal{R})$.match($\tau$, $\pi$) |
| **K28** | OpenAccessPolicy   | access(url, req) = policy($\mathcal{R}$).eval($\tau$, req)                        |

### XIV.8 WASM-Axiome

| ID              | Name             | Formalism                                                                                      |
| --------------- | ---------------- | ---------------------------------------------------------------------------------------------- |
| **K_WASM_Det**  | Determinism      | $\forall\pi, \sigma: \Psi_{\text{wasm}}(\pi, \sigma) = \Psi_{\text{wasm}}(\pi, \sigma)$        |
| **K_WASM_Iso**  | Isolation        | $\forall\pi: \text{effects}(\pi) \subseteq \Phi(\text{Bridge})$                                |
| **K_WASM_Fuel** | Fuel Bound       | fuel_consumed($\pi$) $\leq$ fuel_limit ⟹ terminates($\pi$)                                     |
| **K_Mode_Eq**   | Mode Equivalence | $\forall\pi, \sigma: \Psi_{\text{legacy}}(\pi, \sigma) \equiv \Psi_{\text{wasm}}(\pi, \sigma)$ |

### XIV.9 Migrations-Axiome

| ID     | Name                    | Formalism                                                                            |
| ------ | ----------------------- | ------------------------------------------------------------------------------------ |
| **M1** | BackupBeforeDestruction | $\forall \Phi_{\text{destr}}: \exists \Phi_{\text{backup}} < \Phi_{\text{destr}}$    |
| **M2** | CheckAfterMutation      | $\forall \Phi_{\text{mutate}}: \Phi_{\text{mutate}} \rightarrow \Phi_{\text{check}}$ |
| **M3** | RollbackIdempotent      | $\Phi_{\text{rollback}}^2 = \Phi_{\text{rollback}}$                                  |
| **M4** | AtomicPhase             | Phase_complete $\iff$ ($C$ = Success $\land$ $T$.Failed = 0)                         |
| **M5** | GitTrace                | $\forall$ Phase$_i$: $\exists$ commit$_i$                                            |

---

## XV. Kritische Pfade & Performance

> **Quelle:** relations.json, integration.json

### XV.1 Trust-Update Hot Path

$$T_{\text{total}} = \sum_i t_i$$

| Operation               | Latenz (µs) |
| ----------------------- | ----------- |
| τ.update()              | 50          |
| Σ_τ.apply() [atomic]    | 1           |
| StateEvent::TrustUpdate | 5           |
| log_and_apply()         | 30          |
| Hub.dispatch()          | 20          |
| **Total**               | **106**     |
| **Target**              | 50          |

**Optimization needed:** ✓

### XV.2 Realm-Crossing Complex Path

$$T_{\text{cross}} = T_{\text{parse}} + T_{\text{compose}} + T_{\text{eval}}$$

| Operation                     | Latenz (µs) |
| ----------------------------- | ----------- |
| Intent.recv()                 | 10          |
| IntentParser.parse()          | 100         |
| SagaComposer.compose()        | 200         |
| GatewayGuard.evaluate()       | 500         |
| — Ψ.validate()                | (300)       |
| — τ.get()                     | (5)         |
| — Quota.check()               | (10)        |
| StateEvent::CrossingEvaluated | 30          |
| **Total**                     | **1200**    |

**Acceptable:** ✓

### XV.3 Performance-Targets

| Metrik                 | Current | Phase 4  | Phase 6      |
| ---------------------- | ------- | -------- | ------------ |
| Event dispatch latency | 100 µs  | 50 µs    | 30 µs        |
| Observer notification  | sync    | async    | batch-async  |
| StateGraph traversal   | O(n)    | O(log n) | O(1) cached  |
| Memory footprint       | 100 MB  | 80 MB    | 60 MB        |
| Module coupling        | Tight   | Loose    | Event-driven |

---

## XVI. Globale Verbindungsanalyse

### XVI.1 Zentrale Abhängigkeits-Matrix

```
Identity ←[DependsOn]── Trust
Identity ←[DependsOn]── Event
Identity ←[DependsOn]── Realm
Identity ←[DependsOn]── Gateway
Identity ←[DependsOn]── Swarm

Trust ←[DependsOn]── Gateway, Saga, Gas, Mana, Formula, Consensus
Trust ←[Validates]── Diversity

Realm ←[DependsOn]── Identity
Realm ←[Aggregates]── Gateway
Realm ↔[Bidirectional]── Trust

ECLVM ←[DependsOn]── Gas, Mana
ECLVM ←[DependsOn]── Gateway, Saga

Sharding ←[DependsOn]── Realm, Storage
Sharding ←[Aggregates]── Trust, Gas
Sharding ↔[Bidirectional]── P2P
```

### XVI.2 Kritikalitäts-Ranking

| Rang | Komponente | crit(v) | Begründung                           |
| ---- | ---------- | ------- | ------------------------------------ |
| 1    | Identity   | 24      | Fundamentaler Akteur, alle hängen ab |
| 2    | Trust      | 20      | Zentrale emergente Eigenschaft       |
| 3    | Event      | 16      | Kommunikations-Backbone              |
| 4    | ECLVM      | 12      | Execution-Hub                        |
| 5    | Gateway    | 9       | Crossing-Kontrolle                   |
| 6    | Realm      | 9       | Isolation-Domäne                     |

### XVI.3 Synergie-Netzwerk

```
ι ──[10]── τ (fundamental)
τ ──[9]── Consensus (voting weight)
ρ ──[9]── Gateway (crossing)
Event ──[8]── Ω (persistence)
Ψ ──[8]── ρ (policy execution)
Shell ──[8]── ι, τ (capabilities, thresholds)
P2P ──[7]── ι (peer mapping)
🛡️ ──[7]── τ (monitoring)
```

### XVI.4 Das 7-Schichten-Immunsystem

$$\mathcal{L}_7 = \{L_1, L_2, \ldots, L_7\}$$

| $L_i$ | Name       | Funktion           | Ressource            |
| ----- | ---------- | ------------------ | -------------------- |
| $L_1$ | Gateway    | Preflight-Abwehr   | Mana-Check           |
| $L_2$ | Mana       | Anti-Spam/Flooding | $\mu$ regenerierend  |
| $L_3$ | Gas        | Anti-DoS/Loops     | $\gamma$ erschöpfend |
| $L_4$ | Trust      | Langfrist-Filter   | $\tau$ asymmetrisch  |
| $L_5$ | Realm      | Sandbox-Isolation  | Quota + Policies     |
| $L_6$ | DID        | Krypto-Bindung     | UTI + VC             |
| $L_7$ | Protection | Self-Healing       | CircuitBreaker       |

**Defense-Synergie:**
$$\text{Defense}(A) = \prod_{i=1}^{7} (1 - P_{\text{breach}}(L_i | A))$$

**Angreifer-Erschöpfungs-Theorem:**
$$\boxed{\forall A \in \text{Attackers}: \lim_{t \to \infty} \text{Resources}(A, t) = 0}$$

### XVI.5 Cross-Cutting Concerns

**Error-Hierarchy:**
$$\text{ErynoaError} = \Sigma(\text{IdentityError} \mid \text{ExecutionError} \mid \text{RealmError} \mid \text{StorageError} \mid \text{P2PError} \mid \text{ECLVMError} \mid \text{StateError})$$

**Telemetry:**
$$\forall \text{module}: \text{module} \rightarrow \text{telemetry}.\{\text{trace}, \text{metric}, \text{log}\}$$

**Config:**
$$\text{Config} = \text{Env} \oplus \text{ModuleConfigs}$$

---

## XVII. Zusammenfassung & Haupttheorem

### XVII.1 Das Pluto-System

$$
\boxed{\begin{aligned}
\mathbb{U}_{\text{Pluto}} &= \langle \mathcal{E}, \mathcal{R}, \mathcal{O}, \mathcal{K}, \mathcal{S}, \mathcal{N}, \Psi, \Phi \rangle \\[8pt]
\text{wobei:} \quad &\mathcal{E} = \{\iota, \rho, \tau, \Sigma, \Omega, \pi, \text{🛡️}, \text{Hub}\} \\
&\mathcal{R} = \{\triangleright, \rightarrow, \vdash, \uplus, \leftrightarrow, \rightsquigarrow\} \\
&\mathcal{K} = \{K_1, \ldots, K_{28}, M_1, \ldots, M_5\} \\
&\mathcal{N} = \langle \Sigma, \text{Hub}, \mathcal{E}_{\text{ng}}, \mathcal{O}_{\text{bs}}, \mathcal{A} \rangle \\
&\Psi = \Psi_{\text{legacy}} \cup \Psi_{\text{wasm}}
\end{aligned}}
$$

### XVII.2 Haupt-Invarianten

1. **Hierarchie:** $L_i \prec L_j \Leftrightarrow i < j$
2. **Kausalität:** $\text{seq}(p) < \text{seq}(e) \; \forall p \in \text{parents}(e)$
3. **Trust-Bound:** $\tau \in [0,1]^6$
4. **Asymmetrie:** $\Delta\tau^- / \Delta\tau^+ \approx 2$
5. **Snapshot-Isolation:** Lesezugriffe blockieren nie
6. **Realm-Isolation:** $\text{State}(r) \cap \text{State}(r') = \emptyset$
7. **Mode-Äquivalenz:** $\Psi_{\text{legacy}} \equiv \Psi_{\text{wasm}}$

### XVII.3 Das Haupttheorem

$$\boxed{\mathcal{S}_{\text{Pluto}} = \mathcal{L}_7 \circ (\tau, \gamma, \mu) \implies \text{Self-Healing} \land \text{Angreifer-Erschöpfung} \land \text{Dezentrale Souveränität}}$$

**Korollare:**

1. **Sybil-Resistenz:**
   $$\text{Rate}_{\text{Veteran}} = 10 \times \text{Rate}_{\text{Sybil-Cluster}}$$

2. **Trust-Emergenz:**
   $$\tau \uparrow \implies (\mu, \gamma) \uparrow \implies P(\text{Erfolg}) \uparrow \implies \tau \uparrow$$

3. **Angreifer-Erschöpfung:**
   $$\forall A: \lim_{t \to \infty} \text{Resources}(A, t) = 0$$

4. **Governance-Fairness:**
   $$W(m) = G(m) \cdot (1 + \alpha \cdot T_{\text{rel}}(m))$$

5. **URL-Determinismus:**
   $$\forall \text{URL}: \text{resolve}(\text{URL}) \text{ terminiert} \in O(1) + O(\log n)$$

---

## Anhang A: Vollständige Symbolreferenz

| Symbol                               | Bedeutung                | Quelle                       |
| ------------------------------------ | ------------------------ | ---------------------------- |
| $\mathbb{U}_{\text{Pluto}}$          | Das Pluto-Universum      | README.md                    |
| $\mathcal{E}$                        | Entities                 | entities.json                |
| $\mathcal{R}$                        | Relations                | relations.json               |
| $\mathcal{O}$                        | Operations               | formulas.json                |
| $\mathcal{K}$                        | Constraints              | constraints.json             |
| $\mathcal{S}$                        | Synergies                | formulas.json                |
| $\mathcal{N}$                        | Nervous System           | integration.json             |
| $\Psi$                               | ECLVM                    | 06-eclvm-wasm-migration.json |
| $\Phi$                               | Migrations               | migrations.json              |
| $\Sigma$                             | UnifiedState             | entities.json                |
| Hub                                  | SynapseHub               | integration.json             |
| $\iota$                              | Identity                 | entities.json                |
| $\tau$ / $\vec{\tau}$ / $\mathbb{T}$ | Trust / TrustVector      | 09-TRUST-GAS-MANA.pluto.md   |
| $\gamma$                             | Gas                      | 09-TRUST-GAS-MANA.pluto.md   |
| $\mu$                                | Mana                     | 09-TRUST-GAS-MANA.pluto.md   |
| $\rho$                               | Realm                    | entities.json                |
| $\Omega$                             | Storage                  | entities.json                |
| $\pi$                                | Package / Policy         | entities.json                |
| 🛡️                                   | Protection               | entities.json                |
| $\mathcal{L}$                        | Sharding-System          | 14-SHARDING.pluto.md         |
| $\mathcal{G}$                        | Governance               | 16-REALM-GOVERNANCE.pluto.md |
| $h$                                  | Hash-Funktion (Sharding) | 14-SHARDING.pluto.md         |
| $W$                                  | Stimmgewicht             | 16-REALM-GOVERNANCE.pluto.md |
| $G$                                  | Governance-Basis         | 16-REALM-GOVERNANCE.pluto.md |
| $\Delta$                             | Delegation               | 10-IDENTITY.pluto.md         |
| $\partial$                           | Ableitungsfunktion       | 10-IDENTITY.pluto.md         |
| $\triangleright$                     | DependsOn                | relations.json               |
| $\rightarrow$                        | Triggers                 | relations.json               |
| $\vdash$                             | Validates                | relations.json               |
| $\uplus$                             | Aggregates               | relations.json               |
| $\leftrightarrow$                    | Bidirectional            | relations.json               |
| $\rightsquigarrow$                   | Updates                  | relations.json               |

---

## Anhang B: Quellenverzeichnis

| Datei                                      | Inhalt                                                |
| ------------------------------------------ | ----------------------------------------------------- |
| README.md                                  | Übersicht & Struktur                                  |
| entities.json                              | Typen & Objekte ($\mathcal{E}$)                       |
| relations.json                             | Beziehungsmatrix ($\mathcal{R}$)                      |
| formulas.json                              | Operations & Synergies ($\mathcal{O}$, $\mathcal{S}$) |
| constraints.json                           | Axiome & Regeln ($\mathcal{K}$)                       |
| integration.json                           | Nervensystem ($\mathcal{N}$)                          |
| migrations.json                            | Migrations-Algebra ($\Phi$)                           |
| 06-eclvm-wasm-migration.json               | Execution Engine ($\Psi$)                             |
| 08-STATE-KERNGEDANKEN.md                   | State-Design & Graph                                  |
| 09-TRUST-GAS-MANA-DREIEINIGKEIT.pluto.md   | Dreieinigkeit & Immunsystem                           |
| 10-IDENTITY-MULTI-DID-ARCHITEKTUR.pluto.md | Identity & DIDs                                       |
| 14-SHARDING-ARCHITEKTUR.pluto.md           | Horizontale Skalierung                                |
| 16-REALM-GOVERNANCE.pluto.md               | Souveräne Entscheidungsfindung                        |
| 17-REALM-URL-RESOURCE-ADDRESSING.pluto.md  | URL-Adressierung                                      |
| **00-OVERVIEW.md**                         | Vision, Roadmap, Kern-Prinzipien 🆕                   |
| **02-ZIEL-ARCHITEKTUR.md**                 | Target-State-Verzeichnisstruktur 🆕                   |
| **03-BEZIEHUNGSMATRIX.md**                 | Detaillierte Modul-Beziehungen 🆕                     |

---

## XVIII. Ziel-Architektur (Target-State) 🆕

> **Quelle:** 00-OVERVIEW.md, 02-ZIEL-ARCHITEKTUR.md

### XVIII.1 Vision & Metapher

$$\boxed{\mathbb{U}_{\text{Pluto}} \cong \text{Organismus}}$$

| Biologische Metapher | System-Komponente | Funktion                    |
| -------------------- | ----------------- | --------------------------- |
| 🧠 Gehirn            | `UnifiedState`    | Zentrale Koordination       |
| ⚡ Synapsen          | `SynapseHub`      | Signal-Übertragung          |
| 💪 Muskeln           | `Engines`         | Ausführung                  |
| 🛡️ Immunsystem       | `Protection`      | Schutz & Selbstheilung      |
| 🧬 DNA               | `Domain`          | Typen & Invarianten         |
| 💾 Gedächtnis        | `Storage`         | Persistenz                  |
| 🔗 Nervenbahnen      | `P2P`             | Kommunikation               |
| 🫁 Organe            | `Realm`           | Isolation & Spezialisierung |

### XVIII.2 Kern-Prinzipien $\mathcal{P}_{\text{core}}$

$$\mathcal{P}_{\text{core}} = \{P_{\Sigma}, P_{\text{syn}}, P_K, P_{\text{eff}}\}$$

| Symbol           | Prinzip                    | Formalisierung                                                                        |
| ---------------- | -------------------------- | ------------------------------------------------------------------------------------- |
| $P_{\Sigma}$     | State als Nervensystem     | $\forall m \in \mathcal{M}: \text{read/write}(m) \subseteq \text{Interfaces}(\Sigma)$ |
| $P_{\text{syn}}$ | Synergistische Integration | $\forall m_1, m_2 \in \mathcal{M}: \neg(m_1 \triangleright_{\text{direct}} m_2)$      |
| $P_K$            | Axiom-Treue                | $\forall c \in \text{Components}: c.\text{axioms} \subseteq \mathcal{K}$              |
| $P_{\text{eff}}$ | Effizienz durch Design     | $\text{critical\_paths} \in O(1)$                                                     |

### XVIII.3 Ist-Zustand → Ziel-Zustand

**Transformation $\mathcal{T}$:**
$$\mathcal{T}: \mathbb{U}_{\text{Ist}} \rightarrow \mathbb{U}_{\text{Ziel}}$$

| Metrik                     | $\mathbb{U}_{\text{Ist}}$ | $\mathbb{U}_{\text{Ziel}}$ | $\Delta$ |
| -------------------------- | ------------------------- | -------------------------- | -------- |
| `state.rs` LOC             | 21.495                    | < 2.000                    | -90%     |
| `state_integration.rs` LOC | 6.427                     | < 1.500                    | -77%     |
| Max. Dateigröße            | 823 KB                    | < 50 KB                    | -94%     |
| Trait-Coverage             | 60%                       | 100%                       | +40%     |
| Test-Coverage              | 60%                       | > 85%                      | +25%     |
| Compile-Zeit               | 4 min                     | 2 min                      | -50%     |
| Event-Dispatch             | 100 µs                    | 50 µs                      | -50%     |
| Memory                     | 100 MB                    | 60 MB                      | -40%     |

### XVIII.4 Ziel-Verzeichnisstruktur $\mathcal{D}_{\text{Target}}$

$$\mathcal{D}_{\text{Target}} = \langle \text{Modules}, \text{Hierarchy}, \text{Responsibility} \rangle$$

**Top-Level-Module:**

```
backend/src/
├── 🧠 nervous_system/     (~8.000 LOC)  — Zentraler State, Event-Sourcing, Merkle
├── 🔌 synapses/           (~2.000 LOC)  — Observer-Hub, Adapter
├── 🆔 identity/           (~1.500 LOC)  — DID-Management, Keys, Credentials
├── ⚙️ engines/            (~3.000 LOC)  — Trust, Event, Formula, Consensus
├── 💰 execution/          (~1.000 LOC)  — Gas, Mana, ExecutionContext
├── 🌐 realm/              (~2.500 LOC)  — Realm-State, Gateway, Saga
├── 🛡️ protection/         (~2.000 LOC)  — Anomaly, Diversity, Calibration
├── 🔗 p2p/                (~3.000 LOC)  — libp2p Integration
├── 🎛️ eclvm/              (~3.500 LOC)  — ECLVM + WASM
├── 📦 storage/            (~4.000 LOC)  — Fjall-Stores, Archive
└── 📊 domain/             (~2.000 LOC)  — Typen, Traits, Errors
```

**LOC-Invariante:**
$$\boxed{\sum_m \text{LOC}(m) \approx 32.500 \quad \text{vs. Ist: } 27.922}$$

### XVIII.5 Nervous-System-Dekomposition

$$\text{nervous\_system} = \langle \Sigma, \mathcal{E}_s, \mathcal{M}, \mathcal{G}, \mathcal{I} \rangle$$

| Submodul           | Inhalt                                                  | LOC    |
| ------------------ | ------------------------------------------------------- | ------ |
| `unified_state.rs` | UnifiedState                                            | ~2.000 |
| `event_sourcing/`  | StateEvent (42 Var.), WrappedEvent, Log, Replay         | ~2.000 |
| `merkle/`          | MerkleStateTracker, Delta, Proofs                       | ~1.000 |
| `components/`      | Core, Execution, Protection, Peer, P2P, Identity, ECLVM | ~2.000 |
| `graph/`           | StateComponent (37 Var.), Relations, Analysis           | ~500   |
| `infrastructure/`  | EventBus, Broadcaster, CircuitBreaker, MultiGas         | ~500   |

### XVIII.6 Trait-Hierarchie $\mathcal{H}_{\text{Trait}}$

$$\mathcal{H}_{\text{Trait}} = \langle \text{StateLayer}, \text{StateObserver}, \text{Metered}, \text{Resettable} \rangle$$

**StateLayer-Trait:**
$$\text{StateLayer} = \langle \text{Snapshot}, \text{health\_score}, \text{apply\_event}, \text{component} \rangle$$

**StateObserver-Trait:**
$$\text{StateObserver} = \langle \text{on\_event}, \text{target\_component}, \text{priority} \rangle$$

**Constraint:**
$$\boxed{\forall m \in \mathcal{M}_{\text{state}}: m \vDash \text{StateLayer}}$$

---

## XIX. Erweiterte Beziehungsmatrix 🆕

> **Quelle:** 03-BEZIEHUNGSMATRIX.md

### XIX.1 Identity-Abhängigkeitsmatrix

| Von                    | Zu            | Relation                   | Axiom | Formalisierung                                             |
| ---------------------- | ------------- | -------------------------- | ----- | ---------------------------------------------------------- |
| engines/trust          | identity      | $\triangleright$           | K6    | $\tau(x) \triangleright \iota(x)$                          |
| identity               | engines/trust | $\rightarrow$              | K2    | $\text{spawn}(\iota) \rightarrow \text{init}(\tau(\iota))$ |
| engines/event          | identity      | $\triangleright$           | K9    | $\forall e: \text{author}(e) \in \iota$                    |
| realm/gateway          | identity      | $\triangleright$, $\vdash$ | K23   | $\text{crossing} \implies \text{verify}(\iota)$            |
| p2p/swarm              | identity      | $\triangleright$           | —     | $\text{PeerId} \equiv \text{DeviceDID} \in \iota$          |
| protection/anomaly     | identity      | $\vdash$                   | K26   | $\text{🛡️.A} \vdash \neg\text{anomalous}(\iota)$           |
| synapses               | identity      | $\uplus$                   | —     | $\text{Hub} \uplus \text{Observer}(\iota)$                 |
| eclvm                  | identity      | $\triangleright$           | —     | $\Psi.\text{call} \triangleright \text{caller} \in \iota$  |
| storage/identity_store | identity      | $\uplus$                   | —     | DID-Persistenz                                             |

### XIX.2 Trust-Abhängigkeitsmatrix

| Von                  | Zu            | Relation         | Axiom |
| -------------------- | ------------- | ---------------- | ----- |
| realm/gateway        | engines/trust | $\triangleright$ | K23   |
| realm/saga           | engines/trust | $\triangleright$ | K24   |
| execution/gas        | engines/trust | $\triangleright$ | —     |
| execution/mana       | engines/trust | $\triangleright$ | —     |
| engines/formula      | engines/trust | $\triangleright$ | K15   |
| engines/consensus    | engines/trust | $\triangleright$ | K18   |
| p2p/gossip           | engines/trust | $\triangleright$ | —     |
| protection/diversity | engines/trust | $\vdash$         | K19   |

### XIX.3 Realm-Abhängigkeitsmatrix

| Von           | Zu            | Relation          | Axiom |
| ------------- | ------------- | ----------------- | ----- |
| realm         | identity      | $\triangleright$  | K22   |
| realm         | engines/trust | $\leftrightarrow$ | K22   |
| realm/gateway | eclvm         | $\triangleright$  | K23   |
| realm/saga    | eclvm         | $\triangleright$  | K24   |
| realm/quota   | protection    | $\rightarrow$     | K22   |
| realm         | storage/realm | $\uplus$          | —     |

### XIX.4 P2P-Abhängigkeitsmatrix

| Von         | Zu            | Relation         |
| ----------- | ------------- | ---------------- |
| p2p/swarm   | identity      | $\triangleright$ |
| p2p/gossip  | engines/trust | $\triangleright$ |
| p2p/gossip  | engines/event | $\rightarrow$    |
| p2p/dht     | p2p/swarm     | $\uplus$         |
| p2p/relay   | engines/trust | $\triangleright$ |
| p2p/privacy | identity      | $\triangleright$ |

### XIX.5 Shell-Access-Abhängigkeiten (Agent → Peer)

| Von                 | Zu                 | Relation           | Axiom | Formalisierung            |
| ------------------- | ------------------ | ------------------ | ----- | ------------------------- |
| peer/shell          | identity           | $\triangleright$   | K8    | Agent-DID validieren      |
| peer/shell          | engines/trust      | $\triangleright$   | K6    | Trust-Schwellen prüfen    |
| peer/shell          | eclvm              | $\triangleright$   | K23   | ShellAccessGateway Policy |
| peer/shell          | domain/capability  | $\triangleright$   | K8    | ShellCapability prüfen    |
| identity/delegation | peer/shell         | $\rightarrow$      | K8    | Capability-Delegation     |
| peer/shell          | engines/event      | $\rightarrow$      | K9    | Shell-Audit-Events        |
| peer/shell          | protection/anomaly | $\rightarrow$      | K26   | Anomalie-Erkennung        |
| peer/shell          | engines/trust      | $\rightsquigarrow$ | K6    | Trust-Impact nach Aktion  |

### XIX.6 Vollständige Synergy-Matrix

| Modul A    | Modul B      | Score    | Grund             | Axiom |
| ---------- | ------------ | -------- | ----------------- | ----- |
| identity   | trust        | 10/10    | Fundamental       | K6    |
| trust      | consensus    | 9/10     | Voting            | K18   |
| realm      | gateway      | 9/10     | Crossing          | K23   |
| event      | storage      | 8/10     | Persistenz        | K9    |
| eclvm      | realm        | 8/10     | Policies          | K23   |
| **shell**  | **identity** | **8/10** | Capabilities      | K8    |
| **shell**  | **trust**    | **8/10** | Trust-Schwellen   | K6    |
| **shell**  | **eclvm**    | **7/10** | Policy-Evaluation | K23   |
| p2p        | identity     | 7/10     | PeerId            | —     |
| protection | trust        | 7/10     | Monitoring        | K19   |
| formula    | trust        | 6/10     | Input             | K15   |

### XIX.7 Event-Flow-Formalisierung

$$\mathcal{F}_{\text{Event}} = \langle \text{Src}, \text{Pipe}, \text{Dispatch}, \text{Observers} \rangle$$

**Pipeline:**
$$\text{P2P} \xrightarrow{\text{NetworkEvent}} \text{EventBus}_I \xrightarrow{\text{try\_send}} \Sigma \xrightarrow{\text{log\_apply}} \text{Hub} \xrightarrow{\text{dispatch}} \mathcal{O}$$

**Dispatch-Algorithmus:**
$$\text{dispatch}(e) = \bigcup_{o \in \mathcal{O}_{\text{direct}}(e)} o.\text{on\_event}(e) \cup \bigcup_{o \in \mathcal{O}_{\text{trans}}(e)} o.\text{on\_event}(e)$$

wobei:

- $\mathcal{O}_{\text{direct}}(e) = \mathcal{O}_{\text{reg}}[e.\text{component}]$
- $\mathcal{O}_{\text{trans}}(e) = \bigcup_{c \in \text{triggered\_by}(e.\text{component})} \mathcal{O}_{\text{reg}}[c]$

### XIX.8 Integration-Pattern (Code-Beispiel)

$$\text{GatewayGuard.evaluate} = \iota \circ \tau \circ Q \circ \Psi \circ \Sigma$$

**Formale Sequenz:**

1. $\iota_{\text{resolve}}: \text{UniversalId} \rightarrow \text{DID}$ (identity/)
2. $\tau_{\text{get}}: \iota \rightarrow \vec{\tau}$ (engines/trust)
3. $Q_{\text{check}}: \rho \times \text{ResourceType} \rightarrow \text{Result}$ (realm/quota)
4. $\Psi_{\text{eval}}: \text{Policy} \times \text{Context} \rightarrow \text{Decision}$ (eclvm/)
5. $\Sigma_{\text{emit}}: \text{StateEvent} \rightarrow \text{()}$ (nervous_system/)

---

## XX. Implementierungs-Roadmap 🆕

> **Quelle:** 00-OVERVIEW.md

### XX.1 Phasen-Übersicht

$$\mathcal{R}_{\text{Impl}} = \langle \Phi_1, \Phi_2, \Phi_3, \Phi_4, \Phi_5, \Phi_6 \rangle$$

| $\Phi_i$ | Name         | Wochen | Hauptaktivitäten                |
| -------- | ------------ | ------ | ------------------------------- |
| $\Phi_1$ | Foundation   | 1-2    | Traits, Errors, Directories     |
| $\Phi_2$ | Decompose    | 3-5    | Split state.rs, Extract Modules |
| $\Phi_3$ | Synaptic Hub | 6-7    | Observer Hub, Adapters          |
| $\Phi_4$ | Integrate    | 8-10   | P2P, Storage, Engines           |
| $\Phi_5$ | ECLVM→WASM   | 11-13  | Wasmtime, WIT, Bridge           |
| $\Phi_6$ | Optimize     | 14     | Performance, Memory, Polish     |

### XX.2 Zeitliche Abfolge

```
Woche 1-2     Woche 3-5       Woche 6-7      Woche 8-10     Woche 11-13    Woche 14
    │             │               │               │               │            │
    ▼             ▼               ▼               ▼               ▼            ▼
┌────────┐   ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐   ┌────────┐
│ Φ₁     │   │   Φ₂    │    │   Φ₃    │    │   Φ₄    │    │   Φ₅    │   │  Φ₆    │
│Foundatn│   │Decompose│    │Synaptic │    │Integrate│    │ECLVM    │   │Optimize│
│        │   │         │    │  Hub    │    │         │    │ →WASM   │   │        │
│•Traits │   │•Split   │    │•Observer│    │•P2P     │    │•Wasmtime│   │•Perf   │
│•Errors │   │ state.rs│    │ Hub     │    │•Storage │    │•WIT     │   │•Memory │
│•Dirs   │   │•Extract │    │•Adapters│    │•Engines │    │•Bridge  │   │•Polish │
└────────┘   └─────────┘    └─────────┘    └─────────┘    └─────────┘   └────────┘
```

### XX.3 Phasen-Erfolgsmetriken

| Metrik         | $\Phi_1$ | $\Phi_2$ | $\Phi_3$ | $\Phi_4$ | $\Phi_5$ | $\Phi_6$ |
| -------------- | -------- | -------- | -------- | -------- | -------- | -------- |
| Max. LOC/Datei | 21.495   | 5.000    | 3.000    | 2.500    | 2.000    | 2.000    |
| Trait-Coverage | 60%      | 70%      | 80%      | 90%      | 95%      | 100%     |
| Compile-Zeit   | 4 min    | 3.5 min  | 3 min    | 2.5 min  | 2.5 min  | 2 min    |
| Event-Dispatch | 100 µs   | 100 µs   | 75 µs    | 60 µs    | 55 µs    | 50 µs    |
| Memory         | 100 MB   | 95 MB    | 85 MB    | 75 MB    | 70 MB    | 60 MB    |

### XX.4 Migrations-Invarianten

$$\boxed{\forall \Phi_i: \Phi_i \text{ complete} \iff (\text{Build} = \text{Success}) \land (\text{Tests} = \text{Pass})}$$

**Git-Workflow:**
$$\text{Branch} = \texttt{refactor/projekt-pluto}$$
$$\forall \Phi_i: \text{complete}(\Phi_i) \implies \text{commit}(\Phi_i) \land \text{tag}(\texttt{phase-}i)$$

### XX.5 Kritische Pfade (Performance-Targets)

**Trust-Update Hot Path:**
$$T_{\text{target}} = 50 \text{ µs}$$

| Operation                    | Ist (µs) | Ziel (µs) | $\Phi$   |
| ---------------------------- | -------- | --------- | -------- |
| TrustEngine.update()         | 50       | 30        | $\Phi_4$ |
| TrustState.update() [Atomic] | 1        | 1         | —        |
| StateEvent erstellen         | 5        | 3         | $\Phi_3$ |
| log_and_apply()              | 30       | 10        | $\Phi_2$ |
| SynapseHub.dispatch()        | 20       | 6         | $\Phi_3$ |
| **Total**                    | **106**  | **50**    | —        |

**Realm-Crossing Complex Path:**
$$T_{\text{target}} = 1.0 \text{ ms}$$

| Operation                     | Ist (µs)  | Ziel (µs) | $\Phi$   |
| ----------------------------- | --------- | --------- | -------- |
| Intent empfangen              | 10        | 5         | $\Phi_4$ |
| IntentParser.parse()          | 100       | 50        | $\Phi_4$ |
| SagaComposer.compose()        | 200       | 100       | $\Phi_4$ |
| GatewayGuard.evaluate()       | 500       | 300       | $\Phi_5$ |
| StateEvent::CrossingEvaluated | 30        | 15        | $\Phi_3$ |
| **Total**                     | **1.200** | **1.000** | —        |

---

**∎ Q.E.D.**

---

## XXI. Package-Manager-Algebra 📦

> **Quelle:** 11-PACKAGEMANAGER-BLUEPRINT-TRANSFORMATION.md

### XXI.1 Package-Definition

$$\boxed{\pi = \langle \text{Manifest}, \mathcal{D}, \text{Artifacts}, \sigma, \text{lifecycle} \rangle}$$

| Komponente    | Typ                                       | Beschreibung                    |
| ------------- | ----------------------------------------- | ------------------------------- |
| Manifest      | ECL-Struct                                | Metadaten (name, version, etc.) |
| $\mathcal{D}$ | $\mathcal{P}(\text{Dependency})$          | Abhängigkeiten mit Constraints  |
| Artifacts     | $\text{BlobId}$                           | Content-Hash (BLAKE3)           |
| $\sigma$      | $\text{Sig}_{\text{DID}}$                 | Publisher-Signatur              |
| lifecycle     | ∈ {Draft, Published, Deprecated, Revoked} | Lebenszyklusstatus              |

### XXI.2 Manifest-Struktur (ECL)

$$\text{Manifest} = \langle \text{name}, \text{version}, \mathcal{D}, \text{policy}, \text{stores}, \text{policies}, \text{ui}, \text{wallet} \rangle$$

**SemVer-Definition:**
$$v = (M, m, p) \in \mathbb{N}^3, \quad v_1 \prec v_2 \iff (M_1, m_1, p_1) <_{\text{lex}} (M_2, m_2, p_2)$$

**Version-Constraint-Algebra:**

| Constraint    | Formale Definition                                  |
| ------------- | --------------------------------------------------- |
| `^1.2.3`      | $\{v \mid v.M = 1 \land v \succeq (1,2,3)\}$        |
| `~1.2.3`      | $\{v \mid v.M = 1 \land v.m = 2 \land v.p \geq 3\}$ |
| `>=1.0, <2.0` | $\{v \mid (1,0,0) \preceq v \prec (2,0,0)\}$        |
| `1.x`         | $\{v \mid v.M = 1\}$                                |

### XXI.3 Dependency-Graph

$$G_{\mathcal{D}} = (V, E), \quad V = \mathcal{P}(\pi), \quad E \subseteq V \times V \times \text{Constraint}$$

**Transitive Closure:**
$$\text{deps}^*(\pi) = \bigcup_{n=0}^{\infty} \text{deps}^n(\pi)$$

**Acyclicity-Axiom (K_PkgAcyclic):**
$$\boxed{\neg \exists \pi: \pi \in \text{deps}^*(\pi)}$$

### XXI.4 Package-Lifecycle FSM

$$\mathcal{L}_\pi = (\Sigma_\pi, \Sigma_0, \delta_\pi, F_\pi)$$

```
                    ┌───────────────────────────────────────────────────────────────┐
                    │                    PACKAGE LIFECYCLE FSM                      │
                    └───────────────────────────────────────────────────────────────┘

     ┌──────────┐   validate()   ┌──────────┐   build()   ┌───────────┐   publish()   ┌───────────┐
     │  CREATE  │───────────────►│ VALIDATE │────────────►│   BUILD   │──────────────►│  PUBLISH  │
     └──────────┘                └──────────┘             └───────────┘              └───────────┘
                                      │                                                    │
                                      │ error                                              │
                                      ▼                                             discover()
                                 ┌──────────┐                                              │
                                 │  ERROR   │                                              ▼
                                 └──────────┘                                       ┌───────────┐
                                                                                    │ DISCOVER  │
                                                                                    └───────────┘
                                                                                          │
     ┌───────────┐   deprecate()  ┌───────────┐   upgrade()   ┌──────────┐   install()    │
     │ DEPRECATE │◄───────────────│    RUN    │◄──────────────│ UPGRADE  │◄───────────────┘
     └───────────┘                └───────────┘               └──────────┘
```

**State-Transition-Funktion:**
$$\delta_\pi: \Sigma_\pi \times \text{Action} \rightarrow \Sigma_\pi$$

| $s$      | Action      | $s'$      | Preconditions                                    |
| -------- | ----------- | --------- | ------------------------------------------------ |
| CREATE   | validate()  | VALIDATE  | $\text{Manifest} \vDash \text{Schema}$           |
| VALIDATE | build()     | BUILD     | ECL compiles                                     |
| BUILD    | publish()   | PUBLISH   | $\tau_R \geq \theta_R \land \nu \geq \theta_\nu$ |
| PUBLISH  | discover()  | DISCOVER  | Gossip propagation                               |
| DISCOVER | install()   | UPGRADE   | $\text{resolve}(\mathcal{D}) \neq \emptyset$     |
| UPGRADE  | run()       | RUN       | Stores created, Policies deployed                |
| RUN      | upgrade()   | UPGRADE   | New version available                            |
| RUN      | deprecate() | DEPRECATE | Owner action                                     |

### XXI.5 Resolution-Algorithmus

$$\boxed{\text{resolve}: \mathcal{P}(\pi) \times \text{Policy} \rightarrow \text{DAG} \cup \{\bot\}}$$

**5-Step-Pipeline:**

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                         RESOLUTION ALGORITHM                                        │
└─────────────────────────────────────────────────────────────────────────────────────┘

    Step 1: COLLECT         Step 2: FILTER          Step 3: SOLVE
    ─────────────────      ─────────────────       ─────────────────
    DFS traverse deps      Apply trust filter      SAT-based version
    │                      │                       │
    ▼                      ▼                       ▼
    Candidates[]           Filtered[]              Solution{}

                                                          │
    Step 5: VERIFY         Step 4: LOCK                   │
    ─────────────────      ─────────────────              │
    Hash + Sig check       Write lockfile                 │
    │                      │                              │
    ▼                      ▼                              ▼
    Verified{}             Locked{}      ◄────────────────┘
```

**Formale Schritte:**

$$\text{Collect}(\pi) = \{(\pi', c) \mid \pi' \in \text{registry} \land c \in \text{constraints}(\pi, \pi')\}$$

$$\text{Filter}(\mathcal{C}, \theta) = \{(\pi, c) \in \mathcal{C} \mid \tau(\text{publisher}(\pi)) \geq \theta\}$$

$$\text{Solve}(\mathcal{F}) = \text{SAT}(\bigwedge_{(\pi, c) \in \mathcal{F}} \text{version}(\pi) \in c)$$

$$\text{Lock}(S) = \{(\pi, v, h) \mid \pi \in S \land v = \text{selected}(\pi) \land h = \text{hash}(\pi)\}$$

$$\text{Verify}(L) = \bigwedge_{(\pi, v, h) \in L} (\text{hash}(\pi@v) = h \land \text{sig}(\pi@v) \vDash \text{publisher}(\pi))$$

### XXI.6 Trust-Gated Publishing

**Publishing-Preconditions (Κ_PkgTrust):**
$$\boxed{\text{publish}(\pi) \iff \tau_R(\iota) \geq \theta_R \land \tau_\Omega(\iota) \geq \theta_\Omega \land \nu(\pi) \geq \theta_\nu}$$

| Parameter       | Default | Bedeutung             |
| --------------- | ------- | --------------------- |
| $\theta_R$      | 0.8     | Min Reliability-Trust |
| $\theta_\Omega$ | 1.5     | Min Omega-Trust       |
| $\theta_\nu$    | 3.0     | Min Novelty-Score     |

**AgentManaged Trust-Penalty:**
$$\tau'_R = \tau_R \cdot 0.8 \quad \text{falls mode} = \text{AgentManaged}$$

### XXI.7 Content-Integrity (Κ_PkgIntegrity)

$$\boxed{\text{PackageId}(\pi) = \text{BLAKE3}(\text{Content}(\pi))}$$

**Invarianten:**

- Immutabilität: $\forall t_1, t_2: \text{Content}(\pi, t_1) = \text{Content}(\pi, t_2)$
- Collision-Resistance: $\Pr[\text{BLAKE3}(c_1) = \text{BLAKE3}(c_2) \mid c_1 \neq c_2] \approx 2^{-256}$
- Signatur-Bindung: $\sigma = \text{Sign}_{\text{sk}}(\text{PackageId}(\pi))$

### XXI.8 Lockfile-Struktur

$$\text{Lockfile} = \langle \text{version}, \text{generated}, \{(\pi_i, v_i, h_i, \text{source}_i)\}_{i=1}^n \rangle$$

**Determinismus-Axiom:**
$$\boxed{\forall t_1, t_2: \text{Manifest}(t_1) = \text{Manifest}(t_2) \implies \text{Lockfile}(t_1) = \text{Lockfile}(t_2)}$$

### XXI.9 StateGraph-Integration

$$\text{PackageManager} \in \mathcal{V}(\text{StateGraph})$$

| Relation      | Target       | Semantik                       | Axiom |
| ------------- | ------------ | ------------------------------ | ----- |
| DependsOn     | Identity/DID | Publisher-Auth, Signatur       | K6    |
| DependsOn     | Trust        | Publish/Install Trust-Checks   | K2    |
| DependsOn     | Gas          | Resolution/Install Gas-Kosten  | K11   |
| DependsOn     | Mana         | Publish/Install Mana-Kosten    | K13   |
| Aggregates    | Storage      | Packages in StorageState       | K10   |
| Triggers      | Event        | Publish/Install/Upgrade Events | K9    |
| Validates     | Realm        | Realm-Compatibility            | K24   |
| Bidirectional | P2P          | Registry-Sync via Gossip       | —     |
| Bidirectional | ECLVM        | Resolution-Policies als ECL    | —     |

### XXI.10 Package-Manager-State

$$\Sigma_{\text{PM}} = \langle \mathcal{M}_{\text{pub}}, \mathcal{M}_{\text{res}}, \mathcal{M}_{\text{inst}}, \mathcal{M}_{\text{reg}}, \mathcal{M}_{\text{cache}}, \mathcal{M}_{\rho} \rangle$$

| Metrik-Gruppe                | Felder                                                        |
| ---------------------------- | ------------------------------------------------------------- |
| $\mathcal{M}_{\text{pub}}$   | packages_published, publish_errors, novelty_rejections        |
| $\mathcal{M}_{\text{res}}$   | dependencies_resolved, conflicts, errors, avg_time, max_depth |
| $\mathcal{M}_{\text{inst}}$  | packages_installed, errors, upgrades, rollbacks               |
| $\mathcal{M}_{\text{reg}}$   | registry_packages, syncs, downloaded, uploaded                |
| $\mathcal{M}_{\text{cache}}$ | cache_hits, cache_misses, cache_size_bytes                    |
| $\mathcal{M}_{\rho}$         | gas_consumed, mana_consumed                                   |

### XXI.11 StateEvents (Package-Domain)

$$\mathcal{E}_\pi = \{\text{PackagePublished}, \text{PackageDeprecated}, \text{DependencyResolved}, \text{ResolutionConflict}, \text{PackageInstalled}, \text{PackageUpgraded}, ...\}$$

**Event-Struktur (Publish):**
$$e_{\text{pub}} = \langle \text{package\_id}, v, \text{publisher\_did}, h, \nu, \mu_{\text{cost}} \rangle$$

---

## XXII. Package-Synergien & Emergente Features 📦

> **Quelle:** 12-PACKAGEMANAGER-SYNERGIEN-FEATURES.md

### XXII.1 Synergy-Matrix

$$\mathcal{S}_{\text{PM}} = \{(\text{Trust}, \text{PM}), (\text{Identity}, \text{PM}), (\text{Gas/Mana}, \text{PM}), (\text{Realm}, \text{PM}), (\text{P2P}, \text{PM}), (\text{Storage}, \text{PM}), (\text{ECLVM}, \text{PM})\}$$

```
                           ┌─────────────────────────────────────────────────────────────┐
                           │              SYNERGY HEXAGON: PACKAGEMANAGER                │
                           └─────────────────────────────────────────────────────────────┘

                                                   TRUST
                                                     │
                                     ┌───────────────┼───────────────┐
                                     │               │               │
                                     ▼               ▼               ▼
                                 IDENTITY ◄──── PACKAGE ────► GAS/MANA
                                     │          MANAGER          │
                                     │               │               │
                                     ▼               ▼               ▼
                                     └───────────────┼───────────────┘
                                                     │
                              ┌───────────────┬──────┴──────┬───────────────┐
                              │               │             │               │
                              ▼               ▼             ▼               ▼
                           REALM            P2P         STORAGE         ECLVM
```

### XXII.2 Trust × PackageManager

#### Feature: Trust-Weighted Discovery

**Ranking-Formel:**
$$\boxed{\text{score}(\pi) = 0.3 \cdot \tau_R + 0.2 \cdot \tau_\Omega + 0.2 \cdot \omega_{\text{align}} + 0.15 \cdot \log(\text{installs}) + 0.15 \cdot \nu}$$

| Faktor                    | Gewicht | Quelle                          |
| ------------------------- | ------- | ------------------------------- |
| $\tau_R$ (Publisher)      | 0.30    | TrustEngine                     |
| $\tau_\Omega$ (Publisher) | 0.20    | TrustEngine                     |
| $\omega_{\text{align}}$   | 0.20    | User-ω vs. Package-ω Similarity |
| $\log(\text{installs})$   | 0.15    | Registry Stats                  |
| $\nu$ (Novelty)           | 0.15    | Novelty-Engine                  |

#### Feature: Auto-Upgrade Policies

$$\text{AutoUpgrade}: \text{Realm} \times \text{Package} \times \text{Policy} \rightarrow \{\text{Upgrade}, \text{Skip}, \text{Notify}\}$$

| Policy               | Bedingung                                            | Aktion  |
| -------------------- | ---------------------------------------------------- | ------- |
| `security_only`      | $\text{vuln}(\pi) \land \exists \pi'_{\text{patch}}$ | Upgrade |
| `minor_updates`      | $v'.M = v.M \land v' \succ v$                        | Upgrade |
| `trusted_publishers` | $\tau_R(\text{pub}(\pi')) \geq 0.9$                  | Upgrade |
| `manual`             | —                                                    | Notify  |

### XXII.3 Identity × PackageManager

#### Feature: Verified Publisher Badges

$$
\text{Badge}(\iota) = \begin{cases}
  \text{🏛️ Verified Org} & \text{if } \exists \text{VC}: \text{type} = \texttt{org:verified} \\
  \text{🔒 Security-Audited} & \text{if } \exists \text{VC}: \text{issuer} \in \text{SecurityGuild} \\
  \text{⭐ Trusted} & \text{if } \tau_R \geq 0.9 \\
  \emptyset & \text{otherwise}
\end{cases}
$$

#### Feature: Package Signing (Sub-DIDs)

$$\text{sig}(\pi) = \text{Sign}_{\text{did:erynoa:self:}\iota\text{:packages:}\pi.\text{name}}(\text{hash}(\pi))$$

**Hierarchie:**
$$\text{RootDID} \xrightarrow{\text{delegate}} \text{PackagesDID} \xrightarrow{\text{sign}} \pi$$

#### Feature: Organization Packages

$$\text{OrgPackage} = \langle \text{did:erynoa:guild:org}, \mathcal{M}, \text{attestations} \rangle$$

**Trust-Propagation:**
$$\tau(\pi_{\text{org}}) = 0.9 \cdot \tau(\text{guild})$$

### XXII.4 Gas/Mana × PackageManager

#### Feature: Lazy Loading

$$
\text{LazyLoad}(\pi, \rho) = \begin{cases}
  \text{Load}(\pi.\text{ui}) & \text{sofort} \\
  \text{Load}(\pi.\text{logic}) & \text{bei erstem Handler-Call} \\
  \text{Load}(\pi.\text{stores}) & \text{bei erstem Store-Access}
\end{cases}
$$

**Gas-Ersparnis:**
$$\gamma_{\text{lazy}} \approx 0.3 \cdot \gamma_{\text{eager}} \quad \text{(typisch)}$$

#### Feature: Premium Packages (Mana-Monetization)

$$\text{Tier}(\pi) = \langle \text{name}, \text{features}, \text{requirements} \rangle$$

| Tier       | $\mu_{\text{install}}$ | $\mu_{\text{monthly}}$ | Features            |
| ---------- | ---------------------- | ---------------------- | ------------------- |
| Free       | 0                      | 0                      | Basic               |
| Pro        | 500                    | 100                    | + Advanced          |
| Enterprise | 5000                   | 2000                   | + Priority + Custom |

**DID-Gated Tiers:**
$$\text{Access}(\iota, \text{tier}) \iff \tau(\iota) \geq \theta_{\text{tier}} \land \text{attestations}(\iota) \supseteq \text{required}(\text{tier})$$

#### Feature: Mana-Bounded Resolution

$$\boxed{\text{resolve}(\mathcal{D}) \text{ aborts if } \mu_{\text{consumed}} > \mu_{\text{budget}}}$$

**Budget-Formel:**
$$\mu_{\text{budget}} = \mu_{\text{base}} + |\mathcal{D}| \cdot \mu_{\text{per\_dep}}, \quad \mu_{\text{base}} = 100, \mu_{\text{per\_dep}} = 10$$

### XXII.5 Realm × PackageManager

#### Feature: Realm Templates (Meta-Packages)

$$\text{Template} = \langle \text{name}, \{\pi_i\}_{i=1}^n, \text{governance}, \text{stores} \rangle$$

**Beispiel:**

```
Template "dao-starter" = {
    packages: [voting-system, treasury, membership],
    governance: QuadraticVoting,
    stores: [proposals, treasury-state]
}
```

**Installation:**
$$\text{realm.create\_from\_template}(T) \equiv \bigwedge_{i=1}^n \text{install}(\pi_i) \land \text{apply}(T.\text{governance})$$

#### Feature: Cross-Realm Package Sharing

$$\text{SharedPackage}(\pi, \rho_1, \rho_2) \iff \text{Content}(\pi, \rho_1) = \text{Content}(\pi, \rho_2)$$

**Deduplication:**
$$\text{StorageUsage}(\pi, \{\rho_1, ..., \rho_n\}) = |\text{Content}(\pi)| \quad \text{(nicht } n \cdot |\text{Content}(\pi)|)$$

### XXII.6 P2P × PackageManager

#### Feature: Gossip-Based Registry

$$\text{Registry}_{\text{P2P}} = \bigcup_{\text{peer} \in \mathcal{P}} \text{LocalRegistry}(\text{peer})$$

**Sync-Invariante:**
$$\text{Eventually}: \forall \text{peer}_1, \text{peer}_2: \text{Registry}(\text{peer}_1) = \text{Registry}(\text{peer}_2)$$

**Gossip-Nachricht:**
$$\text{msg} = \langle \text{PackageId}, v, \text{publisher\_did}, \nu, \tau, h \rangle$$

#### Feature: Seeder Incentives

$$\boxed{\Delta\tau_I(\text{seeder}) = \min(0.01, n_{\text{packages}} \times 0.0001)}$$

**Trust-Boost für Seeders:**

- Pro geseedetes Package: +0.0001 Trust-I
- Maximum pro Periode: +0.01 Trust-I

#### Feature: Geo-Aware Download Priority

$$\text{priority}(\text{peer}) = \frac{0.4}{\text{latency}_{\text{ms}}} + 0.3 \cdot \tau_R(\text{peer}) + 0.2 \cdot \text{uptime} + 0.1 \cdot v_{\text{hist}}$$

### XXII.7 ECLVM × PackageManager

#### Feature: Hot-Reload (Zero-Downtime)

$$\text{HotReload}: \pi_{\text{old}} \rightarrow \pi_{\text{new}}$$

**3-Phasen-Protokoll:**

1. **Pre-Upgrade:** $\text{Load}(\pi_{\text{new}}) \parallel \text{Validate}(\pi_{\text{new}}) \parallel \text{ComputeMigration}()$
2. **Atomic Swap:** $\text{Pause}(\text{requests}) \rightarrow \text{Swap}(\text{ref}) \rightarrow \text{Resume}() \quad [<10\text{ms}]$
3. **Post-Upgrade:** $\text{Migrate}_{\text{async}}() \rightarrow \text{GC}(\pi_{\text{old}}) \rightarrow \text{Emit}(\text{PackageHotReloaded})$

**Precondition:**
$$\pi.\text{manifest.hot\_reloadable} = \text{true}$$

#### Feature: Sandboxed Testing

$$\text{Test}(\pi) = \text{ECLVM}_{\text{sandbox}}.\text{exec}(\pi.\text{tests})$$

**Test-Pipeline:**

1. Unit Tests (ECL-defined)
2. Integration Tests (Mock-Stores)
3. Gas-Profiling (max Gas pro Handler)
4. Coverage-Report

### XXII.8 Storage × PackageManager

#### Feature: Content-Addressed Deduplication

$$\boxed{\forall \pi_1, \pi_2: \text{Content}(\pi_1) = \text{Content}(\pi_2) \implies \text{StorageId}(\pi_1) = \text{StorageId}(\pi_2)}$$

**Vorteile:**

- Deduplication bei Forks
- Global Cache über alle Realms
- CDN-freundlich (immutable content)

#### Feature: Tiered Storage

$$
\text{Tier}(\pi) = \begin{cases}
  \text{HOT (SSD/Memory)} & \text{if frequently used} \lor \text{recently installed} \\
  \text{WARM (SSD)} & \text{if installed but idle} \\
  \text{COLD (Archive)} & \text{if old} \lor \text{deprecated}
\end{cases}
$$

### XXII.9 Security Features

#### Vulnerability Alert System

$$\text{VA} = \langle \text{package}, \text{severity}, \text{affected}, \text{patched}, \text{issuer} \rangle$$

**Propagation:**
$$\text{SecurityGuild} \xrightarrow{\text{VC}} \text{Gossip} \xrightarrow{\text{broadcast}} \text{AllPeers} \xrightarrow{\text{check}} \text{AffectedRealms}$$

**Auto-Block für Critical:**
$$\text{severity} = \text{critical} \implies \text{Block}(\text{affected\_versions}) \land \text{Notify}(\text{realms})$$

#### Publisher Trust-Penalty

$$\tau'_R(\text{pub}) = \tau_R(\text{pub}) - \lambda \cdot \text{severity\_weight} \quad \text{falls nicht gepatcht nach } T_{\text{grace}}$$

### XXII.10 Premium Features (10/10 Edition)

#### Universal Trust Identifier (UTI)

$$\boxed{\text{UTI}(\iota) = \text{BLAKE3}(\text{Canonical}(\text{DID}(\iota)))}$$

**Eigenschaften:**

- Chain-agnostisch
- Deterministisch
- Privacy-preserving

**UTI im Ranking:**
$$\text{score}'(\pi) = 0.35 \cdot \tau_{\text{UTI}} + 0.2 \cdot \tau_\Omega + 0.2 \cdot \omega_{\text{align}} + 0.15 \cdot \log(\text{installs}) + 0.1 \cdot \nu$$

#### WalletConnect V2 Auto-Connect

$$
\text{WCAutoConnect}(\pi, \rho) = \begin{cases}
  \text{ConnectExisting}() & \text{if session exists} \\
  \text{RequestSession}() & \text{otherwise}
\end{cases}
$$

**Manifest-Erweiterung:**

```ecl
walletconnect_v2: {
    required_chains: ["eip155:1", "eip155:137"],
    wc_auto_connect: true
}
```

#### Privacy-Preserving Analytics

$$\text{Analytics}(\pi) = \text{Aggregate}(\text{installs}, k=10, \varepsilon=0.1)$$

| Privacy-Garantie     | Parameter           |
| -------------------- | ------------------- |
| k-Anonymity          | $k \geq 10$         |
| Differential Privacy | $\varepsilon = 0.1$ |
| Data Retention       | 90 Tage aggregiert  |

### XXII.11 Package-Axiome (Erweiterung von $\mathcal{K}$)

| Axiom             | Formel                                                                                     |
| ----------------- | ------------------------------------------------------------------------------------------ |
| K_PkgTrust        | $\text{publish}(\pi) \implies \tau_R \geq 0.8 \land \tau_\Omega \geq 1.5$                  |
| K_PkgIntegrity    | $\text{PackageId} = \text{BLAKE3}(\text{Content})$                                         |
| K_PkgAcyclic      | $\neg \exists \pi: \pi \in \text{deps}^*(\pi)$                                             |
| K_PkgDeterminism  | $\text{Manifest}(t_1) = \text{Manifest}(t_2) \implies \text{Lock}(t_1) = \text{Lock}(t_2)$ |
| K_PkgIsolation    | $\forall \pi, \rho: \text{Access}(\pi) \subseteq \text{declared}(\pi.\text{stores})$       |
| K_PkgSeederReward | $\Delta\tau_I \leq 0.01$ pro Periode                                                       |

### XXII.12 Emergentes Haupttheorem (Package-Domain)

$$\boxed{\mathcal{PM} = \langle \pi, \text{resolve}, \text{install}, \mathcal{S}_{\text{PM}} \rangle \text{ ist trust-nativ, dezentral, und realm-isoliert}}$$

**Beweis-Sketch:**

1. **Trust-Nativität:** $\forall \text{op} \in \{\text{publish}, \text{install}, \text{discover}\}: \text{op} \text{ erfordert } \tau$-Prüfung
2. **Dezentralität:** Registry via Gossip, keine zentrale Autorität
3. **Realm-Isolation:** Packages in Sandbox, nur deklarierte Stores/Policies

---

## XXIII. Realm-Isolation-Algebra 🏰

> **Quelle:** 13-REALM-ARCHITEKTUR-ISOLATION.md

### XXIII.1 Realm-Definition

$$\boxed{\rho = \langle \text{id}, \text{parent}, \mathcal{R}_\rho, M, \mathcal{G}, \mathcal{Q}, \mathcal{I} \rangle}$$

| Komponente         | Typ                        | Beschreibung             |
| ------------------ | -------------------------- | ------------------------ |
| id                 | $H_{256}$                  | Realm-Identifikator      |
| parent             | $\rho?$                    | Optionaler Parent-Realm  |
| $\mathcal{R}_\rho$ | $\mathcal{P}(\text{Rule})$ | Regel-Menge (Κ1-konform) |
| $M$                | $\mathcal{P}(\iota)$       | Membership-Set           |
| $\mathcal{G}$      | GovernanceConfig           | Governance-Konfiguration |
| $\mathcal{Q}$      | RealmQuota                 | Self-Healing Quotas      |
| $\mathcal{I}$      | $\{0, 1, 2\}$              | Isolation-Level          |

### XXIII.2 Realm-Hierarchie

$$\mathcal{H}_\rho = (\mathcal{V}_\rho, \mathcal{E}_\rho), \quad \mathcal{E}_\rho = \{(\rho_c, \rho_p) \mid \rho_c.\text{parent} = \rho_p\}$$

**Hierarchie-Typen:**

| Typ          | Definition                                   | Beispiel                |
| ------------ | -------------------------------------------- | ----------------------- |
| RootRealm    | $\rho_0: \text{parent} = \emptyset$          | System-Root (28 Axiome) |
| VirtualRealm | $\rho_v: \text{parent} \neq \emptyset$       | EU-Realm, Gaming-Realm  |
| Partition    | $\rho_p: \text{shard\_index} \in \mathbb{N}$ | DE-Shard, FR-Shard      |

```
                              ┌──────────────────────┐
                              │     ROOT REALM       │
                              │   (28 Kern-Axiome)   │
                              │     Κ1 - Κ28         │
                              └──────────┬───────────┘
                                         │
            ┌────────────────────────────┼────────────────────────────┐
            │                            │                            │
            ▼                            ▼                            ▼
   ┌────────────────┐           ┌────────────────┐           ┌────────────────┐
   │ EU-Realm       │           │ Gaming-Realm   │           │ DAO-Realm      │
   │ +GDPR, +MiCA   │           │ +Fair-Play     │           │ +Token-Vote    │
   │ min_τ = 0.5    │           │ min_τ = 0.3    │           │ min_τ = 0.7    │
   └───────┬────────┘           └───────┬────────┘           └────────────────┘
           │                            │
    ┌──────┴──────┐              ┌──────┴──────┐
    ▼             ▼              ▼             ▼
┌──────────┐ ┌──────────┐  ┌──────────┐ ┌──────────┐
│ DE-Shard │ │ FR-Shard │  │ Shard-0  │ │ Shard-1  │
└──────────┘ └──────────┘  └──────────┘ └──────────┘
```

### XXIII.3 Axiom Κ1: Monotone Regelvererbung

$$\boxed{\forall \rho_c \subset \rho_p: \mathcal{R}_{\rho_c} \supseteq \mathcal{R}_{\rho_p}}$$

**Bedeutung:**

- Kind-Realms erben ALLE Regeln des Parent
- Regeln können nur HINZUGEFÜGT werden, nie entfernt
- Root-Realm enthält die 28 Kern-Axiome (unveränderlich)

**Validation:**
$$\text{validate}_{K1}(\rho_c, \rho_p) = \mathcal{R}_{\rho_p} \subseteq \mathcal{R}_{\rho_c}$$

### XXIII.4 Isolation-Level-Algebra

$$\mathcal{I} \in \{\text{PUBLIC}, \text{MEMBERS}, \text{STRICT}\}$$

| Level   | Code | Lese-Zugriff          | Join-Methode          | Crossing               |
| ------- | ---- | --------------------- | --------------------- | ---------------------- |
| PUBLIC  | 0    | Jeder (public Stores) | Gateway-Policy        | Erlaubt                |
| MEMBERS | 1    | Nur Members           | Invitation + Approval | Mit Member-Status      |
| STRICT  | 2    | Verschlüsselt (E2E)   | Multi-Vouching + KYC  | Blockiert (nur Bridge) |

**Formale Definition:**

$$
\text{access}(\iota, \rho, \text{op}) = \begin{cases}
  \top & \text{if } \mathcal{I}_\rho = 0 \lor \iota \in M_\rho \\
  \iota \in M_\rho & \text{if } \mathcal{I}_\rho = 1 \\
  \iota \in M_\rho \land \text{hasKey}(\iota, \rho) & \text{if } \mathcal{I}_\rho = 2
\end{cases}
$$

### XXIII.5 Governance-Typen (Κ21)

$$\mathcal{G} \in \{\text{Quadratic}, \text{Token}, \text{Reputation}, \text{Delegated}\}$$

| Typ        | Formel                                       | Beschreibung               |
| ---------- | -------------------------------------------- | -------------------------- |
| Quadratic  | $v = \lfloor\sqrt{\text{tokens}}\rfloor$     | Quadratisches Voting (Κ21) |
| Token      | $v = \text{tokens}$                          | 1 Token = 1 Vote           |
| Reputation | $v = \tau_R \cdot w_\text{base}$             | Trust-gewichtetes Voting   |
| Delegated  | $v = v_\text{own} + \sum v_\text{delegated}$ | Liquid Democracy           |

**Quadratisches Voting (Κ21):**
$$\boxed{\text{votes}(\iota) = \lfloor\sqrt{\text{tokens}(\iota)}\rfloor}$$

### XXIII.6 RealmSpecificState

$$\Sigma_\rho = \langle \vec{\tau}_\rho, M, \mathcal{P}, \mathcal{C}, \mathcal{S}, \mathcal{Q} \rangle$$

| Komponente        | Typ                   | Beschreibung                     |
| ----------------- | --------------------- | -------------------------------- |
| $\vec{\tau}_\rho$ | TrustVector6D         | Realm-aggregierter Trust         |
| $M$               | Membership-Strukturen | members, pending, banned, admins |
| $\mathcal{P}$     | Vec⟨ECLPolicy⟩        | Aktive Policies                  |
| $\mathcal{C}$     | Crossing-State        | in/out/denied, allow/blocklist   |
| $\mathcal{S}$     | Saga-State            | initiated, involved, failed      |
| $\mathcal{Q}$     | RealmQuota            | Mana/Storage-Budget              |

### XXIII.7 Self-Healing Quotas

$$\mathcal{Q} = \langle \mu_\text{budget}, \mu_\text{used}, r_\mu, s_\text{quota}, s_\text{used} \rangle$$

**Quota-Health:**
$$h_\mathcal{Q} = 1 - \frac{\mu_\text{used}}{\mu_\text{budget}}$$

**Throttling-Trigger:**
$$h_\mathcal{Q} < 0.2 \implies \text{Throttle}(\rho) \land \text{Emit}(\text{QuotaWarning})$$

**Self-Healing-Regeneration:**
$$\mu(t+1) = \min(\mu_\text{budget}, \mu(t) + r_\mu \cdot \Delta t)$$

### XXIII.8 Realm Storage

$$\Omega_\rho = \langle \text{meta}, \text{data}, \text{schemas} \rangle$$

**Key-Struktur:**

- Shared: `realm:{realm_id}:shared:store:{name}:{key}`
- Personal: `realm:{realm_id}:personal:{did}:store:{name}:{key}`

**Schema-Definition:**
$$\text{Schema} = \langle \text{name}, v, \text{fields}, \text{personal}, \text{max\_entries}, \text{indices} \rangle$$

### XXIII.9 LazyShardedRealmState (Skalierung)

$$\Sigma_\rho^\text{sharded} = \langle \text{shards}[n], \text{LRU}[n], \text{stats}[n], \text{loader} \rangle$$

**Performance-Charakteristiken:**

| Operation  | Komplexität | Bedingung          |
| ---------- | ----------- | ------------------ |
| Read       | $O(1)$      | Cache-Hit          |
| Write      | $O(1)$      | Lock-free          |
| Memory     | $O(k)$      | Nur aktive Realms  |
| Contention | $\approx 0$ | Unabhängige Realms |

---

## XXIV. Realm-Synergien & Cross-Realm-Operationen 🏰

> **Quelle:** 13-REALM-ARCHITEKTUR-ISOLATION.md

### XXIV.1 Axiom Κ23: Realm-Crossing Trust-Dämpfung

$$\boxed{\tau_\text{eff}(\iota, \rho_B) = \tau(\iota, \rho_A) \cdot \phi_\text{cross}(\rho_A, \rho_B)}$$

**Crossing-Factor:**

$$
\phi_\text{cross}(\rho_A, \rho_B) = \begin{cases}
  1.0 & \text{if } \rho_B \in \text{Allowlist}(\rho_A) \\
  0.0 & \text{if } \rho_B \in \text{Blocklist}(\rho_A) \\
  \alpha_\text{sibling} & \text{if } \text{parent}(\rho_A) = \text{parent}(\rho_B) \\
  \alpha_\text{cousin} & \text{if } \text{grandparent}(\rho_A) = \text{grandparent}(\rho_B) \\
  \alpha_\text{foreign} & \text{otherwise}
\end{cases}
$$

| Beziehung | $\alpha$ | Beispiel                   |
| --------- | -------- | -------------------------- |
| Allowlist | 1.0      | Vertrauenswürdiger Partner |
| Sibling   | 0.8      | Gleicher Parent-Realm      |
| Cousin    | 0.6      | Gleicher Grandparent       |
| Foreign   | 0.4      | Keine Verwandtschaft       |
| Blocklist | 0.0      | Blockiert                  |

### XXIV.2 Axiom Κ24: Realm-lokaler Trust

$$\boxed{\forall \rho_1 \neq \rho_2: \vec{\tau}(\iota, \rho_1) \perp \vec{\tau}(\iota, \rho_2)}$$

**Bedeutung:**

- Jedes Realm hat eigenes TrustVector6D pro Identity
- Trust-Aktionen in Realm A beeinflussen NICHT Trust in Realm B
- Begrenzte Portabilität via Κ23-Dämpfung

**Ausnahme (Trust-Leak bei schweren Verstößen):**
$$\tau(\iota, \rho) < 0.1 \implies \text{CrossRealmWarning}(\iota)$$
$$\tau(\iota, \rho) = 0 \implies \text{GlobalBanMarker}(\iota)$$

### XXIV.3 Realm × Identity (Sub-DIDs)

$$\text{DID}_\rho(\iota) = \text{did:erynoa:circle:}\rho\text{-}\iota$$

**Hierarchie:**
$$\text{RootDID} \xrightarrow{\text{delegate}} \text{RealmDID}_1 \xrightarrow{\text{delegate}} \text{RealmDID}_2 \xrightarrow{\text{...}} \text{RealmDID}_n$$

**Wallet-Derivation pro Realm:**
$$\text{path}(\rho) = m/44'/\text{erynoa}'/0'/\text{realm}/\text{realm\_id}/0$$

**Privacy-Garantie:**
$$\nexists f: \text{RealmDID}_1 \xrightarrow{f} \text{RealmDID}_2 \quad \text{(ohne RootDID-Zugriff)}$$

### XXIV.4 Axiom Κ22: Saga-Pattern (Cross-Realm)

$$\text{Saga} = \langle \text{id}, \{\rho_i\}_{i=1}^n, \{s_j\}_{j=1}^m, \{c_j\}_{j=1}^m \rangle$$

| Komponente   | Typ               | Beschreibung          |
| ------------ | ----------------- | --------------------- |
| id           | String            | Saga-Identifikator    |
| $\{\rho_i\}$ | Set⟨RealmId⟩      | Beteiligte Realms     |
| $\{s_j\}$    | Vec⟨Step⟩         | Geordnete Schritte    |
| $\{c_j\}$    | Vec⟨Compensation⟩ | Compensation-Aktionen |

**Saga-Semantik:**

$$
\text{Saga.execute}() = \begin{cases}
  \text{Success} & \text{if } \forall j: s_j.\text{execute}() = \text{Ok} \\
  \text{Compensate}(k) & \text{if } s_k.\text{execute}() = \text{Err} \land \forall j < k: c_j.\text{execute}()
\end{cases}
$$

**Compensation-Garantie:**
$$\boxed{\text{Saga failed at step } k \implies \forall j < k: c_j \text{ executed}}$$

### XXIV.5 Realm × PackageManager

$$\text{InstalledPackages}(\rho) = \{(\pi, v, \text{overrides}) \mid \pi \in \text{registry} \land \text{installed}(\pi, \rho)\}$$

**Features:**

| Feature           | Formel                                                                              |
| ----------------- | ----------------------------------------------------------------------------------- | -------------------------------------------- | --- | ------------------- | --- |
| Realm-Isolation   | $\text{access}(\pi, \rho_1) \cap \text{access}(\pi, \rho_2) = \emptyset$            |
| Realm-Overrides   | $\text{policy}(\pi, \rho) = \text{merge}(\pi.\text{policy}, \rho.\text{overrides})$ |
| Cross-Realm-Dedup | $                                                                                   | \text{Storage}(\pi, \{\rho_1, ..., \rho_n\}) | =   | \text{Content}(\pi) | $   |
| Realm-Templates   | $\text{create}(\rho, T) = \bigwedge_{\pi \in T} \text{install}(\pi, \rho)$          |

### XXIV.6 Realm × Gas/Mana

$$\mathcal{Q}_\rho = \langle \mu_\text{budget}, \mu_\text{used}, r_\mu, s_\text{quota}, s_\text{used} \rangle$$

**Mana-Flow in Realm:**
$$\mu_\rho(t+1) = \min(\mu_\text{budget}, \mu_\rho(t) + r_\mu - \sum_{\text{op} \in \text{Ops}(t)} \mu(\text{op}))$$

**Throttling bei Low-Quota:**
$$\mu_\rho < 0.2 \cdot \mu_\text{budget} \implies \text{rate\_limit}(\rho) = 0.5 \cdot \text{rate\_limit}_\text{normal}$$

### XXIV.7 Realm × P2P (Gossip-Scoping)

**Realm-Topic:**
$$\text{Topic}(\rho) = \texttt{/erynoa/realm/}\rho\texttt{/events}$$

**Subscription-Regel:**
$$\text{subscribe}(\text{peer}, \text{Topic}(\rho)) \iff \text{peer}.\iota \in M_\rho$$

**Cross-Realm-Topics (eingeschränkt):**

- `/erynoa/cross-realm/sagas` → Saga-Koordination
- `/erynoa/cross-realm/announcements` → Öffentliche Announcements

**Peer-Discovery:**
$$\text{DHT\_Key}(\rho) = \texttt{/realm/}\rho\texttt{/peers}$$

### XXIV.8 Gateway-Policy-Algebra

$$\text{Gateway}(\rho) = \langle \text{requirements}, \text{verification}, \text{on\_join}, \mu_\text{cost} \rangle$$

**Join-Preconditions:**
$$\text{join}(\iota, \rho) \iff \tau_R(\iota) \geq \theta_R \land \tau_\Omega(\iota) \geq \theta_\Omega \land \text{verify}(\iota, \rho)$$

**Verification-Typen:**

$$
\text{verify}(\iota, \rho) = \begin{cases}
  \exists \text{VC}: \text{type} = \text{required\_attestation} & \text{(Attestation)} \\
  |\{v \mid v \in M_\rho \land \tau(v) \geq \theta_v\}| \geq k & \text{(Vouching)}
\end{cases}
$$

### XXIV.9 Realm-Discovery

$$\text{search}(q) = \text{rank}(\{\rho \mid \mathcal{I}_\rho \leq 1 \land \text{match}(\rho, q)\})$$

**Ranking-Formel:**
$$\text{rank}(\rho, q) = 0.3 \cdot \tau_\rho + 0.25 \cdot \log(|M_\rho|) + 0.25 \cdot \text{activity}(\rho) + 0.2 \cdot \text{relevance}(\rho, q)$$

### XXIV.10 StateEvents (Realm-Domain)

$$\mathcal{E}_\rho = \{\text{RealmCreated}, \text{MemberJoined}, \text{MemberLeft}, \text{RuleAdded}, \text{CrossingAttempted}, \text{SagaStarted}, ...\}$$

**Kategorien:**

| Kategorie  | Events                                               |
| ---------- | ---------------------------------------------------- |
| Lifecycle  | RealmCreated, RealmUpdated                           |
| Membership | MemberJoined, MemberLeft, MemberBanned, RoleChanged  |
| Rules      | RuleAdded, PolicyActivated, PolicyDeactivated        |
| Crossing   | CrossingAttempted, CrossingSucceeded, CrossingDenied |
| Saga       | SagaStarted, SagaStepCompleted, SagaCompleted        |
| Quota      | QuotaWarning, QuotaExceeded                          |

### XXIV.11 StateGraph-Integration

$$\rho \in \mathcal{V}(\text{StateGraph})$$

| Relation      | Target   | Semantik                    | Axiom |
| ------------- | -------- | --------------------------- | ----- |
| DependsOn     | Identity | Membership, Realm-Sub-DIDs  | K6    |
| DependsOn     | Trust    | Realm-lokaler Trust (Κ24)   | K24   |
| DependsOn     | Gas/Mana | Quotas, Self-Healing        | K13   |
| Aggregates    | Storage  | Realm-spezifische Stores    | K10   |
| Aggregates    | Packages | Installed Packages          | —     |
| Triggers      | Event    | Join, Leave, Crossing, Saga | K9    |
| Validates     | Rules    | Κ1 Monotone Vererbung       | K1    |
| Bidirectional | P2P      | Gossip, Realm-Topics        | —     |
| Bidirectional | ECLVM    | Policies, Governance        | —     |

### XXIV.12 Realm-Axiome (Erweiterung von $\mathcal{K}$)

| Axiom               | Formel                                                                               |
| ------------------- | ------------------------------------------------------------------------------------ |
| K1 (Monotonie)      | $\forall \rho_c \subset \rho_p: \mathcal{R}_{\rho_c} \supseteq \mathcal{R}_{\rho_p}$ |
| K21 (Quadratic)     | $\text{votes}(\iota) = \lfloor\sqrt{\text{tokens}(\iota)}\rfloor$                    |
| K22 (Saga)          | $\text{Saga failed} \implies \text{Compensations executed}$                          |
| K23 (Crossing)      | $\tau_\text{eff} = \tau \cdot \phi_\text{cross}$                                     |
| K24 (Lokaler Trust) | $\vec{\tau}(\iota, \rho_1) \perp \vec{\tau}(\iota, \rho_2)$                          |
| K_RealmIsolation    | $\text{effects}(\rho) \subseteq \text{boundary}(\rho)$                               |
| K_RealmQuota        | $\mu_\rho(t) \leq \mu_\text{budget}$                                                 |

### XXIV.13 Emergentes Haupttheorem (Realm-Domain)

$$\boxed{\mathcal{R}_{\text{Domain}} = \langle \rho, \mathcal{H}_\rho, \mathcal{I}, \mathcal{G}, \text{Saga} \rangle \text{ ist souverän, hierarchisch, und saga-sicher}}$$

**Beweis-Sketch:**

1. **Souveränität:** $\forall \rho: \text{eigene Regeln}, \text{Trust}, \text{Governance}, \text{Quotas}$
2. **Hierarchie:** $\mathcal{H}_\rho$ ist DAG mit Root und Κ1-Monotonie
3. **Saga-Sicherheit:** $\forall \text{Saga}: \text{Compensation-Garantie via Κ22}$

---

**∎ Q.E.D. (Phase 3)**

---

## §XXV Agent-Shell-Algebra

> _Quelle: 18-AGENT-SHELL-ZUGRIFF.md — Integration Phase 4_

### XXV.1 Shell-Tupel Definition

$$\boxed{\text{Shell} = \langle \text{AgentDID}, \mathcal{C}, \text{Context}, \vec{\tau} \rangle}$$

| Komponente    | Typ             | Beschreibung                  |
| ------------- | --------------- | ----------------------------- |
| AgentDID      | DID             | Identität des Agenten         |
| $\mathcal{C}$ | Set⟨Capability⟩ | Delegierte Shell-Capabilities |
| Context       | ShellContext    | Realm, Pfade, Umgebung        |
| $\vec{\tau}$  | TrustVector6D   | Effektiver Trust des Agenten  |

### XXV.2 Capability-Algebra

$$\mathcal{C}_\text{Shell} = \{\text{FullShell}, \text{Restricted}, \text{PathAccess}, \text{Service}, \text{Container}, \text{Scheduled}, \text{Network}, \text{Package}\}$$

**Capability-Hierarchie (partiell geordnet):**
$$\text{FullShell} \succ \text{Service} \succ \text{Container} \succ \text{Restricted}$$
$$\text{FullShell} \succ \text{Network} \succ \text{Package}$$
$$\text{FullShell} \succ \text{Scheduled}$$

**Capability-Implikation:**
$$c_1 \succ c_2 \implies \text{grant}(a, c_1) \Rightarrow \text{grant}(a, c_2)$$

**Capability-Typen:**

| Capability         | Operationen                       | Trust-Threshold   |
| ------------------ | --------------------------------- | ----------------- |
| FullShell          | Alle                              | Ω ≥ 0.95, R ≥ 0.9 |
| RestrictedCommands | {cmd₁, cmd₂, ...} ⊂ Allowlist     | Ω ≥ 0.5           |
| PathAccess         | read/write auf Pfad-Pattern       | Ω ≥ 0.6           |
| ServiceControl     | start/stop/restart Services       | Ω ≥ 0.7, R ≥ 0.7  |
| ContainerControl   | create/start/stop/exec Containers | Ω ≥ 0.7, C ≥ 0.7  |
| ScheduledTasks     | cron create/modify/delete         | Ω ≥ 0.7           |
| NetworkConfig      | firewall/routes/dns               | Ω ≥ 0.8, I ≥ 0.8  |
| PackageManagement  | apt/dnf install/remove            | Ω ≥ 0.8, I ≥ 0.8  |

### XXV.3 Trust-Threshold-Axiom

$$\boxed{\text{action}(a) \iff \vec{\tau}(a) \geq \vec{\theta}_\text{action}}$$

**Formalisierung:**
$$\text{authorize}(a, \text{op}) = \bigwedge_{d \in \mathcal{D}} \tau_d(a) \geq \theta_d(\text{op})$$

wobei $\mathcal{D} = \{R, I, C, P, V, \Omega\}$ die 6 Trust-Dimensionen sind.

**Trust-Decay bei Delegation:**
$$\vec{\tau}_\text{delegated} = \vec{\tau}_\text{owner} \times \phi_\text{delegation}$$

mit $\phi_\text{delegation} \in [0.3, 0.9]$ je nach Delegations-Tiefe.

### XXV.4 Sandbox-Layer-Modell

$$\text{Sandbox} = \langle \mathcal{N}, \mathcal{S}, \mathcal{G}, \mathcal{M} \rangle$$

| Layer               | Symbol        | Technik           | Funktion                    |
| ------------------- | ------------- | ----------------- | --------------------------- |
| Namespace Isolation | $\mathcal{N}$ | nsjail/bubblewrap | PID/Net/Mount/IPC Isolation |
| Seccomp Filtering   | $\mathcal{S}$ | seccomp-bpf       | Syscall-Allowlist           |
| Resource Limits     | $\mathcal{G}$ | cgroups v2        | CPU/RAM/IO Limits           |
| Capability Mounts   | $\mathcal{M}$ | bind-mounts       | Nur erlaubte Pfade sichtbar |

**Sandbox-Invariante:**
$$\boxed{\forall \text{cmd} \in \text{Sandbox}: \text{effects}(\text{cmd}) \subseteq \text{boundary}(\mathcal{C})}$$

**nsjail-Konfiguration (formalisiert):**
$$\text{nsjail}(\mathcal{C}) = \{\text{clone\_newpid}, \text{clone\_newnet}, \text{clone\_newns}\} \cup \text{mounts}(\mathcal{C})$$

### XXV.5 Command-Validation-Funktor

$$\text{validate}: \text{Command} \times \mathcal{C} \to \{\text{Allow}, \text{Deny}\}$$

**Validation-Logik:**

$$
\text{validate}(\text{cmd}, \mathcal{C}) = \begin{cases}
  \text{Allow} & \text{if } \text{FullShell} \in \mathcal{C} \\
  \text{Allow} & \text{if } \text{cmd} \in \text{allowlist}(\mathcal{C}) \\
  \text{Deny}  & \text{otherwise}
\end{cases}
$$

**Path-Validation:**
$$\text{path\_valid}(p, \mathcal{C}) = \exists \text{PathAccess}(\text{pattern}, \text{mode}) \in \mathcal{C}: p \sim \text{pattern}$$

### XXV.6 Trust-Impact-Funktor

$$\Delta\vec{\tau}: \text{ShellOp} \to \mathbb{R}^6$$

**Trust-Delta nach Operation:**

| Operation              | $\Delta R$ | $\Delta I$ | $\Delta C$ | $\Delta P$ | $\Delta V$ | $\Delta \Omega$ |
| ---------------------- | ---------- | ---------- | ---------- | ---------- | ---------- | --------------- |
| Successful Read        | +0.001     | 0          | 0          | 0          | 0          | +0.0005         |
| Successful Write       | +0.002     | +0.001     | 0          | 0          | 0          | +0.001          |
| Service Managed        | +0.003     | 0          | +0.002     | 0          | 0          | +0.002          |
| Failed Command         | -0.01      | 0          | -0.005     | 0          | 0          | -0.005          |
| Policy Violation       | -0.05      | -0.05      | -0.03      | 0          | 0          | -0.03           |
| Sandbox Escape Attempt | -0.5       | -0.5       | -0.5       | -0.3       | -0.5       | -0.5            |

**Temporale Decay:**
$$\vec{\tau}(t+1) = \vec{\tau}(t) + \Delta\vec{\tau} \cdot e^{-\lambda(t - t_\text{op})}$$

### XXV.7 Audit-Trail-Algebra

$$\text{AuditEvent} = \langle \text{id}, t, \iota, \text{cmd}, \text{result}, \text{context}, \Delta\vec{\tau} \rangle$$

**Unveränderlichkeits-Invariante:**
$$\boxed{\forall e \in \text{AuditLog}: \neg\exists f: f(\text{AuditLog}) \setminus \{e\} = \text{AuditLog}'}$$

**Retention-Policy:**
$$\text{retain}(e) \iff t_\text{now} - t(e) < \text{retention\_period}(\text{severity}(e))$$

### XXV.8 Axiom Κ25: Shell-Sandbox-Garantie

$$\boxed{\forall a \in \text{Agents}, \text{op} \in \text{ShellOps}: \text{exec}(\text{op}, a) \implies \text{sandboxed}(\text{op}) \land \text{logged}(\text{op})}$$

**Korollar:**
$$\text{escape}(\text{Sandbox}) \implies \tau(a) := 0 \land \text{GlobalBanMarker}(a)$$

---

## §XXVI Agent-Synergien und Compute-Algebra

> _Quelle: 18-AGENT-SHELL-ZUGRIFF.md (AI-Agent, Host-Crossing, KV-Store, Compute) — Integration Phase 4_

### XXVI.1 AI-Agent-DID-Schema

$$\text{DID}_\text{AI} = \texttt{did:erynoa:agent:ai:}\langle\text{model}\rangle\texttt{:}\langle\text{instance}\rangle$$

**Beispiele:**

- `did:erynoa:agent:ai:claude:analyst-001`
- `did:erynoa:agent:ai:gpt4:coder-alpha`

**AI-Agent-Typen:**
$$\mathcal{T}_\text{AI} = \{\text{Conversational}, \text{Coder}, \text{Analyst}, \text{Orchestrator}, \text{Monitor}, \text{Custom}\}$$

### XXVI.2 Trust-Dampening für AI-Agenten

$$\boxed{\vec{\tau}_\text{AI} = \vec{\tau}_\text{owner} \times \phi_\text{AI}}$$

wobei $\phi_\text{AI} = 0.6$ der Standard-Dämpfungsfaktor ist.

**Trust-Obergrenze:**
$$\tau_\Omega(\text{AI}) \leq 0.8 \cdot \tau_\Omega(\text{owner})$$

**Trust-Vererbung bei Realm-Beitritt:**
$$\tau_\text{AI}(\rho) = \min(\tau_\text{owner}(\rho) \times \phi_\text{AI}, \tau_\text{voucher}(\rho))$$

### XXVI.3 Realm-Membership für AI-Agenten

$$\text{RealmMembership}_\text{AI} = \langle \rho, \text{DID}_\text{AI}, \text{role}, \mathcal{C}, \text{expires} \rangle$$

**Multi-Realm-Algebra:**
$$\text{Realms}(\text{AI}) = \{\rho \mid \exists m \in \text{Memberships}: m.\rho = \rho \land m.\text{expires} > t_\text{now}\}$$

**Kapazitäts-Constraint:**
$$|\text{Realms}(\text{AI})| \leq \text{max\_realms}(\text{owner})$$

### XXVI.4 Host-Crossing-Erweiterung (Κ23+)

$$\text{Goal}_\text{HostCrossing} = \langle \text{source}, \text{target\_host}, \text{operation}, \text{saga} \rangle$$

**Erweiterung von Κ23:**
$$\text{Crossing}_\text{Host}(\rho \to \mathcal{H}) = \tau_\text{eff} \cdot \phi_\text{host}$$

wobei $\mathcal{H}$ ein physischer Host und $\phi_\text{host} \in [0.5, 0.9]$.

**Host-Crossing-Saga:**

$$
\text{Saga}_\text{Host} = \langle
  \text{PermissionCheck},
  \text{Lock},
  \text{Transfer},
  \text{Execute},
  \text{Verify},
  \text{Unlock}
\rangle
$$

**Compensation-Mapping:**
$$c(\text{Execute}) = \text{Rollback}$$
$$c(\text{Lock}) = \text{Unlock}$$

### XXVI.5 KV-Store-Access-Algebra

$$\text{KVCapability} = \langle \text{store\_pattern}, \text{key\_pattern}, \mathcal{O}, \text{personal\_only}, \text{rate\_limit} \rangle$$

**Operationen:**
$$\mathcal{O}_\text{KV} = \{\text{Read}, \text{Write}, \text{Delete}, \text{List}, \text{Watch}\}$$

**Access-Prädikat:**
$$\text{access}(a, s, k, \text{op}) \iff \exists c \in \mathcal{C}(a): s \sim c.\text{store} \land k \sim c.\text{key} \land \text{op} \in c.\mathcal{O}$$

**Personal-Only-Constraint:**
$$c.\text{personal\_only} \implies k \sim a.\text{did} \texttt{:*}$$

**Trust-Thresholds für KV:**

| Operation     | $\theta_\Omega$ | Zusätzliche Constraints                |
| ------------- | --------------- | -------------------------------------- |
| Read          | 0.3             | —                                      |
| List          | 0.3             | —                                      |
| Watch         | 0.4             | —                                      |
| Write         | 0.5             | $\theta_I \geq 0.5$                    |
| Delete        | 0.6             | $\theta_I \geq 0.6, \theta_R \geq 0.6$ |
| Schema-Modify | 0.8             | Schema-Write-Capability                |

### XXVI.6 Compute-Marketplace-Algebra

$$\text{ComputeOffer} = \langle \text{provider}, \mathcal{K}, \text{price}, \vec{\tau}, \mathcal{T}_\text{tasks}, \text{expires} \rangle$$

wobei $\mathcal{K}$ die Kapazität (CPU, RAM, GPU) beschreibt.

**Compute-Intent:**
$$\text{ComputeIntent} = \langle \text{requester}, \text{task}, \text{requirements}, \text{budget}, \text{deadline} \rangle$$

**Matching-Prädikat:**
$$\text{match}(I, O) \iff \mathcal{K}(O) \geq \text{requirements}(I) \land \text{price}(O) \leq \text{budget}(I) \land \vec{\tau}(O) \geq \vec{\theta}_\text{min}$$

### XXVI.7 Compute-Task-Typen

$$\mathcal{T}_\text{Compute} = \{\text{WASM}, \text{Container}, \text{MLInference}, \text{MapReduce}, \text{Script}\}$$

**Trust-Requirements pro Typ:**

| Task-Typ    | $\theta_\Omega$ | $\theta_R$ | $\theta_C$ | Execution-Env  |
| ----------- | --------------- | ---------- | ---------- | -------------- |
| WASM        | 0.4             | 0.5        | 0.4        | WASM-Sandbox   |
| Container   | 0.5             | 0.6        | 0.5        | Docker/nsjail  |
| MLInference | 0.5             | 0.6        | 0.6        | GPU-Sandbox    |
| MapReduce   | 0.5             | 0.6        | 0.5        | Distributed    |
| Script      | 0.9             | 0.9        | 0.8        | Trusted Native |

### XXVI.8 Compute-Saga-Composition

$$
\text{Saga}_\text{Compute} = \langle
  \text{Match},
  \text{Select},
  \text{LockPayment},
  \text{TransferTask},
  \text{Execute},
  \text{Verify},
  \text{ReleasePayment},
  \text{TrustUpdate}
\rangle
$$

**Selection-Strategy:**
$$\text{score}(O) = w_\tau \cdot \tau_\Omega(O) + w_p \cdot (1 - \frac{\text{price}(O)}{\text{max\_price}}) + w_k \cdot \frac{\mathcal{K}(O)}{\mathcal{K}_\text{max}}$$

mit Standard-Gewichten $w_\tau = 0.4, w_p = 0.3, w_k = 0.3$.

**Atomic-Compensation:**
$$\text{fail}(\text{Execute}) \implies c(\text{LockPayment}) \land c(\text{TransferTask})$$

### XXVI.9 Trust-Update nach Compute

$$
\Delta\vec{\tau}_\text{provider} = \begin{cases}
  (+0.01, 0, +0.01, 0, 0, +0.005) & \text{success} \\
  (-0.05, -0.02, -0.03, 0, 0, -0.03) & \text{failure} \\
  (-0.2, -0.1, -0.2, 0, 0, -0.15) & \text{fraud\_detected}
\end{cases}
$$

### XXVI.10 Security-Layer-Stack

$$\text{SecurityStack} = \langle \mathcal{L}_1, \mathcal{L}_2, \mathcal{L}_3, \mathcal{L}_4, \mathcal{L}_5, \mathcal{L}_6, \mathcal{L}_7 \rangle$$

| Layer | Symbol          | Komponente        | Funktion                           |
| ----- | --------------- | ----------------- | ---------------------------------- |
| 1     | $\mathcal{L}_1$ | Identity (DID)    | Kryptografische Identität          |
| 2     | $\mathcal{L}_2$ | Trust (6D)        | Trust muss verdient werden         |
| 3     | $\mathcal{L}_3$ | Capabilities (Κ8) | Explizite Delegation mit Decay     |
| 4     | $\mathcal{L}_4$ | Policies (ECL)    | Declarative Regeln                 |
| 5     | $\mathcal{L}_5$ | Saga (Κ22-24)     | Atomic Transactions + Compensation |
| 6     | $\mathcal{L}_6$ | Sandbox           | Kernel-Level Isolation             |
| 7     | $\mathcal{L}_7$ | Audit Trail       | Unveränderliche Logs               |

**Sicherheits-Invariante:**
$$\boxed{\forall \text{op}: \text{allowed}(\text{op}) \iff \bigwedge_{i=1}^{7} \mathcal{L}_i.\text{check}(\text{op}) = \text{Pass}}$$

### XXVI.11 Axiom Κ26: AI-Agent-Trust-Ceiling

$$\boxed{\forall a \in \text{AIAgents}: \tau_\Omega(a) \leq 0.8 \cdot \tau_\Omega(\text{owner}(a))}$$

**Korollar (Transitive Dämpfung):**
$$\text{AI}_1 \xrightarrow{\text{delegate}} \text{AI}_2 \implies \tau(\text{AI}_2) \leq 0.64 \cdot \tau(\text{owner})$$

### XXVI.12 Axiom Κ27: Compute-Atomicity

$$\boxed{\forall \text{Saga}_\text{Compute}: \text{fail}(s_k) \implies \bigwedge_{j < k} c_j.\text{executed}()}$$

Dies ist eine Spezialisierung von Κ24 für Compute-Sagas.

### XXVI.13 StateGraph-Integration (Agent-Shell-Domain)

$$\{\text{Shell}, \text{AI-Agent}, \text{KV-Access}, \text{Compute}\} \subset \mathcal{V}(\text{StateGraph})$$

| Node      | Relations                                                    | Axiome   |
| --------- | ------------------------------------------------------------ | -------- |
| Shell     | Aggregates Capabilities, Validates via Trust, Triggers Audit | Κ8, Κ25  |
| AI-Agent  | DependsOn Owner, Aggregates Memberships, Validates via Κ26   | Κ26      |
| KV-Access | DependsOn Realm, Validates via Trust, Triggers Events        | Κ24      |
| Compute   | Aggregates Offers, Composes Saga, Validates via Κ27          | Κ22, Κ27 |

### XXVI.14 Emergentes Haupttheorem (Agent-Shell-Domain)

$$\boxed{\mathcal{A}_{\text{Shell}} = \langle \text{Agents}, \mathcal{C}, \text{Sandbox}, \text{Trust}, \text{Saga} \rangle \text{ ist capability-basiert, trust-bounded, und saga-sicher}}$$

**Beweis-Sketch:**

1. **Capability-basiert:** Jede Operation erfordert explizite Capability via Κ8
2. **Trust-bounded:** AI-Agents durch Κ26 nach oben begrenzt, Trust-Thresholds via XXV.3
3. **Saga-sicher:** Alle Cross-Domain-Operationen via Saga mit Κ22, Κ24, Κ27

---

**∎ Q.E.D. (Phase 4)**

---

## §XXVII Synergistische System-Integration

> _Quelle: 07-SYNERGISTISCHE-INTEGRATION.md — Integration Phase 5_

### XXVII.1 Nervensystem-Metapher

$$\boxed{\mathbb{N}_{\text{Erynoa}} = \langle \text{Gehirn}, \text{Synapsen}, \text{Muskeln}, \text{Immunsystem}, \text{Gedächtnis}, \text{Nervenbahnen} \rangle}$$

| Organ        | Symbol | Komponente   | Funktion                    |
| ------------ | ------ | ------------ | --------------------------- |
| Gehirn       | 🧠     | UnifiedState | Zentrale Koordination       |
| Synapsen     | 🔌     | SynapseHub   | Signalübertragung           |
| Muskeln      | ⚙️     | Engines      | Ausführung                  |
| Immunsystem  | 🛡️     | Protection   | Schutz                      |
| Gedächtnis   | 💾     | Storage      | Persistenz                  |
| Nervenbahnen | 🌐     | P2P          | Kommunikation               |
| Organe       | 🏛️     | Realm        | Isolation & Spezialisierung |
| DNA          | 📜     | Domain       | Typen & Invarianten         |

### XXVII.2 StateComponent-Algebra

$$\mathcal{C}_\text{State} = \bigcup_{l \in \{Core, Engine, Peer, Storage, Protection\}} \mathcal{C}_l$$

**37 StateComponents nach Layer:**

| Layer      | Komponenten                                                           | Kritikalität |
| ---------- | --------------------------------------------------------------------- | ------------ |
| Core       | Identity, Trust, Event, Formula, Consensus                            | 🔴 Kritisch  |
| Execution  | Gas, Mana, Execution                                                  | 🟡 Hoch      |
| Engine     | ECLVM, ECLPolicy, ECLBlueprint, UI, API, Governance, Controller       | 🟡 Mittel    |
| Protection | Anomaly, Diversity, Quadratic, AntiCalcification, Calibration         | 🔴 Kritisch  |
| Peer       | Realm, Gateway, SagaComposer, IntentParser, Room                      | 🔴 Kritisch  |
| P2P        | Swarm, Gossip, DHT, Relay, Privacy, TrustGate                         | 🟡 Hoch      |
| Storage    | Storage, EventStore, IdentityStore, TrustStore, ContentStore, Archive | 🔴 Kritisch  |

### XXVII.3 Observer-Trait-Algebra

$$\text{Observer}: \text{StateEvent} \to \text{StateTransition}$$

**Observer-Kategorien:**

| Trait              | Events                                            | Reaktion                        |
| ------------------ | ------------------------------------------------- | ------------------------------- |
| TrustObserver      | TrustUpdate, IdentityBootstrapped, TrustViolation | Trust-State aktualisieren       |
| EventObserver      | EventAdded, EventFinalized                        | Event-Log aktualisieren         |
| ProtectionObserver | AnomalyDetected, EntropyUpdate                    | CircuitBreaker triggern         |
| RealmObserver      | CrossingSucceeded, RealmRegistered                | Realm-Graph aktualisieren       |
| StorageObserver    | EventPersisted, ArchiveCompleted                  | Speicher-Metriken aktualisieren |
| P2PObserver        | PeerConnectionChange, NetworkMetricUpdate         | Netzwerk-State aktualisieren    |

**Dispatch-Semantik:**
$$\text{Hub.dispatch}(e) = \bigcup_{c \in \text{affected}(e)} \{o(e) \mid o \in \text{observers}(c)\}$$

### XXVII.4 StateRelation-Typen

$$\mathcal{R}_\text{State} = \{\text{DependsOn}, \text{Triggers}, \text{Aggregates}, \text{Validates}, \text{Bidirectional}\}$$

**Semantik:**

| Relation      | Notation              | Bedeutung                        | Beispiel          |
| ------------- | --------------------- | -------------------------------- | ----------------- |
| DependsOn     | $A \to B$             | A benötigt B für Initialisierung | Gateway → Trust   |
| Triggers      | $A \Rightarrow B$     | A-Update löst B-Update aus       | Trust → Quota     |
| Aggregates    | $A \supseteq B$       | A enthält/aggregiert B           | Realm ⊇ Storage   |
| Validates     | $A \vdash B$          | A validiert B                    | Policy ⊢ Crossing |
| Bidirectional | $A \leftrightarrow B$ | Gegenseitige Abhängigkeit        | P2P ↔ ECLVM       |

### XXVII.5 Event-Kaskaden-Modell

$$\text{Cascade}(e_0) = \{e_0\} \cup \bigcup_{c \in \text{triggered}(e_0)} \text{Cascade}(\text{emit}(c, e_0))$$

**Beispiel Trust-Update-Kaskade:**

$$
\text{TrustUpdate}(\iota, +0.1) \xrightarrow{\text{Triggers}}
\begin{cases}
\text{GatewayRecalc} & \text{(neue Realms)} \\
\text{QuotaUpdate} & \text{(mehr Quota)} \\
\text{ECLVMBudget} & \text{(mehr Gas/Mana)}
\end{cases}
$$

### XXVII.6 Adapter-Pattern-Algebra

$$\text{Adapter}: \text{Engine} \to \text{StateComponent}$$

**Adapter-Interface:**
$$\text{Adapter} = \langle \text{component}, \text{init}, \text{on\_event}, \text{health\_score} \rangle$$

**Adapter-Mapping:**

| Engine          | Adapter            | StateComponent |
| --------------- | ------------------ | -------------- |
| TrustEngine     | TrustEngineAdapter | Trust          |
| EventEngine     | EventEngineAdapter | Event          |
| AnomalyDetector | AnomalyAdapter     | Anomaly        |
| GatewayGuard    | GatewayAdapter     | Gateway        |
| SwarmManager    | P2PAdapter         | Swarm          |

### XXVII.7 StateIntegrator-Fassade

$$\text{Integrator} = \langle \Sigma, \text{Hub}, \text{Adapters} \rangle$$

**Propagations-Algorithmus:**

$$
\text{propagate}(e) = \begin{cases}
\Sigma.\text{log\_and\_apply}(e) & \text{(1. State-Update)} \\
\text{Hub.dispatch}(e) & \text{(2. Observer-Notification)} \\
\Sigma.\text{persist}() & \text{(3. falls persistent)}
\end{cases}
$$

### XXVII.8 Metriken-Algebra

$$\mathcal{M}_\text{Synapse} = \langle \mu_\text{latency}, \mu_\text{notification}, \mu_\text{traversal}, \mu_\text{memory} \rangle$$

**Performance-Ziele:**

| Metrik               | Aktuell | Phase 4  | Phase 6 |
| -------------------- | ------- | -------- | ------- |
| Event-Dispatch       | 100 µs  | 50 µs    | 30 µs   |
| Observer-Notify      | sync    | async    | batch   |
| StateGraph-Traversal | O(n)    | O(log n) | O(1)    |
| Memory-Footprint     | 100 MB  | 80 MB    | 60 MB   |

### XXVII.9 Axiom Κ28: Synapse-Konsistenz

$$\boxed{\forall e \in \text{StateEvents}: \text{dispatch}(e) \implies \text{consistent}(\Sigma)}$$

**Korollar (Eventual Consistency):**
$$\lim_{t \to \infty} \Sigma(t) = \Sigma_\text{final} \quad \text{(alle Kaskaden terminieren)}$$

---

## §XXVIII Dezentraler Storage & Use-Case-Algebra

> _Quelle: 19-USE-CASES-DEZENTRALER-STORAGE.md — Integration Phase 5_

### XXVIII.1 Blob-Store-Fundamentaldefinition

$$\boxed{\text{BlobStore} = \langle \text{CAS}, \text{Chunks}, \text{Compression}, \text{P2P}, \text{Trust}, \text{Mana} \rangle}$$

**Content-Addressable Storage (CAS):**
$$\text{BlobId} = \langle \text{BLAKE3}(\text{content}), \rho \rangle$$

**Chunk-Struktur:**
$$\text{Chunk} = \langle \text{index}, \text{data}[4..64\text{MB}], \text{hash}, \text{compression} \rangle$$

**Manifest für Multi-Chunk-Blobs:**
$$\text{Manifest} = \langle \text{root\_hash}, \text{size}, \{\text{chunk\_hashes}\}, t, \text{creator}, \text{mime} \rangle$$

### XXVIII.2 Realm-URL-Adressierung (Κ26+)

$$\text{URL}_\rho = \texttt{erynoa://}\rho\texttt{/store/}\langle\text{store}\rangle\texttt{/}\langle\text{key}\rangle[\texttt{?params}]$$

**URL-Komponenten:**

| Komponente | Beispiel                        | Bedeutung          |
| ---------- | ------------------------------- | ------------------ |
| Realm      | `docker-registry`               | Ziel-Realm         |
| Store      | `layers`, `manifests`, `tags`   | Store-Typ          |
| Key        | `sha256:abc123...`              | Content-Identifier |
| Params     | `?chunk=0-5`, `?version=latest` | Query-Parameter    |

### XXVIII.3 Kosten-Algebra für Blob-Operationen

$$\text{Cost}_\text{Blob} = \langle \mu_\text{upload}, \mu_\text{download}, \mu_\text{pin}, g_\text{delete} \rangle$$

**Kostenformeln:**

| Operation | Mana-Kosten                                          | Gas-Kosten | Trust-Minimum  |
| --------- | ---------------------------------------------------- | ---------- | -------------- |
| Upload    | $1.0 \cdot \text{size}_\text{MB}$                    | 0.1/chunk  | $\theta = 0.3$ |
| Download  | $0.1 \cdot \text{size}_\text{MB}$                    | 0          | $\theta = 0.0$ |
| Pin       | $0.01 \cdot \text{size}_\text{MB} \cdot \text{days}$ | 0          | $\theta = 0.5$ |
| Delete    | 0                                                    | 0.5        | $\theta = 0.7$ |

### XXVIII.4 Use-Case-Realm-Algebra

$$\text{UseCase} = \langle \rho, \mathcal{S}_\text{stores}, \mathcal{T}_\text{trust}, \mathcal{Q}_\text{mana}, \mathcal{G}_\text{governance} \rangle$$

**6 Haupt-Use-Cases:**

| Use Case          | Symbol | Store-Typen                               | Governance  |
| ----------------- | ------ | ----------------------------------------- | ----------- |
| Docker Registry   | 🐳     | layers, manifests, tags                   | Reputation  |
| AI Model Registry | 🤖     | base_models, deltas, metadata, benchmarks | Peer-Review |
| Social Media      | 🎬     | videos, images, audio, posts, comments    | Quadratic   |
| Game Assets       | 🎮     | models, textures, audio, shaders, bundles | Reputation  |
| Enterprise Vault  | 🔐     | artifacts, licenses, firmware, secrets    | Delegated   |
| Science Hub       | 🔬     | raw_data, processed, notebooks, reviews   | Peer-Review |

### XXVIII.5 Trust-Threshold-Matrix

$$\Theta_\text{UseCase} \in \mathbb{R}^{6 \times 5}$$

| Operation    | Docker | AI   | Social | Games | Vault | Science |
| ------------ | ------ | ---- | ------ | ----- | ----- | ------- |
| Join/Browse  | 0.3    | 0.2  | 0.0    | 0.0   | 0.9   | 0.3     |
| Read         | 0.3    | 0.2  | 0.0    | 0.2   | 0.85  | 0.4     |
| Write        | 0.6    | 0.7  | 0.4    | 0.5   | 0.95  | 0.6     |
| Delete       | 0.8    | 0.8  | 0.7    | 0.7   | N/A   | N/A     |
| Curate/Admin | 0.95   | 0.85 | 0.8    | 0.8   | 0.99  | 0.9     |

### XXVIII.6 Mana-Regenerations-Algebra

$$\mu_\rho(t+1) = \min(\mu_\rho(t) + \mu_\text{budget} \cdot r_\mu, \mu_\text{budget})$$

**Use-Case-spezifische Regeneration:**

| Use Case     | Daily Budget | Regen/h | Begründung                      |
| ------------ | ------------ | ------- | ------------------------------- |
| Docker       | 10,000       | 10%     | Häufige kleine Operationen      |
| AI Models    | 1,000,000    | 5%      | Seltene große Uploads           |
| Social Media | 50,000       | 20%     | Viele kleine Interaktionen      |
| Game Assets  | 100,000      | 15%     | Mittlere Frequenz               |
| Enterprise   | 10,000,000   | 1%      | Kritisch, langsame Regeneration |
| Science      | 500,000      | 10%     | Mittlere Frequenz               |

### XXVIII.7 Globale Deduplizierung

$$\boxed{\forall b_1, b_2 \in \text{Blobs}: \text{hash}(b_1) = \text{hash}(b_2) \implies \text{storage}(b_1) = \text{storage}(b_2)}$$

**Cross-Realm-Dedup:**

$$
\text{store}(b, \rho_1) = \text{store}(b, \rho_2) = \text{store}(b, \rho_3) \quad \text{(physisch 1×)}
$$

**ABER:** Zugriffskontrolle bleibt realm-spezifisch:
$$\text{access}(b, \iota, \rho) \iff \tau_\rho(\iota) \geq \theta_\rho$$

### XXVIII.8 P2P-Sync-Strategien

$$\text{SyncStrategy} = \langle n_\text{min\_peers}, \text{regions}, \text{protocol}, \text{bandwidth} \rangle$$

**Protokoll-Typen:**

| Protocol  | Eigenschaft         | Use Case                |
| --------- | ------------------- | ----------------------- |
| BitSwap   | Aggressiv, parallel | Docker, Games           |
| Streaming | Sequentiell         | AI Models (große Files) |
| Encrypted | E2E-verschlüsselt   | Enterprise Vault        |

### XXVIII.9 Agent-Shell für Blob-Operationen

$$\text{BlobAgent} = \langle \text{DID}, \mathcal{C}_\text{shell}, \rho, \text{workflow} \rangle$$

**Agent-Typen pro Use Case:**

| Agent                | Use Case   | Capabilities                                   |
| -------------------- | ---------- | ---------------------------------------------- |
| DockerRegistryAgent  | Docker     | Container, Logs, ScheduledTasks                |
| ModelCuratorAgent    | AI Models  | Container (GPU), PathAccess, ScheduledTasks    |
| ModerationAgent      | Social     | Container (ML), RestrictedCommands, PathAccess |
| AssetValidatorAgent  | Games      | Container, RestrictedCommands, PathAccess      |
| SecurityAuditAgent   | Enterprise | PathAccess (readonly), RestrictedCommands      |
| ReproducibilityAgent | Science    | Container, PackageManagement, PathAccess       |

### XXVIII.10 ECL-Policy-Pattern für Blob-Upload

$$\text{UploadPolicy} = \langle \theta_\text{trust}, \mu_\text{min}, \text{validations}, \text{rate\_limit}, \text{cost} \rangle$$

**Generisches Pattern:**

```
policy BlobUploadPolicy {
    require trust >= θ;
    require mana >= μ_min;
    require validations.all_pass();
    rate_limit: n per period;
    cost: {
        mana: size_mb × factor,
        gas: operation_weight
    };
}
```

### XXVIII.11 Identifier-Integration (DOI, OCI)

$$\text{ExternalId}: \text{Realm-URL} \leftrightarrow \text{External-Standard}$$

**Mappings:**

| Standard | Realm   | Mapping                                                               |
| -------- | ------- | --------------------------------------------------------------------- |
| OCI      | Docker  | `/v2/<name>/blobs/<digest>` ↔ `erynoa://docker/store/layers/<digest>` |
| DOI      | Science | `10.erynoa/<dataset>` ↔ `erynoa://science/store/raw_data/<dataset>`   |
| ORCID    | Science | Creator-Verification via DID                                          |

### XXVIII.12 Governance-Typen pro Use Case

$$\mathcal{G}_\text{UseCase} \in \{\text{Reputation}, \text{PeerReview}, \text{Quadratic}, \text{Delegated}\}$$

| Governance  | Mechanismus                           | Use Cases          |
| ----------- | ------------------------------------- | ------------------ |
| Reputation  | Trust-gewichtetes Voting              | Docker, Games      |
| Peer-Review | 2+ Reviewer mit $\tau > 0.8$          | AI Models, Science |
| Quadratic   | $\sqrt{\text{tokens}} = \text{votes}$ | Social Media       |
| Delegated   | Admin-Quorum (3/5)                    | Enterprise         |

### XXVIII.13 Security-Levels pro Use Case

$$\mathcal{L}_\text{Security} \in \{\text{Public}, \text{TrustGated}, \text{Encrypted}, \text{DoubleEncrypted}\}$$

| Level           | Eigenschaften                     | Use Cases              |
| --------------- | --------------------------------- | ---------------------- |
| Public          | Keine Verschlüsselung, öffentlich | Docker (layers), Games |
| TrustGated      | Trust-Check vor Zugriff           | Social, Science        |
| Encrypted       | Client-Side AES-256               | Enterprise (artifacts) |
| DoubleEncrypted | Realm + Client Encryption         | Enterprise (secrets)   |

### XXVIII.14 Axiom Κ29: Blob-Integrität

$$\boxed{\forall b \in \text{Blobs}: \text{stored}(b) \implies \text{BLAKE3}(b.\text{data}) = b.\text{id}.\text{hash}}$$

### XXVIII.15 Axiom Κ30: Realm-Speicher-Isolation

$$\boxed{\forall \rho_1, \rho_2: \text{policy}(b, \rho_1) \perp \text{policy}(b, \rho_2)}$$

Gleicher Blob, unterschiedliche Zugriffsregeln pro Realm.

### XXVIII.16 StateGraph-Integration (Blob-Domain)

$$\text{BlobStore} \in \mathcal{V}(\text{StateGraph})$$

| Relation      | Target | Semantik                             | Axiom |
| ------------- | ------ | ------------------------------------ | ----- |
| DependsOn     | Realm  | Store gehört zu Realm                | K1    |
| DependsOn     | Trust  | Zugriff benötigt Trust-Check         | K24   |
| DependsOn     | Mana   | Operationen kosten Mana              | K13   |
| Aggregates    | Chunks | Blob besteht aus Chunks              | —     |
| Triggers      | Event  | Upload/Download emittiert StateEvent | K9    |
| Validates     | Policy | ECL-Policy validiert Operation       | K23   |
| Bidirectional | P2P    | BitSwap/Gossip für Sync              | —     |

### XXVIII.17 Emergentes Haupttheorem (Synergien-Domain)

$$\boxed{\mathcal{S}_{\text{Pluto}} = \langle \text{Nervensystem}, \text{BlobStore}, \text{UseCases} \rangle \text{ ist event-driven, content-addressed, und trust-gated}}$$

**Beweis-Sketch:**

1. **Event-driven:** Alle State-Änderungen via SynapseHub mit Observer-Pattern (Κ28)
2. **Content-addressed:** Blobs via BLAKE3-CAS mit globaler Dedup (Κ29)
3. **Trust-gated:** Realm-spezifische Trust-Thresholds (Κ30)
4. **Use-Case-adaptiv:** 6 spezialisierte Realm-Konfigurationen

---

**∎ Q.E.D. (Phase 5)**

---

## Appendix C: IST-Zustand-Defizite 🔴

> **Quelle:** 01-IST-ANALYSE.md, 16.1-LEGACY-MEGA-REFACTORING-PLAN.md

### C.1 Fundamentale Defizit-Metrik

$$\boxed{\mathcal{D}_\text{IST} = \langle \Sigma_\text{state.rs}, \mathcal{R}_\text{redundant}, \mathcal{C}_\text{circular}, \mathcal{T}_\text{coverage} \rangle}$$

| Symbol                    | Metrik                     | IST-Wert | SOLL-Wert |
| ------------------------- | -------------------------- | -------- | --------- |
| $\Sigma_\text{state.rs}$  | Zeilen in state.rs         | 21,495   | ≤ 2,000   |
| $\mathcal{R}_\text{dups}$ | Redundante Patterns        | 8+       | 0         |
| $\mathcal{C}_\text{circ}$ | Zirkuläre Abhängigkeiten   | 5+       | 0         |
| $\mathcal{T}_\text{cov}$  | Test-Coverage (%)          | 60%      | ≥ 85%     |
| $\mathcal{M}_\text{size}$ | Durchschn. Dateigröße (KB) | 30       | ≤ 15      |

### C.2 Modul-Zerlegung von state.rs

$$\text{state.rs} = \bigsqcup_{i=1}^{12} M_i$$

| Zeilen        | Modul $M_i$               | Ziel-Datei                                | Kardinalität |
| ------------- | ------------------------- | ----------------------------------------- | ------------ |
| 1–800         | Infrastructure            | `nervous_system/infrastructure/`          | ~800 LOC     |
| 800–1,900     | StateEvent (42 Varianten) | `nervous_system/event_sourcing/`          | ~1,100 LOC   |
| 1,900–2,500   | Event-Sourcing-Core       | `nervous_system/event_sourcing/`          | ~600 LOC     |
| 2,500–3,000   | Merkle-Tracking           | `nervous_system/merkle/`                  | ~500 LOC     |
| 3,000–4,100   | Identity + StateGraph     | `nervous_system/graph/`, `identity/`      | ~1,100 LOC   |
| 4,100–6,000   | Core-States               | `nervous_system/components/core.rs`       | ~1,900 LOC   |
| 6,000–8,000   | Protection-States         | `nervous_system/components/protection.rs` | ~2,000 LOC   |
| 8,000–10,000  | Peer-States               | `nervous_system/components/peer.rs`       | ~2,000 LOC   |
| 10,000–12,000 | Engine-States             | `nervous_system/components/eclvm.rs`      | ~2,000 LOC   |
| 12,000–21,495 | UnifiedState + Tests      | `nervous_system/unified_state.rs`         | ~9,495 LOC   |

**Transformationsoperator:**

$$\Phi_\text{decompose}: \text{state.rs}_{21,495} \rightarrow \bigsqcup_{i=1}^{12} M_i \quad \text{s.t.} \quad \sum_i |M_i| \leq 5,000$$

### C.3 Redundanz-Katalog

$$\mathcal{R}_\text{redundant} = \{R_1, R_2, ..., R_8\}$$

| ID    | Pattern                    | Vorkommen | Konsolidierungs-Ziel       |
| ----- | -------------------------- | --------- | -------------------------- |
| $R_1$ | Snapshot-Pattern (doppelt) | 5         | `StateLayer::Snapshot`     |
| $R_2$ | Error-Types                | 8         | `ErynoaError` (unified)    |
| $R_3$ | Health-Score-Berechnung    | 4         | `StateLayer::health_score` |
| $R_4$ | Observer-Traits            | 30+       | `StateObserver` (single)   |
| $R_5$ | Config-Structs             | 12        | `domain/unified/config.rs` |
| $R_6$ | Serialization-Logic        | 6         | `serde` derive macros      |
| $R_7$ | Atomic-Counter-Pattern     | 15        | `AtomicMetrics` trait      |
| $R_8$ | Lock-Pattern (RwLock)      | 20        | `DashMap` (lock-free)      |

### C.4 Kritischer Pfad (IST-Analyse)

$$\text{CriticalPath}_\text{IST} = \text{Identity} \xrightarrow{\text{Auth}} \text{Trust} \xrightarrow{\text{Eval}} \text{Event} \xrightarrow{\text{Vote}} \text{Consensus} \xrightarrow{\text{Finalize}} \text{Finality}$$

**Latenz-Analyse:**

| Segment                                | IST-Latenz | SOLL-Latenz | Bottleneck          |
| -------------------------------------- | ---------- | ----------- | ------------------- |
| $\text{Identity} \to \text{Trust}$     | 5ms        | 1ms         | Key-Lookup          |
| $\text{Trust} \to \text{Event}$        | 2ms        | 0.5ms       | Trust-Aggregation   |
| $\text{Event} \to \text{Consensus}$    | 10ms       | 2ms         | Gossip-Latency      |
| $\text{Consensus} \to \text{Finality}$ | 50ms       | 10ms        | 2/3 Threshold       |
| **Total**                              | **67ms**   | **13.5ms**  | **5x Verbesserung** |

### C.5 Axiom Κ31: Defizit-Reduktion

$$\boxed{\forall \Phi_\text{refactor}: \mathcal{D}_\text{IST}' = \Phi_\text{refactor}(\mathcal{D}_\text{IST}) \implies |\mathcal{D}_\text{IST}'| < |\mathcal{D}_\text{IST}|}$$

Jede Refactoring-Operation muss Defizite strikt reduzieren.

---

## Appendix D: Phasenplan-Timeline 📅

> **Quelle:** 04-PHASENPLAN.md, 16.1-LEGACY-MEGA-REFACTORING-PLAN.md

### D.1 Phasen-Algebra

$$\boxed{\mathcal{P}_\text{Pluto} = \langle P_1, P_2, P_3, P_4, P_5, P_6 \rangle \quad \text{über} \quad T = 14 \text{ Wochen}}$$

| Phase | Name          | Wochen | Kern-Deliverables                                |
| ----- | ------------- | ------ | ------------------------------------------------ |
| $P_1$ | Foundation    | 1–2    | Verzeichnisstruktur, Traits, Error-Hierarchie    |
| $P_2$ | Decomposition | 3–5    | state.rs Aufspaltung, Module extrahieren         |
| $P_3$ | Synapse Hub   | 6–7    | Unified Observer, Adapter-Pattern, Bridges       |
| $P_4$ | Integration   | 8–9    | Engine-Layer Refactoring, P2P Konsolidierung     |
| $P_5$ | ECLVM→WASM    | 10–13  | Wasmtime Integration, WIT-Interface, Dual-Mode   |
| $P_6$ | Optimization  | 14     | Performance-Tuning, Memory-Footprint, Benchmarks |

### D.2 Wochen-Task-Mapping

$$W_k: \mathcal{P} \rightarrow \text{Tasks}$$

**Phase 1 (Foundation):**

| Woche | Tag   | Task                       | Deliverable                    |
| ----- | ----- | -------------------------- | ------------------------------ |
| W1    | Mo    | Verzeichnisse erstellen    | `nervous_system/`, `synapses/` |
| W1    | Di    | `nervous_system/traits.rs` | `StateLayer`, `StateObserver`  |
| W1    | Mi    | `domain/unified/error.rs`  | `ErynoaError` Hierarchie       |
| W1    | Do    | `ObserverPriority` enum    | Priority-Queue-Support         |
| W1    | Fr    | Compilation-Check          | `cargo check` ✓                |
| W2    | Mo–Fr | Tests + Dokumentation      | ADRs, Module-Docs              |

**Phase 5 (ECLVM→WASM):**

| Woche | Tag | Task                         | Deliverable                 |
| ----- | --- | ---------------------------- | --------------------------- |
| W11   | Mo  | Wasmtime zu `Cargo.toml`     | Feature-Flag `wasm`         |
| W11   | Di  | `eclvm/wasm/mod.rs` Struktur | Modul-Layout                |
| W11   | Mi  | WIT-Datei + wit-bindgen      | `erynoa-ecl.wit`            |
| W11   | Do  | Basic Host-Functions         | `host/mod.rs`               |
| W11   | Fr  | Unit-Tests                   | `tests/wasm_basic.rs`       |
| W12   | Mo  | WasmStateBridge              | `host/bridge.rs`            |
| W12   | Di  | Trust-Host-Functions         | `host/trust.rs`             |
| W12   | Mi  | State-Host-Functions         | `host/state.rs`             |
| W12   | Do  | Budget/Gas-Integration       | `host/budget.rs`            |
| W12   | Fr  | Integration-Tests            | `tests/wasm_integration.rs` |
| W13   | Mo  | AST→WASM Compiler            | `codegen/compiler.rs`       |
| W13   | Di  | Alle OpCodes gemappt         | `codegen/opcodes.rs`        |
| W13   | Mi  | Dual-Mode Runner             | `runtime/runner.rs`         |
| W13   | Do  | ProgrammableGateway WASM     | `programmable_gateway.rs`   |
| W13   | Fr  | Benchmarks + Docs            | `benchmarks/wasm_perf.rs`   |

### D.3 Abhängigkeits-DAG

$$\text{DAG}_\text{Phase} = \{P_1 \prec P_2 \prec P_3 \prec P_4 \prec P_5 \prec P_6\}$$

```
P₁ (Foundation)
   │
   ▼
P₂ (Decomposition) ─────────────────┐
   │                                │
   ▼                                ▼
P₃ (Synapse Hub)               P₄ (Integration)
   │                                │
   └────────────┬───────────────────┘
                ▼
           P₅ (WASM)
                │
                ▼
           P₆ (Optimization)
```

### D.4 Metriken-Evolution

$$\mathcal{M}(P_i): \text{Phase} \rightarrow \mathbb{R}^5$$

| Phase | state.rs LOC | Coverage | Event-Dispatch | Memory (MB) | Compile (min) |
| ----- | ------------ | -------- | -------------- | ----------- | ------------- |
| IST   | 21,495       | 60%      | 100µs          | 100         | 4.0           |
| $P_1$ | 21,495       | 62%      | 100µs          | 100         | 4.0           |
| $P_2$ | 12,000       | 70%      | 80µs           | 90          | 3.5           |
| $P_3$ | 8,000        | 75%      | 60µs           | 80          | 3.0           |
| $P_4$ | 5,000        | 80%      | 50µs           | 70          | 2.5           |
| $P_5$ | 3,500        | 82%      | 50µs           | 65          | 2.2           |
| $P_6$ | **2,000**    | **85%**  | **<50µs**      | **<60**     | **<2.0**      |

### D.5 Axiom Κ32: Phasen-Monotonie

$$\boxed{\forall i < j: \mathcal{M}(P_i)_\text{defizit} \geq \mathcal{M}(P_j)_\text{defizit}}$$

Defizite müssen monoton fallen.

---

## Appendix E: Legacy-Refactoring-Algebra 🔧

> **Quelle:** 05-MIGRATION-SCRIPTS.md, 16.2-LEGACY-PHASE1-QUICKSTART.md

### E.1 Refactoring-Operatoren

$$\boxed{\Phi_\text{Refactor} = \langle \phi_\text{setup}, \phi_\text{extract}, \phi_\text{backup}, \phi_\text{check}, \phi_\text{update}, \phi_\text{rollback} \rangle}$$

| Operator               | Funktion                        | Bash-Script                 |
| ---------------------- | ------------------------------- | --------------------------- |
| $\phi_\text{setup}$    | Verzeichnisstruktur erstellen   | `pluto-setup.sh`            |
| $\phi_\text{extract}$  | Module aus state.rs extrahieren | `extract-event-bus.sh`      |
| $\phi_\text{backup}$   | Snapshot vor Änderung           | `backup-before-refactor.sh` |
| $\phi_\text{check}$    | Compilation + Tests prüfen      | `check-after-extraction.sh` |
| $\phi_\text{update}$   | Import-Pfade aktualisieren      | `update-imports.sh`         |
| $\phi_\text{rollback}$ | Auf Backup zurücksetzen         | `rollback.sh`               |

### E.2 Migrations-Workflow

$$\text{Workflow}_\text{safe} = \phi_\text{backup} \circ \phi_\text{setup} \circ \phi_\text{extract} \circ \phi_\text{check} \circ \phi_\text{update}$$

**Rollback-Invariante:**
$$\forall \Phi: \text{failed}(\Phi) \implies \phi_\text{rollback}(\text{backup}) = \text{state}_\text{pre}$$

### E.3 Ziel-Verzeichnisstruktur

$$\text{Dir}_\text{SOLL} = \text{Tree}(\text{nervous\_system}, \text{synapses}, \text{realm}, \text{storage}, \text{p2p})$$

```
backend/src/
├── nervous_system/          # Zentrales State-Management
│   ├── event_sourcing/      # StateEvent, WrappedEvent, EventLog
│   ├── merkle/              # MerkleTracker, Delta, Proofs
│   ├── components/          # Core, Execution, Protection, Peer, P2P, ECLVM
│   ├── coordination/        # Coordinator, Transaction, Health
│   ├── graph/               # StateGraph, Relations, Analysis
│   └── infrastructure/      # EventBus, Broadcaster, CircuitBreaker, MultiGas
├── synapses/                # Observer-Hub & Adapter
│   ├── traits.rs            # StateObserver, StateLayer
│   ├── hub.rs               # SynapseHub (Dispatch)
│   └── adapters/            # Trust-, Event-, ECLVM-, P2P-, Realm-Adapter
├── realm/                   # Realm-Layer
│   ├── sharding/            # LazyState, Eviction
│   ├── quota/               # Enforcer, SelfHealing
│   ├── gateway/             # Guard, Policy
│   └── saga/                # Composer, Orchestrator, Compensation
├── storage/                 # Storage-Layer
│   ├── kv/                  # Fjall, Traits
│   ├── event_store/         # Event-Sourcing Backend
│   └── blueprint/           # Marketplace
└── p2p/                     # P2P-Layer (konsolidiert)
    ├── swarm/               # Manager, Config
    ├── gossip/              # Handler, Topics
    ├── dht/                 # Resolver
    └── privacy/             # Circuit, CoverTraffic
```

### E.4 Trait-Konsolidierung

$$\mathcal{T}_\text{unified} = \{\text{StateLayer}, \text{StateObserver}, \text{Resettable}, \text{Metered}\}$$

```rust
// StateLayer: Basis für alle State-Komponenten
pub trait StateLayer: Send + Sync + 'static {
    type Snapshot: Clone + Serialize + DeserializeOwned;
    fn snapshot(&self) -> Self::Snapshot;
    fn health_score(&self) -> f64;
    fn apply_event(&self, event: &WrappedStateEvent);
    fn component(&self) -> StateComponent;
}

// StateObserver: Universeller Observer
pub trait StateObserver: Send + Sync + 'static {
    fn on_event(&self, event: &WrappedStateEvent);
    fn target_component(&self) -> StateComponent;
    fn priority(&self) -> ObserverPriority { ObserverPriority::Normal }
}

// ObserverPriority: Dispatch-Reihenfolge
pub enum ObserverPriority {
    Critical = 0,  // Anomaly, CircuitBreaker
    High = 1,      // Trust, Consensus
    Normal = 2,    // Default
    Low = 3,       // Metrics, Logging
}
```

### E.5 Unified Error Hierarchie

$$\mathcal{E}_\text{unified} = \text{ErynoaError} \supset \{\text{Identity}, \text{Execution}, \text{Realm}, \text{Storage}, \text{P2P}, \text{ECLVM}, \text{State}\}$$

| Error-Typ        | Subsystem | Beispiele                                    |
| ---------------- | --------- | -------------------------------------------- |
| `IdentityError`  | Identity  | NotBootstrapped, SignatureVerificationFailed |
| `ExecutionError` | Execution | GasExhausted, ManaExhausted, Timeout         |
| `RealmError`     | Realm     | NotFound, Quarantined, QuotaExceeded         |
| `StorageError`   | Storage   | Io, Serialization, KeyNotFound               |
| `P2PError`       | P2P       | ConnectionFailed, PeerNotFound               |
| `ECLVMError`     | ECLVM     | Compilation, Runtime                         |
| `StateError`     | State     | ComponentNotFound, InvalidTransition         |

### E.6 SynapseHub-Algebra

$$\text{Hub}: \mathcal{E}_\text{event} \times \mathcal{O}_\text{observers} \rightarrow \text{Dispatch}$$

```rust
pub struct SynapseHub {
    observers: DashMap<StateComponent, Vec<Arc<dyn StateObserver>>>,
    dispatch_queue: mpsc::Sender<DispatchTask>,
    dispatch_count: AtomicU64,
}

impl SynapseHub {
    // Dispatch mit Priority + StateGraph-Propagation
    pub async fn dispatch(&self, event: WrappedStateEvent) {
        // 1. Direkte Observer
        if let Some(obs) = self.observers.get(&event.component) {
            for o in obs.value() { o.on_event(&event); }
        }
        // 2. Transitive via StateGraph
        for target in STATE_GRAPH.triggered_by(event.component) {
            if let Some(obs) = self.observers.get(&target) {
                for o in obs.value() { o.on_event(&event); }
            }
        }
    }
}
```

### E.7 Axiom Κ33: Rückwärtskompatibilität

$$\boxed{\forall \text{API}_\text{alt}: \exists \text{compat}(\text{API}_\text{alt}) \in \text{core/compat.rs}}$$

Alle alten APIs werden via Re-Export kompatibel gehalten.

### E.8 Axiom Κ34: Inkrementelle Validierung

$$\boxed{\forall \phi_i \in \Phi_\text{Refactor}: \text{test}(\phi_i) \land \text{compile}(\phi_i) \implies \text{commit}(\phi_i)}$$

Jeder Refactoring-Schritt muss vor Commit validiert sein.

---

## Appendix F: WASM-Migrations-Algebra (Extended) 🔄

> **Quelle:** 06-ECLVM-WASM-MIGRATION.md

### F.1 WASM-Engine-Architektur

$$\boxed{\Psi_\text{WASM} = \langle \text{Engine}, \text{Linker}, \text{Cache}, \text{Config} \rangle}$$

| Komponente | Typ                                    | Funktion                        |
| ---------- | -------------------------------------- | ------------------------------- |
| Engine     | `wasmtime::Engine`                     | Shared Compilation Engine       |
| Linker     | `Linker<WasmHostState>`                | Host-Function Registration      |
| Cache      | `Arc<RwLock<HashMap<String, Module>>>` | Pre-compiled Module Cache       |
| Config     | `WasmEngineConfig`                     | Fuel-Limits, Memory-Pages, SIMD |

### F.2 WIT-Interface Formalisierung

$$\text{WIT}_\text{erynoa} = \langle \text{TrustVector}, \text{GasLayer}, \text{HostFunctions} \rangle$$

```wit
// erynoa-ecl.wit
package erynoa:ecl@0.1.0;

record trust-vector {
    r: f64, i: f64, c: f64, p: f64, v: f64, omega: f64
}

enum gas-layer { network, compute, storage, realm }

interface host {
    // Trust (Κ2-Κ5)
    get-trust: func(did: string) -> result<trust-vector, string>;
    trust-norm: func(tv: trust-vector) -> f64;

    // Identity (Κ6-Κ8)
    has-credential: func(did: string, schema: string) -> result<bool, string>;
    resolve-did: func(did: string) -> result<bool, string>;

    // State (via StateView/StateHandle)
    store-get: func(store: string, key: string) -> result<option<string>, string>;
    store-put: func(store: string, key: string, value: string) -> result<unit, string>;

    // Budget (ECLVMBudget + MultiGas)
    consume-gas: func(layer: gas-layer, amount: u64) -> result<unit, string>;
    get-budget: func() -> tuple<u64, u64, u64, u64>;

    // Context + Events
    get-caller: func() -> string;
    get-realm: func() -> string;
    emit-event: func(event-type: string, payload: string) -> result<unit, string>;
}
```

### F.3 OpCode-Mapping Algebra

$$\Phi_\text{opcode}: \text{ECL}_\text{OpCode} \rightarrow \text{WASM}_\text{Instruction}$$

| ECL OpCode       | WASM Equivalent               | Kategorie        |
| ---------------- | ----------------------------- | ---------------- |
| `Push(f64)`      | `f64.const`                   | Direct           |
| `Add, Sub, Mul`  | `f64.add, f64.sub, f64.mul`   | Direct           |
| `Eq, Lt`         | `f64.eq, f64.lt`              | Direct           |
| `And, Or, Not`   | `i32.and, i32.or, i32.eqz`    | Bool-Conversion  |
| `LoadTrust(dim)` | `call $erynoa.get_trust`      | Host-Call        |
| `HasCredential`  | `call $erynoa.has_credential` | Host-Call        |
| `StoreGet/Put`   | `call $erynoa.store_*`        | Host-Call + Mana |
| `Require`        | `br_if + unreachable`         | Conditional      |
| `Return`         | `return`                      | Direct           |

### F.4 Dual-Mode Runner

$$\text{Runner}: \text{Policy} \times \text{Context} \times \text{Mode} \rightarrow \text{Result}$$

```rust
pub enum ExecutionMode {
    Legacy,  // Bytecode-Interpreter (Rückwärtskompatibilität)
    Wasm,    // WasmPolicyEngine (10x Performance)
    Auto,    // Basierend auf OpCode-Count
}

impl PolicyRunner {
    pub async fn execute(&self, policy: &CompiledPolicy, ctx: ECLVMStateContext) -> Result<ExecutionResult> {
        match self.mode {
            ExecutionMode::Legacy => self.execute_legacy(policy, ctx),
            ExecutionMode::Wasm   => self.execute_wasm(policy, ctx).await,
            ExecutionMode::Auto   => {
                if policy.opcodes.len() > self.auto_threshold {
                    self.execute_wasm(policy, ctx).await
                } else {
                    self.execute_legacy(policy, ctx)
                }
            }
        }
    }
}
```

### F.5 Performance-Metriken

$$\mathcal{P}_\text{WASM}: \text{Dimension} \rightarrow (\text{Legacy}, \text{WASM}, \text{Faktor})$$

| Dimension          | Legacy | WASM Ziel | Verbesserung   |
| ------------------ | ------ | --------- | -------------- |
| Policy-Latenz      | 2ms    | 0.2ms     | **10×**        |
| Trust-Ops/ms       | 50     | 500       | **10×**        |
| Startup (cold)     | 0.1ms  | 1ms       | -10× (akzept.) |
| Startup (hot)      | 0.1ms  | 0.1ms     | 1× (Cache)     |
| Memory/Policy      | 1MB    | 2MB       | -2× (akzept.)  |
| Throughput (Pol/s) | 500    | 5,000     | **10×**        |

### F.6 Migrations-Strategie (4-Phasen)

$$\text{Migration}_\text{WASM} = \langle A, B, C, D \rangle$$

| Phase | Aktion                                     | Trigger           |
| ----- | ------------------------------------------ | ----------------- |
| A     | Neue Policies mit WASM kompilieren         | Ab v0.5.0         |
| B     | Bestehende Policies bei Änderung migrieren | Bei Policy-Update |
| C     | Legacy-Interpreter deprecated              | Ab v0.8.0         |
| D     | Legacy-Code entfernen                      | v1.0.0            |

### F.7 Feature-Flags

$$\text{Features}_\text{WASM} = \{\text{wasm}, \text{wasm-simd}, \text{legacy-only}\}$$

```toml
[features]
default = ["wasm"]           # WASM standardmäßig aktiv
wasm = ["wasmtime", "wit-bindgen"]
wasm-simd = ["wasm", "wasmtime/simd"]
legacy-only = []             # Nur Bytecode-Interpreter
```

### F.8 Axiom Κ35: WASM-Determinismus

$$\boxed{\forall p \in \text{Policies}, c_1, c_2 \in \text{Context}: c_1 = c_2 \implies \Psi_\text{WASM}(p, c_1) = \Psi_\text{WASM}(p, c_2)}$$

WASM-Ausführung ist deterministisch (IEEE 754 strict).

### F.9 Axiom Κ36: Fuel-Gas-Äquivalenz

$$\boxed{\text{Fuel}_\text{WASM} \equiv \sum_{L \in \{\text{network}, \text{compute}, \text{storage}, \text{realm}\}} \text{Gas}_L}$$

Wasmtime-Fuel wird auf MultiGas-Layers gemappt.

---

## Appendix G: Code-Mapping (state.rs ↔ Formalisierung) 🔗

> **Quelle:** `backend/src/core/state.rs` (21,495 Zeilen)

Dieser Appendix stellt die direkte Verbindung zwischen der mathematischen Formalisierung und dem echten Rust-Code her.

### G.1 Fundamentale Code-Struktur

$$\boxed{\text{state.rs} = \bigsqcup_{i=1}^{9} \mathcal{M}_i \quad \text{mit} \quad |\text{state.rs}| = 21,495 \text{ LOC}}$$

| Zeilen    | Modul $\mathcal{M}_i$     | Mathematisches Symbol         | Rust-Struktur                    |
| --------- | ------------------------- | ----------------------------- | -------------------------------- |
| 1–500     | EventBus & Infrastructure | $\mathcal{N}_\text{EventBus}$ | `EventBus`, `NetworkEvent`       |
| 500–800   | StateBroadcaster          | $\mathcal{N}_\text{CQRS}$     | `StateBroadcaster`, `StateDelta` |
| 800–1200  | CircuitBreaker            | $🛡️_\text{CB}$                | `CircuitBreaker`, `SystemMode`   |
| 1200–2000 | StateEvent (42 Varianten) | $\mathcal{E}_\text{Event}$    | `StateEvent` enum                |
| 2000–2800 | WrappedStateEvent         | $\text{DAG}_\text{Event}$     | `WrappedStateEvent`              |
| 2800–4000 | IdentityState             | $\Sigma_\text{Identity}$      | `IdentityState`                  |
| 4000–4500 | StateGraph                | $\mathcal{G}_\text{State}$    | `StateGraph`                     |
| 4500–5500 | TrustState                | $\Sigma_\text{Trust}$         | `TrustState`, `TrustEntry`       |
| 5500+     | Weitere States            | $\Sigma_\text{*}$             | `EventState`, `FormulaState`...  |

### G.2 Design-Prinzipien ↔ Axiome

Die 9 Design-Prinzipien in state.rs korrespondieren direkt mit den Axiomen:

| #   | Design-Prinzip (Rust-Doc) | Axiom | Formel                                                      |
| --- | ------------------------- | ----- | ----------------------------------------------------------- |
| 1   | Hierarchische Komposition | Κ9    | $\Sigma = \prod_i \Sigma_i$                                 |
| 2   | Thread-Safety (Atomics)   | —     | Lock-free Counters                                          |
| 3   | Dependency Injection      | Κ19   | Layer-Abhängigkeiten                                        |
| 4   | Event-Driven Updates      | Κ28   | Observer-Pattern                                            |
| 5   | Snapshot-Isolation        | Κ9    | MVCC-Semantik                                               |
| 6   | Per-Realm Isolation       | Κ24   | $\vec{\tau}(\iota, \rho_1) \perp \vec{\tau}(\iota, \rho_2)$ |
| 7   | Event-Inversion           | Κ28   | Ingress/Egress Queues                                       |
| 8   | Circuit Breaker           | Κ19   | Automatische Degradation                                    |
| 9   | CQRS light                | Κ28   | Broadcast-Channels                                          |

### G.3 StateEvent-Varianten ↔ StateComponent

$$|\text{StateEvent}| = 42 \text{ Varianten}$$

```rust
pub enum StateEvent {
    // CORE (Κ2-Κ5): TrustUpdate, EventProcessed, FormulaComputed, ConsensusRoundCompleted
    // EXECUTION (Κ11-Κ14): ExecutionStarted, ExecutionCompleted, PolicyEvaluated
    // PROTECTION (Κ19): AnomalyDetected, DiversityMetricUpdate, SystemModeChanged
    // PEER/REALM (Κ22-Κ24): RealmLifecycle, MembershipChange, CrossingEvaluated
    // P2P (Κ28): NetworkMetricUpdate, PeerConnectionChange, TrustUpdated
    // PRIVACY: PrivacyCircuitCreated, PrivacyMessageSent, CoverTrafficGenerated
    // IDENTITY (Κ6-Κ8): IdentityBootstrapped, SubDIDDerived, DelegationCreated
    // GOVERNANCE: ProposalCreated, VoteCast, ProposalResolved
    // ...
}
```

**Mapping-Funktion:**

$$\text{primary\_component}: \text{StateEvent} \rightarrow \text{StateComponent}$$

```rust
impl StateEvent {
    pub fn primary_component(&self) -> StateComponent {
        match self {
            StateEvent::TrustUpdate { .. }     => StateComponent::Trust,
            StateEvent::IdentityBootstrapped { .. } => StateComponent::Identity,
            StateEvent::RealmLifecycle { .. } => StateComponent::Realm,
            // ... 39 weitere Mappings
        }
    }
}
```

### G.4 StateGraph-Implementierung

$$\mathcal{G}_\text{State} = (V, E) \quad \text{mit} \quad |V| = 37, |E| = 110+$$

```rust
pub struct StateGraph {
    pub edges: Vec<(StateComponent, StateRelation, StateComponent)>,
}

impl StateGraph {
    pub fn erynoa_graph() -> Self {
        Self {
            edges: vec![
                // Identity-Layer (Κ6-Κ8)
                (Trust, DependsOn, Identity),
                (Identity, Triggers, Trust),
                (Event, DependsOn, Identity),
                // ... 107+ weitere Kanten
            ],
        }
    }
}
```

**Graph-Operationen:**

| Operation                    | Signatur                                          | Komplexität |
| ---------------------------- | ------------------------------------------------- | ----------- | --- | ----- | --- | --- |
| `dependents(c)`              | $\mathcal{G} \times C \rightarrow \mathcal{P}(C)$ | $O(         | E   | )$    |
| `triggered_by(c)`            | $\mathcal{G} \times C \rightarrow \mathcal{P}(C)$ | $O(         | E   | )$    |
| `transitive_dependencies(c)` | $\mathcal{G} \times C \rightarrow \mathcal{P}(C)$ | $O(         | V   | +     | E   | )$  |
| `criticality_score(c)`       | $\mathcal{G} \times C \rightarrow \mathbb{N}$     | $O(         | V   | \cdot | E   | )$  |

### G.5 TrustState ↔ Trust-Axiome

$$\Sigma_\text{Trust} = \langle \text{Atomics}, \text{Complex}, \text{Relations} \rangle$$

```rust
pub struct TrustState {
    // Atomics (Lock-free, High-Frequency)
    pub entities_count: AtomicUsize,
    pub updates_total: AtomicU64,
    pub positive_updates: AtomicU64,      // Κ4: Asymmetrie
    pub negative_updates: AtomicU64,      // Κ4: neg ≈ 2× pos

    // Complex State (RwLock)
    pub trust_by_id: RwLock<HashMap<UniversalId, TrustEntry>>,

    // Relationship-Tracking (StateGraph)
    pub triggered_events: AtomicU64,       // Trust → Event
    pub event_triggered_updates: AtomicU64, // Event → Trust
    pub realm_triggered_updates: AtomicU64, // Realm → Trust (Κ24)
}
```

**Axiom Κ4 Implementierung:**

$$\boxed{\text{asymmetry\_ratio}() = \frac{\text{negative\_updates}}{\text{positive\_updates}} \approx 2.0}$$

```rust
pub fn asymmetry_ratio(&self) -> f64 {
    let pos = self.positive_updates.load(Ordering::Relaxed) as f64;
    let neg = self.negative_updates.load(Ordering::Relaxed) as f64;
    if pos > 0.0 { neg / pos } else { 0.0 }
}
```

### G.6 IdentityState ↔ Identity-Axiome (Κ6-Κ8)

$$\Sigma_\text{Identity} = \langle \text{Bootstrap}, \text{Mode}, \text{DIDs}, \text{Delegations}, \text{Handles} \rangle$$

```rust
pub struct IdentityState {
    // Κ6: Existenz-Eindeutigkeit
    pub bootstrap_completed: AtomicBool,
    pub root_did: RwLock<Option<DID>>,           // ∃! did

    // Κ7: Permanenz
    pub root_created_at_ms: AtomicU64,           // □⟨s⟩

    // Κ8: Delegations-Struktur
    pub delegations: RwLock<HashMap<UniversalId, Delegation>>,
    pub active_delegations_count: AtomicU64,

    // Mode ∈ {Interactive, AgentManaged, Ephemeral, Test}
    pub mode: AtomicU8,

    // Orthogonale Handles
    pub key_store: Option<SharedKeyStore>,       // TEE/TPM
    pub passkey_manager: Option<SharedPasskeyManager>,
}
```

**Axiom Κ8 (Trust-Decay) Implementierung:**

$$\boxed{\tau(\text{delegate}) \leq \text{trust\_factor} \cdot \tau(\text{delegator})}$$

```rust
pub struct TrustEntry {
    pub decay_factor: f64,  // Κ8

    pub fn apply_decay(&mut self, decay_rate: f64) {
        self.decay_factor *= 1.0 - decay_rate;
        self.global_trust *= 1.0 - decay_rate;
    }
}
```

### G.7 EventBus ↔ Nervensystem (Κ28)

$$\mathcal{N}_\text{EventBus} = \langle \text{Ingress}, \text{Egress}, \text{Priority}, \text{Metrics} \rangle$$

```rust
pub struct EventBus {
    // P2P → Core (Ingress)
    pub ingress_tx: mpsc::Sender<NetworkEvent>,
    pub priority_ingress_tx: mpsc::Sender<NetworkEvent>,  // Critical/High

    // Core → P2P (Egress)
    pub egress_tx: mpsc::Sender<NetworkEvent>,

    // Metriken
    pub ingress_count: AtomicU64,
    pub egress_count: AtomicU64,
    pub dropped_count: AtomicU64,

    const DEFAULT_QUEUE_SIZE: usize = 10_000;
    const PRIORITY_QUEUE_SIZE: usize = 1_000;
}
```

**Bounded Queue Invariante:**

$$\boxed{|\text{Queue}| \leq 10,000 \implies \text{Backpressure statt Memory-Exhaustion}}$$

### G.8 CircuitBreaker ↔ Protection (Κ19)

$$🛡️_\text{CB}: \text{AnomalyCount} \rightarrow \text{SystemMode}$$

```rust
pub struct CircuitBreaker {
    mode: AtomicU8,  // SystemMode ∈ {Normal, Degraded, EmergencyShutdown}
    critical_window: RwLock<Vec<u64>>,  // Timestamps der letzten Minute

    pub degraded_threshold: AtomicU64,   // Default: 10
    pub emergency_threshold: AtomicU64,  // Default: 50
    pub gini_threshold: RwLock<f64>,     // Κ19: Default 0.8
}

impl CircuitBreaker {
    pub fn record_critical_anomaly(&self) -> SystemMode {
        // Alte Einträge entfernen (> 1 Minute)
        // Zählen und Schwellwerte prüfen
        if count >= self.emergency_threshold { SystemMode::EmergencyShutdown }
        else if count >= self.degraded_threshold { SystemMode::Degraded }
        else { SystemMode::Normal }
    }
}
```

**SystemMode-FSM:**

$$\text{Normal} \xrightarrow{10+ \text{Anomalies}} \text{Degraded} \xrightarrow{50+ \text{Anomalies}} \text{Emergency}$$

### G.9 RealmQuota ↔ Quota-Axiome (Κ22)

$$\mathcal{Q}_\rho = \langle \text{Limits}, \text{Used}, \text{Violations}, \text{Quarantine} \rangle$$

```rust
pub struct RealmQuota {
    // 5 Ressource-Typen
    pub queue_slots_limit: AtomicU64,
    pub storage_bytes_limit: AtomicU64,
    pub compute_gas_limit: AtomicU64,
    pub events_limit: AtomicU64,
    pub crossings_limit: AtomicU64,

    // Auto-Quarantine
    pub violations: AtomicU64,
    pub quarantined: AtomicU8,

    const AUTO_QUARANTINE_THRESHOLD: u64 = 10;
}

impl RealmQuota {
    pub fn consume(&self, resource: ResourceType, amount: u64) -> bool {
        if !self.check_quota(resource, amount) {
            self.violations.fetch_add(1, Ordering::Relaxed);
            if self.violations.load(..) >= 10 { self.quarantine(); }
            return false;
        }
        // ...
    }
}
```

### G.10 Axiom Κ37: Code-Formalisierung-Isomorphismus

$$\boxed{\forall \mathcal{A} \in \mathcal{K}_{36}: \exists \text{impl}(\mathcal{A}) \in \text{state.rs}}$$

Jedes mathematische Axiom hat eine konkrete Implementierung im Code.

| Axiom | Rust-Implementierung            | Zeile (ca.) |
| ----- | ------------------------------- | ----------- |
| Κ1    | `Realm.rules ⊇ parent.rules`    | RealmState  |
| Κ4    | `TrustState::asymmetry_ratio()` | 4650        |
| Κ6    | `IdentityState::bootstrap_*()`  | 3300        |
| Κ8    | `TrustEntry::apply_decay()`     | 4580        |
| Κ19   | `CircuitBreaker::check_gini()`  | 640         |
| Κ22   | `RealmQuota::consume()`         | 2870        |
| Κ24   | `TrustEntry::per_realm_trust`   | 4520        |
| Κ28   | `EventBus::try_send_ingress()`  | 285         |

### G.11 StateEventEmitter-Trait

$$\text{Emitter}: \text{Module} \rightarrow \text{StateEvent} \rightarrow \text{UnifiedState}$$

```rust
pub trait StateEventEmitter: Send + Sync {
    fn emit(&self, event: StateEvent);
    fn emit_batch(&self, events: Vec<StateEvent>);
    fn is_active(&self) -> bool;
}

// Implementierungen
pub struct NoOpEmitter;           // Tests
pub struct ChannelEmitter { tx: mpsc::UnboundedSender<StateEvent> };
pub struct CallbackEmitter { callback: Box<dyn Fn(StateEvent)> };
```

### G.12 MerkleStateTracker

$$\text{Merkle}_\Sigma: \text{StateComponent} \rightarrow H_{256}$$

```rust
pub struct MerkleStateTracker {
    component_hashes: RwLock<HashMap<StateComponent, MerkleHash>>,
    root_hash: RwLock<MerkleHash>,
    updates_count: AtomicU64,
}

pub type MerkleHash = [u8; 32];  // Blake3
```

**State-Proof-Generierung:**

$$\boxed{\pi_\text{state}(c) = \text{merkle\_path}(\text{root}, \text{hash}(c))}$$

### G.13 Zusammenfassung: Code ↔ Formalisierung

| Konzept    | Mathematik                       | Rust-Typ                   | LOC     |
| ---------- | -------------------------------- | -------------------------- | ------- |
| Universum  | $\mathbb{U}_\text{Pluto}$        | `UnifiedState`             | ~12,000 |
| Trust      | $\vec{\tau} \in \mathbb{R}^6$    | `TrustState`               | ~1,000  |
| Identity   | $\iota = (did, \vec{\tau}, \nu)$ | `IdentityState`            | ~1,200  |
| Events     | $\mathcal{E}_\text{DAG}$         | `StateEvent` (42 variants) | ~800    |
| StateGraph | $\mathcal{G} = (V, E)$           | `StateGraph`               | ~400    |
| Protection | $🛡️$                             | `CircuitBreaker`           | ~200    |
| Quotas     | $\mathcal{Q}_\rho$               | `RealmQuota`               | ~200    |
| EventBus   | $\mathcal{N}$                    | `EventBus`                 | ~300    |

---

## Appendix H: Concept-V4 Integration (Kategorientheorie + P2P-Relay + Generative Realms) 🆕

> **Quellen:** `concept-v4/LOGIC.md`, `concept-v4/P2P-PRIVATE-RELAY-LOGIC.md`, `concept-v4/LOGIC-GENERATIVE-REALMS.md`

Dieser Appendix integriert die fortgeschrittenen Formalisierungen aus concept-v4, die das Axiom-System erheblich erweitern.

### H.1 Meta-Axiom Μ1: Partielle Ordnung (Kategorientheorie)

$$\boxed{\text{Μ1: } \forall R \text{ auf } M: R \text{ ist streng partiell geordnet} \Leftrightarrow \text{Irreflexiv} \land \text{Antisymm.} \land \text{Transitiv}}$$

**Unifizierung:** Sowohl Delegation (⊳) als auch Kausalität (⊲) sind Instanzen von Μ1:

| Relation       | Objekt-Menge | Interpretation                         |
| -------------- | ------------ | -------------------------------------- |
| ⊳ (Delegation) | DID          | $s \triangleright s'$ → Trust fließt   |
| ⊲ (Kausalität) | Event        | $e \triangleleft e'$ → Kausal abhängig |

### H.2 Die Erynoa-Kategorie $\mathcal{C}_{Ery}$

$$\mathcal{C}_{Ery} = (Ob, Mor, \circ, id)$$

| Komponente               | Definition                                                                                                                                                                                  |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| $Ob(\mathcal{C}_{Ery})$  | $\{\text{DID}_{self}, \text{DID}_{guild}, \text{DID}_{spirit}, \text{AMO}, \text{VC}, \text{Partition}, \text{VirtualRealm}, \text{RootRealm}\}$                                            |
| $Mor(\mathcal{C}_{Ery})$ | $\{\text{Delegation}(\triangleright), \text{Attestation}(\vdash), \text{Transfer}(\rightarrow), \text{Membership}(\in), \text{Causation}(\triangleleft), \text{Transition}(\Rrightarrow)\}$ |

**Kategorien-Axiome:**

- $\forall f: A \rightarrow B, g: B \rightarrow C: \exists! (g \circ f): A \rightarrow C$ (Komposition)
- $\forall A \in Ob: \exists! id_A: A \rightarrow A$ (Identität)
- $(h \circ g) \circ f = h \circ (g \circ f)$ (Assoziativität)
- $id_B \circ f = f = f \circ id_A$ (Neutralität)

### H.3 Fünf Isomorphismen (Entdeckte Strukturen)

$$\boxed{\text{5 fundamentale Isomorphismen vereinheitlichen das System}}$$

| #   | Isomorphismus              | Implikation                                           |
| --- | -------------------------- | ----------------------------------------------------- | ---------- |
| 1   | Delegation ≅ Kausalität    | Beide sind strenge partielle Ordnungen (Μ1)           |
| 2   | Trust-Kombination ≅ P(A∨B) | $t_1 \oplus t_2 = 1-(1-t_1)(1-t_2) \cong P(A \vee B)$ |
| 3   | Partition-Konsens ≅ Bayes  | $\Psi(\Sigma)(\phi) \cong P(\phi                      | evidence)$ |
| 4   | Weltformel ≅ Freie Energie | $\mathbb{E} \cong F = U - TS$ (Helmholtz)             |
| 5   | Realm-Hierarchie ≅ Topos   | Intuitionistische Logik per Partition                 |

### H.4 Weltformel V2.0 (Präzisiert)

$$\boxed{\mathbb{E} = \sum_{s \in \mathcal{C}} \mathbb{A}(s) \cdot \vec{\sigma}\left(\|\mathbb{W}(s)\|_w \cdot \ln|\mathcal{C}(s)| \cdot \mathcal{S}(s)\right) \cdot \hat{H}(s) \cdot w(s,t)}$$

**Neu in V2.0:**

| Symbol             | Definition                                 | Axiom                            |
| ------------------ | ------------------------------------------ | -------------------------------- | ------------------------ |
| $\mathcal{S}(s)$   | $\|\mathbb{W}(s)\|^2 \cdot \mathcal{I}(s)$ | Κ38a (Trust-gedämpfte Surprisal) |
| $\mathcal{I}(s)$   | $-\log_2 P(e                               | \mathcal{C}(s))$                 | Κ38b (Shannon-Surprisal) |
| $\|\mathbb{W}\|_w$ | $\sqrt{\sum_i w_i \cdot \mathbb{W}_i^2}$   | Κ38c (Kontextabhängige Norm)     |

**Anti-Hype-Mechanismus:**
$$\mathcal{S}(s) = \|\mathbb{W}(s)\|^2 \cdot \mathcal{I}(s) \implies \text{Noise von } \mathbb{W}=0.3 \text{ wird 91\% gedämpft}$$

### H.5 P2P-Private-Relay-Axiome (RL1-RL7)

> **Quelle:** `concept-v4/P2P-PRIVATE-RELAY-LOGIC.md` (2,608 Zeilen)

$$\boxed{\text{Relay-Kategorie } \mathcal{R} \subset \mathcal{C}_{Ery}}$$

**Axiom RL1 (Relay-Eignung mit ZK-Beweis):**
$$p \in Peers(\mathcal{R}) \Leftrightarrow ZK.Verify(\pi_{elig}, commitment(\mathbb{W}(p)), \vec{\tau})$$

**Axiom RL2 (Wissens-Separation):**
$$\forall R_i \in Route: I(Sender; Empfänger | View(R_i)) \leq \epsilon_{leak}$$

**Axiom RL3 (Schichten-Integrität):**
$$\forall R_i: D_{K_i}(Layer_i) = Layer_{i+1} \| addr(R_{i+1})$$

**Axiom RL4 (Forward + Backward Secrecy):**

- Forward: $compromise(sk_i, t_2) \Rightarrow \neg reveal(session\_key_i, t_1)$ für $t_1 < t_2$
- Backward: Key-Rotation alle 24h (Hochsicherheit: 1h)

**Axiom RL5 (Game-Theoretische Anreize):**
$$\text{Nash-GG: } \forall R: U_R(relay\_honestly) \geq U_R(defect)$$

**Axiom RL6 (Relay-Diversität):**
$$H_{route}(\pi) = -\sum_i \sum_{attr} P(attr_i) \cdot \log P(attr_i) \geq H_{min}$$

**Axiom RL7 (Adaptive Hop-Anzahl):**
$$n(\sigma) = n_{base} + \Delta n(\sigma) + \Delta n_{threat}$$

| Level σ  | Hops n | Mixing τ | Latency  | Use Case  |
| -------- | ------ | -------- | -------- | --------- |
| LOW      | 2      | 50ms     | < 200ms  | Public    |
| MEDIUM   | 3      | 100ms    | < 500ms  | Normal    |
| HIGH     | 4      | 200ms    | < 1000ms | Private   |
| CRITICAL | 5      | 500ms    | < 2000ms | Sensitive |

### H.6 Onion-Verschlüsselungs-Algebra

$$\Omega(M, \pi) = E_{K_1}(E_{K_2}(...E_{K_n}(M \| addr(dest))...\| addr(R_3)) \| addr(R_2))$$

**Sicherheits-Eigenschaft:**
$$P(R_i \text{ kennt } M | R_i \neq R_n) = negl(\lambda)$$

### H.7 Generative-Realm-Axiome (GR1-GR12)

> **Quelle:** `concept-v4/LOGIC-GENERATIVE-REALMS.md` (1,236 Zeilen)

$$\boxed{\mathcal{C}_{Ery+} = \mathcal{C}_{Ery} \cup \mathcal{C}_{Gen}}$$

**Neue Objekte:**
$$Ob(\mathcal{C}_{Gen}) = \{GenerativeRealm, UIBundle, BridgeChannel, Sandbox\}$$

**Neue Morphismen:**
$$Mor(\mathcal{C}_{Gen}) = \{Generation(\vDash), Hosting(\diamond), Rendering(\rightsquigarrow), Interaction(\rightleftarrows)\}$$

**Axiom GR1 (Bundle-Struktur):**
$$UIBundle = \langle manifest, assets, logic, signature \rangle$$

**Axiom GR2 (Content-Addressierung):**
$$id(B) = \text{"erynoa://bundle/"} \| base58(hash256(B))$$

**Axiom GR3 (GenerativeRealm-Definition):**
$$GenerativeRealm \subset VirtualRealm + \{ui\_bundle, creator, update\_policy, interaction\_mode\}$$

**Axiom GR4 (Join-Protokoll):**
$$join(User, GenerativeRealm) = \{parse \rightarrow resolve \rightarrow verify \rightarrow fetch \rightarrow sandbox \rightarrow bridge \rightarrow subscribe \rightarrow render\}$$

**Axiom GR5 (Bridge-Architektur):**
$$BridgeAPI = \{send, receive, subscribe, getState, updateUI\}$$

**Axiom GR6 (Bridge-Sicherheit):**

- Isolation: $\forall op \in Sandbox: \neg access(op, filesystem)$
- Rate-Limiting: $rate(send) \leq 60/min$
- Onion-Encryption: Alle Bridge-Nachrichten via RL2-RL4

**Axiom GR7 (Dynamische UI-Patches):**
$$UIPatch = \langle selector, operation, content, signature \rangle$$

**Axiom GR8 (Creator-Eligibility):**
$$eligible\_creator(\mathcal{A}) \Leftrightarrow namespace(did(\mathcal{A})) = \text{"spirit"} \land \mathbb{W}(\mathcal{A}).C \geq \tau_C \land controller(\mathcal{A}) = verified\_human$$

**Axiom GR9 (Generation-Process):**
$$\{eligible\_creator(\mathcal{A}) \land valid\_prompt(p)\} \quad \Pi\text{-GEN}(\mathcal{A}, p) \quad \{\exists realm: creator(realm) = \mathcal{A}\}$$

**Axiom GR10 (Dungeon-Master-Semantik):**
$$DungeonMaster(\mathcal{A}, R) \Leftrightarrow creator(R) = \mathcal{A} \land update\_policy(R) = DYNAMIC$$

**Axiom GR11 (Multiplayer-Konsistenz):**
$$State(R) = \{shared: SharedState, private: Map\langle User, PrivateState \rangle\}$$

### H.8 Weltformel-Parameter-Herleitung (Κ38c-d)

**Κ38c (Prinzipienbasierte Parameter):**

| Parameter        | Herleitung                 | Wert                         |
| ---------------- | -------------------------- | ---------------------------- |
| $\gamma_{neg}$   | Ebbinghaus-Vergessenskurve | $\ln(2) / (3 \text{ Jahre})$ |
| $\gamma_{pos}$   | Ebbinghaus (länger)        | $\ln(2) / (5 \text{ Jahre})$ |
| $k$ (Sigmoid)    | Max. Entropie-Transfer     | 1.0                          |
| $\lambda_{asym}$ | Kahneman-Tversky           | 1.5 (R,I,C,P), 2.0 (V,Ω)     |

**Κ38d (Hierarchische Approximation):**
$$\mathbb{E} \approx \sum_{partitions} |partition| \cdot \bar{\mathbb{E}}_{sample}$$

**Komplexität:** $O(|Partitions| \cdot k)$ statt $O(|\mathcal{C}|)$

### H.9 Axiom-Konsolidierung: V4.0 Mapping

Die concept-v4 LOGIC.md konsolidiert **126 ursprüngliche Axiome** auf:

| Kategorie                  | Kern-Axiome | IDs                     |
| -------------------------- | ----------- | ----------------------- |
| Kategorische Fundierung    | 2           | Μ1, Κ1-Κ2               |
| Trust-Algebra              | 3           | Κ3-Κ5                   |
| Identitäts-Algebra         | 3           | Κ6-Κ8                   |
| Kausale Algebra            | 2           | Κ9-Κ10                  |
| Prozess-Algebra            | 4           | Κ11-Κ14                 |
| Weltformel (NEU V4)        | 4           | Κ38a-d (= Κ15a-d in V4) |
| Humanismus                 | 2           | Κ16-Κ17                 |
| Konsens                    | 1           | Κ18                     |
| Schutz                     | 3           | Κ19-Κ21                 |
| Peer-Logik                 | 3           | Κ22-Κ24                 |
| System-Garantien           | 4           | Κ25-Κ28                 |
| **Relay-Axiome (NEU)**     | **7**       | **RL1-RL7 → Κ39-Κ45**   |
| **Generative-Realm (NEU)** | **12**      | **GR1-GR12 → Κ46-Κ57**  |

### H.10 Erweitertes Axiom-Register

$$\boxed{|\mathcal{K}_{extended}| = 37 + 4 + 7 + 12 = 60 \text{ Axiome}}$$

| Range   | Domäne            | Beschreibung                     |
| ------- | ----------------- | -------------------------------- |
| Κ1-Κ37  | Core              | Bisherige UNIFIED-Axiome         |
| Κ38a-d  | Weltformel V2.0   | Präzisierte Surprisal, Parameter |
| Κ39-Κ45 | P2P-Relay         | RL1-RL7 (Privacy-Layer)          |
| Κ46-Κ57 | Generative Realms | GR1-GR12 (KI-Welten)             |

---

## 🏆 Finales Haupttheorem (UNIFIED v1.9.0)

$$\boxed{\mathcal{U}_\text{Pluto}^{\text{FINAL}} = \langle \mathcal{K}_0, \mathcal{E}, \mathcal{R}, \mathcal{O}, \mathcal{K}_{61}, \mathcal{S}, \mathcal{N}, \Psi, \Phi, \mathcal{D}, \mathcal{P}, \mathcal{L}, \mathcal{C}, \mathcal{R}_L, \mathcal{G}_R \rangle}$$

| Komponente         | Inhalt                                                | Axiome   |
| ------------------ | ----------------------------------------------------- | -------- |
| $\mathcal{K}_0$    | **🔑 Passkey-Primacy (Authentifizierungs-Fundament)** | **Κ0**   |
| $\mathcal{E}$      | 8 Hauptentitäten + 4 Gen-Entitäten                    | —        |
| $\mathcal{R}$      | 6 Relationstypen + 4 Gen-Morphismen, 110+ Kanten      | —        |
| $\mathcal{O}$      | 20+ Operations                                        | —        |
| $\mathcal{K}_{61}$ | **61 Axiome** (Κ0–Κ57 + Μ1)                           | ✅       |
| $\mathcal{S}$      | 11+ Synergien + 5 Isomorphismen                       | —        |
| $\mathcal{N}$      | Nervensystem (SynapseHub)                             | —        |
| $\Psi$             | ECLVM (Legacy + WASM Dual-Mode)                       | Κ35, Κ36 |
| $\Phi$             | 6 Refactoring-Operatoren                              | Κ33, Κ34 |
| $\mathcal{D}$      | IST-Defizit-Katalog                                   | Κ31      |
| $\mathcal{P}$      | 6-Phasen-Plan über 14 Wochen                          | Κ32      |
| $\mathcal{L}$      | Legacy-Refactoring-Algebra                            | Κ33, Κ34 |
| $\mathcal{C}$      | Code-Isomorphismus (state.rs)                         | Κ37      |
| $\mathcal{R}_L$    | **P2P-Relay-Layer**                                   | Κ39-Κ45  |
| $\mathcal{G}_R$    | **Generative-Realm-Layer**                            | Κ46-Κ57  |

**Beweis der Vollständigkeit:**

1. **🔑 Phase 0 (Fundament):** **Κ0 Passkey-Primacy** — Einzige Authentifizierungs-Wurzel ✅ 🆕
2. **Phase 1 (Compressed):** Κ1–Κ26 aus 14 Quelldateien ✅
3. **Phase 2 (Architektur):** Κ19–Κ21 aus 00-OVERVIEW, 02-ZIEL-ARCHITEKTUR, 03-BEZIEHUNGSMATRIX ✅
4. **Phase 3 (Packages):** Κ22 aus 11-_, 12-_ ✅
5. **Phase 4 (Realm):** Κ23–Κ24 aus 13-REALM-ARCHITEKTUR-ISOLATION ✅
6. **Phase 5 (Agent):** Κ25–Κ26 aus 18-AGENT-SHELL-ZUGRIFF ✅
7. **Phase 6 (Synergien):** Κ27–Κ30 aus 07-_, 19-_ ✅
8. **Phase 7 (Migration):** Κ31–Κ36 aus 01-_, 04-_, 05-_, 06-_, 16.1, 16.2 ✅
9. **Phase 8 (Code-Mapping):** Κ37 aus backend/src/core/state.rs (21,495 LOC) ✅
10. **Phase 9 (Concept-V4):** Κ38a-d, Κ39-Κ45, Κ46-Κ57 aus concept-v4/\*.md ✅

---

**∎ Q.E.D. (UNIFIED COMPLETE + CODE-VERIFIED + CONCEPT-V4 + PASSKEY-PRIMACY)**

> _Dieses Dokument konsolidiert ALLE 25 Quelldokumente (14 compressed + 8 projekt-pluto + 3 concept-v4) + state.rs (21,495 LOC) in einer vollständigen mathematisch-logischen Formalisierung mit **61 Axiomen**. Inklusive kategorientheoretischer Fundierung, P2P-Privacy-Layer, KI-generierte Realm-Architektur und **Passkey als fundamentales Authentifizierungs-Axiom (Κ0)**._

**Pluto-Signatur:** `UNIFIED::v1.9.0::FINAL::2026-02-04::AllPhasesComplete::CodeVerified::ConceptV4::PasskeyPrimacy`
