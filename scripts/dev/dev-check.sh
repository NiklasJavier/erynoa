#!/bin/bash
# Development Environment Health Check
# Prüft ob alle Services erreichbar sind

# Kein set -e, damit alle Tests durchlaufen werden auch bei Fehlern

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results
PASSED=0
FAILED=0

# API Version (harmonized with frontend/console/src/lib/api-config.ts and backend/src/api/constants.rs)
# Backend läuft direkt, daher vollständiger Pfad
API_VERSION="/api/v1"

# Service URLs - Harmonized with frontend/console/src/lib/service-urls.ts and backend/src/config/constants.rs
# Proxy URLs für Frontends (single entry point)
PROXY_URL="${PROXY_URL:-http://localhost:3001}"
CONSOLE_URL="${CONSOLE_URL:-${PROXY_URL}/console}"
PLATFORM_URL="${PLATFORM_URL:-${PROXY_URL}/platform}"
DOCS_URL="${DOCS_URL:-${PROXY_URL}/docs}"
# Backend läuft direkt (nicht über Proxy)
API_URL="${API_URL:-http://localhost:3000}"
ZITADEL_URL="${ZITADEL_URL:-http://localhost:8080}"
MINIO_URL="${MINIO_URL:-http://localhost:9000}"

# Function to test endpoint with retries
test_service() {
    local name=$1
    local url=$2
    local expected_status=${3:-200}
    local max_retries=${4:-5}
    local retry_delay=${5:-2}
    
    echo -n "  Testing $name... "
    
    local attempt=0
    local http_code=""
    while [ $attempt -lt $max_retries ]; do
        # Use -o /dev/null to suppress body, -w to get only HTTP code
        # Use -f to fail on HTTP errors, but we check the code manually
        if response=$(curl -s -o /dev/null -w "%{http_code}" --max-time 5 "$url" 2>/dev/null); then
            # Trim whitespace from response
            http_code=$(echo "$response" | tr -d '[:space:]')
            # Prüfe ob HTTP Code dem erwarteten Status entspricht
            if [ "$http_code" = "$expected_status" ]; then
                echo -e "${GREEN}✓${NC}"
                ((PASSED++))
                return 0
            elif [ "$expected_status" = "any" ] && [ "$http_code" != "000" ] && [ -n "$http_code" ]; then
                # Für "any" akzeptiere jeden Code außer 000 (nicht erreichbar)
                echo -e "${GREEN}✓${NC}"
                ((PASSED++))
                return 0
            fi
        fi
        
        attempt=$((attempt + 1))
        if [ $attempt -lt $max_retries ]; then
            sleep $retry_delay
        fi
    done
    
    # Alle Versuche fehlgeschlagen
    if [ -n "$http_code" ] && [ "$http_code" != "000" ]; then
        echo -e "${RED}✗ (HTTP $http_code, erwartet: $expected_status)${NC}"
    else
        echo -e "${RED}✗ (nicht erreichbar nach ${max_retries} Versuchen)${NC}"
    fi
    ((FAILED++))
    return 1
}

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  🔍 Development Environment Health Check${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Test services mit Retries (Console/Platform/Docs brauchen Zeit zum Starten)
# Akzeptiere jeden HTTP-Code außer 000 (nicht erreichbar)
test_service "Console" "${CONSOLE_URL}" "any" 10 3
test_service "Platform" "${PLATFORM_URL}" "any" 10 3
test_service "Docs" "${DOCS_URL}" "any" 10 3

# Nur noch nicht-RPC-basierte Checks
test_service "ZITADEL OIDC" "${ZITADEL_URL}/.well-known/openid-configuration" "200" 5 2
test_service "MinIO Health" "${MINIO_URL}/minio/health/live" "200" 5 2

# Summary
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "  ${GREEN}✓ Passed: $PASSED${NC}"
if [ $FAILED -gt 0 ]; then
    echo -e "  ${RED}✗ Failed: $FAILED${NC}"
else
    echo -e "  ${GREEN}✗ Failed: $FAILED${NC}"
fi
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ $FAILED -eq 0 ]; then
    exit 0
else
    exit 1
fi
