# Pluto::Validation ≡ Weltformel V6

> **Notation:** Pluto (komprimiert-formal)
> **Version:** 6.0 | **Datum:** 2026-02

---

## §1 Testmatrix

### Τ1.1 Testkategorien

$$\begin{array}{|l|l|l|}
\hline
\textbf{Kategorie} & \textbf{Fokus} & \textbf{Methodik} \\
\hline
\text{Unit} & \text{Grenzwerte, Numerik} & \text{Deterministisch} \\
\text{Integration} & \text{E2E, Pipeline} & \text{Szenario-basiert} \\
\text{Property} & \text{Invarianten} & \text{Generativ } (n=10^4) \\
\text{Regression} & \text{V5-Bugs} & \text{Spezifisch} \\
\hline
\end{array}$$

---

## §2 Unit-Tests: Σigmoid

### Υ2.1 test_sigmoid_scaling_fix

**Prädikat:**
$$\forall c \in \mathcal{C}_{\text{high}}: 0.3 < c.\text{compute}() < 0.95$$

**Szenarien:**
$$\begin{array}{|l|c|c|c|}
\hline
\textbf{Profil} & \textbf{V5} & \textbf{V6} & \textbf{Status} \\
\hline
\text{High} & \approx 0.99 & \approx 0.69 & \checkmark \\
\text{Medium} & \approx 0.95 & \approx 0.55 & \checkmark \\
\text{Low} & \approx 0.80 & \approx 0.40 & \checkmark \\
\hline
\end{array}$$

---

### Υ2.2 test_ln_offset_fix

**Prädikat:**
$$|\mathbb{C}| = 1 \implies c.\text{compute}() > 0$$

**Beweis:**
$$\begin{aligned}
\text{V5:}\quad & \ln(\max(1,1)) = \ln(1) = 0 \implies \text{inner} = 0 \\
\text{V6:}\quad & \ln(1+1) = \ln(2) \approx 0.693 \implies \text{inner} > 0
\end{aligned}$$

**Trust-Erhaltung:**
$$\frac{\partial\, c.\text{compute}()}{\partial\, \|\mathbb{W}\|} \neq 0 \quad\text{für}\quad |\mathbb{C}| = 1$$

---

## §3 Unit-Tests: Chain-Trust

### Χ3.1 test_chain_trust_corrected_formula

**Eigenschaften:**

$$\begin{aligned}
\text{P1 (Identität):}\quad   & t_{\text{chain}}([t]) = t \\
\text{P2 (Monotonie):}\quad   & n_1 < n_2, t < 1 \implies t_{\text{chain}}^{(n_1)} > t_{\text{chain}}^{(n_2)} \\
\text{P3 (Dämpfung):}\quad    & t_{\text{chain}}([0.8;4]) > 0.5 \quad\text{(V5: } \approx 0.25\text{)}
\end{aligned}$$

**Erwartungswerte:**

$$\begin{array}{|l|c|c|l|}
\hline
\textbf{Kette} & \textbf{V5} & \textbf{V6} & \textbf{Formel V6} \\
\hline
[0.8] & 0.80 & 0.80 & 0.8^1 \\
[0.8;2] & 0.57 & 0.72 & 0.64^{0.707} \\
[0.8;4] & 0.26 & 0.64 & 0.41^{0.5} \\
[0.9;10] & 0.35 & 0.71 & 0.35^{0.316} \\
\hline
\end{array}$$

---

## §4 Integration-Tests

### Ι4.1 test_full_contribution_pipeline_v6

**Entitätsprofile:**
$$\begin{array}{|l|cccccc|}
\hline
\textbf{Name} & n & \tau & \alpha & |\mathbb{C}| & \mathcal{I} & \hat{H} \\
\hline
\text{Alice} & 1000 & 0.9 & 0.8 & 500 & 1.5 & 1.2 \\
\text{Bob} & 100 & 0.7 & 0.7 & 50 & 2.0 & 1.0 \\
\text{Carol} & 5000 & 0.95 & 0.85 & 2000 & 0.5 & 1.5 \\
\text{Dave} & 1 & 0.5 & 0.6 & 1 & 3.0 & 1.0 \\
\hline
\end{array}$$

**Prädikate:**
$$\begin{aligned}
\text{(A1)}\quad & \forall e: c(e) > 0 \\
\text{(A2)}\quad & \forall e: c(e) < 0.95 \\
\text{(A3)}\quad & \text{Var}(\{c(e)\}) > 0.01 \\
\text{(A4)}\quad & c(\text{Dave}) > 0.1
\end{aligned}$$

---

### Ι4.2 test_trust_propagation_v6

**Kette:** $A \xrightarrow{0.9} B \xrightarrow{0.85} C \xrightarrow{0.8} D \xrightarrow{0.75} E \xrightarrow{0.7}$

$$t_{\text{prop}} = (0.9 \cdot 0.85 \cdot 0.8 \cdot 0.75 \cdot 0.7)^{1/\sqrt{5}} = 0.321^{0.447} \approx 0.608$$

**Prädikate:**
$$\begin{aligned}
& t_{\text{prop}} > 0 \\
& t_{\text{prop}} < \min_i(t_i) \\
& t_{\text{prop}} > 0.5 \quad\text{(V5-Bug: } \approx 0.28\text{)}
\end{aligned}$$

---

## §5 Property-Based Tests

### Π5.1 Monotonie

$$\forall x_1, x_2 \in \mathbb{R}: x_1 < x_2 \implies \sigma(x_1/\kappa) < \sigma(x_2/\kappa)$$

$$\forall n_1, n_2 \in \mathbb{N}, t \in (0,1): n_1 < n_2 \implies t_{\text{chain}}^{(n_1)} \geq t_{\text{chain}}^{(n_2)}$$

### Π5.2 Beschränktheit

$$\forall \|\mathbb{W}\| \in [0,1], |\mathbb{C}| \in \mathbb{N}, \mathcal{S} \in \mathbb{R}_{\geq 0}: \sigma\left(\frac{\|\mathbb{W}\| \cdot \ln(|\mathbb{C}|+1) \cdot \mathcal{S}}{\kappa}\right) \in (0,1)$$

$$\forall \vec{t} \in [0,1]^n: t_{\text{chain}}(\vec{t}) \in [0, \max_i(t_i)]$$

### Π5.3 Spezialfälle

$$\begin{aligned}
\text{(S1)}\quad & t_{\text{chain}}([t]) = t \\
\text{(S2)}\quad & \exists\, t_i = 0 \implies t_{\text{chain}} \approx 0 \\
\text{(S3)}\quad & \forall n \in \mathbb{N}: \ln(n+1) \geq 0
\end{aligned}$$

---

## §6 Regressionstests

### Ρ6.1 V5-Bug: Sigmoid-Sättigung

$$\text{Bug}: c_{\text{high}} \approx c_{\text{medium}} \approx 0.99$$

$$\text{Fix}: |c_{\text{high}} - c_{\text{medium}}| > 0.1$$

---

### Ρ6.2 V5-Bug: Neue Entitäten unsichtbar

$$\text{Bug}: |\mathbb{C}| = 1 \implies \frac{\partial c}{\partial \|\mathbb{W}\|} = 0$$

$$\text{Fix}: c(\tau_{\text{high}}) - c(\tau_{\text{low}}) > 0.05 \quad\text{bei}\quad |\mathbb{C}| = 1$$

---

### Ρ6.3 V5-Bug: Chain-Trust zu streng

$$\text{Bug}: t_{\text{chain}}([0.8;4]) \approx 0.25$$

$$\text{Fix}: t_{\text{chain}}([0.8;4]) \in (0.5, 0.85)$$

---

## §7 Test-Fixtures

### Φ7.1 Entitäts-Generatoren

$$\begin{aligned}
\text{new\_user}() &\equiv \{n=1, \alpha=1, \tau=\text{default}, |\mathbb{C}|=1, \mathcal{I}=3.0\} \\
\text{active}() &\equiv \{n=500, \alpha=50/90, \tau=0.8, |\mathbb{C}|=200, \mathcal{I}=1.5\} \\
\text{veteran}() &\equiv \{n=10^4, \alpha=1.0, \tau=0.95, |\mathbb{C}|=5000, \mathcal{I}=0.3\} \\
\text{suspicious}() &\equiv \{n=50, \alpha=50/2, \tau=0.3, |\mathbb{C}|=10, \mathcal{I}=4.5\}
\end{aligned}$$

### Φ7.2 Trust-Ketten

$$\begin{aligned}
\text{typical}() &\equiv [0.9, 0.85, 0.8, 0.75, 0.7] \\
\text{long}() &\equiv [0.8;10] \\
\text{weak\_link}() &\equiv [0.9, 0.9, 0.3, 0.9, 0.9]
\end{aligned}$$

---

## §8 Performance-Benchmarks

### Β8.1 Komplexitätsbeweis

$$\mathcal{O}(\sigma) = \mathcal{O}(1), \quad \mathcal{O}(t_{\text{chain}}) = \mathcal{O}(n)$$

### Β8.2 Toleranzgrenzen

$$\Delta t_{\text{V5} \to \text{V6}} < 5\%$$

---

## §9 Validierungs-Checkliste

$$\begin{array}{|l|c|}
\hline
\textbf{Kriterium} & \textbf{Status} \\
\hline
\forall\, \text{Unit-Tests}: \text{pass} & \square \\
\forall\, \text{Integration-Tests}: \text{pass} & \square \\
\text{Property-Tests } (n=10^4): \text{pass} & \square \\
\text{Regressions-Tests V5}: \text{pass} & \square \\
\text{Benchmarks}: \Delta < 5\% & \square \\
\text{Numerische Stabilität}: \text{validiert} & \square \\
\text{Dokumentation} \equiv \text{Implementation} & \square \\
\hline
\end{array}$$

---

## §10 CI/CD-Integration

$$\text{Trigger}: \Delta(\texttt{formula.rs}) \lor \Delta(\texttt{trust.rs})$$

$$\text{Pipeline}: \text{Unit} \to \text{Property}_{10^4} \to \text{Bench} \to \text{Validate}$$

---

**∎ QED**
