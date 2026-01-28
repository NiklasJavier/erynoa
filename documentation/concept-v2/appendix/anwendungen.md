# 📖 APPENDIX – Anwendungen

> **Typ:** Referenz
> **Zweck:** Konkrete Use Cases

---

## Use Case 1: EV-Charging

### Szenario

Ein Elektrofahrzeug sucht autonom eine Ladestation, verhandelt den Preis und bezahlt während des Ladens.

### Beteiligte

| Rolle            | DID                                          | Typ          |
| ---------------- | -------------------------------------------- | ------------ |
| Fahrzeug-Agent   | `did:erynoa:agent:seeker:vehicle-123`        | Seeker       |
| Ladesäulen-Agent | `did:erynoa:agent:provider:swm-001`          | Provider     |
| Ladesäule        | `did:erynoa:amo:material:station-munich-001` | AMO          |
| Betreiber        | `did:erynoa:org:stadtwerke-munich`           | Organization |

### Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   1. PERCEPTION                                                            │
│      Fahrzeug: Batterie 20%, Position: München-Zentrum                     │
│      Agent startet Discovery                                               │
│                                                                             │
│   2. DISCOVERY                                                             │
│      Query: 50kW+, CCS, 5km Radius, Trust > 0.7                           │
│      Result: 5 Stationen gefunden                                          │
│                                                                             │
│   3. INTENT                                                                │
│      Fahrzeug-Agent erstellt Intent                                        │
│      Budget: max 30€, Priorität: Preis > Distanz                          │
│                                                                             │
│   4. NEGOTIATION                                                           │
│      Station SWM-001: Angebot 0.42€/kWh                                   │
│      Policy: Auto-Accept (unter 0.50€) → ACCEPT                           │
│                                                                             │
│   5. AGREEMENT                                                             │
│      Vertrag erstellt, beide Parteien signieren                           │
│                                                                             │
│   6. EXECUTION                                                             │
│      Ladevorgang startet                                                   │
│      Streaming Payment: 0.42€ pro kWh                                     │
│                                                                             │
│   7. FINALIZATION                                                          │
│      45 kWh geladen, 18.90€ bezahlt                                       │
│      Event finalized auf IOTA                                              │
│                                                                             │
│   8. FEEDBACK                                                              │
│      Provider Trust: +0.02                                                 │
│      Seeker Trust: +0.02                                                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Use Case 2: Fleet Management

### Szenario

Ein Flottenmanager verwaltet 50 Elektrofahrzeuge, die autonom laden, warten und optimieren.

### Struktur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   FLEET HIERARCHY                                                          │
│                                                                             │
│   did:erynoa:org:logistics-corp                                            │
│   │                                                                        │
│   ├── did:erynoa:agent:broker:fleet-manager                                │
│   │   (Koordiniert alle Fahrzeuge)                                         │
│   │                                                                        │
│   ├── did:erynoa:agent:seeker:vehicle-001                                  │
│   │   └── did:erynoa:amo:material:ev-001                                   │
│   │                                                                        │
│   ├── did:erynoa:agent:seeker:vehicle-002                                  │
│   │   └── did:erynoa:amo:material:ev-002                                   │
│   │                                                                        │
│   └── ... (48 weitere Fahrzeuge)                                           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Features

- **Zentrale Policy**: Alle Fahrzeuge folgen Fleet-Policy
- **Budget-Management**: Gesamt-Budget wird auf Fahrzeuge verteilt
- **Trust-Aggregation**: Fleet-Trust basiert auf allen Fahrzeugen
- **Reporting**: Alle Events werden aggregiert

---

## Use Case 3: Energy Trading

### Szenario

Prosumer mit Solaranlage verkauft Überschuss direkt an Nachbarn.

### Beteiligte

| Rolle         | DID                                        | Funktion         |
| ------------- | ------------------------------------------ | ---------------- |
| Prosumer      | `did:erynoa:agent:provider:solar-house-01` | Verkauft Energie |
| Consumer      | `did:erynoa:agent:seeker:house-02`         | Kauft Energie    |
| Grid Operator | `did:erynoa:agent:validator:grid-op`       | Validiert Physik |
| Smart Meter   | `did:erynoa:amo:material:meter-001`        | Misst Verbrauch  |

### Flow

```yaml
# Prosumer bietet Energie an
intent {
  type:   sell_energy
  provider: @identity("did:erynoa:agent:provider:solar-house-01")

  offer: {
    amount:    5.0   # kWh
    price:     0.18  # EUR/kWh
    available: "2025-01-28T12:00:00Z"
    duration:  3600  # 1 Stunde
  }

  constraints: {
    max_distance: 500m  # Lokaler Verkauf
  }
}

# Consumer akzeptiert
agreement {
  seller:   @identity("...solar-house-01")
  buyer:    @identity("...house-02")

  terms: {
    amount: 3.0  # kWh
    price:  0.18
    total:  0.54
  }

  # Grid Operator validiert Transaktion
  validator: @identity("...grid-op")
}
```

---

## Use Case 4: KYC & Compliance

### Szenario

Finanzdienstleister verifiziert Identität und teilt Credentials.

### Credential-Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   KYC CREDENTIAL FLOW                                                      │
│                                                                             │
│   1. VERIFICATION                                                          │
│      ══════════════                                                        │
│      User → Bank: Identitätsdokumente                                      │
│      Bank prüft: Pass, Adresse, etc.                                       │
│                                                                             │
│   2. CREDENTIAL ISSUANCE                                                   │
│      ═══════════════════════                                               │
│      Bank issues: did:erynoa:credential:kyc:user-123                       │
│      Claims: name_verified, address_verified, aml_cleared                  │
│                                                                             │
│   3. CREDENTIAL USAGE                                                      │
│      ═══════════════════                                                   │
│      User → Service: Präsentiert Credential                                │
│      Service prüft: Signatur, Issuer-Trust, Gültigkeit                    │
│                                                                             │
│   4. SELECTIVE DISCLOSURE                                                  │
│      ════════════════════════                                              │
│      User teilt nur: "Ich bin über 18" (ohne Geburtsdatum)                │
│      Zero-Knowledge-Proof möglich                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Use Case 5: Maintenance & Certification

### Szenario

Wartungsdienstleister führt Wartung durch und stellt Zertifikat aus.

### Credential-Kette

```yaml
# 1. Wartungsauftrag
agreement {
  id:       "did:erynoa:agreement:maintenance-001"
  customer: @identity("did:erynoa:org:stadtwerke-munich")
  provider: @identity("did:erynoa:org:tuev-sued")

  asset: @ref("did:erynoa:amo:material:station-munich-001")
  service: "annual_inspection"
}

# 2. Wartungsdurchführung (Service-AMO)
amo {
  id:   "did:erynoa:amo:service:inspection-001"
  type: service

  attributes: {
    inspection_date: "2025-01-28"
    result:          "passed"
    findings:        []
    next_due:        "2026-01-28"
  }
}

# 3. Zertifikat (Attestation)
attestation {
  id:      "did:erynoa:attestation:inspection-cert-001"
  issuer:  @identity("did:erynoa:org:tuev-sued")
  subject: @ref("did:erynoa:amo:material:station-munich-001")

  claims: {
    inspection_passed: true
    valid_until:       "2026-01-28"
    standard:          @ref("did:erynoa:standard:din-vde-0100")
  }

  # Trust-Impact
  trust_impact: {
    dimension: capability
    boost:     0.10
  }
}
```

---

## Use Case 6: Cross-Border Roaming

### Szenario

Deutsches Fahrzeug lädt in Frankreich über Roaming-Netzwerk.

### Environment-Überbrückung

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   CROSS-ENVIRONMENT CHARGING                                               │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   env:ev-charging-de              env:ev-charging-fr               │  │
│   │   ══════════════════              ══════════════════               │  │
│   │                                                                     │  │
│   │   ┌─────────────┐                 ┌─────────────┐                  │  │
│   │   │  Vehicle    │                 │  Station    │                  │  │
│   │   │  (German)   │                 │  (French)   │                  │  │
│   │   └──────┬──────┘                 └──────┬──────┘                  │  │
│   │          │                               │                         │  │
│   │          │                               │                         │  │
│   │          └───────────┐   ┌───────────────┘                         │  │
│   │                      │   │                                         │  │
│   │                      ▼   ▼                                         │  │
│   │                  ┌───────────┐                                     │  │
│   │                  │  Broker   │                                     │  │
│   │                  │ (Roaming) │                                     │  │
│   │                  │           │                                     │  │
│   │                  │ Beide Env │                                     │  │
│   │                  │ Member    │                                     │  │
│   │                  └───────────┘                                     │  │
│   │                                                                     │  │
│   │   Broker vermittelt zwischen Environments.                         │  │
│   │   Credentials werden gegenseitig anerkannt (Trust-Agreement).      │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Weiterführende Dokumente

- [glossar.md](./glossar.md) – Begriffsdefinitionen
- [ecl-referenz.md](./ecl-referenz.md) – Sprach-Referenz
