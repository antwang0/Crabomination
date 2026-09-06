#!/usr/bin/env bash
# After gate3b's search-pilot confirm: the test gate on the adopted default, the rebuild,
# the incumbent consistency check (dflt must now equal the allfix cells), then the search rerun.
set -u
cd /home/archuser/repos/Crabomination
R3=.ladder/deckwork/r3; G=./target/release-fast/deck_gauntlet
until grep -q "=== gate3b done" $R3/gate3b.log 2>/dev/null; do sleep 20; done
echo "=== after_confirm start $(date)"
cargo nextest run -p crabomination -E 'test(/server::bot::/)' 2>&1 | tail -2
cargo nextest run -p crabomination_tests --test core_rules -E 'test(/target|golden/)' 2>&1 | tail -2
cargo nextest run -p crabomination_tests --test sos -E 'test(/target|omens|together|critique|homesick|outburst|mind_roots|mathemagics|brilliance/)' 2>&1 | tail -2
cargo check --profile release-fast -p crabomination --bin bot_ladder 2>&1 | tail -1
cargo build --profile release-fast -p crabomination --bin deck_gauntlet --bin bot_ladder --bin deck_duel 2>&1 | tail -2
ls -la --time-style=+%H:%M:%S target/release-fast/deck_gauntlet target/release-fast/bot_ladder target/release-fast/deck_duel
echo "--- incumbents on the adopted default (expect converge3 52.14 / 52.65, converge4 68.24 / 68.84)"
for d in decks/real_build_converge3.txt decks/real_build_converge4.txt; do for s in 43 97; do
  $G --deck $d --pairs 500 --seed $s --threads 20 --out $R3/results_c.txt 2>/dev/null | grep "^RESULT" | sed 's/field=[^ ]* //'
done; done
echo "--- r67-off control must reproduce the old default (43.20 / 43.95, 66.36 / 67.10)"
for d in decks/real_build_converge3.txt decks/real_build_converge4.txt; do for s in 43 97; do
  $G --deck $d --pilot r67off --pairs 500 --seed $s --threads 20 --out $R3/results_c.txt 2>/dev/null | grep "^RESULT" | sed 's/field=[^ ]* //'
done; done
echo "=== after_confirm done $(date); launching the search"
setsid nohup $R3/run_r3_search.sh > $R3/run_r3_search.log 2>&1 &
