#!/bin/bash
# ============================================================================
# Profile-Guided Optimization (PGO) Build Script
# ============================================================================
# Dieses Script erstellt einen PGO-optimierten Production Build.
# PGO kann +10-15% mehr Throughput (Requests/Sec) bringen.
#
# Voraussetzungen:
#   - cargo-pgo installiert: cargo binstall cargo-pgo
#   - k6 oder wrk für Load-Tests (optional, für Profiling)
#
# Verwendung:
#   ./scripts/build/pgo-build.sh
#
# ============================================================================

set -e

echo "🚀 PGO (Profile-Guided Optimization) Build"
echo "==========================================="
echo ""

cd "$(dirname "$0")/../../backend"

# Prüfe ob cargo-pgo installiert ist
if ! command -v cargo-pgo &> /dev/null; then
    echo "❌ cargo-pgo nicht gefunden!"
    echo "   Installation: cargo binstall cargo-pgo -y"
    exit 1
fi

echo "📊 Phase 1: Instrumentierter Build"
echo "   Erstelle Build mit Profiling-Instrumentierung..."
cargo pgo build

echo ""
echo "🏃 Phase 2: Profiling Run"
echo "   Starte Server für Profiling (30 Sekunden)..."
echo "   In einem anderen Terminal: wrk -t4 -c100 -d30s http://localhost:3000/api/v1/health"
echo ""

# Starte den instrumentierten Build im Hintergrund
cargo pgo run &
PID=$!

echo "   Server gestartet (PID: $PID)"
echo "   Warte auf Traffic für Profiling..."
echo ""
echo "   ⚠️  WICHTIG: Generiere jetzt typischen Traffic!"
echo "   Beispiel mit wrk:"
echo "     wrk -t4 -c100 -d30s http://localhost:3000/api/v1/connect/erynoa.v1.HealthService/Check"
echo ""
echo "   Oder mit k6:"
echo "     k6 run --vus 50 --duration 30s scripts/test/load-test.js"
echo ""

# Warte auf User-Input oder Timeout
read -p "Drücke Enter wenn das Profiling abgeschlossen ist..." -t 120 || true

# Stoppe den Server
kill $PID 2>/dev/null || true
wait $PID 2>/dev/null || true

echo ""
echo "🔨 Phase 3: Optimierter Build"
echo "   Kompiliere mit gesammelten Profildaten..."

cargo pgo optimize

echo ""
echo "✅ PGO Build abgeschlossen!"
echo ""
echo "   Der optimierte Build befindet sich in:"
echo "   target/release/erynoa-api"
echo ""
echo "   Erwartete Verbesserung: +10-15% Throughput"
echo ""
