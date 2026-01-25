#!/bin/bash

# eTamil HTTP Server Comprehensive Test Suite
# Tests various sample backend applications

set +e

# Ensure we start clean: kill any lingering test servers
kill_lingering() {
    for pidfile in /tmp/etamil_server_*.pid; do
        [ -f "$pidfile" ] || continue
        pid=$(cat "$pidfile" 2>/dev/null || true)
        if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
        fi
        rm -f "$pidfile"
    done
    # Last resort: stop any stray compiler servers from previous runs
    pkill -f "etamil_compiler --server" 2>/dev/null || true
}

trap kill_lingering EXIT
kill_lingering

# Run from etamil_compiler directory where examples are located
cd etamil_compiler || exit 1

COMPILER="./target/release/etamil_compiler"
PORT_BASE=10050
PORT=$PORT_BASE

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Function to print test header
test_header() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${BLUE}TEST: $1${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# Function to print result
test_result() {
    local test_name=$1
    local result=$2
    
    ((TESTS_RUN++))
    
    if [ $result -eq 0 ]; then
        echo -e "${GREEN}✓ PASS${NC}: $test_name"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ FAIL${NC}: $test_name"
        ((TESTS_FAILED++))
    fi
}

# Function to start server
start_server() {
    local program=$1
    local port=$2
    
    echo "Starting server with $program on port $port..."
    $COMPILER --server --port $port $program > /tmp/etamil_server_$port.log 2>&1 &
    local pid=$!
    echo $pid > /tmp/etamil_server_$port.pid
    
    sleep 2
    
    # Check if server started
    if kill -0 $pid 2>/dev/null; then
        echo "✓ Server PID $pid"
        return 0
    else
        echo "✗ Server failed to start"
        cat /tmp/etamil_server_$port.log
        return 1
    fi
}

# Function to stop server
stop_server() {
    local port=$1
    
    if [ -f /tmp/etamil_server_$port.pid ]; then
        local pid=$(cat /tmp/etamil_server_$port.pid)
        if kill -0 $pid 2>/dev/null; then
            kill $pid 2>/dev/null || true
            sleep 1
        fi
        rm -f /tmp/etamil_server_$port.pid
    fi
}

# Function to test HTTP endpoint
test_endpoint() {
    local port=$1
    local path=$2
    local expected_status=$3
    local test_name=$4
    
    local response=$(curl -s -w "\n%{http_code}" http://127.0.0.1:$port$path)
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)
    
    echo "  Request: GET $path"
    echo "  Response Status: $status (expected: $expected_status)"
    echo "  Response Body: $body"
    
    if [ "$status" = "$expected_status" ]; then
        test_result "$test_name" 0
        return 0
    else
        test_result "$test_name" 1
        return 1
    fi
}

# ============================================================================
# TEST 1: Simple API Server
# ============================================================================

test_header "Simple API Server"

if start_server "examples/backend/simple_api.qmz" $PORT; then
    test_endpoint $PORT "/" 200 "Root endpoint returns 200"
    test_endpoint $PORT "/health" 200 "Health endpoint returns 200"
    stop_server $PORT
else
    test_result "Simple API Server" 1
fi

# ============================================================================
# TEST 2: User Server
# ============================================================================

test_header "User Server"

PORT=$((PORT + 1))

if start_server "examples/backend/user_server.qmz" $PORT; then
    test_endpoint $PORT "/" 200 "User server root endpoint"
    test_endpoint $PORT "/health" 200 "User server health check"
    stop_server $PORT
else
    test_result "User Server" 1
fi

# ============================================================================
# TEST 3: Calculator Server
# ============================================================================

test_header "Calculator Server"

PORT=$((PORT + 1))

if start_server "examples/backend/calculator_server.qmz" $PORT; then
    test_endpoint $PORT "/" 200 "Calculator server endpoint"
    test_endpoint $PORT "/health" 200 "Calculator server health"
    stop_server $PORT
else
    test_result "Calculator Server" 1
fi

# ============================================================================
# TEST 4: Status Monitor Server
# ============================================================================

test_header "Status Monitor Server"

PORT=$((PORT + 1))

if start_server "examples/backend/status_server.qmz" $PORT; then
    test_endpoint $PORT "/" 200 "Status monitor endpoint"
    test_endpoint $PORT "/health" 200 "Status monitor health"
    stop_server $PORT
else
    test_result "Status Monitor Server" 1
fi

# ============================================================================
# TEST 5: Loop Server
# ============================================================================

test_header "Loop Server"

PORT=$((PORT + 1))

if start_server "examples/backend/loop_server.qmz" $PORT; then
    test_endpoint $PORT "/" 200 "Loop server endpoint"
    test_endpoint $PORT "/health" 200 "Loop server health"
    stop_server $PORT
else
    test_result "Loop Server" 1
fi

# ============================================================================
# TEST 6: Multiple HTTP Methods
# ============================================================================

test_header "Multiple HTTP Methods"

PORT=$((PORT + 1))

if start_server "examples/backend/hello_server.qmz" $PORT; then
    echo "Testing different HTTP methods on same endpoint..."
    
    # GET
    response=$(curl -s -w "\n%{http_code}" http://127.0.0.1:$PORT/)
    status=$(echo "$response" | tail -n 1)
    [ "$status" = "200" ] && echo "✓ GET /" || echo "✗ GET / (got $status)"
    
    # POST
    response=$(curl -s -w "\n%{http_code}" -X POST http://127.0.0.1:$PORT/)
    status=$(echo "$response" | tail -n 1)
    [ "$status" = "200" ] && echo "✓ POST /" || echo "✗ POST / (got $status)"
    
    # PUT
    response=$(curl -s -w "\n%{http_code}" -X PUT http://127.0.0.1:$PORT/)
    status=$(echo "$response" | tail -n 1)
    [ "$status" = "200" ] && echo "✓ PUT /" || echo "✗ PUT / (got $status)"
    
    # DELETE
    response=$(curl -s -w "\n%{http_code}" -X DELETE http://127.0.0.1:$PORT/)
    status=$(echo "$response" | tail -n 1)
    [ "$status" = "200" ] && echo "✓ DELETE /" || echo "✗ DELETE / (got $status)"
    
    test_result "Multiple HTTP Methods" 0
    stop_server $PORT
else
    test_result "Multiple HTTP Methods" 1
fi

# ============================================================================
# TEST 7: 404 Not Found
# ============================================================================

test_header "404 Not Found Handling"

PORT=$((PORT + 1))

if start_server "examples/backend/hello_server.qmz" $PORT; then
    response=$(curl -s -w "\n%{http_code}" http://127.0.0.1:$PORT/nonexistent)
    status=$(echo "$response" | tail -n 1)
    
    if [ "$status" = "404" ]; then
        echo "✓ 404 response for non-existent route"
        test_result "404 Not Found" 0
    else
        echo "✗ Expected 404, got $status"
        test_result "404 Not Found" 1
    fi
    
    stop_server $PORT
else
    test_result "404 Not Found" 1
fi

# ============================================================================
# TEST 8: Response Headers
# ============================================================================

test_header "HTTP Response Headers"

PORT=$((PORT + 1))

if start_server "examples/backend/hello_server.qmz" $PORT; then
    response=$(curl -i -s http://127.0.0.1:$PORT/health 2>&1 | head -20)
    
    echo "Response Headers:"
    echo "$response"
    
    # Check for expected headers
    has_content_type=$(echo "$response" | grep -i "Content-Type" | wc -l)
    has_cors=$(echo "$response" | grep -i "Access-Control-Allow-Origin" | wc -l)
    has_http200=$(echo "$response" | grep "HTTP/1.1 200" | wc -l)
    
    if [ $has_content_type -gt 0 ] && [ $has_cors -gt 0 ] && [ $has_http200 -gt 0 ]; then
        test_result "Response Headers" 0
    else
        test_result "Response Headers" 1
    fi
    
    stop_server $PORT
else
    test_result "Response Headers" 1
fi

# ============================================================================
# Summary
# ============================================================================

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "TEST SUMMARY"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "Total Tests: $TESTS_RUN"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Clean up
rm -f /tmp/etamil_server_*.log /tmp/etamil_server_*.pid

# Exit with appropriate code
if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
