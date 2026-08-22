#!/usr/bin/env bash
#
# Compare the two backends: does --llvm produce the same answers as --vm?
#
# The VM is the reference. It runs the whole language, and every example is
# expected to work under it. The LLVM backend covers less, and refuses what it
# cannot build rather than emitting IR that computes something else — so for
# most programs the honest result today is "refused", not "wrong".
#
# That refusal is the point of this script. Each one names the constructs that
# stopped it, and the summary counts them, so the gap between the backends is
# a list of things to build rather than an impression.
#
#   ./scripts/run_parity.sh
#
# Needs a binary built with the LLVM feature:
#
#   (cd etamil_compiler && cargo build --release --features llvm)
#
# If it finds one built without, it says so and exits 0 — a machine that
# cannot build LLVM (Windows, today) is not a failing machine.
#
# Where clang is on PATH, an accepted program is taken all the way: IR is
# compiled, the binary is run, and its output is compared against the VM's.
# Without clang the IR is accepted as far as it goes and reported separately,
# because "it compiled" and "it computes the same thing" are different claims.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${ETAMIL_BIN:-$ROOT/etamil_compiler/target/release/etamil}"

if [[ ! -x "$BIN" ]]; then
    if [[ -x "$BIN.exe" ]]; then
        BIN="$BIN.exe"
    else
        echo "error: $BIN not found."
        echo "       build it first: (cd etamil_compiler && cargo build --release --features llvm)"
        exit 1
    fi
fi

export ETAMIL_PATH="$ROOT"

# Does this binary have the LLVM backend at all?
probe="$(mktemp -d)"
printf 'a = 1;\n' > "$probe/probe.qmz"
llvm_check="$("$BIN" --llvm "$probe/probe.qmz" 2>&1)"
rm -rf "$probe"

if grep -q "LLVM backend is not available" <<<"$llvm_check"; then
    echo "This binary was built without the LLVM backend, so there is nothing to compare."
    echo "  build it with:  (cd etamil_compiler && cargo build --release --features llvm)"
    echo "  Linux and macOS only — see README."
    exit 0
fi

have_clang=0
command -v clang >/dev/null 2>&1 && have_clang=1

# Examples the VM is expected to refuse: they use route statements it cannot
# execute. Nothing to compare when there is no reference answer.
declare -A SKIP=(
    ["examples/api/simple_api.qmz"]=1
    ["examples/api/vari_cEvY.qmz"]=1
    ["examples/kadai/kadai_cEvY.qmz"]=1
)

matched=0; mismatched=0; refused=0; compiled=0; skipped=0
declare -a MISMATCHES=()
unsupported_log="$(mktemp)"

while IFS= read -r file; do
    rel="${file#"$ROOT"/}"
    rel="${rel//\\//}"

    if [[ -n "${SKIP[$rel]:-}" ]]; then
        ((skipped++)); continue
    fi

    # The reference answer. An example the VM cannot run is not a parity
    # question, so it is skipped rather than counted against the backend.
    vm_out="$(cd "$(dirname "$file")" && "$BIN" --vm "$file" 2>&1)"
    if [[ $? -ne 0 ]]; then
        ((skipped++)); continue
    fi

    # --llvm writes output.ll into the working directory, so it runs in a
    # temporary one while the VM runs beside the example. That differs for a
    # program reading a data file by relative path — but every such example is
    # refused by the backend today, so the difference is unreachable. If one
    # ever compiles, it will show up here as a mismatch to look at rather than
    # as a wrong answer nobody sees.
    work="$(mktemp -d)"
    llvm_out="$(cd "$work" && "$BIN" --llvm "$file" 2>&1)"
    llvm_status=$?

    if [[ $llvm_status -ne 0 ]]; then
        # Refused, and it says what stopped it. Those lines are the roadmap.
        grep -E '^    - ' <<<"$llvm_out" | sed 's/^    - //' >> "$unsupported_log"
        echo "  refused          $rel"
        ((refused++))
        rm -rf "$work"
        continue
    fi

    if [[ $have_clang -eq 0 ]]; then
        echo "  compiled         $rel  (no clang: IR not run)"
        ((compiled++))
        rm -rf "$work"
        continue
    fi

    if ! (cd "$work" && clang output.ll -o prog -lm >/dev/null 2>&1); then
        echo "  IR REJECTED      $rel  (clang would not build the emitted IR)"
        MISMATCHES+=("$rel — clang rejected the IR")
        ((mismatched++))
        rm -rf "$work"
        continue
    fi

    native_out="$(cd "$work" && ./prog 2>&1)"

    # The VM prints a banner around a program's output; compare only what the
    # program itself wrote.
    vm_body="$(sed -n '/=== Execution Output ===/,$p' <<<"$vm_out" \
               | sed '1d;/^✓ Execution completed successfully$/d' | sed '/^$/d')"
    native_body="$(sed '/^$/d' <<<"$native_out")"

    if [[ "$vm_body" == "$native_body" ]]; then
        echo "  match            $rel"
        ((matched++))
    else
        echo "  MISMATCH         $rel"
        MISMATCHES+=("$rel")
        ((mismatched++))
    fi
    rm -rf "$work"
done < <(find "$ROOT/examples" "$ROOT/nUlakam" \
             -type f \( -name '*.qmz' -o -name '*.etamil' \) | sort)

echo
echo "-------------------------------------------"
echo "  $matched match, $mismatched mismatch, $refused refused, $compiled compiled-only, $skipped skipped"

if [[ -s "$unsupported_log" ]]; then
    echo
    echo "  What the LLVM backend still cannot build, most frequent first:"
    sort "$unsupported_log" | uniq -c | sort -rn | head -20 | sed 's/^/    /'
fi
rm -f "$unsupported_log"

if [[ $mismatched -gt 0 ]]; then
    echo
    echo "  The two backends disagree — a refusal is expected, a wrong answer is not:"
    printf '    - %s\n' "${MISMATCHES[@]}"
    exit 1
fi

echo
echo "  No disagreement: where the LLVM backend accepted a program, it agreed with the VM."
