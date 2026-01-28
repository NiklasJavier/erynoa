# Erynoa – Getting Started

> **Dokumenttyp:** Guide
> **Zielgruppe:** Neue Entwickler
> **Dauer:** ca. 10 Minuten
> **Voraussetzungen:** Nix, Docker

---

## Willkommen

Dieser Guide bringt dich in **3 Schritten** zu einem laufenden Erynoa-Entwicklungsumfeld.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   🚀 GETTING STARTED                                                        │
│                                                                             │
│   ┌─────────┐      ┌─────────┐      ┌─────────┐      ┌─────────┐           │
│   │    1    │ ───▶ │    2    │ ───▶ │    3    │ ───▶ │   ✅    │           │
│   │  Clone  │      │   Nix   │      │  Start  │      │  Done!  │           │
│   └─────────┘      └─────────┘      └─────────┘      └─────────┘           │
│                                                                             │
│   ~30 Sek.         ~60 Sek.         ~2 Min.          🎉                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Voraussetzungen

Bevor du startest, installiere diese beiden Tools:

| Tool       | Zweck                | Installation                                                      |
| :--------- | :------------------- | :---------------------------------------------------------------- |
| **Nix**    | Entwicklungsumgebung | Siehe unten                                                       |
| **Docker** | Container-Services   | [Docker Desktop](https://www.docker.com/products/docker-desktop/) |

### Nix installieren

<details>
<summary><strong>macOS</strong></summary>

```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

Terminal neu starten, dann:

```bash
nix --version
```

</details>

<details>
<summary><strong>Linux (Ubuntu/Debian)</strong></summary>

```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

Terminal neu starten, dann:

```bash
nix --version
```

</details>

---

## 🚀 3 Schritte zum Start

### Schritt 1: Repository klonen

```bash
git clone git@github.com:NiklasJavier/erynoa.git
cd erynoa
```

### Schritt 2: Nix Dev-Shell betreten

```bash
nix develop
```

> ⏳ Beim ersten Mal dauert das 1-2 Minuten. Nix lädt alle Tools automatisch.

### Schritt 3: Projekt starten

```bash
just dev
```

<div align="center">

⏳ **~2 Minuten warten** → 🌐 **http://localhost:3001** öffnen

</div>

---

## 🎉 Fertig!

Nach dem Start sind folgende Services verfügbar:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   🌐 http://localhost:3001                                                 │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   /console   ───▶  📊 Admin Console                                │  │
│   │   /platform  ───▶  🖥️ Hauptplattform                               │  │
│   │   /docs      ───▶  📖 Dokumentation                                │  │
│   │   /api       ───▶  🔌 Backend API                                  │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Alle Services

| Service      | URL                            | Beschreibung        |
| :----------- | :----------------------------- | :------------------ |
| 🌐 **Proxy** | http://localhost:3001          | Hauptzugang (Caddy) |
| 📊 Console   | http://localhost:3001/console  | Admin-Bereich       |
| 🖥️ Platform  | http://localhost:3001/platform | Hauptplattform      |
| 📖 Docs      | http://localhost:3001/docs     | Dokumentation       |
| 🔌 API       | http://localhost:3001/api      | Backend API         |
| 🦀 Backend   | http://localhost:3000          | Direkt (für Tests)  |
| 🔐 ZITADEL   | http://localhost:8080          | Auth Server         |
| 📦 MinIO     | http://localhost:9001          | Storage Console     |

### Test-Login

| Rolle | User            | Passwort     |
| :---- | :-------------- | :----------- |
| User  | `testuser`      | `Test123!`   |
| Admin | `zitadel-admin` | `Password1!` |

---

## 🛠️ Was Nix automatisch bereitstellt

Wenn du `nix develop` ausführst, werden alle Tools geladen – ohne manuelle Installation:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ✅ AUTOMATISCH INSTALLIERT                                               │
│                                                                             │
│   🦀 Rust Toolchain        rust-analyzer · clippy · cargo-nextest          │
│   📦 Node.js & pnpm        Frontend-Entwicklung                            │
│   📋 buf                   Protobuf Code-Generierung                       │
│   ⚙️ just                  Task Runner                                     │
│   🗄️ sqlx CLI              Datenbank-Migrationen                           │
│   🔗 mold                  Schneller Linker                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Vorteile:**

- ⚡ **Schnell** – Keine manuelle Installation
- 🔒 **Reproduzierbar** – Gleiche Tools für alle
- 🧹 **Sauber** – Keine System-Verschmutzung

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

### Code Quality

| Befehl       | Beschreibung |
| :----------- | :----------- |
| `just check` | Cargo check  |
| `just lint`  | Clippy       |
| `just test`  | Tests        |

<details>
<summary><strong>📋 Alle Befehle</strong></summary>

```bash
just --list
```

</details>

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
lsof -i :3001
```

### Nix: "experimental-features" Fehler

```bash
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

### Docker-Probleme

```bash
# Docker Desktop neustarten, dann:
just reset
just dev
```

### Häufige Probleme

| Problem                | Lösung                   |
| :--------------------- | :----------------------- |
| Services starten nicht | `just reset && just dev` |
| Auth-Fehler            | `just zitadel-reset`     |
| Port belegt            | `just stop`              |
| Nix-Fehler             | Terminal neu starten     |

---

## 📖 Nächste Schritte

| Was                   | Dokument                                     |
| :-------------------- | :------------------------------------------- |
| Alles auf einen Blick | [Essential Guide](../essential_guide.md)     |
| System-Architektur    | [Architecture](../reference/architecture.md) |
| Code Standards        | [Style Guide](../development/style-guide.md) |
| Offene Aufgaben       | [TODOs](../development/todos.md)             |
| Auth konfigurieren    | [ZITADEL Guide](./zitadel.md)                |

---

## 🧠 Protokoll-Konzepte

Für das Protokoll-Design (ERY, ECHO, NOA) siehe:

| Dokument                                       | Inhalt                     |
| :--------------------------------------------- | :------------------------- |
| [📋 Fachkonzept](../../concept/fachkonzept.md) | Vollständige Spezifikation |
| [🎯 Kernkonzept](../../concept/kernkonzept.md) | High-Level Überblick       |
| [📖 Glossar](../../concept/glossary.md)        | Begriffsdefinitionen       |

---

<div align="center">

```
┌─────────────────────────────────────────────┐
│                                             │
│   🎉 Du bist startklar!                     │
│                                             │
│   http://localhost:3001                     │
│                                             │
└─────────────────────────────────────────────┘
```

**Viel Erfolg bei der Entwicklung!**

</div>
