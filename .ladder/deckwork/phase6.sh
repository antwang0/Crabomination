#!/usr/bin/env bash
# Gate for the converge trio: deck seat on the gauntlet (converge4 for all four profiles,
# v4_removal for the trick flag since it plays both charms), dflt reference cells on the
# same binary, and sealed-ladder mirrors. 500 pairs x 24, seeds 43/97.
set -u
cd /home/archuser/repos/Crabomination
W=.ladder/deckwork; G=./target/release-fast/deck_gauntlet; R=$W/results6.txt
echo "=== phase6 start $(date)"
for s in 43 97; do
  for p in dflt convrarest convfetch trickmodes convfixes; do
    $G --deck decks/real_build_converge4.txt --pairs 500 --seed $s --threads 20 --pilot $p --out $R 2>/dev/null | grep -E "^RESULT" | sed 's/field=[^ ]* //'
  done
  for p in dflt trickmodes convfixes; do
    $G --deck $W/variants/v4_removal.txt --pairs 500 --seed $s --threads 20 --pilot $p --out $R 2>/dev/null | grep -E "^RESULT" | sed 's/field=[^ ]* //'
  done
done
echo "--- sealed ladder mirrors"
for p in conv-rarest conv-fetch trick-modes conv-fixes; do for s in 43 97; do
  ./target/release-fast/bot_ladder --a $p --b dflt --decks sealed --games 500 --seed $s --threads 20 > $W/ladder_${p}_s$s.txt 2>&1
  echo -n "$p s$s: "; grep -E "A win%" $W/ladder_${p}_s$s.txt | head -1
done; done
echo "--- summary"
python3 scripts/gauntlet_summary.py $R --vs real_build_converge4 | grep -v "^pilot" | sort -k7
echo "=== phase6 done $(date)"
