#!/usr/bin/env bash
# Round 35: distillation at scale — train the net on what MCTS concluded.
#
# Round 33: the net can absorb a policy target (0.35 -> 0.81 on the
# heuristic's picks). Round 34: its own ranking of immediate successors
# agrees with MCTS *below chance* (0.67-0.79x lift, ten checkpoints, two
# seeds). So there is a large, consistent, learnable gap between what the
# evaluator likes now and what search concludes — which is exactly the
# target distillation is for, and the policy-improvement operator the
# twice-null stacking result has been missing since round 14.
#
# Arms (two seeds each):
#   control  --policy-every 0   record + measure, no policy gradient
#   distil   --policy-every 4   soft targets from the search's arm values
#
# `--policy-temp 0.1`: arm values are win probabilities, so their spread
# is small and the temperature is what makes the target a ranking rather
# than mush.
#
# SCALE. MCTS actors run ~2 games/s against heuristic actors' ~235, so
# game count is the wrong budget here — decisions are. The round-34 probe
# recorded ~20 decisions/game, so 20k games is ~400k decisions per cell,
# a substantial policy dataset, at roughly 2.5-3 h per cell.
#
# Read `val_policy` against `val_policy_chance` (both logged), and
# `pilot_policy` as the reference: it is the same holdout scored by the
# very net MCTS rolled out on, so distil > pilot_policy means the net has
# learned something its own search's evaluator did not have.
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train
PILOT=nets_r33_control_s43/best.safetensors

[ -f "$PILOT" ] || { echo "missing $PILOT" >&2; exit 1; }

for seed in 43 97; do
  for arm in control distil; do
    out="nets_r35_${arm}_s${seed}"
    every=0
    [ "$arm" = "distil" ] && every=4
    rm -rf "$out"; mkdir -p "$out"
    $TRAIN --attn --lambda 0.7 --seed "$seed" \
        --use-best "$PILOT" --mcts-actors 64 --gpu-eval \
        --record-decisions --policy-every "$every" --policy-temp 0.1 \
        --games 20000 --window 500000 --lr 1e-4 --lr-cosine 60000 \
        --relabel-mode new --out "$out" \
        > "$out/log.txt" 2>&1
    grep -q "learner device: cuda" "$out/log.txt" || {
        echo "ABORT: $out is not on cuda" >&2; head -3 "$out/log.txt" >&2; exit 1; }
    grep -q "pilot baseline scored" "$out/log.txt" || {
        echo "ABORT: $out has no pilot reference; the result would be uninterpretable" >&2
        exit 1; }
    echo "$out done"
  done
done
echo "round-35 distillation complete"
