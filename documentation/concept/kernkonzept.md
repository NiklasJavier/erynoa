# Erynoa – Kernkonzept

> **Zielgruppe:** Gründer:innen, Product/Business-Entscheider, technisch interessierte Stakeholder
> **Lesezeit:** ca. 12 Minuten
> **Version:** ECL v2.1 – Identity-First + ECLVM
> **Verwandte Dokumente:** [System Architecture](./system-architecture-overview.md) · [ECL Spezifikation](./erynoa-configuration-language.md) · [Glossar](./glossary.md)

---

## Auf einen Blick

**Erynoa** ist ein kybernetisches Protokoll für die Maschinenökonomie. Es ermöglicht Maschinen, Unternehmen und digitalen Agenten, autonom und vertrauensbasiert miteinander zu handeln – ohne zentrale Vermittler.

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│         🔮 ERY                  🤖 ECHO                 ⚡ NOA      │
│   Semantic & Identity         Intelligenz &          Wahrheit &    │
│        Lattice                  Agenten              Exekution     │
│                                                                     │
│   ┌─────────────────┐                                               │
│   │ 🔐 DACS Module  │         Wer handelt           Was ist        │
│   │ (Identity)      │         mit wem?              passiert?      │
│   ├─────────────────┤                                               │
│   │ Trust & Wissen  │                                               │
│   └─────────────────┘                                               │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Das Problem

Die heutige Maschinenökonomie ist kaputt:

| Problem                         | Auswirkung                                                    |
| ------------------------------- | ------------------------------------------------------------- |
| **Fragmentierte Daten**         | Technische, rechtliche und betriebliche Infos liegen in Silos |
| **Kein Vertrauen**              | Maschinen können sich gegenseitig nicht einschätzen           |
| **Blockchains skalieren nicht** | Alles auf einer Ebene → Flaschenhals                          |
| **Chain Lock-in**               | Identität nur auf einer Chain = keine Interoperabilität       |

**Die Konsequenz:** Autonome Maschinen können nicht sicher miteinander handeln.

---

## Die Lösung: Die kybernetische Triade

Erynoa löst das Problem durch **radikale Trennung** in drei spezialisierte Sphären:

### 🔮 ERY – Das Semantic & Identity Lattice

> _„Wer bin ich? Was bedeutet etwas? Wem kann man vertrauen?"_

ERY ist das **Gedächtnis und die Identitätsschicht** des Netzwerks – modular aufgebaut:

| Modul                 | Funktion                                        |
| --------------------- | ----------------------------------------------- |
| **🔐 DACS**           | Multi-Chain Identity: DIDs, VCs, Self-Anchoring |
| **📚 Semantic Index** | Blueprints, Normen, Wissen (Qdrant-basiert)     |
| **⚖️ Karmic Engine**  | Trust-Berechnung mit Karma Tiers & Asymmetrie   |
| **🌍 Discovery**      | DHT, Geohashing für dezentrale Suche            |

**Neu in v2.1 – Identity-First Paradigma:**

- **Sub-Identities:** 16 spezialisierte Typen (Trading, Voting, Recovery, etc.)
- **Capability-basiert:** Jede Sub-Identity hat definierte Berechtigungen
- **Revocation:** Kompromittierte Sub-Identities einzeln widerrufbar
- **Karma Tiers:** Gestaffelte Trust-Level (Newcomer → Veteran → Elder)
- **Asymmetrie:** Negative Events wiegen 1.5× stärker als positive

**DACS-Modul im Detail:**

- **Multi-Chain Anchoring:** Eine DID, verankert auf IOTA, Ethereum, Solana
- **Dezentrale Validatoren:** DACS Nodes koordinieren via BFT-Konsens
- **Self-Anchoring:** Das System verankert seine eigene Registry
- **Verifiable Credentials:** W3C-konforme Credentials für Agenten & Assets

**Technologie:** BFT Konsens, BLS Threshold Signatures, Qdrant, DHT, libp2p

> 📖 **Mehr erfahren:** [DACS Identity](./dacs-identity.md)

---

### 🤖 ECHO – Die Intelligenz

> _„Wer braucht was? Wer bietet es an? Wie programmiert sich das System selbst?“_

- **Seeker-Agenten:** Repräsentieren Nachfrage (Nutzer, Maschinen, Unternehmen)
- **Provider-Agenten:** Repräsentieren Angebot (Infrastruktur, Services)
- **Verhandlung:** Private, verschlüsselte Off-Chain-Kommunikation
- **Multi-Chain Wallet:** Agenten verwalten Guthaben auf mehreren Chains gleichzeitig
- **Network Selection:** Agenten wählen autonom das optimale Netzwerk für Transaktionen

**Neu in v2.1 – ECLVM (Erynoa Virtual Machine):**

- **Dynamische Programmierung:** Agenten schreiben und führen ECL-Code zur Laufzeit aus
- **Template-System:** Schablonen für Environments, Blueprints, Agents
- **Hot-Code-Reload:** Funktionen werden live aktualisiert ohne Neustart
- **Sandboxed Execution:** Sichere, ressourcenlimitierte Ausführung

**Technologie:** WASM-Sandbox, ECLVM, libp2p, XMTP, Multi-Chain Wallet Engine

> 📖 **Mehr erfahren:** [Agents & ADL](./agents-and-adl.md)

---

### ⚡ NOA – Die Wahrheit

> _„Was ist wirklich passiert?"_

- **Atomic Market Objects (AMOs):** Digitale Zwillinge von Assets, Credentials, Services
- **Logic Guards:** Unveränderliche Regeln auf Bytecode-Ebene
- **Finalität:** Transaktionen sind in < 2 Sekunden unwiderruflich

**Technologie:** MoveVM, Starfish BFT, IOTA Rebased

---

## Wie es zusammenspielt

```
                    ┌─────────────────────┐
                    │   Nutzer/Maschine   │
                    │   formuliert Intent │
                    └──────────┬──────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────────┐
│                         🤖 ECHO                                  │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐          │
│  │   Intent    │───▶│  Discovery  │───▶│ Verhandlung │          │
│  │   (ADL)     │    │             │    │             │          │
│  └─────────────┘    └──────┬──────┘    └──────┬──────┘          │
└────────────────────────────┼─────────────────┼───────────────────┘
                             │                 │
          ┌──────────────────┼─────────────────┼──────────────────┐
          │                  │                 │                  │
          │                  ▼                 │                  │
          │    ┌─────────────────────────────┐ │                  │
          │    │         🔮 ERY              │ │                  │
          │    │  ┌───────────┬───────────┐  │ │                  │
          │    │  │🔐 DACS    │⚖️ Karmic  │  │ │                  │
          │    │  │DID Resolve│Trust Query│  │ │                  │
          │    │  │VC Verify  │Blueprints │  │ │                  │
          │    │  └───────────┴───────────┘  │ │                  │
          │    └─────────────┬───────────────┘ │                  │
          │                  │                 │                  │
          │                  │    ┌────────────▼────────────┐     │
          │                  │    │        ⚡ NOA           │     │
          │                  │    │  Transaktion finalisiert│     │
          │                  │    │  AMOs aktualisiert      │     │
          │                  │    └────────────┬────────────┘     │
          │                  │                 │                  │
          │                  │    Feedback     │                  │
          │                  ◀─────────────────┘                  │
          │              Trust-Update                             │
          │                                                       │
          └───────────────────────────────────────────────────────┘
```

---

## Die Bausteine im Detail

### 1. Das Liquide Datenmodell

Erynoa trennt **Definition** von **Instanz**:

| Ebene          | Ort | Beschreibung                                             |
| -------------- | --- | -------------------------------------------------------- |
| **Blueprints** | ERY | _Wie_ soll etwas sein? (Normen, Validierungsregeln)      |
| **AMOs**       | NOA | _Was_ existiert konkret? (Assets, Credentials, Services) |

**Drei AMO-Typen:**

| Typ               | Beschreibung                               | Beispiele                              |
| ----------------- | ------------------------------------------ | -------------------------------------- |
| 🏭 **Material**   | Transferierbare physische Assets           | Ladesäule, Sensor, Maschine            |
| 🎫 **Credential** | Soulbound-Nachweise (nicht transferierbar) | KYC, Zertifikat, Lizenz                |
| ⏱️ **Service**    | Zeitgebundene Dienstleistungen             | Ladevorgang, API-Nutzung, Energiefluss |

**Object Placement & Anchoring (v2.1):**

Objekte leben in **Umgebungen** – standardmäßig in der realen Welt (Root):

```
┌────────────────────────────────────────────────────────────┐
│                    OBJECT LIFECYCLE                        │
├────────────────────────────────────────────────────────────┤
│                                                            │
│   🌍 ROOT (env:erynoa:real_world)  ← Default für alle     │
│       │                                                    │
│       │ plan_move()                                        │
│       ▼                                                    │
│   📝 PLANNED → Membership geprüft, Chain-Branch ermittelt │
│       │                                                    │
│       │ anchor()                                           │
│       ▼                                                    │
│   ⚓ ANCHORED → Auf Environment-Chain geankert            │
│       │                                                    │
│       │ activate_scoring()                                 │
│       ▼                                                    │
│   ✅ ACTIVE → Scoring & Discovery aktiv                   │
│                                                            │
│   ⚠️ Ohne Anchoring: Kein Scoring in virtuellen Envs!     │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

**Chain-Branches:** Jede virtuelle Umgebung definiert eine Chain (IOTA, ETH, SOL, etc.). ERY kennt die Hierarchie und kann Object-Placement sowie Netzwerkinformationen liefern.

**Fallback:** Wenn eine Umgebung deaktiviert wird, fallen Objekte automatisch zur Parent-Umgebung zurück – ultimativ zur Root.

→ 📖 **Mehr erfahren:** [Search Environments](./search-environments.md#6-object-placement--chain-anchoring-v21)

---

### 2. Vertrauen als Kernprinzip

Vertrauen ist keine Metadaten – es ist **Zugangskontrolle**.

```
┌─────────────────────────────────────────────────────────────┐
│                     Karmic Engine                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   Event (z.B. erfolgreiche Lieferung)                       │
│                    │                                        │
│                    ▼                                        │
│   ┌─────────────────────────────────────┐                   │
│   │  Trust Vector aktualisieren         │                   │
│   │  R_new = R_old + η(F_event - E[F])  │                   │
│   └─────────────────────────────────────┘                   │
│                    │                                        │
│                    ▼                                        │
│   Vertrauen propagiert entlang Hierarchien:                 │
│   Hersteller → Betreiber → Asset                            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Trust-Gating:** Agenten setzen Mindest-Reputation. Wer darunter liegt, wird gar nicht erst angefragt.

---

### 3. Der Cybernetic Loop

Jede Interaktion durchläuft acht Phasen:

| #   | Phase            | Ort          | Was passiert                              |
| --- | ---------------- | ------------ | ----------------------------------------- |
| 1   | **Intent**       | ECHO         | Agent beschreibt Ziel in ADL              |
| 2   | **Discovery**    | ECHO↔ERY     | Passende Partner finden                   |
| 3   | **Identity**     | ERY (DACS)   | DID auflösen, Credentials verifizieren    |
| 4   | **Trust-Gating** | ERY (Karmic) | Reputation & Karma Tier prüfen            |
| 5   | **Verhandlung**  | ECHO         | Privat, verschlüsselt, Off-Chain          |
| 6   | **ECLVM**        | ECHO (VM)    | Dynamische ECL-Logik zur Laufzeit         |
| 7   | **Exekution**    | NOA          | Transaktion finalisieren                  |
| 8   | **Feedback**     | NOA→ERY      | Trust Vectors aktualisieren (±Asymmetrie) |

**Das Besondere:** Phase 8 beeinflusst Phase 2-4 der nächsten Interaktion. Das System **lernt**.

---

## Warum diese Architektur?

### Off-Chain vs. On-Chain

| Aspekt         | Off-Chain (ERY, ECHO)      | On-Chain (NOA)         |
| -------------- | -------------------------- | ---------------------- |
| **Zweck**      | Denken, Suchen, Verhandeln | Finalisieren, Beweisen |
| **Konsens**    | Keiner nötig               | Starfish BFT           |
| **Daten**      | Reich, semantisch          | Minimal, kausal        |
| **Skalierung** | Horizontal                 | Durch Entlastung       |

**Ergebnis:** Der Ledger enthält nur, was wirklich zählt – ohne auf Sicherheit zu verzichten.

---

## Was Erynoa ermöglicht

### 🔋 Autonome Maschinenökonomie

Maschinen handeln selbstständig unter klaren Regeln und messbarem Vertrauen.

### ⚖️ Rechtssichere Automatisierung

Industriestandards und Regularien werden in Blueprints und Logic Guards kodifiziert.

### 🚀 Skalierbare Infrastruktur

Semantik und Intelligenz Off-Chain, Wahrheit On-Chain – das Beste aus beiden Welten.

---

## Ein konkretes Beispiel

> **Szenario:** Ein E-Fahrzeug sucht eine Ladesäule.

1. **Intent (ECHO):** _„50 kWh laden, nur erneuerbar, Region München, MinTrust 0.8"_
2. **Discovery (ERY):** Semantic Index findet 12 Ladesäulen im Umkreis
3. **Identity (ERY/DACS):** Fahrzeug-DID wird aufgelöst (`did:erynoa:vehicle-abc123`), Credentials (Versicherung, Zahlungsfähigkeit) werden verifiziert – über alle verankerten Chains hinweg
4. **Trust-Gating (ERY/Karmic):** 3 Ladesäulen fallen wegen niedriger Reputation raus
5. **Verhandlung (ECHO):** Fahrzeug-Agent verhandelt mit Ladesäulen-Agent → 0,35€/kWh
6. **Network Selection (ECHO):** Agent analysiert beide Wallets:
   - Fahrzeug: IOTA ✓, ETH ✓, SOL ✓
   - Ladesäule: IOTA ✓, ETH ✓
   - **Entscheidung:** IOTA (niedrigste Gebühr: 0,001€, gemeinsame Chain)
7. **Exekution (NOA/IOTA):** Service-AMO wird auf IOTA erstellt, Continuous Value Streaming startet
8. **Feedback (ERY):** Nach erfolgreichem Laden steigt die Reputation beider Parteien

**Dauer:** < 5 Sekunden vom Intent bis zum Ladestart.
**Netzwerkwahl:** Vollautomatisch, kostenoptimiert, ohne Benutzerinteraktion.

---

## Zusammenfassung

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│   Erynoa = Wissen & Identität + Intelligenz + Wahrheit          │
│                                                                 │
│   ┌─────────────────────┐     ┌─────────────┐   ┌─────────┐    │
│   │       🔮 ERY        │◀───▶│   🤖 ECHO   │◀─▶│ ⚡ NOA  │    │
│   │  Semantic & Identity│     │   Handeln   │   │ Beweisen│    │
│   │      Lattice        │     └─────────────┘   └─────────┘    │
│   │  ┌────────┬────────┐│           ▲               │          │
│   │  │🔐 DACS │⚖️Karmic││           │               │          │
│   │  └────────┴────────┘│           └── Feedback ───┘          │
│   └─────────────────────┘                                       │
│                                                                 │
│   Multi-Chain Anchors (via ERY/DACS): IOTA | Ethereum | Solana  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Erynoa verwandelt fragmentierte, misstrauische Märkte in eine vernetzte, vertrauensbasierte Maschinenökonomie – in der jede Interaktion das System klüger macht und jede Identität chain-agnostisch und selbstsouverän ist.**

---

## Weiterführend

| Dokument                                                 | Für wen          | Inhalt                                        |
| -------------------------------------------------------- | ---------------- | --------------------------------------------- |
| [DACS Identity](./dacs-identity.md)                      | Architekt:innen  | Multi-Chain DIDs, BFT Konsens, Self-Anchoring |
| [System Architecture](./system-architecture-overview.md) | Architekt:innen  | Technische Details zu ERY, ECHO, NOA          |
| [Liquides Datenmodell](./liquides-datenmodell.md)        | Data Architects  | Blueprints, AMOs, Fluid Extensions            |
| [Trust & Reputation](./trust-and-reputation.md)          | Risk/Security    | Karmic Engine, Trust Vectors                  |
| [Cybernetic Loop](./cybernetic-loop.md)                  | Engineers        | Workflow im Detail                            |
| [Agents & ADL](./agents-and-adl.md)                      | Developers       | Agentenmodell, Agent Definition Language      |
| [Use Cases](./use-cases.md)                              | Business/Product | Konkrete Anwendungsszenarien                  |
