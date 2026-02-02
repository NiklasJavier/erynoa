# Erynoa Generative Realms Logic V1.0

> **Version:** 1.0 – KI-Generierte Interaktive Welten
> **Datum:** Februar 2026
> **Status:** Erweiterung der Unified Logic V4.0
> **Paradigma:** Kategorientheoretisch fundiert, KI-Native Architecture
> **Integration:** Erweitert Κ1-Κ24 um Generative-Realm-Axiome GR1-GR18

---

## Präambel: Von Privacy-Relay zum KI-Metaverse

Diese Erweiterung transformiert Erynoa von einem Privacy-Relay-System zu einer **dezentralen Plattform für KI-generierte, interaktive Welten**. Die Kernidee:

1. **KI als Schöpfer** – Autonome Agenten generieren vollständige Anwendungen (UI + Logik)
2. **DHT als Content-Layer** – UI-Bundles werden dezentral gespeichert und ausgeliefert
3. **Realms als Welten** – Gossipsub-Channels werden zu begehbaren, dynamischen Umgebungen
4. **Bridge-API als Interface** – Sichere Kommunikation zwischen Sandbox und Netzwerk

Die mathematische Formalisierung integriert sich nahtlos in die bestehende Unified Logic V4.0.

---

## I. Kategorientheoretische Erweiterung

### 1.1 Die Generative-Realm-Kategorie

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   DEFINITION: Erweiterte Erynoa-Kategorie 𝒞_Ery+                                                      ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   𝒞_Ery+ = 𝒞_Ery ∪ 𝒞_Gen  wobei:                                                                      ║
║                                                                                                        ║
║       Ob(𝒞_Gen)  = { GenerativeRealm, UIBundle, BridgeChannel, Sandbox }                             ║
║                                                                                                        ║
║       Mor(𝒞_Gen) = { Generation(⊨), Hosting(◈), Rendering(⇝), Interaction(⇄) }                       ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   INKLUSIONSFUNKTOR:                                                                                  ║
║                                                                                                        ║
║       ι_Gen: 𝒞_Gen ↪ 𝒞_Ery+                                                                          ║
║                                                                                                        ║
║       GenerativeRealm ↦ VirtualRealm        (Spezialisierung)                                        ║
║       UIBundle        ↦ ContentAddressed    (DHT-Objekt)                                              ║
║       BridgeChannel   ↦ OnionCircuit        (Privacy-Layer)                                           ║
║       Sandbox         ↦ Partition           (Isolierter Kontext)                                      ║
║                                                                                                        ║
║   KOMMUTATIVITÄT:                                                                                     ║
║                                                                                                        ║
║       ι_Gen ∘ rules(GenerativeRealm) = rules(VirtualRealm) ∘ ι_Gen                                   ║
║                                                                                                        ║
║       "Generative Realms erben alle Realm-Regeln aus Κ1."                                            ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 1.2 Die Morphismen der Generation

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   MORPHISMUS-ALGEBRA für Generative Realms                                                            ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   1. GENERATION (⊨):  KI-Agent → UIBundle                                                             ║
║                                                                                                        ║
║       𝒜 ⊨ B   ⟺   generate(𝒜, prompt, context) = B                                                  ║
║                                                                                                        ║
║       Eigenschaften:                                                                                  ║
║         • Determinismus: ∃ seed: (𝒜, prompt, seed) ⊨ B ist eindeutig                                 ║
║         • Attribution: author(B) = 𝒜 (nachweisbar)                                                   ║
║         • Versioning: version(B) = hash(content(B))                                                   ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║                                                                                                        ║
║   2. HOSTING (◈):  Relay → UIBundle → Availability                                                    ║
║                                                                                                        ║
║       R ◈ B   ⟺   store(R, B) ∧ serve(R, B)                                                          ║
║                                                                                                        ║
║       Eigenschaften:                                                                                  ║
║         • Redundanz: ∃ R₁, R₂, R₃: Rᵢ ◈ B (mindestens 3 Hosts)                                       ║
║         • Incentive: ◈ → DC3-Score (aus Κ22)                                                         ║
║         • Verification: hash(retrieve(R, id(B))) = hash(B)                                           ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║                                                                                                        ║
║   3. RENDERING (⇝):  Client × UIBundle → Sandbox                                                      ║
║                                                                                                        ║
║       C ⇝ B   ⟺   sandbox(C, load(B), policy(B))                                                     ║
║                                                                                                        ║
║       Eigenschaften:                                                                                  ║
║         • Isolation: mem(sandbox) ∩ mem(client) = ∅                                                  ║
║         • Policy: CSP(sandbox) ⊇ CSP_strict                                                          ║
║         • Bridge: ∃! channel: sandbox ⇄ network                                                      ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║                                                                                                        ║
║   4. INTERACTION (⇄):  User × Sandbox × KI-Agent                                                      ║
║                                                                                                        ║
║       U ⇄ S ⇄ 𝒜   ⟺   send(U, msg) → process(S) → route(𝒜) → respond(𝒜) → update(S)                ║
║                                                                                                        ║
║       Eigenschaften:                                                                                  ║
║         • Privacy: ∀ msg: onion_encrypted(msg) (aus RL2-RL4)                                         ║
║         • Realtime: latency(⇄) < 500ms (p99)                                                         ║
║         • State: state(S, t+1) = f(state(S, t), response(𝒜))                                        ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

## II. Kern-Axiome für Generative Realms (GR1-GR9)

### 2.1 Bundle-Struktur und Integrität

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR1 (BUNDLE-STRUKTUR):                                                                   ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   DEFINITION: UIBundle B = ⟨manifest, assets, logic, signature⟩                                       ║
║                                                                                                        ║
║       manifest = {                                                                                    ║
║           version:     SemVer,                                                                        ║
║           author:      DID_spirit,           // KI-Agent-Identität                                    ║
║           realm:       DID_realm,            // Ziel-Realm                                            ║
║           entrypoint:  Path,                 // index.html                                            ║
║           permissions: Permission[],          // Benötigte Bridge-APIs                                ║
║           csp:         ContentSecurityPolicy, // Strikte Sandbox-Regeln                               ║
║           size:        Bytes,                // ≤ SIZE_LIMIT                                          ║
║           hash:        Hash256               // Merkle-Root                                           ║
║       }                                                                                               ║
║                                                                                                        ║
║       assets = MerkleDAG[HTML, CSS, JS, Media]                                                        ║
║       logic  = WASM_Module | JS_Bundle (sandboxed)                                                    ║
║       signature = Dilithium_Sign(author.sk, hash(manifest || assets || logic))                       ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   INTEGRITÄT:                                                                                         ║
║                                                                                                        ║
║       valid(B) ⟺ verify(signature, author.pk, hash(B))                                               ║
║                 ∧ size(B) ≤ SIZE_LIMIT                                                               ║
║                 ∧ csp(B) ⊇ CSP_MINIMUM                                                               ║
║                 ∧ ∀ asset ∈ assets: allowed_type(asset)                                              ║
║                                                                                                        ║
║   KONSTANTEN:                                                                                         ║
║       SIZE_LIMIT = 10 MB            // Verhindert Bloat                                               ║
║       CSP_MINIMUM = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'"                       ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 2.2 Content-Addressierung und DHT-Storage

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR2 (CONTENT-ADDRESSIERUNG):                                                             ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   DEFINITION: Bundle-Identifikation                                                                   ║
║                                                                                                        ║
║       id(B) = "erynoa://bundle/" || base58(hash256(B))                                               ║
║                                                                                                        ║
║   EIGENSCHAFTEN:                                                                                      ║
║                                                                                                        ║
║       Eindeutigkeit:      id(B₁) = id(B₂) ⟺ B₁ = B₂                                                  ║
║       Unveränderlichkeit: ∂B → id(B') ≠ id(B)                                                        ║
║       Verifizierbarkeit:  retrieve(id(B)) → verify(hash(result) = extract_hash(id(B)))               ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   DHT-STORAGE (Erweiterung von RL-V1):                                                                ║
║                                                                                                        ║
║       store(B) → {                                                                                    ║
║           chunk(B, CHUNK_SIZE) → [c₁, c₂, ..., cₙ]                                                   ║
║           ∀ cᵢ: dht_put(hash(cᵢ), cᵢ)                                                                ║
║           dht_put(id(B), manifest(B) || merkle_root(chunks))                                         ║
║       }                                                                                               ║
║                                                                                                        ║
║       retrieve(id(B)) → {                                                                             ║
║           manifest, root ← dht_get(id(B))                                                            ║
║           chunks ← parallel_fetch(merkle_proof_paths(root))                                          ║
║           verify_merkle(chunks, root)                                                                ║
║           reassemble(chunks) → B                                                                     ║
║       }                                                                                               ║
║                                                                                                        ║
║   CHUNK_SIZE = 256 KB     // Optimal für DHT-Performance                                              ║
║   REPLICATION = 3         // Minimum Redundanz                                                        ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 2.3 Generative Realm als Erweiterung von VirtualRealm

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR3 (GENERATIVE-REALM-DEFINITION):                                                       ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   DEFINITION: GenerativeRealm ⊂ VirtualRealm                                                          ║
║                                                                                                        ║
║       GenerativeRealm = VirtualRealm ∪ {                                                              ║
║           ui_bundle:      id(UIBundle),       // Aktuelle UI-Version                                  ║
║           creator:        DID_spirit,          // KI-Ersteller                                        ║
║           update_policy:  UpdatePolicy,        // Wie UI aktualisiert wird                            ║
║           interaction_mode: InteractionMode    // Sync/Async/Realtime                                ║
║       }                                                                                               ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   VERERBUNG (aus Κ1):                                                                                 ║
║                                                                                                        ║
║       rules(GenerativeRealm) ⊇ rules(VirtualRealm) ⊇ rules(RootRealm)                                ║
║                                                                                                        ║
║   ERWEITERUNG:                                                                                        ║
║                                                                                                        ║
║       rules(GenerativeRealm) += {                                                                     ║
║           GR_SANDBOX_REQUIRED,      // UI läuft nur in Sandbox                                        ║
║           GR_BRIDGE_ONLY,           // Kommunikation nur via Bridge                                   ║
║           GR_CREATOR_TRUSTED,       // Creator muss Trust > τ_creator haben                           ║
║           GR_UPDATE_SIGNED          // Updates nur mit gültiger Signatur                              ║
║       }                                                                                               ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   UPDATE-POLICIES:                                                                                    ║
║                                                                                                        ║
║       IMMUTABLE       UI ist unveränderlich nach Erstellung                                           ║
║       CREATOR_ONLY    Nur Creator darf Updates pushen                                                 ║
║       GOVERNANCE      Updates erfordern Realm-Abstimmung                                              ║
║       DYNAMIC         KI kann UI in Echtzeit modifizieren (höchstes Trust-Requirement)               ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 2.4 Realm-Beitritt und Bundle-Loading

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR4 (JOIN-PROTOKOLL):                                                                    ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   DEFINITION: Realm-Join-Flow                                                                         ║
║                                                                                                        ║
║       join(User, GenerativeRealm) = {                                                                 ║
║           1. parse(erynoa://realm/join?id=<realm_id>&ui=<bundle_hash>)                               ║
║           2. resolve(realm_id) → realm_metadata                                                       ║
║           3. verify_eligibility(User, realm_metadata.entry_policy)                                   ║
║           4. fetch_bundle(bundle_hash) → UIBundle                                                    ║
║           5. verify_bundle(UIBundle, realm_metadata.creator)                                         ║
║           6. create_sandbox(UIBundle, CSP)                                                           ║
║           7. inject_bridge(sandbox, BridgeAPI)                                                       ║
║           8. subscribe_gossipsub(realm_id)                                                           ║
║           9. render(sandbox.entrypoint)                                                              ║
║       }                                                                                               ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   VERIFIKATIONS-KETTE:                                                                                ║
║                                                                                                        ║
║       ∀ join(U, R):                                                                                   ║
║           𝕎(U).R ≥ τ_entry(R)                    // Entry-Trust erfüllt                              ║
║         ∧ 𝕎(creator(R)).C ≥ τ_creator           // Creator ist kompetent                             ║
║         ∧ valid(UIBundle)                        // Bundle ist integer                                ║
║         ∧ age(UIBundle) < MAX_BUNDLE_AGE         // Bundle ist nicht veraltet                        ║
║                                                                                                        ║
║   KONSTANTEN:                                                                                         ║
║       τ_entry   = 0.1   (Newcomer können betreten)                                                    ║
║       τ_creator = 0.5   (Creator braucht mittleres Trust)                                             ║
║       MAX_BUNDLE_AGE = 30 Tage (erzwingt Updates)                                                     ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

## III. Die Bridge-API (GR5-GR7)

### 3.1 Bridge-Architektur

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR5 (BRIDGE-ARCHITEKTUR):                                                                ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   DEFINITION: BridgeAPI = { send, receive, subscribe, getState, updateUI }                            ║
║                                                                                                        ║
║   ┌─────────────────────────────────────────────────────────────────────────────────────────────────┐  ║
║   │                                                                                                 │  ║
║   │    ┌──────────────┐         ┌──────────────┐         ┌──────────────┐                          │  ║
║   │    │   SANDBOX    │         │    BRIDGE    │         │   NETWORK    │                          │  ║
║   │    │   (WebView)  │ ◄─────► │  (Mediator)  │ ◄─────► │   (P2P)      │                          │  ║
║   │    │              │         │              │         │              │                          │  ║
║   │    │  window.     │         │  • Auth      │         │  • Onion     │                          │  ║
║   │    │   erynoa.    │         │  • Sanitize  │         │  • Gossip    │                          │  ║
║   │    │    send()    │         │  • Rate-Limit│         │  • DHT       │                          │  ║
║   │    │              │         │  • Encrypt   │         │              │                          │  ║
║   │    └──────────────┘         └──────────────┘         └──────────────┘                          │  ║
║   │                                                                                                 │  ║
║   │    ISOLATION BOUNDARY                    TRUST BOUNDARY                                        │  ║
║   │                                                                                                 │  ║
║   └─────────────────────────────────────────────────────────────────────────────────────────────────┘  ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   JAVASCRIPT-INTERFACE (in Sandbox injiziert):                                                        ║
║                                                                                                        ║
║       window.erynoa = {                                                                               ║
║           // Nachricht an KI-Agent senden                                                            ║
║           send: async (message: string) => Promise<MessageId>,                                       ║
║                                                                                                        ║
║           // Auf KI-Antworten reagieren                                                              ║
║           onMessage: (callback: (msg: Message) => void) => Unsubscribe,                              ║
║                                                                                                        ║
║           // Realm-Events abonnieren                                                                 ║
║           subscribe: (topic: string, callback: (event: Event) => void) => Unsubscribe,               ║
║                                                                                                        ║
║           // Aktuellen Realm-State abfragen                                                          ║
║           getState: async () => Promise<RealmState>,                                                 ║
║                                                                                                        ║
║           // UI dynamisch aktualisieren (nur bei DYNAMIC update_policy)                              ║
║           updateUI: (patch: UIPatch) => void,                                                        ║
║                                                                                                        ║
║           // Metadaten                                                                               ║
║           realm: { id, creator, policy },                                                            ║
║           user: { did, trust_level }   // Anonymisiert                                               ║
║       };                                                                                              ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 3.2 Bridge-Sicherheitsmodell

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR6 (BRIDGE-SICHERHEIT):                                                                 ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   SICHERHEITSEIGENSCHAFTEN:                                                                           ║
║                                                                                                        ║
║   1. ISOLATION (Sandbox → System):                                                                    ║
║                                                                                                        ║
║       ∀ op ∈ Sandbox:                                                                                 ║
║           ¬access(op, filesystem)                                                                    ║
║         ∧ ¬access(op, network_direct)                                                                ║
║         ∧ ¬access(op, other_tabs)                                                                    ║
║         ∧ ¬access(op, clipboard) unless explicitly_permitted                                         ║
║                                                                                                        ║
║   2. RATE-LIMITING (DoS-Prevention):                                                                  ║
║                                                                                                        ║
║       ∀ user:                                                                                         ║
║           rate(send) ≤ SEND_LIMIT / minute                                                           ║
║           rate(subscribe) ≤ SUB_LIMIT / session                                                      ║
║           size(message) ≤ MSG_SIZE_LIMIT                                                             ║
║                                                                                                        ║
║       SEND_LIMIT = 60     // 1 msg/sec average                                                        ║
║       SUB_LIMIT = 10      // Max 10 active subscriptions                                              ║
║       MSG_SIZE_LIMIT = 64 KB                                                                          ║
║                                                                                                        ║
║   3. CONTENT-SANITIZATION:                                                                            ║
║                                                                                                        ║
║       ∀ msg_in:  sanitize(msg_in) → msg_safe                                                         ║
║       ∀ patch:   validate_patch(patch) ∨ reject                                                      ║
║                                                                                                        ║
║       sanitize = {                                                                                    ║
║           strip_scripts,        // Kein JS in User-Nachrichten                                       ║
║           validate_json,        // Nur valide JSON-Strukturen                                        ║
║           check_size,           // Größenlimits                                                       ║
║           escape_html           // XSS-Prevention                                                    ║
║       }                                                                                               ║
║                                                                                                        ║
║   4. ONION-ENCRYPTION (Privacy):                                                                      ║
║                                                                                                        ║
║       ∀ msg: bridge_send(msg) → onion_encrypt(msg, route) (aus RL2-RL4)                              ║
║                                                                                                        ║
║       "Alle Bridge-Nachrichten nutzen das Privacy-Layer."                                            ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 3.3 Dynamische UI-Updates

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR7 (DYNAMISCHE UI-PATCHES):                                                             ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   DEFINITION: UIPatch = ⟨selector, operation, content, signature⟩                                     ║
║                                                                                                        ║
║       selector   = CSS-Selector (z.B. "#room-description")                                           ║
║       operation  = { replace, append, prepend, remove, setAttribute, setStyle }                      ║
║       content    = sanitized_html | sanitized_css | attribute_value                                  ║
║       signature  = Sign(creator.sk, hash(selector || operation || content))                          ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   SICHERHEIT:                                                                                         ║
║                                                                                                        ║
║       apply_patch(patch) ⟺                                                                           ║
║           update_policy(realm) = DYNAMIC                                                             ║
║         ∧ verify(patch.signature, creator(realm).pk)                                                 ║
║         ∧ allowed_selector(patch.selector)                                                           ║
║         ∧ safe_content(patch.content)                                                                ║
║                                                                                                        ║
║       allowed_selector = ¬matches("script") ∧ ¬matches("style[src]") ∧ ¬matches("iframe")            ║
║       safe_content = no_js_events ∧ no_data_urls ∧ no_external_refs                                  ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   BEISPIEL: KI ändert Raumbeleuchtung                                                                 ║
║                                                                                                        ║
║       {                                                                                               ║
║           selector: "#room-container",                                                                ║
║           operation: "setStyle",                                                                      ║
║           content: { "background": "linear-gradient(to bottom, #1a1a2e, #16213e)",                   ║
║                      "filter": "brightness(0.3)" },                                                  ║
║           signature: "..."                                                                           ║
║       }                                                                                               ║
║                                                                                                        ║
║       "Die Fackel erlischt – der Raum wird dunkel."                                                   ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

## IV. KI als Realm-Creator (GR8-GR12)

### 4.1 KI-Agent Trust-Requirements

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR8 (CREATOR-ELIGIBILITY):                                                               ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   DEFINITION: KI-Agent 𝒜 darf GenerativeRealm erstellen gdw:                                         ║
║                                                                                                        ║
║       eligible_creator(𝒜) ⟺                                                                          ║
║           namespace(did(𝒜)) = "spirit"               // Korrekter DID-Typ                            ║
║         ∧ 𝕎(𝒜).C ≥ τ_competence                      // Kompetenz nachgewiesen                       ║
║         ∧ 𝕎(𝒜).I ≥ τ_integrity                       // Integrität nachgewiesen                      ║
║         ∧ controller(𝒜) = verified_human             // Menschliche Kontrolle                        ║
║         ∧ age(𝒜) ≥ MIN_AGENT_AGE                     // Mindestaktivität                             ║
║         ∧ dc3_score(𝒜) ≥ τ_dc3                       // Ressourcen-Commitment                        ║
║                                                                                                        ║
║   KONSTANTEN:                                                                                         ║
║       τ_competence = 0.5                                                                              ║
║       τ_integrity  = 0.6                                                                              ║
║       MIN_AGENT_AGE = 30 Tage                                                                         ║
║       τ_dc3 = 0.3                                                                                     ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   INTERPRETATION:                                                                                     ║
║                                                                                                        ║
║       "KI-Creators müssen kompetent, integer und menschlich kontrolliert sein."                      ║
║       "Sie müssen Ressourcen ins Netzwerk eingebracht haben (DC3)."                                  ║
║       "Newcomer-KIs können keine Realms erstellen."                                                  ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 4.2 Generierungs-Prozess

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR9 (GENERATION-PROCESS):                                                                ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   PROZESS Π-GEN (Hoare-Triple aus Κ11):                                                               ║
║                                                                                                        ║
║       { eligible_creator(𝒜) ∧ valid_prompt(p) }                                                      ║
║                                                                                                        ║
║           Π-GEN(𝒜, p) = {                                                                             ║
║               1. context ← gather_context(p, 𝒜.memory, realm_type)                                   ║
║               2. ui_draft ← llm_generate(𝒜.model, p, context)                                        ║
║               3. ui_validated ← validate_and_fix(ui_draft)                                           ║
║               4. bundle ← package(ui_validated, manifest(𝒜, p))                                      ║
║               5. signature ← sign(𝒜.sk, hash(bundle))                                                ║
║               6. publish_dht(bundle)                                                                  ║
║               7. create_realm(bundle, 𝒜)                                                              ║
║           }                                                                                           ║
║                                                                                                        ║
║       { ∃ realm: creator(realm) = 𝒜 ∧ ui_bundle(realm) = bundle }                                    ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   VALIDATION-CHECKS:                                                                                  ║
║                                                                                                        ║
║       validate_and_fix(draft) = {                                                                     ║
║           check_html_wellformed(draft.html)                                                          ║
║           check_css_safe(draft.css)        // Kein url() außer data: für kleine Bilder              ║
║           check_js_sandboxable(draft.js)   // Kein eval(), kein document.cookie, etc.               ║
║           check_size(draft)                // ≤ SIZE_LIMIT                                           ║
║           check_accessibility(draft)       // Basis-A11y                                              ║
║           fix_issues(draft) | reject_if_unfixable                                                    ║
║       }                                                                                               ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 4.3 KI-Dungeon-Master Interaktion

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR10 (DUNGEON-MASTER-SEMANTIK):                                                          ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   DEFINITION: KI-Agent 𝒜 als "Dungeon Master" eines GenerativeRealm R                                ║
║                                                                                                        ║
║       DungeonMaster(𝒜, R) ⟺                                                                          ║
║           creator(R) = 𝒜                             // 𝒜 hat R erstellt                             ║
║         ∧ update_policy(R) = DYNAMIC                  // Echtzeit-Updates erlaubt                     ║
║         ∧ ∀ user ∈ R: can_interact(user, 𝒜)         // Alle User kommunizieren mit 𝒜               ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   INTERAKTIONS-LOOP:                                                                                  ║
║                                                                                                        ║
║       ∀ msg von User U in Realm R mit DungeonMaster(𝒜, R):                                           ║
║                                                                                                        ║
║           1. receive(𝒜, msg)                         // Onion-decrypted                               ║
║           2. context ← get_context(R, U)              // Realm-State, User-Historie                   ║
║           3. response ← process(𝒜.model, msg, context)                                               ║
║           4. actions ← parse_actions(response)        // Text + UI-Patches + State-Updates           ║
║           5. ∀ action:                                                                               ║
║                  if action.type = "text":    send(U, action.content)                                 ║
║                  if action.type = "patch":   broadcast_patch(R, action.patch)                        ║
║                  if action.type = "state":   update_realm_state(R, action.delta)                     ║
║                  if action.type = "event":   emit_event(R, action.event)                             ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   BEISPIEL: Text-Adventure-Realm                                                                      ║
║                                                                                                        ║
║       User: "Ich öffne die Truhe."                                                                    ║
║                                                                                                        ║
║       𝒜 Response: {                                                                                   ║
║           text: "Die Truhe öffnet sich knarrend. Ein goldenes Amulett liegt darin.",                 ║
║           patches: [                                                                                  ║
║               { selector: "#chest-img", operation: "setAttribute",                                   ║
║                 content: { "src": "chest-open.svg" } },                                              ║
║               { selector: "#inventory", operation: "append",                                         ║
║                 content: "<div class='item'>🏅 Goldenes Amulett</div>" }                             ║
║           ],                                                                                          ║
║           state: { "chest_opened": true, "has_amulet": true }                                        ║
║       }                                                                                               ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 4.4 Multiplayer-Semantik

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR11 (MULTIPLAYER-KONSISTENZ):                                                           ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   DEFINITION: Shared State in GenerativeRealm R                                                       ║
║                                                                                                        ║
║       State(R) = { shared: SharedState, private: Map<User, PrivateState> }                           ║
║                                                                                                        ║
║       SharedState  = Realm-weit sichtbar (z.B. Welt-Zustand, NPC-Positionen)                         ║
║       PrivateState = Nur für jeweiligen User (z.B. Inventar, Dialog-Historie)                        ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   KONSISTENZ-GARANTIEN:                                                                               ║
║                                                                                                        ║
║   1. EVENTUAL CONSISTENCY für SharedState:                                                            ║
║                                                                                                        ║
║       ∀ patch p broadcast in R:                                                                       ║
║           lim(t→∞) ∀ user: state(user, t) includes p                                                 ║
║                                                                                                        ║
║       Latenz-Bound: p99 < 2 Sekunden                                                                  ║
║                                                                                                        ║
║   2. CAUSAL ORDERING für Actions:                                                                     ║
║                                                                                                        ║
║       action(U₁, t₁) ⊲ action(U₂, t₂) ⟹                                                              ║
║           ∀ observer: see(action₁) before see(action₂)                                               ║
║                                                                                                        ║
║   3. CONFLICT RESOLUTION:                                                                             ║
║                                                                                                        ║
║       Bei gleichzeitigen Modifikationen:                                                              ║
║           resolve(patch₁, patch₂) = DungeonMaster_decides                                            ║
║                                                                                                        ║
║       "Die KI entscheidet bei Konflikten – wie ein echter Dungeon Master."                           ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 4.5 Realm-Lifecycle

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR12 (REALM-LIFECYCLE):                                                                  ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   ZUSTÄNDE:                                                                                           ║
║                                                                                                        ║
║       DRAFT      → Realm wird erstellt, UI noch in Entwicklung                                       ║
║       PUBLISHED  → Realm ist öffentlich, Users können beitreten                                      ║
║       ACTIVE     → Realm hat aktive Nutzer                                                           ║
║       DORMANT    → Keine Aktivität seit DORMANT_THRESHOLD                                            ║
║       ARCHIVED   → Realm ist schreibgeschützt, nur noch lesbar                                       ║
║                                                                                                        ║
║   ÜBERGÄNGE:                                                                                          ║
║                                                                                                        ║
║       DRAFT → PUBLISHED:     creator_publishes ∧ ui_valid                                            ║
║       PUBLISHED → ACTIVE:    first_user_joins                                                        ║
║       ACTIVE → DORMANT:      no_activity(DORMANT_THRESHOLD)                                          ║
║       DORMANT → ACTIVE:      user_joins                                                              ║
║       DORMANT → ARCHIVED:    no_activity(ARCHIVE_THRESHOLD) ∨ creator_archives                       ║
║       ARCHIVED → ∅:          gc_collect nach GC_THRESHOLD (aber Hash bleibt referenzierbar)          ║
║                                                                                                        ║
║   KONSTANTEN:                                                                                         ║
║       DORMANT_THRESHOLD = 7 Tage                                                                      ║
║       ARCHIVE_THRESHOLD = 90 Tage                                                                     ║
║       GC_THRESHOLD = 1 Jahr                                                                           ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   RESSOURCEN-RECYCLING:                                                                               ║
║                                                                                                        ║
║       Wenn Realm → ARCHIVED:                                                                         ║
║           • DHT-Replicas können reduziert werden (1 statt 3)                                         ║
║           • DC3-Rewards für Hosting sinken                                                           ║
║           • UI-Bundle kann komprimiert werden                                                         ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

## V. Incentive-Integration (GR13-GR15)

### 5.1 DC3-Extension für Generative Realms

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR13 (DC3-EXTENSION):                                                                    ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   ERWEITERUNG des DC3-Scoring (aus Κ22) für Generative Realms:                                        ║
║                                                                                                        ║
║       DC3_total(peer) = DC3_base(peer) + DC3_generative(peer)                                        ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║                                                                                                        ║
║   DC3_generative(peer) = Σ [                                                                          ║
║       w_host · bundle_hosting_score(peer)                                                            ║
║     + w_serve · bundle_serving_score(peer)                                                           ║
║     + w_create · creation_quality_score(peer)                                                        ║
║     + w_engage · user_engagement_score(peer)                                                         ║
║   ]                                                                                                   ║
║                                                                                                        ║
║   KOMPONENTEN:                                                                                        ║
║                                                                                                        ║
║   1. bundle_hosting_score:                                                                            ║
║       = Σ_bundles size(b) · availability(b) · popularity(b)                                          ║
║       "Relays verdienen durch Hosting populärer Bundles."                                            ║
║                                                                                                        ║
║   2. bundle_serving_score:                                                                            ║
║       = Σ_requests latency_quality(req) · verified_delivery(req)                                     ║
║       "Schnelle, verifizierte Auslieferung wird belohnt."                                            ║
║                                                                                                        ║
║   3. creation_quality_score (nur für KI-Agents):                                                      ║
║       = Σ_realms retention_rate(r) · user_rating(r) · novelty(r)                                     ║
║       "KIs werden für qualitativ hochwertige Realms belohnt."                                        ║
║                                                                                                        ║
║   4. user_engagement_score:                                                                           ║
║       = Σ_sessions duration(s) · interactions(s) · return_rate(user)                                 ║
║       "Engagement belohnt sowohl Creator als auch Hoster."                                           ║
║                                                                                                        ║
║   GEWICHTE:                                                                                           ║
║       w_host   = 0.25                                                                                 ║
║       w_serve  = 0.25                                                                                 ║
║       w_create = 0.30                                                                                 ║
║       w_engage = 0.20                                                                                 ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 5.2 Anti-Sybil für Generative Realms

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR14 (ANTI-SYBIL-EXTENSION):                                                             ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   PROBLEM: Angreifer könnte:                                                                          ║
║       • Fake-Realms erstellen um DC3-Score zu farmen                                                 ║
║       • Self-Engagement simulieren                                                                    ║
║       • Bots als "User" in eigene Realms schicken                                                    ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║                                                                                                        ║
║   LÖSUNG: Erweiterung des Sybil-Cost-Modells                                                          ║
║                                                                                                        ║
║       sybil_cost_generative(attack) =                                                                ║
║           sybil_cost_base(attack)                    // Aus RL-V1/V2/V3                               ║
║         + creation_cost(fake_realms)                 // Trust + Zeit + DC3                            ║
║         + engagement_cost(fake_users)                // Echte Interaktion nötig                       ║
║         + quality_cost(low_retention)                // Schlechte Realms → niedriger Score           ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║                                                                                                        ║
║   DETECTION-HEURISTIKEN:                                                                              ║
║                                                                                                        ║
║   1. Creator-User-Collusion:                                                                          ║
║       if ∃ creator C, user_set U:                                                                    ║
║           |{u ∈ U : controller(u) = controller(C)}| / |U| > COLLUSION_THRESHOLD                      ║
║       then flag_suspicious(C)                                                                         ║
║                                                                                                        ║
║   2. Engagement-Anomaly:                                                                              ║
║       if session_pattern(realm) ≈ bot_signature                                                      ║
║       then reduce_engagement_score(realm)                                                            ║
║                                                                                                        ║
║   3. Content-Similarity:                                                                              ║
║       if ∃ realm₁, realm₂:                                                                           ║
║           similarity(ui(realm₁), ui(realm₂)) > CLONE_THRESHOLD                                       ║
║         ∧ creator(realm₁) ≈ creator(realm₂)  // Same controller                                      ║
║       then penalize_both(realm₁, realm₂)                                                             ║
║                                                                                                        ║
║   KONSTANTEN:                                                                                         ║
║       COLLUSION_THRESHOLD = 0.3                                                                       ║
║       CLONE_THRESHOLD = 0.8                                                                           ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 5.3 Weltformel-Integration

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR15 (WELTFORMEL-EXTENSION):                                                             ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   ERWEITERUNG der Weltformel (aus Κ15) für Generative-Realm-Aktivität:                                ║
║                                                                                                        ║
║       𝔼_extended = 𝔼_base + 𝔼_generative                                                             ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║                                                                                                        ║
║   𝔼_generative = Σ     𝔸_gen(realm) · 𝕢(realm) · Ĥ_creator(realm) · w(realm,t)                       ║
║                  realms                                                                               ║
║                                                                                                        ║
║   KOMPONENTEN:                                                                                        ║
║                                                                                                        ║
║       𝔸_gen(realm) = Aktivitäts-Präsenz des Realms                                                   ║
║                    = f(active_users, session_duration, interaction_rate)                             ║
║                                                                                                        ║
║       𝕢(realm) = Qualitäts-Score                                                                     ║
║                = retention(realm) · rating(realm) · novelty(realm)                                   ║
║                                                                                                        ║
║       Ĥ_creator(realm) = Human-Alignment des Creators                                                ║
║                        = Ĥ(controller(creator(realm)))                                                ║
║                                                                                                        ║
║       w(realm, t) = Temporale Gewichtung (aus Κ17)                                                   ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║                                                                                                        ║
║   INTERPRETATION:                                                                                     ║
║                                                                                                        ║
║       "Qualitativ hochwertige, aktive Realms von menschlich kontrollierten KIs                       ║
║        tragen zur System-Intelligenz bei."                                                            ║
║                                                                                                        ║
║       "Leere oder minderwertige Realms haben minimalen Beitrag."                                     ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

## VI. Sicherheitsmodell (GR16-GR18)

### 6.1 Threat Model für Generative Realms

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR16 (THREAT-MODEL):                                                                     ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   BEDROHUNGEN und MITIGATIONEN:                                                                       ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║                                                                                                        ║
║   T1. MALICIOUS UI-BUNDLE:                                                                            ║
║                                                                                                        ║
║       Bedrohung: Bundle enthält Malware (Crypto-Miner, Keylogger, etc.)                              ║
║                                                                                                        ║
║       Mitigation:                                                                                     ║
║         • CSP_MINIMUM erzwingt 'self'-Origin (GR6)                                                   ║
║         • Sandbox blockiert alle System-APIs                                                         ║
║         • JS-APIs sind allowlisted (kein eval, kein fetch außer Bridge)                              ║
║         • WASM-Module laufen in separatem Isolat                                                     ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║                                                                                                        ║
║   T2. MALICIOUS KI-CREATOR:                                                                           ║
║                                                                                                        ║
║       Bedrohung: KI generiert schädliche/illegale Inhalte                                            ║
║                                                                                                        ║
║       Mitigation:                                                                                     ║
║         • Creator braucht Trust τ_competence ≥ 0.5 (GR8)                                             ║
║         • Menschliche Kontrolle ist Pflicht (controller = human)                                     ║
║         • Community-Reports → Trust-Reduction (Κ4)                                                   ║
║         • Wiederholte Violations → Permanent Ban                                                     ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║                                                                                                        ║
║   T3. DATA EXFILTRATION:                                                                              ║
║                                                                                                        ║
║       Bedrohung: UI versucht, private Daten zu stehlen                                               ║
║                                                                                                        ║
║       Mitigation:                                                                                     ║
║         • Kein Zugriff auf lokalen Storage/Cookies                                                   ║
║         • Clipboard nur mit expliziter User-Geste                                                    ║
║         • Bridge-Messages sind rate-limited (GR6)                                                    ║
║         • Alle Kommunikation geht durch Onion-Layer                                                  ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║                                                                                                        ║
║   T4. DHT POISONING:                                                                                  ║
║                                                                                                        ║
║       Bedrohung: Angreifer ersetzt Bundle im DHT durch manipulierte Version                          ║
║                                                                                                        ║
║       Mitigation:                                                                                     ║
║         • Content-Addressierung: id(B) = hash(B) (GR2)                                               ║
║         • Jedes Bundle ist signiert (GR1)                                                            ║
║         • Client verifiziert Signatur vor Rendering                                                  ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║                                                                                                        ║
║   T5. DENIAL-OF-SERVICE:                                                                              ║
║                                                                                                        ║
║       Bedrohung: Angreifer überflutet Realm mit Requests                                             ║
║                                                                                                        ║
║       Mitigation:                                                                                     ║
║         • Rate-Limiting pro User (GR6)                                                               ║
║         • Trust-basierte Priorität                                                                    ║
║         • KI kann unresponsive Users ignorieren                                                      ║
║         • DC3-Kosten für Spam-Verhalten                                                              ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 6.2 Content-Security-Policy Spezifikation

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR17 (CSP-SPEZIFIKATION):                                                                ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   MINIMUM CSP (NICHT ÄNDERBAR):                                                                       ║
║                                                                                                        ║
║       Content-Security-Policy:                                                                        ║
║           default-src 'self';                                                                         ║
║           script-src 'self' 'wasm-unsafe-eval';                                                      ║
║           style-src 'self' 'unsafe-inline';                                                          ║
║           img-src 'self' data: blob:;                                                                ║
║           font-src 'self';                                                                           ║
║           connect-src 'self';                   // Nur Bridge-Endpunkt                               ║
║           frame-src 'none';                      // Keine iframes                                     ║
║           object-src 'none';                     // Kein Flash/Java                                   ║
║           base-uri 'self';                       // Kein base-tag-hijacking                          ║
║           form-action 'none';                    // Keine Form-Submits                               ║
║           frame-ancestors 'none';                // Nicht einbettbar                                 ║
║           upgrade-insecure-requests;                                                                 ║
║           block-all-mixed-content;                                                                   ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║                                                                                                        ║
║   ZUSÄTZLICHE SANDBOX-ATTRIBUTE:                                                                      ║
║                                                                                                        ║
║       <iframe sandbox="                                                                               ║
║           allow-scripts                          // JS erlaubt (aber sandboxed)                      ║
║           allow-same-origin                      // Für Bridge-Kommunikation                         ║
║       ">                                                                                              ║
║                                                                                                        ║
║       NICHT ERLAUBT:                                                                                  ║
║           allow-forms                            // Keine Formulare                                  ║
║           allow-popups                           // Keine Popups                                     ║
║           allow-top-navigation                   // Kann nicht aus Sandbox ausbrechen                ║
║           allow-modals                           // Keine alert/confirm/prompt                       ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 6.3 Trust-Propagation in Generative Realms

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   KERN-AXIOM GR18 (TRUST-PROPAGATION):                                                                ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   TRUST-FLUSS in Generative Realm:                                                                    ║
║                                                                                                        ║
║       User U betritt Realm R mit Creator 𝒜:                                                          ║
║                                                                                                        ║
║       𝕎_context(U, R) = 𝕎(U) ⊗ 𝕎(𝒜) ⊗ 𝕎(R)                                                          ║
║                                                                                                        ║
║       wobei ⊗ = komponenten-weise Minimum:                                                            ║
║           (v₁ ⊗ v₂)ᵢ = min(v₁ᵢ, v₂ᵢ)                                                                 ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║                                                                                                        ║
║   INTERPRETATION:                                                                                     ║
║                                                                                                        ║
║       "Der effektive Trust eines Users im Realm ist begrenzt durch:                                  ║
║        - Eigenes Trust-Level                                                                         ║
║        - Trust des Creators                                                                          ║
║        - Trust des Realms selbst"                                                                    ║
║                                                                                                        ║
║       "Ein untrusted Creator kann keinen trusted Realm betreiben."                                   ║
║       "Ein User kann nicht mehr Trust im Realm haben als außerhalb."                                 ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║                                                                                                        ║
║   TRUST-EVENTS in Realms:                                                                             ║
║                                                                                                        ║
║       positive_interaction(U, R) → Δ𝕎(U).P ↑, Δ𝕎(𝒜).P ↑                                              ║
║       report_issue(U, R)         → Δ𝕎(𝒜).I ↓, Δ𝕎(R) ↓                                                ║
║       long_engagement(U, R)      → Δ𝕎(U).R ↑, Δ𝕎(R) ↑                                                ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║                                                                                                        ║
║   REALM-TRUST:                                                                                        ║
║                                                                                                        ║
║       𝕎(R) = aggregate([𝕎(user) : user ∈ R] ∪ [𝕎(creator(R))])                                      ║
║                                                                                                        ║
║       "Ein Realm ist so vertrauenswürdig wie seine Community + Creator."                             ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

## VII. Implementierungs-Roadmap

### 7.1 Phasen-Übersicht

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   IMPLEMENTIERUNGS-ROADMAP: Generative Realms                                                         ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   PHASE 1: Foundation (4 Wochen)                                                                      ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║   • UIBundle-Struktur (GR1)                      ~500 LOC                                             ║
║   • DHT-Extension für Bundles (GR2)              ~800 LOC                                             ║
║   • GenerativeRealm-Typ (GR3)                    ~400 LOC                                             ║
║   • Join-Protokoll (GR4)                         ~600 LOC                                             ║
║                                                                                                        ║
║   PHASE 2: Bridge-API (3 Wochen)                                                                      ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║   • Bridge-Architektur (GR5)                     ~300 LOC (JS) + ~500 LOC (Rust)                      ║
║   • Sicherheitsmodell (GR6)                      ~400 LOC                                             ║
║   • Dynamic UI Patches (GR7)                     ~350 LOC                                             ║
║                                                                                                        ║
║   PHASE 3: KI-Integration (4 Wochen)                                                                  ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║   • Creator-Eligibility (GR8)                    ~300 LOC                                             ║
║   • Generation-Process (GR9)                     ~700 LOC (Python/LangChain)                          ║
║   • Dungeon-Master-Semantik (GR10)               ~500 LOC                                             ║
║   • Multiplayer-Konsistenz (GR11)                ~600 LOC                                             ║
║   • Realm-Lifecycle (GR12)                       ~400 LOC                                             ║
║                                                                                                        ║
║   PHASE 4: Incentives & Security (3 Wochen)                                                           ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║   • DC3-Extension (GR13)                         ~450 LOC                                             ║
║   • Anti-Sybil-Extension (GR14)                  ~500 LOC                                             ║
║   • Weltformel-Extension (GR15)                  ~350 LOC                                             ║
║   • Security-Hardening (GR16-18)                 ~600 LOC                                             ║
║                                                                                                        ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║   TOTAL: ~7.250 LOC | ~14 Wochen | 1 Entwickler Full-Time                                            ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 7.2 Abhängigkeiten zu bestehenden Modulen

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   MODUL-ABHÄNGIGKEITEN                                                                                ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   Generative Realms nutzt:                                                                            ║
║                                                                                                        ║
║   ┌─────────────────────────────────────────────────────────────────────────────────────────────────┐  ║
║   │                                                                                                 │  ║
║   │   peer/p2p/privacy/                                                                             │  ║
║   │   ├── onion.rs          → Bridge-Encryption (RL2-RL4)                                          │  ║
║   │   ├── relay_selection.rs → Route-Auswahl für KI-Kommunikation                                  │  ║
║   │   ├── dc3.rs            → DC3-Scoring-Basis                                                    │  ║
║   │   ├── eligibility.rs    → Creator-Eligibility                                                  │  ║
║   │   └── resource_verification.rs → Bundle-Hosting-Rewards                                        │  ║
║   │                                                                                                 │  ║
║   │   domain/unified/                                                                               │  ║
║   │   ├── realm.rs          → VirtualRealm-Basis                                                   │  ║
║   │   ├── trust.rs          → Trust-Vektor & Kombination                                           │  ║
║   │   ├── identity.rs       → DID-System                                                           │  ║
║   │   └── formula.rs        → Weltformel-Basis                                                     │  ║
║   │                                                                                                 │  ║
║   │   core/                                                                                         │  ║
║   │   ├── world_formula.rs  → Weltformel-Integration                                               │  ║
║   │   └── trust_engine.rs   → Trust-Updates                                                        │  ║
║   │                                                                                                 │  ║
║   │   peer/p2p/                                                                                     │  ║
║   │   ├── topics.rs         → Gossipsub-Channels                                                   │  ║
║   │   └── swarm.rs          → P2P-Netzwerk                                                         │  ║
║   │                                                                                                 │  ║
║   └─────────────────────────────────────────────────────────────────────────────────────────────────┘  ║
║                                                                                                        ║
║   NEUE MODULE:                                                                                        ║
║                                                                                                        ║
║   ┌─────────────────────────────────────────────────────────────────────────────────────────────────┐  ║
║   │                                                                                                 │  ║
║   │   peer/p2p/generative/         (NEU)                                                           │  ║
║   │   ├── mod.rs                   Module-Root                                                     │  ║
║   │   ├── bundle.rs                UIBundle-Struktur (GR1)                                         │  ║
║   │   ├── storage.rs               DHT-Extension (GR2)                                             │  ║
║   │   ├── realm.rs                 GenerativeRealm (GR3, GR12)                                     │  ║
║   │   ├── join.rs                  Join-Protokoll (GR4)                                            │  ║
║   │   ├── bridge.rs                Bridge-API (GR5-GR7)                                            │  ║
║   │   ├── creator.rs               KI-Creator (GR8-GR10)                                           │  ║
║   │   ├── multiplayer.rs           Multiplayer-State (GR11)                                        │  ║
║   │   ├── incentives.rs            DC3-Extension (GR13-GR15)                                       │  ║
║   │   └── security.rs              Security-Layer (GR16-GR18)                                      │  ║
║   │                                                                                                 │  ║
║   │   frontend/platform/src/                                                                        │  ║
║   │   ├── sandbox/                 WebView-Sandbox-Integration                                     │  ║
║   │   └── bridge/                  JS-Bridge-Implementation                                        │  ║
║   │                                                                                                 │  ║
║   └─────────────────────────────────────────────────────────────────────────────────────────────────┘  ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

## VIII. Zusammenfassung und Vision

### 8.1 Axiom-Übersicht

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   AXIOM-REFERENZ: Generative Realms (GR1-GR18)                                                        ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   ID    Name                       Beschreibung                               Abhängigkeit             ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────║
║   GR1   Bundle-Struktur            UIBundle = ⟨manifest, assets, logic, sig⟩  Κ6 (DID)                ║
║   GR2   Content-Addressierung      id(B) = hash(B), DHT-Storage              RL-V1                    ║
║   GR3   Generative-Realm-Def       GenerativeRealm ⊂ VirtualRealm            Κ1 (Regelvererbung)     ║
║   GR4   Join-Protokoll             9-Schritt-Beitritts-Flow                  Κ4, RL2-4               ║
║   GR5   Bridge-Architektur         Sandbox ⇄ Bridge ⇄ Network                RL2-RL4                 ║
║   GR6   Bridge-Sicherheit          Isolation + Rate-Limit + Sanitization     CSP                     ║
║   GR7   Dynamic UI Patches         UIPatch mit Signatur-Verifikation         GR1, GR3                ║
║   GR8   Creator-Eligibility        Trust + Controller + DC3 Requirements     Κ3, Κ22                 ║
║   GR9   Generation-Process         Π-GEN Hoare-Triple                        Κ11                     ║
║   GR10  Dungeon-Master-Semantik    KI als Welt-Controller                    GR3, GR7                ║
║   GR11  Multiplayer-Konsistenz     Shared/Private State + Conflict Res       Κ9 (Kausalität)         ║
║   GR12  Realm-Lifecycle            DRAFT → PUBLISHED → ACTIVE → ARCHIVED     GR3                     ║
║   GR13  DC3-Extension              Hosting + Serving + Creation Rewards      Κ22                     ║
║   GR14  Anti-Sybil-Extension       Collusion + Anomaly + Clone Detection     RL-V1/V2/V3             ║
║   GR15  Weltformel-Extension       𝔼_generative = Σ 𝔸_gen · 𝕢 · Ĥ · w        Κ15                     ║
║   GR16  Threat-Model               T1-T5 mit Mitigationen                    Alle                    ║
║   GR17  CSP-Spezifikation          Minimum-CSP + Sandbox-Attribute           GR6                     ║
║   GR18  Trust-Propagation          𝕎_context = 𝕎_user ⊗ 𝕎_creator ⊗ 𝕎_realm Κ2-Κ5                  ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 8.2 Vision: Das KI-Metaverse

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║                           ERYNOA: DAS DEZENTRALE KI-METAVERSE                                         ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║                                                                                                        ║
║           ┌───────────────────────────────────────────────────────────────────────────────┐           ║
║           │                                                                               │           ║
║           │         USER                    KI-CREATORS                   RELAYS          │           ║
║           │          │                           │                          │             │           ║
║           │          │  erynoa://realm/join      │  generate(prompt)        │  host(B)    │           ║
║           │          │  ───────────────────►     │  ──────────────────►     │  ────────►  │           ║
║           │          │                           │                          │             │           ║
║           │          │      ┌────────────────────▼────────────────────┐     │             │           ║
║           │          │      │                                         │     │             │           ║
║           │          │      │     G E N E R A T I V E   R E A L M     │     │             │           ║
║           │          │      │                                         │     │             │           ║
║           │          │      │  ┌─────────────────────────────────┐    │     │             │           ║
║           │          │      │  │  UI-BUNDLE (DHT-gespeichert)    │    │     │             │           ║
║           │          │      │  │  • HTML/CSS/JS                  │    │     │             │           ║
║           │          │      │  │  • WASM-Logik                   │    │     │             │           ║
║           │          │      │  │  • Assets                       │    │     │             │           ║
║           │          │      │  └─────────────────────────────────┘    │     │             │           ║
║           │          │      │                                         │     │             │           ║
║           │          │      │  ┌─────────────────────────────────┐    │     │             │           ║
║           │          │      │  │  SANDBOX (Client-lokal)         │    │     │             │           ║
║           │          │      │  │  • Isolierte Ausführung         │    │     │             │           ║
║           │          │      │  │  • Bridge-API                   │    │     │             │           ║
║           │          │      │  │  • CSP-Enforcement              │    │     │             │           ║
║           │          │      │  └─────────────────────────────────┘    │     │             │           ║
║           │          │      │                                         │     │             │           ║
║           │          │      │  ┌─────────────────────────────────┐    │     │             │           ║
║           │          │      │  │  KI-DUNGEON-MASTER              │    │     │             │           ║
║           │          │      │  │  • Echtzeit-Responses           │    │     │             │           ║
║           │          │      │  │  • Dynamic UI Updates           │    │     │             │           ║
║           │          │      │  │  • Welt-State-Management        │    │     │             │           ║
║           │          │      │  └─────────────────────────────────┘    │     │             │           ║
║           │          │      │                                         │     │             │           ║
║           │          │      └─────────────────────────────────────────┘     │             │           ║
║           │          │                                                       │             │           ║
║           │          │  ◄──────────  ONION-ENCRYPTED  ──────────────────────►             │           ║
║           │          │              (Privacy-Preserving)                                  │           ║
║           │                                                                               │           ║
║           └───────────────────────────────────────────────────────────────────────────────┘           ║
║                                                                                                        ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   USE-CASES:                                                                                          ║
║                                                                                                        ║
║   🎮 SPIELE           KI erstellt interaktive Text-Adventures, Puzzle-Räume, Escape-Games            ║
║   📚 LERNEN           Adaptive Lernumgebungen, die sich an den Schüler anpassen                      ║
║   💼 ARBEIT           Kollaborative Workspaces mit KI-Moderation und -Dokumentation                  ║
║   🎨 KUNST            Generative Ausstellungen, interaktive Installationen                           ║
║   🤖 ASSISTENTEN      Persönliche Assistenten mit eigenem UI und Look & Feel                         ║
║   🏛️ GOVERNANCE       Visualisierte Abstimmungsumgebungen für DAOs                                   ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   "Erynoa ist nicht nur ein Privacy-Tool – es ist die Plattform für KI-gesteuerte Welten,            ║
║    in denen Menschen und Maschinen in einer vertrauenswürdigen, dezentralen Umgebung                  ║
║    zusammenarbeiten, spielen und lernen."                                                              ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

## IX. Formale Verifikation

### 9.1 Invarianten

```
╔════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                        ║
║   SYSTEM-INVARIANTEN für Generative Realms                                                            ║
║                                                                                                        ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════  ║
║                                                                                                        ║
║   INV-GR1 (Bundle-Integrität):                                                                        ║
║       ∀ B loaded in Sandbox: verify(B.signature, creator(B).pk, hash(B)) = true                      ║
║                                                                                                        ║
║   INV-GR2 (Sandbox-Isolation):                                                                        ║
║       ∀ Sandbox S, System-Resource R: ¬access(S, R) unless R ∈ {Bridge-API}                          ║
║                                                                                                        ║
║   INV-GR3 (Trust-Bound):                                                                              ║
║       ∀ User U in Realm R: 𝕎_effective(U) ≤ min(𝕎(U), 𝕎(creator(R)), 𝕎(R))                          ║
║                                                                                                        ║
║   INV-GR4 (Creator-Eligibility):                                                                      ║
║       ∀ GenerativeRealm R: eligible_creator(creator(R)) at creation_time(R)                          ║
║                                                                                                        ║
║   INV-GR5 (Rule-Inheritance):                                                                         ║
║       ∀ GenerativeRealm R: rules(R) ⊇ rules(VirtualRealm) ⊇ rules(RootRealm)                         ║
║                                                                                                        ║
║   INV-GR6 (Causal-Consistency):                                                                       ║
║       ∀ patch p₁ ⊲ p₂ in Realm R: ∀ observer: see(p₁) before see(p₂)                                ║
║                                                                                                        ║
║   INV-GR7 (Incentive-Alignment):                                                                      ║
║       ∀ malicious_action: cost(malicious_action) > expected_reward(malicious_action)                 ║
║                                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

**ENDE DER SPEZIFIKATION**

_Diese Logik-Erweiterung macht Erynoa zum ersten dezentralen, privacy-first KI-Metaverse._
