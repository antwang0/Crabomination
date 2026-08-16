#!/usr/bin/env bash
# Round 18 resume: the first attempt was SIGTERMed 11 minutes into seed
# 97's training. Seed 43 completed (train + calib, nets_r18_s43); this
# picks up from seed 97 and then runs the full gate battery for both.
set -eu
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder

# --gpu-eval: batched actor inference (112 vs 51.5 games/s learner-idle;
# the gain is smaller while the learner is active — see ML_NOTES).
out="nets_r18_s97"
$TRAIN --blocks 2 --lambda 0.7 --seed 97 --games 250000 --steps 200000 \
    --window 500000 --lr 1e-4 \
    --relabel-mode new --stop-after-stale 12 \
    --use-best "nets_r17_s97/best.safetensors" \
    --gpu-eval --actors 512 --eval-batch 256 --eval-flush-us 200 \
    --out "$out" > ".ladder/r18_train_s97.log" 2>&1
echo "train seed 97 done"
$TRAIN --calibrate 500 --blocks 2 --use-best "$out/best.safetensors" \
    > ".ladder/r18_calib_s97.txt" 2>&1
echo "calib seed 97 done"

for tseed in 43 97; do
  export CRAB_NET="nets_r18_s$tseed/best.safetensors"
  for gseed in 43 97; do
    $LADDER --a net --b gang --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r18_s${tseed}_net_gang_s$gseed.txt" 2>&1
    $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r18_s${tseed}_net_atksim_s$gseed.txt" 2>&1
    echo "gates train-seed $tseed ladder-seed $gseed done"
  done
done
echo "round-18 resume complete"
