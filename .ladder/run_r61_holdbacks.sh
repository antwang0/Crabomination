#!/usr/bin/env bash
# Round 61: the holdback menu capped at three on the round-60 default — a
# THROUGHPUT gate (no loss), the round-58 shape.
#
# WHAT CHANGED. Nothing in the engine: `attack_search` capped at 3 instead
# of 6 (profile `dflt-as3` = the round-60 default + the cap). The holdback
# menu is greedy, all-home, then greedy-minus-one for the `attack_search`
# lowest-toughness attackers; with the attack chain growing declarations
# from nobody since round 55, the fourth-to-sixth holdbacks are the
# candidates to read: step 0's census prints holdback wins by menu index
# against the searches that offered that index.
#
# BASE. `dflt` = the round-60 default (`EvalWeights::default()`) as of the
# (-256) tip; re-run against `default_const()` at that tip if it moves.
#
# STEP 0 is the census (the value at stake by index) and the paired wall
# clock of `dflt-as3` against `dflt`.
#
# PRE-REGISTERED READINGS (pooled over four seeds; the r50 rule on cells).
#   * pooled >= 49.9 and no cell's interval wholly below 50  -> adopt on
#     the default.
#   * 49.5 <= pooled < 49.9                                  -> adopt only
#     if the wall-clock win is >= 5 %; otherwise park.
#   * pooled < 49.5                                          -> park; the
#     deep holdbacks were carrying something the chain does not reach.
set -eu
cd "$(dirname "$0")/.."
LADDER=${LADDER:-./target/release-fast/bot_ladder}
GAMES=${GAMES:-1000}
COST_GAMES=${COST_GAMES:-200}
COST_REPS=${COST_REPS:-5}

[ -x "$LADDER" ] || { echo "build it: cargo build --profile release-fast -p crabomination --bin bot_ladder" >&2; exit 1; }

echo "=== step 0a: census (sealed, $COST_GAMES games, seed 43) ==="
for arm in dflt dflt-as3; do
  f=".ladder/r61_census_${arm}.txt"
  if ! [ -s "$f" ]; then
    CRAB_ATTACK_CENSUS=1 $LADDER --a "$arm" --b "$arm" --decks sealed --games "$COST_GAMES" --seed 43 > "$f" 2>&1 || true
  fi
  echo "  $arm: $(grep -E 'attack_census' "$f" | sed -E 's/.*chain sims (.*)$/chain sims \1/')  $(grep -E 'decided' "$f" | tail -1)"
done

echo "=== step 0b: paired wall clock, $COST_REPS reps x (dflt then dflt-as3) ==="
f=".ladder/r61_cost.txt"
if ! [ -s "$f" ]; then
  for rep in $(seq 1 "$COST_REPS"); do
    for arm in dflt dflt-as3; do
      t=$($LADDER --a "$arm" --b "$arm" --decks sealed --games "$COST_GAMES" --seed 43 2>&1 \
          | grep -E 'decided' | tail -1 | sed -E 's/.* in ([0-9.]+)s.*/\1/')
      echo "$rep $arm $t" >> "$f"
    done
  done
fi
python3 - "$f" <<'PY'
import sys, statistics as st, collections
by = collections.defaultdict(dict)
for l in open(sys.argv[1]):
    if l.strip():
        rep, arm, t = l.split(); by[int(rep)][arm] = float(t)
ratios = [by[r]["dflt-as3"] / by[r]["dflt"] for r in by if "dflt-as3" in by[r] and "dflt" in by[r]]
print(f"  dflt-as3 wall/dflt median {st.median(ratios):.3f}  mean {st.mean(ratios):.3f}  per-rep {[round(x, 3) for x in ratios]}")
PY

run () { # tag A B seed
  local tag="$1" a="$2" b="$3" seed="$4"
  if [ -s ".ladder/r61_${tag}.txt" ] && grep -q "A win%" ".ladder/r61_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r61_${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  $LADDER --a "$a" --b "$b" --decks sealed --games "$GAMES" --seed "$seed" \
      > ".ladder/r61_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r61_${tag}.txt" | tail -1 | tr -s ' ')  $(grep -E 'decided' ".ladder/r61_${tag}.txt" | tail -1)"
}

echo "=== dflt-as3 vs dflt ==="
for lseed in 43 97 151 199; do run "gate_l${lseed}" dflt-as3 dflt "$lseed"; done

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
v = []
for l in (43, 97, 151, 199):
    p = pathlib.Path(f".ladder/r61_gate_l{l}.txt")
    if not p.exists():
        continue
    m = re.findall(r"A win%\s+([0-9.]+)%\s+\[([0-9.]+)%, ([0-9.]+)%\]", p.read_text())
    if m:
        v.append(tuple(float(x) for x in m[-1]))
if v:
    pooled = st.mean(x[0] for x in v)
    below = [x for x in v if x[2] < 50.0]
    if pooled >= 49.9 and not below:
        verdict = "no loss — adopt"
    elif pooled >= 49.5:
        verdict = "leans a loss — the step-0 cost rule decides"
    else:
        verdict = "LOSS — park"
    print(f"  dflt-as3 pooled {pooled:5.2f}%   cells {[x[0] for x in v]}   highs {[x[2] for x in v]}")
    print(f"             verdict: {verdict}")
PY
echo "round-61 complete"
