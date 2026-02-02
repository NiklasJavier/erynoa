#!/bin/bash
# ============================================================================
# Erynoa P2P Testnet Control Script
# ============================================================================
#
# Verwendung:
#   ./scripts/dev/testnet.sh [COMMAND]
#
# Befehle:
#   start     - Startet das Testnet (4 Nodes)
#   stop      - Stoppt alle Nodes
#   restart   - Neustart aller Nodes
#   logs      - Zeigt Logs aller Nodes
#   logs:f    - Folgt den Logs (tail -f)
#   status    - Zeigt Status aller Nodes
#   build     - Baut die Container neu
#   clean     - Entfernt alle Container und Volumes
#   shell     - Öffnet Shell in einem Container
#
# ============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
COMPOSE_FILE="$PROJECT_ROOT/infra/docker/docker-compose.testnet.yml"

# Farben für Output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging-Funktionen
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Prüfe ob Docker läuft
check_docker() {
    if ! docker info >/dev/null 2>&1; then
        log_error "Docker ist nicht gestartet!"
        exit 1
    fi
}

# Starte Testnet
cmd_start() {
    log_info "🚀 Starte Erynoa P2P Testnet..."
    check_docker

    docker compose -f "$COMPOSE_FILE" up -d

    log_success "Testnet gestartet!"
    echo ""
    echo "  Nodes:"
    echo "    relay1: http://localhost:9001/status (Genesis)"
    echo "    relay2: http://localhost:9002/status"
    echo "    relay3: http://localhost:9003/status"
    echo "    client: http://localhost:9004/status"
    echo ""
    echo "  Logs: ./scripts/dev/testnet.sh logs:f"
}

# Stoppe Testnet
cmd_stop() {
    log_info "🛑 Stoppe Testnet..."
    check_docker

    docker compose -f "$COMPOSE_FILE" down

    log_success "Testnet gestoppt."
}

# Neustart
cmd_restart() {
    cmd_stop
    sleep 2
    cmd_start
}

# Zeige Logs
cmd_logs() {
    check_docker
    docker compose -f "$COMPOSE_FILE" logs "$@"
}

# Folge Logs
cmd_logs_follow() {
    check_docker
    docker compose -f "$COMPOSE_FILE" logs -f "$@"
}

# Status abfragen
cmd_status() {
    log_info "📊 Testnet Status:"
    echo ""

    check_docker

    for node in relay1 relay2 relay3 client; do
        port=$((9000 + $(echo $node | sed 's/relay//' | sed 's/client/4/')))

        # Versuche Status abzurufen
        status=$(curl -s "http://localhost:$port/status" 2>/dev/null || echo "offline")

        if [ "$status" != "offline" ]; then
            name=$(echo "$status" | jq -r '.node_name // "?"')
            mode=$(echo "$status" | jq -r '.mode // "?"')
            peers=$(echo "$status" | jq -r '.peer_count // 0')
            uptime=$(echo "$status" | jq -r '.uptime_secs // 0')

            printf "  ${GREEN}●${NC} %-10s Mode: %-8s Peers: %2d  Uptime: %ds\n" \
                "$name" "$mode" "$peers" "$uptime"
        else
            printf "  ${RED}○${NC} %-10s (offline)\n" "$node"
        fi
    done

    echo ""
    docker compose -f "$COMPOSE_FILE" ps
}

# Container neu bauen
cmd_build() {
    log_info "🔨 Baue Testnet-Container..."
    check_docker

    docker compose -f "$COMPOSE_FILE" build --no-cache

    log_success "Build abgeschlossen."
}

# Alles aufräumen
cmd_clean() {
    log_warn "⚠️  Dies löscht alle Container, Volumes und Build-Cache!"
    read -p "Fortfahren? (y/N) " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        log_info "🧹 Räume auf..."
        check_docker

        docker compose -f "$COMPOSE_FILE" down -v --remove-orphans

        # Optional: Alle testnet-bezogenen Images löschen
        docker images | grep "erynoa-testnet" | awk '{print $3}' | xargs -r docker rmi -f 2>/dev/null || true

        log_success "Aufräumen abgeschlossen."
    else
        log_info "Abgebrochen."
    fi
}

# Shell in Container
cmd_shell() {
    node="${1:-relay1}"
    log_info "🐚 Öffne Shell in $node..."
    check_docker

    docker compose -f "$COMPOSE_FILE" exec "$node" bash
}

# Hilfe anzeigen
cmd_help() {
    echo "Erynoa P2P Testnet Control"
    echo ""
    echo "Verwendung: $0 [COMMAND]"
    echo ""
    echo "Befehle:"
    echo "  start         Startet das Testnet (4 Nodes)"
    echo "  stop          Stoppt alle Nodes"
    echo "  restart       Neustart aller Nodes"
    echo "  logs          Zeigt Logs aller Nodes"
    echo "  logs:f        Folgt den Logs (tail -f)"
    echo "  status        Zeigt Status aller Nodes"
    echo "  build         Baut die Container neu"
    echo "  clean         Entfernt alle Container und Volumes"
    echo "  shell [node]  Öffnet Shell in Container (default: relay1)"
    echo "  help          Zeigt diese Hilfe"
    echo ""
    echo "Beispiele:"
    echo "  $0 start"
    echo "  $0 logs:f relay1"
    echo "  $0 shell client"
}

# Main
case "${1:-help}" in
    start)
        cmd_start
        ;;
    stop)
        cmd_stop
        ;;
    restart)
        cmd_restart
        ;;
    logs)
        shift
        cmd_logs "$@"
        ;;
    logs:f)
        shift
        cmd_logs_follow "$@"
        ;;
    status)
        cmd_status
        ;;
    build)
        cmd_build
        ;;
    clean)
        cmd_clean
        ;;
    shell)
        shift
        cmd_shell "$@"
        ;;
    help|--help|-h)
        cmd_help
        ;;
    *)
        log_error "Unbekannter Befehl: $1"
        cmd_help
        exit 1
        ;;
esac
