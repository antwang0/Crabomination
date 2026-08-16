#!/usr/bin/env bash
# Round 19 stage 1: Muon lr probe. Muon's orthogonalized update has
# RMS ≈ 1, so the champion regime's lr 1e-4 says nothing about the right
# Muon step size — probe 0.05/0.02/0.005 single-seed at 90k games, plus
# one probe of a faster AdamW side (3e-4) since Muon setups typically run
# their Adam params hotter than 1e-4. Winner by best holdout AUC goes to
# the two-seed full-regime run (stage 2).
set -eu
TRAIN=./target/release/selfplay_train

for mlr in 0.05 0.02 0.005; do
  $TRAIN --attn --muon --muon-lr "$mlr" --lambda 0.7 --seed 43 \
      --games 90000 --steps 200000 --window 500000 --lr 1e-4 \
      --relabel-mode new --stop-after-stale 12 \
      --out "nets_r19_probe_m$mlr" > ".ladder/r19_probe_m$mlr.log" 2>&1
  echo "probe muon-lr $mlr done"
done
$TRAIN --attn --muon --muon-lr 0.02 --lambda 0.7 --seed 43 \
    --games 90000 --steps 200000 --window 500000 --lr 3e-4 \
    --relabel-mode new --stop-after-stale 12 \
    --out "nets_r19_probe_adamw3e4" > ".ladder/r19_probe_adamw3e4.log" 2>&1
echo "probe adamw 3e-4 done"
echo "round-19 probes complete"
