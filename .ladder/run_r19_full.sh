#!/usr/bin/env bash
# Round 19 stage 2: Muon at the probe-winning config (muon-lr 0.02,
# adamw side 3e-4 — the probes showed the muon lr flat across 10x and
# the adamw side starving at 1e-4), champion regime otherwise, two
# seeds, full gate battery. Control: r17b arm A (attn + new regime,
# AUC 0.8126/0.8069, pooled gates 51.4/52.9%).
set -eu
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder

for tseed in 43 97; do
  out="nets_r19_s$tseed"
  $TRAIN --attn --muon --muon-lr 0.02 --lambda 0.7 --seed "$tseed" \
      --games 250000 --steps 200000 --window 500000 --lr 3e-4 \
      --relabel-mode new --stop-after-stale 12 \
      --out "$out" > ".ladder/r19_train_s$tseed.log" 2>&1
  echo "train seed $tseed done"
  $TRAIN --calibrate 500 --attn --use-best "$out/best.safetensors" \
      > ".ladder/r19_calib_s$tseed.txt" 2>&1
done

for tseed in 43 97; do
  export CRAB_NET="nets_r19_s$tseed/best.safetensors"
  for gseed in 43 97; do
    $LADDER --a net --b gang --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r19_s${tseed}_net_gang_s$gseed.txt" 2>&1
    $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r19_s${tseed}_net_atksim_s$gseed.txt" 2>&1
  done
  echo "gates seed $tseed done"
done
echo "round-19 full complete"
