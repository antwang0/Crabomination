#!/usr/bin/env bash
# Round 75 (2026-09-14): the attack blocker guard under the 256-search, read
# with the clean instrument round 65 asked for — a PAIRED search-level A/B,
# `mcts-guard-256` vs `mcts-dflt-256`, instead of two unpaired cells against
# a stale `dflt` reference (r65 stage 2 read −1.0 / −1.3 against round 64's
# cells, taken on an older default and an older engine, ±0.95 a side).
#
# WHY. The guard (`EvalWeights::attack_blocker_guard`) closes greedy's suicide
# holes — fliers into bigger fliers, ground into reach, six into six — and is
# strength-neutral at the heuristic level (r65 stage 1: +0.0 pooled, no cell
# below 50). It is PARKED off the default only on the stage-2 search read,
# whose mechanism (a rollout opponent that never suicides makes every line
# look harder) is argued, not measured. The lobby and the client run the
# search, so this is the reading that decides what the human faces.
#
# PRE-REGISTERED (sealed, paired, seeds 43 / 97, 500 games × 12 decks a
# cell, ±0.95; both sides search, so ~2x a scored cell's wall clock):
#   guard >= 51.0 on both seeds  -> the guard helps under the search; adopt
#                                   `attack_blocker_guard` on the default
#                                   (stage 1 already cleared no-loss).
#   within [49.0, 51.0] pooled   -> neutral; adopt on the client-quality
#                                   case (the seven replay suicides), the
#                                   r54 / r65-stage-1 shape.
#   <= 49.0 on both seeds        -> r65's park is confirmed on a paired read;
#                                   record the rollout-opponent mechanism as
#                                   measured and close the lead.
set -u
cd /home/archuser/repos/Crabomination
LADDER=${LADDER:-./target/release-fast/bot_ladder}
R=.ladder/r75
mkdir -p $R
for g in 43 97; do
  f=$R/guard_vs_dflt256_g$g.txt
  if [ -s "$f" ] && grep -q "A win%" "$f"; then continue; fi
  echo "=== cell g$g start $(date)"
  $LADDER --a mcts-guard-256 --b mcts-dflt-256 --decks sealed --games 500 --seed $g --threads 22 > "$f" 2>&1
  echo "  g$g: $(grep -E 'A win%' "$f" | tail -1 | tr -s ' ')"
done
echo "=== r75 done $(date)"
