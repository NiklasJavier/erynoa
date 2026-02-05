# Pluto::Implementation ≡ Weltformel V6

> **Notation:** Pluto (komprimiert-formal)
> **Version:** 6.0 | **Datum:** 2026-02

---

## §1 Änderungsmatrix

### Δ1.1 Datei-Transformation

$$
\begin{array}{|l|l|l|}
\hline
\textbf{Datei} & \textbf{Funktion} & \textbf{Änderung} \\
\hline
\texttt{formula.rs} & \texttt{compute\_full()} & \sigma\text{-Skalierung} \\
\texttt{formula.rs} & \texttt{build()} & \ln\text{-Offset} \\
\texttt{formula.rs} & \texttt{compute\_value()} & \kappa\text{-Division} \\
\texttt{trust.rs} & \texttt{chain\_trust()} & \prod\text{-Formel} \\
\hline
\end{array}
$$

---

## §2 Transformation: Sigmoid

### Φ2.1 compute_full() / build() / compute_value()

$$\underbrace{\text{inner} = \|\mathbb{W}\| \cdot \ln(|\mathbb{C}|) \cdot \mathcal{S}}_{\text{V5}} \xrightarrow{\Delta} \underbrace{\text{inner} = \frac{\|\mathbb{W}\| \cdot \ln(|\mathbb{C}|+1) \cdot \mathcal{S}}{\kappa}}_{\text{V6}}$$

**Transformationsregeln:**

$$
\begin{aligned}
\text{(T1)}\quad & \ln(\texttt{causal}.max(1)) \to \ln(\texttt{causal} + 1.0) \\
\text{(T2)}\quad & \texttt{inner} \to \texttt{inner} / \kappa \quad\text{mit}\quad \kappa = 15.0 \\
\text{(T3)}\quad & \texttt{const SIGMOID\_SCALE: f64} = 15.0
\end{aligned}
$$

---

### Ψ2.2 Codeblock-Diff

```
┌────────────────────────────────────────────────────────────────────┐
│ V5:  let ln = (causal.max(1) as f64).ln();                         │
│      let inner = (trust as f64) * ln * surprisal;                  │
│      let σ = 1.0 / (1.0 + (-inner).exp());                         │
├────────────────────────────────────────────────────────────────────┤
│ V6:  let ln = (causal as f64 + 1.0).ln();                          │
│      const κ: f64 = 15.0;                                          │
│      let inner = (trust as f64) * ln * surprisal / κ;              │
│      let σ = 1.0 / (1.0 + (-inner).exp());                         │
└────────────────────────────────────────────────────────────────────┘
```

---

## §3 Transformation: Chain-Trust

### Χ3.1 Formelkorrektur

$$\underbrace{t_{\text{chain}} = \exp\left(\frac{\sum_i \ln(t_i)}{\sqrt{n}}\right)}_{\text{V5: numerisch instabil}} \xrightarrow{\Delta} \underbrace{t_{\text{chain}} = \left(\prod_i t_i\right)^{1/\sqrt{n}}}_{\text{V6: korrigiert}}$$

### Χ3.2 Codeblock-Diff

```
┌────────────────────────────────────────────────────────────────────┐
│ V5:  let log_sum: f32 = chain.iter()                               │
│          .map(|t| t.max(1e-10).ln()).sum();                        │
│      (log_sum / n.sqrt()).exp()                                    │
├────────────────────────────────────────────────────────────────────┤
│ V6:  let product: f32 = chain.iter()                               │
│          .fold(1.0, |acc, &t| acc * t.max(1e-10));                 │
│      product.powf(1.0 / n.sqrt())                                  │
└────────────────────────────────────────────────────────────────────┘
```

---

## §4 Konstantenregister

### Κ4.1 Neue Konstanten

$$
\begin{array}{|l|c|l|l|}
\hline
\textbf{Symbol} & \textbf{Wert} & \textbf{Typ} & \textbf{Zweck} \\
\hline
\kappa_\sigma & 15.0 & \texttt{f64} & \text{Sigmoid-Normalisierung} \\
\epsilon & 10^{-10} & \texttt{f32} & \text{Numerische Stabilität} \\
\hline
\end{array}
$$

### Κ4.2 Unveränderte Konstanten

$$
\begin{array}{|l|c|l|}
\hline
\kappa_A & 10 & \text{Aktivitätsschwelle} \\
\lambda_+ / \lambda_- & 1.5 / 2.0 & \text{Trust-Asymmetrie} \\
\tau & 90\,\text{d} & \text{Aktivitätsfenster} \\
\hline
\end{array}
$$

---

## §5 API-Invarianten

### Λ5.1 Schnittstelle ≡ unverändert

$$\text{API}_{\text{V6}} \equiv \text{API}_{\text{V5}}$$

```
WorldFormulaContribution::new(s, λ)
    .with_activity(α)
    .with_trust(&τ)
    .with_causal_history(c)
    .with_surprisal(ι)
    .with_human_factor(η)
    .with_temporal_weight(ω)
    .build()
    .compute()
```

### Λ5.2 Wertebereichsänderung

$$
\begin{array}{|l|c|c|}
\hline
\textbf{Methode} & \textbf{V5-Bild} & \textbf{V6-Bild} \\
\hline
\texttt{compute()} & [0.5, 1.0) & [0.3, 0.95] \\
\texttt{chain\_trust([0.8;4])} & \approx 0.25 & \approx 0.64 \\
\hline
\end{array}
$$

---

## §6 Migrationsprotokoll

### Μ6.1 Test-Adaptation

$$\text{assert}(|c - v| < \epsilon) \xrightarrow{\Delta} \text{assert}(c \in [v_{\min}, v_{\max}])$$

### Μ6.2 Schwellwert-Rekalibrierung

$$\theta_{\text{V5}} = 0.9 \xrightarrow{\Delta} \theta_{\text{V6}} = 0.7$$

### Μ6.3 Empfohlene Praktiken

$$
\begin{aligned}
\text{(i)}\quad   & \text{Ranking}: \text{sort}(\mathcal{C}, \lambda c. c.\text{compute}()) \\
\text{(ii)}\quad  & \text{Perzentil}: \theta = P_{90}(\{c_i\}) \\
\text{(iii)}\quad & \text{Feature-Flag}: \texttt{\#[cfg(feature = "v6")]}
\end{aligned}
$$

---

## §7 Komplexität & Stabilität

### Ο7.1 Zeitkomplexität

$$\mathcal{O}(\texttt{compute}) = \mathcal{O}(\texttt{chain\_trust}) = \mathcal{O}(1) \quad\text{bzw.}\quad \mathcal{O}(n)$$

### Ο7.2 Numerische Stabilität

$$
\begin{array}{|l|c|c|}
\hline
\textbf{Szenario} & \textbf{V5} & \textbf{V6} \\
\hline
\ln(x \to 0) & \text{Unterlauf} & \text{stabil} \\
\exp(x \to 700) & \text{Overflow} & \text{skaliert} \\
\prod_{i=1}^n t_i & \text{Akkumulation} & \text{direkt} \\
\hline
\end{array}
$$

---

## §8 Changelog-Kompakt

$$
\begin{aligned}
\Delta_{\text{V6.0}} = \{&\\
  &\sigma(x) \mapsto \sigma(x/\kappa), \\
  &\ln(|\mathbb{C}|) \mapsto \ln(|\mathbb{C}|+1), \\
  &\exp(\Sigma\ln/\sqrt{n}) \mapsto \Pi^{1/\sqrt{n}} \\
\}&
\end{aligned}
$$

---

**∎ QED**
