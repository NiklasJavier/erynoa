# 🚀 Getting Started

**Schnellstart-Anleitung für neue Entwickler**

---

## Voraussetzungen

- Docker Desktop installiert und gestartet
- VS Code mit Dev Containers Extension (optional, aber empfohlen)

---

## Quick Start

```bash
just dev
```

Das startet alles:
- **Proxy** auf http://localhost:3001 (Caddy Reverse Proxy)
  - **Console** auf http://localhost:3001/console
  - **Platform** auf http://localhost:3001/platform
  - **Docs** auf http://localhost:3001/docs
- **Backend** direkt auf http://localhost:3000 (Rust API)
- **ZITADEL** auf http://localhost:8080 (Auth)
- **MinIO** auf http://localhost:9001 (S3 Storage Console)

**Test Login:**
- User: `testuser` / `Test123!`
- Admin: `zitadel-admin` / `Password1!`

---

## Was passiert beim Start?

1. **Hintergrund-Services starten** (DB, Cache, MinIO, ZITADEL)
2. **Health-Checks warten** (bis alle Services bereit sind)
3. **Init-Skripte ausführen** (nur beim ersten Mal)
4. **Console + Backend starten** mit Hot-Reload

---

## Nächste Schritte

1. **Setup**: Siehe [Setup Guide (macOS)](../setup/setup.md) oder [Dev Setup](../setup/dev_setup.md) für vollständige Entwicklungsumgebung
2. **Architektur**: Lese [Architecture](../reference/architecture.md) für System-Überblick
3. **ZITADEL**: Folge [ZITADEL Setup](zitadel.md) für Authentifizierung

---

## Wichtige Befehle

| Befehl | Beschreibung |
|--------|--------------|
| `just dev` | Startet alles (Console + Platform + Docs + Backend) |
| `just dev [frontend]` | Startet spezifisches Frontend (console, platform, docs) |
| `just status` | Zeigt Status aller Services |
| `just check` | Health Check aller Services |
| `just restart` | Schneller Neustart aller Dev-Services |
| `just stop` | Stoppt alle Container |
| `just logs [service]` | Logs anzeigen (alle oder spezifischer Service) |

---

## Troubleshooting

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

### Weitere Hilfe
- [Setup Guide (macOS)](../setup/setup.md) - Detaillierte Setup-Anleitung
- [Dev Setup](../setup/dev_setup.md) - Container-in-Container Entwicklung
- [Configuration](../reference/config.md) - Service-Konfiguration
- [todos](../development/todos.md) - Bekannte Issues

---

**Fertig!** Du kannst jetzt mit der Entwicklung beginnen. 🎉
