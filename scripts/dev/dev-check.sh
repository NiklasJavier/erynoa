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

# Test services with retries (Frontend/Backend brauchen Zeit zum Starten)
test_service "Frontend" "${FRONTEND_URL}" "200" 10 3

# Test Backend via Connect-RPC (POST requests with JSON)
echo -n "  Testing Backend Health (Connect-RPC)... "
backend_health_attempt=0
backend_health_ok=false
while [ $backend_health_attempt -lt 10 ]; do
    # Connect-RPC endpoint: /api/v1/connect/godstack.v1.HealthService/Check
    # Expects POST with empty JSON body: {}
    if response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -H "Connect-Protocol-Version: 1" \
        -d '{}' \
        --max-time 5 \
        "${API_URL}${API_VERSION}/connect/godstack.v1.HealthService/Check" 2>/dev/null); then
        # Check if response contains "status" or "SERVING_STATUS" (successful Connect-RPC response)
        if echo "$response" | grep -qiE '"status"|"SERVING_STATUS"'; then
            echo -e "${GREEN}✓${NC}"
            ((PASSED++))
            backend_health_ok=true
            break
        fi
    fi
    backend_health_attempt=$((backend_health_attempt + 1))
    if [ $backend_health_attempt -lt 10 ]; then
        sleep 3
    fi
done

if [ "$backend_health_ok" != "true" ]; then
    echo -e "${RED}✗ (nicht erreichbar nach 10 Versuchen)${NC}"
    ((FAILED++))
fi

echo -n "  Testing Backend Info (Connect-RPC)... "
backend_info_attempt=0
backend_info_ok=false
while [ $backend_info_attempt -lt 10 ]; do
    # Connect-RPC endpoint: /api/v1/connect/godstack.v1.InfoService/GetInfo
    if response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -H "Connect-Protocol-Version: 1" \
        -d '{}' \
        --max-time 5 \
        "${API_URL}${API_VERSION}/connect/godstack.v1.InfoService/GetInfo" 2>/dev/null); then
        # Check if response contains "version" (successful Connect-RPC response)
        if echo "$response" | grep -qi '"version"'; then
            echo -e "${GREEN}✓${NC}"
            ((PASSED++))
            backend_info_ok=true
            break
        fi
    fi
    backend_info_attempt=$((backend_info_attempt + 1))
    if [ $backend_info_attempt -lt 10 ]; then
        sleep 3
    fi
done

if [ "$backend_info_ok" != "true" ]; then
    echo -e "${RED}✗ (nicht erreichbar nach 10 Versuchen)${NC}"
    ((FAILED++))
fi

test_service "ZITADEL OIDC" "${ZITADEL_URL}/.well-known/openid-configuration" "200" 5 2
test_service "MinIO Health" "${MINIO_URL}/minio/health/live" "200" 5 2

# Test database (via backend Connect-RPC Ready endpoint)
# Connect-RPC Ready gibt {"ready":true,"database":{"healthy":true,...}} zurück
echo -n "  Testing Database (via Backend Connect-RPC)... "
db_attempt=0
db_connected=false
while [ $db_attempt -lt 10 ]; do
    if ready_response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -H "Connect-Protocol-Version: 1" \
        -d '{}' \
        --max-time 5 \
        "${API_URL}${API_VERSION}/connect/godstack.v1.HealthService/Ready" 2>/dev/null); then
        # Prüfe Connect-RPC Format: "database":{"healthy":true}
        if echo "$ready_response" | grep -qiE '"database".*"healthy".*true' 2>/dev/null; then
            echo -e "${GREEN}✓${NC}"
            ((PASSED++))
            db_connected=true
            break
        fi
    fi
    db_attempt=$((db_attempt + 1))
    if [ $db_attempt -lt 10 ]; then
        sleep 2
    fi
done

if [ "$db_connected" != "true" ]; then
    echo -e "${YELLOW}⚠ (nicht verfügbar)${NC}"
    ((FAILED++))
fi

# Test cache (via backend Connect-RPC Ready endpoint)
echo -n "  Testing Cache (via Backend Connect-RPC)... "
cache_attempt=0
cache_connected=false
while [ $cache_attempt -lt 10 ]; do
    if ready_response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -H "Connect-Protocol-Version: 1" \
        -d '{}' \
        --max-time 5 \
        "${API_URL}${API_VERSION}/connect/godstack.v1.HealthService/Ready" 2>/dev/null); then
        # Prüfe Connect-RPC Format: "cache":{"healthy":true}
        if echo "$ready_response" | grep -qiE '"cache".*"healthy".*true' 2>/dev/null; then
            echo -e "${GREEN}✓${NC}"
            ((PASSED++))
            cache_connected=true
            break
        fi
    fi
    cache_attempt=$((cache_attempt + 1))
    if [ $cache_attempt -lt 10 ]; then
        sleep 2
    fi
done

if [ "$cache_connected" != "true" ]; then
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
