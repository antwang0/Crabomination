#!/usr/bin/env bash
# Round 77b owed read: the magecraft stop's removal + shortlist 4 under the
# 256-search, paired. `mcts-dflt-256` carries the adopted default;
# `mcts-r77off-256` is the same search on the pre-77b default. Seed 97 first
# (the larger scored gain), then 43. Runs from a copied binary so later
# rebuilds cannot touch it.
cd /home/archuser/repos/Crabomination
for g in 97 43; do
  f=.ladder/r78/mcts_r77b_vs_off_g$g.txt
  if [ -s "$f" ] && grep -q "A win%" "$f"; then continue; fi
  .ladder/r78/bot_ladder_r78 --a mcts-dflt-256 --b mcts-r77off-256 --decks sealed --games 500 --seed $g --threads 22 > "$f" 2>&1
  echo "exit: $?" >> "$f"
done
echo "== done $(date -Is)" > .ladder/r78/search_transfer.done
