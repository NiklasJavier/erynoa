#!/bin/bash
# ============================================================================
# ZITADEL Vollautomatisches Setup für Erynoa Development
# ============================================================================
# Erstellt automatisch:
# - Projekt "erynoa"
# - Console OIDC App (PKCE)
# - Test User
#
# Verwendung:
#   ./scripts/infra/setup-zitadel.sh
#
# ============================================================================
set -e

# Service URLs - Harmonized with frontend/console/src/lib/service-urls.ts and backend/src/config/constants.rs
# Proxy URLs für Frontends (single entry point)
PROXY_URL="${PROXY_URL:-http://localhost:3001}"
ZITADEL_URL="${ZITADEL_URL:-http://localhost:8080}"
# Normalisiere URLs: entferne trailing slashes
CONSOLE_URL="${CONSOLE_URL:-${PROXY_URL}/console}"
CONSOLE_URL="${CONSOLE_URL%/}"  # Entferne trailing slash
PLATFORM_URL="${PLATFORM_URL:-${PROXY_URL}/platform}"
PLATFORM_URL="${PLATFORM_URL%/}"  # Entferne trailing slash
DOCS_URL="${DOCS_URL:-${PROXY_URL}/docs}"
DOCS_URL="${DOCS_URL%/}"  # Entferne trailing slash
API_URL="${API_URL:-http://localhost:3001/api}"
# Workspace-Root ermitteln (funktioniert sowohl im DevContainer als auch auf dem Host)
# Im DevContainer: /workspace
# Auf dem Host: Aktuelles Verzeichnis (Projekt-Root)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="${WORKSPACE_DIR:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
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
# Hinweis: Wir prüfen nicht mehr früh, ob Setup abgeschlossen ist,
# damit Redirect-URIs immer aktualisiert werden können
# (z.B. wenn sich Proxy-URLs ändern)

echo ""
echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║          🔐 ZITADEL Auto-Setup für Erynoa                          ║"
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
# Hole PAT aus Docker Volume (automatisch von ZITADEL erstellt)
# ─────────────────────────────────────────────────────────────────────────────
log_info "Warte auf automatisch generierten PAT..."

# Versuche verschiedene Volume-Namen (erynoa vs godstack für Migration)
ACCESS_TOKEN=""
MAX_PAT_RETRIES=30
PAT_RETRY_COUNT=0

while [ $PAT_RETRY_COUNT -lt $MAX_PAT_RETRIES ]; do
    for volume in "erynoa-services_zitadel-machinekey" "erynoa_zitadel-machinekey" "godstack-services_zitadel-machinekey" "godstack_zitadel-machinekey"; do
        ACCESS_TOKEN=$(docker run --rm -v ${volume}:/machinekey busybox cat /machinekey/pat.txt 2>/dev/null | tr -d '\n\r ' || echo "")
        if [ -n "$ACCESS_TOKEN" ] && [ "$ACCESS_TOKEN" != "" ] && [ ${#ACCESS_TOKEN} -gt 20 ]; then
            log_info "PAT gefunden in Volume: $volume"
            break 2
        fi
    done
    
    if [ -z "$ACCESS_TOKEN" ] || [ "$ACCESS_TOKEN" == "" ]; then
        PAT_RETRY_COUNT=$((PAT_RETRY_COUNT + 1))
        if [ $PAT_RETRY_COUNT -lt $MAX_PAT_RETRIES ]; then
            printf "."
            sleep 2
        fi
    else
        break
    fi
done

echo ""

# Fallback: PAT aus Umgebungsvariable
if [ -z "$ACCESS_TOKEN" ] || [ "$ACCESS_TOKEN" == "" ]; then
    ACCESS_TOKEN="${ZITADEL_PAT:-}"
fi

if [ -z "$ACCESS_TOKEN" ] || [ "$ACCESS_TOKEN" == "" ]; then
    log_warn "Kein PAT in Docker Volume gefunden. Warte auf ZITADEL Initialisierung..."
    log_info "ZITADEL erstellt den PAT automatisch beim ersten Start."
    log_info "Bitte warten Sie, bis ZITADEL vollständig initialisiert ist."
    log_info ""
    log_info "Prüfe ZITADEL Status..."
    
    # Warte auf ZITADEL und prüfe ob Init abgeschlossen ist
    for i in {1..60}; do
        if curl -sf "${ZITADEL_URL}/debug/ready" > /dev/null 2>&1; then
            # Versuche nochmal PAT zu holen
            for volume in "erynoa-services_zitadel-machinekey" "erynoa_zitadel-machinekey" "godstack-services_zitadel-machinekey" "godstack_zitadel-machinekey"; do
                ACCESS_TOKEN=$(docker run --rm -v ${volume}:/machinekey busybox cat /machinekey/pat.txt 2>/dev/null | tr -d '\n\r ' || echo "")
                if [ -n "$ACCESS_TOKEN" ] && [ "$ACCESS_TOKEN" != "" ] && [ ${#ACCESS_TOKEN} -gt 20 ]; then
                    log_ok "PAT gefunden nach Wartezeit"
                    break 2
                fi
            done
        fi
        if [ $i -lt 60 ]; then
            printf "."
            sleep 2
        fi
    done
    echo ""
fi

if [ -z "$ACCESS_TOKEN" ] || [ "$ACCESS_TOKEN" == "" ]; then
    echo ""
    echo "❌ Kein PAT verfügbar nach Wartezeit!"
    echo ""
    echo "   Der PAT sollte automatisch von ZITADEL erstellt werden."
    echo "   Falls das Problem weiterhin besteht:"
    echo ""
    echo "   Option 1: ZITADEL neu initialisieren"
    echo "   just zitadel-reset"
    echo ""
    echo "   Option 2: PAT manuell setzen"
    echo "   ZITADEL_PAT='<token>' $0"
    echo ""
    exit 1
fi

# Teste PAT-Gültigkeit durch Versuch, Projekte zu listen (einfacherer Endpoint)
log_info "Teste PAT-Gültigkeit..."
TEST_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" -X POST "${ZITADEL_URL}/management/v1/projects/_search" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"queries":[]}' 2>/dev/null)

if [ "$TEST_RESPONSE" != "200" ] && [ "$TEST_RESPONSE" != "201" ]; then
    log_warn "PAT ist ungültig oder hat nicht die richtigen Berechtigungen! (HTTP $TEST_RESPONSE)"
    log_info "Der PAT benötigt 'PROJECT_OWNER_GLOBAL' Berechtigung."
    log_info ""
    log_info "Falls ZITADEL neu initialisiert werden muss:"
    log_info "  just zitadel-reset"
    log_info ""
    log_info "Oder erstellen Sie manuell einen PAT:"
    log_info "  1. Öffne: ${ZITADEL_URL}/ui/console/"
    log_info "  2. Login: zitadel-admin / Password1!"
    log_info "  3. Settings > Personal Access Tokens"
    log_info "  4. Erstelle PAT mit 'PROJECT_OWNER_GLOBAL'"
    log_info "  5. ZITADEL_PAT='<token>' $0"
    exit 1
fi

log_ok "PAT geladen und gültig"

# ─────────────────────────────────────────────────────────────────────────────
# Projekt erstellen
# ─────────────────────────────────────────────────────────────────────────────
log_step "Projekt erstellen"

RESPONSE=$(curl -sf -X POST "${ZITADEL_URL}/management/v1/projects/_search" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"queries":[{"nameQuery":{"name":"erynoa","method":"TEXT_QUERY_METHOD_EQUALS"}}]}' 2>/dev/null || echo '{}')

PROJECT_ID=$(echo "$RESPONSE" | jq -r '.result[0].id // empty' 2>/dev/null)

if [ -n "$PROJECT_ID" ] && [ "$PROJECT_ID" != "null" ]; then
    log_ok "Projekt existiert bereits (ID: ${PROJECT_ID})"
else
    HTTP_CODE=$(curl -s -o /tmp/zitadel_project.json -w "%{http_code}" -X POST "${ZITADEL_URL}/management/v1/projects" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{"name":"erynoa","projectRoleAssertion":true}' 2>&1)
    
    RESPONSE=$(cat /tmp/zitadel_project.json 2>/dev/null || echo '{}')
    rm -f /tmp/zitadel_project.json
    
    if [ "$HTTP_CODE" != "200" ] && [ "$HTTP_CODE" != "201" ]; then
        log_warn "Fehler beim Erstellen des Projekts. HTTP Code: $HTTP_CODE"
        log_warn "Response: $RESPONSE"
        exit 1
    fi
    
    PROJECT_ID=$(echo "$RESPONSE" | jq -r '.id // empty' 2>/dev/null)
    
    if [ -z "$PROJECT_ID" ] || [ "$PROJECT_ID" == "null" ]; then
        log_warn "Fehler beim Erstellen des Projekts. Keine Projekt-ID in Response gefunden."
        log_warn "HTTP Code: $HTTP_CODE"
        log_warn "Response: $RESPONSE"
        exit 1
    fi
    
    log_ok "Projekt erstellt (ID: ${PROJECT_ID})"
fi

# ─────────────────────────────────────────────────────────────────────────────
# Console App erstellen (PKCE)
# ─────────────────────────────────────────────────────────────────────────────
log_step "Console App erstellen"

RESPONSE=$(curl -sf -X POST "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/_search" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"queries":[{"nameQuery":{"name":"erynoa-console","method":"TEXT_QUERY_METHOD_EQUALS"}}]}' 2>/dev/null || echo '{}')

EXISTING_APP_ID=$(echo "$RESPONSE" | jq -r '.result[0].id // empty' 2>/dev/null)
CLIENT_ID=$(echo "$RESPONSE" | jq -r '.result[0].oidcConfig.clientId // empty' 2>/dev/null)

# Prüfe ob App wirklich existiert und gültig ist
if [ -n "$CLIENT_ID" ] && [ "$CLIENT_ID" != "null" ] && [ -n "$EXISTING_APP_ID" ]; then
    # Prüfe ob die App wirklich funktioniert, indem wir die App-Details abrufen
    APP_DETAILS=$(curl -sf -X GET "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/${EXISTING_APP_ID}" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" 2>/dev/null || echo '{}')
    
    APP_VALID=$(echo "$APP_DETAILS" | jq -r '.oidcConfig.clientId // empty' 2>/dev/null)
    
    if [ -n "$APP_VALID" ] && [ "$APP_VALID" == "$CLIENT_ID" ]; then
        # Prüfe ob Redirect-URIs aktualisiert werden müssen
        CURRENT_REDIRECT_URIS=$(echo "$APP_DETAILS" | jq -r '.oidcConfig.redirectUris[]?' 2>/dev/null | sort | tr '\n' ' ')
        EXPECTED_REDIRECT_URIS="${CONSOLE_URL}/callback ${CONSOLE_URL}"
        
        if echo "$CURRENT_REDIRECT_URIS" | grep -q "${CONSOLE_URL}/callback" && echo "$CURRENT_REDIRECT_URIS" | grep -q "${CONSOLE_URL}"; then
            log_ok "Console App existiert bereits mit korrekten Redirect-URIs (Client ID: ${CLIENT_ID})"
        else
            log_info "Console App existiert, aber Redirect-URIs müssen aktualisiert werden..."
            # Update Redirect-URIs
            RESPONSE=$(curl -sf -X PUT "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/${EXISTING_APP_ID}/oidc_config" \
                -H "Authorization: Bearer ${ACCESS_TOKEN}" \
                -H "Content-Type: application/json" \
                -d "{
                    \"redirectUris\":[\"${CONSOLE_URL}/callback\",\"${CONSOLE_URL}\"],
                    \"postLogoutRedirectUris\":[\"${CONSOLE_URL}\"]
                }" 2>/dev/null || echo '{}')
            
            if echo "$RESPONSE" | jq -e '.redirectUris' > /dev/null 2>&1; then
                log_ok "Console App Redirect-URIs aktualisiert (Client ID: ${CLIENT_ID})"
            else
                log_warn "Fehler beim Aktualisieren der Redirect-URIs. Response: $RESPONSE"
                log_info "Lösche alte App und erstelle neu..."
                curl -sf -X DELETE "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/${EXISTING_APP_ID}" \
                    -H "Authorization: Bearer ${ACCESS_TOKEN}" > /dev/null 2>&1 || true
                CLIENT_ID=""
            fi
        fi
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
    # Prüfe ob PROJECT_ID gesetzt ist
    if [ -z "$PROJECT_ID" ] || [ "$PROJECT_ID" == "null" ]; then
        log_warn "PROJECT_ID ist leer! Kann App nicht erstellen."
        exit 1
    fi
    
    # Erstelle App mit vollständiger Fehlerausgabe
    HTTP_CODE=$(curl -s -o /tmp/zitadel_response.json -w "%{http_code}" -X POST "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/oidc" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        -H "Content-Type: application/json" \
        -d "{
            \"name\":\"erynoa-console\",
            \"redirectUris\":[\"${CONSOLE_URL}/callback\",\"${CONSOLE_URL}\"],
            \"postLogoutRedirectUris\":[\"${CONSOLE_URL}\"],
            \"responseTypes\":[\"OIDC_RESPONSE_TYPE_CODE\"],
            \"grantTypes\":[\"OIDC_GRANT_TYPE_AUTHORIZATION_CODE\",\"OIDC_GRANT_TYPE_REFRESH_TOKEN\"],
            \"appType\":\"OIDC_APP_TYPE_USER_AGENT\",
            \"authMethodType\":\"OIDC_AUTH_METHOD_TYPE_NONE\",
            \"accessTokenType\":\"OIDC_TOKEN_TYPE_JWT\",
            \"accessTokenRoleAssertion\":true,
            \"idTokenRoleAssertion\":true,
            \"idTokenUserinfoAssertion\":true,
            \"devMode\":true
        }" 2>&1)
    
    RESPONSE=$(cat /tmp/zitadel_response.json 2>/dev/null || echo '{}')
    rm -f /tmp/zitadel_response.json
    
    if [ "$HTTP_CODE" != "200" ] && [ "$HTTP_CODE" != "201" ]; then
        log_warn "Fehler beim Erstellen der App. HTTP Code: $HTTP_CODE"
        log_warn "Response: $RESPONSE"
        exit 1
    fi
    
    CLIENT_ID=$(echo "$RESPONSE" | jq -r '.clientId // .oidcConfig.clientId // empty' 2>/dev/null)
    
    if [ -z "$CLIENT_ID" ] || [ "$CLIENT_ID" == "null" ]; then
        log_warn "Fehler beim Erstellen der App. Keine Client-ID in Response gefunden."
        log_warn "HTTP Code: $HTTP_CODE"
        log_warn "Response: $RESPONSE"
        exit 1
    fi
    
    log_ok "Console App erstellt (Client ID: ${CLIENT_ID})"
fi

CONSOLE_CLIENT_ID="$CLIENT_ID"

# ─────────────────────────────────────────────────────────────────────────────
# Platform App erstellen (PKCE)
# ─────────────────────────────────────────────────────────────────────────────
log_step "Platform App erstellen"

RESPONSE=$(curl -sf -X POST "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/_search" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"queries":[{"nameQuery":{"name":"erynoa-platform","method":"TEXT_QUERY_METHOD_EQUALS"}}]}' 2>/dev/null || echo '{}')

EXISTING_APP_ID=$(echo "$RESPONSE" | jq -r '.result[0].id // empty' 2>/dev/null)
CLIENT_ID=$(echo "$RESPONSE" | jq -r '.result[0].oidcConfig.clientId // empty' 2>/dev/null)

# Prüfe ob App wirklich existiert und gültig ist
if [ -n "$CLIENT_ID" ] && [ "$CLIENT_ID" != "null" ] && [ -n "$EXISTING_APP_ID" ]; then
    # Prüfe ob die App wirklich funktioniert, indem wir die App-Details abrufen
    APP_DETAILS=$(curl -sf -X GET "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/${EXISTING_APP_ID}" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" 2>/dev/null || echo '{}')
    
    APP_VALID=$(echo "$APP_DETAILS" | jq -r '.oidcConfig.clientId // empty' 2>/dev/null)
    
    if [ -n "$APP_VALID" ] && [ "$APP_VALID" == "$CLIENT_ID" ]; then
        # Prüfe ob Redirect-URIs aktualisiert werden müssen
        CURRENT_REDIRECT_URIS=$(echo "$APP_DETAILS" | jq -r '.oidcConfig.redirectUris[]?' 2>/dev/null | sort | tr '\n' ' ')
        EXPECTED_REDIRECT_URIS="${PLATFORM_URL}/callback ${PLATFORM_URL}"
        
        if echo "$CURRENT_REDIRECT_URIS" | grep -q "${PLATFORM_URL}/callback" && echo "$CURRENT_REDIRECT_URIS" | grep -q "${PLATFORM_URL}"; then
            log_ok "Platform App existiert bereits mit korrekten Redirect-URIs (Client ID: ${CLIENT_ID})"
        else
            log_info "Platform App existiert, aber Redirect-URIs müssen aktualisiert werden..."
            # Update Redirect-URIs
            RESPONSE=$(curl -sf -X PUT "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/${EXISTING_APP_ID}/oidc_config" \
                -H "Authorization: Bearer ${ACCESS_TOKEN}" \
                -H "Content-Type: application/json" \
                -d "{
                    \"redirectUris\":[\"${PLATFORM_URL}/callback\",\"${PLATFORM_URL}\"],
                    \"postLogoutRedirectUris\":[\"${PLATFORM_URL}\"]
                }" 2>/dev/null || echo '{}')
            
            if echo "$RESPONSE" | jq -e '.redirectUris' > /dev/null 2>&1; then
                log_ok "Platform App Redirect-URIs aktualisiert (Client ID: ${CLIENT_ID})"
            else
                log_warn "Fehler beim Aktualisieren der Redirect-URIs. Response: $RESPONSE"
                log_info "Lösche alte App und erstelle neu..."
                curl -sf -X DELETE "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/${EXISTING_APP_ID}" \
                    -H "Authorization: Bearer ${ACCESS_TOKEN}" > /dev/null 2>&1 || true
                CLIENT_ID=""
            fi
        fi
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
            \"name\":\"erynoa-platform\",
            \"redirectUris\":[\"${PLATFORM_URL}/callback\",\"${PLATFORM_URL}\"],
            \"postLogoutRedirectUris\":[\"${PLATFORM_URL}\"],
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
    
    log_ok "Platform App erstellt (Client ID: ${CLIENT_ID})"
fi

PLATFORM_CLIENT_ID="$CLIENT_ID"

# ─────────────────────────────────────────────────────────────────────────────
# Docs App erstellen (PKCE)
# ─────────────────────────────────────────────────────────────────────────────
log_step "Docs App erstellen"

RESPONSE=$(curl -sf -X POST "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/_search" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"queries":[{"nameQuery":{"name":"erynoa-docs","method":"TEXT_QUERY_METHOD_EQUALS"}}]}' 2>/dev/null || echo '{}')

EXISTING_APP_ID=$(echo "$RESPONSE" | jq -r '.result[0].id // empty' 2>/dev/null)
CLIENT_ID=$(echo "$RESPONSE" | jq -r '.result[0].oidcConfig.clientId // empty' 2>/dev/null)

# Prüfe ob App wirklich existiert und gültig ist
if [ -n "$CLIENT_ID" ] && [ "$CLIENT_ID" != "null" ] && [ -n "$EXISTING_APP_ID" ]; then
    # Prüfe ob die App wirklich funktioniert, indem wir die App-Details abrufen
    APP_DETAILS=$(curl -sf -X GET "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/${EXISTING_APP_ID}" \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" 2>/dev/null || echo '{}')
    
    APP_VALID=$(echo "$APP_DETAILS" | jq -r '.oidcConfig.clientId // empty' 2>/dev/null)
    
    if [ -n "$APP_VALID" ] && [ "$APP_VALID" == "$CLIENT_ID" ]; then
        # Prüfe ob Redirect-URIs aktualisiert werden müssen
        CURRENT_REDIRECT_URIS=$(echo "$APP_DETAILS" | jq -r '.oidcConfig.redirectUris[]?' 2>/dev/null | sort | tr '\n' ' ')
        EXPECTED_REDIRECT_URIS="${DOCS_URL}/callback ${DOCS_URL}"
        
        if echo "$CURRENT_REDIRECT_URIS" | grep -q "${DOCS_URL}/callback" && echo "$CURRENT_REDIRECT_URIS" | grep -q "${DOCS_URL}"; then
            log_ok "Docs App existiert bereits mit korrekten Redirect-URIs (Client ID: ${CLIENT_ID})"
        else
            log_info "Docs App existiert, aber Redirect-URIs müssen aktualisiert werden..."
            # Update Redirect-URIs
            RESPONSE=$(curl -sf -X PUT "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/${EXISTING_APP_ID}/oidc_config" \
                -H "Authorization: Bearer ${ACCESS_TOKEN}" \
                -H "Content-Type: application/json" \
                -d "{
                    \"redirectUris\":[\"${DOCS_URL}/callback\",\"${DOCS_URL}\"],
                    \"postLogoutRedirectUris\":[\"${DOCS_URL}\"]
                }" 2>/dev/null || echo '{}')
            
            if echo "$RESPONSE" | jq -e '.redirectUris' > /dev/null 2>&1; then
                log_ok "Docs App Redirect-URIs aktualisiert (Client ID: ${CLIENT_ID})"
            else
                log_warn "Fehler beim Aktualisieren der Redirect-URIs. Response: $RESPONSE"
                log_info "Lösche alte App und erstelle neu..."
                curl -sf -X DELETE "${ZITADEL_URL}/management/v1/projects/${PROJECT_ID}/apps/${EXISTING_APP_ID}" \
                    -H "Authorization: Bearer ${ACCESS_TOKEN}" > /dev/null 2>&1 || true
                CLIENT_ID=""
            fi
        fi
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
            \"name\":\"erynoa-docs\",
            \"redirectUris\":[\"${DOCS_URL}/callback\",\"${DOCS_URL}\"],
            \"postLogoutRedirectUris\":[\"${DOCS_URL}\"],
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
    
    log_ok "Docs App erstellt (Client ID: ${CLIENT_ID})"
fi

DOCS_CLIENT_ID="$CLIENT_ID"

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

# Erstelle oder überschreibe die local.toml mit den aktuellen Client-IDs
cat > "$BACKEND_CONFIG" << EOF
# Local Development - Auto-generated by ZITADEL Setup
# Letzte Aktualisierung: $(date)

[application]
environment = "local"

[auth]
issuer = "${ZITADEL_URL}"
console_client_id = "${CONSOLE_CLIENT_ID}"
platform_client_id = "${PLATFORM_CLIENT_ID}"
docs_client_id = "${DOCS_CLIENT_ID}"
EOF
log_ok "Backend config aktualisiert (${BACKEND_CONFIG})"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎉 ZITADEL Setup abgeschlossen!"
echo ""
echo "   Console:           ${ZITADEL_URL}/ui/console/"
echo "   Console Client:    ${CONSOLE_CLIENT_ID}"
echo "   Platform Client:   ${PLATFORM_CLIENT_ID}"
echo "   Docs Client:       ${DOCS_CLIENT_ID}"
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
