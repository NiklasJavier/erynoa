#!/bin/bash
# Comprehensive Test Suite
# Tests Backend API, Console API, and Integration

set -e

echo "════════════════════════════════════════════════════════════"
echo "  🧪 Comprehensive Test Suite"
echo "════════════════════════════════════════════════════════════"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test results
TESTS_PASSED=0
TESTS_FAILED=0

# Function to print test result
print_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ $2${NC}"
        ((TESTS_FAILED++))
    fi
}

# Function to run test
run_test() {
    local test_name=$1
    shift
    echo -n "  Testing $test_name... "
    if "$@" > /dev/null 2>&1; then
        print_result 0 "$test_name"
        return 0
    else
        print_result 1 "$test_name"
        return 1
    fi
}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  1. Backend Structure Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if cargo is available
if command -v cargo &> /dev/null; then
    echo ""
    echo "  Running Rust compilation check..."
    cd backend
    if cargo check --quiet 2>&1; then
        print_result 0 "Backend compiles"
    else
        print_result 1 "Backend compilation"
        echo "    Run 'cargo check' in backend/ for details"
    fi
    cd ..
    
    echo ""
    echo "  Running Rust tests..."
    cd backend
    if cargo test --quiet --lib 2>&1; then
        print_result 0 "Backend unit tests"
    else
        print_result 1 "Backend unit tests"
        echo "    Run 'cargo test' in backend/ for details"
    fi
    cd ..
else
    echo -e "${YELLOW}⚠️  Cargo nicht verfügbar - Backend-Tests übersprungen${NC}"
    echo "    Installiere Rust/Cargo oder verwende Docker/DevContainer"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  2. Backend API Structure Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check API structure
echo ""
run_test "API v1 structure exists" test -d backend/src/api/v1
run_test "Health feature exists" test -d backend/src/api/v1/health
run_test "Info feature exists" test -d backend/src/api/v1/info
run_test "Users feature exists" test -d backend/src/api/v1/users
run_test "Storage feature exists" test -d backend/src/api/v1/storage
run_test "Middleware exists" test -d backend/src/api/middleware
run_test "Shared utilities exist" test -d backend/src/api/shared

# Check route files
echo ""
run_test "Health routes exist" test -f backend/src/api/v1/health/routes.rs
run_test "Info routes exist" test -f backend/src/api/v1/info/routes.rs
run_test "Users routes exist" test -f backend/src/api/v1/users/routes.rs
run_test "Storage routes exist" test -f backend/src/api/v1/storage/routes.rs

# Check handler files
echo ""
run_test "Health handlers exist" test -f backend/src/api/v1/health/handler.rs
run_test "Info handlers exist" test -f backend/src/api/v1/info/handler.rs
run_test "Users handlers exist" test -f backend/src/api/v1/users/handler.rs
run_test "Storage handlers exist" test -f backend/src/api/v1/storage/handler.rs

# Check model files
echo ""
run_test "Health models exist" test -f backend/src/api/v1/health/models.rs
run_test "Info models exist" test -f backend/src/api/v1/info/models.rs
run_test "Users models exist" test -f backend/src/api/v1/users/models.rs
run_test "Storage models exist" test -f backend/src/api/v1/storage/models.rs

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  3. Control API Structure Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check Control API structure
echo ""
run_test "API index exists" test -f frontend/console/src/api/index.ts
run_test "Types directory exists" test -d frontend/console/src/api/types
run_test "REST client exists" test -d frontend/console/src/api/rest
run_test "Connect client exists" test -d frontend/console/src/api/connect
run_test "Storage client exists" test -d frontend/console/src/api/storage

# Check specific files
echo ""
run_test "Types index exists" test -f frontend/console/src/api/types/index.ts
run_test "REST client exists" test -f frontend/console/src/api/rest/client.ts
run_test "REST endpoints exist" test -f frontend/console/src/api/rest/endpoints.ts
run_test "Connect transport exists" test -f frontend/console/src/api/connect/transport.ts
run_test "Connect services exist" test -f frontend/console/src/api/connect/services.ts
run_test "Storage client exists" test -f frontend/console/src/api/storage/client.ts

# Check old files are removed
echo ""
run_test "Old client.ts removed" test ! -f frontend/console/src/api/client.ts
run_test "Old connect.ts removed" test ! -f frontend/console/src/api/connect.ts
run_test "Old storage.ts removed" test ! -f frontend/console/src/api/storage.ts

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  4. Import Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check imports in console files
echo ""
if grep -q 'from.*["\047]\.\.?/api["\047]' frontend/console/src/App.tsx 2>/dev/null; then
    print_result 0 "App.tsx imports from api"
else
    print_result 1 "App.tsx imports"
fi

if grep -q 'from.*["\047]\.\.?/api["\047]' frontend/console/src/pages/Home.tsx 2>/dev/null; then
    print_result 0 "Home.tsx imports from api"
else
    print_result 1 "Home.tsx imports"
fi

if grep -q 'from.*["\047]\.\.?/api["\047]' frontend/console/src/pages/Users.tsx 2>/dev/null; then
    print_result 0 "Users.tsx imports from api"
else
    print_result 1 "Users.tsx imports"
fi

if grep -q 'from.*["\047]\.\.?/api["\047]' frontend/console/src/pages/StoragePage.tsx 2>/dev/null; then
    print_result 0 "StoragePage.tsx imports from api"
else
    print_result 1 "StoragePage.tsx imports"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  5. Test Files Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check test files
echo ""
run_test "Backend test file exists" test -f backend/tests/api.rs
run_test "Test documentation exists" test -f docs/backend_test_suite.md

# Count tests in test file
if [ -f backend/tests/api.rs ]; then
    TEST_COUNT=$(grep -c "#\[tokio::test\]" backend/tests/api.rs 2>/dev/null || echo "0")
    echo "    Found $TEST_COUNT integration tests"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  6. Documentation Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check documentation
echo ""
run_test "API restructure docs exist" test -f docs/api_restructure_complete.md
run_test "Console API docs exist" test -f docs/console_api_restructure_complete.md
run_test "Test results docs exist" test -f docs/test_results.md
run_test "Backend test suite docs exist" test -f docs/backend_test_suite.md

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  📊 Test Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "  ${GREEN}✅ Passed: $TESTS_PASSED${NC}"
echo -e "  ${RED}❌ Failed: $TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  ✅ All Structure Tests Passed!${NC}"
    echo -e "${GREEN}════════════════════════════════════════════════════════════${NC}"
    exit 0
else
    echo -e "${RED}════════════════════════════════════════════════════════════${NC}"
    echo -e "${RED}  ⚠️  Some Tests Failed${NC}"
    echo -e "${RED}════════════════════════════════════════════════════════════${NC}"
    exit 1
fi
