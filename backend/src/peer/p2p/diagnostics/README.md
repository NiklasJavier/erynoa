# Erynoa Diagnostics Portal

Umfassendes Echtzeit-Monitoring für alle Erynoa-Module.

## Architektur

```
┌─────────────────────────────────────────────────────────────────┐
│ ERYNOA DIAGNOSTICS PORTAL │
├─────────────────────────────────────────────────────────────────┤
│ ┌─────────────────────┐ ┌─────────────────────┐ │
│ │ SwarmState │ │ SystemState │ │
│ │ (P2P Layer) │ │ (All Modules) │ │
│ └─────────┬───────────┘ └─────────┬───────────┘ │
│ │ │ │
│ ┌─────────┴───────────────────────────┴───────────┐ │
│ │ DiagnosticState │ │
│ │ • swarm_state: Option<Arc<SwarmState>> │ │
│ │ • system_state: Option<Arc<SystemState>> │ │
│ └──────────────────────┬───────────────────────────┘ │
│ │ │
│ ┌──────────────────────┴───────────────────────────┐ │
│ │ SSE Stream (500ms) │ │
│ │ StreamSnapshot { │ │
│ │ swarm, layers, summary, │ │
│ │ system_layers, system │ │
│ │ } │ │
│ └───────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Module

### SwarmState (P2P Layer)

- Bytes In/Out
- Connected Peers
- Gossipsub Mesh
- Kademlia DHT
- NAT/Relay Status

### SystemState (Core, ECLVM, Local, Protection)

| Layer                          | Metriken                                      |
| ------------------------------ | --------------------------------------------- |
| **Trust Engine (Κ2-Κ5)**       | Entities, Relationships, Updates, Violations  |
| **Event Engine (Κ9-Κ12)**      | Events, Genesis, Finalized, Witnessed         |
| **World Formula (Κ15b)**       | 𝔼 Value, Contributors, Human Verified         |
| **Consensus (Κ18)**            | Epoch, Validators, Success/Failed Rounds      |
| **ECLVM Runtime**              | Programs Executed, Gas, OOG, Active VMs       |
| **Mana System**                | Accounts, Consumed, Regenerated, Rate Limited |
| **Policy Gateway**             | Policies, Evaluations, Allowed/Denied         |
| **Local Storage**              | KV Store, Events, Identities, Realms          |
| **Archive (ψ_archive)**        | Epochs, Events, Merkle Roots                  |
| **Anomaly Detection**          | Detected, By Severity                         |
| **Diversity Monitor (Κ20)**    | Entropy, Monoculture Warnings                 |
| **Quadratic Governance (Κ21)** | Votes, Participants                           |
| **Anti-Calcification (Κ19)**   | Interventions, Violations                     |

## Quick Start

```rust
use erynoa::peer::p2p::diagnostics::{
    create_system_state, create_full_diagnostic_state,
    SwarmState, SystemState, diagnostic_routes,
};
use std::sync::Arc;

// 1. State erstellen
let swarm_state = Arc::new(SwarmState::new());
let system_state = create_system_state();

// 2. Diagnostic State mit beiden verbinden
let diagnostic_state = create_full_diagnostic_state(
    peer_id.to_string(),
    swarm_state.clone(),
    system_state.clone(),
);

// 3. Routes registrieren
let app = axum::Router::new()
    .merge(diagnostic_routes(diagnostic_state));
```

## Engine Integration

Die Engines können SystemState-Metriken über Observer-Pattern aktualisieren:

### TrustEngine

```rust
use crate::core::trust_engine::TrustObserver;

let observer: TrustObserver = system_state.clone();

// Mit Observer
trust_engine.process_event_observed(&event, &observer)?;

// Oder manuell
trust_engine.process_event(&event)?;
system_state.trust_updated(!event.is_negative_trust());
```

### EventEngine

```rust
use crate::core::event_engine::EventObserver;

let observer: EventObserver = system_state.clone();

// Mit Observer
event_engine.add_event_observed(event, &observer)?;

// Oder manuell
event_engine.add_event(event)?;
system_state.event_added(event.parents.is_empty());
```

### ManaManager

```rust
use crate::eclvm::mana::ManaObserver;

let observer: ManaObserver = system_state.clone();

// Mit Observer
mana_manager.deduct_observed(did, trust, gas, &observer)?;

// Oder manuell
mana_manager.deduct(did, trust, gas)?;
system_state.mana_consumed(gas);
```

## API Endpoints

| Endpoint                     | Beschreibung             |
| ---------------------------- | ------------------------ |
| `GET /diagnostics`           | JSON Summary             |
| `GET /diagnostics/report`    | ASCII Report             |
| `GET /diagnostics/stream`    | SSE Live Updates (500ms) |
| `GET /diagnostics/metrics`   | Detailed Network Metrics |
| `GET /diagnostics/events`    | Event Log                |
| `GET /diagnostics/layers`    | Layer Diagnostics        |
| `GET /diagnostics/dashboard` | HTML Dashboard           |

## Dashboard Features

- **KPI Strip**: 6 P2P KPIs + 6 System KPIs
- **P2P Layers**: 8 Layer Health Checks
- **System Layers**: 13 Module Diagnostics
- **Live Traffic Chart**: 60s rolling window
- **Peer Table**: Connected peers with RTT
- **Event Log**: Real-time event stream

## Files

```
diagnostics/
├── mod.rs              # Module exports + Axum routes
├── state.rs            # DiagnosticState
├── swarm_state.rs      # P2P metrics (atomics)
├── system_state.rs     # All module metrics (atomics)
├── system_layers.rs    # Layer diagnostic generators
├── integration.rs      # Observer traits for engines
├── types.rs            # StreamSnapshot, LayerDiagnostic
├── layers.rs           # P2P layer checks
├── metrics.rs          # Network metrics
├── events.rs           # Event buffer
├── dashboard.rs        # HTML generation
└── README.md           # This file
```
