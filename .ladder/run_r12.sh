#!/usr/bin/env bash
# Round 12: representation (relations + stack), aux responders, capacity —
# one change per run so each effect is attributable.
#
#   norel — v5 binary with the round-12 block ablated: the encoder
#           control, and (vs ab_full's stats.jsonl) the measurement of
#           what --stop-after-stale + --relabel-mode new bought.
#   full  — the v5 encoder: relation flags, special counters, stack groups.
#   aux   — full + the short-horizon aux head.
#   big   — full + double width (obj 128, trunk 768/384).
#
# All runs stop at 5 stale checkpoints and relabel incrementally; scored
# on best.safetensors via --calibrate with matching flags. The winner gets
# a seed-97 replication before anything is believed.
set -eu
BIN=./target/release/selfplay_train
COMMON="--attn --lambda 0.7 --seed 43 --games 90000 --steps 60000 --relabel-mode new --stop-after-stale 5"
CAP="--obj-hidden 128 --h1 768 --h2 384"

run() { # name, extra-args...
  local name=$1; shift
  $BIN $COMMON "$@" --out "nets_r12_$name" > ".ladder/r12_$name.log" 2>&1
  $BIN --calibrate 500 --attn --use-best "nets_r12_$name/best.safetensors" "$@" \
      > ".ladder/r12_${name}_calib.txt" 2>&1
  echo "$name done"
}

run norel --ablate rel
run full
run aux --aux
run big $CAP
echo "round-12 batch complete"
