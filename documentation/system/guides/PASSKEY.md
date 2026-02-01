# Passkey-basierte DID-Erstellung in Erynoa

> **WebAuthn/FIDO2 Integration für selbstbestimmte digitale Identitäten**

## Übersicht

Erynoa unterstützt Passkeys (WebAuthn/FIDO2) als **optionale** Methode zur DID-Erstellung. Dies ermöglicht:

- **Biometrische Authentifizierung** (Touch ID, Face ID, Windows Hello)
- **Hardware Security Keys** (YubiKey, Titan Key)
- **Plattform-Authentifikatoren** (integriert in OS/Browser)

Der bestehende Ed25519/did:key-Flow bleibt vollständig funktionsfähig und unverändert.

## Architektur

```
┌─────────────────────────────────────────────────────────────────┐
│                        FRONTEND (SvelteKit)                      │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────┐ │
│  │  PasskeySetup   │    │  PasskeyLogin   │    │ PasskeyMgr  │ │
│  │   (Component)   │    │   (Component)   │    │ (Component) │ │
│  └────────┬────────┘    └────────┬────────┘    └──────┬──────┘ │
│           │                      │                     │        │
│           ▼                      ▼                     ▼        │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    passkeyStore (Svelte)                  │  │
│  │  - credentials[], activeCredential, supportStatus        │  │
│  └────────────────────────────┬─────────────────────────────┘  │
│                               │                                 │
│  ┌────────────────────────────▼─────────────────────────────┐  │
│  │                    PasskeyService                         │  │
│  │  - registerPasskey()    - authenticateWithPasskey()      │  │
│  │  - signWithPasskey()    - checkPasskeySupport()          │  │
│  └────────────────────────────┬─────────────────────────────┘  │
│                               │                                 │
│  ┌────────────────────────────▼─────────────────────────────┐  │
│  │              @simplewebauthn/browser                      │  │
│  │  - startRegistration()   - startAuthentication()         │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                │ HTTPS (JSON)
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                        BACKEND (Rust/Axum)                       │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    /api/v1/auth/*                         │  │
│  │  GET  /challenge        - Generiert 32-Byte Challenge    │  │
│  │  POST /passkey/register - Speichert Public Key           │  │
│  │  POST /passkey/verify   - Verifiziert Signatur           │  │
│  └────────────────────────────┬─────────────────────────────┘  │
│                               │                                 │
│  ┌────────────────────────────▼─────────────────────────────┐  │
│  │                    IdentityStore (Fjall KV)               │  │
│  │  - passkey_credentials: KvStore                          │  │
│  │  - passkey_did_index: KvStore                            │  │
│  │  - store_passkey_credential()                            │  │
│  │  - get_passkey_credential()                              │  │
│  │  - update_passkey_last_used()                            │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Kryptographie

### Algorithmen (Priorität)

| Priorität | COSE Alg | Name    | Beschreibung                     |
| --------- | -------- | ------- | -------------------------------- |
| 1         | -8       | Ed25519 | EdDSA mit Curve25519 (bevorzugt) |
| 2         | -7       | ES256   | ECDSA mit P-256 (Fallback)       |

> **Hinweis:** Ed25519 wird bevorzugt, da es mit dem bestehenden Erynoa DID-System kompatibel ist.

### DID-Format

```
did:erynoa:<namespace>:<unique-id>

Beispiel: did:erynoa:self:abc123def456
```

**Verfügbare Namespaces:**

- `self` - Persönliche Identität
- `guild` - Organisation/Gruppe
- `spirit` - KI-Agent
- `thing` - IoT-Gerät
- `vessel` - Container/Prozess
- `source` - Datenquelle
- `craft` - Handwerk/Fähigkeit
- `vault` - Tresor/Speicher
- `pact` - Vertrag/Vereinbarung
- `circle` - Vertrauenskreis

## Frontend-Integration

### Installation

```bash
pnpm add @simplewebauthn/browser @simplewebauthn/types
```

### Passkey registrieren

```typescript
import { passkeyStore } from "$lib/auth/passkey";

// Registrierung starten
const result = await passkeyStore.register({
  displayName: "Max Mustermann",
  namespace: "self",
});

if (result.success) {
  console.log("DID erstellt:", result.did);
  console.log("Credential ID:", result.credentialId);
}
```

### Mit Passkey authentifizieren

```typescript
// Alle gespeicherten Credentials laden
const credentials = passkeyStore.getStoredCredentials();

// Mit spezifischem Credential authentifizieren
const result = await passkeyStore.authenticate(credentials[0].credential_id);

if (result.success) {
  console.log("Authentifiziert als:", result.did);
}
```

### Daten signieren

```typescript
// Beliebige Daten signieren
const signature = await passkeyStore.sign(
  credentials[0].credential_id,
  "Zu signierende Nachricht",
);

if (signature) {
  console.log("Signatur:", signature);
}
```

### Store-Status abfragen

```typescript
import {
  isPasskeySupported,
  isPasskeyAuthenticated,
  activePasskeyDid,
  passkeyCredentials,
} from "$lib/auth/passkey";

// Reaktive Stores
$: supported = $isPasskeySupported;
$: authenticated = $isPasskeyAuthenticated;
$: currentDid = $activePasskeyDid;
$: allCredentials = $passkeyCredentials;
```

## Backend-Integration

### API Endpoints

#### GET /api/v1/auth/challenge

Generiert eine kryptographisch sichere Challenge.

**Response:**

```json
{
  "challenge": "base64url-encoded-32-bytes",
  "expires_at": 1706745600
}
```

#### POST /api/v1/auth/passkey/register

Registriert ein neues Passkey Credential.

**Request:**

```json
{
  "credential_id": "base64url-encoded",
  "public_key": "base64url-encoded-cose-key",
  "algorithm": -8,
  "did": "did:erynoa:self:abc123",
  "namespace": "self",
  "display_name": "Max Mustermann",
  "transports": ["internal", "hybrid"]
}
```

**Response:**

```json
{
  "success": true,
  "did": "did:erynoa:self:abc123"
}
```

#### POST /api/v1/auth/passkey/verify

Verifiziert eine Passkey-Signatur.

**Request:**

```json
{
  "credential_id": "base64url-encoded",
  "signature": "base64url-encoded",
  "authenticator_data": "base64url-encoded",
  "client_data_json": "base64url-encoded"
}
```

**Response:**

```json
{
  "success": true,
  "did": "did:erynoa:self:abc123"
}
```

### Rust Types

```rust
use crate::api::v1::auth::StoredPasskeyCredential;

// Credential speichern
state.storage.identities.store_passkey_credential(&credential)?;

// Credential abrufen
let cred = state.storage.identities.get_passkey_credential(&credential_id)?;

// Last-Used aktualisieren
state.storage.identities.update_passkey_last_used(&credential_id)?;
```

## UI-Komponenten

### PasskeySetup

Multi-Step Wizard für Passkey-Registrierung.

```svelte
<script>
import PasskeySetup from '$lib/components/passkey/PasskeySetup.svelte';

function handleSuccess(e) {
  console.log('DID:', e.detail.did);
}
</script>

<PasskeySetup
  onSuccess={handleSuccess}
  onError={(e) => console.error(e.detail.error)}
  onCancel={() => goto('/')}
/>
```

### PasskeyLogin

Credential-Auswahl für Authentifizierung.

```svelte
<script>
import PasskeyLogin from '$lib/components/passkey/PasskeyLogin.svelte';
</script>

<PasskeyLogin
  onSuccess={(e) => console.log('Eingeloggt:', e.detail.did)}
  onError={(e) => console.error(e.detail.error)}
/>
```

### PasskeyManager

Verwaltung mehrerer Passkey Credentials.

```svelte
<script>
import PasskeyManager from '$lib/components/passkey/PasskeyManager.svelte';
</script>

<PasskeyManager
  onCredentialDeleted={(e) => console.log('Gelöscht:', e.detail.credentialId)}
/>
```

## Onboarding Flow

Die Onboarding-Seite (`/onboarding`) führt Benutzer durch:

1. **Welcome** - Einführung in selbstbestimmte Identität
2. **Choose Method** - Passkey vs. Traditional
3. **Setup** - Passkey-Registrierung oder Ed25519-Generierung
4. **Complete** - Erfolgsbestätigung mit DID

```
/onboarding
    │
    ├─► Welcome Screen
    │       │
    │       ▼
    ├─► Method Selection
    │       │
    │       ├─► Passkey (empfohlen)
    │       │       │
    │       │       ▼
    │       │   PasskeySetup Component
    │       │       │
    │       │       ▼
    │       │   WebAuthn Registration
    │       │
    │       └─► Traditional
    │               │
    │               ▼
    │           Ed25519 Generation
    │
    └─► Complete (DID anzeigen)
            │
            ▼
        Dashboard (/)
```

## Sicherheit

### Client-seitig

- Private Keys verlassen **niemals** das Gerät
- Challenge-Response verhindert Replay-Attacken
- Authenticator Data enthält RP ID Hash zur Phishing-Prävention
- Credentials werden im LocalStorage verschlüsselt gespeichert

### Server-seitig

- Challenges sind 32 Bytes kryptographisch random
- Challenge-Gültigkeit: 5 Minuten
- Sign-Counter wird bei jeder Authentifizierung geprüft
- Ed25519 Signatur-Verifikation mit `ed25519-dalek`

### Empfehlungen

1. **HTTPS zwingend erforderlich** - WebAuthn funktioniert nicht über HTTP
2. **Origin-Validierung** aktiviert
3. **Rate Limiting** für Auth-Endpoints
4. **Audit Logging** für alle Authentifizierungsversuche

## Fehlerbehandlung

### Passkey-spezifische Errors

| Code                      | Beschreibung                           |
| ------------------------- | -------------------------------------- |
| `PASSKEY_NOT_SUPPORTED`   | Browser/OS unterstützt keine Passkeys  |
| `REGISTRATION_FAILED`     | WebAuthn Registration fehlgeschlagen   |
| `AUTHENTICATION_FAILED`   | WebAuthn Authentication fehlgeschlagen |
| `CREDENTIAL_NOT_FOUND`    | Credential ID nicht gefunden           |
| `ALGORITHM_NOT_SUPPORTED` | Kein unterstützter Algorithmus         |
| `SIGNATURE_INVALID`       | Signatur-Verifikation fehlgeschlagen   |

### Fallback-Strategie

Wenn Passkeys nicht unterstützt werden:

1. UI zeigt "Nicht verfügbar" Badge
2. Passkey-Option ist deaktiviert
3. Traditional Ed25519 bleibt verfügbar

```typescript
const support = await checkPasskeySupport();
if (!support.supported) {
  console.warn("Fallback zu Ed25519:", support.reason);
}
```

## Testing

### Frontend (Vitest)

```bash
cd frontend/console
pnpm test src/lib/auth/passkey
```

### Backend (Cargo)

```bash
cd backend
cargo test auth
```

### E2E (Playwright)

```bash
pnpm test:e2e --grep "passkey"
```

> **Hinweis:** Für automatisierte Tests kann die WebAuthn Virtual Authenticator API verwendet werden.

## Dateien

### Frontend

```
frontend/console/src/lib/
├── auth/
│   ├── passkey/
│   │   ├── index.ts          # Module exports
│   │   ├── types.ts          # TypeScript types & constants
│   │   ├── utils.ts          # Encoding & DID utilities
│   │   ├── service.ts        # Passkey service (WebAuthn)
│   │   └── store.ts          # Svelte store
│   └── index.ts              # Auth module exports
├── components/
│   └── passkey/
│       ├── index.ts          # Component exports
│       ├── PasskeySetup.svelte
│       ├── PasskeyLogin.svelte
│       └── PasskeyManager.svelte
└── routes/
    └── onboarding/
        └── +page.svelte      # Onboarding page
```

### Backend

```
backend/src/
├── api/
│   ├── v1/
│   │   ├── auth/
│   │   │   ├── mod.rs        # Module definition
│   │   │   ├── types.rs      # Request/Response types
│   │   │   └── handlers.rs   # Axum handlers
│   │   └── mod.rs            # V1 API exports
│   ├── routes.rs             # Router with /auth routes
│   └── mod.rs                # API module
└── local/
    └── identity_store.rs     # Passkey credential storage
```

## Abhängigkeiten

### Frontend

- `@simplewebauthn/browser` ^13.0.0
- `@simplewebauthn/types` ^13.0.0

### Backend

- `ed25519-dalek` 2.x (bereits vorhanden)
- `base64` 0.22 (hinzugefügt)
- `chrono` 0.4 (bereits vorhanden)
- `rand` 0.8 (bereits vorhanden)

## Changelog

### v0.1.0 (Initial)

- ✅ WebAuthn Passkey Support (Ed25519, ES256)
- ✅ Frontend Service & Store
- ✅ UI-Komponenten (Setup, Login, Manager)
- ✅ Backend Auth Endpoints
- ✅ Identity Store Integration
- ✅ Onboarding Page
- ✅ Dokumentation
