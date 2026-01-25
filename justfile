# Godstack Monorepo - Justfile

set dotenv-load

default:
    @just --list

# ═══════════════════════════════════════════════════════
# DEVELOPMENT (Container-in-Container)
# ═══════════════════════════════════════════════════════

# [DEFAULT] Dev server - Frontend + Backend mit Hot-Reload in Containern, Services im Hintergrund
dev:
    #!/usr/bin/env bash
    set -e
    
    echo ""
    echo "╔════════════════════════════════════════════════════════════════════╗"
    echo "║     🚀 Godstack Development Environment                            ║"
    echo "╚════════════════════════════════════════════════════════════════════╝"
    echo ""
    echo "  Frontend:  http://localhost:5173  (Vite HMR)"
    echo "  Backend:   http://localhost:3000  (cargo watch)"
    echo "  ZITADEL:   http://localhost:8080  (Auth)"
    echo "  MinIO:     http://localhost:9001  (Storage Console)"
    echo ""
    
    # 1. Starte Hintergrund-Services (DB, Cache, MinIO, ZITADEL)
    echo "━━━ [1/4] Starte Hintergrund-Services ━━━"
    cd /workspace/infra
    docker compose --profile auth up -d db cache minio zitadel-db zitadel-init zitadel
    echo "✓ Hintergrund-Services gestartet"
    
    # 2. Warte auf Services
    echo ""
    echo "━━━ [2/4] Warte auf Services ━━━"
    echo -n "  Warte auf PostgreSQL..."
    until docker compose exec -T db pg_isready -U godstack -h localhost > /dev/null 2>&1; do
        sleep 1
        echo -n "."
    done
    echo " ✓"
    
    echo -n "  Warte auf Redis/Dragonfly..."
    until docker compose exec -T cache redis-cli ping > /dev/null 2>&1; do
        sleep 1
        echo -n "."
    done
    echo " ✓"
    
    echo -n "  Warte auf MinIO..."
    until curl -sf http://localhost:9000/minio/health/live > /dev/null 2>&1; do
        sleep 1
        echo -n "."
    done
    echo " ✓"
    
    echo -n "  Warte auf ZITADEL..."
    for i in {1..60}; do
        if curl -sf http://localhost:8080/debug/ready > /dev/null 2>&1; then
            echo " ✓"
            break
        fi
        if [ $i -eq 60 ]; then
            echo " (Timeout - wird später geprüft)"
        fi
        sleep 2
        echo -n "."
    done
    
    # 3. Initialisierungsskripte (nur wenn nötig)
    echo ""
    echo "━━━ [3/4] Initialisierung ━━━"
    cd /workspace
    
    # MinIO Setup
    if [ ! -f ".data/.minio-setup-complete" ]; then
        echo "  → MinIO Setup wird ausgeführt..."
        chmod +x infra/scripts/setup-minio.sh
        ./infra/scripts/setup-minio.sh || echo "  ⚠ MinIO Setup übersprungen"
    else
        echo "  ✓ MinIO bereits eingerichtet"
    fi
    
    # ZITADEL Setup
    if [ ! -f ".data/zitadel-setup-complete" ]; then
        echo "  → ZITADEL Setup wird ausgeführt..."
        chmod +x infra/scripts/setup-zitadel.sh
        ./infra/scripts/setup-zitadel.sh || echo "  ⚠ ZITADEL Setup übersprungen"
    else
        echo "  ✓ ZITADEL bereits eingerichtet"
        if [ -f ".data/zitadel-client-id" ]; then
            echo "    Client-ID: $(cat .data/zitadel-client-id)"
        fi
    fi
    
    # 4. Starte Frontend + Backend mit sichtbaren Logs
    echo ""
    echo "━━━ [4/4] Starte Frontend + Backend (Hot-Reload) ━━━"
    echo ""
    echo "  Ctrl+C stoppt Frontend & Backend, Services laufen weiter."
    echo "  Komplett stoppen: just docker-stop"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    cd /workspace/infra
    # Trap Ctrl+C um eine saubere Nachricht anzuzeigen
    trap 'echo ""; echo ""; echo "━━━ Frontend + Backend gestoppt ━━━"; echo "  Services laufen weiter. Status: just status"; echo "  Neustart: just dev"; echo ""' INT
    docker compose up --build frontend backend
    
# Dev ohne ZITADEL (minimal)
dev-minimal:
    #!/usr/bin/env bash
    set -e
    echo "🚀 Starte minimalen Dev-Stack (ohne Auth)..."
    cd /workspace/infra
    docker compose up -d db cache minio
    sleep 5
    docker compose up --build frontend backend

# Nur Backend mit Hot-Reload (Services müssen laufen)
dev-backend:
    cd /workspace/infra && docker compose up --build backend

# Nur Frontend mit Hot-Reload (Services müssen laufen)  
dev-frontend:
    cd /workspace/infra && docker compose up --build frontend

# ═══════════════════════════════════════════════════════
# DOCKER SERVICES
# ═══════════════════════════════════════════════════════

# Baue Docker Images ohne zu starten
docker-build:
    cd /workspace/infra && docker compose build

# Stoppt alle Container
docker-stop:
    cd /workspace/infra && docker compose --profile auth down

# Logs anzeigen (alle)
docker-logs:
    cd /workspace/infra && docker compose --profile auth logs -f

# Backend-Logs anzeigen
docker-logs-backend:
    cd /workspace/infra && docker compose logs -f backend

# Frontend-Logs anzeigen
docker-logs-frontend:
    cd /workspace/infra && docker compose logs -f frontend

# Shell in Container
docker-backend-shell:
    cd /workspace/infra && docker compose exec backend bash

docker-frontend-shell:
    cd /workspace/infra && docker compose exec frontend sh

# ─────────────────────────────────────────────────────────────────────────────
# Protobuf / Connect-RPC
# ─────────────────────────────────────────────────────────────────────────────

# Generate TypeScript from Proto files
proto-gen:
    @echo "🔧 Generating TypeScript from Protobuf..."
    buf generate
    @echo "✅ Generated files in frontend/src/gen/"

# Lint proto files
proto-lint:
    buf lint

# Format proto files
proto-fmt:
    buf format -w

# Clippy lint
lint:
    cd backend && cargo clippy -- -D warnings

# Format code
fmt:
    cd backend && cargo fmt

# Quick check
check:
    cd backend && cargo check

# Run tests
test:
    cd backend && cargo test

# All checks
ci: fmt lint test

# Nix checks (clippy + fmt + build)
ci-nix:
    nix flake check

# ─────────────────────────────────────────────────────
# Build (Nix)
# ─────────────────────────────────────────────────────

# Build with Nix (default)
build:
    nix build

# Build static musl binary
build-static:
    nix build .#static

# Build Docker image via Nix
build-docker:
    nix build .#docker
    @echo "Image: ./result (load with: docker load < result)"

# Build and load Docker image
docker-load:
    nix build .#docker
    docker load < result

# ─────────────────────────────────────────────────────
# Database
# ─────────────────────────────────────────────────────

# Run migrations
db-migrate:
    cd backend && sqlx migrate run

# Create migration
db-new name:
    cd backend && sqlx migrate add {{name}}

# Prepare for offline
db-prepare:
    cd backend && cargo sqlx prepare

# Reset database
db-reset:
    cd backend && sqlx database drop -y || true
    cd backend && sqlx database create
    cd backend && sqlx migrate run

# Clean up stuck development processes
cleanup:
    @./infra/scripts/cleanup-ports.sh

# ═══════════════════════════════════════════════════════
# FRONTEND
# ═══════════════════════════════════════════════════════

# Frontend dev server (standalone)
frontend-only:
    cd frontend && npm run dev

# Frontend dev with Backend dependency
frontend-dev:
    just docker-dev

# Frontend build
frontend-build:
    cd frontend && npm run build

# Frontend install dependencies
frontend-install:
    cd frontend && npm install

# Frontend preview production build
frontend-preview: frontend-build
    cd frontend && npm run preview

# ═══════════════════════════════════════════════════════
# INFRASTRUCTURE
# ═══════════════════════════════════════════════════════

# Start infrastructure (DB + Cache + MinIO + ZITADEL) im Hintergrund
services:
    cd /workspace/infra && docker compose --profile auth up -d db cache minio zitadel-db zitadel-init zitadel

# Start infrastructure without ZITADEL
services-minimal:
    cd /workspace/infra && docker compose up -d db cache minio

# Stop services
services-down:
    cd /workspace/infra && docker compose --profile auth down

# View service logs
services-logs service="":
    cd /workspace/infra && docker compose --profile auth logs -f {{service}}

# Restart services
services-restart:
    cd /workspace/infra && docker compose --profile auth restart

# Service status
services-ps:
    cd /workspace/infra && docker compose --profile auth ps -a

# Restart nur Frontend + Backend (schneller als alles)
restart-dev:
    cd /workspace/infra && docker compose restart frontend backend

# ═══════════════════════════════════════════════════════
# MINIO / S3 STORAGE
# ═══════════════════════════════════════════════════════

# MinIO Setup (Buckets + Policies)
minio-setup:
    @chmod +x /workspace/infra/scripts/setup-minio.sh
    @/workspace/infra/scripts/setup-minio.sh

# Open MinIO Console
minio:
    @echo "Opening MinIO Console..."
    @echo "Login: godstack / godstack123"
    @echo ""
    @$BROWSER "http://localhost:9001" || echo "Öffne: http://localhost:9001"

# MinIO reset (löscht alle Daten)
minio-reset:
    cd /workspace/infra && docker compose stop minio
    cd /workspace/infra && docker compose rm -f minio
    docker volume rm godstack-services_minio-data 2>/dev/null || true
    rm -f /workspace/.data/.minio-setup-complete /workspace/.data/.minio-credentials
    cd /workspace/infra && docker compose up -d minio
    @echo "Warte 10 Sekunden..."
    @sleep 10
    @just minio-setup

# ═══════════════════════════════════════════════════════
# FULL STACK
# ═══════════════════════════════════════════════════════

# Alias für dev
start: dev

# Initialisierung ohne Dev-Server zu starten
init:
    #!/usr/bin/env bash
    set -e
    echo "🔧 Initialisiere Godstack..."
    
    # Starte Services
    cd /workspace/infra
    docker compose --profile auth up -d db cache minio zitadel-db zitadel-init zitadel
    
    echo "⏳ Warte auf Services..."
    sleep 15
    
    # MinIO Setup
    cd /workspace
    if [ ! -f ".data/.minio-setup-complete" ]; then
        chmod +x infra/scripts/setup-minio.sh
        ./infra/scripts/setup-minio.sh || true
    fi
    
    # ZITADEL Setup
    if [ ! -f ".data/zitadel-setup-complete" ]; then
        echo "⏳ Warte auf ZITADEL..."
        sleep 20
        chmod +x infra/scripts/setup-zitadel.sh
        ./infra/scripts/setup-zitadel.sh || true
    fi
    
    echo ""
    echo "✅ Initialisierung abgeschlossen!"
    echo "   Starte mit: just dev"

# ═══════════════════════════════════════════════════════
# ZITADEL
# ═══════════════════════════════════════════════════════

# Open ZITADEL Console
zitadel:
    @echo "Opening ZITADEL Console..."
    @echo "Login: zitadel-admin / Password1!"
    @echo "Test User: testuser / Test123!"
    @echo ""
    @$BROWSER "http://localhost:8080/ui/console" || echo "Öffne: http://localhost:8080/ui/console"

# ZITADEL setup guide
zitadel-guide:
    @cat /workspace/docs/ZITADEL_SETUP.md

# ZITADEL automatisches Setup (Projekt + Apps + Test-User)
zitadel-setup:
    @chmod +x /workspace/infra/scripts/setup-zitadel.sh
    @/workspace/infra/scripts/setup-zitadel.sh

# ZITADEL reset (löscht alle Daten und startet neu)
zitadel-reset:
    cd /workspace/infra && docker compose --profile auth stop zitadel zitadel-db
    cd /workspace/infra && docker compose --profile auth rm -f zitadel zitadel-db
    docker volume rm godstack-services_zitadel-pgdata godstack-services_zitadel-machinekey 2>/dev/null || true
    rm -f /workspace/.data/zitadel-setup-complete /workspace/.data/zitadel-client-id
    cd /workspace/infra && docker compose --profile auth up -d zitadel-db zitadel-init zitadel
    @echo "Warte 30 Sekunden auf Init..."
    @sleep 30
    @curl -sf http://localhost:8080/debug/ready && echo " ✓ ZITADEL bereit" || echo " ⚠ ZITADEL noch nicht bereit"
    @just zitadel-setup

# ═══════════════════════════════════════════════════════
# CLEANUP
# ═══════════════════════════════════════════════════════

# Clean backend
clean-backend:
    cd /workspace/backend && cargo clean

# Clean frontend
clean-frontend:
    rm -rf /workspace/frontend/node_modules /workspace/frontend/dist

# Clean all
clean: clean-backend clean-frontend
    rm -f result
    rm -rf /workspace/.data/
    cd /workspace/infra && docker compose --profile auth down -v 2>/dev/null || true

# Reset alles (Volumes, Setup-Dateien, etc.)
reset:
    #!/usr/bin/env bash
    set -e
    echo "⚠️  Lösche alle Daten und Container..."
    cd /workspace/infra
    docker compose --profile auth down -v 2>/dev/null || true
    rm -rf /workspace/.data/
    echo "✅ Reset abgeschlossen. Starte mit: just dev"

# Status aller Services anzeigen
status:
    #!/usr/bin/env bash
    echo ""
    echo "═══════════════════════════════════════════════════════"
    echo "  Godstack Service Status"
    echo "═══════════════════════════════════════════════════════"
    echo ""
    cd /workspace/infra && docker compose --profile auth ps
    echo ""
    echo "───────────────────────────────────────────────────────"
    echo "  Health Checks:"
    echo "───────────────────────────────────────────────────────"
    curl -sf http://localhost:3000/api/v1/health > /dev/null 2>&1 && echo "  ✓ Backend:   http://localhost:3000" || echo "  ✗ Backend:   nicht erreichbar"
    curl -sf http://localhost:5173/ > /dev/null 2>&1 && echo "  ✓ Frontend:  http://localhost:5173" || echo "  ✗ Frontend:  nicht erreichbar"
    curl -sf http://localhost:8080/debug/ready > /dev/null 2>&1 && echo "  ✓ ZITADEL:   http://localhost:8080" || echo "  ✗ ZITADEL:   nicht erreichbar"
    curl -sf http://localhost:9000/minio/health/live > /dev/null 2>&1 && echo "  ✓ MinIO:     http://localhost:9001 (Console)" || echo "  ✗ MinIO:     nicht erreichbar"
    echo ""
