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
# saturated seats. CR 104.4's turn watch reads `repeats 10/12` there at the
# 50,000-action budget — two samples short — so the board is unwinnable AND
# aperiodic and the one verdict such a game has is never reached. `cap_diagnosis` prints the watch's own state now, so the dump answers
# the question — `repeats N/12` under 12 on a `[SATURATED LIFE]` board is that
# case, and anything else that survives is the signal this script exists for.
#
# ⚠⚠ **`NO_PROGRESS_MAX_PERIOD` IS NOT THE PARAMETER, AND THE BUDGET IS NOT
# EITHER — measured 2026-09-13, and this retires the handoff's guess.** Both
# dumps of `all` 1159 read `since 0/8`: the anchor is alive and matching at the
# moment the cap fires, so it is not the period bound that loses it. And more
# budget does not converge — the SAME cell at `CRAB_MAX_ACTIONS=200000` runs to
# turn **9,099** (4x the actions, 4x the turns) and reads **`repeats 6/12`,
# LOWER than the 10/12 it reached at 50,000**. The anchor is re-formed
# continuously and the count never crosses 12, because what moves the digest on
# that board is the TAPPED SET: both seats spend a varying amount of mana every
# turn (Underworld Connections, extort), and `tapped` is one bit per permanent
# in the fingerprint. **The watch is a periodicity detector and this board is
# aperiodic**; no value of its three constants closes that — a longer period or
# a lower repeat count only changes which aperiodic board survives. The shape
# that would close it is a different predicate ("no seat can win or lose"),
# which is an adjudication change, not a tuning one. Do not re-price the
# constants; they have been.
#
# ⚠ **A RUNAWAY BOARD SKIPS THE RE-RUN, AND THAT IS THE THIRD THING THE `cap`
# BUCKET HELD.** `cube` 1215: 951 Scute Swarms on a 987-permanent board at turn
# 59, `cap 2 / board 2`, and the cell costs **1,588 s against ~40 s** for the
# seeds either side. The two action caps there are the same runaway a few
# actions short of the bound, so re-running them at 50,000 re-derives a bound
# that has already fired — 12 CPU-minutes in and still going when it was
# stopped. A dump with a seat past `BIG_BOARD` (500) permanents is counted with
# the board caps and the re-run is skipped, with the number printed.
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
# A seat's permanent count past this is the `MAX_BATTLEFIELD` runaway rather
# than a lead, and its 50,000-action re-run is guaranteed waste — see the block
# that reads it. Raise it only with a board that justifies the number.
BIG_BOARD=${BIG_BOARD:-500}
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
        # ⚠ **A RUNAWAY BOARD IS NOT A LEAD AND ITS RE-RUN IS GUARANTEED
        # WASTE.** The re-run exists to tell a LONG game from a stuck one, and
        # a board in the hundreds of permanents is neither — it is the runaway
        # `MAX_BATTLEFIELD` exists for, and the action caps beside it are the
        # same runaway a few actions short of the bound. `cube` 1215 is the
        # worked example: 951 Scute Swarms on a 987-permanent board at turn 59,
        # `cap 2 / board 2`, and the cell itself already costs 1,588 s against
        # ~40 s for the seeds either side (PERF's candidates). Its 50,000-action
        # re-run was still going after 12 CPU-minutes and would have spent the
        # whole `timeout 7200` to re-derive a bound that had already fired.
        # So: a dump with a seat past `BIG_BOARD` permanents is counted as the
        # known runaway and the re-run is skipped, with the number printed.
        big=$(echo "$out" | grep -oE "^  p[0-9]: life -?[0-9]+ bf [0-9]+" \
              | grep -oE "bf [0-9]+" | grep -oE "[0-9]+" | sort -rn | head -1)
        if [ "$1" -gt 0 ] && [ "${big:-0}" -ge "${BIG_BOARD:-500}" ]; then
          echo "  cap — NOT re-run: a runaway board (${big} permanents), which is what"
          echo "     MAX_BATTLEFIELD is for. Counted with the board caps, not as a defect."
          echo "$out" | grep -A7 "^cap: " | head -30
          known_board=$((known_board + $1))
        elif [ "$1" -gt 0 ]; then
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
            # either. `all` 1159 is the worked example — `repeats 10/12` in
            # the `no-progress watch:` line of the dump below, i.e. CR 104.4's
            # turn watch gets two samples short of the draw and cannot hold an
            # anchor on a board the bot plays slightly differently each turn.
            # Reported, counted, not a failure.
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
# `sat` is diagnostic only (which BOARD the cap was on) and decides nothing —
# subtracting it as well as `slow` double-counted a cap that is labelled AND
# cleared, and scored `cube` 1204 at `failures=-8`. What clears a cap is the
# RE-RUN: `slow` if it decided the games, `known_board` if it kept them on the
# saturated board.
# A cap is a defect unless the re-run CLEARED it (`slow`) or the re-run kept it
# AND the board is the saturated one (`known_board`). The label alone never
# excuses one — that is what `cube` 1204 cost — and a survived cap without the
# label is the signal this whole script exists for.
novel=$((cap - slow - known_board))
[ $((novel + stuck + fail)) -eq 0 ] || fail=$((fail + novel + stuck))
echo "SWEEP DONE cells=$cells games=$games failures=$fail   undecided cap $cap (of which $slow slow-not-stuck, $known_board a known board — saturated life or a MAX_BATTLEFIELD runaway; $sat carried the saturated label) / board $board / stuck $stuck / draw $draw"
echo "  stuck and a cap that SURVIVES the re-run without the saturated label ($novel) are defects;"
echo "  a draw is CR 104.4 and a BOARD cap is the 1,024-permanent bound doing its job"
