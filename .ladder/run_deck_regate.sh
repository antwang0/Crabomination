#!/usr/bin/env bash
# Re-gate the deck net on current-vocab run artifacts — closing the
# 2026-08-24 ML defect ("every committed deck net fails to load").
#
# All seven committed deck-latest checkpoints are vocab 153 against a live
# vocab of 164, so `--use-deck-best`, `--gate-builder-hc` and
# `--distill-gen` are dead. No retrain is needed: the r41/r45 run
# directories already hold deck nets trained at vocab 164 (the deck stream
# rides along in every selfplay_train run). This gates two of them —
# different rounds, both 250k-game runs — before committing one as
# nets/deck-champion.safetensors.
#
# Sizing matches the round-11 re-gate: 800 games x 12 pools = 9,600 games
# per cell, ladder seeds 43/97. Reference band: the four historical gates
# read 61.7 / 60.7 / 60.0 / 61.8 % vs the static judge. Pre-registered
# reading: a cell in the 60-62 band is the known deck-net level; above 50
# but below the band still revives the consumers (the net's role is
# best-of-32 judge, and the bar it must beat is the static judge at 50%);
# at or below 50, nothing is committed and the defect stays open.
set -eu
TRAIN=./target/release/selfplay_train

for out in nets_r45_ctrl_s43 nets_r41_v7_s43; do
  for seed in 43 97; do
    $TRAIN --gate-builder 800 --out "$out" --seed "$seed" \
        > ".ladder/deck_regate_${out}_s${seed}.txt" 2>&1
    echo "gate $out seed $seed done:"
    tail -2 ".ladder/deck_regate_${out}_s${seed}.txt"
  done
done
