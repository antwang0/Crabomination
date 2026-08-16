#!/usr/bin/env bash
# Round 22a: blend gate on the round-20 champion nets. Blend historically
# beat replacement by ~1-2 points on weaker nets but has never been
# measured with a net whose replacement pilot is already above parity
# (r20: pooled 51.8% gang / 54.4% atk-sim). Both blend strengths, both
# training seeds, both ladder seeds.
set -eu
LADDER=./target/release-fast/bot_ladder

for tseed in 43 97; do
  export CRAB_NET="nets_r20_s$tseed/best.safetensors"
  for prof in net-blend net-blend300; do
    for gseed in 43 97; do
      $LADDER --a "$prof" --b gang --decks sealed --games 100 --seed "$gseed" \
          > ".ladder/r22_${prof}_s${tseed}_gang_s$gseed.txt" 2>&1
      $LADDER --a "$prof" --b atk-sim --decks sealed --games 100 --seed "$gseed" \
          > ".ladder/r22_${prof}_s${tseed}_atksim_s$gseed.txt" 2>&1
    done
    echo "$prof seed $tseed done"
  done
done
echo "round-22a blend gates complete"
