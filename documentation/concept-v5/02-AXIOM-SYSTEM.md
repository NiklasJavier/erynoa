# Axiom-System

> **Version:** V5.0 – Vollständige & Konsolidierte Axiomatik
> **Basis:** Erynoa Unified Logic V4.1
> **Status:** 28 Kern-Axiome + 4 Unter-Axiome + 5 Theoreme

---

## Übersicht

Das Erynoa-Axiom-System definiert die mathematischen Grundlagen des gesamten Systems. Alle Implementierungsdetails, Architekturentscheidungen und Algorithmen sind aus diesen Axiomen ableitbar.

### Axiom-Kategorien

| Kategorie               | Axiome    | Anzahl | Fokus                         |
| ----------------------- | --------- | ------ | ----------------------------- |
| Kategorische Fundierung | Μ1, Κ1-Κ2 | 3      | Meta-Struktur, Regelvererbung |
| Trust-Algebra           | Κ3-Κ5     | 3      | 6D-Vektor, Asymmetrie         |
| Identitäts-Algebra      | Κ6-Κ8     | 3      | DID, Permanenz, Delegation    |
| Kausale Algebra         | Κ9-Κ10    | 2      | DAG, Finalität                |
| Prozess-Algebra         | Κ11-Κ14   | 4      | Korrektheit, Atomarität       |
| Weltformel              | Κ15a-d    | 4      | Surprisal, Skalierung         |
| Humanismus              | Κ16-Κ17   | 2      | Human-Alignment, Vergebung    |
| Konsens                 | Κ18       | 1      | Partition-Wahrheit            |
| Schutz                  | Κ19-Κ21   | 3      | Anti-Degeneration             |
| Peer-Logik              | Κ22-Κ24   | 3      | Gateway, Saga, Funktor        |
| System-Garantien        | Κ25-Κ28   | 4      | Determinismus, Offenheit      |
| **Gesamt**              |           | **32** | + 5 Theoreme                  |

---

## I. Kategorische Fundierung

### Meta-Axiom Μ1 (Partielle Ordnung)

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   META-AXIOM Μ1 (PARTIELLE ORDNUNG):                                          ║
║                                                                                ║
║   Erynoa basiert auf Relationen, die strenge Halbordnungen sind:              ║
║                                                                                ║
║       • Irreflexiv:      ¬(a ≺ a)                                             ║
║       • Antisymmetrisch: (a ≺ b) ∧ (b ≺ a) → a = b                            ║
║       • Transitiv:       (a ≺ b) ∧ (b ≺ c) → (a ≺ c)                          ║
║                                                                                ║
║   Anwendungen:                                                                ║
║       • ⊳ (Delegation)    auf DIDs                                            ║
║       • ⊲ (Kausalität)    auf Events                                          ║
║       • ⊃ (Realm-Enthält) auf Realms                                          ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ1 – Monotone Regelvererbung

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ1 (MONOTONE REGELVERERBUNG):                                    ║
║                                                                                ║
║       ∀ 𝒞₁ ⊂ 𝒞₂ : rules(𝒞₁) ⊇ rules(𝒞₂)                                      ║
║                                                                                ║
║   "Kind-Kategorien können Regeln hinzufügen, nie entfernen."                  ║
║                                                                                ║
║   ═══════════════════════════════════════════════════════════════════════════  ║
║                                                                                ║
║   IMPLEMENTIERUNG:                                                            ║
║       Root-Realm:     rules = {Κ1, Κ2, ..., Κ28}                              ║
║       Virtual-Realm:  rules = Root.rules ∪ {domain_specific}                  ║
║       Partition:      rules = Virtual.rules ∪ {partition_specific}            ║
║                                                                                ║
║   → Absorbiert: A18 (Monotonie), A19 (Regelvererbung), E11 (Realm-Struktur)  ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ2 – Trust-Funktor-Gesetz

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ2 (TRUST-FUNKTOR-GESETZ):                                       ║
║                                                                                ║
║       𝕋 : 𝒞_Ery → [0,1]⁶                                                      ║
║                                                                                ║
║       𝕋(id_s) = id_{𝕋(s)}           [Identität]                               ║
║       𝕋(g ∘ f) = 𝕋(g) ∘ 𝕋(f)        [Kompositionalität]                       ║
║                                                                                ║
║   "Trust ist ein kovarianter Funktor aus der Erynoa-Kategorie."               ║
║                                                                                ║
║   ═══════════════════════════════════════════════════════════════════════════  ║
║                                                                                ║
║   KATEGORIENTHEORIE-BASIS:                                                    ║
║       𝒞_Ery = (Ob, Mor, ∘, id)                                                ║
║       Ob = {DIDs, Events, Realms, ...}                                        ║
║       Mor = {Transaktionen, Attestierungen, Delegationen, ...}                ║
║                                                                                ║
║   → Absorbiert: A6-A11 (Teil), E5-E7, PR6                                    ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

---

## II. Trust-Algebra

### Κ3 – Dimensionale Unabhängigkeit

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ3 (DIMENSIONALE UNABHÄNGIGKEIT):                                ║
║                                                                                ║
║       𝕎 ∈ [0,1]⁶  wobei Dimensionen konzeptuell unabhängig                   ║
║                                                                                ║
║       𝕎(s,ε,t) = (R, I, C, P, V, Ω)                                          ║
║                                                                                ║
║       R = Reliability    (Verhaltens-Historie)                                ║
║       I = Integrity      (Aussage-Konsistenz)                                 ║
║       C = Competence     (Fähigkeits-Nachweis)                                ║
║       P = Prestige       (Externe Attestation)                                ║
║       V = Vigilance      (Anomalie-Erkennung)                                 ║
║       Ω = Omega          (Axiom-Treue)                                        ║
║                                                                                ║
║   → Absorbiert: E6 (Dimensionen), E7 (Unabhängigkeit)                        ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ4 – Asymmetrische Evolution

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ4 (ASYMMETRISCHE EVOLUTION):                                    ║
║                                                                                ║
║       Δ⁺(dim) = base_delta                                                    ║
║       Δ⁻(dim) = λ_asym · base_delta    wobei λ > 1                            ║
║                                                                                ║
║   ASYMMETRIE-FAKTOREN (aus Verhaltensökonomie: Kahneman-Tversky):             ║
║       λ_asym = 1.5    für R, I, C, P  (konservativ)                           ║
║       λ_asym = 2.0    für V, Ω        (sicherheitskritisch)                   ║
║                                                                                ║
║   FLOOR-MECHANISMUS:                                                          ║
║       min(𝕎ᵢ) = 0.01  (niemand erreicht exakt 0)                             ║
║                                                                                ║
║   "Vertrauensverlust wiegt schwerer als Vertrauensgewinn."                    ║
║                                                                                ║
║   → Absorbiert: A8 (Asymmetrie), E9 (Evolution)                              ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ5 – Probabilistische Kombination

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ5 (PROBABILISTISCHE KOMBINATION):                               ║
║                                                                                ║
║       t₁ ⊕ t₂ = 1 - (1-t₁)(1-t₂)                                              ║
║                                                                                ║
║   ISOMORPHISMUS: Trust-ODER ≅ Wahrscheinlichkeits-ODER                        ║
║       P(A ∨ B) = 1 - P(¬A)P(¬B)                                               ║
║                                                                                ║
║   EIGENSCHAFTEN:                                                              ║
║       • Kommutativ:   t₁ ⊕ t₂ = t₂ ⊕ t₁                                       ║
║       • Assoziativ:   (t₁ ⊕ t₂) ⊕ t₃ = t₁ ⊕ (t₂ ⊕ t₃)                        ║
║       • Idempotent:   t ⊕ t = t (nur wenn t = 1)                              ║
║       • Neutral:      t ⊕ 0 = t                                               ║
║       • Bounds:       t₁ ⊕ t₂ ∈ [max(t₁,t₂), 1]                               ║
║                                                                                ║
║   → Absorbiert: A11 (Kombination), C4 (Transitiv), E13 (Teil)                ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Theorem Τ1 (Ketten-Trust)

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   THEOREM Τ1 (KETTEN-TRUST):                                                  ║
║                                                                                ║
║       chain_trust([t₁, t₂, ..., tₙ]) = exp( (Σᵢ ln(tᵢ)) / √n )               ║
║                                                                                ║
║   EIGENSCHAFT:                                                                ║
║       Längere Ketten haben niedrigeren Trust (Dämpfung mit √n)                ║
║       Jeder Sprung in der Kette reduziert Vertrauen                           ║
║                                                                                ║
║   HERLEITUNG: Aus Κ5 (Kombination) + geometrischer Durchschnitt               ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

---

## III. Identitäts-Algebra

### Κ6 – Existenz-Eindeutigkeit

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ6 (EXISTENZ-EINDEUTIGKEIT):                                     ║
║                                                                                ║
║       ∀ entity e : ∃! did ∈ DID : identity(e) = did                           ║
║                                                                                ║
║   "Jede Entität hat genau eine eindeutige dezentrale Identität."              ║
║                                                                                ║
║   IMPLEMENTIERUNG:                                                            ║
║       did:erynoa:<namespace>:<unique_id>                                      ║
║       unique_id = hash(public_key || creation_timestamp || salt)              ║
║                                                                                ║
║   → Absorbiert: A1 (Identität), O1 (Eindeutigkeit)                           ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ7 – Permanenz mit Aktivitäts-Modulation

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ7 (PERMANENZ MIT AKTIVITÄTS-MODULATION):                        ║
║                                                                                ║
║       ⟨s⟩ ∧ ⟦create(s)⟧ ⟹ □⟨s⟩                                               ║
║                                                                                ║
║   "Einmal erstellt, existiert eine Identität permanent."                      ║
║                                                                                ║
║   AKTIVITÄTS-MODULATION:                                                      ║
║       𝔸(s) = σ(activity_score(s))    ∈ [0, 1]                                 ║
║                                                                                ║
║       Inaktive Subjekte existieren weiterhin, aber:                           ║
║       • Ihr Einfluss auf 𝔼 sinkt                                              ║
║       • Ihre Stimmen in Governance werden abgewertet                          ║
║       • Sie können jederzeit reaktiviert werden                               ║
║                                                                                ║
║   → Absorbiert: A2 (Permanenz), E1 (Existenz), E2 (Aktivität)                ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ8 – Delegations-Struktur

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ8 (DELEGATIONS-STRUKTUR):                                       ║
║                                                                                ║
║       s ⊳ s' → 𝕋(s') ≤ 𝕋(s)                                                  ║
║                                                                                ║
║   Die Relation ⊳ ("delegiert an") ist eine strenge Halbordnung:               ║
║       • Irreflexiv:      ¬(s ⊳ s)                                             ║
║       • Antisymmetrisch: (s ⊳ s') ∧ (s' ⊳ s) → s = s' (Widerspruch)          ║
║       • Transitiv:       (s ⊳ s') ∧ (s' ⊳ s'') → (s ⊳ s'')                   ║
║                                                                                ║
║   IMPLIKATION:                                                                ║
║       Delegationsketten formen einen DAG (keine Zyklen möglich)               ║
║       Trust fließt immer "abwärts" in der Hierarchie                          ║
║                                                                                ║
║   → Absorbiert: A3 (Delegation), A4 (Tiefe), A10 (Kette), E3 (Struktur)      ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Theorem Τ2 (Aktivitäts-Fluss)

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   THEOREM Τ2 (AKTIVITÄTS-FLUSS):                                              ║
║                                                                                ║
║       s ⊳ s' ⟹ (𝔸(s') > 0 → 𝔸(s) ≥ δ·𝔸(s'))                                 ║
║                                                                                ║
║   "Wenn ein Delegierter aktiv ist, muss der Delegierende mindestens           ║
║    anteilig aktiv sein."                                                       ║
║                                                                                ║
║   HERLEITUNG: Aus Κ7 (Aktivität) + Κ8 (Delegation)                            ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

---

## IV. Kausale Algebra

### Κ9 – Kausale Struktur

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ9 (KAUSALE STRUKTUR):                                           ║
║                                                                                ║
║       Die Relation e₁ ⊲ e₂ ("e₁ ist kausal vor e₂") ist eine                  ║
║       strenge Halbordnung auf Events.                                         ║
║                                                                                ║
║       • Irreflexiv:      ¬(e ⊲ e)                                             ║
║       • Antisymmetrisch: (e₁ ⊲ e₂) → ¬(e₂ ⊲ e₁)                              ║
║       • Transitiv:       (e₁ ⊲ e₂) ∧ (e₂ ⊲ e₃) → (e₁ ⊲ e₃)                   ║
║                                                                                ║
║   IMPLIKATION:                                                                ║
║       Der Event-Graph ist ein DAG (Directed Acyclic Graph)                    ║
║       Zyklen sind strukturell unmöglich                                        ║
║                                                                                ║
║   → Absorbiert: A12 (Kausalität), A13 (Ordnung), A14 (DAG)                   ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ10 – Bezeugung-Finalität

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ10 (BEZEUGUNG-FINALITÄT):                                       ║
║                                                                                ║
║       |Witnesses(e)| ≥ θ_finality ⟹ Finality(e) ↑                             ║
║                                                                                ║
║   FINALITÄTS-SPEKTRUM:                                                        ║
║       NASCENT   → VALIDATED → WITNESSED → ANCHORED → ETERNAL                  ║
║                                                                                ║
║   MONOTONIE:                                                                  ║
║       Finality(e, t₁) ≤ Finality(e, t₂)  für t₁ < t₂                          ║
║       "Finalität kann nur steigen, nie fallen."                               ║
║                                                                                ║
║   → Absorbiert: A15 (Bezeugung), A16 (Finalität), A29 (Monotonie)            ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

---

## V. Prozess-Algebra

### Κ11 – Prozess-Korrektheit (Hoare-Logik)

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ11 (PROZESS-KORREKTHEIT):                                       ║
║                                                                                ║
║       {P} process {Q}                                                         ║
║                                                                                ║
║   "Wenn Vorbedingung P erfüllt ist und process ausgeführt wird,               ║
║    dann ist Nachbedingung Q erfüllt."                                         ║
║                                                                                ║
║   ANWENDUNG AUF SAGAS:                                                        ║
║       {budget_ok ∧ trust_ok} saga_step {resources_transferred}                ║
║                                                                                ║
║   → Absorbiert: P1 (Korrektheit), P3-P6 (Prozess-Regeln)                     ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ12 – Event-Erzeugung

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ12 (EVENT-ERZEUGUNG):                                           ║
║                                                                                ║
║       Jeder Prozess erzeugt genau ein Event pro atomarer Aktion.              ║
║                                                                                ║
║   STRUKTUR EINES EVENTS:                                                      ║
║       Event {                                                                 ║
║           id:        EventId (hash-basiert),                                  ║
║           parents:   Vec<EventId>,                                            ║
║           author:    DID,                                                     ║
║           payload:   Payload,                                                 ║
║           timestamp: LamportClock,                                            ║
║           signature: Signature,                                               ║
║           finality:  FinalityLevel,                                           ║
║       }                                                                       ║
║                                                                                ║
║   → Absorbiert: P2 (Event-Erzeugung)                                         ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ13 – Streaming-Fairness

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ13 (STREAMING-FAIRNESS):                                        ║
║                                                                                ║
║       ∀ streams S₁, S₂ : fair_share(S₁) ≈ fair_share(S₂)                      ║
║                                                                                ║
║   IMPLEMENTIERUNG:                                                            ║
║       • Round-Robin bei gleicher Priorität                                     ║
║       • Prioritäts-gewichtet bei unterschiedlichem Trust                      ║
║       • Keine Aushungerung (starvation-free)                                  ║
║                                                                                ║
║   → Absorbiert: A27 (Fairness), T5 (Streaming), T7 (Priorisierung)           ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ14 – Transaktions-Atomarität

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ14 (TRANSAKTIONS-ATOMARITÄT):                                   ║
║                                                                                ║
║       ∀ saga S : (all_steps_succeed(S) → commit(S))                           ║
║                  ∧ (any_step_fails(S) → compensate(S))                        ║
║                                                                                ║
║   SAGA-PATTERN:                                                               ║
║       Forward: [step₁, step₂, ..., stepₙ]                                     ║
║       Compensate: [comp₁, comp₂, ..., compₙ]                                  ║
║                                                                                ║
║   GARANTIE:                                                                   ║
║       Entweder alle Schritte erfolgreich ODER System im Ausgangszustand       ║
║                                                                                ║
║   → Absorbiert: A25 (Atomarität), PR2 (Saga)                                 ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

---

## VI. Weltformel-Präzisierung

### Κ15a – Informationstheoretische Surprisal

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   UNTER-AXIOM Κ15a (INFORMATIONSTHEORETISCHE SURPRISAL):                      ║
║                                                                                ║
║       ℐ(e|s) = −log₂ P(e | ℂ(s))                                              ║
║                                                                                ║
║   SHANNON-SURPRISAL:                                                          ║
║       Seltene Events haben hohe Surprisal                                      ║
║       Häufige Events haben niedrige Surprisal                                  ║
║                                                                                ║
║   TRUST-GEDÄMPFTE SURPRISAL (Anti-Hype):                                      ║
║       𝒮(s) = ‖𝕎(s)‖² · ℐ(s)                                                  ║
║                                                                                ║
║       Agent mit 𝕎 = 0.3:  𝒮 = 0.09 · ℐ   (91% Dämpfung)                      ║
║       Agent mit 𝕎 = 0.9:  𝒮 = 0.81 · ℐ   (19% Dämpfung)                      ║
║                                                                                ║
║   → Verhindert: Hype-Zyklen, Spam-Belohnung, Sybil-Novelty-Farming           ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ15b – Gewichtete Trust-Norm

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   UNTER-AXIOM Κ15b (GEWICHTETE TRUST-NORM):                                   ║
║                                                                                ║
║       ‖𝕎‖_w = √(Σᵢ wᵢ · 𝕎ᵢ²)     wobei Σwᵢ = 1                               ║
║                                                                                ║
║   KONTEXT-GEWICHTE:                                                           ║
║       Kontext              R     I     C     P     V     Ω                    ║
║       ─────────────────────────────────────────────────────                   ║
║       Finanztransaktion   0.30  0.25  0.15  0.10  0.15  0.05                  ║
║       Wissensaustausch    0.10  0.30  0.30  0.15  0.10  0.05                  ║
║       Governance          0.15  0.20  0.15  0.20  0.10  0.20                  ║
║       Default             0.17  0.17  0.17  0.17  0.16  0.16                  ║
║                                                                                ║
║   ALTERNATIVE: Vektorielle Sigmoid (behält alle 6 Dimensionen)                ║
║       σ⃗(𝕎) = (σ(𝕎_R), σ(𝕎_I), σ(𝕎_C), σ(𝕎_P), σ(𝕎_V), σ(𝕎_Ω))              ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ15c – Prinzipienbasierte Parameter

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   UNTER-AXIOM Κ15c (PRINZIPIENBASIERTE PARAMETER):                            ║
║                                                                                ║
║   1. TEMPORALE DECAY-RATE γ (aus Psychologie/Ebbinghaus):                     ║
║       γ_neg = ln(2) / (3 Jahre) ≈ 0.000633/Tag  (negative Events)             ║
║       γ_pos = ln(2) / (5 Jahre) ≈ 0.000380/Tag  (positive Events)             ║
║                                                                                ║
║       BEGRÜNDUNG: Negative Erfahrungen sollen schneller vergessen werden      ║
║                   (Resozialisierung), positive länger nachwirken (Reputation) ║
║                                                                                ║
║   2. SIGMOID-STEILHEIT k (aus Informationstheorie):                           ║
║       k = 1 (Standard-Logistik)                                               ║
║       BEGRÜNDUNG: k=1 maximiert Entropie-Transfer                             ║
║                                                                                ║
║   3. ASYMMETRIE-FAKTOR λ (aus Verhaltensökonomie/Kahneman-Tversky):           ║
║       λ_asym = 1.5 (konservativ)  für R, I, C, P                              ║
║       λ_asym = 2.0 (streng)       für V, Ω                                    ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ15d – Hierarchische Approximation

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   UNTER-AXIOM Κ15d (HIERARCHISCHE APPROXIMATION):                             ║
║                                                                                ║
║   PROBLEM: Exakte Berechnung von 𝔼 = Σₛ f(s) ist O(|𝒞|) – unpraktisch.       ║
║                                                                                ║
║   LÖSUNG 1: Hierarchische Aggregation (für Batch-Analyse)                     ║
║       𝔼 ≈ Σ_partitions 𝔼_partition                                            ║
║       𝔼_partition = |partition| · mean(sample(partition, k))                  ║
║       Komplexität: O(|Partitions| · k) statt O(|𝒞|)                           ║
║                                                                                ║
║   LÖSUNG 2: Streaming-Approximation (für Echtzeit)                            ║
║       𝔼(t+1) = α · 𝔼(t) + (1-α) · Σ_new f(s_new)                             ║
║       α = exp(-Δt / τ_update)    mit τ_update = 1 Stunde                      ║
║       Komplexität: O(|neue Events|) pro Update                                ║
║                                                                                ║
║   LÖSUNG 3: Importance Sampling (für Analyse)                                 ║
║       𝔼 ≈ (1/k) · Σᵢ f(sᵢ) / q(sᵢ)    wobei sᵢ ~ q(s) ∝ 𝔸(s) · ‖𝕎(s)‖       ║
║       Minimiert Varianz durch intelligentes Sampling                          ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Theorem Τ3 (Weltformel-Evolution)

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   THEOREM Τ3 (WELTFORMEL-EVOLUTION):                                          ║
║                                                                                ║
║       ∂𝔼/∂t = Σ [ ∂𝔸/∂t · f(𝕎,ℂ) + 𝔸 · ∂f/∂𝕎 · ∂𝕎/∂t + 𝔸 · ∂f/∂ℂ · ∂ℂ/∂t ] ║
║                 s                                                             ║
║                                                                                ║
║   INTERPRETATION (Kybernetisches System):                                     ║
║       Das System ist ein Gradient-Descent auf 𝔼:                              ║
║       • Agenten optimieren ihren eigenen Beitrag zu 𝔼                         ║
║       • Das Gesamtsystem "lernt" durch emergente Selektion                    ║
║       • Bösartige Agenten: 𝔸 → 0 (werden inaktiv durch Ausschluss)           ║
║       • Kompetente Agenten: 𝕎 → 1 (werden einflussreicher)                   ║
║       • Novelty-Farmer: 𝒮 → 0 durch quadratische Trust-Dämpfung (Κ15a)       ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

---

## VII. Humanismus

### Κ16 – Human-Alignment

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ16 (HUMAN-ALIGNMENT):                                           ║
║                                                                                ║
║       Ĥ(s) = { 2.0  wenn s = verified_human                                   ║
║                1.5  wenn controller(s) = human                                 ║
║                1.0  sonst }                                                    ║
║                                                                                ║
║   "Menschliche Interaktion ist doppelt wertvoll."                             ║
║                                                                                ║
║   IMPLIKATION:                                                                ║
║       Auch bei zunehmender Automatisierung bleiben menschliche Interessen     ║
║       priorisiert. KI-Agenten können nicht durch bloße Aktivität Menschen     ║
║       übertreffen.                                                            ║
║                                                                                ║
║   → Absorbiert: H1 (Human-Alignment)                                         ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ17 – Temporale Vergebung

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ17 (TEMPORALE VERGEBUNG):                                       ║
║                                                                                ║
║       w(e, t) = exp(-γ · age(e))                                              ║
║                                                                                ║
║       γ_negative = ln(2) / (3 Jahre) ≈ 0.000633/Tag                           ║
║       γ_positive = ln(2) / (5 Jahre) ≈ 0.000380/Tag                           ║
║                                                                                ║
║   "Negative Events verblassen schneller als positive."                        ║
║                                                                                ║
║   BEGRÜNDUNG (Psychologie):                                                   ║
║       • Ermöglicht Resozialisierung nach Fehlverhalten                        ║
║       • Positive Reputation bleibt länger relevant                            ║
║       • Spiegelt menschliche Vergebungsbereitschaft                           ║
║                                                                                ║
║   → Absorbiert: A9 (Decay), H3 (Temporale Gnade)                             ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

---

## VIII. Konsens

### Κ18 – Konsens-Konstitution

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ18 (KONSENS-KONSTITUTION):                                      ║
║                                                                                ║
║                    Σ 𝕎(s) · [s ⊢ φ]                                           ║
║       Ψ(Σ)(φ) = ─────────────────────                                         ║
║                      Σ 𝕎(s)                                                   ║
║                  s ⊢ Σ                                                        ║
║                                                                                ║
║   EIGENSCHAFTEN:                                                              ║
║       Ψ(Σ)(φ) ∈ [0,1]                           [Normierung]                  ║
║       Ψ(Σ)(φ) > θ_konsens → φ ist Partition-Wahrheit                          ║
║       θ_konsens = 2/3 (Supermajorität)                                        ║
║                                                                                ║
║   → Absorbiert: E13 (Partition-Konsens), E14 (State-Kommunikation)           ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Theorem Τ4 (Konsens-Konvergenz)

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   THEOREM Τ4 (KONSENS-KONVERGENZ):                                            ║
║                                                                                ║
║       lim_{t→∞} Ψ(Σ)(φ) = μ_true(φ)    unter folgenden Bedingungen:          ║
║                                                                                ║
║       (i)   majority(honest) > 1/2                                            ║
║       (ii)  ∀ honest s : V(s) > 0.5                                           ║
║       (iii) 𝔸(dishonest) → 0 über Zeit (durch Κ19-Κ21)                       ║
║                                                                                ║
║   BEWEIS-SKIZZE:                                                              ║
║       Aus Κ4 (Asymmetrie): Unehrliche Agenten verlieren Trust schneller       ║
║       Aus Κ18 (Gewichtung): Niedriger Trust = niedrigere Stimme               ║
║       Aus Κ19-Κ21 (Schutz): Sybil-Ringe werden gedämpft                       ║
║       ⟹ Honest agents dominate ⟹ Ψ konvergiert zur Wahrheit                  ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

---

## IX. Schutz-Algebra

### Κ19 – Anti-Calcification

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ19 (ANTI-CALCIFICATION):                                        ║
║                                                                                ║
║       rank_final(s) = relevance(s) · (σ(s)^γ + β·(1-𝔸(s))·e^(-age/τ))        ║
║                       · (1 + ξ·noise)                                         ║
║                                                                                ║
║   KOMPONENTEN:                                                                ║
║       σ(s)^γ                    Diminishing Returns (γ = 0.7)                 ║
║       β·(1-𝔸(s))·e^(-age/τ)    Exploration Bonus für Newcomer                ║
║       (1 + ξ·noise)            Stochastic Fairness (ξ = 0.1)                  ║
║                                                                                ║
║   → Absorbiert: S1 (Exploration), S3 (Jitter), S4 (Diminishing Returns)      ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ20 – Diversity-Requirement

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ20 (DIVERSITY-REQUIREMENT):                                     ║
║                                                                                ║
║       Δ𝕎(s, tx) = base · Quality(tx) · diversity_mult(s) · (1 - collusion(tx))║
║                                                                                ║
║       diversity_mult(s) = min(1, unique_partners(s, τ) / θ_diversity)         ║
║       collusion(tx) = f(similarity, exclusivity, temporal_correlation)        ║
║                                                                                ║
║   EFFEKT:                                                                     ║
║       Sybil-Ringe erhalten Malus (geringe Diversity)                          ║
║       Echte diverse Interaktion wird belohnt                                   ║
║                                                                                ║
║   → Absorbiert: S9-S12 (Quality Objectivity)                                 ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ21 – Quadratic Governance

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ21 (QUADRATIC GOVERNANCE):                                      ║
║                                                                                ║
║       vote_weight(s, p) = √(σ(s)) · relevance(s, domain(p)) · freshness(s)   ║
║                                                                                ║
║       freshness(s) = 1 - (consecutive_rounds(s) / max_rounds)²                ║
║                                                                                ║
║   EFFEKTE:                                                                    ║
║       • √σ: Quadratwurzel verhindert Plutokratie                             ║
║       • relevance: Domänenspezifische Expertise zählt                         ║
║       • freshness: Dauer-Abstimmer werden abgewertet                          ║
║                                                                                ║
║   → Absorbiert: S13-S18 (Fair Governance)                                    ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

---

## X. Peer-Algebra

### Κ22 – Intent-Auflösung

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ22 (INTENT-AUFLÖSUNG):                                          ║
║                                                                                ║
║       ∀ Intent I : ∃! Saga S = compose(I) mit S = [s₁, ..., sₙ]               ║
║                                                                                ║
║   COMPOSER-FUNKTION:                                                          ║
║       compose: Intent → Saga                                                  ║
║                                                                                ║
║   ALGORITHMUS (Rückwärts-Auflösung):                                          ║
║       1. Parse goal → required_resources                                      ║
║       2. ∀ resource: find_source(resource, budget)                            ║
║       3. Build dependency_graph                                               ║
║       4. Topological_sort → execution_order                                   ║
║       5. Return Saga                                                          ║
║                                                                                ║
║   → Absorbiert: PR1 (Intent-Auflösung)                                       ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ23 – Gateway-Vollständigkeit

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ23 (GATEWAY-VOLLSTÄNDIGKEIT):                                   ║
║                                                                                ║
║       ∀ (src → tgt) ∈ Saga : guard(user, tgt) muss evaluiert werden           ║
║                                                                                ║
║       guard(u, ctx) = ∧ᵢ Predicateᵢ(u.identity, u.trust, ctx.rules)           ║
║                                                                                ║
║   PRÄDIKATE:                                                                  ║
║       • min_trust_check:     ‖𝕎(u)‖ ≥ ctx.min_trust                          ║
║       • credential_check:    u.credentials ⊇ ctx.required_creds               ║
║       • rule_check:          ∀ rule ∈ ctx.rules : u satisfies rule            ║
║                                                                                ║
║   → Absorbiert: PR3 (Gateway-Vollständigkeit), A22 (Erlaubnis)               ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ24 – Funktor-Transformation

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ24 (FUNKTOR-TRANSFORMATION):                                    ║
║                                                                                ║
║       ∀ boundary_crossing : 𝕎_target = M_ctx × 𝕎_source                       ║
║                                                                                ║
║       mit ‖M_ctx‖ ≤ 1    (Trust kann nicht steigen)                           ║
║                                                                                ║
║   TRUST-DÄMPFUNGS-MATRIX:                                                     ║
║       M_ctx ist eine 6×6-Matrix, die Trust bei Realm-Übergang dämpft          ║
║       Diagonalelemente ≤ 1, Off-Diagonal ermöglicht Dimension-Mapping         ║
║                                                                                ║
║   BEISPIEL:                                                                   ║
║       Crossing von "Gaming" nach "Finance":                                   ║
║       Competence in Gaming ≠ Competence in Finance → starke C-Dämpfung       ║
║                                                                                ║
║   → Absorbiert: PR4, PR6, Q7 (Realm-Funktor)                                 ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

---

## XI. System-Garantien

### Κ25 – Determinismus

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ25 (DETERMINISMUS):                                             ║
║                                                                                ║
║       ∀ prog ∈ ECLVM, input : exec(prog, input) = exec(prog, input)           ║
║                                                                                ║
║   "Gleiche Eingaben → Gleiche Ausgaben (für Logic Guards, Policy Evaluation)"║
║                                                                                ║
║   IMPLIKATION:                                                                ║
║       ECLVM-Programme sind pure functions                                     ║
║       Keine Seiteneffekte, keine Randomness (außer explizit)                  ║
║       Ermöglicht Replay und Verifizierung                                      ║
║                                                                                ║
║   → Absorbiert: A28 (Determinismus)                                          ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ26 – Offenheit

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ26 (OFFENHEIT):                                                 ║
║                                                                                ║
║       ∀s : (s erfüllt rules(𝒞)) → ◇(s ∈ 𝒞)                                   ║
║                                                                                ║
║   "Jeder, der die Regeln erfüllt, kann beitreten."                            ║
║                                                                                ║
║   IMPLIKATION:                                                                ║
║       Keine Gatekeeping durch bestehende Mitglieder                           ║
║       Regeln sind transparent und überprüfbar                                  ║
║       Permissionless by default, permissions by rules                          ║
║                                                                                ║
║   → Absorbiert: A30 (Offenheit)                                              ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ27 – Semantische Verankerung

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ27 (SEMANTISCHE VERANKERUNG):                                   ║
║                                                                                ║
║       ∀ Blueprint B : ∃ NLD(B) ∧ ∃ FormalSpec(B) ∧ Equivalent(NLD, FormalSpec)║
║                                                                                ║
║   "Jede Abstraktion muss menschlich verständlich dokumentiert sein."          ║
║                                                                                ║
║   IMPLIKATION:                                                                ║
║       Keine "magic" Konstanten ohne Erklärung                                 ║
║       Alle Parameter haben dokumentierte Herleitung                            ║
║       Code und Dokumentation sind äquivalent                                   ║
║                                                                                ║
║   → Absorbiert: H4 (Semantische Verankerung)                                 ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

### Κ28 – Verhältnismäßigkeit

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KERN-AXIOM Κ28 (VERHÄLTNISMÄSSIGKEIT):                                      ║
║                                                                                ║
║       ∀ tx : Cost_verification(tx) ≤ α · Value(tx)    mit α = 0.05            ║
║                                                                                ║
║   "Verifikationskosten dürfen 5% des Transaktionswerts nicht übersteigen."    ║
║                                                                                ║
║   IMPLIKATION:                                                                ║
║       Micro-Transaktionen sind praktikabel                                     ║
║       Keine prohibitiven Gebühren                                              ║
║       Skaliert mit Transaktionsgröße                                           ║
║                                                                                ║
║   → Absorbiert: H2 (Verhältnismäßigkeit)                                     ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

---

## XII. Konsolidierungs-Tabelle

### Axiom-Reduktion: 126 Ursprünglich → 28+4 Kern-Axiome

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   KONSOLIDIERUNGS-TABELLE                                                     ║
║                                                                                ║
║   KERN-AXIOM    ABSORBIERT                              KATEGORIE             ║
║   ─────────────────────────────────────────────────────────────────────────   ║
║   Κ1            A18, A19, E11                           Kategorie             ║
║   Κ2            A6-A11 (Teil), E5-E7, PR6               Trust-Funktor         ║
║   Κ3            E6, E7                                  Trust-Dimensionen     ║
║   Κ4            A8, E9                                  Asymmetrie            ║
║   Κ5            A11, C4, E13 (Teil)                     Kombination           ║
║   Κ6            A1, O1                                  Identität             ║
║   Κ7            A2, E1, E2                              Permanenz+Aktivität   ║
║   Κ8            A3, A4, A10, E3                         Delegation            ║
║   Κ9            A12, A13, A14                           Kausalität            ║
║   Κ10           A15, A16, A29                           Bezeugung+Finalität   ║
║   Κ11           P1, P3, P4, P5, P6                      Prozess-Korrektheit   ║
║   Κ12           P2                                      Event-Erzeugung       ║
║   Κ13           A27, T5, T7                             Streaming-Fairness    ║
║   Κ14           A25, PR2                                Atomarität            ║
║   Κ15a          NEU: Shannon-Surprisal                  Informationstheorie   ║
║   Κ15b          NEU: Trust-Vektor-Norm                  Aggregation           ║
║   Κ15c          NEU: Prinzipienbasierte Parameter       Parameterherleitung   ║
║   Κ15d          NEU: Hierarchische Approximation        Skalierbarkeit        ║
║   Κ16           H1                                      Human-Alignment       ║
║   Κ17           A9, H3                                  Temporale Vergebung   ║
║   Κ18           E13, E14                                Konsens               ║
║   Κ19           S1, S3, S4                              Anti-Calcification    ║
║   Κ20           S9, S10, S11, S12                       Quality Objectivity   ║
║   Κ21           S13, S14, S15, S16, S17, S18            Fair Governance       ║
║   Κ22           PR1                                     Intent-Auflösung      ║
║   Κ23           PR3, A22                                Gateway               ║
║   Κ24           PR4, PR6, Q7                            Funktor-Transformation║
║   Κ25           A28                                     Determinismus         ║
║   Κ26           A30                                     Offenheit             ║
║   Κ27           H4                                      Semantische Verankerung║
║   Κ28           H2                                      Verhältnismäßigkeit   ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

---

## XIII. Theorem-Übersicht

| Theorem | Name                 | Herleitung         | Aussage                                 |
| ------- | -------------------- | ------------------ | --------------------------------------- |
| Τ1      | Ketten-Trust         | Κ5                 | Längere Ketten → niedrigerer Trust      |
| Τ2      | Aktivitäts-Fluss     | Κ7, Κ8             | Delegierter aktiv ⟹ Delegierender aktiv |
| Τ3      | Weltformel-Evolution | Κ2, Κ7, Κ9, Κ15a-d | System ist Gradient-Descent auf 𝔼       |
| Τ4      | Konsens-Konvergenz   | Κ4, Κ18, Κ20       | Ψ → Wahrheit bei honest majority        |
| Τ5      | System-Konsistenz    | Alle Κ             | Kein Axiom widerspricht einem anderen   |

---

_Weiter zu [03-SYSTEM-ARCHITECTURE.md](03-SYSTEM-ARCHITECTURE.md) für die 4-Schichten-Architektur._
