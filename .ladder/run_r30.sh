#!/usr/bin/env bash
# Round 30: champion re-baseline sweep, part 1 — the det0 arms. With the
# GPU learner restored (r28f), the healthy-regime det1 nets exist at
# seeds 43/97 (nets_r28f_full_*: 51.6/53.2 pooled). These two runs add
# --actor-det 0 (the pre-r25 generator the champion trained on) at the
# same seeds. Together: a 2x2 data-x-seed grid, all healthy, all v6.
# Questions, in order: (1) does det0 data still buy anything at full
# training (it read +1 pt on crippled CPU runs — unverified since);
# (2) does any cell beat the committed champion's 51.7/54.5, i.e. was
# the r20-s97 champion a high draw or a real standout. Best-of-grid
# becomes the champion candidate only if it clears the incumbent.
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder

grep -q "learner device: cuda" <($TRAIN --attn --games 1 --steps 1 --out /tmp/crab_gpu_check 2>&1) \
  || { echo "ABORT: learner is not on cuda — rebuild with --features cuda"; exit 1; }
echo "gpu check passed"

for tseed in 43 97; do
  out="nets_r30_det0_s$tseed"
  $TRAIN --attn --lambda 0.7 --seed "$tseed" --actor-det 0 \
      --games 250000 --steps 200000 --window 500000 --lr 1e-4 --lr-cosine 60000 \
      --relabel-mode new --stop-after-stale 12 --tail-reuse 12 \
      --out "$out" > ".ladder/r30_det0_train_s$tseed.log" 2>&1
  $TRAIN --calibrate 500 --attn --use-best "$out/best.safetensors" \
      > ".ladder/r30_det0_calib_s$tseed.txt" 2>&1
  echo "det0 seed $tseed trained+calibrated"
done

for tseed in 43 97; do
  export CRAB_NET="nets_r30_det0_s$tseed/best.safetensors"
  for gseed in 43 97; do
    $LADDER --a net --b gang --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r30_det0_s${tseed}_gang_s$gseed.txt" 2>&1
    $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r30_det0_s${tseed}_atksim_s$gseed.txt" 2>&1
  done
  echo "det0 seed $tseed gated"
done
echo "round-30 det0 arms complete"
