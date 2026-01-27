# Setup Guide (macOS)

Diese Anleitung beschreibt alle Schritte, um das **GS-Backend** Projekt auf einem frischen macOS-System einzurichten.

---

## Voraussetzungen

- macOS 12+ (Monterey oder neuer)
- Admin-Rechte (für Homebrew & Nix)
- GitHub Account mit Zugriff auf das Repository

---

## 1. Xcode Command Line Tools

```bash
xcode-select --install
```

---

## 2. Homebrew installieren

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Nach der Installation (Apple Silicon):
```bash
echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
eval "$(/opt/homebrew/bin/brew shellenv)"
```

---

## 3. Nix installieren (Package Manager)

Wir nutzen Nix für reproduzierbare Builds. Installiere den Determinate Nix Installer:

```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

Terminal neu starten, dann verifizieren:
```bash
nix --version
```

---

## 4. Docker Desktop installieren

Download von: https://www.docker.com/products/docker-desktop/

Oder via Homebrew:
```bash
brew install --cask docker
```

Nach der Installation Docker Desktop starten und warten bis es läuft.

---

## 5. SSH-Key für GitHub einrichten

### 5.1 SSH-Key erstellen (falls noch nicht vorhanden)

```bash
# Key für Authentication (Repository klonen/pushen)
ssh-keygen -t ed25519 -C "deine-email@example.com" -f ~/.ssh/id_ed25519

# Key für Commit-Signierung
ssh-keygen -t ed25519 -C "git-signing" -f ~/.ssh/id_ed25519_signing -N ""
```

### 5.2 SSH-Agent konfigurieren

```bash
# SSH-Agent starten
eval "$(ssh-agent -s)"

# SSH-Config erstellen/erweitern
cat >> ~/.ssh/config << 'EOF'
Host github.com
    AddKeysToAgent yes
    UseKeychain yes
    IdentityFile ~/.ssh/id_ed25519
EOF

# Key zum Agent hinzufügen
ssh-add --apple-use-keychain ~/.ssh/id_ed25519
```

### 5.3 Public Keys zu GitHub hinzufügen

```bash
# Authentication Key anzeigen
cat ~/.ssh/id_ed25519.pub

# Signing Key anzeigen
cat ~/.ssh/id_ed25519_signing.pub
```

1. Gehe zu **GitHub → Settings → SSH and GPG keys**
2. **New SSH key** → Key Type: **Authentication Key** → Füge `id_ed25519.pub` ein
3. **New SSH key** → Key Type: **Signing Key** → Füge `id_ed25519_signing.pub` ein

### 5.4 Verbindung testen

```bash
ssh -T git@github.com
# Erwartete Ausgabe: "Hi USERNAME! You've successfully authenticated..."
```

---

## 6. Git konfigurieren

### 6.1 Basis-Konfiguration

```bash
git config --global user.name "Dein Name"
git config --global user.email "deine-email@example.com"
```

### 6.2 SSH-Signierung aktivieren (statt GPG)

```bash
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519_signing.pub
git config --global commit.gpgsign true
git config --global tag.gpgsign true

# Lokale Signatur-Verifizierung (optional)
echo "deine-email@example.com $(cat ~/.ssh/id_ed25519_signing.pub)" > ~/.ssh/allowed_signers
git config --global gpg.ssh.allowedSignersFile ~/.ssh/allowed_signers
```

---

## 7. Repository klonen

```bash
# Projektverzeichnis erstellen (optional)
mkdir -p ~/Development/erynoa
cd ~/Development/erynoa

# Repository klonen
git clone git@github.com:NiklasJavier/GS-Backend.git
cd GS-Backend
```

---

## 8. Entwicklungsumgebung starten

### Option A: Mit Nix (empfohlen)

```bash
# Nix Dev-Shell betreten (lädt alle Tools automatisch)
nix develop

# Infrastruktur starten (PostgreSQL + DragonflyDB)
docker compose -f docker/docker-compose.yml up -d

# Dev-Server starten
just dev
```

### Option B: Mit VS Code DevContainer

1. VS Code öffnen: `code .`
2. `Cmd+Shift+P` → "Dev Containers: Reopen in Container"
3. Warten bis der Container bereit ist (Nix-Umgebung wird automatisch geladen)
4. Terminal öffnen → alle Tools sind sofort verfügbar:
   - `just dev` - Dev-Server starten
   - `just db-migrate` - Migrationen ausführen
   - `cargo check` - Projekt prüfen

**Features des DevContainers:**
- ✅ **Automatische Nix-Umgebung** via `direnv` - alle Tools (cargo, just, sqlx, etc.) sind direkt verfügbar
- ✅ **Automatische `.env`** - wird aus `.env.example` erstellt falls nicht vorhanden
- ✅ **Docker-in-Docker** - Services (DB, Cache, ZITADEL) laufen automatisch
- ✅ **Migrationen** - werden beim Start automatisch ausgeführt
- ✅ **SSH/GPG-Keys** - vom Host übernommen für Git-Signing

---

## 9. Verfügbare Befehle

Alle Befehle über `just`:

| Befehl | Beschreibung |
|--------|--------------|
| `just dev` | Dev-Server mit Hot Reload |
| `just run` | Server einmal starten |
| `just check` | Cargo check |
| `just test` | Tests ausführen |
| `just lint` | Clippy Linter |
| `just fmt` | Code formatieren |
| `just ci` | fmt + lint + test |
| `just build` | Nix Build |
| `just build-static` | Statisches musl Binary |
| `just docker-load` | Docker Image bauen & laden |
| `just db-migrate` | Migrations ausführen |
| `just db-reset` | Datenbank zurücksetzen |

---

## 10. Infrastruktur verwalten

### Services starten
```bash
docker compose -f docker/docker-compose.yml up -d
```

### Services stoppen
```bash
docker compose -f docker/docker-compose.yml down
```

### Mit ZITADEL (Auth-Service)
```bash
docker compose -f docker/docker-compose.yml --profile auth up -d
```

### Logs anzeigen
```bash
docker compose -f docker/docker-compose.yml logs -f
```

---

## 11. Endpoints

Nach dem Start läuft der Server auf:

| Service | URL |
|---------|-----|
| API | http://localhost:3000 |
| Health Check | http://localhost:3000/health |
| ZITADEL (optional) | http://localhost:8080 |

---

## Troubleshooting

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

### Git Push: GPG/SSH Signing Fehler
Stelle sicher, dass der Signing Key zu GitHub hinzugefügt wurde:
```bash
# Prüfen welcher Key konfiguriert ist
git config --global user.signingkey

# Key nochmal anzeigen
cat ~/.ssh/id_ed25519_signing.pub
```

### direnv: ".envrc is blocked"
Beim ersten Öffnen des Projekts erscheint diese Fehlermeldung:
```
direnv: error .envrc is blocked. Run `direnv allow` to approve its content
```

**Lösung:**
```bash
# Im Projektverzeichnis ausführen:
cd /path/to/GS-Backend-2
direnv allow
```

Danach die Shell neu laden:
```bash
exec zsh
# oder einfach ein neues Terminal öffnen
```

**Hinweis:** Dies ist ein Sicherheitsfeature von `direnv`. Die `.envrc` muss einmalig erlaubt werden, danach ist sie persistent. Die Fehlermeldung erscheint nur beim ersten Mal oder wenn die `.envrc` geändert wurde.

### SQLx: "DATABASE_URL must be set"
```bash
# Automatisch aus .env.example erstellen:
just init-env

# Oder manuell:
cp .env.example .env

# Oder manuell setzen:
export DATABASE_URL="postgres://erynoa:erynoa@localhost:5432/erynoa"
```

### Port bereits belegt
```bash
# Prozess auf Port finden
lsof -i :5432
lsof -i :3000

# Docker-Container stoppen
docker compose -f docker/docker-compose.yml down
```

---

## Nützliche Tools (optional)

```bash
# Besseres Terminal
brew install --cask iterm2

# VS Code
brew install --cask visual-studio-code

# Datenbank-Client
brew install --cask tableplus

# API-Testing
brew install --cask bruno
```

---

## Zusammenfassung der Installationsreihenfolge

1. ✅ Xcode Command Line Tools
2. ✅ Homebrew
3. ✅ Nix
4. ✅ Docker Desktop
5. ✅ SSH-Keys erstellen & zu GitHub hinzufügen
6. ✅ Git konfigurieren (mit SSH-Signierung)
7. ✅ Repository klonen
8. ✅ `nix develop` → `docker compose up -d` → `just dev`

**Fertig!** 🚀
