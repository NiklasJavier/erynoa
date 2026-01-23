#!/bin/bash
set -e

echo "🚀 Initializing God-Stack DevContainer..."

# Wait for services
echo "⏳ Waiting for services..."
sleep 5

# Enter nix shell and run migrations
echo "📦 Running database migrations..."
cd /workspace
nix develop --command sqlx migrate run 2>/dev/null || echo "⚠️  Migrations skipped (run manually with: just db-migrate)"

echo ""
echo "✅ DevContainer ready!"
echo ""
echo "📍 Services:"
echo "   Database:  db:5432 (OrioleDB)"
echo "   Cache:     cache:6379 (DragonflyDB)"
echo "   Auth:      http://zitadel:8080 (ZITADEL)"
echo ""
echo "🛠️  Commands:"
echo "   just dev        - Start dev server"
echo "   just test       - Run tests"
echo "   just db-migrate - Run migrations"
echo ""
echo "🔐 ZITADEL Setup:"
echo "   Open http://localhost:8080"
echo "   Default admin: zitadel-admin@zitadel.localhost"
echo ""
