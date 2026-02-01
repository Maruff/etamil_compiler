#!/bin/bash
# Test the file I/O example
echo "Compiling simple_fileio.qmz..."

# Compile the eTamil program
cargo run --quiet --bin etamil_compiler < examples/simple_fileio.qmz 2>/dev/null
if [ $? -ne 0 ]; then
    echo "Error during compilation"
    exit 1
fi

echo ""
echo "=== LLVM IR (output.ll) ==="
head -40 output.ll

echo ""
echo "=== Compiling to native binary ==="
llc -no-pie output.ll -o output.s && cc -no-pie output.s -o output.bin
if [ $? -eq 0 ]; then
    echo "✓ Native compilation successful"
    echo ""
    echo "=== Running the program ==="
    echo "100" | ./output.bin
else
    echo "✗ Native compilation failed"
fi
