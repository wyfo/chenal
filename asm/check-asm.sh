#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

MODE="${1:-}"
if [[ "$MODE" == "clean" ]]; then
    find . -name '*.err' -delete
    echo "cleaned .err files"
    exit 0
fi

# arch label : rust target triple : extra rustflags
# +lse on aarch64 emits inline cas/casal instead of the __aarch64_cas8_* outline-atomics calls
ARCHES=(
    "x86_64:x86_64-unknown-linux-gnu:"
    "aarch64:aarch64-unknown-linux-gnu:-C target-feature=+lse"
)

FAMILIES=(mpmc mpsc spmc spsc)

# base function name : cold-variant function name
ENTRIES=(
    "recv:poll_acquire_slot_cold"
    "recv_blocking:acquire_slot_blocking_cold"
    "send:poll_acquire_slot_cold"
    "send_blocking:acquire_slot_blocking_cold"
)

# check_unit <arch> <target> <extra_flags> <family> <cfg> <fn> <name>
#   cfg  - the --cfg flag value (e.g. mpmc_recv)
#   fn   - the symbol passed to `cargo asm`
#   name - reference file base name within <arch>/<family>/
# Runs as a backgrounded job, so it reports its outcome via exit status
# (0 = ok/regenerated, non-zero = FAIL/MISSING) and prints its own status line.
check_unit() {
    local arch="$1" target="$2" extra="$3" family="$4" cfg="$5" fn="$6" name="$7"
    local label="$arch/$family/$name"

    # Each cfg gets its own target dir, so parallel units don't contend on a
    # shared cargo build lock and incremental reruns stay fast. cargo-asm
    # ignores CARGO_TARGET_DIR and won't create the dir, so we pass
    # --target-dir explicitly and mkdir it ourselves.
    local tdir="$SCRIPT_DIR/target/$arch/$cfg"
    mkdir -p "$tdir"

    local actual=$(CARGO_TARGET_DIR=$tdir RUSTFLAGS="--cfg $cfg $extra" cargo asm --lib --target "$target" --simplify "$fn" 2>/dev/null) || true

    local asm_file="$arch/$family/$name.s"

    if [[ "$MODE" == "regenerate" ]]; then
        mkdir -p "$(dirname "$asm_file")"
        rm -f "$asm_file.err"
        printf '%s\n' "$actual" > "$asm_file"
        echo "updated: $label"
        return 0
    fi

    if [[ ! -f "$asm_file" ]]; then
        echo "MISSING ref: $asm_file (run with 'regenerate' to generate)"
        return 1
    fi
    if diff -u "$asm_file" <(printf '%s\n' "$actual") > /dev/null 2>&1; then
        echo "ok: $label"
        return 0
    else
        echo "FAIL: $label  (see $asm_file.err)"
        printf '%s\n' "$actual" > "$asm_file.err"
        return 1
    fi
}

pids=()
for arch_entry in "${ARCHES[@]}"; do
    IFS=: read -r arch target extra <<< "$arch_entry"

    for family in "${FAMILIES[@]}"; do
        for entry in "${ENTRIES[@]}"; do
            base="${entry%%:*}"
            cold="${entry##*:}"
            cfg="${family}_${base}"

            # hot path: the function itself
            check_unit "$arch" "$target" "$extra" "$family" "$cfg" "$cfg" "$base" &
            pids+=("$!")
            # cold path: the spilled slow-path helper
            check_unit "$arch" "$target" "$extra" "$family" "$cfg" "$cold" "${base}__${cold}" &
            pids+=("$!")
        done
    done
done

FAIL=0
for pid in "${pids[@]}"; do
    wait "$pid" || FAIL=$((FAIL + 1))
done
PASS=$(( ${#pids[@]} - FAIL ))

echo ""
if [[ "$MODE" == "regenerate" ]]; then
    echo "regenerated ${#pids[@]} function(s)."
    exit 0
fi
if [[ $FAIL -gt 0 ]]; then
    echo "$FAIL function(s) failed, $PASS passed."
    exit 1
fi
echo "All $PASS function(s) passed."
