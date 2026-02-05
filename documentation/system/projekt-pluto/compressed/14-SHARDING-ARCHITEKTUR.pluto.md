# Pluto::ShardArch ≡ Horizontale Skalierung

> **Notation:** Pluto (komprimiert-formal)
> **Version:** 1.0 | **Datum:** 2026-02

---

## §1 Sharding-Modell – Formaldefinition

### Δ1.1 Hauptstruktur

$$\boxed{\mathcal{L} = \langle \mathcal{S}, h, \mathcal{C}, \mathcal{E}, \mathcal{M} \rangle}$$

**Komponenten:**
- $\mathcal{S} = \{S_0, S_1, \ldots, S_{n-1}\}$ — Shard-Menge
- $h: \text{RealmID} \to [0, n)$ — Hash-Funktion (Shard-Selektion)
- $\mathcal{C}: \mathcal{S} \to 2^{\text{Realm}}$ — Cache-Mapping
- $\mathcal{E}: \mathcal{S} \to \text{LRU}$ — Eviction-Policy
- $\mathcal{M}: \mathcal{S} \to \text{Metrics}$ — Monitoring

### Δ1.2 Symboltafel

| Symbol | Definition | Domäne |
|--------|-----------|--------|
| $\mathcal{S}$ | Shard-Menge | $|\mathcal{S}| \in \{4, 64, 128, 256\}$ |
| $h$ | FxHash-Funktion | $\mathbb{Z}_n$ |
| $\mathcal{C}$ | Cache (DashMap) | lock-free Map |
| $\mathcal{E}$ | LRU-Eviction | time-based |
| $\rho$ | Shard-Reputation | $[0, 1]$ |
| $\eta$ | Shard-Entropy | $[0, 1]$ |
| $\gamma$ | Gas-Multiplikator | $[1, \gamma_{\max}]$ |

---

## §2 Shard-Selektion

### Σ2.1 Hash-Funktion

$$h(r) \coloneqq \text{FxHash}(r) \mod n$$

**Eigenschaften:**
$$\begin{aligned}
\text{(i)}\quad   & h : \text{RealmID} \to \mathbb{Z}_n \quad\text{(deterministisch)} \\
\text{(ii)}\quad  & \mathbb{E}[|S_i|] = \frac{|\mathcal{R}|}{n} \quad\text{(gleichverteilung)} \\
\text{(iii)}\quad & O(1) \text{ Berechnung}
\end{aligned}$$

### Σ2.2 Shard-Index

$$\forall r \in \mathcal{R}: \quad \text{shard}(r) = S_{h(r)}$$

---

## §3 Cache-Operationen

### Κ3.1 Lookup (synchron)

$$\text{get\_cached}(r) = \begin{cases}
\mathcal{C}(S_{h(r)})[r] & \text{if } r \in \text{dom}(\mathcal{C}(S_{h(r)})) \\
\bot & \text{otherwise}
\end{cases}$$

### Κ3.2 Lazy Loading (asynchron)

$$\text{get\_or\_load}(r) = \begin{cases}
\mathcal{C}(S_{h(r)})[r] & \text{cache-hit} \\
\text{load}(r) \circ \text{replay}(r) \circ \text{insert}(r) & \text{cache-miss}
\end{cases}$$

**Pipeline:**
$$\text{Storage} \xrightarrow{\text{load}} \text{Snapshot} \xrightarrow{\text{replay}} \text{State} \xrightarrow{\text{insert}} \mathcal{C}$$

### Κ3.3 LRU-Eviction

$$\text{evict}(S_i) = \{r \in \mathcal{C}(S_i) : \text{access\_time}(r) < t_{\text{threshold}}\}$$

**Invariante:**
$$|\mathcal{C}(S_i)| \leq \kappa_{\max} \quad \forall S_i \in \mathcal{S}$$

---

## §4 ShardMonitor – Sicherheitsmodell

### Μ4.1 Entropy-Metrik

$$\eta(S_i) \coloneqq -\sum_{s \in \text{sources}(S_i)} p_s \cdot \log_2(p_s)$$

**Normalisiert:**
$$\hat{\eta}(S_i) = \frac{\eta(S_i)}{\log_2(|\text{sources}(S_i)|)} \in [0, 1]$$

**Bias-Detektion:**
$$\text{bias}(S_i) \iff \hat{\eta}(S_i) < \theta_{\text{bias}}$$

### Μ4.2 Reputation-Funktion

$$\rho(S_i) \coloneqq \frac{\text{success}(S_i)}{\text{success}(S_i) + \text{fail}(S_i)}$$

**EWMA-Update:**
$$\rho_{t+1} = \alpha \cdot \rho_t + (1 - \alpha) \cdot \rho_{\text{new}}$$

### Μ4.3 Quarantäne-Prädikat

$$Q(S_i) \iff \text{fail}(S_i) > \phi_Q \lor \rho(S_i) < \rho_{\min}$$

---

## §5 Cross-Shard-Interaktion

### Χ5.1 Gas-Penalty

$$\boxed{\gamma(S_i) = 1 + (1 - \rho(S_i)) \cdot \gamma_{\max}}$$

**Effektive Kosten:**
$$\text{gas}_{\text{eff}} = \text{gas}_{\text{base}} \cdot \gamma(S_{\text{source}})$$

| $\rho$ | $\gamma$ (bei $\gamma_{\max}=5$) |
|--------|----------------------------------|
| 1.0 | 1.0× |
| 0.5 | 3.0× |
| 0.0 | 5.0× |

### Χ5.2 Trust-Dämpfung

$$\Delta T_{\text{eff}} = \Delta T \cdot \rho(S_{\text{source}})$$

**Bedingung:**
$$Q(S_{\text{source}}) \implies \Delta T_{\text{eff}} = 0$$

---

## §6 Konfigurationsprofile

### Π6.1 Parametertafel

| Profil | $n$ | $\kappa_{\max}$ | $\tau_{\text{evict}}$ | Use Case |
|--------|-----|-----------------|----------------------|----------|
| minimal | 4 | 100 | 60s | Tests |
| default | 64 | 20.000 | 600s | Dev |
| production | 128 | 50.000 | 300s | Prod |
| auto | $4 \cdot \text{CPU}$ | 30.000 | 600s | Auto |

### Π6.2 Monitor-Parameter

| Parameter | Symbol | Default | Strict |
|-----------|--------|---------|--------|
| Bias-Threshold | $\theta_{\text{bias}}$ | 0.5 | 0.7 |
| Quarantäne-Threshold | $\phi_Q$ | 100 | 50 |
| Max Penalty | $\gamma_{\max}$ | 5.0 | 10.0 |
| Entropy-Decay | $\alpha$ | 0.9 | 0.95 |

---

## §7 Theoreme

### Τ7.1 Lookup-Komplexität

$$\text{get\_cached}(r) \in O(1)$$

**Beweis:** FxHash $O(1)$ + DashMap $O(1)$. $\square$

### Τ7.2 Load-Balancing

$$\text{stddev}\left(\frac{|S_i|}{|\mathcal{R}|/n}\right) \xrightarrow{|\mathcal{R}| \to \infty} 0$$

**Beweis:** Folgt aus Gleichverteilung von FxHash. $\square$

### Τ7.3 Quarantäne-Sicherheit

$$Q(S_i) \implies \forall r \in \mathcal{C}(S_i): \text{cross-shard}(r) = \bot$$

**Beweis:** Cross-Shard-Operationen prüfen $\neg Q(S_{\text{source}})$. $\square$

### Τ7.4 Reputation-Konvergenz

$$\lim_{t \to \infty} \rho(S_i) = \frac{\lambda_{\text{success}}}{\lambda_{\text{success}} + \lambda_{\text{fail}}}$$

**Beweis:** Stationäre Rate des Poisson-Prozesses. $\square$

---

## §8 Relationen & Abhängigkeiten

### ℜ8.1 StateGraph-Integration

$$\begin{aligned}
\text{Sharding} &\xrightarrow{\text{DependsOn}} \text{Realm} \\
\text{Sharding} &\xrightarrow{\text{DependsOn}} \text{Storage} \\
\text{Sharding} &\xrightarrow{\text{Aggregates}} \text{Trust} \\
\text{Sharding} &\xrightarrow{\text{Aggregates}} \text{Gas} \\
\text{Sharding} &\xrightarrow{\text{Triggers}} \text{Event} \\
\text{Sharding} &\xrightarrow{\text{Validates}} \text{Protection} \\
\text{Sharding} &\xleftrightarrow{\text{Bidir}} \text{P2P}
\end{aligned}$$

### ℜ8.2 Datenfluss

```
┌─────────┐     h(r)     ┌─────────┐
│ RealmID │─────────────►│ Shard_i │
└─────────┘              └────┬────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
         ┌────────┐     ┌──────────┐    ┌──────────┐
         │DashMap │     │ LRU-Cache│    │ Metrics  │
         │   𝒞    │     │    ℰ     │    │    ℳ     │
         └────────┘     └──────────┘    └──────────┘
```

---

## §9 Skalierungsgrenzen

### Ω9.1 Kapazitätsmodell

$$\text{Memory} \approx n \cdot \kappa_{\max} \cdot \bar{s}_{\text{realm}}$$

**Beispiel:**
$$128 \cdot 50.000 \cdot 10\text{KB} = 64\text{GB}$$

### Ω9.2 Horizontale Skalierung

$$\text{Node}_j \leftarrow \{S_i : i \mod m = j\}$$

**Verteilung auf $m$ Nodes:**
$$|\text{Shards}(\text{Node}_j)| = \lceil n/m \rceil$$

---

## §10 Events

### Ε10.1 CrossShardIdentityResolved

$$\text{Event} = \langle \text{identity}, S_{\text{source}}, S_{\text{target}} \rangle$$

### Ε10.2 ShardQuarantined

$$\text{Event} = \langle S_i, \text{reason}, t \rangle$$

### Ε10.3 ShardEviction

$$\text{Event} = \langle S_i, |\text{evicted}|, t \rangle$$

---

## §11 Invarianten

### Ι11.1 Cache-Konsistenz

$$r \in \mathcal{C}(S_i) \implies h(r) = i$$

### Ι11.2 Eviction-Bound

$$|\mathcal{C}(S_i)| > \kappa_{\max} \implies \text{evict}(S_i) \neq \emptyset$$

### Ι11.3 Reputation-Range

$$\forall S_i: \rho(S_i) \in [0, 1]$$

### Ι11.4 Entropy-Normalisierung

$$\forall S_i: \hat{\eta}(S_i) \in [0, 1]$$

---

## §12 Kompaktnotation

```
𝕊 ≡ ShardingSystem
ℒ ≡ LazyShardedRealmState  
ℳ ≡ ShardMonitor

h: RealmID → ℤₙ              # Shard-Selektion
𝒞: Shard → Map<RealmID, State>  # Cache
ℰ: Shard → LRU               # Eviction
ρ: Shard → [0,1]             # Reputation
η: Shard → [0,1]             # Entropy
γ: Shard → [1,γₘₐₓ]          # Gas-Penalty
Q: Shard → 𝔹                 # Quarantäne

# Operationen
get_cached(r)    ≡ 𝒞(Sₕ₍ᵣ₎)[r]
get_or_load(r)   ≡ 𝒞(Sₕ₍ᵣ₎)[r] ∨ (load ∘ replay ∘ insert)(r)
evict(Sᵢ)        ≡ {r ∈ 𝒞(Sᵢ) : t(r) < τ}

# Sicherheit
γ(Sᵢ) = 1 + (1-ρ(Sᵢ))·γₘₐₓ
ΔTₑ = ΔT·ρ(Sₛₒᵤᵣ꜀ₑ)·¬Q(Sₛₒᵤᵣ꜀ₑ)
```

---

## §13 Referenzen

| Ref | Beschreibung |
|-----|-------------|
| Κ19 | Trust-Calibration (← Shard-Entropy) |
| Κ23 | Realm-Crossing (← Shard-Reputation) |
| Κ14 | Protection-State (← ShardMonitor) |
| Κ08 | Gas/Mana (← Cross-Shard-Penalty) |
| Κ11 | P2P-Gossip (← Shard-Topics) |
| Κ06 | Event-Sourcing (← Lazy-Load-Replay) |

---

> **Pluto-Signatur:** `SHARD::v1.0::2026-02`
