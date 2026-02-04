// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                          PLUTO-DNA v1.0                                    ║
// ║      Komprimiertes formales Modell des Erynoa Nervensystems               ║
// ║          476 KB Dokumentation → 2 KB Algebra-Kern                         ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

// ═══════════════════════════════════════════════════════════════════════════
// ENTITÄTEN (E)
// ═══════════════════════════════════════════════════════════════════════════

ENTITY ι(Identity) {
  did: H256,                    // Decentralized Identifier (BLAKE3)
  type: Self | Spirit | Guild,  // Selbst | Agent | Organisation
  τ⃗: [0,1]⁶,                    // TrustVector6D
  age: u64,                      // Novelty-Timestamp
  wallets: Map<Chain, Address>   // Multi-Chain-Wallets
}

ENTITY ρ(Realm) {
  id: H256,
  type: Root | Virtual(ρ) | Partition(ρ),
  rules: Set<Rule>,              // Monoton vererbbar (K1)
  members: Set<ι>,
  τ_min: [0,1],                  // Min-Trust für Membership
  isolation: 0|1|2,              // PUBLIC|MEMBERS|STRICT
  governance: Quadratic|Token|Reputation
}

ENTITY τ⃗(TrustVector6D) = (R, I, C, S, Σ, Ω) : [0,1]⁶
  // R=Reliability, I=Integrity, C=Competence
  // S=Social, Σ=Stake, Ω=Wisdom(∫past)

ENTITY γ(Gas) = u64              // Compute-Kosten (pro Operation)
ENTITY μ(Mana) = u64             // Bandwidth-Kosten (regenerierbar)
ENTITY π(Package) {
  id: String,
  version: SemVer,
  publisher: ι,
  trust_min: [0,1]
}
ENTITY σ(Shard) {
  idx: u32,
  realms: Map<id, Arc<ρ>>,
  reputation: [0,1],
  entropy: [0,1]
}
ENTITY ε(Event) = variant { TrustUpdate | Membership | Crossing | Transaction | ... }

// ═══════════════════════════════════════════════════════════════════════════
// AXIOME (Κ) - Die 28 Grundgesetze
// ═══════════════════════════════════════════════════════════════════════════

// TRUST-AXIOME
AXIOM Κ₁  : ρ'.parent = ρ ⟹ rules(ρ) ⊆ rules(ρ')          // Monotone Vererbung
AXIOM Κ₂  : age(ι) < 7d ⟹ max(τ⃗(ι)) ≤ 0.3                  // Newcomer-Cap
AXIOM Κ₃  : |Δτ| ≤ 0.1 per update                           // Delta-Limit
AXIOM Κ₄  : create(ρ) requires τ_R(ι) ≥ 0.5                 // Realm-Creation
AXIOM Κ₅  : τ(t) = τ(0) × 0.99^{inactive_days}              // Trust-Decay
AXIOM Κ₆  : votes(ι) = ⌊√tokens(ι)⌋                         // Deprecated→K21
AXIOM Κ₇  : ι_agent.trust ≤ ι_parent.trust × 0.8            // Agent-Cap

// RESOURCE-AXIOME
AXIOM Κ₈  : γ_eff(op) = γ_base(op) × (2 - τ_R(ι))          // Trust→Gas
AXIOM Κ₉  : μ_regen(ι) = μ_max × (1 - e^{-t/τ_decay})       // Mana-Regen
AXIOM Κ₁₀ : μ_regen ∝ τ_Ω(ι)                                // Wisdom→Mana

// PROTECTION-AXIOME
AXIOM Κ₁₉ : Σ(|Δτ|)/t > threshold ⟹ recalibrate            // Anti-Calcification
AXIOM Κ₂₀ : entropy(τ_distribution) ≥ min_entropy          // Diversity
AXIOM Κ₂₁ : votes(ι) = ⌊√tokens(ι)⌋                         // Quadratic Voting
AXIOM Κ₂₂ : saga(steps) → compensation(reverse(steps))      // Saga-Pattern
AXIOM Κ₂₃ : τ_eff(ι,ρ→ρ') = τ(ι) × factor(ρ,ρ')            // Crossing-Damping
AXIOM Κ₂₄ : τ(ι,ρ₁) independent τ(ι,ρ₂)                     // Local-Trust

// SHARD-AXIOME
AXIOM Κ₂₅ : σ(ρ) = FxHash(ρ.id) mod N                       // Deterministic Shard
AXIOM Κ₂₆ : entropy(σ) < 0.5 × global_entropy ⟹ bias_alarm // Bias-Detection
AXIOM Κ₂₇ : failures(σ) > 100 ⟹ quarantine(σ)              // Auto-Quarantine
AXIOM Κ₂₈ : γ_shard = γ_base × (2 - reputation(σ))          // Shard-Penalty

// ═══════════════════════════════════════════════════════════════════════════
// OPERATIONEN (Op)
// ═══════════════════════════════════════════════════════════════════════════

OP trust(ι, δ) → τ⃗(ι)' = clamp(τ⃗(ι) + δ, 0, 1)
   REQUIRES |δ| ≤ 0.1
   COSTS γ = 10
   EMITS ε::TrustUpdate{ι, δ}

OP join(ι, ρ) → ρ.members += ι, τ(ι,ρ) = 0.3
   REQUIRES τ_R(ι) ≥ ρ.τ_min
   COSTS γ = 50
   EMITS ε::Membership{ρ, ι, Joined}

OP leave(ι, ρ) → ρ.members -= ι, drop τ(ι,ρ)
   COSTS γ = 10
   EMITS ε::Membership{ρ, ι, Left}

OP cross(ι, ρ_src, ρ_dst) → ok | err
   REQUIRES τ_eff(ι) ≥ ρ_dst.τ_min
   WHERE τ_eff = τ(ι) × factor(ρ_src, ρ_dst)
   COSTS γ = 100 × factor
   EMITS ε::Crossing{ι, ρ_src, ρ_dst}

OP create_realm(ι, parent, config) → ρ'
   REQUIRES τ_R(ι) ≥ 0.5
   ENSURES rules(parent) ⊆ rules(ρ')
   COSTS μ = 1000
   EMITS ε::RealmCreated{ρ', parent, ι}

OP install(ι, ρ, π) → ρ.packages += π
   REQUIRES τ_R(π.publisher) ≥ π.trust_min
   COSTS γ = 50, μ = size(π)
   EMITS ε::PackageInstalled{ρ, π}

OP vote(ι, proposal, direction) → proposal.votes += √tokens(ι)
   REQUIRES ι ∈ proposal.realm.members
   COSTS γ = 20
   EMITS ε::Vote{ι, proposal, √tokens}

// ═══════════════════════════════════════════════════════════════════════════
// SYNERGY-MATRIX (→ gegenseitige Verstärkung)
// ═══════════════════════════════════════════════════════════════════════════

SYNERGY Trust → Gas     : γ_eff = γ_base × (2 - τ_R)         // High-Trust = billiger
SYNERGY Trust → Mana    : μ_regen ∝ τ_Ω                       // Wisdom = mehr Regen
SYNERGY Realm → Trust   : τ(ι,ρ) lokal, portabel via Κ₂₃     // Realm-isoliert
SYNERGY Shard → Gas     : γ_shard = γ_base × (2 - rep(σ))    // Toxisch = teurer
SYNERGY Package → Realm : π installed per ρ, config per ρ    // Realm-scoped
SYNERGY Identity → Realm: ι hat Sub-DID per ρ                // Privacy via DIDs
SYNERGY Event → State   : apply(ε) → State' (Event-Sourcing)  // Replay-fähig

// ═══════════════════════════════════════════════════════════════════════════
// STATE-TRANSITIONS (δ)
// ═══════════════════════════════════════════════════════════════════════════

STATE = (Identities, Realms, Trust, Resources, Shards, Events)

TRANSITION δ : State × Event → State

δ(s, ε::TrustUpdate{ι, Δ}) = s{ Trust[ι] += Δ }
δ(s, ε::Membership{ρ, ι, Joined}) = s{ Realms[ρ].members += ι }
δ(s, ε::Membership{ρ, ι, Left}) = s{ Realms[ρ].members -= ι }
δ(s, ε::Crossing{ι, src, dst}) = s{ /* metrics update */ }
δ(s, ε::RealmCreated{ρ, parent, _}) = s{ Realms += ρ }
δ(s, ε::PackageInstalled{ρ, π}) = s{ Realms[ρ].packages += π }

// ═══════════════════════════════════════════════════════════════════════════
// INVARIANTEN (I) - Muss IMMER gelten
// ═══════════════════════════════════════════════════════════════════════════

INVARIANT I₁ : ∀ι: 0 ≤ τ⃗(ι) ≤ 1                              // Trust-Range
INVARIANT I₂ : ∀ρ ≠ Root: ∃ρ': parent(ρ) = ρ'                // Realm-Tree
INVARIANT I₃ : ∀ρ,ρ': parent(ρ') = ρ ⟹ rules(ρ) ⊆ rules(ρ') // Rule-Monotonie
INVARIANT I₄ : ∀σ: |σ.realms| ≤ max_per_shard                // Shard-Cap
INVARIANT I₅ : ∀ι: age(ι) < 7d ⟹ τ(ι) ≤ 0.3                  // Newcomer-Cap
INVARIANT I₆ : ∀op: γ(op) ≤ γ_block_limit                    // Gas-Limit
INVARIANT I₇ : ∀ι,ρ: ι ∈ ρ ⟺ τ(ι,ρ) defined                 // Membership-Trust

// ═══════════════════════════════════════════════════════════════════════════
// FORMELN (F) - Berechnungsregeln
// ═══════════════════════════════════════════════════════════════════════════

FORMULA τ_weighted(ι, ctx) = τ⃗(ι) · w⃗(ctx)                   // Kontextgewichtung
FORMULA τ_eff(ι, ρ→ρ') = τ(ι) × crossing_factor(ρ, ρ')
FORMULA crossing_factor(ρ, ρ') = base × allowlist_bonus × trust_factor
FORMULA γ_total(tx) = Σ(γ(op) for op in tx.operations)
FORMULA μ_balance(ι, t) = min(μ_max, μ(ι,0) + μ_regen × t)
FORMULA votes(ι) = ⌊√tokens(ι)⌋
FORMULA quorum(ρ) = √|ρ.members| × τ_avg(ρ)
FORMULA shard(ρ) = FxHash(ρ.id) % num_shards
FORMULA reputation(σ) = successes(σ) / (successes(σ) + failures(σ))

// ═══════════════════════════════════════════════════════════════════════════
// KONFIGURATION (C) - Tunbare Parameter
// ═══════════════════════════════════════════════════════════════════════════

CONFIG {
  trust_decay_factor: 0.99,       // Per inactive day
  newcomer_period: 7d,            // Κ₂ threshold
  newcomer_max_trust: 0.3,        // Κ₂ cap
  trust_delta_max: 0.1,           // Κ₃ limit
  realm_create_min_trust: 0.5,    // Κ₄ threshold
  mana_regen_base: 100/hour,      // Base regeneration
  gas_trust_multiplier: 0.5,      // For Κ₈
  num_shards: 64,                 // Production: 128
  max_per_shard: 20000,           // LRU-Eviction threshold
  shard_quarantine_threshold: 100, // Failures before quarantine
  crossing_base_factor: 0.7,      // Κ₂₃ base
}

// ═══════════════════════════════════════════════════════════════════════════
// ENDE PLUTO-DNA
// ═══════════════════════════════════════════════════════════════════════════
