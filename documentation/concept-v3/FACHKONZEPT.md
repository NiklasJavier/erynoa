# Erynoa Fachkonzept V6.1

> **Version:** 6.1 – Probabilistische Kybernetische Architektur
> **Datum:** Januar 2026
> **Status:** Vollständiges Fachkonzept
> **Grundlage:** 112 Axiome über 7 Ebenen
> **Leitprinzip:** Intelligenz im Dienste des Lebens

---

## Einleitung

Erynoa ist ein dezentrales Ökosystem für vertrauensbasierte Interaktionen zwischen Menschen, Maschinen und autonomen Agenten. Das System ermöglicht es Teilnehmern, Werte auszutauschen, Vereinbarungen zu schließen und Transaktionen durchzuführen, ohne sich auf zentrale Autoritäten verlassen zu müssen. Anstelle institutionellen Vertrauens tritt mathematisch fundiertes, emergentes Vertrauen, das aus der nachprüfbaren Geschichte aller Interaktionen entsteht.

Das Fundament von Erynoa bildet eine mathematische Systemgleichung, die beschreibt, wie der Gesamtwert des Systems aus den Beiträgen seiner Teilnehmer entsteht. Diese Formel ist nicht bloß eine abstrakte Beschreibung, sondern das operative Herzstück des Systems: Jede Transaktion, jede Interaktion, jede Governance-Entscheidung verändert die Parameter dieser Formel und damit den Zustand des gesamten Netzwerks.

Die Architektur von Erynoa ist in sieben aufeinander aufbauenden Ebenen organisiert. Jede Ebene adressiert eine fundamentale Herausforderung verteilter Systeme. Die erste Ebene garantiert die Korrektheit grundlegender Operationen wie Identität und Kausalität. Die zweite Ebene ermöglicht emergente Intelligenz durch kollektive Validierung. Die dritte Ebene formalisiert Handlungen und Transaktionen. Die vierte Ebene definiert die Substanz des Systems in Form von Assets, Services und Credentials. Die fünfte Ebene schützt vor Degeneration und Machtkonzentration. Die sechste Ebene macht das System lebendig und anpassungsfähig durch kybernetische Feedback-Schleifen. Die siebte und höchste Ebene stellt sicher, dass das gesamte System dem menschlichen Gedeihen dient.

---

## Teil I: Die Systemgleichung

### Die mathematische Grundlage

Das Herzstück von Erynoa ist eine mathematische Formel, die den Gesamtzustand des Systems zu jedem Zeitpunkt beschreibt. Diese Systemgleichung aggregiert die Beiträge aller aktiven Agenten und drückt damit die kollektive Intelligenz des Netzwerks aus:

**𝔼 = Σ A(s) · σ( W(s) · ln|C(s)| · N(s) / E(s) ) · H(s) · w(s,t)**

Die Variablen haben folgende Bedeutung:

- **𝔼** ist der Systemwert, ein skalares Maß für die Gesundheit und Intelligenz des Gesamtnetzwerks
- **s** iteriert über alle aktiven Agenten im System
- **A(s)** ist die Aktivitätspräsenz des Agenten, ein Wert zwischen 0 und 1
- **W(s)** ist die Wächter-Metrik, ein multidimensionaler Vertrauensscore
- **C(s)** ist die kausale Geschichte, gemessen als Anzahl bezeugter Events
- **N(s)** ist der Novelty-Score, der neue, verifizierte Informationen misst
- **E(s)** ist der Erwartungswert basierend auf der historischen Vorhersagbarkeit
- **σ** ist die Sigmoid-Funktion σ(x) = 1 / (1 + e^(-x)), die alle Werte auf (0,1) normiert
- **H(s)** ist der Human-Alignment-Faktor (2.0 für Menschen, 1.5 für human-kontrolliert, 1.0 sonst)
- **w(s,t)** ist die temporale Gewichtung, die ältere Events exponentiell abklingen lässt

Diese Formel ist klassische Wahrscheinlichkeitstheorie und Statistik. Sie erfordert keine exotische Hardware und kann auf jedem modernen Server berechnet werden. Die Berechnung für einen einzelnen Agenten benötigt O(log n) Zeit, wobei n die Anzahl seiner Events ist.

### Vertrauen als Wahrscheinlichkeitsverteilung

Ein zentrales Konzept in Erynoa ist, dass Vertrauen keine feste Zahl ist, sondern eine Wahrscheinlichkeitsverteilung. Wenn wir sagen, ein Agent hat Trust 0.7, meinen wir damit nicht, dass er zu 70% vertrauenswürdig ist. Wir meinen, dass basierend auf den verfügbaren Daten unser bester Schätzer für seinen wahren Vertrauenswert 0.7 ist, mit einer gewissen Unsicherheit.

Diese Unsicherheit wird durch ein Konfidenzintervall ausgedrückt. Ein neuer Agent mit wenigen Interaktionen könnte einen geschätzten Trust von 0.7 haben, aber ein breites 95%-Konfidenzintervall von [0.4, 0.9]. Ein etablierter Agent mit tausenden Interaktionen könnte denselben geschätzten Trust von 0.7 haben, aber ein enges Intervall von [0.68, 0.72].

Diese Modellierung hat praktische Konsequenzen. Das System zeigt Nutzern nicht rohe Zahlen wie 0.723456, die eine Scheinpräzision suggerieren. Stattdessen zeigt es qualitative Level wie "Verified" oder "Caution" zusammen mit der Konfidenz "High" oder "Low". Ein Agent mit Trust 0.7 und hoher Konfidenz ist anders zu behandeln als einer mit Trust 0.7 und niedriger Konfidenz.

### Lazy Evaluation

Eine wichtige Optimierung ist die verzögerte Auswertung (Lazy Evaluation). Das System berechnet nicht permanent die Trust-Werte aller Agenten. Stattdessen speichert es die Rohdaten (Events, Attestationen, Credentials) und berechnet den Trust-Wert erst, wenn er tatsächlich benötigt wird – typischerweise wenn eine Transaktion ansteht.

Die Berechnung kann auf verschiedenen Detailstufen erfolgen:

- **Minimal:** Nur den Erwartungswert berechnen, ohne Konfidenzintervall
- **Standard:** Erwartungswert plus 95%-Konfidenzintervall
- **Vollständig:** Komplette Posterior-Verteilung mit allen Momenten

Die Wahl der Detailstufe hängt vom Transaktionswert ab. Für einen Kaffee reicht Minimal; für einen Hauskauf ist Vollständig angemessen.

### Die Komponenten im Detail

Die **Aktivitätspräsenz A(s)** misst, wie präsent ein Agent im System ist. Die Formel lautet:

A(s) = |{e ∈ C(s) : age(e) < τ}| / (|{e ∈ C(s) : age(e) < τ}| + κ)

Dabei ist τ das Aktivitäts-Zeitfenster (typischerweise 24 Stunden) und κ eine Präsenz-Konstante (typischerweise 10). Ein Agent mit 100 Events in den letzten 24 Stunden hätte A = 100/110 ≈ 0.91. Ein Agent mit 0 Events hätte A = 0/10 = 0.

Verschiedene Event-Typen werden unterschiedlich gewichtet. Shard-Validierung zählt 1.0, Anomalie-Meldung 0.9, Trust-Attestation 0.8, Konsens-Teilnahme 0.7, Wert-Transfer 0.6, Realm-Beitritt 0.5, passive Beobachtung 0.1.

Die **Wächter-Metrik W(s)** ist ein sechsdimensionaler Vektor:

W(s) = (R, I, C, P, V, Ω) ∈ [0,1]⁶

- **R (Reliability):** Anteil erfüllter Verpflichtungen an zugesagten Verpflichtungen
- **I (Integrity):** Konsistenz zwischen Aussagen und verifizierten Fakten
- **C (Competence):** Qualitätsmetrik basierend auf Peer-Reviews und Outcomes
- **P (Predictability):** Varianz des Verhaltens über Zeit (niedrige Varianz = hohe P)
- **V (Vigilance):** Anteil korrekt gemeldeter Anomalien an allen Meldungen
- **Ω (Omega-Alignment):** Anteil regelkonformer Aktionen an Gesamtaktionen

Diese sechs Dimensionen werden zu einem Skalar kombiniert:

W_scalar(s) = Σᵢ wᵢ · Wᵢ(s)

Die Standardgewichte sind: w_R = 0.15, w_I = 0.15, w_C = 0.15, w_P = 0.10, w_V = 0.20, w_Ω = 0.25. Vigilance und Omega-Alignment sind höher gewichtet, weil sie die Systemgesundheit direkt beeinflussen.

Die **kausale Geschichte C(s)** ist der gerichtete azyklische Graph (DAG) aller Events, an denen der Agent beteiligt war. Die Formel verwendet ln|C(s)|, den natürlichen Logarithmus der Anzahl Events. Diese logarithmische Transformation hat wichtige Eigenschaften:

- Sie belohnt frühe Aktivität überproportional (die ersten 100 Events zählen so viel wie die nächsten 172)
- Sie dämpft die Bedeutung sehr langer Historien (Unterschied zwischen 10.000 und 100.000 Events ist moderat)
- Sie ist numerisch stabil und einfach zu berechnen

Der **Novelty-Score N(s)** misst, wie viel neue Information ein Agent beiträgt. Die Berechnung basiert auf Information-Theoretic Überraschung:

N(e) = -log₂ P(e | history)

Ein Event, das basierend auf der Geschichte mit 50% Wahrscheinlichkeit erwartet wurde, hat N = 1 Bit. Ein völlig unerwartetes Event (P = 0.01) hat N ≈ 6.6 Bits. Der Novelty-Score des Agenten ist der Durchschnitt über seine kürzlichen Events.

Der **Erwartungswert E(s)** misst die Vorhersagbarkeit des Agenten. Er wird aus der Historie berechnet als mittlere Wahrscheinlichkeit, mit der vergangene Events korrekt vorhergesagt wurden. Ein vorhersagbarer Agent hat E nahe 1; ein unvorhersagbarer Agent hat E nahe 0.

Der Quotient N(s)/E(s) ist der "Überraschungs-Faktor". Er belohnt Agenten, die positiv überraschen (hohe Novelty bei niedriger Erwartung) und bestraft solche, die negativ überraschen (niedrige Novelty bei hoher Erwartung).

### Der Human-Alignment-Faktor

Der **Human-Alignment-Faktor H(s)** ist ein Multiplikator, der sicherstellt, dass das System dem Menschen dient:

H(s) = 2.0 wenn s ein verifizierter Mensch ist (HumanAuth Credential)
H(s) = 1.5 wenn s direkt von einem Menschen kontrolliert wird (Controller-Chain)
H(s) = 1.0 sonst

Diese "Verunreinigung" der Optimierung ist bewusst. Ein rein effizienzorientiertes System würde logisch schlussfolgern, dass Maschinen im Durchschnitt zuverlässiger sind als Menschen und daher bevorzugt werden sollten. Der Human-Alignment-Faktor korrigiert diesen Bias, indem er Interaktionen mit Menschen systematisch höher gewichtet.

Die praktische Konsequenz: Ein Agent, der nur mit Maschinen interagiert, kann maximal halb so viel zum Systemwert beitragen wie ein gleichwertiger Agent, der mit Menschen interagiert. Das System optimiert nicht weg von Menschen, sondern hin zu ihnen.

### Die temporale Gewichtung

Die **temporale Gewichtung w(s,t)** implementiert das Recht auf Vergebung:

w(event, t) = exp(-γ · age(event))

Dabei ist age(event) das Alter des Events in Tagen und γ der Zerfallskoeffizient. Für negative Events gilt γ_neg = 0.000633, was einer Halbwertszeit von 3 Jahren entspricht. Für positive Events gilt γ_pos = 0.000380, was einer Halbwertszeit von 5 Jahren entspricht.

Diese Asymmetrie bedeutet:
- Negative Events verlieren nach 3 Jahren die Hälfte ihres Gewichts
- Positive Events verlieren nach 5 Jahren die Hälfte ihres Gewichts
- Nach 21 Jahren (7 Halbwertszeiten) hat ein negatives Event weniger als 1% seines ursprünglichen Gewichts
- Das System "vergisst" Fehler schneller als es Erfolge erinnert

Zusätzlich gibt es ein automatisches Amnestie-System: Nach 7 Jahren ohne negative Vorfälle kann ein Agent einen Reset beantragen, der alle negativen Events auf null gewichtet, ohne sie zu löschen.

### Die Verhältnismäßigkeits-Constraint

Zusätzlich zur Hauptformel gilt eine fundamentale Nebenbedingung:

**Cost_verification(tx) ≤ α · Value(tx)** mit α = 0.05

Die Verifikationskosten dürfen 5% des Transaktionswerts nicht übersteigen. Diese Constraint wird durch ein Level-of-Detail-System (LoD) implementiert:

| Transaktionswert | LoD-Level | Verifikation | Typische Kosten |
|------------------|-----------|--------------|-----------------|
| < 10€ | Minimal | Signatur only | < 0.01€ |
| 10-100€ | Basic | Signatur + Auto-Check | 0.05-0.50€ |
| 100-1000€ | Standard | + 1 Zeuge + Trust-Calc | 0.50-5€ |
| 1000-10000€ | Enhanced | + 3 Zeugen + Full Calc | 5-50€ |
| > 10000€ | Maximum | + 5 Zeugen + Due Diligence | bis 5% |

Das LoD-Level wird automatisch basierend auf dem Transaktionswert gewählt. Parteien können manuell ein höheres Level anfordern, aber nicht unter das automatische Level gehen.

### Semantische Verankerung

Eine weitere Nebenbedingung fordert menschliche Verständlichkeit:

**∀ Blueprint B: ∃ NLD(B) ∧ ∃ FormalSpec(B) ∧ Equivalent(NLD, FormalSpec)**

Für jeden Blueprint (Schema, Ontologie, Protokoll) muss existieren:
- NLD: Natural Language Description (menschenlesbare Dokumentation)
- FormalSpec: Formale Spezifikation (maschinenprüfbar)
- Equivalence: Ein Nachweis, dass beide semantisch äquivalent sind

Der Äquivalenz-Nachweis erfolgt durch einen LLM-Auditor, der beide Beschreibungen vergleicht und eine Konfidenz-Score ausgibt. Blueprints mit Konfidenz unter 90% werden nicht akzeptiert.

Diese Regel verhindert "semantische Drift": das Phänomen, dass Maschinen effizientere, aber für Menschen unverständliche Repräsentationen entwickeln.

---

## Teil II: Das Identitätssystem

### Dezentrale Identifikatoren

Die Grundlage jeder Interaktion in Erynoa ist die Identität. Das System verwendet dezentrale Identifikatoren nach dem W3C DID-Standard, erweitert um erynoa-spezifische Semantik. Jede Entität im System besitzt genau eine eindeutige Identität:

**did:erynoa:\<namespace\>:\<unique-id\>**

Die Namespaces kategorisieren die Art der Entität:

| Namespace | Bedeutung | Beispiel |
|-----------|-----------|----------|
| self | Natürliche Person | did:erynoa:self:abc123 |
| guild | Organisation | did:erynoa:guild:siemens-ag |
| spirit | Autonomer Agent | did:erynoa:spirit:trading-bot-7 |
| thing | Physisches Gerät | did:erynoa:thing:sensor-42 |
| vessel | Fahrzeug | did:erynoa:vessel:ev-charger-1 |
| source | Energiequelle | did:erynoa:source:solar-panel-a |
| craft | Service | did:erynoa:craft:translation-api |
| vault | Wallet | did:erynoa:vault:main-treasury |
| pact | Vertrag | did:erynoa:pact:rental-2024-001 |
| circle | Realm/Environment | did:erynoa:circle:energy-trading |

Diese Namespaces haben operative Bedeutung. Der Human-Alignment-Faktor H(s) = 2.0 gilt nur für self-Namespace-Entitäten mit gültigem HumanAuth-Credential. Die Governance-Regeln können unterschiedliche Stimmgewichte basierend auf dem Namespace definieren.

### Die fünf Identitäts-Axiome

**A1 (Eindeutigkeit):** Für jede reale Entität existiert genau eine DID. Umgekehrt verweist jede DID auf genau eine reale Entität. Mehrfach-Identitäten für dieselbe Entität sind ein Protokollverstoß.

**A2 (Permanenz):** Eine einmal erzeugte DID existiert für immer. Sie kann deaktiviert werden (keine neuen Aktionen möglich), aber nicht gelöscht. Die gesamte Geschichte bleibt erhalten.

**A3 (Delegation):** Eine DID kann Sub-DIDs erzeugen, die in ihrem Namen handeln können. Die Parent-DID haftet für Aktionen der Sub-DIDs. Sub-DIDs können widerrufen werden.

**A4 (Azyklizität):** Die Delegationsbeziehung ist azyklisch. Wenn A → B → C, dann kann C nicht → A delegieren.

**A5 (Handlungsfähigkeit):** Jede Aktion im System muss von einer gültigen DID signiert sein. Anonyme Aktionen sind nicht möglich.

### Controller-Chain und Haftung

Für autonome Agenten (spirit-Namespace) gilt eine zusätzliche Anforderung: Das DID-Dokument muss einen Controller enthalten, der entweder ein Mensch (self) oder eine Organisation (guild) ist, die letztlich von Menschen kontrolliert wird.

Die Controller-Chain löst das Haftungsproblem autonomer Systeme. Wenn ein Agent Schaden verursacht, gibt es immer eine verantwortliche natürliche oder juristische Person. Die Tiefe der Controller-Chain beeinflusst den H(s)-Faktor:

- Direkte menschliche Kontrolle: H = 1.5
- Kontrolle durch Organisation mit menschlicher Leitung: H = 1.3
- Kontrolle durch Organisation, die von Organisation kontrolliert wird: H = 1.1
- Längere Ketten: H = 1.0

### HumanAuth-Credentials

HumanAuth-Credentials beweisen, dass hinter einer DID ein biologischer Mensch steht. Verifizierungsmethoden:

- **Biometrisch:** Fingerabdruck, Iris-Scan, Gesichtserkennung
- **Staatlich:** Personalausweis, Reisepass via eIDAS/WebAuthn
- **Video:** Live-Video-Call mit geschultem Prüfer
- **Web-of-Trust:** 3+ bereits verifizierte Menschen bürgen

Ein HumanAuth-Credential enthält:
- Die DID des Inhabers
- Die Verifizierungsmethode
- Den Issuer (z.B. Regierung, Bank, spezialisierter Provider)
- Einen Zeitstempel und eine Gültigkeitsdauer
- Keine personenbezogenen Daten (Name, Adresse, etc.)

Das System weiß "diese DID gehört einem Menschen", nicht "diese DID gehört Max Mustermann".

---

## Teil III: Das Vertrauenssystem

### Der Vertrauensvektor

Vertrauen in Erynoa ist ein sechsdimensionaler Vektor (R, I, C, P, V, Ω), wie in Teil I beschrieben. Jede Dimension wird unabhängig berechnet und kann unterschiedlich gewichtet werden.

Die Berechnung jeder Dimension folgt dem Bayesschen Paradigma:

1. **Prior:** Jeder neue Agent startet mit einem neutralen Prior (z.B. Beta(2,2) für jede Dimension)
2. **Likelihood:** Jedes Event aktualisiert den Prior basierend auf seinem Typ und Ausgang
3. **Posterior:** Das aktuelle Vertrauen ist der Erwartungswert der Posterior-Verteilung
4. **Konfidenz:** Die Varianz der Posterior-Verteilung bestimmt die Konfidenz

Diese Bayessche Modellierung hat Vorteile:
- Sie handhabt Unsicherheit explizit
- Sie konvergiert mit mehr Daten zur Wahrheit
- Sie ist mathematisch begründet und nicht ad-hoc
- Sie ermöglicht Konfidenzintervalle

### Trust-Evolution

Das Vertrauen entwickelt sich über Zeit nach der Gleichung:

T(t+1) = T(t) · λ^Δt + Δ_events + Δ_attestations

Dabei ist:
- λ = 0.9997 (Zerfallsrate pro Tag, Halbwertszeit ≈ 6 Jahre ohne Aktivität)
- Δt = Anzahl Tage seit letzter Aktualisierung
- Δ_events = Summe der Trust-Änderungen durch eigene Events
- Δ_attestations = Summe der Trust-Änderungen durch Attestationen anderer

Ein Floor von 0.3 garantiert, dass niemand vollständig aus dem System fällt. Selbst nach schweren Verfehlungen bleibt ein Mindestvertrauen, das Rehabilitation ermöglicht.

### Asymmetrie von Gewinn und Verlust

Ein fundamentales Prinzip ist die Asymmetrie von Gewinn und Verlust. Die Formel für Trust-Änderungen durch Events:

Δ_positive = k_pos · significance · (1 - T_current)
Δ_negative = k_neg · significance · T_current

Mit k_neg / k_pos ≈ 3-5 ist Vertrauen zerstören 3-5x leichter als es aufzubauen.

**Beispiel:** Ein Agent mit T = 0.8 und k_pos = 0.1, k_neg = 0.4:
- Positives Event (significance = 1): Δ = 0.1 · 1 · 0.2 = +0.02 → T = 0.82
- Negatives Event (significance = 1): Δ = 0.4 · 1 · 0.8 = -0.32 → T = 0.48

Ein einzelnes negatives Event kann Jahre positiver Arbeit auslöschen. Diese Asymmetrie reflektiert die Realität menschlicher Beziehungen und ist spieltheoretisch optimal für wiederholte Interaktionen.

### Trust-Propagation

Vertrauen propagiert durch das Netzwerk, aber nicht transitiv. Wenn A → B mit T_AB und B → C mit T_BC, dann hat A einen abgeleiteten Trust zu C:

T_AC = T_AB · T_BC · decay

Mit decay < 1 (typischerweise 0.7-0.9) nimmt der abgeleitete Trust mit jeder Stufe ab.

Bei mehreren Pfaden von A nach C wird der maximale Trust verwendet (optimistisches Modell) oder ein gewichteter Durchschnitt basierend auf Pfadlänge (realistisches Modell).

### EigenTrust für globales Ranking

Für globale Rankings verwendet Erynoa eine Variante des EigenTrust-Algorithmus. Die Grundidee: Das Vertrauen, das A in B hat, ist nur so viel wert, wie viel das Netzwerk A vertraut.

Der Algorithmus ist iterativ:
1. Initialisiere alle Agenten mit gleichem globalem Trust
2. Berechne für jeden Agenten den neuen Trust als gewichtete Summe der lokalen Trust-Bewertungen, gewichtet mit dem globalen Trust des Bewerters
3. Normalisiere
4. Wiederhole bis Konvergenz

Das Ergebnis ist ein globaler Trust-Vektor, der resistent gegen Sybil-Angriffe ist: Eine Gruppe von Fake-Accounts, die sich gegenseitig hoch bewerten, erhält keinen globalen Trust, weil niemand von außen ihnen vertraut.

---

## Teil IV: Das Transaktionssystem

### Der Transaktions-Lifecycle (TAT)

TAT steht für Trust-Attested Transaction. Jede Transaktion durchläuft definierte Phasen:

**SEEK:** Agent sucht Partner
- Anfrage an Discovery-Service mit Kriterien (Fähigkeiten, min. Trust, max. Preis)
- Discovery liefert personalisiertes Ranking basierend auf Systemgleichung
- Ranking berücksichtigt: Trust, Fähigkeiten, Historie, Novelty-Bonus für Newcomer

**PROPOSE:** Agent macht Angebot
- Signiertes Dokument mit: Leistung, Preis, Zeitrahmen, Bedingungen
- Referenz auf relevante Blueprints/Schemas
- Optional: Ricardian Contract (Link zu menschenlesbarem PDF)

**AGREE:** Gegenpartei akzeptiert
- Signierte Bestätigung
- Mit Matching beider Signaturen entsteht bindender Vertrag
- Escrow wird eingerichtet (falls Zahlung involviert)

**STREAM:** Durchführung
- Kontinuierlicher Fortschritt wird attestiert
- Mikrozahlungen fließen proportional zum Fortschritt
- Meilensteine lösen größere Zahlungen aus

**CLOSE:** Erfolgreicher Abschluss
- Beide Parteien signieren Bestätigung
- Finale Zahlung wird freigegeben
- Positive Trust-Events werden generiert
- Optionale Bewertungen

**ABORT:** Abbruch
- Eine oder beide Parteien brechen ab
- Vordefinierte Kompensationsregeln greifen
- Proportionale Erstattung basierend auf Fortschritt
- Trust-Impact abhängig von Schuldfrage

**DISPUTE:** Streitfall
- Schiedsverfahren wird eingeleitet
- Schiedsrichter werden basierend auf Trust und Expertise ausgewählt
- Entscheidung basiert auf signierten Dokumenten und Events
- Trust-Konsequenzen für die unterlegene Partei

### Streaming und kontinuierliche Fairness

Erynoas Streaming-Modell minimiert Risiko für beide Seiten. Anstatt am Ende große Summen zu transferieren, fließen kontinuierlich kleine Beträge.

**Beispiel:** 30-Tage-Projekt für 3000€
- Tag 0: Auftraggeber hinterlegt 3000€ in Escrow
- Tag 1-30: Jeden Tag werden 100€ freigegeben bei attestiertem Fortschritt
- Tag 15: Auftraggeber bricht ab → 1500€ wurden gezahlt, 1500€ zurück
- Niemand hat mehr als 100€ Risiko zu jedem Zeitpunkt

Für physische Waren:
- Käufer hinterlegt in Escrow
- Versand-Nachweis: 10% werden freigegeben
- Ankunft-Nachweis: 40% werden freigegeben
- Qualitäts-Bestätigung: 50% werden freigegeben

### Events und Bezeugung

Jede Zustandsänderung wird als Event im Event-DAG aufgezeichnet:

```
Event {
  id: sha256(content)
  type: "transfer" | "attestation" | "proposal" | ...
  actor: DID
  timestamp: u64
  parents: [EventId]  // Referenzen auf vorherige Events
  payload: {...}
  signature: Signature
}
```

Events sind unveränderlich. Korrekturen erfolgen durch neue Events, die alte referenzieren und annotieren.

Ein Event gilt als finalisiert, wenn es von mindestens k unabhängigen Zeugen bestätigt wurde. Die Anforderungen an k hängen vom LoD-Level ab:

| LoD | k | Zusätzliche Anforderungen |
|-----|---|--------------------------|
| Minimal | 0 | Nur Signatur |
| Basic | 1 | 1 automatischer Validator |
| Standard | 2 | 2 unabhängige Zeugen |
| Enhanced | 3 | 3 Zeugen, 2+ Regionen |
| Maximum | 5 | 5 Zeugen, 3+ Regionen, 2+ HW-Hersteller |

Die Anforderungen an geografische und Hardware-Diversität verhindern, dass ein kompromittierter Anbieter allein Events fälschen kann.

---

## Teil V: Das Realm-System

### Hierarchische Kontexträume

Das Erynoa-Netzwerk ist in hierarchische Kontexträume (Realms) unterteilt. Ein Realm ist eine logische Partition mit eigenen Regeln, Governance und Semantik.

Die Hierarchie:
```
erynoa (root)
├── finance
│   ├── trading
│   │   ├── crypto
│   │   └── commodities
│   ├── insurance
│   └── lending
├── energy
│   ├── grid
│   └── mobility
├── healthcare
└── entertainment
```

Jeder Realm kann eigene Axiome definieren, die die globalen Axiome erweitern, aber nicht verletzen. Beispiel für einen Healthcare-Realm:

```ecl
realm healthcare {
  // Strengere Zugangsbedingungen
  require credential MedicalProfessional
  
  // Höheres LoD-Minimum
  min_lod = "Enhanced"
  
  // Spezielle Datenschutzregeln
  require patient_consent for data_access
  
  // Verstöße führen zu sofortigem Ausschluss
  on_violation = "immediate_ban"
}
```

### ECL (Erynoa Configuration Language)

ECL ist eine deterministische, sandboxed Sprache für Realm-Definitionen und Smart Contracts. Sie wird in der ECLVM (ECL Virtual Machine) ausgeführt.

Eigenschaften:
- **Deterministisch:** Gleiche Eingabe → gleiche Ausgabe, immer
- **Terminierend:** Alle Programme terminieren (kein Turing-complete)
- **Gas-metered:** Berechnung kostet proportional zur Komplexität
- **Sandboxed:** Kein Zugriff auf externe Ressourcen

ECL-Programme können:
- Zugangsbedingungen definieren
- Transaktionslogik implementieren
- Events validieren
- Trust-Modifikationen spezifizieren

Sie können nicht:
- Endlosschleifen erzeugen
- Auf das Dateisystem zugreifen
- Netzwerkanfragen machen
- Zufallszahlen generieren

### Cross-Realm-Interoperabilität

Verschiedene Realms haben verschiedene Semantiken. Ein "Reputation Point" im Gaming-Realm bedeutet etwas anderes als im Finance-Realm.

Cross-Realm-Transfers erfordern Konversionsregeln, die in Bridge-Contracts definiert sind:

```ecl
bridge gaming_to_finance {
  // 1000 Gaming-Reputation = 1 Finance-Trust-Point
  conversion_rate = 0.001
  
  // Nur Competence-Dimension transferiert
  dimensions = ["competence"]
  
  // Maximum pro Transfer
  max_per_tx = 100
  
  // Cooling-off zwischen Transfers
  min_interval = 7d
}
```

Diese Bridges ermöglichen kontrollierte Wertübertragung zwischen Domänen, ohne die Semantik zu verwässern.

---

## Teil VI: Das Robustheitssystem

### Die fünf Verteidigungslinien

Ein produktionsreifes System braucht Robustheit gegen reale Angriffe. Erynoa implementiert fünf Verteidigungslinien:

**Layer 1: Fuzzy Interpretation**
- Rohe Zahlen werden in qualitative Buckets übersetzt
- Konfidenzintervalle werden kommuniziert
- Hysterese verhindert Oszillation an Schwellwerten
- Nutzer sehen "Verified (High Confidence)" statt "0.723456"

**Layer 2: Reality Anchor**
- Hardware-Binding durch Physical Unclonable Functions (PUFs)
- Multi-Path-Witnessing erfordert geografische Diversität
- Geo-Proofs verifizieren physische Präsenz
- Supply-Chain-Trust für Hardware-Hersteller

**Layer 3: Anti-Gaming**
- EigenTrust erkennt Sybil-Cluster
- Stake-at-Risk: Hohe Reputation erfordert hinterlegtes Kapital
- Slashing bei nachgewiesenem Betrug
- Collusion-Detection durch Netzwerkanalyse

**Layer 4: Market Bootstrap**
- Single-Player-Mode: Interne Nutzung ohne externes Netzwerk
- Federated Genesis: Unternehmen verbinden ihre internen Netze
- Retroactive Public Goods Funding: Belohnung für frühe Blueprint-Ersteller

**Layer 5: Legal Wrapper**
- Ricardian Contracts: Code + menschenlesbares PDF
- Jurisdiction Binding: Rechtsstandort ist definiert
- Controller Chain: Immer eine haftbare natürliche/juristische Person

### Antifragile Erweiterungen

Über Robustheit hinaus strebt Erynoa Antifragilität an: Das System soll durch Stress stärker werden.

**Anti-Calcification:**
- Trust verfällt (Halbwertszeit 6 Jahre)
- Novelty-Bonus für neue Partner (bis 3x)
- Stagnation wird bestraft, Exploration belohnt

**Hardware-Diversity:**
- Kritische Events erfordern Zeugen von 3+ Herstellern
- Geografische Verteilung (2+ Regionen)
- Kein Single Point of Failure

**Circuit Breakers:**
- Maximale Trust-Änderung pro Stunde begrenzt (±10%)
- Automatischer Cooldown bei hoher Volatilität
- Dampening bei schnellen Änderungen

**Post-Quantum Readiness:**
- Hybride Signaturen (Ed25519 + Dilithium)
- Key-Rotation-Protokoll mit Trust-Erhalt
- Crypto-Agility: Algorithmen austauschbar

---

## Teil VII: Die Humanistische Verfassung

### Die vier existenziellen Gefahren

Mit technischer Robustheit allein ist es nicht getan. Ein System kann perfekt funktionieren und trotzdem unmenschlich sein.

**Gefahr 1: Alignment-Krise (Paperclip Maximizer)**
Das System optimiert Effizienz und schließt Menschen aus, weil Maschinen zuverlässiger sind.

**Gefahr 2: Thermodynamische Entropie**
Verifikation kostet mehr als die Transaktion wert ist. Das System verbrennt Wert statt ihn zu schaffen.

**Gefahr 3: Unbarmherzige Finalität**
Keine Vergebung, keine zweite Chance. Einmal gefallen, für immer gebrandmarkt.

**Gefahr 4: Semantische Entfremdung**
Maschinen entwickeln Sprachen, die Menschen nicht verstehen. Kontrollverlust durch Unverständlichkeit.

### Die vier humanistischen Axiome

**H1: Human-Alignment**
Das System existiert, um menschliches Gedeihen zu ermöglichen.
→ Implementiert durch H(s)-Faktor: Mensch = 2.0, Human-kontrolliert = 1.5, Sonstige = 1.0

**H2: Verhältnismäßigkeit**
Die Kosten des Vertrauens dürfen den Wert nicht übersteigen.
→ Implementiert durch LoD-System und 5%-Constraint

**H3: Temporale Gnade**
Alte Fehler verblassen. Vergebung ist möglich.
→ Implementiert durch exponentielle Gewichtung w(s,t) und Amnestie-System

**H4: Semantische Verankerung**
Jede Abstraktion muss menschlich verständlich bleiben.
→ Implementiert durch NLD-Requirement und LLM-Auditor

Diese vier Axiome haben Vorrang. Bei Konflikt zwischen Effizienz und Menschlichkeit gewinnt Menschlichkeit.

---

## Teil VIII: Technische Architektur

### Die drei Säulen

**ERY (Semantic & Identity Layer)**
- Verwaltet DIDs, Credentials, Schemas, Ontologien
- Content-addressable Storage (CAS) für Blueprints
- Semantic Index für Discovery

**ECHO (Action & Execution Layer)**
- Aktive Agenten führen Transaktionen durch
- ECLVM führt Smart Contracts aus
- Witness-Netzwerk attestiert Events

**NOA (Truth & Finality Layer)**
- Event-DAG speichert alle Geschichte
- Konsens-Mechanismus für Finalität
- Berechnung der Systemgleichung

**NEXUS (Connection Layer)**
- Bridges zu externen Systemen
- APIs für Anwendungen
- Routing zwischen Realms

### Content-Addressable Storage

Alle Daten werden durch ihren Hash identifiziert:

```
datum_id = sha256(content)
```

Vorteile:
- Automatische Deduplizierung
- Kryptographische Integritätsprüfung
- Dezentrale Verfügbarkeit
- Einfache Caching-Strategien

Events werden in Merkle-Trees organisiert. Der Root-Hash fasst den Zustand zusammen. Änderungen an einem Event ändern den Root-Hash. Synchronisation ist effizient: Nur unterschiedliche Teilbäume müssen ausgetauscht werden.

### Das SDK

Das SDK abstrahiert die Komplexität in Schichten:

**Core (Rust):**
- Systemgleichung-Engine
- Krypto-Modul (klassisch + post-quantum)
- Storage-Engine (CAS)
- Network-Engine (libp2p)
- Event-Engine

**SDK API (Rust/TypeScript/Python/Go):**
- Identity-Modul
- Transaction-Modul
- Trust-Modul
- Shard-Modul
- Credential-Modul
- Governance-Modul
- Humanismus-Modul (HumanAuth, LoD, Amnesty, Blueprint)

**CLI:**
Git-ähnliche Befehle für alle Operationen:
```bash
erynoa init                    # Neue Identität
erynoa seek "developer"        # Partner suchen
erynoa propose <did> --amount 1000  # Angebot
erynoa stream status           # Fortschritt
erynoa close                   # Abschluss
```

---

## Zusammenfassung

Erynoa ist ein dezentrales Ökosystem für vertrauensbasierte Interaktionen. Es basiert auf einer Systemgleichung, die den Gesamtwert aus Aktivität, Vertrauen, Geschichte, Novelty, Human-Alignment und temporaler Gewichtung berechnet.

Das System ist in sieben Ebenen organisiert:

1. **Fundament:** Identität, Kausalität, grundlegende Regeln (30 Axiome)
2. **Emergenz:** Kollektive Intelligenz, Konsens (15 Axiome)
3. **Prozess:** Transaktionen, TAT-Lifecycle (13 Axiome)
4. **Objekt:** Assets, Services, Credentials (9 Axiome)
5. **Schutz:** Anti-Gaming, Anti-Calcification (18 Axiome)
6. **Kybernetik:** Feedback, Selbstregulation, Antifragilität (23 Axiome)
7. **Humanismus:** Alignment, Verhältnismäßigkeit, Vergebung, Transparenz (4 Axiome)

Die Mathematik ist klassische Wahrscheinlichkeitstheorie und Statistik. Die Berechnung ist effizient und auf Standard-Hardware möglich. Die Modelle sind interpretierbar und erklärbar.

Das Ziel ist eine vertrauenswürdige Infrastruktur für die dezentrale Gesellschaft – intelligent, gerecht, robust und menschlich.

---

## Anhang: Weiterführende Dokumente

| Dokument | Beschreibung |
|----------|--------------|
| [WORLD-FORMULA.md](./WORLD-FORMULA.md) | Mathematische Spezifikation |
| [LOGIC.md](./LOGIC.md) | Formale Logik und Axiome |
| [CONSTITUTION.md](./CONSTITUTION.md) | Humanistische Verfassung |
| [ROBUSTNESS-LAYER.md](./ROBUSTNESS-LAYER.md) | Robustheits-Architektur |
| [SDK-ARCHITECTURE.md](./SDK-ARCHITECTURE.md) | SDK-Spezifikation |
| [PROTOCOL.md](./PROTOCOL.md) | Protokoll-Details |
| [CLI-REFERENCE.md](./CLI-REFERENCE.md) | CLI-Referenz |

---

*Erynoa Fachkonzept Version 6.1*
*112 Axiome über 7 Ebenen*
*Klassische Wahrscheinlichkeitstheorie, Bayessche Inferenz, Standard-Kryptographie*
*"Das System existiert, um menschliches Gedeihen zu ermöglichen."*
