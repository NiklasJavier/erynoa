#!/bin/bash
# Runtime Test Script
# Startet Backend und testet API-Endpoints

set -e

echo "════════════════════════════════════════════════════════════"
echo "  🚀 Runtime Test - Backend API"
echo "════════════════════════════════════════════════════════════"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BACKEND_PORT=${BACKEND_PORT:-3000}
BACKEND_URL="http://localhost:${BACKEND_PORT}"
MAX_WAIT=30
WAIT_INTERVAL=1

# Test results
TESTS_PASSED=0
TESTS_FAILED=0

# Function to print result
print_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ $2${NC}"
        ((TESTS_FAILED++))
    fi
}

# Function to test endpoint
test_endpoint() {
    local method=$1
    local endpoint=$2
    local expected_status=$3
    local description=$4
    
    echo -n "  Testing $description... "
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "\n%{http_code}" "${BACKEND_URL}${endpoint}" 2>/dev/null || echo -e "\n000")
    elif [ "$method" = "POST" ]; then
        response=$(curl -s -w "\n%{http_code}" -X POST "${BACKEND_URL}${endpoint}" 2>/dev/null || echo -e "\n000")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" "${BACKEND_URL}${endpoint}" 2>/dev/null || echo -e "\n000")
    fi
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "$expected_status" ]; then
        print_result 0 "$description (HTTP $http_code)"
        if [ -n "$body" ] && [ "$body" != "null" ]; then
            echo "    Response: $(echo "$body" | head -c 100)..."
        fi
        return 0
    else
        print_result 1 "$description (Expected $expected_status, got $http_code)"
        return 1
    fi
}

# Check if backend is running
check_backend() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo "  Checking if backend is running..."
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    
    echo -n "  Checking ${BACKEND_URL}/api/v1/health... "
    
    for i in $(seq 1 $MAX_WAIT); do
        if curl -sf "${BACKEND_URL}/api/v1/health" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Backend is running!${NC}"
            echo ""
            return 0
        fi
        echo -n "."
        sleep $WAIT_INTERVAL
    done
    
    echo ""
    echo -e "${RED}❌ Backend is not running${NC}"
    echo ""
    echo "  To start the backend, run:"
    echo "    just dev"
    echo "  or"
    echo "    cd backend && cargo run"
    echo ""
    return 1
}

# Run tests
run_tests() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo "  Running API Tests"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    
    # Public endpoints
    echo -e "${YELLOW}Public Endpoints:${NC}"
    echo ""
    
    test_endpoint "GET" "/api/v1/health" "200" "Health Check"
    test_endpoint "GET" "/api/v1/info" "200" "Info Endpoint"
    test_endpoint "GET" "/api/v1/status" "200" "Status Endpoint"
    
    # Readiness might fail if services aren't running
    echo ""
    echo -n "  Testing Readiness Check... "
    response=$(curl -s -w "\n%{http_code}" "${BACKEND_URL}/api/v1/ready" 2>/dev/null || echo -e "\n000")
    http_code=$(echo "$response" | tail -n1)
    if [ "$http_code" = "200" ] || [ "$http_code" = "503" ]; then
        print_result 0 "Readiness Check (HTTP $http_code)"
    else
        print_result 1 "Readiness Check (Expected 200/503, got $http_code)"
    fi
    
    echo ""
    echo -e "${YELLOW}Protected Endpoints (should require auth):${NC}"
    echo ""
    
    # Protected endpoints should return 401/403
    test_endpoint "GET" "/api/v1/users" "401" "Users List (requires auth)"
    test_endpoint "GET" "/api/v1/me" "401" "Current User (requires auth)"
    test_endpoint "GET" "/api/v1/storage/list" "401" "Storage List (requires auth)"
    
    echo ""
    echo -e "${YELLOW}Route Structure Tests:${NC}"
    echo ""
    
    # Non-existent route should return 404
    test_endpoint "GET" "/api/v1/nonexistent" "404" "Non-existent Route"
    
    echo ""
}

# Main
main() {
    # Check if backend is running
    if ! check_backend; then
        echo -e "${RED}════════════════════════════════════════════════════════════${NC}"
        echo -e "${RED}  ⚠️  Backend not running - Cannot run tests${NC}"
        echo -e "${RED}════════════════════════════════════════════════════════════${NC}"
        exit 1
    fi
    
    # Run tests
    run_tests
    
    # Summary
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo "  📊 Test Summary"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "  ${GREEN}✅ Passed: $TESTS_PASSED${NC}"
    echo -e "  ${RED}❌ Failed: $TESTS_FAILED${NC}"
    echo ""
    
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}════════════════════════════════════════════════════════════${NC}"
        echo -e "${GREEN}  ✅ All Runtime Tests Passed!${NC}"
        echo -e "${GREEN}════════════════════════════════════════════════════════════${NC}"
        exit 0
    else
        echo -e "${RED}════════════════════════════════════════════════════════════${NC}"
        echo -e "${RED}  ⚠️  Some Tests Failed${NC}"
        echo -e "${RED}════════════════════════════════════════════════════════════${NC}"
        exit 1
    fi
}

main
