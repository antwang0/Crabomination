#!/usr/bin/env bash
# After the r3 fixes are adopted into the default and the binaries rebuilt:
# the greedy search from converge3 (the user's 46-card list) on the fixed pool,
# then the three-way confirmation of the final against converge3 and converge4.
set -u
cd /home/archuser/repos/Crabomination
W=.ladder/deckwork; R3=$W/r3; S=$R3/search; G=./target/release-fast/deck_gauntlet
mkdir -p $S
echo "=== r3 search start $(date)"
for d in decks/real_build_converge3.txt decks/real_build_converge4.txt; do for s in 43 97; do
  $G --deck $d --pairs 500 --seed $s --threads 20 --out $S/results.txt 2>/dev/null | grep "^RESULT" | sed 's/field=[^ ]* //'
done; done
python3 scripts/deck_search.py --base decks/real_build_converge3.txt --pool decks/sealed_pool.txt \
  --cards $W/cards.json --work $S --gens 6 --pairs 500 --threads 20 --tag r3
echo "SEARCH EXIT $?"
F=$S/final.txt
echo "--- confirmation: held-out field"
for d in $F decks/real_build_converge4.txt decks/real_build_converge3.txt; do for s in 43 97; do
  $G --deck $d --field-seed 0xDECC1000 --pairs 500 --seed $s --threads 20 --out $S/results.txt 2>/dev/null | grep "^RESULT" | sed 's/field=[^ ]* //'
done; done
echo "--- confirmation: mcts256 both seats, replays"
rm -rf target/deckwork/replays/r3_final_search
$G --deck $F --pairs 25 --seed 43 --threads 12 --pilot mcts256 --opp-pilot mcts256 \
   --replays target/deckwork/replays/r3_final_search --out $S/games_final_search.txt 2>/dev/null | grep -E "^RESULT"
grep "^RESULT" $S/games_final_search.txt >> $S/results.txt
echo "--- confirmation: duels"
./target/release-fast/deck_duel $F decks/real_build_converge4.txt 2000 7 > $S/duel_final_vs_converge4.txt 2>&1; grep -E "A win%" $S/duel_final_vs_converge4.txt
./target/release-fast/deck_duel $F decks/real_build_converge3.txt 2000 7 > $S/duel_final_vs_converge3.txt 2>&1; grep -E "A win%" $S/duel_final_vs_converge3.txt
python3 scripts/replay_scan.py target/deckwork/replays/r3_final_search --deck-prefix final > $S/scan_final_search.txt 2>&1
python3 $R3/whohit.py target/deckwork/replays/r3_final_search final "Arcane Omens;Traumatic Critique;Together as One;Divergent Equation;Bind to Life" > $S/whohit_final.txt 2>&1
echo "=== r3 search done $(date)"
