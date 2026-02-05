# Pluto::RealmGovernance ≡ Souveräne Entscheidungsfindung

> **Notation:** Pluto (komprimiert-formal)
> **Version:** 1.0 | **Datum:** 2026-02

---

## §1 Governance-Modell – Formaldefinition

### Δ1.1 Axiom: Realm-Exklusivität

$$\boxed{\mathcal{G} \iff \exists\, \mathcal{R} : \mathcal{G} \subseteq \mathcal{R}}$$

**Begründung:**

$$
\begin{aligned}
\neg\mathcal{G}(\text{Identity}) &\quad \text{(keine Identitäts-Governance)} \\
\neg\mathcal{G}(\text{Package}) &\quad \text{(keine Package-Governance)} \\
\neg\mathcal{G}(\text{Global}) &\quad \text{(keine globale direktdemokratische Governance)}
\end{aligned}
$$

### Δ1.2 Stimmgewicht-Hauptformel

$$\boxed{W(m) = G(m) \cdot \left(1 + \alpha \cdot T_{\text{rel}}(m)\right)}$$

**Komponenten:**

- $W(m)$ — Finales Stimmgewicht des Members $m$
- $G(m)$ — Governance-Basis-Gewicht (aus GovernanceType)
- $\alpha \in [0, 1]$ — Trust-Einfluss-Faktor (Realm-konfiguriert)
- $T_{\text{rel}}(m)$ — Relativer Trust im Realm

### Δ1.3 Symboltafel

| Symbol           | Definition               | Domäne                              |
| ---------------- | ------------------------ | ----------------------------------- |
| $\mathcal{G}$    | Governance-System        | $\mathcal{G} \subseteq \mathcal{R}$ |
| $W$              | Stimmgewicht-Funktion    | $\mathbb{R}_{\geq 0}$               |
| $G$              | Basis-Gewicht            | $\mathbb{R}_{\geq 0}$               |
| $\alpha$         | Trust-Einfluss           | $[0, 1]$                            |
| $T_{\text{rel}}$ | Relativer Trust          | $[-1, 1]$                           |
| $T_{\text{avg}}$ | Realm-Trust-Durchschnitt | $(0, 1]$                            |
| $\mathcal{P}$    | Proposal-Menge           | $\mathcal{P} \subseteq \mathcal{R}$ |
| $q$              | Quorum                   | $(0, 1]$                            |
| $\theta$         | Approval-Threshold       | $(0.5, 1]$                          |

---

## §2 Relativer Trust

### Τ2.1 Definition

$$\boxed{T_{\text{rel}}(m) = \frac{T(m) - T_{\text{avg}}}{T_{\text{avg}}}}$$

**Eigenschaften:**

$$
\begin{aligned}
T(m) > T_{\text{avg}} &\implies T_{\text{rel}}(m) > 0 \quad \text{(Bonus)} \\
T(m) = T_{\text{avg}} &\implies T_{\text{rel}}(m) = 0 \quad \text{(Neutral)} \\
T(m) < T_{\text{avg}} &\implies T_{\text{rel}}(m) < 0 \quad \text{(Malus)}
\end{aligned}
$$

### Τ2.2 Aggregierter Trust

$$T(m) = \frac{\sum_{d \in \mathcal{D}} w_d \cdot T_d(m)}{\sum_{d \in \mathcal{D}} w_d}$$

**Trust-Dimensionen:** $\mathcal{D} = \{R, I, C, P, V, \Omega\}$

**Standard-Gewichtung:**
$$\mathbf{w} = (1.0, 1.0, 1.0, 1.0, 1.0, 2.0)^T$$

$\Omega$ (Axiom-Treue) ist doppelt gewichtet.

---

## §3 Governance-Typen

### Γ3.1 Typ-Algebra

$$
\boxed{G(m) = \begin{cases}
\sqrt{\tau(m)} & \text{Quadratic} \\
\tau(m) & \text{Token} \\
T(m) & \text{Reputation} \\
1 & \text{MemberEqual} \\
G_{\text{base}}(m) + \sum_{d \in D(m)} G(d) \cdot \delta^{\text{depth}(d)} & \text{Delegated}
\end{cases}}
$$

**Wobei:**

- $\tau(m)$ — Token-Balance
- $D(m)$ — Delegatoren zu $m$
- $\delta \in (0, 1)$ — Delegation-Decay-Faktor

### Γ3.2 Vergleichsmatrix

| Typ         | $G(m)$        | $\alpha$ | Anwendung         |
| ----------- | ------------- | -------- | ----------------- |
| Quadratic   | $\sqrt{\tau}$ | Optional | DAOs              |
| Token       | $\tau$        | Optional | Investment-DAOs   |
| Reputation  | $T$           | $1.0$    | Merit-Guilds      |
| Delegated   | rekursiv      | Via Base | Große Communities |
| MemberEqual | $1$           | Optional | Cooperatives      |

### Γ3.3 Trust-Einfluss-Skalierung

$$W_\alpha(m) = G(m) \cdot (1 + \alpha \cdot T_{\text{rel}}(m))$$

| $\alpha$ | $T_{\text{rel}} = +1$ | $T_{\text{rel}} = -0.5$ |
| -------- | --------------------- | ----------------------- |
| $0.0$    | $1.00 \cdot G$        | $1.00 \cdot G$          |
| $0.5$    | $1.50 \cdot G$        | $0.75 \cdot G$          |
| $1.0$    | $2.00 \cdot G$        | $0.50 \cdot G$          |

---

## §4 Liquid Democracy (Delegated)

### Λ4.1 Delegation-Relation

$$\mathcal{D} \subseteq \mathcal{M} \times \mathcal{M}$$

$$(m_1, m_2) \in \mathcal{D} \iff m_1 \text{ delegiert an } m_2$$

### Λ4.2 Delegation mit Trust-Decay (Κ8)

$$\boxed{W_{\text{del}}(m) = G(m) + \sum_{d \in D(m)} G(d) \cdot t_d^{\text{depth}(d)}}$$

**Wobei:**

- $t_d$ — Trust des Delegators $d$
- $\text{depth}(d)$ — Kettentiefe der Delegation

**Invarianten:**

$$
\begin{aligned}
\text{depth}(d) &\leq \text{depth}_{\max} \\
t_d^{\text{depth}(d)} &\geq \delta_{\min}
\end{aligned}
$$

---

## §5 Proposal-Lifecycle

### Π5.1 Zustandsautomat

$$\mathcal{S}_\mathcal{P} = \{\text{Draft}, \text{Discussion}, \text{Voting}, \text{Timelock}, \text{Executed}, \text{Defeated}, \text{Vetoed}\}$$

**Transitionen:**

$$
\begin{aligned}
\text{Draft} &\xrightarrow{\text{submit}} \text{Discussion} \\
\text{Discussion} &\xrightarrow{t \geq t_{\text{disc}}} \text{Voting} \\
\text{Voting} &\xrightarrow{v \geq q \land a \geq \theta} \text{Timelock} \\
\text{Voting} &\xrightarrow{v < q \lor a < \theta} \text{Defeated} \\
\text{Timelock} &\xrightarrow{t \geq t_{\text{lock}} \land \neg\text{veto}} \text{Executed} \\
\text{Timelock} &\xrightarrow{\text{veto} \geq \theta_v} \text{Vetoed}
\end{aligned}
$$

### Π5.2 Quorum & Approval

$$\boxed{\text{accepted} \iff \left(\frac{\sum W_{\text{voted}}}{\sum W_{\text{total}}} \geq q\right) \land \left(\frac{\sum W_{\text{for}}}{\sum W_{\text{voted}}} \geq \theta\right)}$$

**Dynamisches Quorum:**
$$q_{\text{dyn}} = \min\left(q_{\text{base}} + \beta \cdot \text{participation}_{\text{history}}, q_{\max}\right)$$

---

## §6 Proposal-Kategorien

### Κ6.1 Kategorie-Thresholds

$$\boxed{\theta_c = f_c(\text{severity})}$$

| Kategorie        | $\theta$ | Timelock | Supermajority |
| ---------------- | -------- | -------- | ------------- |
| ParameterChange  | $0.50$   | 24h      | ✗             |
| TreasurySpend    | $0.60$   | 48h      | ✗             |
| RuleChange       | $0.67$   | 72h      | ✓             |
| MemberAction     | $0.75$   | 24h      | ✓             |
| GovernanceChange | $0.80$   | 7d       | ✓             |

---

## §7 Veto-Mechanismus

### Ω7.1 Veto-Threshold

$$\boxed{\text{vetoed} \iff \frac{\sum W_{\text{veto}}}{\sum W_{\text{total}}} \geq \theta_v}$$

**Typisch:** $\theta_v = 0.33$

### Ω7.2 Veto-Window

$$t_{\text{veto}} \in [t_{\text{approval}}, t_{\text{approval}} + \Delta t_{\text{veto}}]$$

**Schutzziel:** Minderheitenschutz gegen übereilte Mehrheitsentscheidungen.

---

## §8 Anti-Sybil durch Trust

### Σ8.1 Newcomer-Dämpfung

$$\text{newcomer: } T(m) = 0.1 \implies T_{\text{rel}} = \frac{0.1 - 0.6}{0.6} = -0.83$$

**Mit $\alpha = 0.5$:**
$$W(m) = G(m) \cdot (1 + 0.5 \cdot (-0.83)) = 0.58 \cdot G(m)$$

### Σ8.2 Sybil-Kosten

$$\text{Cost}_{\text{sybil}} = n \cdot (\text{Mana}_{\text{join}} + t_{\text{trust-build}} \cdot \text{effort})$$

**Asymmetrie (Κ4):** Trust-Aufbau dauert lange, Abbau ist schnell.

---

## §9 Resource-Kosten

### Ρ9.1 Governance-Mana

$$
\begin{aligned}
\text{Mana}_{\text{proposal}} &= \kappa_p \quad \text{(Anti-Spam)} \\
\text{Mana}_{\text{vote}} &= \kappa_v \quad \text{(minimal)} \\
\text{Mana}_{\text{execute}} &\leq \mu_{\max}
\end{aligned}
$$

### Ρ9.2 Token-Deposit

$$\text{Deposit} = \tau_d \quad \text{(refundable bei Nicht-Spam)}$$

---

## §10 Trust-Bidirektionale Kopplung

### Β10.1 Governance → Trust

$$
\Delta T = \begin{cases}
+0.02 & \text{ProposalAccepted} \\
-0.01 & \text{ProposalRejected} \\
-0.10 & \text{ProposalSpam} \\
+0.005 & \text{VotingParticipation} \\
+0.01 & \text{DelegationReceived} \\
+0.02 & \text{SuccessfulVeto}
\end{cases}
$$

### Β10.2 Trust → Governance

$$W(m) = G(m) \cdot (1 + \alpha \cdot T_{\text{rel}}(m))$$

---

## §11 StateEvents

### Ε11.1 Event-Typen

$$\mathcal{E}_\mathcal{G} = \{\text{ProposalCreated}, \text{VoteCast}, \text{DelegationCreated}, \text{ProposalExecuted}, \text{ProposalVetoed}\}$$

### Ε11.2 Event-Schema

$$e_{\text{vote}} = \langle \text{realm}, \text{proposal}, \text{voter}, \text{vote} \in \{\text{For}, \text{Against}, \text{Abstain}\}, W \rangle$$

---

## §12 Zusammenfassung

### Ζ12.1 Governance-DNA

$$\boxed{\mathcal{G} = \langle W, T_{\text{rel}}, \alpha, \mathcal{P}, q, \theta, \theta_v \rangle}$$

### Ζ12.2 Invarianten

$$
\begin{aligned}
&\forall m \in \mathcal{M}: W(m) \geq 0 \\
&\forall p \in \mathcal{P}: \text{state}(p) \in \mathcal{S}_\mathcal{P} \\
&\forall \text{category}: \theta_c \in (0.5, 1] \\
&\alpha \in [0, 1] \implies \text{Trust-Einfluss begrenzt} \\
&T_{\text{rel}} = 0 \iff T(m) = T_{\text{avg}} \quad \text{(Neutralität)}
\end{aligned}
$$

### Ζ12.3 Pluto-Integration

| Konstante | Integration                                  |
| --------- | -------------------------------------------- |
| Κ4        | Asymmetrische Trust-Evolution → Sybil-Schutz |
| Κ8        | Trust-Decay bei Delegation                   |
| Κ17/Κ18   | Membership als Governance-Basis              |
| Κ24       | Lokaler Trust für $T_{\text{rel}}$           |

---

**∎ QED**
