#!/usr/bin/env bash
# 2026-09-06 targeting work, gates on the fresh release-fast binary:
#  (1) gauntlet: converge3 (has Arcane Omens, Traumatic Critique, Together as One, Divergent Equation)
#      and converge4 (Together as One, Vastlands/Bind to Life) under each flag, deck seat only, opponents dflt;
#  (2) sealed-ladder mirrors, A = flag vs B = dflt, seeds 43 / 97 (must not read < 49.5);
#  (3) the search pilot on converge3: mcts256-allfix vs mcts256, both seats, 1,200 games with replays.
set -u
cd /home/archuser/repos/Crabomination
G=./target/release-fast/deck_gauntlet; L=./target/release-fast/bot_ladder; R3=.ladder/deckwork/r3
echo "=== gate3 start $(date)"
for d in decks/real_build_converge3.txt decks/real_build_converge4.txt; do
  for p in dflt hostile parms targetfix x0skip gyfix allfix; do
    for s in 43 97; do
      $G --deck $d --pilot $p --pairs 500 --seed $s --threads 20 --out $R3/results.txt 2>/dev/null | grep "^RESULT" | sed 's/field=[^ ]* //'
    done
  done
done
echo "--- sealed ladder mirrors $(date)"
for a in hostile-targets player-arms target-fixes x0-skip gy-fixes all-fixes; do
  for s in 43 97; do
    $L --a $a --b dflt --decks sealed --games 500 --seed $s --threads 20 > $R3/ladder_${a}_s${s}.txt 2>&1
    echo "ladder $a s$s: $(grep -E 'A win%|paired:' $R3/ladder_${a}_s${s}.txt | tr '\n' ' ' | cut -c1-200)"
  done
done
echo "--- search pilot on converge3 $(date)"
for p in mcts256 mcts256-allfix; do
  rm -rf target/deckwork/replays/r3_c3_$p
  $G --deck decks/real_build_converge3.txt --pairs 25 --seed 43 --threads 12 --pilot $p --opp-pilot $p \
     --replays target/deckwork/replays/r3_c3_$p --out $R3/games_c3_$p.txt 2>/dev/null | grep -E "^RESULT"
  grep "^RESULT" $R3/games_c3_$p.txt >> $R3/results.txt
  python3 .ladder/deckwork/r3/whohit.py target/deckwork/replays/r3_c3_$p real_build_converge3 "Arcane Omens;Traumatic Critique;Together as One;Divergent Equation" | head -5
done
echo "=== gate3 done $(date)"
