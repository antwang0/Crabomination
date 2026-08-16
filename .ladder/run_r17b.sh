#!/usr/bin/env bash
# Round 17b: attribution ablation. Round 17 changed four levers at once
# (blocks, window, lr, run length); these two arms separate architecture
# from optimization regime. Heuristic pilots, matching round 17's design.
#
#   Arm A "regime-only":  --attn (old architecture) under the NEW regime.
#   Arm B "blocks-only":  --blocks 2 under the OLD regime
#                         (250k window, lr 1e-3, 90k games, patience 5).
#
# If A reproduces the sweep, the regime did it; if B does, the blocks
# did; if neither, the combination is the treatment. Two training seeds
# per arm, gates vs gang and atk-sim on two ladder seeds.
set -eu
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder

for tseed in 43 97; do
  out="nets_r17b_attn_s$tseed"
  $TRAIN --attn --lambda 0.7 --seed "$tseed" --games 250000 --steps 200000 \
      --window 500000 --lr 1e-4 \
      --relabel-mode new --stop-after-stale 12 \
      --out "$out" > ".ladder/r17b_attn_train_s$tseed.log" 2>&1
  echo "arm A (regime-only) train seed $tseed done"
  $TRAIN --calibrate 500 --attn --use-best "$out/best.safetensors" \
      > ".ladder/r17b_attn_calib_s$tseed.txt" 2>&1

  out="nets_r17b_old_s$tseed"
  $TRAIN --blocks 2 --lambda 0.7 --seed "$tseed" --games 90000 --steps 60000 \
      --relabel-mode new --stop-after-stale 5 \
      --out "$out" > ".ladder/r17b_old_train_s$tseed.log" 2>&1
  echo "arm B (blocks-only) train seed $tseed done"
  $TRAIN --calibrate 500 --blocks 2 --use-best "$out/best.safetensors" \
      > ".ladder/r17b_old_calib_s$tseed.txt" 2>&1
done

for arm in attn old; do
  for tseed in 43 97; do
    export CRAB_NET="nets_r17b_${arm}_s$tseed/best.safetensors"
    for gseed in 43 97; do
      $LADDER --a net --b gang --decks sealed --games 100 --seed "$gseed" \
          > ".ladder/r17b_${arm}_s${tseed}_net_gang_s$gseed.txt" 2>&1
      $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$gseed" \
          > ".ladder/r17b_${arm}_s${tseed}_net_atksim_s$gseed.txt" 2>&1
    done
    echo "gates arm $arm train-seed $tseed done"
  done
done
echo "round-17b ablation complete"
