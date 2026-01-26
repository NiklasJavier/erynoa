#!/usr/bin/env bash
# Cleanup stuck development processes (only Control + Backend)
# Usage: ./scripts/dev/cleanup-ports.sh

# Don't fail on errors - some processes might not exist
set +e

echo "🧹 Cleaning up stuck development processes..."

# Kill ONLY development-related processes, not Docker services
echo "  Stopping Cargo/Backend processes..."
pkill -f "cargo run" 2>/dev/null || true
pkill -f "cargo watch" 2>/dev/null || true

sleep 0.5

echo "  Stopping Vite/Control processes..."
pkill -f "vite" 2>/dev/null || true
pkill -f "npm run dev" 2>/dev/null || true

sleep 0.5

# Kill processes on backend port ONLY if they're not Docker
echo "  Freeing port 3000 (if stuck)..."
PID=$(lsof -i :3000 -t 2>/dev/null | grep -v "docker\|containerd" | head -1 || true)
if [ ! -z "$PID" ]; then
    echo "  Killing backend process on port 3000 (PID: $PID)"
    kill -9 $PID 2>/dev/null || true
fi

sleep 0.5

echo "  Freeing port 5173/5174 (if stuck)..."
PID=$(lsof -i :5173 -t 2>/dev/null | grep -v "docker\|containerd" | head -1 || true)
[ ! -z "$PID" ] && kill -9 $PID 2>/dev/null || true

PID=$(lsof -i :5174 -t 2>/dev/null | grep -v "docker\|containerd" | head -1 || true)
[ ! -z "$PID" ] && kill -9 $PID 2>/dev/null || true

sleep 1

echo "✅ Cleanup complete! Docker services should still be running."
