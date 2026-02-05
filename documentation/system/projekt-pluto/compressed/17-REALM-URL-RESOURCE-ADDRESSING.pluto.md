# Pluto::RealmURL ≡ Resource-Adressierung

> **Notation:** Pluto (komprimiert-formal)
> **Version:** 1.0 | **Datum:** 2026-02
> **Konstanten:** Κ26 (URL-Schema), Κ27 (Resource-Resolution), Κ28 (Open-Access-Policy)

---

## §1 URL-Schema – Formaldefinition (Κ26)

### Δ1.1 URL-Grammatik

$$\boxed{\text{URL} = \texttt{erynoa://} \langle \text{authority} \rangle / \langle \text{type} \rangle / \langle \text{path} \rangle \, [?\langle \text{params} \rangle] \, [\#\langle \text{fragment} \rangle]}$$

### Δ1.2 Komponenten-Algebra

$$\text{URL} \coloneqq \langle \mathcal{A}, \tau, \pi, \phi, \psi \rangle$$

**Wobei:**
- $\mathcal{A} \in \text{DID} \cup \text{Alias}$ — Authority (Realm-ID)
- $\tau \in \mathcal{T}$ — Resource-Type
- $\pi \in \Sigma^*$ — Path-Segments
- $\phi : \text{Key} \rightharpoonup \text{Value}$ — Query-Parameter
- $\psi \in \Sigma^* \cup \{\bot\}$ — Fragment (optional)

### Δ1.3 Symboltafel

| Symbol | Definition | Domäne |
|--------|-----------|--------|
| $\mathcal{A}$ | Authority | $\text{DID} \cup \text{Alias}$ |
| $\tau$ | Resource-Type | $\{$store, profile, contract, asset, event, meta, governance, trust$\}$ |
| $\pi$ | Path | $[\text{String}]$ |
| $\phi$ | Query-Params | Map$\langle$String, String$\rangle$ |
| $\mathcal{R}$ | Resolver | $\{$Storage, Identity, ECLVM, EventLog$\}$ |
| $\mathcal{S}$ | Schema | $\text{Type} \to \text{Def}$ |

---

## §2 Authority-Resolution

### Α2.1 Realm-ID Auflösung

$$\boxed{\text{resolve}(\mathcal{A}) = \begin{cases}
\mathcal{A} & \text{if } \mathcal{A} \in \text{DID} \\
\text{Registry}(\mathcal{A}) & \text{if } \mathcal{A} \in \text{Alias}
\end{cases}}$$

**Authority-Typen:**
$$\begin{aligned}
\text{DID:} \quad & \texttt{did:erynoa:circle:abc123...} \\
\text{Alias:} \quad & \texttt{gaming-dao}
\end{aligned}$$

### Α2.2 Alias-Registrierung

$$\text{register}(\text{alias}) : \text{Alias} \times \text{DID} \to \text{Registry}$$

**Kosten:** $\text{Mana} = 10000$ (Anti-Squatting)

---

## §3 Resource-Schema (Κ27)

### Σ3.1 Schema-Struktur

$$\boxed{\mathcal{S} = \langle \text{version}, \mathcal{T}, \text{fallback}, \text{inheritance} \rangle}$$

**Type-Definition:**
$$\text{TypeDef} = \langle \text{pattern}, \mathcal{R}, \text{access}, \text{fields} \rangle$$

### Σ3.2 Standard-Types

| Type | Pattern | Resolver | Access |
|------|---------|----------|--------|
| store | `store/<name>/<key>` | Storage | realm-policy |
| profile | `profile/<did>` | Identity | owner-or-public |
| contract | `contract/<name>/<method>` | ECLVM | contract-policy |
| asset | `asset/<category>/<id>` | Storage | policy-controlled |
| event | `event/<type>/<ts>` | EventLog | members-only |
| meta | `meta/<key>` | Metadata | public |
| governance | `governance/<proposal-id>` | Governance | members-only |
| trust | `trust/<did>` | TrustCore | members-only |

### Σ3.3 Schema-Vererbung (Κ1)

$$\boxed{\mathcal{S}_{\text{child}} \supseteq \mathcal{S}_{\text{parent}}}$$

**Regel:** Kind-Realms können nur erweitern, nicht einschränken.

$$\text{inherited\_types} \subseteq \text{parent.types}$$

---

## §4 Resolution-Engine

### Ρ4.1 Resolution-Algorithmus

$$\boxed{\text{resolve}: \text{URL} \times \text{DID} \to \text{Resource} \cup \{\bot\}}$$

**Pipeline:**
$$\text{URL} \xrightarrow{\text{parse}} \langle \mathcal{A}, \tau, \pi \rangle \xrightarrow{\text{schema}} \text{TypeDef} \xrightarrow{\text{access}} \text{Policy} \xrightarrow{\mathcal{R}} \text{Resource}$$

### Ρ4.2 Resolver-Dispatch

$$\boxed{\mathcal{R}(\tau) = \begin{cases}
\text{StorageResolver} & \tau \in \{\text{store}, \text{asset}\} \\
\text{IdentityResolver} & \tau = \text{profile} \\
\text{ECLVMResolver} & \tau = \text{contract} \\
\text{EventLogResolver} & \tau = \text{event} \\
\text{MetadataResolver} & \tau = \text{meta}
\end{cases}}$$

### Ρ4.3 Storage-Mapping

$$\text{URL} \to \text{Key}$$

$$\texttt{erynoa://R/store/inventory/items} \mapsto \texttt{realm:\{R\}:shared:store:inventory:items}$$

---

## §5 Open-Access-Policy (Κ28)

### Ο5.1 Access-Dichotomie

$$\boxed{\text{Access} = \begin{cases}
\text{Allow}(\mathcal{F}) & \text{if } \text{policy} \vdash \text{requester} \\
\text{Deny} & \text{otherwise}
\end{cases}}$$

**Wobei:** $\mathcal{F}$ — erlaubte Felder

### Ο5.2 Member vs. Non-Member

$$\text{eval}(r, \text{req}) = \begin{cases}
\text{member-access}(\tau, \pi) & \text{if } \text{req} \in \mathcal{M}(\mathcal{R}) \\
\text{open-access}(\tau, \text{req}) & \text{if } \tau \in \mathcal{T}_{\text{public}} \land \text{policy}(\tau) \\
\text{Deny} & \text{otherwise}
\end{cases}$$

### Ο5.3 Trust-Requirements für Non-Members

$$\text{non-member-access} \iff T_\Omega(\text{req}) \geq T_{\Omega,\min} \lor \text{req} \in \bigcup_{R \in \mathcal{R}_{\text{trusted}}} \mathcal{M}(R)$$

**Typisch:** $T_{\Omega,\min} = 0.1$

### Ο5.4 Field-Filtering

$$\mathcal{F}_{\text{allowed}} = \mathcal{F}_{\text{type}} \setminus \mathcal{F}_{\text{excluded}}$$

**Rate-Limiting:**
$$\text{requests}(\text{req}, \Delta t) \leq \rho_{\max}$$

---

## §6 Access-Evaluation-Matrix

### Ε6.1 Evaluationsregel

$$\boxed{\text{access}(\text{req}, \tau, \pi) = \bigvee_{p \in \mathcal{P}} \text{eval}_p(\text{req}, \tau, \pi)}$$

### Ε6.2 Prioritätsordnung

$$\text{Member} \succ \text{Open-Policy} \succ \text{Crossing-Eval} \succ \text{Deny}$$

### Ε6.3 Matrix

| Requester | Type | Policy | Result |
|-----------|------|--------|--------|
| Member | private | any | ✓ Allow |
| Member | public | any | ✓ Allow |
| Non-Member | public | Κ28 | ✓ Allow(filtered) |
| Non-Member | private | any | ✗ Deny |
| Cross-Realm | any | Κ23+Κ28 | ⚖ Crossing-Eval |

---

## §7 Cross-Realm Resolution (Κ23)

### Χ7.1 Crossing-Dampening

$$\boxed{T_{\text{cross}} = T_{\text{local}} \cdot (1 - \kappa_{23})}$$

**Typisch:** $\kappa_{23} = 0.3$

### Χ7.2 Cross-Realm URL

$$\texttt{erynoa://R_1/link/erynoa://R_2/asset/item}$$

**Resolution:** Cascading mit Trust-Dampening

---

## §8 Query-Parameter

### Φ8.1 Standard-Parameter

| Param | Typ | Beschreibung |
|-------|-----|--------------|
| `view` | enum | `{public, full, raw}` |
| `fields` | list | Feld-Selektion |
| `version` | semver | Spezifische Version |
| `at` | ISO8601 | Historischer Zeitpunkt |
| `limit` | int | Pagination |
| `offset` | int | Pagination |
| `sort` | expr | `field:asc|desc` |
| `filter` | expr | `field:value` |

### Φ8.2 Kombinierte Query

$$\texttt{?filter=rarity:legendary\&limit=10\&sort=price:desc}$$

---

## §9 URL-Operationen

### Ω9.1 Operation-Typen

$$\mathcal{O} = \{\text{Read}, \text{Write}, \text{Subscribe}, \text{Execute}\}$$

### Ω9.2 Signatur-Anforderung

$$\boxed{\text{sig-required}(o) \iff o \in \{\text{Write}, \text{Execute}\}}$$

### Ω9.3 Contract-Methoden

$$\texttt{erynoa://R/contract/C/call/method?args=...}$$

**Erzeugt:** Transaction mit Signatur

---

## §10 Integration

### Ι10.1 URL × Storage

$$\text{URL} \xrightarrow{\text{map}} \text{StorageKey}$$

$$\text{erynoa://R/store/S/K} \mapsto \text{realm:}R\text{:shared:store:}S\text{:}K$$

### Ι10.2 URL × DID

$$\text{erynoa://R/profile/\textasciitilde{}alice} \mapsto \text{did:erynoa:self:alice} \text{ (Realm-Kontext)}$$

### Ι10.3 URL × Governance

$$\text{erynoa://R/governance/proposal/P} \to \mathcal{P}(P)$$

### Ι10.4 URL × ECLVM

$$\text{erynoa://R/contract/C/state} \to \text{Contract-Storage}$$

### Ι10.5 URL × Package

$$\text{erynoa://packages/pkg/lib@1.0.0/src/utils.ecl}$$

---

## §11 Sicherheit

### Σ11.1 Path-Traversal-Prevention

$$\forall \pi: \texttt{".."} \notin \pi$$

### Σ11.2 Authority-Verifizierung

$$\text{DID} \implies \text{kryptographisch verifiziert}$$

$$\text{Alias} \implies \text{Registry-Lookup}$$

### Σ11.3 Information-Leakage

$$\text{error}(\text{not-found}) = \text{error}(\text{access-denied})$$

---

## §12 Zusammenfassung

### Ζ12.1 URL-DNA

$$\boxed{\mathcal{U} = \langle \text{Κ26}, \text{Κ27}, \text{Κ28} \rangle}$$

### Ζ12.2 Konstanten-Definitionen

| Konstante | Definition |
|-----------|------------|
| **Κ26** | $\text{url}(\mathcal{R}, \tau, \pi) \to \texttt{erynoa://}\mathcal{R}/\tau/\pi$ |
| **Κ27** | $\text{resolve}(\text{url}, \text{ctx}) \to \text{resource} \iff \mathcal{S}(\mathcal{R}).\text{match}(\tau, \pi)$ |
| **Κ28** | $\text{access}(\text{url}, \text{req}) = \text{policy}(\mathcal{R}).\text{eval}(\tau, \text{req})$ |

### Ζ12.3 Invarianten

$$\begin{aligned}
&\forall \text{url}: \text{resolve}(\text{url}) \text{ terminiert} \implies O(1) \text{ Realm-Lookup} + O(\log n) \text{ Path} \\
&\forall \mathcal{R}: \mathcal{S}(\mathcal{R}) \supseteq \mathcal{S}(\text{parent}(\mathcal{R})) \\
&\forall \text{req}: \text{access}(\text{url}, \text{req}) \in \{\text{Allow}(\mathcal{F}), \text{Deny}\} \\
&\forall \text{non-member}: \text{access}(\text{url}, \text{non-member}) \to \text{Κ28 policy evaluation} \\
&\forall \text{cross-realm}: \text{resolve}(\text{url}) \to \text{Κ23 dampening applied}
\end{aligned}$$

### Ζ12.4 Pluto-Integration

| Konstante | Integration |
|-----------|-------------|
| Κ1 | Schema-Vererbung folgt Regel-Vererbung |
| Κ17/Κ18 | Membership-Status → Access |
| Κ23 | Cross-Realm mit Crossing-Dampening |
| Κ24 | Lokaler Trust bleibt unabhängig bei URL-Access |

---

## §13 URL-Beispiele

### Β13.1 Basis-Zugriffe

$$\begin{aligned}
&\texttt{erynoa://gaming-dao/store/inventory/items} \\
&\texttt{erynoa://social-hub/profile/\textasciitilde{}alice?view=public} \\
&\texttt{erynoa://defi-realm/contract/staking/state}
\end{aligned}$$

### Β13.2 Komplexe Queries

$$\texttt{erynoa://nft-realm/asset/art?filter=creator:alice\&sort=price:desc\&limit=10}$$

### Β13.3 Cross-Realm

$$\texttt{erynoa://R_1/link/erynoa://R_2/asset/item}$$

---

**∎ QED**
