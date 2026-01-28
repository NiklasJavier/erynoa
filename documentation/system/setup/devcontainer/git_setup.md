# 🔐 Git-Konfiguration im DevContainer

**Letzte Aktualisierung**: 2026-01-27

Dieses Dokument erklärt, wie die Git-Konfiguration vom Host in den DevContainer übernommen wird, damit du 1:1 die gleichen Einstellungen hast.

---

## ✅ Automatische Übernahme

Der DevContainer übernimmt automatisch folgende Git-Einstellungen vom Host:

### 1. Git User-Konfiguration

- `user.name` - Dein Name
- `user.email` - Deine E-Mail-Adresse

### 2. Git Signing-Konfiguration

- `gpg.format` - Format (ssh oder gpg)
- `user.signingkey` - Signing-Key (SSH oder GPG)
- `commit.gpgsign` - Commits signieren
- `tag.gpgsign` - Tags signieren
- `gpg.ssh.allowedSignersFile` - Allowed Signers Datei

### 3. SSH-Keys

- SSH-Keys werden vom Host gemountet (`.ssh-host`)
- SSH-Agent Socket wird vom Host durchgeleitet
- Signing-Keys werden automatisch verlinkt

### 4. GPG-Keys

- GPG-Keys werden vom Host gemountet (`.gnupg-host`)
- GPG-Agent wird im Container gestartet

---

## 🔧 Setup-Methoden

### Methode 1: Environment-Variablen (Empfohlen)

Setze auf dem **Host** folgende Environment-Variablen:

```bash
export GIT_USER_NAME="Dein Name"
export GIT_USER_EMAIL="deine-email@example.com"
```

Diese werden automatisch in den DevContainer übernommen.

**Vorteil**: Funktioniert sofort, keine manuelle Konfiguration nötig.

### Methode 2: Gemountete .gitconfig

Die `.gitconfig` vom Host wird automatisch gemountet (falls vorhanden):

- Host: `~/.gitconfig`
- Container: `~/.gitconfig-host` (read-only)

Der DevContainer liest die Werte und setzt sie automatisch.

**Vorteil**: Alle Git-Einstellungen werden übernommen.

### Methode 3: Manuelle Konfiguration im Container

Falls keine automatische Übernahme funktioniert, konfiguriere manuell im Container:

```bash
git config --global user.name "Dein Name"
git config --global user.email "deine-email@example.com"
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519_signing.pub
git config --global commit.gpgsign true
git config --global tag.gpgsign true
```

---

## 🔍 Prüfen der Konfiguration

### Im DevContainer prüfen:

```bash
# Git User-Konfiguration
git config --global user.name
git config --global user.email

# Git Signing-Konfiguration
git config --global gpg.format
git config --global user.signingkey
git config --global commit.gpgsign
git config --global tag.gpgsign

# SSH-Keys
ssh-add -l

# Git-Status prüfen
git status
```

---

## 🚀 Git Push vom DevContainer

Nach der Konfiguration kannst du direkt vom DevContainer pushen:

```bash
# Änderungen committen
git add .
git commit -m "Deine Commit-Nachricht"

# Push (verwendet automatisch SSH-Keys vom Host)
git push
```

**Hinweis**: Der SSH-Agent vom Host wird automatisch verwendet, daher funktionieren alle SSH-Keys, die auf dem Host im SSH-Agent geladen sind.

---

## 🐛 Troubleshooting

### Problem: Git user.name/user.email nicht gesetzt

**Lösung**: Setze Environment-Variablen auf dem Host oder konfiguriere manuell im Container.

### Problem: SSH-Keys funktionieren nicht

**Lösung**: 
1. Prüfe, ob SSH-Keys auf dem Host im SSH-Agent geladen sind: `ssh-add -l` (auf dem Host)
2. Falls nicht: `ssh-add ~/.ssh/id_ed25519` (auf dem Host)
3. DevContainer neu starten

### Problem: Signing funktioniert nicht

**Lösung**:
1. Prüfe, ob Signing-Key vorhanden ist: `ls ~/.ssh/id_ed25519_signing.pub`
2. Prüfe Git-Konfiguration: `git config --global user.signingkey`
3. Falls nicht gesetzt: `git config --global user.signingkey ~/.ssh/id_ed25519_signing.pub`

### Problem: GPG-Signing funktioniert nicht

**Lösung**:
1. Prüfe, ob GPG-Keys vorhanden sind: `ls ~/.gnupg-host/`
2. DevContainer neu starten (GPG-Keys werden beim Start kopiert)

---

## 📝 Zusammenfassung

✅ **Automatisch übernommen**:
- Git user.name und user.email (via Environment-Variablen oder .gitconfig)
- SSH-Keys (gemountet vom Host)
- SSH-Agent (Socket vom Host)
- GPG-Keys (kopiert vom Host)
- Git Signing-Konfiguration

✅ **Funktioniert sofort**:
- `git push` verwendet SSH-Keys vom Host
- Commits werden automatisch signiert (falls konfiguriert)
- Alle Git-Einstellungen sind identisch zum Host

---

**Hinweis**: Bei Problemen prüfe die Logs beim DevContainer-Start. Die Git-Konfiguration wird in `.devcontainer/setup-and-init.sh` automatisch eingerichtet.
