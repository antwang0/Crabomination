#!/usr/bin/env bash
# Round 76b (2026-09-17): X by outcome — `EvalWeights::x_by_outcome`
# (profile `xout`), the round the round-76 census bought.
#
# WHY. The census priced every smaller X of every X finalist with the
# pick's own evaluator: the max-X rule is right on seven of the pool's
# nine X cards (0.3–4 % of casts have a better smaller X) and wrong on
# 70 % / 73 % of the casts of Fix What's Broken and Vicious Rivalry — the
# two pay-X-life cards whose X is a mana-value threshold, which
# `max_affordable_x` sizes off the mana pool. Seed 43's decks carry both
# (0.27 casts/game); seed 97's carry neither. Under the flag
# `pick_by_outcome` prices the X candidates of every X finalist (the
# mana values on the table and in the graveyard for a pay-X-life spell,
# the smaller values for a mana-X spell) and swaps in the best on strict
# improvement only.
#
# PRE-REGISTERED (sealed, paired, 1 000 games × 12 decks a cell, ±0.6):
#   seed 43 >= +0.5 AND seed 97 >= 49.5  -> ADOPT on the default (the flag
#                                          fires where the cards are and
#                                          costs nothing where they are not).
#   seed 43 within ±0.5                  -> the mis-sizing does not reach the
#                                          result at 0.27 casts/game; PARK,
#                                          keep the flag for the client
#                                          (a human sees a 4-mana no-op).
#   any cell <= 49.0                     -> a LOSS; read the census under
#                                          the flag before anything else.
#   CHECK: `CRAB_MENU_CENSUS=2 --a xout --b xout --seed 43` must read
#   "a smaller X scored higher" ~0 % of X wins (the mechanism fires).
set -u
cd /home/archuser/repos/Crabomination
LADDER=${LADDER:-./target/release-fast/bot_ladder}
R=.ladder/r76
mkdir -p "$R"
GAMES=${GAMES:-1000}
THREADS=${THREADS:-22}

[ -x "$LADDER" ] || { echo "no $LADDER — cargo build --profile release-fast -p crabomination --bin bot_ladder"; exit 2; }

run_cell() { # name args...
  local f="$R/$1.txt"; shift
  if [ -s "$f" ] && grep -q "paired\|win rate\|inconclusive\|MARGINAL" "$f"; then
    echo "have $f"; return
  fi
  echo "== $f $(date -Is)"
  "$@" > "$f" 2>&1
  echo "exit: $?"
}

for g in ${SEEDS:-43 97}; do
  run_cell "xout_g${g}" $LADDER --a xout --b dflt --decks sealed --games "$GAMES" --seed "$g" --threads "$THREADS"
done
run_cell "xout_census_g43" env CRAB_MENU_CENSUS=2 $LADDER --a xout --b xout --decks sealed --games "$GAMES" --seed 43 --threads "$THREADS"

echo "== readings"
for g in ${SEEDS:-43 97}; do
  echo "-- seed $g"; grep -E "^TOTAL|A win%|^verdict" "$R/xout_g${g}.txt"
done
grep -h "menu_census\|x_card" "$R/xout_census_g43.txt"
