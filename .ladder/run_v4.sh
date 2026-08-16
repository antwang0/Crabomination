#!/usr/bin/env bash
# Round 11: new encoder (library group + castability), plus the embedding
# transfer test. Two runs differing in exactly one thing — where the card
# embeddings start — on the same seed, so they see the same games.
set -eu
BIN=./target/release/selfplay_train
COMMON="--attn --lambda 0.7 --seed 43 --games 90000 --steps 60000"

# Control: cold embeddings. Also produces the deck net run B seeds from.
$BIN $COMMON --out nets_v4_ctrl > .ladder/v4_ctrl.log 2>&1

# Treatment: identical, except the card embeddings start from the control
# run's deck net.
$BIN $COMMON --out nets_v4_seed \
     --seed-emb nets_v4_ctrl/deck-latest.safetensors > .ladder/v4_seed.log 2>&1

# Ply-stratified calibration on both.
$BIN --calibrate 500 --attn --use-best nets_v4_ctrl/latest.safetensors \
     > .ladder/v4_ctrl_calib.txt 2>&1
$BIN --calibrate 500 --attn --use-best nets_v4_seed/latest.safetensors \
     > .ladder/v4_seed_calib.txt 2>&1
echo "v4 batch complete"
