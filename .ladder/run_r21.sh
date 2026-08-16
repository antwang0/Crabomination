#!/usr/bin/env bash
# Round 21: replay window 500k -> 1M on the round-20 champion (attn +
# new regime + cosine 60k). Control: round 20 itself (AUC 0.8070/0.8090,
# pooled gates 51.8% gang / 54.4% atk-sim).
set -eu
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder

for tseed in 43 97; do
  out="nets_r21_s$tseed"
  $TRAIN --attn --lambda 0.7 --seed "$tseed" --games 250000 --steps 200000 \
      --window 1000000 --lr 1e-4 --lr-cosine 60000 \
      --relabel-mode new --stop-after-stale 12 \
      --out "$out" > ".ladder/r21_train_s$tseed.log" 2>&1
  echo "train seed $tseed done"
  $TRAIN --calibrate 500 --attn --use-best "$out/best.safetensors" \
      > ".ladder/r21_calib_s$tseed.txt" 2>&1
done

for tseed in 43 97; do
  export CRAB_NET="nets_r21_s$tseed/best.safetensors"
  for gseed in 43 97; do
    $LADDER --a net --b gang --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r21_s${tseed}_net_gang_s$gseed.txt" 2>&1
    $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r21_s${tseed}_net_atksim_s$gseed.txt" 2>&1
  done
  echo "gates seed $tseed done"
done
echo "round-21 complete"
