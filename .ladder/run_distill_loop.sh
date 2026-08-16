#!/usr/bin/env bash
# Distillation to convergence: iterate until the climb stops losing.
set -eu
TRAIN=./target/release/selfplay_train
PREV=nets_distill2
for it in 3 4 5 6 7; do
  DIR=nets_distill$it
  mkdir -p $DIR ${DIR}_gate
  cp $PREV/deck-distilled.safetensors $DIR/deck-latest.safetensors
  cp $PREV/deck_labels.bin $DIR/
  $TRAIN --distill-gen 200 --out $DIR --seed $((42+it)) > .ladder/distill${it}_gen.log 2>&1
  $TRAIN --distill-train --out $DIR > .ladder/distill${it}_train.log 2>&1
  cp $DIR/deck-distilled.safetensors ${DIR}_gate/deck-latest.safetensors
  $TRAIN --gate-builder-hc 400 --out ${DIR}_gate --seed 43 > .ladder/distill${it}_hc_gate.txt 2>&1
  PCT=$(grep "^TOTAL" .ladder/distill${it}_hc_gate.txt | grep -oE '= [0-9.]+' | cut -d' ' -f2)
  echo "iteration $it: climb gate $PCT%"
  PREV=$DIR
  if awk "BEGIN{exit !($PCT >= 48.0)}"; then echo "converged at iteration $it"; break; fi
done
$TRAIN --gate-builder 400 --out ${PREV}_gate --seed 43 > .ladder/distill_final_static.txt 2>&1
echo "loop done; final judge: $PREV/deck-distilled.safetensors"
