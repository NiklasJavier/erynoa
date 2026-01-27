#!/bin/bash
# Kombiniertes Setup- und Init-Script für DevContainer
# Läuft einmal beim Erstellen (postCreateCommand) und bei jedem Start (postStartCommand)
# WICHTIG: Beide Commands nutzen dieses Script, um nur ein Terminal zu öffnen
# Funktioniert sowohl im DevContainer als auch direkt auf dem Host
set +e  # Fehler nicht blockierend

# Erkenne, ob wir im Container oder auf dem Host sind
if [ -d "/workspace" ] && [ -f "/workspace/.devcontainer/setup-and-init.sh" ]; then
    # Im DevContainer: Workspace ist /workspace
    WORKSPACE_ROOT="/workspace"
elif [ -f "$(dirname "$0")/setup-and-init.sh" ]; then
    # Auf dem Host: Script-Pfad verwenden
    SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
    WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
else
    # Fallback: Aktuelles Verzeichnis
    WORKSPACE_ROOT="${PWD}"
fi

MARKER_FILE="$HOME/.devcontainer/.setup-complete"
LOCK_FILE="$HOME/.devcontainer/.setup-and-init.lock"

# Prüfe, ob Script bereits läuft (verhindert doppelte Ausführung)
if [ -f "$LOCK_FILE" ]; then
  # Prüfe, ob der Prozess noch läuft
  LOCK_PID=$(cat "$LOCK_FILE" 2>/dev/null || echo "")
  if [ -n "$LOCK_PID" ] && kill -0 "$LOCK_PID" 2>/dev/null; then
    echo "⚠️  Setup/Init läuft bereits (PID: $LOCK_PID), überspringe..."
    exit 0
  else
    # Lock-File existiert, aber Prozess läuft nicht mehr - entferne Lock
    rm -f "$LOCK_FILE"
  fi
fi

# Erstelle Lock-File
mkdir -p "$HOME/.devcontainer"
echo $$ > "$LOCK_FILE"
trap "rm -f $LOCK_FILE" EXIT

# Prüfe, ob Setup bereits gelaufen ist
SETUP_COMPLETE=false
if [ -f "$MARKER_FILE" ]; then
  SETUP_COMPLETE=true
fi

# ─────────────────────────────────────────────────────────────────────────────
# SETUP (nur einmal beim Erstellen)
# ─────────────────────────────────────────────────────────────────────────────
if [ "$SETUP_COMPLETE" = false ]; then
  echo "🔧 One-time DevContainer setup..."
  
  # ─────────────────────────────────────────────────────────────────────────────
  # 0. Setup SSH keys from host
  # ─────────────────────────────────────────────────────────────────────────────
  echo "🔑 Setting up SSH keys..."
  
  if [ -d "$HOME/.ssh-host" ]; then
    mkdir -p "$HOME/.ssh"
    # Copy SSH keys and config (exclude sockets)
    find "$HOME/.ssh-host" -maxdepth 1 -type f \( -name "*.pub" -o -name "id_*" -o -name "config" -o -name "known_hosts*" -o -name "allowed_signers" \) -exec cp {} "$HOME/.ssh/" \; 2>/dev/null || true
    # Copy directories (like priv-key, pub-keys)
    for dir in priv-key pub-keys; do
      if [ -d "$HOME/.ssh-host/$dir" ]; then
        cp -r "$HOME/.ssh-host/$dir" "$HOME/.ssh/" 2>/dev/null || true
      fi
    done
    # Set correct permissions
    chmod 700 "$HOME/.ssh"
    find "$HOME/.ssh" -type f -name "id_*" ! -name "*.pub" -exec chmod 600 {} \; 2>/dev/null || true
    find "$HOME/.ssh" -type f -name "*.pub" -exec chmod 644 {} \; 2>/dev/null || true
    [ -f "$HOME/.ssh/config" ] && chmod 600 "$HOME/.ssh/config"
    echo "   SSH keys configured"
  else
    echo "   ⚠️ No SSH keys found from host"
  fi
  
  # ─────────────────────────────────────────────────────────────────────────────
  # 1. Configure Nix experimental features
  # ─────────────────────────────────────────────────────────────────────────────
  echo "❄️  Configuring Nix..."
  
  mkdir -p "$HOME/.config/nix"
  if [ ! -f "$HOME/.config/nix/nix.conf" ] || ! grep -q "experimental-features" "$HOME/.config/nix/nix.conf"; then
    cat > "$HOME/.config/nix/nix.conf" << 'NIXCONF'
# Nix Flakes aktivieren
experimental-features = nix-command flakes

# ⚡ Binary Caches - WICHTIG für schnelles Setup (verhindert lokales Kompilieren)
# Hinweis: trusted-public-keys muss systemweit gesetzt werden (wird durch Nix Feature gehandhabt)
substituters = https://cache.nixos.org

# Performance-Optimierungen
max-jobs = auto

# Hinweis: keep-outputs, keep-derivations und log-lines sind restricted settings
# und müssen systemweit gesetzt werden. Diese werden durch das Nix Feature gehandhabt.
NIXCONF
    echo "   Nix flakes und Binary Caches konfiguriert"
  else
    echo "   Nix bereits konfiguriert"
  fi
  
  # ─────────────────────────────────────────────────────────────────────────────
  # 2. Setup direnv for automatic Nix environment loading
  # ─────────────────────────────────────────────────────────────────────────────
  echo "📦 Configuring direnv for automatic Nix environment..."
  
  # Ensure direnv is installed (fallback if feature fails)
  if ! command -v direnv &> /dev/null; then
    echo "   Installing direnv..."
    sudo apt-get update -qq && sudo apt-get install -y -qq direnv
  fi
  
  # Add direnv hook to shell profiles for automatic activation
  for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
    if [ -f "$rc" ]; then
      if ! grep -q 'eval "$(direnv hook' "$rc"; then
        echo "" >> "$rc"
        echo "# Direnv - automatic environment loading" >> "$rc"
        if [[ "$rc" == *"zshrc"* ]]; then
          echo 'eval "$(direnv hook zsh)"' >> "$rc"
        else
          echo 'eval "$(direnv hook bash)"' >> "$rc"
        fi
      fi
    fi
  done
  
  # ─────────────────────────────────────────────────────────────────────────────
  # 3. Create .env file if not exists
  # ─────────────────────────────────────────────────────────────────────────────
  echo "📝 Ensuring .env file exists..."
  
  if [ ! -f "$WORKSPACE_ROOT/.env" ]; then
    if [ -f "$WORKSPACE_ROOT/.env.example" ]; then
      cp "$WORKSPACE_ROOT/.env.example" "$WORKSPACE_ROOT/.env"
      echo "   ✅ Created .env from .env.example"
    else
      echo "   ⚠️  .env.example not found, creating default .env"
      cat > "$WORKSPACE_ROOT/.env" << 'ENVEOF'
# Environment Variables

APP_ENVIRONMENT=local
RUST_LOG=info,erynoa_api=debug

# Database
APP_DATABASE__HOST=localhost
APP_DATABASE__PORT=5432
APP_DATABASE__USERNAME=erynoa
APP_DATABASE__PASSWORD=erynoa
APP_DATABASE__DATABASE=erynoa

# SQLx Migration URL (required for sqlx-cli)
DATABASE_URL=postgres://erynoa:erynoa@localhost:5432/erynoa

# Cache
APP_CACHE__URL=redis://localhost:6379

# Auth (ZITADEL)
APP_AUTH__ISSUER=http://localhost:8080
APP_AUTH__CLIENT_ID=erynoa-backend
APP_AUTH__FRONTEND_CLIENT_ID=erynoa-frontend
ENVEOF
      echo "   Created .env from default template"
    fi
  else
    echo "   ✓ .env already exists (not overwriting)"
  fi
  
  # ─────────────────────────────────────────────────────────────────────────────
  # 4. Pre-warm Nix development environment
  # ─────────────────────────────────────────────────────────────────────────────
  echo "❄️  Pre-warming Nix development environment..."
  echo "   ℹ️  Beim ersten Start werden alle Tools heruntergeladen (kann 5-15 Min dauern)"
  echo "   ℹ️  Fortschritt wird unten angezeigt. Bei 'copying path' lädt Nix Binaries."
  echo ""
  
  cd "$WORKSPACE_ROOT"
  
  # Build the devShell mit verbose Output (zeigt Download-Fortschritt)
  if nix develop --verbose --print-build-logs --command echo "✅ Nix environment ready!"; then
    echo "   Nix devShell erfolgreich gecached"
  else
    echo "⚠️  Nix pre-warm fehlgeschlagen, wird beim ersten 'direnv allow' nachgeholt"
    echo "   Tipp: Schaue in 'Dev Containers: Show Container Log' für Details"
  fi
  
  # Jetzt direnv erlauben (nutzt den gecachten devShell)
  echo ""
  echo "📦 Aktiviere direnv..."
  direnv allow . 2>/dev/null || true
  
  # ─────────────────────────────────────────────────────────────────────────────
  # 5. Setup shell aliases for convenience (Nix-aware)
  # ─────────────────────────────────────────────────────────────────────────────
  echo "🔗 Adding helpful shell aliases..."
  
  # Aliases mit Workspace-Root (wird zur Laufzeit aufgelöst)
  ALIASES="
# Erynoa DevContainer aliases
alias dev=\"cd $WORKSPACE_ROOT && just dev\"
alias build=\"cd $WORKSPACE_ROOT && just build\"
alias test=\"cd $WORKSPACE_ROOT && just test\"
alias migrate=\"cd $WORKSPACE_ROOT && just db-migrate\"
alias logs=\"docker compose -f $WORKSPACE_ROOT/infra/docker/docker-compose.yml logs -f\"

# Nix path alias (daemon handles permissions)
alias nix=\"/nix/var/nix/profiles/default/bin/nix\"
"
  
  for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
    if [ -f "$rc" ]; then
      if ! grep -q "Erynoa DevContainer aliases" "$rc"; then
        echo "$ALIASES" >> "$rc"
      fi
    fi
  done
  
  # Markiere Setup als abgeschlossen
  mkdir -p "$HOME/.devcontainer"
  touch "$MARKER_FILE"
  echo "✅ One-time setup complete!"
fi

# ─────────────────────────────────────────────────────────────────────────────
# INIT (läuft immer, auch wenn Setup bereits gelaufen ist)
# ─────────────────────────────────────────────────────────────────────────────
if [ "$SETUP_COMPLETE" = true ]; then
  echo "🚀 Re-initializing Erynoa DevContainer..."
else
  echo "🚀 Initializing Erynoa DevContainer..."
fi

# ─────────────────────────────────────────────────────────────────────────────
# 0. Fix Docker credentials for Cursor compatibility
# ─────────────────────────────────────────────────────────────────────────────
echo "🐳 Fixing Docker credentials for Cursor compatibility..."
if [ -f "$HOME/.docker/config.json" ]; then
  if grep -q "dev-containers-" "$HOME/.docker/config.json" 2>/dev/null; then
    echo "   Fixing Docker config (removing VS Code Remote Containers credential helper)..."
    cp "$HOME/.docker/config.json" "$HOME/.docker/config.json.backup" 2>/dev/null || true
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
echo "📦 Checking Nix development environment..."

cd "$WORKSPACE_ROOT/backend"

if [ -f "../flake.nix" ]; then
  /nix/var/nix/profiles/default/bin/nix develop --command true 2>/dev/null && echo "   ✅ Nix environment ready" || echo "   ⚠️  Nix environment build failed or will happen on first use"
  
  if command -v direnv &> /dev/null; then
    direnv allow . 2>/dev/null || true
  fi
fi

# ─────────────────────────────────────────────────────────────────────────────
# 2. GPG Setup (Keys vom Host importieren)
# ─────────────────────────────────────────────────────────────────────────────
if [ -d "$HOME/.gnupg-host" ]; then
  echo "🔐 Configuring GPG..."
  
  mkdir -p "$HOME/.gnupg"
  chmod 700 "$HOME/.gnupg"
  
  cp -r "$HOME/.gnupg-host/"*.gpg "$HOME/.gnupg/" 2>/dev/null || true
  cp -r "$HOME/.gnupg-host/"*.kbx "$HOME/.gnupg/" 2>/dev/null || true
  cp -r "$HOME/.gnupg-host/trustdb.gpg" "$HOME/.gnupg/" 2>/dev/null || true
  
  if [ -d "$HOME/.gnupg-host/private-keys-v1.d" ]; then
    mkdir -p "$HOME/.gnupg/private-keys-v1.d"
    cp -r "$HOME/.gnupg-host/private-keys-v1.d/"* "$HOME/.gnupg/private-keys-v1.d/" 2>/dev/null || true
  fi
  
  chmod 700 "$HOME/.gnupg"
  find "$HOME/.gnupg" -type f -exec chmod 600 {} \; 2>/dev/null || true
  find "$HOME/.gnupg" -type d -exec chmod 700 {} \; 2>/dev/null || true
  
  rm -f "$HOME/.gnupg/S."* "$HOME/.gnupg/"*.lock 2>/dev/null || true
  
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

  gpgconf --kill all 2>/dev/null || true
  gpg-agent --daemon 2>/dev/null || true
  
  git config --global gpg.program gpg
  
  echo "   ✅ GPG configured"
fi

# ─────────────────────────────────────────────────────────────────────────────
# 3. Git Configuration vom Host übernehmen (1:1)
# ─────────────────────────────────────────────────────────────────────────────
echo "📝 Configuring Git (1:1 from host)..."

if [ -n "$GIT_USER_NAME" ] && [ -n "$GIT_USER_EMAIL" ]; then
  git config --global user.name "$GIT_USER_NAME"
  git config --global user.email "$GIT_USER_EMAIL"
  echo "   ✅ Git user.name und user.email von Environment-Variablen übernommen"
fi

if [ -f "$HOME/.gitconfig-host" ]; then
  if ! git config --global user.name >/dev/null 2>&1; then
    HOST_USER_NAME=$(git config --file "$HOME/.gitconfig-host" user.name 2>/dev/null || true)
    HOST_USER_EMAIL=$(git config --file "$HOME/.gitconfig-host" user.email 2>/dev/null || true)
    
    if [ -n "$HOST_USER_NAME" ]; then
      git config --global user.name "$HOST_USER_NAME"
    fi
    if [ -n "$HOST_USER_EMAIL" ]; then
      git config --global user.email "$HOST_USER_EMAIL"
    fi
    
    if [ -n "$HOST_USER_NAME" ] || [ -n "$HOST_USER_EMAIL" ]; then
      echo "   ✅ Git config vom Host übernommen"
    fi
  fi
  
  HOST_GPGFORMAT=$(git config --file "$HOME/.gitconfig-host" gpg.format 2>/dev/null || true)
  HOST_COMMIT_SIGN=$(git config --file "$HOME/.gitconfig-host" commit.gpgsign 2>/dev/null || true)
  HOST_TAG_SIGN=$(git config --file "$HOME/.gitconfig-host" tag.gpgsign 2>/dev/null || true)
  
  if [ -n "$HOST_GPGFORMAT" ] && ! git config --global gpg.format >/dev/null 2>&1; then
    git config --global gpg.format "$HOST_GPGFORMAT"
  fi
  if [ -n "$HOST_COMMIT_SIGN" ] && ! git config --global commit.gpgsign >/dev/null 2>&1; then
    git config --global commit.gpgsign "$HOST_COMMIT_SIGN"
  fi
  if [ -n "$HOST_TAG_SIGN" ] && ! git config --global tag.gpgsign >/dev/null 2>&1; then
    git config --global tag.gpgsign "$HOST_TAG_SIGN"
  fi
fi

if git config --global user.name >/dev/null 2>&1 && git config --global user.email >/dev/null 2>&1; then
  echo "   ✅ Git user.name: $(git config --global user.name)"
  echo "   ✅ Git user.email: $(git config --global user.email)"
else
  echo "   ⚠️  Git user.name oder user.email nicht gesetzt"
  echo "      Bitte manuell konfigurieren:"
  echo "      git config --global user.name \"Dein Name\""
  echo "      git config --global user.email \"deine-email@example.com\""
fi

# ─────────────────────────────────────────────────────────────────────────────
# 4. SSH Setup (Signing & Auth)
# ─────────────────────────────────────────────────────────────────────────────
echo "🔑 Configuring SSH..."

if [ -S "$SSH_AUTH_SOCK" ]; then
  echo "   ✅ SSH-Agent vom Host verbunden"
  ssh-add -l 2>/dev/null && echo "   ✅ SSH Keys im Agent geladen" || echo "   ⚠️  Keine Keys im SSH-Agent (führe 'ssh-add' auf dem Host aus)"
else
  echo "   ⚠️  SSH-Agent nicht verbunden - Fallback auf lokale Keys"
  if [ -d "$HOME/.ssh" ]; then
    eval "$(ssh-agent -s)" > /dev/null 2>&1
    find "$HOME/.ssh" -type f -name "id_*" ! -name "*.pub" -exec ssh-add {} \; 2>/dev/null || true
  fi
fi

if [ -d "$HOME/.ssh" ]; then
  GPG_FORMAT=$(git config --global gpg.format 2>/dev/null || echo "")
  
  if [ -z "$GPG_FORMAT" ] && [ -f "$HOME/.ssh/id_ed25519_signing.pub" ]; then
    git config --global gpg.format ssh
    echo "   ✅ Git gpg.format auf 'ssh' gesetzt"
  fi
  
  if [ -f "$HOME/.ssh/id_ed25519_signing.pub" ]; then
    CONTAINER_KEY_PATH="$HOME/.ssh/id_ed25519_signing.pub"
    git config --global user.signingkey "$CONTAINER_KEY_PATH"
    echo "   ✅ SSH signing key linked: id_ed25519_signing.pub"
  fi

  if [ -f "$HOME/.ssh/allowed_signers" ]; then
    git config --global gpg.ssh.allowedSignersFile "$HOME/.ssh/allowed_signers"
    echo "   ✅ Allowed signers file linked"
  fi
  
  if ! git config --global commit.gpgsign >/dev/null 2>&1; then
    git config --global commit.gpgsign true
    echo "   ✅ Commit signing aktiviert"
  fi
  if ! git config --global tag.gpgsign >/dev/null 2>&1; then
    git config --global tag.gpgsign true
    echo "   ✅ Tag signing aktiviert"
  fi
fi

# ─────────────────────────────────────────────────────────────────────────────
# 5. Infrastructure Services (Docker-in-Docker)
# ─────────────────────────────────────────────────────────────────────────────
echo "🐳 Starting infrastructure services..."
cd "$WORKSPACE_ROOT"

if command -v docker &> /dev/null && docker ps >/dev/null 2>&1; then
  docker compose -f "$WORKSPACE_ROOT/infra/docker/docker-compose.yml" up -d || {
    echo "   ⚠️  Docker services failed to start"
  }

  echo "⏳ Waiting for database..."
  for i in {1..30}; do
    if docker compose -f "$WORKSPACE_ROOT/infra/docker/docker-compose.yml" exec -T db pg_isready -U erynoa >/dev/null 2>&1; then
      echo "   ✅ Database ready!"
      break
    fi
    [ $i -eq 30 ] && echo "   ⚠️  Database not ready after 30s - continuing anyway"
    sleep 1
  done

  echo "⏳ Waiting for cache..."
  for i in {1..30}; do
    if docker compose -f "$WORKSPACE_ROOT/infra/docker/docker-compose.yml" exec -T cache redis-cli ping >/dev/null 2>&1; then
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
# 6. Migrations
# ─────────────────────────────────────────────────────────────────────────────
echo "📦 Running database migrations..."
cd "$WORKSPACE_ROOT/backend"

if [ -z "$DATABASE_URL" ]; then
  export DATABASE_URL="postgres://erynoa:erynoa@localhost:5432/erynoa"
fi

cd "$WORKSPACE_ROOT" && nix develop --command bash -c "cd \"$WORKSPACE_ROOT/backend\" && sqlx database create 2>/dev/null || true; sqlx migrate run" 2>/dev/null || {
  echo "   ⚠️  Migrations skipped (check logs or run 'just db-migrate')"
}

echo ""
echo "✅ DevContainer initialization complete!"
