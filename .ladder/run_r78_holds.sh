#!/usr/bin/env bash
# Round 78 (2026-09-18): the default's main-phase HOLDS, ablated. Round 77's
# lesson — an ordering rule is cheap, a stop the outcome eval cannot
# overrule is where strength hides — applied to the stops that remain in
# the scored main-phase pick. A temporal hold cannot be priced by the
# pick's evaluator (holding is "the same value later", which a one-tick
# outcome sim does not see), so the honest instrument is the paired
# off-arm, which costs five seconds a cell; the census supplies the
# denominators.
#
# ARMS, each the default with one hold changed, paired vs `dflt`:
#   hold-off         `hold_sick: false`   — the summon-sick hold: a creature
#                                           that cannot attack this turn is
#                                           deferred to the second main and
#                                           the tick PASSES
#   hold-next        `hold_sick_next`     — the hold stays, but instead of
#                                           passing it plays the runner-up
#                                           finalist that scored above the
#                                           baseline and improves this turn
#   trick-modes-off  `trick_modes_combat_only: false` — instants with an
#                                           until-end-of-turn pump offered
#                                           in the main phase again
#   x0-skip-off      `skip_noop_x0: false` — the X=0 no-op cast back on the
#                                           menu
#
# CENSUS (`CRAB_MENU_CENSUS=1`, `holds_census` line): how often each hold
# fires per game, and for the summon-sick hold how often a qualifying
# runner-up was on the menu when the tick passed.
#
# CELLS: sealed, paired, 1 000 games × 12 decks, seeds 43 / 97; then the
# mirror wall-clock ABBA per arm on a quiet box.
#
# PRE-REGISTERED, per arm, pooled over the two seeds:
#   >= +0.5, no cell <= 49.5  -> the hold COSTS as written; adopt the arm
#                                (for `hold-next` that is the refinement,
#                                for the others the removal), wall clock
#                                <= +10 %.
#   <= -0.5                   -> the hold PAYS; recorded as its first paired
#                                measurement on the current default, closed.
#   between                   -> neutral at this pool; keep the default, no
#                                churn. `hold-next` and `hold-off` both
#                                qualifying: the larger pooled gain wins.
set -u
cd /home/archuser/repos/Crabomination
LADDER=${LADDER:-./target/release-fast/bot_ladder}
R=.ladder/r78
mkdir -p "$R"
GAMES=${GAMES:-1000}
THREADS=${THREADS:-22}
ARMS=${ARMS:-"hold-off hold-next trick-modes-off x0-skip-off"}

[ -x "$LADDER" ] || { echo "no $LADDER — cargo build --profile release-fast -p crabomination --bin bot_ladder"; exit 2; }

run_cell() { # name args...
  local f="$R/$1.txt"; shift
  if [ -s "$f" ] && grep -q "A win%" "$f"; then echo "have $f"; return; fi
  echo "== $f $(date -Is)"
  "$@" > "$f" 2>&1
  echo "exit: $?"
}

for g in ${SEEDS:-43 97}; do
  run_cell "census_g${g}" env CRAB_MENU_CENSUS=1 $LADDER --a dflt --b dflt --decks sealed --games "$GAMES" --seed "$g" --threads "$THREADS"
done
for arm in $ARMS; do
  for g in ${SEEDS:-43 97}; do
    run_cell "${arm}_g${g}" $LADDER --a "$arm" --b dflt --decks sealed --games "$GAMES" --seed "$g" --threads "$THREADS"
  done
done

echo "== wall clock ABBA (sealed mirror, seed 43)"
for arm in $ARMS; do
  f="$R/abba_${arm}.txt"
  if [ -s "$f" ]; then echo "have $f"; else
    for side in dflt "$arm" "$arm" dflt; do
      e=$($LADDER --a "$side" --b "$side" --decks sealed --games "$GAMES" --seed 43 --threads "$THREADS" 2>&1 | grep -o "in [0-9.]*s")
      echo "$side $e" >> "$f"
    done
  fi
  paste -s -d' ' "$f"
done

echo "== readings"
grep -h "holds_census" "$R"/census_g*.txt
for arm in $ARMS; do
  for g in ${SEEDS:-43 97}; do
    printf "%-16s g%s " "$arm" "$g"; grep -h "A win%" "$R/${arm}_g${g}.txt" | tail -1
  done
done
