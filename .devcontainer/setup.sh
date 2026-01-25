#!/bin/bash
# One-time DevContainer setup (runs once on container creation)
set -e

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
  echo "experimental-features = nix-command flakes" > "$HOME/.config/nix/nix.conf"
  echo "   Nix flakes enabled"
else
  echo "   Nix already configured"
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

# Allow direnv in workspace
cd /workspace
direnv allow . 2>/dev/null || true

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

if [ ! -f /workspace/.env ]; then
  cp /workspace/.env.example /workspace/.env 2>/dev/null || cat > /workspace/.env << 'ENVEOF'
# Environment Variables

APP_ENVIRONMENT=local
RUST_LOG=info,godstack_api=debug

# Database
APP_DATABASE__HOST=localhost
APP_DATABASE__PORT=5432
APP_DATABASE__USERNAME=godstack
APP_DATABASE__PASSWORD=godstack
APP_DATABASE__DATABASE=godstack

# SQLx Migration URL (required for sqlx-cli)
DATABASE_URL=postgres://godstack:godstack@localhost:5432/godstack

# Cache
APP_CACHE__URL=redis://localhost:6379

# Auth (ZITADEL)
APP_AUTH__ISSUER=http://localhost:8080
APP_AUTH__CLIENT_ID=godstack-backend
APP_AUTH__FRONTEND_CLIENT_ID=godstack-frontend
ENVEOF
  echo "   Created .env from template"
else
  echo "   .env already exists"
fi

# ─────────────────────────────────────────────────────────────────────────────
# 3. Pre-warm Nix development environment
# ─────────────────────────────────────────────────────────────────────────────
echo "❄️  Pre-warming Nix development environment (this may take a few minutes)..."

cd /workspace
# Build the devShell to cache all dependencies
nix develop --command true 2>/dev/null || {
  echo "⚠️  Nix pre-warm failed, will be built on first use"
}

# ─────────────────────────────────────────────────────────────────────────────
# 4. Setup shell aliases for convenience (Nix-aware)
# ─────────────────────────────────────────────────────────────────────────────
echo "🔗 Adding helpful shell aliases..."

ALIASES='
# God-Stack DevContainer aliases
alias dev="cd /workspace && just dev"
alias build="cd /workspace && just build"
alias test="cd /workspace && just test"
alias migrate="cd /workspace && just db-migrate"
alias logs="docker compose -f /workspace/infra/docker-compose.yml logs -f"

# Nix path alias (daemon handles permissions)
alias nix="/nix/var/nix/profiles/default/bin/nix"
'

for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
  if [ -f "$rc" ]; then
    if ! grep -q "God-Stack DevContainer aliases" "$rc"; then
      echo "$ALIASES" >> "$rc"
    fi
  fi
done

echo "✅ One-time setup complete!"
