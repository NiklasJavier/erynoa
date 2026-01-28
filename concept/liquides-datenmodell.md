## Erynoa – Liquides Datenmodell

**Zielgruppe:** Protokoll- und Datenarchitekt:innen, Research/Domain-Expert:innen, die das Objekt- und Datenmodell tief verstehen wollen.

### 1. Motivation

Klassische Distributed-Ledger-Systeme modellieren digitale Güter häufig als starre Datenstrukturen:

- Ein „Asset“ ist ein festes Datenobjekt mit wenigen, hart codierten Attributen.
- Kontext (rechtlich, physikalisch, normativ) liegt außerhalb des Ledgers in Dokumenten, Verträgen oder proprietären Systemen.
- Änderungen in Domänen (z. B. neue Normen, regulatorische Anforderungen) erzwingen oft Protokoll- oder Smart-Contract-Migrationen.

Das **Liquide Datenmodell** von Erynoa trennt konsequent:

- **semantische Definitionen** (Blueprints, Normen, Regeln) und
- **exekutive Instanzen** (Atomic Market Objects, AMOs),

um eine anpassungsfähige, normbasierte und skalierbare Maschinenökonomie zu ermöglichen.

---

### 2. Architektonische Basis: Erynoa Object Standard (EOS)

Der **Erynoa Object Standard (EOS)** definiert, wie Objekte in Erynoa beschrieben, validiert und ausgeführt werden.

Er unterscheidet drei Ebenen:

1. **Normative Standards (Evolutionary Blueprints)**  
2. **Domain Blueprints**  
3. **Atomic Market Objects (AMOs)**  

Während die ersten beiden Ebenen in **ERY** (Semantic Lattice) verankert sind, manifestieren sich AMOs auf **NOA** (Causal Ledger).

---

### 3. Normative Standards als „Evolutionary Blueprints“

**Normative Standards** bilden die stabile Grundlage des Liquiden Datenmodells.

Beispiele:

- **ISO 19112** – Geo-Kontexte
- **eCl@ss** – technische Merkmale und Produktklassifikation
- weitere Industrie- und Regulierungsstandards

Sie erfüllen drei Funktionen:

- **Ontologische Verankerung**  
  - Ein Standard definiert, _was_ ein Objekt oder eine Eigenschaft in einer Domäne bedeutet.
  - Beispiel: Was ist eine Ladesäule, welche Parameter sind relevant, wie werden sie gemessen?

- **Normative Referenz**  
  - Standards dienen als Referenz für Compliance und Zertifizierung.
  - Agenten können prüfen, ob Objekte einem Standard entsprechen, ohne dessen Inhalt „raten“ zu müssen.

- **Vertrauensanker**  
  - Normative Standards tragen einen eigenen Vertrauenswert.  
  - Über die Karmic Engine propagiert sich dieses Vertrauen fraktal auf alle abgeleiteten Blueprints und AMOs.

**Evolutionäres Verhalten:**

- Der Kern eines Standards bleibt **immutabel**, um Referenzstabilität zu gewährleisten.
- Gleichzeitig erlaubt Erynoa eine **kontrollierte Evolution**:
  - Neue Versionen eines Standards können eingeführt werden.
  - Governance-Mechanismen definieren, wann und wie Agenten auf neue Normversionen migrieren.

Damit wird ein Spannungsfeld aufgelöst:

- Stabil genug für Rechtssicherheit.
- Flexibel genug für technologische und regulatorische Weiterentwicklung.

---

### 4. Domain Blueprints – Spezialisierte Objektdefinitionen

Auf Basis normativer Standards werden **Domain Blueprints** definiert.

**Aufgabe der Domain Blueprints:**

- Operationalisieren Normen für konkrete Anwendungsdomänen:
  - z. B. Industriebatterien, Energie-Assets, Mobilitätsservices, Zertifikate.
- Definieren:
  - relevante Attribute (z. B. Kapazität, Effizienz, Standort).
  - zulässige Wertebereiche.
  - Validierungslogik in Form von **MoveScripts**.

**Beispiele:**

- Blueprint „EV-Charging-Station“  
  - basiert auf ISO- und eCl@ss-Standards für Energie- und Infrastruktur.  
  - definiert technische Parameter wie Ladeleistung, Steckertyp, Spannungsbereich.  
  - enthält Policies zur Abrechnung, Verfügbarkeit, Sicherheitsanforderungen.

- Blueprint „KYC-Credential“  
  - basiert auf regulatorischen Anforderungen (z. B. AML/KYC-Richtlinien).  
  - definiert, welche Attribute ein Credential haben muss und wie es verifiziert wird.

**Technische Verankerung:**

- Domain Blueprints werden als Einträge im **Static Knowledge Layer** des Semantic Index (Qdrant) gespeichert.
- Sie verlinken:
  - auf ihre normativen Wurzeln (Evolutionary Blueprints),
  - auf die dazugehörigen **MoveScripts** in NOA.

---

### 5. Atomic Market Objects (AMOs) – Exekutive Ebene in NOA

In **NOA** materialisieren sich die abstrakten Definitionen aus ERY als **Atomic Market Objects (AMOs)**.

**Allgemeine Eigenschaften von AMOs:**

- Ein AMO ist ein digitaler Container, dessen Verhalten durch:
  - den referenzierten Blueprint und
  - die zugrunde liegende **MoveVM**
  bestimmt wird.
- Jede Zustandsänderung eines AMOs unterliegt:
  - den **Logic Guards** des zugehörigen Blueprints,
  - den globalen Invarianten von NOA (Resource Safety).

**Drei fundamentale Archetypen:**

1. **Material AMOs (Physische Güter)**  
   - Modellieren Real World Assets und IoT-Hardware.  
   - Eigenschaften:
     - **Transferierbar** über Atomic Settlement.
     - Abbildbarkeit physischer Knappheit (ein Asset kann nicht an zwei Stellen gleichzeitig genutzt werden).
   - Beispiele:
     - Ladesäulen, Sensoren, Speicheranlagen, Maschinen.

2. **Credential AMOs (Immaterielle Nachweise)**  
   - Modellieren Nachweise, Zertifikate, Qualifikationen.  
   - Eigenschaften:
     - **Soulbound** – untrennbar mit einer DID verknüpft.
     - Nicht transferierbar, nur **verifizierbar** (Verifiable Presentations).
   - Beispiele:
     - KYC-/AML-Nachweise, Sicherheitszertifikate, Wartungszertifikate.

3. **Service AMOs (Zeitgebundene Verträge)**  
   - Modellieren laufende Dienstleistungen und Flüsse.  
   - Eigenschaften:
     - **Flüchtig** – existieren nur für die Dauer der Dienstleistung.
     - Unterstützen **Continuous Value Streaming**:
       - z. B. sekundenbasierte Bezahlung von Energie, Rechenleistung, Datenstreams.
   - Beispiele:
     - Ladevorgang, Energieliefervertrag in Echtzeit, API-Nutzung pro Anfrage.

Diese Archetypen können kombiniert und erweitert werden, um komplexe Wertschöpfungsketten abzubilden.

---

### 6. Fluid Extensions – Umgang mit flüchtigen Daten

Ein zentrales Problem klassischer Ledgers:

- Dynamische Zustände (z. B. Sensordaten, Geo-Positionen) werden dauerhaft gespeichert.
- Die Kette wächst unkontrolliert („State Bloat“), ohne dass der Großteil der Daten langfristig relevant ist.

Erynoa führt hierfür **Fluid Extensions** ein.

**Fluid Extensions:**

- Temporäre Erweiterungen eines AMOs um flüchtige Attribute.
- Beispiele:
  - aktuelle Standortkoordinate einer Ladesäule,
  - momentane Auslastung,
  - kurzfristige Preissignale oder Messwerte.

**Time-To-Live (TTL):**

- Jede Fluid Extension besitzt ein **TTL-Attribut**.
- Nach Ablauf der TTL:
  - verfällt die Erweiterung automatisch,
  - wird aus dem Semantic Index entfernt,
  - ohne dass zusätzliche On-Chain-Transaktionen nötig sind.

**Effekte:**

- Der Ledger wird von kurzlebigen Daten entkoppelt.
- ERY kann Milliarden von Objekten und Attributen verwalten, ohne unkontrolliert zu wachsen.
- Agenten erhalten trotzdem Zugriff auf hochaktuelle Informationen.

---

### 7. Liquide Ontologie: Kombination aus Stabilität und Dynamik

Das Liquide Datenmodell vereint:

- **Stabile, normative Schichten**:
  - Normative Standards als Evolutionary Blueprints.
  - Domain Blueprints als domänenspezifische Spezialisierung.

- **Dynamische, exekutive Schichten**:
  - AMOs auf NOA als ausführbare Container.
  - Fluid Extensions und Trust-Daten in ERY.

Dadurch wird der Ledger:

- nicht nur eine Historie von Transaktionen,
- sondern eine **kontextbewusste Exekutivinstanz**, die:
  - Normen respektiert,
  - Domänenwissen einbettet,
  - und sich gleichzeitig an die Realität der Maschinenökonomie anpasst.

---

### 8. Ausblick und Verknüpfung

In Kombination mit dem **Trust- und Reputationsmodell** (Karmic Engine, Trust Vectors)
entsteht eine Ontologie, in der:

- Objekte nicht nur „existieren“, sondern
- mit **Bedeutung**, **Qualität** und **Vertrauen** verknüpft sind.

Die nächste Ebene ist daher das Dokument `trust-and-reputation.md`, das beschreibt,
wie Vertrauen im System berechnet, propagiert und für Entscheidungen genutzt wird.

