#!/usr/bin/env bash
# Round 71: the block chain gated on the menu's outcome — a THROUGHPUT gate
# (no loss), the round-70 shape on the block side.
#
# WHAT CHANGED. Nothing in the engine: three one-flag arms on the default.
#   bchain-skipg  `block_chain_skip_greedy`  — no chain when the menu alone
#                 picked a non-empty greedy plan (round 70's gate, block side).
#   bchain-empty  `block_chain_empty_only`   — the chain only from the
#                 no-blocks board (round 56's own case: the bare menu's gang).
#   bchain-seed   `block_chain_from_menu`    — the chain grows from the menu's
#                 winner when that plan is non-empty, instead of from nothing.
# The census at the run's tip (PERF candidates, "THE BLOCK CHAIN SPLIT";
# `CRAB_ATTACK_CENSUS=1`, sealed dflt mirror, 14,400 games, seed 43): the
# chain ran on 98.8 % of 88,328 block searches at 4.77 sims each; where the
# menu alone picked greedy it spent 46.6 % of the sims (5.88 a search) and
# won 24.9 %, no blocks 27.6 % (2.62 a search) and won 48.7 %, another
# candidate 25.8 % (11.2 a search) and won 40.1 %. Cube: 52 / 30 / 18 % of
# the sims, won 21.7 / 38.6 / 35.8 %.
#
# BASE. `dflt` = `EvalWeights::default()` at the run's tip (round 70 in).
#
# STEP 0 is the paired wall clock of each arm against the base on sealed and
# cube.
#
# PRE-REGISTERED READINGS (pooled over four seeds; the r50 rule on cells).
#   * pooled >= 49.9 and no cell's interval wholly below 50  -> adopt on the
#     default only; of several passing arms, the cheapest at step 0.
#   * 49.5 <= pooled < 49.9                                  -> adopt only
#     if the wall-clock win is >= 10 % on both pools.
#   * pooled < 49.5                                          -> park.
# `bchain-seed` is a strength arm as much as a cost arm (it can extend
# greedy where the chain from nothing could not): a reading above 50.5
# pooled is adoptable on strength alone at any cost <= 1.0.
set -eu
cd "$(dirname "$0")/.."
LADDER=${LADDER:-./target/profiling-fast/bot_ladder}
GAMES=${GAMES:-1000}
COST_GAMES=${COST_GAMES:-200}
COST_REPS=${COST_REPS:-5}
ARMS=${ARMS:-"bchain-skipg bchain-empty bchain-seed"}
BASE=${BASE:-dflt}
THREADS=${THREADS:-3}
OUT=.ladder/r71

[ -x "$LADDER" ] || { echo "build it: cargo build --profile profiling-fast -p crabomination --bin bot_ladder" >&2; exit 1; }
mkdir -p "$OUT"

echo "=== step 0: paired wall clock, $COST_REPS reps x (dflt then each arm), $COST_GAMES x 12 ==="
for pool in sealed cube; do
  f="$OUT/cost_${pool}.txt"
  if ! [ -s "$f" ]; then
    for rep in $(seq 1 "$COST_REPS"); do
      for arm in $BASE $ARMS; do
        t=$($LADDER --a "$arm" --b "$arm" --decks "$pool" --games "$COST_GAMES" --seed 43 --threads "$THREADS" 2>&1 \
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
    print(f"  {sys.argv[2]:6} {arm:12} wall/{base} median {st.median(ratios):.3f}  mean {st.mean(ratios):.3f}  per-rep {[round(x, 3) for x in ratios]}")
PY
done

run () { # tag A B seed pool
  local tag="$1" a="$2" b="$3" seed="$4" pool="$5"
  if [ -s "$OUT/${tag}.txt" ] && grep -q "A win%" "$OUT/${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' "$OUT/${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  $LADDER --a "$a" --b "$b" --decks "$pool" --games "$GAMES" --seed "$seed" --threads "$THREADS" \
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
        p = pathlib.Path(f".ladder/r71/{arm}_l{l}.txt")
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
    print(f"  {arm:12} pooled {pooled:5.2f}%   cells {[x[0] for x in v]}   highs {[x[2] for x in v]}   verdict: {verdict}")
PY
echo "round-71 complete"
