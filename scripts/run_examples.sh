#!/usr/bin/env bash
#
# Run every example and check each one behaves as expected.
#
# Three examples are expected to FAIL: they use database or route statements
# the VM cannot execute, and failing loudly is the intended behaviour. This
# script fails if any other example breaks, or if one of those three starts
# passing without the expectation being updated.
#
#   ./scripts/run_examples.sh

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${ETAMIL_BIN:-$ROOT/etamil_compiler/target/release/etamil}"

if [[ ! -x "$BIN" ]]; then
    echo "error: $BIN not found."
    echo "       build it first: (cd etamil_compiler && cargo build --release)"
    exit 1
fi

# இறக்கு resolves relative to the importing file first, but set this so
# examples work no matter where they are run from.
export ETAMIL_PATH="$ROOT"

# Examples that must fail, and the text their error must contain.
declare -A EXPECT_FAIL=(
    ["examples/api/simple_api.qmz"]="not implemented"
    ["examples/db_samples/multi_db_test.qmz"]="not implemented"
    ["examples/db_samples/test_db_connectivity.qmz"]="not implemented"
)

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

pass=0
fail=0
declare -a FAILURES=()

while IFS= read -r file; do
    rel="${file#"$ROOT"/}"

    # I/O examples create files; keep the repository clean.
    output="$(cd "$WORK" && echo "0" | "$BIN" --vm "$file" 2>&1)"
    status=$?

    expected="${EXPECT_FAIL[$rel]:-}"

    if [[ -n "$expected" ]]; then
        if [[ $status -eq 0 ]]; then
            echo "  UNEXPECTED PASS  $rel"
            echo "                   expected it to fail with: $expected"
            FAILURES+=("$rel (unexpected pass)")
            ((fail++))
        elif grep -qF "$expected" <<<"$output"; then
            echo "  fails as designed  $rel"
            ((pass++))
        else
            echo "  WRONG ERROR      $rel"
            echo "                   wanted: $expected"
            echo "                   got:    $(head -n 3 <<<"$output")"
            FAILURES+=("$rel (wrong error)")
            ((fail++))
        fi
    else
        if [[ $status -eq 0 ]]; then
            echo "  ok               $rel"
            ((pass++))
        else
            echo "  FAILED           $rel"
            sed 's/^/                   /' <<<"$(head -n 5 <<<"$output")"
            FAILURES+=("$rel")
            ((fail++))
        fi
    fi
done < <(find "$ROOT/examples" "$ROOT/nUlakam" \
             -type f \( -name '*.qmz' -o -name '*.etamil' \) | sort)

echo
echo "-------------------------------------------"
echo "  $pass as expected, $fail unexpected"

if [[ $fail -gt 0 ]]; then
    echo
    echo "Unexpected results:"
    printf '  - %s\n' "${FAILURES[@]}"
    exit 1
fi

echo "  all examples behaved as expected"
