# ⚙️ Setup Guide

**Vollständige Anleitung zur Einrichtung der Entwicklungsumgebung**

**Letzte Aktualisierung**: 2026-01-28

---

## 📋 Voraussetzungen

Für die Entwicklung benötigst du:

| Tool               | Beschreibung                                       | Installation                                           |
| ------------------ | -------------------------------------------------- | ------------------------------------------------------ |
| **Nix**            | Package Manager (stellt alle anderen Tools bereit) | [→ Nix installieren](#-nix-installieren)               |
| **Docker Desktop** | Container Runtime für Services                     | [→ Docker installieren](#-docker-desktop-installieren) |
| **Git + SSH**      | Repository-Zugriff (optional)                      | [→ Git/SSH Setup](#-git--ssh-setup-optional)           |

**Zeitaufwand**: ~5-10 Minuten

---

## 📦 Nix installieren

Nix ist der einzige Package Manager, den du installieren musst. Alle anderen Tools (Rust, Node.js, pnpm, buf, just, etc.) werden automatisch von Nix bereitgestellt.

### macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

Terminal neu starten, dann verifizieren:

```bash
nix --version
```

### Ubuntu/Debian

```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

Terminal neu starten, dann verifizieren:

```bash
nix --version
```

**Hinweis:** Für Ubuntu/Debian wird `systemd` benötigt. Falls nicht vorhanden, siehe [Nix Installation Guide](https://nixos.org/download).

---

## 🐳 Docker Desktop installieren

### macOS

**Option 1: Via Nix (empfohlen, wenn Nix bereits installiert)**

```bash
nix profile install nixpkgs#docker
```

**Option 2: Via Homebrew**

```bash
brew install --cask docker
```

**Option 3: Manuell**

Download von: https://www.docker.com/products/docker-desktop/

### Ubuntu/Debian

**Option 1: Via Nix (empfohlen, wenn Nix bereits installiert)**

```bash
nix profile install nixpkgs#docker
```

**Option 2: Via Installationsskript**

```bash
# Docker installieren
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Docker Desktop installieren (optional, für GUI)
# Download von: https://www.docker.com/products/docker-desktop/
```

Nach der Installation Docker Desktop starten und warten bis es läuft.

---

## 🔑 Git & SSH Setup (Optional)

> **Hinweis:** Nur nötig, wenn du das Repository über SSH klonen oder Commits signieren möchtest.

### Git installieren

**macOS:**

```bash
# Option 1: Via Nix
nix profile install nixpkgs#git

# Option 2: Via Homebrew
brew install git

# Option 3: Via Xcode Command Line Tools (oft bereits installiert)
xcode-select --install
```

**Ubuntu/Debian:**

```bash
# Option 1: Via Nix
nix profile install nixpkgs#git

# Option 2: Via apt
sudo apt update && sudo apt install git
```

Verifizieren:

```bash
git --version
```

### SSH-Key erstellen

```bash
# Key für Authentication (Repository klonen/pushen)
ssh-keygen -t ed25519 -C "deine-email@example.com" -f ~/.ssh/id_ed25519

# Key für Commit-Signierung
ssh-keygen -t ed25519 -C "git-signing" -f ~/.ssh/id_ed25519_signing -N ""
```

### SSH-Agent konfigurieren

**macOS:**

```bash
eval "$(ssh-agent -s)"
cat >> ~/.ssh/config << 'EOF'
Host github.com
    AddKeysToAgent yes
    UseKeychain yes
    IdentityFile ~/.ssh/id_ed25519
EOF
ssh-add --apple-use-keychain ~/.ssh/id_ed25519
```

**Ubuntu/Debian:**

```bash
eval "$(ssh-agent -s)"
cat >> ~/.ssh/config << 'EOF'
Host github.com
    AddKeysToAgent yes
    IdentityFile ~/.ssh/id_ed25519
EOF
ssh-add ~/.ssh/id_ed25519
```

### Public Keys zu GitHub hinzufügen

```bash
# Authentication Key anzeigen
cat ~/.ssh/id_ed25519.pub

# Signing Key anzeigen
cat ~/.ssh/id_ed25519_signing.pub
```

1. Gehe zu **GitHub → Settings → SSH and GPG keys**
2. **New SSH key** → Key Type: **Authentication Key** → Füge `id_ed25519.pub` ein
3. **New SSH key** → Key Type: **Signing Key** → Füge `id_ed25519_signing.pub` ein

### Git konfigurieren

```bash
git config --global user.name "Dein Name"
git config --global user.email "deine-email@example.com"
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519_signing.pub
git config --global commit.gpgsign true
```

---

## 🚀 Projekt starten

Sobald Nix und Docker installiert sind, kannst du das Projekt starten:

### 1. Repository klonen

```bash
git clone git@github.com:NiklasJavier/erynoa.git
cd erynoa
```

### 2. Nix Dev-Shell betreten

```bash
nix develop
```

Dies lädt automatisch alle Tools:

- ✅ Rust Toolchain (inkl. rust-analyzer, clippy)
- ✅ Node.js & pnpm
- ✅ buf (Protobuf)
- ✅ just (Task Runner)
- ✅ sqlx CLI
- ✅ Alle Build-Tools

### 3. Projekt starten

```bash
just dev
```

### 4. Warte 2 Minuten ⏳

Die Services starten und ZITADEL wird automatisch konfiguriert.

**Was passiert automatisch:**

- Services starten (PostgreSQL, DragonflyDB, MinIO, ZITADEL)
- ZITADEL wird konfiguriert (Projekt, Apps, Test-User)
- Frontends werden über Caddy Proxy bereitgestellt
- Backend läuft auf Port 3000

### 5. Im Browser öffnen

```
http://localhost:3001
```

**Fertig!** 🎉

**Alle URLs:**
| Service | URL |
|---------|-----|
| **Proxy (Hauptzugang)** | http://localhost:3001 |
| Console | http://localhost:3001/console |
| Platform | http://localhost:3001/platform |
| Docs | http://localhost:3001/docs |
| Backend API (via Proxy) | http://localhost:3001/api |
| Backend API (direkt) | http://localhost:3000 |
| ZITADEL | http://localhost:8080 |
| MinIO Console | http://localhost:9001 |

**Test Login:**

- User: `testuser` / `Test123!`
- Admin: `zitadel-admin` / `Password1!`

---

## 🔧 Wichtige Befehle

### Entwicklung

| Befehl                | Beschreibung                                                       |
| --------------------- | ------------------------------------------------------------------ |
| `just dev`            | **Startet alles** - Console + Platform + Docs + Backend + Services |
| `just dev [frontend]` | Startet spezifisches Frontend (console, platform, docs)            |
| `just status`         | Zeigt Status aller Services                                        |
| `just logs [service]` | Logs anzeigen (alle oder spezifischer Service)                     |
| `just stop`           | Stoppt alle Container                                              |
| `just restart`        | Schneller Neustart aller Dev-Services                              |

### Setup & Reset

| Befehl               | Beschreibung                       |
| -------------------- | ---------------------------------- |
| `just init`          | Initialisierung ohne Dev-Server    |
| `just init-env`      | Erstellt `.env` aus `.env.example` |
| `just zitadel-setup` | ZITADEL neu konfigurieren          |
| `just minio-setup`   | MinIO Buckets erstellen            |
| `just reset`         | **Alles löschen** und neu starten  |

### Backend

| Befehl            | Beschreibung         |
| ----------------- | -------------------- |
| `just check`      | Cargo check          |
| `just lint`       | Clippy Linter        |
| `just fmt`        | Code formatieren     |
| `just test`       | Tests ausführen      |
| `just ci`         | fmt + lint + test    |
| `just db-migrate` | Migrations ausführen |

Alle Befehle: `just --list`

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

### Nix: "experimental-features" Fehler

```bash
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

### Docker: Permission Denied

Docker Desktop muss gestartet sein. Überprüfen mit:

```bash
docker ps
```

### direnv: ".envrc is blocked"

Beim ersten Öffnen des Projekts erscheint diese Fehlermeldung:

```
direnv: error .envrc is blocked. Run `direnv allow` to approve its content
```

**Lösung:**

```bash
# Im Projektverzeichnis ausführen:
cd /path/to/erynoa
direnv allow
```

Danach die Shell neu laden:

```bash
exec zsh  # oder exec bash
```

---

## 📚 Weitere Dokumentation

- [Getting Started](../guides/getting-started.md) - Schnellstart
- [ZITADEL Setup](../guides/zitadel.md) - Authentifizierung konfigurieren
- [Configuration](../reference/config.md) - Service-Konfiguration
- [Architecture](../reference/architecture.md) - System-Architektur

---

**Fertig!** Die Entwicklungsumgebung ist eingerichtet. 🎉
