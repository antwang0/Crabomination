#!/usr/bin/env bash
# Round 72: the mana-sink generators defer a sacrifice-cost ability to its
# priced owner — a DEFECT gate (no loss), not a strength arm.
#
# WHAT CHANGED. One flag, `sac_sinks_priced`, on the default:
#   sac-sinks  five sink generators (`pick_team_pump`, `pick_card_draw_ability`,
#              `pick_impulse_draw_ability`, `pick_self_pump_counter`,
#              `pick_token_maker`) ask `ability_sacrifices_a_permanent` instead
#              of `ab.sac_cost`, so an ability that sacrifices ANOTHER
#              permanent falls through to `pick_sacrifice_value`, which prices
#              both sides of the exchange. Four of the five run AFTER that
#              generator in the chain, so they were taking, unpriced, exactly
#              the exchanges it had just refused.
#
# WHY. `all` seed 1274 of the fresh-seed sweep, a capped game at turn 5,769:
# Thopter Foundry ("{1}, Sacrifice a nontoken artifact: create a 1/1 Thopter,
# gain 1 life") on a board whose only nontoken artifact is Blightsteel
# Colossus — 11/11 trample infect, against an opponent on 0 poison.
# `pick_token_maker` fed it to the Foundry the turn it was cast, for 1 life and
# a 1/1; the Colossus `shuffles_into_library_instead`, so it came back on top
# of a one-card library and was drawn and cast again, forever. The cell reads
# `6,798 decided / 2 cap` on `dflt` and `6,800 decided / 0 undecided` here.
#
# BASE. `dflt` = `EvalWeights::default()` at the run's tip.
#
# PRE-REGISTERED READINGS (pooled over four seeds; the r50 rule on cells).
#   * pooled >= 49.9 and no cell's interval wholly below 50  -> adopt.
#   * 49.5 <= pooled < 49.9   -> park and say so; the stall fix is not worth
#                                a measured strength loss.
#   * pooled < 49.5           -> park.
# ⚠ **THE INCIDENCE IS PART OF THE READING.** The two profiles differ only on
# a board with a sacrifice-OTHER ability, so most sealed pairs are exact
# mirrors and the paired estimator's interval is about the discordant pairs
# only — the round-50 rare-class rule. The split counts are printed.
set -eu
cd "$(dirname "$0")/.."
LADDER=${LADDER:-./target/profiling-fast/bot_ladder}
GAMES=${GAMES:-1000}
ARMS=${ARMS:-"sac-sinks"}
BASE=${BASE:-dflt}
THREADS=${THREADS:-3}
OUT=.ladder/r72

[ -x "$LADDER" ] || { echo "build it: cargo build --profile profiling-fast -p crabomination --bin bot_ladder" >&2; exit 1; }
mkdir -p "$OUT"

run () { # tag A B seed pool
  local tag="$1" a="$2" b="$3" seed="$4" pool="$5"
  if [ -s "$OUT/${tag}.txt" ] && grep -q "A win%" "$OUT/${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' "$OUT/${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  $LADDER --a "$a" --b "$b" --decks "$pool" --games "$GAMES" --seed "$seed" --threads "$THREADS" \
      > "$OUT/${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' "$OUT/${tag}.txt" | tail -1 | tr -s ' ')  $(grep -E 'decided' "$OUT/${tag}.txt" | tail -1)  $(grep -E '^paired' "$OUT/${tag}.txt" | tail -1 | tr -s ' ')"
}

for arm in $ARMS; do
  echo "=== $arm vs $BASE (sealed) ==="
  for lseed in 43 97 151 199; do run "${arm}_l${lseed}" "$arm" "$BASE" "$lseed" sealed; done
  echo "=== $arm vs $BASE (cube — the pool that carries the sacrifice outlets) ==="
  for lseed in 43 97; do run "${arm}_c${lseed}" "$arm" "$BASE" "$lseed" cube; done
done

echo "=== summary ==="
python3 - $ARMS <<'PY'
import re, sys, pathlib, statistics as st
for arm in sys.argv[1:]:
    for pool, tags in (("sealed", "l"), ("cube", "c")):
        v = []
        for l in (43, 97, 151, 199):
            p = pathlib.Path(f".ladder/r72/{arm}_{tags}{l}.txt")
            if not p.exists():
                continue
            m = re.findall(r"A win%\s+([0-9.]+)%\s+\[([0-9.]+)%, ([0-9.]+)%\]", p.read_text())
            if m:
                v.append(tuple(float(x) for x in m[-1]))
        if not v:
            continue
        pooled = st.mean(x[0] for x in v)
        below = [x for x in v if x[2] < 50.0]
        if pooled >= 49.9 and not below:
            verdict = "no loss — adopt"
        elif pooled >= 49.5:
            verdict = "leans a loss — park and say so"
        else:
            verdict = "LOSS — park"
        print(f"  {arm:10} {pool:6} pooled {pooled:5.2f}%   cells {[x[0] for x in v]}   highs {[x[2] for x in v]}   verdict: {verdict}")
PY
echo "round-72 complete"
