#!/bin/bash
# The `-C debug-assertions=yes` robustness gate, as a grid.
#
# Every `debug_assert!`-backed invariant in the engine — the memo families on
# `CardMemo`, the `*_scan` presence-gate audits, the `computed_kw` subset
# guard — is compiled out of every release profile, so the suite is not their
# audit on real boards. PERF's (-58) is the case in point: 18,795 tests missed
# a board that sixty games of `cube` found in four seconds.
#
# Two legs. The **ladder** leg is `bot_ladder` over five pools x six seeds.
# The **actor** leg is `selfplay_train`, and it is not redundant: `bot_ladder`
# runs the encoder on no pool, so the four `debug_assert!`s in
# `server/encode.rs` — the vocab-index slot among them — have no other audit.
# Costs ~9 min of build into its own target dir (the flag re-fingerprints
# every crate), ~4 min for the 30-cell ladder grid, and ~10 min for the actor
# leg's own build plus ~1 min for its three cells.
#
#   scripts/robustness_grid.sh              # build + verify + both legs
#   scripts/robustness_grid.sh --no-build   # both legs, reusing target-audit/
#   scripts/robustness_grid.sh --no-actor   # ladder leg only
#   scripts/robustness_grid.sh --actor-only # actor leg only (still builds it)
#
# ⚠ `undecided_by draw` is a **rules outcome, not a stall** — two players
# losing at once is a legal game end. Only `cap` (turn limit) and `stuck` (no
# legal progress) are defects, so the totals below keep the three apart.
set -u
cd "$(dirname "$0")/.."

POOLS="${POOLS:-fixed cube sos sealed all}"
SEEDS="${SEEDS:-1 3 7 11 23 97}"
GAMES="${GAMES:-120}"
ACTOR_SEEDS="${ACTOR_SEEDS:-1 7 23}"
ACTOR_GAMES="${ACTOR_GAMES:-600}"
BIN=target-audit/overflow/bot_ladder
ACTOR_BIN=target-audit/overflow/selfplay_train

build=1 ladder=1 actor=1
for arg in "$@"; do
  case "$arg" in
    --no-build) build=0 ;;
    --no-actor) actor=0 ;;
    --actor-only) ladder=0 ;;
    *) echo "unknown flag $arg"; exit 2 ;;
  esac
done

# **Check the binary, not the flags.** `RUSTFLAGS` reaching the crate that
# carries the assertion is the thing that can silently not happen, and the
# proof is two seconds: the audit binary carries the messages and a
# `release-fast` one carries none.
check_asserts() {
  [ -x "$1" ] || { echo "no $1 — run without --no-build"; exit 1; }
  local n
  n=$(strings "$1" 2>/dev/null | grep -c "memo is stale")
  echo "assertion strings in $1: $n  (0 means the flag did not reach it — stop)"
  [ "$n" -gt 0 ] || exit 1
}

fail=0

if [ "$ladder" = 1 ]; then
  if [ "$build" = 1 ]; then
    RUSTFLAGS="-C debug-assertions=yes" CARGO_TARGET_DIR=target-audit \
      cargo build --profile overflow -p crabomination --bin bot_ladder || exit 1
  fi
  check_asserts "$BIN"

  cells=0 games=0 cap=0 stuck=0 draw=0
  for pool in $POOLS; do
    for seed in $SEEDS; do
      out=$(RUST_MIN_STACK=33554432 timeout 1800 "$BIN" \
        --a gang --b gang --games "$GAMES" --threads 3 --seed "$seed" --decks "$pool" 2>&1)
      rc=$?
      line=$(echo "$out" | grep -E "^[0-9]+ decided" | tail -1)
      # `undecided_by cap N / stuck N / draw N` — printed whenever the run has
      # any undecided game. Split here so a moving rate names its own cause
      # without a second run.
      by=$(echo "$out" | grep -E "^  undecided_by" | tail -1)
      if [ $rc -ne 0 ] || [ -z "$line" ]; then
        echo "FAIL pool=$pool seed=$seed rc=$rc"; echo "$out" | tail -20; fail=$((fail + 1))
      else
        echo "ok pool=$pool seed=$seed  $line ${by:+ |$by}"
        games=$((games + $(echo "$line" | awk '{print $1 + $3}')))
        if [ -n "$by" ]; then
          set -- $(echo "$by" | awk '{print $3, $6, $9}')
          cap=$((cap + $1)) stuck=$((stuck + $2)) draw=$((draw + $3))
        fi
      fi
      cells=$((cells + 1))
    done
  done
  echo "GRID DONE cells=$cells games=$games failures=$fail"
  echo "  undecided      cap $cap / stuck $stuck / draw $draw   (only cap+stuck are defects)"
  [ $((cap + stuck)) -eq 0 ] || fail=$((fail + 1))
fi

if [ "$actor" = 1 ]; then
  if [ "$build" = 1 ]; then
    RUSTFLAGS="-C debug-assertions=yes" CARGO_TARGET_DIR=target-audit \
      cargo build --profile overflow -p crabomination_ml --bin selfplay_train || exit 1
  fi
  check_asserts "$ACTOR_BIN"
  acells=0
  out_dir=$(mktemp -d)
  trap 'rm -rf "$out_dir"' EXIT
  for seed in $ACTOR_SEEDS; do
    out=$(RUST_MIN_STACK=33554432 timeout 3600 "$ACTOR_BIN" \
      --actors 3 --games "$ACTOR_GAMES" --steps 2 --seed "$seed" \
      --out "$out_dir/s$seed" 2>&1)
    rc=$?
    line=$(echo "$out" | grep -E "^actors: " | tail -1)
    if [ $rc -ne 0 ]; then
      echo "FAIL actor seed=$seed rc=$rc"; echo "$out" | tail -25; fail=$((fail + 1))
    else
      echo "ok actor seed=$seed  ${line:-ran}"
    fi
    acells=$((acells + 1))
  done
  echo "ACTOR DONE cells=$acells failures=$fail"
fi

[ "$fail" -eq 0 ]
