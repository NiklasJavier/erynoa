#!/bin/bash
set -e

echo "🚀 Initializing God-Stack DevContainer..."

# ─────────────────────────────────────────────────────────────────────────────
# Start Nix daemon (required for multi-user Nix in devcontainer)
# ─────────────────────────────────────────────────────────────────────────────
echo "❄️  Starting Nix daemon..."
if ! pgrep -x "nix-daemon" > /dev/null; then
  sudo /nix/var/nix/profiles/default/bin/nix-daemon &
  sleep 1
  echo "   ✅ Nix daemon started"
else
  echo "   ✅ Nix daemon already running"
fi

# ─────────────────────────────────────────────────────────────────────────────
# Pre-build Nix environment (so it's ready when you open a terminal)
# ─────────────────────────────────────────────────────────────────────────────
echo "📦 Preparing Nix development environment..."
cd /workspace

# Build the dev shell in background - this caches all dependencies
# Next time you enter the shell it will be instant
if [ -f "flake.nix" ]; then
  # Build devShell (downloads/builds all Nix dependencies)
  /nix/var/nix/profiles/default/bin/nix develop --command true 2>/dev/null && echo "   ✅ Nix environment ready" || echo "   ⚠️  Nix environment will be built on first use"
  
  # Ensure direnv is allowed (for automatic activation in terminals)
  if command -v direnv &> /dev/null; then
    direnv allow . 2>/dev/null || true
  fi
fi

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

# Fix SSH permissions and configure Git SSH signing
if [ -d "$HOME/.ssh" ]; then
  echo "🔑 Configuring SSH..."
  
  # Fix permissions for SSH keys (they might be read-only from mount)
  # Note: We work with the mounted keys as-is since they're read-only
  
  # Configure Git SSH signing - translate host paths to container paths
  # Check if Git is configured to use SSH signing with a host-specific path
  CURRENT_SIGNING_KEY=$(git config --global user.signingkey 2>/dev/null || true)
  GPG_FORMAT=$(git config --global gpg.format 2>/dev/null || true)
  
  if [ "$GPG_FORMAT" = "ssh" ] && [ -n "$CURRENT_SIGNING_KEY" ]; then
    # Extract just the filename from the host path
    KEY_BASENAME=$(basename "$CURRENT_SIGNING_KEY")
    CONTAINER_KEY_PATH="$HOME/.ssh/$KEY_BASENAME"
    
    # Check if the key exists in the container's .ssh directory
    if [ -f "$CONTAINER_KEY_PATH" ]; then
      echo "   Updating SSH signing key path: $KEY_BASENAME"
      git config --global user.signingkey "$CONTAINER_KEY_PATH"
    else
      # Try to find any signing key
      for key in "$HOME/.ssh/id_"*"_signing.pub" "$HOME/.ssh/id_"*"_signing"; do
        if [ -f "$key" ]; then
          echo "   Found signing key: $(basename "$key")"
          git config --global user.signingkey "$key"
          break
        fi
      done
    fi
    
    # For SSH signing, Git needs the ssh-keygen binary to sign
    # The private key must be accessible - check if agent has keys or if we need to add them
    SIGNING_KEY=$(git config --global user.signingkey 2>/dev/null || true)
    if [ -n "$SIGNING_KEY" ]; then
      # Get the private key path (remove .pub if present)
      PRIVATE_KEY="${SIGNING_KEY%.pub}"
      
      # Check if SSH agent has identities, if not try to add the signing key
      if ! ssh-add -l >/dev/null 2>&1; then
        # Start ssh-agent if not running
        if [ -z "$SSH_AUTH_SOCK" ]; then
          eval "$(ssh-agent -s)" >/dev/null 2>&1
          echo "   Started SSH agent"
        fi
        
        # Try to add the signing key (this will prompt for passphrase if needed)
        if [ -f "$PRIVATE_KEY" ]; then
          # Note: This may fail silently if key requires passphrase
          # User will be prompted during commit if needed
          ssh-add "$PRIVATE_KEY" 2>/dev/null || echo "   Note: SSH signing key may need passphrase during commit"
        fi
      fi
    fi
  fi
  
  # Fix allowed_signers file path if configured
  CURRENT_ALLOWED_SIGNERS=$(git config --global gpg.ssh.allowedSignersFile 2>/dev/null || true)
  if [ -n "$CURRENT_ALLOWED_SIGNERS" ]; then
    SIGNERS_BASENAME=$(basename "$CURRENT_ALLOWED_SIGNERS")
    CONTAINER_SIGNERS_PATH="$HOME/.ssh/$SIGNERS_BASENAME"
    
    if [ -f "$CONTAINER_SIGNERS_PATH" ]; then
      echo "   Updating allowed_signers path: $SIGNERS_BASENAME"
      git config --global gpg.ssh.allowedSignersFile "$CONTAINER_SIGNERS_PATH"
    fi
  fi
  
  # Show current signing configuration
  echo "   SSH signing configured:"
  echo "     Key: $(git config --global user.signingkey 2>/dev/null || echo 'not set')"
  echo "     Format: $(git config --global gpg.format 2>/dev/null || echo 'not set')"
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

# Ensure DATABASE_URL is set (from containerEnv or .env)
if [ -z "$DATABASE_URL" ]; then
  export DATABASE_URL="postgres://godstack:godstack@localhost:5432/godstack"
fi

# Run migrations using nix develop
nix develop --command bash -c "sqlx database create 2>/dev/null || true; sqlx migrate run" 2>/dev/null || {
  echo "⚠️  Migrations skipped (run manually with: just db-migrate)"
}

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
