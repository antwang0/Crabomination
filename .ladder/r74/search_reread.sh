#!/usr/bin/env bash
# Round 74 (2026-09-14): the net leaf inside the 256-search, re-read on the
# CURRENT default (rounds 70/71 moved dflt after round 69's cells). Round 69:
# mcts-net67-256 vs mcts-dflt-256 with nets/champion.safetensors = 53.0 / 55.5
# (pooled +4.25). PRE-REGISTERED: pooled > +2.0 -> adopt the net leaf in the
# lobby search (lobby.rs); within ±2 -> re-argue; the cells are the reading.
set -u
cd /home/archuser/repos/Crabomination
LADDER=./target/release-fast/bot_ladder
for g in 43 97; do
  f=.ladder/r74/search_champion_g$g.txt
  if [ -s "$f" ] && grep -q "A win%" "$f"; then continue; fi
  echo "=== cell g$g start $(date)"
  CRAB_NET=nets/champion.safetensors $LADDER --a mcts-net67-256 --b mcts-dflt-256 --decks sealed --games 500 --seed $g --threads 22 > "$f" 2>&1
  echo "  g$g: $(grep -E 'A win%' "$f" | tail -1 | tr -s ' ')"
done
echo "=== r74 done $(date)"
