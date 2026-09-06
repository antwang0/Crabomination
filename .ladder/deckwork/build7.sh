#!/usr/bin/env bash
cd /home/archuser/repos/Crabomination
while pgrep -x rustc >/dev/null || pgrep -x cargo >/dev/null; do sleep 10; done
echo "=== build7 start $(date)"
cargo check -p crabomination --bin deck_gauntlet --bin bot_ladder 2>&1 | grep -E "^(error|warning)" -A 8 | head -40
echo "CHECK EXIT ${PIPESTATUS[0]}"
cargo nextest run -p crabomination -E 'test(/server::bot::/)' 2>&1 | grep -E "FAIL|panicked|Summary|^error" -A 12 | head -80
echo "BOT TESTS EXIT ${PIPESTATUS[0]}"
cargo nextest run -p crabomination_tests --test core_rules -E 'test(/golden/)' 2>&1 | grep -E "FAIL|panicked|Summary|digest|expected|left|right" -A 3 | head -40
echo "GOLDEN EXIT ${PIPESTATUS[0]}"
cargo build --profile release-fast -p crabomination --bin deck_gauntlet --bin bot_ladder 2>&1 | grep -E "^error|Finished" -A 6
echo "BUILD EXIT ${PIPESTATUS[0]}"
cargo check --profile release-fast -p crabomination --bin bot_ladder 2>&1 | grep -E "^(error|warning)" -A 6 | head -20
echo "RELEASE-FAST CHECK EXIT ${PIPESTATUS[0]}"
echo "--- new default reproduces the flag-on cells; the off control reproduces the old default"
W=.ladder/deckwork; G=./target/release-fast/deck_gauntlet
for s in 43 97; do
  $G --deck $W/variants/v4_removal.txt --pairs 500 --seed $s --threads 20 --out $W/results7.txt 2>/dev/null | grep -E "^RESULT" | sed 's/field=[^ ]* //'
  $G --deck $W/variants/v4_removal.txt --pairs 500 --seed $s --threads 20 --pilot trickoff --out $W/results7.txt 2>/dev/null | grep -E "^RESULT" | sed 's/field=[^ ]* //'
  $G --deck decks/real_build_converge4.txt --pairs 500 --seed $s --threads 20 --out $W/results7.txt 2>/dev/null | grep -E "^RESULT" | sed 's/field=[^ ]* //'
done
for s in 43 97; do
  ./target/release-fast/bot_ladder --a trick-modes-off --b dflt --decks sealed --games 500 --seed $s --threads 20 > $W/ladder_trick-modes-off_s$s.txt 2>&1
  echo -n "trick-modes-off vs dflt s$s: "; grep -E "^\s+A win%" $W/ladder_trick-modes-off_s$s.txt
done
echo "=== build7 done $(date)"
