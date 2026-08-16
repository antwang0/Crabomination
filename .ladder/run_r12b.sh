#!/usr/bin/env bash
# Round-12 replication on seed 97: control, representation, capacity.
# aux showed the weakest signal on seed 43 and is not replicated.
set -eu
BIN=./target/release/selfplay_train
COMMON="--attn --lambda 0.7 --seed 97 --games 90000 --steps 60000 --relabel-mode new --stop-after-stale 5"
run() { local name=$1; shift
  $BIN $COMMON "$@" --out "nets_r12b_$name" > ".ladder/r12b_$name.log" 2>&1
  $BIN --calibrate 500 --attn --use-best "nets_r12b_$name/best.safetensors" "$@" \
      > ".ladder/r12b_${name}_calib.txt" 2>&1
  echo "$name done"
}
run norel --ablate rel
run full
run big --obj-hidden 128 --h1 768 --h2 384
echo "replication complete"
