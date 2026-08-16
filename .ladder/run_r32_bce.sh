#!/usr/bin/env bash
# Round 32: cross-entropy vs MSE on the win head, two seeds.
#
# The claim is mechanistic: MSE through a sigmoid has gradient
# proportional to (p - y)*p*(1-p), so the rows where the net is
# confidently wrong -- the ones with the most to teach -- produce the
# weakest updates. BCE's gradient at the logit is (p - y).
#
# Screen scale, not champion scale: 60k games per cell rather than 250k,
# so this reads the *direction* on two seeds before spending three hours
# on the full gate. Recorded as a screen so nobody later quotes it as a
# champion-regime number.
#
# Compare on `val_auc` and on `train_raw` / `val_win`, which stay MSE in
# the checkpoint under both arms. `loss_win` is NOT comparable across
# arms -- it is the optimised objective and changes units with --bce.
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train

for seed in 43 97; do
  for arm in mse bce; do
    out="nets_r32_${arm}_s${seed}"
    flag=""
    [ "$arm" = "bce" ] && flag="--bce"
    rm -rf "$out"; mkdir -p "$out"
    # shellcheck disable=SC2086
    $TRAIN --attn --lambda 0.7 --seed "$seed" $flag \
        --games 60000 --window 500000 --lr 1e-4 --lr-cosine 60000 \
        --relabel-mode new --stop-after-stale 12 --out "$out" \
        > "$out/log.txt" 2>&1
    # Round 28's lesson: a trainer built without --features cuda runs on
    # CPU at a tenth the speed and every conclusion from it is about an
    # undertrained net.
    if ! grep -q "learner device: cuda" "$out/log.txt"; then
        echo "ABORT: $out is not on cuda" >&2
        head -3 "$out/log.txt" >&2
        exit 1
    fi
    echo "$out done"
  done
done
echo "round-32 screen complete"
