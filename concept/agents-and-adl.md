## Erynoa – Agents & Agent Definition Language (ADL)

**Zielgruppe:** Agenten-Entwickler:innen, Protocol Engineers, Architekt:innen für Agenten-/Policy-Layer.

### 1. Ziel dieses Dokuments

Dieses Dokument beschreibt:

- das Agentenmodell von Erynoa (**ECHO-Sphäre**) und
- die Grundprinzipien der **Agent Definition Language (ADL)**,

über die Intents, Constraints und Policies für autonome Interaktionen ausgedrückt werden.

---

### 2. Agentenmodell in ECHO

ECHO ist die Sphäre der **operativen Intelligenz**. Hier werden Intents
von Menschen, Unternehmen und Maschinen durch Agenten ausgeführt.

**Grundannahmen:**

- Agenten sind:
  - **zustandsloser Code („Agent as Code“)**
  - der **Just-in-Time** instanziiert wird,
  - und in einer **WASM-Sandbox** läuft.

- Persistenter Zustand liegt nicht im Agenten selbst, sondern:
  - in **ERY** (Semantik, Trust, Kontext),
  - und in **NOA** (AMOs, finale Zustände).

---

### 3. Agententypen

Erynoa unterscheidet zwei primäre Agentenrollen:

#### 3.1 Seeker Agents

**Rolle:**

- Repräsentieren die **Nachfrageseite**:
  - Nutzer,
  - Unternehmen,
  - IoT-Geräte oder andere Protokolle.

**Aufgaben:**

- Intent-Definition via ADL.
- Discovery und Auswahl passender Provider (in Zusammenarbeit mit ERY).
- Führen Off-Chain-Verhandlungen in Consensus Bubbles.
- Übermitteln finalisierte Vertragsparameter an NOA.

#### 3.2 Provider Agents

**Rolle:**

- Repräsentieren die **Angebotsseite**:
  - Betreiber von Infrastrukturen,
  - Dienstleister,
  - Hersteller,
  - Plattformen.

**Aufgaben:**

- Empfangen und interpretieren Intents.
- Entscheiden über die Teilnahme an Verhandlungen (basierend auf Policies).
- Verhandeln Preise, Konditionen, Service-Levels.
- Binden konkrete AMOs und Services an Vertragszusagen.

---

### 4. Ausführungsumgebung – WASM-Sandbox

Agenten laufen in einer strikt isolierten **WebAssembly (WASM) Sandbox**.

**Eigenschaften:**

- Sprachagnostisch:
  - Agenten können in Sprachen geschrieben werden, die nach WASM kompilieren.
- Sicherheit:
  - Kein direkter Zugriff auf das Host-System.
  - Zugriff nur über explizit definierte Host-APIs.

**Exemplarische Host-APIs:**

- Lesen aus ERY:
  - Query des Semantic Index (z. B. Kandidatensuche, Trust-Abfragen).
- Interaktion mit Netzwerk:
  - Aufbauen von P2P-Verbindungen (libp2p).
  - Eröffnung von XMTP-Tunneln.
- Interaktion mit NOA:
  - Konstruktion von Transaktionen (z. B. Nutzung von AMOs).

Diese Trennung erlaubt:

- schnelle Iteration im Agentencode,
- bei gleichzeitig klar definiertem, sicheren Interaktionsrahmen.

---

### 5. Agent Definition Language (ADL) – Grundprinzipien

Die **Agent Definition Language (ADL)** ist eine deklarative Sprache, mit der
Intents und Policies beschrieben werden.

**Zentrale Designziele:**

- **Deklarativ statt imperativ**:
  - Der Agent beschreibt _was_ erreicht werden soll, nicht _wie_ genau.
- **Domänenspezifisch erweiterbar**:
  - Neue Domänen können eigene Constraints einführen, ohne das Kernprotokoll zu ändern.
- **Maschinenlesbar & formalisierbar**:
  - ADL-Spezifikationen können automatisiert ausgewertet, validiert und in Agentenlogik übersetzt werden.

---

### 6. Kernbausteine von ADL

Ein ADL-Dokument (oder -Objekt) besteht typischerweise aus folgenden Bausteinen:

1. **Identity & Context**
   - Referenzen auf:
     - DID des Auftraggebers,
     - Domain oder Organisation,
     - ggf. referenzierte Credentials.

2. **Objective**
   - Beschreibung des Ziels:
     - z. B. „Lade Fahrzeug X mit mindestens 50 kW in Region Y aus erneuerbaren Quellen.“
   - Verknüpfung mit Domain Blueprints:
     - z. B. Blueprint „EV-Charging-Session“.

3. **Functional Constraints**
   - Mindest- oder Zielparameter:
     - Leistung, Kapazität, Latenz, Verfügbarkeit, etc.

4. **Normative Constraints**
   - Erforderliche Norm- bzw. Blueprint-Konformität:
     - z. B. „Muss Standard Z erfüllen“, „Nur zertifizierte Betreiber mit Credential-Typ X“.

5. **Trust Constraints**
   - MinTrust-Schwellen:
     - global oder pro Dimension,
     - z. B. „Reputation >= 0.9 in Zuverlässigkeit, >= 0.8 in Compliance“.
   - Mindestanforderungen an Attestations:
     - DNS-Verknüpfung, bestimmte Zertifikate.

6. **Geospatial Constraints**
   - Geohashing-basierte Regionen:
     - z. B. „Nur Anbieter in Region GH123*“, ohne exakte Koordinaten offenzulegen.

7. **Economic Constraints**
   - Preisspannen, Vergütungsmodelle:
     - fixe Preise, dynamische Preise, Value Streaming-Modelle.

8. **Policy & Risk Preferences**
   - Risikoprofil:
     - konservativ (bevorzugt etablierte, hoch reputierte Provider),
     - opportunistisch (erlaubt günstigere, aber riskantere Provider).

---

### 7. Von ADL zur laufenden Agenteninstanz

Der Weg von einer ADL-Spezifikation zur ausführenden Agenteninstanz umfasst:

1. **Parsing & Validierung**
   - Syntax-Check der ADL-Spezifikation.
   - Validierung gegen bekannte Blueprints und Normen in ERY.

2. **Planung**
   - Ableitung eines Ausführungsplans:
     - Welche Queries an den Semantic Index?
     - Welche Filter und Schwellen?
     - Welche Verhandlungsstrategie?

3. **Instantiation**
   - Start einer WASM-Agenteninstanz (Seeker oder Provider).
   - Übergabe des ADL-Objekts und des Ausführungsplans.

4. **Ausführung im Cybernetic Loop**
   - Discovery, Validation, Negotiation.
   - Erzeugung des transaktionalen Pakets für NOA.

5. **Termination**
   - Agenteninstanz wird beendet, sobald der Intent:
     - erfolgreich erfüllt wurde,
     - abgebrochen wurde,
     - oder ausgelaufen ist.

Persistenter Kontext (z. B. langfristige Präferenzen) liegt nicht im Agenten,
sondern wird über ERY und NOA referenziert.

---

### 8. Provider-Policies und Matching

Provider Agents nutzen ebenfalls ADL-ähnliche Strukturen, um ihre Angebote und
Teilnahmebedingungen zu beschreiben.

**Beispiele für Provider-Policies:**

- Minimalpreise, maximale Auslastung, bevorzugte Kundensegmente.
- Bedingungen, unter denen keine Angebote abgegeben werden:
  - unzureichender Trust des Seekers,
  - fehlende Attestations,
  - bestimmte regulatorische Einschränkungen.

**Matching-Prozess:**

- Matching ist ein **wechselseitiger Constraint-Satisfiability-Prozess**:
  - Seeker-Constraints ∩ Provider-Policies ∩ Kontext (ERY) → Potenzial für Deal.
- Nur wenn beide Seiten ihre Constraints erfüllt sehen, kommt es zur Verhandlung.

---

### 9. Sicherheit & Missbrauchsprävention auf Agentenebene

Mögliche Risiken:

- bösartige Agentenlogik,
- Ressourcenmissbrauch,
- Denial-of-Service-Angriffe,
- Privacy-Lecks.

Erynoa begegnet dem mit:

- **WASM-Isolation**:
  - begrenzte und kontrollierte Host-APIs.
- **Rate-Limiting & Quotas**:
  - Beschränkung der Anzahl und Frequenz von Agentenaktionen.
- **Trust-basierten Limits**:
  - Gering vertrauenswürdige Akteure erhalten strengere Quoten und Limits.
- **Auditing & Observability**:
  - Telemetrie über Agentenaktionen (aggregiert, ohne Geschäftsgeheimnisse zu verletzen).

---

### 10. Zusammenspiel mit anderen Konzepten

Das Agenten- und ADL-Modell steht nicht isoliert, sondern ist eingebettet in:

- das **Liquide Datenmodell**:
  - ADL referenziert Blueprints und AMO-Typen.
- das **Trust- & Reputationsmodell**:
  - ADL-Constraints nutzen Trust Vectors und Attestations.
- den **Cybernetic Loop**:
  - Agenten sind die aktiven Akteure in Phasen 1–4.

Damit bildet ECHO mit seinen Agenten die **operative Oberfläche** von Erynoa:
Menschen, Maschinen und Organisationen interagieren nicht direkt mit dem Ledger,
sondern über Agenten, die ihre Interessen in einer sicheren, normbasierten
und vertrauensbewussten Weise durchsetzen.

