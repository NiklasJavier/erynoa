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
            echo "    relay1: http://localhost:9001/status (Genesis)"
            echo "    relay2: http://localhost:9002/status"
            echo "    relay3: http://localhost:9003/status"
            echo "    client: http://localhost:9004/status"
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
                    relay1) port=9001 ;;
                    relay2) port=9002 ;;
                    relay3) port=9003 ;;
                    client) port=9004 ;;
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
