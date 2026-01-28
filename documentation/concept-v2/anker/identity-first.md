# ◉ ANKER – Identity-First Paradigma

> **Schicht:** 0 – Fundament
> **Sphäre:** ERY (DACS-Modul)
> **Kernfrage:** _„Wer existiert?"_

---

## Das Axiom

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   AXIOM: "Existenz durch Identifizierbarkeit"                              │
│   ═══════════════════════════════════════════                               │
│                                                                             │
│   Im Erynoa-Ökosystem gilt:                                                │
│                                                                             │
│   1. Eine Entität EXISTIERT, weil sie eine Identität HAT                   │
│   2. Ohne Identität ist keine Interaktion möglich                          │
│   3. Identität ist nicht optional – sie ist konstitutiv                    │
│   4. Alle Beziehungen sind Identitäts-Beziehungen                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Warum Identity-First?

### Das Problem ohne Identitätsfundament

| Symptom                   | Konsequenz                           |
| ------------------------- | ------------------------------------ |
| Fragmentierte IDs         | Keine systemweite Referenzierbarkeit |
| String-basierte Namen     | Kollisionen, keine Kryptografie      |
| Chain-Lock-in             | Identität nur auf einer Blockchain   |
| Zentralisierte Registries | Single Point of Failure              |

### Die Erynoa-Lösung

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   🎯 KERNAUSSAGE: ALLES HAT EINE IDENTITÄT                                 │
│                                                                             │
│   ENTITÄT                    →  HAT DID                                    │
│   ════════                      ════════                                   │
│                                                                             │
│   👤 Agent (Seeker)           →  did:erynoa:agent:seeker:...               │
│   👤 Agent (Provider)         →  did:erynoa:agent:provider:...             │
│   🏢 Organisation             →  did:erynoa:org:...                        │
│   🚗 Fahrzeug                 →  did:erynoa:vehicle:...                    │
│   📦 AMO (Objekt-Instanz)     →  did:erynoa:amo:...                        │
│   📋 Blueprint                →  did:erynoa:blueprint:...                  │
│   🌍 Environment              →  did:erynoa:env:...                        │
│   📜 Standard/Norm            →  did:erynoa:standard:...                   │
│   🎫 Credential               →  did:erynoa:vc:...                         │
│   💳 Wallet                   →  did:erynoa:wallet:...                     │
│   🗳️ Governance Proposal      →  did:erynoa:proposal:...                   │
│   📄 Intent                   →  did:erynoa:intent:...                     │
│   📜 Policy                   →  did:erynoa:policy:...                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Konsequenzen des Identity-First Paradigmas

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   KONSEQUENZEN                                                              │
│   ════════════                                                              │
│                                                                             │
│   • Jede Definition BEGINNT mit einer Identity-Deklaration                 │
│   • Referenzen zwischen Entitäten sind IMMER DID-Referenzen                │
│   • Berechtigungen werden an DIDs gebunden, nie an Namen                   │
│   • Audit-Trails verfolgen DIDs, nie ephemere Identifier                   │
│   • Trust akkumuliert sich an DIDs über Zeit                               │
│   • Versionen sind DID-linked (Blueprint-DID → Version-History)            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Die Transformation

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   VORHER: Fragmentierte Identität                                          │
│   ═══════════════════════════════                                           │
│                                                                             │
│   ┌───────────────┐  ┌───────────────┐  ┌───────────────┐                 │
│   │   Agents      │  │   Blueprints  │  │   Environments│                 │
│   │               │  │               │  │               │                 │
│   │  agent_id:123 │  │  bp:ev-charge │  │  env:germany  │                 │
│   │  (internal)   │  │  (string ref) │  │  (string ref) │                 │
│   └───────────────┘  └───────────────┘  └───────────────┘                 │
│          │                  │                  │                           │
│          ❌ Keine kryptografische Verifizierung                            │
│          ❌ Identifier-Kollisionen möglich                                 │
│          ❌ Keine dezentrale Auflösung                                     │
│          ❌ Trust kann nicht konsistent aggregiert werden                  │
│                                                                             │
│   ─────────────────────────────────────────────────────────────────────    │
│                                                                             │
│   NACHHER: Universal Identity Foundation                                   │
│   ══════════════════════════════════════                                    │
│                                                                             │
│                     ┌─────────────────────────────┐                        │
│                     │    🔐 IDENTITY LAYER        │                        │
│                     │    did:erynoa:*             │                        │
│                     │    Universal Namespace      │                        │
│                     └──────────────┬──────────────┘                        │
│                                    │                                       │
│       ┌────────────────────────────┼────────────────────────────┐         │
│       │                            │                            │         │
│   ┌───▼───────┐  ┌─────────────────▼────────────────┐  ┌───────▼───┐     │
│   │  Agent    │  │         Blueprint                 │  │Environment│     │
│   │           │  │                                   │  │           │     │
│   │ did:erynoa│  │ did:erynoa:blueprint:ev-charge:v1│  │did:erynoa │     │
│   │ :agent:   │  │                                   │  │:env:      │     │
│   │ seeker:123│  │ ← Eigene DID, versioniert        │  │germany    │     │
│   └───────────┘  │ ← Signiert vom Author            │  └───────────┘     │
│                  │ ← Trust akkumuliert               │                    │
│                  └───────────────────────────────────┘                    │
│                                                                             │
│          ✅ Kryptografisch verifizierbar (Ed25519)                         │
│          ✅ Global eindeutig (DID-Spezifikation)                           │
│          ✅ Dezentral auflösbar (DACS Network)                             │
│          ✅ Trust konsistent aggregierbar (Karmic Engine)                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Vorteile

| Aspekt                                  | Vorteil                                    |
| --------------------------------------- | ------------------------------------------ |
| **Universelle Referenzierbarkeit**      | Alles kann von überall referenziert werden |
| **Kryptografische Nachvollziehbarkeit** | Jede Aktion ist signiert und verifizierbar |
| **Unveränderliche Audit-Trails**        | Vollständige Historie an DIDs gebunden     |
| **Cross-System Interoperabilität**      | DIDs funktionieren über Chains hinweg      |
| **Dezentrale Verifizierung**            | Kein zentraler Authority-Server nötig      |

---

## Weiterführende Dokumente

- [did-erynoa.md](./did-erynoa.md) – DID-Syntax und Namespaces
- [sub-identities.md](./sub-identities.md) – 16 spezialisierte Identitätstypen
- [credentials.md](./credentials.md) – Verifiable Credentials
- [dacs.md](./dacs.md) – Multi-Chain Anchoring Protokoll
