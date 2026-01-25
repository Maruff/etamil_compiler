#!/bin/bash
# eTamil VM Executor Test Suite
# Tests the independent DSL execution

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ETAMIL="$SCRIPT_DIR/etamil"
COMPILER_DIR="$SCRIPT_DIR/etamil_compiler"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test counters
TOTAL=0
PASSED=0
FAILED=0

# Run a test
run_test() {
    local test_name="$1"
    local qmz_file="$2"
    local expected_output="$3"
    
    TOTAL=$((TOTAL + 1))
    
    echo -e "${BLUE}Test $TOTAL: $test_name${NC}"
    
    if [[ ! -f "$qmz_file" ]]; then
        echo -e "  ${RED}✗ SKIP (file not found)${NC}"
        return
    fi
    
    output=$("$ETAMIL" "$qmz_file" 2>&1 | grep -v "^warning:" | tail -5)
    
    if echo "$output" | grep -q "$expected_output"; then
        echo -e "  ${GREEN}✓ PASS${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}✗ FAIL${NC}"
        echo "    Expected: $expected_output"
        echo "    Got: $output"
        FAILED=$((FAILED + 1))
    fi
    echo ""
}

echo "================================================="
echo "   eTamil VM Executor Test Suite"
echo "================================================="
echo ""

echo -e "${YELLOW}=== File I/O Tests ===${NC}"
run_test "Simple File I/O" \
    "$COMPILER_DIR/examples/io_samples/simple_fileio.qmz" \
    "Execution completed"

echo -e "${YELLOW}=== Basic Execution Tests ===${NC}"
run_test "Basic Sample" \
    "$COMPILER_DIR/examples/basic_samples/example.qmz" \
    "Execution completed"

echo -e "${YELLOW}=== Database Tests ===${NC}"
run_test "Database Connectivity" \
    "$COMPILER_DIR/examples/db_samples/test_db_connectivity.qmz" \
    "Execution completed"

run_test "Student Management" \
    "$COMPILER_DIR/examples/db_samples/student_management.qmz" \
    "Execution completed"

run_test "Payroll System" \
    "$COMPILER_DIR/examples/db_samples/payroll_system.qmz" \
    "Execution completed"

echo -e "${YELLOW}=== Performance Tests ===${NC}"

# Time a simple operation
echo "Timing VM startup..."
time "$ETAMIL" "$COMPILER_DIR/examples/io_samples/simple_fileio.qmz" > /dev/null 2>&1

echo ""
echo "================================================="
echo "   Test Results"
echo "================================================="
echo -e "Total:  $TOTAL"
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"
echo ""

if [[ $FAILED -eq 0 ]]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
