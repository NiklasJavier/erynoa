# 🔧 Troubleshooting: ZITADEL Authentication

## Problem

Nach den API-Konfigurations-Änderungen funktioniert die ZITADEL-Authentifizierung im Frontend nicht mehr.

## Mögliche Ursachen

### 1. Config-Fetch schlägt fehl
- Die `fetchConfig()` Funktion kann die Config nicht vom Backend laden
- Prüfe Browser-Konsole für Fehler
- Prüfe ob Backend läuft: `curl http://localhost:3000/api/v1/info`

### 2. Auth-Initialisierung schlägt fehl
- Die Auth-Konfiguration wird nicht korrekt übergeben
- Prüfe Browser-Konsole für Auth-Fehler
- Prüfe ob `issuer` und `clientId` korrekt sind

### 3. API-URL Konfiguration
- Die zentrale API-URL-Konfiguration könnte falsch sein
- Prüfe `VITE_API_URL` Environment-Variable
- Prüfe `getApiBaseUrl()` Rückgabewert

## Debugging

### 1. Browser-Konsole prüfen
```javascript
// Sollte erscheinen:
"Fetching config from: http://localhost:3000/api/v1/info"
"Config loaded successfully: {...}"
"Initializing Auth with: { issuer: '...', clientId: '...' }"
```

### 2. Backend prüfen
```bash
curl http://localhost:3000/api/v1/info
# Sollte zurückgeben:
# {
#   "version": "0.1.0",
#   "environment": "local",
#   "auth_issuer": "http://localhost:8080",
#   "auth_client_id": "357041892409540616",
#   ...
# }
```

### 3. ZITADEL prüfen
```bash
curl http://localhost:8080/.well-known/openid-configuration
# Sollte ZITADEL OIDC Config zurückgeben
```

## Lösung

### Debug-Logs hinzugefügt
- `fetchConfig()` loggt jetzt die API-URL und die geladene Config
- `AuthProvider` loggt jetzt die Auth-Initialisierung

### Nächste Schritte
1. Browser-Konsole öffnen
2. Seite neu laden
3. Logs prüfen:
   - Wird die Config geladen?
   - Wird Auth initialisiert?
   - Gibt es Fehler?

## Falls Problem weiterhin besteht

1. **Prüfe Environment-Variablen:**
   ```bash
   echo $VITE_API_URL
   ```

2. **Prüfe Backend-Logs:**
   ```bash
   docker logs godstack-services-backend-1 --tail 50
   ```

3. **Prüfe ZITADEL-Logs:**
   ```bash
   docker logs godstack-services-zitadel-1 --tail 50
   ```

4. **Prüfe Browser-Network-Tab:**
   - Wird `/api/v1/info` erfolgreich aufgerufen?
   - Gibt es CORS-Fehler?
   - Gibt es andere HTTP-Fehler?
