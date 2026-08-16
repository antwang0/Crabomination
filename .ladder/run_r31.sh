#!/usr/bin/env bash
# Round 31: MCTS combat coverage. mcts-net-combat searches attack and
# block declarations over the same candidate menus the sim searches
# score, with rollouts + net rewards in place of the one-turn sims;
# mcts-net-deep (the round-26 adoption) leaves combat to the heuristic.
# Same 64-iteration budget both sides, so the gate measures coverage,
# not compute. Per-decision cost rises (combat ticks now search), so
# the cells' wall clock is part of the record. 100-game paired cells,
# seeds 43/97, champion net both sides via the ladder fallback.
set -eu
cd /home/archuser/repos/Crabomination
LADDER=./target/release-fast/bot_ladder

for gseed in 43 97; do
  $LADDER --a mcts-net-combat --b mcts-net-deep --decks sealed \
      --games 100 --seed "$gseed" \
      > ".ladder/r31_combat_vs_deep_s$gseed.txt" 2>&1
  echo "seed $gseed done"
done
echo "round-31 gates complete"
