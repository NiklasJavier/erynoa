# Erynoa CLI Reference V4.1

> **Version:** 4.1 – Axiom-Abgeleitete Befehlsreferenz
> **Datum:** Januar 2026
> **Status:** Referenz
> **Basis:** LOGIC.md V4.1 (28 Kern-Axiome Κ1-Κ28 + 4 Unter-Axiome Κ15a-d)
> **Architektur:** 4-Schichten (Client/Peer, Core Logic, Storage/Realm, Protection)

---

## Schnellübersicht

```
IDENTITÄT            REALM                EVENTS               TRUST
──────────────────────────────────────────────────────────────────────────────
init                 realm                commit               trust
sub-identity           list                 push               inspect
key                    create               pull               attest
recover                join                 status             delegate
export                 info                 log                revoke
                       cross                diff
                       rules                show

TRANSAKTIONEN        SAGA                 GOVERNANCE           SCHUTZ
──────────────────────────────────────────────────────────────────────────────
seek                 saga                 governance           protection
propose                submit               proposal             anti-calcification
agree                  status               vote                 diversity
stream                 execute              veto                 anomaly
close                  cancel               delegate             fairness
dispute                rollback
                       simulate

NETZWERK             KONFIGURATION        DIAGNOSE             WELTFORMEL
──────────────────────────────────────────────────────────────────────────────
peer                 config               inspect              formula
  status               set                  verify               compute
  info                 get                  blame                status
  sync                 profile              audit                components
remote               alias                benchmark            simulate
```

---

## Axiom-Befehl-Mapping

| Axiom                | Primäre Befehle                          | Funktion                     |
| -------------------- | ---------------------------------------- | ---------------------------- |
| Κ1 (Regelvererbung)  | `realm create`, `realm rules`            | Realm-Hierarchie verwalten   |
| Κ2-Κ5 (Trust)        | `trust`, `attest`, `delegate`            | Trust-Vektor 𝕎 aktualisieren |
| Κ6-Κ8 (Identität)    | `init`, `sub-identity`, `key`            | DIDs und Delegationen        |
| Κ9-Κ12 (Kausalität)  | `commit`, `push`, `log`                  | Event-DAG ℂ verwalten        |
| Κ13-Κ14 (TAT)        | `seek`, `propose`, `agree`               | Transaktions-Lifecycle       |
| Κ15a-d (Weltformel)  | `formula compute`, `formula status`      | 𝔼-Berechnung und Surprisal   |
| Κ16-Κ17 (Governance) | `governance proposal`, `governance vote` | Quadratische Governance Ψ    |
| Κ18 (Konsens)        | `peer sync`, `push --wait`               | Partition-Konsens            |
| Κ19-Κ21 (Schutz)     | `protection *`                           | Anti-Degeneration            |
| Κ22-Κ24 (Peer)       | `saga *`, `peer *`                       | Gateway + Composer Pattern   |
| Κ25-Κ28 (Human)      | `init --human`, `trust inspect --Ĥ`      | Human-Aligned Mechanismen    |

---

## I. Identitäts-Befehle (Κ6-Κ8)

### `erynoa init`

Erstellt eine neue Identität (DID) gemäß **Κ6 (Existenz-Eindeutigkeit)**.

```bash
erynoa init [OPTIONS]

OPTIONS:
    --namespace <NS>        Namespace (default: self)
                            Werte: self, guild, spirit, thing, vessel,
                                   source, craft, vault, pact, circle
    --algorithm <ALG>       Kryptographischer Algorithmus
                            Werte: ed25519 (default), secp256k1, bls12-381
    --label <LABEL>         Menschenlesbares Label
    --recover <SEED>        Aus 24-Wort Seed-Phrase wiederherstellen
    --human                 Human-Attestation anfordern (Κ25-Κ28)
                            Aktiviert Ĥ-Bonus in Weltformel

BEISPIELE:
    # Persönliche Identität erstellen
    erynoa init --label "Alice"

    # Organisation mit Human-Attestation
    erynoa init --namespace guild --label "Meine Firma GmbH" --human

    # KI-Agent (spirit) ohne Human-Flag
    erynoa init --namespace spirit --label "Trading-Bot"

    # Wiederherstellung aus Backup
    erynoa init --recover "word1 word2 ... word24"

AXIOM-REFERENZ:
    Κ6: ∀ entity e : ∃! did ∈ DID : identity(e) = did
    Κ7: ⟨s⟩ ∧ ⟦create(s)⟧ ⟹ □⟨s⟩ (Permanenz)
    Κ25: Human ⟹ Ĥ(s) ∈ {1.0, 1.2, 1.5}

OUTPUT:
    ┌────────────────────────────────────────────────────────┐
    │ ✓ Identität erstellt                                   │
    ├────────────────────────────────────────────────────────┤
    │ DID:       did:erynoa:self:abc123...                   │
    │ Namespace: self                                        │
    │ Algorithm: Ed25519                                     │
    │ Label:     Alice                                       │
    │ Human:     pending (Attestation angefordert)           │
    │ 𝔸(s):      0.01 (initial)                              │
    │ 𝕎(s):      [0.5, 0.5, 0.5, 0.5, 0.5, 0.5]             │
    │ Ĥ(s):      1.0 (pending human verification)           │
    └────────────────────────────────────────────────────────┘

    ⚠️  Backup-Seed sicher aufbewahren:
    word1 word2 word3 ... word24
```

### `erynoa sub-identity`

Verwaltet delegierte Sub-Identitäten gemäß **Κ8 (Delegations-Struktur)**.

```bash
erynoa sub-identity <COMMAND> [OPTIONS]

COMMANDS:
    create <NAME>           Neue Sub-Identität (s ⊳ s')
    list                    Alle Sub-Identitäten auflisten
    switch <NAME>           Zu Sub-Identität wechseln
    delete <NAME>           Sub-Identität löschen
    info <NAME>             Details einer Sub-Identität

OPTIONS (create):
    --inherit-trust <F>     Trust-Vererbungsfaktor (0.0-1.0, default: 0.5)
                            Gemäß Κ8: 𝕋(s') ≤ 𝕋(s)
    --realm <REALM>         Kontext-Beschränkung auf Realm
    --permissions <PERMS>   Erlaubte Aktionen
                            Werte: transfer, attest, claim, governance, all
    --expires <DUR>         Ablaufzeit (z.B. "30d", "1y")

BEISPIELE:
    # Gaming-Identität mit reduziertem Trust
    erynoa sub-identity create gaming --inherit-trust 0.7

    # Arbeits-Identität beschränkt auf Business-Realm
    erynoa sub-identity create work \
        --realm "realm:erynoa:business" \
        --permissions transfer,attest \
        --expires 1y

    # Alle Sub-Identitäten anzeigen
    erynoa sub-identity list

AXIOM-REFERENZ:
    Κ8: s ⊳ s' → 𝕋(s') ≤ 𝕋(s)  [Trust-Beschränkung]
    Τ2: s ⊳ s' ⟹ (𝔸(s') > 0 → 𝔸(s) ≥ δ·𝔸(s'))  [Aktivitätsfluss]

OUTPUT (list):
    ┌─────────────┬──────────────────────┬───────────┬──────────────┐
    │ Name        │ DID                  │ Trust     │ Realm        │
    ├─────────────┼──────────────────────┼───────────┼──────────────┤
    │ gaming      │ did:erynoa:self:g123 │ 0.35 (70%)│ *            │
    │ work        │ did:erynoa:self:w456 │ 0.25 (50%)│ business     │
    │ iot-sensor  │ did:erynoa:thing:i78 │ 0.10 (20%)│ home-realm   │
    └─────────────┴──────────────────────┴───────────┴──────────────┘
```

### `erynoa key`

Verwaltet kryptographische Schlüssel.

```bash
erynoa key <COMMAND> [OPTIONS]

COMMANDS:
    list                    Alle Schlüssel auflisten
    generate                Neuen Schlüssel generieren
    rotate                  Primärschlüssel rotieren
    revoke <KEY_ID>         Schlüssel widerrufen
    export <KEY_ID>         Öffentlichen Schlüssel exportieren
    import <FILE>           Schlüssel importieren
    derive <PATH>           HD-Schlüssel ableiten (BIP-44)

OPTIONS:
    --algorithm <ALG>       Algorithmus (ed25519, secp256k1, bls12-381)
    --purpose <PURPOSE>     Verwendungszweck
                            Werte: primary, signing, encryption, recovery
    --chain <CHAIN>         Ziel-Chain für derive
                            Werte: erynoa, ethereum, iota

BEISPIELE:
    erynoa key list
    erynoa key generate --algorithm bls12-381 --purpose signing
    erynoa key derive --chain ethereum
    erynoa key rotate --purpose primary

OUTPUT (list):
    ┌────────────┬───────────┬──────────────┬─────────────┬────────────┐
    │ ID         │ Algorithm │ Purpose      │ Created     │ Status     │
    ├────────────┼───────────┼──────────────┼─────────────┼────────────┤
    │ k_primary  │ Ed25519   │ primary      │ 2026-01-15  │ active     │
    │ k_sign_001 │ BLS12-381 │ signing      │ 2026-01-20  │ active     │
    │ k_eth_001  │ secp256k1 │ ethereum     │ 2026-01-20  │ active     │
    └────────────┴───────────┴──────────────┴─────────────┴────────────┘
```

### `erynoa recover`

Stellt Identität aus Backup wieder her gemäß **Κ7 (Permanenz)**.

```bash
erynoa recover [OPTIONS]

OPTIONS:
    --seed <SEED>           24-Wort Seed-Phrase
    --file <FILE>           Verschlüsselte Backup-Datei
    --verify-only           Nur verifizieren, nicht wiederherstellen
    --target-realm <REALM>  In spezifischem Realm wiederherstellen

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
    --include-trust         Trust-Vektor 𝕎 einschließen
    --output <FILE>         Ausgabedatei

BEISPIELE:
    erynoa export --format did-document > my-did.json
    erynoa export --include-trust --format json
```

---

## II. Realm-Befehle (Κ1)

### `erynoa realm`

Verwaltet die Realm-Hierarchie gemäß **Κ1 (Monotone Regelvererbung)**.

```bash
erynoa realm <COMMAND> [OPTIONS]

COMMANDS:
    list                    Alle zugänglichen Realms auflisten
    create <NAME>           Neues VirtualRealm erstellen
    join <REALM_ID>         Realm beitreten
    leave <REALM_ID>        Realm verlassen
    info <REALM_ID>         Realm-Details anzeigen
    cross <SRC> <DST>       Realm-Übergang simulieren
    rules <REALM_ID>        Regelset eines Realms anzeigen

OPTIONS (create):
    --parent <REALM>        Übergeordnetes Realm (default: RootRealm)
    --rules <FILE>          Zusätzliche Regeln (ECLVM-Format)
    --governance <TYPE>     Governance-Modell
                            Werte: quadratic (default), token, reputation
    --min-trust <T>         Minimaler Trust für Beitritt (default: 0.3)

BEISPIELE:
    # VirtualRealm für EU-Handel erstellen
    erynoa realm create eu-trade \
        --parent "realm:root" \
        --rules gdpr-compliance.ecl \
        --governance quadratic

    # Realm beitreten
    erynoa realm join realm:erynoa:eu-trade

    # Realm-Crossing analysieren
    erynoa realm cross "realm:gaming" "realm:finance"

AXIOM-REFERENZ:
    Κ1: ∀ 𝒞₁ ⊂ 𝒞₂ : rules(𝒞₁) ⊇ rules(𝒞₂)
        "Kind-Realms können Regeln hinzufügen, nie entfernen."

OUTPUT (list):
    ┌─────────────────────────────────────────────────────────────────────────┐
    │ REALM-HIERARCHIE                                                        │
    ├─────────────────────────────────────────────────────────────────────────┤
    │ 🌐 RootRealm (realm:root)                                               │
    │ │  Rules: 28 Kern-Axiome (Κ1-Κ28)                                       │
    │ │  Members: ∞                                                           │
    │ │                                                                       │
    │ ├─📦 VirtualRealm: EU-Trade (realm:erynoa:eu-trade)                    │
    │ │ │  Rules: +GDPR, +eIDAS                                               │
    │ │ │  Members: 1,245                                                     │
    │ │ │  Min-Trust: 0.3                                                     │
    │ │ │                                                                     │
    │ │ ├─🧩 Partition: Energy (partition:eu-trade:energy)                   │
    │ │ │    Rules: +RE100                                                    │
    │ │ │    Members: 342                                                     │
    │ │ │                                                                     │
    │ │ └─🧩 Partition: Finance (partition:eu-trade:finance)                 │
    │ │      Rules: +MiCA, +PSD2                                              │
    │ │      Members: 567                                                     │
    │ │                                                                       │
    │ └─📦 VirtualRealm: Gaming (realm:erynoa:gaming)                        │
    │      Rules: +Fair-Play                                                  │
    │      Members: 8,901                                                     │
    └─────────────────────────────────────────────────────────────────────────┘

OUTPUT (cross):
    ┌─────────────────────────────────────────────────────────────────────────┐
    │ REALM-CROSSING: gaming → finance                                        │
    ├─────────────────────────────────────────────────────────────────────────┤
    │ Gemeinsamer Vorfahr: RootRealm                                          │
    │ Pfadlänge: 2 (gaming → root → finance)                                  │
    │                                                                         │
    │ Trust-Transformation (Κ2):                                              │
    │   𝕎_finance = M_cross × 𝕎_gaming                                        │
    │                                                                         │
    │   Competence:  0.85 → 0.34 (×0.4 Kontext-Wechsel)                       │
    │   Integrity:   0.90 → 0.81 (×0.9 übertragbar)                           │
    │   Reliability: 0.75 → 0.60 (×0.8 teilweise übertragbar)                 │
    │   Prestige:    0.60 → 0.18 (×0.3 Kontext-spezifisch)                    │
    │   Vigilance:   0.50 → 0.50 (×1.0 universal)                             │
    │   Omega:       0.95 → 0.95 (×1.0 universal)                             │
    │                                                                         │
    │   ‖𝕎_gaming‖ = 0.76  →  ‖𝕎_finance‖ = 0.58                              │
    │                                                                         │
    │ ⚠️  Zusätzliche Regeln in finance: MiCA, PSD2, AML/KYC                  │
    └─────────────────────────────────────────────────────────────────────────┘
```

---

## III. Event-Befehle (Κ9-Κ12)

### `erynoa commit`

Erstellt ein neues Event im DAG gemäß **Κ9 (Kausale Struktur)** und **Κ12 (Event-Erzeugung)**.

```bash
erynoa commit [OPTIONS]

OPTIONS:
    --type <TYPE>           Event-Typ
                            Werte: transfer, mint, burn, claim, attest,
                                   credential_issue, credential_revoke,
                                   proposal, vote, saga_step
    --message <MSG>         Beschreibung
    -m <MSG>                Kurzform für --message
    --parents <EVENTS>      Explizite Parent-Events (comma-separated)
    --realm <REALM>         Ziel-Realm (default: aktueller)
    --dry-run               Simulieren ohne Ausführung

BEISPIELE:
    # Einfaches Event erstellen
    erynoa commit -m "Monatliche Energielieferung dokumentiert"

    # Transfer-Event mit explizitem Parent
    erynoa commit --type transfer \
        --message "Zahlung Q1 2026" \
        --parents "event:abc123"

    # Attestation in spezifischem Realm
    erynoa commit --type attest \
        --realm "realm:erynoa:eu-trade" \
        -m "Qualitätszertifikat bestätigt"

AXIOM-REFERENZ:
    Κ9:  ℂ = (E, ⊲) ist DAG (keine Zyklen)
    Κ12: ∀Π : ⟦Π⟧ → Δ|ℂ| ≥ 1

OUTPUT:
    ┌─────────────────────────────────────────────────────────────────────────┐
    │ ✓ Event erstellt                                                        │
    ├─────────────────────────────────────────────────────────────────────────┤
    │ Event-ID:    event:sha3:7f8a9b...                                       │
    │ Type:        transfer                                                   │
    │ Message:     Zahlung Q1 2026                                            │
    │ Parents:     [event:abc123]                                             │
    │ Realm:       realm:erynoa:eu-trade                                      │
    │ Timestamp:   2026-01-31T14:30:00Z                                       │
    │ Status:      NASCENT (finality: 0.5)                                    │
    │                                                                         │
    │ Weltformel-Impact:                                                      │
    │   Δ|ℂ|:  +1                                                             │
    │   Δ𝔸:   +0.02 (Aktivität erhöht)                                        │
    │   Δℐ:   +0.15 (Surprisal)                                               │
    │   Δ𝔼:   +0.003 (Beitrag zur Weltformel)                                 │
    │                                                                         │
    │ Nächster Schritt: `erynoa push` um Event zu propagieren                 │
    └─────────────────────────────────────────────────────────────────────────┘
```

### `erynoa push`

Propagiert lokale Events ins Netzwerk gemäß **Κ18 (Konsens)**.

```bash
erynoa push [OPTIONS]

OPTIONS:
    --partition <PART>      Ziel-Partition (default: automatisch)
    --priority <PRIO>       Priorität (low, normal, high)
    --wait                  Auf Finalität warten
    --timeout <SECS>        Timeout für --wait (default: 60)
    --min-finality <F>      Minimale Finalität (default: 0.9)

BEISPIELE:
    # Standard-Push
    erynoa push

    # Mit hoher Priorität und Warten auf Konsens
    erynoa push --priority high --wait --min-finality 0.99

AXIOM-REFERENZ:
    Κ10: ⟦e⟧ → □⟦e⟧ (Permanenz der Bezeugung)
    Κ18: Ψ(𝒫, e) = Σᵢ sign(vᵢ, e) · w(vᵢ) / Σⱼ w(vⱼ)

OUTPUT:
    ┌─────────────────────────────────────────────────────────────────────────┐
    │ 📡 Propagating Events...                                                │
    ├─────────────────────────────────────────────────────────────────────────┤
    │ Event:       event:sha3:7f8a9b...                                       │
    │ Partition:   partition:eu-trade:finance                                 │
    │                                                                         │
    │ Konsens-Fortschritt (Κ18):                                              │
    │   ████████████████████░░░░ 80% (4/5 Validators)                         │
    │                                                                         │
    │ Validator-Responses:                                                    │
    │   ✓ validator:alice   (w=0.25)  accepted in 120ms                       │
    │   ✓ validator:bob     (w=0.20)  accepted in 145ms                       │
    │   ✓ validator:carol   (w=0.30)  accepted in 98ms                        │
    │   ✓ validator:dave    (w=0.15)  accepted in 210ms                       │
    │   ⏳ validator:eve    (w=0.10)  pending...                              │
    │                                                                         │
    │ Finality: 0.92 (WITNESSED)                                              │
    │ Merkle-Root: 0x3a7f...                                                  │
    └─────────────────────────────────────────────────────────────────────────┘
```

### `erynoa pull`

Synchronisiert lokalen Zustand mit dem Netzwerk.

```bash
erynoa pull [OPTIONS]

OPTIONS:
    --partition <PART>      Quell-Partition (default: alle abonnierten)
    --since <EVENT>         Nur Events seit diesem Event
    --depth <N>             Maximale DAG-Tiefe (default: 100)
    --verify                Alle Merkle-Proofs verifizieren
    --trust-filter <T>      Nur Events von DIDs mit Trust ≥ T

BEISPIELE:
    erynoa pull
    erynoa pull --partition finance --since "event:abc123" --verify
    erynoa pull --trust-filter 0.5
```

### `erynoa status`

Zeigt vollständigen Zustand gemäß **Weltformel V2.0**.

```bash
erynoa status [OPTIONS]

OPTIONS:
    --full                  Vollständige Analyse
    --brief                 Kurzfassung
    --json                  JSON-Ausgabe
    --component <COMP>      Nur spezifische Komponente
                            Werte: 𝔸, 𝕎, ℂ, ℐ, 𝒮, Ĥ, 𝔼, all

BEISPIELE:
    erynoa status
    erynoa status --component 𝕎
    erynoa status --json > status.json

AXIOM-REFERENZ:
    Κ15b: 𝔼 = Σ 𝔸(s) · σ⃗( ‖𝕎(s)‖_w · ln|ℂ(s)| · 𝒮(s) ) · Ĥ(s) · w(s,t)

OUTPUT:
    ╔════════════════════════════════════════════════════════════════════════╗
    ║                     ERYNOA STATUS                                      ║
    ║                     Weltformel V2.0                                    ║
    ╠════════════════════════════════════════════════════════════════════════╣
    ║                                                                        ║
    ║   Identität: did:erynoa:self:alice                                     ║
    ║   Realm:     realm:erynoa:eu-trade                                     ║
    ║   Human:     ✓ verified (Ĥ = 1.2)                                      ║
    ║                                                                        ║
    ╠════════════════════════════════════════════════════════════════════════╣
    ║   𝔸 AKTIVITÄT                                                          ║
    ║   ───────────────────────────────────────────────────────────────────  ║
    ║   |{e ∈ ℂ(s) : age(e) < τ}| = 42                                       ║
    ║   κ = 10                                                               ║
    ║   𝔸(s) = 42 / (42 + 10) = 0.81                                         ║
    ║                                                                        ║
    ╠════════════════════════════════════════════════════════════════════════╣
    ║   𝕎 TRUST-VEKTOR (6D)                                                  ║
    ║   ───────────────────────────────────────────────────────────────────  ║
    ║                                                                        ║
    ║   R (Reliability):  ████████████████████░░░░ 0.85                      ║
    ║   I (Integrity):    ███████████████████████░ 0.92                      ║
    ║   C (Competence):   ██████████████████░░░░░░ 0.75                      ║
    ║   P (Prestige):     ████████████████░░░░░░░░ 0.68                      ║
    ║   V (Vigilance):    ███████████░░░░░░░░░░░░░ 0.50                      ║
    ║   Ω (Omega):        ██████████████████████░░ 0.90                      ║
    ║                                                                        ║
    ║   ‖𝕎‖_w = 0.78  (gewichtete Norm)                                      ║
    ║                                                                        ║
    ╠════════════════════════════════════════════════════════════════════════╣
    ║   ℂ HISTORIE                                                           ║
    ║   ───────────────────────────────────────────────────────────────────  ║
    ║   |ℂ(s)| = 1,247 Events                                                ║
    ║   ln|ℂ(s)| = 7.13                                                      ║
    ║   Tiefe (DAG): 892                                                     ║
    ║   Breite (max): 12                                                     ║
    ║                                                                        ║
    ╠════════════════════════════════════════════════════════════════════════╣
    ║   ℐ SURPRISAL & 𝒮 DÄMPFUNG                                             ║
    ║   ───────────────────────────────────────────────────────────────────  ║
    ║   ℐ(s) = -log₂(f(s)) = 3.2 bits                                        ║
    ║   𝒮(s) = ‖𝕎(s)‖² · ℐ(s) = 0.61 · 3.2 = 1.95                           ║
    ║                                                                        ║
    ║   Anti-Hype: Hoher Trust dämpft Surprisal (Κ15a)                       ║
    ║                                                                        ║
    ╠════════════════════════════════════════════════════════════════════════╣
    ║   𝔼 WELTFORMEL-BEITRAG                                                 ║
    ║   ───────────────────────────────────────────────────────────────────  ║
    ║                                                                        ║
    ║   𝔼_you = 𝔸 · σ( ‖𝕎‖ · ln|ℂ| · 𝒮 ) · Ĥ · w                           ║
    ║         = 0.81 · σ(0.78 · 7.13 · 1.95) · 1.2 · 0.001                   ║
    ║         = 0.81 · 0.9997 · 1.2 · 0.001                                  ║
    ║         = 0.00097                                                      ║
    ║                                                                        ║
    ║   Globales 𝔼 (Partition): 847.32                                       ║
    ║   Dein Anteil: 0.00011%                                                ║
    ║                                                                        ║
    ╚════════════════════════════════════════════════════════════════════════╝
```

### `erynoa log`

Zeigt Event-Historie im DAG.

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
    --trust-impact          Trust-Änderungen anzeigen

BEISPIELE:
    erynoa log
    erynoa log --limit 50 --type transfer
    erynoa log --graph --limit 20
    erynoa log --trust-impact

OUTPUT (--graph):
    *   7f8a9b [2026-01-31] transfer: Zahlung Q1 2026 (Δ𝕎.R +0.02)
    |\
    | * 6e7d8c [2026-01-30] attest: Lieferung bestätigt
    * | 5c4b3a [2026-01-29] claim: Verfügbarkeit aktualisiert
    |/
    *   4a3b2c [2026-01-28] transfer: Anzahlung erhalten
    *   3d2e1f [2026-01-27] credential_issue: Zertifikat ausgestellt
```

### `erynoa diff`

Zeigt Unterschiede zwischen Zuständen.

```bash
erynoa diff <EVENT1>..<EVENT2> [OPTIONS]
erynoa diff <EVENT> [OPTIONS]

OPTIONS:
    --stat                  Nur Statistiken
    --trust                 Nur Trust-Änderungen (Δ𝕎)
    --formula               Weltformel-Komponenten (Δ𝔼)

BEISPIELE:
    erynoa diff event:abc..event:def
    erynoa diff HEAD~5..HEAD --trust
    erynoa diff event:abc --formula
```

### `erynoa show`

Zeigt Details eines Events.

```bash
erynoa show <ID> [OPTIONS]

OPTIONS:
    --format <FMT>          Ausgabeformat (human, json, cbor)
    --verify                Signaturen und Proofs verifizieren
    --expand                Referenzierte Objekte einbetten
    --causality             Kausale Abhängigkeiten anzeigen

BEISPIELE:
    erynoa show event:sha3:abc123
    erynoa show event:sha3:abc123 --verify --causality
```

---

## IV. Trust-Befehle (Κ2-Κ5)

### `erynoa trust`

Zeigt und verwaltet Trust-Beziehungen.

```bash
erynoa trust <COMMAND> [OPTIONS]

COMMANDS:
    inspect <DID>           Trust-Vektor einer Entität anzeigen
    history <DID>           Trust-Evolution über Zeit
    path <FROM> <TO>        Trust-Pfad zwischen zwei DIDs
    simulate <EVENT>        Simuliere Trust-Änderung

OPTIONS (inspect):
    --context <REALM>       Kontext für Abfrage
    --weighted              Gewichtete Norm anzeigen
    --Ĥ                     Human-Bonus anzeigen

BEISPIELE:
    # Trust einer DID inspizieren
    erynoa trust inspect did:erynoa:guild:supplier

    # Trust-Pfad analysieren
    erynoa trust path did:erynoa:self:alice did:erynoa:guild:supplier

    # Event-Auswirkung simulieren
    erynoa trust simulate --type attest --target did:erynoa:self:bob

AXIOM-REFERENZ:
    Κ2: 𝕋(id_s) = id_𝕋(s), 𝕋(g ∘ f) = 𝕋(f) ∘ 𝕋(g)
    Κ3: ∀ i,j : ∂𝕎ᵢ/∂event ⊥ ∂𝕎ⱼ/∂event
    Κ4: Δ⁻(dim) = λ_asym · Δ⁺(dim)
    Κ5: t₁ ⊕ t₂ = 1 - (1-t₁)(1-t₂)

OUTPUT (inspect):
    ┌─────────────────────────────────────────────────────────────────────────┐
    │ TRUST-ANALYSE: did:erynoa:guild:supplier                                │
    ├─────────────────────────────────────────────────────────────────────────┤
    │                                                                         │
    │ 𝕎 TRUST-VEKTOR                                                          │
    │                                                                         │
    │   R (Reliability):  ████████████████████░░░░ 0.82  ↑ +0.03 (7d)        │
    │   I (Integrity):    ███████████████████████░ 0.94  = +0.00 (7d)        │
    │   C (Competence):   ██████████████████░░░░░░ 0.78  ↑ +0.05 (7d)        │
    │   P (Prestige):     ████████████████░░░░░░░░ 0.65  ↓ -0.02 (7d)        │
    │   V (Vigilance):    ████████████████████░░░░ 0.80  = +0.00 (7d)        │
    │   Ω (Omega):        ███████████████████████░ 0.95  = +0.00 (7d)        │
    │                                                                         │
    │ Gewichtete Norm: ‖𝕎‖_w = 0.83                                           │
    │ Human-Bonus: Ĥ = 1.0 (nicht human-verifiziert)                         │
    │                                                                         │
    │ TRUST-KETTE (Τ1)                                                        │
    │   alice → supplier (direkt): 0.83                                       │
    │   alice → consortium → supplier: 0.76                                   │
    │   Kombiniert (⊕): 0.96                                                  │
    │                                                                         │
    │ KONTEXT-VARIATION                                                       │
    │   Kontext: finance    → ‖𝕎‖ = 0.81 (-0.02)                             │
    │   Kontext: energy     → ‖𝕎‖ = 0.90 (+0.07)                             │
    │   Kontext: gaming     → ‖𝕎‖ = 0.45 (-0.38)                             │
    │                                                                         │
    └─────────────────────────────────────────────────────────────────────────┘

OUTPUT (path):
    ┌─────────────────────────────────────────────────────────────────────────┐
    │ TRUST-PFAD: alice → supplier                                            │
    ├─────────────────────────────────────────────────────────────────────────┤
    │                                                                         │
    │ Direkte Verbindung: ✗ (keine direkte Attestation)                       │
    │                                                                         │
    │ Pfad 1 (Länge 2):                                                       │
    │   alice ──(0.85)──► consortium ──(0.92)──► supplier                     │
    │   Ketten-Trust (Τ1): exp(ln(0.85) + ln(0.92) / √2) = 0.81              │
    │                                                                         │
    │ Pfad 2 (Länge 3):                                                       │
    │   alice ──(0.78)──► bob ──(0.80)──► carol ──(0.88)──► supplier          │
    │   Ketten-Trust (Τ1): exp((ln(0.78) + ln(0.80) + ln(0.88)) / √3) = 0.72 │
    │                                                                         │
    │ Kombinierter Trust (⊕): 1 - (1-0.81)(1-0.72) = 0.95                     │
    │                                                                         │
    └─────────────────────────────────────────────────────────────────────────┘
```

### `erynoa attest`

Erstellt eine Attestation für eine andere Entität.

```bash
erynoa attest <TARGET_DID> [OPTIONS]

OPTIONS:
    --dimension <DIM>       Trust-Dimension zu attestieren
                            Werte: R, I, C, P, all
    --strength <S>          Attestations-Stärke (0.1-1.0, default: 0.5)
    --claim <CLAIM>         Spezifische Behauptung attestieren
    --credential <VC>       Verifiable Credential attestieren
    --expires <DUR>         Ablaufzeit der Attestation

BEISPIELE:
    # Competence attestieren
    erynoa attest did:erynoa:self:bob --dimension C --strength 0.8

    # Zertifikat attestieren
    erynoa attest did:erynoa:guild:supplier \
        --credential energy-certificate.json \
        --dimension P
```

### `erynoa delegate`

Delegiert Trust an eine andere Entität (Κ8).

```bash
erynoa delegate <TARGET_DID> [OPTIONS]

OPTIONS:
    --scope <SCOPE>         Delegations-Scope
                            Werte: attest, transfer, governance, all
    --factor <F>            Trust-Vererbungsfaktor (0.0-1.0, default: 0.5)
    --realm <REALM>         Auf Realm beschränken
    --expires <DUR>         Ablaufzeit
    --revocable             Widerrufbar (default: true)

BEISPIELE:
    erynoa delegate did:erynoa:self:assistant \
        --scope attest,transfer \
        --factor 0.7 \
        --realm "realm:erynoa:business"
```

### `erynoa revoke`

Widerruft Attestationen oder Delegationen.

```bash
erynoa revoke <TYPE> <ID> [OPTIONS]

TYPES:
    attestation             Attestation widerrufen
    delegation              Delegation widerrufen
    credential              Credential widerrufen

OPTIONS:
    --reason <REASON>       Widerrufsgrund
    --effective <DATE>      Wirksamkeitsdatum (default: sofort)

BEISPIELE:
    erynoa revoke attestation attest:sha3:abc123 --reason "Fehlerhafte Daten"
    erynoa revoke delegation deleg:sha3:def456
```

---

## V. Transaktions-Befehle (Κ13-Κ14)

### `erynoa seek`

Sucht nach Transaktionspartnern gemäß **Κ13 (TAT-Lifecycle)**.

```bash
erynoa seek <QUERY> [OPTIONS]

OPTIONS:
    --type <TYPE>           Partner-Typ (self, guild, spirit, thing)
    --realm <REALM>         Realm-Beschränkung
    --min-trust <T>         Minimaler Trust (default: 0.5)
    --max-results <N>       Maximale Ergebnisse (default: 10)
    --include-emerging      Auch niedrigen Trust einschließen
    --diversity-bonus       Diversity-Slots priorisieren (Κ20)
    --sort <FIELD>          Sortierung (trust, surprisal, relevance)

BEISPIELE:
    # Energielieferanten suchen
    erynoa seek "renewable energy supplier" \
        --realm "realm:erynoa:eu-trade" \
        --min-trust 0.6

    # Mit Diversity-Bonus für neue Anbieter
    erynoa seek "software developer" \
        --type self \
        --diversity-bonus \
        --include-emerging

AXIOM-REFERENZ:
    Κ13: TAT = (seek, propose, agree, exec, settle)
    Κ20: D(𝒞) = H(distribution) / H_max

OUTPUT:
    ┌─────────────────────────────────────────────────────────────────────────┐
    │ 🔍 SUCHERGEBNISSE: "renewable energy supplier"                          │
    │    Realm: eu-trade | Min-Trust: 0.6 | Diversity-Bonus: ON               │
    ├─────────────────────────────────────────────────────────────────────────┤
    │                                                                         │
    │ #1 GreenPower AG                                                        │
    │    DID:      did:erynoa:guild:greenpower                                │
    │    Trust:    ‖𝕎‖ = 0.89                                                 │
    │    Ĥ:        1.2 (human-verified)                                       │
    │    Surprisal: ℐ = 2.1 bits (etabliert)                                  │
    │    Match:    94%                                                        │
    │    🏆 Top-Empfehlung                                                    │
    │                                                                         │
    │ #2 SolarStart GmbH                                                      │
    │    DID:      did:erynoa:guild:solarstart                                │
    │    Trust:    ‖𝕎‖ = 0.65                                                 │
    │    Ĥ:        1.0 (pending)                                              │
    │    Surprisal: ℐ = 5.8 bits (neu, hohes Potential)                       │
    │    Match:    78%                                                        │
    │    🌱 Diversity-Slot (Κ20)                                              │
    │                                                                         │
    │ #3 WindForce Collective                                                 │
    │    DID:      did:erynoa:guild:windforce                                 │
    │    Trust:    ‖𝕎‖ = 0.82                                                 │
    │    Ĥ:        1.5 (full human attestation)                               │
    │    Surprisal: ℐ = 3.4 bits                                              │
    │    Match:    85%                                                        │
    │                                                                         │
    └─────────────────────────────────────────────────────────────────────────┘
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
    --saga                  Als Saga-Intent (für komplexe Transaktionen)

BEISPIELE:
    # Einfache Transaktion
    erynoa propose did:erynoa:guild:greenpower \
        --amount "500 kWh" \
        --price "125 EUR" \
        --duration 30d

    # Streaming-Vertrag
    erynoa propose did:erynoa:guild:solarstart \
        --amount "10000 kWh" \
        --price "2500 EUR" \
        --duration 1y \
        --streaming

AXIOM-REFERENZ:
    Κ13: PROPOSE ∈ TAT-Lifecycle
    Κ14: Streaming-Semantik

OUTPUT:
    ┌─────────────────────────────────────────────────────────────────────────┐
    │ 📝 TRANSAKTIONSVORSCHLAG                                                │
    ├─────────────────────────────────────────────────────────────────────────┤
    │ Proposal-ID: proposal:sha3:8a9b7c...                                    │
    │ Target:      did:erynoa:guild:greenpower                                │
    │ Amount:      500 kWh                                                    │
    │ Price:       125 EUR                                                    │
    │ Duration:    30 days                                                    │
    │ Mode:        Standard (nicht streaming)                                 │
    │ Expires:     2026-02-07T14:30:00Z                                       │
    │                                                                         │
    │ Trust-Analyse:                                                          │
    │   Dein Trust für Partner:  ‖𝕎‖ = 0.89                                   │
    │   Partner Trust für dich:  ‖𝕎‖ = 0.82                                   │
    │   P(success):              94.2%                                        │
    │                                                                         │
    │ Weltformel-Prognose:                                                    │
    │   Bei Erfolg: Δ𝕎.R ≈ +0.02, Δ|ℂ| = +3                                   │
    │   Bei Ablehnung: Keine Änderung                                         │
    │                                                                         │
    │ Status: Warte auf Antwort...                                            │
    └─────────────────────────────────────────────────────────────────────────┘
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
    erynoa agree proposal:sha3:8a9b7c --accept
    erynoa agree proposal:sha3:8a9b7c --reject --message "Preis zu hoch"
    erynoa agree proposal:sha3:8a9b7c --counter-price "110 EUR"
```

### `erynoa stream`

Verwaltet laufende Streaming-Transaktionen (Κ14).

```bash
erynoa stream <COMMAND> <CONTRACT_ID> [OPTIONS]

COMMANDS:
    status                  Status anzeigen
    pause                   Pausieren
    resume                  Fortsetzen
    abort                   Abbrechen mit Settlement
    extend                  Verlängern

OPTIONS (abort):
    --reason <REASON>       Abbruchgrund
                            Werte: buyer-request, seller-failure, mutual, force-majeure

BEISPIELE:
    erynoa stream status contract:sha3:abc
    erynoa stream pause contract:sha3:abc
    erynoa stream abort contract:sha3:abc --reason seller-failure
```

### `erynoa close`

Schließt eine Transaktion ab.

```bash
erynoa close <CONTRACT_ID> [OPTIONS]

OPTIONS:
    --rating <1-5>          Bewertung des Partners
    --attest <DIMS>         Trust-Dimensionen attestieren
    --comment <TEXT>        Kommentar zur Transaktion

BEISPIELE:
    erynoa close contract:sha3:abc --rating 5 --attest R,C
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

OPTIONS (open):
    --type <TYPE>           Dispute-Typ (non-delivery, quality, payment)
    --description <TEXT>    Beschreibung
    --evidence <FILE>       Beweismaterial

BEISPIELE:
    erynoa dispute open contract:sha3:abc --type quality
```

---

## VI. Saga-Befehle (Κ22-Κ24)

### `erynoa saga`

Verwaltet komplexe Multi-Step-Transaktionen gemäß **Κ22-Κ24 (Peer-Logik)**.

```bash
erynoa saga <COMMAND> [OPTIONS]

COMMANDS:
    submit <INTENT>         Intent zur Saga-Auflösung einreichen
    status <SAGA_ID>        Saga-Status anzeigen
    execute <SAGA_ID>       Saga manuell ausführen
    cancel <SAGA_ID>        Saga abbrechen
    rollback <SAGA_ID>      Saga zurückrollen (Compensation)
    simulate <INTENT>       Saga simulieren ohne Ausführung
    list                    Aktive Sagas auflisten

OPTIONS (submit):
    --goal <GOAL>           Ziel-Zustand beschreiben
    --budget <BUDGET>       Maximales Budget
    --timeout <DUR>         Timeout (default: 1h)
    --auto-execute          Automatisch ausführen wenn bereit

BEISPIELE:
    # Komplexen Intent einreichen
    erynoa saga submit \
        --goal "Kaufe 500 kWh erneuerbare Energie" \
        --budget "150 EUR" \
        --auto-execute

    # Saga simulieren
    erynoa saga simulate \
        --goal "Transfer 100 USDC zu did:erynoa:guild:supplier"

    # Status prüfen
    erynoa saga status saga:sha3:abc123

AXIOM-REFERENZ:
    Κ22: Saga-Composer-Axiom
         ∀ Intent i : ∃! Saga S : resolve(i) = S
    Κ23: Gateway-Guard-Axiom
         cross(s, 𝒞₁, 𝒞₂) requires G(s, 𝒞₂) = true
    Κ24: Atomare Kompensation
         fail(Sᵢ) → compensate(S₁..Sᵢ₋₁)

OUTPUT (status):
    ┌─────────────────────────────────────────────────────────────────────────┐
    │ 🔄 SAGA STATUS: saga:sha3:abc123                                        │
    ├─────────────────────────────────────────────────────────────────────────┤
    │ Intent:   "Kaufe 500 kWh erneuerbare Energie"                           │
    │ Budget:   150 EUR (verbraucht: 125 EUR)                                 │
    │ Status:   IN_PROGRESS (3/4 Schritte)                                    │
    │ Timeout:  45:32 verbleibend                                             │
    │                                                                         │
    │ SAGA-SCHRITTE:                                                          │
    │                                                                         │
    │ S₁ ✓ Lock USDC                                                          │
    │    Status:    COMPLETED                                                 │
    │    TX:        0xabc123... (Ethereum)                                    │
    │    Amount:    150 USDC locked                                           │
    │    Duration:  2.3s                                                      │
    │                                                                         │
    │ S₂ ✓ Mint wEUR                                                          │
    │    Status:    COMPLETED                                                 │
    │    TX:        event:sha3:def456... (Erynoa DAG)                         │
    │    Amount:    138 wEUR minted                                           │
    │    Duration:  0.8s                                                      │
    │                                                                         │
    │ S₃ ✓ Gateway-Check (Κ23)                                                │
    │    Status:    COMPLETED                                                 │
    │    From:      realm:finance → realm:energy                              │
    │    Guards:    [Human: ✓] [Trust ≥ 0.6: ✓] [Compliance: ✓]              │
    │                                                                         │
    │ S₄ ⏳ Execute Energy Purchase                                           │
    │    Status:    PENDING                                                   │
    │    Target:    did:erynoa:guild:greenpower                               │
    │    Amount:    500 kWh @ 0.25 EUR/kWh                                    │
    │    ETA:       ~30s                                                      │
    │                                                                         │
    │ COMPENSATION PLAN (bei Fehler):                                         │
    │    S₄ fail → burn wEUR → unlock USDC → refund                          │
    │                                                                         │
    └─────────────────────────────────────────────────────────────────────────┘

OUTPUT (simulate):
    ┌─────────────────────────────────────────────────────────────────────────┐
    │ 🧪 SAGA SIMULATION                                                      │
    ├─────────────────────────────────────────────────────────────────────────┤
    │ Intent: "Transfer 100 USDC zu did:erynoa:guild:supplier"                │
    │                                                                         │
    │ DEPENDENCY GRAPH:                                                       │
    │                                                                         │
    │    ┌─────────────┐                                                      │
    │    │ Goal:       │                                                      │
    │    │ 100 USDC    │                                                      │
    │    │ @ supplier  │                                                      │
    │    └──────┬──────┘                                                      │
    │           │                                                             │
    │    ┌──────▼──────┐     ┌─────────────┐                                  │
    │    │ Benötigt:   │────►│ Realm-Cross │                                  │
    │    │ 100 wUSDC   │     │ finance →   │                                  │
    │    │ @ finance   │     │ trade       │                                  │
    │    └──────┬──────┘     └─────────────┘                                  │
    │           │                                                             │
    │    ┌──────▼──────┐                                                      │
    │    │ Benötigt:   │                                                      │
    │    │ 100 USDC    │                                                      │
    │    │ @ Ethereum  │                                                      │
    │    │ (vorhanden) │                                                      │
    │    └─────────────┘                                                      │
    │                                                                         │
    │ GEPLANTE SCHRITTE:                                                      │
    │    S₁: lock(100 USDC, Ethereum)         ~ 15s, ~$2.50 gas               │
    │    S₂: mint(100 wUSDC, Erynoa)          ~ 1s, kostenlos                 │
    │    S₃: gateway(finance → trade)         ~ 0.5s                          │
    │    S₄: transfer(100 wUSDC, supplier)    ~ 1s                            │
    │                                                                         │
    │ KOSTEN-SCHÄTZUNG:                                                       │
    │    Gas (Ethereum):  ~$2.50                                              │
    │    Erynoa Fees:     0 (kostenlos)                                       │
    │    Gesamt:          ~$2.50                                              │
    │                                                                         │
    │ RISIKO-ANALYSE:                                                         │
    │    P(success):      98.2%                                               │
    │    P(rollback):     1.5%                                                │
    │    P(timeout):      0.3%                                                │
    │                                                                         │
    └─────────────────────────────────────────────────────────────────────────┘
```

---

## VII. Governance-Befehle (Κ16-Κ17, Κ21)

### `erynoa governance`

Verwaltet quadratische Governance gemäß **Κ21 (Quadratisches Voting)**.

```bash
erynoa governance <COMMAND> [OPTIONS]

COMMANDS:
    proposal <ACTION>       Proposal erstellen/anzeigen
      create                Neuen Proposal erstellen
      list                  Alle Proposals auflisten
      info <ID>             Proposal-Details
    vote <PROPOSAL_ID>      Abstimmen
    veto <PROPOSAL_ID>      Veto einlegen (wenn berechtigt)
    delegate <DID>          Stimmrecht delegieren

OPTIONS (proposal create):
    --type <TYPE>           Proposal-Typ
                            Werte: rule-change, parameter, membership, emergency
    --title <TITLE>         Titel
    --description <DESC>    Beschreibung
    --realm <REALM>         Ziel-Realm
    --duration <DUR>        Abstimmungsdauer (default: 7d)

OPTIONS (vote):
    --weight <W>            Stimmgewicht (wird quadratisch verrechnet)
    --direction <D>         Richtung (for, against, abstain)

BEISPIELE:
    # Proposal erstellen
    erynoa governance proposal create \
        --type rule-change \
        --title "Erhöhe min-trust auf 0.4" \
        --realm "realm:erynoa:eu-trade" \
        --duration 14d

    # Abstimmen mit quadratischem Voting
    erynoa governance vote proposal:sha3:abc123 \
        --weight 4 \
        --direction for
        # Kosten: √4 = 2 Voting-Credits

AXIOM-REFERENZ:
    Κ21: vote_power(s) = √(credits_spent(s))
    Κ16: Rules: Ψ → (vote, rules)
    Κ17: Emergency: Ψ_emergency ⊃ Ψ_normal

OUTPUT (vote):
    ┌─────────────────────────────────────────────────────────────────────────┐
    │ 🗳️ ABSTIMMUNG                                                           │
    ├─────────────────────────────────────────────────────────────────────────┤
    │ Proposal:   "Erhöhe min-trust auf 0.4"                                  │
    │ ID:         proposal:sha3:abc123                                        │
    │                                                                         │
    │ Deine Stimme:                                                           │
    │   Direction:  FOR                                                       │
    │   Weight:     4 Credits → √4 = 2.0 Voting Power                         │
    │                                                                         │
    │ Aktueller Stand:                                                        │
    │                                                                         │
    │   FOR:      ████████████████████░░░░░░░░░░ 65.3% (127.4 VP)            │
    │   AGAINST:  ████████████░░░░░░░░░░░░░░░░░░ 28.7% (56.2 VP)             │
    │   ABSTAIN:  ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 6.0% (11.8 VP)              │
    │                                                                         │
    │   Teilnehmer: 89 DIDs                                                   │
    │   Quorum:     ✓ erreicht (> 50 DIDs)                                    │
    │   Verbleibend: 5d 12h 30m                                               │
    │                                                                         │
    │ Prognose:                                                               │
    │   P(pass) = 78.4%                                                       │
    │                                                                         │
    └─────────────────────────────────────────────────────────────────────────┘
```

---

## VIII. Schutz-Befehle (Κ19-Κ21)

### `erynoa protection`

Verwaltet Schutz-Mechanismen gegen System-Degeneration.

```bash
erynoa protection <COMMAND> [OPTIONS]

COMMANDS:
    anti-calcification      Anti-Verkalkung Status (Κ19)
    diversity               Diversity-Monitor (Κ20)
    anomaly                 Anomalie-Detektion
    fairness                Fairness-Metriken

OPTIONS:
    --realm <REALM>         Realm-spezifische Analyse
    --detailed              Detaillierte Ausgabe
    --suggest               Verbesserungsvorschläge generieren

BEISPIELE:
    erynoa protection anti-calcification --realm "realm:erynoa:eu-trade"
    erynoa protection diversity --detailed
    erynoa protection anomaly --suggest

AXIOM-REFERENZ:
    Κ19: ∃ f: Established × Fresh → Resource : f(e,f) > f(e,e)
    Κ20: D(𝒞) = H(distribution) / H_max
    Κ21: vote_power(s) = √(credits_spent(s))

OUTPUT (anti-calcification):
    ┌─────────────────────────────────────────────────────────────────────────┐
    │ 🛡️ ANTI-CALCIFICATION STATUS (Κ19)                                      │
    │    Realm: eu-trade                                                      │
    ├─────────────────────────────────────────────────────────────────────────┤
    │                                                                         │
    │ TIER-VERTEILUNG:                                                        │
    │                                                                         │
    │   ESTABLISHED (Trust ≥ 0.8):                                            │
    │     Count: 234 (23.4%)                                                  │
    │     Avg Activity: 0.85                                                  │
    │     Resources: 45% of total                                             │
    │                                                                         │
    │   GROWING (0.5 ≤ Trust < 0.8):                                          │
    │     Count: 412 (41.2%)                                                  │
    │     Avg Activity: 0.72                                                  │
    │     Resources: 38% of total                                             │
    │                                                                         │
    │   EMERGING (0.3 ≤ Trust < 0.5):                                         │
    │     Count: 289 (28.9%)                                                  │
    │     Avg Activity: 0.54                                                  │
    │     Resources: 14% of total                                             │
    │                                                                         │
    │   FRESH (Trust < 0.3):                                                  │
    │     Count: 65 (6.5%)                                                    │
    │     Avg Activity: 0.31                                                  │
    │     Resources: 3% of total                                              │
    │                                                                         │
    │ DIVERSITY-SLOTS AKTIV:                                                  │
    │   ✓ 5% Ressourcen für FRESH reserviert                                  │
    │   ✓ Interaktions-Bonus: ESTABLISHED × FRESH +20%                        │
    │                                                                         │
    │ GINI-KOEFFIZIENT: 0.34 (gut, < 0.5 = gesund)                           │
    │                                                                         │
    │ ⚠️ WARNUNG: FRESH-Anteil sinkt (war 8.2% vor 30d)                       │
    │    Empfehlung: Onboarding-Kampagne starten                              │
    │                                                                         │
    └─────────────────────────────────────────────────────────────────────────┘

OUTPUT (diversity):
    ┌─────────────────────────────────────────────────────────────────────────┐
    │ 🌈 DIVERSITY MONITOR (Κ20)                                              │
    ├─────────────────────────────────────────────────────────────────────────┤
    │                                                                         │
    │ D(𝒞) = H(distribution) / H_max                                          │
    │                                                                         │
    │ NAMESPACE-VERTEILUNG:                                                   │
    │   self:    ████████████████████░░░░░░░░░░ 62% (H=0.89)                 │
    │   guild:   ████████░░░░░░░░░░░░░░░░░░░░░░ 24% (H=0.72)                 │
    │   spirit:  ███░░░░░░░░░░░░░░░░░░░░░░░░░░░ 8%  (H=0.45)                 │
    │   thing:   ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 6%  (H=0.38)                 │
    │                                                                         │
    │   Gesamt-Diversity: D = 0.74 (gut)                                      │
    │                                                                         │
    │ AKTIVITÄTS-DIVERSITY:                                                   │
    │   Top 10%:  ████████████████░░░░░░░░░░░░░░ 52% der Events              │
    │   Mittel:   ████████████░░░░░░░░░░░░░░░░░░ 38%                         │
    │   Bottom:   ███░░░░░░░░░░░░░░░░░░░░░░░░░░░ 10%                         │
    │                                                                         │
    │   Aktivitäts-Gini: 0.41 (akzeptabel)                                   │
    │                                                                         │
    │ CROSS-REALM INTERAKTIONEN:                                              │
    │   finance ↔ energy:   23%                                               │
    │   finance ↔ gaming:   5%                                                │
    │   energy ↔ logistics: 18%                                               │
    │                                                                         │
    │   Cross-Realm-Index: 0.31 (Raum für Verbesserung)                      │
    │                                                                         │
    └─────────────────────────────────────────────────────────────────────────┘
```

---

## IX. Weltformel-Befehle (Κ15a-d)

### `erynoa formula`

Berechnet und analysiert die Weltformel V2.0.

```bash
erynoa formula <COMMAND> [OPTIONS]

COMMANDS:
    compute <DID>           𝔼-Beitrag einer Entität berechnen
    status                  Globaler Weltformel-Status
    components <DID>        Einzelne Komponenten analysieren
    simulate <EVENT>        Simuliere Event-Auswirkung
    leaderboard             Top-Beiträge zur Weltformel

OPTIONS:
    --realm <REALM>         Realm-spezifische Berechnung
    --time-window <DUR>     Zeitfenster für Berechnung
    --approximation <ALG>   Approximations-Algorithmus (Κ15d)
                            Werte: exact, bloom, cms (Count-Min Sketch)
    --mobile                Low-Power-Modus für Mobile (τ=30d)

BEISPIELE:
    erynoa formula compute did:erynoa:self:alice
    erynoa formula status --realm "realm:erynoa:eu-trade"
    erynoa formula simulate --event "type:transfer,amount:1000"
    erynoa formula leaderboard --limit 20

AXIOM-REFERENZ:
    Κ15a: 𝒮(s) = ‖𝕎(s)‖² · ℐ(s)  (Trust-gedämpfte Surprisal)
    Κ15b: 𝔼 = Σ 𝔸(s) · σ⃗( ‖𝕎(s)‖_w · ln|ℂ(s)| · 𝒮(s) ) · Ĥ(s) · w(s,t)
    Κ15c: σ⃗(x) = 1 / (1 + e^(-x))  (Sigmoid für Sättigung)
    Κ15d: Count-Min Sketch für ℐ-Approximation

OUTPUT (status):
    ╔════════════════════════════════════════════════════════════════════════╗
    ║                     WELTFORMEL V2.0 STATUS                             ║
    ║                     Realm: eu-trade                                    ║
    ╠════════════════════════════════════════════════════════════════════════╣
    ║                                                                        ║
    ║   𝔼 = Σ 𝔸(s) · σ⃗( ‖𝕎(s)‖_w · ln|ℂ(s)| · 𝒮(s) ) · Ĥ(s) · w(s,t)       ║
    ║                                                                        ║
    ╠════════════════════════════════════════════════════════════════════════╣
    ║   GLOBALE METRIKEN                                                     ║
    ║   ───────────────────────────────────────────────────────────────────  ║
    ║                                                                        ║
    ║   𝔼_total = 12,847.32                                                  ║
    ║   Δ𝔼 (24h) = +127.45 (+0.99%)                                          ║
    ║                                                                        ║
    ║   Entitäten: 1,000                                                     ║
    ║   Events (τ=90d): 2.4M                                                 ║
    ║   Avg 𝔸: 0.67                                                          ║
    ║   Avg ‖𝕎‖: 0.72                                                        ║
    ║                                                                        ║
    ╠════════════════════════════════════════════════════════════════════════╣
    ║   KOMPONENTEN-VERTEILUNG                                               ║
    ║   ───────────────────────────────────────────────────────────────────  ║
    ║                                                                        ║
    ║   Beitrag durch 𝔸 (Aktivität):     ████████████░░░░░░░░ 42%            ║
    ║   Beitrag durch 𝕎 (Trust):         ████████████████░░░░ 35%            ║
    ║   Beitrag durch Ĥ (Human-Bonus):   ████░░░░░░░░░░░░░░░░ 15%            ║
    ║   Beitrag durch 𝒮 (Surprisal):     ██░░░░░░░░░░░░░░░░░░ 8%             ║
    ║                                                                        ║
    ╠════════════════════════════════════════════════════════════════════════╣
    ║   Ĥ HUMAN-BONUS VERTEILUNG                                             ║
    ║   ───────────────────────────────────────────────────────────────────  ║
    ║                                                                        ║
    ║   Ĥ = 1.5 (full attestation): 12.3% (123 DIDs)                        ║
    ║   Ĥ = 1.2 (basic attestation): 34.7% (347 DIDs)                       ║
    ║   Ĥ = 1.0 (nicht verifiziert): 53.0% (530 DIDs)                       ║
    ║                                                                        ║
    ║   Human-Bonus-Beitrag zum 𝔼: +1,927.09 (+15%)                         ║
    ║                                                                        ║
    ╠════════════════════════════════════════════════════════════════════════╣
    ║   SURPRISAL ANTI-HYPE (Κ15a)                                           ║
    ║   ───────────────────────────────────────────────────────────────────  ║
    ║                                                                        ║
    ║   𝒮(s) = ‖𝕎(s)‖² · ℐ(s)                                                ║
    ║                                                                        ║
    ║   Hoher Trust dämpft Überraschung:                                     ║
    ║     ‖𝕎‖=0.9 → 𝒮-Faktor = 0.81 (gedämpft)                               ║
    ║     ‖𝕎‖=0.3 → 𝒮-Faktor = 0.09 (stark gedämpft)                         ║
    ║                                                                        ║
    ║   Anti-Hype-Effekt: Neue Entitäten mit niedrigem Trust                ║
    ║   können nicht durch pure Aktivität dominieren.                        ║
    ║                                                                        ║
    ╠════════════════════════════════════════════════════════════════════════╣
    ║   COUNT-MIN SKETCH (Κ15d)                                              ║
    ║   ───────────────────────────────────────────────────────────────────  ║
    ║                                                                        ║
    ║   Modus: cms (Count-Min Sketch)                                        ║
    ║   Parameter: w=2^20, d=7                                               ║
    ║   Fehler: ε ≤ 0.0001 (0.01%)                                           ║
    ║   Konfidenz: 1 - δ = 99.999%                                           ║
    ║   Speicher: 28 MB (vs. 2.4 GB exakt)                                   ║
    ║                                                                        ║
    ╚════════════════════════════════════════════════════════════════════════╝

OUTPUT (leaderboard):
    ┌─────────────────────────────────────────────────────────────────────────┐
    │ 🏆 WELTFORMEL LEADERBOARD                                               │
    │    Realm: eu-trade | Zeitraum: 30d                                      │
    ├─────────────────────────────────────────────────────────────────────────┤
    │ #  │ DID                          │ 𝔼-Beitrag │ 𝔸    │ ‖𝕎‖  │ Ĥ   │
    ├────┼──────────────────────────────┼───────────┼──────┼──────┼─────┤
    │  1 │ did:erynoa:guild:greenpower  │   127.45  │ 0.94 │ 0.91 │ 1.5 │
    │  2 │ did:erynoa:guild:consortium  │   118.32  │ 0.89 │ 0.88 │ 1.2 │
    │  3 │ did:erynoa:self:alice        │    98.76  │ 0.81 │ 0.78 │ 1.2 │
    │  4 │ did:erynoa:guild:solarstart  │    87.21  │ 0.77 │ 0.65 │ 1.0 │
    │  5 │ did:erynoa:spirit:tradingbot │    76.54  │ 0.95 │ 0.82 │ 1.0 │
    │ ...│ ...                          │    ...    │ ...  │ ...  │ ... │
    │ 20 │ did:erynoa:self:zoe          │    34.12  │ 0.62 │ 0.71 │ 1.2 │
    └─────────────────────────────────────────────────────────────────────────┘

    📊 Dein Rang: #3 (98.76)
       Δ gegenüber Vormonat: +12 Plätze ↑
```

---

## X. Netzwerk-Befehle

### `erynoa peer`

Verwaltet Peer-Node-Verbindungen.

```bash
erynoa peer <COMMAND> [OPTIONS]

COMMANDS:
    status                  Peer-Status anzeigen
    info                    Detaillierte Peer-Informationen
    sync                    Mit Netzwerk synchronisieren
    list                    Verbundene Peers auflisten
    connect <PEER_ID>       Mit spezifischem Peer verbinden
    disconnect <PEER_ID>    Verbindung trennen

OPTIONS (sync):
    --partition <PART>      Nur spezifische Partition synchronisieren
    --full                  Vollständige Synchronisation
    --verify                Alle Daten verifizieren

BEISPIELE:
    erynoa peer status
    erynoa peer sync --partition finance --verify
    erynoa peer list --detailed
```

### `erynoa remote`

Verwaltet Remote-Konfigurationen.

```bash
erynoa remote <COMMAND> [OPTIONS]

COMMANDS:
    add <NAME> <URL>        Remote hinzufügen
    remove <NAME>           Remote entfernen
    list                    Remotes auflisten
    set-default <NAME>      Default-Remote setzen

BEISPIELE:
    erynoa remote add mainnet "https://peer.erynoa.network"
    erynoa remote list
```

---

## XI. Konfigurations-Befehle

### `erynoa config`

Verwaltet CLI-Konfiguration.

```bash
erynoa config <COMMAND> [OPTIONS]

COMMANDS:
    set <KEY> <VALUE>       Konfiguration setzen
    get <KEY>               Konfiguration auslesen
    list                    Alle Konfigurationen auflisten
    reset                   Auf Defaults zurücksetzen
    profile <ACTION>        Profile verwalten

WICHTIGE KEYS:
    default-realm           Default-Realm für Operationen
    default-algorithm       Default-Schlüssel-Algorithmus
    sync-interval           Sync-Intervall in Sekunden
    mobile-mode             Low-Power-Modus für Mobile
    surprisal-algorithm     cms | bloom | exact

BEISPIELE:
    erynoa config set default-realm "realm:erynoa:eu-trade"
    erynoa config set mobile-mode true
    erynoa config set surprisal-algorithm cms
    erynoa config get sync-interval
```

### `erynoa alias`

Verwaltet Befehlsaliase.

```bash
erynoa alias <NAME> <COMMAND>

BEISPIELE:
    erynoa alias status "status --brief"
    erynoa alias push-wait "push --wait --min-finality 0.99"
```

---

## XII. Diagnose-Befehle

### `erynoa inspect`

Inspiziert System-Komponenten detailliert.

```bash
erynoa inspect <COMPONENT> [OPTIONS]

COMPONENTS:
    dag                     Event-DAG-Struktur
    trust                   Trust-Engine-Zustand
    consensus               Konsens-Engine
    protection              Schutz-Mechanismen
    cache                   LRU-Cache-Status

OPTIONS:
    --detailed              Detaillierte Ausgabe
    --json                  JSON-Ausgabe
    --realm <REALM>         Realm-spezifisch

BEISPIELE:
    erynoa inspect dag --detailed
    erynoa inspect trust --realm "realm:erynoa:eu-trade"
    erynoa inspect cache
```

### `erynoa verify`

Verifiziert Datenintegrität.

```bash
erynoa verify <TARGET> [OPTIONS]

TARGETS:
    event <EVENT_ID>        Einzelnes Event verifizieren
    chain <FROM>..<TO>      Event-Kette verifizieren
    merkle <ROOT>           Merkle-Proof verifizieren
    trust <DID>             Trust-Berechnung verifizieren

OPTIONS:
    --deep                  Tiefe Verifikation (alle Abhängigkeiten)
    --report                Verifikations-Bericht generieren

BEISPIELE:
    erynoa verify event event:sha3:abc123
    erynoa verify chain "HEAD~100..HEAD" --deep
```

### `erynoa audit`

Führt Audit-Funktionen aus.

```bash
erynoa audit <TYPE> [OPTIONS]

TYPES:
    trust <DID>             Trust-Audit für Entität
    realm <REALM>           Realm-Compliance-Audit
    governance              Governance-Audit
    formula                 Weltformel-Konsistenz-Audit

OPTIONS:
    --export <FILE>         Audit-Bericht exportieren
    --period <DUR>          Audit-Zeitraum

BEISPIELE:
    erynoa audit trust did:erynoa:self:alice --export audit-report.json
    erynoa audit realm "realm:erynoa:eu-trade" --period 90d
```

### `erynoa benchmark`

Führt Performance-Benchmarks aus.

```bash
erynoa benchmark <COMPONENT> [OPTIONS]

COMPONENTS:
    formula                 Weltformel-Berechnung
    consensus               Konsens-Engine
    trust                   Trust-Berechnung
    sync                    Netzwerk-Synchronisation

OPTIONS:
    --iterations <N>        Anzahl Iterationen (default: 100)
    --warm-up <N>           Warm-up Iterationen (default: 10)
    --output <FILE>         Ergebnisse speichern

BEISPIELE:
    erynoa benchmark formula --iterations 1000
    erynoa benchmark consensus
```

---

## XIII. Umgebungsvariablen

```bash
# Identität
ERYNOA_DID              # Aktive DID (überschreibt --did)
ERYNOA_KEYFILE          # Pfad zur Schlüsseldatei
ERYNOA_SEED             # BIP39 Seed (⚠️ nur für Entwicklung!)

# Netzwerk
ERYNOA_ENDPOINT         # API-Endpoint URL
ERYNOA_PEER_ID          # Bevorzugter Peer
ERYNOA_NETWORK          # mainnet | testnet | local

# Realm
ERYNOA_DEFAULT_REALM    # Default-Realm für Operationen

# Performance
ERYNOA_CACHE_SIZE       # LRU-Cache-Größe in MB (default: 256)
ERYNOA_CMS_WIDTH        # Count-Min Sketch Breite (default: 2^18)
ERYNOA_CMS_DEPTH        # Count-Min Sketch Tiefe (default: 5)
ERYNOA_MOBILE_MODE      # true | false

# Logging
ERYNOA_LOG_LEVEL        # trace | debug | info | warn | error
ERYNOA_LOG_FORMAT       # json | pretty

# Entwicklung
ERYNOA_DEV_MODE         # Entwicklungsmodus aktivieren
```

---

## XIV. Exit-Codes

| Code | Bedeutung                             |
| ---- | ------------------------------------- |
| 0    | Erfolg                                |
| 1    | Allgemeiner Fehler                    |
| 2    | Ungültige Argumente                   |
| 3    | Authentifizierung fehlgeschlagen      |
| 4    | Autorisierung fehlgeschlagen          |
| 5    | Netzwerkfehler                        |
| 6    | Konsens nicht erreicht                |
| 7    | Trust-Prüfung fehlgeschlagen          |
| 8    | Realm-Crossing verweigert             |
| 9    | Gateway-Guard verweigert              |
| 10   | Saga-Kompensation ausgelöst           |
| 11   | Weltformel-Validierung fehlgeschlagen |
| 12   | Timeout                               |
| 64   | Interner Fehler                       |

---

## XV. Bash/Zsh Completion

```bash
# Bash
source <(erynoa completion bash)

# Zsh
source <(erynoa completion zsh)

# Fish
erynoa completion fish | source

# PowerShell
erynoa completion powershell | Out-String | Invoke-Expression
```

---

## Appendix A: Axiom-Schnellreferenz

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                           ERYNOA V4.1 AXIOM-SCHNELLREFERENZ                                           ║
╠════════════════════════════════════════════════════════════════════════════════════════════════════════╣
║                                                                                                        ║
║   REALM & STRUKTUR                                                                                     ║
║   Κ1:  rules(𝒞₁) ⊇ rules(𝒞₂)                    Monotone Regelvererbung                               ║
║                                                                                                        ║
║   TRUST (6D-Vektor 𝕎)                                                                                  ║
║   Κ2:  𝕋(g ∘ f) = 𝕋(f) ∘ 𝕋(g)                   Trust-Funktor (kontravariant)                         ║
║   Κ3:  ∂𝕎ᵢ/∂event ⊥ ∂𝕎ⱼ/∂event                 Dimensionale Unabhängigkeit                           ║
║   Κ4:  Δ⁻ = λ · Δ⁺                              Asymmetrische Evolution                               ║
║   Κ5:  t₁ ⊕ t₂ = 1 - (1-t₁)(1-t₂)              Probabilistische Kombination                          ║
║                                                                                                        ║
║   IDENTITÄT                                                                                           ║
║   Κ6:  ∃! did : identity(e) = did               Existenz-Eindeutigkeit                                ║
║   Κ7:  ⟨s⟩ → □⟨s⟩                               Permanenz mit Aktivität                               ║
║   Κ8:  s ⊳ s' → 𝕋(s') ≤ 𝕋(s)                   Delegations-Struktur                                  ║
║                                                                                                        ║
║   KAUSALITÄT (DAG ℂ)                                                                                   ║
║   Κ9:  ℂ = (E, ⊲) ist DAG                       Kausale Struktur                                      ║
║   Κ10: ⟦e⟧ → □⟦e⟧                               Bezeugung-Finalität                                   ║
║   Κ11: {pre} Π {post}                           Prozess-Korrektheit                                   ║
║   Κ12: ∀Π : ⟦Π⟧ → Δ|ℂ| ≥ 1                     Event-Erzeugung                                       ║
║                                                                                                        ║
║   TRANSAKTIONEN                                                                                        ║
║   Κ13: TAT = (seek, propose, agree, exec, settle)                                                     ║
║   Κ14: stream: ∀t : delivered(t) = ∫₀ᵗ rate(τ)dτ                                                      ║
║                                                                                                        ║
║   WELTFORMEL V2.0                                                                                      ║
║   Κ15a: 𝒮(s) = ‖𝕎(s)‖² · ℐ(s)                  Trust-gedämpfte Surprisal                             ║
║   Κ15b: 𝔼 = Σ 𝔸·σ⃗(‖𝕎‖·ln|ℂ|·𝒮)·Ĥ·w             Weltformel                                            ║
║   Κ15c: σ⃗(x) = 1/(1+e⁻ˣ)                       Sigmoid-Sättigung                                     ║
║   Κ15d: CMS für ℐ-Approximation                 Skalierung                                            ║
║                                                                                                        ║
║   GOVERNANCE                                                                                           ║
║   Κ16: Ψ → (vote, rules)                        Governance-Funktion                                   ║
║   Κ17: Ψ_emergency ⊃ Ψ_normal                   Emergency-Governance                                  ║
║   Κ18: Ψ(𝒫, e) = Σ sign(v,e)·w(v) / Σw          Partition-Konsens                                     ║
║                                                                                                        ║
║   SCHUTZ (Anti-Degeneration)                                                                          ║
║   Κ19: f(established, fresh) > f(e, e)          Anti-Calcification                                    ║
║   Κ20: D(𝒞) = H(dist) / H_max                   Diversity-Monitor                                     ║
║   Κ21: vote_power = √credits                    Quadratisches Voting                                  ║
║                                                                                                        ║
║   PEER-LOGIK                                                                                          ║
║   Κ22: ∀ Intent i : ∃! Saga S                   Saga-Composer                                         ║
║   Κ23: cross(s,𝒞₁,𝒞₂) requires G(s,𝒞₂)         Gateway-Guard                                         ║
║   Κ24: fail(Sᵢ) → compensate(S₁..Sᵢ₋₁)         Atomare Kompensation                                  ║
║                                                                                                        ║
║   HUMAN-ALIGNED                                                                                        ║
║   Κ25: Human → Ĥ ∈ {1.0, 1.2, 1.5}             Human-Bonus                                            ║
║   Κ26: ∃ appeal(decision)                       Menschliches Einspruchsrecht                          ║
║   Κ27: autonomous_action → human_audit          Autonomie-Grenzen                                     ║
║   Κ28: privacy(personal_data) ≥ threshold       Datenschutz-Minimum                                   ║
║                                                                                                        ║
║   META-AXIOM                                                                                          ║
║   Μ1:  Partielle Ordnung = Irreflexiv ∧ Antisym ∧ Transitiv                                          ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

## Appendix B: Weltformel V2.0 Berechnung

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                           WELTFORMEL V2.0 BERECHNUNGS-WORKFLOW                                        ║
╠════════════════════════════════════════════════════════════════════════════════════════════════════════╣
║                                                                                                        ║
║   SCHRITT 1: Aktivität berechnen                                                                      ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────── ║
║                                                                                                        ║
║       𝔸(s) = |{e ∈ ℂ(s) : age(e) < τ}| / (|{e ∈ ℂ(s) : age(e) < τ}| + κ)                             ║
║                                                                                                        ║
║       Parameter:                                                                                      ║
║           τ = 90d (Full Node), 30d (Mobile)                                                           ║
║           κ = 10 (Aktivitäts-Schwelle)                                                                ║
║                                                                                                        ║
║   SCHRITT 2: Trust-Norm berechnen                                                                     ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────── ║
║                                                                                                        ║
║       ‖𝕎(s)‖_w = √(Σᵢ wᵢ · 𝕎ᵢ²) / √(Σᵢ wᵢ)                                                          ║
║                                                                                                        ║
║       Gewichte:                                                                                       ║
║           w_R = 1.0 (Reliability)                                                                     ║
║           w_I = 1.2 (Integrity)                                                                       ║
║           w_C = 0.8 (Competence)                                                                      ║
║           w_P = 0.6 (Prestige)                                                                        ║
║           w_V = 1.5 (Vigilance)                                                                       ║
║           w_Ω = 2.0 (Omega)                                                                           ║
║                                                                                                        ║
║   SCHRITT 3: Surprisal berechnen (Κ15a)                                                               ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────── ║
║                                                                                                        ║
║       ℐ(s) = -log₂(f(s))    wobei f(s) = Count-Min-Sketch(s) / total_events                          ║
║                                                                                                        ║
║       𝒮(s) = ‖𝕎(s)‖² · ℐ(s)   ← Trust-Dämpfung (Anti-Hype)                                           ║
║                                                                                                        ║
║   SCHRITT 4: Sigmoid anwenden (Κ15c)                                                                  ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────── ║
║                                                                                                        ║
║       x = ‖𝕎(s)‖_w · ln|ℂ(s)| · 𝒮(s)                                                                 ║
║       σ⃗(x) = 1 / (1 + e^(-x))                                                                        ║
║                                                                                                        ║
║   SCHRITT 5: Finale Berechnung (Κ15b)                                                                 ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────── ║
║                                                                                                        ║
║       𝔼(s) = 𝔸(s) · σ⃗(x) · Ĥ(s) · w(s,t)                                                             ║
║                                                                                                        ║
║       Ĥ(s) ∈ {1.0, 1.2, 1.5}   (Human-Bonus)                                                         ║
║       w(s,t) = Gewichtungsfaktor (z.B. 1/N für Normalisierung)                                       ║
║                                                                                                        ║
║       𝔼_total = Σₛ 𝔼(s)                                                                               ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

**Erynoa CLI V4.1** – Dezentrales Vertrauen für Menschen, Organisationen und KI-Agenten.

_Basierend auf 28 Kern-Axiomen. Formal verifiziert mit TLA+._
