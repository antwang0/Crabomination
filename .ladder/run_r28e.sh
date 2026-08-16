#!/usr/bin/env bash
# Round 28e: reconstruct the champion recipe on today's code. The
# eliminations so far (all seed 43): v6 encoder innocent (28b), stale
# patience irrelevant (28c-P), det0 alone +1 pt at 8k steps (28c-D),
# training length alone nothing at 26k steps (28d — gates 47.3/48.7,
# worse if anything). The r20 champion trained on det0 actors AT full
# length; nothing since the container change has run that combination.
# This does: --actor-det 0 --tail-reuse 12, full v6 encoder, champion
# regime. Target: the honest champion's gates, 50.4-50.7 gang / 52.2
# atk-sim. Recover them and the regime is understood — det0 generation
# plus stale-governed length — and the two-seed v6-vs-v5 verdict reruns
# under it. Miss and the residue is in the merged code drift, which
# golden traces say is behaviour-preserving — a different hunt.
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder

out="nets_r28e_s43"
$TRAIN --attn --lambda 0.7 --seed 43 \
    --games 250000 --steps 200000 --window 500000 --lr 1e-4 --lr-cosine 60000 \
    --relabel-mode new --stop-after-stale 12 --tail-reuse 12 --actor-det 0 \
    --out "$out" > .ladder/r28e_train.log 2>&1
echo "r28e trained"
$TRAIN --calibrate 500 --attn --use-best "$out/best.safetensors" \
    > .ladder/r28e_calib.txt 2>&1
export CRAB_NET="$out/best.safetensors"
for gseed in 43 97; do
  $LADDER --a net --b gang --decks sealed --games 100 --seed "$gseed" \
      > ".ladder/r28e_net_gang_s$gseed.txt" 2>&1
  $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$gseed" \
      > ".ladder/r28e_net_atksim_s$gseed.txt" 2>&1
done
echo "round-28e complete"
