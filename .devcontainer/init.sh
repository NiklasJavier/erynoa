#!/bin/bash
set -e

echo "🚀 Initializing God-Stack DevContainer..."

# ─────────────────────────────────────────────────────────────────────────────
# 1. Nix Environment Pre-flight
# ─────────────────────────────────────────────────────────────────────────────
# Hinweis: Der Nix-Daemon läuft bereits durch das DevContainer Feature.
echo "📦 Checking Nix development environment..."

cd /workspace

if [ -f "flake.nix" ]; then
  # Pre-warm: Baut die Umgebung einmal, damit Caches gefüllt sind.
  # Wir unterdrücken den Output, außer es gibt Fehler.
  /nix/var/nix/profiles/default/bin/nix develop --command true 2>/dev/null && echo "   ✅ Nix environment ready" || echo "   ⚠️  Nix environment build failed or will happen on first use"
  
  # Direnv aktivieren
  if command -v direnv &> /dev/null; then
    direnv allow . 2>/dev/null || true
  fi
fi

# ─────────────────────────────────────────────────────────────────────────────
# 2. GPG Setup (Keys vom Host importieren)
# ─────────────────────────────────────────────────────────────────────────────
# Wir kopieren die Keys, da Socket-Forwarding oft instabil ist bei GPG.
if [ -d "$HOME/.gnupg-host" ]; then
  echo "🔐 Configuring GPG..."
  
  # Verzeichnis vorbereiten
  mkdir -p "$HOME/.gnupg"
  chmod 700 "$HOME/.gnupg"
  
  # Keys kopieren (Fehler ignorieren, falls keine da sind)
  cp -r "$HOME/.gnupg-host/"*.gpg "$HOME/.gnupg/" 2>/dev/null || true
  cp -r "$HOME/.gnupg-host/"*.kbx "$HOME/.gnupg/" 2>/dev/null || true
  cp -r "$HOME/.gnupg-host/trustdb.gpg" "$HOME/.gnupg/" 2>/dev/null || true
  
  # Private Keys (Unterordner)
  if [ -d "$HOME/.gnupg-host/private-keys-v1.d" ]; then
    mkdir -p "$HOME/.gnupg/private-keys-v1.d"
    cp -r "$HOME/.gnupg-host/private-keys-v1.d/"* "$HOME/.gnupg/private-keys-v1.d/" 2>/dev/null || true
  fi
  
  # Berechtigungen korrigieren (GPG ist hier sehr strikt)
  chmod 700 "$HOME/.gnupg"
  find "$HOME/.gnupg" -type f -exec chmod 600 {} \; 2>/dev/null || true
  find "$HOME/.gnupg" -type d -exec chmod 700 {} \; 2>/dev/null || true
fi

# GPG Konfiguration für Container-Nutzung schreiben
if [ -d "$HOME/.gnupg" ]; then
  # Alte Locks/Sockets löschen
  rm -f "$HOME/.gnupg/S."* "$HOME/.gnupg/"*.lock 2>/dev/null || true
  
  # Config für VS Code Terminal optimieren (Pinentry Loopback)
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

  # Agent neu starten
  gpgconf --kill all 2>/dev/null || true
  gpg-agent --daemon 2>/dev/null || true
  
  # Git Config global setzen
  git config --global gpg.program gpg
  
  echo "   ✅ GPG configured"
fi

# ─────────────────────────────────────────────────────────────────────────────
# 3. SSH Setup (Signing & Auth)
# ─────────────────────────────────────────────────────────────────────────────
if [ -d "$HOME/.ssh" ]; then
  echo "🔑 Configuring SSH..."
  
  # Signing Key für Git übernehmen, falls auf dem Host konfiguriert
  CURRENT_SIGNING_KEY=$(git config --global user.signingkey 2>/dev/null || true)
  GPG_FORMAT=$(git config --global gpg.format 2>/dev/null || true)
  
  if [ "$GPG_FORMAT" = "ssh" ] && [ -n "$CURRENT_SIGNING_KEY" ]; then
    KEY_BASENAME=$(basename "$CURRENT_SIGNING_KEY")
    CONTAINER_KEY_PATH="$HOME/.ssh/$KEY_BASENAME"
    
    if [ -f "$CONTAINER_KEY_PATH" ]; then
      git config --global user.signingkey "$CONTAINER_KEY_PATH"
      echo "   ✅ SSH signing key linked: $KEY_BASENAME"
    fi
  fi

  # Allowed Signers fixen
  CURRENT_ALLOWED_SIGNERS=$(git config --global gpg.ssh.allowedSignersFile 2>/dev/null || true)
  if [ -n "$CURRENT_ALLOWED_SIGNERS" ]; then
    SIGNERS_BASENAME=$(basename "$CURRENT_ALLOWED_SIGNERS")
    CONTAINER_SIGNERS_PATH="$HOME/.ssh/$SIGNERS_BASENAME"
    if [ -f "$CONTAINER_SIGNERS_PATH" ]; then
      git config --global gpg.ssh.allowedSignersFile "$CONTAINER_SIGNERS_PATH"
    fi
  fi
fi

# ─────────────────────────────────────────────────────────────────────────────
# 4. Infrastructure Services (Docker-in-Docker)
# ─────────────────────────────────────────────────────────────────────────────
echo "🐳 Starting infrastructure services..."
cd /workspace/.devcontainer

# Services starten
docker compose -f services.yml up -d

# Warten auf Datenbank
echo "⏳ Waiting for database..."
until docker compose -f services.yml exec -T db pg_isready -U godstack >/dev/null 2>&1; do
  sleep 1
done
echo "   ✅ Database ready!"

# Warten auf Cache
echo "⏳ Waiting for cache..."
until docker compose -f services.yml exec -T cache redis-cli ping >/dev/null 2>&1; do
  sleep 1
done
echo "   ✅ Cache ready!"

# ─────────────────────────────────────────────────────────────────────────────
# 5. Migrations
# ─────────────────────────────────────────────────────────────────────────────
echo "📦 Running database migrations..."
cd /workspace

# DATABASE_URL setzen, falls leer
if [ -z "$DATABASE_URL" ]; then
  export DATABASE_URL="postgres://godstack:godstack@localhost:5432/godstack"
fi

# Wir nutzen 'nix develop', um sicherzustellen, dass sqlx-cli verfügbar ist
nix develop --command bash -c "sqlx database create 2>/dev/null || true; sqlx migrate run" 2>/dev/null || {
  echo "   ⚠️  Migrations skipped (check logs or run 'just db-migrate')"
}

echo ""
echo "✅ DevContainer initialization complete!"