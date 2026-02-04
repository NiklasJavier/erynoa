# 🌐 Erynoa P2P Testnet

Multi-Node Docker-Umgebung für P2P-Entwicklung und Tests mit Hot-Reloading.

## Quick Start

```bash
# Dev-Testnet starten (Hot-Reloading + NAT-Simulation)
just testnet-dev run

# Status prüfen
just testnet-dev status

# Logs verfolgen
just testnet-dev logs

# Relay-Verbindung testen
just testnet-dev test-relay

# Stoppen
just testnet-dev down
```

## Architektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    erynoa-testnet (Bridge Networks)                         │
│                                                                             │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │                    testnet (172.28.0.0/16)                        │    │
│   │                                                                   │    │
│   │  ┌─────────┐     ┌─────────┐     ┌─────────┐                     │    │
│   │  │ relay1  │◄───►│ relay2  │◄───►│ relay3  │                     │    │
│   │  │ .0.10   │     │ .0.11   │     │ .0.12   │                     │    │
│   │  │ :4001   │     │ :4002   │     │ :4003   │                     │    │
│   │  │ :4433/u │     │ :4434/u │     │ :4435/u │                     │    │
│   │  │ GENESIS │     │  RELAY  │     │  RELAY  │                     │    │
│   │  └────┬────┘     └────┬────┘     └────┬────┘                     │    │
│   │       │               │               │                          │    │
│   │       └───────────────┴───────────────┴───────────────┐          │    │
│   │                    Gossipsub Mesh                     │          │    │
│   │                    Kademlia DHT                       │          │    │
│   └───────────────────────────────────────────────────────┼──────────┘    │
│                                                           │               │
│   ┌───────────────────────────────────────────────────────┼──────────┐    │
│   │               testnet-nat (172.29.0.0/16)             │          │    │
│   │                                                       │          │    │
│   │  ┌─────────┐                              ┌───────────┴────┐     │    │
│   │  │ client  │◄─────── Circuit Relay ──────►│  nat-gateway   │     │    │
│   │  │ .0.20   │                              │  .0.1 / .0.254 │     │    │
│   │  │ :4004   │                              │   MASQUERADE   │     │    │
│   │  │ (NAT)   │                              └────────────────┘     │    │
│   │  └─────────┘                                                     │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Nodes

| Node   | Rolle   | TCP Port | QUIC Port | API Port | Netzwerk    | Bootstrap              |
| ------ | ------- | -------- | --------- | -------- | ----------- | ---------------------- |
| relay1 | Genesis | 4001     | 4433/udp  | 9101     | testnet     | -                      |
| relay2 | Relay   | 4002     | 4434/udp  | 9102     | testnet     | relay1                 |
| relay3 | Relay   | 4003     | 4435/udp  | 9103     | testnet     | relay1, relay2         |
| client | Client  | 4004     | 4436/udp  | 9104     | testnet-nat | relay1, relay2, relay3 |

## API Endpoints

Jeder Node stellt einen HTTP-API bereit:

```bash
# Health-Check
curl http://localhost:9101/health

# Status (inkl. Peer-Count)
curl http://localhost:9101/status | jq

# Verbundene Peers
curl http://localhost:9101/peers | jq
```

**Beispiel-Response:**

```json
{
  "node_name": "relay1",
  "mode": "relay",
  "peer_id": "12D3KooW...",
  "is_genesis": true,
  "peer_count": 3,
  "connected_peers": ["12D3KooW..."],
  "uptime_secs": 120,
  "version": "0.1.0"
}
```

## Hot-Reloading (Dev-Modus)

Das Dev-Testnet unterstützt Hot-Reloading via `cargo-watch`:

1. **Source-Code mounten**: `backend/src/` wird in alle Container gemountet (read-only)
2. **Auto-Rebuild**: Änderungen an `.rs`, `Cargo.toml` oder `config/` lösen Rebuild aus
3. **Shared Caches**: Cargo-Registry und Git-Cache werden zwischen Nodes geteilt
4. **Polling-Modus**: `--poll` für zuverlässige Erkennung in Docker-Volumes

**Typischer Workflow:**

```bash
# 1. Testnet starten
just testnet-dev run

# 2. Logs verfolgen (zeigt Rebuilds)
just testnet-dev logs

# 3. Code ändern
# → cargo-watch erkennt Änderung (~2s Delay)
# → Rebuild startet (~10-20s)
# → Node startet mit neuem Code

# 4. Status prüfen
just testnet-dev status
```

## Befehle

```bash
just testnet-dev [COMMAND]

run         # Startet das Dev-Testnet (4 Nodes + NAT-Gateway)
down        # Stoppt alle Nodes
status      # Zeigt Status aller Nodes mit Peer-Count
logs        # Folgt den Logs aller Nodes
build       # Baut Container neu (ohne Cache)
clean       # Entfernt Container, Volumes, Netzwerke
rebuild     # Komplett neu: clean + build + start
shell       # Öffnet Shell in relay1
test-relay  # Testet Relay-Verbindung vom Client
test-gossip # Testet Gossipsub-Mesh aller Nodes
```

stop # Stoppt alle Nodes
restart # Neustart aller Nodes
logs # Zeigt Logs aller Nodes
logs:f # Folgt den Logs (tail -f)
status # Zeigt Status aller Nodes mit Peer-Count
build # Baut Container neu (ohne Cache)
clean # Löscht Container und Volumes
shell # Öffnet Shell in Container (default: relay1)

````

## Manueller Docker-Compose

```bash
cd infra/docker

# Starten
docker compose -f docker-compose.testnet.yml up -d

# Logs
docker compose -f docker-compose.testnet.yml logs -f

# Stoppen
docker compose -f docker-compose.testnet.yml down

# Mit Volumes löschen
docker compose -f docker-compose.testnet.yml down -v
````

## P2P-Konfiguration

Die Nodes verwenden folgende libp2p-Features:

- **Transport**: TCP mit Noise-Verschlüsselung, Yamux-Multiplexing
- **Discovery**: mDNS (LAN) + Kademlia DHT
- **PubSub**: Gossipsub für Event-Propagation
- **Protocol**: Custom Sync-Protocol für Event-Synchronisation

**Environment-Variablen:**

| Variable          | Beschreibung                  | Default |
| ----------------- | ----------------------------- | ------- |
| `NODE_NAME`       | Name des Nodes (für Logging)  | `node`  |
| `NODE_MODE`       | Modus (`relay` oder `client`) | `relay` |
| `P2P_PORT`        | libp2p Swarm Port             | `4001`  |
| `API_PORT`        | HTTP API Port                 | `9000`  |
| `BOOTSTRAP_PEERS` | Komma-separierte Multiaddrs   | -       |
| `P2P_ENABLE_MDNS` | mDNS für LAN-Discovery        | `true`  |
| `GENESIS_NODE`    | Ob dies der Genesis-Node ist  | `false` |

## Troubleshooting

### Container startet nicht

```bash
# Logs prüfen
docker compose -f infra/docker/docker-compose.testnet.yml logs relay1

# Container neu bauen
./scripts/dev/testnet.sh build
./scripts/dev/testnet.sh start
```

### Peers verbinden sich nicht

```bash
# Prüfe ob alle Container laufen
docker ps | grep erynoa

# Prüfe Network-Konnektivität
docker exec -it erynoa-relay2 ping 172.28.0.10

# Prüfe mDNS
docker exec -it erynoa-relay1 avahi-browse -at
```

### Hot-Reloading funktioniert nicht

```bash
# Prüfe Volume-Mounts
docker inspect erynoa-relay1 | jq '.[0].Mounts'

# Manueller Rebuild
docker compose -f infra/docker/docker-compose.testnet.yml up -d --build
```

## Development

### Eigene Testszenarien

Du kannst eigene Nodes hinzufügen indem du `docker-compose.testnet.yml` erweiterst:

```yaml
services:
  relay4:
    <<: *common-build  # Wiederverwendet Build-Konfiguration
    container_name: erynoa-relay4
    hostname: relay4
    networks:
      testnet:
        ipv4_address: 172.28.0.13
    ports:
      - "4005:4001"
      - "9005:9000"
    environment:
      NODE_NAME: relay4
      NODE_MODE: relay
      BOOTSTRAP_PEERS: "/ip4/172.28.0.10/tcp/4001"
    volumes:
      - *common-volumes
      - relay4-target:/workspace/backend/target
      - relay4-data:/data
```

### Performance-Tuning

Für schnellere Builds:

1. **MOLD-Linker**: Bereits in Dockerfile konfiguriert
2. **Cargo-Registry Cache**: Shared zwischen allen Nodes
3. **Inkrementelle Builds**: `CARGO_INCREMENTAL=1` aktiviert

### Privacy-Layer testen

Um den Privacy-Layer zu testen, ändere das Feature in `Dockerfile.testnet`:

```dockerfile
# In entrypoint.sh:
exec cargo watch --poll --why \
  -x "run --features privacy --bin erynoa-testnet-node" \
  ...
```
