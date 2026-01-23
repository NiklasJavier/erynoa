#!/bin/bash
# One-time DevContainer setup (runs once on container creation)
set -e

echo "🔧 One-time DevContainer setup..."

# ─────────────────────────────────────────────────────────────────────────────
# 1. Setup direnv for automatic Nix environment loading
# ─────────────────────────────────────────────────────────────────────────────
echo "📦 Configuring direnv for automatic Nix environment..."

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
# 2. Create .env file if not exists
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
alias logs="docker compose -f /workspace/.devcontainer/services.yml logs -f"
'

for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
  if [ -f "$rc" ]; then
    if ! grep -q "God-Stack DevContainer aliases" "$rc"; then
      echo "$ALIASES" >> "$rc"
    fi
  fi
done

echo "✅ One-time setup complete!"
