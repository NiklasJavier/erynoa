# Appendix

> **Version:** V5.0 – Konsolidiert

---

## I. Glossar

### A

**Aktivität 𝔸(s)**
: Präsenz-Maß eines Subjekts basierend auf kürzlichen Events: $\mathbb{A}(s) = \frac{|E_{recent}|}{|E_{recent}| + \kappa}$

**Anti-Calcification (Κ19)**
: Schutz-Mechanismus gegen Macht-Konzentration durch Diminishing Returns und Exploration-Bonus

**Asymmetrie (Κ4)**
: Trust sinkt schneller als er steigt: $\Delta^{-} = \lambda_{asym} \cdot \Delta^{+}$

**Attestation**
: Bestätigung von Eigenschaften oder Fähigkeiten einer Entität durch eine andere

### B

**BLS12-381**
: Kryptographische Kurve für Signatur-Aggregation (Consensus)

### C

**Causal Graph ℂ**
: Directed Acyclic Graph (DAG) aller Events mit kausaler Ordnung

**Consensus (Κ18)**
: Gewichteter Partition-Konsens: $\Psi(\Sigma)(\varphi) = \frac{\Sigma \mathbb{W}(s) \cdot [s \vdash \varphi]}{\Sigma \mathbb{W}(s)}$

**Count-Min Sketch**
: Probabilistische Datenstruktur für Frequenz-Schätzung (Κ15d)

### D

**DAG (Directed Acyclic Graph)**
: Gerichteter azyklischer Graph für Events (Κ9)

**Delegation (Κ8)**
: Trust-Vererbung an Sub-Identitäten: $s \rhd s' \Rightarrow \mathbb{T}(s') \leq \mathbb{T}(s)$

**DID (Decentralized Identifier)**
: Dezentrale Identität gemäß W3C Standard, Format: `did:erynoa:{namespace}:{id}`

**Diversity (Κ20)**
: Shannon-Entropie-basierte Diversitätsmessung: $D(\mathcal{C}) = H(dist) / H_{max}$

### E

**ECLVM**
: Erynoa Core Language Virtual Machine für Policy-Ausführung

**Event**
: Atomare Zustandsänderung im DAG mit Autor, Payload, Parents und Timestamp

### F

**Finalität**
: Grad der Unveränderlichkeit eines Events (Nascent → Witnessed → Finalized)

### G

**Gateway Guard (Κ23)**
: Validierungs-Instanz für Realm-Übergänge

**Gini-Koeffizient**
: Ungleichheits-Maß: $G = \frac{\sum|x_i - x_j|}{2n^2\bar{x}}$

### H

**Human-Faktor Ĥ (Κ16)**
: Bonus für menschlich verifizierte Entitäten: 1.0 (unbekannt), 1.2 (kontrolliert), 1.5 (verifiziert)

### I

**Intent**
: Strukturierte Nutzerabsicht zur Saga-Auflösung

### K

**Κ (Kappa)**
: Präfix für Kern-Axiome (Κ1-Κ28)

### L

**Lamport Clock**
: Logische Uhr für Event-Ordnung

### M

**Mana**
: Bandwidth-Ressource im IPS-System

### N

**Namespace**
: Identitäts-Kategorie (self, guild, spirit, thing, vessel, source, craft, vault, pact, circle)

### O

**Omega Ω**
: Axiom-Treue Dimension im Trust-Vektor

### P

**Partition**
: Unter-Realm für lokalen Konsens

**Prestige P**
: Reputations-Dimension im Trust-Vektor

### Q

**Quadratic Voting (Κ21)**
: Abstimmungs-Mechanismus: $cost(v) = v^2$, $power(c) = \sqrt{c}$

### R

**Realm**
: Regelraum mit vererbter Struktur (Root → Virtual → Partition)

**Reliability R**
: Zuverlässigkeits-Dimension im Trust-Vektor

### S

**Saga (Κ22)**
: Multi-Step-Transaktion mit Kompensations-Logik

**Shannon-Entropie**
: $H = -\sum p_i \log_2(p_i)$

**Sigmoid σ⃗**
: Sättigungs-Funktion: $\sigma(x) = \frac{1}{1+e^{-x}}$ (Κ15c)

**Surprisal ℐ**
: Informationsgehalt: $\mathcal{I} = -\log_2 P$

**Surprisal (gedämpft) 𝒮**
: Trust-gewichtete Surprisal: $\mathcal{S}(s) = \|\mathbb{W}(s)\|^2 \cdot \mathcal{I}(s)$ (Κ15a)

### T

**TAT-Lifecycle (Κ13-Κ14)**
: Transaction As Transition: seek → propose → agree → execute → settle

**Trust-Vektor 𝕎**
: 6-dimensionaler Vertrauens-Vektor: (R, I, C, P, V, Ω) ∈ [0,1]⁶

### V

**Vigilance V**
: Wachsamkeits-Dimension im Trust-Vektor

**VirtualRealm**
: Benutzer-erstellter Regelraum unter RootRealm

### W

**Weltformel 𝔼 (Κ15b)**
: $\mathbb{E} = \sum \mathbb{A}(s) \cdot \sigma(\|\mathbb{W}(s)\|_w \cdot \ln|\mathbb{C}(s)| \cdot \mathcal{S}(s)) \cdot \hat{H}(s) \cdot w(s,t)$

---

## II. Symbol-Verzeichnis

### Griechische Symbole

| Symbol | Name   | Bedeutung             |
| ------ | ------ | --------------------- |
| Κ      | Kappa  | Kern-Axiom Präfix     |
| Μ      | My     | Meta-Axiom Präfix     |
| Τ      | Tau    | Theorem Präfix        |
| Σ      | Sigma  | Summe / Partition     |
| Ψ      | Psi    | Konsens-Funktion      |
| Ω      | Omega  | Axiom-Treue Dimension |
| σ      | sigma  | Sigmoid-Funktion      |
| λ      | lambda | Asymmetrie-Faktor     |
| τ      | tau    | Zeitkonstante         |
| γ      | gamma  | Diminishing Returns   |
| β      | beta   | Exploration Bonus     |
| ξ      | xi     | Stochastic Fairness   |
| θ      | theta  | Schwellwert           |

### Mathematische Symbole

| Symbol | Bedeutung                       |
| ------ | ------------------------------- |
| 𝔸      | Aktivität                       |
| 𝕎      | Trust-Vektor                    |
| ℂ      | Kausaler Graph                  |
| ℐ      | Surprisal                       |
| 𝒮      | Gedämpfte Surprisal             |
| 𝔼      | Weltformel-Ergebnis             |
| Ĥ      | Human-Faktor                    |
| ‖·‖    | Norm                            |
| ⊕      | Probabilistische Kombination    |
| ⊲      | Kausal-vor                      |
| ⊳      | Delegiert-an                    |
| □      | Temporal: immer in Zukunft      |
| ◇      | Temporal: irgendwann in Zukunft |

---

## III. Axiom-Index

### Kern-Axiome (Κ1-Κ28)

| Axiom | Name                      | Kurzbeschreibung                         |
| ----- | ------------------------- | ---------------------------------------- |
| Κ1    | Monotone Regelvererbung   | Kind-Realms erben Eltern-Regeln          |
| Κ2    | Trust als Funktor         | Kategorientheoretische Trust-Struktur    |
| Κ3    | Orthogonalität            | Trust-Dimensionen unabhängig             |
| Κ4    | Asymmetrie                | Negative Δ > Positive Δ                  |
| Κ5    | Probabilistische Kombi    | t₁ ⊕ t₂ = 1-(1-t₁)(1-t₂)                 |
| Κ6    | Existenz-Eindeutigkeit    | Jede Entität hat genau eine DID          |
| Κ7    | Permanenz                 | Erzeugte DIDs existieren permanent       |
| Κ8    | Delegations-Struktur      | Sub-Identitäten mit Trust-Beschränkung   |
| Κ9    | Kausale Struktur          | Events bilden DAG                        |
| Κ10   | Zeugnis-Permanenz         | Bezeugte Events bleiben bezeugt          |
| Κ11   | Determinismus             | Gleiche Inputs → gleiche Outputs         |
| Κ12   | Event-Minimalität         | Jede Aktion erzeugt ≥1 Event             |
| Κ13   | TAT-Konsistenz            | Transaktions-Lifecycle                   |
| Κ14   | Streaming-Äquivalenz      | Kontinuierliche = diskrete Transaktionen |
| Κ15a  | Trust-gedämpfte Surprisal | 𝒮 = ‖𝕎‖² · ℐ                             |
| Κ15b  | Weltformel                | Vollständige 𝔼-Definition                |
| Κ15c  | Sigmoid-Sättigung         | Bounded Growth                           |
| Κ15d  | Streaming-Approximation   | Effiziente Online-Berechnung             |
| Κ16   | Human-Alignment           | Ĥ-Bonus für Menschen                     |
| Κ17   | Würde-Unantastbarkeit     | Minimum Trust-Floor                      |
| Κ18   | Gewichteter Konsens       | Trust-gewichtete Abstimmung              |
| Κ19   | Anti-Verkalkung           | Diminishing Returns + Exploration        |
| Κ20   | Diversity-Erhaltung       | Shannon-Entropie-Schwelle                |
| Κ21   | Quadratisches Voting      | cost(v) = v²                             |
| Κ22   | Saga-Auflösung            | Intent → eindeutige Saga                 |
| Κ23   | Gateway-Vollständigkeit   | Alle Predicates für Crossing             |
| Κ24   | Trust-Dämpfung            | ‖M_ctx‖ ≤ 1                              |
| Κ25   | ECLVM-Determinismus       | Deterministische Ausführung              |
| Κ26   | Open-by-Default           | Realms standardmäßig offen               |
| Κ27   | Dokumentations-Pflicht    | Regeln müssen dokumentiert sein          |
| Κ28   | Gebühren-Beschränkung     | Faire Kostenverteilung                   |

### Theoreme (Τ1-Τ5)

| Theorem | Name               | Aussage                                   |
| ------- | ------------------ | ----------------------------------------- |
| Τ1      | Ketten-Trust       | Trust über Pfade berechenbar              |
| Τ2      | Aktivitäts-Fluss   | Aktivität fließt zu Delegierenden         |
| Τ3      | Skalierbarkeit     | 𝔼 bleibt beschränkt bei wachsenden Events |
| Τ4      | Konsens-Konvergenz | Gewichteter Konsens konvergiert           |
| Τ5      | DAG-Konsistenz     | DAG bleibt zyklusfrei                     |

---

## IV. Namespace-Referenz

| Namespace | Symbol | Beschreibung      | Beispiel-DID                   |
| --------- | ------ | ----------------- | ------------------------------ |
| self      | 👤     | Natürliche Person | did:erynoa:self:alice          |
| guild     | 🏢     | Organisation      | did:erynoa:guild:acme-corp     |
| spirit    | 🤖     | KI-Agent          | did:erynoa:spirit:trading-bot  |
| thing     | 📱     | IoT-Gerät         | did:erynoa:thing:sensor-001    |
| vessel    | ⚓     | Transportmittel   | did:erynoa:vessel:container-42 |
| source    | 🔋     | Energiequelle     | did:erynoa:source:solar-farm   |
| craft     | 🛠️     | Produkt/Handwerk  | did:erynoa:craft:batch-2024    |
| vault     | 🔐     | Tresor/Multi-Sig  | did:erynoa:vault:treasury      |
| pact      | 📜     | Vertrag           | did:erynoa:pact:supply-2026    |
| circle    | ⭕     | DAO/Gruppe        | did:erynoa:circle:governance   |

---

## V. Kontext-Gewichte

Standard-Gewichtungen für Trust-Norm nach Kontext:

| Kontext    | R    | I    | C    | P    | V    | Ω    |
| ---------- | ---- | ---- | ---- | ---- | ---- | ---- |
| Default    | 0.17 | 0.17 | 0.17 | 0.17 | 0.17 | 0.17 |
| Finance    | 0.25 | 0.30 | 0.15 | 0.10 | 0.10 | 0.10 |
| Energy     | 0.20 | 0.20 | 0.30 | 0.10 | 0.10 | 0.10 |
| Governance | 0.15 | 0.25 | 0.15 | 0.15 | 0.15 | 0.15 |
| Gaming     | 0.10 | 0.10 | 0.30 | 0.30 | 0.10 | 0.10 |

---

## VI. Finalitäts-Spektrum

| Level     | Symbol | Threshold     | Reversibilität      |
| --------- | ------ | ------------- | ------------------- |
| Nascent   | ○      | 0 Witnesses   | Vollständig         |
| Witnessed | ◐      | < 2/3 Konsens | Erschwert           |
| Finalized | ●      | ≥ 2/3 Konsens | Praktisch unmöglich |

---

## VII. Parameter-Defaults

| Parameter        | Wert | Axiom | Beschreibung               |
| ---------------- | ---- | ----- | -------------------------- |
| λ_asym (R,I,C,P) | 1.5  | Κ4    | Asymmetrie-Faktor          |
| λ_asym (V,Ω)     | 2.0  | Κ4    | Erhöhte Asymmetrie         |
| κ (Aktivität)    | 10   | Κ15b  | Aktivitäts-Sättigung       |
| τ_update         | 1h   | Κ15d  | Streaming-Update-Intervall |
| γ (Diminishing)  | 0.7  | Κ19   | Power-Reduktion            |
| β (Exploration)  | 0.1  | Κ19   | Newcomer-Bonus             |
| ξ (Fairness)     | 0.1  | Κ19   | Stochastischer Faktor      |
| θ_diversity      | 5    | Κ20   | Min. unique Partners       |
| θ_konsens        | 2/3  | Κ18   | Konsens-Schwelle           |
| Trust Floor      | 0.01 | Κ17   | Minimum Trust              |

---

## VIII. Changelog von V4

### Konsolidierung

- **126 → 32 Axiome**: Redundanzen eliminiert, Kern-Axiome konsolidiert
- **Einheitliche Notation**: Konsistente Verwendung von Κ-Präfix
- **Unter-Axiome**: Κ15a-d für Weltformel-Präzision

### Neue Struktur

| V4 Dokumente           | V5 Ziel                         |
| ---------------------- | ------------------------------- |
| LOGIC.md               | 02-AXIOM-SYSTEM.md              |
| SYSTEM-ARCHITECTURE.md | 03-SYSTEM-ARCHITECTURE.md       |
| STATE-MANAGEMENT.md    | 04-STATE-MANAGEMENT.md          |
| FACHKONZEPT.md         | 01-VISION-AND-FOUNDATIONS.md    |
| CLI-REFERENCE.md       | 06-CLI-REFERENCE.md             |
| P2P-_.md, LOGIC-_.md   | In relevante Kapitel integriert |

### Verbesserungen

- **Klarere Axiom-Hierarchie**: Meta → Kern → Unter-Axiome
- **Explizite Theoreme**: Τ1-Τ5 formalisiert
- **Vollständiges Glossar**: Alle Terme definiert
- **Symbol-Index**: Schnelle Referenz

---

## IX. Referenzen

### Standards

- W3C DID Core Specification 1.0
- W3C Verifiable Credentials Data Model 2.0
- IPFS Content Identifiers (CID)

### Kryptographie

- Ed25519: Bernstein et al. (2012)
- BLS12-381: Boneh, Lynn, Shacham (2001)

### Datenstrukturen

- Count-Min Sketch: Cormode, Muthukrishnan (2005)
- MinHash: Broder (1997)

### Konzepte

- Quadratic Voting: Posner, Weyl (2018)
- Byzantine Fault Tolerance: Lamport, Shostak, Pease (1982)

---

_Ende der Dokumentation._
