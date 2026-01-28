# ◉ ANKER – Sub-Identities

> **Schicht:** 0 – Fundament
> **Sphäre:** ERY (DACS-Modul)
> **Version:** 2.0 – 16 spezialisierte Typen

---

## Konzept

Sub-Identities sind **spezialisierte Identitäten** für unterschiedliche Zwecke. Jede Haupt-DID kann bis zu 16 Sub-Identity-Typen erstellen.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         SUB-IDENTITY ARCHITEKTUR                            │
│                                                                             │
│                    HAUPT-IDENTITÄT (ROOT)                                   │
│                did:erynoa:org:erynoa-gmbh                                   │
│                                                                             │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │                    SUB-IDENTITIES                                 │    │
│   ├───────────────────────────────────────────────────────────────────┤    │
│   │                                                                   │    │
│   │  🔁 Trading       🗳️ Voting        🔐 Recovery                     │    │
│   │  did:...:sub:1    did:...:sub:2    did:...:sub:3                   │    │
│   │  Transfer, Stake  Vote, Delegate   Recover Only                    │    │
│   │                                                                   │    │
│   │  👥 Social        📱 Device        🔧 Service                      │    │
│   │  did:...:sub:4    did:...:sub:5    did:...:sub:6                   │    │
│   │  Connect, Msg     Sensor, Report   Provide, Consume                │    │
│   │                                                                   │    │
│   │  👮 Admin         📜 Compliance    🔍 Audit                        │    │
│   │  did:...:sub:7    did:...:sub:8    did:...:sub:9                   │    │
│   │  Full Control     Regulatory       Read-Only                       │    │
│   │                                                                   │    │
│   │  + 7 weitere spezialisierte Typen...                               │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Die 16 Sub-Identity-Typen

| Typ            | Capabilities                          | Use Case                     |
| -------------- | ------------------------------------- | ---------------------------- |
| **Trading**    | Transfer, Receive, Stake, Unstake     | Finanzielle Transaktionen    |
| **Voting**     | Vote, Delegate, Propose               | Environment Governance       |
| **Recovery**   | Recover, Reset (eingeschränkt)        | Notfall-Wiederherstellung    |
| **Social**     | Connect, Message, Endorse             | Soziale Interaktionen        |
| **Device**     | Sensor, Actuate, Report               | IoT-Geräte-Binding           |
| **Service**    | Provide, Consume, Subscribe           | Service-spezifische Aktionen |
| **Admin**      | Full Control (alle Capabilities)      | Hauptverwaltung              |
| **Compliance** | Regulatory, Audit, Report             | Regulatorische Anforderungen |
| **Audit**      | Read, Verify (keine Schreibrechte)    | Externe Prüfung              |
| **Delegation** | Delegate, Revoke                      | Berechtigungsweitergabe      |
| **Emergency**  | Emergency Actions (zeitlich begrenzt) | Notfallsituationen           |
| **Backup**     | Backup, Restore                       | Datensicherung               |
| **API**        | API Access, Rate-Limited              | Programmatischer Zugriff     |
| **Analytics**  | Read Aggregated Data                  | Datenanalyse                 |
| **Testing**    | Sandbox Operations                    | Test-Umgebungen              |
| **Custom**     | User-Defined Capabilities             | Benutzerdefiniert            |

---

## Sub-Identity Hierarchie

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   SUB-IDENTITY HIERARCHIE                                    │
│   ═══════════════════════════                                │
│                                                              │
│   Root-Identity (Agent/User/Org)                            │
│      │                                                       │
│      ├── Avatar (Umgebung A)                                │
│      │      └── Session (Transaktion)                       │
│      │                                                       │
│      ├── Avatar (Umgebung B)                                │
│      │      └── Session (Transaktion)                       │
│      │                                                       │
│      ├── Delegate (Autonome Aufgabe)                        │
│      │      └── Session (Verhandlung)                       │
│      │                                                       │
│      ├── Ownership Anchor (Asset)                           │
│      │                                                       │
│      └── Bundle (Asset-Sammlung)                            │
│             ├── Ownership Anchor → Asset A                  │
│             ├── Ownership Anchor → Asset B                  │
│             └── Ownership Anchor → Asset C                  │
│                                                              │
│   💡 Scope verengt sich mit jeder Ebene                     │
│   💡 Trust wird anteilig vererbt (70%)                      │
│   💡 Vollständige Audit-Trail                               │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## Sub-Identity Namespaces

| Namespace          | Beschreibung               | Beispiel-DID                                          |
| ------------------ | -------------------------- | ----------------------------------------------------- |
| **sub:avatar**     | Umgebungs-Repräsentation   | `did:erynoa:sub:avatar:a1b2c3d4:hubject-network`      |
| **sub:delegate**   | Delegierte Befugnisse      | `did:erynoa:sub:delegate:e5f6g7h8:night-negotiator`   |
| **sub:ownership**  | Besitz-Anker für Assets    | `did:erynoa:sub:ownership:i9j0k1l2:vehicle-vin-123`   |
| **sub:session**    | Session-gebundene Identity | `did:erynoa:sub:session:m3n4o5p6:charging-20250128`   |
| **sub:bundle**     | Asset-Bündel               | `did:erynoa:sub:bundle:q7r8s9t0:fleet-north`          |
| **sub:proxy**      | Temporärer Stellvertreter  | `did:erynoa:sub:proxy:u1v2w3x4:emergency-handler`     |
| **sub:capability** | Capability-Träger          | `did:erynoa:sub:capability:y5z6a7b8:payment-auth`     |
| **sub:persona**    | Kontext-spezifische Rolle  | `did:erynoa:sub:persona:c9d0e1f2:business-context`    |
| **sub:guardian**   | Treuhänder/Vormund         | `did:erynoa:sub:guardian:g3h4i5j6:iot-device-custody` |
| **sub:custodian**  | Verwahrer für Assets       | `did:erynoa:sub:custodian:k7l8m9n0:cold-storage`      |

---

## Lifecycle in ECL

```yaml
# Sub-Identity erstellen
sub_identity create {
  parent:       @identity("did:erynoa:org:erynoa-gmbh")
  kind:         "Trading"
  capabilities: [Transfer, Receive, Stake]
  expiry:       @duration("365d")
  revocable_by: ["Admin", "Recovery"]
}

# Sub-Identity verwenden
action transfer_funds {
  using_identity: @sub_identity("Trading")
  # Nur Trading-Capabilities verfügbar
}

# Sub-Identity widerrufen (bei Kompromittierung)
sub_identity revoke {
  target:  @sub_identity("Trading")
  reason:  "Key compromised"
  by:      @sub_identity("Admin")
}
```

---

## Vorteile

| Aspekt                   | Vorteil                                                 |
| ------------------------ | ------------------------------------------------------- |
| **Minimale Exposition**  | Nur benötigte Capabilities werden offengelegt           |
| **Einzelne Revocation**  | Kompromittierte Sub-IDs widerrufbar ohne Hauptidentität |
| **Audit Trail**          | Jede Sub-ID führt eigenes Event-Log                     |
| **Karma-Integration**    | Sub-Identities erben und akkumulieren Trust             |
| **Zeitliche Begrenzung** | Sub-IDs können automatisch ablaufen                     |

---

## Weiterführende Dokumente

- [identity-first.md](./identity-first.md) – Das Paradigma
- [did-erynoa.md](./did-erynoa.md) – DID-Namespaces
- [dacs.md](./dacs.md) – Multi-Chain Anchoring
