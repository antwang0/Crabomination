#!/usr/bin/env bash
# Round 58: the wide attack chain's pair move, restricted — a THROUGHPUT
# gate (no loss), not a strength one.
#
# WHAT CHANGED. Round 56 adopted the wide chain (+0.5, every cell's interval
# clear of 50) and priced it at +28 % of the default's sealed-mirror wall
# clock, against +16 % for the block chain's +5.8 — the pair move offers
# `C(n, 2)` full-turn-cycle sims at EVERY chain's first step, though it was
# built for the empty-greedy overload (two attackers into one blocker connect
# where each alone is blocked and traded). Two restrictions, each a flag on
# `EvalWeights`, each keeping the overload board the r56 test pins:
#  * `attack_pairs_empty_only` (profile `pairs-empty`): pairs only when
#    greedy declared nobody — the board the move was built for. Off the
#    55 % of chains that start beside a non-empty greedy declaration.
#  * `attack_pairs_lazy` (profile `pairs-lazy`): the singles are priced
#    first and the pairs only when every single tied "finalize" — which is
#    the overload's own definition. A single that wins grows the set, and
#    the second attacker is then a single addition at the next step.
#  * both (profile `pairs-both`).
#
# BASE. `dflt56` = `EvalWeights::round56_default()`, the default as it stood
# after round 56, frozen (the r52 precedent), so this gate stays re-runnable
# after adoption. Every cell is flag-on-r56-default vs r56-default.
#
# STEP 0 is the throughput reading, and it is the point: the same binary,
# arms alternated, `--games $COST_GAMES` sealed mirrors, wall clock off the
# "decided ... in Xs" line, median of the per-rep ratio against `dflt56`;
# plus the census (chain sims per searched declaration) once per arm.
#
# PRE-REGISTERED READINGS (per arm, pooled over four seeds; the r50 rule
# on the cells' intervals).
#   * pooled >= 49.9 and no cell's interval wholly below 50  -> no loss:
#     adopt the CHEAPEST such arm (step 0 decides which).
#   * 49.5 <= pooled < 49.9, or one cell's interval wholly below 50
#                                                             -> leans a
#     loss; adopt only if step 0 says it buys >= 15 % and the loss is
#     under the wide chain's own +0.5 (i.e. the chain still nets positive
#     against `dflt55`); otherwise park.
#   * pooled < 49.5                                           -> a loss;
#     park, do not iterate the shape.
#   * clearly above 50.5                                      -> the pairs
#     were HURTING (an over-attack the singles refuse); adopt and say so.
set -eu
cd "$(dirname "$0")/.."
LADDER=${LADDER:-./target/release-fast/bot_ladder}
GAMES=${GAMES:-1000}
COST_GAMES=${COST_GAMES:-300}
COST_REPS=${COST_REPS:-5}
ARMS=${ARMS:-"pairs-empty pairs-lazy pairs-both"}

[ -x "$LADDER" ] || { echo "build it: cargo build --profile release-fast -p crabomination --bin bot_ladder" >&2; exit 1; }

echo "=== step 0a: census, one run per arm (sealed, $COST_GAMES games, seed 43) ==="
for arm in dflt56 $ARMS; do
  f=".ladder/r58_census_${arm}.txt"
  if ! [ -s "$f" ]; then
    CRAB_ATTACK_CENSUS=1 $LADDER --a "$arm" --b "$arm" --decks sealed --games "$COST_GAMES" --seed 43 > "$f" 2>&1 || true
  fi
  echo "  $arm: $(grep -E 'attack_census' "$f" | sed -E 's/.*chain sims ([0-9]+ \([0-9.]+\/search\)).*/chain sims \1/')  $(grep -E 'decided' "$f" | tail -1)"
done

echo "=== step 0b: paired wall clock, $COST_REPS reps x (dflt56 then each arm) ==="
f=".ladder/r58_cost.txt"
if ! [ -s "$f" ]; then
  for rep in $(seq 1 "$COST_REPS"); do
    for arm in dflt56 $ARMS; do
      t=$($LADDER --a "$arm" --b "$arm" --decks sealed --games "$COST_GAMES" --seed 43 2>&1 \
          | grep -E 'decided' | tail -1 | sed -E 's/.* in ([0-9.]+)s.*/\1/')
      echo "$rep $arm $t" >> "$f"
    done
  done
fi
python3 - "$f" <<'PY'
import sys, statistics as st, collections
rows = [l.split() for l in open(sys.argv[1]) if l.strip()]
by = collections.defaultdict(dict)
for rep, arm, t in rows:
    by[int(rep)][arm] = float(t)
arms = [a for a in dict.fromkeys(r[1] for r in rows) if a != "dflt56"]
for arm in arms:
    ratios = [by[r][arm] / by[r]["dflt56"] for r in by if arm in by[r] and "dflt56" in by[r]]
    print(f"  {arm:12s} wall/dflt56 median {st.median(ratios):.3f}  mean {st.mean(ratios):.3f}  per-rep {[round(x, 3) for x in ratios]}")
PY

run () { # tag A B seed
  local tag="$1" a="$2" b="$3" seed="$4"
  if [ -s ".ladder/r58_${tag}.txt" ] && grep -q "A win%" ".ladder/r58_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r58_${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  $LADDER --a "$a" --b "$b" --decks sealed --games "$GAMES" --seed "$seed" \
      > ".ladder/r58_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r58_${tag}.txt" | tail -1 | tr -s ' ')  $(grep -E 'decided' ".ladder/r58_${tag}.txt" | tail -1)"
}

for arm in $ARMS; do
  echo "=== $arm vs dflt56 ==="
  for lseed in 43 97 151 199; do run "${arm}_l${lseed}" "$arm" dflt56 "$lseed"; done
done

echo "=== summary ==="
python3 - $ARMS <<'PY'
import re, sys, pathlib, statistics as st
for arm in sys.argv[1:]:
    v = []
    for l in (43, 97, 151, 199):
        p = pathlib.Path(f".ladder/r58_{arm}_l{l}.txt")
        if not p.exists():
            continue
        m = re.findall(r"A win%\s+([0-9.]+)%\s+\[([0-9.]+)%, ([0-9.]+)%\]", p.read_text())
        if m:
            v.append(tuple(float(x) for x in m[-1]))
    if not v:
        continue
    pooled = st.mean(x[0] for x in v)
    below = [x for x in v if x[2] < 50.0]
    if pooled > 50.5:
        verdict = "clear WIN — the pairs were hurting; adopt"
    elif pooled >= 49.9 and not below:
        verdict = "no loss — adopt if cheapest"
    elif pooled >= 49.5:
        verdict = "leans a loss — adopt only on the step-0 cost rule"
    else:
        verdict = "LOSS — park"
    print(f"  {arm:12s} pooled {pooled:5.2f}%   cells {[x[0] for x in v]}   highs {[x[2] for x in v]}")
    print(f"               verdict: {verdict}")
PY
echo "round-58 complete"
