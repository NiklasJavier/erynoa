#!/bin/bash
set -e

echo "🚀 Initializing God-Stack DevContainer..."

# Setup GPG from host keys
if [ -d "$HOME/.gnupg-host" ]; then
  echo "🔐 Configuring GPG..."
  
  # Create fresh .gnupg directory if needed
  mkdir -p "$HOME/.gnupg"
  chmod 700 "$HOME/.gnupg"
  
  # Copy keys from host (not sockets)
  for f in "$HOME/.gnupg-host/"*.gpg "$HOME/.gnupg-host/"*.kbx "$HOME/.gnupg-host/private-keys-v1.d" "$HOME/.gnupg-host/trustdb.gpg" "$HOME/.gnupg-host/pubring.kbx"; do
    if [ -e "$f" ]; then
      cp -r "$f" "$HOME/.gnupg/" 2>/dev/null || true
    fi
  done
  
  # Copy private keys directory
  if [ -d "$HOME/.gnupg-host/private-keys-v1.d" ]; then
    mkdir -p "$HOME/.gnupg/private-keys-v1.d"
    cp -r "$HOME/.gnupg-host/private-keys-v1.d/"* "$HOME/.gnupg/private-keys-v1.d/" 2>/dev/null || true
  fi
  
  # Copy keyboxd data if using modern GPG
  if [ -d "$HOME/.gnupg-host/public-keys.d" ]; then
    cp -r "$HOME/.gnupg-host/public-keys.d" "$HOME/.gnupg/" 2>/dev/null || true
  fi
  
  # Fix permissions
  chmod 700 "$HOME/.gnupg" 2>/dev/null || true
  find "$HOME/.gnupg" -type f -exec chmod 600 {} \; 2>/dev/null || true
  find "$HOME/.gnupg" -type d -exec chmod 700 {} \; 2>/dev/null || true
fi

# Configure GPG for container use (works for both new and existing .gnupg)
if [ -d "$HOME/.gnupg" ]; then
  # Remove host-specific sockets and locks
  rm -f "$HOME/.gnupg/S."* "$HOME/.gnupg/"*.lock 2>/dev/null || true
  
  # Write Linux-compatible config (overwrite macOS-specific settings)
  cat > "$HOME/.gnupg/gpg.conf" << 'GPGEOF'
use-agent
pinentry-mode loopback
no-tty
GPGEOF

  cat > "$HOME/.gnupg/gpg-agent.conf" << 'AGENTEOF'
allow-loopback-pinentry
allow-preset-passphrase
default-cache-ttl 34560000
max-cache-ttl 34560000
disable-scdaemon
AGENTEOF

  # Restart GPG agent
  gpgconf --kill all 2>/dev/null || true
  gpg-agent --daemon 2>/dev/null || true
  
  # Configure Git to use GPG
  git config --global gpg.program gpg
  
  # Show available keys
  echo "   Available GPG keys:"
  gpg --list-secret-keys --keyid-format=long 2>/dev/null | grep -E "sec|uid" | head -8 || echo "   No keys found"
  
  # Add GPG_TTY to shell profiles for interactive use
  for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
    if [ -f "$rc" ]; then
      grep -q "GPG_TTY" "$rc" || echo 'export GPG_TTY=$(tty)' >> "$rc"
    fi
  done
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
