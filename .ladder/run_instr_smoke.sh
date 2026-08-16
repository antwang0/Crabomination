#!/usr/bin/env bash
# Instrumentation smoke: the checkpoint now scores both sides against both
# labels (raw 0/1 result and lambda-return). Two short runs so the two
# readings can be checked against each other:
#
#   lambda 1.0 -> the lambda-target IS the result, so train_tgt must equal
#                 train_raw and val_tgt must equal val_win, to the decimal.
#   lambda 0.7 -> they must separate, and train_tgt < train_raw is the
#                 bootstrap divergence we could not previously see.
#
# Short runs (heuristic actors, small window) -- this checks the numbers
# are wired and self-consistent, not that any net is good.
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train

for lam in 1.0 0.7; do
  out="nets_instr_l${lam}"
  rm -rf "$out"; mkdir -p "$out"
  $TRAIN --attn --lambda "$lam" --seed 43 \
      --games 3000 --window 50000 --checkpoint-every 500 \
      --relabel-mode new --out "$out" \
      > "$out/log.txt" 2>&1
  # The round-28 lesson: a trainer built without --features cuda silently
  # runs on CPU at a tenth the speed and every conclusion drawn from it is
  # about an undertrained net. Refuse to report numbers from such a run.
  if ! grep -q "learner device: cuda" "$out/log.txt"; then
      echo "ABORT: learner is not on cuda for lambda $lam" >&2
      head -3 "$out/log.txt" >&2
      exit 1
  fi
  echo "lambda $lam done"
done
echo "instrumentation smoke complete"
