#!/usr/bin/env bash
# Re-gate the arm-bearing profiles after the polarity-aware score term (2026-09-06 evening),
# then the search-pilot confirm on converge3: mcts256-allfix both seats, replays.
set -u
cd /home/archuser/repos/Crabomination
G=./target/release-fast/deck_gauntlet; L=./target/release-fast/bot_ladder; R3=.ladder/deckwork/r3
echo "=== gate3b start $(date)"
for d in decks/real_build_converge3.txt decks/real_build_converge4.txt; do
  for p in parms targetfix allfix; do
    for s in 43 97; do
      $G --deck $d --pilot $p --pairs 500 --seed $s --threads 20 --out $R3/results_b.txt 2>/dev/null | grep "^RESULT" | sed 's/field=[^ ]* //'
    done
  done
done
echo "--- sealed ladder mirrors $(date)"
for a in player-arms target-fixes all-fixes; do
  for s in 43 97; do
    $L --a $a --b dflt --decks sealed --games 500 --seed $s --threads 20 > $R3/ladder_b_${a}_s${s}.txt 2>&1
    echo "ladder $a s$s: $(grep -E 'A win%|paired:' $R3/ladder_b_${a}_s${s}.txt | tr '\n' ' ' | cut -c1-200)"
  done
done
echo "--- search pilot on converge3 $(date)"
p=mcts256-allfix
rm -rf target/deckwork/replays/r3_c3_$p
$G --deck decks/real_build_converge3.txt --pairs 25 --seed 43 --threads 12 --pilot $p --opp-pilot $p \
   --replays target/deckwork/replays/r3_c3_$p --out $R3/games_c3_$p.txt 2>/dev/null | grep -E "^RESULT"
grep "^RESULT" $R3/games_c3_$p.txt >> $R3/results_b.txt
python3 $R3/whohit.py target/deckwork/replays/r3_c3_$p real_build_converge3 "Arcane Omens;Traumatic Critique;Together as One;Divergent Equation" | head -5
echo "=== gate3b done $(date)"
