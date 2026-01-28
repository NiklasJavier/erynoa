# Unified Deployment (Single Binary Deployment)

> **Status**: Geplant
> **Ziel**: Ein einziges Docker-Image für Backend + Frontends

## Übersicht

Dieser Guide beschreibt den **"Single Binary Deployment"** oder **"Modular Monolith"** Ansatz für Erynoa. Die Idee: SvelteKit-Apps werden zu statischen Assets (HTML/JS/CSS) kompiliert und vom Rust-Server (Axum) ausgeliefert.

### Architektur

```
┌─────────────────────────────────────────────────────────┐
│                    Docker Container                      │
│  ┌───────────────────────────────────────────────────┐  │
│  │              Axum Server (Rust)                   │  │
│  │                                                   │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌──────────┐  │  │
│  │  │  /api/v1/*  │  │  /console/* │  │    /*    │  │  │
│  │  │   (API)     │  │  (Console)  │  │(Platform)│  │  │
│  │  └─────────────┘  └─────────────┘  └──────────┘  │  │
│  │         │                │               │        │  │
│  │         ▼                ▼               ▼        │  │
│  │   API Handler      Static Files    Static Files   │  │
│  │                   (public/console) (public/platform) │
│  └───────────────────────────────────────────────────┘  │
│                           │                              │
│                           ▼                              │
│                      Port 8080                           │
└─────────────────────────────────────────────────────────┘
```

### Vorteile

| Vorteil                  | Beschreibung                                                                                |
| ------------------------ | ------------------------------------------------------------------------------------------- |
| **Keine CORS-Probleme**  | Frontend und Backend laufen auf demselben Port. Cookies funktionieren automatisch.          |
| **Atomare Updates**      | Ein neues Docker-Image aktualisiert Frontend und Backend zusammen. Keine Versionskonflikte. |
| **Performance**          | Axum/Rust liefert statische Dateien extrem schnell aus.                                     |
| **Einfaches Deployment** | Ein Container statt drei. Weniger Orchestrierung nötig.                                     |
| **Ressourceneffizienz**  | Geringerer Memory-Footprint als separate Node.js-Container.                                 |

---

## Schritt 1: SvelteKit auf Static Adapter umstellen

Da Rust kein Node.js-Server ist, kann es kein Server-Side-Rendering (SSR) für SvelteKit ausführen. Die Apps werden als **Single Page Applications (SPA)** gebaut.

### 1.1 Adapter installieren

Für beide Frontends (`frontend/console` und `frontend/platform`):

```bash
# Im jeweiligen Frontend-Verzeichnis
pnpm add -D @sveltejs/adapter-static
```

### 1.2 Platform Frontend konfigurieren

Die Platform-App soll unter `/` (Root) laufen.

**`frontend/platform/svelte.config.js`**:

```javascript
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      pages: "build",
      assets: "build",
      fallback: "index.html", // WICHTIG für SPA Routing!
      precompress: false,
      strict: true,
    }),
  },
};

export default config;
```

### 1.3 Console Frontend konfigurieren

Die Console-App soll unter `/console` laufen. Der `base` path ist erforderlich.

**`frontend/console/svelte.config.js`**:

```javascript
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      pages: "build",
      assets: "build",
      fallback: "index.html", // WICHTIG für SPA Routing!
      precompress: false,
      strict: true,
    }),
    paths: {
      base: "/console", // Alle Links erhalten dieses Präfix
    },
  },
};

export default config;
```

### 1.4 SPA-Modus aktivieren

Erstelle/aktualisiere in **beiden** Frontends die Layout-Datei:

**`src/routes/+layout.ts`**:

```typescript
// Deaktiviert SSR und Prerendering für SPA-Modus
export const prerender = false;
export const ssr = false;
```

> **Hinweis**: Diese Datei ist `.ts`, nicht `.svelte`. Sie definiert Load-Optionen für alle Routes.

---

## Schritt 2: Rust Backend vorbereiten

### 2.1 Dependencies hinzufügen

**`backend/Cargo.toml`**:

```toml
[dependencies]
# ... bestehende Dependencies

# Für Static File Serving
tower-http = { version = "0.6", features = ["fs", "trace"] }
```

### 2.2 Frontend-Routes implementieren

Erstelle eine neue Datei oder erweitere das bestehende Routing:

**`backend/src/api/frontend.rs`**:

````rust
use axum::Router;
use std::path::PathBuf;
use tower_http::services::{ServeDir, ServeFile};

/// Integriert die Frontend-Assets in den API-Router.
///
/// # Routing-Priorität
/// 1. API Routes (`/api/v1/*`) - höchste Priorität
/// 2. Console Frontend (`/console/*`)
/// 3. Platform Frontend (`/*`) - Fallback/Catch-all
///
/// # Argumente
/// * `api_router` - Der Router mit allen API-Endpunkten
///
/// # Beispiel
/// ```rust
/// let api_router = Router::new()
///     .nest("/api/v1", api_v1_routes());
///
/// let app = merge_frontend_routes(api_router);
/// ```
pub fn merge_frontend_routes(api_router: Router) -> Router {
    // Pfade zu den Assets im Docker Container
    let platform_assets = PathBuf::from("./public/platform");
    let console_assets = PathBuf::from("./public/console");

    api_router
        // 1. Console Frontend (unter /console)
        .nest_service(
            "/console",
            ServeDir::new(&console_assets)
                .not_found_service(ServeFile::new(console_assets.join("index.html"))),
        )
        // 2. Platform Frontend (unter / - Root)
        // ACHTUNG: Das muss als letztes kommen, da es "catch-all" ist
        .fallback_service(
            ServeDir::new(&platform_assets)
                .not_found_service(ServeFile::new(platform_assets.join("index.html"))),
        )
}
````

### 2.3 In Server integrieren

**`backend/src/server.rs`** (Beispiel-Integration):

```rust
use crate::api::frontend::merge_frontend_routes;

pub async fn run_server() -> Result<(), Error> {
    // API Routes aufbauen
    let api_router = Router::new()
        .nest("/api/v1", api_v1_routes())
        .layer(/* middleware */);

    // Frontend-Routes hinzufügen (NUR in Production)
    let app = if cfg!(feature = "serve-frontend") {
        merge_frontend_routes(api_router)
    } else {
        api_router
    };

    // Server starten
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

### 2.4 Feature Flag (optional)

Um das Frontend-Serving nur in Production zu aktivieren:

**`backend/Cargo.toml`**:

```toml
[features]
default = []
serve-frontend = ["tower-http/fs"]
```

---

## Schritt 3: Unified Dockerfile

Multi-Stage Build für optimale Image-Größe.

**`infra/docker/Dockerfile.unified`**:

```dockerfile
# ===================================================
# Stage 1: Frontend Builder (Node.js)
# ===================================================
FROM node:20-alpine AS frontend-builder
WORKDIR /app

# pnpm aktivieren
RUN corepack enable && corepack prepare pnpm@latest --activate

# Dependency Caching: Nur package files zuerst
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY frontend/platform/package.json ./frontend/platform/
COPY frontend/console/package.json ./frontend/console/

# Dependencies installieren
RUN pnpm install --frozen-lockfile

# Source Code kopieren
COPY frontend ./frontend

# Build Platform
WORKDIR /app/frontend/platform
RUN pnpm build

# Build Console
WORKDIR /app/frontend/console
RUN pnpm build

# ===================================================
# Stage 2: Backend Builder (Rust)
# ===================================================
FROM rust:1.83-slim-bookworm AS backend-builder
WORKDIR /app

# System Dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Dependency Caching: Leeres Projekt für Cargo
COPY backend/Cargo.toml backend/Cargo.lock ./backend/
RUN mkdir -p backend/src && \
    echo "fn main() {}" > backend/src/main.rs && \
    echo "// dummy" > backend/src/lib.rs

# Proto files (falls benötigt)
COPY proto ./proto
COPY backend/proto ./backend/proto
COPY backend/build.rs ./backend/

# Dependencies pre-build
RUN cargo build --release --manifest-path backend/Cargo.toml || true

# Echten Code kopieren
COPY backend ./backend

# Rebuild erzwingen und finaler Build
RUN touch backend/src/main.rs && \
    cargo build --release --manifest-path backend/Cargo.toml \
    --features serve-frontend

# ===================================================
# Stage 3: Runtime Image (Minimal)
# ===================================================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Runtime Dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false erynoa

# Backend Binary
COPY --from=backend-builder /app/backend/target/release/erynoa-server ./server
COPY --from=backend-builder /app/backend/target/release/erynoa-worker ./worker

# Config
COPY backend/config ./config

# Frontend Assets
COPY --from=frontend-builder /app/frontend/platform/build ./public/platform
COPY --from=frontend-builder /app/frontend/console/build ./public/console

# Permissions
RUN chown -R erynoa:erynoa /app

# Non-root User
USER erynoa

# Environment
ENV ERYNOA_ENV=production
ENV HOST=0.0.0.0
ENV PORT=8080

EXPOSE 8080

# Health Check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

# Default: Server starten
CMD ["./server"]
```

---

## Schritt 4: Docker Compose Konfiguration

**`infra/docker/docker-compose.unified.yml`**:

```yaml
version: "3.8"

services:
  # ===========================================
  # Erynoa App (Backend + Frontends)
  # ===========================================
  erynoa-app:
    build:
      context: ../..
      dockerfile: infra/docker/Dockerfile.unified
    ports:
      - "8080:8080"
    environment:
      - ERYNOA_ENV=production
      - DATABASE_URL=postgres://erynoa:secret@postgres:5432/erynoa
      - REDIS_URL=redis://redis:6379
      - ZITADEL_URL=http://zitadel:8080
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/api/v1/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s
    restart: unless-stopped

  # ===========================================
  # Erynoa Worker (Background Jobs)
  # ===========================================
  erynoa-worker:
    build:
      context: ../..
      dockerfile: infra/docker/Dockerfile.unified
    command: ["./worker"]
    environment:
      - ERYNOA_ENV=production
      - DATABASE_URL=postgres://erynoa:secret@postgres:5432/erynoa
      - REDIS_URL=redis://redis:6379
      - TEMPORAL_URL=temporal:7233
    depends_on:
      - erynoa-app
      - temporal
    restart: unless-stopped

  # ===========================================
  # Infrastructure Services
  # ===========================================
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: erynoa
      POSTGRES_PASSWORD: secret
      POSTGRES_DB: erynoa
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U erynoa"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5

  temporal:
    image: temporalio/auto-setup:latest
    environment:
      - DB=postgresql
      - DB_PORT=5432
      - POSTGRES_USER=erynoa
      - POSTGRES_PWD=secret
      - POSTGRES_SEEDS=postgres
    depends_on:
      postgres:
        condition: service_healthy

volumes:
  postgres_data:
  redis_data:
```

---

## Schritt 5: Build & Deploy

### Lokales Testen

```bash
# Unified Image bauen
docker build -f infra/docker/Dockerfile.unified -t erynoa:unified .

# Lokal starten
docker run -p 8080:8080 erynoa:unified
```

### Mit Docker Compose

```bash
# Alles starten
docker compose -f infra/docker/docker-compose.unified.yml up --build

# Nur neu bauen
docker compose -f infra/docker/docker-compose.unified.yml build --no-cache
```

### Production Deployment

```bash
# Image taggen
docker tag erynoa:unified registry.example.com/erynoa:v1.0.0

# Pushen
docker push registry.example.com/erynoa:v1.0.0
```

---

## Routing-Übersicht

| Pfad         | Ziel         | Beschreibung              |
| ------------ | ------------ | ------------------------- |
| `/api/v1/*`  | Axum API     | REST/gRPC Endpunkte       |
| `/console/*` | Console SPA  | Admin-Oberfläche          |
| `/*`         | Platform SPA | Hauptanwendung (Fallback) |

### Routing-Priorität

```
Request: GET /api/v1/users
    → API Handler (Match!)

Request: GET /console/dashboard
    → ServeDir(/console) → public/console/dashboard/index.html
    → Falls nicht gefunden: public/console/index.html (SPA Fallback)

Request: GET /projects/123
    → Kein API Match
    → Kein /console Match
    → Fallback: public/platform/index.html (SPA übernimmt Routing)
```

---

## Troubleshooting

### Assets werden nicht gefunden (404)

1. Prüfe, ob die Build-Ausgabe im richtigen Verzeichnis liegt:

   ```bash
   docker run --rm erynoa:unified ls -la ./public/platform
   docker run --rm erynoa:unified ls -la ./public/console
   ```

2. Stelle sicher, dass `fallback: 'index.html'` in der SvelteKit-Config gesetzt ist.

### Console-Links funktionieren nicht

- Prüfe, ob `base: '/console'` in der `svelte.config.js` der Console gesetzt ist.
- Alle internen Links müssen das `$app/paths` Modul nutzen:

  ```svelte
  <script>
    import { base } from '$app/paths';
  </script>

  <a href="{base}/dashboard">Dashboard</a>
  ```

### API-Calls schlagen fehl

- Im SPA-Modus müssen API-Calls relativ sein (`/api/v1/...`), nicht absolut.
- Prüfe die Browser-Konsole auf CORS-Fehler (sollte es keine mehr geben).

### Docker Build schlägt fehl

```bash
# Cache leeren und neu bauen
docker build --no-cache -f infra/docker/Dockerfile.unified -t erynoa:unified .

# Nur einzelne Stage bauen (Debugging)
docker build --target frontend-builder -f infra/docker/Dockerfile.unified -t erynoa:frontend .
```

---

## Migration von separaten Containern

### Vorher (3 Container)

```yaml
services:
  backend:
    image: erynoa-backend
    ports: ["8080:8080"]

  platform:
    image: erynoa-platform
    ports: ["3000:3000"]

  console:
    image: erynoa-console
    ports: ["3001:3001"]
```

### Nachher (1 Container)

```yaml
services:
  erynoa-app:
    image: erynoa:unified
    ports: ["8080:8080"]
```

### Checkliste für Migration

- [ ] `@sveltejs/adapter-static` in beiden Frontends installiert
- [ ] `svelte.config.js` angepasst (mit `fallback: 'index.html'`)
- [ ] `+layout.ts` mit `ssr = false` erstellt
- [ ] Console: `base: '/console'` konfiguriert
- [ ] Backend: `tower-http` Dependency hinzugefügt
- [ ] Backend: Frontend-Routing implementiert
- [ ] `Dockerfile.unified` erstellt
- [ ] `docker-compose.unified.yml` aktualisiert
- [ ] Reverse Proxy (Caddy/Nginx) Konfiguration vereinfacht
- [ ] CI/CD Pipeline angepasst

---

## Weiterführende Ressourcen

- [SvelteKit Adapter Static Docs](https://kit.svelte.dev/docs/adapter-static)
- [Axum Tower-HTTP Docs](https://docs.rs/tower-http/latest/tower_http/services/struct.ServeDir.html)
- [Docker Multi-Stage Builds](https://docs.docker.com/build/building/multi-stage/)
