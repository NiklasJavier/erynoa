# 📝 Style Guide

## Übersicht

Dieses Dokument definiert die Naming Conventions und Code-Standards für das gesamte Projekt.

---

## 🎯 Naming Conventions

### Backend (Rust)

#### Functions
- **Format**: `snake_case`
- **Beispiele**: 
  - `create_user()`
  - `get_user_by_id()`
  - `list_users()`

#### Structs & Enums
- **Format**: `PascalCase`
- **Beispiele**:
  - `UserResponse`
  - `ApiError`
  - `ServiceStatus`

#### Modules
- **Format**: `snake_case`
- **Beispiele**:
  - `user_handler`
  - `storage_client`
  - `error_handler`

#### Constants
- **Format**: `SCREAMING_SNAKE_CASE`
- **Beispiele**:
  - `API_VERSION`
  - `MAX_RETRIES`
  - `DEFAULT_TIMEOUT`

#### File Names
- **Format**: `snake_case.rs`
- **Beispiele**:
  - `user_handler.rs`
  - `storage_client.rs`
  - `error_handler.rs`

---

### Console (TypeScript)

#### Functions
- **Format**: `camelCase`
- **Beispiele**:
  - `createUser()`
  - `getUserById()`
  - `listUsers()`

#### Classes & Interfaces
- **Format**: `PascalCase`
- **Beispiele**:
  - `UserResponse`
  - `ApiError`
  - `ServiceStatus`

#### Variables & Constants
- **Format**: `camelCase` (variables), `SCREAMING_SNAKE_CASE` (constants)
- **Beispiele**:
  - `const user = ...`
  - `const API_VERSION = ...`
  - `const MAX_RETRIES = ...`

#### File Names
- **Format**: `kebab-case.ts` oder `PascalCase.tsx` (Components)
- **Beispiele**:
  - `user-handler.ts`
  - `storage-client.ts`
  - `UserCard.tsx`

---

## 📁 File Organization

### Backend API Structure

```
backend/src/api/v1/{feature}/
├── handler.rs      # REST handlers
├── connect.rs      # Connect-RPC handlers
├── models.rs       # Request/Response types
├── routes.rs       # Route definitions
└── mod.rs          # Module exports
```

**Beispiel**:
```
backend/src/api/v1/users/
├── handler.rs      # REST: list_users, get_user
├── connect.rs     # Connect-RPC: list_users_handler, get_user_handler
├── models.rs      # UserResponse, ListUsersQuery
├── routes.rs      # create_users_routes()
└── mod.rs         # Public exports
```

### Console API Structure

```
frontend/console/src/api/{feature}/
├── connect-client.ts  # Connect-RPC client
├── types.ts          # Type definitions (re-export from proto)
└── index.ts          # Public API
```

**Beispiel**:
```
frontend/console/src/api/users/
├── connect-client.ts  # ConnectUsersClient
├── types.ts          # User types (from proto)
└── index.ts          # export { ConnectUsersClient, ... }
```

---

## 🔤 Code Patterns

### Error Handling

#### Backend
```rust
// Use ApiError enum
return Err(ApiError::NotFound("User not found".to_string()));

// Or use ResultExt trait
some_operation()
    .context_api("Failed to fetch user")?;
```

#### Console
```typescript
// Use ApiErrorResponse
try {
  const response = await client.getUser(id);
} catch (error) {
  if (isErrorCode(error, "NOT_FOUND")) {
    // Handle not found
  }
  throw error;
}
```

### Function Naming

#### Backend
- **Create**: `create_{resource}`
- **Read**: `get_{resource}` oder `list_{resources}`
- **Update**: `update_{resource}`
- **Delete**: `delete_{resource}`

#### Console
- **Create**: `create{Resource}`
- **Read**: `get{Resource}` oder `list{Resources}`
- **Update**: `update{Resource}`
- **Delete**: `delete{Resource}`

---

## 📝 Documentation

### Rust
```rust
/// Brief description
///
/// Detailed description if needed
/// 
/// # Examples
/// 
/// ```rust
/// let user = get_user(1)?;
/// ```
pub fn get_user(id: u64) -> Result<User> {
    // ...
}
```

### TypeScript
```typescript
/**
 * Brief description
 * 
 * Detailed description if needed
 * 
 * @param id - User ID
 * @returns User object
 * @throws {ApiErrorResponse} If user not found
 * 
 * @example
 * ```typescript
 * const user = await getUser(1);
 * ```
 */
export async function getUser(id: number): Promise<User> {
  // ...
}
```

---

## 🎨 Code Style

### Rust
- Use `rustfmt` for formatting
- Use `clippy` for linting
- Prefer `?` operator over `match` for error handling
- Use `anyhow` for internal errors, `thiserror` for API errors

### TypeScript
- Use `prettier` for formatting
- Use `eslint` for linting
- Prefer `async/await` over Promises
- Use type annotations for public APIs

---

## ✅ Best Practices

### 1. Type Safety
- Always use types, avoid `any`
- Use Protobuf-generated types when available
- Create helper types for complex structures

### 2. Error Handling
- Always handle errors explicitly
- Use structured error responses
- Log errors appropriately

### 3. Code Organization
- Keep functions small and focused
- Use modules to group related functionality
- Avoid deep nesting

### 4. Performance
- Use lazy loading where appropriate
- Avoid unnecessary re-renders (React/Solid)
- Use connection pooling (Backend)

---

## 📚 Weitere Ressourcen

- [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/README.html)
- [TypeScript Style Guide](https://github.com/basarat/typescript-book/blob/master/docs/styleguide/styleguide.md)
- [Project Architecture](architecture.md)
- [Harmonization Roadmap](HARMONIZATION_ROADMAP.md)
