#!/usr/bin/env bash
# Wait for gate3's control search cell, stop gate3 before its allfix cell (stale binary),
# rebuild, and launch gate3b.
set -u
cd /home/archuser/repos/Crabomination
R3=.ladder/deckwork/r3
until grep -q "^RESULT" $R3/games_c3_mcts256.txt 2>/dev/null; do sleep 15; done
echo "control cell done $(date)"
for pid in $(pgrep -f "deckwork/r3/gate3.sh"); do kill $pid; done
sleep 2
for pid in $(pgrep -f "mcts256-allfix"); do kill $pid; done
sleep 3
echo "gate3 stopped $(date); building"
cargo build --profile release-fast -p crabomination --bin deck_gauntlet --bin bot_ladder --bin deck_duel 2>&1 | tail -2
ls -la --time-style=+%H:%M:%S target/release-fast/deck_gauntlet target/release-fast/bot_ladder
setsid nohup $R3/gate3b.sh > $R3/gate3b.log 2>&1 &
echo "gate3b launched $(date)"
