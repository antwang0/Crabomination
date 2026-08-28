#!/bin/bash
# The `-C debug-assertions=yes` robustness gate, as a grid.
#
# Every `debug_assert!`-backed invariant in the engine — the memo families on
# `CardMemo`, the `*_scan` presence-gate audits, the `computed_kw` subset
# guard — is compiled out of every release profile, so the suite is not their
# audit on real boards. PERF's (-58) is the case in point: 18,795 tests missed
# a board that sixty games of `cube` found in four seconds.
#
# Costs ~9 min of build into its own target dir (the flag re-fingerprints
# every crate) and ~4 min for the whole 30-cell grid.
#
#   scripts/robustness_grid.sh            # build + verify + grid
#   scripts/robustness_grid.sh --no-build # grid only, reusing target-audit/
#
# The actor leg is separate and matters when a memo is only reachable from the
# encoder (the vocab-index slot is): build `-p crabomination_ml --bin
# selfplay_train` with the same flags and run
#   target-audit/overflow/selfplay_train --actors 3 --games 120 --steps 2 --seed N
set -u
cd "$(dirname "$0")/.."

POOLS="${POOLS:-fixed cube sos sealed all}"
SEEDS="${SEEDS:-1 3 7 11 23 97}"
GAMES="${GAMES:-120}"
BIN=target-audit/overflow/bot_ladder

if [ "${1:-}" != "--no-build" ]; then
  RUSTFLAGS="-C debug-assertions=yes" CARGO_TARGET_DIR=target-audit \
    cargo build --profile overflow -p crabomination --bin bot_ladder || exit 1
fi

# **Check the binary, not the flags.** `RUSTFLAGS` reaching the crate that
# carries the assertion is the thing that can silently not happen, and the
# proof is two seconds: the audit binary carries the messages and a
# `release-fast` one carries none.
[ -x "$BIN" ] || { echo "no $BIN — run without --no-build"; exit 1; }
n=$(strings "$BIN" 2>/dev/null | grep -c "memo is stale")
echo "assertion strings in $BIN: $n  (0 means the flag did not reach it — stop)"
[ "$n" -gt 0 ] || exit 1

fail=0 cells=0 games=0
for pool in $POOLS; do
  for seed in $SEEDS; do
    out=$(RUST_MIN_STACK=33554432 timeout 1800 "$BIN" \
      --a gang --b gang --games "$GAMES" --threads 3 --seed "$seed" --decks "$pool" 2>&1)
    rc=$?
    line=$(echo "$out" | grep -E "^[0-9]+ decided" | tail -1)
    if [ $rc -ne 0 ] || [ -z "$line" ]; then
      echo "FAIL pool=$pool seed=$seed rc=$rc"; echo "$out" | tail -20; fail=$((fail + 1))
    else
      echo "ok pool=$pool seed=$seed  $line"
      games=$((games + $(echo "$line" | awk '{print $1 + $3}')))
    fi
    cells=$((cells + 1))
  done
done
echo "GRID DONE cells=$cells games=$games failures=$fail"
[ "$fail" -eq 0 ]
