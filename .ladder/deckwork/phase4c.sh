#!/usr/bin/env bash
# After phase4b: the own-graveyard-pick flag (gypick) and both flags together, deck seat on the
# gauntlet (inc1, seeds 43/97) and gy-pick on the sealed ladder mirror.
set -u
cd /home/archuser/repos/Crabomination
W=.ladder/deckwork; G=./target/release-fast/deck_gauntlet; R=$W/results.txt
while pgrep -f "phase4b.sh" >/dev/null; do sleep 15; done
echo "=== phase4c start $(date)"
for p in gypick both; do for s in 43 97; do
  $G --deck $W/variants/inc1.txt --pairs 500 --seed $s --threads 20 --pilot $p --out $R 2>/dev/null | grep -E "^RESULT"
done; done
for s in 43 97; do
  ./target/release-fast/bot_ladder --a gy-pick --b dflt --decks sealed --games 500 --seed $s --threads 20 > $W/ladder_gypick_s$s.txt 2>&1
  grep -E "A win%|verdict" $W/ladder_gypick_s$s.txt
done
python3 scripts/gauntlet_summary.py $R --vs inc1 | grep -E "inc1|stunhold|gypick|both" | grep -v "12x1"
echo "=== phase4c done $(date)"
