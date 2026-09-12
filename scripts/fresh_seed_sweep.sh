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
# cap names itself in the log. `stuck` is always a defect and `draw` never is
# (CR 104.4); a `cap` is one unless the 50,000-action re-run below clears it, or
# keeps it on the one board that is diagnosed and unwinnable — both seats past
# `SCALE_CEILING * 1_000` life, which is Beacon of Immortality doubling and
# shuffling itself back (`cube` 1018, 1069, 1076, `all` 1159).
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
# So EVERY cap is re-run at 50,000 automatically, labelled or not — `cube` 1204
# read `cap 8 / draw 12` at 6,000 and `cap 0 / draw 20` at 50,000, so excusing a
# labelled cap without the re-run reported 14 non-defects in one block. Caps
# that clear are counted `slow-not-stuck`.
#
# ⚠ **AND A CAP THAT SURVIVES THE RE-RUN IS A DEFECT UNLESS IT IS SATURATED.**
# `all` 1159 is the first cap ever to survive one: a Beacon of Immortality board
# (both seats past `SCALE_CEILING * 1_000` life, each library holding the Beacon
# that shuffles itself back, so neither seat can be killed OR decked) with a
# Basilica Screecher on it, whose extort moves 1 life a turn between two
# saturated seats. CR 104.4's turn watch reads `repeats 2/12` there: the board
# is unwinnable AND aperiodic, so the one verdict such a game has is never
# reached. `cap_diagnosis` prints the watch's own state now, so the dump answers
# the question — `repeats N/12` under 12 on a `[SATURATED LIFE]` board is that
# case, and anything else that survives is the signal this script exists for.
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
cells=0 games=0 cap=0 board=0 stuck=0 draw=0 fail=0 sat=0 slow=0 known_board=0
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
        # `sat` is DIAGNOSTIC ONLY — which board the cap was on, printed in the
        # closing line so a reader can tell the Beacon boards from the rest. It
        # decides nothing: the verdict is the re-run's, below. (`cube` 1018,
        # 1069, 1076 and `all` 1090 / 1159 are the seeds that carry it, i.e. a
        # pool property rather than a seed one.)
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
          rerc=$?
          reby=$(echo "$re" | grep -E "^  undecided_by" | tail -1)
          redec=$(echo "$re" | grep -E "^[0-9]+ decided" | tail -1)
          recap=$(echo "$reby" | awk '{print $3}')
          # ⚠ AN ABSENT `undecided_by` MEANS TWO THINGS AND ONLY ONE OF THEM IS
          # GOOD. The line is absent when every game decided — the cleared case
          # — and also when the re-run never printed a summary at all: a
          # `timeout 7200`, an abort, an OOM. Reading the second as the first
          # scores a cell that HUNG as `slow-not-stuck`, which is the one way
          # this gate could pass a defect it was built to catch. So the clear
          # requires the `N decided` line and a zero exit; anything else is a
          # failure of the cell, reported as one.
          if [ $rerc -ne 0 ] || [ -z "$redec" ]; then
            echo "  -> RE-RUN ITSELF FAILED (rc $rerc) — NOT counted as cleared."
            echo "$re" | tail -20
          elif [ -z "$reby" ] || [ "${recap:-0}" -eq 0 ]; then
            echo "  -> the ACTION BUDGET, not the board: $(echo "$re" | grep -E '^[0-9]+ decided' | tail -1)"
            echo "     $reby"
            slow=$((slow + $1))
          elif echo "$re" | grep -q "SATURATED LIFE"; then
            # ⚠ THE LABEL EXCUSES A CAP ONLY *AFTER* THE RE-RUN, NEVER BEFORE.
            # A cap that clears at 50,000 was the budget whatever its label
            # (`cube` 1204: `cap 8` here, `cap 0 / draw 20` there). One that
            # SURVIVES and carries the label is the known unwinnable board:
            # both seats past `SCALE_CEILING * 1_000` life, a Beacon of
            # Immortality shuffling itself back so neither can be decked
            # either. `all` 1159 is the worked example — `repeats 2/12` in the
            # `no-progress watch:` line of the dump below, i.e. CR 104.4's turn
            # watch cannot hold an anchor on a board the bot plays slightly
            # differently each turn. Reported, counted, not a failure.
            echo "  -> STILL CAPPED at 50,000 and SATURATED — the known unwinnable board."
            echo "     read the \`no-progress watch:\` line: \`repeats N/12\` under 12 is why."
            echo "$re" | grep -A7 "^cap: " | head -40
            known_board=$((known_board + $1))
          else
            echo "  -> STILL CAPPED at 50,000 actions, NOT saturated — a defect. $reby"
            echo "$re" | grep -A7 "^cap: " | head -40
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
# A cap is a defect unless the re-run CLEARED it (`slow`) or the re-run kept it
# AND the board is the saturated one (`known_board`). The label alone never
# excuses one — that is what `cube` 1204 cost — and a survived cap without the
# label is the signal this whole script exists for.
novel=$((cap - slow - known_board))
[ $((novel + stuck + fail)) -eq 0 ] || fail=$((fail + novel + stuck))
echo "SWEEP DONE cells=$cells games=$games failures=$fail   undecided cap $cap (of which $slow slow-not-stuck, $known_board the known unwinnable board; $sat carried the label) / board $board / stuck $stuck / draw $draw"
echo "  stuck and a cap that SURVIVES the re-run without the saturated label ($novel) are defects;"
echo "  a draw is CR 104.4 and a BOARD cap is the 1,024-permanent bound doing its job"
