# VS Code Database & Cache Verbindungen im DevContainer

## Übersicht

Die VS Code IDE im DevContainer kann direkt mit der PostgreSQL-Datenbank und dem Dragonfly-Cache verbinden. Die benötigten Extensions sind automatisch installiert.

## Installierte Extensions

### PostgreSQL Extension
- **Extension ID**: `ms-ossdata.vscode-postgresql`
- **Hersteller**: Microsoft
- **Funktionen**:
  - Datenbank-Explorer
  - SQL Query Editor
  - Schema-Visualisierung
  - GitHub Copilot Integration (@pgsql agent)

### Redis/Dragonfly Extension
- **Extension ID**: `Redis.redis-for-vscode`
- **Hersteller**: Redis
- **Funktionen**:
  - Key-Explorer
  - Key-Editor (Strings, Hashes, Lists, Sets, Sorted Sets, JSON)
  - TTL-Verwaltung
  - Bulk-Operationen

## Verbindungsdetails

### PostgreSQL (App Database - OrioleDB)
- **Host**: `localhost`
- **Port**: `5432`
- **Database**: `erynoa`
- **Username**: `erynoa`
- **Password**: `erynoa`
- **Connection String**: `postgres://erynoa:erynoa@localhost:5432/erynoa`

### PostgreSQL (ZITADEL Database)
- **Host**: `localhost`
- **Port**: `5433`
- **Database**: `zitadel`
- **Username**: `zitadel`
- **Password**: `zitadel`
- **Connection String**: `postgres://zitadel:zitadel@localhost:5433/zitadel`
- **Hinweis**: Nur verfügbar wenn ZITADEL gestartet ist (`--profile auth`)

### Dragonfly Cache (Redis-kompatibel)
- **Host**: `localhost`
- **Port**: `6379`
- **Connection String**: `redis://localhost:6379`
- **Hinweis**: Dragonfly ist Redis-kompatibel, daher funktioniert die Redis Extension

## Verwendung

### PostgreSQL Extension

1. **Extension öffnen**:
   - Klicke auf das Elefanten-Icon in der Sidebar
   - Oder: `Cmd+Shift+P` → "PostgreSQL: Focus on Connections View"

2. **Verbindung herstellen**:
   - Die Verbindungen sind bereits in `devcontainer.json` konfiguriert
   - Klicke auf "Connect" neben der gewünschten Datenbank
   - Oder: Rechtsklick → "Connect"

3. **Datenbank erkunden**:
   - Erweitere die Verbindung in der Sidebar
   - Navigiere durch Schemas, Tabellen, Views, etc.
   - Rechtsklick auf Tabellen für: "Show Table Data", "Generate SQL", etc.

4. **SQL Queries ausführen**:
   - Öffne eine neue SQL-Datei (`.sql`)
   - Schreibe deine Query
   - `Cmd+E` oder Rechtsklick → "Execute Query"
   - Oder: `Cmd+Shift+P` → "PostgreSQL: Execute Query"

5. **GitHub Copilot Integration**:
   - Schreibe `@pgsql` in einem SQL-File
   - Copilot kann SQL-Queries generieren und optimieren

### Redis/Dragonfly Extension

1. **Extension öffnen**:
   - Klicke auf das Redis-Icon in der Sidebar
   - Oder: `Cmd+Shift+P` → "Redis: Focus on Connections View"

2. **Verbindung herstellen**:
   - Die Verbindung ist bereits in `devcontainer.json` konfiguriert
   - Klicke auf "Connect" neben "Dragonfly Cache"
   - Oder: Rechtsklick → "Connect"

3. **Keys erkunden**:
   - Erweitere die Verbindung in der Sidebar
   - Navigiere durch die Key-Struktur
   - Filter: `*pattern*` für Suche

4. **Keys bearbeiten**:
   - Rechtsklick auf einen Key → "Edit"
   - Unterstützte Typen: String, Hash, List, Set, Sorted Set, JSON
   - TTL setzen/ändern
   - Keys löschen (einzeln oder Bulk)

5. **Commands ausführen**:
   - Öffne das Command Panel (`Cmd+Shift+P`)
   - "Redis: Execute Command"
   - Gib Redis-Commands ein (z.B. `GET key`, `SET key value`, etc.)

## Troubleshooting

### PostgreSQL Verbindung schlägt fehl

1. **Prüfe ob Datenbank läuft**:
   ```bash
   docker compose -f /workspace/infra/docker/docker-compose.yml ps db
   ```

2. **Prüfe Port-Weiterleitung**:
   - In VS Code: Ports-Tab → Suche nach `5432`
   - Sollte als "PostgreSQL (App)" gelistet sein

3. **Teste Verbindung manuell**:
   ```bash
   psql postgres://erynoa:erynoa@localhost:5432/erynoa -c "SELECT version();"
   ```

4. **Extension neu laden**:
   - `Cmd+Shift+P` → "Developer: Reload Window"

### Redis/Dragonfly Verbindung schlägt fehl

1. **Prüfe ob Cache läuft**:
   ```bash
   docker compose -f /workspace/infra/docker/docker-compose.yml ps cache
   ```

2. **Prüfe Port-Weiterleitung**:
   - In VS Code: Ports-Tab → Suche nach `6379`
   - Sollte als "Redis/Dragonfly" gelistet sein

3. **Teste Verbindung manuell**:
   ```bash
   redis-cli -h localhost -p 6379 ping
   # Sollte "PONG" zurückgeben
   ```

4. **Extension neu laden**:
   - `Cmd+Shift+P` → "Developer: Reload Window"

### Ports werden nicht weitergeleitet

1. **Prüfe `devcontainer.json`**:
   - `forwardPorts` sollte `5432`, `5433`, `6379` enthalten
   - `portsAttributes` sollte die Ports konfiguriert haben

2. **DevContainer neu starten**:
   - `Cmd+Shift+P` → "Dev Containers: Rebuild Container"

3. **Ports manuell weiterleiten**:
   - Ports-Tab → "+" → Port-Nummer eingeben

## Weitere Ressourcen

- [PostgreSQL Extension Dokumentation](https://marketplace.visualstudio.com/items?itemName=ms-ossdata.vscode-postgresql)
- [Redis Extension Dokumentation](https://marketplace.visualstudio.com/items?itemName=Redis.redis-for-vscode)
- [DevContainer Port Forwarding](https://code.visualstudio.com/docs/devcontainers/containers#_forwarding-or-publishing-a-port)
