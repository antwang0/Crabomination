#!/bin/bash
# Fresh-seed dflt-mirror sweep on the debug-assertions build — the cheapest
# bug finder on the branch (2026-09-09: two defects in 154 k games, one a
# 6,001-action loop, one an assertion abort; `robustness_grid.sh`'s `gang`
# cells had walked past both).
#
# `dflt` is the pilot the training actors run, so its search reaches boards
# `gang` never builds. Each cell is one pool x one seed x `--games` per
# archetype, both orientations of every pair, on 3 threads.
# `CRAB_MAX_ACTIONS=6000` ends a looping game in under a minute instead of
# holding a thread to the 50,000 cap, and `CRAB_CAP_DIAG=4000` prints the
# board of any game past 4,000 actions (stack targets, linked exiles), so a
# cap names itself in the log. Only `cap` and `stuck` are defects; `draw` is
# a rules outcome (CR 104.4).
#
#   RUSTFLAGS="-C debug-assertions=yes" CARGO_TARGET_DIR=target-audit \
#     cargo build --profile overflow -p crabomination --bin bot_ladder
#   scripts/fresh_seed_sweep.sh "cube all sealed" "725 726 727 728"   # 400 games
#   scripts/fresh_seed_sweep.sh cube "738 739" 400 target-audit/overflow/bot_ladder
#
# Seeds used so far are in PERF's Baseline (search "fresh seeds"); take the
# next ones. A `FAIL` with rc 134 is a panic / assertion under
# `panic = "abort"` — the message is in the log. To replay one capped or
# aborted game, find its pair with a finder test over
# `cube_archetypes(seed, 8)` (see `golden_trace::the_two_hole_relic_warder_
# pair_decides` for the deck and pair-seed derivation) and pin it there.
set -u
POOLS=${1:?pools, e.g. "cube all sealed"}
SEEDS=${2:?seeds, e.g. "725 726"}
GAMES=${3:-400}
BIN=${4:-target-audit/overflow/bot_ladder}
cd "$(dirname "$0")/.."
[ -x "$BIN" ] || { echo "no $BIN — build it (header)"; exit 1; }
cells=0 games=0 cap=0 stuck=0 draw=0 fail=0
for pool in $POOLS; do
  for seed in $SEEDS; do
    t0=$(date +%s)
    out=$(RUST_MIN_STACK=33554432 CRAB_CAP_DIAG=4000 CRAB_MAX_ACTIONS=6000 timeout 3600 "$BIN" \
      --a dflt --b dflt --games "$GAMES" --threads 3 --seed "$seed" --decks "$pool" 2>&1)
    rc=$?
    t1=$(date +%s)
    line=$(echo "$out" | grep -E "^[0-9]+ decided" | tail -1)
    by=$(echo "$out" | grep -E "^  undecided_by" | tail -1)
    if [ $rc -ne 0 ] || [ -z "$line" ]; then
      echo "FAIL pool=$pool seed=$seed rc=$rc ($((t1 - t0)) s)"; echo "$out" | tail -30; fail=$((fail + 1))
    else
      echo "ok pool=$pool seed=$seed  $line ${by:+ |$by} ($((t1 - t0)) s)"
      echo "$out" | grep -A6 "^cap: " | head -40
      games=$((games + $(echo "$line" | awk '{print $1 + $3}')))
      if [ -n "$by" ]; then
        set -- $(echo "$by" | awk '{print $3, $6, $9}')
        cap=$((cap + $1)) stuck=$((stuck + $2)) draw=$((draw + $3))
      fi
    fi
    cells=$((cells + 1))
  done
done
[ $((cap + stuck + fail)) -eq 0 ] || fail=$((fail + cap + stuck))
echo "SWEEP DONE cells=$cells games=$games failures=$fail   undecided cap $cap / stuck $stuck / draw $draw (only cap+stuck are defects)"
