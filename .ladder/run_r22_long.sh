#!/usr/bin/env bash
# Round 22b: the unexplored regime direction — longer cosine horizon
# (60k -> 90k) paired with more games (250k -> 400k), champion config
# otherwise. Control: round 20 (AUC 0.8070/0.8090, pooled 51.8% gang /
# 54.4% atk-sim).
set -eu
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder

for tseed in 43 97; do
  out="nets_r22_s$tseed"
  $TRAIN --attn --lambda 0.7 --seed "$tseed" --games 400000 --steps 200000 \
      --window 500000 --lr 1e-4 --lr-cosine 90000 \
      --relabel-mode new --stop-after-stale 12 \
      --out "$out" > ".ladder/r22_train_s$tseed.log" 2>&1
  echo "train seed $tseed done"
  $TRAIN --calibrate 500 --attn --use-best "$out/best.safetensors" \
      > ".ladder/r22_calib_s$tseed.txt" 2>&1
done

for tseed in 43 97; do
  export CRAB_NET="nets_r22_s$tseed/best.safetensors"
  for gseed in 43 97; do
    $LADDER --a net --b gang --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r22_s${tseed}_net_gang_s$gseed.txt" 2>&1
    $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r22_s${tseed}_net_atksim_s$gseed.txt" 2>&1
  done
  echo "gates seed $tseed done"
done
echo "round-22b long-run complete"
