# DevContainer auf Remote-Host

## Übersicht

Wenn du einen Remote-Host (z.B. einen Server) zum Entwickeln nutzt, laufen die Services **auf dem Remote-Host**, nicht auf deinem lokalen Rechner.

## Wie es funktioniert

### Standard-Verhalten

- **Lokaler DevContainer**: Services laufen auf deinem lokalen Rechner
- **Remote DevContainer**: Services laufen auf dem Remote-Server

`host.docker.internal` zeigt immer auf den Docker-Host, auf dem der Container läuft:
- Lokal → lokaler Rechner
- Remote → Remote-Server

## Services starten

### Auf Remote-Server

```bash
# SSH zum Remote-Server
ssh user@remote-server

# Services starten
cd /path/to/erynoa
just services
# oder
cd infra/docker && docker compose up -d db cache minio
```

### DevContainer öffnen

1. VS Code/Cursor: "Dev Containers: Connect to Host" → Remote-Server
2. DevContainer öffnen
3. Services sind automatisch über `host.docker.internal` erreichbar

## Alternative: Services auf anderem Host

Falls die Services auf einem anderen Host laufen sollen (z.B. lokaler Rechner bei Remote-DevContainer):

### Option 1: Umgebungsvariable auf Host-System setzen

**WICHTIG**: `SERVICE_HOST` muss auf dem **Host-System** gesetzt werden (nicht im Container), da Docker Compose diese Variable vom Host liest.

**Auf lokalem Rechner** (vor DevContainer-Start):
```bash
export SERVICE_HOST=192.168.1.100  # IP des Hosts mit Services
# Dann DevContainer öffnen
```

**Auf Remote-Server** (vor DevContainer-Start):
```bash
export SERVICE_HOST=192.168.1.100  # IP des Hosts mit Services
# Dann DevContainer öffnen
```

**Oder in `.env` Datei** im Workspace-Root:
```bash
SERVICE_HOST=192.168.1.100
```

### Option 2: Docker Compose Override

Erstelle `.devcontainer/docker-compose.override.yml`:

```yaml
services:
  dev:
    environment:
      - APP_DATABASE__HOST=192.168.1.100
      - APP_CACHE__URL=redis://192.168.1.100:6379
      - APP_STORAGE__ENDPOINT=http://192.168.1.100:9000
      - APP_AUTH__ISSUER=http://192.168.1.100:8080
      - APP_AUTH__INTERNAL_ISSUER=http://192.168.1.100:8080
```

**Hinweis**: Diese Option überschreibt die `SERVICE_HOST` Variable komplett.

## Port-Forwarding

Port-Forwarding funktioniert automatisch:
- Services auf Remote-Server → Ports werden zu deinem lokalen Rechner weitergeleitet
- VS Code Extensions können auf `localhost` zugreifen

## Empfehlung

**Für Remote-Entwicklung**: Services sollten auf dem Remote-Server laufen
- Bessere Performance (keine Netzwerk-Latenz)
- Einfacheres Setup
- Konsistente Umgebung

**Nur wenn nötig**: Services auf anderem Host
- Z.B. wenn Services bereits auf lokalem Rechner laufen
- Z.B. wenn Services auf dediziertem Server laufen
