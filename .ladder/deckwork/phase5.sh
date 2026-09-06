#!/usr/bin/env bash
# phase5.sh FINAL_LIST.txt [DECK_PILOT_FOR_SEARCH=mcts256] — final confirmation:
#  (a) held-out field (field seed 0xDECC1000) for FINAL and converge3, dflt, seeds 43/97;
#  (b) FINAL under the search pilot both seats, replays on (deck seat may carry a passed flag);
#  (c) deck_duel FINAL vs converge3, 2000 antithetic pairs;
#  (d) replay scan of (b).
set -u
cd /home/archuser/repos/Crabomination
W=.ladder/deckwork; G=./target/release-fast/deck_gauntlet; R=$W/results.txt
F=$1; P=${2:-mcts256}; N=$(basename "$F" .txt)
echo "=== phase5 $N start $(date)"
echo "--- (a) held-out field"
for d in "$F" decks/real_build_converge3.txt; do for s in 43 97; do
  $G --deck "$d" --field-seed 0xDECC1000 --pairs 500 --seed $s --threads 20 --out $R 2>/dev/null | grep -E "^RESULT"
done; done
echo "--- (b) $N under $P both seats, replays"
rm -rf target/deckwork/replays/${N}_search
$G --deck "$F" --pairs 25 --seed 43 --threads 12 --pilot $P --opp-pilot mcts256 \
   --replays target/deckwork/replays/${N}_search --out $W/games_${N}_search.txt 2>/dev/null | grep -E "^RESULT|deck win%"
grep "^RESULT" $W/games_${N}_search.txt >> $R
echo "--- (c) deck_duel $N vs converge3"
./target/release-fast/deck_duel "$F" decks/real_build_converge3.txt 2000 7 > $W/duel_${N}_vs_converge3.txt 2>&1
tail -8 $W/duel_${N}_vs_converge3.txt
echo "--- (d) scan"
python3 scripts/replay_scan.py target/deckwork/replays/${N}_search --deck-prefix "$N" > $W/scan_${N}_search.txt 2>&1
head -12 $W/scan_${N}_search.txt
echo "=== phase5 $N done $(date)"
