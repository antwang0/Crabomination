#!/usr/bin/env bash
# Round 73: the REST of round 72's class — a bounce, an exile and a discard
# cost reach the priced owner too. A DEFECT gate (no loss), not a strength arm.
#
# WHAT CHANGED. One flag, `sink_costs_priced`, on the default (which already
# carries round 72's `sac_sinks_priced`):
#   sink-costs  the five sink generators, and `pick_sacrifice_value` itself,
#               read `ability_spends_a_permanent_or_card` instead of the
#               sacrifice-only predicate — so a **bounce**, an **exile** (of
#               another permanent or of the source) and a **discard** cost
#               reach the priced owner too.
#
# WHY. Those three had no priced owner AT ALL: `ability_sink_bits` did not set
# `AB_SAC` for them and `pick_sacrifice_value` did not consider them, so the
# five sink generators were the only thing that ever fired them — unpriced, as
# a last-resort mana sink. That is round 72's defect one field over, and the
# census is 48 catalog abilities: `discard_cost` + draw 18, + token 9,
# + self-counter 8; `exile_other_filter` + token 6, + self-counter 3,
# + team-pump 1; `bounce_other_filter` + draw 2, + token 1. "Exile another
# permanent: create a 1/1" and "discard a card: put a +1/+1 counter on this"
# are the shapes.
#
# ⚠⚠ **THE ARM IS NOT IN THE TREE AND THIS SCRIPT WILL NOT RUN AS IS.** It was
# built, laddered and REVERTED: a parked flag on this path costs the shipped
# default **+0.034 / +0.015 / +0.022 %** on fixed / cube / sealed (the extra
# field tests in `ability_sink_bits`, measured), which is a seventh of what
# `(-296)` won, for a flag `(-297)` parks off. PERF `(-297)` has the whole
# design — the predicate, the separate `sink::AB_SPEND` gate bit that keeps the
# default's gate byte-identical, and the reading — so re-adding the arm is
# `EvalWeights::sink_costs_priced` + `sink_costs_priced_on()` + one
# `bot_ladder.rs` match arm. **Read `(-297)` first: it says the predicate is
# NOT what to change.** What is kept here is the pre-registered thresholds and
# the pool set, which is the part worth not re-deriving.
#
# BASE. `dflt` = `EvalWeights::default()` at the run's tip.
#
# PRE-REGISTERED READINGS (pooled over four seeds; the r50 rule on cells).
#   * pooled >= 49.9 and no cell's interval wholly below 50  -> adopt.
#   * 49.5 <= pooled < 49.9   -> park and say so; pricing a play the bot was
#                                taking blind is not worth a measured loss.
#   * pooled < 49.5           -> park.
# ⚠ **THE INCIDENCE IS PART OF THE READING.** The two profiles differ only on
# a board with one of those cost fields, so most sealed pairs are exact
# mirrors and the paired estimator's interval is about the discordant pairs
# only — the round-50 rare-class rule. The split counts are printed.
set -eu
cd "$(dirname "$0")/.."
LADDER=${LADDER:-./target/profiling-fast/bot_ladder}
GAMES=${GAMES:-1000}
ARMS=${ARMS:-"sink-costs"}
BASE=${BASE:-dflt}
THREADS=${THREADS:-3}
OUT=.ladder/r73

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
  echo "=== $arm vs $BASE (cube) ==="
  for lseed in 43 97; do run "${arm}_c${lseed}" "$arm" "$BASE" "$lseed" cube; done
  echo "=== $arm vs $BASE (all — the pool round 72 had its only incidence on) ==="
  for lseed in 43 97; do run "${arm}_a${lseed}" "$arm" "$BASE" "$lseed" all; done
done

echo "=== summary ==="
python3 - $ARMS <<'PY'
import re, sys, pathlib, statistics as st
for arm in sys.argv[1:]:
    for pool, tags in (("sealed", "l"), ("cube", "c"), ("all", "a")):
        v = []
        for l in (43, 97, 151, 199):
            p = pathlib.Path(f".ladder/r73/{arm}_{tags}{l}.txt")
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
echo "round-73 complete"
