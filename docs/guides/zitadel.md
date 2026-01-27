# ZITADEL Setup Guide

**Letzte Aktualisierung**: 2026-01-27 (23:40)

## Schnellstart

**Automatisches Setup (Empfohlen):**

ZITADEL wird automatisch beim ersten Start konfiguriert:
```bash
just dev
```

Oder manuell ausführen:
```bash
just zitadel-setup
```

Dieses Skript konfiguriert automatisch:
- Projekt `erynoa`
- OIDC Applications für alle Frontends (console, platform, docs) mit **dynamischen Client-IDs**
- Test-User `testuser` / `Test123!`
- Aktualisiert `backend/config/local.toml` mit den generierten Client-IDs

**Wichtig:** Die Client-IDs werden dynamisch von ZITADEL generiert (z.B. `357454249719824388`) und automatisch in die Backend-Konfiguration geschrieben. Die Frontends holen sich die Client-IDs über den `/api/v1/info` Endpoint.

**Manuelles Setup:**

### 1. ZITADEL Console öffnen
```
http://localhost:8080/ui/console
```

### 2. Erstanmeldung
- **Benutzer:** `zitadel-admin@zitadel.localhost`
- **Passwort:** `Password1!`
- Neues Passwort festlegen (z.B. `Admin123!`)

### 3. Projekt erstellen
1. Projects → **New Project**
2. Name: `erynoa`
3. Speichern

### 4. API Application erstellen
1. Im Projekt → Applications → **New**
2. Type: **API**
3. Name: `erynoa-api`
4. Auth Method: **Basic** (einfacher für Tests)
5. **Client ID und Secret notieren!**

### 5. Test-Benutzer `admin` erstellen
1. Users → **New** → Human User
2. Ausfüllen:
   - **Username:** `admin`
   - **Email:** `admin@localhost`
   - **First Name:** `Admin`
   - **Last Name:** `User`
3. Nach Erstellung: Actions → **Set Password**
   - Password: `admin`
   - ✓ "User must change password on next sign in" **deaktivieren**

### 6. Rollen erstellen
1. Projekt `erynoa` → Roles → **New**
2. Erstelle:
   - Key: `admin`, Display: `Administrator`
   - Key: `user`, Display: `User`

### 7. Rolle zuweisen
1. Projekt → Authorizations → **New**
2. User: `admin`
3. Roles: `admin` auswählen
4. Speichern

### 8. Token testen

```bash
# Client Credentials für API erhalten (ersetze CLIENT_ID und CLIENT_SECRET)
TOKEN=$(curl -s -X POST "http://localhost:8080/oauth/v2/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer" \
  -d "scope=openid profile email" \
  -d "client_id=<CLIENT_ID>" \
  -d "client_secret=<CLIENT_SECRET>" \
  | jq -r '.access_token')

# Oder Resource Owner Password Grant (für Test-Benutzer):
TOKEN=$(curl -s -X POST "http://localhost:8080/oauth/v2/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password" \
  -d "username=admin" \
  -d "password=admin" \
  -d "scope=openid profile email urn:zitadel:iam:org:project:id:<PROJECT_ID>:aud" \
  -d "client_id=<CLIENT_ID>" \
  -d "client_secret=<CLIENT_SECRET>" \
  | jq -r '.access_token')

echo $TOKEN

# API aufrufen
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/api/v1/me | jq .
```

## Config aktualisieren

Nach dem Setup, aktualisiere `config/local.toml`:

```toml
[auth]
issuer = "http://localhost:8080"
audiences = ["<PROJECT_ID>", "<CLIENT_ID>"]
jwks_cache_duration = 300
```

Die Project ID findest du in der URL wenn du das Projekt öffnest.
