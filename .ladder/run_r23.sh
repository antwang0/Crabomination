#!/usr/bin/env bash
# Round 23: actor-side softmax action sampling (--sample-temp 120 = one
# mid-size creature on the heuristic scale, through turn 6) on the
# champion config with cosine decay. Exploration is data-generation only;
# gates and calibration stay argmax by construction (thread-local).
# Control: round 20 (AUC 0.8070/0.8090, pooled 51.8% gang / 54.4%
# atk-sim) — or round 22 if it beat round 20.
set -eu
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder

for tseed in 43 97; do
  out="nets_r23_s$tseed"
  $TRAIN --attn --lambda 0.7 --seed "$tseed" --games 250000 --steps 200000 \
      --window 500000 --lr 1e-4 --lr-cosine 60000 \
      --sample-temp 120 --sample-turns 6 \
      --relabel-mode new --stop-after-stale 12 \
      --out "$out" > ".ladder/r23_train_s$tseed.log" 2>&1
  echo "train seed $tseed done"
  $TRAIN --calibrate 500 --attn --use-best "$out/best.safetensors" \
      > ".ladder/r23_calib_s$tseed.txt" 2>&1
done

for tseed in 43 97; do
  export CRAB_NET="nets_r23_s$tseed/best.safetensors"
  for gseed in 43 97; do
    $LADDER --a net --b gang --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r23_s${tseed}_net_gang_s$gseed.txt" 2>&1
    $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r23_s${tseed}_net_atksim_s$gseed.txt" 2>&1
  done
  echo "gates seed $tseed done"
done
echo "round-23 complete"
