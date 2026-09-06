#!/usr/bin/env bash
# Search rerun under the round-66 default (trick modes held for combat): the greedy loop
# from converge4 with the charms back in the pool, then the three-way confirmation.
set -u
cd /home/archuser/repos/Crabomination
W=.ladder/deckwork; R2=$W/r2; G=./target/release-fast/deck_gauntlet
echo "=== r2 start $(date)"
# Baselines on this binary: converge3 (has Quandrix Charm) and converge4.
for d in decks/real_build_converge3.txt decks/real_build_converge4.txt; do for s in 43 97; do
  $G --deck $d --pairs 500 --seed $s --threads 20 --out $R2/results.txt 2>/dev/null | grep "^RESULT" | sed 's/field=[^ ]* //'
done; done
python3 scripts/deck_search.py --base decks/real_build_converge4.txt --pool decks/sealed_pool.txt \
  --cards $W/cards.json --work $R2 --gens 3 --pairs 500 --threads 20 --tag r2
echo "SEARCH EXIT $?"
F=$R2/final.txt
echo "--- confirmation: held-out field"
for d in $F decks/real_build_converge4.txt decks/real_build_converge3.txt; do for s in 43 97; do
  $G --deck $d --field-seed 0xDECC1000 --pairs 500 --seed $s --threads 20 --out $R2/results.txt 2>/dev/null | grep "^RESULT" | sed 's/field=[^ ]* //'
done; done
echo "--- confirmation: mcts256 both seats, replays"
rm -rf target/deckwork/replays/r2_final_search
$G --deck $F --pairs 25 --seed 43 --threads 12 --pilot mcts256 --opp-pilot mcts256 \
   --replays target/deckwork/replays/r2_final_search --out $R2/games_final_search.txt 2>/dev/null | grep -E "^RESULT|deck win%"
grep "^RESULT" $R2/games_final_search.txt >> $R2/results.txt
echo "--- confirmation: duels"
./target/release-fast/deck_duel $F decks/real_build_converge4.txt 2000 7 > $R2/duel_final_vs_converge4.txt 2>&1; grep -E "A win%" $R2/duel_final_vs_converge4.txt
./target/release-fast/deck_duel $F decks/real_build_converge3.txt 2000 7 > $R2/duel_final_vs_converge3.txt 2>&1; grep -E "A win%" $R2/duel_final_vs_converge3.txt
python3 scripts/replay_scan.py target/deckwork/replays/r2_final_search --deck-prefix final > $R2/scan_final_search.txt 2>&1
echo "=== r2 done $(date)"
