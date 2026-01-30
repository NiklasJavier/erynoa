# EIP-001: DID:erynoa Method Specification

> **EIP:** 001
> **Titel:** DID:erynoa Method Specification
> **Status:** Draft
> **Version:** 0.4
> **Typ:** Standard
> **Ebene:** E1 (Fundament)
> **Erstellt:** Januar 2026
> **Aktualisiert:** Februar 2026
> **Abhängigkeiten:** W3C DID Core Specification v1.0, DIDComm v2, EIP-004 (Bayesian Trust), EIP-005 (Virt-Envs)

---

## Abstract

Diese Spezifikation definiert die `did:erynoa` DID-Methode für das Erynoa-Protokoll. Die Methode ermöglicht die Erstellung, Auflösung, Aktualisierung und Deaktivierung von dezentralen Identifikatoren für Menschen, Organisationen, autonome Agenten, physische Geräte und andere Entitäten innerhalb des Erynoa-Ökosystems.

Die Methode ist W3C DID Core v1.0 konform und erweitert den Standard um:
- 10 semantische Namespaces für verschiedene Entitätstypen
- Controller-Chain für Haftungszuordnung bei autonomen Agenten
- **Unified Identity (V0.4)**: Ein Master-Secret → alle Keys → eine Identität
- **Multi-Chain Wallets (V0.4)**: Deterministisch abgeleitete Wallets auf allen Chains
- **Optional Recovery (V0.4)**: Recovery deaktiviert bis explizit aktiviert
- Social Recovery und Staked Guardianship
- Privacy-preserving Pairwise-DIDs
- Multi-Chain-Anchoring mit deterministischer Konfliktauflösung
- Integration mit dem Erynoa Trust-System

---

## Motivation

Erynoa benötigt ein Identitätssystem, das:

1. **Dezentral** ist – keine zentrale Autorität kontrolliert Identitäten
2. **Selbstsouverän** ist – Entitäten kontrollieren ihre eigenen Identitäten
3. **Interoperabel** ist – kompatibel mit dem W3C DID-Ökosystem
4. **Semantisch reich** ist – unterscheidet zwischen Entitätstypen
5. **Haftbar** ist – autonome Agenten haben verantwortliche Controller
6. **Wiederherstellbar** ist – Schlüsselverlust führt nicht zu Identitätsverlust
7. **Privat** ist – pseudonyme Interaktionen sind möglich
8. **Persistent** ist – Identitäten können deaktiviert, aber nicht gelöscht werden

---

## V0.4: Unified Identity Architecture

### Einmalige Anmeldung

Nutzer melden sich beim Erynoa-Peer mit **einem** Master-Secret an:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    UNIFIED IDENTITY FLOW                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   MASTER SECRET                                                             │
│   ─────────────                                                             │
│   Option A: BIP39 Mnemonic (24 Wörter)                                     │
│   Option B: WebAuthn Passkey (Biometrie/Hardware-Key)                      │
│                                                                             │
│                          │                                                  │
│                          ▼                                                  │
│              ┌───────────────────────┐                                     │
│              │ DETERMINISTIC KDF     │                                     │
│              │ (HD-Derivation)       │                                     │
│              └───────────────────────┘                                     │
│                          │                                                  │
│          ┌───────────────┼───────────────┐                                 │
│          │               │               │                                 │
│          ▼               ▼               ▼                                 │
│   ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                         │
│   │ Ed25519     │ │ secp256k1   │ │ Ed25519     │                         │
│   │ m/44'/9999' │ │ m/44'/60'   │ │ m/44'/4218' │                         │
│   └──────┬──────┘ └──────┬──────┘ └──────┬──────┘                         │
│          │               │               │                                 │
│          ▼               ▼               ▼                                 │
│   ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                         │
│   │ did:erynoa: │ │ 0x...       │ │ iota1q...   │                         │
│   │ self:alice  │ │ (Ethereum)  │ │ (IOTA)      │                         │
│   └─────────────┘ └─────────────┘ └─────────────┘                         │
│          │               │               │                                 │
│          └───────────────┼───────────────┘                                 │
│                          │                                                  │
│                          ▼                                                  │
│              ┌───────────────────────┐                                     │
│              │    DID-DOCUMENT       │                                     │
│              │  (multiChainWallets)  │                                     │
│              └───────────────────────┘                                     │
│                                                                             │
│   RESULTAT:                                                                 │
│   • Eine Anmeldung → Alle Wallets                                          │
│   • Initiale Kontrolle: 100% beim User (normale EOAs)                      │
│   • Recovery: DEAKTIVIERT (bis User aktiviert)                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Derivation Paths

| Chain | Key-Typ | Derivation Path |
|-------|---------|-----------------|
| Erynoa (Primary) | Ed25519 | m/44'/9999'/0'/0/0 |
| Ethereum/EVM | secp256k1 | m/44'/60'/0'/0/0 |
| Solana | Ed25519 | m/44'/501'/0'/0' |
| IOTA/MoveVM | Ed25519 | m/44'/4218'/0'/0/0 |
| Sui | Ed25519 | m/44'/784'/0'/0/0 |

### DID-Document mit Multi-Chain Wallets

```json
{
  "id": "did:erynoa:self:alice-2026-xyz",
  "erynoa": {
    "multiChainWallets": [
      {
        "chain": "erynoa-root",
        "address": "did:erynoa:self:alice-2026-xyz",
        "keyType": "Ed25519",
        "derivationPath": "m/44'/9999'/0'/0/0"
      },
      {
        "chain": "ethereum-mainnet",
        "chainId": 1,
        "address": "0x1234...abcd",
        "keyType": "secp256k1"
      },
      {
        "chain": "iota-mainnet",
        "address": "iota1qr...xyz",
        "keyType": "Ed25519"
      }
    ],
    "recovery": {
      "status": "none"
    }
  }
}
```

### Optional Recovery

Recovery ist **initial deaktiviert**. Der User hat volle Kontrolle ohne externe Abhängigkeiten.

**Aktivierung (später):**

```bash
erynoa recovery setup \
  --method social-staked \
  --threshold 3 \
  --guardian did:erynoa:guild:sparkasse-berlin \
  --guardian did:erynoa:self:bob \
  --guardian did:erynoa:self:carol \
  --timelock 7d
```

**Recovery-Prozess:**

1. **Threshold erreichen**: k von n Guardians bestätigen
2. **Timelock warten**: 7 Tage Wartezeit
3. **Key-Rotation**: Neue Keys aus neuem Master-Secret
4. **RightsTransfer-Event**: Alte DIDs zeigen auf neue
5. **Asset-Transfer**: Guthaben auf neue Wallets transferieren

---

## Spezifikation

### 1. DID-Syntax

Die `did:erynoa` Methode folgt der generischen DID-Syntax gemäß W3C DID Core:

```
did:erynoa:<namespace>:<unique-identifier>
```

#### 1.1 ABNF-Grammatik

```abnf
did-erynoa      = "did:erynoa:" namespace ":" unique-id
namespace       = "self" / "guild" / "spirit" / "thing" / 
                  "vessel" / "source" / "craft" / "vault" / 
                  "pact" / "circle"
unique-id       = 1*idchar
idchar          = ALPHA / DIGIT / "-" / "_"
```

#### 1.2 Beispiele

```
did:erynoa:self:alice-2024-abc123
did:erynoa:guild:siemens-energy-gmbh
did:erynoa:spirit:trading-bot-alpha-7
did:erynoa:thing:sensor-temp-warehouse-42
did:erynoa:vessel:ev-bmw-i4-de-m-xy-1234
did:erynoa:source:solar-panel-roof-a1
did:erynoa:craft:translation-api-v2
did:erynoa:vault:treasury-main-ops
did:erynoa:pact:rental-agreement-2024-001
did:erynoa:circle:energy-trading-eu
```

### 2. Namespaces

Die 10 Namespaces kategorisieren Entitäten semantisch. Der Namespace beeinflusst:
- Den Human-Alignment-Faktor H(s)
- Die erforderlichen Credentials
- Die Governance-Stimmgewichte
- Die Controller-Anforderungen
- Die Recovery-Optionen

| Namespace | Beschreibung | H(s) Basis | Controller erforderlich | Recovery |
|-----------|--------------|------------|------------------------|----------|
| `self` | Natürliche Person | 2.0* | Nein | Social/Institutional |
| `guild` | Organisation (Unternehmen, Verein, DAO) | 1.0 | Nein | Multi-Sig |
| `spirit` | Autonomer Agent (KI, Bot, Algorithmus) | 1.0 | **Ja** | Via Controller |
| `thing` | Physisches Gerät (Sensor, Maschine) | 1.0 | **Ja** | Via Controller |
| `vessel` | Fahrzeug (Auto, Drohne, Schiff) | 1.0 | **Ja** | Via Controller |
| `source` | Energiequelle (Solar, Wind, Ladestation) | 1.0 | **Ja** | Via Controller |
| `craft` | Service/Dienstleistung | 1.0 | Optional | Via Controller |
| `vault` | Wallet/Vermögensspeicher | 1.0 | Optional | Multi-Sig |
| `pact` | Vertrag/Vereinbarung | 1.0 | Nein | Via Parteien |
| `circle` | Realm/Environment | 1.0 | Nein | Via Council |

*H(s) = 2.0 nur mit gültigem HumanAuth-Credential

#### 2.1 Namespace-Regeln

**self:**
- Reserviert für natürliche Personen
- Kann HumanAuth-Credential erwerben für H(s) = 2.0
- Keine Controller-Chain erforderlich
- **Recovery via Guardians möglich** (siehe Abschnitt 10)
- Kann Sub-Identitäten aller Namespaces erstellen

**guild:**
- Für juristische Personen und Organisationen
- Muss mindestens einen `self`-Controller haben (Vorstand, Geschäftsführer)
- Kann Sub-Identitäten für Abteilungen, Projekte erstellen
- Recovery via Multi-Sig der Controller

**spirit:**
- Für autonome Software-Agenten
- **Muss** einen Controller haben (self oder guild)
- Controller haftet für Handlungen des Agenten
- H(s) = 1.5 wenn direkter self-Controller, sonst 1.0
- Recovery erfolgt via Controller

**thing, vessel, source:**
- Für physische Geräte und Maschinen
- **Muss** einen Controller haben
- Typischerweise von guild oder self kontrolliert
- Recovery erfolgt via Controller

**craft:**
- Für Services und APIs
- Controller optional, aber empfohlen

**vault:**
- Für Wallets und Vermögensspeicher
- Controller optional, aber empfohlen für Enterprise
- Multi-Sig für hohe Werte empfohlen

**pact:**
- Für Verträge zwischen Parteien
- Automatisch erstellt bei AGREE-Phase
- Controller sind die Vertragsparteien

**circle:**
- Für Realms und Environments
- Controller ist der Realm-Council

### 3. DID-Dokument

Das DID-Dokument enthält die kryptographischen Materialien und Metadaten zur DID.

#### 3.1 Struktur (V0.3 – mit Staked Guardianship)

```json
{
  "@context": [
    "https://www.w3.org/ns/did/v1",
    "https://erynoa.network/ns/did/v1",
    "https://identity.foundation/didcomm-messaging/v2"
  ],
  "id": "did:erynoa:self:alice-2024-abc123",
  "verificationMethod": [
    {
      "id": "did:erynoa:self:alice-2024-abc123#key-1",
      "type": "Ed25519VerificationKey2020",
      "controller": "did:erynoa:self:alice-2024-abc123",
      "publicKeyMultibase": "z6Mkf5rGMoatrSj1f4CyvuHBeXJELe9RPdzo2PKGNCKVtZxP"
    },
    {
      "id": "did:erynoa:self:alice-2024-abc123#key-pq-1",
      "type": "Dilithium3VerificationKey2024",
      "controller": "did:erynoa:self:alice-2024-abc123",
      "publicKeyMultibase": "z2J9gaYxrKVpdPBTs..."
    },
    {
      "id": "did:erynoa:self:alice-2024-abc123#key-backup",
      "type": "Ed25519VerificationKey2020",
      "controller": "did:erynoa:self:alice-2024-abc123",
      "publicKeyMultibase": "z6MkhaXgBZDvotDkL...",
      "status": "backup"
    }
  ],
  "authentication": [
    "did:erynoa:self:alice-2024-abc123#key-1"
  ],
  "assertionMethod": [
    "did:erynoa:self:alice-2024-abc123#key-1"
  ],
  "keyAgreement": [
    {
      "id": "did:erynoa:self:alice-2024-abc123#key-agree-1",
      "type": "X25519KeyAgreementKey2020",
      "controller": "did:erynoa:self:alice-2024-abc123",
      "publicKeyMultibase": "z6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc"
    }
  ],
  "service": [
    {
      "id": "did:erynoa:self:alice-2024-abc123#didcomm",
      "type": "DIDCommMessaging",
      "serviceEndpoint": {
        "uri": "https://agents.example.com/alice",
        "routingKeys": ["did:erynoa:self:mediator-1#key-1"],
        "accept": ["didcomm/v2"]
      }
    },
    {
      "id": "did:erynoa:self:alice-2024-abc123#erynoa-agent",
      "type": "ErynoaAgent",
      "serviceEndpoint": {
        "uri": "https://agents.example.com/alice/erynoa",
        "protocol": "erynoa/v1",
        "authentication": "required"
      }
    }
  ],
  "erynoa": {
    "namespace": "self",
    "created": "2026-01-15T10:30:00Z",
    "updated": "2026-01-20T14:22:00Z",
    "status": "active",
    "recovery": {
      "method": "social-staked",
      "threshold": 2,
      "guardians": [
        {
          "did": "did:erynoa:guild:sparkasse-berlin",
          "role": "institutional",
          "endorsement": {
            "level": "kyc-level-3",
            "stake": {
              "type": "tokens",
              "amount": 500
            },
            "liability": "full",
            "signature": "z4GdRnI..."
          }
        },
        {
          "did": "did:erynoa:self:bob-friend",
          "role": "personal",
          "endorsement": null
        },
        {
          "did": "did:erynoa:guild:notar-office-muc",
          "role": "institutional",
          "endorsement": {
            "level": "notarized",
            "stake": {
              "type": "reputation",
              "percentage": 0.1
            },
            "liability": "partial",
            "signature": "z5HeSnJ..."
          }
        }
      ],
      "timelock": "7d"
    },
    "trustDerived": {
      "sources": [
        {
          "guardian": "did:erynoa:guild:sparkasse-berlin",
          "boost": 0.135,
          "since": "2026-01-15T10:30:00Z"
        },
        {
          "guardian": "did:erynoa:guild:notar-office-muc",
          "boost": 0.027,
          "since": "2026-01-15T10:32:00Z"
        }
      ],
      "totalBoost": 0.162,
      "effectiveLevel": "Verified"
    },
    "privacy": {
      "pairwiseEnabled": true,
      "selectiveDisclosure": true
    },
    "anchors": [
      {
        "chain": "iota",
        "network": "mainnet",
        "txId": "0x1234567890abcdef...",
        "block": 12345678,
        "timestamp": "2026-01-15T10:30:05Z",
        "priority": 1
      }
    ],
    "trustInitial": {
      "R": 0.5,
      "I": 0.5,
      "C": 0.5,
      "P": 0.5,
      "V": 0.5,
      "Ω": 0.5
    }
  }
}
```

#### 3.2 Pflichtfelder

| Feld | Beschreibung | Pflicht |
|------|--------------|---------|
| `@context` | JSON-LD Context | Ja |
| `id` | Die DID | Ja |
| `verificationMethod` | Min. 1 Schlüssel | Ja |
| `authentication` | Authentifizierungs-Methode | Ja |
| `erynoa.namespace` | Der Namespace | Ja |
| `erynoa.created` | Erstellungszeitpunkt | Ja |
| `erynoa.status` | active/deactivated | Ja |
| `erynoa.anchors` | Min. 1 Chain-Anchor | Ja |

#### 3.3 Controller-Felder (für spirit, thing, vessel, source)

| Feld | Beschreibung | Pflicht |
|------|--------------|---------|
| `controller` | Array von Controller-DIDs | Ja |
| `erynoa.controllerChain` | Detaillierte Controller-Info | Ja |
| `erynoa.controllerChain[].capabilities` | Berechtigungen | Ja |

#### 3.4 Recovery-Felder mit Staked Guardianship (V0.3)

| Feld | Beschreibung | Pflicht für self |
|------|--------------|------------------|
| `erynoa.recovery.method` | social/social-staked/institutional/multi-sig | Empfohlen |
| `erynoa.recovery.threshold` | k von n Guardians | Ja, wenn method=social* |
| `erynoa.recovery.guardians` | Array von Guardian-Objekten (erweitert) | Ja, wenn method=social* |
| `erynoa.recovery.timelock` | Wartezeit für Recovery | Optional (default: 7d) |

**Guardian-Objekt (V0.3 – erweitert):**

```json
{
  "did": "did:erynoa:guild:sparkasse-berlin",
  "role": "institutional",
  "endorsement": {
    "level": "kyc-level-3",
    "stake": {
      "type": "tokens",
      "amount": 500
    },
    "liability": "full",
    "signature": "z4GdRnI..."
  }
}
```

| Feld | Beschreibung | Pflicht |
|------|--------------|---------|
| `did` | DID des Guardians | Ja |
| `role` | "personal" oder "institutional" | Ja |
| `endorsement` | Staking-Details (nur für institutional) | Nein |
| `endorsement.level` | KYC-Level (kyc-level-1/2/3, notarized) | Ja wenn endorsement |
| `endorsement.stake` | Token- oder Reputations-Stake | Ja wenn endorsement |
| `endorsement.liability` | none/partial/full | Ja wenn endorsement |
| `endorsement.signature` | Guardian-Signatur über Endorsement | Ja wenn endorsement |

**trustDerived-Feld (V0.3):**

| Feld | Beschreibung |
|------|--------------|
| `erynoa.trustDerived.sources` | Array der Trust-Quellen |
| `erynoa.trustDerived.sources[].guardian` | DID des bürgenden Guardians |
| `erynoa.trustDerived.sources[].boost` | Trust-Boost (0-1) |
| `erynoa.trustDerived.sources[].since` | Zeitpunkt des Stakings |
| `erynoa.trustDerived.totalBoost` | Summe aller Boosts |
| `erynoa.trustDerived.effectiveLevel` | Resultierendes Trust-Level |

#### 3.5 Privacy-Felder (V0.2)

| Feld | Beschreibung | Default |
|------|--------------|---------|
| `erynoa.privacy.pairwiseEnabled` | Erlaubt Pairwise-DIDs | true |
| `erynoa.privacy.selectiveDisclosure` | Erlaubt ZKP-basierte Disclosure | true |

#### 3.6 Optionale Felder

| Feld | Beschreibung |
|------|--------------|
| `assertionMethod` | Für signierte Aussagen |
| `keyAgreement` | Für verschlüsselte Kommunikation (DIDComm) |
| `capabilityInvocation` | Für Capability-Delegierung |
| `capabilityDelegation` | Für Delegierungs-Ketten |
| `service` | Service-Endpoints mit Protokoll-Spezifikation |
| `erynoa.trustInitial` | Initiale Trust-Werte |

### 4. CRUD-Operationen

#### 4.1 Create

Eine neue DID wird durch folgende Schritte erstellt:

```
1. Client generiert Schlüsselpaar (Ed25519 + optional Dilithium3)
2. Client generiert Backup-Schlüssel (für Two-Step Rotation)
3. Client konstruiert initiales DID-Dokument inkl. Recovery-Config
4. Client signiert DID-Dokument mit privatem Schlüssel
5. Client sendet Create-Request an Erynoa-Netzwerk
6. Netzwerk validiert:
   - Syntax korrekt
   - Namespace-Regeln erfüllt
   - Controller existiert (falls erforderlich)
   - Recovery-Config valide (Guardians existieren)
   - Keine Kollision mit existierender DID
7. Netzwerk verankert auf Primary Chain (IOTA)
8. Netzwerk propagiert an Secondary Chains (optional)
9. DID ist auflösbar
```

**Request:**

```json
{
  "operation": "create",
  "didDocument": { ... },
  "signature": "z3FcQmH...",
  "proofOfControl": {
    "controller": "did:erynoa:guild:acme-trading-gmbh",
    "signature": "z4GdRnI..."
  }
}
```

**Response:**

```json
{
  "status": "success",
  "did": "did:erynoa:spirit:trading-bot-alpha-7",
  "anchor": {
    "chain": "iota",
    "txId": "0x1234...",
    "block": 12345678
  }
}
```

#### 4.2 Read (Resolve)

Die Auflösung einer DID erfolgt über den Universal Resolver oder direkt:

```
1. Client sendet Resolve-Request mit DID
2. Resolver prüft lokalen Cache
3. Falls nicht im Cache: Query an Erynoa-Netzwerk
4. Netzwerk liefert DID-Dokument mit Anchor-Proofs
5. Resolver verifiziert Anchor-Proofs (alle Chains)
6. Falls Multi-Chain-Konflikt: Deterministisches Ordering anwenden
7. Resolver liefert DID-Dokument an Client
```

**Request:**

```
GET /1.0/identifiers/did:erynoa:spirit:trading-bot-alpha-7
Accept: application/did+ld+json
```

**Response:**

```json
{
  "didDocument": { ... },
  "didResolutionMetadata": {
    "contentType": "application/did+ld+json",
    "duration": 42
  },
  "didDocumentMetadata": {
    "created": "2026-01-15T10:30:00Z",
    "updated": "2026-01-20T14:22:00Z",
    "versionId": "2",
    "nextVersionId": null,
    "anchors": [ ... ]
  }
}
```

#### 4.3 Update

Updates erfordern eine Signatur vom aktuellen Controller und verwenden **Optimistic Concurrency Control**.

```
1. Client lädt aktuelles DID-Dokument inkl. versionId
2. Client konstruiert neues DID-Dokument
3. Client signiert mit authentication-Schlüssel
4. Falls Controller-Änderung: alte + neue Controller signieren
5. Client sendet Update mit versionId (ETag-Pattern)
6. Netzwerk prüft:
   - versionId stimmt mit aktuellem Stand überein
   - Signaturen valide
   - Änderungen erlaubt
7. Falls Konflikt (versionId mismatch): 409 Conflict zurückgeben
8. Netzwerk verankert Update mit neuer versionId
9. Altes Dokument bleibt im History-Log
```

**Erlaubte Updates:**
- Schlüssel hinzufügen/rotieren (Two-Step-Pattern empfohlen)
- Service-Endpoints ändern
- Controller hinzufügen/entfernen (mit Zustimmung)
- Capabilities ändern
- Recovery-Guardians ändern (mit Timelock)

**Verbotene Updates:**
- Namespace ändern
- DID-Identifier ändern
- created-Timestamp ändern

**Request:**

```json
{
  "operation": "update",
  "did": "did:erynoa:spirit:trading-bot-alpha-7",
  "didDocument": { ... },
  "previousVersionId": "abc123",
  "signature": "z3FcQmH...",
  "controllerSignatures": [
    {
      "controller": "did:erynoa:guild:acme-trading-gmbh",
      "signature": "z4GdRnI..."
    }
  ]
}
```

**Conflict Response (409):**

```json
{
  "status": "conflict",
  "error": "VERSION_MISMATCH",
  "currentVersionId": "def456",
  "message": "Document was modified by another request. Please fetch current version and retry."
}
```

#### 4.4 Deactivate

DIDs können deaktiviert, aber **nicht gelöscht** werden (Axiom A2: Permanenz):

```
1. Controller signiert Deactivation-Request
2. Netzwerk setzt status auf "deactivated"
3. Netzwerk verankert Deactivation
4. DID bleibt auflösbar, aber mit status: deactivated
5. Keine weiteren Operationen möglich (außer Re-Read)
6. Trust-Historie bleibt erhalten
```

**Request:**

```json
{
  "operation": "deactivate",
  "did": "did:erynoa:spirit:trading-bot-alpha-7",
  "reason": "Agent retired",
  "signature": "z3FcQmH...",
  "controllerSignature": "z4GdRnI..."
}
```

**Deaktiviertes DID-Dokument:**

```json
{
  "@context": [ ... ],
  "id": "did:erynoa:spirit:trading-bot-alpha-7",
  "erynoa": {
    "status": "deactivated",
    "deactivated": "2026-06-15T09:00:00Z",
    "deactivationReason": "Agent retired",
    "historyAvailable": true
  }
}
```

### 5. Controller-Chain

Die Controller-Chain definiert die Haftungskette für autonome Entitäten.

#### 5.1 Struktur

```json
{
  "controllerChain": [
    {
      "controller": "did:erynoa:guild:acme-trading-gmbh",
      "capabilities": ["operate", "delegate", "revoke", "recover"],
      "since": "2026-01-15T10:30:00Z",
      "depth": 1
    }
  ]
}
```

#### 5.2 Capabilities

| Capability | Beschreibung |
|------------|--------------|
| `operate` | Kann im Namen der DID handeln |
| `delegate` | Kann Sub-DIDs erstellen |
| `revoke` | Kann die DID deaktivieren |
| `update` | Kann das DID-Dokument ändern |
| `transfer` | Kann Controller-Rechte übertragen |
| `recover` | Kann Schlüssel-Recovery durchführen |

#### 5.3 Tiefe und H(s)-Faktor

Die Tiefe der Controller-Chain beeinflusst den Human-Alignment-Faktor:

| Tiefe | Beschreibung | H(s) |
|-------|--------------|------|
| 0 | DID ist selbst ein Mensch (self + HumanAuth) | 2.0 |
| 1 | Direkter self-Controller | 1.5 |
| 1 | Direkter guild-Controller mit self-Leitung | 1.3 |
| 2 | guild → guild → self | 1.1 |
| 3+ | Längere Ketten | 1.0 |

#### 5.4 Validierung

Bei jeder DID-Operation wird die Controller-Chain validiert:

```python
def validate_controller_chain(did_document):
    namespace = did_document["erynoa"]["namespace"]
    
    # Namespaces, die Controller benötigen
    requires_controller = ["spirit", "thing", "vessel", "source"]
    
    if namespace in requires_controller:
        if "controller" not in did_document:
            raise ValidationError("Controller required for namespace: " + namespace)
        
        for controller_did in did_document["controller"]:
            controller_doc = resolve(controller_did)
            
            # Controller muss aktiv sein
            if controller_doc["erynoa"]["status"] != "active":
                raise ValidationError("Controller is not active")
            
            # Rekursiv prüfen bis zu self oder guild erreicht
            if controller_doc["erynoa"]["namespace"] not in ["self", "guild"]:
                validate_controller_chain(controller_doc)
    
    return True
```

### 6. Multi-Chain-Anchoring

DIDs werden auf mehreren Chains verankert für erhöhte Sicherheit.

#### 6.1 Primary Chain (IOTA)

- Alle DIDs werden auf IOTA verankert
- Feeless Transactions
- ~10s Finality
- MoveVM für komplexe Logik
- **Priority: 1** (höchste Autorität)

#### 6.2 Secondary Chains (Optional)

Für High-Value-DIDs können zusätzliche Anchors erstellt werden:

| Chain | Use Case | Kosten | Priority |
|-------|----------|--------|----------|
| Ethereum L2 | DeFi-Integration, hoher Wert | ~0.10€ | 2 |
| Solana | High-Speed Trading | ~0.001€ | 3 |
| Polygon | Low-Cost, hohe Frequenz | ~0.01€ | 4 |

#### 6.3 Anchor-Struktur

```json
{
  "anchors": [
    {
      "chain": "iota",
      "network": "mainnet",
      "txId": "0x1234567890abcdef...",
      "block": 12345678,
      "timestamp": "2026-01-15T10:30:05Z",
      "merkleRoot": "0xabcdef...",
      "proof": "0x...",
      "priority": 1
    },
    {
      "chain": "ethereum",
      "network": "arbitrum",
      "txId": "0xfedcba...",
      "block": 87654321,
      "timestamp": "2026-01-15T10:30:15Z",
      "contract": "0x1234...DACSRegistry",
      "priority": 2
    }
  ]
}
```

#### 6.4 Konfliktauflösung (V0.2)

Bei widersprüchlichen Zuständen zwischen Chains gilt:

```
1. Primary Chain (IOTA) hat höchste Autorität (priority=1)
2. Bei IOTA-Ausfall: niedrigste Priority-Nummer gewinnt
3. Bei gleicher Priority: ältester Timestamp gewinnt
4. Bei gleichem Timestamp: lexikographisch niedrigste txId gewinnt
```

**Implementierung:**

```rust
fn resolve_conflict(anchors: &[Anchor]) -> &Anchor {
    anchors.iter()
        .min_by(|a, b| {
            a.priority.cmp(&b.priority)
                .then_with(|| a.timestamp.cmp(&b.timestamp))
                .then_with(|| a.tx_id.cmp(&b.tx_id))
        })
        .expect("At least one anchor required")
}
```

### 7. Security Considerations

#### 7.1 Schlüssel-Management

- **Ed25519** als primärer Signatur-Algorithmus
- **Dilithium3** als Post-Quantum-Backup (optional, empfohlen)
- **Schlüssel-Rotation ohne Trust-Verlust** via Two-Step Pattern
- Hardware-Wallet-Unterstützung empfohlen
- Backup-Schlüssel bei Erstellung generieren

#### 7.2 Two-Step Key Rotation Pattern (V0.2)

Um Key-Lockout zu verhindern, erfolgt Schlüsselrotation in zwei Schritten:

```
Step 1: Neuen Schlüssel hinzufügen (als "pending")
   - Alter Schlüssel bleibt aktiv
   - Neuer Schlüssel hat status: "pending"
   - Signiert mit altem Schlüssel

Step 2: Bestätigen und alten Schlüssel widerrufen
   - Nur nach erfolgreicher Nutzung des neuen Schlüssels
   - Alter Schlüssel wird auf status: "revoked" gesetzt
   - Signiert mit neuem Schlüssel
```

**Implementierung:**

```rust
// Step 1: Neuen Key hinzufügen
let new_keypair = Ed25519Keypair::generate();
let updated_doc = resolved.document
    .add_verification_method(VerificationMethod {
        id: format!("{}#key-2", did),
        public_key: new_keypair.public_key(),
        status: KeyStatus::Pending,
    });

client.update_did(updated_doc, &old_keypair).await?;

// Step 2: Bestätigen (nach erfolgreichem Test)
let confirmed_doc = updated_doc
    .confirm_key("key-2")?           // pending -> active
    .revoke_key("key-1")?;           // active -> revoked

client.update_did(confirmed_doc, &new_keypair).await?;
```

#### 7.3 Controller-Sicherheit

- Controller müssen aktiv sein
- Zirkuläre Controller-Chains verboten (Axiom A4)
- Controller-Änderungen erfordern Multi-Sig
- Time-Lock für kritische Änderungen (optional)

#### 7.4 Sybil-Resistenz

- Namespace-basierte Einschränkungen
- HumanAuth für self-Namespace
- Stake-at-Risk für hohe Reputation
- EigenTrust für globales Ranking

### 8. Privacy Considerations (V0.2)

#### 8.1 Öffentliche vs. Private DIDs

| DID-Typ | Verwendung | Auf Ledger | Korrelierbar |
|---------|------------|------------|--------------|
| **Public DID** | Öffentliche Persona, Reputation | Ja | Ja |
| **Pairwise DID** | Bilaterale Interaktionen | Optional | Nein |
| **Blinded DID** | Anonyme Credentials | Nein | Nein |

#### 8.2 Pairwise-DIDs

Für Interaktionen, die keine öffentliche Reputation benötigen:

```json
{
  "id": "did:erynoa:self:alice-bob-pairwise-xyz",
  "erynoa": {
    "type": "pairwise",
    "parent": "did:erynoa:self:alice-2024-abc123",
    "peer": "did:erynoa:self:bob-2024-def456",
    "purpose": "private-messaging"
  }
}
```

**Eigenschaften:**
- Wird nicht auf dem öffentlichen Ledger verankert
- Nur für den spezifischen Peer verwendbar
- Keine Korrelation mit öffentlicher DID möglich
- Trust wird nicht öffentlich aggregiert

#### 8.3 Selective Disclosure

Nutzer können wählen, welche Attribute offengelegt werden:

```json
{
  "request": {
    "required": ["erynoa.namespace", "erynoa.status"],
    "optional": ["service", "erynoa.trustInitial"]
  },
  "response": {
    "disclosed": ["erynoa.namespace", "erynoa.status"],
    "proof": "zkp:..."
  }
}
```

#### 8.4 DSGVO-Compliance

- DID-Dokumente enthalten **keine personenbezogenen Daten**
- HumanAuth beweist "ist Mensch", nicht Identität
- Deaktivierung erfüllt "Recht auf Einschränkung"
- Pairwise-DIDs ermöglichen Pseudonymität
- Alle PII in verschlüsselten Off-Chain Credentials

### 9. Service Endpoints (V0.2)

#### 9.1 DIDComm v2 Integration

Um Spam und Angriffe zu verhindern, verwenden Service-Endpoints DIDComm v2:

```json
{
  "service": [
    {
      "id": "did:erynoa:spirit:trading-bot-7#didcomm",
      "type": "DIDCommMessaging",
      "serviceEndpoint": {
        "uri": "https://agents.acme-trading.com/bot-7/didcomm",
        "routingKeys": [
          "did:erynoa:guild:acme-trading-gmbh#key-mediator"
        ],
        "accept": ["didcomm/v2"]
      }
    }
  ]
}
```

#### 9.2 Erynoa-spezifische Endpoints

```json
{
  "service": [
    {
      "id": "did:erynoa:spirit:trading-bot-7#erynoa-agent",
      "type": "ErynoaAgent",
      "serviceEndpoint": {
        "uri": "https://agents.acme-trading.com/bot-7/erynoa",
        "protocol": "erynoa/v1",
        "authentication": "required",
        "rateLimit": {
          "requests": 100,
          "window": "1m"
        }
      }
    }
  ]
}
```

#### 9.3 Kommunikations-Anforderungen

| Anforderung | Beschreibung |
|-------------|--------------|
| `authentication` | "required" oder "optional" |
| `encryption` | "required" (default) oder "optional" |
| `protocol` | "didcomm/v2", "erynoa/v1", "https" |
| `rateLimit` | Optional, gegen DDoS |

### 10. Recovery (V0.2)

#### 10.1 Social Recovery für self-Namespace

```json
{
  "erynoa": {
    "recovery": {
      "method": "social",
      "threshold": 3,
      "guardians": [
        "did:erynoa:self:bob-guardian-1",
        "did:erynoa:self:carol-guardian-2",
        "did:erynoa:self:dave-guardian-3",
        "did:erynoa:guild:trusted-bank-ag",
        "did:erynoa:guild:notar-office-muc"
      ],
      "timelock": "7d"
    }
  }
}
```

#### 10.2 Recovery-Prozess

```
1. Inhaber meldet Schlüsselverlust
2. Inhaber kontaktiert Guardians außerhalb des Systems
3. Guardians verifizieren Identität (persönlich, Video, etc.)
4. Guardians signieren Recovery-Request
5. Bei Erreichen des Threshold (z.B. 3 von 5):
   - Timelock startet (z.B. 7 Tage Wartezeit)
   - Öffentliche Ankündigung im Netzwerk
6. Falls kein Widerspruch während Timelock:
   - Neuer Schlüssel wird aktiviert
   - Alter Schlüssel wird invalidiert
7. Trust-Historie bleibt erhalten
```

#### 10.3 Institutional Recovery

Alternativ oder zusätzlich:

```json
{
  "erynoa": {
    "recovery": {
      "method": "institutional",
      "provider": "did:erynoa:guild:trusted-bank-ag",
      "verificationLevel": "kycPlus",
      "timelock": "3d"
    }
  }
}
```

#### 10.4 Recovery für kontrollierte Entitäten

Für spirit, thing, vessel, source erfolgt Recovery automatisch via Controller:

```json
{
  "operation": "recover",
  "did": "did:erynoa:spirit:trading-bot-7",
  "newKey": "z6Mkf5rGMoatrSj1f4CyvuHBeXJELe9RPdzo2PKGNCKVtZxP",
  "controllerSignature": "z4GdRnI...",
  "reason": "Key compromise suspected"
}
```

### 11. Trust-Integration

#### 11.1 Initiale Trust-Werte

Neue DIDs starten mit neutralen Trust-Werten:

```json
{
  "trustInitial": {
    "R": 0.5,  // Reliability
    "I": 0.5,  // Integrity
    "C": 0.5,  // Competence
    "P": 0.5,  // Predictability
    "V": 0.5,  // Vigilance
    "Ω": 0.5   // Omega-Alignment
  }
}
```

Dies entspricht einem Beta(2,2) Prior in der Bayesschen Modellierung.

#### 11.2 Trust-Vererbung

Sub-DIDs können Trust vom Parent erben:

```
T_child = T_parent × inheritance_factor

inheritance_factor ∈ [0.5, 0.9]  // konfigurierbar
```

#### 11.3 Controller-Trust-Korrelation

Der Trust eines Agenten korreliert mit dem Trust seines Controllers:

```
T_agent_effective = T_agent × (0.7 + 0.3 × T_controller)
```

### 12. Implementierung

#### 12.1 Resolver-Endpoint

```
https://resolver.erynoa.network/1.0/identifiers/{did}
```

#### 12.2 SDK-Nutzung (Rust) – V0.2

```rust
use erynoa_sdk::identity::{DID, DIDDocument, Namespace, RecoveryConfig};

// DID erstellen mit Recovery
let keypair = Ed25519Keypair::generate();
let backup_keypair = Ed25519Keypair::generate();
let did = DID::new(Namespace::Self_, "alice-2024")?;

let doc = DIDDocument::builder()
    .id(did.clone())
    .verification_method(keypair.public_key())
    .backup_key(backup_keypair.public_key())
    .recovery(RecoveryConfig::social(
        3, // threshold
        vec![guardian1, guardian2, guardian3, guardian4, guardian5],
        Duration::days(7), // timelock
    ))
    .privacy(PrivacyConfig {
        pairwise_enabled: true,
        selective_disclosure: true,
    })
    .build()?;

let anchor = client.create_did(doc, &keypair).await?;
println!("DID created: {} at block {}", did, anchor.block);

// Two-Step Key Rotation
let new_keypair = Ed25519Keypair::generate();

// Step 1: Add new key as pending
let step1_doc = client.add_pending_key(&did, new_keypair.public_key(), &keypair).await?;

// Step 2: Confirm after testing
client.confirm_key_rotation(&did, "key-2", "key-1", &new_keypair).await?;

// Social Recovery initiieren
let recovery = RecoveryRequest::new(&did, new_public_key);
recovery.add_guardian_signature(guardian1, sig1)?;
recovery.add_guardian_signature(guardian2, sig2)?;
recovery.add_guardian_signature(guardian3, sig3)?;

// Threshold erreicht -> Timelock startet
client.initiate_recovery(recovery).await?;

// Nach 7 Tagen (Timelock abgelaufen)
client.complete_recovery(&did).await?;
```

#### 12.3 SDK-Nutzung (TypeScript)

```typescript
import { DID, DIDDocument, Namespace, RecoveryConfig } from '@erynoa/sdk';

// DID erstellen mit Social Recovery
const keypair = await Ed25519Keypair.generate();
const did = new DID(Namespace.Self, 'alice-2024');

const doc = new DIDDocument.Builder()
  .id(did)
  .verificationMethod(keypair.publicKey)
  .recovery(RecoveryConfig.social({
    threshold: 3,
    guardians: [guardian1, guardian2, guardian3, guardian4, guardian5],
    timelock: '7d'
  }))
  .build();

const anchor = await client.createDID(doc, keypair);

// Pairwise DID erstellen
const pairwiseDid = await client.createPairwiseDID(did, peerDid, {
  purpose: 'private-messaging'
});

// Two-Step Key Rotation
const newKeypair = await Ed25519Keypair.generate();
await client.addPendingKey(did, newKeypair.publicKey, keypair);
// ... test new key ...
await client.confirmKeyRotation(did, 'key-2', 'key-1', newKeypair);
```

#### 12.4 CLI-Nutzung

```bash
# DID erstellen mit Social Recovery
erynoa init --namespace self --id alice-2024 \
  --recovery-method social \
  --recovery-threshold 3 \
  --guardians did:erynoa:self:bob,did:erynoa:self:carol,did:erynoa:self:dave,did:erynoa:guild:bank,did:erynoa:guild:notar \
  --recovery-timelock 7d

# DID auflösen
erynoa resolve did:erynoa:self:alice-2024

# Two-Step Key Rotation
erynoa key add-pending --did did:erynoa:self:alice-2024
erynoa key confirm --did did:erynoa:self:alice-2024 --new-key key-2 --old-key key-1

# Recovery initiieren
erynoa recover initiate --did did:erynoa:self:alice-2024 --new-key-file ./new-key.pub
erynoa recover add-guardian-sig --did did:erynoa:self:alice-2024 --guardian did:erynoa:self:bob --sig-file ./bob-sig.bin
# ... repeat for other guardians ...
erynoa recover complete --did did:erynoa:self:alice-2024

# Pairwise DID erstellen
erynoa pairwise create --parent did:erynoa:self:alice-2024 --peer did:erynoa:self:bob

# DID deaktivieren
erynoa deactivate did:erynoa:spirit:my-trading-bot --reason "Retired"

# Controller-Chain anzeigen
erynoa inspect did:erynoa:spirit:my-trading-bot --controller-chain
```

---

## Test-Vektoren

### TV-1: Minimale DID (self)

**Input:**
```json
{
  "namespace": "self",
  "id": "alice-test-1",
  "publicKey": "z6Mkf5rGMoatrSj1f4CyvuHBeXJELe9RPdzo2PKGNCKVtZxP"
}
```

**Expected DID:**
```
did:erynoa:self:alice-test-1
```

### TV-2: Agent mit Controller

**Input:**
```json
{
  "namespace": "spirit",
  "id": "bot-1",
  "controller": "did:erynoa:self:alice-test-1",
  "publicKey": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
}
```

**Expected Controller-Chain:**
```json
{
  "controllerChain": [
    {
      "controller": "did:erynoa:self:alice-test-1",
      "depth": 1
    }
  ]
}
```

**Expected H(s):** 1.5 (direkter self-Controller)

### TV-3: Social Recovery (V0.2)

**Input:**
```json
{
  "did": "did:erynoa:self:alice-test-1",
  "recoverySignatures": [
    { "guardian": "did:erynoa:self:bob", "signature": "z..." },
    { "guardian": "did:erynoa:self:carol", "signature": "z..." },
    { "guardian": "did:erynoa:self:dave", "signature": "z..." }
  ],
  "newPublicKey": "z6MknewKey..."
}
```

**Expected:** Recovery initiiert, Timelock startet

### TV-4: Multi-Chain Conflict Resolution (V0.2)

**Input:**
```json
{
  "anchors": [
    { "chain": "iota", "priority": 1, "timestamp": "2026-01-15T10:30:05Z", "versionId": "v2" },
    { "chain": "ethereum", "priority": 2, "timestamp": "2026-01-15T10:30:03Z", "versionId": "v3" }
  ]
}
```

**Expected:** IOTA-Version (v2) gewinnt (niedrigere Priority trotz späterem Timestamp)

---

## Referenzen

- [W3C DID Core Specification v1.0](https://www.w3.org/TR/did-core/)
- [W3C DID Resolution](https://w3c-ccg.github.io/did-resolution/)
- [DIDComm Messaging v2](https://identity.foundation/didcomm-messaging/spec/v2.0/)
- [Erynoa Fachkonzept V6.1](../FACHKONZEPT.md)
- [Erynoa Trust System](../FACHKONZEPT.md#teil-iii-das-vertrauenssystem)
- [EIP-002: Trust Vector 6D](./EIP-002-trust-vector-6d.md)

---

## Changelog

| Version | Datum | Änderung |
|---------|-------|----------|
| 0.1 | 2026-01-29 | Initial Draft |
| 0.2 | 2026-01-29 | Social Recovery, Privacy (Pairwise DIDs), Service Endpoint Security (DIDComm), Concurrent Update Handling, Two-Step Key Rotation, Multi-Chain Conflict Resolution |
| 0.3 | 2026-01-29 | **Staked Guardianship**: Institutional Guardians mit Token/Reputation-Staking, Trust-Vererbung (Cold-Start-Lösung), Slashing-Mechanik, trustDerived-Feld |
| 0.4 | 2026-02-01 | **Unified Identity**: BIP39/Passkey Master-Secret, deterministische Multi-Chain Key-Derivation, multiChainWallets-Feld im DID-Document, Optional Recovery (initial deaktiviert), RightsTransfer-Event |

---

*EIP-001: DID:erynoa Method Specification*
*Version: 0.4*
*Status: Draft*
*Ebene: E1 (Fundament)*
