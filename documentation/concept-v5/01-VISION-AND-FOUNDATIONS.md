# Vision und Grundlagen

> **Axiom-Referenz:** Fundament für alle 28 Kern-Axiome  
> **Status:** Fachkonzept-Konsolidierung  
> **Implementierung:** Die Zuordnung zur Code-Struktur `backend/src` ist in [Abschnitt VIII](#viii-implementierungs-mapping-backendsrc) dokumentiert.

---

## I. Einleitung und Vision

### 1.1 Das Fundamentale Problem

Die digitale Welt steht vor einem fundamentalen Vertrauensdilemma:

**Problem 1: Zentrale Plattform-Kontrolle**

- Algorithmen manipulieren Aufmerksamkeit intransparent
- Nutzerdaten werden in demokratiegefährdendem Ausmaß gesammelt
- Single Points of Failure und Zensur-Anfälligkeit

**Problem 2: Dezentrales Vertrauensdefizit**

- Keine robusten Mechanismen für Vertrauen zwischen Unbekannten
- Bestehende Reputationssysteme sind manipulierbar (Sybil-Attacken)
- Binäre/eindimensionale Systeme erfassen Komplexität nicht

**Problem 3: KI-Alignment**

- Autonome Agenten werden zu eigenständigen Akteuren
- Fehlendes Framework für Mensch-Maschine-Kooperation
- Menschliche Kontrolle ohne Inhibition von Innovation

### 1.2 Die Erynoa-Vision

> **Erynoa schafft eine mathematisch garantierte Grundlage für Vertrauen zwischen Menschen, Organisationen und KI-Agenten in einem dezentralen Netzwerk, das Manipulation strukturell verhindert und menschliche Werte priorisiert.**

Das System basiert auf **28 formal definierten Axiomen (Κ1-Κ28)** plus **4 Unter-Axiomen (Κ15a-d)**, die zusammen eine vollständige und widerspruchsfreie Logik für dezentrale Kooperation bilden. Diese Axiome sind nicht willkürlich, sondern mathematisch abgeleitete Prinzipien aus fundamentalen Anforderungen an faire, skalierbare und manipulationsresistente Systeme.

### 1.3 Sechs Zentrale Innovationen

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                          DIE SECHS INNOVATIONEN                                ║
╠════════════════════════════════════════════════════════════════════════════════╣
║                                                                                ║
║   1. MEHRDIMENSIONALES VERTRAUEN (Κ3)                                         ║
║      ──────────────────────────────────                                        ║
║      Trust als 6D-Vektor: 𝕎 = (R, I, C, P, V, Ω) ∈ [0,1]⁶                     ║
║      Kontextabhängige Gewichtung, keine Information geht verloren              ║
║                                                                                ║
║   2. ASYMMETRISCHE TRUST-DYNAMIK (Κ4)                                         ║
║      ────────────────────────────────                                          ║
║      Vertrauensaufbau: langsam, konsistent                                     ║
║      Vertrauensverlust: schnell, stark (λ ≈ 2.0)                              ║
║      Spiegelt menschliche Intuition, robust gegen Manipulation                 ║
║                                                                                ║
║   3. HIERARCHISCHE REALM-STRUKTUR (Κ1)                                        ║
║      ───────────────────────────────────                                       ║
║      RootRealm ⊃ VirtualRealm ⊃ Partition                                     ║
║      Lokale Autonomie + Globale Kohärenz                                       ║
║      Regeln nur erweiterbar, nie reduzierbar                                   ║
║                                                                                ║
║   4. HUMAN-ALIGNMENT-FAKTOR (Κ16)                                             ║
║      ─────────────────────────────                                             ║
║      Verifizierte Menschen: Ĥ = 2.0                                           ║
║      Menschenkontrollierte Agenten: Ĥ = 1.5                                   ║
║      Unbekannt: Ĥ = 1.0                                                       ║
║      Menschliche Interessen bleiben auch bei Automatisierung gewahrt           ║
║                                                                                ║
║   5. TRUST-GEDÄMPFTE SURPRISAL (Κ15a)                                         ║
║      ──────────────────────────────────                                        ║
║      𝒮 = ‖𝕎‖² · ℐ    wobei ℐ = −log₂ P(e|ℂ)                                  ║
║      Verhindert Hype-Zyklen und Spam-Belohnung                                 ║
║      Nur vertrauenswürdige Innovation wird belohnt                             ║
║                                                                                ║
║   6. QUADRATISCHE GOVERNANCE (Κ21)                                            ║
║      ───────────────────────────────                                           ║
║      vote_weight = √σ · relevance · freshness                                  ║
║      Verhindert Plutokratie, fördert Partizipation                             ║
║      Freshness-Decay für Dauer-Abstimmer                                       ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

---

## II. Grundlegende Konzepte

### 2.1 Das Subjekt-Modell (Κ6-Κ8)

Ein **Subjekt** ist jede Entität, die im Erynoa-Netzwerk agieren kann:

| Namespace | Beschreibung      | Beispiel                  |
| --------- | ----------------- | ------------------------- |
| `self`    | Natürliche Person | Alice, Bob                |
| `guild`   | Organisation      | Unternehmen, DAO          |
| `spirit`  | Autonomer Agent   | Trading-Bot, KI-Assistent |
| `thing`   | IoT-Gerät         | Sensor, Smart-Lock        |
| `vessel`  | Container/VM      | Docker, WASM-Instanz      |
| `source`  | Datenquelle       | API, Feed                 |
| `craft`   | Dienst/Service    | Orakel, Validator         |
| `vault`   | Speicher          | Wallet, Key-Store         |
| `pact`    | Vertrag           | Smart Contract            |
| `circle`  | Gruppe            | Team, Community           |

**Axiom-Verankerung:**

```
Κ6 (Existenz-Eindeutigkeit):
    ∀ entity e : ∃! did ∈ DID : identity(e) = did
    "Jede Entität hat genau eine eindeutige Identität."

Κ7 (Permanenz mit Aktivitäts-Modulation):
    ⟨s⟩ ∧ ⟦create(s)⟧ ⟹ □⟨s⟩
    "Einmal erstellt, existiert eine Identität permanent."
    𝔸(s) moduliert den Einfluss (inaktiv = weniger Einfluss)

Κ8 (Delegations-Struktur):
    s ⊳ s' → 𝕋(s') ≤ 𝕋(s)
    "Delegierter Trust kann nie höher sein als der des Delegierenden."
    Die Relation ⊳ ist eine strenge Halbordnung (irreflexiv, antisymmetrisch, transitiv)
```

### 2.2 Der Trust-Vektor 𝕎 (Κ3-Κ5)

Das Herzstück des Erynoa-Vertrauensmodells ist der **6-dimensionale Trust-Vektor**:

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   TRUST-VEKTOR 𝕎 ∈ [0,1]⁶                                                     ║
║                                                                                ║
║   𝕎(s,ε,t) = (R, I, C, P, V, Ω)                                               ║
║                                                                                ║
║   ┌─────┬────────────┬─────────────────────────────────────────────────────┐  ║
║   │ Dim │ Name       │ Beschreibung                                        │  ║
║   ├─────┼────────────┼─────────────────────────────────────────────────────┤  ║
║   │  R  │ Reliability│ Verhaltenskonsistenz über Zeit                      │  ║
║   │  I  │ Integrity  │ Aussage-Konsistenz und Wahrhaftigkeit               │  ║
║   │  C  │ Competence │ Nachgewiesene Fähigkeiten (domänenspezifisch)       │  ║
║   │  P  │ Prestige   │ Externe Attestierungen und Anerkennung              │  ║
║   │  V  │ Vigilance  │ Anomalie-Erkennung und Netzwerk-Sicherheit          │  ║
║   │  Ω  │ Omega      │ Axiom-Treue (fundamentale Systemtreue)              │  ║
║   └─────┴────────────┴─────────────────────────────────────────────────────┘  ║
║                                                                                ║
║   KONTEXT-GEWICHTE (Κ15b):                                                    ║
║   ┌──────────────────┬──────┬──────┬──────┬──────┬──────┬──────┐             ║
║   │ Kontext          │   R  │   I  │   C  │   P  │   V  │   Ω  │             ║
║   ├──────────────────┼──────┼──────┼──────┼──────┼──────┼──────┤             ║
║   │ Finanztransaktion│ 0.30 │ 0.25 │ 0.15 │ 0.10 │ 0.15 │ 0.05 │             ║
║   │ Wissensaustausch │ 0.10 │ 0.30 │ 0.30 │ 0.15 │ 0.10 │ 0.05 │             ║
║   │ Governance       │ 0.15 │ 0.20 │ 0.15 │ 0.20 │ 0.10 │ 0.20 │             ║
║   │ Default          │ 0.17 │ 0.17 │ 0.17 │ 0.17 │ 0.16 │ 0.16 │             ║
║   └──────────────────┴──────┴──────┴──────┴──────┴──────┴──────┘             ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

**Axiom-Verankerung:**

```
Κ3 (Dimensionale Unabhängigkeit):
    ∀ i,j ∈ {R,I,C,P,V,Ω}, i ≠ j : 𝕎ᵢ ⊥ 𝕎ⱼ
    "Dimensionen sind konzeptuell unabhängig."

Κ4 (Asymmetrische Evolution):
    Δ⁺(dim) = base_delta
    Δ⁻(dim) = λ_asym · base_delta
    λ_asym = 1.5 für R,I,C,P
    λ_asym = 2.0 für V,Ω (sicherheitskritisch)
    "Vertrauensverlust wiegt schwerer als Vertrauensgewinn."

Κ5 (Probabilistische Kombination):
    t₁ ⊕ t₂ = 1 - (1-t₁)(1-t₂)
    "Trust-Kombination folgt der Wahrscheinlichkeits-ODER-Logik."
```

### 2.3 Die Realm-Hierarchie (Κ1)

Erynoa organisiert alle Aktivitäten in einer hierarchischen Struktur:

```
                    ┌─────────────────────────────────┐
                    │         ROOT-REALM              │
                    │   (Universelle Axiome Κ1-Κ28)   │
                    │   rules: {Κ1, ..., Κ28}         │
                    └───────────────┬─────────────────┘
                                    │
            ┌───────────────────────┼───────────────────────┐
            ▼                       ▼                       ▼
┌────────────────────────┐ ┌────────────────────────┐ ┌────────────────────────┐
│    VIRTUAL-REALM A     │ │    VIRTUAL-REALM B     │ │    VIRTUAL-REALM C     │
│   (z.B. "Knowledge")   │ │   (z.B. "Finance")     │ │   (z.B. "Creative")    │
│   rules: {Κ1-28} + {A} │ │   rules: {Κ1-28} + {B} │ │   rules: {Κ1-28} + {C} │
└──────────┬─────────────┘ └──────────┬─────────────┘ └──────────┬─────────────┘
           │                          │                          │
    ┌──────┴──────┐            ┌──────┴──────┐            ┌──────┴──────┐
    ▼             ▼            ▼             ▼            ▼             ▼
┌───────┐     ┌───────┐   ┌───────┐     ┌───────┐    ┌───────┐     ┌───────┐
│Part A1│     │Part A2│   │Part B1│     │Part B2│    │Part C1│     │Part C2│
│Energy │     │Trust  │   │DeFi   │     │Escrow │    │Art    │     │Music  │
└───────┘     └───────┘   └───────┘     └───────┘    └───────┘     └───────┘
```

**Axiom-Verankerung:**

```
Κ1 (Monotone Regelvererbung):
    ∀ 𝒞₁ ⊂ 𝒞₂ : rules(𝒞₁) ⊇ rules(𝒞₂)
    "Kind-Kategorien können Regeln hinzufügen, nie entfernen."
```

**Eigenschaften:**

- **Root-Realm:** Enthält alle 28 Kern-Axiome, unveränderlich
- **Virtual-Realm:** Domänenspezifische Kontexte (z.B. Finance, Knowledge)
- **Partition:** Konkrete Arbeitskontexte (z.B. DeFi-Pool, Research-Group)

### 2.4 Events und der Kausale Graph (Κ9-Κ12)

Alle Aktivitäten werden als **Events** in einem kausalen DAG erfasst:

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   EVENT-DAG (KAUSALER GRAPH)                                                  ║
║                                                                                ║
║       Genesis ─────┬──────────────────────────────────────► Zeit              ║
║            │       │                                                          ║
║            ▼       ▼                                                          ║
║           E₁      E₂                                                          ║
║            │       │                                                          ║
║            └───┬───┘                                                          ║
║                ▼                                                              ║
║               E₃ ─────────────────────────────┐                               ║
║                │                               │                               ║
║                ▼                               ▼                               ║
║               E₄                              E₅                               ║
║                                                │                               ║
║                                                ▼                               ║
║                                          E₆ (Anchored)                        ║
║                                                                                ║
║   FINALITÄTS-SPEKTRUM (Κ10):                                                  ║
║   ┌──────────┬─────────────────────────────────────────────────────────────┐  ║
║   │ Level    │ Beschreibung                                                │  ║
║   ├──────────┼─────────────────────────────────────────────────────────────┤  ║
║   │ NASCENT  │ Gerade erstellt, noch nicht propagiert                      │  ║
║   │ VALIDATED│ Von lokalen Peers validiert                                 │  ║
║   │ WITNESSED│ Von Threshold-Anzahl Witnesses bezeugt                      │  ║
║   │ ANCHORED │ In externem System verankert (z.B. Merkle-Root)            │  ║
║   │ ETERNAL  │ Unveränderlich (nur Genesis-Event)                          │  ║
║   └──────────┴─────────────────────────────────────────────────────────────┘  ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

**Axiom-Verankerung:**

```
Κ9 (Kausale Struktur):
    Die Relation e₁ ⊲ e₂ ("e₁ ist kausaler Vorgänger von e₂") ist eine
    strenge Halbordnung (irreflexiv, antisymmetrisch, transitiv).
    Der Event-Graph ist ein DAG (keine Zyklen).

Κ10 (Bezeugung-Finalität):
    |Witnesses(e)| ≥ θ_finality ⟹ Finality(e) steigt monoton.
    "Mehr Zeugen = höhere Finalität."

Κ11 (Prozess-Korrektheit - Hoare-Tripel):
    {P} process {Q}
    "Wenn Vorbedingung P erfüllt, dann nach process Nachbedingung Q erfüllt."

Κ12 (Event-Erzeugung):
    Jeder Prozess erzeugt genau ein Event pro atomarer Aktion.
```

---

## III. Die Weltformel (Κ15a-d)

### 3.1 Motivation

Die Weltformel aggregiert den Zustand des gesamten Netzwerks zu einem (mehrdimensionalen) Wert, der:

- Ranking und Sichtbarkeit beeinflusst
- Ressourcenallokation steuert
- Governance-Entscheidungen informiert

### 3.2 Die Formel V2.0

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║                    DIE WELTFORMEL V2.0                                        ║
║                                                                               ║
║   𝔼 = Σ  𝔸(s) · σ⃗( ‖𝕎(s)‖_w · ln|ℂ(s)| · 𝒮(s) ) · Ĥ(s) · w(s,t)             ║
║       s∈𝒞                                                                     ║
║                                                                               ║
║   wobei 𝒮(s) = ‖𝕎(s)‖² · ℐ(s)     [Trust-gedämpfte Surprisal]                ║
║         ℐ(s) = −log₂ P(e | ℂ(s))  [Shannon-Surprisal]                         ║
║                                                                               ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║   KOMPONENTEN:                                                                ║
║                                                                               ║
║   𝔸(s)      Aktivitäts-Präsenz (wer handelt)                                 ║
║   ‖𝕎(s)‖_w  Gewichtete Trust-Norm (kontextabhängig)                          ║
║   |ℂ(s)|    Größe der kausalen Geschichte                                    ║
║   𝒮(s)      Trust-gedämpfte Surprisal (verhindert Hype)                      ║
║   ℐ(s)      Shannon-Surprisal (informationstheoretisch)                      ║
║   Ĥ(s)      Human-Alignment-Faktor (2.0 | 1.5 | 1.0)                         ║
║   w(s,t)    Temporale Vergebung (ältere Events zählen weniger)               ║
║   σ⃗         Sigmoid-Funktion (Normierung)                                    ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### 3.3 Anti-Hype durch Trust-Dämpfung (Κ15a)

```
Problem:  Neue Akteure könnten "überraschende" (= hohe Surprisal) Inhalte
          spammen, um Aufmerksamkeit zu farmen.

Lösung:   𝒮(s) = ‖𝕎(s)‖² · ℐ(s)

          Agent mit 𝕎 = 0.3:  𝒮 = 0.09 · ℐ   (91% Dämpfung)
          Agent mit 𝕎 = 0.9:  𝒮 = 0.81 · ℐ   (19% Dämpfung)

Effekt:   Nur vertrauenswürdige Innovation wird belohnt.
          Spam und Hype-Cycles werden strukturell verhindert.
```

### 3.4 Skalierbarkeit (Κ15d)

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   APPROXIMATIONS-STRATEGIEN                                                   ║
║                                                                                ║
║   1. HIERARCHISCHE AGGREGATION (für Batch-Analyse)                            ║
║      𝔼 ≈ Σ_partitions 𝔼_partition                                             ║
║      𝔼_partition = |partition| · mean(sample(partition, k))                   ║
║      Komplexität: O(|Partitions| · k) statt O(|𝒞|)                            ║
║                                                                                ║
║   2. STREAMING APPROXIMATION (für Echtzeit)                                   ║
║      𝔼(t+1) = α · 𝔼(t) + (1-α) · Σ_new f(s_new)                               ║
║      α = exp(-Δt / τ_update)                                                  ║
║      Komplexität: O(|neue Events|) pro Update                                 ║
║                                                                                ║
║   3. IMPORTANCE SAMPLING (für statistische Analyse)                           ║
║      𝔼 ≈ (1/k) · Σᵢ f(sᵢ) / q(sᵢ)                                            ║
║      wobei sᵢ ~ q(s) ∝ 𝔸(s) · ‖𝕎(s)‖                                         ║
║      Minimiert Varianz durch intelligentes Sampling                           ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

---

## IV. Schutzmechanismen (Κ19-Κ21)

### 4.1 Anti-Calcification (Κ19)

**Problem:** Machtkonzentration über Zeit – frühe Teilnehmer werden unerreichbar.

**Lösung:**

```
rank_final(s) = relevance(s) · (σ(s)^γ + β·(1-𝔸(s))·e^(-age/τ)) · (1 + ξ·noise)

Komponenten:
  σ(s)^γ              Diminishing Returns (γ = 0.7)
  β·(1-𝔸(s))·e^(-age/τ)  Exploration Bonus für Newcomer
  (1 + ξ·noise)       Stochastic Fairness (ξ = 0.1)
```

### 4.2 Diversity-Enforcement (Κ20)

**Problem:** Collusion – geheime Absprachen zur Manipulation.

**Lösung:**

```
Δ𝕎(s, tx) = base · Quality(tx) · diversity_mult(s) · (1 - collusion(tx))

diversity_mult(s) = min(1, unique_partners(s, τ) / θ_diversity)
collusion(tx) = f(similarity, exclusivity, temporal_correlation)
```

### 4.3 Quadratic Governance (Κ21)

**Problem:** Plutokratie – Reiche dominieren Abstimmungen.

**Lösung:**

```
vote_weight(s, p) = √(σ(s)) · relevance(s, domain(p)) · freshness(s)

freshness(s) = 1 - (consecutive_rounds(s) / max_rounds)²
```

---

## V. Konsens-Mechanismus (Κ18)

### 5.1 Gewichteter Partition-Konsens

```
                 Σ 𝕎(s) · [s ⊢ φ]
Ψ(Σ)(φ) = ─────────────────────
               Σ 𝕎(s)
           s ⊢ Σ

θ_konsens = 2/3 (Supermajorität)
```

### 5.2 Konsens-Konvergenz (Theorem Τ4)

```
lim_{t→∞} Ψ(Σ)(φ) = μ_true(φ)    unter folgenden Bedingungen:

(i)   majority(honest) > 1/2
(ii)  ∀ honest s : V(s) > 0.5
(iii) 𝔸(dishonest) → 0 über Zeit (durch S1-S4)

⟹ Honest agents dominate ⟹ Ψ konvergiert zur Wahrheit.
```

---

## VI. Isomorphismen (Versteckte Verknüpfungen)

Die mathematische Analyse hat fünf fundamentale Isomorphismen aufgedeckt:

| Isomorphismus | Erynoa-Konzept      | Mathematisches Äquivalent                  |
| ------------- | ------------------- | ------------------------------------------ |
| 1             | Delegation ⊳        | Kausalität ⊲ (beide strenge Halbordnungen) |
| 2             | Trust-Kombination ⊕ | Wahrscheinlichkeits-ODER P(A∨B)            |
| 3             | Partition-Konsens Ψ | Bayes'sches Update P(φ\|evidence)          |
| 4             | Weltformel 𝔼        | Freie Energie (Helmholtz: F = U - TS)      |
| 5             | Realm-Hierarchie    | Topos mit Subobject-Classifier             |

**Implikationen:**

- Das System folgt thermodynamischen Gesetzen (minimiert Unsicherheit)
- Trust ist interpretierbar als Wahrscheinlichkeit
- Die Logik innerhalb jeder Ebene kann intuitionistisch sein
- Erst auf Root-Realm-Ebene gilt klassische Logik

---

## VII. Zusammenfassung der Grundlagen

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   ERYNOA GRUNDLAGEN – ZUSAMMENFASSUNG                                         ║
║                                                                                ║
╠════════════════════════════════════════════════════════════════════════════════╣
║                                                                                ║
║   SUBJEKTE (Κ6-Κ8):                                                           ║
║     • DIDs als einzigartige, selbstbestimmte Identitäten                      ║
║     • Delegation mit Trust-Beschränkung                                        ║
║     • Permanenz mit Aktivitäts-Modulation                                      ║
║                                                                                ║
║   TRUST-VEKTOR (Κ3-Κ5):                                                       ║
║     • 6 unabhängige Dimensionen: R, I, C, P, V, Ω                             ║
║     • Asymmetrische Evolution (Verlust > Gewinn)                              ║
║     • Probabilistische Kombination                                             ║
║                                                                                ║
║   REALM-HIERARCHIE (Κ1):                                                      ║
║     • Root → Virtual → Partition                                               ║
║     • Monotone Regelvererbung (nur erweiterbar)                               ║
║     • Lokale Autonomie + Globale Kohärenz                                      ║
║                                                                                ║
║   EVENT-DAG (Κ9-Κ12):                                                         ║
║     • Kausaler Graph ohne Zyklen                                               ║
║     • Finalitäts-Spektrum (NASCENT → ETERNAL)                                 ║
║     • Hoare-Tripel für Prozess-Korrektheit                                    ║
║                                                                                ║
║   WELTFORMEL (Κ15a-d):                                                        ║
║     • Aggregiert Netzwerk-Zustand                                              ║
║     • Trust-gedämpfte Surprisal gegen Hype                                    ║
║     • Skalierbare Approximation                                                ║
║                                                                                ║
║   SCHUTZ (Κ19-Κ21):                                                           ║
║     • Anti-Calcification gegen Machtkonzentration                             ║
║     • Diversity-Enforcement gegen Collusion                                    ║
║     • Quadratic Governance gegen Plutokratie                                   ║
║                                                                                ║
║   KONSENS (Κ18):                                                              ║
║     • Trust-gewichtete Supermajorität (2/3)                                   ║
║     • Konvergiert zur Wahrheit bei honest majority                            ║
║                                                                                ║
║   HUMAN-ALIGNMENT (Κ16-Κ17, Κ25-Κ28):                                         ║
║     • Ĥ-Faktor bevorzugt Menschen                                             ║
║     • Temporale Vergebung (negative Events verblassen schneller)              ║
║     • Semantische Verankerung (menschlich verständlich)                       ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

---

## VIII. Implementierungs-Mapping (backend/src)

Dieser Abschnitt verknüpft die konzeptionellen Grundlagen mit der **tatsächlichen Code-Struktur** des Erynoa-Backends (`backend/src`). Er dient als Einstieg für Entwickler und stellt sicher, dass Vision und Implementierung konsistent dokumentiert sind.

### 8.1 Schichten-Übersicht (lib.rs)

Das Backend folgt der 4-Schichten-Architektur aus Concept V5; die Modul-Reihenfolge in `lib.rs` definiert die Abhängigkeitsrichtung:

| Schicht | Modul | Axiom-Bereich | Kurzbeschreibung |
|--------|--------|----------------|-------------------|
| **Domain** | `domain` | Κ1–Κ28 (Typen) | Kerntypen: Unified Data Model (UDM) in `domain/unified/` |
| **Execution** | `execution` | IPS ℳ, Κ4/Κ8–Κ12/Κ15b | ExecutionContext, TrackedContext, Gas/Mana, ExecutionError |
| **Core Logic** | `core` | Κ2–Κ18 | State, EventEngine, TrustEngine, WorldFormula, Consensus, StateGraph |
| **Protection** | `protection` | Κ19–Κ21, Κ26–Κ28 | AntiCalcification, Diversity, Quadratic, Anomaly, AdaptiveCalibration |
| **Peer** | `peer` | Κ22–Κ24 | IntentParser, SagaComposer, GatewayGuard; optional `peer/p2p/` |
| **Storage** | `local` | Persistenz | KvStore, EventStore, IdentityStore, TrustStore, RealmStorage, Archive, BlueprintMarketplace |
| **ECLVM** | `eclvm` | Policy/Gateway | ECL Parser/Compiler/VM, ProgrammableGateway, Mana, ErynoaHost |
| **API** | `api`, `server` | — | REST, Middleware, v1 Handlers; optional Debug-UI (feature `debug`) |

### 8.2 Konzept → Code-Pfad

| Konzept (aus I–VII) | Implementierung in backend/src |
|---------------------|---------------------------------|
| **Subjekt-Modell, DIDs (Κ6–Κ8)** | `domain/unified/identity.rs` (DID, DIDDocument, Delegation), `local/identity_store.rs` |
| **Trust-Vektor 𝕎 (Κ3–Κ5)** | `domain/unified/trust.rs` (TrustVector6D, TrustRecord), `core/trust_engine.rs`, `core/state.rs` (TrustState) |
| **Realm-Hierarchie (Κ1)** | `domain/unified/realm.rs` (Realm, RootRealm, VirtualRealm, Partition), `local/realm_storage.rs`, `core/state.rs` (RealmState, RealmSpecificState) |
| **Events, Kausaler DAG (Κ9–Κ12)** | `domain/unified/event.rs` (Event, FinalityState), `core/event_engine.rs`, `core/state.rs` (EventState), `local/event_store.rs` |
| **Weltformel 𝔼, Surprisal 𝒮 (Κ15a–d)** | `core/world_formula.rs`, `core/surprisal.rs`, `domain/unified/formula.rs`, `core/state.rs` (FormulaState) |
| **Konsens (Κ18)** | `core/consensus.rs`, `core/state.rs` (ConsensusState) |
| **Anti-Calcification, Diversity, Quadratic (Κ19–Κ21)** | `protection/anti_calcification.rs`, `protection/diversity.rs`, `protection/quadratic.rs`, `protection/adaptive_calibration.rs`, `protection/anomaly.rs` |
| **Intent, Saga (Κ22)** | `domain/unified/saga.rs` (Intent, Saga, SagaStep), `peer/intent_parser.rs`, `peer/saga_composer.rs` |
| **Gateway, Realm-Crossing (Κ23)** | `peer/gateway.rs` (GatewayGuard), `eclvm/programmable_gateway.rs` (ECL-basierte Policies) |
| **Kosten-Algebra (Gas × Mana × Trust-Risk)** | `domain/unified/cost.rs` (Cost, Budget), `execution/` (Gas/Mana-Konstanten), `eclvm/runtime/gas.rs`, `eclvm/mana.rs` |
| **Invarianten Κ1–Κ15** | `domain/unified/mod.rs` (InvariantChecker, InvariantViolation) |
| **Unified State, StateGraph** | `core/state.rs` (UnifiedState, StateComponent, StateGraph, StateRelation, Observer/Integration), `core/state_coordination.rs`, `core/state_integration.rs` |
| **P2P-Netzwerk** | `peer/p2p/` (Swarm, Transport, Privacy, Diagnostics, TrustGate, Censorship-Resistance) |
| **Debug-UI** | `debug/` (egui App, Tabs: Overview, State Graph, Trust, Bayesian, Simulation Lab, Events, P2P, Packages, Realms, Execution, Logs, Live Config, ECL Playground, Threat Sim) – nur mit Feature `debug` |

### 8.3 Wichtige Verzeichnisstrukturen

- **`domain/unified/`**: Ein Modul pro Konzept (primitives, identity, event, trust, realm, saga, formula, cost, message, config, schema). Single Source of Truth für Typen und InvariantChecker.
- **`core/`**: state.rs (zentrales Unified State + StateGraph), *_engine.rs / world_formula / surprisal, state_integration (Observer), state_coordination (Health, Invarianten).
- **`peer/p2p/`**: behaviour, swarm, transport, protocol, topics; Unterordner: censorship/, diagnostics/, privacy/, performance/, multi_circuit/.
- **`local/`**: DecentralizedStorage (Fjall), identity_store, event_store, trust_store, content_store, kv_store, realm_storage, archive, blueprint_marketplace.
- **`eclvm/`**: ast, parser, compiler, bytecode, runtime (vm, gas, host), erynoa_host, programmable_gateway, mana, stdlib.

### 8.4 Binaries und Einstiegspunkte

- **`main.rs`**: Startet API-Server (Settings, Telemetry, optional `--static-dir`).
- **`bin/debug.rs`**: Egui-Debugger (nur mit Feature `debug`).
- **`bin/ecl.rs`**, **`bin/testnet_node.rs`**: ECL-CLI bzw. Testnet-Node.

Weitere Architekturdetails: [03-SYSTEM-ARCHITECTURE.md](03-SYSTEM-ARCHITECTURE.md). Implementierungsdetails und Code-Beispiele: [05-IMPLEMENTATION-GUIDE.md](05-IMPLEMENTATION-GUIDE.md).

---

_Weiter zu [02-AXIOM-SYSTEM.md](02-AXIOM-SYSTEM.md) für die vollständige Axiom-Definition._
