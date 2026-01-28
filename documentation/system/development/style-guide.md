# Erynoa – Style Guide

> **Dokumenttyp:** Referenz
> **Bereich:** Entwicklung
> **Status:** Aktiv
> **Lesezeit:** ca. 10 Minuten

---

## Übersicht

Code-Standards und Naming Conventions für **Backend** (Rust) und **Frontend** (TypeScript/Svelte).

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   📝 STYLE GUIDE                                                            │
│                                                                             │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │                                                                   │    │
│   │   🦀 Rust              📘 TypeScript           🎨 Svelte          │    │
│   │   ──────────           ─────────────           ───────────        │    │
│   │   snake_case           camelCase               PascalCase         │    │
│   │   rustfmt              Biome                   Biome              │    │
│   │   clippy               ESLint                  Svelte Check       │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│   💡 Konsistenz > Perfektion – Gleiche Patterns überall anwenden           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🎯 Naming Conventions

### Schnellreferenz

| Element               | Rust              | TypeScript            |
| :-------------------- | :---------------- | :-------------------- |
| **Funktionen**        | `snake_case`      | `camelCase`           |
| **Structs/Classes**   | `PascalCase`      | `PascalCase`          |
| **Interfaces/Traits** | `PascalCase`      | `PascalCase`          |
| **Enums**             | `PascalCase`      | `PascalCase`          |
| **Konstanten**        | `SCREAMING_SNAKE` | `SCREAMING_SNAKE`     |
| **Variablen**         | `snake_case`      | `camelCase`           |
| **Module**            | `snake_case`      | `kebab-case` (Ordner) |
| **Dateien**           | `snake_case.rs`   | `kebab-case.ts`       |
| **Komponenten**       | –                 | `PascalCase.svelte`   |

---

## 🦀 Backend (Rust)

### Naming

<details>
<summary><strong>Functions</strong></summary>

```rust
// ✅ Gut
fn create_user() { }
fn get_user_by_id(id: Uuid) { }
fn list_users() { }
fn delete_user(id: Uuid) { }

// ❌ Schlecht
fn CreateUser() { }    // PascalCase
fn getUser() { }       // camelCase
```

</details>

<details>
<summary><strong>Structs & Enums</strong></summary>

```rust
// ✅ Gut
struct UserResponse { }
struct ApiError { }
enum ServiceStatus { Running, Stopped }

// ❌ Schlecht
struct user_response { }  // snake_case
enum service_status { }   // snake_case
```

</details>

<details>
<summary><strong>Modules & Files</strong></summary>

```rust
// ✅ Gut
mod user_handler;
mod storage_client;
mod error_handler;

// Dateien
// user_handler.rs
// storage_client.rs

// ❌ Schlecht
mod UserHandler;     // PascalCase
mod userHandler;     // camelCase
```

</details>

<details>
<summary><strong>Constants</strong></summary>

```rust
// ✅ Gut
const API_VERSION: &str = "v1";
const MAX_RETRIES: u32 = 3;
const DEFAULT_TIMEOUT_MS: u64 = 5000;

// ❌ Schlecht
const apiVersion: &str = "v1";    // camelCase
const api_version: &str = "v1";   // snake_case
```

</details>

### CRUD Naming Pattern

| Operation       | Pattern             | Beispiel        |
| :-------------- | :------------------ | :-------------- |
| **Create**      | `create_{resource}` | `create_user()` |
| **Read (one)**  | `get_{resource}`    | `get_user()`    |
| **Read (many)** | `list_{resources}`  | `list_users()`  |
| **Update**      | `update_{resource}` | `update_user()` |
| **Delete**      | `delete_{resource}` | `delete_user()` |

### Dateistruktur

```
backend/src/api/v1/{feature}/
│
├── handler.rs      REST handlers
├── connect.rs      Connect-RPC handlers
├── models.rs       Request/Response types
├── routes.rs       Route definitions
└── mod.rs          Module exports
```

**Beispiel: Users**

```
backend/src/api/v1/users/
├── handler.rs      → list_users(), get_user(), create_user()
├── connect.rs      → list_users_handler(), get_user_handler()
├── models.rs       → UserResponse, ListUsersQuery, CreateUserRequest
├── routes.rs       → create_users_routes()
└── mod.rs          → pub use handler::*; pub use models::*;
```

### Error Handling

```rust
// ApiError enum verwenden
return Err(ApiError::NotFound("User not found".to_string()));

// Oder mit ResultExt trait
some_operation()
    .context_api("Failed to fetch user")?;

// Error types definieren
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error")]
    Internal(#[from] anyhow::Error),
}
```

### Dokumentation

````rust
/// Holt einen User anhand der ID.
///
/// Gibt `ApiError::NotFound` zurück wenn der User nicht existiert.
///
/// # Beispiel
///
/// ```rust
/// let user = get_user(user_id).await?;
/// println!("{}", user.name);
/// ```
pub async fn get_user(id: Uuid) -> Result<User, ApiError> {
    // ...
}
````

### Tooling

| Tool            | Zweck         | Befehl         |
| :-------------- | :------------ | :------------- |
| **rustfmt**     | Formatierung  | `cargo fmt`    |
| **clippy**      | Linting       | `cargo clippy` |
| **cargo check** | Type Checking | `cargo check`  |

---

## 📘 Frontend (TypeScript/Svelte)

### Naming

<details>
<summary><strong>Functions</strong></summary>

```typescript
// ✅ Gut
function createUser() {}
function getUserById(id: string) {}
function listUsers() {}

// ❌ Schlecht
function create_user() {} // snake_case
function CreateUser() {} // PascalCase
```

</details>

<details>
<summary><strong>Interfaces & Types</strong></summary>

```typescript
// ✅ Gut
interface UserResponse {}
type ApiError = {};
enum ServiceStatus {
  Running,
  Stopped,
}

// ❌ Schlecht
interface user_response {} // snake_case
type apiError = {}; // camelCase
```

</details>

<details>
<summary><strong>Variables & Constants</strong></summary>

```typescript
// ✅ Gut
const user = getUser();
const userList = listUsers();
const API_VERSION = "v1";
const MAX_RETRIES = 3;

// ❌ Schlecht
const User = getUser(); // PascalCase
const user_list = listUsers(); // snake_case
```

</details>

<details>
<summary><strong>Files & Folders</strong></summary>

```
// ✅ Gut
user-handler.ts
storage-client.ts
UserCard.svelte
user-profile/

// ❌ Schlecht
userHandler.ts       // camelCase
user_handler.ts      // snake_case
usercard.svelte      // lowercase
```

</details>

### CRUD Naming Pattern

| Operation       | Pattern            | Beispiel       |
| :-------------- | :----------------- | :------------- |
| **Create**      | `create{Resource}` | `createUser()` |
| **Read (one)**  | `get{Resource}`    | `getUser()`    |
| **Read (many)** | `list{Resources}`  | `listUsers()`  |
| **Update**      | `update{Resource}` | `updateUser()` |
| **Delete**      | `delete{Resource}` | `deleteUser()` |

### Dateistruktur

```
frontend/{app}/src/lib/
│
├── api/                   API Layer
│   └── {feature}/
│       ├── client.ts      Connect-RPC Client
│       ├── types.ts       Type Re-exports
│       └── index.ts       Public API
│
├── components/            UI Components
│   └── {Feature}/
│       ├── FeatureCard.svelte
│       └── FeatureList.svelte
│
└── stores/                State Management
    └── {feature}.svelte.ts
```

### Svelte 5 Runes

```svelte
<script lang="ts">
  // ✅ State mit $state
  let count = $state(0);
  let user = $state<User | null>(null);

  // ✅ Derived mit $derived
  let doubled = $derived(count * 2);
  let isLoggedIn = $derived(user !== null);

  // ✅ Effects mit $effect
  $effect(() => {
    console.log(`Count changed: ${count}`);
  });

  // ✅ Props mit $props
  let { name, onClick } = $props<{
    name: string;
    onClick: () => void;
  }>();
</script>
```

### Error Handling

```typescript
// ApiErrorResponse verwenden
try {
  const response = await client.getUser({ id });
  return response;
} catch (error) {
  if (isConnectError(error)) {
    if (error.code === Code.NotFound) {
      // Handle not found
      return null;
    }
  }
  throw error;
}
```

### Dokumentation

````typescript
/**
 * Holt einen User anhand der ID.
 *
 * @param id - Die User ID
 * @returns Das User Objekt oder null wenn nicht gefunden
 * @throws {ConnectError} Bei Netzwerk-Fehlern
 *
 * @example
 * ```typescript
 * const user = await getUser("abc-123");
 * if (user) {
 *   console.log(user.name);
 * }
 * ```
 */
export async function getUser(id: string): Promise<User | null> {
  // ...
}
````

### Tooling

| Tool             | Zweck         | Befehl             |
| :--------------- | :------------ | :----------------- |
| **Biome**        | Format + Lint | `pnpm biome check` |
| **svelte-check** | Type Checking | `pnpm check`       |
| **TypeScript**   | Compilation   | `pnpm build`       |

---

## ✅ Best Practices

### Type Safety

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   🔒 TYPE SAFETY                                                            │
│                                                                             │
│   ✅ DO                              ❌ DON'T                               │
│   ─────                              ────────                               │
│                                                                             │
│   • Protobuf Types verwenden         • `any` verwenden                      │
│   • Explizite Return Types           • Type Assertions ohne Grund          │
│   • Strict Mode aktiviert            • @ts-ignore ohne Kommentar           │
│   • Zod für Runtime Validation       • Unvalidierte externe Daten          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Code Organisation

| Prinzip                   | Beschreibung                 |
| :------------------------ | :--------------------------- |
| **Single Responsibility** | Eine Funktion = eine Aufgabe |
| **DRY**                   | Don't Repeat Yourself        |
| **KISS**                  | Keep It Simple, Stupid       |
| **Flat Hierarchy**        | Max. 2-3 Ebenen Nesting      |

### Performance

| Bereich      | Best Practice                       |
| :----------- | :---------------------------------- |
| **Backend**  | Connection Pooling, Lazy Loading    |
| **Frontend** | Lazy Components, Memoization        |
| **API**      | Pagination, Field Selection         |
| **Queries**  | Indexed Fields, Prepared Statements |

---

## 📋 Checkliste

### Vor dem Commit

- [ ] `cargo fmt` / `pnpm format` ausgeführt
- [ ] `cargo clippy` / `pnpm lint` ohne Fehler
- [ ] `cargo test` / `pnpm test` bestanden
- [ ] Keine `TODO` ohne Issue-Referenz
- [ ] Keine hardcodierten Secrets
- [ ] Dokumentation aktualisiert

### Code Review

- [ ] Naming Conventions eingehalten
- [ ] Error Handling vollständig
- [ ] Types korrekt verwendet
- [ ] Tests vorhanden
- [ ] Keine Magic Numbers/Strings

---

## 📚 Weiterführende Dokumente

| Dokument                                     | Beschreibung        |
| :------------------------------------------- | :------------------ |
| [Testing](testing.md)                        | Test-Strategien     |
| [Architecture](../reference/architecture.md) | System-Architektur  |
| [Essential Guide](../essential_guide.md)     | Entwickler-Referenz |

### Externe Ressourcen

| Ressource           | Link                                                                             |
| :------------------ | :------------------------------------------------------------------------------- |
| Rust API Guidelines | [rust-lang.github.io/api-guidelines](https://rust-lang.github.io/api-guidelines) |
| TypeScript Handbook | [typescriptlang.org/docs](https://www.typescriptlang.org/docs)                   |
| Svelte 5 Docs       | [svelte.dev/docs](https://svelte.dev/docs)                                       |

---

<div align="center">

```
┌─────────────────────────────────────────────┐
│                                             │
│   📝 Style   →   🔍 Review   →   ✅ Merge   │
│   Conventions    Consistent     Quality     │
│                                             │
└─────────────────────────────────────────────┘
```

</div>
