# ◈ SCHEMA – Normative Standards

> **Schicht:** 1 – Wissen
> **Sphäre:** ERY (Semantic-Modul)
> **Typ:** Externe Industrienormen

---

## Konzept

**Normative Standards** sind etablierte Industrie- und Regulierungsstandards, die als fundamentale Referenzen in ERY verankert werden. Sie bilden die stabilsten Wissensschichten.

---

## Standard-Kategorien

### Geografie & Lokalisierung

| Standard      | Domäne         | Anwendung                        |
| ------------- | -------------- | -------------------------------- |
| **ISO 19112** | Geografie      | Geo-Kontexte, Koordinatensysteme |
| **Geohash**   | Lokalisierung  | Räumliche Indizierung            |
| **H3**        | Hexagonal Grid | Räumliche Aggregation            |

### Industrie & Produkte

| Standard         | Domäne                | Anwendung           |
| ---------------- | --------------------- | ------------------- |
| **eCl@ss**       | Produktklassifikation | Technische Merkmale |
| **ISO 55000**    | Asset Management      | Lifecycle-Daten     |
| **DIN EN 61850** | Energietechnik        | Kommunikation       |

### E-Mobilität

| Standard            | Domäne         | Anwendung                     |
| ------------------- | -------------- | ----------------------------- |
| **OCPP 2.0.1**      | Ladeprotokolle | Station-Backend-Kommunikation |
| **ISO 15118**       | Plug & Charge  | Fahrzeug-Station-Auth         |
| **Eichrecht (PTB)** | Metrologie     | Abrechnungssicherheit         |
| **OCPI**            | Roaming        | Interoperabilität             |

### Finanzen & Compliance

| Standard    | Domäne      | Anwendung              |
| ----------- | ----------- | ---------------------- |
| **AML/KYC** | Identität   | Geldwäscheprävention   |
| **PSD2**    | Zahlungen   | Open Banking           |
| **GDPR**    | Datenschutz | Personenbezogene Daten |

### Identität & Credentials

| Standard      | Domäne               | Anwendung                |
| ------------- | -------------------- | ------------------------ |
| **W3C DID**   | Dezentrale Identität | DID-Spezifikation        |
| **W3C VC**    | Credentials          | Verifiable Credentials   |
| **eIDAS 2.0** | EU-Identität         | Qualifizierte Signaturen |

---

## Standard-Struktur in ERY

```yaml
standard {
  # Identität
  id: "did:erynoa:standard:ocpp:2.0.1"
  name: "Open Charge Point Protocol"
  version: "2.0.1"

  # Herausgeber
  publisher: {
    name: "Open Charge Alliance"
    url:  "https://openchargealliance.org"
  }

  # Referenz
  reference: {
    type:    "specification"
    url:     "https://openchargealliance.org/protocols/ocpp-201/"
    date:    "2020-04-01"
  }

  # Anwendungsbereich
  scope: {
    domain:    "ev-charging"
    applies_to: ["station", "backend", "csms"]
  }

  # Verknüpfte Blueprints
  used_by: [
    @ref("did:erynoa:blueprint:ev-charging-station:*"),
    @ref("did:erynoa:blueprint:csms:*")
  ]

  # Trust-Eigenschaften
  trust: {
    established:  true
    trust_boost:  0.15  # Compliance erhöht Trust
  }
}
```

---

## Standard-Verknüpfung

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   STANDARD → BLUEPRINT → AMO                                               │
│                                                                             │
│   ┌─────────────────┐                                                      │
│   │  ISO 15118      │ ← Normative Basis                                   │
│   │  (Standard)     │                                                      │
│   └────────┬────────┘                                                      │
│            │                                                                │
│            │ referenziert von                                               │
│            ▼                                                                │
│   ┌─────────────────┐                                                      │
│   │  EV-Charging    │ ← Domänen-Anwendung                                 │
│   │  Blueprint      │                                                      │
│   │  + iso15118_support: required                                         │
│   └────────┬────────┘                                                      │
│            │                                                                │
│            │ instanziiert zu                                                │
│            ▼                                                                │
│   ┌─────────────────┐                                                      │
│   │  Station-001    │ ← Konkretes Objekt                                  │
│   │  AMO            │                                                      │
│   │  + iso15118: true                                                     │
│   └─────────────────┘                                                      │
│                                                                             │
│   Standard-Compliance wird transitiv vererbt und verifizierbar.            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Trust-Propagation durch Standards

Compliance mit anerkannten Standards erhöht Trust:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   STANDARD TRUST BOOST                                                     │
│                                                                             │
│   Station ohne ISO 15118    →  Trust: 0.75                                 │
│   Station mit ISO 15118     →  Trust: 0.75 + 0.10 = 0.85                   │
│   Station mit ISO + Eichrecht →  Trust: 0.75 + 0.10 + 0.08 = 0.93          │
│                                                                             │
│   Compliance-Nachweise werden als Credentials gespeichert.                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Weiterführende Dokumente

- [semantic-index.md](./semantic-index.md) – Speicherung
- [blueprints.md](./blueprints.md) – Standard-Anwendung
- [ontologie.md](./ontologie.md) – Begriffsrelationen
