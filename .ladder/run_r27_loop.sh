#!/usr/bin/env bash
# Round 27 track C: the search-amplification training loop, first pass.
# Arm A: 50k games piloted by MCTS-64 over the champion (batched eval,
# ~2.6 games/s measured) — labels from play ~3 points stronger than any
# prior generator. Arm B: 50k plain net-piloted games, same regime — the
# same-budget control that isolates label quality from volume.
# Seed 43 both arms; seed 97 replication only if there is signal.
set -eu
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder

$TRAIN --attn --lambda 0.7 --seed 43 --games 50000 --steps 200000 \
    --window 500000 --lr 1e-4 --lr-cosine 60000 \
    --relabel-mode new --stop-after-stale 12 \
    --use-best nets/champion.safetensors --gpu-eval --actors 256 \
    --mcts-actors 64 \
    --out nets_r27_mcts_s43 > .ladder/r27_loop_mcts_s43.log 2>&1
echo "arm A (mcts-piloted) done"

$TRAIN --attn --lambda 0.7 --seed 43 --games 50000 --steps 200000 \
    --window 500000 --lr 1e-4 --lr-cosine 60000 \
    --relabel-mode new --stop-after-stale 12 \
    --use-best nets/champion.safetensors --gpu-eval --actors 256 \
    --out nets_r27_ctrl_s43 > .ladder/r27_loop_ctrl_s43.log 2>&1
echo "arm B (net-piloted control) done"

for arm in mcts ctrl; do
  export CRAB_NET="nets_r27_${arm}_s43/best.safetensors"
  for gseed in 43 97; do
    $LADDER --a net --b gang --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r27_loop_${arm}_gang_s$gseed.txt" 2>&1
    $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r27_loop_${arm}_atksim_s$gseed.txt" 2>&1
  done
  echo "gates arm $arm done"
done
echo "round-27 loop first pass complete"
