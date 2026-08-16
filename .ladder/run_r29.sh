#!/usr/bin/env bash
# Round 29: MCTS internals, one knob at a time, measured head-to-head
# against the mcts-net-deep control (64 iters / horizon 3 / c 1.0 / no
# priors / fixed budget) that earned the round-26 adoption. Direct A-vs-B
# paired cells rather than vs-gang: the contrast under test is the knob,
# not the pilot. Champion net on both sides via the ladder fallback.
# (a) exploration constant: c05 / c14 / c20 (control is c 1.0)
# (b) root priors from candidate scores, P-UCT w=1.5
# (c) adaptive budget: early-stop decided roots + 4x close-call extension
#     — for this one games/s is part of the result (cost is the claim).
set -eu
cd /home/archuser/repos/Crabomination
LADDER=./target/release-fast/bot_ladder

for arm in c05 c14 c20 prior adapt; do
  for gseed in 43 97; do
    $LADDER --a "mcts-net-$arm" --b mcts-net-deep --decks sealed \
        --games 100 --seed "$gseed" \
        > ".ladder/r29_${arm}_vs_deep_s$gseed.txt" 2>&1
  done
  echo "arm $arm done"
done
echo "round-29 gates complete"
