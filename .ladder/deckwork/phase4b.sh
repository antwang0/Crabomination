#!/usr/bin/env bash
# Waits for the rebuild, reproduces inc1 on the new binary, screens adds+lands,
# then the stun-hold gate (deck seat on the gauntlet; both seats on the sealed ladder).
set -u
cd /home/archuser/repos/Crabomination
W=.ladder/deckwork; G=./target/release-fast/deck_gauntlet; R=$W/results.txt
until grep -q "BUILD EXIT" $W/rebuild.log; do sleep 15; done
grep -q "BUILD EXIT 0" $W/rebuild.log || { echo "REBUILD FAILED"; exit 1; }
while pgrep -f "confirm2.sh" >/dev/null; do sleep 15; done
echo "=== phase4b start $(date)"
echo "--- reproduce inc1 on the rebuilt binary (old: 61.11 pooled; seed 43 cell below)"
$G --deck $W/variants/inc1.txt --pairs 500 --seed 43 --threads 20 --out $W/repro.txt 2>/dev/null | grep -E "^RESULT"
grep "^RESULT deck=cut_quandrix_charm .*seed=43 " $R | tail -1
$W/screen_dir.sh $W/gen1_adds inc1 20
echo "--- stun-hold gate: inc1 deck seat stunhold vs dflt opponents"
for s in 43 97; do $G --deck $W/variants/inc1.txt --pairs 500 --seed $s --threads 20 --pilot stunhold --out $R 2>/dev/null | grep -E "^RESULT"; done
for s in 43 97; do $G --deck $W/variants/inc1.txt --pairs 500 --seed $s --threads 20 --out $R 2>/dev/null | grep -E "^RESULT"; done
echo "--- stun-hold gate: sealed ladder mirror (both seats), 500 games x 12 decks"
for s in 43 97; do
  ./target/release-fast/bot_ladder --a stun-hold --b dflt --decks sealed --games 500 --seed $s --threads 20 > $W/ladder_stunhold_s$s.txt 2>&1
  grep -E "A win%|verdict" $W/ladder_stunhold_s$s.txt
done
echo "=== phase4b done $(date)"
