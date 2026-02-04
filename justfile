# Erynoa Monorepo - Justfile
# Vereinfacht nach ECLVM-Migration (kein DB, Cache, Auth)

set dotenv-load

WORKSPACE_ROOT := env_var_or_default("WORKSPACE_ROOT", ".")

default:
    @just --list

# ═══════════════════════════════════════════════════════
# 🚀 DEVELOPMENT
# ═══════════════════════════════════════════════════════

# Startet vollstaendigen Dev-Stack mit Hot-Reload
dev frontend="":
    #!/usr/bin/env bash
    set -e

    # WORKSPACE_ROOT absolut aufloesen
    if [ -z "${WORKSPACE_ROOT}" ] || [ "${WORKSPACE_ROOT}" = "." ]; then
        CURRENT_DIR="$(pwd)"
        WORKSPACE_ROOT="$CURRENT_DIR"
        while [ "$WORKSPACE_ROOT" != "/" ]; do
            if [ -f "$WORKSPACE_ROOT/justfile" ]; then
                break
            fi
            WORKSPACE_ROOT="$(dirname "$WORKSPACE_ROOT")"
        done
        if [ ! -f "$WORKSPACE_ROOT/justfile" ]; then
            WORKSPACE_ROOT="$CURRENT_DIR"
        fi
    else
        WORKSPACE_ROOT="$(cd "$WORKSPACE_ROOT" 2>/dev/null && pwd || echo "$WORKSPACE_ROOT")"
    fi
    export WORKSPACE_ROOT

    # Frontend validieren
    FRONTEND_NAME=$(echo "{{frontend}}" | tr '[:upper:]' '[:lower:]')
    case "$FRONTEND_NAME" in
        ""|all)
            FRONTENDS="console platform docs"
            FRONTEND_DISPLAY="alle Frontends"
            ;;
        console|platform|docs)
            FRONTENDS="$FRONTEND_NAME"
            FRONTEND_DISPLAY="$FRONTEND_NAME"
            ;;
        *)
            echo "❌ Ungueltiger Frontend-Name: $FRONTEND_NAME"
            echo "   Gueltige Optionen: console, platform, docs, all"
            exit 1
            ;;
    esac

    PROXY_URL="${PROXY_URL:-http://localhost:3001}"

    echo ""
    echo "╔════════════════════════════════════════════════════════════════════╗"
    echo "║     🚀 Erynoa Development Environment (ECLVM)                      ║"
    echo "╚════════════════════════════════════════════════════════════════════╝"
    echo ""
    echo "  Frontend:  ${FRONTEND_DISPLAY}"
    echo "  Proxy:     ${PROXY_URL}"
    echo "  Console:   ${PROXY_URL}/console"
    echo "  Platform:  ${PROXY_URL}/platform"
    echo "  Docs:      ${PROXY_URL}/docs"
    echo "  Backend:   ${PROXY_URL}/api"
    echo ""

    # Starte Services
    echo "━━━ Starte Services ━━━"
    DOCKER_DIR="$WORKSPACE_ROOT/infra/docker"
    cd "$DOCKER_DIR" || exit 1

    docker compose up --build -d $FRONTENDS backend proxy

    echo ""
    echo "  ⏳ Warte auf Container-Start..."
    sleep 5

    # Health Check
    echo ""
    echo "━━━ Health Check ━━━"
    API_URL="${PROXY_URL}/api"

    for i in {1..30}; do
        if curl -sf -X POST -H "Content-Type: application/json" -d '{}' \
           "${API_URL}/v1/connect/erynoa.v1.HealthService/Check" >/dev/null 2>&1; then
            echo "  ✓ Backend bereit"
            break
        fi
        sleep 2
        [ $((i % 5)) -eq 0 ] && echo "  ⏳ Warte auf Backend..."
    done

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  ${FRONTEND_DISPLAY}, Backend & Proxy laufen"
    echo "  Logs:      just logs [service]"
    echo "  Status:    just status"
    echo "  Stoppen:   just stop"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Logs anzeigen
    docker compose logs -f $FRONTENDS backend proxy

# Einzelne Services
dev-backend:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose up --build backend

dev-console:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose up --build console

dev-platform:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose up --build platform

dev-docs:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose up --build docs

# ═══════════════════════════════════════════════════════
# 🐳 DOCKER SERVICES
# ═══════════════════════════════════════════════════════

# Stoppt alle Container
stop:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose down

# Baue alle Docker Images
build:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose build

# Logs anzeigen
logs service="":
    #!/usr/bin/env bash
    cd {{WORKSPACE_ROOT}}/infra/docker
    if [ -z "{{service}}" ]; then
        docker compose logs -f
    else
        docker compose logs -f {{service}}
    fi

logs-backend:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose logs -f backend

logs-console:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose logs -f console

logs-platform:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose logs -f platform

logs-docs:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose logs -f docs

logs-proxy:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose logs -f proxy

# Shell in Container
shell service:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose exec {{service}} sh

# ═══════════════════════════════════════════════════════
# 📊 STATUS & MONITORING
# ═══════════════════════════════════════════════════════

# Status aller Services
status:
    #!/usr/bin/env bash
    echo ""
    echo "═══════════════════════════════════════════════════════"
    echo "  Erynoa Service Status"
    echo "═══════════════════════════════════════════════════════"
    echo ""
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose ps
    echo ""
    echo "───────────────────────────────────────────────────────"
    echo "  Health Checks:"
    echo "───────────────────────────────────────────────────────"

    PROXY_URL="${PROXY_URL:-http://localhost:3001}"
    API_URL="${API_URL:-${PROXY_URL}/api}"

    curl -sf -X POST -H "Content-Type: application/json" -d '{}' \
        "${API_URL}/v1/connect/erynoa.v1.HealthService/Check" >/dev/null 2>&1 && \
        echo "  ✓ Backend:   ${API_URL}" || echo "  ✗ Backend:   nicht erreichbar"
    curl -sf "${PROXY_URL}/console/" >/dev/null 2>&1 && \
        echo "  ✓ Console:   ${PROXY_URL}/console" || echo "  ✗ Console:   nicht erreichbar"
    curl -sf "${PROXY_URL}/platform/" >/dev/null 2>&1 && \
        echo "  ✓ Platform:  ${PROXY_URL}/platform" || echo "  ✗ Platform:  nicht erreichbar"
    curl -sf "${PROXY_URL}/docs/" >/dev/null 2>&1 && \
        echo "  ✓ Docs:      ${PROXY_URL}/docs" || echo "  ✗ Docs:      nicht erreichbar"
    echo ""

# ═══════════════════════════════════════════════════════
# 🔨 BACKEND BUILD & TEST
# ═══════════════════════════════════════════════════════

# Backend bauen
backend-build:
    cd {{WORKSPACE_ROOT}}/backend && cargo build

# Backend testen
backend-test:
    cd {{WORKSPACE_ROOT}}/backend && cargo test

# Backend testen (verbose)
backend-test-verbose:
    cd {{WORKSPACE_ROOT}}/backend && cargo test -- --nocapture

# Backend pruefen
backend-check:
    cd {{WORKSPACE_ROOT}}/backend && cargo check

# Backend formatieren
backend-fmt:
    cd {{WORKSPACE_ROOT}}/backend && cargo fmt

# Backend clippy
backend-clippy:
    cd {{WORKSPACE_ROOT}}/backend && cargo clippy

# Backend lokal starten (ohne Docker)
backend-run:
    cd {{WORKSPACE_ROOT}}/backend && cargo run

# ═══════════════════════════════════════════════════════
# 📦 FRONTEND BUILD
# ═══════════════════════════════════════════════════════

# Frontend Dependencies installieren
frontend-install:
    cd {{WORKSPACE_ROOT}} && pnpm install

# Console bauen
console-build:
    cd {{WORKSPACE_ROOT}}/frontend/console && pnpm build

# Platform bauen
platform-build:
    cd {{WORKSPACE_ROOT}}/frontend/platform && pnpm build

# Docs bauen
docs-build:
    cd {{WORKSPACE_ROOT}}/frontend/docs && pnpm build

# ═══════════════════════════════════════════════════════
# 🧹 CLEANUP
# ═══════════════════════════════════════════════════════

# Alle Container und Volumes entfernen
clean:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose down -v --remove-orphans

# Backend target bereinigen
clean-backend:
    cd {{WORKSPACE_ROOT}}/backend && cargo clean

# Node modules bereinigen
clean-node:
    cd {{WORKSPACE_ROOT}} && rm -rf node_modules frontend/*/node_modules

# Alles bereinigen
clean-all: clean clean-backend clean-node

# ═══════════════════════════════════════════════════════
# 🔄 PROTO GENERATION
# ═══════════════════════════════════════════════════════

# Proto-Dateien generieren (TypeScript + Rust)
proto-gen:
    cd {{WORKSPACE_ROOT}} && buf generate

# Proto lint
proto-lint:
    cd {{WORKSPACE_ROOT}} && buf lint

# Proto format
proto-format:
    cd {{WORKSPACE_ROOT}} && buf format -w

# ═══════════════════════════════════════════════════════
# 🌐 P2P TESTNET
# ═══════════════════════════════════════════════════════

# Testnet steuern (run, down, status, logs, build, clean)
testnet cmd="status":
    #!/usr/bin/env bash
    set -e
    COMPOSE_FILE="{{WORKSPACE_ROOT}}/infra/docker/docker-compose.testnet.yml"

    case "{{cmd}}" in
        run|up|start)
            echo "🌐 Starte Erynoa P2P Testnet..."
            docker compose -f "$COMPOSE_FILE" up -d
            echo ""
            echo "  Nodes:"
            echo "    relay1: http://localhost:9101/status (Genesis)"
            echo "    relay2: http://localhost:9102/status"
            echo "    relay3: http://localhost:9103/status"
            echo "    client: http://localhost:9104/status"
            echo ""
            echo "  Logs:   just testnet logs"
            echo "  Status: just testnet status"
            echo "  Stop:   just testnet down"
            ;;
        down|stop)
            echo "🛑 Stoppe Testnet..."
            docker compose -f "$COMPOSE_FILE" down
            echo "✓ Testnet gestoppt"
            ;;
        status)
            echo "📊 Testnet Status:"
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
                    peers=$(echo "$status" | grep -o '"peer_count":[0-9]*' | cut -d: -f2)
                    uptime=$(echo "$status" | grep -o '"uptime_secs":[0-9]*' | cut -d: -f2)
                    printf "  ✓ %-10s Peers: %2s  Uptime: %ss\n" "$node" "${peers:-?}" "${uptime:-?}"
                else
                    printf "  ✗ %-10s (offline)\n" "$node"
                fi
            done
            echo ""
            ;;
        logs)
            docker compose -f "$COMPOSE_FILE" logs -f
            ;;
        build)
            echo "🔨 Baue Testnet-Container..."
            docker compose -f "$COMPOSE_FILE" build
            echo "✓ Build abgeschlossen"
            ;;
        clean)
            echo "🧹 Räume Testnet auf..."
            docker compose -f "$COMPOSE_FILE" down -v --remove-orphans
            echo "✓ Aufräumen abgeschlossen"
            ;;
        restart)
            docker compose -f "$COMPOSE_FILE" down
            docker compose -f "$COMPOSE_FILE" up -d
            echo "✓ Testnet neugestartet"
            ;;
        shell)
            docker compose -f "$COMPOSE_FILE" exec relay1 bash
            ;;
        *)
            echo "Verwendung: just testnet [COMMAND]"
            echo ""
            echo "Commands:"
            echo "  run      Startet das Testnet (4 Nodes)"
            echo "  down     Stoppt alle Nodes"
            echo "  status   Zeigt Status aller Nodes"
            echo "  logs     Zeigt Logs aller Nodes"
            echo "  build    Baut Container neu"
            echo "  clean    Entfernt Container und Volumes"
            echo "  restart  Neustart aller Nodes"
            echo "  shell    Shell in relay1"
            ;;
    esac

# Testnet Logs für spezifischen Node
testnet-logs node="":
    #!/usr/bin/env bash
    COMPOSE_FILE="{{WORKSPACE_ROOT}}/infra/docker/docker-compose.testnet.yml"
    if [ -z "{{node}}" ]; then
        docker compose -f "$COMPOSE_FILE" logs -f
    else
        docker compose -f "$COMPOSE_FILE" logs -f {{node}}
    fi

# ═══════════════════════════════════════════════════════
# 🔥 P2P TESTNET - DEV MODE (Hot-Reloading)
# ═══════════════════════════════════════════════════════

# Testnet im Dev-Modus mit Hot-Reloading + NAT-Simulation
testnet-dev cmd="status":
    #!/usr/bin/env bash
    set -e
    COMPOSE_FILE="{{WORKSPACE_ROOT}}/infra/docker/docker-compose.testnet.dev.yml"

    case "{{cmd}}" in
        run|up|start)
            echo ""
            echo "╔════════════════════════════════════════════════════════════════════╗"
            echo "║  🔥 Erynoa P2P Testnet - DEV MODE                                  ║"
            echo "║     Hot-Reloading + NAT-Simulation + QUIC Support                  ║"
            echo "╚════════════════════════════════════════════════════════════════════╝"
            echo ""
            docker compose -f "$COMPOSE_FILE" up -d --build
            echo ""
            echo "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo "  Nodes (Hot-Reloading aktiviert!):"
            echo "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo "    relay1: http://localhost:9101/status (Genesis + Relay)"
            echo "    relay2: http://localhost:9102/status (Relay)"
            echo "    relay3: http://localhost:9103/status (Relay)"
            echo "    client: http://localhost:9104/status (hinter NAT)"
            echo ""
            echo "  Ports:"
            echo "    TCP:   4001, 4002, 4003, 4004"
            echo "    QUIC:  4433, 4434, 4435, 4436/udp"
            echo "    API:   9101, 9102, 9103, 9104"
            echo ""
            echo "  ⚡ Code ändern → automatischer Rebuild in ~10-20s"
            echo ""
            echo "  Commands:"
            echo "    Logs:        just testnet-dev logs"
            echo "    Status:      just testnet-dev status"
            echo "    Relay-Test:  just testnet-dev test-relay"
            echo "    Stop:        just testnet-dev down"
            echo ""
            ;;
        down|stop)
            echo "🛑 Stoppe Dev-Testnet..."
            docker compose -f "$COMPOSE_FILE" down
            echo "✓ Gestoppt"
            ;;
        status)
            echo ""
            echo "═══════════════════════════════════════════════════════════════"
            echo "  📊 Testnet Status (DEV MODE)"
            echo "═══════════════════════════════════════════════════════════════"
            echo ""
            # Container-Status
            docker compose -f "$COMPOSE_FILE" ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}" 2>/dev/null || \
                docker compose -f "$COMPOSE_FILE" ps
            echo ""
            echo "───────────────────────────────────────────────────────────────"
            echo "  Node Health:"
            echo "───────────────────────────────────────────────────────────────"
            for node in relay1 relay2 relay3 client; do
                case $node in
                    relay1) port=9101 ;;
                    relay2) port=9102 ;;
                    relay3) port=9103 ;;
                    client) port=9104 ;;
                esac
                status=$(curl -s "http://localhost:$port/status" 2>/dev/null || echo "")
                if [ -n "$status" ]; then
                    peers=$(echo "$status" | jq -r '.peer_count // 0' 2>/dev/null || echo "?")
                    uptime=$(echo "$status" | jq -r '.uptime_secs // 0' 2>/dev/null || echo "?")
                    mode=$(echo "$status" | jq -r '.mode // "?"' 2>/dev/null || echo "?")
                    peer_id=$(echo "$status" | jq -r '.peer_id // "?"' 2>/dev/null | head -c 20)
                    printf "  ✓ %-10s Mode: %-8s Peers: %2s  Uptime: %ss  ID: %s...\n" "$node" "$mode" "$peers" "$uptime" "$peer_id"
                else
                    printf "  ⏳ %-10s (compiliert oder startet...)\n" "$node"
                fi
            done
            echo ""
            ;;
        logs)
            docker compose -f "$COMPOSE_FILE" logs -f --tail=100
            ;;
        build)
            echo "🔨 Baue Dev-Testnet-Container (ohne Cache)..."
            docker compose -f "$COMPOSE_FILE" build --no-cache
            echo "✓ Build abgeschlossen"
            ;;
        clean)
            echo "🧹 Räume Dev-Testnet komplett auf..."
            docker compose -f "$COMPOSE_FILE" down -v --remove-orphans
            echo "✓ Container, Volumes und Netzwerke entfernt"
            ;;
        rebuild)
            echo "🔄 Kompletter Rebuild..."
            docker compose -f "$COMPOSE_FILE" down -v --remove-orphans
            docker compose -f "$COMPOSE_FILE" build --no-cache
            docker compose -f "$COMPOSE_FILE" up -d
            echo "✓ Rebuild abgeschlossen"
            ;;
        shell)
            node="${2:-relay1}"
            echo "🐚 Shell in $node..."
            docker compose -f "$COMPOSE_FILE" exec relay1 bash
            ;;
        test-relay)
            echo ""
            echo "🧪 Teste Relay-Verbindung (Client → Relays)..."
            echo ""
            CLIENT_STATUS=$(curl -s http://localhost:9104/status 2>/dev/null)
            if [ -n "$CLIENT_STATUS" ]; then
                PEERS=$(echo "$CLIENT_STATUS" | jq -r '.peer_count // 0')
                CONNECTED=$(echo "$CLIENT_STATUS" | jq -r '.connected_peers // []')
                echo "  Client-Status:"
                echo "    Peer-Count: $PEERS"
                if [ "$PEERS" -gt 0 ]; then
                    echo "    ✓ Client hat Verbindung zu Peers"
                    echo ""
                    echo "  Verbundene Peers:"
                    echo "$CONNECTED" | jq -r '.[]' 2>/dev/null | while read peer; do
                        echo "    - ${peer:0:20}..."
                    done
                else
                    echo "    ⚠ Client hat noch keine Peers"
                    echo "    → Warte auf Relay-Verbindung (kann 30-60s dauern)"
                fi
            else
                echo "  ✗ Client nicht erreichbar (noch am Compilieren?)"
            fi
            echo ""
            ;;
        test-gossip)
            echo ""
            echo "📨 Teste Gossipsub-Mesh..."
            echo ""
            for node in relay1 relay2 relay3 client; do
                case $node in
                    relay1) port=9101 ;;
                    relay2) port=9102 ;;
                    relay3) port=9103 ;;
                    client) port=9104 ;;
                esac
                status=$(curl -s "http://localhost:$port/status" 2>/dev/null)
                if [ -n "$status" ]; then
                    peers=$(echo "$status" | jq -r '.peer_count // 0')
                    printf "  %-10s Peers im Mesh: %s\n" "$node" "$peers"
                fi
            done
            echo ""
            ;;
        *)
            echo "Verwendung: just testnet-dev [COMMAND]"
            echo ""
            echo "Commands:"
            echo "  run         Startet Dev-Testnet mit Hot-Reloading"
            echo "  down        Stoppt alle Nodes"
            echo "  status      Zeigt Status aller Nodes"
            echo "  logs        Zeigt Logs (tail -f)"
            echo "  build       Baut Container neu (ohne Cache)"
            echo "  clean       Entfernt Container, Volumes, Netzwerke"
            echo "  rebuild     Komplett neu: clean + build + start"
            echo "  shell       Shell in relay1"
            echo "  test-relay  Testet Relay-Verbindung"
            echo "  test-gossip Testet Gossipsub-Mesh"
            echo ""
            echo "Tipps:"
            echo "  - Code ändern → automatischer Rebuild (~10-20s)"
            echo "  - Bei Problemen: just testnet-dev rebuild"
            echo "  - Privacy-Mode: CARGO_FEATURES=p2p,privacy-full just testnet-dev run"
            ;;
    esac

# Testnet-Dev Logs für spezifischen Node
testnet-dev-logs node="":
    #!/usr/bin/env bash
    COMPOSE_FILE="{{WORKSPACE_ROOT}}/infra/docker/docker-compose.testnet.dev.yml"
    if [ -z "{{node}}" ]; then
        docker compose -f "$COMPOSE_FILE" logs -f --tail=100
    else
        docker compose -f "$COMPOSE_FILE" logs -f --tail=100 {{node}}
    fi
