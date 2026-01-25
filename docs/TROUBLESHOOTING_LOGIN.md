# 🔧 Troubleshooting: Login-Weiterleitung funktioniert nicht

## Problem

Beim Klick auf "Anmelden" im Frontend erfolgt keine Weiterleitung zu ZITADEL.

## Debugging-Schritte

### 1. Browser-Konsole prüfen (WICHTIG!)

Öffne die Browser-Konsole (F12) und prüfe:

**Erwartete Logs beim Klick auf "Anmelden":**
```
Login button clicked
Starting OIDC redirect... { authority: "...", client_id: "...", redirect_uri: "..." }
Redirect initiated successfully
```

**Falls Fehler:**
- Prüfe die roten Fehlermeldungen
- Kopiere die vollständige Fehlermeldung

### 2. ZITADEL erreichbar?

```bash
curl http://localhost:8080/.well-known/openid-configuration
```

**Sollte zurückgeben:**
- JSON mit OIDC-Konfiguration
- `authorization_endpoint` sollte vorhanden sein

**Falls nicht erreichbar:**
- ZITADEL-Container starten: `docker compose up -d zitadel`
- Prüfe Logs: `docker logs godstack-services-zitadel-1 --tail 50`

### 3. Auth-Konfiguration prüfen

```bash
curl http://localhost:3000/api/v1/info | jq '.auth_issuer, .auth_client_id'
```

**Sollte zurückgeben:**
- `auth_issuer`: z.B. `"http://localhost:8080"`
- `auth_client_id`: z.B. `"357041892409540616"`

### 4. Browser-Network-Tab prüfen

1. Browser-Konsole öffnen (F12)
2. Network-Tab öffnen
3. Auf "Anmelden" klicken
4. Prüfe:
   - Wird eine Anfrage an ZITADEL gesendet?
   - Gibt es CORS-Fehler?
   - Gibt es 404/500 Fehler?

### 5. Häufige Probleme

#### Problem: ZITADEL nicht erreichbar
**Symptom:** Keine Weiterleitung, Fehler in Konsole
**Lösung:**
```bash
docker compose up -d zitadel
docker logs godstack-services-zitadel-1 --tail 50
```

#### Problem: Falsche Client-ID
**Symptom:** Weiterleitung funktioniert, aber ZITADEL zeigt Fehler
**Lösung:**
- Prüfe `auth_client_id` in Backend-Config
- Prüfe ob Client-ID in ZITADEL existiert

#### Problem: CORS-Fehler
**Symptom:** Fehler in Browser-Konsole: "CORS policy"
**Lösung:**
- Prüfe Backend CORS-Konfiguration
- Prüfe ob Frontend-URL in CORS-Allowed-Origins

#### Problem: UserManager nicht initialisiert
**Symptom:** Log zeigt "UserManager not initialized!"
**Lösung:**
- Seite neu laden
- Prüfe ob Config korrekt geladen wird

## Debug-Logs aktiviert

Die Login-Funktion loggt jetzt:
- ✅ Button-Klick
- ✅ UserManager-Status
- ✅ OIDC-Konfiguration
- ✅ Redirect-Status
- ✅ Fehler-Details

## Nächste Schritte

1. **Browser-Konsole öffnen (F12)**
2. **Auf "Anmelden" klicken**
3. **Logs kopieren und teilen:**
   - Alle Logs die erscheinen
   - Rote Fehlermeldungen
   - Network-Tab Fehler (falls vorhanden)

## Port-Forwarding Status

Die Logs zeigen, dass Port-Forwarding für Port 5173 funktioniert:
```
Connection established on localhost:5173
```

Das bedeutet, das Frontend sollte erreichbar sein. Das Problem liegt wahrscheinlich bei:
- ZITADEL-Verbindung
- OIDC-Konfiguration
- Browser-Sicherheitsrichtlinien
