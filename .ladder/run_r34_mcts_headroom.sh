#!/usr/bin/env bash
# Round 34, step 1: how far is the value net from MCTS's rankings?
#
# Round 33 measured the net against the *heuristic's* picks: 0.43 against
# a 0.354 chance rate. That caps out at the heuristic, so it says nothing
# about whether distilling a search would help. This measures the same
# agreement against MCTS's root decisions instead.
#
# Read it as headroom:
#   low agreement  -> the search wants something the evaluator does not,
#                     and distillation has real room.
#   high agreement -> MCTS and the value net already rank alike, and
#                     distilling buys little. That would be a genuine
#                     surprise and worth knowing before spending the
#                     compute on step 2.
#
# MEASURE ONLY (--policy-every 0): no policy gradient, so `val_policy`
# reports the untrained net's agreement rather than its ability to fit.
#
# --mcts-actors only engages when --use-best is set (see actor_loop), so
# a net is loaded and the pilot is MCTS-over-that-net. Games are far
# slower per decision than heuristic actors, hence the much smaller
# --games.
#
# NOT the champion, and the reason is a latent breakage worth recording.
# `nets/champion.safetensors` is a pre-encoder-v6 checkpoint (trunk1
# [512, 1060] against v6's [512, 1067] -- exactly the seven globals v6
# added). The *engine* loader zero-pads legacy checkpoints
# (`PlayNet::load`, round 28), but --gpu-eval builds a candle trainer and
# `VarMap::load` is a strict shape set, so it panics outright. Any
# `--gpu-eval --use-best nets/champion.safetensors` run has been broken
# since v6 landed.
#
# Using a v6-native net avoids the mismatch and is also the better
# science: a zero-padded champion would pilot with untrained columns for
# every v6 feature, which is a degraded player rather than the strong one
# the probe wants.
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train
PILOT=nets_r33_control_s43/best.safetensors

[ -f "$PILOT" ] || { echo "missing $PILOT" >&2; exit 1; }

for seed in 43 97; do
  out="nets_r34_mcts_s${seed}"
  rm -rf "$out"; mkdir -p "$out"
  $TRAIN --attn --lambda 0.7 --seed "$seed" \
      --use-best "$PILOT" --mcts-actors 64 --gpu-eval \
      --record-decisions --policy-every 0 \
      --games 4000 --window 200000 --lr 1e-4 \
      --relabel-mode new --out "$out" \
      > "$out/log.txt" 2>&1
  if ! grep -q "learner device: cuda" "$out/log.txt"; then
      echo "ABORT: $out is not on cuda" >&2
      head -3 "$out/log.txt" >&2
      exit 1
  fi
  if ! grep -q "MEASURING ONLY" "$out/log.txt"; then
      echo "ABORT: $out trained on the decisions; it is not a baseline" >&2
      exit 1
  fi
  echo "$out done"
done
echo "round-34 headroom probe complete"
