#!/bin/bash
# ============================================================================
# ZITADEL Vollautomatisches Setup für Godstack Development
# ============================================================================
# Erstellt automatisch:
# - Projekt "godstack"
# - Frontend OIDC App (PKCE)
# - Test User
#
# Verwendung:
#   ./scripts/setup/setup-zitadel.sh
#
# ============================================================================
set -e

# Service URLs - Harmonized with frontend/src/lib/service-urls.ts and backend/src/config/constants.rs
ZITADEL_URL="${ZITADEL_URL:-http://localhost:8080}"
FRONTEND_URL="${FRONTEND_URL:-http://localhost:5173}"
API_URL="${API_URL:-http://localhost:3000}"
WORKSPACE_DIR="${WORKSPACE_DIR:-/workspace}"
DATA_DIR="${WORKSPACE_DIR}/.data"
CLIENT_ID_FILE="${DATA_DIR}/zitadel-client-id"
SETUP_COMPLETE="${DATA_DIR}/zitadel-setup-complete"

# Stelle sicher, dass .data existiert
mkdir -p "$DATA_DIR"

# Farben
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
YELLOW='\033[0;33m'
NC='\033[0m'

log_info()  { echo -e "${BLUE}[ZITADEL]${NC} $1"; }
log_ok()    { echo -e "${GREEN}[ZITADEL]${NC} ✓ $1"; }
log_warn()  { echo -e "${YELLOW}[ZITADEL]${NC} ⚠ $1"; }
log_step()  { echo -e "\n${CYAN}━━━ $1 ━━━${NC}"; }

# ─────────────────────────────────────────────────────────────────────────────
# Prüfe ob bereits eingerichtet
# ─────────────────────────────────────────────────────────────────────────────
if [ -f "$SETUP_COMPLETE" ] && [ -f "$CLIENT_ID_FILE" ]; then
    client_id=$(cat "$CLIENT_ID_FILE" 2>/dev/null)
    if [ -n "$client_id" ] && curl -sf "${ZITADEL_URL}/debug/ready" > /dev/null 2>&1; then
        log_ok "Setup bereits abgeschlossen (Client ID: $client_id)"
        log_info "Test Login: testuser / Test123!"
        exit 0
    fi
fi

echo ""
echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║          🔐 ZITADEL Auto-Setup für Godstack                        ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo ""

# ─────────────────────────────────────────────────────────────────────────────
# Warte auf ZITADEL
# ─────────────────────────────────────────────────────────────────────────────
log_step "Warte auf ZITADEL"
for i in {1..60}; do
    if curl -sf "${ZITADEL_URL}/debug/ready" > /dev/null 2>&1; then
        log_ok "ZITADEL ist bereit"
        break
    fi
    if [ $i -eq 60 ]; then
        log_warn "ZITADEL nicht erreichbar nach 120 Sekunden"
        exit 1
    fi
    printf "."
    sleep 2
done

# ─────────────────────────────────────────────────────────────────────────────
# Hole PAT aus Docker Volume
# ─────────────────────────────────────────────────────────────────────────────
log_info "Hole PAT aus Docker Volume..."

# Versuche verschiedene Volume-Namen (godstack vs godstack-services)
ACCESS_TOKEN=""
for volume in "godstack_zitadel-machinekey" "godstack-services_zitadel-machinekey"; do
    ACCESS_TOKEN=$(docker run --rm -v ${volume}:/machinekey busybox cat /machinekey/pat.txt 2>/dev/null || echo "")
    if [ -n "$ACCESS_TOKEN" ]; then
        break
    fi
done

if [ -z "$ACCESS_TOKEN" ]; then
    ACCESS_TOKEN="${ZITADEL_PAT:-}"
fi

if [ -z "$ACCESS_TOKEN" ]; then
    echo ""
    echo "❌ Kein PAT verfügbar!"
    echo "   Erstelle manuell unter: ${ZITADEL_URL}/ui/console/"
    echo "   Login: zitadel-admin / Password1!"
    echo "   Dann: ZITADEL_PAT='<token>' $0"
    exit 1
fi

log_ok "PAT geladen"

# ─────────────────────────────────────────────────────────────────────────────
# Projekt erstellen
# ─────────────────────────────────────────────────────────────────────────────
log_step "Projekt erstellen"

RESPONSE=$(curl -sf -X POST "${ZITADEL_URL}/management/v1/projects/_search" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"queries":[{"nameQuery":{"name":"godstack","method":"TEXT_QUERY_METHOD_EQUALS"}}]}' 2>/dev/null || echo '{}')

PROJECT_ID=$(echo "$RESPONSE" | jq -r '.result[0].id // empty' 2>/dev/null)

if [ -n "$PROJECT_ID" ] && [ "$PROJECT_ID" != "null" ]; then
    log_ok "Projekt existiert bereits (ID: ${PROJECT_ID})"
else
    RESPONSE=$(curl -sf -X POST "${ZITADEL_URL}/management/v1/projects" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{"name":"godstack","projectRoleAssertion":true}' 2>/dev/null || echo '{}')
    
    PROJECT_ID=$(echo "$RESPONSE" | jq -r '.id // empty' 2>/dev/null)
    log_ok "Projekt erstellt (ID: ${PROJECT_ID})"
fi

# ─────────────────────────────────────────────────────────────────────────────
# Frontend App erstellen (PKCE)
# ─────────────────────────────────────────────────────────────────────────────
log_step "Frontend App erstellen"

RESPONSE=$(curl -sf -X POST "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/_search" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"queries":[{"nameQuery":{"name":"godstack-frontend","method":"TEXT_QUERY_METHOD_EQUALS"}}]}' 2>/dev/null || echo '{}')

EXISTING_APP_ID=$(echo "$RESPONSE" | jq -r '.result[0].id // empty' 2>/dev/null)
CLIENT_ID=$(echo "$RESPONSE" | jq -r '.result[0].oidcConfig.clientId // empty' 2>/dev/null)

# Prüfe ob App wirklich existiert und gültig ist
if [ -n "$CLIENT_ID" ] && [ "$CLIENT_ID" != "null" ] && [ -n "$EXISTING_APP_ID" ]; then
    # Prüfe ob die App wirklich funktioniert, indem wir die App-Details abrufen
    APP_DETAILS=$(curl -sf -X GET "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/${EXISTING_APP_ID}" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" 2>/dev/null || echo '{}')
    
    APP_VALID=$(echo "$APP_DETAILS" | jq -r '.oidcConfig.clientId // empty' 2>/dev/null)
    
    if [ -n "$APP_VALID" ] && [ "$APP_VALID" == "$CLIENT_ID" ]; then
        log_ok "Frontend App existiert bereits (Client ID: ${CLIENT_ID})"
    else
        log_warn "App existiert, aber ist ungültig. Erstelle neu..."
        # Lösche alte App
        curl -sf -X DELETE "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/${EXISTING_APP_ID}" \
            -H "Authorization: Bearer ${ACCESS_TOKEN}" > /dev/null 2>&1 || true
        CLIENT_ID=""
    fi
else
    log_info "App nicht gefunden, erstelle neu..."
    CLIENT_ID=""
fi

if [ -z "$CLIENT_ID" ] || [ "$CLIENT_ID" == "null" ]; then
    RESPONSE=$(curl -sf -X POST "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/oidc" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        -H "Content-Type: application/json" \
        -d "{
            \"name\":\"godstack-frontend\",
            \"redirectUris\":[\"${FRONTEND_URL}/callback\",\"http://localhost:5174/callback\",\"${API_URL}/callback\"],
            \"postLogoutRedirectUris\":[\"${FRONTEND_URL}\",\"http://localhost:5174\",\"${API_URL}\"],
            \"responseTypes\":[\"OIDC_RESPONSE_TYPE_CODE\"],
            \"grantTypes\":[\"OIDC_GRANT_TYPE_AUTHORIZATION_CODE\",\"OIDC_GRANT_TYPE_REFRESH_TOKEN\"],
            \"appType\":\"OIDC_APP_TYPE_USER_AGENT\",
            \"authMethodType\":\"OIDC_AUTH_METHOD_TYPE_NONE\",
            \"accessTokenType\":\"OIDC_TOKEN_TYPE_JWT\",
            \"accessTokenRoleAssertion\":true,
            \"idTokenRoleAssertion\":true,
            \"idTokenUserinfoAssertion\":true,
            \"devMode\":true
        }" 2>/dev/null || echo '{}')
    
    CLIENT_ID=$(echo "$RESPONSE" | jq -r '.clientId // empty' 2>/dev/null)
    
    if [ -z "$CLIENT_ID" ] || [ "$CLIENT_ID" == "null" ]; then
        log_warn "Fehler beim Erstellen der App. Response: $RESPONSE"
        exit 1
    fi
    
    log_ok "Frontend App erstellt (Client ID: ${CLIENT_ID})"
fi

# ─────────────────────────────────────────────────────────────────────────────
# Test User erstellen
# ─────────────────────────────────────────────────────────────────────────────
log_step "Test User erstellen"

RESPONSE=$(curl -sf -X POST "${ZITADEL_URL}/management/v1/users/_search" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"queries":[{"userNameQuery":{"userName":"testuser","method":"TEXT_QUERY_METHOD_EQUALS"}}]}' 2>/dev/null || echo '{}')

USER_ID=$(echo "$RESPONSE" | jq -r '.result[0].id // empty' 2>/dev/null)

if [ -n "$USER_ID" ] && [ "$USER_ID" != "null" ]; then
    log_ok "Test User existiert bereits"
else
    RESPONSE=$(curl -sf -X POST "${ZITADEL_URL}/management/v1/users/human/_import" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{
            "userName":"testuser",
            "profile":{"firstName":"Test","lastName":"User","displayName":"Test User"},
            "email":{"email":"test@localhost","isEmailVerified":true},
            "password":"Test123!",
            "passwordChangeRequired":false
        }' 2>/dev/null || echo '{}')
    
    log_ok "Test User erstellt"
fi

# ─────────────────────────────────────────────────────────────────────────────
# Speichere Konfiguration
# ─────────────────────────────────────────────────────────────────────────────
echo "$CLIENT_ID" > "$CLIENT_ID_FILE"
date > "$SETUP_COMPLETE"

# Update Backend Config - immer aktualisieren
BACKEND_CONFIG="${WORKSPACE_DIR}/backend/config/local.toml"
log_info "Aktualisiere Backend-Konfiguration..."

# Erstelle oder überschreibe die local.toml mit der aktuellen Client-ID
cat > "$BACKEND_CONFIG" << EOF
# Local Development - Auto-generated by ZITADEL Setup
# Letzte Aktualisierung: $(date)

[application]
environment = "local"

[auth]
issuer = "${ZITADEL_URL}"
frontend_client_id = "${CLIENT_ID}"
EOF
log_ok "Backend config aktualisiert (${BACKEND_CONFIG})"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎉 ZITADEL Setup abgeschlossen!"
echo ""
echo "   Console:           ${ZITADEL_URL}/ui/console/"
echo "   Frontend Client:   ${CLIENT_ID}"
echo ""
echo "   Test Login:"
echo "   User:              testuser"
echo "   Password:          Test123!"
echo ""
echo "   Admin Login:"
echo "   User:              zitadel-admin"
echo "   Password:          Password1!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
