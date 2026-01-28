# Erynoa – Cybernetic Loop (Universeller Workflow)

> **Zielgruppe:** Architekt:innen, Protocol/Backend-Engineers, Product Owner
> **Kontext:** End-to-End-Prozesse des Protokolls modellieren und verstehen
> **Verwandte Dokumente:** [Agents & ADL](./agents-and-adl.md), [Trust & Reputation](./trust-and-reputation.md), [Glossar](./glossary.md)

---

## 1. Ziel dieses Dokuments

Dieses Dokument beschreibt den **Cybernetic Loop** von Erynoa – den universellen
Prozess, mit dem ein subjektiver Intent zu einer objektiv finalisierten Transaktion
auf dem Ledger wird und wie das System daraus lernt.

Fokus:

- Phasen des Workflows (von Intent bis Feedback)
- Rollen von ERY, ECHO und NOA in jeder Phase
- Typische Inputs/Outputs pro Schritt

---

## 2. Überblick: Vom Intent zur Feedback-Schleife

Der Cybernetic Loop besteht aus sechs Phasen:

1. **Sensing & Intent (ECHO)**
2. **Discovery & Context (ECHO ↔ ERY)**
3. **Validation & Trust-Gating (ERY)**
4. **Negotiation & Progressive Disclosure (ECHO)**
5. **Execution & Logic Guards (NOA)**
6. **Feedback & Ripple Effect (NOA → ERY)**

Gedankliches Sequenzdiagramm:

- Nutzer / Maschine → **Seeker-Agent (ECHO)**
  → nutzt **Semantic Index & Karmic Engine (ERY)**
  → verhandelt mit **Provider-Agent (ECHO)**
  → finalisiert über **MoveVM & AMOs (NOA)**
  → Events zurück an **Karmic Engine (ERY)**
  → aktualisierte Trust-Daten beeinflussen zukünftige Intents und Entscheidungen.

---

## 3. Phase 1 – Sensing & Intent (Initialisierung, ECHO)

**Ziel:** Ein Intent beschreibt, _was_ erreicht werden soll, ohne die exakte Gegenpartei
oder alle technischen Details zu kennen.

**Akteure:**

- Nutzer, Unternehmen oder Maschine
- Seeker-Agent (in der ECHO-Sphäre)

**Technische Mittel:**

- **Agent Definition Language (ADL)**

**Typische Inhalte einer ADL-Intent-Definition:**

- Funktionale Anforderungen:
  - z. B. „Ladeleistung > 50 kW“, „Energiequelle erneuerbar“, „Latenz < X ms“.
- Normative Anforderungen:
  - z. B. Konformität zu bestimmten Norm-Blueprints (ISO, eCl@ss, regulatorische Standards).
- Vertrauensanforderungen:
  - Mindest-Trust (MinTrust) pro Dimension oder aggregiert.
- Geografische / kontextuelle Anforderungen:
  - Region über **Geohashing** statt exakter Koordinaten.
- Ökonomische Parameter:
  - Preisspannen, Laufzeiten, Vertragskonditionen.

**Ergebnis der Phase:**

- Ein vollständig spezifizierter Intent in ADL, der als Eingabe für die Discovery-Phase dient.

---

## 4. Phase 2 – Discovery & Context (ECHO ↔ ERY)

**Ziel:** Passende Provider und Objekte finden, die den Intent potenziell erfüllen können.

**Akteure:**

- Seeker-Agent (ECHO)
- ERY Semantic Index

**Ablauf:**

1. Der Seeker-Agent übergibt die Intent-Parameter an den **Semantic Index**.
2. Der Semantic Index führt:
   - Vektor-Suchen (semantische Ähnlichkeit),
   - Filter auf Domain Blueprints und Normative Standards,
   - geografische Filter via DHT + Geohashing
     durch.
3. Potenzielle Provider-DIDs und relevante AMOs werden identifiziert.

**Wichtige Eigenschaften:**

- Suche basiert nicht nur auf Keywords, sondern auf **semantischer Kompatibilität**:
  - Blueprint-Referenzen, Domänenkontexte und Normen werden berücksichtigt.
- Skalierbarkeit durch:
  - horizontales Sharding (DHT),
  - geographische Partitionierung.

**Ergebnis der Phase:**

- Eine **Kandidatenliste** von Providern und AMOs, die technisch und kontextuell passen könnten.

---

## 5. Phase 3 – Validation & Trust-Gating (ERY)

**Ziel:** Kandidaten auf strukturellen Trust und Reputation prüfen, bevor Ressourcen in Verhandlungen fließen.

**Akteure:**

- ERY-Node (Verifiable Oracle)
- Karmic Engine

**Prüfschritte:**

1. **Struktureller Trust**
   - Verifizierung des **DNS-Bootstrap**:
     - Bindung von DIDs an Domains (z. B. via DNS-TXT).
   - Überprüfung von **Attestations**:
     - Zertifikate, regulatorische Nachweise, externe Signaturen.

2. **Reputationsprüfung**
   - Abfrage der **Trust Vectors** für Kandidaten.
   - Vergleich mit den in ADL definierten MinTrust-Schwellen.

**Effekt von Trust-Gating:**

- Kandidaten, die:
  - strukturelle Anforderungen nicht erfüllen,
  - oder unterhalb des geforderten Vertrauen-Levels liegen,
    werden verworfen.

**Ergebnis der Phase:**

- Eine **bereinigte Liste** von Providern, die:
  - technisch passen,
  - und vertrauenswürdig genug sind, um in eine Verhandlung einzutreten.

---

## 6. Phase 4 – Negotiation & Progressive Disclosure (ECHO)

**Ziel:** Einen privatwirtschaftlichen Konsens über konkrete Vertragsbedingungen finden.

**Akteure:**

- Seeker-Agent und Provider-Agent (beide in ECHO)

**Kommunikationskanal:**

- **XMTP Secure Tunnels** (verschlüsselte Off-Chain-Kommunikation)
- Ein solcher Tunnel bildet eine **Consensus Bubble**

**Prinzip der Progressive Disclosure:**

- Sensible Daten (z. B. interne Kostenstrukturen, genaue Standorte, proprietäre Parameter):
  - werden erst offengelegt, wenn:
    - die Basis-Kompatibilität geklärt ist,
    - beidseitiges Interesse und Mindestvertrauen bestehen.
- Schrittweiser Informationsaustausch:
  - reduziert Informationslecks,
  - schützt Geschäftsgeheimnisse,
  - und minimiert unnötigen Datenverkehr.

**Verhandlungsergebnis:**

- Konkreter Vertrag mit:
  - referenziertem Blueprint und AMO-Typ,
  - Preisen, Mengen, Zeiträumen,
  - Service-Levels und Sanktionen,
  - Identitäten der beteiligten Parteien (DIDs / AMOs).

**Ergebnis der Phase:**

- Ein **transaktionales Paket**, das in die Exekution überführt werden kann.

---

## 7. Phase 5 – Execution & Logic Guards (NOA)

**Ziel:** Den Off-Chain-Konsens als On-Chain-Faktum in NOA finalisieren.

**Akteure:**

- MoveVM in NOA
- Logic Guards
- AMOs der beteiligten Parteien

**Ablauf:**

1. Das transaktionale Paket wird in eine **Move-Transaktion** übersetzt.
2. Vor Ausführung greifen **Logic Guards**:
   - Prüfen, ob:
     - alle Domain-spezifischen Regeln eingehalten sind,
     - Soulbound-Eigenschaften respektiert werden,
     - Ressourcenverbräuche konsistent sind (Resource Safety).
3. Bei erfolgreicher Prüfung:
   - wird der Zustand der betroffenen AMOs aktualisiert (z. B. Eigentumswechsel, Credential-Ausstellung, Start eines Service-Streams).
4. **Starfish BFT** finalisiert die Transaktion:
   - deterministische Finalität,
   - unter zwei Sekunden,
   - ohne zentrale Block-Produzenten (leaderless).

**Ergebnis der Phase:**

- Eine **unumkehrbare, kausal verankerte Transaktion** im NOA-Ledger.

---

## 8. Phase 6 – Feedback & Ripple Effect (NOA → ERY)

**Ziel:** Aus jeder Interaktion lernen und das Vertrauensgefüge aktualisieren.

**Akteure:**

- Event Ingestor (ERY)
- Karmic Engine (ERY)

**Ablauf:**

1. NOA emittiert nach jeder finalisierten Transaktion ein oder mehrere **Events**.
2. Der **Event Ingestor** in ERY nimmt diese Events auf und klassifiziert sie:
   - erfolgreich, neutral, fehlerhaft, betrügerisch, SLA-verletzend, usw.
3. Die Karmic Engine berechnet den neuen Trust Vector:

   \[
   R*\text{new}(t) = R*\text{old}(t-1) + \eta \left(F\_\text{Event} - E[F]\right)
   \]

4. **Trust Inheritance**:
   - Die Auswirkungen propagieren fraktal:
     - vom unmittelbaren AMO zu:
       - Betreiber,
       - Hersteller,
       - Zertifizierern,
       - und ggf. betroffenen Blueprints.

**Ergebnis der Phase:**

- Aktualisierte **Trust Vectors** und Kontextdaten im Semantic Index.
- Die nächste Discovery-Phase baut automatisch auf einem veränderten Vertrauensgefüge auf.

---

## 9. Zusammenfassung: Eigenschaften des Cybernetic Loop

Der Cybernetic Loop von Erynoa zeichnet sich aus durch:

- **Funktionstrennung:**
  - ECHO: Intents, Agenten, Verhandlung
  - ERY: Semantik, Kontext, Vertrauen
  - NOA: Wahrheit, Finalität, Exekution

- **Kontinuierliches Lernen:**
  - Jede Transaktion verändert die Vertrauenslandschaft.
  - Entscheidungen werden dynamisch besser, ohne zentrale Koordinatoren.

- **Privacy-by-Design:**
  - Nur minimale, notwendige Fakten landen On-Chain.
  - Details bleiben in Consensus Bubbles und Fluid Extensions.

- **Skalierbarkeit:**
  - Rechenintensive Prozesse Off-Chain
  - Formale Sicherheit und Kausalität On-Chain

---

## 10. Fazit

Der Cybernetic Loop fungiert als „Herzschlag“ von Erynoa: Er wandelt Intents in verlässliche Fakten um – und nutzt jede Interaktion, um das System langfristig robuster und intelligenter zu machen.

---

**Weiterführende Dokumente:**

- [Agents & ADL](./agents-and-adl.md) – Agentenmodell und Agent Definition Language
- [Trust & Reputation](./trust-and-reputation.md) – Details zum Vertrauensmodell
- [Use Cases](./use-cases.md) – Der Cybernetic Loop in der Praxis
