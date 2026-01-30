# Erynoa Protocol Specification V6.0

> **Version:** 6.0 – Humanistisches Quanten-Kybernetisches Protokoll
> **Datum:** Januar 2026
> **Grundlage:** Weltformel V6.0, 120 Axiome über 8 Ebenen
> **Paradigma:** Content-Addressable, Merkle-Verified, Quantum-Trust, Category-Bridged, Human-Aligned

---

## Präambel: Die Weltformel als Protokoll-Grundlage

Jede Operation in Erynoa verändert den Systemzustand gemäß der Weltformel:

```
𝔼 = Σ  ⟨Ψₛ| 𝔸̂ · σ̂( 𝕎̂ · ln|ℂ̂| · ℕ̂ / 𝔼x̂p ) · Ĥ |Ψₛ⟩ · w(t)
    s∈𝒞
```

Das Protokoll definiert, wie diese abstrakten Operatoren konkret berechnet und verändert werden. Jede Benutzeraktion ist eine Transformation der Weltformel.

**V6.0 Erweiterungen:**
- `Ĥ` = Human-Alignment Operator (2.0 für Menschen, 1.5 für human-kontrolliert)
- `w(t)` = Temporale Gewichtung (Vergebungs-Faktor mit Halbwertszeiten)
- LoD = Level-of-Detail basierte Vertrauens-Auflösung
- Semantische Verankerung für alle Blueprints

---

# TEIL I: OBJEKT-MODELL

## 1. Fundamentale Objekt-Typen

### 1.1 Das Datum-Objekt

Das `datum` ist die atomare Dateneinheit – analog zum Git-Blob, aber mit semantischer Anreicherung.

```
DATUM-STRUKTUR
══════════════════════════════════════════════════════════════════════

datum {
    // Identifikation
    id:           hash(DATUM_PREFIX || encoding || content)
    
    // Inhalt
    content:      bytes                    // Rohdaten
    encoding:     Encoding                 // cbor | json | protobuf | raw
    
    // Semantische Anreicherung
    schema_ref:   SchemaHash?              // Optional: Verweis auf Schema
    embedding:    Vector<f32, 128>?        // Optional: Semantisches Embedding
    
    // Metadaten
    created_at:   LogicalTimestamp
    size_bytes:   u64
}

HASH-BERECHNUNG (Axiom A12: Kausalität)
───────────────────────────────────────
    datum.id = SHA3-256(
        "datum:" ||                        // Typ-Präfix
        varint(encoding) ||                // Encoding als Varint
        content                            // Roher Inhalt
    )

EMBEDDING-BERECHNUNG (Axiom Q11: Axiom-Embeddings)
──────────────────────────────────────────────────
    Wenn schema_ref vorhanden:
        embedding = Embed(content, schema_ref)
        
    Embed : (Content × Schema) → ℝ¹²⁸
    
    Die 128 Dimensionen:
        d[0..19]:   Ethik-Dimensionen
        d[20..39]:  Prozess-Dimensionen
        d[40..59]:  Ressourcen-Dimensionen
        d[60..79]:  Kontext-Dimensionen
        d[80..99]:  Beziehungs-Dimensionen
        d[100..127]: Abstrakte Dimensionen
```

### 1.2 Das Event-Objekt

Das `event` ist die atomare Zustandsänderung – analog zum Git-Commit, aber mit Quanten-Trust-Integration.

```
EVENT-STRUKTUR
══════════════════════════════════════════════════════════════════════

event {
    // Identifikation
    id:           hash(EVENT_PREFIX || header || payload_hash)
    
    // Header
    header {
        type:         EventType
        actor:        DID
        parents:      [EventHash; 1..N]    // DAG-Vorgänger
        shard:        ShardID
        realm:        RealmID
        timestamp:    LamportClock
        nonce:        u64                  // Replay-Schutz
    }
    
    // Payload
    payload_hash:     DatumHash            // Verweis auf datum
    
    // Kryptographie
    signature:        Signature
    
    // Bezeugung (nach Propagation)
    attestations:     [Attestation; 0..M]
    
    // Trust-Snapshot (berechnet)
    trust_snapshot {
        actor_psi:    QuantumState         // |Ψ_actor⟩ vor Event
        delta_W:      f64                  // Erwartete 𝕎-Änderung
        novelty:      f64                  // ℕ des Events
        expectation:  f64                  // 𝔼xp des Events
    }
}

EVENT-TYPEN MIT AXIOM-REFERENZ
──────────────────────────────

EventType {
    // Ebene 1: Fundament
    GENESIS         // A1-A5: Identitäts-Erstellung
    REVOKE          // A3: Schlüssel-Widerruf
    
    // Ebene 3: Prozess
    TRANSFER        // P1-P6: Asset-Transfer
    STREAM_START    // T4: Streaming beginnen
    STREAM_TICK     // T4: Streaming-Inkrement
    STREAM_END      // T5: Streaming beenden
    ABORT           // T7: Abbruch mit Settlement
    
    // Ebene 4: Objekt
    MINT            // O1: AMO erstellen
    BURN            // O1: AMO zerstören
    CREDENTIAL_ISSUE// C1: Credential ausstellen
    CREDENTIAL_REVOKE// C4: Credential widerrufen
    
    // Ebene 2: Emergenz
    ATTEST          // E5-E10: Bezeugung
    CLAIM           // E11-E15: Behauptung
    
    // Ebene 5: Schutz
    DISPUTE         // S9-S12: Qualitäts-Dispute
    REPORT          // S11: Kollusion melden
    
    // Ebene 5: Governance
    PROPOSAL        // S13-S18: Governance-Vorschlag
    VOTE            // S14: Abstimmung
    VETO            // S17: Minderheits-Veto
}
```

### 1.3 Der Quanten-Zustand

Der `QuantumState` repräsentiert den Trust eines Agenten als Superposition (Axiom Q1).

```
QUANTEN-ZUSTAND-STRUKTUR
══════════════════════════════════════════════════════════════════════

QuantumState {
    // Basis-Zustände mit komplexen Amplituden
    amplitudes: Map<TrustBasis, Complex>
    
    // Normierungs-Invariante (Axiom Q1)
    // Σ |αᵢ|² = 1
    
    // Kontext-Abhängigkeit (Axiom Q4)
    context:    ShardID
    
    // Verschränkungen (Axiom Q3)
    entanglements: [EntanglementRef]
}

TrustBasis {
    HONEST,         // |honest⟩      Vollständig vertrauenswürdig
    RELIABLE,       // |reliable⟩    Meist zuverlässig
    NEUTRAL,        // |neutral⟩     Unbekannt/Standard
    UNRELIABLE,     // |unreliable⟩  Meist unzuverlässig
    MALICIOUS       // |malicious⟩   Aktiv bösartig
}

BEISPIEL: NEUER AGENT
─────────────────────
|Ψ_new⟩ = 0.10|honest⟩ + 0.20|reliable⟩ + 0.95|neutral⟩ 
        + 0.10|unreliable⟩ + 0.05|malicious⟩

Normierung: 0.01 + 0.04 + 0.9025 + 0.01 + 0.0025 = 0.965 ≈ 1
(leichte Abweichung durch Rundung)

ERWARTUNGSWERT-BERECHNUNG
─────────────────────────
Die Eigenwerte der Basis-Zustände:
    λ(honest)     = 1.0
    λ(reliable)   = 0.75
    λ(neutral)    = 0.5
    λ(unreliable) = 0.25
    λ(malicious)  = 0.0

𝕎 = ⟨Ψ|𝕎̂|Ψ⟩ = Σᵢ |αᵢ|² · λᵢ

Für neuen Agent:
    𝕎 = 0.01×1.0 + 0.04×0.75 + 0.9025×0.5 + 0.01×0.25 + 0.0025×0.0
      = 0.01 + 0.03 + 0.45 + 0.0025 + 0
      = 0.49
```

### 1.4 Die Kategorie-Struktur

Jeder Realm ist eine Kategorie (Axiom Q6), Shards sind Sub-Kategorien.

```
KATEGORIE-STRUKTUR
══════════════════════════════════════════════════════════════════════

Category {
    // Objekte = Agenten
    objects:        Set<DID>
    
    // Morphismen = Transaktionen zwischen Agenten
    morphisms:      Set<Transaction>
    
    // Identitäts-Morphismus für jedes Objekt
    identity:       DID → Transaction    // id_s : s → s
    
    // Komposition von Morphismen
    compose:        (Transaction × Transaction) → Transaction
    
    // Axiome einer Kategorie (mathematisch garantiert)
    // 1. Assoziativität: (f ∘ g) ∘ h = f ∘ (g ∘ h)
    // 2. Identität: f ∘ id = f = id ∘ f
}

REALM ALS KATEGORIE
───────────────────
𝒞_Realm = (
    Ob:     { did:erynoa:*:* | registered in Realm },
    Mor:    { tx : s₁ → s₂ | tx.realm = Realm },
    ∘:      tx₂ ∘ tx₁ = Sequential(tx₁, tx₂),
    id:     id_s = NoOp(s)
)

SHARD ALS SUB-KATEGORIE
───────────────────────
𝒞_Shard ⊂ 𝒞_Realm

Inklusions-Funktor:
    I : 𝒞_Shard → 𝒞_Realm
    I(s) = s                    // Objekte unverändert
    I(tx) = tx                  // Morphismen unverändert
```

---

# TEIL II: IDENTITÄTS-OPERATIONEN

## 2. INIT – Identität erstellen

Die fundamentalste Operation: Erzeugung einer neuen Existenz im System.

```
OPERATION: erynoa init
══════════════════════════════════════════════════════════════════════

SYNTAX
──────
erynoa init [OPTIONS]

OPTIONS
    --namespace <NS>      Namespace der DID (default: personal)
    --algorithm <ALG>     Kryptographischer Algorithmus
                          ed25519 | secp256k1 | bls12-381
    --sub-identity <DID>  Als Sub-Identität erstellen
    --recover <SEED>      Aus Seed-Phrase wiederherstellen

AXIOM-REFERENZEN
    A1:  Existenz erfordert Identität
    A2:  Einzigartigkeit der DID
    A3:  Schlüssel-Binding
    A4:  Sub-Identitäts-Verknüpfung
    Q1:  Initialer Quanten-Zustand

ALGORITHMUS
───────────
1. SCHLÜSSEL-GENERIERUNG
   
   (sk, pk) ← KeyGen(algorithm)
   
   Für ed25519:
       sk ∈ {0,1}²⁵⁶              // 256-bit Zufallszahl
       pk = sk · G                 // Skalarmultiplikation auf Ed25519
   
2. DID-BERECHNUNG
   
   unique_id = Base58(SHA3-256(pk)[0..16])
   did = "did:erynoa:" || namespace || ":" || unique_id

3. INITIALER QUANTEN-ZUSTAND (Axiom Q1)
   
   |Ψ_init⟩ = √0.01|honest⟩ + √0.04|reliable⟩ + √0.90|neutral⟩ 
            + √0.04|unreliable⟩ + √0.01|malicious⟩
   
   Vereinfacht: Fast vollständig im |neutral⟩ Zustand

4. GENESIS-EVENT ERSTELLEN
   
   genesis_event = Event {
       type:     GENESIS,
       actor:    did,
       parents:  [NETWORK_GENESIS],        // Wurzel des Netzwerks
       payload:  datum(DIDDocument),
       trust_snapshot: {
           actor_psi:   |Ψ_init⟩,
           delta_W:     0,                 // Keine Änderung
           novelty:     1.0,               // Neuer Agent = hohe Novelty
           expectation: 1.0                // Keine Historie = neutral
       }
   }

5. SIGNATUR
   
   sig = Sign(sk, hash(genesis_event))
   genesis_event.signature = sig

6. LOKALE SPEICHERUNG
   
   Store(~/.erynoa/keys/did.key, sk)
   Store(~/.erynoa/identity.json, DIDDocument)
   Store(~/.erynoa/events/genesis.event, genesis_event)

WELTFORMEL-IMPACT
─────────────────
Vor Init:   Δ𝔼 = 0 (Agent existiert nicht)
Nach Init:  Δ𝔼 = 𝔸(s_new) · σ(𝕎_init · ln(1) · ℕ_init / 𝔼xp_init)
                = 0 · σ(0.49 · 0 · 1.0 / 1.0)
                = 0

Der Agent existiert, trägt aber noch nichts bei (𝔸 = 0, |ℂ| = 1).

AUSGABE
───────
    Erynoa Identity Initialization
    ═══════════════════════════════
    
    Algorithm:       ed25519
    Namespace:       personal
    
    DID:             did:erynoa:personal:7xK9m2P4q8Yz
    Public Key:      ed25519:2Wj8kL...truncated...9xNp
    
    Quantum State:   |Ψ⟩ = 0.95|neutral⟩ + ...
    Expected Trust:  𝕎 = 0.49
    Trust Tier:      FRESH
    
    Genesis Event:   event:sha3:a1b2c3d4...
    
    ⚠️  BACKUP YOUR PRIVATE KEY!
    Location: ~/.erynoa/keys/did:erynoa:personal:7xK9m2P4q8Yz.key
    
    Run 'erynoa push' to announce your identity to the network.
```

## 3. SUB-IDENTITY – Verschränkte Identität erstellen

Sub-Identitäten sind quantenmechanisch mit der Haupt-Identität verschränkt (Axiom Q3).

```
OPERATION: erynoa sub-identity create
══════════════════════════════════════════════════════════════════════

SYNTAX
──────
erynoa sub-identity create <NAME> [OPTIONS]

OPTIONS
    --inherit-trust <FACTOR>    Trust-Vererbung (0.0 - 1.0, default: 0.5)
    --context <SHARD>           Kontext-Beschränkung
    --permissions <PERMS>       Erlaubte Aktionen

VERSCHRÄNKUNGS-MECHANIK (Axiom Q3)
──────────────────────────────────
Der Zustand der Sub-Identität ist mit der Haupt-Identität verschränkt:

|Ψ_entangled⟩ = α|τ_main, τ_sub⟩

Konkret:
    |Ψ_system⟩ = Σᵢⱼ αᵢⱼ |τᵢ⟩_main ⊗ |τⱼ⟩_sub

Die Korrelationsmatrix:
    
    Sub\Main    honest  reliable  neutral  unreliable  malicious
    ─────────────────────────────────────────────────────────────
    honest      0.70    0.20      0.08     0.02        0.00
    reliable    0.20    0.50      0.25     0.04        0.01
    neutral     0.08    0.25      0.55     0.10        0.02
    unreliable  0.02    0.04      0.10     0.70        0.14
    malicious   0.00    0.01      0.02     0.14        0.83

Interpretation:
    - Wenn Main als |honest⟩ gemessen wird, ist Sub mit 70% auch |honest⟩
    - Die Korrelation ist stärker für extreme Zustände
    - |neutral⟩ korreliert am schwächsten (mehr Unabhängigkeit)

KOLLAPS-PROPAGATION
───────────────────
Wenn die Haupt-Identität eine Interaktion hat:

1. Main wird gemessen: |Ψ_main⟩ → |τₖ⟩
2. Sub kollabiert bedingt:
   
   |Ψ_sub | τₖ⟩ = Σⱼ (αₖⱼ / √Σⱼ|αₖⱼ|²) |τⱼ⟩

Beispiel:
    Main gemessen als |honest⟩:
    |Ψ_sub | honest⟩ = 0.84|honest⟩ + 0.45|reliable⟩ + 0.28|neutral⟩ + ...
    
    → Sub-Trust steigt ebenfalls!

ALGORITHMUS
───────────
1. Generiere neuen Schlüssel für Sub-Identität
2. Erstelle DID mit Verweis auf Main
3. Berechne initiale Verschränkungsmatrix basierend auf inherit-trust
4. Erstelle GENESIS-Event mit Entanglement-Referenz
5. Registriere Verschränkung im lokalen Zustand

AUSGABE
───────
    Sub-Identity Created
    ════════════════════
    
    Main DID:        did:erynoa:personal:7xK9m2P4q8Yz
    Sub DID:         did:erynoa:personal:7xK9m2P4q8Yz:gaming
    
    Entanglement:
        Type:        POSITIVE (correlated trust)
        Strength:    0.5 (inherit-trust factor)
        
        Correlation Matrix:
        ┌─────────────────────────────────────────┐
        │ P(sub=honest | main=honest) = 0.70     │
        │ P(sub=honest | main=reliable) = 0.20   │
        │ ...                                     │
        └─────────────────────────────────────────┘
    
    Context:         gaming (restricted)
    Permissions:     transfer, attest, claim
    
    Initial State:
        Main 𝕎:      0.78
        Sub 𝕎:       0.39 (= 0.78 × 0.5)
```

---

# TEIL III: DATEN-OPERATIONEN

## 4. ADD – Daten mit semantischer Validierung hinzufügen

Das Hinzufügen von Daten involviert die topologische Validierung (Axiom Q13).

```
OPERATION: erynoa add
══════════════════════════════════════════════════════════════════════

SYNTAX
──────
erynoa add <FILE> [OPTIONS]

OPTIONS
    --type <TYPE>         asset | credential | claim | service
    --schema <SCHEMA>     Schema-Referenz für Validierung
    --private             Nicht öffentlich speichern
    --zkp                 Zero-Knowledge-Proof generieren

WEICHE VALIDIERUNG (Axiom Q13: Ω_soft)
──────────────────────────────────────
Anstatt binärer Validierung (gültig/ungültig) wird semantische
Ähnlichkeit gemessen:

Ω_soft(data) = Σᵢ wᵢ · sim(Embed(data), Embed(Axiomᵢ))

Wobei:
    sim(a, b) = cos(θ) = (a · b) / (‖a‖ · ‖b‖)    (Axiom Q12)

VALIDIERUNGS-SCHWELLEN
──────────────────────
    Ω_soft > 0.95    →  FULL_COMPLIANCE      (grün)
    Ω_soft ∈ [0.80, 0.95] →  COMPLIANT       (gelb, Hinweis)
    Ω_soft ∈ [0.60, 0.80] →  MARGINAL        (orange, Review)
    Ω_soft < 0.60    →  NON_COMPLIANT        (rot, Ablehnung)

ALGORITHMUS
───────────
1. DATEI LESEN UND PARSEN
   
   content = ReadFile(file)
   parsed = Parse(content, encoding)

2. SCHEMA LADEN (falls angegeben)
   
   schema = FetchSchema(schema_ref)

3. EMBEDDING BERECHNEN (Axiom Q11)
   
   embedding = Embed(parsed, schema)
   
   // Embedding-Modell: Transformer mit 128-dim Output
   // Trainiert auf Erynoa-Axiome und Domain-Daten

4. WEICHE VALIDIERUNG
   
   // Relevante Axiome für diesen Datentyp
   relevant_axioms = GetRelevantAxioms(type, schema)
   
   // Für jedes Axiom: Ähnlichkeit berechnen
   scores = []
   for axiom in relevant_axioms:
       axiom_embedding = Embed(axiom)
       score = CosineSimilarity(embedding, axiom_embedding)
       scores.append((axiom, score))
   
   // Gewichteter Durchschnitt
   Ω_soft = WeightedAverage(scores)

5. VALIDIERUNGS-ENTSCHEIDUNG
   
   if Ω_soft < 0.60:
       REJECT("Non-compliant with axioms")
   elif Ω_soft < 0.80:
       WARN("Marginal compliance, review recommended")
       // Zeige welche Axiome problematisch sind
       problematic = [a for (a, s) in scores if s < 0.80]
       ShowProblematicAxioms(problematic)

6. DATUM ERSTELLEN
   
   datum = Datum {
       content:    content,
       encoding:   detected_encoding,
       schema_ref: schema_ref,
       embedding:  embedding
   }
   
   // Hash berechnen
   datum.id = SHA3-256("datum:" || encoding || content)

7. STAGING
   
   Stage(datum)

DETAILLIERTE AUSGABE
────────────────────
    Erynoa Add: meter-reading.json
    ══════════════════════════════════════════════════════════════
    
    File Analysis
    ─────────────
    Size:           1,247 bytes
    Encoding:       JSON (detected)
    Schema:         amo:energy:meter-reading:v2
    
    Semantic Embedding
    ──────────────────
    Dimensions:     128
    Norm:           1.000 (normalized)
    
    Top-5 semantic clusters:
        [0.92] Energy/Metering
        [0.87] Measurement/Precision
        [0.71] Temporal/Timestamp
        [0.65] Location/Geographic
        [0.43] Financial/Billing
    
    Axiom Compliance Analysis (Ω_soft)
    ───────────────────────────────────
    
    Axiom                              Similarity    Status
    ─────────────────────────────────────────────────────────
    A12 (Kausalität)                   0.98         ✓ Full
    A23 (Wert-Definition)              0.95         ✓ Full
    A24 (Wert-Erhalt)                  0.92         ✓ Compliant
    O1  (AMO-Existenz)                 0.97         ✓ Full
    O3  (Blueprint-Konformität)        0.94         ✓ Compliant
    O5  (Logic-Guard-Bindung)          0.88         ⚠ Compliant
    P1  (Prozess-Korrektheit)          0.91         ✓ Compliant
    
    Aggregated Ω_soft: 0.936
    Status: COMPLIANT ✓
    
    ⚠  Note on O5: Logic Guard binding is present but uses
       deprecated gas model. Consider upgrading to v2 gas.
    
    Datum Created
    ─────────────
    ID:             datum:sha3:7k2m9x4p...
    Staged:         Yes
    
    Manifold Analysis (Axiom Q14)
    ─────────────────────────────
    Distance to validity manifold: 0.023 (normal range: < 0.1)
    Nearest neighbors on manifold:
        1. datum:sha3:similar1... (sim: 0.94)
        2. datum:sha3:similar2... (sim: 0.91)
        3. datum:sha3:similar3... (sim: 0.89)
    
    No anomalies detected.
    
    Run 'erynoa commit' to create an event.
```

---

# TEIL IV: EVENT-OPERATIONEN

## 5. COMMIT – Event mit Trust-Berechnung erstellen

Das Erstellen eines Events berechnet den vollen Trust-Impact gemäß der Weltformel.

```
OPERATION: erynoa commit
══════════════════════════════════════════════════════════════════════

SYNTAX
──────
erynoa commit [OPTIONS]

OPTIONS
    --type <TYPE>         Event-Typ (auto-detected if omitted)
    --message <MSG>       Beschreibung
    --parents <EVENTS>    Explizite Parent-Events

WELTFORMEL-BERECHNUNG
─────────────────────
Jedes Event verändert die Weltformel. Der Impact wird vorab berechnet.

VOR DEM EVENT:
    𝔼_before = Σ  ⟨Ψₛ| 𝔸̂ · σ̂( 𝕎̂ · ln|ℂ̂| · ℕ̂ / 𝔼x̂p ) |Ψₛ⟩
               s∈𝒞

NACH DEM EVENT:
    𝔼_after = 𝔼_before + Δ𝔼(event)

DELTA-BERECHNUNG
────────────────
Der Delta hängt vom Event-Typ ab:

Δ𝔼(event) = 𝔸'(actor) · σ'(𝕎'(actor) · ln|ℂ'(actor)| · ℕ'(event) / 𝔼xp'(event))
          - 𝔸(actor) · σ(𝕎(actor) · ln|ℂ(actor)| · ℕ(actor) / 𝔼xp(actor))

Wobei:
    𝔸'(actor)  = 𝔸(actor) + activity_boost(event_type)
    |ℂ'(actor)| = |ℂ(actor)| + 1
    ℕ'(event)  = ComputeNovelty(event)
    𝔼xp'(event) = ComputeExpectation(event, history)

NOVELTY-BERECHNUNG (Axiom K1)
─────────────────────────────
ℕ(event) = α · information_gain(event) + β · verification_rate(actor)

information_gain(event) = H(system_before) - H(system_after | event)

H = Shannon-Entropie über die Verteilung der Zustände

verification_rate(actor) = verified_novel_claims(actor) / total_novel_claims(actor)

Parameter:
    α = 0.6    (Gewicht für Information Gain)
    β = 0.4    (Gewicht für Verification Rate)

EXPECTATION-BERECHNUNG (Axiom K2)
─────────────────────────────────
𝔼xp(event) = 1 + |predicted_behavior(actor) - actual_behavior(event)| / σ_baseline

predicted_behavior(actor) = Modell-Vorhersage basierend auf Historie
actual_behavior(event) = Tatsächliches Verhalten in diesem Event
σ_baseline = Standard-Abweichung im System (gleitend)

ALGORITHMUS
───────────
1. STAGED ITEMS SAMMELN
   
   staged_datums = GetStaged()
   
   if staged_datums.empty():
       ERROR("Nothing staged. Use 'erynoa add' first.")

2. EVENT-TYP BESTIMMEN
   
   if --type specified:
       event_type = specified_type
   else:
       event_type = InferEventType(staged_datums)

3. PARENTS BESTIMMEN
   
   if --parents specified:
       parents = specified_parents
   else:
       // Letzte bekannte Events im aktuellen Shard
       parents = GetLatestEvents(current_shard, max=3)

4. TRUST-SNAPSHOT BERECHNEN
   
   // Aktueller Quanten-Zustand
   actor_psi = GetQuantumState(my_did)
   
   // Novelty für dieses Event
   novelty = ComputeNovelty(staged_datums)
   
   // Expectation basierend auf Historie
   expectation = ComputeExpectation(event_type, my_history)
   
   // Erwarteter Trust-Impact
   current_W = ExpectedValue(actor_psi)  // ⟨Ψ|𝕎̂|Ψ⟩
   current_C = GetHistorySize(my_did)    // |ℂ|
   current_A = GetActivity(my_did)       // 𝔸
   
   // Nach Event
   new_C = current_C + 1
   new_A = current_A + ActivityBoost(event_type)
   
   // Delta berechnen
   old_contribution = current_A * sigma(current_W * ln(current_C) * novelty / expectation)
   new_contribution = new_A * sigma(current_W * ln(new_C) * novelty / expectation)
   delta_W = new_contribution - old_contribution

5. EVENT ERSTELLEN
   
   event = Event {
       header: {
           type:      event_type,
           actor:     my_did,
           parents:   parents,
           shard:     current_shard,
           realm:     current_realm,
           timestamp: LamportClock.increment(),
           nonce:     SecureRandom()
       },
       payload_hash: Hash(staged_datums),
       trust_snapshot: {
           actor_psi:   actor_psi,
           delta_W:     delta_W,
           novelty:     novelty,
           expectation: expectation
       }
   }

6. SIGNIEREN
   
   event_hash = Hash(event.header || event.payload_hash || event.trust_snapshot)
   event.signature = Sign(my_private_key, event_hash)

7. LOKAL SPEICHERN
   
   Store(~/.erynoa/events/, event)
   ClearStaged()

DETAILLIERTE AUSGABE
────────────────────
    Erynoa Commit
    ══════════════════════════════════════════════════════════════
    
    Event Construction
    ──────────────────
    Type:           TRANSFER
    Actor:          did:erynoa:personal:7xK9m2P4q8Yz
    Shard:          energy-trading
    Parents:        [event:sha3:p4r3nt1..., event:sha3:p4r3nt2...]
    
    Payload
    ───────
    Datums:         2
        datum:sha3:7k2m9x4p... (meter-reading.json, 1.2 KB)
        datum:sha3:8l3n0y5q... (payment-proof.json, 0.8 KB)
    
    Trust Calculation (Weltformel V5.0)
    ───────────────────────────────────
    
    Current State:
        |Ψ⟩ = 0.72|honest⟩ + 0.45|reliable⟩ + 0.35|neutral⟩ + ...
        𝕎 (expected) = 0.78
        𝔸 (activity) = 0.65
        |ℂ| (history) = 1,247 events
        ln|ℂ| = 7.13
    
    Event Metrics:
        ℕ (novelty):
            information_gain = 0.15 (new energy data)
            verification_rate = 0.92 (historical)
            ℕ = 0.6 × 0.15 + 0.4 × 0.92 = 0.458
        
        𝔼xp (expectation):
            predicted: regular monthly transfer
            actual: regular monthly transfer
            deviation = 0.05
            𝔼xp = 1 + 0.05 / 0.3 = 1.17
        
        Surprise factor: ℕ / 𝔼xp = 0.458 / 1.17 = 0.39
        (Lower than average - this is expected behavior)
    
    Contribution Calculation:
        Before: 0.65 × σ(0.78 × 7.13 × 0.39) = 0.65 × σ(2.17) = 0.65 × 0.90 = 0.585
        After:  0.68 × σ(0.78 × 7.13 × 0.39) = 0.68 × σ(2.17) = 0.68 × 0.90 = 0.612
        
        Δ𝔼 = +0.027
    
    Signature
    ─────────
    Algorithm:      ed25519
    Public Key:     ed25519:2Wj8kL...
    Signature:      sig:ed25519:9xNp7m...
    
    Event Created
    ─────────────
    ID:             event:sha3:3v3nth4sh...
    Status:         LOCAL (not yet propagated)
    
    Trust Impact (pending attestation):
        Expected Δ𝕎:  +0.02 (if successfully attested)
        New 𝕎:        0.80
        New Tier:     STABLE → TRUSTED (promotion pending!)
    
    Run 'erynoa push' to propagate to network.
    Run 'erynoa request-witness' to request attestations.
```

## 6. PUSH – Event mit Konsens propagieren

Die Propagation involviert Validatoren und die Erreichung von Konsens.

```
OPERATION: erynoa push
══════════════════════════════════════════════════════════════════════

SYNTAX
──────
erynoa push [OPTIONS]

OPTIONS
    --shard <SHARD>       Ziel-Shard (default: current)
    --priority <PRIO>     Priorität (low | normal | high)
    --wait                Auf Finalität warten

KONSENS-MECHANISMUS (Axiom E11-E15)
───────────────────────────────────
Wahrheit emergiert aus gewichtetem Konsens:

Konsens-Gewicht eines Validators v:
    weight(v) = 𝕎(v) · vigilance(v) · stake(v)

Konsens-Schwelle:
    Σ weight(v) ≥ θ_consensus    für alle v die zustimmen

θ_consensus ist shard-spezifisch:
    - High-security shards: θ = 0.80
    - Standard shards:      θ = 0.67
    - Low-stakes shards:    θ = 0.51

FINALITÄTS-STUFEN
─────────────────
    TENTATIVE:    Noch nicht genug Konsens (< 50%)
    SOFT_FINAL:   Mehrheit erreicht (50-80%)
    FINAL:        Volle Finalität (> 80%)
    IRREVERSIBLE: Im Merkle-Tree verankert

ALGORITHMUS
───────────
1. EVENT LADEN
   
   events_to_push = GetUnpushedEvents()
   
   if events_to_push.empty():
       INFO("Nothing to push. All events already propagated.")
       return

2. VALIDATOREN ENTDECKEN
   
   validators = DiscoverValidators(target_shard)
   
   // Sortiert nach Trust und Verfügbarkeit
   validators = SortByTrustAndLatency(validators)

3. PARALLEL PROPAGIEREN
   
   responses = ParallelSend(validators, events_to_push)

4. KONSENS AGGREGIEREN
   
   for event in events_to_push:
       accept_weight = 0
       reject_weight = 0
       
       for (validator, response) in responses:
           if response.accepted:
               accept_weight += weight(validator)
           else:
               reject_weight += weight(validator)
               // Log rejection reason
               LogRejection(event, validator, response.reason)
       
       total_weight = accept_weight + reject_weight
       consensus_ratio = accept_weight / total_weight
       
       if consensus_ratio >= θ_consensus:
           event.finality = FINAL
       elif consensus_ratio >= 0.5:
           event.finality = SOFT_FINAL
       else:
           event.finality = TENTATIVE

5. MERKLE-VERANKERUNG (für FINAL events)
   
   for event in events_to_push where event.finality == FINAL:
       merkle_proof = AwaitMerkleInclusion(event)
       event.merkle_root = merkle_proof.root
       event.finality = IRREVERSIBLE

DETAILLIERTE AUSGABE
────────────────────
    Erynoa Push
    ══════════════════════════════════════════════════════════════
    
    Target Shard:   energy-trading
    Events:         1
    
    Validator Discovery
    ───────────────────
    Active validators: 15
    
    Validator                           𝕎      Vigilance  Weight
    ─────────────────────────────────────────────────────────────
    did:erynoa:validator:alpha         0.95    0.98       0.186
    did:erynoa:validator:beta          0.92    0.95       0.175
    did:erynoa:validator:gamma         0.91    0.94       0.171
    did:erynoa:validator:delta         0.89    0.92       0.164
    did:erynoa:validator:epsilon       0.88    0.91       0.160
    ... (10 more)
    
    Total validator weight: 1.000 (normalized)
    Consensus threshold: 0.67 (standard shard)
    
    Propagation
    ───────────
    Event: event:sha3:3v3nth4sh...
    
    [████████████████████████████████████████] 100%
    
    Responses:
        ✓ alpha:   ACCEPTED  (weight: 0.186)
        ✓ beta:    ACCEPTED  (weight: 0.175)
        ✓ gamma:   ACCEPTED  (weight: 0.171)
        ✓ delta:   ACCEPTED  (weight: 0.164)
        ✓ epsilon: ACCEPTED  (weight: 0.160)
        ✓ zeta:    ACCEPTED  (weight: 0.144)
        ⏳ eta:    PENDING   (weight: 0.000)
        ... (8 more accepted)
    
    Consensus Analysis
    ──────────────────
    Accept weight:  0.92
    Reject weight:  0.00
    Pending:        0.08
    
    Consensus ratio: 0.92 / 0.92 = 100%
    Threshold: 0.67
    
    Status: FINAL ✓
    
    Merkle Anchoring
    ────────────────
    Waiting for Merkle inclusion...
    
    Block:          #1,247,892
    Merkle Root:    merkle:sha3:r00t...
    Proof Path:     [h1, h2, h3, h4, h5]
    Position:       42 / 128
    
    Verification:
        Computed root: merkle:sha3:r00t... ✓
        Matches block: Yes ✓
    
    Status: IRREVERSIBLE ✓
    
    Summary
    ───────
    Event:          event:sha3:3v3nth4sh...
    Finality:       IRREVERSIBLE
    Merkle Root:    merkle:sha3:r00t...
    Block:          #1,247,892
    
    Trust Update Applied:
        𝕎: 0.78 → 0.80 (+0.02)
        Tier: STABLE → TRUSTED ★
    
    Congratulations! You've been promoted to TRUSTED tier.
```

---

# TEIL V: TRANSAKTIONS-WORKFLOWS

## 7. SEEK – Quanten-basierte Partner-Suche

Die Suche verwendet die Interaktions-Wahrscheinlichkeit (Axiom Q5).

```
OPERATION: erynoa seek
══════════════════════════════════════════════════════════════════════

SYNTAX
──────
erynoa seek <QUERY> [OPTIONS]

OPTIONS
    --type <TYPE>         Gesuchter Partner-Typ
    --location <LOC>      Geografische Einschränkung
    --min-trust <T>       Minimaler Trust-Erwartungswert
    --max-results <N>     Maximale Ergebnisse
    --include-emerging    Auch FRESH/EMERGING Tiers einschließen

INTERAKTIONS-WAHRSCHEINLICHKEIT (Axiom Q5)
──────────────────────────────────────────
Die Wahrscheinlichkeit einer erfolgreichen Interaktion:

P(success | s₁, s₂, Ô) = |⟨Ψ_s₁|Ô|Ψ_s₂⟩|²

Wobei:
    |Ψ_s₁⟩ = Dein Quanten-Zustand
    |Ψ_s₂⟩ = Zustand des potentiellen Partners
    Ô = Interaktions-Operator (abhängig vom Transaktionstyp)

INTERAKTIONS-OPERATOREN
───────────────────────
Für verschiedene Transaktionstypen gibt es verschiedene Operatoren:

Ô_exchange (Gütertausch):
    Matrix in der Trust-Basis:
    ⎛ 1.0  0.8  0.5  0.2  0.0 ⎞
    ⎜ 0.8  0.9  0.6  0.3  0.1 ⎟
    ⎜ 0.5  0.6  0.7  0.4  0.2 ⎟
    ⎜ 0.2  0.3  0.4  0.5  0.3 ⎟
    ⎝ 0.0  0.1  0.2  0.3  0.4 ⎠
    
    Interpretation: honest×honest = 1.0 (perfekte Übereinstimmung)
                    honest×malicious = 0.0 (keine Chance)

Ô_service (Dienstleistung):
    Höhere Gewichtung für reliable und competent
    
Ô_governance (Governance):
    Höhere Gewichtung für honest und vigilant

RANKING-ALGORITHMUS
───────────────────
Für jeden Kandidaten c:

score(c) = relevance(c, query) 
         × P(success | me, c, Ô)
         × (1 + noise(c))           // Stochastic fairness (S3)
         × diversity_bonus(c)       // Anti-calcification (S2)

Wobei:
    relevance = semantische Ähnlichkeit zur Query
    noise = Uniform(-ξ, +ξ) mit ξ = 0.05
    diversity_bonus = 1.3 für FRESH/EMERGING, 1.0 sonst

ALGORITHMUS
───────────
1. QUERY EMBEDDEN
   
   query_embedding = Embed(query)

2. KANDIDATEN SAMMELN
   
   candidates = SearchIndex(current_shard, query_embedding)
   
   // Filter nach Constraints
   candidates = Filter(candidates, min_trust, location, type)

3. INTERAKTIONS-OPERATOR WÄHLEN
   
   O_hat = GetInteractionOperator(transaction_type)

4. FÜR JEDEN KANDIDATEN: SCORE BERECHNEN
   
   my_psi = GetQuantumState(my_did)
   
   for c in candidates:
       c_psi = GetQuantumState(c.did)
       
       // Transition amplitude
       amplitude = InnerProduct(my_psi, O_hat @ c_psi)
       
       // Success probability
       p_success = abs(amplitude) ** 2
       
       // Relevance (cosine similarity)
       relevance = CosineSimilarity(query_embedding, c.embedding)
       
       // Stochastic fairness
       noise = Uniform(-0.05, 0.05)
       
       // Diversity bonus
       if c.tier in [FRESH, EMERGING]:
           diversity = 1.3
       else:
           diversity = 1.0
       
       // Final score
       c.score = relevance * p_success * (1 + noise) * diversity

5. SORTIEREN UND DIVERSITY SLOTS RESERVIEREN
   
   // Top 70% nach Score
   top_candidates = SortByScore(candidates)[:int(0.7 * max_results)]
   
   // 30% Diversity Slots
   emerging_candidates = [c for c in candidates if c.tier in [FRESH, EMERGING]]
   diversity_slots = Sample(emerging_candidates, int(0.3 * max_results))
   
   // Kombinieren
   results = Interleave(top_candidates, diversity_slots)

DETAILLIERTE AUSGABE
────────────────────
    Erynoa Seek: "renewable energy supplier Berlin"
    ══════════════════════════════════════════════════════════════
    
    Search Parameters
    ─────────────────
    Query:          "renewable energy supplier Berlin"
    Type:           energy-supplier
    Location:       Berlin, 50km radius
    Min Trust:      𝕎 ≥ 0.60
    Shard:          energy-trading
    
    Your Quantum State
    ──────────────────
    |Ψ_you⟩ = 0.72|honest⟩ + 0.45|reliable⟩ + 0.35|neutral⟩ + 0.12|unreliable⟩
    𝕎_you = 0.78
    
    Interaction Operator: Ô_exchange
    
    Results (7 found, showing top 5)
    ─────────────────────────────────
    
    #1  GreenPower Berlin
        DID:        did:erynoa:biz:greenpower-berlin
        
        Quantum State:
        |Ψ_gp⟩ = 0.85|honest⟩ + 0.42|reliable⟩ + 0.25|neutral⟩ + ...
        𝕎 = 0.82
        Tier: TRUSTED
        
        Interaction Analysis:
        ⟨Ψ_you|Ô|Ψ_gp⟩ = 0.72×0.85×1.0 + 0.45×0.42×0.9 + ...
                       = 0.612 + 0.170 + ...
                       = 0.87
        P(success) = 0.87² = 0.76 (76%)
        
        Relevance:  0.94 (semantic match)
        Diversity:  1.0 (TRUSTED tier)
        Noise:      +0.02
        
        Final Score: 0.94 × 0.76 × 1.02 × 1.0 = 0.73
        
        ┌─────────────────────────────────────────────────────┐
        │ ████████████████████████████████████░░░░░ 76%      │
        │ Success Probability                                 │
        └─────────────────────────────────────────────────────┘
        
    ─────────────────────────────────────────────────────────────
    
    #2  SolarBerlin GmbH
        DID:        did:erynoa:biz:solarberlin
        
        Quantum State:
        |Ψ_sb⟩ = 0.68|honest⟩ + 0.55|reliable⟩ + 0.40|neutral⟩ + ...
        𝕎 = 0.75
        Tier: STABLE
        
        P(success) = 71%
        Relevance:  0.91
        
        Final Score: 0.70
        
    ─────────────────────────────────────────────────────────────
    
    #3* NewEnergy Startup (DIVERSITY SLOT)
        DID:        did:erynoa:biz:newenergy-startup
        
        Quantum State:
        |Ψ_ne⟩ = 0.45|honest⟩ + 0.35|reliable⟩ + 0.75|neutral⟩ + ...
        𝕎 = 0.58
        Tier: EMERGING ★
        
        P(success) = 52%
        Relevance:  0.88
        Diversity:  1.3 (EMERGING bonus!)
        
        Final Score: 0.88 × 0.52 × 1.03 × 1.3 = 0.61
        
        ⚠️ Higher risk, but contributing to network diversity
        
    ─────────────────────────────────────────────────────────────
    
    #4  WindKraft AG
        DID:        did:erynoa:biz:windkraft
        𝕎 = 0.79, Tier: TRUSTED
        P(success) = 73%, Final Score: 0.58
        
    #5* FreshPower (DIVERSITY SLOT)
        DID:        did:erynoa:biz:freshpower
        𝕎 = 0.51, Tier: FRESH ★
        P(success) = 45%, Final Score: 0.52
        
    ═══════════════════════════════════════════════════════════════
    
    Legend:
        * = Diversity Slot (S2: 30% reserved for emerging tiers)
        ★ = New entrant (eligible for exploration bonus S1)
    
    Actions:
        erynoa inspect <#>           Details anzeigen
        erynoa propose <#> ...       Angebot machen
        erynoa compare <#> <#>       Kandidaten vergleichen
```

## 8. PROPOSE – Angebot mit Erfolgswahrscheinlichkeit

```
OPERATION: erynoa propose
══════════════════════════════════════════════════════════════════════

SYNTAX
──────
erynoa propose <TARGET> [OPTIONS]

OPTIONS
    --amount <AMT>        Menge/Betrag
    --price <PRICE>       Preis
    --duration <DUR>      Laufzeit
    --streaming           Streaming-Transaktion
    --escrow <DID>        Escrow-Service

ERFOLGSWAHRSCHEINLICHKEIT VOR PROPOSAL
──────────────────────────────────────
Bevor das Proposal gesendet wird, berechnet das System die 
Erfolgswahrscheinlichkeit basierend auf der Quanten-Analyse:

P(accept | proposal) = P(success) × P(terms_acceptable)

P(success) = |⟨Ψ_me|Ô|Ψ_target⟩|²   (aus SEEK)

P(terms_acceptable) wird geschätzt aus:
    - Historische Akzeptanzrate des Targets
    - Ähnlichkeit zu akzeptierten Proposals
    - Marktkonditionen im Shard

SMART CONTRACT GENERIERUNG
──────────────────────────
Das System generiert automatisch einen Smart Contract mit Logic Guards:

contract = {
    parties:      [me, target],
    terms:        parsed_terms,
    streaming:    streaming_config,
    guards:       generated_logic_guards,
    settlement:   abort_settlement_rules
}

LOGIC GUARD GENERIERUNG (Axiom O5)
──────────────────────────────────
Logic Guards werden aus den Terms abgeleitet:

guard_delivery = """
    assert quantity_delivered >= quantity_promised * 0.95
    assert quality_metric >= quality_threshold
    assert delivery_time <= deadline + grace_period
"""

guard_payment = """
    assert payment_amount == agreed_price
    assert payment_time <= payment_deadline
"""

guard_abort = """
    // Axiom T7: Fair settlement on abort
    settlement_ratio = time_elapsed / total_duration
    refund_amount = total_price * (1 - settlement_ratio)
    delivered_value = quantity_delivered * unit_price
    
    if abort_by_buyer:
        seller_receives = delivered_value
        buyer_receives = refund_amount - delivered_value
    elif abort_by_seller:
        seller_receives = delivered_value * 0.9  // 10% penalty
        buyer_receives = refund_amount
"""

ALGORITHMUS
───────────
1. TERMS PARSEN
   
   terms = ParseTerms(amount, price, duration, streaming)

2. TARGET-ZUSTAND HOLEN
   
   target_psi = GetQuantumState(target_did)
   target_history = GetHistory(target_did)

3. ERFOLGSWAHRSCHEINLICHKEIT BERECHNEN
   
   p_success = ComputeSuccessProbability(my_psi, target_psi, O_exchange)
   p_terms = EstimateTermsAcceptance(terms, target_history)
   p_accept = p_success * p_terms

4. SMART CONTRACT GENERIEREN
   
   contract = GenerateContract(terms, streaming)
   guards = GenerateLogicGuards(contract)
   
   // Validate guards (Axiom P1: Hoare-Triple)
   for guard in guards:
       assert ValidateHoareTriple(guard.pre, guard.inv, guard.post)

5. PROPOSAL EVENT ERSTELLEN
   
   proposal_datum = Datum {
       content: Serialize(contract),
       schema: "contract:proposal:v2"
   }
   
   proposal_event = Event {
       type: PROPOSAL,
       actor: my_did,
       payload: proposal_datum.id,
       metadata: {
           target: target_did,
           p_accept: p_accept,
           expires: now() + 7 days
       }
   }

6. SIGNIEREN UND SENDEN
   
   Sign(proposal_event)
   Send(target_did, proposal_event)

DETAILLIERTE AUSGABE
────────────────────
    Erynoa Propose
    ══════════════════════════════════════════════════════════════
    
    From:           did:erynoa:personal:7xK9m2P4q8Yz (you)
    To:             did:erynoa:biz:greenpower-berlin
    
    Terms
    ─────
    Asset:          Renewable Energy
    Quantity:       500 kWh
    Price:          125 EUR (0.25 EUR/kWh)
    Duration:       30 days
    Mode:           Streaming
    
    Streaming Configuration
    ───────────────────────
    Rate:           16.67 kWh/day
    Payment Rate:   4.17 EUR/day
    Settlement:     Daily reconciliation
    
    Success Analysis (Quantum)
    ──────────────────────────
    Your state:     |Ψ_you⟩, 𝕎 = 0.78
    Target state:   |Ψ_gp⟩, 𝕎 = 0.82
    
    Interaction Analysis:
        |⟨Ψ_you|Ô_exchange|Ψ_gp⟩|² = 0.76
        
        Breakdown:
        ┌───────────────────────────────────────────────────────┐
        │ Your State    ×    Operator    ×    Their State      │
        │                                                       │
        │ honest:0.72   ×    1.0         ×    honest:0.85      │
        │ = 0.612                                               │
        │                                                       │
        │ reliable:0.45 ×    0.9         ×    reliable:0.42    │
        │ = 0.170                                               │
        │                                                       │
        │ (cross terms...)                                      │
        │                                                       │
        │ Total amplitude: 0.87                                 │
        │ P(success) = 0.87² = 0.76                            │
        └───────────────────────────────────────────────────────┘
    
    Terms Acceptance Estimate:
        Historical acceptance rate: 78%
        Price comparison to market: -3% (competitive)
        Duration preference match: 85%
        
        P(terms acceptable): 0.82
    
    Combined: P(accept) = 0.76 × 0.82 = 0.62 (62%)
    
    Smart Contract Generated
    ────────────────────────
    Contract ID:    contract:sha3:c0ntr4ct...
    
    Logic Guards:
    
    GUARD: Delivery (Axiom P1, O5)
    ┌─────────────────────────────────────────────────────────┐
    │ PRE:  seller.balance >= 500 kWh                        │
    │ INV:  daily_delivery >= 15.84 kWh (95% of rate)        │
    │ POST: buyer.received >= 475 kWh (95% of total)         │
    └─────────────────────────────────────────────────────────┘
    
    GUARD: Payment (Axiom P1, A24)
    ┌─────────────────────────────────────────────────────────┐
    │ PRE:  buyer.balance >= 125 EUR                         │
    │ INV:  daily_payment = delivered_today × 0.25 EUR       │
    │ POST: seller.received = total_delivered × 0.25 EUR     │
    └─────────────────────────────────────────────────────────┘
    
    GUARD: Abort Settlement (Axiom T7)
    ┌─────────────────────────────────────────────────────────┐
    │ ON ABORT:                                               │
    │   settlement = delivered / promised                     │
    │   if abort_by_buyer:                                   │
    │     seller.keep = delivered × 0.25 EUR                 │
    │     buyer.refund = (125 - seller.keep) EUR             │
    │   if abort_by_seller:                                  │
    │     seller.penalty = 10%                               │
    │     seller.keep = delivered × 0.25 × 0.9 EUR           │
    └─────────────────────────────────────────────────────────┘
    
    Event
    ─────
    Type:           PROPOSAL
    ID:             event:sha3:pr0p0s4l...
    Expires:        2026-02-05 14:30:00 UTC (7 days)
    
    Sending proposal...
    
    ✓ Delivered to did:erynoa:biz:greenpower-berlin
    ✓ Stored locally for tracking
    
    Status: PENDING_RESPONSE
    
    Track with: erynoa proposals
```

## 9. STREAM – Kontinuierliche Transaktion mit Live-Berechnungen

```
OPERATION: erynoa stream status
══════════════════════════════════════════════════════════════════════

SYNTAX
──────
erynoa stream status <CONTRACT>
erynoa stream pause <CONTRACT>
erynoa stream resume <CONTRACT>
erynoa stream abort <CONTRACT> --reason <REASON>

STREAMING-MODELL (Axiom T4)
───────────────────────────
Streaming ermöglicht kontinuierliche, proportionale Wertübertragung:

    value_transferred(t) = rate × elapsed_time(t)
    
    Wobei:
        rate = total_value / total_duration
        elapsed_time(t) = min(t - start_time, total_duration)

TRUST-EVOLUTION WÄHREND STREAMING
─────────────────────────────────
Der Trust beider Parteien evoliert kontinuierlich:

Für Seller (bei guter Performance):
    Δ𝕎_seller(t) = base_gain × performance_ratio(t) × time_weight(t)
    
    performance_ratio(t) = actual_delivered(t) / expected_delivered(t)
    time_weight(t) = log(1 + t) / log(1 + total_duration)

Für Buyer (bei pünktlicher Zahlung):
    Δ𝕎_buyer(t) = base_gain × payment_punctuality(t)

QUANTEN-ZUSTAND EVOLUTION
─────────────────────────
Während des Streamings kollabiert der Quanten-Zustand schrittweise:

Vor Streaming:  |Ψ⟩ = α|honest⟩ + β|reliable⟩ + γ|neutral⟩ + ...
Nach Tick n:    |Ψ'⟩ = α'|honest⟩ + β'|reliable⟩ + γ'|neutral⟩ + ...

Die Amplituden-Update-Regel:
    α' = α × (1 + ε × performance)  wenn performance > 0
    α' = α × (1 - ε × |performance|) wenn performance < 0
    
    Renormierung: |Ψ'⟩ = |Ψ'⟩ / ‖|Ψ'⟩‖

DETAILLIERTE AUSGABE
────────────────────
    Erynoa Stream Status
    ══════════════════════════════════════════════════════════════
    
    Contract:       contract:sha3:c0ntr4ct...
    Type:           Energy Streaming
    Phase:          STREAMING (Day 12 of 30)
    
    ═══════════════════════════════════════════════════════════════
    DELIVERY METRICS
    ═══════════════════════════════════════════════════════════════
    
    Progress:
    ┌──────────────────────────────────────────────────────────────┐
    │ ████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  40%     │
    │                                                              │
    │ Delivered: 205.3 kWh / 500 kWh                              │
    │ Expected:  200.0 kWh (at this point)                        │
    │ Variance:  +2.65% (over-delivery)                           │
    └──────────────────────────────────────────────────────────────┘
    
    Daily Breakdown (last 7 days):
    
    Day     Delivered    Expected    Variance    Status
    ────────────────────────────────────────────────────
    6       17.2 kWh     16.67       +3.2%       ✓
    7       16.5 kWh     16.67       -1.0%       ✓
    8       18.1 kWh     16.67       +8.6%       ✓✓
    9       16.8 kWh     16.67       +0.8%       ✓
    10      15.9 kWh     16.67       -4.6%       ⚠
    11      17.0 kWh     16.67       +2.0%       ✓
    12      16.9 kWh     16.67       +1.4%       ✓
    
    Average daily: 16.91 kWh (+1.4% vs expected)
    
    ═══════════════════════════════════════════════════════════════
    PAYMENT METRICS
    ═══════════════════════════════════════════════════════════════
    
    Progress:
    ┌──────────────────────────────────────────────────────────────┐
    │ ████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  40%     │
    │                                                              │
    │ Paid:      50.00 EUR / 125 EUR                              │
    │ For:       200.0 kWh delivered                              │
    │ Rate:      0.25 EUR/kWh (as agreed)                         │
    └──────────────────────────────────────────────────────────────┘
    
    Outstanding:
        Delivered but unpaid: 5.3 kWh × 0.25 = 1.33 EUR
        (Will be settled in next tick)
    
    ═══════════════════════════════════════════════════════════════
    TRUST EVOLUTION (Weltformel V5.0)
    ═══════════════════════════════════════════════════════════════
    
    YOUR TRUST EVOLUTION
    ────────────────────
    Start:      𝕎 = 0.78, |Ψ⟩ = 0.72|honest⟩ + ...
    Current:    𝕎 = 0.79, |Ψ⟩ = 0.74|honest⟩ + ...
    
    Quantum State Trajectory:
    
    Day │ |honest⟩  |reliable⟩  |neutral⟩  𝕎
    ────┼────────────────────────────────────────
    0   │ 0.720     0.450       0.350      0.780
    3   │ 0.725     0.455       0.345      0.783
    6   │ 0.730     0.458       0.340      0.786
    9   │ 0.735     0.460       0.335      0.789
    12  │ 0.740     0.462       0.330      0.792
    
    Trend: ↑ Improving (payments on time)
    
    COUNTERPARTY TRUST EVOLUTION
    ────────────────────────────
    Start:      𝕎 = 0.82, |Ψ⟩ = 0.85|honest⟩ + ...
    Current:    𝕎 = 0.83, |Ψ⟩ = 0.86|honest⟩ + ...
    
    Performance bonus: +1.4% over-delivery → faster trust gain
    
    ═══════════════════════════════════════════════════════════════
    ABORT SCENARIO ANALYSIS (Axiom T7)
    ═══════════════════════════════════════════════════════════════
    
    If aborted NOW:
    
    Scenario A: You abort
    ┌─────────────────────────────────────────────────────────────┐
    │ Settlement:                                                 │
    │   Delivered:     205.3 kWh                                  │
    │   Paid:          50.00 EUR                                  │
    │   Fair value:    51.33 EUR (205.3 × 0.25)                  │
    │                                                             │
    │   You owe:       1.33 EUR (for delivered-but-unpaid)       │
    │   Seller keeps:  51.33 EUR                                  │
    │   You get back:  73.67 EUR (125 - 51.33)                   │
    │                                                             │
    │   Trust impact (you):    -0.03 (abort penalty)              │
    │   Trust impact (seller): +0.02 (successful partial)         │
    └─────────────────────────────────────────────────────────────┘
    
    Scenario B: Seller aborts
    ┌─────────────────────────────────────────────────────────────┐
    │ Settlement:                                                 │
    │   Delivered:     205.3 kWh                                  │
    │   Fair value:    51.33 EUR                                  │
    │   Penalty (10%): 5.13 EUR                                   │
    │                                                             │
    │   Seller gets:   46.20 EUR (51.33 - 5.13)                  │
    │   You get back:  78.80 EUR                                  │
    │                                                             │
    │   Trust impact (seller): -0.05 (abort + penalty)            │
    │   Trust impact (you):    +0.01 (victim of abort)            │
    └─────────────────────────────────────────────────────────────┘
    
    ═══════════════════════════════════════════════════════════════
    PROJECTION TO COMPLETION
    ═══════════════════════════════════════════════════════════════
    
    If current trend continues:
    
    Projected final delivery:  507.5 kWh (+1.5%)
    Projected final payment:   125.00 EUR (exact)
    
    Projected trust changes:
        Your 𝕎:    0.78 → 0.82 (+0.04)
        Their 𝕎:   0.82 → 0.85 (+0.03)
    
    ═══════════════════════════════════════════════════════════════
    
    Actions:
        erynoa stream pause   - Pause streaming (requires mutual consent)
        erynoa stream abort   - Abort with settlement
        erynoa stream extend  - Extend duration (requires negotiation)
    
    Next tick in: 23:45:12
```

---

# TEIL VI: CROSS-SHARD OPERATIONEN

## 10. MERGE – Kategorietheoretische Cross-Shard-Transaktion

```
OPERATION: erynoa merge
══════════════════════════════════════════════════════════════════════

SYNTAX
──────
erynoa merge --from <SHARD> --to <SHARD> [OPTIONS]

OPTIONS
    --amount <AMT>        Zu übertragende Menge
    --asset <ASSET>       Spezifisches Asset
    --functor <F>         Spezifischer Funktor (auto-detect if omitted)

FUNKTOR-THEORIE (Axiom Q7)
──────────────────────────
Ein Funktor F : 𝒞_source → 𝒞_target muss folgende Eigenschaften erfüllen:

1. Objekt-Abbildung:
   F(s) ∈ Ob(𝒞_target) für alle s ∈ Ob(𝒞_source)

2. Morphismus-Abbildung:
   F(tx : s₁ → s₂) : F(s₁) → F(s₂)

3. Identitäts-Erhaltung:
   F(id_s) = id_{F(s)}

4. Kompositions-Erhaltung:
   F(tx₂ ∘ tx₁) = F(tx₂) ∘ F(tx₁)

SEMANTISCHE INTEROPERABILITÄT (Axiom Q10)
─────────────────────────────────────────
Zwei Shards sind interoperabel wenn:

Vollständig: F ∘ G ≅ Id ∧ G ∘ F ≅ Id
Teilweise:   F ⊣ G (Adjunktion)
Minimal:     ∃F : 𝒞₁ → 𝒞₂

ALGORITHMUS
───────────
1. FUNKTOR FINDEN
   
   available_functors = FindFunctors(source_shard, target_shard)
   
   if --functor specified:
       functor = GetFunctor(specified_functor)
   else:
       // Wähle besten Funktor nach Struktur-Erhaltung
       functor = SelectBestFunctor(available_functors)

2. STRUKTUR-PRÜFUNG
   
   // Prüfe ob die Transaktion strukturerhaltend abgebildet werden kann
   source_structure = GetTransactionStructure(amount, asset, source_shard)
   target_structure = functor.map(source_structure)
   
   assert IsValidInCategory(target_structure, target_shard)

3. KONVERSIONS-BERECHNUNG
   
   // Der Funktor definiert die Konversion
   converted = functor.convert(amount, asset)
   
   // Konversionsrate aus Funktor-Definition
   rate = functor.rate(asset)

4. ZWEI-PHASEN-COMMIT
   
   // Phase 1: Prepare
   prepare_source = PrepareDebit(source_shard, amount, asset)
   prepare_target = PrepareCredit(target_shard, converted.amount, converted.asset)
   
   if prepare_source.ok AND prepare_target.ok:
       // Phase 2: Commit
       commit_source = CommitDebit(prepare_source)
       commit_target = CommitCredit(prepare_target)
       
       if commit_source.ok AND commit_target.ok:
           SUCCESS
       else:
           Rollback(prepare_source, prepare_target)
   else:
       Abort(prepare_source, prepare_target)

5. MERGE EVENT ERSTELLEN
   
   merge_event = Event {
       type: CROSS_SHARD_TRANSFER,
       actor: my_did,
       payload: {
           source_shard,
           target_shard,
           functor: functor.id,
           source_asset: (amount, asset),
           target_asset: (converted.amount, converted.asset),
           rate,
           proof: merkle_proof_of_both_commits
       }
   }

DETAILLIERTE AUSGABE
────────────────────
    Erynoa Merge: gaming → finance
    ══════════════════════════════════════════════════════════════
    
    Cross-Shard Transfer
    ────────────────────
    Source Shard:   gaming
    Target Shard:   finance
    Your DID:       did:erynoa:personal:7xK9m2P4q8Yz
    
    ═══════════════════════════════════════════════════════════════
    FUNKTOR ANALYSIS (Axiom Q7)
    ═══════════════════════════════════════════════════════════════
    
    Available Functors: 3
    
    #1  F_reputation_to_credit
        Type:           PARTIAL (Adjunction)
        Structure Loss: 12%
        Rate:           100 game_rep → 1 credit_unit
        
    #2  F_achievement_to_credential (Selected ✓)
        Type:           FULL (Equivalence)
        Structure Loss: 0%
        Preserves:      All transaction structure
        
    #3  F_token_to_token
        Type:           MINIMAL
        Structure Loss: 45%
        Rate:           Fixed 1000:1
    
    Selected: F_achievement_to_credential
    Reason:   Full structure preservation, zero loss
    
    Functor Definition:
    ┌─────────────────────────────────────────────────────────────┐
    │ F : 𝒞_gaming → 𝒞_finance                                    │
    │                                                             │
    │ Object Mapping:                                             │
    │   F(Player) = CreditEntity                                  │
    │   F(Achievement) = FinancialCredential                      │
    │   F(GameToken) = CreditUnit                                 │
    │                                                             │
    │ Morphism Mapping:                                           │
    │   F(earn_achievement) = issue_credential                    │
    │   F(trade_token) = transfer_credit                          │
    │   F(guild_transaction) = corporate_transaction              │
    │                                                             │
    │ Rate Function:                                              │
    │   F.rate(GameToken) = 0.01 CreditUnit                       │
    │   F.rate(Achievement) = credential_level × 10 CreditUnit    │
    └─────────────────────────────────────────────────────────────┘
    
    Structure Preservation Proof:
    
    Identity:   F(id_player) = id_{F(player)} = id_credit_entity ✓
    
    Composition:
        tx₁: earn_gold : player → player
        tx₂: buy_item : player → player
        
        F(tx₂ ∘ tx₁) = F(tx₂) ∘ F(tx₁)
                     = transfer_credit ∘ issue_credit
                     ✓ (verified)
    
    ═══════════════════════════════════════════════════════════════
    CONVERSION
    ═══════════════════════════════════════════════════════════════
    
    Source (gaming):
        Asset:      500 GameTokens
        Type:       game:token:gold
        Your Balance: 12,450 GameTokens
    
    Conversion via Functor:
        F(500 GameTokens) = 500 × 0.01 = 5 CreditUnits
    
    Target (finance):
        Asset:      5 CreditUnits
        Type:       finance:credit:standard
        Your Balance: 0 CreditUnits (new in this shard)
    
    ═══════════════════════════════════════════════════════════════
    TWO-PHASE COMMIT (Atomic Cross-Shard)
    ═══════════════════════════════════════════════════════════════
    
    Phase 1: PREPARE
    ────────────────
    
    Gaming Shard:
        Lock:       500 GameTokens
        Validator:  did:erynoa:validator:gaming-1
        Prepare ID: prepare:sha3:g4m1ng...
        Status:     PREPARED ✓
    
    Finance Shard:
        Reserve:    5 CreditUnits (new mint authorized by functor)
        Validator:  did:erynoa:validator:finance-1
        Prepare ID: prepare:sha3:f1n4nc3...
        Status:     PREPARED ✓
    
    Both shards prepared. Proceeding to commit...
    
    Phase 2: COMMIT
    ───────────────
    
    Gaming Shard:
        Action:     Debit 500 GameTokens
        Event:      event:sha3:g4m3d3b1t...
        Finality:   FINAL ✓
    
    Finance Shard:
        Action:     Credit 5 CreditUnits
        Event:      event:sha3:f1ncr3d1t...
        Finality:   FINAL ✓
    
    ═══════════════════════════════════════════════════════════════
    TRUST UPDATE
    ═══════════════════════════════════════════════════════════════
    
    Your Trust in Gaming:
        Before:     𝕎 = 0.75
        After:      𝕎 = 0.75 (unchanged, normal operation)
    
    Your Trust in Finance:
        Before:     𝕎 = 0.49 (FRESH, no history)
        After:      𝕎 = 0.52 (initial activity boost)
        
        Note: Your gaming trust partially propagates via functor:
              𝕎_finance += 0.1 × 𝕎_gaming × functor_trust_factor
                        += 0.1 × 0.75 × 0.4
                        += 0.03
    
    ═══════════════════════════════════════════════════════════════
    SUMMARY
    ═══════════════════════════════════════════════════════════════
    
    Cross-Shard Transfer Complete
    
    Gaming:
        - 500 GameTokens
        Balance: 11,950 GameTokens
    
    Finance:
        + 5 CreditUnits
        Balance: 5 CreditUnits
    
    Merge Event:    event:sha3:m3rg3...
    Functor Used:   F_achievement_to_credential
    Structure Loss: 0%
    
    Merkle Proofs:
        Gaming:     merkle:sha3:g4m3r00t...
        Finance:    merkle:sha3:f1nr00t...
        Combined:   merkle:sha3:cr0ssr00t...
```

---

# TEIL VII: SYSTEM-OPERATIONEN

## 11. STATUS – Vollständiger Weltformel-Zustand

```
OPERATION: erynoa status
══════════════════════════════════════════════════════════════════════

DETAILLIERTE AUSGABE
────────────────────
    Erynoa Status
    ══════════════════════════════════════════════════════════════
    
    Identity
    ────────
    DID:            did:erynoa:personal:7xK9m2P4q8Yz
    Namespace:      personal
    Created:        2025-06-15 (228 days ago)
    
    Sub-Identities:
        did:erynoa:personal:7xK9m2P4q8Yz:gaming    (𝕎=0.75, entangled)
        did:erynoa:personal:7xK9m2P4q8Yz:work      (𝕎=0.82, entangled)
    
    Current Context
    ───────────────
    Shard:          energy-trading
    Realm:          did:erynoa:realm:energy
    Category:       𝒞_energy
    
    ═══════════════════════════════════════════════════════════════
    WELTFORMEL KOMPONENTEN
    ═══════════════════════════════════════════════════════════════
    
    Your contribution to 𝔼:
    
    𝔼_you = ⟨Ψ| 𝔸̂ · σ̂( 𝕎̂ · ln|ℂ̂| · ℕ̂ / 𝔼x̂p ) |Ψ⟩
    
    ┌─────────────────────────────────────────────────────────────┐
    │ QUANTUM STATE |Ψ⟩ (Axiom Q1)                                │
    ├─────────────────────────────────────────────────────────────┤
    │                                                             │
    │ |Ψ⟩ = 0.74|honest⟩ + 0.46|reliable⟩ + 0.32|neutral⟩        │
    │     + 0.10|unreliable⟩ + 0.03|malicious⟩                   │
    │                                                             │
    │ Visualization:                                              │
    │                                                             │
    │ honest     ████████████████████████████████████░░░  74%    │
    │ reliable   ██████████████████████░░░░░░░░░░░░░░░░░  46%    │
    │ neutral    ███████████████░░░░░░░░░░░░░░░░░░░░░░░░  32%    │
    │ unreliable █████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  10%    │
    │ malicious  █░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   3%    │
    │                                                             │
    │ Normierung: |0.74|² + |0.46|² + |0.32|² + |0.10|² + |0.03|² │
    │           = 0.548 + 0.212 + 0.102 + 0.010 + 0.001           │
    │           = 0.873 (nach Renormierung = 1.0)                 │
    │                                                             │
    │ Entanglements:                                              │
    │   ↔ gaming identity (correlation: 0.7)                      │
    │   ↔ work identity (correlation: 0.8)                        │
    │                                                             │
    └─────────────────────────────────────────────────────────────┘
    
    ┌─────────────────────────────────────────────────────────────┐
    │ WÄCHTER-OPERATOR 𝕎̂ (6-dimensional)                         │
    ├─────────────────────────────────────────────────────────────┤
    │                                                             │
    │ Dimension        Value    Weight   Contribution             │
    │ ────────────────────────────────────────────────            │
    │ Reliability (R)  0.85     0.15     0.128                    │
    │ Integrity (I)    0.82     0.15     0.123                    │
    │ Competence (C)   0.79     0.15     0.119                    │
    │ Predictability(P)0.88     0.10     0.088                    │
    │ Vigilance (V)    0.76     0.25     0.190                    │
    │ Ω-Alignment (Ω)  0.81     0.20     0.162                    │
    │ ────────────────────────────────────────────────            │
    │ 𝕎 = ⟨Ψ|𝕎̂|Ψ⟩                        = 0.810                │
    │                                                             │
    │ Trust Floor (Axiom A7): 0.30                                │
    │ Your 𝕎 is well above floor ✓                                │
    │                                                             │
    │ Tier: TRUSTED (𝕎 ∈ [0.75, 0.90))                           │
    │                                                             │
    └─────────────────────────────────────────────────────────────┘
    
    ┌─────────────────────────────────────────────────────────────┐
    │ AKTIVITÄT 𝔸 (Axiom E1-E4)                                   │
    ├─────────────────────────────────────────────────────────────┤
    │                                                             │
    │ 𝔸 = Σ weight(event_type) × recency(event) / τ              │
    │                                                             │
    │ Time window τ: 30 days                                      │
    │                                                             │
    │ Event Type        Count    Weight   Contribution            │
    │ ────────────────────────────────────────────────            │
    │ TRANSFER          8        1.0      0.267                   │
    │ ATTEST            23       0.5      0.383                   │
    │ CLAIM             5        0.3      0.050                   │
    │ GOVERNANCE        2        0.8      0.053                   │
    │ ────────────────────────────────────────────────            │
    │ Raw Activity:                       0.753                   │
    │ Recency Decay Applied:              0.68                    │
    │                                                             │
    │ 𝔸 = 0.68                                                    │
    │                                                             │
    │ Activity Trend (7d): ↑ +0.05 (increasing)                   │
    │                                                             │
    └─────────────────────────────────────────────────────────────┘
    
    ┌─────────────────────────────────────────────────────────────┐
    │ GESCHICHTE |ℂ| (Axiom A12-A17)                              │
    ├─────────────────────────────────────────────────────────────┤
    │                                                             │
    │ Total Events:     1,892                                     │
    │ Attested Events:  1,847 (97.6%)                            │
    │ Finalized:        1,847                                     │
    │ Pending:          45                                        │
    │                                                             │
    │ |ℂ| = 1,847 (only attested events count)                   │
    │ ln|ℂ| = 7.52                                                │
    │                                                             │
    │ History Growth:                                             │
    │ ┌───────────────────────────────────────────────────┐       │
    │ │     ╭─────────────────────────────────╮           │       │
    │ │    ╱                                               │       │
    │ │   ╱                                                │       │
    │ │  ╱                                                 │       │
    │ │ ╱                                                  │       │
    │ │╱                                                   │       │
    │ └───────────────────────────────────────────────────┘       │
    │   Jun    Aug    Oct    Dec    Feb                           │
    │                                                             │
    │ Note: ln|ℂ| grows logarithmically - early history          │
    │       is disproportionately valuable (Axiom A15)            │
    │                                                             │
    └─────────────────────────────────────────────────────────────┘
    
    ┌─────────────────────────────────────────────────────────────┐
    │ NOVELTY ℕ (Axiom K1)                                        │
    ├─────────────────────────────────────────────────────────────┤
    │                                                             │
    │ ℕ = α × information_gain + β × verification_rate            │
    │   = 0.6 × 0.42 + 0.4 × 0.91                                │
    │   = 0.252 + 0.364                                          │
    │   = 0.616                                                   │
    │                                                             │
    │ Information Gain (your recent contributions):               │
    │   - New energy supplier data: +0.15                         │
    │   - Market price insights: +0.12                            │
    │   - Regulatory compliance info: +0.08                       │
    │   - Routine transactions: +0.07                             │
    │   Total: 0.42                                               │
    │                                                             │
    │ Verification Rate (how often your novel claims verified):   │
    │   Verified novel claims: 47                                 │
    │   Total novel claims: 52                                    │
    │   Rate: 90.4%                                               │
    │                                                             │
    │ ℕ = 0.616 (above average: 0.5)                             │
    │ You are contributing novel, verified information ✓          │
    │                                                             │
    └─────────────────────────────────────────────────────────────┘
    
    ┌─────────────────────────────────────────────────────────────┐
    │ EXPECTATION 𝔼xp (Axiom K2)                                  │
    ├─────────────────────────────────────────────────────────────┤
    │                                                             │
    │ 𝔼xp = 1 + |predicted - actual| / σ_baseline                 │
    │                                                             │
    │ Your predicted behavior (based on history):                 │
    │   - ~2 transfers/week                                       │
    │   - ~6 attestations/week                                    │
    │   - Focus on energy trading                                 │
    │   - Regular, consistent patterns                            │
    │                                                             │
    │ Your actual behavior (last 30 days):                        │
    │   - 2.1 transfers/week (close to predicted)                 │
    │   - 5.8 attestations/week (close)                           │
    │   - 95% energy trading (as expected)                        │
    │                                                             │
    │ Deviation: |predicted - actual| = 0.08                      │
    │ σ_baseline = 0.3                                            │
    │                                                             │
    │ 𝔼xp = 1 + 0.08 / 0.3 = 1.27                                │
    │                                                             │
    │ Interpretation: You are somewhat predictable (expected).    │
    │ This is neither good nor bad - consistency is valued.       │
    │                                                             │
    └─────────────────────────────────────────────────────────────┘
    
    ┌─────────────────────────────────────────────────────────────┐
    │ SURPRISE FACTOR ℕ/𝔼xp                                       │
    ├─────────────────────────────────────────────────────────────┤
    │                                                             │
    │ Surprise = ℕ / 𝔼xp = 0.616 / 1.27 = 0.485                  │
    │                                                             │
    │ Interpretation:                                             │
    │   < 0.5: Somewhat predictable, lower attention boost        │
    │   = 0.5: Average surprise                                   │
    │   > 1.0: Highly surprising, significant attention boost     │
    │   > 2.0: Exceptional novelty (rare)                         │
    │                                                             │
    │ Your surprise factor is near average.                       │
    │ To increase: Contribute more novel, verified information.   │
    │                                                             │
    └─────────────────────────────────────────────────────────────┘
    
    ═══════════════════════════════════════════════════════════════
    FINAL CALCULATION
    ═══════════════════════════════════════════════════════════════
    
    𝔼_you = 𝔸 × σ( 𝕎 × ln|ℂ| × ℕ / 𝔼xp )
    
    Step by step:
        𝕎 × ln|ℂ| = 0.810 × 7.52 = 6.09
        × ℕ / 𝔼xp  = 6.09 × 0.485 = 2.95
        σ(2.95)    = 1 / (1 + e^(-2.95)) = 0.950
        × 𝔸        = 0.950 × 0.68 = 0.646
    
    ╔═════════════════════════════════════════════════════════════╗
    ║                                                             ║
    ║  Your Contribution to System Intelligence:                  ║
    ║                                                             ║
    ║              𝔼_you = 0.646                                  ║
    ║                                                             ║
    ║  This places you in the top 15% of contributors             ║
    ║  in the energy-trading shard.                               ║
    ║                                                             ║
    ╚═════════════════════════════════════════════════════════════╝
    
    ═══════════════════════════════════════════════════════════════
    PENDING ITEMS
    ═══════════════════════════════════════════════════════════════
    
    Staged:
        2 datums ready to commit
        
    Pending Events:
        45 events awaiting final attestation
        
    Active Streams:
        1 streaming transaction (day 12/30)
        
    Proposals:
        0 sent, awaiting response
        2 received, awaiting your decision
```

---

## Anhang: Axiom-Referenz

| Ebene | Axiome | Protokoll-Relevanz |
|-------|--------|-------------------|
| 0. Fundament | A1-A30 | INIT, REVOKE, alle Events |
| 1. Emergenz | E1-E15 | PUSH, WITNESS, Konsens |
| 2. Prozess | P1-P6, T1-T7 | COMMIT, STREAM, CLOSE, ABORT |
| 3. Objekt | O1-O5, C1-C4 | ADD, MINT, CREDENTIAL |
| 4. Schutz | S1-S18 | SEEK (diversity), GOVERNANCE |
| 5. Kybernetik | K1-K16 | STATUS (ℕ, 𝔼xp), System-Atmung |
| 6. Quanta | Q1-Q15 | SEEK (Q5), MERGE (Q7), ADD (Q13) |
| 7. Humanismus | H1-H4 | HUMAN-AUTH, AMNESTY, LOD, BLUEPRINT (NLD) |

---

## Anhang: V6.0 Humanismus-Befehle

### HUMAN-AUTH (H1)
```
erynoa human-auth verify <credential-id>     # Prüft HumanAuth Credential
erynoa human-auth request --method=video     # Fordert neue Verifizierung an
erynoa human-auth quota                      # Zeigt Human-Interaktions-Quote
```

### AMNESTY (H3)
```
erynoa amnesty status                        # Zeigt Amnestie-Status
erynoa amnesty apply --automatic             # Beantragt automatische Amnestie (nach 7y)
erynoa governance amnesty <did> --reason=".."# Governance-Amnestie Antrag
```

### LOD (H2)
```
erynoa lod compute <tx-value>                # Berechnet empfohlenes LoD-Level
erynoa green-score                           # Zeigt Green-Trust-Score (Effizienz)
erynoa config set lod.auto=true              # Aktiviert automatische LoD-Wahl
```

### BLUEPRINT (H4)
```
erynoa blueprint validate <id>               # Prüft semantische Verankerung
erynoa blueprint nld <id>                    # Zeigt Natural Language Description
erynoa blueprint equivalence-check <id>      # LLM-Äquivalenz-Prüfung
```

---

*Erynoa Protocol Specification V6.0*
*Weltformel-integriertes Protokoll für vertrauensbasierte Interaktionen*
*120 Axiome • 8 Ebenen • Quanten-Trust • Kategorie-Brücken • Human-Aligned*
*"Das System existiert, um menschliches Gedeihen zu ermöglichen."*
