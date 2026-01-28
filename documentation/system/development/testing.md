# Erynoa – Testing Guide

> **Dokumenttyp:** Referenz
> **Bereich:** Entwicklung
> **Status:** Aktiv
> **Lesezeit:** ca. 8 Minuten

---

## Übersicht

Test-Strategien und Ausführung für **Backend** (Rust) und **Frontend** (TypeScript/Svelte).

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   🧪 TESTING PYRAMID                                                        │
│                                                                             │
│                          ┌─────────┐                                        │
│                         /   E2E    \           Wenige, langsam              │
│                        /─────────────\                                      │
│                       /  Integration  \        Moderat                      │
│                      /─────────────────\                                    │
│                     /      Unit         \      Viele, schnell               │
│                    └─────────────────────┘                                  │
│                                                                             │
│   💡 Mehr Unit Tests, weniger E2E – schnelles Feedback                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🦀 Backend Tests

### Test-Struktur

```
backend/
├── src/
│   └── **/*.rs          Unit Tests (inline #[cfg(test)])
└── tests/
    └── api.rs           Integration Tests
```

### Befehle

| Befehl                      | Beschreibung            |
| :-------------------------- | :---------------------- |
| `cargo test`                | Alle Tests ausführen    |
| `cargo test --test api`     | Nur Integration Tests   |
| `cargo nextest run`         | Schneller (~60% faster) |
| `cargo test -- --nocapture` | Mit Ausgabe             |

### Test-Kategorien

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   🧪 BACKEND TEST SUITE                                                     │
│                                                                             │
│   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│   │  Health (2)     │  │  Info (2)       │  │  Users (2)      │            │
│   │  ────────────   │  │  ────────────   │  │  ────────────   │            │
│   │  • GET /health  │  │  • GET /info    │  │  • GET /users   │            │
│   │  • Response OK  │  │  • Version      │  │  • CRUD Ops     │            │
│   └─────────────────┘  └─────────────────┘  └─────────────────┘            │
│                                                                             │
│   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│   │  Storage (3)    │  │  Routes (3)     │  │  CORS (1)       │            │
│   │  ────────────   │  │  ────────────   │  │  ────────────   │            │
│   │  • Upload       │  │  • Structure    │  │  • Headers      │            │
│   │  • Download     │  │  • Nesting      │  │  • Preflight    │            │
│   │  • Delete       │  │  • Versioning   │  │                 │            │
│   └─────────────────┘  └─────────────────┘  └─────────────────┘            │
│                                                                             │
│   Total: 13 Integration Tests                                              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### TestApp Helper

```rust
use crate::helpers::TestApp;

#[tokio::test]
async fn health_check_works() {
    // Spawn test application
    let app = TestApp::spawn().await;

    // Make request
    let response = app.get("/api/v1/health").await;

    // Assert
    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["status"], "healthy");
}
```

### Unit Test Beispiel

```rust
// In src/auth/jwt.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_token_decodes() {
        let token = create_test_token();
        let claims = decode_token(&token).unwrap();

        assert_eq!(claims.sub, "test-user");
    }

    #[test]
    fn expired_token_fails() {
        let token = create_expired_token();
        let result = decode_token(&token);

        assert!(matches!(result, Err(AuthError::TokenExpired)));
    }
}
```

### Integration Test Beispiel

```rust
// In tests/api.rs
#[tokio::test]
async fn create_user_returns_201() {
    let app = TestApp::spawn().await;

    let payload = json!({
        "email": "test@example.com",
        "name": "Test User"
    });

    let response = app
        .post("/api/v1/users")
        .json(&payload)
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let user: UserResponse = response.json().await.unwrap();
    assert_eq!(user.email, "test@example.com");
}
```

---

## 📘 Frontend Tests

### Test-Struktur

```
frontend/{app}/
├── src/
│   └── lib/
│       └── **/*.test.ts    Unit Tests
└── tests/
    └── e2e/
        └── *.spec.ts       E2E Tests (Playwright)
```

### Befehle

| Befehl            | Beschreibung           |
| :---------------- | :--------------------- |
| `pnpm test`       | Alle Tests             |
| `pnpm test:unit`  | Unit Tests (Vitest)    |
| `pnpm test:e2e`   | E2E Tests (Playwright) |
| `pnpm test:watch` | Watch Mode             |

### Unit Test Beispiel

```typescript
// In src/lib/utils/format.test.ts
import { describe, it, expect } from "vitest";
import { formatDate, formatCurrency } from "./format";

describe("formatDate", () => {
  it("formats ISO date to German locale", () => {
    const result = formatDate("2026-01-28");
    expect(result).toBe("28.01.2026");
  });

  it("returns empty string for null", () => {
    const result = formatDate(null);
    expect(result).toBe("");
  });
});

describe("formatCurrency", () => {
  it("formats number to EUR", () => {
    const result = formatCurrency(1234.56);
    expect(result).toBe("1.234,56 €");
  });
});
```

### Component Test Beispiel

```typescript
// In src/lib/components/UserCard.test.ts
import { render, screen } from "@testing-library/svelte";
import { describe, it, expect } from "vitest";
import UserCard from "./UserCard.svelte";

describe("UserCard", () => {
  it("renders user name", () => {
    render(UserCard, {
      props: {
        user: { name: "Max Mustermann", email: "max@example.com" },
      },
    });

    expect(screen.getByText("Max Mustermann")).toBeInTheDocument();
  });

  it("shows loading state", () => {
    render(UserCard, { props: { loading: true } });

    expect(screen.getByTestId("loading-spinner")).toBeInTheDocument();
  });
});
```

### E2E Test Beispiel

```typescript
// In tests/e2e/login.spec.ts
import { test, expect } from "@playwright/test";

test.describe("Login Flow", () => {
  test("successful login redirects to dashboard", async ({ page }) => {
    await page.goto("/login");

    await page.fill('[data-testid="email"]', "user@example.com");
    await page.fill('[data-testid="password"]', "password123");
    await page.click('[data-testid="submit"]');

    await expect(page).toHaveURL("/dashboard");
    await expect(page.getByText("Willkommen")).toBeVisible();
  });

  test("invalid credentials show error", async ({ page }) => {
    await page.goto("/login");

    await page.fill('[data-testid="email"]', "wrong@example.com");
    await page.fill('[data-testid="password"]', "wrongpassword");
    await page.click('[data-testid="submit"]');

    await expect(page.getByText("Ungültige Anmeldedaten")).toBeVisible();
  });
});
```

---

## 🔧 Runtime Tests

### Script

```bash
# Ausführen
./scripts/test/runtime-test.sh
```

### Was wird getestet?

| Kategorie               | Tests              |
| :---------------------- | :----------------- |
| **Public Endpoints**    | Health, Info       |
| **Protected Endpoints** | Auth-Validation    |
| **Route Structure**     | API Versioning     |
| **CORS**                | Headers, Preflight |

---

## 📋 Test Patterns

### AAA Pattern

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   AAA PATTERN                                                               │
│                                                                             │
│   1. ARRANGE    →    Setup: Daten, Mocks, State                            │
│   2. ACT        →    Action: Funktion aufrufen                             │
│   3. ASSERT     →    Verify: Ergebnis prüfen                               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

```rust
#[test]
fn test_user_creation() {
    // Arrange
    let input = CreateUserRequest {
        email: "test@example.com".to_string(),
        name: "Test".to_string(),
    };

    // Act
    let result = create_user(input);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap().email, "test@example.com");
}
```

### Test Isolation

| Prinzip             | Beschreibung                        |
| :------------------ | :---------------------------------- |
| **Unabhängig**      | Jeder Test steht für sich           |
| **Deterministisch** | Gleiche Eingabe = Gleiches Ergebnis |
| **Schnell**         | Keine externen Dependencies         |
| **Cleanup**         | State nach Test zurücksetzen        |

---

## ✅ Best Practices

### DO ✅

| Praxis              | Beschreibung                                 |
| :------------------ | :------------------------------------------- |
| Beschreibende Namen | `test_user_with_invalid_email_returns_error` |
| Edge Cases testen   | Null, Empty, Boundary Values                 |
| Error Paths testen  | Nicht nur Happy Path                         |
| Test Data Fixtures  | Wiederverwendbare Testdaten                  |

### DON'T ❌

| Anti-Pattern        | Problem                             |
| :------------------ | :---------------------------------- |
| Test-Interdependenz | Tests beeinflussen sich gegenseitig |
| Hardcoded Waits     | `sleep(1000)` statt proper async    |
| Zu viel Mocking     | Tests spiegeln nicht Realität       |
| Ignorierte Tests    | `#[ignore]` ohne Grund              |

---

## 🚀 CI/CD Integration

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all

  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
      - run: pnpm install
      - run: pnpm test
```

---

## 📊 Coverage

### Backend

```bash
# Mit cargo-tarpaulin
cargo tarpaulin --out Html
```

### Frontend

```bash
# Mit Vitest
pnpm test:coverage
```

### Ziele

| Metrik              | Ziel  |
| :------------------ | :---- |
| **Line Coverage**   | > 80% |
| **Branch Coverage** | > 70% |
| **Critical Paths**  | 100%  |

---

## 📚 Weiterführende Dokumente

| Dokument                                     | Beschreibung        |
| :------------------------------------------- | :------------------ |
| [Style Guide](style-guide.md)                | Code-Standards      |
| [Architecture](../reference/architecture.md) | System-Architektur  |
| [Essential Guide](../essential_guide.md)     | Entwickler-Referenz |

### Externe Ressourcen

| Ressource         | Link                                                                                               |
| :---------------- | :------------------------------------------------------------------------------------------------- |
| Rust Testing Book | [doc.rust-lang.org/book/ch11-00-testing.html](https://doc.rust-lang.org/book/ch11-00-testing.html) |
| Vitest Docs       | [vitest.dev](https://vitest.dev)                                                                   |
| Playwright Docs   | [playwright.dev](https://playwright.dev)                                                           |

---

<div align="center">

```
┌───────────────────────────────────────────────┐
│                                               │
│   🧪 Test   →   ✅ Pass   →   🚀 Deploy      │
│   Write        Verify        Confidence       │
│                                               │
└───────────────────────────────────────────────┘
```

</div>
