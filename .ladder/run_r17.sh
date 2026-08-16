#!/usr/bin/env bash
# Round 17: proper transformer blocks. The single tagged-attention layer
# (round 6) and every capacity/feature treatment since (round 12) read
# NULL against the pooled control, but all of them trained short (early
# stop ~5 stale checks) at lr 1e-3 on a 250k window. This run changes the
# architecture *and* the optimization regime together, per request:
#
#   * --blocks 2      pre-LN transformer stack (tblocks.*), 4 heads, 2x FFN
#   * --window 500000 doubled replay window
#   * --lr 1e-4       10x lower, constant
#   * longer run:     250k games, 200k step cap, stale patience 12
#
# Two training seeds (round-12 rule: single-seed calibration deltas are
# unreadable), each gated as the replacement pilot vs gang and atk-sim on
# two ladder seeds with the paired ladder.
set -eu
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder

for tseed in 43 97; do
  out="nets_r17_s$tseed"
  $TRAIN --blocks 2 --lambda 0.7 --seed "$tseed" --games 250000 --steps 200000 \
      --window 500000 --lr 1e-4 \
      --relabel-mode new --stop-after-stale 12 \
      --out "$out" > ".ladder/r17_train_s$tseed.log" 2>&1
  echo "train seed $tseed done"
  $TRAIN --calibrate 500 --blocks 2 --use-best "$out/best.safetensors" \
      > ".ladder/r17_calib_s$tseed.txt" 2>&1
  echo "calib seed $tseed done"
done

for tseed in 43 97; do
  export CRAB_NET="nets_r17_s$tseed/best.safetensors"
  for gseed in 43 97; do
    $LADDER --a net --b gang --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r17_s${tseed}_net_gang_s$gseed.txt" 2>&1
    $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r17_s${tseed}_net_atksim_s$gseed.txt" 2>&1
    echo "gates train-seed $tseed ladder-seed $gseed done"
  done
done
echo "round-17 batch complete"
