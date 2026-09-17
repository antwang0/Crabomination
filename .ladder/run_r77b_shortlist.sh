#!/usr/bin/env bash
# Round 77b (2026-09-17): the shortlist round the round-77 census bought.
# Four arms, each paired vs the adopted default (X by outcome on):
#   top4        `eval_top: 4`           — the shortlist at four
#   top5        `eval_top: 5`           — at five
#   mc-nocut    `magecraft_cut: false`  — with a magecraft permanent out the
#                                         ranking still puts instants and
#                                         sorceries first, but the shortlist
#                                         no longer STOPS at the first
#                                         non-spell (the census put two
#                                         thirds of the cut's losses there)
#   top4-nocut  both
#
# CELLS: sealed, paired, 1 000 games × 12 decks, seeds 43 / 97 (±0.6 a
# cell, ~5 s each at 22 threads), then a wall-clock ABBA of the mirror
# (dflt / arm / arm / dflt) for every arm.
#
# PRE-REGISTERED: adopt at >= +0.5 pooled over the two seeds, no cell
# <= 49.5, wall clock <= +10 % on the mirror ABBA. Between two qualifying
# arms the larger (more sims) wins only if it adds >= +0.3 pooled over the
# smaller. `mc-nocut` on its own qualifying says the magecraft stop was the
# hole and the shortlist size was not; `top4` on its own says the reverse.
# Any arm <= 49.5 on a cell: the extra outcome sims are choosing worse (the
# static score's top three were a filter the outcome eval needed), record
# and park.
set -u
cd /home/archuser/repos/Crabomination
LADDER=${LADDER:-./target/release-fast/bot_ladder}
R=.ladder/r77
mkdir -p "$R"
GAMES=${GAMES:-1000}
THREADS=${THREADS:-22}
ARMS=${ARMS:-"top4 top5 mc-nocut top4-nocut"}

[ -x "$LADDER" ] || { echo "no $LADDER — cargo build --profile release-fast -p crabomination --bin bot_ladder"; exit 2; }

run_cell() { # name args...
  local f="$R/$1.txt"; shift
  if [ -s "$f" ] && grep -q "A win%" "$f"; then echo "have $f"; return; fi
  echo "== $f $(date -Is)"
  "$@" > "$f" 2>&1
  echo "exit: $?"
}

for arm in $ARMS; do
  for g in ${SEEDS:-43 97}; do
    run_cell "${arm}_g${g}" $LADDER --a "$arm" --b dflt --decks sealed --games "$GAMES" --seed "$g" --threads "$THREADS"
  done
done

echo "== wall clock ABBA (sealed mirror, seed 43)"
for arm in $ARMS; do
  f="$R/abba_${arm}.txt"
  if [ -s "$f" ]; then echo "have $f"; cat "$f"; continue; fi
  for side in dflt "$arm" "$arm" dflt; do
    e=$($LADDER --a "$side" --b "$side" --decks sealed --games "$GAMES" --seed 43 --threads "$THREADS" 2>&1 | grep -o "in [0-9.]*s")
    echo "$side $e" | tee -a "$f"
  done
done

echo "== readings"
for arm in $ARMS; do
  for g in ${SEEDS:-43 97}; do
    printf "%-11s g%s  " "$arm" "$g"; grep -h "A win%" "$R/${arm}_g${g}.txt"
  done
done
