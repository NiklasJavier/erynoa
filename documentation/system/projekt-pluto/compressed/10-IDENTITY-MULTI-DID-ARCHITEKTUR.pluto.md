# 🪪 Pluto::Identity ≡ Multi-DID Architektur

> **Notation:** Pluto (komprimiert-formal)
> **Komprimierung von:** `10-IDENTITY-MULTI-DID-ARCHITEKTUR.md`
> **Version:** 1.0 | **Datum:** 2026-02

---

## §1 Fundamentale Strukturen

### Δ1.1 DID-Definition

$$\boxed{\text{DID} = \langle \mathcal{N}, \mathcal{U}, K_{pub} \rangle}$$

wobei:
- $\mathcal{N} \in \{\text{Self\\_}, \text{Guild}, \text{Spirit}, \text{Thing}, \text{Vessel}, \text{Source}, \text{Craft}, \text{Vault}, \text{Pact}, \text{Circle}\}$
- $\mathcal{U} = H_{\text{Blake3}}(\mathcal{N} \| K_{pub})$ — UniversalId (32 Bytes)
- $K_{pub}$ — Ed25519 Public Key (32 Bytes)

**Format:**
$$\text{did:erynoa}:\langle\text{namespace}\rangle:\langle\text{universal-id-hex}\rangle$$

### Δ1.2 Namespace-Kodierung $\mathcal{N}$

| Byte | Namespace | Semantik | Entitäts-Typ |
|------|-----------|----------|--------------|
| `0x01` | Self\_ | Natürliche Person | Mensch |
| `0x02` | Guild | Organisation/DAO | Kollektiv |
| `0x03` | Spirit | KI-Agent | Autonom |
| `0x04` | Thing | IoT-Gerät | Physisch |
| `0x05` | Vessel | Container/Transport | Mobil |
| `0x06` | Source | Datenquelle/API | Feed |
| `0x07` | Craft | Service/Dienstleistung | Funktion |
| `0x08` | Vault | Speicher/Safe | Persistent |
| `0x09` | Pact | Vertrag/Vereinbarung | Bindend |
| `0x0A` | Circle | Gruppe/Realm | Kollaborativ |

**Invariante:**
$$|\mathcal{N}| = 10 \quad \land \quad \forall n \in \mathcal{N}: n \in [0x01, 0x0A]$$

---

## §2 Hierarchische Ableitung

### Α2.1 DID-Baum $\mathcal{T}$

$$\mathcal{T} = \langle \text{Root}, \mathcal{D}, \mathcal{A}, \mathcal{R} \rangle$$

wobei:
- $\text{Root}$ — Root-DID (Self\_)
- $\mathcal{D} = \{\text{Device}_0, \text{Device}_1, \ldots\}$ — Device-DIDs
- $\mathcal{A} = \{\text{Agent}_0, \text{Agent}_1, \ldots\}$ — Agent-DIDs (Spirit)
- $\mathcal{R} = \{\text{Realm}_1, \text{Realm}_2, \ldots\}$ — Realm-DIDs (Circle)

### Α2.2 Ableitungsfunktionen $\partial$

$$\partial_{\text{device}}(\text{Root}, i) = \text{DID}(\text{Self\_}, H_{\text{Blake3}}(K_{pub}^{\text{Root}} \| \texttt{"device"} \| i))$$

$$\partial_{\text{agent}}(\text{Root}, i) = \text{DID}(\text{Spirit}, H_{\text{Blake3}}(K_{pub}^{\text{Root}} \| \texttt{"agent"} \| i))$$

$$\partial_{\text{realm}}(\text{Root}, \mathcal{U}_r) = \text{DID}(\text{Circle}, H_{\text{Blake3}}(K_{pub}^{\text{Root}} \| \texttt{"realm"} \| \mathcal{U}_r))$$

$$\partial_{\text{custom}}(\text{Root}, \mathcal{N}, \text{ctx}, i) = \text{DID}(\mathcal{N}, H_{\text{Blake3}}(K_{pub}^{\text{Root}} \| \text{ctx} \| i))$$

### Α2.3 BIP44-Derivation-Pfade

$$m / 44' / \text{erynoa}' / 0' / \langle\text{zweck}\rangle / \langle\text{index}\rangle$$

| Pfad | Semantik |
|------|----------|
| $m/44'/\text{ery}'/0'/\text{device}/0$ | Erstes Gerät |
| $m/44'/\text{ery}'/0'/\text{agent}/k$ | $(k+1)$-ter KI-Agent |
| $m/44'/\text{ery}'/0'/\text{realm}/r$ | Realm-spezifische ID |

---

## §3 Betriebsmodi $\mathcal{M}$

### Σ3.1 IdentityMode-Enumeration

$$\mathcal{M} = \{M_0, M_1, M_2, M_3\}$$

| $M_i$ | Name | Signatur-Typ | Trust-Penalty $\tau$ | Realm-fähig |
|-------|------|--------------|---------------------|-------------|
| $M_0$ | Interactive | WebAuthn (HW-bound) | $1.0$ | ✓ |
| $M_1$ | AgentManaged | Software-Key (autonom) | $0.8$ | ✓ |
| $M_2$ | Ephemeral | Flüchtig (kein Persist) | $0.5$ | ✗ |
| $M_3$ | Test | Deterministisch (Fake) | $1.0$ | ✓ |

### Σ3.2 Effektiver Trust

$$\mathbb{T}_{\text{eff}}(s) = \mathbb{T}_{\text{raw}}(s) \cdot \tau(\mathcal{M}(s))$$

**Mana-Berechnung:**
$$\text{Mana}_{\max} = \text{Mana}_{\text{base}} \cdot (1 + \mathbb{T}_{\text{eff}} \cdot 100)$$

---

## §4 Wallet-Integration

### Ω4.1 WalletAddress-Struktur

$$\mathbb{W} = \langle \chi, \alpha, \pi, \delta, t, p \rangle$$

| Symbol | Typ | Beschreibung |
|--------|-----|--------------|
| $\chi$ | String | Chain-ID (CAIP-2: `eip155:1`, `solana:mainnet`) |
| $\alpha$ | String | Adresse auf Chain |
| $\pi$ | String | BIP44 Derivation-Pfad |
| $\delta$ | $\mathcal{U}$ | Abgeleitet von DID |
| $t$ | $\mathbb{N}_{64}$ | Erstellungszeitpunkt |
| $p$ | $\mathbb{B}$ | Primär-Flag |

### Ω4.2 Chain-Derivation-Regeln

$$\partial_{\text{EVM}}(\text{DID}) = \text{keccak256}(K_{\text{secp256k1}})_{[12..32]}$$

$$\partial_{\text{Solana}}(\text{DID}) = \text{base58}(K_{\text{Ed25519}})$$

$$\partial_{\text{Cosmos}}(\text{DID}) = \text{bech32}(\texttt{"cosmos"}, \text{ripemd160}(\text{sha256}(K_{\text{secp256k1}})))$$

---

## §5 DID-Document $\mathcal{D}$

### Κ5.1 Struktur

$$\mathcal{D} = \langle \text{id}, \mathcal{V}, \mathcal{A}, \mathcal{S}, \Delta, t, \mathcal{X} \rangle$$

| Symbol | Typ | Beschreibung |
|--------|-----|--------------|
| $\text{id}$ | DID | Die DID selbst |
| $\mathcal{V}$ | $\text{Vec}\langle\text{VerificationMethod}\rangle$ | Verifikations-Keys |
| $\mathcal{A}$ | $\text{Vec}\langle\mathcal{U}\rangle$ | Authentifizierungs-IDs |
| $\mathcal{S}$ | $\text{Vec}\langle\mathcal{U}\rangle$ | Assertion-IDs |
| $\Delta$ | $\text{Vec}\langle\text{Delegation}\rangle$ | Delegationen |
| $t$ | TemporalCoord | Letztes Update |
| $\mathcal{X}$ | $\text{Map}\langle u16, \text{Vec}\langle u8\rangle\rangle$ | Extension-Slots |

### Κ5.2 VerificationMethod

$$\mathcal{V}_i = \langle \text{id}, \text{ctrl}, \tau, K_{pub} \rangle$$

wobei $\tau \in \{\text{Ed25519}, \text{Secp256k1}, \text{X25519}\}$

### Κ5.3 Extension-Slots

| Slot-ID | Name | Beschreibung |
|---------|------|--------------|
| `0x0001` | RECOVERY\_KEYS | Key-Rotation Recovery |
| `0x0002` | BIOMETRIC\_BINDING | Biometrische Verifikation |
| `0x0003` | HARDWARE\_ATTESTATION | TEE/TPM Attestation |
| `0x0004` | CROSS\_CHAIN\_LINKS | Multi-Chain Verknüpfungen |
| `0x0005` | AI\_AGENT\_MANIFEST | KI-Agent-Konfiguration |

---

## §6 Delegation $\Delta$ (Κ8)

### Ψ6.1 Trust-Vererbung

$$\boxed{s \rhd s' \Rightarrow \mathbb{T}(s') \leq \tau_{\text{factor}} \cdot \mathbb{T}(s)}$$

wobei $\tau_{\text{factor}} \in (0, 1]$

### Ψ6.2 Delegation-Struktur

$$\Delta = \langle \text{id}, s, s', \tau, \mathcal{C}, t_{\text{exp}}?, t_{\text{create}}, \rho \rangle$$

| Symbol | Typ | Invariante |
|--------|-----|------------|
| $\text{id}$ | $\mathcal{U}$ | Eindeutige ID |
| $s$ | $\mathcal{U}$ | Delegator |
| $s'$ | $\mathcal{U}$ | Delegate |
| $\tau$ | $[0,1]$ | Trust-Faktor |
| $\mathcal{C}$ | $\text{Vec}\langle\text{Capability}\rangle$ | Fähigkeiten |
| $t_{\text{exp}}?$ | $\text{Option}\langle\text{TemporalCoord}\rangle$ | Ablaufzeit |
| $t_{\text{create}}$ | TemporalCoord | Erstellungszeit |
| $\rho$ | $\mathbb{B}$ | Revoked-Flag |

### Ψ6.3 Capability-Algebra $\mathcal{C}$

$$\mathcal{C} = \{\star, \text{read}:r, \text{write}:r, \text{execute}:a, \text{delegate}:n, \text{attest}:\vec{t}, \text{custom}:k:p\}$$

| Capability | Format | Semantik |
|------------|--------|----------|
| $\star$ | `*` | Vollzugriff (⚠️) |
| $\text{read}:r$ | `read:resource` | Lesezugriff |
| $\text{write}:r$ | `write:resource` | Schreibzugriff |
| $\text{execute}:a$ | `execute:action` | Aktionsausführung |
| $\text{delegate}:n$ | `delegate:N` | Weiterdelegation (max $n$ Tiefe) |
| $\text{attest}:\vec{t}$ | `attest:type1,type2` | Claim-Attestierung |

### Ψ6.4 Ketten-Trust-Propagation

Für Delegationskette $[s_0 \rhd s_1 \rhd \ldots \rhd s_n]$:

$$\mathbb{T}_{\text{eff}}(s_n) = \mathbb{T}(s_0) \cdot \prod_{i=0}^{n-1} \tau_i$$

**Tiefenbegrenzung:**
$$\text{depth}(\Delta) \leq n_{\max} \quad \text{wobei } n_{\max} = \text{delegate}:n$$

---

## §7 Realm-Membership $\mathcal{R}$

### Φ7.1 Isolationsprinzip

$$\forall r, r' \in \mathcal{R}: r \neq r' \Rightarrow \text{State}(r) \cap \text{State}(r') = \emptyset$$

### Φ7.2 RealmMembership-Struktur

$$\mathcal{R}_m = \langle r, \text{root}, \text{sub}?, t_{\text{join}}, \mathbb{T}_{\text{local}}, \rho, \Delta_r, \alpha \rangle$$

| Symbol | Typ | Beschreibung |
|--------|-----|--------------|
| $r$ | $\mathcal{U}$ | Realm-ID |
| $\text{root}$ | $\mathcal{U}$ | Root-DID des Mitglieds |
| $\text{sub}?$ | $\text{Option}\langle\mathcal{U}\rangle$ | Realm-Sub-DID |
| $t_{\text{join}}$ | TemporalCoord | Beitrittszeitpunkt |
| $\mathbb{T}_{\text{local}}$ | $[0,1]$ | Realm-lokaler Trust |
| $\rho$ | RealmRole | Rolle |
| $\Delta_r$ | $\text{Vec}\langle\mathcal{U}\rangle$ | Realm-Delegationen |
| $\alpha$ | $\mathbb{B}$ | Aktiv-Flag |

### Φ7.3 Rollen-Multiplikatoren

$$\mathbb{T}_{\text{eff}}^{\mathcal{R}} = \min(1.0, \mathbb{T}_{\text{local}} \cdot \mu_\rho)$$

| $\rho$ | $\mu_\rho$ |
|--------|-----------|
| Member | $1.0$ |
| Moderator | $1.1$ |
| Admin | $1.2$ |
| Owner | $1.3$ |

---

## §8 P2P-Konvertierung

### Θ8.1 Identifier-Triangel

$$\text{DID} \xleftrightarrow{K_{pub}} \text{PeerId} \xleftrightarrow{\mathcal{U}} \text{UniversalId}$$

**Alle drei teilen denselben Ed25519 Public Key als Fundament.**

### Θ8.2 Konvertierungsfunktionen

$$f_{\text{DID} \to \text{PeerId}}(\text{DID}) = \text{PeerId}(\text{PublicKey}(K_{pub}^{\text{Ed25519}}))$$

$$f_{\text{PeerId} \to \text{DID}}(K_{pub}) = \text{DID}(\text{Self\_}, H_{\text{Blake3}}(K_{pub}))$$

$$f_{\text{DID} \to \mathcal{U}}(\text{DID}) = \text{DID}.\text{id}$$

### Θ8.3 PeerIdentity

$$\mathcal{P} = \langle \text{DID}, \mathcal{U}, \text{Keypair}, \text{PeerId} \rangle$$

---

## §9 IdentityState $\mathcal{S}_I$

### Λ9.1 State-Partitionen

$$\mathcal{S}_I = \mathcal{S}_{\text{Root}} \cup \mathcal{S}_{\text{Sub}} \cup \mathcal{S}_{\Delta} \cup \mathcal{S}_{\mathcal{R}} \cup \mathcal{S}_{\mathbb{W}} \cup \mathcal{S}_{\text{Mode}} \cup \mathcal{S}_K \cup \mathcal{S}_\mu$$

| Partition | Inhalt | Concurrency |
|-----------|--------|-------------|
| $\mathcal{S}_{\text{Root}}$ | Root-DID, DIDDocument | RwLock |
| $\mathcal{S}_{\text{Sub}}$ | Device-DID, Sub-DIDs | RwLock |
| $\mathcal{S}_{\Delta}$ | Delegations-Map | RwLock |
| $\mathcal{S}_{\mathcal{R}}$ | Realm-Memberships | RwLock |
| $\mathcal{S}_{\mathbb{W}}$ | Wallet-Adressen | RwLock |
| $\mathcal{S}_{\text{Mode}}$ | IdentityMode, Bootstrap-Flag | Atomic |
| $\mathcal{S}_K$ | KeyStore, PasskeyManager | Option |
| $\mathcal{S}_\mu$ | Gas, Mana, Signatures, Events | Atomic |

### Λ9.2 Bootstrap-Flow

$$\text{User} \xrightarrow{\text{init}} \text{bootstrap}(K_{pub}) \xrightarrow{\text{create}} \text{Root-DID} \xrightarrow{\text{doc}} \mathcal{D} \xrightarrow{\text{store}} \mathcal{S}_I \xrightarrow{\text{derive}} \partial_{\text{device}}(0) \xrightarrow{\text{emit}} \text{IdentityBootstrapped}$$

---

## §10 StateGraph-Relationen

### Ξ10.1 Identity-Relationen

$$\begin{aligned}
\text{Identity} &\xrightarrow{\text{Triggers}} \text{Trust} \\
\text{Identity} &\xrightarrow{\text{Triggers}} \text{Event} \\
\text{Gas} &\xrightarrow{\text{DependsOn}} \text{Identity} \\
\text{Mana} &\xrightarrow{\text{DependsOn}} \text{Identity} \\
\text{Identity} &\xleftrightarrow{\text{Bidirectional}} \text{Delegation} \\
\text{Wallet} &\xrightarrow{\text{Aggregates}} \text{Identity} \\
\text{SubDID} &\xrightarrow{\text{Aggregates}} \text{Identity} \\
\text{Realm} &\xrightarrow{\text{DependsOn}} \text{Identity}
\end{aligned}$$

### Ξ10.2 Delegation-Relationen

$$\begin{aligned}
\text{Trust} &\xrightarrow{\text{DependsOn}} \text{Delegation} \\
\text{Delegation} &\xrightarrow{\text{Triggers}} \text{Event} \\
\text{Delegation} &\xrightarrow{\text{Validates}} \text{Capability}
\end{aligned}$$

---

## §11 StateEvents $\mathcal{E}_I$

### Π11.1 Event-Kategorien

$$\mathcal{E}_I = \mathcal{E}_{\text{Bootstrap}} \cup \mathcal{E}_{\text{Sub}} \cup \mathcal{E}_\Delta \cup \mathcal{E}_\mathcal{R} \cup \mathcal{E}_\mathbb{W} \cup \mathcal{E}_\text{Cred}$$

| Event | Parameter |
|-------|-----------|
| `IdentityBootstrapped` | $\langle \text{root}, \mathcal{M}, \text{has\_device} \rangle$ |
| `SubDIDDerived` | $\langle \text{root}, \text{sub}, \text{purpose}, \mathcal{N} \rangle$ |
| `DelegationCreated` | $\langle s, s', \tau, \mathcal{C} \rangle$ |
| `DelegationRevoked` | $\langle \text{id}_\Delta \rangle$ |
| `RealmJoined` | $\langle \text{root}, r, \text{sub}?, \rho \rangle$ |
| `RealmLeft` | $\langle \text{root}, r \rangle$ |
| `WalletAddressAdded` | $\langle \text{did}, \chi, \alpha \rangle$ |
| `CredentialIssued` | $\langle \text{issuer}, \text{subject}, \text{type} \rangle$ |
| `CredentialVerified` | $\langle \text{verifier}, \text{id}, \text{valid} \rangle$ |

---

## §12 Axiom-Mapping

| Axiom | Formale Aussage | Implementierung |
|-------|-----------------|-----------------|
| **Κ6** | $\forall e: \exists! \text{did}(e)$ | `DID::new()` mit Content-Addressing |
| **Κ7** | $\text{created}(\mathcal{U}) \Rightarrow \text{immutable}(\mathcal{U})$ | UniversalId ist unveränderlich |
| **Κ8** | $s \rhd s' \Rightarrow \mathbb{T}(s') \leq \tau \cdot \mathbb{T}(s)$ | `Delegation.trust_factor` |
| **Κ2** | $\mathbb{T} \in [0, 1]$ | `local_trust.clamp(0.0, 1.0)` |
| **Κ4** | Asymmetrische Evolution | `IdentityMode.trust_penalty_factor()` |
| **Κ24** | Realm-Crossing Dämpfung | `RealmMembership.local_trust` |

---

## §13 Zusammenfassung

$$
\begin{array}{|l|c|l|}
\hline
\textbf{Konzept} & \textbf{Symbol} & \textbf{Kernaussage} \\
\hline
\text{Root-DID} & \text{did:erynoa:self:...} & \text{Eine Person = Eine Root-Identität} \\
\text{Sub-DIDs} & \partial(\text{Root}, \cdot) & \text{Deterministisch ableitbar} \\
\text{Delegation} & s \rhd s' & \text{Trust-Decay nach Κ8} \\
\text{Realm-Isolation} & \mathcal{R}_i \cap \mathcal{R}_j = \emptyset & \text{Lokaler Trust pro Realm} \\
\text{Wallet-Derivation} & \partial_\chi(\text{DID}) & \text{Multi-Chain aus einer Identität} \\
\text{Content-Addressing} & \mathcal{U} = H_{\text{Blake3}}(\cdot) & \text{Kein zentrales Registry} \\
\hline
\end{array}
$$

---

**∎ QED**
