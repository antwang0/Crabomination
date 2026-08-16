#!/usr/bin/env bash
# Round-11 ablation. The library group, the castability block and a
# vocabulary change (153 -> 164 cards) all landed at once, and the full
# encoder scores worse on --calibrate than last session's net did. Four
# runs, identical but for which feature block is switched off, separate
# the three causes. `neither` is the old encoder on the new vocabulary,
# which is the control the vocab change destroyed.
#
# Scored on best.safetensors (peak held-out AUC), not latest: these runs
# overfit hard past ~step 6000 and the last checkpoint is the worst one.
set -eu
BIN=./target/release/selfplay_train
COMMON="--attn --lambda 0.7 --seed 43 --games 90000 --steps 60000"

run() { # name, ablate-args...
  local name=$1; shift
  $BIN $COMMON --out "nets_ab_$name" "$@" > ".ladder/ab_$name.log" 2>&1
  $BIN --calibrate 500 --attn --use-best "nets_ab_$name/best.safetensors" "$@" \
       > ".ladder/ab_${name}_calib.txt" 2>&1
  echo "$name done"
}

run full
run nolib   --ablate lib
run nocast  --ablate cast
run neither --ablate lib,cast
echo "ablation batch complete"
