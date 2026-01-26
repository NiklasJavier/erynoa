#!/bin/bash
# Fehler nicht blockierend - wir wollen, dass der DevContainer lädt auch wenn einzelne Schritte fehlschlagen
set +e

echo "🚀 Initializing Erynoa DevContainer..."

# ─────────────────────────────────────────────────────────────────────────────
# 0. Fix Docker credentials for Cursor compatibility
# ─────────────────────────────────────────────────────────────────────────────
echo "🐳 Fixing Docker credentials for Cursor compatibility..."
if [ -f "$HOME/.docker/config.json" ]; then
  # Check if config contains VS Code Remote Containers credential helper
  if grep -q "dev-containers-" "$HOME/.docker/config.json" 2>/dev/null; then
    echo "   Fixing Docker config (removing VS Code Remote Containers credential helper)..."
    # Backup original config
    cp "$HOME/.docker/config.json" "$HOME/.docker/config.json.backup" 2>/dev/null || true
    # Remove the problematic credsStore
    cat > "$HOME/.docker/config.json" << 'DOCKEREOF'
{}
DOCKEREOF
    echo "   ✅ Docker config fixed"
  else
    echo "   ✅ Docker config OK"
  fi
fi

# ─────────────────────────────────────────────────────────────────────────────
# 1. Nix Environment Pre-flight
# ─────────────────────────────────────────────────────────────────────────────
# Hinweis: Der Nix-Daemon läuft bereits durch das DevContainer Feature.
echo "📦 Checking Nix development environment..."

cd /workspace/backend

if [ -f "../flake.nix" ]; then
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
echo "🔑 Configuring SSH..."

# SSH-Agent vom Host prüfen
if [ -S "$SSH_AUTH_SOCK" ]; then
  echo "   ✅ SSH-Agent vom Host verbunden"
  ssh-add -l 2>/dev/null && echo "   ✅ SSH Keys im Agent geladen" || echo "   ⚠️  Keine Keys im SSH-Agent (führe 'ssh-add' auf dem Host aus)"
else
  echo "   ⚠️  SSH-Agent nicht verbunden - Fallback auf lokale Keys"
  # Fallback: Lokalen SSH-Agent starten und Keys laden
  if [ -d "$HOME/.ssh" ]; then
    eval "$(ssh-agent -s)" > /dev/null 2>&1
    # Alle private keys ohne Passphrase hinzufügen
    find "$HOME/.ssh" -type f -name "id_*" ! -name "*.pub" -exec ssh-add {} \; 2>/dev/null || true
  fi
fi

# Signing Key für Git übernehmen, falls auf dem Host konfiguriert
if [ -d "$HOME/.ssh" ]; then
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
cd /workspace

# Services starten (aus infra/docker-compose.yml) - mit Timeout falls Docker nicht läuft
if command -v docker &> /dev/null && docker ps >/dev/null 2>&1; then
  docker compose -f infra/docker-compose.yml up -d || {
    echo "   ⚠️  Docker services failed to start"
  }

  # Warten auf Datenbank (mit max 30 Sekunden Timeout)
  echo "⏳ Waiting for database..."
  for i in {1..30}; do
    if docker compose -f infra/docker-compose.yml exec -T db pg_isready -U erynoa >/dev/null 2>&1; then
      echo "   ✅ Database ready!"
      break
    fi
    [ $i -eq 30 ] && echo "   ⚠️  Database not ready after 30s - continuing anyway"
    sleep 1
  done

  # Warten auf Cache (mit max 30 Sekunden Timeout)
  echo "⏳ Waiting for cache..."
  for i in {1..30}; do
    if docker compose -f infra/docker-compose.yml exec -T cache redis-cli ping >/dev/null 2>&1; then
      echo "   ✅ Cache ready!"
      break
    fi
    [ $i -eq 30 ] && echo "   ⚠️  Cache not ready after 30s - continuing anyway"
    sleep 1
  done
else
  echo "   ⚠️  Docker not available - skipping infrastructure startup"
fi

# ─────────────────────────────────────────────────────────────────────────────
# 5. Migrations
# ─────────────────────────────────────────────────────────────────────────────
echo "📦 Running database migrations..."
cd /workspace/backend

# DATABASE_URL setzen, falls leer
if [ -z "$DATABASE_URL" ]; then
  export DATABASE_URL="postgres://erynoa:erynoa@localhost:5432/erynoa"
fi

# Wir nutzen 'nix develop', um sicherzustellen, dass sqlx-cli verfügbar ist
# Das muss in backend/ ausgeführt werden, wo die migrations/ sind
cd /workspace && nix develop --command bash -c "cd /workspace/backend && sqlx database create 2>/dev/null || true; sqlx migrate run" 2>/dev/null || {
  echo "   ⚠️  Migrations skipped (check logs or run 'just db-migrate')"
}

echo ""
echo "✅ DevContainer initialization complete!"