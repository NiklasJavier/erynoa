# 🏗️ System-Architektur

## Übersicht

Architektur-Dokumentation für das Godstack-Projekt.

---

## Backend API-Struktur

### Feature-basierte Organisation

```
backend/src/api/
├── v1/                       # API Version 1
│   ├── health/               # Health Check
│   ├── info/                 # Info & Status
│   ├── users/                # User Management
│   └── storage/              # Storage Operations
├── middleware/               # Middleware Layer
│   ├── auth.rs
│   ├── cors.rs
│   ├── logging.rs
│   └── error_handler.rs
└── shared/                   # Shared Utilities
    └── pagination.rs
```

**Vorteile:**
- Klare Feature-Trennung
- Einfacheres Testing
- Bessere Skalierbarkeit
- API-Versionierung vorbereitet

---

## Frontend API-Struktur

### Client-Organisation

```
frontend/src/api/
├── index.ts                  # Hauptexport
├── types/                    # Shared Types
├── rest/                     # REST Client
├── connect/                  # Connect-RPC Client
└── storage/                  # Storage Client
```

**Vorteile:**
- Klare Trennung der Clients
- Zentrale Types ohne Duplikation
- Einfacheres Warten

---

## Weitere Informationen

- [API Restrukturierung](API_RESTRUCTURE_COMPLETE.md)
- [Frontend API Konsolidierung](FRONTEND_API_RESTRUCTURE_COMPLETE.md)
