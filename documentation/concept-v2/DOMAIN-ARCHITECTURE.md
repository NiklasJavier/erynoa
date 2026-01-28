# Erynoa – Domain Architecture v2.1

> **Version:** 2.1 – Identity-First + ECLVM
> **Datum:** Januar 2026
> **Status:** Domain-Driven Design Spezifikation

---

## 1. Übersicht

```
╔═══════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                       ║
║                    E R Y N O A   D O M A I N   A R C H I T E C T U R E               ║
║                                                                                       ║
║   Sechs Bounded Contexts, die das kybernetische Protokoll fachlich strukturieren.   ║
║                                                                                       ║
╚═══════════════════════════════════════════════════════════════════════════════════════╝
```

---

## 2. Domain Map

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                         ERYNOA DOMAIN MAP                                            │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │                              CORE DOMAINS                                       ││
│   │   ┌───────────────────────────────────────────────────────────────────────────┐││
│   │   │                                                                           │││
│   │   │   ┌─────────────────┐           ┌─────────────────┐                      │││
│   │   │   │                 │           │                 │                      │││
│   │   │   │    IDENTITY     │◀─────────▶│      TRUST      │                      │││
│   │   │   │    DOMAIN       │           │     DOMAIN      │                      │││
│   │   │   │                 │           │                 │                      │││
│   │   │   │ DIDs, Creds,    │           │ Karma, Vectors, │                      │││
│   │   │   │ Sub-Identities  │           │ Attestations    │                      │││
│   │   │   │                 │           │                 │                      │││
│   │   │   └────────┬────────┘           └────────┬────────┘                      │││
│   │   │            │                             │                                │││
│   │   │            └──────────────┬──────────────┘                                │││
│   │   │                           │                                               │││
│   │   │                           ▼                                               │││
│   │   │            ┌─────────────────────────────┐                               │││
│   │   │            │                             │                               │││
│   │   │            │       TRANSACTION           │                               │││
│   │   │            │         DOMAIN              │                               │││
│   │   │            │                             │                               │││
│   │   │            │ Intents, Negotiation,       │                               │││
│   │   │            │ Agreements, Streaming       │                               │││
│   │   │            │                             │                               │││
│   │   │            └─────────────────────────────┘                               │││
│   │   │                                                                           │││
│   │   └───────────────────────────────────────────────────────────────────────────┘││
│   │                                                                                 ││
│   │                            SUPPORTING DOMAINS                                   ││
│   │   ┌───────────────────────────────────────────────────────────────────────────┐││
│   │   │                                                                           │││
│   │   │   ┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐        │││
│   │   │   │                 │   │                 │   │                 │        │││
│   │   │   │    SEMANTIC     │   │   GOVERNANCE    │   │    LEDGER       │        │││
│   │   │   │     DOMAIN      │   │     DOMAIN      │   │    DOMAIN       │        │││
│   │   │   │                 │   │                 │   │                 │        │││
│   │   │   │ Blueprints,     │   │ Environments,   │   │ Events, AMOs,   │        │││
│   │   │   │ Standards,      │   │ Constraints,    │   │ Finality,       │        │││
│   │   │   │ Ontologie       │   │ Proposals       │   │ Anchoring       │        │││
│   │   │   │                 │   │                 │   │                 │        │││
│   │   │   └─────────────────┘   └─────────────────┘   └─────────────────┘        │││
│   │   │                                                                           │││
│   │   └───────────────────────────────────────────────────────────────────────────┘││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Bounded Contexts

### 3.1 Context Map

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                         CONTEXT MAP                                                  │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │                                                                                 ││
│   │         ┌──────────────┐                     ┌──────────────┐                  ││
│   │         │   IDENTITY   │═════════════════════│    TRUST     │                  ││
│   │         │   CONTEXT    │    Conformist       │   CONTEXT    │                  ││
│   │         └──────┬───────┘                     └──────┬───────┘                  ││
│   │                │                                    │                           ││
│   │                │ U/D                                │ U/D                       ││
│   │                │                                    │                           ││
│   │         ┌──────▼───────┐                     ┌──────▼───────┐                  ││
│   │         │   SEMANTIC   │                     │  GOVERNANCE  │                  ││
│   │         │   CONTEXT    │                     │   CONTEXT    │                  ││
│   │         └──────┬───────┘                     └──────┬───────┘                  ││
│   │                │                                    │                           ││
│   │                │ U/D                                │ U/D                       ││
│   │                │                                    │                           ││
│   │                └───────────────┬────────────────────┘                          ││
│   │                                │                                                ││
│   │                         ┌──────▼───────┐                                       ││
│   │                         │ TRANSACTION  │                                       ││
│   │                         │   CONTEXT    │                                       ││
│   │                         └──────┬───────┘                                       ││
│   │                                │                                                ││
│   │                                │ U/D                                            ││
│   │                                │                                                ││
│   │                         ┌──────▼───────┐                                       ││
│   │                         │    LEDGER    │                                       ││
│   │                         │   CONTEXT    │                                       ││
│   │                         └──────────────┘                                       ││
│   │                                                                                 ││
│   │   LEGENDE:                                                                      ││
│   │   ═══════ Shared Kernel / Conformist                                           ││
│   │   U/D    Upstream/Downstream (Customer-Supplier)                               ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Context-Beziehungen

| Upstream           | Downstream          | Beziehung         | Beschreibung                                |
| ------------------ | ------------------- | ----------------- | ------------------------------------------- |
| **Identity**       | Trust               | Conformist        | Trust referenziert DIDs ohne Transformation |
| **Identity**       | Semantic            | Customer-Supplier | Blueprints haben Author-DIDs                |
| **Identity**       | Transaction         | Customer-Supplier | Agents sind Identitäten                     |
| **Trust**          | Governance          | Customer-Supplier | Karma-Tiers bestimmen Voting-Gewicht        |
| **Trust**          | Transaction         | Customer-Supplier | Trust-Gating bei Matching                   |
| **Semantic**       | Transaction         | Customer-Supplier | Intent referenziert Blueprints              |
| **Governance**     | Transaction         | Customer-Supplier | Constraints werden geprüft                  |
| **Transaction**    | Ledger              | Customer-Supplier | Events werden finalisiert                   |
| **Ledger**         | Trust               | Anti-Corruption   | Finalized Events → Trust Updates            |

---

## 4. Identity Domain (Core)

### 4.1 Aggregate: Identity

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                         IDENTITY AGGREGATE                                           │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   «Aggregate Root»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                          IDENTITY                                       │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + id: DID                                                              │  ││
│   │   │  + namespace: IdentityNamespace                                         │  ││
│   │   │  + controller: DID                                                      │  ││
│   │   │  + status: IdentityStatus                                               │  ││
│   │   │  + createdAt: Timestamp                                                 │  ││
│   │   │  + updatedAt: Timestamp                                                 │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + addKey(key: VerificationKey): void                                   │  ││
│   │   │  + revokeKey(keyId: string): void                                       │  ││
│   │   │  + addService(service: ServiceEndpoint): void                           │  ││
│   │   │  + createSubIdentity(type: SubIdentityType): SubIdentity                │  ││
│   │   │  + revokeSubIdentity(subId: DID): void                                  │  ││
│   │   │  + deactivate(): void                                                   │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                          │ 1                                                    ││
│   │                          │                                                      ││
│   │              ┌───────────┼───────────┬──────────────┐                          ││
│   │              │           │           │              │                          ││
│   │              ▼ *         ▼ *         ▼ *            ▼ *                        ││
│   │   ┌──────────────┐ ┌──────────┐ ┌──────────┐ ┌────────────┐                   ││
│   │   │Verification  │ │ Service  │ │   Sub    │ │ Credential │                   ││
│   │   │    Key       │ │ Endpoint │ │ Identity │ │  Holder    │                   ││
│   │   ├──────────────┤ ├──────────┤ ├──────────┤ ├────────────┤                   ││
│   │   │id: string    │ │id: string│ │id: DID   │ │credentialId│                   ││
│   │   │type: KeyType │ │type: str │ │type: enum│ │issuerId    │                   ││
│   │   │publicKey: hex│ │endpoint  │ │scope: [] │ │claims: {}  │                   ││
│   │   │purpose: []   │ │          │ │expiry    │ │validUntil  │                   ││
│   │   └──────────────┘ └──────────┘ └──────────┘ └────────────┘                   ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Value Objects

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│   «Value Object»                                                                     │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                              DID                                                ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  + method: "erynoa"                                                             ││
│   │  + namespace: string     // agent, org, amo, blueprint, env, ...               ││
│   │  + identifier: string    // unique within namespace                             ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  + toString(): string    // "did:erynoa:agent:seeker:vehicle-123"              ││
│   │  + parse(str: string): DID                                                      ││
│   │  + isValid(): boolean                                                           ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
│   «Value Object»                                                                     │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                         VerificationKey                                         ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  + id: string                                                                   ││
│   │  + type: Ed25519 | BLS12381                                                     ││
│   │  + publicKeyMultibase: string                                                   ││
│   │  + purposes: [authentication, assertionMethod, keyAgreement]                   ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
│   «Enum»                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                       IdentityNamespace                                         ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  AGENT_SEEKER, AGENT_PROVIDER, AGENT_BROKER, AGENT_ORACLE, AGENT_VALIDATOR     ││
│   │  ORG, USER, VEHICLE                                                             ││
│   │  AMO_MATERIAL, AMO_SERVICE, AMO_CREDENTIAL, AMO_DATA, AMO_CONTRACT             ││
│   │  BLUEPRINT, STANDARD                                                            ││
│   │  ENV_DOMAIN, ENV_GEO, ENV_PRIVATE                                              ││
│   │  SUB_AVATAR, SUB_DELEGATE, SUB_OWNERSHIP, SUB_SESSION, ...                     ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
│   «Enum»                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                       SubIdentityType (16 Typen)                                ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  TRADING      → Transfer, Receive, Stake, Unstake                              ││
│   │  VOTING       → Vote, Delegate, Propose                                        ││
│   │  RECOVERY     → Recover, Reset                                                 ││
│   │  SOCIAL       → Connect, Message, Endorse                                      ││
│   │  DEVICE       → Sensor, Actuate, Report                                        ││
│   │  SERVICE      → Provide, Consume, Subscribe                                    ││
│   │  ADMIN        → Full Control                                                   ││
│   │  COMPLIANCE   → Regulatory, Audit, Report                                      ││
│   │  AUDIT        → Read, Verify (read-only)                                       ││
│   │  DELEGATION   → Delegate, Revoke                                               ││
│   │  EMERGENCY    → Emergency Actions (time-limited)                               ││
│   │  BACKUP       → Backup, Restore                                                ││
│   │  API          → API Access (rate-limited)                                      ││
│   │  ANALYTICS    → Read Aggregated Data                                           ││
│   │  TESTING      → Sandbox Operations                                             ││
│   │  CUSTOM       → User-Defined                                                   ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 4.3 Domain Events

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                    IDENTITY DOMAIN EVENTS                                            │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   IdentityCreated                                                               ││
│   │   ├── identityId: DID                                                          ││
│   │   ├── namespace: IdentityNamespace                                             ││
│   │   ├── controller: DID                                                          ││
│   │   └── timestamp: Timestamp                                                      ││
│   │                                                                                 ││
│   │   IdentityKeyAdded                                                              ││
│   │   ├── identityId: DID                                                          ││
│   │   ├── keyId: string                                                            ││
│   │   └── keyType: KeyType                                                         ││
│   │                                                                                 ││
│   │   IdentityKeyRevoked                                                            ││
│   │   ├── identityId: DID                                                          ││
│   │   └── keyId: string                                                            ││
│   │                                                                                 ││
│   │   SubIdentityCreated                                                            ││
│   │   ├── parentId: DID                                                            ││
│   │   ├── subIdentityId: DID                                                       ││
│   │   ├── type: SubIdentityType                                                    ││
│   │   └── capabilities: Capability[]                                               ││
│   │                                                                                 ││
│   │   SubIdentityRevoked                                                            ││
│   │   ├── parentId: DID                                                            ││
│   │   ├── subIdentityId: DID                                                       ││
│   │   └── reason: string                                                           ││
│   │                                                                                 ││
│   │   CredentialIssued                                                              ││
│   │   ├── issuerId: DID                                                            ││
│   │   ├── subjectId: DID                                                           ││
│   │   ├── credentialId: DID                                                        ││
│   │   ├── credentialType: string                                                   ││
│   │   └── claims: Map<string, any>                                                 ││
│   │                                                                                 ││
│   │   CredentialRevoked                                                             ││
│   │   ├── issuerId: DID                                                            ││
│   │   ├── credentialId: DID                                                        ││
│   │   └── reason: string                                                           ││
│   │                                                                                 ││
│   │   IdentityDeactivated                                                           ││
│   │   ├── identityId: DID                                                          ││
│   │   └── reason: string                                                           ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 4.4 Domain Services

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                    IDENTITY DOMAIN SERVICES                                          │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                      DIDResolutionService                               │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + resolve(did: DID): DIDDocument                                       │  ││
│   │   │  + resolveWithProof(did: DID): { doc: DIDDocument, proof: MerkleProof } │  ││
│   │   │  + getAnchors(did: DID): ChainAnchor[]                                  │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                      CredentialVerificationService                      │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + verify(credential: VerifiableCredential): VerificationResult         │  ││
│   │   │  + checkRevocation(credentialId: DID): boolean                          │  ││
│   │   │  + verifyPresentation(vp: VerifiablePresentation): VerificationResult   │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                      DelegationService                                  │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + resolveDelegates(identity: DID): DID[]                               │  ││
│   │   │  + canActAs(caller: DID, target: DID, action: Action): boolean          │  ││
│   │   │  + getDelegationChain(from: DID, to: DID): DelegationPath               │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 5. Trust Domain (Core)

### 5.1 Aggregate: TrustProfile

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                         TRUST PROFILE AGGREGATE                                      │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   «Aggregate Root»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                       TRUST PROFILE                                     │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + subjectId: DID                                                       │  ││
│   │   │  + environmentId: DID                                                   │  ││
│   │   │  + trustVector: TrustVector                                             │  ││
│   │   │  + karmaPoints: number                                                  │  ││
│   │   │  + karmaTier: KarmaTier                                                 │  ││
│   │   │  + eventCount: number                                                   │  ││
│   │   │  + lastActivity: Timestamp                                              │  ││
│   │   │  + lastDecay: Timestamp                                                 │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + applyEvent(event: TrustEvent): void                                  │  ││
│   │   │  + applyAttestation(attestation: Attestation): void                     │  ││
│   │   │  + applyDecay(days: number): void                                       │  ││
│   │   │  + recalculateTier(): void                                              │  ││
│   │   │  + getAggregate(): number                                               │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                          │ 1                                                    ││
│   │                          │                                                      ││
│   │              ┌───────────┴───────────┐                                         ││
│   │              │                       │                                         ││
│   │              ▼ *                     ▼ *                                       ││
│   │   ┌──────────────────┐    ┌──────────────────┐                                ││
│   │   │   TrustEvent     │    │   Attestation    │                                ││
│   │   ├──────────────────┤    ├──────────────────┤                                ││
│   │   │id: string        │    │id: DID           │                                ││
│   │   │type: EventType   │    │issuerId: DID     │                                ││
│   │   │dimension: Dim    │    │type: AttestType  │                                ││
│   │   │impact: number    │    │claims: {}        │                                ││
│   │   │timestamp: TS     │    │trustBoost: number│                                ││
│   │   │transactionId: DID│    │validUntil: TS    │                                ││
│   │   └──────────────────┘    └──────────────────┘                                ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Value Objects

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│   «Value Object»                                                                     │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                           TrustVector                                           ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  + reliability: number   // [0, 1] - Zuverlässigkeit                           ││
│   │  + integrity: number     // [0, 1] - Ehrlichkeit                               ││
│   │  + capability: number    // [0, 1] - Leistungsfähigkeit                        ││
│   │  + reputation: number    // [0, 1] - Externe Wahrnehmung                       ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  + getAggregate(weights?: number[]): number                                     ││
│   │  + applyPositive(dim: Dimension, amount: number): TrustVector                  ││
│   │  + applyNegative(dim: Dimension, amount: number): TrustVector                  ││
│   │  + decay(rate: number): TrustVector                                            ││
│   │  + meetsThreshold(requirements: TrustRequirements): boolean                    ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
│   «Value Object»                                                                     │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                       TrustRequirements                                         ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  + minAggregate: number?                                                        ││
│   │  + minReliability: number?                                                      ││
│   │  + minIntegrity: number?                                                        ││
│   │  + minCapability: number?                                                       ││
│   │  + minReputation: number?                                                       ││
│   │  + minKarmaTier: KarmaTier?                                                     ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
│   «Enum»                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                            KarmaTier                                            ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  NEWCOMER     (0-100 Karma)    → Basis-Zugang, Rate-Limited                    ││
│   │  ESTABLISHED  (100-500 Karma)  → Voller Zugang, Voting: 1×                     ││
│   │  VETERAN      (500-2000 Karma) → Premium, Voting: 2×                           ││
│   │  ELDER        (2000+ Karma)    → Governance-Rollen, Voting: 3×                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
│   «Enum»                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                          TrustEventType                                         ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  SUCCESS       → +0.02 (reward_weight: 1.0)                                    ││
│   │  WARNING       → -0.005 × 1.5 (penalty_weight: 1.5)                            ││
│   │  FAILURE       → -0.05 × 1.5                                                   ││
│   │  FRAUD         → -0.30 × 1.5                                                   ││
│   │  ATTESTATION   → +0.10                                                         ││
│   │  DECAY         → exponential decay                                             ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 5.3 Domain Events

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                      TRUST DOMAIN EVENTS                                             │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   TrustProfileCreated                                                           ││
│   │   ├── subjectId: DID                                                           ││
│   │   ├── environmentId: DID                                                       ││
│   │   └── initialVector: TrustVector                                               ││
│   │                                                                                 ││
│   │   TrustVectorUpdated                                                            ││
│   │   ├── subjectId: DID                                                           ││
│   │   ├── environmentId: DID                                                       ││
│   │   ├── previousVector: TrustVector                                              ││
│   │   ├── newVector: TrustVector                                                   ││
│   │   ├── dimension: TrustDimension                                                ││
│   │   └── reason: TrustEvent | Attestation                                         ││
│   │                                                                                 ││
│   │   KarmaTierChanged                                                              ││
│   │   ├── subjectId: DID                                                           ││
│   │   ├── previousTier: KarmaTier                                                  ││
│   │   ├── newTier: KarmaTier                                                       ││
│   │   └── karmaPoints: number                                                      ││
│   │                                                                                 ││
│   │   AttestationReceived                                                           ││
│   │   ├── subjectId: DID                                                           ││
│   │   ├── issuerId: DID                                                            ││
│   │   ├── attestationType: string                                                  ││
│   │   └── trustBoost: number                                                       ││
│   │                                                                                 ││
│   │   AttestationExpired                                                            ││
│   │   ├── subjectId: DID                                                           ││
│   │   ├── attestationId: DID                                                       ││
│   │   └── trustReduction: number                                                   ││
│   │                                                                                 ││
│   │   TrustDecayApplied                                                             ││
│   │   ├── subjectId: DID                                                           ││
│   │   ├── daysSinceActivity: number                                                ││
│   │   └── decayAmount: number                                                      ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 5.4 Domain Services

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                      TRUST DOMAIN SERVICES                                           │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                        KarmaEngine                                      │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  - rewardWeight: 1.0                                                    │  ││
│   │   │  - penaltyWeight: 1.5                                                   │  ││
│   │   │  - decayRate: 0.999                                                     │  ││
│   │   │  - decayFloor: 0.3                                                      │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + calculateImpact(event: TrustEvent): TrustImpact                      │  ││
│   │   │  + calculateDecay(profile: TrustProfile, days: number): TrustVector     │  ││
│   │   │  + calculateTier(karmaPoints: number): KarmaTier                        │  ││
│   │   │  + propagateToParent(event: TrustEvent, rippleFactor: number): void     │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                       TrustGatingService                                │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + checkRequirements(subject: DID, req: TrustRequirements): GatingResult│  ││
│   │   │  + filterCandidates(candidates: DID[], req: TrustRequirements): DID[]   │  ││
│   │   │  + rankByTrust(candidates: DID[], weights: TrustWeights): RankedList    │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                    TrustInheritanceService                              │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + inheritFromParent(child: DID, parent: DID, factor: number): void     │  ││
│   │   │  + propagateRipple(source: DID, event: TrustEvent): void                │  ││
│   │   │  + getEffectiveTrust(entity: DID, env: DID): TrustVector                │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Transaction Domain (Core)

### 6.1 Aggregate: Agent

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                         AGENT AGGREGATE                                              │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   «Aggregate Root»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                           AGENT                                         │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + id: DID                                                              │  ││
│   │   │  + type: AgentType                                                      │  ││
│   │   │  + ownerId: DID                                                         │  ││
│   │   │  + status: AgentStatus                                                  │  ││
│   │   │  + activeEnvironments: DID[]                                            │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + createIntent(spec: IntentSpec): Intent                               │  ││
│   │   │  + evaluateOffer(offer: Offer): PolicyDecision                          │  ││
│   │   │  + acceptOffer(offerId: DID): Agreement                                 │  ││
│   │   │  + rejectOffer(offerId: DID, reason: string): void                      │  ││
│   │   │  + cancelIntent(intentId: DID): void                                    │  ││
│   │   │  + joinEnvironment(envId: DID, credentials: DID[]): void                │  ││
│   │   │  + leaveEnvironment(envId: DID): void                                   │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                          │ 1                                                    ││
│   │                          │                                                      ││
│   │         ┌────────────────┼────────────────┐                                    ││
│   │         │                │                │                                    ││
│   │         ▼ 1              ▼ 1              ▼ *                                  ││
│   │   ┌──────────┐    ┌──────────┐    ┌──────────────┐                            ││
│   │   │  Wallet  │    │  Policy  │    │   Intent     │                            ││
│   │   ├──────────┤    ├──────────┤    ├──────────────┤                            ││
│   │   │balances  │    │autoAccept│    │id: DID       │                            ││
│   │   │limits    │    │autoReject│    │type: IntentT │                            ││
│   │   │methods[] │    │escalate  │    │constraints   │                            ││
│   │   │spent: {} │    │limits    │    │budget        │                            ││
│   │   └──────────┘    └──────────┘    │status        │                            ││
│   │                                    └──────────────┘                            ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 6.2 Aggregate: Negotiation

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                       NEGOTIATION AGGREGATE                                          │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   «Aggregate Root»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                        NEGOTIATION                                      │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + id: DID                                                              │  ││
│   │   │  + intentId: DID                                                        │  ││
│   │   │  + seekerId: DID                                                        │  ││
│   │   │  + providerId: DID                                                      │  ││
│   │   │  + model: NegotiationModel                                              │  ││
│   │   │  + status: NegotiationStatus                                            │  ││
│   │   │  + startedAt: Timestamp                                                 │  ││
│   │   │  + expiresAt: Timestamp                                                 │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + makeOffer(terms: Terms): Offer                                       │  ││
│   │   │  + counterOffer(terms: Terms): Offer                                    │  ││
│   │   │  + accept(offerId: DID): Agreement                                      │  ││
│   │   │  + reject(reason: string): void                                         │  ││
│   │   │  + timeout(): void                                                      │  ││
│   │   │  + cancel(initiator: DID, reason: string): void                         │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                          │ 1                                                    ││
│   │                          │                                                      ││
│   │              ┌───────────┴───────────┐                                         ││
│   │              │                       │                                         ││
│   │              ▼ *                     ▼ 0..1                                    ││
│   │   ┌──────────────────┐    ┌──────────────────┐                                ││
│   │   │      Offer       │    │    Agreement     │                                ││
│   │   ├──────────────────┤    ├──────────────────┤                                ││
│   │   │id: DID           │    │id: DID           │                                ││
│   │   │from: DID         │    │seekerId: DID     │                                ││
│   │   │terms: Terms      │    │providerId: DID   │                                ││
│   │   │validUntil: TS    │    │terms: Terms      │                                ││
│   │   │round: number     │    │serviceId: DID    │                                ││
│   │   │status: OfferStat │    │signedAt: TS      │                                ││
│   │   └──────────────────┘    └──────────────────┘                                ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 6.3 Aggregate: PaymentStream

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                       PAYMENT STREAM AGGREGATE                                       │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   «Aggregate Root»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                       PAYMENT STREAM                                    │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + id: DID                                                              │  ││
│   │   │  + agreementId: DID                                                     │  ││
│   │   │  + senderId: DID                                                        │  ││
│   │   │  + receiverId: DID                                                      │  ││
│   │   │  + rate: StreamRate                                                     │  ││
│   │   │  + maxAmount: Money                                                     │  ││
│   │   │  + escrowAmount: Money                                                  │  ││
│   │   │  + transferred: Money                                                   │  ││
│   │   │  + status: StreamStatus                                                 │  ││
│   │   │  + startedAt: Timestamp                                                 │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + tick(usage: UsageMetric): StreamTransfer                             │  ││
│   │   │  + pause(): void                                                        │  ││
│   │   │  + resume(): void                                                       │  ││
│   │   │  + complete(): StreamSettlement                                         │  ││
│   │   │  + abort(reason: string): StreamSettlement                              │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                          │ 1                                                    ││
│   │                          │                                                      ││
│   │                          ▼ *                                                    ││
│   │              ┌──────────────────────┐                                          ││
│   │              │    StreamTransfer    │                                          ││
│   │              ├──────────────────────┤                                          ││
│   │              │timestamp: Timestamp  │                                          ││
│   │              │amount: Money         │                                          ││
│   │              │units: number         │                                          ││
│   │              │cumulative: Money     │                                          ││
│   │              └──────────────────────┘                                          ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 6.4 Value Objects

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│   «Value Object»                                                                     │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                              Intent                                             ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  + id: DID                                                                      ││
│   │  + type: IntentType                                                             ││
│   │  + seekerId: DID                                                                ││
│   │  + environmentId: DID                                                           ││
│   │  + constraints: Constraints                                                     ││
│   │  + budget: Budget                                                               ││
│   │  + trustRequirements: TrustRequirements                                         ││
│   │  + priorities: Map<string, number>                                              ││
│   │  + status: IntentStatus                                                         ││
│   │  + expiresAt: Timestamp                                                         ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
│   «Value Object»                                                                     │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                              Policy                                             ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  + id: DID                                                                      ││
│   │  + scope: PolicyScope                                                           ││
│   │  + autoAccept: Condition[]                                                      ││
│   │  + autoReject: Condition[]                                                      ││
│   │  + escalate: EscalationRule                                                     ││
│   │  + limits: PolicyLimits                                                         ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  + evaluate(offer: Offer, context: Context): PolicyDecision                     ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
│   «Value Object»                                                                     │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                              Terms                                              ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  + pricePerUnit: Money                                                          ││
│   │  + unit: string                                                                 ││
│   │  + maxAmount: Money                                                             ││
│   │  + currency: Currency                                                           ││
│   │  + paymentMethod: DID                                                           ││
│   │  + startTime: Timestamp?                                                        ││
│   │  + duration: Duration?                                                          ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
│   «Value Object»                                                                     │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                              Wallet                                             ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  + id: DID                                                                      ││
│   │  + ownerId: DID                                                                 ││
│   │  + balances: Map<Currency, Money>                                               ││
│   │  + paymentMethods: PaymentMethod[]                                              ││
│   │  + limits: WalletLimits                                                         ││
│   │  + spent: SpendingRecord                                                        ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  + canSpend(amount: Money): boolean                                             ││
│   │  + reserve(amount: Money): Reservation                                          ││
│   │  + release(reservation: Reservation): void                                      ││
│   │  + transfer(to: DID, amount: Money): Transfer                                   ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
│   «Enum»                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                            AgentType                                            ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  SEEKER     → Sucht Ressourcen/Dienste                                         ││
│   │  PROVIDER   → Bietet Ressourcen/Dienste                                        ││
│   │  BROKER     → Vermittelt zwischen Parteien                                     ││
│   │  ORACLE     → Liefert externe Daten                                            ││
│   │  VALIDATOR  → Prüft und bestätigt                                              ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
│   «Enum»                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                        NegotiationModel                                         ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  DIRECT       → Take-it-or-leave-it                                            ││
│   │  AUCTION      → Competitive Bidding                                            ││
│   │  MULTI_ROUND  → Haggling (Offer/Counter-Offer)                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
│   «Enum»                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                        PolicyDecision                                           ││
│   ├─────────────────────────────────────────────────────────────────────────────────┤│
│   │  ACCEPT    → Automatically accept                                              ││
│   │  REJECT    → Automatically reject                                              ││
│   │  ESCALATE  → Ask owner/human                                                   ││
│   │  COUNTER   → Make counter-offer                                                ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 6.5 Domain Events

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                    TRANSACTION DOMAIN EVENTS                                         │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   IntentCreated                         OfferMade                               ││
│   │   ├── intentId: DID                     ├── offerId: DID                       ││
│   │   ├── seekerId: DID                     ├── negotiationId: DID                 ││
│   │   ├── type: IntentType                  ├── providerId: DID                    ││
│   │   └── constraints: Constraints          └── terms: Terms                       ││
│   │                                                                                 ││
│   │   OfferAccepted                         OfferRejected                           ││
│   │   ├── offerId: DID                      ├── offerId: DID                       ││
│   │   ├── negotiationId: DID                ├── negotiationId: DID                 ││
│   │   └── agreementId: DID                  └── reason: string                     ││
│   │                                                                                 ││
│   │   AgreementReached                      AgreementCancelled                      ││
│   │   ├── agreementId: DID                  ├── agreementId: DID                   ││
│   │   ├── seekerId: DID                     ├── cancelledBy: DID                   ││
│   │   ├── providerId: DID                   └── reason: string                     ││
│   │   └── terms: Terms                                                              ││
│   │                                                                                 ││
│   │   StreamOpened                          StreamTransferred                       ││
│   │   ├── streamId: DID                     ├── streamId: DID                      ││
│   │   ├── agreementId: DID                  ├── amount: Money                      ││
│   │   └── escrowAmount: Money               └── cumulative: Money                  ││
│   │                                                                                 ││
│   │   StreamSettled                         WalletTopUp                             ││
│   │   ├── streamId: DID                     ├── walletId: DID                      ││
│   │   ├── totalTransferred: Money           ├── amount: Money                      ││
│   │   ├── returned: Money                   └── source: PaymentMethod              ││
│   │   └── outcome: StreamOutcome                                                    ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 6.6 Domain Services

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                    TRANSACTION DOMAIN SERVICES                                       │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                        MatchingEngine                                   │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + findMatches(intent: Intent): MatchResult[]                           │  ││
│   │   │  + rankMatches(matches: MatchResult[], priorities: Map): RankedList     │  ││
│   │   │  + calculateScore(intent: Intent, provider: DID): number                │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                      PolicyEvaluator (ECLVM)                            │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + evaluate(policy: Policy, offer: Offer, ctx: Context): PolicyDecision│  ││
│   │   │  + checkLimits(wallet: Wallet, amount: Money): boolean                  │  ││
│   │   │  + generateCounterOffer(policy: Policy, offer: Offer): Terms?           │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                      NegotiationOrchestrator                            │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + startNegotiation(intent: Intent, provider: DID): Negotiation         │  ││
│   │   │  + processOffer(negotiation: Negotiation, offer: Offer): void           │  ││
│   │   │  + finalizeAgreement(negotiation: Negotiation): Agreement               │  ││
│   │   │  + handleTimeout(negotiation: Negotiation): void                        │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                       StreamingPaymentService                           │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + openStream(agreement: Agreement): PaymentStream                      │  ││
│   │   │  + processUsage(stream: PaymentStream, usage: Usage): void              │  ││
│   │   │  + settleStream(stream: PaymentStream): Settlement                      │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 7. Semantic Domain (Supporting)

### 7.1 Aggregate: Blueprint

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                         BLUEPRINT AGGREGATE                                          │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   «Aggregate Root»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                         BLUEPRINT                                       │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + id: DID                                                              │  ││
│   │   │  + name: string                                                         │  ││
│   │   │  + version: SemanticVersion                                             │  ││
│   │   │  + authorId: DID                                                        │  ││
│   │   │  + basedOn: DID[]          // Referenced standards                      │  ││
│   │   │  + parentBlueprint: DID?   // For specialization                        │  ││
│   │   │  + amoTypes: AMOType[]                                                  │  ││
│   │   │  + logicGuardAddress: string                                            │  ││
│   │   │  + status: BlueprintStatus                                              │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + addAttribute(attr: AttributeDefinition): void                        │  ││
│   │   │  + validate(values: Map<string, any>): ValidationResult                 │  ││
│   │   │  + specialize(name: string, additions: Attribute[]): Blueprint          │  ││
│   │   │  + publish(): void                                                      │  ││
│   │   │  + deprecate(successor: DID): void                                      │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                          │ 1                                                    ││
│   │                          │                                                      ││
│   │                          ▼ *                                                    ││
│   │              ┌──────────────────────┐                                          ││
│   │              │ AttributeDefinition  │                                          ││
│   │              ├──────────────────────┤                                          ││
│   │              │name: string          │                                          ││
│   │              │type: DataType        │                                          ││
│   │              │required: boolean     │                                          ││
│   │              │constraints: Constraint[]│                                       ││
│   │              │defaultValue: any?    │                                          ││
│   │              └──────────────────────┘                                          ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 7.2 Domain Services

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                    SEMANTIC DOMAIN SERVICES                                          │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                      SemanticSearchService                              │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + search(query: string, filters: SearchFilters): SearchResult[]        │  ││
│   │   │  + findSimilar(blueprintId: DID): Blueprint[]                           │  ││
│   │   │  + expandQuery(query: string): ExpandedQuery  // via Ontologie          │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                      OntologyService                                    │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + getParentClasses(className: string): string[]                        │  ││
│   │   │  + getChildClasses(className: string): string[]                         │  ││
│   │   │  + getRelations(className: string): Relation[]                          │  ││
│   │   │  + inferProperties(className: string): Property[]                       │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                      BlueprintValidationService                         │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + validateAMO(amo: AMO, blueprint: Blueprint): ValidationResult        │  ││
│   │   │  + checkStandardCompliance(blueprint: Blueprint): ComplianceReport      │  ││
│   │   │  + verifyLogicGuard(blueprint: Blueprint): boolean                      │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 8. Governance Domain (Supporting)

### 8.1 Aggregate: Environment

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                       ENVIRONMENT AGGREGATE                                          │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   «Aggregate Root»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                        ENVIRONMENT                                      │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + id: DID                                                              │  ││
│   │   │  + name: string                                                         │  ││
│   │   │  + type: EnvironmentType                                                │  ││
│   │   │  + parentId: DID?                                                       │  ││
│   │   │  + ownerId: DID                                                         │  ││
│   │   │  + standards: DID[]                                                     │  ││
│   │   │  + requiredBlueprints: DID[]                                            │  ││
│   │   │  + membershipType: MembershipType                                       │  ││
│   │   │  + status: EnvironmentStatus                                            │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + addConstraint(constraint: Constraint): void                          │  ││
│   │   │  + removeConstraint(constraintId: string): void                         │  ││
│   │   │  + addMember(memberId: DID, credentials: DID[]): Membership             │  ││
│   │   │  + removeMember(memberId: DID): void                                    │  ││
│   │   │  + createProposal(proposer: DID, change: Change): Proposal              │  ││
│   │   │  + executeProposal(proposalId: DID): void                               │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                          │ 1                                                    ││
│   │                          │                                                      ││
│   │         ┌────────────────┼────────────────┬────────────────┐                   ││
│   │         │                │                │                │                   ││
│   │         ▼ *              ▼ *              ▼ 1              ▼ *                 ││
│   │   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐                ││
│   │   │Constraint│    │Membership│    │ Council  │    │ Proposal │                ││
│   │   ├──────────┤    ├──────────┤    ├──────────┤    ├──────────┤                ││
│   │   │name      │    │memberId  │    │members[] │    │id: DID   │                ││
│   │   │type      │    │role      │    │quorum    │    │proposer  │                ││
│   │   │rule: ECL │    │joinedAt  │    │threshold │    │change    │                ││
│   │   │severity  │    │credentials│   │karmaWeighted│ │votes     │                ││
│   │   └──────────┘    └──────────┘    └──────────┘    │status    │                ││
│   │                                                    └──────────┘                ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 8.2 Domain Events

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                    GOVERNANCE DOMAIN EVENTS                                          │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   EnvironmentCreated                    MemberJoined                            ││
│   │   ├── environmentId: DID                ├── environmentId: DID                 ││
│   │   ├── type: EnvironmentType             ├── memberId: DID                      ││
│   │   └── ownerId: DID                      └── role: MemberRole                   ││
│   │                                                                                 ││
│   │   ConstraintAdded                       ConstraintRemoved                       ││
│   │   ├── environmentId: DID                ├── environmentId: DID                 ││
│   │   ├── constraintName: string            └── constraintName: string             ││
│   │   └── severity: Severity                                                        ││
│   │                                                                                 ││
│   │   ProposalCreated                       ProposalVoted                           ││
│   │   ├── proposalId: DID                   ├── proposalId: DID                    ││
│   │   ├── environmentId: DID                ├── voterId: DID                       ││
│   │   ├── proposerId: DID                   ├── vote: VoteType                     ││
│   │   └── changeType: ChangeType            └── weight: number                     ││
│   │                                                                                 ││
│   │   ProposalApproved                      ProposalRejected                        ││
│   │   ├── proposalId: DID                   ├── proposalId: DID                    ││
│   │   └── executedAt: Timestamp             └── reason: string                     ││
│   │                                                                                 ││
│   │   ConstraintViolated                                                            ││
│   │   ├── environmentId: DID                                                       ││
│   │   ├── constraintName: string                                                   ││
│   │   ├── violatorId: DID                                                          ││
│   │   └── severity: Severity                                                        ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 8.3 Domain Services

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                    GOVERNANCE DOMAIN SERVICES                                        │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                      ConstraintEngine (ECLVM)                           │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + evaluate(constraints: Constraint[], ctx: EvaluationContext): Report  │  ││
│   │   │  + checkMandatory(constraints: Constraint[], ctx): boolean              │  ││
│   │   │  + getViolations(constraints: Constraint[], ctx): Violation[]           │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                      VotingService                                      │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + castVote(proposal: Proposal, voter: DID, vote: VoteType): void       │  ││
│   │   │  + calculateWeight(voter: DID, env: Environment): number                │  ││
│   │   │  + checkQuorum(proposal: Proposal): boolean                             │  ││
│   │   │  + checkThreshold(proposal: Proposal): boolean                          │  ││
│   │   │  + finalizeVoting(proposal: Proposal): ProposalOutcome                  │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                      MembershipService                                  │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + checkEligibility(applicant: DID, env: Environment): EligibilityResult│  ││
│   │   │  + verifyCredentials(applicant: DID, required: DID[]): boolean          │  ││
│   │   │  + getEffectiveConstraints(member: DID, env: Environment): Constraint[] │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 9. Ledger Domain (Supporting)

### 9.1 Aggregate: AMO

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                           AMO AGGREGATE                                              │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   «Aggregate Root»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                             AMO                                         │  ││
│   │   │                  (Atomic Managed Object)                                │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + id: DID                                                              │  ││
│   │   │  + type: AMOType                                                        │  ││
│   │   │  + blueprintId: DID                                                     │  ││
│   │   │  + ownerId: DID                                                         │  ││
│   │   │  + attributes: Map<string, any>                                         │  ││
│   │   │  + credentials: DID[]                                                   │  ││
│   │   │  + status: AMOStatus                                                    │  ││
│   │   │  + version: number                                                      │  ││
│   │   │  + createdAt: Timestamp                                                 │  ││
│   │   │  + updatedAt: Timestamp                                                 │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + transition(action: TransitionAction, caller: DID): void              │  ││
│   │   │  + updateAttribute(name: string, value: any): void                      │  ││
│   │   │  + addCredential(credentialId: DID): void                               │  ││
│   │   │  + transferOwnership(newOwner: DID): void                               │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   │   AMO STATUS MACHINE:                                                           ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                                                                         │  ││
│   │   │   PENDING ──activate()──▶ ACTIVE ──decommission()──▶ DECOMMISSIONED    │  ││
│   │   │                             │  ▲                                        │  ││
│   │   │                    suspend()│  │resume()                                │  ││
│   │   │                             ▼  │                                        │  ││
│   │   │                          SUSPENDED                                      │  ││
│   │   │                                                                         │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 9.2 Aggregate: Event (NOA)

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                         EVENT AGGREGATE (NOA Ledger)                                 │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   «Aggregate Root»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                           EVENT                                         │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + id: DID                                                              │  ││
│   │   │  + type: EventType                                                      │  ││
│   │   │  + causes: DID[]           // Kausale Referenzen (DAG)                  │  ││
│   │   │  + participants: DID[]                                                  │  ││
│   │   │  + payload: EventPayload                                                │  ││
│   │   │  + timestamp: Timestamp                                                 │  ││
│   │   │  + finalityLevel: FinalityLevel                                         │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + addToMerkleBatch(): void                                             │  ││
│   │   │  + setAnchored(anchor: ChainAnchor): void                               │  ││
│   │   │  + setFinal(): void                                                     │  ││
│   │   │  + getProof(): MerkleProof                                              │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                          │ 1                                                    ││
│   │                          │                                                      ││
│   │                          ▼ *                                                    ││
│   │              ┌──────────────────────┐                                          ││
│   │              │     ChainAnchor      │                                          ││
│   │              ├──────────────────────┤                                          ││
│   │              │chain: ChainType      │                                          ││
│   │              │block: string         │                                          ││
│   │              │txHash: string        │                                          ││
│   │              │timestamp: Timestamp  │                                          ││
│   │              │confirmations: number │                                          ││
│   │              │status: AnchorStatus  │                                          ││
│   │              └──────────────────────┘                                          ││
│   │                                                                                 ││
│   │   FINALITY LEVELS:                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │ Level 0: PENDING     → Erstellt lokal                                   │  ││
│   │   │ Level 1: DISTRIBUTED → An Netzwerk verteilt                             │  ││
│   │   │ Level 2: ANCHORED    → Auf Chain geschrieben                            │  ││
│   │   │ Level 3: FINAL       → Genug Confirmations                              │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 9.3 Domain Services

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                      LEDGER DOMAIN SERVICES                                          │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                      LogicGuardExecutor (ECLVM)                         │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + validate(amo: AMO, transition: Transition, ctx: Context): Result     │  ││
│   │   │  + loadGuard(address: string): LogicGuard                               │  ││
│   │   │  + execute(guard: LogicGuard, input: GuardInput): GuardOutput           │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                      AnchoringService                                   │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + batchEvents(events: Event[]): MerkleBatch                            │  ││
│   │   │  + anchor(batch: MerkleBatch, chains: ChainType[]): AnchorResult        │  ││
│   │   │  + verifyAnchor(eventId: DID, chain: ChainType): VerificationResult     │  ││
│   │   │  + getProof(eventId: DID): MerkleProof                                  │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                      CausalOrderingService                              │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + orderEvents(events: Event[]): OrderedEventList                       │  ││
│   │   │  + getCausalPredecessors(eventId: DID): Event[]                         │  ││
│   │   │  + getCausalSuccessors(eventId: DID): Event[]                           │  ││
│   │   │  + validateCausality(event: Event): boolean                             │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   │   «Domain Service»                                                              ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                      EventQueryService                                  │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + getByTransaction(transactionId: DID): Event[]                        │  ││
│   │   │  + getByParticipant(participantId: DID, range: TimeRange): Event[]      │  ││
│   │   │  + getByType(eventType: EventType, range: TimeRange): Event[]           │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 10. Cross-Domain Integration

### 10.1 Application Services (Use Case Layer)

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                      APPLICATION SERVICES                                            │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   «Application Service»                                                         ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                    ChargingTransactionService                           │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  // Orchestriert: Identity, Trust, Semantic, Governance, Transaction   │  ││
│   │   │                                                                         │  ││
│   │   │  + initiateCharging(vehicleId: DID, location: GeoHash): ChargingFlow    │  ││
│   │   │  + completeCharging(sessionId: DID): ChargingResult                     │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  Steps:                                                                 │  ││
│   │   │  1. Resolve vehicle identity (Identity Domain)                          │  ││
│   │   │  2. Get trust profile (Trust Domain)                                    │  ││
│   │   │  3. Discover stations via semantic search (Semantic Domain)             │  ││
│   │   │  4. Filter by environment constraints (Governance Domain)               │  ││
│   │   │  5. Create intent & negotiate (Transaction Domain)                      │  ││
│   │   │  6. Execute & stream payment (Transaction + Ledger Domain)              │  ││
│   │   │  7. Finalize & update trust (Ledger + Trust Domain)                     │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   │   «Application Service»                                                         ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                    IdentityOnboardingService                            │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + registerOrganization(org: OrgData): Identity                         │  ││
│   │   │  + registerAgent(owner: DID, type: AgentType): Agent                    │  ││
│   │   │  + registerAsset(owner: DID, blueprint: DID, attrs: Map): AMO           │  ││
│   │   │  + issueCredential(issuer: DID, subject: DID, claims: Map): Credential  │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   │   «Application Service»                                                         ││
│   │   ┌─────────────────────────────────────────────────────────────────────────┐  ││
│   │   │                    GovernanceAdministrationService                      │  ││
│   │   ├─────────────────────────────────────────────────────────────────────────┤  ││
│   │   │  + createEnvironment(owner: DID, spec: EnvSpec): Environment            │  ││
│   │   │  + submitProposal(env: DID, proposer: DID, change: Change): Proposal    │  ││
│   │   │  + castVote(proposal: DID, voter: DID, vote: VoteType): void            │  ││
│   │   │  + executeProposal(proposal: DID): ExecutionResult                      │  ││
│   │   └─────────────────────────────────────────────────────────────────────────┘  ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### 10.2 Domain Event Flow

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                      CROSS-DOMAIN EVENT FLOW                                         │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   TRANSACTION DOMAIN                                                            ││
│   │   ───────────────────                                                           ││
│   │   AgreementReached ─────────────────────┐                                       ││
│   │        │                                │                                       ││
│   │        │                                ▼                                       ││
│   │        │                         LEDGER DOMAIN                                  ││
│   │        │                         ─────────────                                  ││
│   │        │                         EventCreated (agreement_reached)               ││
│   │        │                                │                                       ││
│   │        ▼                                │                                       ││
│   │   StreamSettled ────────────────────────┤                                       ││
│   │        │                                │                                       ││
│   │        │                                ▼                                       ││
│   │        │                         EventAnchored                                  ││
│   │        │                                │                                       ││
│   │        └────────────────────────────────┼──────────────────┐                    ││
│   │                                         │                  │                    ││
│   │                                         ▼                  ▼                    ││
│   │                                  TRUST DOMAIN       GOVERNANCE DOMAIN           ││
│   │                                  ────────────       ─────────────────           ││
│   │                                  TrustVectorUpdated  (constraint monitoring)    ││
│   │                                         │                                       ││
│   │                                         ▼                                       ││
│   │                                  KarmaTierChanged (if threshold crossed)        ││
│   │                                         │                                       ││
│   │                                         ▼                                       ││
│   │                                  (affects future Governance voting weight)      ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
│   FEEDBACK LOOP:                                                                     │
│   Ledger Events → Trust Updates → Governance Changes → Transaction Constraints      │
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 11. Repositories

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│                         REPOSITORY INTERFACES                                        │
│                                                                                       │
│   ┌─────────────────────────────────────────────────────────────────────────────────┐│
│   │                                                                                 ││
│   │   IDENTITY DOMAIN                                                               ││
│   │   ───────────────                                                               ││
│   │   «Repository»                                                                  ││
│   │   IdentityRepository                                                            ││
│   │   + save(identity: Identity): void                                              ││
│   │   + findById(id: DID): Identity?                                                ││
│   │   + findByController(controller: DID): Identity[]                               ││
│   │   + findByNamespace(namespace: IdentityNamespace): Identity[]                   ││
│   │                                                                                 ││
│   │   TRUST DOMAIN                                                                  ││
│   │   ────────────                                                                  ││
│   │   «Repository»                                                                  ││
│   │   TrustProfileRepository                                                        ││
│   │   + save(profile: TrustProfile): void                                           ││
│   │   + findBySubject(subjectId: DID, envId: DID): TrustProfile?                    ││
│   │   + findByTierInEnvironment(tier: KarmaTier, envId: DID): TrustProfile[]        ││
│   │                                                                                 ││
│   │   TRANSACTION DOMAIN                                                            ││
│   │   ──────────────────                                                            ││
│   │   «Repository»                                                                  ││
│   │   AgentRepository                                                               ││
│   │   + save(agent: Agent): void                                                    ││
│   │   + findById(id: DID): Agent?                                                   ││
│   │   + findByOwner(ownerId: DID): Agent[]                                          ││
│   │   + findByEnvironment(envId: DID): Agent[]                                      ││
│   │                                                                                 ││
│   │   «Repository»                                                                  ││
│   │   NegotiationRepository                                                         ││
│   │   + save(negotiation: Negotiation): void                                        ││
│   │   + findById(id: DID): Negotiation?                                             ││
│   │   + findByIntent(intentId: DID): Negotiation[]                                  ││
│   │   + findActiveByAgent(agentId: DID): Negotiation[]                              ││
│   │                                                                                 ││
│   │   SEMANTIC DOMAIN                                                               ││
│   │   ───────────────                                                               ││
│   │   «Repository»                                                                  ││
│   │   BlueprintRepository                                                           ││
│   │   + save(blueprint: Blueprint): void                                            ││
│   │   + findById(id: DID): Blueprint?                                               ││
│   │   + findByAuthor(authorId: DID): Blueprint[]                                    ││
│   │   + search(query: SemanticQuery): Blueprint[]                                   ││
│   │                                                                                 ││
│   │   GOVERNANCE DOMAIN                                                             ││
│   │   ─────────────────                                                             ││
│   │   «Repository»                                                                  ││
│   │   EnvironmentRepository                                                         ││
│   │   + save(env: Environment): void                                                ││
│   │   + findById(id: DID): Environment?                                             ││
│   │   + findByParent(parentId: DID): Environment[]                                  ││
│   │   + findByMember(memberId: DID): Environment[]                                  ││
│   │                                                                                 ││
│   │   LEDGER DOMAIN                                                                 ││
│   │   ─────────────                                                                 ││
│   │   «Repository»                                                                  ││
│   │   AMORepository                                                                 ││
│   │   + save(amo: AMO): void                                                        ││
│   │   + findById(id: DID): AMO?                                                     ││
│   │   + findByBlueprint(blueprintId: DID): AMO[]                                    ││
│   │   + findByOwner(ownerId: DID): AMO[]                                            ││
│   │                                                                                 ││
│   │   «Repository»                                                                  ││
│   │   EventRepository                                                               ││
│   │   + save(event: Event): void                                                    ││
│   │   + findById(id: DID): Event?                                                   ││
│   │   + findByCause(causeId: DID): Event[]                                          ││
│   │   + findByParticipant(participantId: DID, range: TimeRange): Event[]            ││
│   │                                                                                 ││
│   └─────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 12. Ubiquitous Language

| Begriff              | Definition                                                                      | Domain              |
| -------------------- | ------------------------------------------------------------------------------- | ------------------- |
| **Identity**         | Eindeutige, kryptografisch verifizierbare Entität mit DID                       | Identity            |
| **DID**              | Decentralized Identifier im Format `did:erynoa:<namespace>:<id>`                | Identity            |
| **Sub-Identity**     | Spezialisierte Identität mit eingeschränkten Capabilities                       | Identity            |
| **Credential**       | Signierte Aussage eines Issuers über ein Subject                                | Identity            |
| **Trust Vector**     | Vierdimensionaler Vertrauenswert [reliability, integrity, capability, reputation] | Trust             |
| **Karma**            | Akkumulierte Vertrauenspunkte aus Events                                        | Trust               |
| **Karma Tier**       | Gestaffelte Vertrauensstufe (Newcomer → Elder)                                  | Trust               |
| **Attestation**      | Externe Bestätigung von vertrauenswürdigen Dritten                              | Trust               |
| **Agent**            | Autonome digitale Einheit, die Interessen vertritt                              | Transaction         |
| **Intent**           | Formalisierte Absichtserklärung eines Agenten                                   | Transaction         |
| **Policy**           | Entscheidungsregeln für autonomes Agenten-Handeln                               | Transaction         |
| **Negotiation**      | Prozess von Intent zu Agreement                                                 | Transaction         |
| **Agreement**        | Vereinbarung zwischen Seeker und Provider                                       | Transaction         |
| **Payment Stream**   | Kontinuierlicher Werttransfer während laufender Dienste                         | Transaction         |
| **Blueprint**        | Domänenspezifische Schablone für Objekte                                        | Semantic            |
| **Standard**         | Normative Referenz (ISO, OCPP, etc.)                                            | Semantic            |
| **Ontologie**        | Begriffsrelationen und Taxonomien                                               | Semantic            |
| **Environment**      | Abgegrenzte Kontextblase mit spezifischen Regeln                                | Governance          |
| **Constraint**       | Regel, die innerhalb eines Environments gilt                                    | Governance          |
| **Proposal**         | Governance-Änderungsvorschlag                                                   | Governance          |
| **Council**          | Governance-Gremium eines Environments                                           | Governance          |
| **AMO**              | Atomic Managed Object – universelle Objektrepräsentation                        | Ledger              |
| **Event**            | Kausales Ereignis auf dem NOA Ledger                                            | Ledger              |
| **Logic Guard**      | Deterministisches Programm zur Transitionsvalidierung                           | Ledger              |
| **Finality**         | Zustand der Unveränderlichkeit durch Anchoring                                  | Ledger              |
| **Anchor**           | Merkle Root eines Event-Batches auf externer Chain                              | Ledger              |

---

## 13. Weiterführende Dokumente

| Bereich               | Pfad                                                 |
| --------------------- | ---------------------------------------------------- |
| Systemarchitektur     | [SYSTEM-ARCHITECTURE.md](./SYSTEM-ARCHITECTURE.md)   |
| Navigator             | [00-navigator.md](./00-navigator.md)                 |
| Identity Details      | [anker/](./anker/)                                   |
| Trust Details         | [metrik/](./metrik/)                                 |
| Transaction Details   | [impuls/](./impuls/)                                 |
| Semantic Details      | [schema/](./schema/)                                 |
| Governance Details    | [sphaere/](./sphaere/)                               |
| Ledger Details        | [chronik/](./chronik/)                               |
| ECL Referenz          | [appendix/ecl-referenz.md](./appendix/ecl-referenz.md) |
| Use Cases             | [appendix/anwendungen.md](./appendix/anwendungen.md) |
