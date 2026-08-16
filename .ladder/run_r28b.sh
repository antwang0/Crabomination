#!/usr/bin/env bash
# Round 28b: attribution control for the r28 regression. r28 (encoder v6,
# champion regime) came in ~2 points under the honest champion on both
# gates with the learner stopping at ~8k steps — but r28 vs r20 is
# confounded twice over: the determinize adoption changed the actors'
# data distribution between them, and this container generates ~40%
# faster, starving the learner relative to the window. This run is the
# same binary, same regime, same seeds, --ablate combat,kw — encoder v5
# feature-for-feature. If it matches r28, the new blocks are innocent
# and the regression belongs to the pipeline conditions; if it recovers
# the champion's numbers, the blocks genuinely hurt.
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train
LADDER=./target/release-fast/bot_ladder

for tseed in 43 97; do
  out="nets_r28b_s$tseed"
  $TRAIN --attn --lambda 0.7 --seed "$tseed" --ablate combat,kw \
      --games 250000 --steps 200000 --window 500000 --lr 1e-4 --lr-cosine 60000 \
      --relabel-mode new --stop-after-stale 12 \
      --out "$out" > ".ladder/r28b_train_s$tseed.log" 2>&1
  echo "train seed $tseed done"
  $TRAIN --calibrate 500 --attn --ablate combat,kw --use-best "$out/best.safetensors" \
      > ".ladder/r28b_calib_s$tseed.txt" 2>&1
done

# Gates: the ablated-trained net has never-trained weight columns for
# the ablated features, so the ladder must encode identically —
# CRAB_ABLATE mirrors the training run's --ablate.
export CRAB_ABLATE="combat,kw"
for tseed in 43 97; do
  export CRAB_NET="nets_r28b_s$tseed/best.safetensors"
  for gseed in 43 97; do
    $LADDER --a net --b gang --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r28b_s${tseed}_net_gang_s$gseed.txt" 2>&1
    $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$gseed" \
        > ".ladder/r28b_s${tseed}_net_atksim_s$gseed.txt" 2>&1
  done
  echo "gates seed $tseed done"
done
echo "round-28b attribution complete"
