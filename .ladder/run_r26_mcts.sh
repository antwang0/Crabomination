#!/usr/bin/env bash
# Round 26: the MCTS rematch. The historical verdict ("MCTS loses, and
# heuristic rollouts make it worse per unit cost") was earned with the
# heuristic reward; these profiles use the champion net's win
# probability as a native UCB1 reward with honest rollouts.
#
#   mcts-net vs net         does the bandit beat the decomposed searcher
#                           on the same evaluator?
#   mcts-net vs gang        absolute standing vs the heuristic reference
#   mcts-net-deep vs net    does depth (64 iters, 3-turn horizon) pay?
#
# Uses the committed champion via the CRAB_NET fallback.
set -eu
LADDER=./target/release-fast/bot_ladder

for gseed in 43 97; do
  $LADDER --a mcts-net --b net --decks sealed --games 100 --seed "$gseed" \
      > ".ladder/r26_mctsnet_net_s$gseed.txt" 2>&1
  $LADDER --a mcts-net --b gang --decks sealed --games 100 --seed "$gseed" \
      > ".ladder/r26_mctsnet_gang_s$gseed.txt" 2>&1
  $LADDER --a mcts-net-deep --b net --decks sealed --games 100 --seed "$gseed" \
      > ".ladder/r26_mctsnetdeep_net_s$gseed.txt" 2>&1
  echo "seed $gseed done"
done
echo "mcts-net gates complete"
