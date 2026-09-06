#!/usr/bin/env bash
# Round 65: the greedy attack filter judged against eligible blockers
# (`attack_blocker_guard`, profile `atk-guard` = the default + the flag).
#
# WHAT CHANGED. Forty human-vs-bot client games (2026-09-01, the client's
# own `net_eval_det1`/64 pilot) showed the bot's attackers dying for
# nothing seven times in 130 attacks — a 2/1 flier into a 5/5 flier, a
# 1/1 flier into a 7/7 reach trampler, six attackers into six untapped
# blockers for four dead. A probe of the greedy filter found the holes:
# fliers skip the suicide check whenever they are fliers (the check reads
# ground blockers only), ground attackers are never held from an opposing
# flier, trample and lifelink bypass the check outright, and "lethal"
# counts raw power with no blocker subtracted. The adopted default's
# material-leaf sims corrected every probe board; the client's net leaf
# corrected four of six. The guard closes the holes in greedy itself:
# per-attacker eligible blockers, hold a pure suicide racing or not, hold
# a trade unless racing, lethal = damage through every chump.
#
# WHY GATE. Greedy is the start of every searched menu AND the declaration
# both seats take inside every sim and rollout, so the guard changes the
# modelled opponent as well as the bot's own opening. The chain can re-add
# anything the guard holds, on a priced sim.
#
# PRE-REGISTERED READINGS.
#  Stage 1, heuristic level (`atk-guard` vs `dflt`, four seeds, 1000 paired
#  games = 12 000 each): pooled >= 50.0 and no cell's interval wholly
#  below 50 -> adopt on the default (the client-quality case stands on the
#  replays, the r54 shape); any cell wholly below 50 or pooled < 49.7 ->
#  the sims' modelled opponent got worse to play against — investigate
#  before adopting.
#  Stage 2, the client's shape (`mcts-guard-256` vs `dflt`, seeds 43/97,
#  500 games x 12 decks, +-0.95): the reference is round 64's
#  `mcts-dflt-256` vs `dflt` = 56.2 / 54.3. Within +-1 of it -> the guard
#  is neutral for the search, adoption rests on stage 1; >= +1 on both
#  seeds -> the rollout-policy effect is real, record it; <= -1 on both ->
#  the guard hurts the search, park it off the default.
#  Step 0 records the attack census and the paired wall clock on both.
set -eu
cd "$(dirname "$0")/.."
LADDER=${LADDER:-./target/release-fast/bot_ladder}
GAMES=${GAMES:-1000}
GAMES_MCTS=${GAMES_MCTS:-500}
COST_GAMES=${COST_GAMES:-200}
COST_REPS=${COST_REPS:-3}
[ -x "$LADDER" ] || { echo "build it: cargo build --profile release-fast -p crabomination --bin bot_ladder" >&2; exit 1; }

run () { # tag A B seed games
  local tag="$1" a="$2" b="$3" seed="$4" games="$5"
  if [ -s ".ladder/r65_${tag}.txt" ] && grep -q "A win%" ".ladder/r65_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r65_${tag}.txt" | tail -1 | tr -s ' ')  $(grep -E 'decided' ".ladder/r65_${tag}.txt" | tail -1)"
    return
  fi
  $LADDER --a "$a" --b "$b" --decks sealed --games "$games" --seed "$seed" > ".ladder/r65_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r65_${tag}.txt" | tail -1 | tr -s ' ')  $(grep -E 'decided' ".ladder/r65_${tag}.txt" | tail -1)"
}

STAGE=${STAGE:-all}
if [ "$STAGE" = all ] || [ "$STAGE" = quick ]; then
  echo "=== step 0a: attack census (sealed, $COST_GAMES games, seed 43) ==="
  for arm in dflt atk-guard; do
    f=".ladder/r65_census_${arm}.txt"
    [ -s "$f" ] || CRAB_ATTACK_CENSUS=1 $LADDER --a "$arm" --b "$arm" --decks sealed --games "$COST_GAMES" --seed 43 > "$f" 2>&1 || true
    echo "  $arm: $(grep -E 'attack_census' "$f" | head -1 | cut -c1-400)"
  done
  echo "=== step 0b: paired wall clock, $COST_REPS reps ==="
  f=".ladder/r65_cost.txt"
  if ! [ -s "$f" ]; then
    for rep in $(seq 1 "$COST_REPS"); do for arm in dflt atk-guard; do
      t=$($LADDER --a "$arm" --b "$arm" --decks sealed --games "$COST_GAMES" --seed 43 2>&1 | grep -E 'decided' | tail -1 | sed -E 's/.* in ([0-9.]+)s.*/\1/')
      echo "$rep $arm $t" >> "$f"
    done; done
  fi
  python3 - "$f" <<'PY'
import sys, statistics as st, collections
by = collections.defaultdict(dict)
for l in open(sys.argv[1]):
    if l.strip(): rep, arm, t = l.split(); by[int(rep)][arm] = float(t)
r = [by[k]["atk-guard"] / by[k]["dflt"] for k in by if len(by[k]) == 2]
print(f"  atk-guard wall / dflt: median {st.median(r):.3f}  per-rep {[round(x, 3) for x in r]}")
PY
  echo "=== stage 1: atk-guard vs dflt ==="
  for s in 43 97 151 199; do run "guard_l$s" atk-guard dflt "$s" "$GAMES"; done
fi
if [ "$STAGE" = all ] || [ "$STAGE" = search ]; then
  echo "=== stage 2: mcts-guard-256 vs dflt (reference mcts-dflt-256: 56.2 / 54.3) ==="
  for s in 43 97; do run "search_l$s" mcts-guard-256 dflt "$s" "$GAMES_MCTS"; done
fi

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
def cells(prefix, seeds):
    v = []
    for l in seeds:
        p = pathlib.Path(f".ladder/r65_{prefix}_l{l}.txt")
        if not p.exists(): continue
        m = re.findall(r"A win%\s+([0-9.]+)%\s+\[([0-9.]+)%, ([0-9.]+)%\]", p.read_text())
        if m: v.append(tuple(float(x) for x in m[-1]))
    return v
v = cells("guard", (43, 97, 151, 199))
if v:
    pooled = st.mean(x[0] for x in v); below = [x for x in v if x[2] < 50.0]
    verdict = "adopt on the default" if (pooled >= 50.0 and not below) else ("investigate before adopting" if (pooled < 49.7 or below) else "no loss (pooled just under 50) — adopt on the replay case")
    print(f"  stage 1  atk-guard vs dflt  pooled {pooled:5.2f}  cells {[x[0] for x in v]}  lows {[x[1] for x in v]}  highs {[x[2] for x in v]}  -> {verdict}")
v = cells("search", (43, 97))
if v:
    ref = {43: 56.2, 97: 54.3}
    d = [round(x[0] - r, 2) for x, r in zip(v, [56.2, 54.3])]
    print(f"  stage 2  mcts-guard-256 vs dflt  cells {[x[0] for x in v]}  vs reference {list(ref.values())}  deltas {d}")
PY
echo "round-65 complete"
