#!/usr/bin/env bash
# Round 24: capacity under the regime that works. Round 12's width test
# ran under the old regime where overfitting bound everything; this is
# the fair retest: 2x representation width (emb 32->64, obj_hidden
# 64->128) on the champion config. Heuristic actors, so training
# throughput is unchanged — the treatment costs only learner FLOPs and
# gate speed. Control: round 20 (AUC 0.8070/0.8090, pooled 51.8% gang /
# 54.4% atk-sim).
set -eu
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder
WIDTH="--emb-dim 64 --obj-hidden 128"

for tseed in 43 97; do
  out="nets_r24_s$tseed"
  $TRAIN --attn $WIDTH --lambda 0.7 --seed "$tseed" \
      --games 250000 --steps 200000 --window 500000 --lr 1e-4 --lr-cosine 60000 \
      --relabel-mode new --stop-after-stale 12 \
      --out "$out" > ".ladder/r24_train_s$tseed.log" 2>&1
  echo "train seed $tseed done"
  $TRAIN --calibrate 500 --attn $WIDTH --use-best "$out/best.safetensors" \
      > ".ladder/r24_calib_s$tseed.txt" 2>&1
done

for tseed in 43 97; do
  export CRAB_NET="nets_r24_s$tseed/best.safetensors"
  for gseed in 43 97; do
    $LADDER --a net --b gang --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r24_s${tseed}_net_gang_s$gseed.txt" 2>&1
    $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r24_s${tseed}_net_atksim_s$gseed.txt" 2>&1
  done
  echo "gates seed $tseed done"
done
echo "round-24 complete"
