#!/usr/bin/env bash
# Round 55: does growing the attack declaration one creature at a time pay?
#
# WHAT CHANGED. `pick_attacks_scored` searches a menu the greedy picker
# seeds: greedy, nobody, and greedy-minus-one holdbacks. That menu can only
# DROP attackers, so a declaration smaller than greedy-minus-one, or one
# carrying a creature the greedy filters refused (a body the suicide rule
# held that the sim would price as free), is unreachable at any
# valuation — the same missing-candidate shape as gang blocks (+1.3) and
# chump blocks (+0.9), one decision earlier. The `attack_chain` flag
# (profiles `atk-chain` on the gang base, `net-chain` on the net pilot)
# grows a declaration from the repaired "nobody" one creature at a time:
# each step simulates "the set so far plus one more eligible creature" for
# every remaining creature, with "finalize" as candidate 0 (ties stop the
# chain, first-wins-ties), keeps a strict improvement, and stops at six
# additions. The finished set then joins the menu for the picker's ONE
# argmax, where greedy still holds index 0 and every tie — so the chain
# can only ever change a declaration by strictly out-scoring the whole
# menu on the same sim, and the forward blind spot (two attackers that
# only pay together are each bad alone) is covered by greedy's alpha
# strike still being on the menu (pinned by
# `attack_chain_stops_at_nobody_where_only_the_pair_is_lethal`).
#
# MEASUREMENT CLASS. Heuristic-level RNG-wise: the sims draw no jitter
# (the redeal is seeded per candidate), so a game where the chain's set
# never wins the argmax plays identically and the antithetic mirror
# survives (the r52 free-precision class). Incidence is HIGHER than
# dmgorder's 0.12/game — every searched declaration runs the chain — so
# expect more divergent pairs and a wider interval than ±0.2; four seeds
# on the gang leg to be safe (r51 budget rule), two on the net leg.
#
# INCIDENCE (r50 discipline): step 0 runs the attack census on 50 games
# per seat and records how often the chain proposed a set the menu lacked
# and how often that set won — .ladder/r55_census.txt. If the ladder
# reads exactly 50.0 ±0.00, the chain never won an argmax: read the
# census before believing the idea is dead.
#
# COST: every step is up to `eligible` simulated turn cycles on top of the
# menu's ≤8; the census line's candidates/search and the wall-clock in
# each cell file are the price. A win that costs 2× the combat search is
# still a win here (the ladder is 6 s a cell) but goes to PERF before any
# default flip.
#
# PRE-REGISTERED READINGS (pooled over seeds, each cell's paired interval).
#   * clearly above 50.5            -> adopt; the menu-hole class pays again.
#   * 50.0-50.5, intervals clear 50 -> real and small; adopt on replication
#                                      across all four gang seeds (r50 rule).
#   * exactly 50.0 ±0.00            -> never fired; check the census.
#   * clearly below 50              -> the chain's sets are worse when they
#                                      win: most plausibly the sim's
#                                      standing blindness (it casts nothing
#                                      for the defender) pricing a
#                                      greedy-refused attacker as free.
#                                      Park; do not iterate the cap.
#   * gang leg and net leg disagree -> record both; the net leg is the
#                                      client's pilot and decides the
#                                      client default, the gang leg the
#                                      EvalWeights default.
set -eu
cd /home/archuser/repos/Crabomination
LADDER=${LADDER:-./target/release-fast/bot_ladder}
NET=nets/champion.safetensors
GAMES=${GAMES:-1000}

[ -x "$LADDER" ] || { echo "build it: cargo build --profile release-fast -p crabomination --bin bot_ladder" >&2; exit 1; }
[ -f "$NET" ] || { echo "missing $NET" >&2; exit 1; }

if ! [ -s .ladder/r55_census.txt ]; then
  echo "=== step 0: incidence census (50 games, seed 43) ==="
  CRAB_ATTACK_CENSUS=1 $LADDER --a atk-chain --b gang --decks sealed --games 50 --seed 43 \
      > .ladder/r55_census.txt 2>&1 || true
fi
grep -E "attack_census" .ladder/r55_census.txt || echo "  (no census line — check CRAB_ATTACK_CENSUS wiring)"

run () { # tag A B seed [env]
  local tag="$1" a="$2" b="$3" seed="$4"
  if [ -s ".ladder/r55_${tag}.txt" ] && grep -q "A win%" ".ladder/r55_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r55_${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  CRAB_NET="$NET" $LADDER --a "$a" --b "$b" \
      --decks sealed --games "$GAMES" --seed "$seed" \
      > ".ladder/r55_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r55_${tag}.txt" | tail -1 | tr -s ' ')"
}

echo "=== atk-chain vs gang (the EvalWeights default's base) ==="
for lseed in 43 97 151 199; do
  run "gang_l${lseed}" atk-chain gang "$lseed"
done

echo "=== net-chain vs net-det1 (the client pilot, minus the flag) ==="
for lseed in 43 97; do
  run "net_l${lseed}" net-chain net-det1 "$lseed"
done

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
def cells(prefix, seeds):
    v = []
    for l in seeds:
        p = pathlib.Path(f".ladder/r55_{prefix}_l{l}.txt")
        if not p.exists():
            continue
        m = re.findall(r"A win%\s+([0-9.]+)%\s+\[([0-9.]+)%, ([0-9.]+)%\]", p.read_text())
        if m:
            v.append(tuple(float(x) for x in m[-1]))
    return v
for leg, seeds in (("gang", (43, 97, 151, 199)), ("net", (43, 97))):
    v = cells(leg, seeds)
    if not v:
        continue
    pooled = st.mean(x[0] for x in v)
    if all(abs(x[0] - 50.0) < 0.05 for x in v):
        verdict = "never fired — read .ladder/r55_census.txt"
    elif pooled < 49.5:
        verdict = "NEGATIVE — the chain's sets lose when they win the argmax; park"
    elif pooled > 50.5:
        verdict = "clear win"
    elif all(x[1] > 50.0 for x in v):
        verdict = "small and replicated — every cell's interval clears 50 (r50 rule)"
    else:
        verdict = "small/null — intervals straddle 50"
    print(f"  {leg:4s} pooled {pooled:5.2f}%   cells {[x[0] for x in v]}   lows {[x[1] for x in v]}")
    print(f"       verdict: {verdict}")
PY
echo "round-55 complete"
