# CLI Reference

> **Version:** V5.0 – Konsolidiert
> **Axiom-Basis:** Κ1-Κ28
> **Status:** Referenz

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

| Axiom                | Primäre Befehle                     | Funktion                     |
| -------------------- | ----------------------------------- | ---------------------------- |
| Κ1 (Regelvererbung)  | `realm create`, `realm rules`       | Realm-Hierarchie verwalten   |
| Κ2-Κ5 (Trust)        | `trust`, `attest`, `delegate`       | Trust-Vektor 𝕎 aktualisieren |
| Κ6-Κ8 (Identität)    | `init`, `sub-identity`, `key`       | DIDs und Delegationen        |
| Κ9-Κ12 (Kausalität)  | `commit`, `push`, `log`             | Event-DAG ℂ verwalten        |
| Κ13-Κ14 (TAT)        | `seek`, `propose`, `agree`          | Transaktions-Lifecycle       |
| Κ15a-d (Weltformel)  | `formula compute`, `formula status` | 𝔼-Berechnung und Surprisal   |
| Κ16-Κ17 (Humanismus) | `init --human`, `trust inspect --Ĥ` | Human-Aligned Mechanismen    |
| Κ18 (Konsens)        | `peer sync`, `push --wait`          | Partition-Konsens            |
| Κ19-Κ21 (Schutz)     | `protection *`                      | Anti-Degeneration            |
| Κ22-Κ24 (Peer)       | `saga *`, `peer *`                  | Gateway + Composer           |

---

## I. Identitäts-Befehle (Κ6-Κ8)

### `erynoa init`

Erstellt eine neue Identität (DID).

```bash
erynoa init [OPTIONS]

OPTIONS:
    --namespace <NS>        Namespace (default: self)
                            Werte: self, guild, spirit, thing, vessel,
                                   source, craft, vault, pact, circle
    --algorithm <ALG>       Algorithmus (ed25519, secp256k1, bls12-381)
    --label <LABEL>         Menschenlesbares Label
    --recover <SEED>        Aus 24-Wort Seed-Phrase wiederherstellen
    --human                 Human-Attestation anfordern (Ĥ-Bonus)

BEISPIELE:
    erynoa init --label "Alice"
    erynoa init --namespace guild --label "Meine GmbH" --human
    erynoa init --namespace spirit --label "Trading-Bot"

AXIOM-REFERENZ:
    Κ6: ∀ entity e : ∃! did ∈ DID : identity(e) = did
    Κ7: ⟨s⟩ ∧ ⟦create(s)⟧ ⟹ □⟨s⟩
```

### `erynoa sub-identity`

Verwaltet delegierte Sub-Identitäten (Κ8).

```bash
erynoa sub-identity <COMMAND> [OPTIONS]

COMMANDS:
    create <NAME>           Neue Sub-Identität
    list                    Alle auflisten
    switch <NAME>           Wechseln
    delete <NAME>           Löschen

OPTIONS (create):
    --inherit-trust <F>     Trust-Vererbung (0.0-1.0)
    --realm <REALM>         Kontext-Beschränkung
    --permissions <PERMS>   transfer, attest, governance, all
    --expires <DUR>         Ablaufzeit

BEISPIELE:
    erynoa sub-identity create gaming --inherit-trust 0.7
    erynoa sub-identity create work --realm "realm:business" --expires 1y

AXIOM-REFERENZ:
    Κ8: s ⊳ s' → 𝕋(s') ≤ 𝕋(s)
```

### `erynoa key`

Schlüssel-Management.

```bash
erynoa key <COMMAND>

COMMANDS:
    list                    Alle Schlüssel
    generate                Neuen generieren
    rotate                  Primärschlüssel rotieren
    revoke <ID>             Widerrufen
    export <ID>             Exportieren
    derive <PATH>           HD-Schlüssel ableiten
```

---

## II. Realm-Befehle (Κ1)

### `erynoa realm`

Verwaltet die Realm-Hierarchie.

```bash
erynoa realm <COMMAND> [OPTIONS]

COMMANDS:
    list                    Alle zugänglichen Realms
    create <NAME>           Neues VirtualRealm
    join <ID>               Beitreten
    leave <ID>              Verlassen
    info <ID>               Details
    cross <SRC> <DST>       Übergang simulieren
    rules <ID>              Regelset anzeigen

OPTIONS (create):
    --parent <REALM>        Übergeordnetes Realm
    --rules <FILE>          ECLVM-Regeln
    --governance <TYPE>     quadratic, token, reputation
    --min-trust <T>         Minimaler Trust

BEISPIELE:
    erynoa realm create eu-trade --rules gdpr.ecl --governance quadratic
    erynoa realm cross "realm:gaming" "realm:finance"

AXIOM-REFERENZ:
    Κ1: ∀ 𝒞₁ ⊂ 𝒞₂ : rules(𝒞₁) ⊇ rules(𝒞₂)
```

---

## III. Event-Befehle (Κ9-Κ12)

### `erynoa commit`

Erstellt ein Event im DAG.

```bash
erynoa commit [OPTIONS]

OPTIONS:
    --type <TYPE>           transfer, mint, burn, claim, attest, proposal, vote
    --message/-m <MSG>      Beschreibung
    --parents <IDs>         Explizite Parents
    --realm <REALM>         Ziel-Realm
    --dry-run               Nur simulieren

AXIOM-REFERENZ:
    Κ9:  ℂ = (E, ⊲) ist DAG
    Κ12: ∀Π : ⟦Π⟧ → Δ|ℂ| ≥ 1
```

### `erynoa push`

Propagiert Events ins Netzwerk.

```bash
erynoa push [OPTIONS]

OPTIONS:
    --partition <PART>      Ziel-Partition
    --priority <PRIO>       low, normal, high
    --wait                  Auf Finalität warten
    --min-finality <F>      Minimale Finalität

AXIOM-REFERENZ:
    Κ10: ⟦e⟧ → □⟦e⟧
    Κ18: Ψ(𝒫, e) = Σᵢ sign(vᵢ, e) · w(vᵢ) / Σⱼ w(vⱼ)
```

### `erynoa status`

Zeigt Zustand gemäß Weltformel.

```bash
erynoa status [OPTIONS]

OPTIONS:
    --full                  Vollständige Analyse
    --component <C>         𝔸, 𝕎, ℂ, ℐ, 𝒮, Ĥ, 𝔼
    --json                  JSON-Ausgabe

OUTPUT:
    ╔════════════════════════════════════════════════════════════════════════╗
    ║   𝔸 AKTIVITÄT                                                          ║
    ║   |{e ∈ ℂ(s) : age(e) < τ}| = 42                                       ║
    ║   𝔸(s) = 42 / (42 + 10) = 0.81                                         ║
    ╠════════════════════════════════════════════════════════════════════════╣
    ║   𝕎 TRUST-VEKTOR (6D)                                                  ║
    ║   R: 0.85 | I: 0.92 | C: 0.75 | P: 0.68 | V: 0.50 | Ω: 0.90           ║
    ║   ‖𝕎‖_w = 0.78                                                         ║
    ╠════════════════════════════════════════════════════════════════════════╣
    ║   𝔼 WELTFORMEL-BEITRAG                                                 ║
    ║   𝔼 = 𝔸 · σ( ‖𝕎‖ · ln|ℂ| · 𝒮 ) · Ĥ · w = 0.00097                     ║
    ╚════════════════════════════════════════════════════════════════════════╝
```

### `erynoa log`

Event-Historie anzeigen.

```bash
erynoa log [OPTIONS]

OPTIONS:
    --limit <N>             Anzahl (default: 10)
    --type <TYPE>           Filtern nach Typ
    --graph                 DAG-Visualisierung
    --trust-impact          Trust-Änderungen

OUTPUT (--graph):
    *   7f8a9b [2026-01-31] transfer: Zahlung Q1 (Δ𝕎.R +0.02)
    |\
    | * 6e7d8c [2026-01-30] attest: Lieferung bestätigt
    * | 5c4b3a [2026-01-29] claim: Update
    |/
    *   4a3b2c [2026-01-28] transfer: Anzahlung
```

---

## IV. Trust-Befehle (Κ2-Κ5)

### `erynoa trust inspect`

Trust-Vektor analysieren.

```bash
erynoa trust inspect <DID> [OPTIONS]

OPTIONS:
    --context <REALM>       Kontext für Abfrage
    --weighted              Gewichtete Norm
    --Ĥ                     Human-Bonus anzeigen

OUTPUT:
    ┌─────────────────────────────────────────────────────────────────────────┐
    │ TRUST-ANALYSE: did:erynoa:guild:supplier                                │
    ├─────────────────────────────────────────────────────────────────────────┤
    │   R (Reliability):  ████████████████████░░░░ 0.82  ↑ +0.03 (7d)        │
    │   I (Integrity):    ███████████████████████░ 0.94  = +0.00 (7d)        │
    │   C (Competence):   ██████████████████░░░░░░ 0.78  ↑ +0.05 (7d)        │
    │   P (Prestige):     ████████████████░░░░░░░░ 0.65  ↓ -0.02 (7d)        │
    │   V (Vigilance):    ████████████████████░░░░ 0.80  = +0.00 (7d)        │
    │   Ω (Omega):        ███████████████████████░ 0.95  = +0.00 (7d)        │
    │                                                                         │
    │ ‖𝕎‖_w = 0.83 | Ĥ = 1.0                                                  │
    └─────────────────────────────────────────────────────────────────────────┘
```

### `erynoa trust path`

Trust-Pfad zwischen DIDs.

```bash
erynoa trust path <FROM> <TO>

OUTPUT:
    Pfad 1 (Länge 2):
      alice ──(0.85)──► consortium ──(0.92)──► supplier
      Ketten-Trust (Τ1): exp((ln(0.85) + ln(0.92)) / √2) = 0.81

    Kombinierter Trust (⊕): 0.95
```

### `erynoa attest`

Attestation erstellen.

```bash
erynoa attest <TARGET_DID> [OPTIONS]

OPTIONS:
    --dimension <DIM>       R, I, C, P, all
    --strength <S>          0.1-1.0
    --credential <VC>       Verifiable Credential
    --expires <DUR>         Ablaufzeit

BEISPIELE:
    erynoa attest did:erynoa:self:bob --dimension C --strength 0.8
```

### `erynoa delegate`

Trust delegieren (Κ8).

```bash
erynoa delegate <TARGET_DID> [OPTIONS]

OPTIONS:
    --scope <SCOPE>         attest, transfer, governance, all
    --factor <F>            Trust-Faktor (0.0-1.0)
    --realm <REALM>         Beschränkung
    --expires <DUR>         Ablauf
```

---

## V. Transaktions-Befehle (Κ13-Κ14)

### `erynoa seek`

Partner suchen.

```bash
erynoa seek <QUERY> [OPTIONS]

OPTIONS:
    --type <TYPE>           self, guild, spirit, thing
    --realm <REALM>         Beschränkung
    --min-trust <T>         Minimum Trust
    --diversity-bonus       Κ20 Slots priorisieren
    --sort <FIELD>          trust, surprisal, relevance

AXIOM-REFERENZ:
    Κ13: TAT = (seek, propose, agree, exec, settle)
    Κ20: D(𝒞) = H(distribution) / H_max
```

### `erynoa propose`

Angebot erstellen.

```bash
erynoa propose <TARGET_DID> [OPTIONS]

OPTIONS:
    --amount <AMT>          Menge
    --price <PRICE>         Preis
    --duration <DUR>        Laufzeit
    --streaming             Streaming-Modus (Κ14)
    --escrow <DID>          Escrow-Service
```

### `erynoa agree`

Auf Angebot reagieren.

```bash
erynoa agree <PROPOSAL_ID> [OPTIONS]

OPTIONS:
    --accept                Akzeptieren
    --reject                Ablehnen
    --counter <TERMS>       Gegenangebot
```

### `erynoa stream`

Streaming-Verträge verwalten.

```bash
erynoa stream <COMMAND> <CONTRACT_ID>

COMMANDS:
    status                  Status anzeigen
    pause                   Pausieren
    resume                  Fortsetzen
    abort                   Abbrechen
```

---

## VI. Saga-Befehle (Κ22-Κ24)

### `erynoa saga`

Multi-Step-Transaktionen.

```bash
erynoa saga <COMMAND> [OPTIONS]

COMMANDS:
    submit <INTENT>         Intent einreichen
    status <ID>             Status anzeigen
    execute <ID>            Manuell ausführen
    cancel <ID>             Abbrechen
    rollback <ID>           Zurückrollen (Κ24)
    simulate <INTENT>       Simulieren

OPTIONS (submit):
    --goal <GOAL>           Ziel-Zustand
    --budget <BUDGET>       Max Budget
    --timeout <DUR>         Timeout
    --auto-execute          Auto-Ausführung

BEISPIELE:
    erynoa saga submit --goal "Kaufe 500 kWh Energie" --budget "150 EUR"
    erynoa saga simulate --goal "Transfer 100 USDC zu supplier"

OUTPUT (status):
    ┌─────────────────────────────────────────────────────────────────────────┐
    │ 🔄 SAGA STATUS: saga:sha3:abc123                                        │
    ├─────────────────────────────────────────────────────────────────────────┤
    │ S₁ ✓ Lock USDC                    COMPLETED   2.3s                      │
    │ S₂ ✓ Mint wEUR                    COMPLETED   0.8s                      │
    │ S₃ ✓ Gateway-Check (Κ23)          COMPLETED   0.5s                      │
    │ S₄ ⏳ Execute Purchase            PENDING     ~30s ETA                  │
    │                                                                         │
    │ COMPENSATION PLAN: S₄ fail → burn wEUR → unlock USDC → refund          │
    └─────────────────────────────────────────────────────────────────────────┘

AXIOM-REFERENZ:
    Κ22: ∀ Intent i : ∃! Saga S : resolve(i) = S
    Κ23: cross(s, 𝒞₁, 𝒞₂) requires G(s, 𝒞₂) = true
    Κ24: fail(Sᵢ) → compensate(S₁..Sᵢ₋₁)
```

---

## VII. Governance-Befehle (Κ21)

### `erynoa governance`

Quadratische Governance.

```bash
erynoa governance <COMMAND> [OPTIONS]

COMMANDS:
    proposal create         Proposal erstellen
    proposal list           Alle Proposals
    proposal info <ID>      Details
    vote <ID>               Abstimmen
    veto <ID>               Veto (wenn berechtigt)
    delegate <DID>          Stimmrecht delegieren

OPTIONS (vote):
    --weight <W>            Stimmgewicht (quadratisch)
    --direction <D>         for, against, abstain

BEISPIELE:
    erynoa governance vote proposal:abc --weight 4 --direction for
    # Kosten: √4 = 2 Voting-Credits

AXIOM-REFERENZ:
    Κ21: vote_power(s) = √(credits_spent(s))
```

---

## VIII. Schutz-Befehle (Κ19-Κ21)

### `erynoa protection`

Schutz-Mechanismen.

```bash
erynoa protection <COMMAND> [OPTIONS]

COMMANDS:
    anti-calcification      Status (Κ19)
    diversity               Diversity-Monitor (Κ20)
    anomaly                 Anomalie-Detektion
    fairness                Fairness-Metriken

OPTIONS:
    --realm <REALM>         Realm-spezifisch
    --detailed              Ausführlich
    --suggest               Verbesserungen

OUTPUT (anti-calcification):
    ┌─────────────────────────────────────────────────────────────────────────┐
    │ 🛡️ ANTI-CALCIFICATION STATUS (Κ19)                                      │
    ├─────────────────────────────────────────────────────────────────────────┤
    │ ESTABLISHED (≥0.8):  234 (23.4%) | Resources: 45%                       │
    │ GROWING (0.5-0.8):   412 (41.2%) | Resources: 38%                       │
    │ EMERGING (0.3-0.5):  289 (28.9%) | Resources: 14%                       │
    │ FRESH (<0.3):         65 (6.5%)  | Resources: 3%                        │
    │                                                                         │
    │ GINI-KOEFFIZIENT: 0.34 (gut, < 0.5 = gesund)                           │
    │ ✓ Diversity-Slots aktiv (5% für FRESH reserviert)                      │
    └─────────────────────────────────────────────────────────────────────────┘
```

---

## IX. Weltformel-Befehle (Κ15)

### `erynoa formula`

Weltformel V2.0 analysieren.

```bash
erynoa formula <COMMAND> [OPTIONS]

COMMANDS:
    compute <DID>           𝔼-Beitrag berechnen
    status                  Globaler Status
    components <DID>        Komponenten
    simulate <EVENT>        Event-Auswirkung
    leaderboard             Top-Beiträge

OPTIONS:
    --realm <REALM>         Realm-spezifisch
    --approximation <ALG>   exact, bloom, cms

OUTPUT (status):
    ╔════════════════════════════════════════════════════════════════════════╗
    ║   𝔼 = Σ 𝔸(s) · σ⃗( ‖𝕎(s)‖_w · ln|ℂ(s)| · 𝒮(s) ) · Ĥ(s) · w(s,t)       ║
    ╠════════════════════════════════════════════════════════════════════════╣
    ║   𝔼_total = 12,847.32  |  Δ𝔼 (24h) = +127.45 (+0.99%)                  ║
    ║   Entitäten: 1,000 | Events (τ=90d): 2.4M                              ║
    ╠════════════════════════════════════════════════════════════════════════╣
    ║   Beitrag durch 𝔸: 42% | 𝕎: 35% | Ĥ: 15% | 𝒮: 8%                       ║
    ╠════════════════════════════════════════════════════════════════════════╣
    ║   COUNT-MIN SKETCH (Κ15d): w=2^20, d=7 | Fehler: ε ≤ 0.01%            ║
    ╚════════════════════════════════════════════════════════════════════════╝

AXIOM-REFERENZ:
    Κ15a: 𝒮(s) = ‖𝕎(s)‖² · ℐ(s)
    Κ15b: 𝔼 = Σ 𝔸(s) · σ⃗( ‖𝕎(s)‖_w · ln|ℂ(s)| · 𝒮(s) ) · Ĥ(s) · w(s,t)
    Κ15c: σ⃗(x) = 1 / (1 + e^(-x))
    Κ15d: Count-Min Sketch für ℐ-Approximation
```

---

## X. Netzwerk-Befehle

### `erynoa peer`

Peer-Management.

```bash
erynoa peer <COMMAND>

COMMANDS:
    status                  Status anzeigen
    info                    Details
    sync                    Synchronisieren
    list                    Verbundene Peers
    connect <ID>            Verbinden
    disconnect <ID>         Trennen
```

### `erynoa config`

Konfiguration.

```bash
erynoa config <COMMAND>

COMMANDS:
    set <KEY> <VALUE>       Setzen
    get <KEY>               Auslesen
    list                    Alle anzeigen
    reset                   Zurücksetzen

WICHTIGE KEYS:
    default-realm           Default-Realm
    sync-interval           Sync-Intervall (Sekunden)
    mobile-mode             Low-Power-Modus
    surprisal-algorithm     cms | bloom | exact
```

---

_Weiter zu [07-APPENDIX.md](07-APPENDIX.md) für Glossar und Referenzen._
