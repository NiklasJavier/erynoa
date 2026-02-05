# Pluto::MathAnalysis ≡ Weltformel V6

> **Notation:** Pluto (komprimiert-formal)
> **Version:** 6.0 | **Datum:** 2026-02

---

## §1 Weltformel – Formaldefinition

### Δ1.1 Hauptformel

$$\boxed{\mathbb{E} = \sum_{s \in \mathcal{C}} \mathbb{A}(s) \cdot \sigma\!\left(\frac{\|\mathbb{W}(s)\|_w \cdot \ln(|\mathbb{C}(s)|+1) \cdot \mathcal{S}(s)}{\kappa}\right) \cdot \hat{H}(s) \cdot w(s,t)}$$

### Δ1.2 Symboltafel

| Symbol | Definition | Domäne |
|--------|-----------|--------|
| $\mathcal{C}$ | Subjektmenge (DIDs) | $\mathcal{C} \subseteq \text{DID}$ |
| $\mathbb{A}$ | Aktivitätsfunktion | $[0,1)$ |
| $\sigma$ | Sigmoidfunktion | $(0,1)$ |
| $\mathbb{W}$ | Trust-Vektor | $\mathbb{R}^6$ |
| $\mathbb{C}$ | Kausale Historie | $\mathbb{N}$ |
| $\mathcal{S}$ | Trust-gedämpfte Surprisal | $\mathbb{R}_{\geq 0}$ |
| $\hat{H}$ | Human-Faktor | $\{1.0, 1.2, 1.5\}$ |
| $w$ | Temporalgewicht | $[0,1]$ |
| $\kappa$ | Skalierungskonstante | $15.0$ |

---

## §2 Komponentenaxiome

### Α2.1 Aktivität $\mathbb{A}$

$$\mathbb{A}(s) \coloneqq \frac{n}{n + \kappa_A} \quad\text{mit}\quad \kappa_A = 10$$

**Eigenschaften:**
$$\begin{aligned}
\text{(i)}\quad   & \mathbb{A} : \mathbb{N} \to [0,1) \\
\text{(ii)}\quad  & \frac{d\mathbb{A}}{dn} = \frac{\kappa_A}{(n+\kappa_A)^2} > 0 \quad\text{(monoton)} \\
\text{(iii)}\quad & \frac{d^2\mathbb{A}}{dn^2} < 0 \quad\text{(konkav → diminishing returns)}
\end{aligned}$$

---

### Σ2.2 Sigmoid $\sigma$

$$\sigma(x) \coloneqq \frac{1}{1 + e^{-x}}$$

**Eigenschaften:**
$$\begin{aligned}
\sigma(0) &= 0.5 \\
\sigma'(x) &= \sigma(x)(1-\sigma(x)) \\
\lim_{x\to-\infty}\sigma(x) &= 0, \quad \lim_{x\to+\infty}\sigma(x) = 1
\end{aligned}$$

**Skalierungsproblem V5:**
$$\text{inner}_{\max} \approx 2817.5 \implies \sigma(\text{inner}) \approx 1 - 10^{-1223}$$

**Lösung V6:**
$$\sigma\!\left(\frac{\text{inner}}{\kappa}\right) \quad\text{mit}\quad \kappa = \frac{\text{inner}_{\text{typ}}}{0.8} = 15$$

---

### Ω2.3 Trust-Norm $\|\mathbb{W}\|_w$

$$\|\mathbb{W}\|_w \coloneqq \sqrt{\sum_{i=1}^{6} w_i \cdot W_i^2}$$

**Trust-Dimensionen:** $\mathbb{W} = (R, I, C, P, V, \Omega)$

**Gewichtsmatrix:**
$$\mathbf{w}_{\text{ctx}} = \begin{pmatrix} w_R & w_I & w_C & w_P & w_V & w_\Omega \end{pmatrix}$$

| ctx | R | I | C | P | V | Ω |
|-----|---|---|---|---|---|---|
| Default | .17 | .17 | .17 | .17 | .16 | .16 |
| Finance | .25 | .25 | .15 | .15 | .10 | .10 |
| Social | .10 | .15 | .10 | .30 | .25 | .10 |
| Governance | .15 | .20 | .10 | .10 | .10 | .35 |
| Technical | .15 | .15 | .35 | .10 | .15 | .10 |

**Invariante:** $\sum_i w_i = 1$

---

### Κ2.4 Kausale Konnektivität

$$\text{V5:}\quad \ln(|\mathbb{C}|) \quad\xrightarrow{\text{Problem: }\ln(1)=0}\quad \text{V6:}\quad \ln(|\mathbb{C}|+1)$$

**Laplace-Smoothing-Analogie:**
$$P_{\text{smooth}}(x) = \frac{\text{count}(x) + 1}{\text{total} + |V|}$$

**Grenzwert:**
$$\lim_{|\mathbb{C}|\to\infty} \frac{\ln(|\mathbb{C}|+1)}{\ln(|\mathbb{C}|)} = 1$$

---

### Ψ2.5 Trust-gedämpfte Surprisal

$$\mathcal{S}(s) \coloneqq \|\mathbb{W}(s)\|^2 \cdot \mathcal{I}(s)$$

**Shannon-Surprisal:**
$$\mathcal{I}(e|s) = -\log_2 P(e|\mathbb{C}(s))$$

**Smoothed Probability:**
$$P(e|\mathbb{C}) = \frac{\text{freq}(e) + 1}{\text{total} + 2}$$

**Skalierungseffekt:**
$$\frac{\mathcal{S}_{\text{high-trust}}}{\mathcal{S}_{\text{low-trust}}} = \frac{0.81 \cdot \mathcal{I}}{0.09 \cdot \mathcal{I}} = 9$$

---

## §3 Theoreme

### Τ3.1 Chain-Trust

$$\boxed{t_{\text{chain}} = \left(\prod_{i=1}^{n} t_i\right)^{1/\sqrt{n}}}$$

**Beweis der Eigenschaften:**

$$\begin{aligned}
\text{(i) Identität:}\quad   & t_{\text{chain}}([t_1]) = t_1^{1/1} = t_1 \quad\checkmark \\[6pt]
\text{(ii) Monotonie:}\quad & t_{\text{chain}} = t^{\sqrt{n}} \quad\text{(fällt für } t < 1\text{)} \\[6pt]
\text{(iii) Dämpfung:}\quad & t^{\sqrt{n}} > t^n \quad\text{für } n > 1, t \in (0,1)
\end{aligned}$$

**V5→V6 Fix:** Numerische Stabilität via direkter Produktberechnung statt log-sum-exp.

---

### Τ3.2 Beschränktheit

$$\mathbb{E} < 1.5 \cdot |\mathcal{C}|$$

**Beweis:**
$$\text{contribution}(s) = \underbrace{\mathbb{A}}_{<1} \cdot \underbrace{\sigma}_{<1} \cdot \underbrace{\hat{H}}_{\leq 1.5} \cdot \underbrace{w}_{\leq 1} < 1.5$$

---

### Τ3.3 Inkrementelles Update

$$\mathbb{E}_{\text{new}} = \mathbb{E}_{\text{old}} - c_{\text{old}}(s) + c_{\text{new}}(s)$$

**Korrektheit:** Folgt aus Linearität von $\sum$. $\square$

---

## §4 Sensitivitätsanalyse

### ∂4.1 Partielle Ableitungen

$$\frac{\partial \mathbb{E}}{\partial \|\mathbb{W}\|} = \mathbb{A} \cdot \underbrace{\sigma'}_{\sigma(1-\sigma)} \cdot \frac{\ln(|\mathbb{C}|+1) \cdot \mathcal{S}}{\kappa} \cdot \hat{H} \cdot w$$

$$\frac{\partial \mathbb{E}}{\partial |\mathbb{C}|} = \mathbb{A} \cdot \sigma' \cdot \frac{\|\mathbb{W}\| \cdot \mathcal{S}}{\kappa \cdot (|\mathbb{C}|+1)} \cdot \hat{H} \cdot w$$

**Max-Sensitivität:** $\sigma \approx 0.5 \implies \sigma' = 0.25$

---

### ∂4.2 κ-Sensitivität

| $\kappa$ | $\sigma_{\text{newcomer}}$ | $\sigma_{\text{etabliert}}$ | Differenzierung |
|----------|---------------------------|----------------------------|-----------------|
| 10 | 0.52 | 0.998 | saturiert |
| **15** | **0.51** | **0.88** | **optimal** |
| 20 | 0.50 | 0.82 | komprimiert |
| 30 | 0.50 | 0.73 | überkomprimiert |

---

## §5 Zusammenfassung

$$\begin{array}{|l|c|l|}
\hline
\textbf{Komponente} & \textbf{Fix V6} & \textbf{Begründung} \\
\hline
\text{Sigmoid} & \sigma(x/\kappa) & \text{Sensitiver Bereich } [0.05, 0.95] \\
\text{Konnektivität} & \ln(|\mathbb{C}|+1) & \text{Laplace-Smoothing} \\
\text{Chain-Trust} & \text{Direkt-Produkt} & \text{Numerische Stabilität} \\
\hline
\end{array}$$

---

**∎ QED**
