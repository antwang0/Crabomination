#!/usr/bin/env bash
# Round 77 (2026-09-17): the SHORTLIST census — what the `EVAL_TOP = 3`
# cut costs the scored main-phase pick. Round 76's instrument, generalised:
# the pick judges only the top three heuristic candidates by outcome; the
# census keeps the next five in rank order and prices each with the pick's
# own rule (temporary pin, else the settled outcome) against the winner's
# score. Not an A/B; it decides whether a shortlist-size round exists.
#
# INSTRUMENT: `CRAB_MENU_CENSUS=2` (bot.rs `menu_census::shortlist_*`),
# per pick whose ranked pool ran past the shortlist: cut candidates priced
# (engine-rejected ones counted apart), how often a cut candidate scores
# STRICTLY above the winner, by what margin, at which rank (4 / 5 / 6+),
# whether the winner was a lone finalist, whether the better line was a
# land drop / ability rather than a cast, and whether the first cut's
# static score tied the third finalist's (the jitter decided the cut).
# Level 2 adds the per-card table of the cut cards that beat the winner.
#
# CELLS: `dflt` mirror (the adopted default, X by outcome on), sealed,
# 1 000 games × 12 decks, seeds 43 / 97 (~5 s a cell at 22 threads).
#
# PRE-REGISTERED READINGS (per 1 000-game cell, both seeds):
#   "a cut candidate scored above the winner" >= 10 % of the picks that
#   ran past the shortlist AND those picks >= 5/game
#        -> a round: EVAL_TOP 3 -> N, N from the rank histogram (4 if rank
#           4 carries >= 70 % of the beats, else 5), paired vs the default
#           with the wall clock priced; adopt at >= +0.5 pooled, no cell
#           <= 49.5, wall clock <= +10 %.
#   < 3 %  -> CLOSE: the heuristic ranking's top three hold the outcome
#             winner; the shortlist is not where strength is lost.
#   between -> record; a round only if the mean margin is >= 1 unit and
#             the per-card table names a repeatable shape (a card class
#             the static score misranks), which then is its own lead.
#   Whatever the rate, "first cut tied the third finalist" high (>= 40 %)
#   says the cut is jitter, not judgement — a reason to widen regardless.
set -u
cd /home/archuser/repos/Crabomination
LADDER=${LADDER:-./target/release-fast/bot_ladder}
R=.ladder/r77
mkdir -p "$R"
GAMES=${GAMES:-1000}
THREADS=${THREADS:-22}

[ -x "$LADDER" ] || { echo "no $LADDER — cargo build --profile release-fast -p crabomination --bin bot_ladder"; exit 2; }

for g in ${SEEDS:-43 97}; do
  f="$R/census_g${g}.txt"
  if [ -s "$f" ] && grep -q "shortlist_census" "$f"; then
    echo "have $f"; continue
  fi
  echo "== census seed $g $(date -Is)"
  CRAB_MENU_CENSUS=2 $LADDER --a dflt --b dflt --decks sealed --games "$GAMES" --seed "$g" --threads "$THREADS" > "$f" 2>&1
  echo "exit: $?"
done
echo "== readings"
grep -h "shortlist_census\|cut_card" "$R"/census_g*.txt
