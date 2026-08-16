#!/usr/bin/env bash
# Round 18: promote the round-17 nets as self-play pilots (the round-14
# gen-1 mechanism, the only quality lever besides round 17 itself that
# ever produced an above-50 gate) and retrain under the round-17 regime.
# Each training seed is piloted by its own seed's round-17 net so the two
# replications stay independent. Gates as in round 17: replacement vs
# gang and atk-sim, two ladder seeds, paired.
set -eu
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder

for tseed in 43 97; do
  out="nets_r18_s$tseed"
  $TRAIN --blocks 2 --lambda 0.7 --seed "$tseed" --games 250000 --steps 200000 \
      --window 500000 --lr 1e-4 \
      --relabel-mode new --stop-after-stale 12 \
      --use-best "nets_r17_s$tseed/best.safetensors" \
      --out "$out" > ".ladder/r18_train_s$tseed.log" 2>&1
  echo "train seed $tseed done"
  $TRAIN --calibrate 500 --blocks 2 --use-best "$out/best.safetensors" \
      > ".ladder/r18_calib_s$tseed.txt" 2>&1
  echo "calib seed $tseed done"
done

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
echo "round-18 batch complete"
