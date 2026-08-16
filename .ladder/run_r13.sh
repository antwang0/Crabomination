#!/usr/bin/env bash
# Round 13: the two survivors of rounds 11-12.
#
# 1. Sim-leaf calibration (--calibrate-leaves): does the net's predictive
#    edge survive on the simulated leaves the searches actually consume?
#    This is explanation 3 from the dead-end entry — the last proposed
#    cause left standing after the clean-checkpoint re-gate. Two seeds =
#    two independent game samples scored on the same checkpoint.
#
# 2. Ply-scheduled blend gate (netb-ply = blend300 with a turn taper:
#    full through turn 5, zero at turn 12). vs gang answers adoption;
#    vs net-blend300 isolates the taper at constant loudness. Two seeds.
#
# Net: nets_r12_full/best.safetensors — the full-v5-encoder checkpoint,
# matching what the ladder's encoder emits (a norel checkpoint would see
# relation features it never trained on).
set -eu
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder
NET=nets_r12_full/best.safetensors
export CRAB_NET=$NET

for seed in 43 97; do
  $TRAIN --calibrate-leaves 300 --attn --use-best $NET --seed "$seed" \
      > ".ladder/r13_leaves_s$seed.txt" 2>&1
  echo "leaves seed $seed done"
done

for seed in 43 97; do
  $LADDER --a netb-ply --b gang --decks sealed --games 100 --seed "$seed" \
      > ".ladder/r13_ply_gang_s$seed.txt" 2>&1
  $LADDER --a netb-ply --b net-blend300 --decks sealed --games 100 --seed "$seed" \
      > ".ladder/r13_ply_b300_s$seed.txt" 2>&1
  echo "ply gates seed $seed done"
done
echo "round-13 batch complete"
