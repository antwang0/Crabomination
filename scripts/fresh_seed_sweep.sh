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
# cap names itself in the log. Only a NOVEL `cap` and `stuck` are defects:
# `draw` is a rules outcome (CR 104.4), and one cap shape is diagnosed and is
# not a defect either — a seat at `i32::MAX` life, which is Beacon of
# Immortality doubling and shuffling itself back, seen at `cube` 1018, 1069 and
# 1076. `cap_diagnosis` labels that board `[SATURATED LIFE …]` and the totals
# below count it apart, so a cube block does not read as a failure for a board
# nobody is going to change.
#
# ⚠ **A NOVEL CAP IS RE-RUN AT THE PRODUCTION ACTION CAP BEFORE IT IS
# REPORTED, and the script does that itself.** `CRAB_MAX_ACTIONS=6000` is a
# tenth of what production allows — set that low so a real loop ends in under a
# minute — which makes a legitimately LONG game read exactly like a stuck one,
# and the `[SATURATED LIFE]` label cannot tell them apart: that board is
# recognisable, a big slow board is not. `all` seed 1149 was the first: 2 caps
# at 6,000 actions, 41 triggers on the stack at turn 89 on a 40/41-permanent
# Lorehold board — and at `CRAB_MAX_ACTIONS=50000` the same cell reads
# **6,800 decided, 0 undecided**. Slow, not stuck.
#
# So a cell that caps with no label is re-run at 50,000 automatically; the cost
# is paid only when a novel cap appears. Caps that clear are counted as
# `slow-not-stuck`, and only a cap that SURVIVES the re-run is a defect.
#
# ⚠ **AND A `board` CAP IS NOT AN ACTION CAP AT ALL.** `StopReason::BoardCap`
# ends a game whose battlefield passes 1,024 permanents — a token-doubling
# runaway, bounded on purpose — and it used to be summed into the same `cap`
# counter, which made it the one undecided shape the 50,000-action re-run can
# NEVER clear: a Krenko board doubles past the bound in one activation whatever
# the budget. `cube` and `all` seed 1169 are the worked example (1,967 Goblins
# at turn 29, 1,535 at turn 46) and they read as ten defects until `SimCost`
# split the two. It has its own column now.
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
cells=0 games=0 cap=0 board=0 stuck=0 draw=0 fail=0 sat=0 slow=0
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
        # `undecided_by   cap N / board N / stuck N / draw N`
        set -- $(echo "$by" | awk '{print $3, $6, $9, $12}')
        cap=$((cap + $1)) board=$((board + $2)) stuck=$((stuck + $3)) draw=$((draw + $4))
        # The one cap shape that is diagnosed and is NOT a defect: a seat at
        # `i32::MAX` life (Beacon of Immortality doubling itself back into the
        # library). `cap_diagnosis` labels it; counted apart so a cube block
        # does not read as a failure for a board nobody is going to change.
        # Three seeds so far — cube 1018, 1069, 1076 — i.e. a pool property.
        known=$(echo "$out" | grep -c "SATURATED LIFE")
        if [ "$known" -gt 0 ]; then sat=$((sat + $1)); fi
        # A cap this cell cannot explain: RE-RUN IT AT THE PRODUCTION ACTION
        # CAP before reporting it. `CRAB_MAX_ACTIONS=6000` is a tenth of what
        # production allows and is set that low so a real loop ends in under a
        # minute — which makes a merely LONG game read exactly like a stuck
        # one (`all` 1149: 41 triggers on the stack at turn 89, and 6,800 /
        # 6,800 decided at 50,000). Only paid when a novel cap appears, so the
        # common case costs nothing, and the sweep's verdict stops needing a
        # second command to interpret.
        # ⚠ RE-RUN ANY CAP, LABELLED OR NOT. The `[SATURATED LIFE]` label used
        # to mean "an unwinnable board nobody will fix", and since `(-291)`'s
        # turn-granular no-progress watch it means "a board the watch DRAWS,
        # given the turns" — which 6,000 actions does not always allow. `cube`
        # 1204 reads `cap 8 / draw 12` here and `cap 0 / draw 20` at 50,000:
        # every one of the eight was the budget, not the board. Excusing a
        # labelled cap without the re-run reported 14 of them in one block.
        if [ "$1" -gt 0 ]; then
          echo "  cap — re-running this cell at CRAB_MAX_ACTIONS=50000 …"
          re=$(RUST_MIN_STACK=33554432 CRAB_CAP_DIAG=20000 CRAB_MAX_ACTIONS=50000 \
            timeout 7200 "$BIN" --a dflt --b dflt --games "$GAMES" --threads 3 \
            --seed "$seed" --decks "$pool" 2>&1)
          reby=$(echo "$re" | grep -E "^  undecided_by" | tail -1)
          recap=$(echo "$reby" | awk '{print $3}')
          if [ -z "$reby" ] || [ "${recap:-0}" -eq 0 ]; then
            echo "  -> the ACTION BUDGET, not the board: $(echo "$re" | grep -E '^[0-9]+ decided' | tail -1)"
            echo "     $reby"
            slow=$((slow + $1))
          else
            echo "  -> STILL CAPPED at 50,000 actions — a defect. $reby"
            echo "$re" | grep -A6 "^cap: " | head -40
          fi
        fi
      fi
    fi
    cells=$((cells + 1))
  done
done
# Every cap goes through the re-run now, so `slow` alone decides: `sat` is
# diagnostic (which BOARD it was) and subtracting both double-counted a cap
# that is labelled AND cleared — `cube` 1204 scored `failures=-8`.
novel=$((cap - slow))
[ $((novel + stuck + fail)) -eq 0 ] || fail=$((fail + novel + stuck))
echo "SWEEP DONE cells=$cells games=$games failures=$fail   undecided cap $cap (of which $sat the known saturated-life board, $slow slow-not-stuck) / board $board / stuck $stuck / draw $draw"
echo "  stuck and a cap that SURVIVES the 50,000-action re-run ($novel) are defects;"
echo "  a draw is CR 104.4 and a BOARD cap is the 1,024-permanent bound doing its job"
