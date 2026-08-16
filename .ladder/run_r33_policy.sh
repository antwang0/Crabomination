#!/usr/bin/env bash
# Round 33: the first policy training run.
#
# The net has only ever been taught to predict *who wins from here*. This
# teaches the same head to rank a decision's candidate successor states
# the way the pilot ranked them -- learning-to-rank on the existing
# evaluator, no new parameters. `val_policy` (held-out top-1 agreement)
# is the first metric in the program that measures choosing rather than
# predicting.
#
# Two arms, two seeds. The control RECORDS the same decisions and scores
# the same holdout but trains on none of them (--policy-every 0). An arm
# that simply omitted --record-decisions could not serve as the control:
# it reports no val_policy at all, so the treatment's number would have
# nothing to be compared against. Recording in both arms also equalises
# the recording cost, isolating the policy gradient.
#
# Screen scale (60k games), because round 32 established that AUC seed
# variance here is ~0.023 and a two-cell design cannot resolve less. The
# question this run can actually answer is whether `val_policy` rises
# above chance at all -- a much larger effect than the value metrics'
# noise floor, and the thing nobody has ever measured.
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train

for seed in 43 97; do
  for arm in control policy; do
    out="nets_r33_${arm}_s${seed}"
    flag="--record-decisions --policy-every 0"
    [ "$arm" = "policy" ] && flag="--record-decisions --policy-every 4"
    rm -rf "$out"; mkdir -p "$out"
    # shellcheck disable=SC2086
    $TRAIN --attn --lambda 0.7 --seed "$seed" $flag \
        --games 60000 --window 500000 --lr 1e-4 --lr-cosine 60000 \
        --relabel-mode new --stop-after-stale 12 --out "$out" \
        > "$out/log.txt" 2>&1
    if ! grep -q "learner device: cuda" "$out/log.txt"; then
        echo "ABORT: $out is not on cuda" >&2
        head -3 "$out/log.txt" >&2
        exit 1
    fi
    echo "$out done"
  done
done
echo "round-33 policy screen complete"
