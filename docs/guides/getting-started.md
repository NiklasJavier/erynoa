# 🚀 Getting Started

**Schnellstart-Anleitung - In 3 Schritten zum laufenden Projekt**

**Letzte Aktualisierung**: 2026-01-27 (23:40)

---

## ⚡ Schnellstart (Keine Installation nötig)

**Voraussetzungen:**
- Nix installiert (siehe unten)
- Docker Desktop installiert und gestartet

**3 Schritte:**

```bash
# 1. Repository klonen
git clone git@github.com:NiklasJavier/erynoa.git
cd erynoa

# 2. Nix Dev-Shell betreten (lädt alle Tools automatisch)
nix develop

# 3. Projekt starten
just dev
```

**Fertig!** 🎉

Das startet alles:
- **Proxy** auf http://localhost:3001 (Caddy Reverse Proxy)
  - **Console** auf http://localhost:3001/console
  - **Platform** auf http://localhost:3001/platform
  - **Docs** auf http://localhost:3001/docs
  - **Backend API** auf http://localhost:3001/api
- **Backend** direkt auf http://localhost:3000 (für Tests)
- **ZITADEL** auf http://localhost:8080 (Auth) - automatisch konfiguriert
- **MinIO** auf http://localhost:9001 (S3 Storage Console)
- **PostgreSQL** (OrioleDB) und **DragonflyDB** (Redis) im Hintergrund

**Test Login:**
- User: `testuser` / `Test123!`
- Admin: `zitadel-admin` / `Password1!`

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

## 🛠️ Was wird automatisch installiert?

Wenn du `nix develop` ausführst, werden folgende Tools automatisch bereitgestellt:

- ✅ **Rust Toolchain** (inkl. rust-analyzer, clippy, cargo-nextest)
- ✅ **Node.js & pnpm** (für Frontend-Entwicklung)
- ✅ **buf** (Protobuf Code-Generierung)
- ✅ **just** (Task Runner - alle `just` Befehle)
- ✅ **sqlx CLI** (Datenbank-Migrationen)
- ✅ **Alle Build-Tools** (mold linker, etc.)

**Hinweis:** Die Protobuf-Konfigurationsdateien (`buf.gen.yaml` und `buf.yaml`) befinden sich im Projekt-Root und werden automatisch von `buf generate` verwendet.

**Vorteile:**
- ⚡ **Schnell**: Keine manuelle Tool-Installation nötig
- 🔒 **Reproduzierbar**: Gleiche Tools für alle Entwickler
- 🧹 **Sauber**: Keine System-Installationen (außer Nix selbst)

---

## 📋 Vollständige Setup-Anleitung

Falls du mehr Details benötigst oder Probleme hast, siehe:

- **[Setup Guide (macOS)](../setup/setup.md)** - Detaillierte Anleitung für macOS
- **[Setup Guide (Ubuntu)](../setup/setup.md#ubuntu)** - Detaillierte Anleitung für Ubuntu
- **[Dev Setup](../setup/dev_setup.md)** - Container-in-Container Entwicklung

---

## 🔧 Wichtige Befehle

| Befehl | Beschreibung |
|--------|--------------|
| `just dev` | **Startet alles** (Console + Platform + Docs + Backend) |
| `just dev [frontend]` | Startet spezifisches Frontend (console, platform, docs) |
| `just status` | Zeigt Status aller Services |
| `just logs [service]` | Logs anzeigen (alle oder spezifischer Service) |
| `just stop` | Stoppt alle Container |
| `just restart` | Schneller Neustart aller Dev-Services |
| `just reset` | Alles löschen und neu starten |

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

### Weitere Hilfe
- [Setup Guide](../setup/setup.md) - Detaillierte Setup-Anleitung
- [Configuration](../reference/config.md) - Service-Konfiguration
- [Architecture](../reference/architecture.md) - System-Architektur

---

**Fertig!** Du kannst jetzt mit der Entwicklung beginnen. 🎉
