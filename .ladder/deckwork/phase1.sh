#!/usr/bin/env bash
# Phase 0 smoke + Phase 1 baselines (see PLAN.md). Results accumulate in
# results.txt (RESULT lines); replays under target/deckwork/replays/.
set -u
cd /home/archuser/repos/Crabomination
W=.ladder/deckwork
G=./target/release-fast/deck_gauntlet
R=$W/results.txt
mkdir -p target/deckwork/replays
echo "=== phase1 start $(date)" | tee -a $W/phase1.log

step() { echo; echo "--- $* ($(date +%H:%M:%S))"; }

step "smoke fast"
$G --deck decks/real_build_converge3.txt --field 12x1 --pairs 2 --pilot dflt --threads 1 || { echo "SMOKE FAST FAILED"; exit 1; }

step "smoke server+replays (mcts64, 1 pair)"
rm -rf target/deckwork/replays/smoke
$G --deck decks/real_build_converge3.txt --field 12x1 --pairs 1 --pilot mcts64 --opp-pilot mcts64 --threads 1 \
   --replays target/deckwork/replays/smoke --out $W/smoke_games.txt || { echo "SMOKE SERVER FAILED"; exit 1; }
ls target/deckwork/replays/smoke | head; grep GAME $W/smoke_games.txt
python3 scripts/replay_scan.py target/deckwork/replays/smoke --deck-prefix real_build_converge3 | head -40

step "dump field"
$G --deck decks/real_build_converge3.txt --pairs 0 --dump-field $W/field --threads 1 >/dev/null 2>&1 || true
ls $W/field | wc -l

step "screen: converge3 seed 43"
$G --deck decks/real_build_converge3.txt --pairs 100 --seed 43 --threads 20 --out $R
step "screen: converge3 seed 97"
$G --deck decks/real_build_converge3.txt --pairs 100 --seed 97 --threads 20 --out $R
step "screen: yardstick recommended W/B seed 43"
$G --deck decks/recommended_from_pool.txt --pairs 100 --seed 43 --threads 20 --out $R
step "screen: convlands pilot on converge3 seed 43 (bot flag gate, deck seat only)"
$G --deck decks/real_build_converge3.txt --pairs 100 --seed 43 --threads 20 --pilot convlands --out $R

step "confirm: converge3 mcts256 both seats, replays (25 pairs x 24)"
rm -rf target/deckwork/replays/converge3_mcts256
$G --deck decks/real_build_converge3.txt --pairs 25 --seed 43 --threads 12 --pilot mcts256 --opp-pilot mcts256 \
   --replays target/deckwork/replays/converge3_mcts256 --out $R
python3 scripts/replay_scan.py target/deckwork/replays/converge3_mcts256 --deck-prefix real_build_converge3 > $W/scan_converge3_mcts256.txt 2>&1
head -60 $W/scan_converge3_mcts256.txt
echo "=== phase1 done $(date)"
