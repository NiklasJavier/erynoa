#!/bin/bash
set -e

echo "🚀 Initializing God-Stack DevContainer..."

# Fix GPG permissions if mounted
if [ -d "$HOME/.gnupg" ]; then
  echo "🔐 Configuring GPG..."
  chmod 700 "$HOME/.gnupg" 2>/dev/null || true
  chmod 600 "$HOME/.gnupg/"* 2>/dev/null || true
  # Configure GPG to use the TTY
  export GPG_TTY=$(tty)
  echo "pinentry-mode loopback" >> "$HOME/.gnupg/gpg.conf" 2>/dev/null || true
fi

# Fix SSH permissions if mounted
if [ -d "$HOME/.ssh" ]; then
  echo "🔑 SSH keys available"
fi

# Start services via Docker-in-Docker
echo "🐳 Starting infrastructure services..."
cd /workspace/.devcontainer
docker compose -f services.yml up -d

# Wait for database to be ready
echo "⏳ Waiting for database..."
until docker compose -f services.yml exec -T db pg_isready -U godstack >/dev/null 2>&1; do
  sleep 2
done
echo "✅ Database ready!"

# Wait for cache to be ready
echo "⏳ Waiting for cache..."
until docker compose -f services.yml exec -T cache redis-cli ping >/dev/null 2>&1; do
  sleep 2
done
echo "✅ Cache ready!"

# Enter nix shell and run migrations
echo "📦 Running database migrations..."
cd /workspace
nix develop --command sqlx migrate run 2>/dev/null || echo "⚠️  Migrations skipped (run manually with: just db-migrate)"

echo ""
echo "✅ DevContainer ready!"
echo ""
echo "📍 Services (running in Docker-in-Docker):"
echo "   Database:  localhost:5432 (OrioleDB)"
echo "   Cache:     localhost:6379 (DragonflyDB)"
echo "   Auth:      http://localhost:8080 (ZITADEL)"
echo ""
echo "🛠️  Commands:"
echo "   just dev        - Start dev server"
echo "   just test       - Run tests"
echo "   just db-migrate - Run migrations"
echo "   just services   - Show service status"
echo ""
echo "🔐 ZITADEL Setup:"
echo "   Open http://localhost:8080"
echo "   Default admin: zitadel-admin@zitadel.localhost"
echo ""
