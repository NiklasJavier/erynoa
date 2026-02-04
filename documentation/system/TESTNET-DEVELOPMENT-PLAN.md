# 🌐 Erynoa P2P Testnet – Entwicklungsplan

> **Ziel**: 4-Peer-Testnet mit Hot-Reloading, Relay-basierter Kommunikation (QUIC + Circuit Relay), vollständigem NAT-Traversal Stack und maximaler Sicherheit unter Docker-Simulation.

---

## 📋 Übersicht

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ERYNOA TESTNET ARCHITEKTUR                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │                    Docker Bridge Network                          │    │
│   │                       172.28.0.0/16                               │    │
│   │                                                                   │    │
│   │  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐       │    │
│   │  │ RELAY1  │◄──►│ RELAY2  │◄──►│ RELAY3  │    │ CLIENT  │       │    │
│   │  │ Genesis │    │  Relay  │    │  Relay  │    │  (NAT)  │       │    │
│   │  │ .0.10   │    │ .0.11   │    │ .0.12   │    │ .0.20   │       │    │
│   │  │ :4001   │    │ :4002   │    │ :4003   │    │ :4004   │       │    │
│   │  │TCP+QUIC │    │TCP+QUIC │    │TCP+QUIC │    │TCP+QUIC │       │    │
│   │  └────┬────┘    └────┬────┘    └────┬────┘    └────┬────┘       │    │
│   │       │              │              │              │             │    │
│   │       └──────────────┴──────────────┴──────────────┘             │    │
│   │                    Gossipsub Mesh                                │    │
│   │                    Kademlia DHT                                  │    │
│   │                    Circuit Relay                                 │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│   Kommunikationsfluss:                                                      │
│   ├── Client verbindet via Circuit Relay (simuliertes NAT)                 │
│   ├── Relay1/2/3 stellen Relay-Dienste bereit                              │
│   ├── DCUTR versucht Holepunching nach Relay-Verbindung                    │
│   └── Gossipsub propagiert Events über Mesh                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🎯 Phase 1: Infrastruktur-Anpassungen

### 1.1 Docker Compose Verbesserungen

**Datei**: `infra/docker/docker-compose.testnet.dev.yml`

#### Aktuelle Probleme:

1. ❌ Keine QUIC-Ports exposed (nur TCP 4001)
2. ❌ Kein dedizierter NAT-Simulations-Container
3. ❌ `image: erynoa/testnet-node:latest` referenziert nicht-existentes Image
4. ❌ Privacy-Features nicht standardmäßig aktiviert

#### Erforderliche Änderungen:

```yaml
# infra/docker/docker-compose.testnet.dev.yml - Erweiterte Version

name: erynoa-testnet-dev

x-common-env: &common-env
  RUST_LOG: "erynoa=debug,libp2p_gossipsub=info,libp2p_kad=info,libp2p_relay=debug,libp2p_dcutr=debug,libp2p_autonat=debug,info"
  RUST_BACKTRACE: "1"
  DEV_MODE: "true"
  CARGO_FEATURES: "p2p,privacy"
  # Sicherheitsfeatures
  P2P_NOISE_ENABLED: "true"
  P2P_TLS_ENABLED: "true"

x-common-volumes: &common-volumes
  - ../../backend/src:/workspace/backend/src:ro
  - ../../backend/config:/workspace/backend/config:ro
  - ../../backend/Cargo.toml:/workspace/backend/Cargo.toml:ro
  - ../../backend/Cargo.lock:/workspace/backend/Cargo.lock:ro
  - ../../backend/build.rs:/workspace/backend/build.rs:ro
  - ../../backend/proto:/workspace/backend/proto:ro
  - cargo-registry:/usr/local/cargo/registry
  - cargo-git:/usr/local/cargo/git

services:
  relay1:
    build:
      context: ../..
      dockerfile: infra/docker/Dockerfile.testnet
    container_name: erynoa-relay1-dev
    hostname: relay1
    networks:
      testnet:
        ipv4_address: 172.28.0.10
    ports:
      - "4001:4001/tcp" # TCP Transport
      - "4433:4433/udp" # QUIC Transport
      - "9101:9000" # HTTP API
    environment:
      <<: *common-env
      NODE_NAME: relay1
      NODE_MODE: relay
      P2P_PORT: "4001"
      QUIC_PORT: "4433"
      API_PORT: "9000"
      GENESIS_NODE: "true"
      BOOTSTRAP_PEERS: ""
      # Relay-Server aktivieren
      P2P_RELAY_SERVER: "true"
      P2P_ENABLE_MDNS: "true"
      # NAT-Traversal
      P2P_ENABLE_AUTONAT: "true"
      P2P_ENABLE_DCUTR: "true"
      P2P_ENABLE_UPNP: "false" # Nicht in Docker
    volumes:
      - *common-volumes
      - relay1-target:/workspace/backend/target
      - relay1-data:/data

  relay2:
    build:
      context: ../..
      dockerfile: infra/docker/Dockerfile.testnet
    container_name: erynoa-relay2-dev
    hostname: relay2
    depends_on:
      relay1:
        condition: service_healthy
    networks:
      testnet:
        ipv4_address: 172.28.0.11
    ports:
      - "4002:4001/tcp"
      - "4434:4433/udp"
      - "9102:9000"
    environment:
      <<: *common-env
      NODE_NAME: relay2
      NODE_MODE: relay
      P2P_PORT: "4001"
      QUIC_PORT: "4433"
      API_PORT: "9000"
      P2P_RELAY_SERVER: "true"
      P2P_ENABLE_MDNS: "true"
      P2P_ENABLE_AUTONAT: "true"
      P2P_ENABLE_DCUTR: "true"
      BOOTSTRAP_PEERS: "/ip4/172.28.0.10/tcp/4001,/ip4/172.28.0.10/udp/4433/quic-v1"

  relay3:
    build:
      context: ../..
      dockerfile: infra/docker/Dockerfile.testnet
    container_name: erynoa-relay3-dev
    hostname: relay3
    depends_on:
      relay1:
        condition: service_healthy
    networks:
      testnet:
        ipv4_address: 172.28.0.12
    ports:
      - "4003:4001/tcp"
      - "4435:4433/udp"
      - "9103:9000"
    environment:
      <<: *common-env
      NODE_NAME: relay3
      NODE_MODE: relay
      P2P_PORT: "4001"
      QUIC_PORT: "4433"
      API_PORT: "9000"
      P2P_RELAY_SERVER: "true"
      P2P_ENABLE_MDNS: "true"
      P2P_ENABLE_AUTONAT: "true"
      P2P_ENABLE_DCUTR: "true"
      BOOTSTRAP_PEERS: "/ip4/172.28.0.10/tcp/4001,/ip4/172.28.0.11/tcp/4001"

  client:
    build:
      context: ../..
      dockerfile: infra/docker/Dockerfile.testnet
    container_name: erynoa-client-dev
    hostname: client
    depends_on:
      relay1:
        condition: service_healthy
      relay2:
        condition: service_healthy
      relay3:
        condition: service_healthy
    networks:
      # Client ist in separatem Netzwerk (simuliertes NAT)
      testnet-nat:
        ipv4_address: 172.29.0.20
    ports:
      - "4004:4001/tcp"
      - "4436:4433/udp"
      - "9104:9000"
    environment:
      <<: *common-env
      NODE_NAME: client
      NODE_MODE: client
      P2P_PORT: "4001"
      QUIC_PORT: "4433"
      API_PORT: "9000"
      # Client nutzt Relay, ist KEIN Server
      P2P_RELAY_SERVER: "false"
      P2P_RELAY_CLIENT: "true"
      P2P_ENABLE_MDNS: "false" # Kein mDNS über NAT
      P2P_ENABLE_AUTONAT: "true"
      P2P_ENABLE_DCUTR: "true"
      # Bootstrap über alle Relays
      BOOTSTRAP_PEERS: "/ip4/172.28.0.10/tcp/4001,/ip4/172.28.0.11/tcp/4001,/ip4/172.28.0.12/tcp/4001"
      # Explizite Relay-Server für Circuit Relay
      RELAY_SERVERS: "/ip4/172.28.0.10/tcp/4001/p2p-circuit,/ip4/172.28.0.11/tcp/4001/p2p-circuit,/ip4/172.28.0.12/tcp/4001/p2p-circuit"

  # NAT-Simulator (iptables-basiert)
  nat-gateway:
    image: alpine:3.19
    container_name: erynoa-nat-gateway
    cap_add:
      - NET_ADMIN
    networks:
      testnet:
        ipv4_address: 172.28.0.254
      testnet-nat:
        ipv4_address: 172.29.0.1
    command: |
      sh -c "
        apk add --no-cache iptables
        echo 1 > /proc/sys/net/ipv4/ip_forward
        # NAT für Client-Netzwerk
        iptables -t nat -A POSTROUTING -s 172.29.0.0/16 -o eth0 -j MASQUERADE
        # Simuliere Symmetric NAT (strenger)
        iptables -A FORWARD -i eth1 -o eth0 -m state --state NEW -j DROP
        iptables -A FORWARD -i eth0 -o eth1 -m state --state ESTABLISHED,RELATED -j ACCEPT
        iptables -A FORWARD -i eth1 -o eth0 -m state --state ESTABLISHED,RELATED -j ACCEPT
        tail -f /dev/null
      "

networks:
  testnet:
    driver: bridge
    ipam:
      config:
        - subnet: 172.28.0.0/16
  testnet-nat:
    driver: bridge
    internal: true # Kein direkter Internet-Zugang
    ipam:
      config:
        - subnet: 172.29.0.0/16

volumes:
  relay1-data:
  relay2-data:
  relay3-data:
  client-data:
  relay1-target:
  relay2-target:
  relay3-target:
  client-target:
  cargo-registry:
  cargo-git:
```

### 1.2 Dockerfile Anpassungen

**Datei**: `infra/docker/Dockerfile.testnet`

#### Erforderliche Änderungen:

```dockerfile
# Zusätzliche Ports für QUIC
EXPOSE 4001 4433/udp 9000

# Verbesserte Entrypoint-Logik
RUN cat > /entrypoint-dev.sh << 'DEVEOF'
#!/bin/bash
set -e

FEATURES="${CARGO_FEATURES:-p2p}"

echo "═══════════════════════════════════════════════════════════════"
echo "🔥 Erynoa P2P Testnet Node - DEV MODE"
echo "═══════════════════════════════════════════════════════════════"
echo "  Node:         ${NODE_NAME:-node}"
echo "  Mode:         ${NODE_MODE:-relay}"
echo "  P2P Port:     ${P2P_PORT:-4001} (TCP)"
echo "  QUIC Port:    ${QUIC_PORT:-4433} (UDP)"
echo "  API Port:     ${API_PORT:-9000}"
echo "  Bootstrap:    ${BOOTSTRAP_PEERS:-none}"
echo "  Relay Server: ${P2P_RELAY_SERVER:-false}"
echo "  Features:     ${FEATURES}"
echo "═══════════════════════════════════════════════════════════════"

# DNS-Auflösung abwarten
if [ -n "$BOOTSTRAP_PEERS" ]; then
    echo "⏳ Warte auf Bootstrap-Peers..."
    for peer in $(echo $BOOTSTRAP_PEERS | tr ',' ' '); do
        host=$(echo $peer | sed 's|/ip4/\([^/]*\)/.*|\1|')
        if [[ ! $host =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
            until getent hosts $host > /dev/null 2>&1; do
                sleep 1
            done
            echo "  ✓ $host aufgelöst"
        fi
    done
fi

# Gestaffelter Start
case $NODE_NAME in
    relay1) DELAY=0 ;;
    relay2) DELAY=8 ;;
    relay3) DELAY=16 ;;
    client) DELAY=24 ;;
    *) DELAY=0 ;;
esac

[ $DELAY -gt 0 ] && echo "⏳ Warte ${DELAY}s..." && sleep $DELAY

echo ""
echo "🚀 Starte mit cargo-watch (Hot-Reloading)..."

exec cargo watch \
    --poll \
    --delay 2 \
    --watch src \
    --watch Cargo.toml \
    --watch config \
    -x "build --features ${FEATURES} --bin erynoa-testnet-node" \
    -s "RUST_LOG=${RUST_LOG:-info} ./target/debug/erynoa-testnet-node"
DEVEOF
```

---

## 🎯 Phase 2: Backend-Code-Erweiterungen

### 2.1 Testnet-Node QUIC-Support

**Datei**: `backend/src/bin/testnet_node.rs`

Erforderliche Erweiterungen:

```rust
// Neue Args für QUIC
struct Args {
    // ... existierende Felder ...
    quic_port: u16,
    enable_relay_server: bool,
    enable_relay_client: bool,
    relay_servers: Vec<String>,
}

impl Args {
    fn parse() -> Self {
        // ...
        quic_port: Self::get_arg(&args, "--quic-port")
            .or_else(|| env::var("QUIC_PORT").ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(4433),

        enable_relay_server: Self::get_arg(&args, "--relay-server")
            .or_else(|| env::var("P2P_RELAY_SERVER").ok())
            .map(|s| s.to_lowercase() == "true" || s == "1")
            .unwrap_or(false),

        enable_relay_client: Self::get_arg(&args, "--relay-client")
            .or_else(|| env::var("P2P_RELAY_CLIENT").ok())
            .map(|s| s.to_lowercase() == "true" || s == "1")
            .unwrap_or(true),

        relay_servers: Self::get_arg(&args, "--relay-servers")
            .or_else(|| env::var("RELAY_SERVERS").ok())
            .map(|s| s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
            .unwrap_or_default(),
    }
}
```

### 2.2 P2P Config für QUIC + Relay

**Datei**: `backend/src/peer/p2p/config.rs`

Listen-Adressen müssen QUIC unterstützen:

```rust
impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            listen_addresses: vec![
                "/ip4/0.0.0.0/tcp/0".to_string(),
                "/ip6/::/tcp/0".to_string(),
                // QUIC hinzufügen
                "/ip4/0.0.0.0/udp/0/quic-v1".to_string(),
                "/ip6/::/udp/0/quic-v1".to_string(),
            ],
            // ...
        }
    }
}
```

### 2.3 Transport-Layer mit QUIC

**Datei**: `backend/src/peer/p2p/testnet.rs`

Der Transport muss QUIC unterstützen:

```rust
use libp2p::quic;

// In TestnetSwarm::new()
pub fn build_transport(keypair: &Keypair) -> Result<Boxed<(PeerId, StreamMuxerBox)>> {
    // TCP + Noise + Yamux
    let tcp_transport = libp2p::tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(Version::V1)
        .authenticate(noise::Config::new(keypair)?)
        .multiplex(yamux::Config::default())
        .boxed();

    // QUIC Transport (libp2p-quic)
    let quic_transport = quic::tokio::Transport::new(quic::Config::new(keypair))
        .map(|(peer_id, muxer), _| (peer_id, StreamMuxerBox::new(muxer)))
        .boxed();

    // Kombiniere: QUIC bevorzugt, TCP als Fallback
    let transport = OrTransport::new(quic_transport, tcp_transport)
        .map(|either, _| match either {
            Either::Left((peer_id, muxer)) => (peer_id, muxer),
            Either::Right((peer_id, muxer)) => (peer_id, muxer),
        })
        .boxed();

    Ok(transport)
}
```

---

## 🎯 Phase 3: Justfile-Erweiterungen

**Datei**: `justfile`

```just
# ═══════════════════════════════════════════════════════
# 🌐 P2P TESTNET (Erweitert)
# ═══════════════════════════════════════════════════════

# Testnet im Dev-Modus mit Hot-Reloading
testnet-dev cmd="status":
    #!/usr/bin/env bash
    set -e
    COMPOSE_FILE="{{WORKSPACE_ROOT}}/infra/docker/docker-compose.testnet.dev.yml"

    case "{{cmd}}" in
        run|up|start)
            echo "🔥 Starte Erynoa P2P Testnet (DEV MODE mit Hot-Reloading)..."
            docker compose -f "$COMPOSE_FILE" up -d --build
            echo ""
            echo "  Nodes (Hot-Reloading aktiviert!):"
            echo "    relay1: http://localhost:9101/status (Genesis + Relay)"
            echo "    relay2: http://localhost:9102/status (Relay)"
            echo "    relay3: http://localhost:9103/status (Relay)"
            echo "    client: http://localhost:9104/status (hinter NAT)"
            echo ""
            echo "  Ports:"
            echo "    TCP:   4001-4004"
            echo "    QUIC:  4433-4436/udp"
            echo ""
            echo "  Logs:   just testnet-dev logs"
            echo "  Status: just testnet-dev status"
            echo "  Stop:   just testnet-dev down"
            ;;
        down|stop)
            echo "🛑 Stoppe Dev-Testnet..."
            docker compose -f "$COMPOSE_FILE" down
            ;;
        status)
            echo "📊 Testnet Status (DEV):"
            echo ""
            docker compose -f "$COMPOSE_FILE" ps
            echo ""
            echo "───────────────────────────────────────────"
            for node in relay1 relay2 relay3 client; do
                case $node in
                    relay1) port=9101 ;;
                    relay2) port=9102 ;;
                    relay3) port=9103 ;;
                    client) port=9104 ;;
                esac
                status=$(curl -s "http://localhost:$port/status" 2>/dev/null || echo "")
                if [ -n "$status" ]; then
                    peers=$(echo "$status" | jq -r '.peer_count // 0')
                    uptime=$(echo "$status" | jq -r '.uptime_secs // 0')
                    mode=$(echo "$status" | jq -r '.mode // "?"')
                    printf "  ✓ %-10s Mode: %-8s Peers: %2s  Uptime: %ss\n" "$node" "$mode" "$peers" "$uptime"
                else
                    printf "  ✗ %-10s (offline oder compiliert...)\n" "$node"
                fi
            done
            echo ""
            ;;
        logs)
            docker compose -f "$COMPOSE_FILE" logs -f
            ;;
        build)
            echo "🔨 Baue Dev-Testnet-Container..."
            docker compose -f "$COMPOSE_FILE" build --no-cache
            ;;
        clean)
            echo "🧹 Räume Dev-Testnet auf..."
            docker compose -f "$COMPOSE_FILE" down -v --remove-orphans
            ;;
        rebuild)
            echo "🔄 Rebuild: Stoppe, lösche Caches, baue neu..."
            docker compose -f "$COMPOSE_FILE" down -v
            docker compose -f "$COMPOSE_FILE" build --no-cache
            docker compose -f "$COMPOSE_FILE" up -d
            ;;
        shell)
            docker compose -f "$COMPOSE_FILE" exec relay1 bash
            ;;
        test-relay)
            echo "🧪 Teste Relay-Verbindung..."
            # Client sollte über Relay verbunden sein
            CLIENT_PEERS=$(curl -s http://localhost:9104/peers 2>/dev/null | jq -r '.peers[]' || echo "")
            if [ -n "$CLIENT_PEERS" ]; then
                echo "  ✓ Client hat Verbindung zu:"
                echo "$CLIENT_PEERS" | while read peer; do
                    echo "    - $peer"
                done
            else
                echo "  ✗ Client hat keine Peers (Relay funktioniert nicht)"
            fi
            ;;
        *)
            echo "Verwendung: just testnet-dev [COMMAND]"
            echo ""
            echo "Commands:"
            echo "  run        Startet Dev-Testnet mit Hot-Reloading"
            echo "  down       Stoppt alle Nodes"
            echo "  status     Zeigt Status aller Nodes"
            echo "  logs       Zeigt Logs (tail -f)"
            echo "  build      Baut Container neu"
            echo "  clean      Entfernt Container und Volumes"
            echo "  rebuild    Kompletter Rebuild (clean + build + start)"
            echo "  shell      Shell in relay1"
            echo "  test-relay Testet Relay-Verbindung"
            ;;
    esac

# Testnet Privacy-Mode (mit Onion-Routing)
testnet-privacy cmd="status":
    #!/usr/bin/env bash
    set -e
    COMPOSE_FILE="{{WORKSPACE_ROOT}}/infra/docker/docker-compose.testnet.dev.yml"
    export CARGO_FEATURES="p2p,privacy-full"

    case "{{cmd}}" in
        run|up)
            echo "🔐 Starte Privacy-Testnet (Onion-Routing + QUIC)..."
            CARGO_FEATURES="p2p,privacy-full" docker compose -f "$COMPOSE_FILE" up -d --build
            ;;
        *)
            just testnet-dev {{cmd}}
            ;;
    esac
```

---

## 🎯 Phase 4: Sicherheits-Härtung

### 4.1 Transport-Sicherheit

| Layer            | Technologie                 | Status           |
| ---------------- | --------------------------- | ---------------- |
| **Encryption**   | Noise Protocol (XX Pattern) | ✅ Implementiert |
| **Transport**    | QUIC (TLS 1.3 built-in)     | 🔄 Hinzufügen    |
| **Multiplexing** | Yamux (TCP) / Native (QUIC) | ✅ Implementiert |
| **Auth**         | Ed25519 PeerID              | ✅ Implementiert |

### 4.2 Docker-Netzwerk-Isolation

```yaml
# Separate Netzwerke für NAT-Simulation
networks:
  testnet: # "Internet" - Relays erreichbar
    driver: bridge
  testnet-nat: # "LAN hinter NAT" - Client isoliert
    driver: bridge
    internal: true # Kein direkter Zugang
```

### 4.3 Relay-Trust-Konfiguration

```rust
// In config.rs
pub struct TrustGateConfig {
    /// Minimum Trust für Relay-Server (Κ19)
    pub min_relay_server_trust: f64,  // 0.5 für Production

    /// Nur bekannte Relays akzeptieren
    pub trusted_relays_only: bool,
}
```

---

## 🎯 Phase 5: Debugging & Monitoring

### 5.1 API-Endpunkte erweitern

**Neue Endpunkte in testnet_node.rs**:

| Endpoint        | Beschreibung                            |
| --------------- | --------------------------------------- |
| `GET /health`   | Simple Health-Check                     |
| `GET /status`   | Node-Status mit Peers, Uptime, Mode     |
| `GET /peers`    | Liste aller verbundenen Peers           |
| `GET /relay`    | Relay-Status (Reservierungen, Circuits) |
| `GET /nat`      | NAT-Status (AutoNAT-Ergebnis)           |
| `GET /gossip`   | Gossipsub-Mesh-Status                   |
| `POST /publish` | Event an Gossipsub senden (Debug)       |

### 5.2 Logging-Kategorien

```bash
# Empfohlenes RUST_LOG für Debugging
RUST_LOG="erynoa=debug,\
libp2p_gossipsub=info,\
libp2p_kad=info,\
libp2p_relay=debug,\
libp2p_dcutr=debug,\
libp2p_autonat=info,\
libp2p_quic=debug,\
info"
```

### 5.3 Hot-Reloading Workflow

```
┌─────────────────────────────────────────────────────────────────┐
│                    HOT-RELOADING WORKFLOW                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Code ändern (backend/src/...)                              │
│         ↓                                                       │
│  2. cargo-watch erkennt Änderung (inotify/poll)                │
│         ↓                                                       │
│  3. Inkrementeller Build (~5-15s mit MOLD)                     │
│         ↓                                                       │
│  4. Alter Prozess gestoppt, neuer gestartet                    │
│         ↓                                                       │
│  5. P2P-Stack initialisiert, verbindet zu Peers                │
│         ↓                                                       │
│  6. Node wieder im Mesh (Gossipsub re-join)                    │
│                                                                 │
│  ⚡ Typische Cycle-Zeit: 10-20 Sekunden                        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📝 Implementierungs-Checkliste

### Sofort (Tag 1)

- [ ] `docker-compose.testnet.dev.yml` aktualisieren (NAT-Simulation, QUIC-Ports)
- [ ] `Dockerfile.testnet` QUIC-Port hinzufügen
- [ ] Justfile mit `testnet-dev` Command erweitern
- [ ] Health-Checks für alle Nodes prüfen

### Kurzfristig (Woche 1)

- [ ] QUIC-Transport in `testnet.rs` aktivieren (libp2p-quic)
- [ ] Neue Environment-Variablen in `testnet_node.rs` implementieren
- [ ] Relay-Status API-Endpunkt hinzufügen
- [ ] NAT-Gateway Container testen

### Mittelfristig (Woche 2-3)

- [ ] Privacy-Layer Features testen (Onion-Routing)
- [ ] Performance-Benchmarks (QUIC vs TCP)
- [ ] Stress-Tests mit vielen Events
- [ ] Dokumentation aktualisieren

---

## 🚀 Quick-Start Guide

```bash
# 1. Dev-Testnet starten
just testnet-dev run

# 2. Logs verfolgen (alle Nodes)
just testnet-dev logs

# 3. Status prüfen
just testnet-dev status

# 4. Relay-Verbindung testen
just testnet-dev test-relay

# 5. Code ändern → automatischer Rebuild in ~10-20s

# 6. Bei Problemen: Komplett neu bauen
just testnet-dev rebuild

# 7. Aufräumen
just testnet-dev clean
```

---

## ⚠️ Bekannte Einschränkungen

1. **Docker-NAT ≠ echtes NAT**: Die NAT-Simulation ist vereinfacht. Echte Symmetric NAT ist komplexer.

2. **QUIC in Docker**: UDP-Routing in Docker kann Probleme machen. Bei Problemen: `--network host` für Tests.

3. **Shared Target-Cache**: Alle Nodes teilen sich den Cargo-Cache. Bei Race-Conditions: Separate Volumes nutzen.

4. **Hot-Reload Latenz**: Bei großen Änderungen (neue Dependencies) dauert der Rebuild länger (~30-60s).

---

## 📚 Referenzen

- [libp2p Relay Specification](https://github.com/libp2p/specs/tree/master/relay)
- [libp2p QUIC](https://github.com/libp2p/rust-libp2p/tree/master/transports/quic)
- [DCUTR Holepunching](https://github.com/libp2p/specs/blob/master/relay/DCUtR.md)
- [Erynoa P2P-PRIVATE-RELAY-LOGIC V2.6](../concept-v4/P2P-PRIVATE-RELAY-LOGIC.md)
