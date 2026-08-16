#!/usr/bin/env bash
# Round 14: close the self-play loop. The label self-consistency candidate
# says the heuristic wins gates because gate outcomes are its own policy's
# fixed point. Test: train a fresh net on games piloted BY the net
# (--use-best), then gate the result as a pilot. Config identical to
# nets_r12_full (the control) except who pilots the training games.
set -eu
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder
PILOT=nets_r12_full/best.safetensors

$TRAIN --attn --lambda 0.7 --seed 43 --games 90000 --steps 60000 \
    --relabel-mode new --stop-after-stale 5 \
    --use-best $PILOT --out nets_r14_selfplay > .ladder/r14_train.log 2>&1
echo "train done"

$TRAIN --calibrate 500 --attn --use-best nets_r14_selfplay/best.safetensors \
    > .ladder/r14_calib.txt 2>&1
echo "calib done"

export CRAB_NET=nets_r14_selfplay/best.safetensors
for seed in 43 97; do
  $LADDER --a net --b gang --decks sealed --games 100 --seed "$seed" \
      > ".ladder/r14_net_gang_s$seed.txt" 2>&1
  $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$seed" \
      > ".ladder/r14_net_atksim_s$seed.txt" 2>&1
  echo "gates seed $seed done"
done
echo "round-14 batch complete"
