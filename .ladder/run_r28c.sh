#!/usr/bin/env bash
# Round 28c: attribute the training regression (r28 ≡ r28b, both ~2 pts
# under the honest champion, learner stalling at ~8k steps). Two
# hypotheses, one arm each, seed 43, full v6 encoder:
#
# Arm P (patience): stale-48 instead of 12. If the stop condition is
#   firing before the cosine anneal does its work, longer patience
#   recovers AUC/gates and the regime just needs retuning for this box.
#
# Arm D (data): --actor-det 0 restores the pre-r25 generator (peeking
#   sims). The r25 adoption changed the *default* pilot for honesty at
#   play — and silently swept the training-data generator with it.
#   Honest pilots are ~1-1.5 pts weaker and their sims noisier; if the
#   labels got noisier the achievable AUC drops and the plateau comes
#   early. Peeking was never a problem for label generation.
#
# Controls: r28 (this pipeline, stale 12, det 1) at AUC 0.775, gates
# 48.3/50.1; the honest champion at 50.4-50.7/52.2.
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder

run_arm() {
  arm="$1"; shift
  out="nets_r28c_$arm"
  $TRAIN --attn --lambda 0.7 --seed 43 \
      --games 250000 --steps 200000 --window 500000 --lr 1e-4 --lr-cosine 60000 \
      --relabel-mode new "$@" \
      --out "$out" > ".ladder/r28c_${arm}_train.log" 2>&1
  echo "arm $arm trained"
  $TRAIN --calibrate 500 --attn --use-best "$out/best.safetensors" \
      > ".ladder/r28c_${arm}_calib.txt" 2>&1
  export CRAB_NET="$out/best.safetensors"
  for gseed in 43 97; do
    $LADDER --a net --b gang --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r28c_${arm}_net_gang_s$gseed.txt" 2>&1
    $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r28c_${arm}_net_atksim_s$gseed.txt" 2>&1
  done
  echo "arm $arm gated"
}

run_arm patience --stop-after-stale 48
run_arm det0 --stop-after-stale 12 --actor-det 0
echo "round-28c attribution complete"
