#!/usr/bin/env bash
# Round 28: encoder v6 — combat structure + keyword classes + exile
# counts, trained under the champion regime. First representation change
# measured at a scale where gates resolve: rounds 11/12 tested feature
# blocks at 90 k games and sub-0.015 AUC deltas; the champion regime
# (250 k games, cosine, 500 k window) moves gates by whole points.
# Control: round 20 (AUC 0.8070/0.8090, pooled 51.8% gang / 54.4%
# atk-sim). Both new blocks on; ablate combat/kw separately only if the
# full encoder shows signal.
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder

for tseed in 43 97; do
  out="nets_r28_s$tseed"
  $TRAIN --attn --lambda 0.7 --seed "$tseed" \
      --games 250000 --steps 200000 --window 500000 --lr 1e-4 --lr-cosine 60000 \
      --relabel-mode new --stop-after-stale 12 \
      --out "$out" > ".ladder/r28_train_s$tseed.log" 2>&1
  echo "train seed $tseed done"
  $TRAIN --calibrate 500 --attn --use-best "$out/best.safetensors" \
      > ".ladder/r28_calib_s$tseed.txt" 2>&1
  echo "calib seed $tseed done"
done

for tseed in 43 97; do
  export CRAB_NET="nets_r28_s$tseed/best.safetensors"
  for gseed in 43 97; do
    $LADDER --a net --b gang --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r28_s${tseed}_net_gang_s$gseed.txt" 2>&1
    $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r28_s${tseed}_net_atksim_s$gseed.txt" 2>&1
  done
  echo "gates seed $tseed done"
done
echo "round-28 first pass complete"
