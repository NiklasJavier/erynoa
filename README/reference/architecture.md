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

## Console API-Struktur

### Feature-basierte Organisation (Phase 2)

```
frontend/console/src/api/
├── health/                   # Health Service
│   ├── types.ts              # Protobuf types + helpers
│   └── index.ts              # Public API
├── info/                     # Info Service
│   ├── types.ts
│   └── index.ts
├── users/                    # User Service
│   ├── connect-client.ts     # Connect-RPC client
│   ├── types.ts              # Protobuf types + helpers
│   └── index.ts
├── storage/                   # Storage Service
│   ├── connect-client.ts
│   ├── types.ts
│   └── index.ts
├── connect/                  # Connect-RPC Transport
│   ├── transport.ts          # Transport configuration
│   └── services.ts           # Service clients
└── rest/                     # REST Client (deprecated)
    ├── client.ts
    └── endpoints.ts
```

**Vorteile:**
- Konsistente Struktur mit Backend
- Protobuf-Types als Single Source of Truth
- Klare Feature-Trennung
- Einfacheres Testing

---

## 🔌 Connect-RPC/gRPC-Web

Das Projekt verwendet Connect-RPC für die Console-Backend-Kommunikation:

- **Protobuf** für Type-Safe Serialisierung
- **gRPC-Web** für Browser-Kompatibilität
- **Feature-basierte** Service-Organisation
- **Automatische** Code-Generierung

Siehe [Connect-RPC Guide](CONNECT_RPC_GUIDE.md) für Details.

---

## 📚 Weitere Informationen

- [API Restrukturierung](../changelog/API_RESTRUCTURE_COMPLETE.md)
- [Console API Konsolidierung](../changelog/CONSOLE_API_RESTRUCTURE_COMPLETE.md)
- [Connect-RPC Guide](CONNECT_RPC_GUIDE.md)
- [Style Guide](STYLE_GUIDE.md)
- [Testing Guide](testing.md)
- [Harmonization Roadmap](HARMONIZATION_ROADMAP.md)
- [TODO Management](TODOS.md)
