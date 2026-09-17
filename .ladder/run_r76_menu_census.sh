#!/usr/bin/env bash
# Round 76 (2026-09-17): the CENSUS of the audit shortlist's two uncensused
# holes — modal spells and X spells — as the scored main-phase pick sees
# them on the sealed pool. Not an A/B: nothing is adopted or parked on this
# run. It decides whether a round exists at all.
#
# WHAT THE CODE ALREADY DOES (read before pricing the "holes"): the cast
# enumerator offers one candidate per mode of a `ChooseMode` spell and one
# per mode (plus the every-mode combination) of a Spree / Tiered / Season
# spell, and the resolution-time `Decision::ChooseMode` is outcome-judged
# (`decide_mode_by_outcome`). So "always the card default" is stale at the
# menu; the live question is whether the alternate modes survive the
# `EVAL_TOP = 3` shortlist to the outcome eval. X is a single value — the
# max affordable (`max_affordable_x`), capped only for creature-only damage
# — and NEVER a branch: the sequence evaluator can see that a smaller X
# leaves mana for the next play only if that candidate exists.
#
# INSTRUMENT: `CRAB_MENU_CENSUS=2` (=1 without the per-card tables) (bot.rs `menu_census`), per scored pick:
# modal candidates offered vs cut by the shortlist, modal wins and the
# non-default share, sibling-mode finalists; X finalists and wins, and for
# every X finalist every smaller X priced by the pick's own evaluator
# (`evaluate_action_outcome`) — how often a smaller X scores STRICTLY higher
# than the chosen line, by what margin, and whether a losing X finalist's
# smaller X would have beaten the winner. Lone-finalist picks are scored
# too under the census (an X spell alone on the menu is still the decision).
#
# CELLS (v1 outputs kept as census_v1_g*.txt: the first read, before the
# unseen-mode pricing and the per-card tables): `dflt` vs `dflt` mirror, sealed, 1 000 games × 12 decks, seeds 43
# and 97 (~15 s a cell at 22 threads). The census is about decisions, not
# results, so the mirror is the cleanest reader: both seats are the default.
#
# PRE-REGISTERED READINGS (per 1 000-game cell, both seeds):
#   X: "smaller X scored higher" >= 15 % of X wins AND X wins >= 0.5/game
#        -> the X branch is a round: offer X-1 / X/2 / 0 as sibling
#           candidates and let the outcome eval judge (round-52 shape).
#      < 5 % of X wins, OR X wins < 0.2/game
#        -> CLOSE the X lead: the max-X rule is right where it matters, or
#           the pool does not cast enough X spells for a cell to see it.
#      between -> record; a round only if the mean margin is >= 1 unit.
#   Modes (the direct measure, added before the second read — the first
#   read had only the shortlist's cut rate, which a 3-mode Charm inflates
#   by construction): "an unseen mode scored higher" >= 10 % of modal wins
#           AND modal wins >= 0.5/game
#        -> a round: reserve a shortlist slot for the best alternate mode
#           (or price every mode at the pick), outcome-judged.
#      < 3 % of modal wins, OR modal wins < 0.3/game
#        -> CLOSE the modes lead: the shortlist's cut is not a strength
#           hole at this pool's incidence.
#      between -> record; a round only if the mean margin is >= 1 unit.
#   Resolution-time modes are reported for the record (already judged).
set -u
cd /home/archuser/repos/Crabomination
LADDER=${LADDER:-./target/release-fast/bot_ladder}
R=.ladder/r76
mkdir -p "$R"
GAMES=${GAMES:-1000}
THREADS=${THREADS:-22}

[ -x "$LADDER" ] || { echo "no $LADDER — cargo build --profile release-fast -p crabomination --bin bot_ladder"; exit 2; }

for g in ${SEEDS:-43 97}; do
  f="$R/census_g${g}.txt"
  if [ -s "$f" ] && grep -q "menu_census" "$f"; then
    echo "have $f"; continue
  fi
  echo "== census seed $g $(date -Is)"
  CRAB_MENU_CENSUS=2 $LADDER --a dflt --b dflt --decks sealed --games "$GAMES" --seed "$g" --threads "$THREADS" > "$f" 2>&1
  echo "exit: $?"
done
echo "== readings"
grep -h "menu_census\|x_card\|modal_card" "$R"/census_g*.txt
