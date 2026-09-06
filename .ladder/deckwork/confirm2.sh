#!/usr/bin/env bash
# Waits for confirm1 (converge3 mcts256) to finish, then the same cell for v4_removal.
cd /home/archuser/repos/Crabomination
W=.ladder/deckwork
while pgrep -f "confirm1.sh" >/dev/null; do sleep 20; done
echo "=== confirm v4_removal mcts256 start $(date)"
./target/release-fast/deck_gauntlet --deck $W/variants/v4_removal.txt --pairs 25 --seed 43 --threads 12 --pilot mcts256 --opp-pilot mcts256 \
   --replays target/deckwork/replays/v4_removal_mcts256 --out $W/games_v4_removal_mcts256.txt
echo "GAUNTLET EXIT $?"
grep "^RESULT" $W/games_v4_removal_mcts256.txt >> $W/results.txt
python3 scripts/replay_scan.py target/deckwork/replays/v4_removal_mcts256 --deck-prefix v4_removal > $W/scan_v4_removal_mcts256.txt 2>&1
echo "=== confirm2 done $(date)"
