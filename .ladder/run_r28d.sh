#!/usr/bin/env bash
# Round 28d: repair the regime, not the hypothesis. 28b cleared the v6
# encoder; 28c cleared patience (stale never fires) and mostly cleared
# the determinize-era data (det0 bought ~0.015 AUC, not the missing
# 0.03). The binding stop was the fixed tail allowance: on this faster
# container the learner streams ~2-3k steps before the 250k games are
# done, then the reuse/2 tail (~5.9k steps) breaks the loop
# mid-improvement — every recent run died at ~8.4k steps with 3 of 4
# checkpoints still setting AUC bests. --tail-reuse 12 grants ~23k
# further tail steps and hands the stop back to stale-12 (24k stale
# steps at the 2000-step checkpoint cadence), which is champion-era
# training time. Seed 43, full v6 encoder. If AUC recovers toward
# 0.807+ and gates to the honest champion's 50.4-50.7/52.2, the
# pipeline is fixed and the real v6-vs-v5 verdict reruns under it.
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder

out="nets_r28d_s43"
$TRAIN --attn --lambda 0.7 --seed 43 \
    --games 250000 --steps 200000 --window 500000 --lr 1e-4 --lr-cosine 60000 \
    --relabel-mode new --stop-after-stale 12 --tail-reuse 12 \
    --out "$out" > .ladder/r28d_train.log 2>&1
echo "r28d trained"
$TRAIN --calibrate 500 --attn --use-best "$out/best.safetensors" \
    > .ladder/r28d_calib.txt 2>&1
export CRAB_NET="$out/best.safetensors"
for gseed in 43 97; do
  $LADDER --a net --b gang --decks sealed --games 100 --seed "$gseed" \
      > ".ladder/r28d_net_gang_s$gseed.txt" 2>&1
  $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$gseed" \
      > ".ladder/r28d_net_atksim_s$gseed.txt" 2>&1
done
echo "round-28d complete"
