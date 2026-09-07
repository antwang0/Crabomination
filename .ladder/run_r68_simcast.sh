#!/usr/bin/env bash
# Round 68: the attack sim's own main-phase casts capped — a THROUGHPUT gate
# (no loss), the round-58 / round-60 shape.
#
# WHAT CHANGED. Nothing in the engine: `sim_main_cast_cap` (profiles
# `sim-cast0` / `sim-cast1` / `sim-cast2` = the default + the cap) lets the
# attack simulation's spell layer cast at most N spells from a main phase
# per sim (tricks, removal and stack responses are uncapped — they are the
# combat information the sim exists for). Read at the (-275) tip on the
# cube default: the sim's own casts are 17.2 % of the run (`accept_on`
# under `sim_spell_action_inner` 10,684 calls / 392 M Ir) and 33 sim
# passes a sim against sealed's 13.2 (PERF candidates, "THE NEW CUBE
# DEFAULT").
#
# BASE. `dflt` = `EvalWeights::default()` at the round-67 tip — the uncapped
# sim. The round adopted `sim-cast1`, so on a later binary run it with
# BASE=sim-cast-off (the round-68 control) to reproduce the cells.
#
# STEP 0 is the paired wall clock of each arm against the base on sealed and
# cube.
#
# PRE-REGISTERED READINGS (pooled over four seeds; the r50 rule on cells).
#   * pooled >= 49.9 and no cell's interval wholly below 50  -> adopt the
#     cheapest cap that passes (the smallest N) on the default only.
#   * 49.5 <= pooled < 49.9                                  -> adopt only
#     if the wall-clock win is >= 10 % on both pools.
#   * pooled < 49.5                                          -> park.
#
# RESULT (2026-09-07, `.ladder/r68/`): sim-cast0 49.08 pooled, every cell
# wholly below 50 (wall 0.730 / 0.753 sealed / cube) — park; sim-cast1 50.10
# (50.0 / 50.1 / 50.2 / 50.1; wall 0.864 / 0.837) and sim-cast2 50.10 (wall
# 0.926 / 0.940) no loss; cube cross-check sim-cast1 50.6 / 50.2, fixed
# 50.4 / 50.1 — sim-cast1 ADOPTED.
set -eu
cd "$(dirname "$0")/.."
LADDER=${LADDER:-./target/profiling-fast/bot_ladder}
GAMES=${GAMES:-1000}
COST_GAMES=${COST_GAMES:-200}
COST_REPS=${COST_REPS:-5}
ARMS=${ARMS:-"sim-cast0 sim-cast1 sim-cast2"}
BASE=${BASE:-dflt}
OUT=.ladder/r68

[ -x "$LADDER" ] || { echo "build it: cargo build --profile profiling-fast -p crabomination --bin bot_ladder" >&2; exit 1; }
mkdir -p "$OUT"

echo "=== step 0: paired wall clock, $COST_REPS reps x (dflt then each arm), $COST_GAMES x 12 ==="
for pool in sealed cube; do
  f="$OUT/cost_${pool}.txt"
  if ! [ -s "$f" ]; then
    for rep in $(seq 1 "$COST_REPS"); do
      for arm in $BASE $ARMS; do
        t=$($LADDER --a "$arm" --b "$arm" --decks "$pool" --games "$COST_GAMES" --seed 43 2>&1 \
            | grep -E 'decided' | tail -1 | sed -E 's/.* in ([0-9.]+)s.*/\1/')
        echo "$rep $arm $t" >> "$f"
      done
    done
  fi
  python3 - "$f" "$pool" "$BASE" <<'PY'
import sys, statistics as st, collections
by = collections.defaultdict(dict)
for l in open(sys.argv[1]):
    if l.strip():
        rep, arm, t = l.split(); by[int(rep)][arm] = float(t)
base = sys.argv[3]
arms = sorted({a for r in by.values() for a in r} - {base})
for arm in arms:
    ratios = [by[r][arm] / by[r][base] for r in by if arm in by[r] and base in by[r]]
    print(f"  {sys.argv[2]:6} {arm:10} wall/{base} median {st.median(ratios):.3f}  mean {st.mean(ratios):.3f}  per-rep {[round(x, 3) for x in ratios]}")
PY
done

run () { # tag A B seed pool
  local tag="$1" a="$2" b="$3" seed="$4" pool="$5"
  if [ -s "$OUT/${tag}.txt" ] && grep -q "A win%" "$OUT/${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' "$OUT/${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  $LADDER --a "$a" --b "$b" --decks "$pool" --games "$GAMES" --seed "$seed" \
      > "$OUT/${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' "$OUT/${tag}.txt" | tail -1 | tr -s ' ')  $(grep -E 'decided' "$OUT/${tag}.txt" | tail -1)"
}

for arm in $ARMS; do
  echo "=== $arm vs $BASE (sealed) ==="
  for lseed in 43 97 151 199; do run "${arm}_l${lseed}" "$arm" "$BASE" "$lseed" sealed; done
done

echo "=== summary ==="
python3 - $ARMS <<'PY'
import re, sys, pathlib, statistics as st
for arm in sys.argv[1:]:
    v = []
    for l in (43, 97, 151, 199):
        p = pathlib.Path(f".ladder/r68/{arm}_l{l}.txt")
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
        verdict = "leans a loss — the step-0 cost rule decides"
    else:
        verdict = "LOSS — park"
    print(f"  {arm:10} pooled {pooled:5.2f}%   cells {[x[0] for x in v]}   highs {[x[2] for x in v]}   verdict: {verdict}")
PY
echo "round-68 complete"
