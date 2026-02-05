# 🌐 Realm URL & Resource Addressing System

> **Teil von:** Projekt Pluto
> **Kategorie:** Adressierung & Ressourcen-Resolution
> **Status:** Spezifikation
> **Konstanten:** Κ26 (URL-Schema), Κ27 (Resource-Resolution), Κ28 (Open-Access-Policy)

---

## 1. Fundamentales Konzept: Realm als Namespace

### 1.1 Das Erynoa URL-Schema

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                         ERYNOA URL SCHEMA (Κ26)                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Format:                                                                    ║
║   ┌──────────────────────────────────────────────────────────────────────┐  ║
║   │  erynoa://<realm-id>/<resource-type>/<resource-path>[?<params>]      │  ║
║   └──────────────────────────────────────────────────────────────────────┘  ║
║                                                                              ║
║   Komponenten:                                                               ║
║   ─────────────────────────────────────────────────────────────────────────  ║
║   • erynoa://     → Protocol-Prefix (immutable)                             ║
║   • <realm-id>    → Realm als Namespace (Circle-DID oder Alias)             ║
║   • <resource-type> → Schema-definierter Ressourcentyp                      ║
║   • <resource-path> → Realm-interner Pfad zur Ressource                     ║
║   • <params>      → Optionale Query-Parameter                               ║
║                                                                              ║
║   Beispiele:                                                                 ║
║   ─────────────────────────────────────────────────────────────────────────  ║
║   erynoa://gaming-dao/asset/legendary-sword-001                             ║
║   erynoa://did:erynoa:circle:abc123/store/inventory/items                   ║
║   erynoa://defi-realm/contract/staking-v2/state                             ║
║   erynoa://social-hub/profile/alice?view=public                             ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 1.2 Realm-ID als Authority

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   REALM-ID RESOLUTION                                                        ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Zwei Formate akzeptiert:                                                  ║
║                                                                              ║
║   1. FULL DID:                                                               ║
║      erynoa://did:erynoa:circle:a1b2c3.../resource/path                     ║
║      → Direkte, unveränderliche Referenz                                    ║
║      → Kryptographisch verifikierbar                                        ║
║                                                                              ║
║   2. REALM ALIAS:                                                            ║
║      erynoa://gaming-dao/resource/path                                      ║
║      → Human-readable                                                       ║
║      → Auflösung via Root-Realm Registry                                    ║
║      → Alias → did:erynoa:circle:... Mapping                                ║
║                                                                              ║
║   Alias-Registrierung:                                                       ║
║   - Einmalig durch Governance-Proposal                                      ║
║   - Mana-Cost: 10000 (verhindert Squatting)                                 ║
║   - Muss unique im Root-Realm sein                                          ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 2. Resource Schema System (Κ27)

### 2.1 Schema-Architektur

Jedes Realm definiert sein eigenes **ResourceSchema**, das die verfügbaren Ressourcentypen und deren Auflösungslogik spezifiziert.

```ecl
// ECL-Definition eines Realm-Resource-Schemas
resource_schema "gaming-realm-schema" {
    version: "1.0.0",
    
    // Definierte Ressourcentypen
    types: {
        // Assets
        asset: {
            path_pattern: "asset/<category>/<asset-id>",
            resolver: "storage",
            store: "assets",
            access: "policy-controlled",
            
            // Felder die bei Auflösung verfügbar sind
            fields: ["id", "name", "rarity", "owner", "metadata"],
        },
        
        // Benutzerprofile
        profile: {
            path_pattern: "profile/<did-suffix>",
            resolver: "identity",
            personal: true,
            access: "owner-or-public",
            
            fields: ["display_name", "avatar", "bio", "achievements"],
        },
        
        // Shared Stores
        store: {
            path_pattern: "store/<store-name>/<key>",
            resolver: "storage",
            access: "realm-policy",
            
            // Dynamisch basierend auf Store-Schema
            fields: "dynamic",
        },
        
        // Contracts
        contract: {
            path_pattern: "contract/<contract-name>/<method>",
            resolver: "eclvm",
            access: "contract-policy",
            
            // Callable methods
            methods: ["state", "call", "events"],
        },
        
        // Events
        event: {
            path_pattern: "event/<event-type>/<timestamp>",
            resolver: "event-log",
            access: "members-only",
            
            fields: ["type", "data", "emitter", "timestamp"],
        },
    },
    
    // Default für unbekannte Typen
    fallback: {
        resolver: "storage",
        access: "deny",
    },
}
```

### 2.2 Built-in Resource Types

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   STANDARD RESOURCE TYPES (alle Realms)                                      ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Type         Pattern                        Resolver    Default Access     ║
║   ──────────────────────────────────────────────────────────────────────────  ║
║   store/       store/<name>/<key>             Storage     realm-policy       ║
║   profile/     profile/<did>                  Identity    owner-or-public    ║
║   contract/    contract/<name>/<method>       ECLVM       contract-policy    ║
║   asset/       asset/<category>/<id>          Storage     policy-controlled  ║
║   event/       event/<type>/<ts>              EventLog    members-only       ║
║   meta/        meta/<key>                     Metadata    public             ║
║   governance/  governance/<proposal-id>       Governance  members-only       ║
║   trust/       trust/<did>                    TrustCore   members-only       ║
║                                                                              ║
║   Custom-Types: Realm kann eigene Types via Schema definieren               ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 2.3 Schema-Vererbung

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   SCHEMA-VERERBUNG (Κ1 konforme Hierarchie)                                  ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Root-Realm (28 Axiome)                                                    ║
║   └── Base-Schema: store/, profile/, contract/, meta/, governance/, trust/  ║
║           │                                                                  ║
║           ▼                                                                  ║
║   Virtual Realm (Gaming-DAO)                                                 ║
║   └── Erbt: Base-Schema                                                     ║
║   └── Erweitert: asset/, inventory/, achievement/, match/                   ║
║           │                                                                  ║
║           ▼                                                                  ║
║   Partition (EU-Gaming)                                                      ║
║   └── Erbt: Gaming-DAO Schema                                               ║
║   └── Erweitert: region/, tournament/                                       ║
║                                                                              ║
║   REGEL: Kind-Realm kann nur ERWEITERN, nicht EINSCHRÄNKEN                  ║
║   → Κ1 (Regelvererbung): inherited_types ⊆ parent.types                     ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 3. Resource Resolution Engine

### 3.1 Auflösungs-Algorithmus

```rust
/// URL Resolution Result
pub struct ResolvedResource {
    pub realm_id: UniversalId,
    pub resource_type: String,
    pub resource_path: Vec<String>,
    pub resolver: ResourceResolver,
    pub access_policy: AccessPolicy,
    pub data: Option<serde_json::Value>,
}

/// Resource Resolver Trait
pub trait ResourceResolver {
    fn resolve(&self, ctx: &ResolutionContext) -> Result<ResolvedResource>;
    fn can_access(&self, ctx: &AccessContext) -> bool;
}

/// Resolution Algorithmus (Κ27)
pub fn resolve_url(url: &str, requester: &DID) -> Result<ResolvedResource> {
    // 1. Parse URL
    let parsed = ErnyoaUrl::parse(url)?;
    
    // 2. Resolve Realm-ID (Alias → DID wenn nötig)
    let realm_id = resolve_realm_id(&parsed.authority)?;
    
    // 3. Lade Realm-Schema
    let schema = load_realm_schema(&realm_id)?;
    
    // 4. Match Resource-Type gegen Schema
    let type_def = schema.match_type(&parsed.resource_type)
        .ok_or(Error::UnknownResourceType)?;
    
    // 5. Parse Path gemäß Type-Pattern
    let path_components = type_def.parse_path(&parsed.resource_path)?;
    
    // 6. Prüfe Access-Policy
    let access_ctx = AccessContext {
        requester: requester.clone(),
        realm_id: realm_id.clone(),
        resource_type: parsed.resource_type.clone(),
        path: path_components.clone(),
    };
    
    let access_result = evaluate_access(&type_def.access, &access_ctx)?;
    
    if !access_result.allowed {
        return Err(Error::AccessDenied(access_result.reason));
    }
    
    // 7. Resolve via Resolver
    let resolver = get_resolver(&type_def.resolver)?;
    resolver.resolve(&ResolutionContext {
        realm_id,
        resource_type: parsed.resource_type,
        path: path_components,
        params: parsed.params,
    })
}
```

### 3.2 Resolution Flow Diagram

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   URL RESOLUTION FLOW                                                        ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   erynoa://gaming-dao/asset/weapons/sword-001                               ║
║                                                                              ║
║   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌────────────┐   ║
║   │  URL Parse  │───▶│ Realm Lookup│───▶│Schema Match │───▶│Access Check│   ║
║   └─────────────┘    └─────────────┘    └─────────────┘    └────────────┘   ║
║         │                   │                  │                  │          ║
║         ▼                   ▼                  ▼                  ▼          ║
║   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌────────────┐   ║
║   │ authority:  │    │ alias →     │    │ type: asset │    │ Policy:    │   ║
║   │ gaming-dao  │    │ circle:abc  │    │ path:       │    │ open? ✓    │   ║
║   │ type: asset │    │             │    │ [weapons,   │    │ trust? ✓   │   ║
║   │ path:       │    │ Schema:     │    │  sword-001] │    │            │   ║
║   │ weapons/... │    │ gaming-v1   │    │             │    │            │   ║
║   └─────────────┘    └─────────────┘    └─────────────┘    └────────────┘   ║
║                                                                   │          ║
║                                                                   ▼          ║
║                                              ┌─────────────────────────────┐ ║
║                                              │       RESOLVER              │ ║
║                                              │  Storage.get(              │ ║
║                                              │    realm: circle:abc,      │ ║
║                                              │    store: "assets",        │ ║
║                                              │    key: "weapons:sword-001"│ ║
║                                              │  )                         │ ║
║                                              └─────────────────────────────┘ ║
║                                                            │                 ║
║                                                            ▼                 ║
║                                              ┌─────────────────────────────┐ ║
║                                              │    RESOLVED RESOURCE        │ ║
║                                              │  { id: "sword-001",        │ ║
║                                              │    name: "Flameblade",     │ ║
║                                              │    rarity: "legendary",    │ ║
║                                              │    owner: did:erynoa:... } │ ║
║                                              └─────────────────────────────┘ ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 4. Open-Access-Policy System (Κ28)

### 4.1 Konzept: Öffentliche Ressourcen für Nicht-Member

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   OPEN-ACCESS-POLICY (Κ28)                                                   ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Problem: Realm-Isolation verhindert externen Zugriff                       ║
║   Lösung: Policy-gesteuerte Open-Access für bestimmte Ressourcen            ║
║                                                                              ║
║   ┌─────────────────────────────────────────────────────────────────────┐   ║
║   │                         REALM                                       │   ║
║   │  ┌─────────────────────────────────────────────────────────────┐   │   ║
║   │  │           MEMBERS ONLY (Default)                            │   │   ║
║   │  │   • Personal Stores                                         │   │   ║
║   │  │   • Governance Proposals                                    │   │   ║
║   │  │   • Trust Scores                                            │   │   ║
║   │  │   • Internal Events                                         │   │   ║
║   │  └─────────────────────────────────────────────────────────────┘   │   ║
║   │                                                                     │   ║
║   │  ┌─────────────────────────────────────────────────────────────┐   │   ║
║   │  │           OPEN ACCESS (Policy-Controlled)                   │───┼──▶ External
║   │  │   • Public Profiles                                         │   │   ║
║   │  │   • Asset Metadata (read-only)                              │   │   ║
║   │  │   • Realm Info (meta/)                                      │   │   ║
║   │  │   • Open Stores (markiert als public)                       │   │   ║
║   │  └─────────────────────────────────────────────────────────────┘   │   ║
║   │                                                                     │   ║
║   └─────────────────────────────────────────────────────────────────────┘   ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 4.2 Access-Policy Definition

```ecl
// ECL: Open-Access-Policy für Realm
open_access_policy "public-gaming-assets" {
    
    // Welche Resource-Types sind öffentlich?
    public_types: {
        
        // Asset-Metadata ist öffentlich lesbar
        "asset": {
            operations: ["read"],
            
            // Nur bestimmte Felder
            fields: ["id", "name", "rarity", "image_url", "description"],
            
            // Keine Ownership-Info
            exclude_fields: ["owner", "history", "internal_value"],
            
            // Rate-Limiting für Non-Members
            rate_limit: {
                requests_per_minute: 60,
                burst: 10,
            },
        },
        
        // Profile ist public wenn Owner erlaubt
        "profile": {
            operations: ["read"],
            condition: "resource.public == true",
            fields: ["display_name", "avatar", "bio"],
        },
        
        // Realm-Meta ist immer public
        "meta": {
            operations: ["read"],
            fields: "*",  // Alle Felder
        },
    },
    
    // Trust-Anforderungen für Non-Member-Zugriff
    non_member_requirements: {
        // Minimaler globaler Trust
        min_global_trust_omega: 0.1,
        
        // Oder Mitglied in vertrautem Realm
        trusted_realms: ["root", "verified-users"],
    },
    
    // Mana-Cost für Non-Member-Requests
    non_member_mana_cost: 2,  // 2× Standardkosten
}
```

### 4.3 Access-Evaluation Matrix

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   ACCESS EVALUATION MATRIX                                                   ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Requester        Resource-Type    Policy          Result                   ║
║   ──────────────────────────────────────────────────────────────────────────  ║
║   Member           private-store    any             ✅ ALLOW                  ║
║   Member           public-type      any             ✅ ALLOW                  ║
║   Member           other-member     owner-only      ❌ DENY                   ║
║   ──────────────────────────────────────────────────────────────────────────  ║
║   Non-Member       public-type      Κ28 policy      ✅ ALLOW (filtered)       ║
║   Non-Member       private-store    any             ❌ DENY                   ║
║   Non-Member       meta/            default-public  ✅ ALLOW                  ║
║   ──────────────────────────────────────────────────────────────────────────  ║
║   Cross-Realm      any              Κ23 + Κ28       ⚖️ CROSSING-EVAL          ║
║   Anonymous        public-type      Κ28 + rate-lim  ⚠️ LIMITED                ║
║                                                                              ║
║   Priorität: Member-Status > Open-Policy > Crossing-Eval > Deny             ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 4.4 Integration mit Trust-System

```ecl
// Access-Evaluation mit Trust-Faktoren
access_evaluation "trust-enhanced" {
    
    // Base-Access aus Policy
    base_access: open_access_policy.evaluate(resource),
    
    // Trust-Modifikation
    trust_modifier: {
        // Höherer Trust = mehr Access
        if requester.global_trust_omega > 0.8 {
            unlock_fields: ["history", "statistics"],
        },
        
        // Negativer Trust = eingeschränkt
        if requester.global_trust_omega < 0.2 {
            rate_limit_factor: 0.5,  // Halb so viele Requests
            deny_fields: ["contact_info"],
        },
    },
    
    // Crossing-Dampening (Κ23)
    if requester.is_cross_realm {
        trust_factor: crossing_trust(requester.home_realm, this.realm),
        // T_cross = T_local × (1 - Κ23)
    },
}
```

---

## 5. URL × System-Integration

### 5.1 URL × Storage (RealmStorage)

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   URL → STORAGE MAPPING                                                      ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   URL: erynoa://gaming-dao/store/inventory/items/sword-001                  ║
║                                                                              ║
║   Mapped to RealmStorage Key:                                                ║
║   ┌────────────────────────────────────────────────────────────────────┐    ║
║   │  realm:{realm_id}:shared:store:inventory:items:sword-001           │    ║
║   └────────────────────────────────────────────────────────────────────┘    ║
║                                                                              ║
║   URL: erynoa://gaming-dao/profile/did:erynoa:self:user123                  ║
║                                                                              ║
║   Mapped to Personal Storage Key:                                            ║
║   ┌────────────────────────────────────────────────────────────────────┐    ║
║   │  realm:{realm_id}:personal:{did}:store:profile:data                │    ║
║   └────────────────────────────────────────────────────────────────────┘    ║
║                                                                              ║
║   Transformation:                                                            ║
║   - erynoa://<realm>/<type>/<path> → realm:<realm_id>:<scope>:<path>        ║
║   - Type bestimmt Scope (shared vs personal)                                ║
║   - Path-Segments werden zu Key-Hierarchie                                  ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 5.2 URL × DID (Identität)

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   URL × DID INTEGRATION                                                      ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   DID-Referenzen in URLs:                                                   ║
║   ──────────────────────────────────────────────────────────────────────────  ║
║                                                                              ║
║   Profile by DID:                                                            ║
║   erynoa://gaming-dao/profile/did:erynoa:self:abc123                        ║
║                                                                              ║
║   Profile by DID-Suffix (shorthand):                                         ║
║   erynoa://gaming-dao/profile/~abc123                                       ║
║   → Resolved: did:erynoa:self:abc123 (im Realm-Kontext)                     ║
║                                                                              ║
║   Agent's Resources:                                                         ║
║   erynoa://gaming-dao/agent/did:erynoa:spirit:agent42/state                 ║
║                                                                              ║
║   Trust-Query:                                                               ║
║   erynoa://gaming-dao/trust/did:erynoa:self:abc123                          ║
║   → Returns: { local_trust: 0.7, trust_vector: [...] }                      ║
║                                                                              ║
║   Cross-Reference zu anderem Realm:                                          ║
║   erynoa://gaming-dao/link/erynoa://other-realm/asset/item-001              ║
║   → Crossing (Κ23) wird angewendet                                          ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 5.3 URL × Governance

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   URL × GOVERNANCE INTEGRATION                                               ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Governance URLs (alle members-only außer explizit):                       ║
║   ──────────────────────────────────────────────────────────────────────────  ║
║                                                                              ║
║   Proposal lesen:                                                            ║
║   erynoa://gaming-dao/governance/proposal/prop-2024-042                     ║
║                                                                              ║
║   Alle aktiven Proposals:                                                    ║
║   erynoa://gaming-dao/governance/proposals?status=active                    ║
║                                                                              ║
║   Vote-Status eines Proposals:                                               ║
║   erynoa://gaming-dao/governance/proposal/prop-2024-042/votes               ║
║                                                                              ║
║   Treasury-Status (public meta):                                             ║
║   erynoa://gaming-dao/governance/treasury/balance                           ║
║   → Open-Access: Jeder kann Treasury-Balance sehen                          ║
║                                                                              ║
║   Governance-History (members-only):                                         ║
║   erynoa://gaming-dao/governance/history?from=2024-01-01                    ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 5.4 URL × ECLVM (Smart Contracts)

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   URL × ECLVM INTEGRATION                                                    ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Contract State lesen:                                                      ║
║   erynoa://gaming-dao/contract/marketplace/state                            ║
║   → Resolver: ECLVM                                                         ║
║   → Returns: Contract-Storage als JSON                                       ║
║                                                                              ║
║   Contract Method aufrufen (via URL):                                        ║
║   erynoa://gaming-dao/contract/marketplace/call/list_item?item_id=123       ║
║   → Erzeugt Transaction                                                     ║
║   → Benötigt Signatur (nicht idempotent)                                    ║
║                                                                              ║
║   Contract Events abfragen:                                                  ║
║   erynoa://gaming-dao/contract/marketplace/events?type=ItemSold             ║
║   → Returns: Liste von Events                                               ║
║                                                                              ║
║   Contract ABI / Schema:                                                     ║
║   erynoa://gaming-dao/contract/marketplace/abi                              ║
║   → Open-Access: ABI ist immer public                                       ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 5.5 URL × Package Manager

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   URL × PACKAGE MANAGER                                                      ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Package-Manifest Referenz:                                                 ║
║   ──────────────────────────────────────────────────────────────────────────  ║
║                                                                              ║
║   package.ecl:                                                               ║
║   dependencies: {                                                            ║
║       "common-lib": "erynoa://packages/pkg/common-lib@1.0.0",               ║
║       "realm-specific": "erynoa://gaming-dao/pkg/game-utils@2.1.0",         ║
║   },                                                                         ║
║                                                                              ║
║   URL-Auflösung:                                                             ║
║   1. erynoa://packages/pkg/common-lib@1.0.0                                 ║
║      → Root-Realm Package Registry                                          ║
║      → Download + Verify via Content-Hash                                    ║
║                                                                              ║
║   2. erynoa://gaming-dao/pkg/game-utils@2.1.0                               ║
║      → Realm-spezifisches Package                                           ║
║      → Access: Benötigt Membership ODER Open-Package-Policy                 ║
║                                                                              ║
║   Package-URL Format:                                                        ║
║   erynoa://<realm>/pkg/<name>@<version>[/<subpath>]                         ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 6. Query-Parameter & Operationen

### 6.1 Standard Query-Parameter

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   STANDARD QUERY PARAMETERS                                                  ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Parameter       Beschreibung                    Beispiel                   ║
║   ──────────────────────────────────────────────────────────────────────────  ║
║   ?view=         Ansicht (public, full, raw)     ?view=public               ║
║   ?fields=       Felder-Selektion                ?fields=id,name,rarity     ║
║   ?version=      Spezifische Version             ?version=1.2.3             ║
║   ?at=           Zeitpunkt (historisch)          ?at=2024-01-15T10:00:00Z   ║
║   ?limit=        Pagination Limit                ?limit=50                  ║
║   ?offset=       Pagination Offset               ?offset=100                ║
║   ?sort=         Sortierung                      ?sort=created_at:desc      ║
║   ?filter=       Filter-Ausdruck                 ?filter=rarity:legendary   ║
║   ?include=      Nested Resources inkludieren    ?include=owner,history     ║
║                                                                              ║
║   Kombiniert:                                                                ║
║   erynoa://gaming-dao/store/assets/all?filter=rarity:legendary&limit=10     ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 6.2 Operations auf URLs

```rust
/// URL Operations
pub enum UrlOperation {
    /// Read-Operationen (GET-äquivalent)
    Read {
        fields: Option<Vec<String>>,
        version: Option<String>,
        at: Option<DateTime>,
    },
    
    /// Write-Operationen (benötigen Signatur)
    Write {
        data: serde_json::Value,
        nonce: u64,
        signature: Signature,
    },
    
    /// Subscribe (WebSocket/Gossip)
    Subscribe {
        events: Vec<String>,
    },
    
    /// Execute (Contract-Call)
    Execute {
        method: String,
        args: serde_json::Value,
        gas_limit: u64,
        signature: Signature,
    },
}

/// URL mit Operation
pub struct OperationalUrl {
    pub url: ErnyoaUrl,
    pub operation: UrlOperation,
}
```

---

## 7. Rust-Implementierung

### 7.1 Core-Strukturen

```rust
use crate::core::{UniversalId, DID, TrustVector6D};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Erynoa URL (Κ26)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErnyoaUrl {
    /// Realm-ID (DID oder Alias)
    pub authority: RealmAuthority,
    /// Resource-Type (store, profile, contract, etc.)
    pub resource_type: String,
    /// Resource-Path Segments
    pub path: Vec<String>,
    /// Query-Parameters
    pub params: HashMap<String, String>,
    /// Fragment (optional)
    pub fragment: Option<String>,
}

/// Realm Authority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RealmAuthority {
    /// Full DID: did:erynoa:circle:...
    Did(DID),
    /// Human-readable Alias
    Alias(String),
}

impl ErnyoaUrl {
    /// Parse URL String
    pub fn parse(url: &str) -> Result<Self, UrlParseError> {
        // Expect: erynoa://<authority>/<type>/<path...>[?params][#fragment]
        if !url.starts_with("erynoa://") {
            return Err(UrlParseError::InvalidScheme);
        }
        
        let rest = &url[9..]; // Strip "erynoa://"
        let (authority_str, rest) = rest.split_once('/')
            .ok_or(UrlParseError::MissingAuthority)?;
        
        let authority = if authority_str.starts_with("did:erynoa:") {
            RealmAuthority::Did(DID::parse(authority_str)?)
        } else {
            RealmAuthority::Alias(authority_str.to_string())
        };
        
        // Parse rest (type/path?params#fragment)
        let (path_str, fragment) = rest.split_once('#')
            .map(|(p, f)| (p, Some(f.to_string())))
            .unwrap_or((rest, None));
        
        let (path_str, params) = path_str.split_once('?')
            .map(|(p, q)| (p, Self::parse_query(q)))
            .unwrap_or((path_str, HashMap::new()));
        
        let path_parts: Vec<&str> = path_str.split('/').collect();
        let resource_type = path_parts.first()
            .ok_or(UrlParseError::MissingResourceType)?
            .to_string();
        let path = path_parts[1..].iter().map(|s| s.to_string()).collect();
        
        Ok(Self {
            authority,
            resource_type,
            path,
            params,
            fragment,
        })
    }
    
    /// Build URL String
    pub fn to_string(&self) -> String {
        let authority = match &self.authority {
            RealmAuthority::Did(did) => did.to_string(),
            RealmAuthority::Alias(alias) => alias.clone(),
        };
        
        let path = if self.path.is_empty() {
            self.resource_type.clone()
        } else {
            format!("{}/{}", self.resource_type, self.path.join("/"))
        };
        
        let query = if self.params.is_empty() {
            String::new()
        } else {
            format!("?{}", self.params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&"))
        };
        
        let fragment = self.fragment.as_ref()
            .map(|f| format!("#{}", f))
            .unwrap_or_default();
        
        format!("erynoa://{}/{}{}{}", authority, path, query, fragment)
    }
    
    fn parse_query(query: &str) -> HashMap<String, String> {
        query.split('&')
            .filter_map(|pair| {
                let (k, v) = pair.split_once('=')?;
                Some((k.to_string(), v.to_string()))
            })
            .collect()
    }
}
```

### 7.2 Resource Schema

```rust
/// Resource Schema für Realm (Κ27)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSchema {
    pub version: String,
    pub types: HashMap<String, ResourceTypeDef>,
    pub fallback: Option<FallbackPolicy>,
    pub inheritance: Option<SchemaInheritance>,
}

/// Resource Type Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTypeDef {
    /// Path Pattern (z.B. "asset/<category>/<id>")
    pub path_pattern: String,
    /// Resolver (storage, identity, eclvm, etc.)
    pub resolver: String,
    /// Store-Name (falls storage resolver)
    pub store: Option<String>,
    /// Personal-Flag (personal store vs shared)
    pub personal: bool,
    /// Access-Policy
    pub access: AccessPolicyRef,
    /// Available Fields
    pub fields: FieldsDef,
}

/// Fields Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldsDef {
    Static(Vec<String>),
    Dynamic,  // "dynamic" - basierend auf Store-Schema
    All,      // "*" - alle Felder
}

/// Access Policy Reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessPolicyRef {
    /// Realm-weite Policy
    RealmPolicy,
    /// Member-only
    MembersOnly,
    /// Owner or Public
    OwnerOrPublic,
    /// Contract-spezifisch
    ContractPolicy,
    /// Policy-controlled via ECL
    PolicyControlled(String),
    /// Default Public
    Public,
}
```

### 7.3 Open-Access-Policy

```rust
/// Open-Access-Policy (Κ28)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAccessPolicy {
    pub name: String,
    /// Public Types und ihre Einschränkungen
    pub public_types: HashMap<String, PublicTypeAccess>,
    /// Anforderungen für Non-Members
    pub non_member_requirements: NonMemberRequirements,
    /// Mana-Cost Multiplikator für Non-Members
    pub non_member_mana_multiplier: f64,
}

/// Public Type Access Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicTypeAccess {
    /// Erlaubte Operationen (read, list)
    pub operations: Vec<String>,
    /// Erlaubte Felder
    pub fields: Vec<String>,
    /// Ausgeschlossene Felder
    pub exclude_fields: Vec<String>,
    /// Optionale Bedingung
    pub condition: Option<String>,
    /// Rate-Limiting
    pub rate_limit: Option<RateLimit>,
}

/// Non-Member Requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonMemberRequirements {
    /// Minimaler globaler Trust
    pub min_global_trust_omega: f64,
    /// Oder Mitglied in trusted Realm
    pub trusted_realms: Vec<String>,
}

/// Rate Limit Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub burst: u32,
}

impl OpenAccessPolicy {
    /// Prüfe ob Ressource für Non-Member zugänglich
    pub fn evaluate_non_member_access(
        &self,
        requester: &DID,
        requester_trust: &TrustVector6D,
        resource_type: &str,
        operation: &str,
        resource: &serde_json::Value,
    ) -> AccessResult {
        // Check trust requirements
        if requester_trust.omega < self.non_member_requirements.min_global_trust_omega {
            return AccessResult::Denied("Insufficient global trust".into());
        }
        
        // Check if type is public
        let type_access = match self.public_types.get(resource_type) {
            Some(ta) => ta,
            None => return AccessResult::Denied("Resource type not public".into()),
        };
        
        // Check operation
        if !type_access.operations.contains(&operation.to_string()) {
            return AccessResult::Denied("Operation not allowed".into());
        }
        
        // Evaluate condition if present
        if let Some(condition) = &type_access.condition {
            if !Self::evaluate_condition(condition, resource) {
                return AccessResult::Denied("Condition not met".into());
            }
        }
        
        // Build filtered fields
        let allowed_fields: Vec<String> = type_access.fields.iter()
            .filter(|f| !type_access.exclude_fields.contains(f))
            .cloned()
            .collect();
        
        AccessResult::Allowed {
            fields: allowed_fields,
            rate_limit: type_access.rate_limit.clone(),
            mana_multiplier: self.non_member_mana_multiplier,
        }
    }
    
    fn evaluate_condition(condition: &str, resource: &serde_json::Value) -> bool {
        // Simple condition evaluation (e.g., "resource.public == true")
        // Full implementation would use ECL evaluator
        if condition == "resource.public == true" {
            resource.get("public")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        } else {
            true
        }
    }
}

/// Access Result
#[derive(Debug)]
pub enum AccessResult {
    Allowed {
        fields: Vec<String>,
        rate_limit: Option<RateLimit>,
        mana_multiplier: f64,
    },
    Denied(String),
}
```

### 7.4 URL Resolver

```rust
/// URL Resolver Trait
#[async_trait::async_trait]
pub trait UrlResolver: Send + Sync {
    /// Resolve URL zu Ressource
    async fn resolve(
        &self,
        url: &ErnyoaUrl,
        requester: &DID,
        operation: &UrlOperation,
    ) -> Result<ResolvedResource, ResolveError>;
}

/// Resolution Context
pub struct ResolutionContext<'a> {
    pub realm_id: &'a UniversalId,
    pub schema: &'a ResourceSchema,
    pub requester: &'a DID,
    pub requester_membership: Option<&'a MembershipInfo>,
    pub requester_trust: &'a TrustVector6D,
}

/// Resolved Resource
#[derive(Debug, Serialize)]
pub struct ResolvedResource {
    pub url: String,
    pub realm_id: UniversalId,
    pub resource_type: String,
    pub path: Vec<String>,
    pub data: serde_json::Value,
    pub metadata: ResourceMetadata,
}

/// Resource Metadata
#[derive(Debug, Serialize)]
pub struct ResourceMetadata {
    pub version: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub owner: Option<DID>,
    pub access_level: String,
}

/// Main URL Resolver Implementation
pub struct ErnyoaUrlResolver {
    realm_registry: Arc<RealmRegistry>,
    schema_cache: Arc<SchemaCache>,
    storage_resolver: Arc<StorageResolver>,
    identity_resolver: Arc<IdentityResolver>,
    eclvm_resolver: Arc<EclvmResolver>,
    access_evaluator: Arc<AccessEvaluator>,
}

#[async_trait::async_trait]
impl UrlResolver for ErnyoaUrlResolver {
    async fn resolve(
        &self,
        url: &ErnyoaUrl,
        requester: &DID,
        operation: &UrlOperation,
    ) -> Result<ResolvedResource, ResolveError> {
        // 1. Resolve Realm Authority
        let realm_id = self.resolve_authority(&url.authority).await?;
        
        // 2. Load Schema
        let schema = self.schema_cache.get_or_load(&realm_id).await?;
        
        // 3. Get Type Definition
        let type_def = schema.types.get(&url.resource_type)
            .ok_or(ResolveError::UnknownResourceType)?;
        
        // 4. Get Requester Context
        let membership = self.realm_registry
            .get_membership(&realm_id, requester).await.ok();
        let trust = self.get_requester_trust(requester, &realm_id).await?;
        
        let ctx = ResolutionContext {
            realm_id: &realm_id,
            schema: &schema,
            requester,
            requester_membership: membership.as_ref(),
            requester_trust: &trust,
        };
        
        // 5. Evaluate Access
        let access = self.access_evaluator.evaluate(&ctx, type_def, operation).await?;
        if let AccessResult::Denied(reason) = access {
            return Err(ResolveError::AccessDenied(reason));
        }
        
        // 6. Resolve via appropriate resolver
        let data = match type_def.resolver.as_str() {
            "storage" => self.storage_resolver.resolve(&ctx, &url.path, type_def).await?,
            "identity" => self.identity_resolver.resolve(&ctx, &url.path).await?,
            "eclvm" => self.eclvm_resolver.resolve(&ctx, &url.path, operation).await?,
            _ => return Err(ResolveError::UnknownResolver),
        };
        
        // 7. Filter fields based on access
        let filtered_data = self.filter_fields(data, &access)?;
        
        Ok(ResolvedResource {
            url: url.to_string(),
            realm_id,
            resource_type: url.resource_type.clone(),
            path: url.path.clone(),
            data: filtered_data,
            metadata: ResourceMetadata::default(),
        })
    }
}
```

---

## 8. CLI-Integration

```bash
# URL-Auflösung via CLI
$ erynoa url resolve erynoa://gaming-dao/asset/weapons/sword-001

# Output:
{
  "url": "erynoa://gaming-dao/asset/weapons/sword-001",
  "realm": "did:erynoa:circle:abc123...",
  "data": {
    "id": "sword-001",
    "name": "Flameblade",
    "rarity": "legendary",
    "damage": 450
  }
}

# URL mit Parametern
$ erynoa url resolve "erynoa://gaming-dao/store/inventory/all?filter=rarity:legendary&limit=5"

# URL schreiben (benötigt Signatur)
$ erynoa url write erynoa://gaming-dao/store/settings/theme --data '{"dark_mode": true}'

# URL subscriben (Events)
$ erynoa url subscribe erynoa://gaming-dao/event/ItemSold

# Schema anzeigen
$ erynoa url schema gaming-dao
# Shows: ResourceSchema with all types, patterns, and access policies

# Open-Access-Policy anzeigen
$ erynoa url access-policy gaming-dao
# Shows: OpenAccessPolicy configuration
```

---

## 9. Sicherheits-Überlegungen

### 9.1 URL-Injection Prevention

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   SECURITY CONSIDERATIONS                                                    ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   1. Path-Traversal Prevention                                               ║
║      - ".." in Pfaden verboten                                               ║
║      - Pfade werden normalisiert                                             ║
║      - Nur alphanumerisch + "-_." erlaubt                                   ║
║                                                                              ║
║   2. Authority-Spoofing                                                      ║
║      - DID-Authorites werden kryptographisch verifiziert                    ║
║      - Alias-Resolution nur via Root-Realm Registry                          ║
║      - Alias-Änderung erfordert Governance-Proposal                         ║
║                                                                              ║
║   3. Access-Control-Bypass                                                   ║
║      - Jede Resolution durchläuft Access-Evaluator                          ║
║      - Caching berücksichtigt Requester-Context                             ║
║      - Rate-Limiting für Non-Members                                        ║
║                                                                              ║
║   4. Information Leakage                                                     ║
║      - Fehler-Responses verraten keine internen Details                     ║
║      - "Not Found" vs "Access Denied" → uniform "Not Found"                 ║
║      - Field-Filtering vor Response                                          ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 10. Zusammenfassung

### 10.1 Neue Konstanten

| Konstante | Name | Beschreibung |
|-----------|------|--------------|
| **Κ26** | URL-Schema | Erynoa URL Format: `erynoa://<realm>/<type>/<path>` |
| **Κ27** | Resource-Resolution | Schema-basierte Auflösung von Ressourcen im Realm-Kontext |
| **Κ28** | Open-Access-Policy | Policy-gesteuerte öffentliche Ressourcen für Non-Members |

### 10.2 Integration mit bestehenden Konstanten

| Konstante | Integration |
|-----------|-------------|
| Κ1 | Schema-Vererbung folgt Regel-Vererbung |
| Κ17/Κ18 | Membership-Status beeinflusst Access |
| Κ23 | Cross-Realm URL-Resolution mit Crossing-Dampening |
| Κ24 | Lokaler Trust bleibt unabhängig bei URL-Access |

### 10.3 Erynoa-URL DNA

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   ERYNOA URL DNA                                                             ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   U = ⟨ Κ26, Κ27, Κ28 ⟩                                                      ║
║                                                                              ║
║   Κ26: url(realm, type, path) → erynoa://{realm}/{type}/{path}              ║
║   Κ27: resolve(url, ctx) → resource iff schema(realm).match(type, path)     ║
║   Κ28: access(url, requester) = policy(realm).evaluate(type, requester)     ║
║                                                                              ║
║   Invarianten:                                                               ║
║   ─────────────────────────────────────────────────────────────────────────  ║
║   • ∀ url: resolve(url) terminiert ⟹ O(1) Realm-Lookup + O(log n) Path     ║
║   • ∀ realm: schema(realm) ⊇ schema(parent(realm))                          ║
║   • ∀ requester: access(url, requester) ∈ {Allow(fields), Deny}             ║
║   • ∀ non_member: access(url, non_member) → Κ28 policy evaluation           ║
║   • ∀ cross_realm: resolve(url) → Κ23 dampening applied                      ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## Anhang: Vollständige URL-Beispiele

```text
# Basic Resource Access
erynoa://gaming-dao/store/inventory/items

# Profile mit Public-View
erynoa://social-hub/profile/~alice?view=public

# Contract State
erynoa://defi-realm/contract/staking/state

# Governance Proposal
erynoa://community-dao/governance/proposal/prop-2024-001

# Trust-Query
erynoa://gaming-dao/trust/did:erynoa:self:user123

# Asset mit Filter
erynoa://nft-realm/asset/art?filter=creator:alice&sort=price:desc

# Package Reference
erynoa://packages/pkg/common-lib@1.0.0/src/utils.ecl

# Event Subscription (via WebSocket)
erynoa://gaming-dao/event/ItemTrade?since=2024-01-01

# Cross-Realm Link
erynoa://gaming-dao/link/erynoa://marketplace/asset/item-001

# Realm Metadata
erynoa://gaming-dao/meta/info
```
