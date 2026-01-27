# ⚙️ Setup Guide

**Vollständige Anleitung zur Einrichtung der Entwicklungsumgebung**

---

## 🚀 Quick Start (Dev Container)

Die einfachste Methode ist der VS Code Dev Container:

1. **VS Code öffnen**: `code .`
2. **Dev Container starten**: `Cmd+Shift+P` → "Dev Containers: Reopen in Container"
3. **Warten** bis Container bereit ist
4. **Starten**: `just dev`

**Fertig!** Alle Tools sind automatisch verfügbar.

---

## 📋 Vollständige Setup-Anleitung (macOS)

### Voraussetzungen

- macOS 12+ (Monterey oder neuer)
- Admin-Rechte (für Homebrew & Nix)
- GitHub Account mit Zugriff auf das Repository

### 1. Xcode Command Line Tools

```bash
xcode-select --install
```

### 2. Homebrew installieren

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Nach der Installation (Apple Silicon):
```bash
echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
eval "$(/opt/homebrew/bin/brew shellenv)"
```

### 3. Nix installieren

```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

Terminal neu starten, dann verifizieren:
```bash
nix --version
```

### 4. Docker Desktop installieren

Download von: https://www.docker.com/products/docker-desktop/

Oder via Homebrew:
```bash
brew install --cask docker
```

Nach der Installation Docker Desktop starten.

### 5. SSH-Key für GitHub einrichten

```bash
# Key erstellen
ssh-keygen -t ed25519 -C "deine-email@example.com" -f ~/.ssh/id_ed25519
ssh-keygen -t ed25519 -C "git-signing" -f ~/.ssh/id_ed25519_signing -N ""

# SSH-Agent konfigurieren
eval "$(ssh-agent -s)"
cat >> ~/.ssh/config << 'EOF'
Host github.com
    AddKeysToAgent yes
    UseKeychain yes
    IdentityFile ~/.ssh/id_ed25519
EOF
ssh-add --apple-use-keychain ~/.ssh/id_ed25519

# Public Keys anzeigen
cat ~/.ssh/id_ed25519.pub
cat ~/.ssh/id_ed25519_signing.pub
```

Füge beide Keys zu GitHub hinzu: **Settings → SSH and GPG keys**

### 6. Git konfigurieren

```bash
git config --global user.name "Dein Name"
git config --global user.email "deine-email@example.com"
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519_signing.pub
git config --global commit.gpgsign true
```

### 7. Repository klonen

```bash
git clone git@github.com:NiklasJavier/GS-Backend.git
cd GS-Backend
```

### 8. Entwicklungsumgebung starten

**Option A: Mit Nix (empfohlen)**
```bash
nix develop
just dev
```

**Option B: Mit VS Code DevContainer**
1. VS Code öffnen: `code .`
2. `Cmd+Shift+P` → "Dev Containers: Reopen in Container"
3. `just dev`

---

## 🐳 Docker Development Setup

### Architektur

```
┌── DevContainer ─────────────────────────┐
│  ┌── Docker Compose ─────────────────┐ │
│  │  console (5173)  backend (3000)   │ │
│  │  db (5432)  cache (6379)           │ │
│  │  minio (9000/9001)  zitadel (8080) │ │
│  └────────────────────────────────────┘ │
└──────────────────────────────────────────┘
```

### Features

- ✅ **Console Hot-Reload**: Vite HMR (<100ms)
- ✅ **Backend Hot-Reload**: cargo-watch (5-15s)
- ✅ **Isolierte Services**: Alle Abhängigkeiten containerisiert
- ✅ **Volume Mounts**: Code-Änderungen sofort sichtbar
- ✅ **Health Checks**: Automatische Abhängigkeitsprüfung

### Services

| Service | Port | Beschreibung |
|---------|------|--------------|
| Console | 3001/console | SvelteKit + Vite (via Caddy Proxy) |
| Platform | 3001/platform | SvelteKit + Vite (via Caddy Proxy) |
| Docs | 3001/docs | SvelteKit + Vite (via Caddy Proxy) |
| Proxy | 3001 | Caddy Reverse Proxy |
| Backend | 3000 | Rust + Axum |
| Database | 5432 | PostgreSQL (OrioleDB) |
| Cache | 6379 | DragonflyDB (Redis) |
| Storage | 9000/9001 | MinIO (S3) |
| Auth | 8080 | ZITADEL (OIDC) |

---

## 🔧 Wichtige Befehle

### Entwicklung

| Befehl | Beschreibung |
|--------|--------------|
| `just dev` | **Startet alles** - Console + Platform + Docs + Backend + Services |
| `just dev [frontend]` | Startet spezifisches Frontend (console, platform, docs) |
| `just status` | Zeigt Status aller Services |
| `just check` | Health Check aller Services |
| `just restart` | Schneller Neustart aller Dev-Services |
| `just stop` | Stoppt alle Container |
| `just logs [service]` | Logs anzeigen (alle oder spezifischer Service) |

### Setup & Reset

| Befehl | Beschreibung |
|--------|--------------|
| `just init` | Initialisierung ohne Dev-Server |
| `just zitadel-setup` | ZITADEL neu konfigurieren |
| `just minio-setup` | MinIO Buckets erstellen |
| `just reset` | **Alles löschen** und neu starten |

### Logs & Debug

| Befehl | Beschreibung |
|--------|--------------|
| `just logs` | Alle Container-Logs |
| `just logs [service]` | Logs für spezifischen Service |
| `just shell [service]` | Shell in Container (backend, console, platform, docs) |

---

## ⚙️ Konfiguration

### Konfigurationspriorität (höchste zuerst):
1. **Umgebungsvariablen** (`APP_DATABASE__HOST=db`)
2. **local.toml** (auto-generated, gitignored)
3. **base.toml** (Standard-Werte)

### Docker-Compose Umgebungsvariablen

```yaml
environment:
  - APP_DATABASE__HOST=db
  - APP_CACHE__URL=redis://cache:6379
  - APP_AUTH__ISSUER=http://localhost:8080
  - APP_AUTH__INTERNAL_ISSUER=http://zitadel:8080
  - APP_STORAGE__ENDPOINT=http://minio:9000
```

---

## 🐛 Troubleshooting

### Services starten nicht
```bash
just reset
just dev
```

### Port bereits belegt
```bash
just stop
lsof -i :3000  # oder :3001, :8080
```

### ZITADEL Client-ID ungültig
```bash
just zitadel-reset
```

### Backend kompiliert nicht
```bash
just shell backend
cargo check  # Zeigt Fehler
```

### Nix: "experimental-features" Fehler
```bash
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

### SQLx: "DATABASE_URL must be set"
```bash
# Automatisch aus .env.example erstellen:
just init-env

# Oder manuell:
cp .env.example .env
```

---

## 📚 Weitere Dokumentation

- [Getting Started](getting-started.md) - Schnellstart
- [ZITADEL Setup](zitadel.md) - Authentifizierung konfigurieren
- [Configuration](reference/config.md) - Service-Konfiguration
- [Architecture](reference/architecture.md) - System-Architektur

---

**Fertig!** Die Entwicklungsumgebung ist eingerichtet. 🎉
