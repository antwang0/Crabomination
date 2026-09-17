#!/usr/bin/env bash
# Round 76b owed read: the X re-size under the 256-search, paired, seed 43
# (the two threshold cards present). `mcts-dflt-256` carries the adopted
# flag; `mcts-xoutoff-256` is the same search without it.
cd /home/archuser/repos/Crabomination
f=.ladder/r76/mcts_xout_vs_off_g43.txt
if [ -s "$f" ] && grep -q "A win%" "$f"; then echo "have $f"; exit 0; fi
./target/release-fast/bot_ladder --a mcts-dflt-256 --b mcts-xoutoff-256 --decks sealed --games 500 --seed 43 --threads 22 > "$f" 2>&1
echo "exit: $?" >> "$f"
