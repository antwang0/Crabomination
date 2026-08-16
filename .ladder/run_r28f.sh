#!/usr/bin/env bash
# Round 28f: the round-28 experiment, run for real this time. ROOT CAUSE
# of the week's "regression": every trainer build this session omitted
# --features cuda, so the learner ran on CPU at 3-5 steps/s instead of
# the 4090's 42-53 — runs got 8-10k steps where the champion got 70k
# (nets_r20_s97/stats.jsonl: gen finished at step 66k, 162.8 games/s,
# best AUC 0.8090 at step 54k). Everything since r28 was undertrained;
# the container was never the problem and neither was the encoder.
#
# This is the fair test the round wanted: encoder v6 (full) vs
# v5-parity (--ablate combat,kw), two seeds each, champion regime, GPU
# learner, current-default det1 actors. Gates for the ablated arms run
# under CRAB_ABLATE so the net sees the features it trained on.
# Controls: champion re-gate 51.7/54.5 (today's binary, this week).
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder

grep -q "learner device: cuda" <($TRAIN --attn --games 1 --steps 1 --out /tmp/crab_gpu_check 2>&1) \
  || { echo "ABORT: learner is not on cuda — rebuild with --features cuda"; exit 1; }
echo "gpu check passed"

for enc in full v5; do
  ablate=""
  [ "$enc" = v5 ] && ablate="--ablate combat,kw"
  for tseed in 43 97; do
    out="nets_r28f_${enc}_s$tseed"
    $TRAIN --attn --lambda 0.7 --seed "$tseed" $ablate \
        --games 250000 --steps 200000 --window 500000 --lr 1e-4 --lr-cosine 60000 \
        --relabel-mode new --stop-after-stale 12 --tail-reuse 12 \
        --out "$out" > ".ladder/r28f_${enc}_train_s$tseed.log" 2>&1
    $TRAIN --calibrate 500 --attn $ablate --use-best "$out/best.safetensors" \
        > ".ladder/r28f_${enc}_calib_s$tseed.txt" 2>&1
    echo "$enc seed $tseed trained+calibrated"
  done
done

for enc in full v5; do
  if [ "$enc" = v5 ]; then export CRAB_ABLATE="combat,kw"; else unset CRAB_ABLATE; fi
  for tseed in 43 97; do
    export CRAB_NET="nets_r28f_${enc}_s$tseed/best.safetensors"
    for gseed in 43 97; do
      $LADDER --a net --b gang --decks sealed --games 100 --seed "$gseed" \
          > ".ladder/r28f_${enc}_s${tseed}_gang_s$gseed.txt" 2>&1
      $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$gseed" \
          > ".ladder/r28f_${enc}_s${tseed}_atksim_s$gseed.txt" 2>&1
    done
    echo "$enc seed $tseed gated"
  done
done
echo "round-28f complete"
