## Erynoa – Glossar

**Zielgruppe:** Alle Leser:innen der Erynoa-Dokumentation (Concept, Docs, externe Materialien), die eine konsistente Begriffsbasis benötigen.

### 1. Zweck

Dieses Glossar definiert zentrale Begriffe des Erynoa-Protokolls in knapper,
referenzierbarer Form. Es dient als gemeinsame Sprache für Konzept-, Architektur-
und Implementierungsdokumente.

---

### 2. Begriffe

**ADL (Agent Definition Language)**  
Deklarative Sprache zur Beschreibung von Intents, Constraints und Policies für
Seeker- und Provider-Agenten in der ECHO-Sphäre.

**AMO (Atomic Market Object)**  
Zentrale On-Chain-Entität in NOA. Digitaler Container, dessen Verhalten durch
Blueprints und die MoveVM definiert ist. Drei Archetypen: Material, Credential,
Service.

**Attestation**  
Signierte Aussage einer externen oder internen Instanz über ein Subjekt
(z. B. DID, AMO). Beispiele: DNS-Bindung, Zertifikate, Konformitätsnachweise.

**Blueprint**  
Semantische und technische Schablone für Objekte und Prozesse. Es gibt
normative Standards (Evolutionary Blueprints) und Domain Blueprints, die
konkrete Validierungslogiken und Taxonomien definieren.

**Continuous Value Streaming**  
Abrechnungsmodell, bei dem der Wert kontinuierlich und fein granular über
Zeit transferiert wird (z. B. pro Sekunde Energiefluss), statt in diskreten
Einmalzahlungen.

**Consensus Bubble**  
Verschlüsselte Off-Chain-Kommunikationsumgebung (XMTP), in der Seeker- und
Provider-Agenten privat verhandeln, bevor sie das Ergebnis auf NOA finalisieren.

**Credential AMO**  
AMO-Typ, der immaterielle Nachweise modelliert (z. B. KYC, Wartungszertifikate).
Soulbound an eine DID, nicht transferierbar, nur verifizierbar.

**DHT (Distributed Hash Table)**  
Verteilte Datenstruktur zur Partitionierung und Auffindbarkeit von Daten im
Netzwerk. In Erynoa kombiniert mit Geohashing für synaptisches Sharding.

**DID (Decentralized Identifier)**  
Dezentraler Identifikator für Akteure (Personen, Organisationen, Maschinen),
der kryptografisch gesichert ist und on- bzw. off-chain referenziert werden kann.

**ECHO (Emergent Swarm)**  
Sphäre der operativen Intelligenz. Führt Agentenlogik aus, wickelt Discovery,
Verhandlung und Ausführung von Intents ab.

**ERY (Semantic Lattice)**  
Semantische Sphäre und Gedächtnis des Netzwerks. Speichert Blueprints,
Trust Vectors, Attestations und Fluid Extensions. Basis für semantische Suchen
und Trust-Berechnung.

**EOS (Erynoa Object Standard)**  
Architektonische Grundlage des liquiden Datenmodells. Definiert die Beziehung
zwischen Normativen Standards, Domain Blueprints und AMOs.

**Event**  
Abstraktion eines finalisierten Vorgangs in NOA (z. B. erfolgreiche Lieferung,
SLA-Verstoß). Dient als Input für die Karmic Engine.

**Fluid Extensions**  
Temporäre Attribut-Erweiterungen von AMOs, die flüchtige Daten (z. B. aktuelle
Geo-Position, Sensorwerte) modellieren. Besitzen ein TTL und werden automatisch
entfernt, um State Bloat zu vermeiden.

**Geohashing**  
Kodierung geographischer Regionen in kompakte Strings. In Erynoa genutzt zur
räumlichen Partitionierung (DHT) und zur Definition von Geo-Constraints in ADL.

**Karmic Engine**  
Komponente in ERY, die aus Events und Attestations Trust Vectors berechnet.
Nutzt den Ripple-Effekt, um Vertrauen dynamisch und fraktal zu aktualisieren.

**Layer 0 (NOA)**  
On-Chain-Ebene des Erynoa-Protokolls. Basierend auf IOTA Rebased und Starfish BFT.
Speichert kausale Wahrheiten, führt Move-Transaktionen aus und verwaltet AMOs.

**Layer 2 (ERY, ECHO)**  
Off-Chain-Ebene für Semantik und Intelligenz. Umfasst Semantic Lattice (ERY)
und Agentensphäre (ECHO). Entlastet Layer 0 von rechenintensiven Aufgaben.

**Logic Guards**  
Smart-Contract-artige Prüfmechanismen in NOA, die vor jeder Zustandsänderung
Invarianten sicherstellen (z. B. Soulbound-Regeln, Compliance, Ressourcensicherheit).

**Material AMO**  
AMO-Typ, der physische Güter und Real World Assets modelliert (z. B. Ladesäulen,
Maschinen, Sensoren). Transferierbar, knüpft an physische Knappheit an.

**Move / MoveVM**  
Programmiersprache und virtuelle Maschine in NOA. Optimiert auf Resource Safety
und formale Kontrolle über Assets.

**NOA (Causal Ledger)**  
Sphäre der kausalen Wahrheit. On-Chain-Ledger, der Transaktionen finalisiert und
Zustandsänderungen an AMOs vollzieht.

**Normative Standards (Evolutionary Blueprints)**  
Etablierte Industriestandards (z. B. ISO 19112, eCl@ss), die als unveränderliche
Grundlagen für Domain Blueprints dienen und als Evolutionary Blueprints in ERY
verankert sind.

**Progressive Disclosure**  
Prinzip, nach dem sensible Informationen in Verhandlungen nur schrittweise
offengelegt werden, wenn Vertrauen und Interesse beidseitig gegeben sind.

**Seeker Agent**  
Agentenrolle für Nachfrager. Formuliert Intents, führt Discovery durch, wählt
Provider aus und initiiert Verhandlungen.

**Service AMO**  
AMO-Typ, der zeitgebundene Dienstleistungen modelliert (z. B. Ladevorgänge,
Energieflüsse, API-Nutzung). Unterstützt Continuous Value Streaming.

**Starfish BFT**  
Leaderloser Konsensmechanismus in NOA. Sorgt für deterministische Finalität
von Transaktionen in unter zwei Sekunden.

**Synapse (ERY-Synapse)**  
Elementare, inhaltsadressierte Speichereinheit in ERY, die Daten und ihre
semantischen Beziehungen verwaltet. Grundlage der synaptischen Sharding-
Architektur.

**Trust Gating**  
Mechanismus, bei dem minimale Trust-Schwellen und Attestationsanforderungen
als Zugangskriterium für Interaktionen dienen. Implementiert in ADL-Constraints
und ERY-Prüfschritten.

**Trust Vector**  
Mehrdimensionaler Vektor, der das Vertrauen in ein Subjekt beschreibt. Wird
von der Karmic Engine auf Basis von Events und Attestations berechnet und
im Semantic Index gespeichert.

**TTL (Time-To-Live)**  
Lebensdauer eines flüchtigen Dateneintrags (z. B. Fluid Extension). Nach Ablauf
des TTL wird der Eintrag automatisch aus ERY entfernt.

**XMTP (Extensible Message Transport Protocol)**  
Protokoll für verschlüsselte Nachrichtenkanäle zwischen Agenten. In Erynoa
für Consensus Bubbles und Progressive Disclosure genutzt.

---

### 3. Verwendung

Dieses Glossar sollte bei neuen Begriffen erweitert werden und dient als
Referenzpunkt für:

- Konzeptdokumente im `concept/`-Verzeichnis,
- Architektur- und Implementierungsdokumente in `docs/`,
- externe Kommunikation (Whitepaper, Präsentationen).

