#!/usr/bin/env bash
# Round 60: the open-board attack shortcut re-read on the round-58 default — a
# THROUGHPUT gate (no loss), the round-58 shape.
#
# WHAT CHANGED. Nothing in the engine: `attack_skip_open` (profile
# `dflt-open` = the round-58 default + the flag) takes the greedy attack
# declaration without a sim when no opposing seat controls a creature,
# planeswalker or battle. Read at e725e5c2 on the `gang` base (`atk-open`):
# -1.3 / -1.8 % wall clock on cube / sealed, -0.1 pt on a 96 k-game sealed
# ladder — it overrides the sim's hold-back of an attack-trigger creature
# (a Goblin Guide held back to not card the opponent) — and filed as an
# opt-in pilot. Re-read here because the default's search now costs 3.3
# menu sims plus 3.4 chain sims a declaration, and the creatureless board
# is 11.4 % of searched declarations (1 824 of 16 060, sealed, 1 200
# games, greedy winning 1 814 of them).
#
# BASE. `dflt58` = `EvalWeights::round58_default()`, the default as it
# stood after round 58, frozen (the r52 precedent). The recorded cells ran
# as `dflt` before the profile was split; the two were identical then.
#
# STEP 0 is the paired wall clock of `dflt-open` against `dflt` and the
# census on both.
#
# PRE-REGISTERED READINGS (pooled over four seeds; the r50 rule on cells).
#   * pooled >= 49.9 and no cell's interval wholly below 50  -> adopt on
#     the default only (the client pilot keeps the sim's hold-backs).
#   * 49.5 <= pooled < 49.9                                  -> the r50
#     reading again; adopt only if the wall-clock win is >= 10 %.
#   * pooled < 49.5                                          -> park.
set -eu
cd "$(dirname "$0")/.."
LADDER=${LADDER:-./target/release-fast/bot_ladder}
GAMES=${GAMES:-1000}
COST_GAMES=${COST_GAMES:-200}
COST_REPS=${COST_REPS:-5}

[ -x "$LADDER" ] || { echo "build it: cargo build --profile release-fast -p crabomination --bin bot_ladder" >&2; exit 1; }

echo "=== step 0a: census (sealed, $COST_GAMES games, seed 43) ==="
for arm in dflt58 dflt-open; do
  f=".ladder/r60_census_${arm}.txt"
  if ! [ -s "$f" ]; then
    CRAB_ATTACK_CENSUS=1 $LADDER --a "$arm" --b "$arm" --decks sealed --games "$COST_GAMES" --seed 43 > "$f" 2>&1 || true
  fi
  echo "  $arm: $(grep -E 'attack_census' "$f" | sed -E 's/.*chain sims (.*)$/chain sims \1/')  $(grep -E 'decided' "$f" | tail -1)"
done

echo "=== step 0b: paired wall clock, $COST_REPS reps x (dflt then dflt-open) ==="
f=".ladder/r60_cost.txt"
if ! [ -s "$f" ]; then
  for rep in $(seq 1 "$COST_REPS"); do
    for arm in dflt58 dflt-open; do
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
ratios = [by[r]["dflt-open"] / by[r]["dflt58"] for r in by if "dflt-open" in by[r] and "dflt58" in by[r]]
print(f"  dflt-open wall/dflt58 median {st.median(ratios):.3f}  mean {st.mean(ratios):.3f}  per-rep {[round(x, 3) for x in ratios]}")
PY

run () { # tag A B seed
  local tag="$1" a="$2" b="$3" seed="$4"
  if [ -s ".ladder/r60_${tag}.txt" ] && grep -q "A win%" ".ladder/r60_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r60_${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  $LADDER --a "$a" --b "$b" --decks sealed --games "$GAMES" --seed "$seed" \
      > ".ladder/r60_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r60_${tag}.txt" | tail -1 | tr -s ' ')  $(grep -E 'decided' ".ladder/r60_${tag}.txt" | tail -1)"
}

echo "=== dflt-open vs dflt ==="
for lseed in 43 97 151 199; do run "gate_l${lseed}" dflt-open dflt58 "$lseed"; done

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
v = []
for l in (43, 97, 151, 199):
    p = pathlib.Path(f".ladder/r60_gate_l{l}.txt")
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
    print(f"  dflt-open pooled {pooled:5.2f}%   cells {[x[0] for x in v]}   highs {[x[2] for x in v]}")
    print(f"             verdict: {verdict}")
PY
echo "round-60 complete"
