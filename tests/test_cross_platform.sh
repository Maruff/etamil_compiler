#!/bin/bash
# eTamil Cross-Platform Compatibility Test (Linux/macOS)

echo "=========================================="
echo "eTamil Cross-Platform Compatibility Test"
echo "=========================================="
echo "Platform: $(uname -s) $(uname -m)"
echo "Date: $(date)"
echo ""

# Determine binary location
if [ -f "./etamil" ]; then
    ETAMIL_BIN="./etamil"
elif [ -f "./etamil_compiler/target/release/etamil_compiler" ]; then
    ETAMIL_BIN="./etamil_compiler/target/release/etamil_compiler"
else
    echo "❌ Error: eTamil compiler not found"
    echo "Please build first: cd etamil_compiler && cargo build --release"
    exit 1
fi

echo "Using: $ETAMIL_BIN"
echo ""

# Test counter
PASSED=0
FAILED=0

# Test 1: VM Executor
echo "[1/5] Testing VM Executor..."
echo 'அச்சு("Hello from eTamil!");' > test_temp.etamil
$ETAMIL_BIN --vm test_temp.etamil > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✓ VM Executor: PASS"
    ((PASSED++))
else
    echo "✗ VM Executor: FAIL"
    ((FAILED++))
fi
echo ""

# Test 2: Example file execution
echo "[2/5] Testing Example File..."
if [ -f "test_standalone.etamil" ]; then
    $ETAMIL_BIN --vm test_standalone.etamil > /dev/null 2>&1
    if [ $? -eq 0 ]; then
        echo "✓ Example File Execution: PASS"
        ((PASSED++))
    else
        echo "✗ Example File Execution: FAIL"
        ((FAILED++))
    fi
else
    echo "ℹ Example File: SKIPPED (file not found)"
fi
echo ""

# Test 3: HTTP Sync Server
echo "[3/5] Testing HTTP Sync Server..."
timeout 3s $ETAMIL_BIN --server --port 8765 test_temp.etamil > /dev/null 2>&1 &
SERVER_PID=$!
sleep 2
if command -v curl > /dev/null 2>&1; then
    curl -s http://localhost:8765/ > /dev/null 2>&1
    CURL_RESULT=$?
else
    echo "ℹ curl not available, checking process only"
    ps -p $SERVER_PID > /dev/null 2>&1
    CURL_RESULT=$?
fi

if [ $CURL_RESULT -eq 0 ]; then
    echo "✓ HTTP Sync Server: PASS"
    ((PASSED++))
else
    echo "✗ HTTP Sync Server: FAIL"
    ((FAILED++))
fi
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null
echo ""

# Test 4: HTTP Async Server
echo "[4/5] Testing HTTP Async Server..."
timeout 3s $ETAMIL_BIN --async --port 8766 test_temp.etamil > /dev/null 2>&1 &
SERVER_PID=$!
sleep 2
if command -v curl > /dev/null 2>&1; then
    curl -s http://localhost:8766/ > /dev/null 2>&1
    CURL_RESULT=$?
else
    ps -p $SERVER_PID > /dev/null 2>&1
    CURL_RESULT=$?
fi

if [ $CURL_RESULT -eq 0 ]; then
    echo "✓ HTTP Async Server: PASS"
    ((PASSED++))
else
    echo "✗ HTTP Async Server: FAIL"
    ((FAILED++))
fi
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null
echo ""

# Test 5: LLVM Backend (optional)
echo "[5/5] Testing LLVM Backend..."
$ETAMIL_BIN --llvm test_temp.etamil 2>&1 | grep -q "Successfully saved\|not available"
if [ $? -eq 0 ]; then
    if $ETAMIL_BIN --llvm test_temp.etamil 2>&1 | grep -q "not available"; then
        echo "ℹ LLVM Backend: NOT AVAILABLE (expected if not built with --features llvm)"
    else
        echo "✓ LLVM Backend: AVAILABLE"
        ((PASSED++))
    fi
else
    echo "✗ LLVM Backend: UNEXPECTED ERROR"
    ((FAILED++))
fi
echo ""

# Cleanup
rm -f test_temp.etamil output.ll output.bin

# Summary
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "✓ All tests passed! eTamil is working correctly on this platform."
    echo ""
    echo "Platform compatibility: ✅ VERIFIED"
    exit 0
else
    echo "✗ Some tests failed. Please check the errors above."
    exit 1
fi
