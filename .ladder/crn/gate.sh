#!/usr/bin/env bash
# CRN gate (pre-registered 2026-11-08): the client's menu search (main+combat,
# gen-3 priors, champion leaf, 8 arms, 256 rollouts) with common random
# numbers in the halving (-crn) vs the same search without, SOS sealed,
# paired 120 x 12, seeds 43 and 97. "Moved" = pooled >= +1.0 with both seeds
# above 50. Census behind it: .ladder/crn/census_s{43,97}.txt.
set -u
cd "$(dirname "$0")/../.."
bl=.ladder/crn/bot_ladder_crn
for s in 43 97; do
  CRAB_POLICY_NET=nets/policy-champion.safetensors $bl --a mcts-net67-menu-mc-crn --b mcts-net67-menu-mc \
    --games 120 --decks sealed --threads 20 --seed $s > .ladder/crn/gate_s$s.txt 2>&1
  echo "s$s: $(grep -E 'A win%' .ladder/crn/gate_s$s.txt | tail -1)"
done
