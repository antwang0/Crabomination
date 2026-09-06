#!/usr/bin/env bash
# Once the 46-card search logs its generation-1 verdict, stop it and start the search from converge4.
set -u
cd /home/archuser/repos/Crabomination
R3=.ladder/deckwork/r3
until grep -q "gen 1 best" $R3/search/search.log 2>/dev/null; do sleep 15; done
echo "gen 1 verdict logged $(date); stopping the 46-card search"
for pid in $(pgrep -f "r3/run_r3_search.sh"); do kill $pid; done
for pid in $(pgrep -f "deck_search.py --base decks/real_build_converge3.txt"); do kill $pid; done
sleep 2
for pid in $(pgrep -x deck_gauntlet); do kill $pid; done
sleep 2
setsid nohup $R3/run_r3b_search.sh > $R3/run_r3b_search.log 2>&1 &
echo "r3b launched $(date)"
