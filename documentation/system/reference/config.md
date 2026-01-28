# Erynoa – Service-Konfiguration

> **Dokumenttyp:** Referenz
> **Bereich:** Infrastruktur
> **Status:** Aktiv
> **Lesezeit:** ca. 8 Minuten

---

## Übersicht

Zentrale Referenz für **Service-Namen**, **Ports**, **URLs** und **Connection Strings**.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   🔧 KONFIGURATION                                                          │
│                                                                             │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │                                                                   │    │
│   │   📡 Services        🔗 Connections       📁 Config Files         │    │
│   │   ────────────       ─────────────        ─────────────           │    │
│   │   Namen & Ports      Dev & Docker         TOML & ENV              │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│   ⚠️ Änderungen hier → docker-compose.yml + backend/config/ anpassen       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 📡 Service-Übersicht

### Port-Matrix

| Service             | Extern | Intern | URL (extern)                     |
| :------------------ | :----: | :----: | :------------------------------- |
| **Proxy (Caddy)**   |  3001  |  3001  | `http://localhost:3001`          |
| **Backend**         |  3000  |  3000  | `http://localhost:3000`          |
| **Console**         |   –    |  5173  | `http://localhost:3001/console`  |
| **Platform**        |   –    |  5174  | `http://localhost:3001/platform` |
| **Docs**            |   –    |  5175  | `http://localhost:3001/docs`     |
| **PostgreSQL**      |  5432  |  5432  | `localhost:5432`                 |
| **DragonflyDB**     |  6379  |  6379  | `localhost:6379`                 |
| **MinIO (API)**     |  9000  |  9000  | `http://localhost:9000`          |
| **MinIO (Console)** |  9001  |  9001  | `http://localhost:9001`          |
| **ZITADEL**         |  8080  |  8080  | `http://localhost:8080`          |
| **ZITADEL DB**      |  5433  |  5432  | `localhost:5433`                 |

> 💡 **Hinweis:** Frontend-Services (Console, Platform, Docs) sind nur via Caddy Proxy erreichbar.

---

## 🌐 Frontend-Services

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   🔀 CADDY PROXY (:3001)                                                    │
│                                                                             │
│   localhost:3001/console  ──────────▶  Console (:5173)                     │
│   localhost:3001/platform ──────────▶  Platform (:5174)                    │
│   localhost:3001/docs     ──────────▶  Docs (:5175)                        │
│   localhost:3001/api      ──────────▶  Backend (:3000)                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

<details>
<summary><strong>📊 Console</strong></summary>

| Eigenschaft  | Wert                            |
| :----------- | :------------------------------ |
| Service Name | `console`                       |
| Direct Port  | `5173` (nur intern)             |
| Proxy URL    | `http://localhost:3001/console` |
| Container    | `erynoa-services-console-1`     |
| Framework    | SvelteKit                       |

</details>

<details>
<summary><strong>🖥️ Platform</strong></summary>

| Eigenschaft  | Wert                             |
| :----------- | :------------------------------- |
| Service Name | `platform`                       |
| Direct Port  | `5174` (nur intern)              |
| Proxy URL    | `http://localhost:3001/platform` |
| Container    | `erynoa-services-platform-1`     |
| Framework    | SvelteKit                        |

</details>

<details>
<summary><strong>📖 Docs</strong></summary>

| Eigenschaft  | Wert                         |
| :----------- | :--------------------------- |
| Service Name | `docs`                       |
| Direct Port  | `5175` (nur intern)          |
| Proxy URL    | `http://localhost:3001/docs` |
| Container    | `erynoa-services-docs-1`     |
| Framework    | SvelteKit                    |

</details>

<details>
<summary><strong>🔀 Caddy Proxy</strong></summary>

| Eigenschaft  | Wert                      |
| :----------- | :------------------------ |
| Service Name | `caddy`                   |
| Port         | `3001`                    |
| URL          | `http://localhost:3001`   |
| Container    | `erynoa-services-caddy-1` |
| Config       | `infra/proxy/Caddyfile`   |

</details>

---

## 🦀 Backend-Service

<details>
<summary><strong>🦀 Backend API</strong></summary>

| Eigenschaft  | Wert                        |
| :----------- | :-------------------------- |
| Service Name | `backend`                   |
| Port         | `3000`                      |
| URL          | `http://localhost:3000`     |
| Container    | `erynoa-services-backend-1` |
| Framework    | Rust/Axum                   |

</details>

### Config Files

```
backend/config/
├── base.toml         Basis-Konfiguration (alle Umgebungen)
├── local.toml        Lokale Entwicklung
└── production.toml   Produktions-Overrides
```

---

## 💾 Datenbank-Services

<details>
<summary><strong>🐘 PostgreSQL (Haupt-DB)</strong></summary>

| Eigenschaft  | Wert                   |
| :----------- | :--------------------- |
| Service Name | `db`                   |
| Port         | `5432`                 |
| Engine       | OrioleDB               |
| Container    | `erynoa-services-db-1` |

**Connection Strings:**

| Umgebung | String                                             |
| :------- | :------------------------------------------------- |
| Local    | `postgresql://erynoa:erynoa@localhost:5432/erynoa` |
| Docker   | `postgresql://erynoa:erynoa@db:5432/erynoa`        |

</details>

<details>
<summary><strong>🐘 ZITADEL DB</strong></summary>

| Eigenschaft  | Wert                           |
| :----------- | :----------------------------- |
| Service Name | `zitadel-db`                   |
| Extern Port  | `5433`                         |
| Intern Port  | `5432`                         |
| Container    | `erynoa-services-zitadel-db-1` |

**Connection Strings:**

| Umgebung | String                         |
| :------- | :----------------------------- |
| Local    | `postgresql://localhost:5433`  |
| Docker   | `postgresql://zitadel-db:5432` |

</details>

<details>
<summary><strong>⚡ DragonflyDB (Cache)</strong></summary>

| Eigenschaft  | Wert                      |
| :----------- | :------------------------ |
| Service Name | `cache`                   |
| Port         | `6379`                    |
| Protokoll    | Redis-kompatibel          |
| Container    | `erynoa-services-cache-1` |

**Connection Strings:**

| Umgebung | String                   |
| :------- | :----------------------- |
| Local    | `redis://localhost:6379` |
| Docker   | `redis://cache:6379`     |

</details>

---

## 📦 Storage & Auth

<details>
<summary><strong>📦 MinIO (Storage)</strong></summary>

| Eigenschaft  | Wert                      |
| :----------- | :------------------------ |
| Service Name | `minio`                   |
| API Port     | `9000`                    |
| Console Port | `9001`                    |
| Protokoll    | S3-kompatibel             |
| Container    | `erynoa-services-minio-1` |

**URLs:**

| Typ     | Umgebung | URL                     |
| :------ | :------- | :---------------------- |
| API     | Local    | `http://localhost:9000` |
| API     | Docker   | `http://minio:9000`     |
| Console | Local    | `http://localhost:9001` |

</details>

<details>
<summary><strong>🔐 ZITADEL (Auth)</strong></summary>

| Eigenschaft  | Wert                        |
| :----------- | :-------------------------- |
| Service Name | `zitadel`                   |
| Port         | `8080`                      |
| Protokoll    | OIDC/JWT                    |
| Container    | `erynoa-services-zitadel-1` |

**URLs:**

| Umgebung | URL                     |
| :------- | :---------------------- |
| Local    | `http://localhost:8080` |
| Docker   | `http://zitadel:8080`   |

</details>

---

## 🔗 Connection Strings

### Schnellreferenz

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   🔗 CONNECTION STRINGS                                                     │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   DEVELOPMENT (Local)               DOCKER (Internal)              │  │
│   │   ───────────────────               ─────────────────              │  │
│   │                                                                     │  │
│   │   postgresql://erynoa:erynoa        postgresql://erynoa:erynoa     │  │
│   │     @localhost:5432/erynoa            @db:5432/erynoa              │  │
│   │                                                                     │  │
│   │   redis://localhost:6379            redis://cache:6379             │  │
│   │                                                                     │  │
│   │   http://localhost:9000             http://minio:9000              │  │
│   │                                                                     │  │
│   │   http://localhost:8080             http://zitadel:8080            │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Kopiervorlage – Development

```bash
# Database
DATABASE_URL="postgresql://erynoa:erynoa@localhost:5432/erynoa"

# Cache
REDIS_URL="redis://localhost:6379"

# Storage
MINIO_ENDPOINT="http://localhost:9000"
MINIO_ACCESS_KEY="minioadmin"
MINIO_SECRET_KEY="minioadmin"

# Auth
ZITADEL_URL="http://localhost:8080"
```

### Kopiervorlage – Docker

```bash
# Database
DATABASE_URL="postgresql://erynoa:erynoa@db:5432/erynoa"

# Cache
REDIS_URL="redis://cache:6379"

# Storage
MINIO_ENDPOINT="http://minio:9000"

# Auth
ZITADEL_URL="http://zitadel:8080"
```

---

## 📁 Config-Dateien

| Datei                             | Zweck                          |
| :-------------------------------- | :----------------------------- |
| `backend/config/base.toml`        | Basis-Config (alle Umgebungen) |
| `backend/config/local.toml`       | Lokale Entwicklung             |
| `backend/config/production.toml`  | Produktion                     |
| `infra/proxy/Caddyfile`           | Reverse Proxy Routing          |
| `infra/docker/docker-compose.yml` | Service-Definitionen           |
| `.env`                            | Secrets (nicht committen!)     |

---

## ⚠️ Wichtige Hinweise

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ⚠️ KONSISTENZ-REGELN                                                      │
│                                                                             │
│   Bei Änderungen an Service-Namen oder Ports müssen folgende Dateien       │
│   synchron gehalten werden:                                                 │
│                                                                             │
│   1. infra/docker/docker-compose.yml    (Service-Definitionen)             │
│   2. backend/config/*.toml              (Backend-Konfiguration)            │
│   3. infra/proxy/Caddyfile              (Routing-Regeln)                   │
│   4. Diese Dokumentation                (config.md)                        │
│                                                                             │
│   ❌ Inkonsistente Konfigurationen führen zu Verbindungsfehlern!           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Checkliste bei Änderungen

- [ ] `docker-compose.yml` Service-Name aktualisiert
- [ ] `docker-compose.yml` Port-Mapping aktualisiert
- [ ] `backend/config/*.toml` Connection Strings aktualisiert
- [ ] `Caddyfile` Routing aktualisiert
- [ ] `config.md` Dokumentation aktualisiert

---

## 📚 Weiterführende Dokumente

| Dokument                              | Beschreibung            |
| :------------------------------------ | :---------------------- |
| [Architecture](architecture.md)       | System-Architektur      |
| [Connections](connections.md)         | API-Verbindungen        |
| [Docker Setup](../../setup/docker.md) | Container-Konfiguration |
| [Dev Setup](../../setup/dev_setup.md) | Entwicklungsumgebung    |

---

<div align="center">

```
┌───────────────────────────────────────┐
│                                       │
│   🔧 Config   →   🐳 Docker   →   🚀  │
│   TOML/ENV       Compose        Run   │
│                                       │
└───────────────────────────────────────┘
```

</div>
