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

# API Version (harmonized with frontend/src/lib/api-config.ts and backend/src/api/constants.rs)
API_VERSION="/api/v1"

# Service URLs - Harmonized with frontend/src/lib/service-urls.ts and backend/src/config/constants.rs
FRONTEND_URL="${FRONTEND_URL:-http://localhost:5173}"
API_URL="${API_URL:-http://localhost:3000}"
ZITADEL_URL="${ZITADEL_URL:-http://localhost:8080}"
MINIO_URL="${MINIO_URL:-http://localhost:9000}"

# Function to test endpoint
test_service() {
    local name=$1
    local url=$2
    local expected_status=${3:-200}
    
    echo -n "  Testing $name... "
    
    if response=$(curl -s -w "\n%{http_code}" "$url" 2>/dev/null); then
        http_code=$(echo "$response" | tail -n1)
        # Prüfe ob HTTP Code dem erwarteten Status entspricht
        if [ "$http_code" = "$expected_status" ]; then
            echo -e "${GREEN}✓${NC}"
            ((PASSED++))
            return 0
        elif [ "$expected_status" = "any" ] && [ "$http_code" != "000" ]; then
            # Für "any" akzeptiere jeden Code außer 000 (nicht erreichbar)
            echo -e "${GREEN}✓${NC}"
            ((PASSED++))
            return 0
        else
            echo -e "${RED}✗ (HTTP $http_code, erwartet: $expected_status)${NC}"
            ((FAILED++))
            return 1
        fi
    else
        echo -e "${RED}✗ (nicht erreichbar)${NC}"
        ((FAILED++))
        return 1
    fi
}

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  🔍 Development Environment Health Check${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Test services
test_service "Frontend" "${FRONTEND_URL}" "200"
test_service "Backend Health" "${API_URL}${API_VERSION}/health" "200"
test_service "Backend Info" "${API_URL}${API_VERSION}/info" "200"
test_service "ZITADEL OIDC" "${ZITADEL_URL}/.well-known/openid-configuration" "200"
test_service "MinIO Health" "${MINIO_URL}/minio/health/live" "200"

# Test database (via backend)
echo -n "  Testing Database (via Backend)... "
if curl -sf ${API_URL}/api/v1/ready | grep -q "database.*connected" 2>/dev/null; then
    echo -e "${GREEN}✓${NC}"
    ((PASSED++))
else
    echo -e "${YELLOW}⚠ (nicht verfügbar)${NC}"
    ((FAILED++))
fi

# Test cache (via backend)
echo -n "  Testing Cache (via Backend)... "
if curl -sf ${API_URL}/api/v1/ready | grep -q "cache.*connected" 2>/dev/null; then
    echo -e "${GREEN}✓${NC}"
    ((PASSED++))
else
    echo -e "${YELLOW}⚠ (nicht verfügbar)${NC}"
    ((FAILED++))
fi

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
