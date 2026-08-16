#!/usr/bin/env bash
# Round 14, generation 2: the compounding test. Generation 1 (training on
# games piloted by the r12 net) moved the replacement gate vs atk-sim
# from 47.6/48.5% to 51.4/50.4% — the first above-50 point estimates in
# the program. If label self-consistency is the mechanism, promoting the
# gen-1 net as pilot and retraining should move it again; if gen 2 reads
# like gen 1, the gain was a one-off distribution effect instead.
set -eu
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder
PILOT=nets_r14_selfplay/best.safetensors

$TRAIN --attn --lambda 0.7 --seed 43 --games 90000 --steps 60000 \
    --relabel-mode new --stop-after-stale 5 \
    --use-best $PILOT --out nets_r14b > .ladder/r14b_train.log 2>&1
echo "train done"

$TRAIN --calibrate 500 --attn --use-best nets_r14b/best.safetensors \
    > .ladder/r14b_calib.txt 2>&1
echo "calib done"

export CRAB_NET=nets_r14b/best.safetensors
for seed in 43 97; do
  $LADDER --a net --b gang --decks sealed --games 100 --seed "$seed" \
      > ".ladder/r14b_net_gang_s$seed.txt" 2>&1
  $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$seed" \
      > ".ladder/r14b_net_atksim_s$seed.txt" 2>&1
  echo "gates seed $seed done"
done
echo "generation-2 batch complete"
