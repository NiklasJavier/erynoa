# Erynoa Monorepo - Justfile
# Optimiert für Performance und Benutzerfreundlichkeit

set dotenv-load

# Workspace-Root ermitteln (funktioniert sowohl im DevContainer als auch auf dem Host)
# Nutze env_var_or_default() um Fehler bei fehlender Variable zu vermeiden
WORKSPACE_ROOT := env_var_or_default("WORKSPACE_ROOT", ".")

default:
    @just --list

# ═══════════════════════════════════════════════════════
# 🚀 DEVELOPMENT
# ═══════════════════════════════════════════════════════

# [DEFAULT] Startet vollständigen Dev-Stack mit Hot-Reload
# Usage: just dev [frontend]
#   just dev          → Alle Frontends (console, platform, docs)
#   just dev console  → Nur Console
#   just dev platform → Nur Platform
#   just dev docs     → Nur Docs
dev frontend="":
    #!/usr/bin/env bash
    set -e
    
    # Normalisiere Frontend-Name
    FRONTEND_NAME=$(echo "{{frontend}}" | tr '[:upper:]' '[:lower:]')
    
    # Validiere und setze Frontends
    case "$FRONTEND_NAME" in
        ""|all)
            FRONTENDS="console platform docs"
            FRONTEND_DISPLAY="alle Frontends (console, platform, docs)"
            ;;
        console|platform|docs)
            FRONTENDS="$FRONTEND_NAME"
            FRONTEND_DISPLAY="$FRONTEND_NAME"
            ;;
        *)
            echo "❌ Ungültiger Frontend-Name: $FRONTEND_NAME"
            echo "   Gültige Optionen: console, platform, docs, all"
            exit 1
            ;;
    esac
    
    # Service URLs
    PROXY_URL="${PROXY_URL:-http://localhost:3001}"
    CONSOLE_URL="${CONSOLE_URL:-${PROXY_URL}/console}"
    PLATFORM_URL="${PLATFORM_URL:-${PROXY_URL}/platform}"
    DOCS_URL="${DOCS_URL:-${PROXY_URL}/docs}"
    API_URL="${API_URL:-${PROXY_URL}/api}"
    ZITADEL_URL="${ZITADEL_URL:-http://localhost:8080}"
    MINIO_CONSOLE_URL="${MINIO_CONSOLE_URL:-http://localhost:9001}"
    
    echo ""
    echo "╔════════════════════════════════════════════════════════════════════╗"
    echo "║     🚀 Erynoa Development Environment                              ║"
    echo "╚════════════════════════════════════════════════════════════════════╝"
    echo ""
    echo "  Frontend:  ${FRONTEND_DISPLAY}"
    echo "  Proxy:     ${PROXY_URL}"
    echo "  Console:   ${CONSOLE_URL}"
    echo "  Platform:  ${PLATFORM_URL}"
    echo "  Docs:      ${DOCS_URL}"
    echo "  Backend:   ${API_URL}"
    echo "  ZITADEL:   ${ZITADEL_URL}"
    echo "  MinIO:     ${MINIO_CONSOLE_URL}"
    echo ""
    
    # 1. Starte Hintergrund-Services
    echo "━━━ [1/5] Starte Hintergrund-Services ━━━"
    cd {{WORKSPACE_ROOT}}/infra/docker
    docker compose --profile auth up -d db cache minio zitadel-db zitadel-init zitadel 2>/dev/null || \
        docker compose --profile auth up -d db cache minio zitadel-db zitadel-init zitadel
    echo "  ✓ Services gestartet"
    
    # 2. Warte auf Services (parallelisiert)
    echo ""
    echo "━━━ [2/5] Warte auf Services ━━━"
    
    wait_for_service() {
        local name=$1
        local cmd=$2
        local max_attempts=${3:-60}
        local attempt=0
        
        echo -n "  Warte auf ${name}..."
        while [ $attempt -lt $max_attempts ]; do
            if eval "$cmd" >/dev/null 2>&1; then
                echo " ✓"
                return 0
            fi
            sleep 1
            attempt=$((attempt + 1))
            if [ $((attempt % 5)) -eq 0 ]; then
                echo -n "."
            fi
        done
        echo " ⚠ (Timeout)"
        return 1
    }
    
    # Parallele Service-Checks
    wait_for_service "PostgreSQL" "docker compose exec -T db pg_isready -U erynoa" &
    wait_for_service "Dragonfly" "docker compose exec -T cache redis-cli ping" &
    wait_for_service "MinIO" "curl -sf ${MINIO_URL:-http://localhost:9000}/minio/health/live" &
    wait_pid=$!
    
    # ZITADEL separat (braucht länger)
    ZITADEL_READY=false
    echo -n "  Warte auf ZITADEL..."
    for i in {1..60}; do
        if curl -sf ${ZITADEL_URL}/debug/ready >/dev/null 2>&1 || \
           curl -sf ${ZITADEL_URL}/.well-known/openid-configuration >/dev/null 2>&1; then
            echo " ✓"
            ZITADEL_READY=true
            break
        fi
        sleep 2
        if [ $((i % 10)) -eq 0 ]; then
            echo -n "."
        fi
    done
    [ "$ZITADEL_READY" = false ] && echo " ⚠ (wird später geprüft)"
    
    wait $wait_pid 2>/dev/null || true
    
    # 3. Initialisierung (nur wenn nötig)
    echo ""
    echo "━━━ [3/5] Initialisierung ━━━"
    cd {{WORKSPACE_ROOT}}
    mkdir -p .data
    
    # MinIO Setup
    if [ ! -f ".data/.minio-setup-complete" ]; then
        echo "  → MinIO Setup..."
        SETUP_SCRIPT="scripts/infra/setup-minio.sh"
        [ -f "$SETUP_SCRIPT" ] || SETUP_SCRIPT="infra/scripts/setup-minio.sh"
        if [ -f "$SETUP_SCRIPT" ]; then
            chmod +x "$SETUP_SCRIPT"
            "$SETUP_SCRIPT" || echo "  ⚠ MinIO Setup übersprungen"
        else
            echo "  ⚠ MinIO Setup-Script nicht gefunden"
        fi
    else
        echo "  ✓ MinIO bereits eingerichtet"
    fi
    
    # ZITADEL Setup
    if [ ! -f ".data/zitadel-setup-complete" ]; then
        echo "  → ZITADEL Setup..."
        if [ "$ZITADEL_READY" != "true" ]; then
            echo "    Warte auf ZITADEL..."
            for i in {1..30}; do
                curl -sf ${ZITADEL_URL}/.well-known/openid-configuration >/dev/null 2>&1 && break
                sleep 2
            done
        fi
        sleep 10  # PAT-Generierung
        SETUP_SCRIPT="scripts/infra/setup-zitadel.sh"
        [ -f "$SETUP_SCRIPT" ] || SETUP_SCRIPT="infra/scripts/setup-zitadel.sh"
        if [ -f "$SETUP_SCRIPT" ]; then
            chmod +x "$SETUP_SCRIPT"
            "$SETUP_SCRIPT" || echo "  ⚠ ZITADEL Setup übersprungen (später: just zitadel-setup)"
        else
            echo "  ⚠ ZITADEL Setup-Script nicht gefunden"
        fi
    else
        echo "  ✓ ZITADEL bereits eingerichtet"
        [ -f ".data/zitadel-client-id" ] && echo "    Client-ID: $(cat .data/zitadel-client-id)"
    fi
    
    # 4. Starte Frontend(s) + Backend + Proxy
    echo ""
    echo "━━━ [4/5] Starte ${FRONTEND_DISPLAY} + Backend + Proxy ━━━"
    echo ""
    echo "  Ctrl+C stoppt Frontend(s) & Backend, Services laufen weiter"
    echo "  Status:    just status"
    echo "  Logs:      just logs"
    echo "  Stoppen:   just stop"
    echo ""
    
    cd {{WORKSPACE_ROOT}}/infra/docker
    trap 'echo ""; echo "━━━ Frontend(s) + Backend gestoppt ━━━"; echo "  Services laufen weiter. Neustart: just dev"; echo ""' INT
    
    docker compose up --build -d $FRONTENDS backend proxy
    
    echo "  ⏳ Warte auf Container-Start..."
    sleep 8
    
    # 5. Health Check
    echo ""
    echo "━━━ [5/5] Health Check ━━━"
    if command -v curl >/dev/null 2>&1; then
        {{WORKSPACE_ROOT}}/scripts/dev/dev-check.sh || echo "  ⚠ Einige Services noch nicht bereit"
    else
        echo "  ⚠ curl nicht verfügbar - Health Check übersprungen"
    fi
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  ${FRONTEND_DISPLAY}, Backend & Proxy laufen"
    echo "  Logs:      just logs [service]"
    echo "  Status:    just status"
    echo "  Stoppen:   just stop"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    # Zeige Logs (blockierend)
    docker compose logs -f $FRONTENDS backend proxy

# Minimaler Dev-Stack (ohne ZITADEL)
dev-minimal:
    #!/usr/bin/env bash
    set -e
    echo "🚀 Starte minimalen Dev-Stack (ohne Auth)..."
    cd {{WORKSPACE_ROOT}}/infra/docker
    docker compose up -d db cache minio
    sleep 5
    docker compose up --build console backend proxy

# Einzelne Services (Services müssen bereits laufen)
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
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose --profile auth down

# Alias für Kompatibilität
docker-stop: stop

# Baue alle Docker Images
docker-build:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose build

# Logs anzeigen (alle oder spezifischer Service)
logs service="":
    #!/usr/bin/env bash
    cd {{WORKSPACE_ROOT}}/infra/docker
    if [ -z "{{service}}" ]; then
        docker compose --profile auth logs -f
    else
        docker compose logs -f {{service}}
    fi

# Alias für Kompatibilität
docker-logs: logs

# Spezifische Service-Logs
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

# Alias für Kompatibilität
docker-logs-backend: logs-backend
docker-logs-console: logs-console
docker-logs-platform: logs-platform
docker-logs-docs: logs-docs

# Shell in Container
shell service:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose exec {{service}} sh

shell-backend:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose exec backend sh

shell-console:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose exec console sh

shell-platform:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose exec platform sh

shell-docs:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose exec docs sh

# Alias für Kompatibilität
docker-backend-shell: shell-backend
docker-console-shell: shell-console
docker-platform-shell: shell-platform
docker-docs-shell: shell-docs

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
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose --profile auth ps
    echo ""
    echo "───────────────────────────────────────────────────────"
    echo "  Health Checks:"
    echo "───────────────────────────────────────────────────────"
    
    PROXY_URL="${PROXY_URL:-http://localhost:3001}"
    API_URL="${API_URL:-${PROXY_URL}/api}"
    CONSOLE_URL="${CONSOLE_URL:-${PROXY_URL}/console}"
    PLATFORM_URL="${PLATFORM_URL:-${PROXY_URL}/platform}"
    DOCS_URL="${DOCS_URL:-${PROXY_URL}/docs}"
    ZITADEL_URL="${ZITADEL_URL:-http://localhost:8080}"
    MINIO_URL="${MINIO_URL:-http://localhost:9000}"
    MINIO_CONSOLE_URL="${MINIO_CONSOLE_URL:-http://localhost:9001}"
    
    check_service() {
        local name=$1
        local url=$2
        local method=${3:-GET}
        if [ "$method" = "POST" ]; then
            curl -sf -X POST -H "Content-Type: application/json" -d '{}' "$url" >/dev/null 2>&1
        else
            curl -sf "$url" >/dev/null 2>&1
        fi
    }
    
    check_service "Backend" "${API_URL}/api/v1/connect/erynoa.v1.HealthService/Check" "POST" && \
        echo "  ✓ Backend:   ${API_URL}" || echo "  ✗ Backend:   nicht erreichbar"
    check_service "Console" "${CONSOLE_URL}/" && \
        echo "  ✓ Console:   ${CONSOLE_URL}" || echo "  ✗ Console:   nicht erreichbar"
    check_service "Platform" "${PLATFORM_URL}/" && \
        echo "  ✓ Platform:  ${PLATFORM_URL}" || echo "  ✗ Platform:  nicht erreichbar"
    check_service "Docs" "${DOCS_URL}/" && \
        echo "  ✓ Docs:      ${DOCS_URL}" || echo "  ✗ Docs:      nicht erreichbar"
    check_service "ZITADEL" "${ZITADEL_URL}/debug/ready" && \
        echo "  ✓ ZITADEL:   ${ZITADEL_URL}" || echo "  ✗ ZITADEL:   nicht erreichbar"
    check_service "MinIO" "${MINIO_URL}/minio/health/live" && \
        echo "  ✓ MinIO:     ${MINIO_CONSOLE_URL}" || echo "  ✗ MinIO:     nicht erreichbar"
    echo ""

# Health Check Script
dev-check:
    {{WORKSPACE_ROOT}}/scripts/dev/dev-check.sh

# Restart nur Dev-Services (schneller)
restart:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose restart console platform docs backend proxy

# Alias für Kompatibilität
restart-dev: restart

# ═══════════════════════════════════════════════════════
# 🏗️ INFRASTRUCTURE
# ═══════════════════════════════════════════════════════

# Starte nur Hintergrund-Services
# Services auf dem Host starten (für DevContainer)
# Diese Services laufen auf dem Host und werden vom DevContainer über host.docker.internal erreicht
services:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose --profile auth up -d db cache minio zitadel-db zitadel-init zitadel

# Services ohne ZITADEL
services-minimal:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose up -d db cache minio

# Stoppe Services
services-stop:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose --profile auth down

# Alias für Kompatibilität
services-down: services-stop

# Service-Logs
services-logs service="":
    #!/usr/bin/env bash
    cd {{WORKSPACE_ROOT}}/infra/docker
    if [ -z "{{service}}" ]; then
        docker compose --profile auth logs -f db cache minio zitadel
    else
        docker compose logs -f {{service}}
    fi

# Service-Status
services-status:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose --profile auth ps -a

# Alias für Kompatibilität
services-ps: services-status

# Services neu starten
services-restart:
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose --profile auth restart

# ═══════════════════════════════════════════════════════
# 🔐 ZITADEL
# ═══════════════════════════════════════════════════════

# Öffne ZITADEL Console
zitadel:
    #!/usr/bin/env bash
    ZITADEL_URL="${ZITADEL_URL:-http://localhost:8080}"
    echo "Öffne ZITADEL Console..."
    echo "Login: zitadel-admin / Password1!"
    echo "Test User: testuser / Test123!"
    echo ""
    ${BROWSER:-open} "${ZITADEL_URL}/ui/console" 2>/dev/null || \
        echo "Öffne: ${ZITADEL_URL}/ui/console"

# ZITADEL Setup
zitadel-setup:
    @chmod +x {{WORKSPACE_ROOT}}/scripts/infra/setup-zitadel.sh
    @{{WORKSPACE_ROOT}}/scripts/infra/setup-zitadel.sh

# ZITADEL Reset
zitadel-reset:
    #!/usr/bin/env bash
    set -e
    echo "⚠️  Setze ZITADEL zurück..."
    cd {{WORKSPACE_ROOT}}/infra/docker
    docker compose --profile auth stop zitadel zitadel-db 2>/dev/null || true
    docker compose --profile auth rm -f zitadel zitadel-db 2>/dev/null || true
    docker volume rm erynoa-services_zitadel-pgdata erynoa-services_zitadel-machinekey 2>/dev/null || true
    rm -f {{WORKSPACE_ROOT}}/.data/zitadel-setup-complete {{WORKSPACE_ROOT}}/.data/zitadel-client-id
    docker compose --profile auth up -d zitadel-db zitadel-init zitadel
    echo "⏳ Warte 30 Sekunden auf Init..."
    sleep 30
    ZITADEL_URL="${ZITADEL_URL:-http://localhost:8080}"
    curl -sf ${ZITADEL_URL}/debug/ready >/dev/null 2>&1 && echo "✓ ZITADEL bereit" || echo "⚠ ZITADEL noch nicht bereit"
    just zitadel-setup

# ═══════════════════════════════════════════════════════
# 💾 MINIO / S3
# ═══════════════════════════════════════════════════════

# MinIO Setup
minio-setup:
    @chmod +x {{WORKSPACE_ROOT}}/scripts/infra/setup-minio.sh
    @{{WORKSPACE_ROOT}}/scripts/infra/setup-minio.sh

# Öffne MinIO Console
minio:
    #!/usr/bin/env bash
    MINIO_CONSOLE_URL="${MINIO_CONSOLE_URL:-http://localhost:9001}"
    echo "Öffne MinIO Console..."
    echo "Login: erynoa / erynoa123"
    echo ""
    ${BROWSER:-open} "${MINIO_CONSOLE_URL}" 2>/dev/null || \
        echo "Öffne: ${MINIO_CONSOLE_URL}"

# MinIO Reset
minio-reset:
    #!/usr/bin/env bash
    set -e
    echo "⚠️  Setze MinIO zurück..."
    cd {{WORKSPACE_ROOT}}/infra/docker
    docker compose stop minio 2>/dev/null || true
    docker compose rm -f minio 2>/dev/null || true
    docker volume rm erynoa-services_minio-data 2>/dev/null || true
    rm -f {{WORKSPACE_ROOT}}/.data/.minio-setup-complete {{WORKSPACE_ROOT}}/.data/.minio-credentials
    docker compose up -d minio
    echo "⏳ Warte 10 Sekunden..."
    sleep 10
    just minio-setup

# ═══════════════════════════════════════════════════════
# 🗄️ DATABASE
# ═══════════════════════════════════════════════════════

# Führe Migrationen aus
db-migrate:
    cd {{WORKSPACE_ROOT}}/backend && sqlx migrate run

# Erstelle neue Migration
db-new name:
    cd {{WORKSPACE_ROOT}}/backend && sqlx migrate add {{name}}

# SQLx Prepare (für Offline-Modus)
db-prepare:
    cd {{WORKSPACE_ROOT}}/backend && cargo sqlx prepare

# Datenbank zurücksetzen
db-reset:
    #!/usr/bin/env bash
    set -e
    cd {{WORKSPACE_ROOT}}/backend
    sqlx database drop -y 2>/dev/null || true
    sqlx database create
    sqlx migrate run
    echo "✅ Datenbank zurückgesetzt"

# ═══════════════════════════════════════════════════════
# 🔧 BACKEND
# ═══════════════════════════════════════════════════════

# Cargo Check
check:
    cd {{WORKSPACE_ROOT}}/backend && cargo check

# Clippy Lint
lint:
    cd {{WORKSPACE_ROOT}}/backend && cargo clippy -- -D warnings

# Format Code
fmt:
    cd {{WORKSPACE_ROOT}}/backend && cargo fmt

# Tests ausführen
test:
    cd {{WORKSPACE_ROOT}}/backend && cargo test

# Alle Checks (fmt + lint + test)
ci: fmt lint test

# Nix Flake Check
ci-nix:
    nix flake check

# ═══════════════════════════════════════════════════════
# 📦 BUILD (Nix)
# ═══════════════════════════════════════════════════════

# Nix Build (Standard)
build-nix:
    nix build

# Alias für Kompatibilität
build: build-nix

# Statisches musl Binary
build-static:
    nix build .#static

# Docker Image via Nix
build-docker:
    nix build .#docker
    @echo "Image: ./result (laden mit: docker load < result)"

# Docker Image bauen und laden
docker-load: build-docker
    docker load < result

# ═══════════════════════════════════════════════════════
# 📝 PROTOBUF / CONNECT-RPC
# ═══════════════════════════════════════════════════════

# Generiere TypeScript aus Proto-Dateien
proto-gen:
    @echo "🔧 Generiere TypeScript aus Protobuf..."
    buf generate
    @echo "✅ Generiert in frontend/*/src/gen/"

# Lint Proto-Dateien
proto-lint:
    buf lint

# Format Proto-Dateien
proto-fmt:
    buf format -w

# ═══════════════════════════════════════════════════════
# 🎨 FRONTEND
# ═══════════════════════════════════════════════════════

# Installiere alle Frontend-Dependencies
frontend-install:
    pnpm install

# Build alle Frontends
frontend-build:
    pnpm run build

# TypeScript Check für alle Frontends
frontend-check:
    pnpm run check

# Lint alle Frontends
frontend-lint:
    pnpm run lint

# Console-spezifische Befehle
console-only:
    cd {{WORKSPACE_ROOT}}/frontend/console && pnpm run dev

console-build:
    cd {{WORKSPACE_ROOT}}/frontend/console && pnpm run build

console-install:
    cd {{WORKSPACE_ROOT}}/frontend/console && pnpm install

console-preview: console-build
    cd {{WORKSPACE_ROOT}}/frontend/console && pnpm run preview

# Platform-spezifische Befehle
platform-only:
    cd {{WORKSPACE_ROOT}}/frontend/platform && pnpm run dev

platform-build:
    cd {{WORKSPACE_ROOT}}/frontend/platform && pnpm run build

platform-install:
    cd {{WORKSPACE_ROOT}}/frontend/platform && pnpm install

platform-preview: platform-build
    cd {{WORKSPACE_ROOT}}/frontend/platform && pnpm run preview

# Docs-spezifische Befehle
docs-only:
    cd {{WORKSPACE_ROOT}}/frontend/docs && pnpm run dev

docs-build:
    cd {{WORKSPACE_ROOT}}/frontend/docs && pnpm run build

docs-install:
    cd {{WORKSPACE_ROOT}}/frontend/docs && pnpm install

docs-preview: docs-build
    cd {{WORKSPACE_ROOT}}/frontend/docs && pnpm run preview

# ═══════════════════════════════════════════════════════
# 🧹 CLEANUP
# ═══════════════════════════════════════════════════════

# Clean Backend
clean-backend:
    cd {{WORKSPACE_ROOT}}/backend && cargo clean

# Clean Frontend Build-Artifakte
clean-frontend:
    rm -rf {{WORKSPACE_ROOT}}/frontend/console/.svelte-kit {{WORKSPACE_ROOT}}/frontend/console/dist
    rm -rf {{WORKSPACE_ROOT}}/frontend/platform/.svelte-kit {{WORKSPACE_ROOT}}/frontend/platform/dist
    rm -rf {{WORKSPACE_ROOT}}/frontend/docs/.svelte-kit {{WORKSPACE_ROOT}}/frontend/docs/dist

# Clean Console
clean-console:
    rm -rf {{WORKSPACE_ROOT}}/frontend/console/.svelte-kit {{WORKSPACE_ROOT}}/frontend/console/dist

# Clean Platform
clean-platform:
    rm -rf {{WORKSPACE_ROOT}}/frontend/platform/.svelte-kit {{WORKSPACE_ROOT}}/frontend/platform/dist

# Clean Docs
clean-docs:
    rm -rf {{WORKSPACE_ROOT}}/frontend/docs/.svelte-kit {{WORKSPACE_ROOT}}/frontend/docs/dist

# Clean All (ohne Volumes)
clean: clean-backend clean-frontend
    rm -f result
    rm -rf {{WORKSPACE_ROOT}}/.turbo
    cd {{WORKSPACE_ROOT}}/infra/docker && docker compose --profile auth down 2>/dev/null || true

# Reset Alles (inkl. Volumes und Daten)
reset:
    #!/usr/bin/env bash
    set -e
    echo "⚠️  Lösche alle Daten, Container, Volumes und Build-Artifakte..."
    echo ""
    
    # 1. Docker Container & Volumes
    echo "━━━ [1/6] Docker Container & Volumes ━━━"
    cd {{WORKSPACE_ROOT}}/infra/docker
    docker compose --profile auth down -v 2>/dev/null || true
    echo "  ✓ Container gestoppt und Volumes entfernt"
    
    # 2. Zusätzliche Volumes
    echo ""
    echo "━━━ [2/6] Zusätzliche Docker Volumes ━━━"
    VOLUMES=$(docker volume ls -q 2>/dev/null | grep -E "^(erynoa|godstack)-" || true)
    if [ -n "$VOLUMES" ]; then
        echo "$VOLUMES" | xargs -r docker volume rm 2>/dev/null || true
        echo "  ✓ Zusätzliche Volumes entfernt"
    else
        echo "  ✓ Keine zusätzlichen Volumes gefunden"
    fi
    
    # 3. Setup-Dateien
    echo ""
    echo "━━━ [3/6] Setup-Dateien ━━━"
    rm -rf {{WORKSPACE_ROOT}}/.data/
    echo "  ✓ .data/ Verzeichnis gelöscht"
    
    # 4. Frontend Build-Artifakte
    echo ""
    echo "━━━ [4/6] Frontend Build-Artifakte ━━━"
    rm -rf {{WORKSPACE_ROOT}}/frontend/console/.svelte-kit {{WORKSPACE_ROOT}}/frontend/console/dist
    rm -rf {{WORKSPACE_ROOT}}/frontend/platform/.svelte-kit {{WORKSPACE_ROOT}}/frontend/platform/dist
    rm -rf {{WORKSPACE_ROOT}}/frontend/docs/.svelte-kit {{WORKSPACE_ROOT}}/frontend/docs/dist
    echo "  ✓ Frontend Build-Artifakte gelöscht"
    
    # 5. Backend Build-Artifakte
    echo ""
    echo "━━━ [5/6] Backend Build-Artifakte ━━━"
    cd {{WORKSPACE_ROOT}}/backend
    cargo clean 2>/dev/null || true
    echo "  ✓ Backend target/ Verzeichnis gelöscht"
    
    # 6. Cache & Build-Artifakte
    echo ""
    echo "━━━ [6/6] Cache & Build-Artifakte ━━━"
    rm -rf {{WORKSPACE_ROOT}}/.turbo
    rm -f {{WORKSPACE_ROOT}}/result
    echo "  ✓ Turbo Cache und Nix Build-Artifakte gelöscht"
    
    echo ""
    echo "✅ Reset vollständig abgeschlossen!"
    echo "   Starte mit: just dev"

# Cleanup Ports
cleanup:
    @{{WORKSPACE_ROOT}}/scripts/dev/cleanup-ports.sh

# ═══════════════════════════════════════════════════════
# 🔄 INITIALISATION
# ═══════════════════════════════════════════════════════

# Initialisierung ohne Dev-Server
init:
    #!/usr/bin/env bash
    set -e
    echo "🔧 Initialisiere Erynoa..."
    
    # Erstelle .env aus .env.example falls nicht vorhanden
    cd {{WORKSPACE_ROOT}}
    if [ ! -f ".env" ]; then
        if [ -f ".env.example" ]; then
            cp .env.example .env
            echo "✅ .env erstellt aus .env.example"
        else
            echo "⚠️  .env.example nicht gefunden, bitte manuell .env erstellen"
        fi
    else
        echo "✓ .env existiert bereits"
    fi
    
    cd {{WORKSPACE_ROOT}}/infra/docker
    docker compose --profile auth up -d db cache minio zitadel-db zitadel-init zitadel
    
    echo "⏳ Warte auf Services..."
    sleep 15
    
    cd {{WORKSPACE_ROOT}}
    mkdir -p .data
    
    if [ ! -f ".data/.minio-setup-complete" ]; then
        chmod +x scripts/infra/setup-minio.sh
        ./scripts/infra/setup-minio.sh || true
    fi
    
    if [ ! -f ".data/zitadel-setup-complete" ]; then
        echo "⏳ Warte auf ZITADEL..."
        sleep 20
        chmod +x scripts/infra/setup-zitadel.sh
        ./scripts/infra/setup-zitadel.sh || true
    fi
    
    echo ""
    echo "✅ Initialisierung abgeschlossen!"
    echo "   Starte mit: just dev"

# Erstelle .env aus .env.example (für Neuaufstellung)
init-env:
    #!/usr/bin/env bash
    set -e
    cd {{WORKSPACE_ROOT}}
    
    if [ -f ".env" ]; then
        echo "⚠️  .env existiert bereits"
        echo "   Zum Überschreiben: rm .env && just init-env"
        exit 1
    fi
    
    if [ ! -f ".env.example" ]; then
        echo "❌ .env.example nicht gefunden!"
        exit 1
    fi
    
    cp .env.example .env
    echo "✅ .env erstellt aus .env.example"
    echo "   Du kannst jetzt .env nach Bedarf anpassen"

# Alias für dev
start: dev

# ═══════════════════════════════════════════════════════
# 🧪 CI WORKFLOW SIMULATION
# ═══════════════════════════════════════════════════════

# Simuliert alle CI-Workflows lokal und listet Fehler auf
# Usage: just test-ci
test-ci:
    #!/usr/bin/env bash
    set +e  # Don't exit on error, we want to collect all errors
    cd {{WORKSPACE_ROOT}}
    
    ERRORS=0
    WARNINGS=0
    ERROR_LIST=()
    WARNING_LIST=()
    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT
    
    echo "🧪 Simulating CI Workflows..."
    echo "═══════════════════════════════════════════════════════"
    echo ""
    
    # ────────────────────────────────────────────────────────────
    # Protobuf Checks
    # ────────────────────────────────────────────────────────────
    echo "📋 Testing Protobuf Lint..."
    if command -v buf &> /dev/null; then
        if buf lint > "$TEMP_DIR/proto-lint.log" 2>&1; then
            echo "✅ Protobuf lint: PASSED"
        else
            echo "❌ Protobuf lint: FAILED"
            ERROR_LIST+=("Protobuf lint - see $TEMP_DIR/proto-lint.log")
            cat "$TEMP_DIR/proto-lint.log"
            ERRORS=$((ERRORS + 1))
        fi
    else
        echo "⚠️  buf not found, skipping protobuf lint"
        WARNING_LIST+=("buf not found")
        WARNINGS=$((WARNINGS + 1))
    fi
    
    echo ""
    echo "📋 Testing Protobuf Format..."
    if command -v buf &> /dev/null; then
        if buf format --diff --exit-code > "$TEMP_DIR/proto-fmt.log" 2>&1; then
            echo "✅ Protobuf format: PASSED"
        else
            echo "❌ Protobuf format: FAILED"
            ERROR_LIST+=("Protobuf format - see $TEMP_DIR/proto-fmt.log")
            cat "$TEMP_DIR/proto-fmt.log"
            ERRORS=$((ERRORS + 1))
        fi
    else
        echo "⚠️  buf not found, skipping protobuf format"
        WARNINGS=$((WARNINGS + 1))
    fi
    
    echo ""
    
    # ────────────────────────────────────────────────────────────
    # Backend Checks
    # ────────────────────────────────────────────────────────────
    echo "🦀 Testing Backend Format..."
    cd backend
    
    # Generate protobuf code first (needed for fmt check)
    if command -v buf &> /dev/null && [ -f "../buf.gen.yaml" ]; then
        echo "   Generating protobuf code..."
        cd ..
        buf generate > "$TEMP_DIR/buf-gen.log" 2>&1 || true
        cd backend
    fi
    
    if command -v cargo &> /dev/null && cargo fmt --version &> /dev/null 2>&1; then
        if SQLX_OFFLINE=true cargo fmt --all -- --check > "$TEMP_DIR/backend-fmt.log" 2>&1; then
            echo "✅ Backend format: PASSED"
        else
            echo "❌ Backend format: FAILED"
            ERROR_LIST+=("Backend format - see $TEMP_DIR/backend-fmt.log")
            cat "$TEMP_DIR/backend-fmt.log"
            ERRORS=$((ERRORS + 1))
        fi
    else
        echo "⚠️  cargo/rustfmt not found, skipping backend format"
        WARNING_LIST+=("cargo/rustfmt not found")
        WARNINGS=$((WARNINGS + 1))
    fi
    
    echo ""
    echo "🦀 Testing Backend Clippy..."
    if command -v cargo &> /dev/null; then
        if cargo clippy --version &> /dev/null 2>&1; then
            if SQLX_OFFLINE=true cargo clippy --all-targets --all-features -- -D warnings > "$TEMP_DIR/backend-clippy.log" 2>&1; then
                echo "✅ Backend clippy: PASSED"
            else
                echo "❌ Backend clippy: FAILED"
                ERROR_LIST+=("Backend clippy - see $TEMP_DIR/backend-clippy.log")
                cat "$TEMP_DIR/backend-clippy.log"
                ERRORS=$((ERRORS + 1))
            fi
        else
            echo "⚠️  clippy not installed, skipping"
            WARNING_LIST+=("clippy not installed")
            WARNINGS=$((WARNINGS + 1))
        fi
    else
        echo "⚠️  cargo not found, skipping backend clippy"
        WARNING_LIST+=("cargo not found")
        WARNINGS=$((WARNINGS + 1))
    fi
    
    echo ""
    echo "🦀 Testing Backend Build..."
    if command -v cargo &> /dev/null; then
        if SQLX_OFFLINE=true cargo check --all-features > "$TEMP_DIR/backend-build.log" 2>&1; then
            echo "✅ Backend build: PASSED"
        else
            echo "❌ Backend build: FAILED"
            ERROR_LIST+=("Backend build - see $TEMP_DIR/backend-build.log")
            cat "$TEMP_DIR/backend-build.log"
            ERRORS=$((ERRORS + 1))
        fi
    else
        echo "⚠️  cargo not found, skipping backend build"
        WARNING_LIST+=("cargo not found")
        WARNINGS=$((WARNINGS + 1))
    fi
    
    cd ..
    echo ""
    
    # ────────────────────────────────────────────────────────────
    # Frontend Checks
    # ────────────────────────────────────────────────────────────
    echo "🎨 Testing Frontend TypeScript Check..."
    if command -v pnpm &> /dev/null; then
        if pnpm run check > "$TEMP_DIR/frontend-check.log" 2>&1; then
            echo "✅ Frontend TypeScript: PASSED"
        else
            echo "❌ Frontend TypeScript: FAILED"
            ERROR_LIST+=("Frontend TypeScript - see $TEMP_DIR/frontend-check.log")
            cat "$TEMP_DIR/frontend-check.log"
            ERRORS=$((ERRORS + 1))
        fi
    else
        echo "⚠️  pnpm not found, skipping frontend check"
        WARNING_LIST+=("pnpm not found")
        WARNINGS=$((WARNINGS + 1))
    fi
    
    echo ""
    echo "🎨 Testing Frontend Lint..."
    if command -v pnpm &> /dev/null; then
        if pnpm run lint > "$TEMP_DIR/frontend-lint.log" 2>&1; then
            echo "✅ Frontend lint: PASSED"
        else
            echo "❌ Frontend lint: FAILED"
            ERROR_LIST+=("Frontend lint - see $TEMP_DIR/frontend-lint.log")
            cat "$TEMP_DIR/frontend-lint.log"
            ERRORS=$((ERRORS + 1))
        fi
    else
        echo "⚠️  pnpm not found, skipping frontend lint"
        WARNINGS=$((WARNINGS + 1))
    fi
    
    echo ""
    echo "🎨 Testing Frontend Build..."
    if command -v pnpm &> /dev/null; then
        if pnpm run build > "$TEMP_DIR/frontend-build.log" 2>&1; then
            echo "✅ Frontend build: PASSED"
        else
            echo "❌ Frontend build: FAILED"
            ERROR_LIST+=("Frontend build - see $TEMP_DIR/frontend-build.log")
            cat "$TEMP_DIR/frontend-build.log"
            ERRORS=$((ERRORS + 1))
        fi
    else
        echo "⚠️  pnpm not found, skipping frontend build"
        WARNINGS=$((WARNINGS + 1))
    fi
    
    echo ""
    echo "═══════════════════════════════════════════════════════"
    echo "📊 Summary:"
    echo "   Errors:   $ERRORS"
    echo "   Warnings: $WARNINGS"
    echo "═══════════════════════════════════════════════════════"
    
    if [ ${#ERROR_LIST[@]} -gt 0 ]; then
        echo ""
        echo "❌ Errors found:"
        for error in "${ERROR_LIST[@]}"; do
            echo "   - $error"
        done
    fi
    
    if [ ${#WARNING_LIST[@]} -gt 0 ]; then
        echo ""
        echo "⚠️  Warnings:"
        for warning in "${WARNING_LIST[@]}"; do
            echo "   - $warning"
        done
    fi
    
    echo ""
    
    # ────────────────────────────────────────────────────────────
    # Optional: Backend Tests (benötigt laufende Services)
    # ────────────────────────────────────────────────────────────
    echo "🧪 Testing Backend Tests (optional - requires services)..."
    if command -v cargo &> /dev/null; then
        # Check if services are running
        if pg_isready -h localhost -p 5432 -U test > /dev/null 2>&1 && \
           redis-cli -h localhost -p 6379 ping > /dev/null 2>&1; then
            echo "   Services detected, running tests..."
            cd backend
            if SQLX_OFFLINE=true cargo test --lib --no-run > "$TEMP_DIR/backend-tests.log" 2>&1; then
                echo "✅ Backend tests compile: PASSED"
            else
                echo "❌ Backend tests compile: FAILED"
                ERROR_LIST+=("Backend tests compile - see $TEMP_DIR/backend-tests.log")
                cat "$TEMP_DIR/backend-tests.log"
                ERRORS=$((ERRORS + 1))
            fi
            cd ..
        else
            echo "⚠️  Services not running, skipping tests"
            echo "   Start services with: just services"
            WARNING_LIST+=("Tests skipped - services not running")
            WARNINGS=$((WARNINGS + 1))
        fi
    else
        echo "⚠️  cargo not found, skipping tests"
        WARNINGS=$((WARNINGS + 1))
    fi
    
    echo ""
    echo "═══════════════════════════════════════════════════════"
    echo "📊 Final Summary:"
    echo "   Errors:   $ERRORS"
    echo "   Warnings: $WARNINGS"
    echo "═══════════════════════════════════════════════════════"
    
    if [ ${#ERROR_LIST[@]} -gt 0 ]; then
        echo ""
        echo "❌ Errors found:"
        for error in "${ERROR_LIST[@]}"; do
            echo "   - $error"
        done
    fi
    
    if [ ${#WARNING_LIST[@]} -gt 0 ]; then
        echo ""
        echo "⚠️  Warnings:"
        for warning in "${WARNING_LIST[@]}"; do
            echo "   - $warning"
        done
    fi
    
    echo ""
    echo "📁 Error logs saved in: $TEMP_DIR"
    echo "   (Logs will be cleaned up on exit)"
    echo ""
    echo "💡 Tip: To view logs before they're cleaned up:"
    echo "   ls -la $TEMP_DIR"
    
    if [ $ERRORS -gt 0 ]; then
        echo ""
        echo "❌ CI simulation failed with $ERRORS error(s)"
        exit 1
    elif [ $WARNINGS -gt 0 ]; then
        echo ""
        echo "⚠️  CI simulation completed with $WARNINGS warning(s)"
        exit 0
    else
        echo ""
        echo "✅ All CI checks passed!"
        exit 0
    fi

# ═══════════════════════════════════════════════════════
# 🧪 TESTING
# ═══════════════════════════════════════════════════════

# Entfernt den DevContainer komplett (Container, Volumes, Images)
# Usage: just devcontainer-remove [mode]
#   just devcontainer-remove        → Container stoppen und entfernen
#   just devcontainer-remove volumes → Zusätzlich Volumes löschen
#   just devcontainer-remove images → Zusätzlich Images löschen
#   just devcontainer-remove all    → Alles löschen (Volumes + Images)
devcontainer-remove mode="":
    #!/usr/bin/env bash
    set -e
    echo "🗑️  Entferne DevContainer..."
    
    cd {{WORKSPACE_ROOT}}/.devcontainer
    
    # Prüfe, ob Container laufen
    if docker compose ps -q dev >/dev/null 2>&1; then
        echo "  ⏳ Stoppe DevContainer..."
        docker compose down || true
        echo "  ✅ DevContainer gestoppt"
    else
        echo "  ℹ️  DevContainer läuft nicht"
    fi
    
    # Entferne Container
    echo "  🗑️  Entferne Container..."
    docker compose rm -f dev 2>/dev/null || true
    
    MODE=$(echo "{{mode}}" | tr '[:upper:]' '[:lower:]')
    
    # Optionale Volumes löschen
    if [ "$MODE" = "volumes" ] || [ "$MODE" = "all" ]; then
        echo "  🗑️  Entferne Volumes..."
        docker compose down -v 2>/dev/null || true
        echo "  ✅ Volumes entfernt"
    fi
    
    # Optionale Images löschen
    if [ "$MODE" = "images" ] || [ "$MODE" = "all" ]; then
        echo "  🗑️  Entferne Images..."
        docker compose down --rmi local 2>/dev/null || true
        echo "  ✅ Images entfernt"
    fi
    
    echo ""
    echo "✅ DevContainer entfernt!"
    echo "   Neu erstellen: DevContainer in VS Code/Cursor öffnen"

# Alias für devcontainer-remove
devcontainer-clean: devcontainer-remove

# Löscht ALLE Container, Volumes und Images (kompletter Cleanup)
# Usage: just docker-cleanup [mode]
#   just docker-cleanup        → Alle Container und Volumes (Standard)
#   just docker-cleanup images → Zusätzlich alle Images löschen
#   just docker-cleanup all    → Alles löschen (inkl. Images)
docker-cleanup mode="":
    #!/usr/bin/env bash
    set -e
    echo "🗑️  Lösche alle Docker-Ressourcen..."
    echo ""
    
    # 1. Services aus infra/docker stoppen und entfernen
    echo "━━━ [1/4] Services (infra/docker) ━━━"
    cd {{WORKSPACE_ROOT}}/infra/docker
    if docker compose ps -q 2>/dev/null | grep -q .; then
        echo "  ⏳ Stoppe alle Services..."
        docker compose --profile auth down -v 2>/dev/null || true
        echo "  ✅ Services gestoppt und Volumes entfernt"
    else
        echo "  ℹ️  Keine Services laufen"
    fi
    
    # 2. DevContainer stoppen und entfernen
    echo ""
    echo "━━━ [2/4] DevContainer ━━━"
    cd {{WORKSPACE_ROOT}}/.devcontainer
    if docker compose ps -q dev >/dev/null 2>&1; then
        echo "  ⏳ Stoppe DevContainer..."
        docker compose down -v 2>/dev/null || true
        echo "  ✅ DevContainer gestoppt und Volumes entfernt"
    else
        echo "  ℹ️  DevContainer läuft nicht"
    fi
    
    # 3. Zusätzliche Volumes (falls vorhanden)
    echo ""
    echo "━━━ [3/4] Zusätzliche Volumes ━━━"
    VOLUMES=$(docker volume ls -q 2>/dev/null | grep -E "^(erynoa|erynoa-dev|godstack)-" || true)
    if [ -n "$VOLUMES" ]; then
        echo "  🗑️  Entferne zusätzliche Volumes..."
        echo "$VOLUMES" | xargs -r docker volume rm 2>/dev/null || true
        echo "  ✅ Zusätzliche Volumes entfernt"
    else
        echo "  ℹ️  Keine zusätzlichen Volumes gefunden"
    fi
    
    # 4. Optionale Images löschen
    MODE=$(echo "{{mode}}" | tr '[:upper:]' '[:lower:]')
    if [ "$MODE" = "images" ] || [ "$MODE" = "all" ]; then
        echo ""
        echo "━━━ [4/4] Images ━━━"
        
        # Images aus docker-compose.yml finden und löschen
        cd {{WORKSPACE_ROOT}}/infra/docker
        IMAGES=$(docker compose config --images 2>/dev/null || true)
        if [ -n "$IMAGES" ]; then
            echo "  🗑️  Entferne Service-Images..."
            echo "$IMAGES" | xargs -r docker rmi -f 2>/dev/null || true
        fi
        
        cd {{WORKSPACE_ROOT}}/.devcontainer
        DEV_IMAGES=$(docker compose config --images 2>/dev/null || true)
        if [ -n "$DEV_IMAGES" ]; then
            echo "  🗑️  Entferne DevContainer-Images..."
            echo "$DEV_IMAGES" | xargs -r docker rmi -f 2>/dev/null || true
        fi
        
        # Alle Container stoppen (falls noch welche laufen)
        echo "  🗑️  Stoppe alle laufenden Container..."
        docker stop $(docker ps -q) 2>/dev/null || true
        
        echo "  ✅ Images entfernt"
    else
        echo ""
        echo "━━━ [4/4] Images ━━━"
        echo "  ℹ️  Images werden nicht gelöscht (nutze 'just docker-cleanup images')"
    fi
    
    echo ""
    echo "✅ Docker-Cleanup abgeschlossen!"
    echo ""
    echo "📊 Verbleibende Ressourcen:"
    echo "   Container: $(docker ps -a -q 2>/dev/null | wc -l | tr -d ' ')"
    echo "   Volumes:   $(docker volume ls -q 2>/dev/null | wc -l | tr -d ' ')"
    echo "   Images:    $(docker images -q 2>/dev/null | wc -l | tr -d ' ')"
    echo ""
    echo "💡 Zum kompletten Reset (inkl. Build-Artifakte): just reset"

# Testet den DevContainer Build und Services
# Usage: just test-devcontainer
test-devcontainer:
    #!/usr/bin/env bash
    set -e
    cd {{WORKSPACE_ROOT}}/.devcontainer
    
    echo "🔨 Building DevContainer..."
    docker compose build dev || (echo "❌ DevContainer build failed" && exit 1)
    echo "✅ DevContainer built successfully"
    
    echo ""
    echo "🚀 Starting services on host..."
    cd {{WORKSPACE_ROOT}}/infra/docker
    docker compose up -d db cache minio || (echo "❌ Failed to start services" && exit 1)
    
    echo "⏳ Waiting for services to be ready..."
    timeout 60 bash -c 'until docker compose exec -T db pg_isready -U erynoa; do sleep 2; done' || (echo "❌ PostgreSQL not ready" && exit 1)
    timeout 60 bash -c 'until docker compose exec -T cache redis-cli ping; do sleep 2; done' || (echo "❌ Redis not ready" && exit 1)
    timeout 60 bash -c 'until curl -f http://localhost:9000/minio/health/live 2>/dev/null; do sleep 2; done' || (echo "❌ MinIO not ready" && exit 1)
    echo "✅ All services ready"
    
    echo ""
    echo "🔍 Testing installed tools..."
    cd {{WORKSPACE_ROOT}}/.devcontainer
    docker compose run --rm dev bash -c "
        set -e
        echo 'Testing Rust...'
        rustc --version || (echo '❌ rustc not found' && exit 1)
        cargo --version || (echo '❌ cargo not found' && exit 1)
        echo '✅ Rust tools OK'
        
        echo 'Testing Node.js & pnpm...'
        node --version || (echo '❌ node not found' && exit 1)
        pnpm --version || (echo '❌ pnpm not found' && exit 1)
        echo '✅ Node.js & pnpm OK'
        
        echo 'Testing buf...'
        buf --version || (echo '❌ buf not found' && exit 1)
        echo '✅ buf OK'
        
        echo 'Testing just...'
        just --version || (echo '❌ just not found' && exit 1)
        echo '✅ just OK'
        
        echo 'Testing direnv...'
        direnv --version || (echo '❌ direnv not found' && exit 1)
        echo '✅ direnv OK'
        
        echo 'Testing Docker...'
        docker --version || (echo '❌ docker not found' && exit 1)
        echo '✅ Docker OK'
        
        echo ''
        echo '✅ All tools installed correctly'
    " || (echo "❌ Tool test failed" && exit 1)
    
    echo ""
    echo "🔍 Testing service connections from DevContainer..."
    cd {{WORKSPACE_ROOT}}/.devcontainer
    docker compose run --rm dev bash -c "
        set -e
        echo 'Testing PostgreSQL (via host.docker.internal)...'
        PGPASSWORD=erynoa psql -h host.docker.internal -U erynoa -d erynoa -c 'SELECT version();' > /dev/null || (echo '❌ PostgreSQL connection failed' && exit 1)
        echo '✅ PostgreSQL connection OK'
        
        echo 'Testing Redis (via host.docker.internal)...'
        redis-cli -h host.docker.internal ping > /dev/null || (echo '❌ Redis connection failed' && exit 1)
        echo '✅ Redis connection OK'
        
        echo 'Testing MinIO (via host.docker.internal)...'
        curl -f http://host.docker.internal:9000/minio/health/live > /dev/null || (echo '❌ MinIO connection failed' && exit 1)
        echo '✅ MinIO connection OK'
        
        echo ''
        echo '✅ All services accessible from DevContainer'
    " || (echo "❌ Service connection test failed" && exit 1)
    
    echo ""
    echo "🔍 Testing backend build..."
    docker compose run --rm dev bash -c "
        set -e
        cd /workspace/backend
        cargo check --all-features || (echo '❌ Backend build failed' && exit 1)
        echo '✅ Backend builds successfully'
    " || (echo "❌ Backend build test failed" && exit 1)
    
    echo ""
    echo "🔍 Testing frontend setup..."
    docker compose run --rm dev bash -c "
        set -e
        cd /workspace
        pnpm install --frozen-lockfile || (echo '❌ Frontend install failed' && exit 1)
        echo '✅ Frontend dependencies installed'
    " || (echo "❌ Frontend setup test failed" && exit 1)
    
    echo ""
    echo "🔍 Testing protobuf generation..."
    docker compose run --rm dev bash -c "
        set -e
        cd /workspace
        buf generate || (echo '❌ Protobuf generation failed' && exit 1)
        echo '✅ Protobuf generation OK'
    " || (echo "❌ Protobuf generation test failed" && exit 1)
    
    echo ""
    echo "✅ All DevContainer tests passed!"
    echo ""
    echo "To clean up services: cd infra/docker && docker compose down -v"
