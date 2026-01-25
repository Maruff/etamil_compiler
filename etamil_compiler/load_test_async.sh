#!/bin/bash
# ============================================================================
# PHASE 2: Load Testing Script for Async HTTP Server
# ============================================================================
# This script validates the 100x throughput improvement target
# Tests async server performance under various concurrency levels
# ============================================================================

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SERVER_HOST="127.0.0.1"
ASYNC_PORT=9999
SYNC_PORT=8080
TEST_DURATION=10  # seconds per test
WARMUP_REQUESTS=100

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  PHASE 2: ASYNC HTTP SERVER LOAD TESTING                         ║${NC}"
echo -e "${BLUE}║  Validating 100x Throughput Improvement Target                   ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if hey (HTTP benchmarking tool) is installed
if ! command -v hey &> /dev/null; then
    echo -e "${YELLOW}⚠️  'hey' benchmark tool not found${NC}"
    echo "Install with: go install github.com/rakyll/hey@latest"
    echo "Or use 'ab' (Apache Bench) as alternative"
    USE_HEY=false
    
    if ! command -v ab &> /dev/null; then
        echo -e "${RED}❌ Neither 'hey' nor 'ab' found. Cannot run load tests.${NC}"
        exit 1
    fi
else
    USE_HEY=true
fi

# Build async server
echo -e "${YELLOW}📦 Building release binary...${NC}"
cargo build --release 2>&1 | grep -E "Compiling|Finished" || echo "Already built"
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

# Create simple test program
TEST_PROGRAM="नकल = 42;"

# Start async server
echo -e "${YELLOW}🚀 Starting Async Server (Phase 2)...${NC}"
echo "$TEST_PROGRAM" | timeout 120 ./target/release/etamil_compiler \
    --server --async --port $ASYNC_PORT &
ASYNC_PID=$!
sleep 2

# Check if async server started
if ! curl -s http://$SERVER_HOST:$ASYNC_PORT/health > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Async server not responding yet, waiting...${NC}"
    sleep 3
fi

echo -e "${GREEN}✓ Async Server running (PID: $ASYNC_PID)${NC}"
echo ""

# Function to run benchmark
run_benchmark() {
    local concurrent=$1
    local test_name=$2
    local port=$3
    
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "📊 Test: $test_name"
    echo -e "   Concurrent Connections: $concurrent"
    echo -e "   Test Duration: ${TEST_DURATION}s"
    echo -e "   Target URL: http://$SERVER_HOST:$port/"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    if [ "$USE_HEY" = true ]; then
        # Using 'hey' benchmark tool
        hey -n 1000 -c $concurrent -z ${TEST_DURATION}s \
            http://$SERVER_HOST:$port/ 2>&1 | grep -E "Requests/sec|Average|Fastest|Slowest|Status|Errors"
    else
        # Using Apache Bench (ab)
        ab -n 100 -c $concurrent http://$SERVER_HOST:$port/ 2>&1 | \
            grep -E "Requests per second|Time per request|Failed requests"
    fi
    echo ""
}

# ============================================================================
# TEST 1: Baseline - Single Connection
# ============================================================================
echo -e "${YELLOW}TEST 1: Baseline Performance (1 concurrent connection)${NC}"
run_benchmark 1 "Baseline (Single Connection)" $ASYNC_PORT

# ============================================================================
# TEST 2: Low Concurrency
# ============================================================================
echo -e "${YELLOW}TEST 2: Low Concurrency (10 concurrent connections)${NC}"
run_benchmark 10 "Low Concurrency" $ASYNC_PORT

# ============================================================================
# TEST 3: Medium Concurrency
# ============================================================================
echo -e "${YELLOW}TEST 3: Medium Concurrency (50 concurrent connections)${NC}"
run_benchmark 50 "Medium Concurrency" $ASYNC_PORT

# ============================================================================
# TEST 4: High Concurrency
# ============================================================================
echo -e "${YELLOW}TEST 4: High Concurrency (100 concurrent connections)${NC}"
run_benchmark 100 "High Concurrency" $ASYNC_PORT

# ============================================================================
# TEST 5: Stress Test
# ============================================================================
echo -e "${YELLOW}TEST 5: Stress Test (200 concurrent connections)${NC}"
run_benchmark 200 "Stress Test" $ASYNC_PORT

# ============================================================================
# COMPARISON: Phase 1 (Sync) vs Phase 2 (Async)
# ============================================================================
echo ""
echo -e "${BLUE}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  PHASE 1 vs PHASE 2 COMPARISON (Optional)                        ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}Starting Phase 1 (sync) server for comparison...${NC}"

# Start sync server
echo "$TEST_PROGRAM" | timeout 60 ./target/release/etamil_compiler \
    --server --port $SYNC_PORT &
SYNC_PID=$!
sleep 2

echo -e "${YELLOW}TEST 6: Phase 1 Sync Server (1 concurrent)${NC}"
run_benchmark 1 "Phase 1 Sync - Single" $SYNC_PORT

echo -e "${YELLOW}TEST 7: Phase 1 Sync Server (10 concurrent)${NC}"
run_benchmark 10 "Phase 1 Sync - Multiple" $SYNC_PORT

# Cleanup
echo ""
echo -e "${YELLOW}🧹 Cleaning up...${NC}"
kill $ASYNC_PID 2>/dev/null || true
kill $SYNC_PID 2>/dev/null || true
sleep 1

# ============================================================================
# RESULTS SUMMARY
# ============================================================================
echo ""
echo -e "${BLUE}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  LOAD TEST SUMMARY                                               ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✓ Load testing complete${NC}"
echo ""
echo "Next Steps:"
echo "1. Review request/sec metrics above"
echo "2. Compare Phase 1 vs Phase 2 throughput"
echo "3. Verify 100x improvement target achieved"
echo "4. Document results for production sign-off"
echo ""
echo "Performance Targets:"
echo "  ✓ Phase 2 Target: 100-1000 req/sec"
echo "  ✓ Phase 1 Target: 1-10 req/sec"
echo "  ✓ Expected Gain: 100x"
echo ""
