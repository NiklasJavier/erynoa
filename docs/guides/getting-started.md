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
- Frontend: http://localhost:5173
- Backend: http://localhost:3000
- ZITADEL: http://localhost:8080
- MinIO: http://localhost:9001

**Test Login:**
- User: `testuser` / `Test123!`
- Admin: `zitadel-admin` / `Password1!`

---

## Was passiert beim Start?

1. **Hintergrund-Services starten** (DB, Cache, MinIO, ZITADEL)
2. **Health-Checks warten** (bis alle Services bereit sind)
3. **Init-Skripte ausführen** (nur beim ersten Mal)
4. **Frontend + Backend starten** mit Hot-Reload

---

## Nächste Schritte

1. **Setup**: Siehe [Setup Guide](setup.md) für vollständige Entwicklungsumgebung
2. **Architektur**: Lese [Architecture](reference/architecture.md) für System-Überblick
3. **ZITADEL**: Folge [ZITADEL Setup](zitadel.md) für Authentifizierung

---

## Wichtige Befehle

| Befehl | Beschreibung |
|--------|--------------|
| `just dev` | Startet alles |
| `just status` | Zeigt Status aller Services |
| `just restart-dev` | Schneller Neustart |
| `just docker-stop` | Stoppt alle Container |

---

## Troubleshooting

### Services starten nicht
```bash
just reset
just dev
```

### Port bereits belegt
```bash
just docker-stop
lsof -i :3000  # oder :5173, :8080
```

### Weitere Hilfe
- [Setup Guide](setup.md) - Detaillierte Setup-Anleitung
- [Configuration](reference/config.md) - Service-Konfiguration
- [TODOs](development/todos.md) - Bekannte Issues

---

**Fertig!** Du kannst jetzt mit der Entwicklung beginnen. 🎉
