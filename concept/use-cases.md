# Erynoa – Use Cases & Narrative

> **Zielgruppe:** Product/Business, Partner, Marketing, technische Leser:innen
> **Kontext:** Das Protokoll über konkrete Geschichten verstehen
> **Verwandte Dokumente:** [Kernkonzept](./kernkonzept.md), [Cybernetic Loop](./cybernetic-loop.md), [Glossar](./glossary.md)

---

## 1. Ziel dieses Dokuments

Dieses Dokument beschreibt ausgewählte Use Cases, um das abstrakte Konzept von Erynoa
in konkrete, nachvollziehbare Szenarien zu übersetzen.

Für jeden Use Case werden dargestellt:

- Problem im Status quo
- Ablauf im Cybernetic Loop (phasenweise)
- Nutzen von Erynoa gegenüber bestehenden Lösungen

---

## 2. Use Case: Intelligentes Laden von Elektrofahrzeugen

### 2.1 Problem im Status quo

- Ladeinfrastruktur ist fragmentiert:
  - unterschiedliche Betreiber, Tarife, Authentifizierungsmechanismen.
- Nutzer haben:
  - wenig Transparenz über Herkunft der Energie (z. B. erneuerbar vs. fossil),
  - keine verlässliche Aussage über Verfügbarkeit und Qualität einer Ladesäule.
- Betreiber können:
  - ihre Qualität und Zuverlässigkeit schwer glaubhaft machen,
  - sich kaum durch nachweisbar gutes Verhalten differenzieren.

### 2.2 Ablauf im Cybernetic Loop

**Phase 1 – Intent (ECHO, ADL)**

- Das Fahrzeug (oder der Fahrer) initiiert einen Seeker-Agent mit folgendem Intent:
  - Lade 50 kWh in den nächsten 30 Minuten,
  - ausschließlich aus erneuerbaren Quellen,
  - innerhalb einer bestimmten Geohashing-Region,
  - nur bei Betreibern mit hoher Zuverlässigkeitsreputation und gültigen Zertifikaten.

**Phase 2 – Discovery & Kontext (ECHO ↔ ERY)**

- Der Seeker-Agent nutzt den Semantic Index:
  - filtert nach Blueprints für „EV-Charging-Station“,
  - berücksichtigt Domain- und Normanforderungen,
  - sucht nach Material AMOs (Ladesäulen) in der Zielregion.

**Phase 3 – Validation & Trust-Gating (ERY)**

- Die Karmic Engine liefert Trust Vectors für Betreiber und Ladesäulen:
  - SLA-Einhaltung, Ausfallhistorie, Nutzerfeedback-Ereignisse.
- Attestations verifizieren:
  - DNS-Bindung zwischen Betreiber-DID und dessen Domain,
  - Zertifikate zu erneuerbarer Energie.
- Kandidaten, die MinTrust oder Zertifikatsanforderungen nicht erfüllen, werden ausgeschlossen.

**Phase 4 – Negotiation (ECHO)**

- Seeker- und Provider-Agent (des Betreibers) verhandeln:
  - Preis pro kWh,
  - maximale Ladeleistung,
  - Startzeit und Höchstdauer.
- Die Verhandlung findet in einer XMTP-Consensus-Bubble statt:
  - interne Kostendaten oder Auslastungsprognosen des Betreibers bleiben privat.

**Phase 5 – Execution (NOA)**

- Ein Service AMO wird für die Ladesession erstellt:
  - verknüpft das Fahrzeug, die Ladesäule und den Betreiber.
- Move-Transaktion:
  - startet einen Continuous Value Stream für die abgerufene Energie,
  - aktualisiert den Zustand der beteiligten AMOs (z. B. Nutzungszähler).

**Phase 6 – Feedback (NOA → ERY)**

- Nach Abschluss des Ladevorgangs:
  - werden Events (erfolgreich, unterbrochen, SLA eingehalten/verfehlt) nach ERY gespiegelt.
- Die Karmic Engine:
  - aktualisiert Trust Vectors des Betreibers, der Ladesäule und ggf. des Fahrzeugs (Nutzungsverhalten).

### 2.3 Nutzen von Erynoa

- **Für Nutzer / Fahrzeuge**
  - Transparent nachvollziehbare Qualität und Herkunft der Energie.
  - Automatisierte Auswahl der besten Option nach Preis, Vertrauen und Normen.

- **Für Betreiber**
  - Nachweisbar gute Performance erhöht Reputation und Marktanteil.
  - Keine zentrale Plattform nötig, um Kunden zu erreichen.

- **Für das System**
  - Jede Ladesession verbessert das Vertrauensmodell und das Wissen über Infrastrukturqualität.

---

## 3. Use Case: Zertifizierte Wartung von Industrieanlagen

### 3.1 Problem im Status quo

- Wartungszertifikate werden oft:
  - in isolierten Systemen verwaltet,
  - als PDF oder proprietäre Einträge geführt,
  - schwer automatisiert überprüfbar.
- Betreiber und Versicherer:
  - haben hohen manuellen Prüfaufwand,
  - kämpfen mit Betrug oder unvollständigen Nachweisen.

### 3.2 Ablauf im Cybernetic Loop

**Phase 1 – Intent (ECHO, ADL)**

- Ein Betreiber initiiert einen Seeker-Agent:
  - Ziel: zertifizierte Wartung für eine Maschine mit bestimmten Normanforderungen.
  - Normative Constraints:
    - bestimmte Wartungsstandards,
    - nur Provider mit gültigem „Maintenance-Credential“-AMO.
  - Trust-Constraints:
    - Mindestreputation für Zuverlässigkeit und Compliance.

**Phase 2 – Discovery & Kontext (ECHO ↔ ERY)**

- Semantic Index:
  - filtert Provider Agents, die den Wartungs-Blueprint unterstützen,
  - prüft Domain Blueprints für „Maintenance-Service“,
  - berücksichtigt Geo- und Industriespezifika.

**Phase 3 – Validation & Trust-Gating (ERY)**

- Karmic Engine:
  - prüft historische Erfolgsquote und SLA-Einhaltung der Wartungsanbieter.
- Attestations:
  - Zertifizierungsstellen signieren Credential AMOs,
  - DNS-Bindung zu Unternehmensdomains wird verifiziert.

**Phase 4 – Negotiation (ECHO)**

- Vertragsverhandlung in der Consensus Bubble:
  - Preis, Umfang, Zeitfenster,
  - Haftung und Service-Levels.

**Phase 5 – Execution (NOA)**

- Nach durchgeführter Wartung:
  - wird ein **Credential AMO** ausgestellt:
    - Soulbound an die DID der Maschine,
    - referenziert den Wartungs-Blueprint und Normen.
  - Logic Guards stellen sicher:
    - dass nur autorisierte Provider dieses Credential erzeugen können.

**Phase 6 – Feedback (NOA → ERY)**

- Erfolgreiche oder mangelhafte Wartungen fließen als Events in die Karmic Engine ein.
- Trust Vectors der Provider und ggf. der Zertifizierungsstellen werden angepasst.

### 3.3 Nutzen von Erynoa

- **Automatisierte Verifikation:**
  - Credentials sind maschinenlesbar, normbasiert und on-chain verankert.
- **Reduzierter Prüfaufwand**:
  - Versicherer und Auditoren können Zustände kryptografisch prüfen, statt Dokumente manuell zu sichten.
- **Belohnung guter Akteure**:
  - zuverlässige Wartungsdienstleister bauen nachhaltige Reputation auf.

---

## 4. Use Case: Echtzeit-Energiehandel zwischen Prosumer und Netz

### 4.1 Problem im Status quo

- Prosumer (z. B. Haushalte mit PV-Anlage) haben:
  - eingeschränkte Möglichkeiten, flexibel und granular Energie zu handeln.
- Netzbetreiber:
  - verfügen oft nur verzögert über Informationen zu lokaler Erzeugung und Nachfrage.
- Märkte:
  - sind träge, zentralisiert und schwer für kleine Akteure zugänglich.

### 4.2 Ablauf im Cybernetic Loop

**Phase 1 – Intent (ECHO, ADL)**

- Ein Prosumer initiiert einen Seeker-Agent:
  - Ziel: Verkaufen von Überschussenergie in den nächsten 15 Minuten,
  - Mindestpreis, Herkunftsanforderungen (z. B. lokal), bevorzugte Käufer (z. B. Nachbarn, bestimmte Profile).

**Phase 2 – Discovery & Kontext (ECHO ↔ ERY)**

- ERY:
  - kennt Material AMOs für Erzeugungsanlagen,
  - kennt Service AMOs und mögliche Abnehmer in der Region (über Geohashing).

**Phase 3 – Validation & Trust-Gating (ERY)**

- Trust:
  - bewertet die Zuverlässigkeit des Prosumers (z. B. Einhaltung früherer Lieferzusagen),
  - und der potenziellen Abnehmer (Zahlungsverhalten).

**Phase 4 – Negotiation (ECHO)**

- Aushandlung:
  - Preis pro kWh,
  - Zeitfenster und Menge,
  - ggf. Bedingungen (z. B. CO₂-Intensität).

**Phase 5 – Execution (NOA)**

- Service AMO:
  - repräsentiert den temporären Energiefluss,
  - Continuous Value Streaming fließt vom Käufer zum Verkäufer proportional zur gelieferten Energie.

**Phase 6 – Feedback (NOA → ERY)**

- Ereignisse:
  - erfolgreiche Lieferungen, Ausfälle, Abweichungen.
- Karmic Engine:
  - passt Trust Vectors aller Beteiligten an.

### 4.3 Nutzen von Erynoa

- **Feingranulare Märkte:** Auch kleine Akteure können am Echtzeitmarkt teilnehmen.
- **Systemstabilität:** Netzbetreiber können auf aggregierte Informationen und Trust-Daten zugreifen.
- **Transparenz & Fairness:** Preise und Leistungen werden durch nachvollziehbare Reputation und Normen gestützt.

---

## 5. Narrative Klammer

Über alle Use Cases hinweg zeigt sich:

- **Erynoa:**
  - verbindet reale Assets, Daten und Akteure über Blueprints und AMOs
  - bewertet Verhalten kontinuierlich über Trust & Reputation
  - und orchestriert Interaktionen über Agenten und den Cybernetic Loop

Damit verschiebt sich der Fokus von **„Transaktionen auf einer Blockchain“** hin zu **„lebendigen Märkten zwischen lernenden Maschinen und Organisationen“**.

---

## 6. Fazit

Die Use Cases illustrieren, wie Erynoa abstrakte Konzepte in konkrete Wertschöpfung übersetzt: automatisierte, vertrauensbasierte Interaktionen zwischen Maschinen, Unternehmen und Nutzern – ohne zentrale Vermittler.

---

**Weiterführende Dokumente:**

- [Kernkonzept](./kernkonzept.md) – High-Level-Überblick
- [Cybernetic Loop](./cybernetic-loop.md) – Detaillierter Workflow
- [Agents & ADL](./agents-and-adl.md) – Agentenmodell und Sprache
