#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

MODE="${1:-}"
if [[ "$MODE" == "clean" ]]; then
    rm -f ./*.err
    echo "cleaned .err files"
    exit 0
fi

FUNCTIONS=(
    mpmc_recv
    mpmc_recv::poll_acquire_slot_cold
    mpmc_recv_blocking
    mpmc_recv_blocking::acquire_slot_blocking_cold
    mpmc_send
    mpmc_send::poll_acquire_slot_cold
    mpmc_send_blocking
    mpmc_send_blocking::acquire_slot_blocking_cold
    mpsc_recv
    mpsc_recv::poll_acquire_slot_cold
    mpsc_recv_blocking
    mpsc_recv_blocking::acquire_slot_blocking_cold
    mpsc_send
    mpsc_send::poll_acquire_slot_cold
    mpsc_send_blocking
    mpsc_send_blocking::acquire_slot_blocking_cold
    spmc_recv
    spmc_recv::poll_acquire_slot_cold
    spmc_recv_blocking
    spmc_recv_blocking::acquire_slot_blocking_cold
    spmc_send
    spmc_send::poll_acquire_slot_cold
    spmc_send_blocking
    spmc_send_blocking::acquire_slot_blocking_cold
    spsc_recv
    spsc_recv::poll_acquire_slot_cold
    spsc_recv_blocking
    spsc_recv_blocking::acquire_slot_blocking_cold
    spsc_send
    spsc_send::poll_acquire_slot_cold
    spsc_send_blocking
    spsc_send_blocking::acquire_slot_blocking_cold
)

PASS=0
FAIL=0

for entry in "${FUNCTIONS[@]}"; do
    if [[ "$entry" == *:* ]]; then
        cfg="${entry%%:*}"
        fn="${entry##*:}"
    else
        cfg="$entry"
        fn="$entry"
    fi

    actual=$(RUSTFLAGS="--cfg $cfg" cargo asm --lib --target x86_64-unknown-linux-gnu --simplify "$fn" 2>/dev/null)
    asm_file="${entry//::/__}.s"

    if [[ "$MODE" == "regenerate" ]]; then
        rm -f "$asm_file.err"
        printf '%s\n' "$actual" > "$asm_file"
        echo "updated: $entry"
    else
        if [[ ! -f "$asm_file" ]]; then
            echo "MISSING ref: $asm_file (run with 'regenerate' to generate)"
            FAIL=$((FAIL + 1))
            continue
        fi
        if diff -u "$asm_file" <(printf '%s\n' "$actual") > /dev/null 2>&1; then
            echo "ok: $entry"
            PASS=$((PASS + 1))
        else
            echo "FAIL: $entry  (see $entry.s.err)"
            printf '%s\n' "$actual" > "$asm_file.err"
            FAIL=$((FAIL + 1))
        fi
    fi
done

echo ""
if [[ $FAIL -gt 0 ]]; then
    echo "$FAIL function(s) failed, $PASS passed."
    exit 1
fi
echo "All $PASS function(s) passed."
