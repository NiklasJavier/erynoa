# Pluto::TrustGasMana ≡ Dreieinigkeits-Immunsystem

> **Notation:** Pluto (komprimiert-formal)
> **Version:** 2.0 | **Datum:** 2026-02

---

## §1 Ontologische Grundlagen

### Δ1.1 Negation der Token-Metapher

$$\boxed{\neg(\text{Trust} \equiv \text{Token}) \land \neg(\text{Gas} \equiv \text{Coin}) \land \neg(\text{Mana} \equiv \text{Credit})}$$

**Theorem (Emergenz):**
$$\text{Trust}, \text{Gas}, \text{Mana} \in \text{EmergentProperties}(\mathcal{S})$$

wobei $\mathcal{S}$ das Gesamtsystem bezeichnet.

### Δ1.2 Organische Triaden-Metapher

| Symbol         | Metapher       | Funktion            | Regeneration    |
| -------------- | -------------- | ------------------- | --------------- |
| $\tau$ (Trust) | 🫀 Immunsystem | Entitäts-Bewertung  | Emergent        |
| $\gamma$ (Gas) | ⚡ Muskelkraft | Compute-Budget      | ∅ (erschöpfend) |
| $\mu$ (Mana)   | 🌊 Atem        | Bandwidth-Kapazität | Kontinuierlich  |

---

## §2 Emergenz-Axiome

### Ε2.1 Kausalkette

$$\text{Existenz} \xrightarrow{\text{Handlung}} \text{Beobachtung} \xrightarrow{\text{Bewertung}} \text{Trust} \xrightarrow{\text{Skalierung}} (\gamma, \mu)$$

**Initialzustand (Newcomer):**

$$
\begin{aligned}
\tau_0 &= 0.1 \quad\text{(NEWCOMER\_LEVEL)} \\
\mu_0 &= 10{,}000 \quad\text{(BASE\_MANA)} \\
\gamma_{\text{cost}} &= \gamma_{\text{base}} \cdot (2 - \tau) = 1.9 \cdot \gamma_{\text{base}}
\end{aligned}
$$

### Ε2.2 Feedback-Loop

$$\tau_{t+1} = \tau_t + \sum_i \delta_i \cdot w_i \cdot \text{ctx}_i$$

**Selbstverstärkung (Matthäus-Effekt):**
$$\tau \uparrow \implies (\mu, \gamma) \uparrow \implies P(\text{Erfolg}) \uparrow \implies \tau \uparrow$$

**Selbstauslöschung (Angreifer):**
$$\tau \downarrow \implies (\mu, \gamma) \downarrow \implies P(\text{Erschöpfung}) \uparrow \implies \text{Isolation}$$

---

## §3 7-Schichten-Immunsystem (Defense-in-Depth)

### Λ3.1 Schicht-Hierarchie

$$\mathcal{L} = \{L_1, L_2, \ldots, L_7\}$$

| $L_i$ | Name       | Funktion           | Ressource            |
| ----- | ---------- | ------------------ | -------------------- |
| $L_1$ | Gateway    | Preflight-Abwehr   | Mana-Check           |
| $L_2$ | Mana       | Anti-Spam/Flooding | $\mu$ regenerierend  |
| $L_3$ | Gas        | Anti-DoS/Loops     | $\gamma$ erschöpfend |
| $L_4$ | Trust      | Langfrist-Filter   | $\tau$ asymmetrisch  |
| $L_5$ | Realm      | Sandbox-Isolation  | Quota + Policies     |
| $L_6$ | DID        | Krypto-Bindung     | UTI + VC             |
| $L_7$ | Protection | Self-Healing       | CircuitBreaker       |

### Λ3.2 Synergie-Operator

$$\text{Defense}(A) = \prod_{i=1}^{7} (1 - P_{\text{breach}}(L_i | A))$$

**Theorem (Angreifer-Erschöpfung):**
$$\forall A \in \text{Attackers}: \lim_{t \to \infty} \text{Resources}(A, t) = 0$$

---

## §4 Trust-Vektor $\mathbb{T}$ (6 Dimensionen)

### Τ4.1 Definition

$$\mathbb{T} = (R, I, C, P, V, \Omega) \in [0,1]^6$$

| Dim      | Name           | Semantik                        | Effekt                                 |
| -------- | -------------- | ------------------------------- | -------------------------------------- |
| $R$      | Reliability    | Versprechen-Einhaltung          | $\to \gamma_{\text{budget}}$           |
| $I$      | Integrity      | Konsistenz                      | $\to w_{\text{vote}}$                  |
| $C$      | Competence     | Fähigkeitsnachweis              | $\to \text{Access}_{\text{complex}}$   |
| $P$      | Prestige       | Externe Attestation             | $\to \text{Influence}_{\text{social}}$ |
| $V$      | Vigilance      | Anomalie-Erkennung              | $\to w_{\text{protection}}$            |
| $\Omega$ | Omega (Wisdom) | $\int_{\text{past}}$ Handlungen | $\to \mu_{\text{regen}}$               |

### Τ4.2 Gewichtete Norm

$$\|\mathbb{T}\|_w = \sqrt{\sum_{d \in \{R,I,C,P,V,\Omega\}} w_d \cdot T_d^2}$$

**Invariante:** $\sum_d w_d = 1$

### Τ4.3 Asymmetrie-Axiom (Κ4)

$$\boxed{\Delta^-(d) = \lambda_d \cdot \Delta^+(d)}$$

| Dimension    | $\lambda$ | Interpretation        |
| ------------ | --------- | --------------------- |
| $R, I, C, P$ | 1.5       | Negative 50% stärker  |
| $V, \Omega$  | 2.0       | Negative 100% stärker |

**Beispiel:**
$$\tau_R^+ = +0.05 \implies \tau_R = 0.75$$
$$\tau_R^- = -0.05 \cdot 1.5 = -0.075 \implies \tau_R = 0.675$$

---

## §5 Trust-Level-Klassen

### Π5.1 Partitionierung

$$[0,1] = \bigcup_{k} \mathcal{T}_k$$

| Klasse      | $\tau$-Bereich | $\mu_{\max}$ | $\gamma_{\text{cost}}$ | Rechte                           |
| ----------- | -------------- | ------------ | ---------------------- | -------------------------------- |
| Newcomer    | $[0.0, 0.2)$   | 10k–30k      | $1.8\text{–}2.0\times$ | Basic only                       |
| Established | $[0.2, 0.5)$   | 30k–500k     | $1.5\text{–}1.8\times$ | Voting, Packages                 |
| Trusted     | $[0.5, 0.8)$   | 500k–800k    | $1.2\text{–}1.5\times$ | Publish (Review), Realm-Gründung |
| Veteran     | $[0.8, 1.0]$   | 800k–1M+     | $1.0\text{–}1.2\times$ | Publish (No-Review), Admin       |

### Π5.2 Sybil-Schutz-Theorem

$$\boxed{100 \cdot \tau_{\text{Newcomer}} < \tau_{\text{Veteran}}}$$

in sustained throughput.

---

## §6 Gas-Metriken $\gamma$

### Γ6.1 Budget-Emergenz

$$\gamma_{\text{budget}} = \gamma_{\text{base}} \cdot (1 + \tau_R \cdot \phi_\gamma)$$

wobei $\gamma_{\text{base}} = 10^6$ und $\phi_\gamma = 2.0$

| $\tau_R$ | $\gamma_{\text{budget}}$ |
| -------- | ------------------------ |
| 0.0      | 1,000,000                |
| 0.5      | 2,000,000                |
| 1.0      | 3,000,000                |

### Γ6.2 Kosten-Skalierung

$$\gamma_{\text{cost}}(\text{op}) = \gamma_{\text{base}}(\text{op}) \cdot (2 - \tau_R)$$

**Effekt:** High-Trust zahlt weniger $\implies$ Incentive für gutes Verhalten.

### Γ6.3 Erschöpfungs-Invariante (Κ11)

$$\boxed{\gamma(t+1) \leq \gamma(t) \quad\text{während Execution}}$$

**Monotonie:** Gas regeneriert NICHT.

### Γ6.4 OpCode-Kostentabelle

| OpCode        | $\gamma_{\text{base}}$ |
| ------------- | ---------------------- |
| PUSH/CONST    | 1                      |
| ADD/SUB       | 2                      |
| MUL           | 3                      |
| DIV/MOD       | 5                      |
| LOAD          | 5                      |
| STORE         | 10                     |
| BRANCH        | 3                      |
| CALL (base)   | 10                     |
| HOST_CALL     | 50                     |
| CRYPTO_VERIFY | 500                    |
| ZK_VERIFY     | 10,000                 |

---

## §7 Mana-Metriken $\mu$

### Μ7.1 Kapazitäts-Emergenz

$$\mu_{\max} = \mu_{\text{base}} \cdot (1 + \tau_\Omega \cdot \phi_\mu)$$

wobei $\mu_{\text{base}} = 10{,}000$ und $\phi_\mu = 100$

### Μ7.2 Regenerations-Funktion

$$\frac{d\mu}{dt} = r_{\text{base}} \cdot (1 + \tau_\Omega \cdot \psi_\mu)$$

wobei $r_{\text{base}} = 100/\text{sec}$ und $\psi_\mu = 10$

| $\tau_\Omega$ | $\mu_{\max}$ | $r$ (Mana/sec) |
| ------------- | ------------ | -------------- |
| 0.0           | 10,000       | 100            |
| 0.5           | 510,000      | 600            |
| 1.0           | 1,010,000    | 1,100          |

### Μ7.3 I/O-Kostentabelle

| Operation       | $\mu_{\text{base}}$ |
| --------------- | ------------------- |
| STORAGE_GET     | 5                   |
| STORAGE_PUT     | 10                  |
| STORAGE_PUT /KB | +10                 |
| STORAGE_DELETE  | 5                   |
| P2P_PUBLISH     | 10                  |
| P2P_CONNECT     | 20                  |
| P2P_DHT_PUT     | 20                  |
| P2P_GOSSIP      | 5                   |
| REALM_CROSSING  | 50                  |
| SAGA_STEP       | 30                  |

---

## §8 Kosten-Algebra $\kappa$

### Α8.1 Kostenvektor-Definition

$$\kappa = (\gamma, \mu, \rho) \in \mathbb{R}^+ \times \mathbb{R}^+ \times [0,1]$$

wobei $\rho$ = Trust-Risk.

### Α8.2 Kompositions-Operatoren

**Sequentiell ($\oplus$):**
$$\kappa_1 \oplus \kappa_2 = (\gamma_1 + \gamma_2, \mu_1 + \mu_2, 1 - (1-\rho_1)(1-\rho_2))$$

**Parallel ($\otimes$):**
$$\kappa_1 \otimes \kappa_2 = (\max(\gamma_1, \gamma_2), \mu_1 + \mu_2, \max(\rho_1, \rho_2))$$

### Α8.3 Trust-Adjustierung

$$\kappa_{\text{eff}} = (\gamma \cdot (2 - \tau_R), \mu, \rho)$$

---

## §9 Sybil-Attacke: Unmöglichkeits-Beweis

### Σ9.1 Vergleichsmatrix

| Metrik              | 1 Veteran ($\tau=0.9$) | 100 Sybils ($\tau=0.0$)               |
| ------------------- | ---------------------- | ------------------------------------- |
| $\mu_{\text{init}}$ | 910,000                | $100 \times 10{,}000 = 1{,}000{,}000$ |
| $r_{\text{total}}$  | 60,000/min             | 6,000/min                             |
| Sustained Rate      | 1,000/min              | 100/min                               |

### Σ9.2 Theorem

$$\boxed{\text{Rate}_{\text{Veteran}} = 10 \times \text{Rate}_{\text{Sybil-Cluster}}}$$

**Beweis:**
$$\frac{r_{\text{Veteran}}}{r_{\text{Sybil}}} = \frac{1000}{100} \cdot 1 = 10 \quad\square$$

---

## §10 Protection-Schicht (Self-Healing)

### Ρ10.1 CircuitBreaker-Zustände

$$\mathcal{B} = \{\text{NORMAL}, \text{DEGRADED}, \text{EMERGENCY}\}$$

| Zustand   | Anomalien/min  | Erlaubt             |
| --------- | -------------- | ------------------- |
| NORMAL    | $< 10$         | Alle Ops            |
| DEGRADED  | $10\text{–}50$ | Read + Running Exec |
| EMERGENCY | $\geq 50$      | Read only + Admin   |

**Transition:**
$$\text{NORMAL} \xrightarrow{A \geq 10} \text{DEGRADED} \xrightarrow{A \geq 50} \text{EMERGENCY}$$
$$\text{EMERGENCY} \xrightarrow{A < 10, 5\text{min}} \text{RECOVERY} \to \text{NORMAL}$$

### Ρ10.2 Calibration-Dynamik

**Gas-Preis:**
$$\gamma_{\text{mult}} = \begin{cases} 1.0 & \text{if } u_\gamma \leq 0.8 \\ 1.0 + (u_\gamma - 0.8) \cdot 5 & \text{if } u_\gamma > 0.8 \end{cases}$$

**Mana-Regeneration:**
$$r_{\text{mult}} = \begin{cases} 1.0 & \text{if } \text{spam} \leq \theta \\ 0.5 & \text{if } \text{spam} > \theta \end{cases}$$

### Ρ10.3 Health-Score-Formel

$$H = 0.40 \cdot H_\tau + 0.25 \cdot H_r + 0.20 \cdot (1 - A) + 0.10 \cdot H_{\text{iso}} + 0.05 \cdot D$$

wobei:

- $H_\tau = \bar{\tau} \cdot (1 - \sigma_\tau)$ (Trust-Balance)
- $H_r = 1 - \frac{|u_\gamma - 0.5| + |u_\mu - 0.5|}{2}$ (Resource-Balance)
- $A$ = normierte Anomalie-Rate
- $H_{\text{iso}}$ = Isolation-Integrity
- $D = 1 - \text{Gini}(\tau)$ (Diversity)

**Schwellwerte:**
$$H > 0.8 \implies \text{NORMAL}, \quad H < 0.5 \implies \text{CRITICAL}$$

---

## §11 Realm-Spezialisierung

### Ω11.1 Parameter-Override

$$\text{Realm}(\rho) : (\gamma_{\text{limit}}, r_\mu, \tau_{\min}})$$

| Realm-Typ | $\gamma_{\text{limit}}$ | $r_\mu$  | $\tau_{\min}$ |
| --------- | ----------------------- | -------- | ------------- |
| Compute   | $50 \times 10^6$        | 100/s    | 0.7           |
| Event     | $10^6$                  | 10,000/s | 0.2           |
| Secure    | $5 \times 10^5$         | 50/s     | 0.9           |
| Open      | $10^6$                  | 500/s    | 0.0           |

### Ω11.2 Lockern-Constraints

$$\gamma_{\text{limit}}^{\text{max}} \leq 100 \times \gamma_{\text{global}}$$
$$\text{Protection-Override}: \text{Anomalie} \implies \text{Strict-Mode}$$

---

## §12 StateGraph-Relationen

### Γ12.1 Dependency-Graph

```
Trust ←─[DependsOn]── Identity
Trust ──[Triggers]──→ Event
Gas ←─[DependsOn]── Trust
Gas ←─[DependsOn]── Calibration
Mana ←─[DependsOn]── Trust
Mana ←─[DependsOn]── Calibration
Execution ──[Aggregates]──→ {Gas, Mana}
Protection ──[Validates]──→ {Trust, Gas, Mana}
Calibration ──[Triggers]──→ {Gas, Mana}
```

---

## §13 Invarianten-Katalog

### Κ13.1 Fundamentale Invarianten

$$
\begin{aligned}
\text{Κ2:} \quad & \forall \iota: 0 \leq \tau(\iota) \leq 1 \\[4pt]
\text{Κ3:} \quad & \forall \delta: |\delta| \leq 0.1 \\[4pt]
\text{Κ4:} \quad & \Delta^- = \lambda \cdot \Delta^+, \quad \lambda \in \{1.5, 2.0\} \\[4pt]
\text{Κ11:} \quad & \gamma(t+1) \leq \gamma(t) \quad\text{(Erschöpfung)} \\[4pt]
\text{Κ13:} \quad & \mu(t+1) \geq \mu(t) - c + r \quad\text{(Regeneration)} \\[4pt]
\text{Κ19:} \quad & \text{Gini}(\tau_{\text{dist}}) < 0.8 \\[4pt]
\text{Κ23:} \quad & \tau_{\text{eff}}(\iota, \rho \to \rho') = \tau(\iota) \cdot f(\rho, \rho') \\[4pt]
\text{Κ24:} \quad & \tau(\iota, \rho_1) \perp \tau(\iota, \rho_2)
\end{aligned}
$$

---

## §14 Request-Lifecycle-Automat

### Φ14.1 Zustandsdiagramm

$$\text{Request} \xrightarrow{L_1} \text{Gateway} \xrightarrow{L_{2,3}} \text{Budget} \xrightarrow{L_4} \text{Execution} \xrightarrow{L_{5\text{-}7}} \text{Aftermath} \to \text{Response}$$

### Φ14.2 Transitionsfunktionen

**Gateway ($L_1$):**
$$\text{Pass} \iff (\text{rate} < \text{limit}) \land (\tau \geq \tau_{\min}) \land (\mu \geq \kappa_{\text{est}})$$

**Execution ($L_4$):**
$$\forall \text{op}: \gamma \gets \gamma - \kappa_\gamma(\text{op}) \cdot (2 - \tau_R)$$
$$\gamma < 0 \implies \text{Abort} \land \tau \mathrel{-}= \delta_{\text{fail}} \cdot \lambda$$

**Aftermath ($L_{5\text{-}7}$):**
$$\mu \gets \mu - \mu_{\text{consumed}}$$
$$\tau \gets \tau + \delta_{\text{result}} \cdot (\pm\lambda)$$

---

## §15 Zusammenfassung

$$
\boxed{\begin{array}{rcl}
\tau &\equiv& \text{Emergent Trust (Immunsystem)} \\
\gamma &\equiv& f(\tau) \text{ mit Erschöpfung (Muskelkraft)} \\
\mu &\equiv& f(\tau) \text{ mit Regeneration (Atem)}
\end{array}}
$$

**Kerntheorem:**
$$\mathcal{S} = \mathcal{L}_7 \circ (\tau, \gamma, \mu) \implies \text{Self-Healing} \land \text{Angreifer-Erschöpfung}$$

---

**∎ QED**
