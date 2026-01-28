# ◉ ANKER – Verifiable Credentials

> **Schicht:** 0 – Fundament
> **Sphäre:** ERY (DACS-Modul)
> **Standard:** W3C Verifiable Credentials Data Model

---

## Konzept

**Verifiable Credentials (VCs)** sind kryptografisch signierte Aussagen eines Issuers über ein Subjekt. Sie werden vom DACS ausgestellt und können von Dritten dezentral verifiziert werden.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   VERIFIABLE CREDENTIAL FLOW                                               │
│                                                                             │
│   ┌───────────┐        ┌───────────┐        ┌───────────┐                 │
│   │  ISSUER   │───────▶│ CREDENTIAL│───────▶│  HOLDER   │                 │
│   │           │ signs  │           │ stores │           │                 │
│   │ did:erynoa│        │ did:erynoa│        │ did:erynoa│                 │
│   │ :org:tuev │        │ :vc:cert  │        │ :vehicle: │                 │
│   └───────────┘        └───────────┘        └─────┬─────┘                 │
│                                                   │                        │
│                                                   │ presents               │
│                                                   ▼                        │
│                                            ┌───────────┐                  │
│                                            │ VERIFIER  │                  │
│                                            │           │                  │
│                                            │ did:erynoa│                  │
│                                            │ :agent:   │                  │
│                                            └───────────┘                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Credential-Typen

| Typ               | Beschreibung           | Beispiel                          |
| ----------------- | ---------------------- | --------------------------------- |
| **Identity**      | KYC/AML-Nachweise      | `vc:kyc:verified`                 |
| **License**       | Betriebslizenzen       | `vc:license:fleet-operator`       |
| **Certification** | Technische Zertifikate | `vc:cert:ocpp-2.0.1`              |
| **Membership**    | Mitgliedschaften       | `vc:member:hubject-roaming`       |
| **Capability**    | Berechtigungsnachweise | `vc:capability:charging-provider` |
| **Attestation**   | Trust-Bestätigungen    | `vc:attestation:reliability-high` |

---

## Credential-Struktur

```json
{
  "@context": [
    "https://www.w3.org/2018/credentials/v1",
    "https://erynoa.io/credentials/v1"
  ],
  "id": "did:erynoa:vc:cert:ocpp-station-001",
  "type": ["VerifiableCredential", "OCPPCertification"],

  "issuer": {
    "id": "did:erynoa:org:tuev-sued",
    "name": "TÜV SÜD"
  },

  "issuanceDate": "2025-01-15T10:00:00Z",
  "expirationDate": "2027-01-15T10:00:00Z",

  "credentialSubject": {
    "id": "did:erynoa:amo:material:station-munich-001",
    "certification": {
      "standard": "did:erynoa:standard:ocpp:2.0.1",
      "level": "full",
      "testDate": "2025-01-10"
    }
  },

  "proof": {
    "type": "Ed25519Signature2020",
    "created": "2025-01-15T10:00:00Z",
    "verificationMethod": "did:erynoa:org:tuev-sued#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z58DAdFfa9SkqZMVPxAQp..."
  }
}
```

---

## Trust-Propagation

Credentials propagieren Trust von Issuer zu Subject:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   TRUST PROPAGATION DURCH CREDENTIALS                                      │
│                                                                             │
│   ┌─────────────────┐                                                      │
│   │  TÜV SÜD        │ ← Trust: 0.98 (etablierte Institution)              │
│   │  did:erynoa:    │                                                      │
│   │  org:tuev-sued  │                                                      │
│   └────────┬────────┘                                                      │
│            │                                                                │
│            │ issues credential                                              │
│            │ (trust_weight: 0.8)                                           │
│            ▼                                                                │
│   ┌─────────────────┐                                                      │
│   │  Ladesäule      │ ← Trust-Boost: 0.98 × 0.8 = +0.78                   │
│   │  did:erynoa:    │                                                      │
│   │  amo:station    │                                                      │
│   └─────────────────┘                                                      │
│                                                                             │
│   Das Credential erhöht den Trust-Score des Subjekts                       │
│   proportional zum Issuer-Trust und Credential-Gewicht.                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Credential-Operationen

### Ausstellung

```yaml
credential issue {
  type:    "OCPPCertification"
  issuer:  @identity("did:erynoa:org:tuev-sued")
  subject: @identity("did:erynoa:amo:material:station-001")

  claims: {
    standard: @ref("did:erynoa:standard:ocpp:2.0.1")
    level:    "full"
    test_date: @date("2025-01-10")
  }

  validity: @duration("2y")
}
```

### Präsentation

```yaml
credential present {
  credentials: [
    @vc("did:erynoa:vc:cert:ocpp-station-001"),
    @vc("did:erynoa:vc:license:operator-munich")
  ]

  to:         @identity("did:erynoa:agent:seeker:fleet-001")
  purpose:    "charging_negotiation"
  selective:  ["certification.level", "certification.standard"]
}
```

### Widerruf

```yaml
credential revoke {
  target:  @vc("did:erynoa:vc:cert:ocpp-station-001")
  reason:  "Equipment modified without recertification"
  by:      @identity("did:erynoa:org:tuev-sued")
}
```

---

## Integration mit anderen Schichten

| Schicht       | Integration                                      |
| ------------- | ------------------------------------------------ |
| **◊ METRIK**  | Credentials beeinflussen Trust Vectors           |
| **▣ SPHÄRE**  | Credentials als Membership-Nachweis              |
| **◐ IMPULS**  | Agenten präsentieren Credentials bei Verhandlung |
| **◆ CHRONIK** | Credential-Events werden on-chain protokolliert  |

---

## Weiterführende Dokumente

- [identity-first.md](./identity-first.md) – Das Paradigma
- [dacs.md](./dacs.md) – DACS als Credential-Infrastruktur
- [../metrik/attestations.md](../metrik/attestations.md) – Trust-Attestationen
