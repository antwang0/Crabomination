#!/usr/bin/env bash
cd /home/archuser/repos/Crabomination
W=.ladder/deckwork
echo "=== confirm converge3 mcts256 start $(date)"
./target/release-fast/deck_gauntlet --deck decks/real_build_converge3.txt --pairs 25 --seed 43 --threads 12 --pilot mcts256 --opp-pilot mcts256 \
   --replays target/deckwork/replays/converge3_mcts256 --out $W/games_converge3_mcts256.txt
echo "GAUNTLET EXIT $?"
grep "^RESULT" $W/games_converge3_mcts256.txt >> $W/results.txt
python3 scripts/replay_scan.py target/deckwork/replays/converge3_mcts256 --deck-prefix real_build_converge3 > $W/scan_converge3_mcts256.txt 2>&1
echo "=== confirm done $(date)"
