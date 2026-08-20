#!/usr/bin/env bash
#
# Run every example and check each one behaves as expected.
#
# Three examples are expected to FAIL: they use route statements the VM cannot
# execute, and failing loudly is the intended behaviour. This script fails if
# any other example breaks, or if one of those starts passing without the
# expectation being updated.
#
# Examples needing a database server that this repository does not provide are
# skipped unless their guard variable is set, so a plain run stays hermetic.
#
#   ./scripts/run_examples.sh
#   ETAMIL_TEST_MYSQL=1 ./scripts/run_examples.sh

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
    ["examples/api/vari_cEvY.qmz"]="not implemented"
    ["examples/kadai/kadai_cEvY.qmz"]="not implemented"
)

# Examples needing an external server, and the variable that opts them in.
# The SQLite sample is not here: that driver is bundled and writes a file in
# the temp directory below, so it runs anywhere.
declare -A NEEDS_SERVER=(
    ["examples/db_samples/mYcIkul_qaLam.qmz"]="ETAMIL_TEST_MYSQL"
)

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

pass=0
fail=0
skip=0
declare -a FAILURES=()

while IFS= read -r file; do
    rel="${file#"$ROOT"/}"

    # Opt-in examples: skipped, and said out loud, rather than counted as
    # passing — a silent skip reads as coverage that is not there.
    guard="${NEEDS_SERVER[$rel]:-}"
    if [[ -n "$guard" && -z "${!guard:-}" ]]; then
        echo "  skipped          $rel (set $guard=1 to run it)"
        ((skip++))
        continue
    fi

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
echo "  $pass as expected, $fail unexpected, $skip skipped"

if [[ $fail -gt 0 ]]; then
    echo
    echo "Unexpected results:"
    printf '  - %s\n' "${FAILURES[@]}"
    exit 1
fi

echo "  all examples behaved as expected"
