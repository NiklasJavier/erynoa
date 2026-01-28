# Erynoa – Trust & Reputation

> **Zielgruppe:** Protokoll-Designer, Sicherheits-/Risk-Teams, Data Scientists und Architekt:innen
> **Kontext:** Detailverständnis des Vertrauensmodells
> **Verwandte Dokumente:** [Liquides Datenmodell](./liquides-datenmodell.md), [Cybernetic Loop](./cybernetic-loop.md), [Glossar](./glossary.md)

---

## 1. Motivation: Warum Vertrauen zentral ist

In einer Maschinenökonomie treffen:

- autonome Agenten Entscheidungen,
- Unternehmen schließen hochfrequente Mikroverträge,
- Maschinen handeln in Echtzeit über physische Ressourcen.

Ohne ein explizites Vertrauensmodell entstehen:

- ineffiziente Märkte (hohe Risikoaufschläge, manuelle Prüfprozesse),
- Angriffsflächen für Betrug und Sybil-Attacken,
- und Systeme, die zwar formal korrekt, aber ökonomisch unbrauchbar sind.

Erynoa macht **Vertrauen** zum **erstklassigen Konzept**:

- Jede Interaktion beeinflusst den Vertrauenswert von Akteuren und Objekten.
- Vertrauen wird **mathematisch** modelliert, **ontologisch** verankert und **maschinenlesbar** gemacht.

---

## 2. Grundbegriffe

- **Trust Vector**
  - Mehrdimensionaler Reputationsvektor eines Subjekts (z. B. DID, AMO, Hersteller).
  - Kann verschiedene Dimensionen abbilden (Zuverlässigkeit, Compliance, Performance, etc.).

- **Event**
  - Ein finalisiertes Ereignis auf NOA (z. B. erfolgreicher Service, Ausfall, Vertragsbruch).
  - Dient als Input für die Karmic Engine.

- **Attestation**
  - Externe, signierte Aussage über ein Subjekt (z. B. DNS-Verknüpfung, Zertifikat).
  - Wird in ERY gespeichert und in Trust-Berechnung einbezogen.

---

## 3. Karmic Engine – Mathematischer Kern

Die **Karmic Engine** ist die Komponente in ERY, die aus Events und Attestations
einen dynamischen Trust Vector berechnet.

Zentrale Formel (Ripple Effect):

\[
R*\text{new}(t) = R*\text{old}(t-1) + \eta \left(F\_\text{Event} - E[F]\right)
\]

- \( R\_\text{old}(t-1) \): bisheriger Trust Vector zur Zeit \( t-1 \)
- \( F\_\text{Event} \): Beitrag des aktuellen Events (z. B. erfolgreich, neutral, negativ)
- \( E[F] \): erwarteter oder durchschnittlicher Eventbeitrag
- \( \eta \): Lernrate / Sensitivitätsfaktor

**Intuition:**

- Positive Events (besser als erwartet) erhöhen den Trust.
- Negative Events (schlechter als erwartet) senken den Trust.
- Je nach Domäne können Dimensionen unterschiedlich gewichtet werden.

---

## 4. Fraktale Trust-Vererbung (Trust Inheritance)

Vertrauen ist in Erynoa **hierarchisch** organisiert.

Beispiele:

- Ein Hersteller betreibt mehrere Assets (Material AMOs).
- Ein Zertifizierer stellt Credentials (Credential AMOs) für viele Unternehmen aus.

Einzelne Events (z. B. Serviceausfälle oder erfolgreiche Leistungen) sollen sich nicht
nur auf das direkte AMO auswirken, sondern auch:

- auf übergeordnete Entitäten (Hersteller, Betreiber, Zertifizierer),
- und auf verbundene Norm- und Domain-Blueprints.

Dies wird durch einen Dämpfungsfaktor \( \lambda \) modelliert:

- \( \lambda \ge 0{,}99 \) für stabile, langsame Vererbung von Vertrauen.
- Direkte Teilnehmer werden stärker betroffen als entfernte Entitäten.

**Effekte:**

- **Gute Hersteller** bauen durch konsistent gute Leistungen ihrer Assets
  langfristig reputationsbasierte Vorteile auf.
- **Schlechte Praktiken** auf Ebene von Betreibern oder Zertifizierern hinterlassen
  sichtbare Spuren im System.

---

## 5. Attestations & struktureller Trust

Neben verhaltensbasiertem Vertrauen (Events) erfasst Erynoa auch **strukturellen Trust**.

Beispiele:

- DNS-basierte Attestation:
  - Eine DID wird kryptografisch mit einer Domain verknüpft.
  - Der ERY-Node verifiziert die Ownership (z. B. via DNS-TXT-Records).

- Zertifikats-Attestationen:
  - Externe Zertifizierungsstellen signieren Credentials.
  - Diese werden als Credential AMOs und zugehörige Einträge im Semantic Index abgebildet.

**Rolle in der Karmic Engine:**

- Attestations fließen als zusätzliche Dimensionen in den Trust Vector ein.
- Sie können als **Mindestanforderungen** für bestimmte Domänen fungieren:
  - z. B. Energieanbieter müssen bestimmte Zertifikate nachweisen.

---

## 6. Trust-Gating im Cybernetic Loop

Trust wird nicht nur gemessen, sondern auch **erzwingt Marktzugänge**:

1. **Intent-Phase (ECHO)**
   - Ein Seeker-Agent definiert in ADL einen **MinTrust**-Wert und ggf. benötigte Attestations.

2. **Discovery-Phase (ECHO ↔ ERY)**
   - Die Kandidatenliste wird anhand von:
     - Trust Vectors,
     - Attestations,
     - Norm-/Blueprint-Konformität
       gefiltert.

3. **Verhandlungs-Phase (ECHO)**
   - Nur Kandidaten, die die Trust-Anforderungen erfüllen, gelangen in die Consensus Bubble.

4. **Post-Event (NOA → ERY)**
   - Nach der finalen Transaktion aktualisiert die Karmic Engine die Trust Vectors.

**Ergebnis:**

- Unzuverlässige oder bösartige Akteure werden systematisch aus hochwertigen Märkten verdrängt.
- Gute Akteure profitieren langfristig von ihrem Verhalten, ohne zentrale Rating-Agenturen.

---

## 7. Speicherung von Trust-Daten im Semantic Index

Trust- und Reputationsdaten werden im **Dynamic State Layer** des Semantic Index (ERY) verwaltet:

- Trust Vectors als hochdimensionale Embeddings.
- Attestations als strukturierte Einträge mit Referenzen auf DIDs, AMOs und Blueprints.

**Vorteile der vektororientierten Repräsentation:**

- Semantisch ähnliche Akteure lassen sich clustern (z. B. ähnlich zuverlässige Anbieter).
- ECHO-Agenten können nicht nur harte Schwellen (MinTrust) setzen, sondern auch:
  - nach „ähnlichen“ oder „besseren“ Akteuren suchen,
  - Anomalien erkennen (z. B. plötzlich stark abweichendes Verhalten).

---

## 8. Governance und Missbrauchsprävention

Ein Vertrauenssystem ist angreifbar, wenn:

- Bewertungen gefälscht werden können,
- Sybil-Identitäten das System überschwemmen,
- oder einzelne Akteure übermäßige Kontrolle haben.

Erynoa adressiert diese Risiken durch:

- **Kryptografische Bindung** von Identitäten an DIDs und Domains.
- **On-Chain-Finalität** von Events:
  - Manipulation im Nachhinein ist praktisch ausgeschlossen.
- **Verteilte Berechnung** in der Karmic Engine:
  - mehrere ERY-Nodes können unabhängig Trust Vectors berechnen und vergleichen.
- **Governance-Regeln** für:
  - Einführung neuer Blueprint-Versionen,
  - Gewichtung von Eventtypen,
  - Umgang mit Streitfällen und Disputen (z. B. Off-Chain-Schlichtung + On-Chain-Korrekturen).

---

## 9. Zusammenspiel mit dem Liquiden Datenmodell

Trust & Reputation entfaltet seine volle Wirkung erst in Verbindung mit dem **Liquiden Datenmodell**:

- Normative Standards liefern die **semantische Basis**.
- Domain Blueprints definieren, welche Events und Attestations relevant sind.
- AMOs sind die Träger dieser Ereignisse.
- Fluid Extensions können kurzfristige Qualitäts- oder Performanceindikatoren abbilden.

So entsteht ein System, in dem:

- **jedes Objekt** nicht nur durch „was es ist“, sondern auch durch **„wie es sich verhält“** beschrieben wird,
- und Agenten Entscheidungen treffen, die:
  - Normkonformität,
  - aktuelle Zustände
  - und historisches Verhalten
    gleichzeitig berücksichtigen.

---

## 10. Fazit

Das Trust- und Reputationsmodell von Erynoa macht:

- Vertrauen **quantifizierbar**,
- Verhalten **nachvollziehbar**,
- und Marktzugänge **steuerbar**,

ohne auf zentrale Plattformbetreiber oder intransparente Scoring-Modelle angewiesen zu sein.

Damit bildet es, zusammen mit dem Liquiden Datenmodell, die Grundlage für eine robuste, skalierbare und faire Maschinenökonomie.

---

**Weiterführende Dokumente:**

- [Cybernetic Loop](./cybernetic-loop.md) – Der universelle Workflow
- [Agents & ADL](./agents-and-adl.md) – Agenten und ihre Interaktionen
- [Use Cases](./use-cases.md) – Konkrete Anwendungsbeispiele
