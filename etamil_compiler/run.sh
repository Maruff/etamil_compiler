#!/bin/bash
# eTamil Compiler Run Script

set -e

echo "🔧 Compiling eTamil source..."
LLVM_CONFIG_PATH=/usr/lib/llvm-18/bin/llvm-config cargo run --quiet

echo ""
echo "🏗️  Compiling LLVM IR to native code..."
/usr/lib/llvm-18/bin/llc -filetype=obj output.ll -o output.o
CC_BIN=${CC:-cc}
echo "   Using C compiler: $CC_BIN"
$CC_BIN -no-pie output.o -o output.bin

echo ""
echo "▶️  Running native binary..."
./output.bin
EXIT_CODE=$?

echo ""
if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ Program executed successfully!"
else
    echo "❌ Program failed with exit code: $EXIT_CODE"
fi

exit $EXIT_CODE
