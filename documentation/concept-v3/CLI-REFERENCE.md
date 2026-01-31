# Erynoa CLI Reference V6.0

> Vollständige Befehlsreferenz für das Erynoa-Protokoll
> Basierend auf Weltformel V6.0 mit 126 Axiomen über 8 Ebenen (inkl. 6 Peer-Axiome)
> Humanistisch • Antifragil • Verhältnismäßig

---

## Schnellübersicht

```
IDENTITÄT          DATEN              EVENTS             TRANSAKTIONEN
─────────────────────────────────────────────────────────────────────────
init               add                commit             seek
sub-identity       stage              push               propose
key                unstage            pull               agree
recover            rm                 status             stream
export                                log                close
                                      diff               abort
                                      show               dispute

SHARDS             WITNESS            GOVERNANCE         DIAGNOSE
─────────────────────────────────────────────────────────────────────────
shard              witness            governance         inspect
merge              request-witness    vote               verify
bridge             attestations       veto               blame
                   verify             delegate           bisect
                                                         audit

KONFIGURATION      NETZWERK           CREDENTIALS        ASSETS
─────────────────────────────────────────────────────────────────────────
config             remote             credential         mint
profile            sync               revoke             burn
alias              peers              present            transfer
                   validators         verify-credential  balance

HUMANISMUS (V6.0)                                        ← NEU
─────────────────────────────────────────────────────────────────────────
human-auth         lod                amnesty            blueprint
  verify             compute            status             validate
  request            auto               apply              nld
  quota              green-score        request            equivalence
```

---

## 1. Identitäts-Befehle

### `erynoa init`

Erstellt eine neue Identität (DID) im System.

```bash
erynoa init [OPTIONS]

OPTIONS:
    --namespace <NS>        Namespace (default: personal)
                            Werte: personal, business, service, validator
    --algorithm <ALG>       Kryptographischer Algorithmus
                            Werte: ed25519 (default), secp256k1, bls12-381
    --label <LABEL>         Menschenlesbares Label
    --recover <SEED>        Aus Seed-Phrase wiederherstellen

BEISPIELE:
    erynoa init
    erynoa init --namespace business --label "Meine Firma GmbH"
    erynoa init --algorithm secp256k1
    erynoa init --recover "word1 word2 word3 ... word24"

AXIOM-REFERENZ: A1-A5 (Identität), Q1 (Quanten-Zustand)
```

### `erynoa sub-identity`

Verwaltet verschränkte Sub-Identitäten.

```bash
erynoa sub-identity <COMMAND> [OPTIONS]

COMMANDS:
    create <NAME>           Neue Sub-Identität erstellen
    list                    Alle Sub-Identitäten auflisten
    switch <NAME>           Zu Sub-Identität wechseln
    delete <NAME>           Sub-Identität löschen
    link <DID>              Externe DID als Sub-Identität verknüpfen

OPTIONS (create):
    --inherit-trust <F>     Trust-Vererbungsfaktor (0.0-1.0, default: 0.5)
    --context <SHARD>       Kontext-Beschränkung
    --permissions <PERMS>   Erlaubte Aktionen (comma-separated)
                            Werte: transfer, attest, claim, governance, all

BEISPIELE:
    erynoa sub-identity create gaming --inherit-trust 0.7
    erynoa sub-identity create work --context business-services --permissions transfer,attest
    erynoa sub-identity list
    erynoa sub-identity switch gaming

AXIOM-REFERENZ: A4 (Sub-Identitäten), Q3 (Verschränkung)
```

### `erynoa key`

Verwaltet kryptographische Schlüssel.

```bash
erynoa key <COMMAND> [OPTIONS]

COMMANDS:
    list                    Alle Schlüssel auflisten
    generate                Neuen Schlüssel generieren
    rotate                  Schlüssel rotieren
    revoke <KEY_ID>         Schlüssel widerrufen
    export <KEY_ID>         Öffentlichen Schlüssel exportieren
    import <FILE>           Schlüssel importieren

OPTIONS:
    --algorithm <ALG>       Algorithmus für generate
    --purpose <PURPOSE>     Verwendungszweck
                            Werte: primary, signing, encryption, recovery

BEISPIELE:
    erynoa key list
    erynoa key generate --algorithm ed25519 --purpose signing
    erynoa key rotate --purpose primary
    erynoa key export primary > my-public-key.pem

AXIOM-REFERENZ: A3 (Schlüssel-Binding)
```

### `erynoa recover`

Stellt Identität aus Backup wieder her.

```bash
erynoa recover [OPTIONS]

OPTIONS:
    --seed <SEED>           24-Wort Seed-Phrase
    --file <FILE>           Backup-Datei
    --verify-only           Nur verifizieren, nicht wiederherstellen

BEISPIELE:
    erynoa recover --seed "word1 word2 ... word24"
    erynoa recover --file backup.enc --verify-only
```

### `erynoa export`

Exportiert Identitätsdaten.

```bash
erynoa export [OPTIONS]

OPTIONS:
    --format <FMT>          Ausgabeformat (json, cbor, did-document)
    --include-private       Private Schlüssel einschließen (⚠️ Vorsicht!)
    --output <FILE>         Ausgabedatei

BEISPIELE:
    erynoa export --format did-document > my-did.json
    erynoa export --include-private --output backup.enc
```

---

## 2. Daten-Befehle

### `erynoa add`

Fügt Daten zum Staging-Bereich hinzu.

```bash
erynoa add <FILE|DIR> [OPTIONS]

OPTIONS:
    --type <TYPE>           Datentyp
                            Werte: asset, credential, claim, service, raw
    --schema <SCHEMA>       Schema-Referenz für Validierung
    --private               Nicht öffentlich speichern
    --zkp                   Zero-Knowledge-Proof generieren
    --encrypt <DID>         Für spezifische DID verschlüsseln

BEISPIELE:
    erynoa add invoice.json --type asset --schema "amo:finance:invoice:v2"
    erynoa add certificate.json --type credential
    erynoa add ./contracts/ --type asset
    erynoa add sensitive.json --private --encrypt did:erynoa:biz:partner

AXIOM-REFERENZ: Q11-Q13 (Embeddings, Validierung), O1-O5 (AMOs)

OUTPUT:
    - Schema-Compliance (Ω_soft)
    - Semantisches Embedding
    - Axiom-Analyse
```

### `erynoa stage`

Zeigt oder verwaltet den Staging-Bereich.

```bash
erynoa stage [COMMAND]

COMMANDS:
    (ohne)                  Staging-Status anzeigen
    list                    Alle staged Items auflisten
    clear                   Staging-Bereich leeren

BEISPIELE:
    erynoa stage
    erynoa stage list
    erynoa stage clear
```

### `erynoa unstage`

Entfernt Daten aus dem Staging-Bereich.

```bash
erynoa unstage <DATUM_ID|FILE>

BEISPIELE:
    erynoa unstage datum:sha3:abc123...
    erynoa unstage invoice.json
```

### `erynoa rm`

Entfernt lokale Daten.

```bash
erynoa rm <DATUM_ID> [OPTIONS]

OPTIONS:
    --force                 Ohne Bestätigung löschen
    --keep-references       Referenzen behalten

BEISPIELE:
    erynoa rm datum:sha3:abc123...
    erynoa rm datum:sha3:abc123... --force
```

---

## 3. Event-Befehle

### `erynoa commit`

Erstellt ein neues Event aus staged Daten.

```bash
erynoa commit [OPTIONS]

OPTIONS:
    --type <TYPE>           Event-Typ (auto-detect wenn nicht angegeben)
                            Werte: transfer, mint, burn, claim, attest,
                                   credential_issue, credential_revoke,
                                   proposal, vote
    --message <MSG>         Beschreibung
    -m <MSG>                Kurzform für --message
    --parents <EVENTS>      Explizite Parent-Events (comma-separated)
    --no-auto-witness       Keine automatische Witness-Anfrage

BEISPIELE:
    erynoa commit -m "Monatliche Energielieferung"
    erynoa commit --type transfer --message "Zahlung Q1 2026"
    erynoa commit --type claim -m "Verfügbarkeitsupdate"

AXIOM-REFERENZ: P1-P6 (Prozesse), A12-A17 (Kausalität)

OUTPUT:
    - Trust-Berechnung (Δ𝕎, ℕ, 𝔼xp)
    - Weltformel-Impact (Δ𝔼)
    - Event-ID
```

### `erynoa push`

Propagiert lokale Events ins Netzwerk.

```bash
erynoa push [OPTIONS]

OPTIONS:
    --shard <SHARD>         Ziel-Shard (default: aktueller)
    --priority <PRIO>       Priorität (low, normal, high)
    --wait                  Auf Finalität warten
    --timeout <SECS>        Timeout für --wait (default: 60)

BEISPIELE:
    erynoa push
    erynoa push --shard energy-trading --priority high
    erynoa push --wait --timeout 120

AXIOM-REFERENZ: E11-E15 (Konsens)

OUTPUT:
    - Validator-Responses
    - Konsens-Analyse
    - Finalitäts-Status
    - Merkle-Verankerung
```

### `erynoa pull`

Synchronisiert lokalen Zustand mit dem Netzwerk.

```bash
erynoa pull [OPTIONS]

OPTIONS:
    --shard <SHARD>         Quell-Shard (default: aktueller)
    --since <EVENT>         Nur Events seit diesem Event
    --depth <N>             Maximale Tiefe im DAG
    --verify                Alle Merkle-Proofs verifizieren

BEISPIELE:
    erynoa pull
    erynoa pull --shard finance --verify
    erynoa pull --since event:sha3:abc123... --depth 100
```

### `erynoa status`

Zeigt vollständigen Weltformel-Zustand.

```bash
erynoa status [OPTIONS]

OPTIONS:
    --full                  Vollständige Analyse (default)
    --brief                 Kurzfassung
    --json                  JSON-Ausgabe
    --component <COMP>      Nur spezifische Komponente
                            Werte: psi, W, A, C, N, Exp, all

BEISPIELE:
    erynoa status
    erynoa status --brief
    erynoa status --component psi
    erynoa status --json > status.json

OUTPUT:
    - Quanten-Zustand |Ψ⟩
    - Wächter-Metrik 𝕎 (6 Dimensionen)
    - Aktivität 𝔸
    - Geschichte |ℂ|
    - Novelty ℕ
    - Expectation 𝔼xp
    - Beitrag zur Weltformel 𝔼_you
```

### `erynoa log`

Zeigt Event-Historie.

```bash
erynoa log [OPTIONS]

OPTIONS:
    --limit <N>             Anzahl Events (default: 10)
    --all                   Alle Events
    --type <TYPE>           Nach Event-Typ filtern
    --since <DATE>          Seit Datum
    --until <DATE>          Bis Datum
    --oneline               Einzeilige Ausgabe
    --graph                 DAG-Visualisierung

BEISPIELE:
    erynoa log
    erynoa log --limit 50 --type transfer
    erynoa log --since 2026-01-01 --oneline
    erynoa log --graph --limit 20
```

### `erynoa diff`

Zeigt Unterschiede zwischen Zuständen.

```bash
erynoa diff <EVENT1>..<EVENT2> [OPTIONS]
erynoa diff <EVENT> [OPTIONS]

OPTIONS:
    --stat                  Nur Statistiken
    --trust                 Nur Trust-Änderungen
    --assets                Nur Asset-Änderungen

BEISPIELE:
    erynoa diff event:sha3:abc...event:sha3:def
    erynoa diff HEAD~5..HEAD
    erynoa diff event:sha3:abc --trust
```

### `erynoa show`

Zeigt Details eines Events oder Datums.

```bash
erynoa show <ID> [OPTIONS]

OPTIONS:
    --format <FMT>          Ausgabeformat (human, json, cbor)
    --verify                Signaturen und Proofs verifizieren
    --expand                Referenzierte Objekte einbetten

BEISPIELE:
    erynoa show event:sha3:abc123
    erynoa show datum:sha3:def456 --format json
    erynoa show event:sha3:abc123 --verify --expand
```

---

## 4. Transaktions-Befehle

### `erynoa seek`

Sucht nach Transaktionspartnern.

```bash
erynoa seek <QUERY> [OPTIONS]

OPTIONS:
    --type <TYPE>           Partner-Typ
    --location <LOC>        Geografische Einschränkung
    --min-trust <T>         Minimaler Trust (default: 0.5)
    --max-results <N>       Maximale Ergebnisse (default: 10)
    --include-emerging      Auch FRESH/EMERGING Tiers
    --sort <FIELD>          Sortierung (score, trust, novelty, relevance)

BEISPIELE:
    erynoa seek "renewable energy supplier"
    erynoa seek "software developer" --type freelancer --min-trust 0.7
    erynoa seek "logistics" --location "Berlin, 50km" --max-results 20

AXIOM-REFERENZ: Q5 (Interaktions-Wahrscheinlichkeit), S1-S4 (Anti-Calcification)

OUTPUT:
    - Kandidaten mit Quanten-Analyse
    - P(success) für jeden Kandidaten
    - Diversity-Slots markiert
```

### `erynoa propose`

Erstellt ein Transaktionsangebot.

```bash
erynoa propose <TARGET_DID> [OPTIONS]

OPTIONS:
    --amount <AMT>          Menge/Betrag
    --asset <ASSET>         Asset-Typ
    --price <PRICE>         Preis
    --duration <DUR>        Laufzeit (z.B. "30d", "6h", "1y")
    --streaming             Streaming-Modus aktivieren
    --escrow <DID>          Escrow-Service
    --expires <DUR>         Ablaufzeit des Angebots (default: 7d)
    --message <MSG>         Nachricht an Empfänger

BEISPIELE:
    erynoa propose did:erynoa:biz:supplier --amount "500 kWh" --price "125 EUR" --duration 30d --streaming
    erynoa propose did:erynoa:personal:freelancer --amount "40h" --price "4000 EUR" --escrow did:erynoa:service:escrow

AXIOM-REFERENZ: T1-T3 (SEEK, PROPOSE, AGREE), Q5 (Erfolgswahrscheinlichkeit)

OUTPUT:
    - Erfolgswahrscheinlichkeit P(accept)
    - Generierter Smart Contract
    - Logic Guards
```

### `erynoa agree`

Reagiert auf ein Transaktionsangebot.

```bash
erynoa agree <PROPOSAL_ID> [OPTIONS]

OPTIONS:
    --accept                Angebot akzeptieren
    --reject                Angebot ablehnen
    --counter <TERMS>       Gegenangebot machen
    --counter-price <P>     Nur Preis ändern
    --counter-duration <D>  Nur Dauer ändern
    --message <MSG>         Nachricht

BEISPIELE:
    erynoa agree proposal:sha3:abc --accept
    erynoa agree proposal:sha3:abc --reject --message "Preis zu hoch"
    erynoa agree proposal:sha3:abc --counter-price "110 EUR"

AXIOM-REFERENZ: T3 (AGREE)
```

### `erynoa stream`

Verwaltet laufende Streaming-Transaktionen.

```bash
erynoa stream <COMMAND> <CONTRACT_ID> [OPTIONS]

COMMANDS:
    status                  Status anzeigen
    pause                   Pausieren (benötigt Zustimmung)
    resume                  Fortsetzen
    abort                   Abbrechen mit Settlement
    extend                  Verlängern (benötigt Verhandlung)

OPTIONS (abort):
    --reason <REASON>       Abbruchgrund
                            Werte: buyer-request, seller-failure, mutual, force-majeure

OPTIONS (extend):
    --duration <DUR>        Zusätzliche Dauer
    --amount <AMT>          Zusätzliche Menge

BEISPIELE:
    erynoa stream status contract:sha3:abc
    erynoa stream pause contract:sha3:abc
    erynoa stream abort contract:sha3:abc --reason seller-failure
    erynoa stream extend contract:sha3:abc --duration 15d --amount "250 kWh"

AXIOM-REFERENZ: T4-T7 (STREAM, CLOSE, ATTEST, ABORT)

OUTPUT (status):
    - Lieferungs-/Zahlungsfortschritt
    - Trust-Evolution beider Parteien
    - Abort-Szenario-Analyse
    - Projektion bis Abschluss
```

### `erynoa close`

Schließt eine Transaktion ab.

```bash
erynoa close <CONTRACT_ID> [OPTIONS]

OPTIONS:
    --rating <1-5>          Bewertung des Partners
    --comment <TEXT>        Kommentar zur Transaktion
    --dispute               Dispute eröffnen statt schließen

BEISPIELE:
    erynoa close contract:sha3:abc --rating 5 --comment "Exzellente Zusammenarbeit"
    erynoa close contract:sha3:abc --dispute

AXIOM-REFERENZ: T5 (CLOSE), T6 (ATTEST)
```

### `erynoa abort`

Bricht eine Transaktion ab.

```bash
erynoa abort <CONTRACT_ID> [OPTIONS]

OPTIONS:
    --reason <REASON>       Abbruchgrund (required)
    --evidence <FILE>       Beweismaterial
    --force                 Ohne Bestätigung

BEISPIELE:
    erynoa abort contract:sha3:abc --reason non-delivery
    erynoa abort contract:sha3:abc --reason quality-issue --evidence photos.zip

AXIOM-REFERENZ: T7 (ABORT), A24 (Fair Settlement)
```

### `erynoa dispute`

Eröffnet oder verwaltet einen Dispute.

```bash
erynoa dispute <COMMAND> [OPTIONS]

COMMANDS:
    open <CONTRACT_ID>      Dispute eröffnen
    respond <DISPUTE_ID>    Auf Dispute antworten
    evidence <DISPUTE_ID>   Beweise einreichen
    list                    Offene Disputes auflisten
    status <DISPUTE_ID>     Dispute-Status

OPTIONS (open):
    --type <TYPE>           Dispute-Typ
                            Werte: non-delivery, quality, payment, other
    --description <TEXT>    Beschreibung
    --evidence <FILE>       Beweismaterial

BEISPIELE:
    erynoa dispute open contract:sha3:abc --type quality --description "Nur 80% der vereinbarten Qualität"
    erynoa dispute respond dispute:sha3:xyz --message "Dokumentation beigefügt"
    erynoa dispute evidence dispute:sha3:xyz --file delivery-proof.pdf

AXIOM-REFERENZ: S9-S12 (Quality-Objectivity)
```

---

## 5. Shard-Befehle

### `erynoa shard`

Verwaltet Shards (Kontext-Partitionen).

```bash
erynoa shard <COMMAND> [OPTIONS]

COMMANDS:
    list                    Verfügbare Shards auflisten
    current                 Aktuellen Shard anzeigen
    switch <SHARD>          Zu Shard wechseln
    create <NAME>           Neuen Shard erstellen
    info <SHARD>            Shard-Details anzeigen
    join <SHARD>            Shard beitreten
    leave <SHARD>           Shard verlassen

OPTIONS (create):
    --parent <SHARD>        Parent-Shard
    --rules <FILE>          Regel-Datei
    --description <TEXT>    Beschreibung

OPTIONS (switch):
    --create                Erstellen falls nicht existiert

BEISPIELE:
    erynoa shard list
    erynoa shard switch energy-trading
    erynoa shard create gaming --parent entertainment --description "Gaming marketplace"
    erynoa shard info finance

AXIOM-REFERENZ: A18-A22 (Realms), Q6 (Kategorien)
```

### `erynoa merge`

Führt Cross-Shard-Transaktionen durch.

```bash
erynoa merge [OPTIONS]

OPTIONS:
    --from <SHARD>          Quell-Shard (default: aktueller)
    --to <SHARD>            Ziel-Shard (required)
    --amount <AMT>          Zu übertragende Menge
    --asset <ASSET>         Asset-Typ
    --functor <F>           Spezifischer Funktor (auto-detect wenn nicht angegeben)
    --dry-run               Nur simulieren

BEISPIELE:
    erynoa merge --from gaming --to finance --amount "1000 tokens"
    erynoa merge --to logistics --asset "delivery-contract" --dry-run

AXIOM-REFERENZ: Q7-Q10 (Funktoren, Interoperabilität)

OUTPUT:
    - Funktor-Analyse
    - Konversions-Details
    - Zwei-Phasen-Commit Status
    - Trust-Propagation
```

### `erynoa bridge`

Verwaltet Realm-Brücken.

```bash
erynoa bridge <COMMAND> [OPTIONS]

COMMANDS:
    list                    Verfügbare Brücken auflisten
    info <BRIDGE>           Brücken-Details
    status <BRIDGE>         Brücken-Status
    use <BRIDGE>            Brücke für Transfer nutzen

BEISPIELE:
    erynoa bridge list
    erynoa bridge info ethereum-bridge
    erynoa bridge use polygon-bridge --amount "100 USDC"
```

---

## 6. Witness-Befehle

### `erynoa witness`

Bezeugt ein Event als Wächter.

```bash
erynoa witness <EVENT_ID> [OPTIONS]

OPTIONS:
    --comment <TEXT>        Kommentar zur Bezeugung
    --reject                Event ablehnen (mit Begründung)
    --reason <REASON>       Ablehnungsgrund

BEISPIELE:
    erynoa witness event:sha3:abc
    erynoa witness event:sha3:abc --comment "Verifiziert via Blockchain-Check"
    erynoa witness event:sha3:abc --reject --reason "Inkonsistente Zeitstempel"

AXIOM-REFERENZ: E5-E10 (Wächter)

OUTPUT:
    - Validierungs-Checks
    - Attestation-Details
    - Vigilance-Bonus
```

### `erynoa request-witness`

Fordert Bezeugungen für ein Event an.

```bash
erynoa request-witness <EVENT_ID> [OPTIONS]

OPTIONS:
    --min-witnesses <N>     Minimale Anzahl (default: 3)
    --min-weight <W>        Minimales kombiniertes Gewicht (default: 0.8)
    --validators <DIDs>     Spezifische Validatoren anfragen
    --priority <PRIO>       Priorität (low, normal, high)

BEISPIELE:
    erynoa request-witness event:sha3:abc
    erynoa request-witness event:sha3:abc --min-witnesses 5 --priority high
```

### `erynoa attestations`

Zeigt Attestationen für ein Event.

```bash
erynoa attestations <EVENT_ID> [OPTIONS]

OPTIONS:
    --pending               Nur ausstehende
    --verified              Nur verifizierte
    --json                  JSON-Ausgabe

BEISPIELE:
    erynoa attestations event:sha3:abc
    erynoa attestations event:sha3:abc --pending
```

### `erynoa verify`

Verifiziert ein Event oder Datum.

```bash
erynoa verify <ID> [OPTIONS]

OPTIONS:
    --deep                  Tiefe Verifikation (alle Referenzen)
    --merkle                Merkle-Proof verifizieren
    --signatures            Alle Signaturen prüfen
    --full                  Alle Prüfungen (default)

BEISPIELE:
    erynoa verify event:sha3:abc
    erynoa verify datum:sha3:def --merkle
    erynoa verify event:sha3:abc --deep

OUTPUT:
    - Merkle-Proof-Validierung
    - Signatur-Verifikation
    - Attestations-Analyse
    - Finalitäts-Status
```

---

## 7. Governance-Befehle

### `erynoa governance`

Verwaltet Governance-Aktionen.

```bash
erynoa governance <COMMAND> [OPTIONS]

COMMANDS:
    propose                 Vorschlag einreichen
    list                    Aktive Vorschläge auflisten
    show <PROPOSAL_ID>      Vorschlag-Details
    vote <PROPOSAL_ID>      Abstimmen
    veto <PROPOSAL_ID>      Veto einlegen
    delegate <DID>          Stimmrecht delegieren

OPTIONS (propose):
    --type <TYPE>           Vorschlagstyp
                            Werte: rule-change, parameter, membership, structural
    --title <TITLE>         Titel
    --description <TEXT>    Beschreibung
    --file <FILE>           Vorschlags-Datei
    --shard <SHARD>         Betroffener Shard

OPTIONS (vote):
    --support               Unterstützen
    --oppose                Ablehnen
    --abstain               Enthalten
    --comment <TEXT>        Kommentar

OPTIONS (veto):
    --reason <REASON>       Veto-Begründung (required)
    --minority <GROUP>      Minderheitsgruppe

BEISPIELE:
    erynoa governance propose --type rule-change --title "Trust-Schwelle erhöhen" --file proposal.md
    erynoa governance list --shard energy-trading
    erynoa governance vote proposal:sha3:abc --support --comment "Gute Idee"
    erynoa governance veto proposal:sha3:abc --reason "Schadet Newcomern" --minority fresh-tier
    erynoa governance delegate did:erynoa:expert:governance

AXIOM-REFERENZ: S13-S18 (Fair-Governance)

OUTPUT (vote):
    - Dein Stimmgewicht (quadratisch + domain-spezifisch)
    - Aktueller Abstimmungsstand
    - Quorum-Status
```

### `erynoa vote`

Kurzform für Abstimmung.

```bash
erynoa vote <PROPOSAL_ID> <support|oppose|abstain> [--comment <TEXT>]

BEISPIELE:
    erynoa vote proposal:sha3:abc support
    erynoa vote proposal:sha3:abc oppose --comment "Zu riskant"
```

### `erynoa veto`

Kurzform für Veto.

```bash
erynoa veto <PROPOSAL_ID> --reason <REASON>

BEISPIELE:
    erynoa veto proposal:sha3:abc --reason "Verletzt Axiom A7"
```

### `erynoa delegate`

Delegiert Stimmrecht.

```bash
erynoa delegate <DID> [OPTIONS]

OPTIONS:
    --shard <SHARD>         Nur für spezifischen Shard
    --duration <DUR>        Dauer der Delegation
    --revoke                Delegation widerrufen

BEISPIELE:
    erynoa delegate did:erynoa:expert:energy --shard energy-trading
    erynoa delegate did:erynoa:personal:trusted --duration 30d
    erynoa delegate did:erynoa:expert:energy --revoke
```

---

## 8. Diagnose-Befehle

### `erynoa inspect`

Inspiziert Objekte detailliert.

```bash
erynoa inspect <ID> [OPTIONS]

OPTIONS:
    --trust                 Trust-Details
    --quantum               Quanten-Zustand
    --history               Vollständige Historie
    --relations             Beziehungen zu anderen Objekten
    --all                   Alles

BEISPIELE:
    erynoa inspect did:erynoa:biz:partner --trust --quantum
    erynoa inspect event:sha3:abc --history
    erynoa inspect contract:sha3:xyz --all
```

### `erynoa blame`

Ermittelt Herkunft von Daten.

```bash
erynoa blame <ID> [OPTIONS]

OPTIONS:
    --depth <N>             Maximale Tiefe
    --format <FMT>          Ausgabeformat (human, json, graph)

BEISPIELE:
    erynoa blame datum:sha3:abc
    erynoa blame event:sha3:def --depth 10 --format graph
```

### `erynoa bisect`

Binäre Suche nach problematischen Events.

```bash
erynoa bisect <COMMAND> [OPTIONS]

COMMANDS:
    start                   Bisect starten
    good <EVENT>            Event als gut markieren
    bad <EVENT>             Event als schlecht markieren
    reset                   Bisect abbrechen
    log                     Bisect-Historie

BEISPIELE:
    erynoa bisect start
    erynoa bisect bad event:sha3:current
    erynoa bisect good event:sha3:old
    # System führt durch die Suche
```

### `erynoa audit`

Führt Sicherheits-Audit durch.

```bash
erynoa audit [OPTIONS]

OPTIONS:
    --scope <SCOPE>         Audit-Scope
                            Werte: identity, transactions, trust, all
    --since <DATE>          Seit Datum
    --output <FILE>         Report-Datei
    --format <FMT>          Report-Format (human, json, pdf)

BEISPIELE:
    erynoa audit --scope all
    erynoa audit --scope transactions --since 2026-01-01 --output audit-q1.pdf
```

---

## 9. Credential-Befehle

### `erynoa credential`

Verwaltet Verifiable Credentials.

```bash
erynoa credential <COMMAND> [OPTIONS]

COMMANDS:
    issue                   Credential ausstellen
    list                    Eigene Credentials auflisten
    show <ID>               Credential-Details
    present <ID>            Credential präsentieren
    verify <ID>             Credential verifizieren
    revoke <ID>             Credential widerrufen

OPTIONS (issue):
    --type <TYPE>           Credential-Typ
    --subject <DID>         Subjekt-DID
    --claims <JSON>         Claims als JSON
    --expires <DATE>        Ablaufdatum
    --schema <SCHEMA>       Schema-Referenz

OPTIONS (present):
    --to <DID>              Empfänger
    --selective <FIELDS>    Selektive Offenlegung (comma-separated)
    --zkp                   Zero-Knowledge-Präsentation

BEISPIELE:
    erynoa credential issue --type certification --subject did:erynoa:personal:alice --claims '{"level":"expert","domain":"energy"}'
    erynoa credential list
    erynoa credential present credential:sha3:abc --to did:erynoa:biz:employer --selective "name,certification"
    erynoa credential verify credential:sha3:abc

AXIOM-REFERENZ: C1-C4 (Credentials)
```

### `erynoa revoke`

Widerruft ein Credential.

```bash
erynoa revoke <CREDENTIAL_ID> [OPTIONS]

OPTIONS:
    --reason <REASON>       Widerrufsgrund

BEISPIELE:
    erynoa revoke credential:sha3:abc --reason "Zertifizierung abgelaufen"
```

### `erynoa present`

Präsentiert ein Credential.

```bash
erynoa present <CREDENTIAL_ID> --to <DID> [OPTIONS]

OPTIONS:
    --selective <FIELDS>    Nur bestimmte Felder (comma-separated)
    --zkp                   Zero-Knowledge-Proof
    --challenge <NONCE>     Challenge-Response

BEISPIELE:
    erynoa present credential:sha3:abc --to did:erynoa:biz:verifier
    erynoa present credential:sha3:abc --to did:erynoa:biz:employer --selective "age_over_18" --zkp
```

### `erynoa verify-credential`

Verifiziert ein empfangenes Credential.

```bash
erynoa verify-credential <CREDENTIAL_ID|FILE> [OPTIONS]

OPTIONS:
    --check-revocation      Widerrufsstatus prüfen
    --check-issuer          Issuer-Trust prüfen
    --full                  Vollständige Prüfung

BEISPIELE:
    erynoa verify-credential credential:sha3:abc --full
    erynoa verify-credential ./received-credential.json
```

---

## 10. Asset-Befehle

### `erynoa mint`

Erstellt ein neues AMO (Asset).

```bash
erynoa mint [OPTIONS]

OPTIONS:
    --type <TYPE>           Asset-Typ
    --blueprint <BP>        Blueprint-Referenz
    --amount <AMT>          Menge (für fungible)
    --metadata <JSON>       Metadaten
    --owner <DID>           Initialer Besitzer (default: self)

BEISPIELE:
    erynoa mint --type energy-certificate --blueprint bp:energy:renewable --amount "1000 kWh" --metadata '{"source":"solar","location":"Berlin"}'
    erynoa mint --type nft --blueprint bp:art:digital --metadata '{"title":"Artwork #1"}'

AXIOM-REFERENZ: O1-O5 (AMOs)
```

### `erynoa burn`

Zerstört ein AMO.

```bash
erynoa burn <AMO_ID> [OPTIONS]

OPTIONS:
    --amount <AMT>          Menge (für fungible, default: all)
    --reason <REASON>       Grund

BEISPIELE:
    erynoa burn amo:sha3:abc
    erynoa burn amo:sha3:abc --amount "500 kWh" --reason "Verbraucht"
```

### `erynoa transfer`

Transferiert ein Asset.

```bash
erynoa transfer <AMO_ID> --to <DID> [OPTIONS]

OPTIONS:
    --amount <AMT>          Menge (für fungible)
    --message <MSG>         Nachricht
    --condition <COND>      Bedingte Übertragung

BEISPIELE:
    erynoa transfer amo:sha3:abc --to did:erynoa:personal:bob
    erynoa transfer amo:sha3:abc --to did:erynoa:biz:company --amount "100 units"
```

### `erynoa balance`

Zeigt Asset-Balancen.

```bash
erynoa balance [OPTIONS]

OPTIONS:
    --type <TYPE>           Nur bestimmter Typ
    --shard <SHARD>         Nur bestimmter Shard
    --detailed              Detaillierte Auflistung
    --json                  JSON-Ausgabe

BEISPIELE:
    erynoa balance
    erynoa balance --type energy --detailed
    erynoa balance --shard gaming --json
```

---

## 11. Konfigurations-Befehle

### `erynoa config`

Verwaltet Konfiguration.

```bash
erynoa config <COMMAND> [OPTIONS]

COMMANDS:
    list                    Alle Einstellungen auflisten
    get <KEY>               Wert abrufen
    set <KEY> <VALUE>       Wert setzen
    unset <KEY>             Wert löschen
    edit                    Konfiguration im Editor öffnen

SCHLÜSSEL:
    identity.did            Aktive DID
    identity.default_key    Standard-Schlüssel
    network.default_shard   Standard-Shard
    network.timeout         Netzwerk-Timeout
    trust.min_witness       Minimale Witness-Gewichtung
    privacy.default_vis     Standard-Sichtbarkeit
    performance.cache       Cache-Größe

BEISPIELE:
    erynoa config list
    erynoa config get network.timeout
    erynoa config set network.timeout 60
    erynoa config set trust.min_witness 0.85
```

### `erynoa profile`

Verwaltet Profile (Konfigurationssets).

```bash
erynoa profile <COMMAND> [OPTIONS]

COMMANDS:
    list                    Profile auflisten
    create <NAME>           Neues Profil erstellen
    switch <NAME>           Profil wechseln
    delete <NAME>           Profil löschen
    export <NAME>           Profil exportieren
    import <FILE>           Profil importieren

BEISPIELE:
    erynoa profile create work
    erynoa profile switch work
    erynoa profile export work > work-profile.json
```

### `erynoa alias`

Verwaltet Befehlsaliase.

```bash
erynoa alias <COMMAND>

COMMANDS:
    list                    Aliase auflisten
    set <NAME> <COMMAND>    Alias setzen
    unset <NAME>            Alias löschen

BEISPIELE:
    erynoa alias set st "status --brief"
    erynoa alias set lg "log --oneline --limit 20"
    erynoa alias list
```

---

## 12. Netzwerk-Befehle

### `erynoa remote`

Verwaltet Realm-Verbindungen.

```bash
erynoa remote <COMMAND> [OPTIONS]

COMMANDS:
    list                    Remotes auflisten
    add <NAME> <DID>        Remote hinzufügen
    remove <NAME>           Remote entfernen
    rename <OLD> <NEW>      Remote umbenennen
    show <NAME>             Remote-Details

BEISPIELE:
    erynoa remote list
    erynoa remote add logistics did:erynoa:realm:supply-chain
    erynoa remote show logistics
```

### `erynoa sync`

Synchronisiert mit allen Remotes.

```bash
erynoa sync [OPTIONS]

OPTIONS:
    --remote <NAME>         Nur spezifischer Remote
    --full                  Vollständige Synchronisation
    --quick                 Nur neueste Events

BEISPIELE:
    erynoa sync
    erynoa sync --remote logistics --full
```

### `erynoa peers`

Zeigt verbundene Peers.

```bash
erynoa peers [OPTIONS]

OPTIONS:
    --shard <SHARD>         Peers in spezifischem Shard
    --active                Nur aktive Verbindungen
    --json                  JSON-Ausgabe

BEISPIELE:
    erynoa peers
    erynoa peers --shard energy-trading --active
```

### `erynoa validators`

Zeigt Validator-Informationen.

```bash
erynoa validators [OPTIONS]

OPTIONS:
    --shard <SHARD>         Validatoren für Shard
    --sort <FIELD>          Sortierung (trust, weight, latency)
    --json                  JSON-Ausgabe

BEISPIELE:
    erynoa validators
    erynoa validators --shard finance --sort trust
```

---

## Globale Optionen

Diese Optionen sind für alle Befehle verfügbar:

```
--help, -h              Hilfe anzeigen
--version, -v           Version anzeigen
--verbose               Ausführliche Ausgabe
--quiet, -q             Nur Fehler ausgeben
--json                  JSON-Ausgabe (wo verfügbar)
--config <FILE>         Alternative Konfigurationsdatei
--identity <DID>        Alternative Identität verwenden
--shard <SHARD>         Shard überschreiben
--dry-run               Nur simulieren, nichts ändern
--yes, -y               Alle Bestätigungen überspringen
--no-color              Farbausgabe deaktivieren
```

---

## Umgebungsvariablen

```bash
ERYNOA_HOME             Erynoa-Verzeichnis (default: ~/.erynoa)
ERYNOA_CONFIG           Konfigurationsdatei
ERYNOA_IDENTITY         Standard-Identität
ERYNOA_SHARD            Standard-Shard
ERYNOA_LOG_LEVEL        Log-Level (debug, info, warn, error)
ERYNOA_NO_COLOR         Farbausgabe deaktivieren (1/0)
```

---

## Exit-Codes

```
0       Erfolg
1       Allgemeiner Fehler
2       Ungültige Argumente
3       Konfigurationsfehler
4       Netzwerkfehler
5       Authentifizierungsfehler
6       Autorisierungsfehler
7       Konsens-Fehler
8       Validierungsfehler
9       Timeout
10      Abgebrochen durch Benutzer
```

---

## Schnellstart-Beispiele

### Erste Schritte

```bash
# Identität erstellen
erynoa init --namespace personal --label "Max Mustermann"

# Identität ins Netzwerk bekannt machen
erynoa push

# Status prüfen
erynoa status --brief
```

### Erste Transaktion

```bash
# Partner suchen
erynoa seek "freelance developer" --min-trust 0.6

# Angebot machen
erynoa propose did:erynoa:personal:dev --amount "20h" --price "2000 EUR" --duration 30d

# Wenn akzeptiert: Status verfolgen
erynoa stream status contract:sha3:...

# Abschließen
erynoa close contract:sha3:... --rating 5
```

### Cross-Shard Transfer

```bash
# Shards anzeigen
erynoa shard list

# Transfer vorbereiten
erynoa merge --from gaming --to finance --amount "500 tokens" --dry-run

# Transfer durchführen
erynoa merge --from gaming --to finance --amount "500 tokens"
```

---

## 13. Humanismus-Befehle (V6.0)

### human-auth – Human Authentication

```bash
# Prüft ob DID ein verifizierter Mensch ist (H1)
erynoa human-auth verify <did>

# Fordert neue HumanAuth-Verifizierung an
erynoa human-auth request --method=video|biometric|government-id

# Zeigt Human-Interaktions-Quote
erynoa human-auth quota
# Output: Human Interactions: 45/200 (22.5%) ✓ Quota: 20% met

# Web-of-Trust Verifizierung durch Bürgen
erynoa human-auth wot request --vouchers=3
```

### lod – Level of Detail

```bash
# Berechnet empfohlenes Vertrauens-Level (H2)
erynoa lod compute --value=5000
# Output: Recommended: ENHANCED (3 witnesses, quantum trust)

# Aktiviert automatische LoD-Wahl
erynoa config set lod.auto=true

# Zeigt Green-Trust-Score (Effizienz)
erynoa green-score
# Output: Efficiency: 67x (Good) - €4532 value / €67 verification cost

# Erzwingt minimales LoD für alle Transaktionen
erynoa config set lod.min=standard
```

### amnesty – Vergebungs-System

```bash
# Zeigt Amnestie-Status (H3)
erynoa amnesty status
# Output: Years since last negative: 5.2
#         Automatic amnesty eligible in: 1.8 years
#         Current weight of oldest negative: 0.23

# Beantragt automatische Amnestie (nach 7 Jahren)
erynoa amnesty apply --automatic

# Fresh-Start beantragen (neue DID mit Trust-Transfer)
erynoa amnesty fresh-start --transfer-positive-only

# Governance-Amnestie Antrag
erynoa governance propose amnesty <did> --reason="..."
```

### blueprint – Semantische Verankerung

```bash
# Prüft semantische Verankerung eines Blueprints (H4)
erynoa blueprint validate <blueprint-id>
# Output: NLD: ✓ (English, German)
#         FormalSpec: ✓
#         Equivalence: 94% confidence
#         Glossary: 12/12 terms defined

# Zeigt Natural Language Description
erynoa blueprint nld <blueprint-id> --lang=de

# Führt LLM-Äquivalenz-Prüfung durch
erynoa blueprint equivalence-check <blueprint-id>

# Erstellt Blueprint mit semantischer Verankerung
erynoa blueprint create --nld="./description.md" --spec="./spec.toml"
```

---

_Erynoa CLI Reference V6.0_
_Vollständige Befehlsreferenz basierend auf 126 Axiomen über 8 Ebenen_
_Humanistisch • Antifragil • Verhältnismäßig_
_"Das System existiert, um menschliches Gedeihen zu ermöglichen."_
