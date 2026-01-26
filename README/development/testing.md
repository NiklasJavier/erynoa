# 🧪 Testing Guide

## Übersicht

Umfassender Guide für Tests im Erynoa-Projekt.

---

## Backend Tests

### Integration Tests

**Location:** `backend/tests/api.rs`

**Ausführung:**
```bash
cd backend && cargo test --test api
```

**Test-Kategorien:**
- Health Check Tests (2)
- Info Tests (2)
- User Tests (2)
- Storage Tests (3)
- Route Structure Tests (3)
- CORS Tests (1)

**Total:** 13 Integration Tests

### TestApp Helper

```rust
let app = TestApp::spawn().await;
let res = app.get("/api/v1/health").await;
assert!(res.status().is_success());
```

---

## Console Tests

**Status:** Vorbereitet für zukünftige Implementierung

**Empfohlene Struktur:**
```
console/tests/
├── e2e/
└── setup.ts
```

---

## Runtime Tests

**Script:** `infra/scripts/test/runtime-test.sh`

**Ausführung:**
```bash
./infra/scripts/test/runtime-test.sh
```

**Testet:**
- Public Endpoints
- Protected Endpoints (Auth)
- Route Structure
- CORS

---

## Weitere Informationen

- [Backend Test Suite](BACKEND_TEST_SUITE.md)
- [Test Verification](BACKEND_TEST_VERIFICATION.md)
- [Runtime Test Results](RUNTIME_TEST_RESULTS.md)
