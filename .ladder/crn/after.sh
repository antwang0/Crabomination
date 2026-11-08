#!/usr/bin/env bash
# Runs after the CRN gate: the deck-lever gates, then AIVAT read 2 (a
# search-level cell: the CRN A/B itself at seed 11, 60 x 12, with the luck
# correction on — an extra CRN seed and the correction's ratio where pairs
# diverge most).
set -u
cd "$(dirname "$0")/../.."
while ! grep -q '^s97:' .ladder/crn/gate.log; do sleep 60; done
.ladder/crn/deck_gate.sh > .ladder/crn/deck_gate.log 2>&1
CRAB_AIVAT=1 CRAB_POLICY_NET=nets/policy-champion.safetensors .ladder/crn/bot_ladder_aivat --a mcts-net67-menu-mc-crn --b mcts-net67-menu-mc \
  --games 60 --decks sealed --threads 20 --seed 11 > .ladder/crn/aivat_search_s11.txt 2>&1
echo "after done" >> .ladder/crn/deck_gate.log
