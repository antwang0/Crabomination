#!/usr/bin/env bash
# Round 59: the empty-greedy attack chain behind a blocker gate — a
# THROUGHPUT gate (no loss), the round-58 shape.
#
# WHAT CHANGED. Under the wide flag the chain runs from an empty greedy on
# 44 % of searched declarations (11 200 of 25 384, sealed, 1 200 games) and
# greedy's refusal is usually right there. `attack_empty_gate` (profile
# `empty-gate`) skips that chain — and the sim of "nobody" that feeds it —
# when the defender's untapped creatures that may block are at least as
# many as this seat's untapped creatures: greedy held each creature behind
# a blocker that dominates it, and one blocker per attacker leaves no
# overload for the pair move to find. Instance reads only.
#
# BASE. `dflt` = the round-58 default (`EvalWeights::default()`); the r58
# flags are frozen inside it, and this gate runs before any later adoption
# moves `dflt` — re-run it against `EvalWeights::default_const()` as of
# e4babdbd if that changes.
#
# STEP 0 is the census on `dflt` (the flag off): how many empty-greedy
# searches the gate covers and how many of those the chain WON — the value
# at stake — then the paired wall clock of `empty-gate` against `dflt`.
#
# PRE-REGISTERED READINGS (pooled over four seeds; the r50 rule on cells).
#   * pooled >= 49.9 and no cell's interval wholly below 50  -> adopt.
#   * 49.5 <= pooled < 49.9                                  -> leans a
#     loss; adopt only if the wall-clock win is >= 10 % and the loss is
#     under the wide chain's own +0.5; otherwise park.
#   * pooled < 49.5                                          -> park.
set -eu
cd "$(dirname "$0")/.."
LADDER=${LADDER:-./target/release-fast/bot_ladder}
GAMES=${GAMES:-1000}
COST_GAMES=${COST_GAMES:-200}
COST_REPS=${COST_REPS:-5}

[ -x "$LADDER" ] || { echo "build it: cargo build --profile release-fast -p crabomination --bin bot_ladder" >&2; exit 1; }

echo "=== step 0a: census (sealed, $COST_GAMES games, seed 43) ==="
for arm in dflt empty-gate; do
  f=".ladder/r59_census_${arm}.txt"
  if ! [ -s "$f" ]; then
    CRAB_ATTACK_CENSUS=1 $LADDER --a "$arm" --b "$arm" --decks sealed --games "$COST_GAMES" --seed 43 > "$f" 2>&1 || true
  fi
  echo "  $arm: $(grep -E 'attack_census' "$f" | sed -E 's/.*chain sims (.*)$/chain sims \1/')  $(grep -E 'decided' "$f" | tail -1)"
done

echo "=== step 0b: paired wall clock, $COST_REPS reps x (dflt then empty-gate) ==="
f=".ladder/r59_cost.txt"
if ! [ -s "$f" ]; then
  for rep in $(seq 1 "$COST_REPS"); do
    for arm in dflt empty-gate; do
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
ratios = [by[r]["empty-gate"] / by[r]["dflt"] for r in by if "empty-gate" in by[r] and "dflt" in by[r]]
print(f"  empty-gate wall/dflt median {st.median(ratios):.3f}  mean {st.mean(ratios):.3f}  per-rep {[round(x, 3) for x in ratios]}")
PY

run () { # tag A B seed
  local tag="$1" a="$2" b="$3" seed="$4"
  if [ -s ".ladder/r59_${tag}.txt" ] && grep -q "A win%" ".ladder/r59_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r59_${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  $LADDER --a "$a" --b "$b" --decks sealed --games "$GAMES" --seed "$seed" \
      > ".ladder/r59_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r59_${tag}.txt" | tail -1 | tr -s ' ')  $(grep -E 'decided' ".ladder/r59_${tag}.txt" | tail -1)"
}

echo "=== empty-gate vs dflt ==="
for lseed in 43 97 151 199; do run "gate_l${lseed}" empty-gate dflt "$lseed"; done

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
v = []
for l in (43, 97, 151, 199):
    p = pathlib.Path(f".ladder/r59_gate_l{l}.txt")
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
    print(f"  empty-gate pooled {pooled:5.2f}%   cells {[x[0] for x in v]}   highs {[x[2] for x in v]}")
    print(f"             verdict: {verdict}")
PY
echo "round-59 complete"
