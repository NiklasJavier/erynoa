# Erynoa – Setup Guide

> **Dokumenttyp:** Guide
> **Zielgruppe:** Neue Entwickler
> **Dauer:** ca. 10-15 Minuten
> **OS:** macOS, Linux (Ubuntu/Debian)

---

## Übersicht

Dieser Guide führt dich durch die vollständige Einrichtung der Erynoa-Entwicklungsumgebung.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ⚙️ SETUP-ABLAUF                                                           │
│                                                                             │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│   │     1       │    │     2       │    │     3       │    │     4       │ │
│   │    Nix      │───▶│   Docker    │───▶│    Git      │───▶│   Start     │ │
│   │  ~2 Min.    │    │   ~3 Min.   │    │  Optional   │    │   ~2 Min.   │ │
│   └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘ │
│                                                                             │
│   Nix stellt alle anderen Tools bereit:                                    │
│   Rust · Node.js · pnpm · buf · just · sqlx                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Voraussetzungen

| Tool          | Zweck                                      | Erforderlich |
| :------------ | :----------------------------------------- | :----------: |
| **Nix**       | Package Manager (stellt alle Tools bereit) |      ✅      |
| **Docker**    | Container-Services                         |      ✅      |
| **Git + SSH** | Repository-Zugriff, Commit-Signierung      |   Optional   |

---

## 1️⃣ Nix installieren

Nix ist der einzige Package Manager, den du manuell installieren musst. Alle anderen Tools werden automatisch bereitgestellt.

<details open>
<summary><strong>macOS</strong></summary>

```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

**Terminal neu starten**, dann verifizieren:

```bash
nix --version
```

</details>

<details>
<summary><strong>Linux (Ubuntu/Debian)</strong></summary>

```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

**Terminal neu starten**, dann verifizieren:

```bash
nix --version
```

> ℹ️ Benötigt `systemd`. Falls nicht vorhanden: [Nix Installation Guide](https://nixos.org/download)

</details>

### Was Nix bereitstellt

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ✅ AUTOMATISCH VERFÜGBAR (nach `nix develop`)                            │
│                                                                             │
│   🦀 Rust Toolchain       rustc · cargo · rust-analyzer · clippy           │
│   📦 Node.js & pnpm       Frontend-Entwicklung                             │
│   📋 buf                  Protobuf Code-Generierung                        │
│   ⚙️ just                 Task Runner (alle `just` Befehle)                │
│   🗄️ sqlx CLI             Datenbank-Migrationen                            │
│   🔗 mold                 Schneller Linker                                 │
│   🧪 cargo-nextest        Schnellere Tests                                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2️⃣ Docker installieren

<details open>
<summary><strong>macOS</strong></summary>

**Option A: Download (empfohlen)**

1. Download: [Docker Desktop für Mac](https://www.docker.com/products/docker-desktop/)
2. Installieren und starten
3. Warten bis Docker läuft (Wal-Icon in Menüleiste)

**Option B: Homebrew**

```bash
brew install --cask docker
```

</details>

<details>
<summary><strong>Linux (Ubuntu/Debian)</strong></summary>

```bash
# Docker Engine installieren
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Ohne sudo verwenden
sudo usermod -aG docker $USER
newgrp docker

# Verifizieren
docker --version
```

**Optional:** [Docker Desktop für Linux](https://www.docker.com/products/docker-desktop/) für GUI.

</details>

### Verifizieren

```bash
docker ps
```

> ℹ️ Docker Desktop muss gestartet sein.

---

## 3️⃣ Git & SSH Setup (Optional)

> Nur nötig für SSH-Zugriff auf das Repository oder Commit-Signierung.

<details>
<summary><strong>Git installieren</strong></summary>

**macOS:**

```bash
# Meist bereits installiert, sonst:
xcode-select --install
# oder via Nix:
nix profile install nixpkgs#git
```

**Linux:**

```bash
sudo apt update && sudo apt install git
# oder via Nix:
nix profile install nixpkgs#git
```

</details>

<details>
<summary><strong>SSH-Key erstellen</strong></summary>

```bash
# Authentication Key (Repository klonen/pushen)
ssh-keygen -t ed25519 -C "deine-email@example.com" -f ~/.ssh/id_ed25519

# Signing Key (Commits signieren)
ssh-keygen -t ed25519 -C "git-signing" -f ~/.ssh/id_ed25519_signing -N ""
```

</details>

<details>
<summary><strong>SSH-Agent konfigurieren</strong></summary>

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

**Linux:**

```bash
eval "$(ssh-agent -s)"
cat >> ~/.ssh/config << 'EOF'
Host github.com
    AddKeysToAgent yes
    IdentityFile ~/.ssh/id_ed25519
EOF
ssh-add ~/.ssh/id_ed25519
```

</details>

<details>
<summary><strong>Keys zu GitHub hinzufügen</strong></summary>

```bash
# Keys anzeigen
cat ~/.ssh/id_ed25519.pub
cat ~/.ssh/id_ed25519_signing.pub
```

1. **GitHub → Settings → SSH and GPG keys**
2. **New SSH key** → Type: **Authentication Key** → `id_ed25519.pub` einfügen
3. **New SSH key** → Type: **Signing Key** → `id_ed25519_signing.pub` einfügen

</details>

<details>
<summary><strong>Git konfigurieren</strong></summary>

```bash
git config --global user.name "Dein Name"
git config --global user.email "deine-email@example.com"
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519_signing.pub
git config --global commit.gpgsign true
```

</details>

---

## 4️⃣ Projekt starten

### Schritt für Schritt

```bash
# 1. Repository klonen
git clone git@github.com:NiklasJavier/erynoa.git
cd erynoa

# 2. Nix Dev-Shell betreten
nix develop

# 3. Projekt starten
just dev
```

<div align="center">

⏳ **~2 Minuten warten** → 🌐 **http://localhost:3001**

</div>

### Was passiert automatisch?

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   `just dev` startet:                                                       │
│                                                                             │
│   1. 🐳 Docker-Services                                                     │
│      PostgreSQL · DragonflyDB · MinIO · ZITADEL                            │
│                                                                             │
│   2. 🔐 ZITADEL Auto-Setup                                                  │
│      Projekt · OIDC Apps · Test-User                                       │
│                                                                             │
│   3. 🦀 Backend                                                             │
│      Rust API auf Port 3000                                                │
│                                                                             │
│   4. 🎨 Frontends                                                           │
│      Console · Platform · Docs                                             │
│                                                                             │
│   5. 🔀 Caddy Proxy                                                         │
│      Alles unter Port 3001                                                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🎉 Fertig!

### Alle Services

| Service      | URL                            | Beschreibung   |
| :----------- | :----------------------------- | :------------- |
| 🌐 **Proxy** | http://localhost:3001          | Hauptzugang    |
| 📊 Console   | http://localhost:3001/console  | Admin          |
| 🖥️ Platform  | http://localhost:3001/platform | Hauptplattform |
| 📖 Docs      | http://localhost:3001/docs     | Dokumentation  |
| 🔌 API       | http://localhost:3001/api      | Backend API    |
| 🦀 Backend   | http://localhost:3000          | Direkt         |
| 🔐 ZITADEL   | http://localhost:8080          | Auth           |
| 📦 MinIO     | http://localhost:9001          | Storage        |

### Test-Login

| Rolle | User            | Passwort     |
| :---- | :-------------- | :----------- |
| User  | `testuser`      | `Test123!`   |
| Admin | `zitadel-admin` | `Password1!` |

---

## 🔧 Wichtige Befehle

### Entwicklung

| Befehl             | Beschreibung          |
| :----------------- | :-------------------- |
| `just dev`         | 🚀 **Startet alles**  |
| `just dev console` | Nur Console           |
| `just status`      | Service-Status        |
| `just logs`        | Alle Logs             |
| `just stop`        | Container stoppen     |
| `just restart`     | Neustart              |
| `just reset`       | Komplett zurücksetzen |

### Setup & Wartung

| Befehl               | Beschreibung                     |
| :------------------- | :------------------------------- |
| `just init`          | Initialisieren (ohne Dev-Server) |
| `just init-env`      | `.env` erstellen                 |
| `just zitadel-setup` | Auth konfigurieren               |
| `just minio-setup`   | Storage Buckets                  |

### Backend

| Befehl       | Beschreibung      |
| :----------- | :---------------- |
| `just check` | Cargo check       |
| `just lint`  | Clippy            |
| `just fmt`   | Formatieren       |
| `just test`  | Tests             |
| `just ci`    | CI-Pipeline lokal |

<details>
<summary><strong>📋 Alle Befehle</strong></summary>

```bash
just --list
```

</details>

---

## 🐛 Troubleshooting

### Häufige Probleme

| Problem                    | Lösung                       |
| :------------------------- | :--------------------------- |
| Services starten nicht     | `just reset && just dev`     |
| Port belegt                | `just stop && lsof -i :PORT` |
| Nix: experimental-features | Siehe unten                  |
| Docker: Permission denied  | Docker Desktop starten       |
| direnv: .envrc blocked     | `direnv allow`               |

### Nix: "experimental-features" Fehler

```bash
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

### direnv: ".envrc is blocked"

```bash
cd /path/to/erynoa
direnv allow
exec zsh  # oder exec bash
```

### Docker: Permission Denied

```bash
# Prüfen ob Docker läuft
docker ps

# Falls "permission denied":
sudo usermod -aG docker $USER
newgrp docker
```

### Services starten nicht

```bash
# Komplett zurücksetzen
just reset

# Neu starten
just dev

# Logs prüfen
just logs
```

---

## 📚 Nächste Schritte

| Dokument                                        | Beschreibung              |
| :---------------------------------------------- | :------------------------ |
| [Getting Started](../guides/getting-started.md) | Schnellstart (3 Schritte) |
| [Essential Guide](../essential_guide.md)        | Alles auf einen Blick     |
| [ZITADEL Guide](../guides/zitadel.md)           | Auth konfigurieren        |
| [Architecture](../reference/architecture.md)    | System-Architektur        |
| [Style Guide](../development/style-guide.md)    | Code Standards            |

---

<div align="center">

```
┌─────────────────────────────────────────────┐
│                                             │
│   ✅ Setup abgeschlossen!                   │
│                                             │
│   nix develop                               │
│   just dev                                  │
│   → http://localhost:3001                   │
│                                             │
└─────────────────────────────────────────────┘
```

**Viel Erfolg bei der Entwicklung!**

</div>
