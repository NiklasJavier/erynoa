# ◊ METRIK – Attestations

> **Schicht:** 2 – Vertrauen
> **Sphäre:** ERY (Karmic-Modul)
> **Typ:** Externe Bestätigungen

---

## Konzept

**Attestations** sind signierte Aussagen über ein Subjekt von vertrauenswürdigen Dritten. Sie bilden die zweite Säule des Trust-Modells neben verhaltensbasierten Events.

---

## Attestation-Typen

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            ATTESTATION TYPES                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   🌐 DNS-Attestation                                                        │
│   ══════════════════                                                        │
│   DID ──────────────▶ DNS-TXT-Record ──────────────▶ Domain Ownership       │
│                                                                             │
│   "did:erynoa:operator-123" ↔ "charging.example.com"                       │
│                                                                             │
│   ─────────────────────────────────────────────────────────────────────    │
│                                                                             │
│   📜 Zertifikats-Attestation                                                │
│   ══════════════════════════                                                │
│   Zertifizierer ─────▶ Signierte Aussage ─────▶ Credential AMO             │
│                                                                             │
│   "TÜV bestätigt: Ladesäule entspricht OCPP 2.0.1"                         │
│                                                                             │
│   ─────────────────────────────────────────────────────────────────────    │
│                                                                             │
│   🏢 Organisations-Attestation                                              │
│   ═════════════════════════════                                             │
│   Dachverband ───────▶ Mitgliedschaftsnachweis ──▶ Trust-Bonus             │
│                                                                             │
│   "Bundesverband Elektromobilität e.V. – Mitglied seit 2024"               │
│                                                                             │
│   ─────────────────────────────────────────────────────────────────────    │
│                                                                             │
│   👥 Peer-Attestation                                                       │
│   ═══════════════════                                                       │
│   Anderer Agent ─────▶ Endorsement ──────────────▶ Reputation-Boost        │
│                                                                             │
│   "Agent X bestätigt erfolgreiche Zusammenarbeit mit Agent Y"              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Attestation-Struktur

```yaml
attestation {
  id:      "did:erynoa:attestation:rating-2025-001"
  type:    "certification"

  issuer:  @identity("did:erynoa:org:tuev-sued")
  subject: @identity("did:erynoa:amo:material:station-001")

  claims: {
    standard:    @ref("did:erynoa:standard:ocpp:2.0.1")
    compliance:  "full"
    test_date:   "2025-01-10"
    valid_until: "2027-01-10"
  }

  trust_impact: {
    dimension:  reputation
    boost:      0.15
    weight:     0.9  # Issuer-Trust × Weight
  }

  proof: {
    type:   "Ed25519Signature2020"
    value:  "z58DAdFfa9..."
  }
}
```

---

## Trust-Impact von Attestations

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ATTESTATION TRUST FLOW                                                   │
│                                                                             │
│   ┌─────────────────┐                                                      │
│   │  TÜV SÜD        │ ← Issuer Trust: 0.98                                │
│   │  did:erynoa:    │                                                      │
│   │  org:tuev-sued  │                                                      │
│   └────────┬────────┘                                                      │
│            │                                                                │
│            │ issues attestation                                             │
│            │ (boost: 0.15, weight: 0.9)                                    │
│            ▼                                                                │
│   ┌─────────────────┐                                                      │
│   │  Station-001    │                                                      │
│   │                 │                                                      │
│   │  Trust-Boost:   │                                                      │
│   │  0.98 × 0.9 × 0.15 = +0.132                                           │
│   │                 │                                                      │
│   │  Reputation:    │                                                      │
│   │  0.75 → 0.88    │                                                      │
│   └─────────────────┘                                                      │
│                                                                             │
│   Attestation-Impact = Issuer-Trust × Weight × Boost                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Attestation-Kategorien

| Kategorie            | Issuer                     | Impact          | Dimension              |
| -------------------- | -------------------------- | --------------- | ---------------------- |
| **Certification**    | Prüfinstitute (TÜV, DEKRA) | Hoch (+0.15)    | Capability, Reputation |
| **Membership**       | Verbände, Netzwerke        | Mittel (+0.08)  | Reputation             |
| **DNS Ownership**    | Selbst (via DNS)           | Niedrig (+0.03) | Integrity              |
| **Peer Endorsement** | Andere Agenten             | Variabel        | Reputation             |
| **Compliance**       | Regulatoren                | Hoch (+0.12)    | Integrity, Capability  |

---

## Attestation-Lifecycle

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ATTESTATION LIFECYCLE                                                    │
│                                                                             │
│   1. ISSUANCE                                                              │
│      Issuer erstellt und signiert Attestation                              │
│      → Trust-Boost wird angewendet                                         │
│                                                                             │
│   2. VERIFICATION                                                          │
│      Dritte können Attestation jederzeit prüfen                            │
│      → Signatur + Issuer-DID + Gültigkeit                                  │
│                                                                             │
│   3. EXPIRATION                                                            │
│      Nach valid_until verliert Attestation Wirkung                         │
│      → Trust-Boost wird zurückgenommen                                     │
│                                                                             │
│   4. REVOCATION                                                            │
│      Issuer kann Attestation widerrufen                                    │
│      → Trust-Boost wird sofort entfernt                                    │
│      → Penalty möglich bei Widerrufgrund                                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Weiterführende Dokumente

- [trust-vectors.md](./trust-vectors.md) – Dimensionen
- [karma-engine.md](./karma-engine.md) – Berechnung
- [reputation.md](./reputation.md) – Vererbung
- [../anker/credentials.md](../anker/credentials.md) – Verifiable Credentials
