# Projekt Pluto — Fachkonzept

## Technische Architektur eines dezentralen Trust-basierten Systems

**Version 15.0 | Datum: 2026-02-05 | Status: Implementation-Ready**

---

## Zusammenfassung

Das vorliegende Fachkonzept beschreibt die technische Architektur von Projekt Pluto, einem dezentralen System für vertrauensbasierte digitale Interaktionen. Das System ermöglicht selbstsouveräne Identitäten, reputation-gesteuerte Ressourcenallokation und kryptographisch abgesicherte Zustandsübergänge ohne zentrale Infrastruktur.

Die Architektur basiert auf 90 formalen Axiomen und 12 mathematisch bewiesenen Theoremen. Sie umfasst Zero-Knowledge-Beweise für Privatsphäre, hierarchische Identitäten für Flexibilität und adaptive Regelkreise für Systemstabilität.

---

## 1. Systemgrundlagen

### 1.1 Das Universum

Das Pluto-System wird als mathematische Kategorie modelliert, bestehend aus grundlegenden Objekttypen und den Beziehungen zwischen ihnen. Die Kernobjekte umfassen dezentrale Identifikatoren (DIDs), Realms als Governance-Container, Trust-Vektoren für Reputation, Ressourcen wie Gas und Mana, Events für Zustandsänderungen sowie State-Objekte für persistente Daten.

Alle Relationen im System unterliegen strengen Ordnungseigenschaften: Sie sind irreflexiv (ein Objekt steht nicht in Relation zu sich selbst), antisymmetrisch (keine zirkulären Abhängigkeiten) und transitiv (Beziehungen propagieren entlang von Ketten). Diese Eigenschaften garantieren, dass das System keine Deadlocks oder unauflösbare Zyklen enthält.

### 1.2 Identitätsarchitektur

Jede Identität im System wird durch einen dezentralen Identifikator (DID) repräsentiert. Ein DID besteht aus drei Komponenten: einem Namespace, der den Typ der Identität kennzeichnet, einem eindeutigen Fingerprint, der durch den BLAKE3-Hash des Namespaces und des öffentlichen Schlüssels berechnet wird, sowie dem öffentlichen Schlüssel selbst.

Das System definiert zehn verschiedene Namespaces für unterschiedliche Anwendungsfälle:

- **Self**: Natürliche Personen
- **Guild**: Organisationen und Unternehmen
- **Spirit**: KI-Agenten und autonome Systeme
- **Thing**: IoT-Geräte und physische Objekte
- **Vessel**: Container und Transporteinheiten
- **Source**: Datenquellen und Orakel
- **Craft**: Automatisierte Workflows
- **Vault**: Speicher für sensible Daten
- **Pact**: Smart Contracts
- **Circle**: Gruppen und Gemeinschaften

Die fundamentale Sicherheitsgarantie des Systems basiert auf dem Passkey-Primacy-Axiom: Jede Identität muss durch genau einen hardware-gebundenen Passkey authentifiziert werden. Dies stellt sicher, dass die Kontrolle über eine Identität nicht durch Software-Kompromittierung übernommen werden kann.

### 1.3 Delegation und Schlüsselkontrolle

Das System erlaubt hierarchische Delegation von Berechtigungen. Wenn eine Identität einer anderen Identität Berechtigungen delegiert, unterliegt dieser Vorgang dem Decay-Prinzip: Die delegierte Identität kann maximal einen konfigurierbaren Anteil (typischerweise 30-90%) des Trust-Wertes der ursprünglichen Identität nutzen. Die maximale Delegationstiefe ist auf fünf Ebenen begrenzt, um eine unkontrollierte Ausbreitung von Berechtigungen zu verhindern.

Alle kryptographischen Schlüssel einer Identität verbleiben unter der Kontrolle des Identitätseigners. Eine einmal erstellte eindeutige Kennung kann nachträglich nicht mehr verändert werden, was die Integrität des Systems über die Zeit garantiert.

---

## 2. Trust-Algebra

### 2.1 Der Trust-Vektor

Das Herzstück des Systems ist der mehrdimensionale Trust-Vektor. Jede Identität besitzt einen Vektor mit vier bis sechs Dimensionen, abhängig von der Systemkonfiguration.

Die vollständige sechsdimensionale Variante umfasst:

- **Reliability (R)**: Zuverlässigkeit bei der Erfüllung von Verpflichtungen
- **Integrity (I)**: Konsistenz zwischen Aussagen und Handlungen
- **Competence (C)**: Fähigkeit zur korrekten Ausführung von Aufgaben
- **Predictability (P)**: Vorhersagbarkeit des Verhaltens
- **Vitality (V)**: Aktivitätsniveau und Reaktionsfähigkeit
- **Goodwill (Ω)**: Positive Intentionen gegenüber anderen Teilnehmern

Alternativ kann eine vierdimensionale Konfiguration verwendet werden, die Reliability, Competence, Social Standing und Goodwill kombiniert.

Die Gesamtreputation einer Identität wird durch die gewichtete euklidische Norm des Trust-Vektors berechnet, wobei die Gewichte kontextabhängig konfigurierbar sind.

### 2.2 Trust-Dynamik

Die Veränderung des Trust-Wertes unterliegt strengen mathematischen Beschränkungen. Jede einzelne Trust-Dimension ist auf den Bereich zwischen 0 und 1 (bzw. 0 und 100 Punkte) begrenzt. Pro Interaktion kann sich der Trust-Wert um maximal 10% verändern, was sowohl explosives Wachstum als auch katastrophale Abstürze verhindert.

Ein kritisches Designmerkmal ist die asymmetrische Bewertung von Veränderungen: Negative Trust-Änderungen werden mit einem Faktor von 2,0 verstärkt, während positive Änderungen unverändert bleiben. Diese Asymmetrie spiegelt die alltagspsychologische Beobachtung wider, dass Vertrauen langsam aufgebaut, aber schnell zerstört wird. Sie macht das System zudem resistent gegen Sybil-Angriffe, bei denen ein Angreifer viele falsche Identitäten erstellt.

### 2.3 Trust-Klassen

Basierend auf der gewichteten Norm werden Identitäten in vier Klassen eingeteilt:

- **Newcomer** (0-20%): Neue Teilnehmer mit eingeschränkten Rechten
- **Established** (20-50%): Teilnehmer mit nachgewiesener Aktivität
- **Trusted** (50-80%): Zuverlässige Teilnehmer mit erweiterten Rechten
- **Veteran** (80-100%): Hochvertrauenswürdige Teilnehmer mit maximalen Rechten

Ein mathematisches Theorem garantiert, dass legitime Teilnehmer den Veteran-Status mindestens fünfmal schneller erreichen als Sybil-Angreifer. Dies ergibt sich aus der Kombination von begrenzten Updates, asymmetrischem Decay und der Notwendigkeit vielfältiger positiver Interaktionen.

---

## 3. Ressourcenmodell

### 3.1 Gas: Rechenkosten

Gas repräsentiert die Rechenressourcen im System. Jede Identität erhält ein Gas-Budget, dessen Höhe von ihrem Reliability-Trust abhängt. Vertrauenswürdige Teilnehmer erhalten mehr Gas, da sie weniger wahrscheinlich missbräuchliche Operationen ausführen.

Gleichzeitig sinken die Gas-Kosten für Operationen bei höherem Trust: Ein Veteran zahlt effektiv weniger für dieselbe Operation als ein Newcomer. Das Gas-Budget regeneriert sich nicht automatisch, sondern muss durch Einzahlung von Systemwährung (Flux) oder durch andere Mechanismen wieder aufgefüllt werden.

### 3.2 Mana: Interaktionspotenzial

Mana repräsentiert das Potenzial für soziale Interaktionen. Im Gegensatz zu Gas regeneriert sich Mana automatisch über die Zeit. Die maximale Mana-Kapazität und die Regenerationsrate hängen vom Goodwill-Trust ab: Teilnehmer mit positiven Absichten erhalten mehr Mana schneller.

### 3.3 Kostenalgebra

Das System definiert eine formale Kostenalgebra mit drei Komponenten: Gas, Mana und Risiko. Bei sequentieller Ausführung von Operationen werden Gas und Mana addiert, während das Risiko nach einer speziellen Formel kombiniert wird, die sicherstellt, dass das Gesamtrisiko niemals 100% erreicht, solange einzelne Risiken begrenzt sind.

Zwei fundamentale Theoreme beschreiben das Verhalten von Angreifern und legitimen Nutzern: Erstens: Die verfügbaren Ressourcen eines Angreifers tendieren über die Zeit gegen Null, da Gas nicht regeneriert und negative Aktionen den Trust und damit das Budget reduzieren. Zweitens: Höherer Trust führt zu mehr Ressourcen, mehr Ressourcen ermöglichen mehr erfolgreiche Interaktionen, und mehr erfolgreiche Interaktionen erhöhen den Trust – ein positiver Rückkopplungskreis für legitime Teilnehmer.

---

## 4. Ausführungsmodell

### 4.1 Die ECLVM

Die Erynoa Constrained Logic Virtual Machine (ECLVM) ist die Ausführungsumgebung für Smart Contracts und Policies. Sie basiert auf WebAssembly und bietet deterministische Ausführung: Identische Eingaben führen immer zu identischen Ausgaben, unabhängig vom ausführenden Knoten.

Jede Ausführung erhält einen Kontext bestehend aus der aufrufenden Identität, dem aktuellen Realm, den verfügbaren Gas- und Mana-Budgets sowie dem aktuellen Systemzustand. Die virtuelle Maschine bietet eine begrenzte Menge an Host-Funktionen für den Zugriff auf Trust-Werte, Speicheroperationen, Gas-Verbrauch und Event-Emission.

### 4.2 Fuel-Mapping

WebAssembly-Operationen werden auf Gas-Kosten abgebildet. Jede WASM-Instruktion hat einen definierten Gas-Preis. Die Gesamtkosten einer Ausführung ergeben sich aus der Summe aller ausgeführten Instruktionskosten. Dies verhindert Denial-of-Service-Angriffe durch ressourcenintensive Berechnungen.

---

## 5. Event-System

### 5.1 Kausale Ordnung

Das System verwendet ein Event-basiertes Kommunikationsmodell. Jedes Event besteht aus einer eindeutigen ID (BLAKE3-Hash), einem Typ, einer Payload, einem optionalen Verweis auf ein Eltern-Event, einem Zeitstempel und einer Realm-Zuordnung.

Das Kausalitäts-Axiom garantiert, dass ein Event nur dann eine Referenz auf ein Eltern-Event haben kann, wenn sein Zeitstempel strikt größer ist. Dies stellt sicher, dass Events einen gerichteten azyklischen Graphen (DAG) bilden, in dem die kausale Ordnung eindeutig bestimmt ist.

### 5.2 Event-Dispatch

Wenn ein Event ausgelöst wird, wird es an alle registrierten Observer weitergeleitet. Das System garantiert, dass nach dem Dispatch eines Events der Gesamtzustand konsistent bleibt. Die Menge der zu benachrichtigenden Observer wird durch die vom Event betroffenen Entitäten bestimmt.

---

## 6. Realm-Architektur

### 6.1 Hierarchische Governance-Container

Ein Realm ist ein Governance-Container, der Regeln, Mitglieder, Governance-Mechanismen und Isolationsgrade definiert. Realms können hierarchisch verschachtelt sein: Ein Kind-Realm erbt alle Regeln seines Eltern-Realms und kann zusätzliche, spezifischere Regeln definieren.

Die Regel-Monotonie garantiert, dass Kind-Realms niemals weniger restriktiv sein können als ihre Eltern. Dies ermöglicht es, globale Grundregeln auf oberster Ebene zu definieren, die von keinem untergeordneten Realm aufgehoben werden können.

### 6.2 Isolationsgrade

Das System definiert drei Isolationsgrade für Realms:

- **Public (0)**: Voller Zugang für alle Teilnehmer
- **Members (1)**: Zugang nur für Mitglieder des Realms
- **Strict (2)**: Zugang nur mit expliziter Genehmigung

### 6.3 Realm-lokaler Trust

Ein fundamentales Designprinzip ist die Realm-Lokalität des Trust: Der Trust-Wert einer Identität in einem Realm ist orthogonal (unabhängig) zu ihrem Trust in einem anderen Realm. Ein Veteran im Gaming-Realm hat nicht automatisch hohen Trust im Finanz-Realm.

Für Cross-Realm-Interaktionen wird der Trust mit einem konfigurierbaren Dämpfungsfaktor multipliziert. Ein mathematisches Theorem garantiert, dass die Zustände verschiedener Realms disjunkt sind und sich nicht gegenseitig beeinflussen können.

---

## 7. Saga-Pattern

### 7.1 Verteilte Transaktionen

Das Saga-Pattern ermöglicht atomare Operationen über mehrere Realms hinweg. Eine Saga besteht aus einer geordneten Sequenz von Schritten und einer entsprechenden Sequenz von Kompensationsaktionen.

Das Kompensations-Axiom garantiert: Wenn ein Schritt k fehlschlägt, werden alle Kompensationsaktionen für die Schritte 0 bis k-1 ausgeführt. Das resultierende Theorem besagt, dass eine Saga entweder vollständig erfolgreich ist oder vollständig kompensiert wird – partieller Erfolg ist nicht möglich.

---

## 8. Governance-Mechanismen

### 8.1 Abstimmungsmodi

Das System unterstützt verschiedene Governance-Mechanismen:

- **Quadratic Voting**: Die Stimmzahl entspricht der Wurzel der gehaltenen Token. Dies verhindert Plutokratie.
- **Token-basiert**: Ein Token, eine Stimme.
- **Reputation-basiert**: Stimmgewicht basiert auf Trust.
- **Egalitär**: Jede Identität hat eine Stimme.
- **Delegiert**: Stimmen können an Repräsentanten delegiert werden.

### 8.2 Anti-Calcification

Das System überwacht die Verteilung des Trust mittels des Gini-Koeffizienten. Wenn die Ungleichheit einen konfigurierbaren Schwellenwert überschreitet, werden automatisch Umverteilungsmaßnahmen ausgelöst. Dies verhindert die Entstehung einer "Trust-Aristokratie".

---

## 9. Speichersystem

### 9.1 Content-Addressing

Alle Daten im System werden durch ihren Inhalt adressiert: Die ID eines Blobs ist der kryptographische Hash seines Inhalts. Dies hat zwei wichtige Konsequenzen: Erstens können identische Daten automatisch dedupliziert werden. Zweitens kann jeder Teilnehmer die Integrität von Daten unabhängig verifizieren.

### 9.2 Realm-Isolation

Zugriffsrichtlinien für Blobs sind realm-spezifisch und voneinander unabhängig. Ein Blob kann in einem Realm öffentlich zugänglich sein, während er in einem anderen nur für bestimmte Identitäten lesbar ist.

---

## 10. Sicherheitsarchitektur

### 10.1 7-Schichten-Verteidigung

Das System implementiert eine Defense-in-Depth-Strategie mit sieben unabhängigen Verteidigungsebenen:

1. **Gateway-Layer**: Filterung auf Netzwerkebene
2. **Mana-Layer**: Interaktionsratenbegrenzung
3. **Gas-Layer**: Rechenkostenlimitierung
4. **Trust-Layer**: Reputationsbasierte Zugriffskontrolle
5. **Realm-Layer**: Governance-spezifische Regeln
6. **DID-Layer**: Identitätsbasierte Berechtigungen
7. **Protection-Layer**: Kryptographische Absicherung

Die Gesamtverteidigung gegen einen Angriff ist das Produkt der Durchbruchswahrscheinlichkeiten aller sieben Schichten. Dies bedeutet, dass ein Angreifer alle Schichten überwinden muss, um erfolgreich zu sein.

### 10.2 Sandbox-Invariante

Jede Codeausführung findet in einer isolierten Sandbox statt und wird vollständig protokolliert. Dies ermöglicht sowohl Nachvollziehbarkeit als auch forensische Analyse im Fall von Sicherheitsvorfällen.

### 10.3 AI-Trust-Cap

Für KI-Agenten gilt eine spezielle Beschränkung: Ihr Goodwill-Trust kann niemals 80% des Trust-Werts ihres menschlichen Eigners überschreiten. Dies stellt sicher, dass autonome Systeme nie mehr Einfluss haben als ihre menschlichen Auftraggeber.

---

## 11. Zero-Knowledge State Management

### 11.1 Proof-Carrying State

Das System basiert auf dem Paradigma "Proof-Carrying State": Anstatt alle Transaktionshistorien zu speichern, speichert jede Identität nur ihren aktuellen Zustand zusammen mit einem kryptographischen Beweis, dass dieser Zustand korrekt aus einer validen Historie hervorgegangen ist.

Ein Zero-Knowledge-Proof ist ein kryptographisches Argument konstanter Größe (typischerweise 256 Bytes), das in konstanter Zeit verifiziert werden kann. Der State-Bundle jeder Identität besteht aus dem aktuellen Zustand, dem historischen Beweis und einem Anchor für die öffentliche Verankerung.

### 11.2 Rekursive Beweisführung

Das System nutzt rekursive Beweisführung: Jeder Zustandsübergang erzeugt einen neuen Beweis, der sowohl die Validität des vorherigen Beweises als auch die Gültigkeit der aktuellen Transition beweist. Mathematisch gilt: Die Verifikation des neuen Beweises impliziert die Validität aller vorherigen Beweise.

### 11.3 Anchoring und Liveness

Um Rollback-Angriffe zu verhindern, müssen Zustandsbeweise öffentlich verankert werden. Das Anchoring-Axiom verlangt: Ein Anchor-Update muss signiert sein, die Sequenznummer muss strikt steigen, und der zugehörige Beweis muss gültig sein.

Das Liveness-Axiom stellt sicher, dass nur "frische" Zustände akzeptiert werden: Eine Interaktion ist nur zulässig, wenn der Anchor des Gegenübers nicht älter ist als ein konfigurierbares Maximum.

### 11.4 Receipt-Ketten

Trust-Updates erfordern externe Validierung: Eine Identität kann ihren eigenen Trust-Wert nicht unilateral erhöhen. Stattdessen muss jede Trust-Änderung durch ein signiertes Receipt eines Gegenübers bestätigt werden.

Zwei fundamentale Theoreme folgen aus diesem Design: Die State-Selbstsouveränität besagt, dass jede Identität vollständig für die Persistenz ihres eigenen Zustands verantwortlich ist. Das Forgery-Theorem besagt, dass die Fälschung eines Trust-Werts entweder das Brechen der ZK-Kryptographie oder das Fälschen einer Signatur erfordert.

---

## 12. Resilienz-Mechanismen

### 12.1 Anti-Clustering

Das System bekämpft Kartellbildung durch Entropie-Messung: Für jede Identität wird die Entropie ihrer Interaktionspartner berechnet. Wenn eine Identität nur mit wenigen, immer gleichen Partnern interagiert (niedrige Entropie), wird ihr effektiver Trust durch einen Sigmoid-Dämpfungsfaktor reduziert.

Ein daraus folgendes Theorem besagt: Ein Sybil-Cluster mit n Knoten hat einen effektiven Gesamt-Trust von ungefähr τ_node/n, da alle internen Kanten zu niedriger Entropie führen.

### 12.2 Social Recovery

Um den Verlust von Schlüsseln zu kompensieren, implementiert das System Shamir's Secret Sharing: Der Master-Schlüssel wird in n Fragmente aufgeteilt, von denen mindestens m benötigt werden, um den Schlüssel zu rekonstruieren (typische Konfiguration: 3 von 5).

Die Fragmente werden an vertrauenswürdige "Guardian"-Identitäten verteilt. Im Notfall kann ein Quorum von Guardians die Wiederherstellung ermöglichen. Das Resilienz-Theorem berechnet: Die Wahrscheinlichkeit eines permanenten Identitätsverlusts ist das Produkt aus Geräteverlust-Wahrscheinlichkeit und Quorum-Versagens-Wahrscheinlichkeit – bei 3 von 5 Guardians typischerweise unter 10⁻⁵.

### 12.3 Arbitration

Für Streitfälle, die nicht durch Code gelöst werden können, definiert das System ein Arbitrations-Verfahren. Eine Saga kann in den Zustand "Dispute" übergehen, der durch menschliche Juroren aufgelöst wird.

Juroren werden zufällig (basierend auf einem kryptographischen Seed) aus dem Pool von Identitäten mit hohem Integrity-Trust ausgewählt. Der Verlierer eines Disputes trägt alle Kosten und erleidet einen Trust-Penalty.

### 12.4 Homöostatische Regulation

Das System nutzt PID-Regler (Proportional-Integral-Derivative) zur automatischen Anpassung von Systemparametern. Beispielsweise wird der Decay-Faktor λ automatisch angepasst, um den Anteil der Veteranen im System auf einem Zielwert (typisch 5-10%) zu halten.

Ein Kontrolltheorie-Theorem garantiert: Solange die Ressourcen-Algebra monoton ist, konvergiert das System gegen die definierten Zielmetriken.

---

## 13. Object-Chain-Architektur

### 13.1 Souveräne Objekte

Anstatt einer globalen Blockchain führt jedes Asset (Realm, Token, NFT) eine eigene "Micro-Blockchain" – seine Object-Chain. Ein souveränes Objekt besteht aus einer DID, Metadaten, dem aktuellen Zustand und einer Historie aller Zustandsänderungen als DAG.

Das Object-Local-Chain-Axiom garantiert: Jede Transaktion in der Historie muss den Hash der vorherigen Transaktion enthalten und vom aktuellen Controller signiert sein.

### 13.2 Besitz-Transfer

Ein Eigentumswechsel wird als signiertes Event an die Object-Chain angehängt. Die Finality eines Transfers ist erreicht, sobald der neue Head in der DHT (Distributed Hash Table) verankert ist.

### 13.3 Atomic Swaps

Für den gleichzeitigen Austausch von Assets (z.B. Realm gegen Token) definiert das System Dual-Chain-Transaktionen: Eine Swap-Transaktion ist nur gültig, wenn sie von beiden Parteien signiert ist und in die Historien beider beteiligten Objekte aufgenommen wird. Da die Transaktion denselben Hash hat, können die Ketten nicht divergieren.

### 13.4 Fork-Erkennung

Vor einem Kauf prüft ein Client den DHT-Tip gegen die präsentierte Historie. Wenn eine Diskrepanz besteht, wird ein Fork-Alarm ausgelöst. Das Single-Owner-Theorem garantiert: Zu jedem Zeitpunkt existiert maximal ein gültiger Pfad vom Genesis-Block zum aktuellen Anchor.

---

## 14. Netzwerk-Substrat

### 14.1 Trust-basiertes Routing

Das System nutzt eine modifizierte Kademlia-DHT für das Routing. Im Gegensatz zum Standard-Kademlia, das den ältesten Knoten bevorzugt, ersetzt das System bei vollen k-Buckets den Knoten mit dem niedrigsten Reliability-Trust.

Diese Modifikation hat einen emergenten Effekt: Das Netzwerk-Backbone besteht automatisch aus Veteranen, während Angreifer mit niedrigem Trust aus den Routing-Tabellen verdrängt werden.

### 14.2 Trust-Gated Gossip

Für die Nachrichtenverbreitung nutzt das System Gossipsub v1.1 mit Trust-basierter Bewertung. Der Score eines Peers kombiniert Netzwerk-Metriken (Zeit im Netz, erfolgreiche Zustellungen) mit dem Trust-Wert. Peers unter einem Schwellenwert werden sofort getrennt.

### 14.3 Erasure Coding

Anstatt Daten mehrfach zu replizieren, nutzt das System Reed-Solomon-Codierung: Ein Objekt wird in n Fragmente zerlegt, von denen k (k < n) für die Rekonstruktion genügen. Typische Konfiguration: k=10, n=30 bietet dreifache Sicherheit bei nur 1,5-fachem Speicherplatz.

### 14.4 Proof-of-Storage

Speicherknoten müssen periodische Audit-Challenges bestehen: Der Eigentümer sendet einen zufälligen Salt, und der Speicherknoten muss beweisen, dass er das Fragment noch besitzt, indem er den Hash aus Fragment und Salt liefert. Versagen führt zu sofortigem Trust-Penalty.

---

## 15. Zeit-Substrat

### 15.1 Hybrid Logical Clocks

Ohne zentrale Zeitquelle nutzt das System Hybrid Logical Clocks (HLC): Ein Zeitstempel besteht aus physischer Zeit und logischem Zähler. Bei Empfang einer Nachricht wird die physische Zeit auf das Maximum aus lokaler und empfangener Zeit gesetzt. Der logische Zähler inkrementiert bei gleicher physischer Zeit.

Dies ermöglicht kausale Ordnung von Events auch bei leichten Unterschieden in den Systemuhren.

### 15.2 Verifiable Delay Functions

Für kritische Operationen (z.B. Anchor-Updates) nutzt das System Verifiable Delay Functions (VDFs): Die Berechnung dauert garantiert eine bestimmte Zeit und kann nicht parallelisiert werden, während die Verifikation sofort möglich ist. Dies verhindert High-Frequency-Trading-Angriffe auf die DHT.

---

## 16. ZK-Circuit-Architektur

### 16.1 Plonkish Arithmetisierung

Die Zero-Knowledge-Beweise basieren auf Plonkish-Arithmetisierung: Logik wird in polynomielle Constraints übersetzt. Die Trace-Matrix besteht aus Selektor-Spalten (definieren die Operation), Witness-Spalten (private Variablen) und Public-Input-Spalten.

Jede Zeile muss das Standard-Gate erfüllen: q_L·a + q_R·b + q_M·a·b + q_O·c + q_C + pi = 0. Durch geschickte Wahl der Selektoren kann man Addition, Multiplikation und Konstanten ausdrücken.

### 16.2 Asymmetrie als Polynom

Die Trust-Asymmetrie wird durch ein spezielles Polynom erzwungen: Δ_eff - ((1-s)·Δ + s·λ·Δ) = 0, wobei s ein Bit ist, das angibt, ob das Delta negativ ist. Dies zwingt den Beweiser mathematisch, die Strafe anzuwenden.

### 16.3 Range Checks

Um sicherzustellen, dass Trust-Werte im gültigen Bereich bleiben, nutzt das System Lookup-Argumente (Plookup): Es wird bewiesen, dass ein Wert Element einer vordefinierten Tabelle ist, ohne die Tabelle offenzulegen.

### 16.4 Nova Folding

Für effiziente rekursive Beweise nutzt das System Nova: Anstatt vorherige Beweise vollständig zu verifizieren, werden sie "gefaltet" – mathematische Objekte werden linear kombiniert. Die Kosten pro Fold sind O(1) Gruppenoperationen, unabhängig von der Anzahl bisheriger Schritte.

### 16.5 zkWASM-Gadgets

Für die Ausführung beliebigen Codes im ZK-Circuit nutzt das System den Jolt/Lasso-Ansatz: Anstatt komplexe Circuits für jede Instruktion zu bauen, werden riesige Lookup-Tabellen verwendet, die alle möglichen Input-Output-Kombinationen enthalten.

Memory-Konsistenz wird durch Permutations-Argumente garantiert: Alle Speicheroperationen werden sortiert, und es wird geprüft, dass aufeinanderfolgende Lesungen desselben Speichers denselben Wert liefern.

---

## 17. Evolvierbarkeit

### 17.1 Protocol-Manifest

Um Updates ohne Hard-Fork zu ermöglichen, definiert das System ein Protocol-Manifest: Eine signierte Datenstruktur, die die aktuelle Version, gültige Circuits und einen Pointer auf die nächste Version enthält.

Ein Beweis ist nur gültig relativ zu einer aktiven Protokollversion. Alte Versionen bleiben für einen Übergangszeitraum (typisch 6 Monate) gültig, um sanfte Migration zu ermöglichen.

### 17.2 Crypto-Agility

Identitäten sind nicht an einen bestimmten kryptographischen Algorithmus gebunden. Ein DID-Dokument kann mehrere Schlüssel mit verschiedenen Algorithmen enthalten – z.B. Ed25519 für aktuelle Nutzung und Dilithium5 als quantensicherer Standby.

Key-Rotation erfordert eine Signatur mit dem alten Schlüssel. Im Notfall (z.B. wenn Ed25519 gebrochen wird) kann Social Recovery genutzt werden, um auf einen quantensicheren Schlüssel zu wechseln.

### 17.3 Formale Verifikation

Das langfristige Ziel ist die Isomorphie zwischen Spezifikation und Implementierung: Die ZK-Circuits sollen automatisch aus Lean4-Spezifikationen generiert werden, wodurch Implementierungsfehler ausgeschlossen werden.

---

## 18. Hierarchische Identitäten

### 18.1 HD-DID-System

Das System unterstützt hierarchische deterministische Identitäten nach BIP-32/44-Standard. Aus einem Master-Seed können beliebig viele Child-Identitäten abgeleitet werden, organisiert nach Purpose, Realm-Typ, Realm-ID und Index.

### 18.2 Trust-Projektion

Abgeleitete Identitäten können Trust auf zwei Arten nutzen:

- **Public Link**: Die Child-Identität zeigt eine Signatur der Root-Identität. Voller Trust-Transfer, keine Privatsphäre.
- **Private Proof**: Die Child-Identität zeigt einen ZK-Beweis, dass sie zu einer Root-Identität mit Trust über einem Schwellenwert gehört. Maximale Privatsphäre.

### 18.3 Upstream-Penalty

Um Missbrauch zu verhindern, enthält jeder Trust-Projections-Beweis einen Nullifier. Wenn die Child-Identität betrügt, wird der Nullifier verbrannt, was den Trust der Root-Identität reduziert. Verstecken ist möglich, Konsequenzen-Flucht ist unmöglich.

### 18.4 Enterprise-Delegation

Unternehmen können Mitarbeiter-Identitäten mit eingeschränkten Capabilities ableiten (z.B. maximale Ausgaben von 1000€). Die Revokation einer abgeleiteten Identität setzt ihre Capabilities auf null.

---

## 19. Adversarial Resilience

### 19.1 Dual-Verification

Um Circuit-Bugs zu vermeiden, werden kritische Circuits in zwei verschiedenen Sprachen (z.B. Halo2/Rust und Circom/C++) implementiert. Eine Zustandsänderung erfordert gültige Beweise von beiden Systemen. Die Wahrscheinlichkeit, dass beide Compiler denselben Fehler haben, ist vernachlässigbar (~10⁻⁸).

### 19.2 Lighthouse-Audit

Gegen Eclipse-Angriffe fragt ein Client nicht nur seine DHT-Nachbarn, sondern auch zufällig ausgewählte "Lighthouses" (Veteranen mit τ ≥ 0.9). Bei Diskrepanz zwischen lokaler und globaler Antwort wird Alarm ausgelöst und der unehrliche Nachbar verliert Trust.

### 19.3 Governance-Gating

Gegen AI-Sybil-Angriffe trennt das System ökonomischen Trust (wächst durch Transaktionen, AI kann erreichen) von Governance-Trust (erfordert Proof-of-Personhood oder mehrjähriges Token-Locking). Die ökonomische Barriere macht Massenangriffe unprofitabel.

### 19.4 Appeals-Slashing

Gegen Jury-Bestechung implementiert das System ein mehrstufiges Berufungssystem: Wenn ein Urteil in der Berufung umgekehrt wird, verlieren die Juroren der vorherigen Ebene ihren Stake. Die Kosten für Bestechung aller Ebenen übersteigen schnell jeden möglichen Gewinn.

### 19.5 Watchtowers

Gegen "Lazy Verifiers" bezahlt das System Kopfgeld-Jäger: Wer beweisen kann, dass ein Anchor ungültig ist, erhält 50% des Stakes des Erstellers. Dies macht professionelle Netzwerk-Überwachung profitabel.

---

## 20. DSGVO-konforme Datenhaltung

### 20.1 Zero-Data History

Das System nutzt die Folding-Eigenschaft aggressiv für Pruning: Nach jedem Fold werden die Witness-Daten (die eigentlichen Transaktionsdetails) sofort gelöscht. Zurück bleibt nur der mathematische Beweis, dass eine valide Geschichte existierte.

### 20.2 Beweisbarkeit ohne Daten

Ein Veteran mit Trust 0.85 kann beweisen, dass sein Score legitim erworben wurde (der Beweis verifiziert), ohne dass die zugrunde liegenden Transaktionen rekonstruiert werden können. Dies implementiert technisch das "Recht auf Vergessen" nach DSGVO Art. 17.

### 20.3 Retention-Levels

Das System definiert drei Retention-Level:

- **L0 (Immediate-Prune)**: Standard, nur Beweis bleibt
- **L1 (Hot-Window)**: 7 Tage Details für mögliche Disputes
- **L2 (Cold-Archive)**: Optional, nutzer-kontrolliert, verschlüsselt

---

## 21. Technologie-Stack

Die Referenzimplementierung nutzt folgende Kerntechnologien:

**Kryptographie**: BLAKE3 (Hashing), Ed25519 (Signaturen), X25519 + ChaCha20-Poly1305 (Verschlüsselung), WebAuthn (Hardware-Authentifizierung)

**Zero-Knowledge**: Nova (Folding), Halo2 (Circuit-Framework), Circom (Dual-Verification), Groth16 (Finale SNARKs)

**Execution**: Wasmtime (WASM-Runtime), Tokio (Async Runtime), DashMap (Concurrent State)

**Netzwerk**: libp2p (P2P-Stack), Kademlia (DHT), Gossipsub (Message Propagation)

**Persistenz**: RocksDB (lokaler State), FastCDC (Content-Splitting), ZSTD (Kompression)

**Identität**: BIP-32 (HD-Derivation), SLIP-0010 (Key Derivation), Semaphore (ZK-Gruppenbeweise)

---

## 22. Fazit

Projekt Pluto definiert eine vollständige Architektur für dezentrale, vertrauensbasierte digitale Systeme. Die Kombination aus formaler Spezifikation (90 Axiome, 12 Theoreme), kryptographischer Absicherung (Zero-Knowledge-Beweise, Recursive Folding), und ökonomischer Anreizgestaltung (Slashing, Bounties) schafft ein System, dessen Angriff ökonomisch irrational ist.

Die 11-Schichten-Architektur ermöglicht sowohl tiefe technische Optimierung als auch klare Separation of Concerns. Die Post-Quantum-Readiness und formale Verifizierbarkeit stellen langfristige Relevanz sicher.

Das System ist implementation-ready: Die Spezifikation ist ausreichend detailliert für die direkte Übersetzung in Code, und die Technologie-Auswahl berücksichtigt produktionsreife Bibliotheken.

---

**Dokumentstatus**: Final
**Axiome**: 90
**Theoreme**: 12
**Layer**: 11

---

## Appendix A: 11-Schichten-Architektur

| Layer | Name | Technologie | Funktion |
|-------|------|-------------|----------|
| L0 | Kryptographie | Ed25519, Dilithium5, VDF | Signaturen, Post-Quantum |
| L1 | Netzwerk | libp2p, Kademlia | Peer-Discovery, Routing |
| L2 | Speicher | Reed-Solomon | Erasure Coding |
| L3 | Beweise | Nova, Halo2, Groth16 | ZK-Proofs, Folding |
| L4 | Assets | Object-Chains | Souveräne Objekte |
| L5 | Reputation | Trust-Vektor | Vertrauen |
| L6 | Execution | ECLVM, zkWASM | Smart Contracts |
| L7 | Isolation | Realms | Governance-Container |
| L8 | Recovery | Social Recovery | Schlüsselwiederherstellung |
| L9 | Regulation | PID-Controller | Homöostase |
| L10 | Evolution | Protocol-Manifest | Upgrades |

---

## Appendix B: Axiom-Kompendium (90 Axiome)

**Kern (15):** Μ₁ Relationsordnung · Κ₀ Passkey-Primacy · Κ₁ Regel-Monotonie · Κ₂ Trust-Bounded · Κ₆ Key-Control · Κ₇ UID-Immutability · Κ₉ Kausalität · Κ₁₀ Content-Addressing · Κ₁₁ Gas-Monotonie · Κ₂₂ Saga-Kompensation · Κ₂₈ Dispatch-Konsistenz · Κ₂₉ Storage-Integrität · Κ₅₁ Proof-Carrying-State · Κ₅₉ Object-Chain · Κ₆₂ Atomic-Swap

**Trust (12):** Κ₃ Update-Limit (10%) · Κ₄ Asymmetrie (×2) · Κ₈ Delegation-Decay · Κ₁₃ Mana-Regen · Κ₂₃ Cross-Realm-Dämpfung · Κ₂₄ Realm-Lokalität · Κ₅₂ Anchor-Mono · Κ₅₃ Liveness · Κ₅₄ Receipt-Req · Κ₅₅ Anti-Cluster · Κ₅₆ Social-Recovery · Κ₅₈ PID

**Governance (10):** Κ₁₈ Vote-Weight · Κ₁₉ Gini-Trigger · Κ₂₁ Quadratic · Κ₂₅ Sandbox · Κ₂₆ AI-Cap · Κ₃₀ Policy-Isolation · Κ₅₇ Arbitration · Κ₆₀ Transfer · Κ₆₁ DHT-Finality · Κ₆₃ Fork-Detection

**Execution (6):** Κ₃₅ WASM-Determinismus · Κ₃₆ Fuel-Mapping · Κ₆₈ Proof-Folding · Κ₇₁ Standard-Gate · Κ₇₂ Asymmetry-Poly · Κ₇₃ Range-Lookup

**Netzwerk (8):** Κ₆₄ Trust-Routing · Κ₆₅ Gossip-Score · Κ₆₆ Erasure · Κ₆₇ Proof-of-Storage · Κ₆₉ HLC · Κ₇₀ VDF · Κ₇₄ Nova-Fold · Κ₇₆ Memory-Perm

**zkWASM (2):** Κ₇₅ Instruction-Lookup · Κ₇₆ Memory-Check

**Eternity (5):** Κ₈₂ Dynamic-Verify · Κ₈₃ Backward-Window · Κ₈₄ Sig-Abstraction · Κ₈₅ Key-Rotation · Κ₈₆ Spec-Isomorphism

**Fractal (3):** Κ₉₁ Child-Proof · Κ₉₂ Trust-Projection · Κ₉₃ Upstream-Penalty

**Hardening (5):** Κ₉₄ Dual-Verify · Κ₉₅ Lighthouse · Κ₉₆ Gov-Gating · Κ₉₇ Appeals-Slash · Κ₉₈ Watchtowers

**Privacy (1):** Κ₉₉ Aggressive-Pruning

---

## Appendix C: Theorem-Übersicht

| ID | Name | Aussage | Basis |
|----|------|---------|-------|
| TH₁ | Sybil-Resistenz | Legitime 5× schneller als Angreifer | Κ₂,Κ₃,Κ₄ |
| TH₂ | Attacker-Exhaust | Angreifer-Ressourcen → 0 | Κ₁₁,Κ₁₃ |
| TH₃ | Trust-Emergenz | Positiver Feedback für Legitime | Κ₃,Κ₄ |
| TH₄ | Saga-Safety | Vollständig oder kompensiert | Κ₂₂ |
| TH₅ | Realm-Isolation | Zustände disjunkt | Κ₂₄,Κ₃₀ |
| TH₆ | Event-DAG | Azyklischer Graph | Κ₉ |
| TH₇ | State-Sovereignty | Selbst-Verantwortung | Κ₅₁,Κ₅₄ |
| TH₈ | Forgery-Impossible | Erfordert Krypto-Break | Κ₅₁,Κ₅₄ |
| TH₉ | Cluster-Inefficiency | τ_cluster ≈ τ_node/n | Κ₅₅ |
| TH₁₀ | Key-Resilience | P(Verlust) ≈ 10⁻⁵ | Κ₅₆ |
| TH₁₁ | Equilibrium | Konvergenz zu Zielen | Κ₅₈ |
| TH₁₂ | Single-Owner | Ein Pfad pro Objekt | Κ₅₂,Κ₅₉,Κ₆₀ |

---

## Appendix D: Lean4-Typen

```lean
structure Trust (n : Nat) (h : 4 ≤ n ∧ n ≤ 6) where
  values : Fin n → Fin 101

def asymUpdate (Δ : Int) (λ : Rat) : Int :=
  if Δ < 0 then (λ * Δ.toRat).floor.toInt else Δ

theorem trust_bounded (τ : Trust n h) (d : Fin n) : τ.values d ≤ 100 := Fin.is_le _

inductive SagaResult | success | failed (step comp : Nat)
def SagaResult.safe : SagaResult → Prop | .success => True | .failed k c => c = k
```

---

## Appendix E: Abhängigkeitsgraph

```
Κ₀ ─┬─ Κ₆ ── Κ₇
    ├─ Κ₂ ─┬─ Κ₃ ── Κ₄ ── TH₁
    │      ├─ Κ₁₁ ────── TH₂
    │      └─ Κ₁₃ ────── TH₃
    └─ Κ₉₁ ─┬─ Κ₉₂
            └─ Κ₉₃

Κ₁ ── Κ₂₄ ─┬─ Κ₂₃
           └─ TH₅

Κ₅₁ ─┬─ TH₇
     └─ TH₈

Κ₅₅ ── TH₉   Κ₅₆ ── TH₁₀   Κ₅₈ ── TH₁₁

Κ₅₂ ∧ Κ₅₉ ∧ Κ₆₀ ── TH₁₂
```

---

*Fachkonzept basiert auf λ-𝕌ₚ v15.0*
