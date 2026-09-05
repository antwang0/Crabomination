#!/usr/bin/env bash
# Round 56: the two chains the attack chain (r55, +2.3) left on the table.
#
# WHAT CHANGED.
#  * `attack_chain_wide` (profile `atk-chain-wide`): the r55 chain never ran
#    when greedy declared NOBODY — a one-candidate menu returned before any
#    sim — which is exactly the board where every creature was refused by a
#    per-creature rule and none was ever priced by the sim; and forward
#    growth is blind to the overload (two attackers into one blocker connect
#    where each alone is blocked and traded, so the first step ties and the
#    chain stops). On, the chain runs from an empty greedy and its first
#    step offers every pair beside the singles. Pinned by
#    `attack_chain_wide_overloads_the_lone_blocker_greedy_holds_against`.
#  * `block_chain` (profile `blk-chain`): the block twin. Each step offers
#    one (blocker, attacker) pair per free blocker and legal attacker, plus
#    a per-attacker GANG move (the cheapest free blockers that together
#    kill it — the pair-level step single growth cannot take, since each
#    gang member is a chump alone), priced by the block sim; the finished
#    plan joins the block menu's argmax. It also runs on the bare "no
#    blocks" menu, where `block_candidates_for_mcts` never generated gang
#    candidates at all (greedy had nothing to seed them with) — so a gang
#    that only pays as a pair was unreachable exactly when it mattered.
#    Pinned by `block_chain_finds_the_gang_block_the_bare_menu_cannot` and
#    `block_chain_reaches_a_double_gang_the_menu_cannot`.
#
# BASE. `dflt55` = `EvalWeights::round55_default()`, the default as it stood
# after round 55 (frozen, so this gate stays re-runnable after adoption —
# the r52 precedent); `gang` no longer is the default. Every heuristic cell
# is flag-on-r55-default vs r55-default. Net leg: on `net-chain` (det1 +
# attack chain) vs `net-chain`. (The recorded cells ran as `dflt` before
# the profile was split; the two were identical at the time.)
#
# MEASUREMENT CLASS. As r55: the sims draw no jitter, so a game where the
# new candidate never wins an argmax plays identically and the mirror
# survives; incidence is per combat, so expect r55's ±0.33–0.40 cells,
# not dmgorder's ±0.22. Four seeds per heuristic leg, two per net leg.
#
# COST is the open question for the block chain: it runs on EVERY block
# step the menu found trivial, at (free blockers × attackers) sims a step.
# Step 0 records both censuses (sims per search) and each cell file
# records wall-clock; a win that doubles block-step cost still lands, but
# the number goes to the round entry and PERF's candidates.
#
# PRE-REGISTERED READINGS (per leg, pooled over its seeds).
#   * clearly above 50.5            -> adopt (the menu-hole class).
#   * 50.0-50.5, intervals clear 50 -> adopt on replication across the
#                                      leg's seeds (r50 rule).
#   * exactly 50.0 ±0.00            -> never fired; read the census.
#   * clearly below 50              -> the new candidates lose when they
#                                      win the argmax — for the block
#                                      chain most plausibly the block sim's
#                                      horizon (it stops at end of combat,
#                                      so a gang that wins the exchange but
#                                      taps out of the crack-back is priced
#                                      as pure profit); for the wide chain
#                                      the pair move's C(n,2) sims buying
#                                      an over-attack the r55 census
#                                      already saw greedy make. Park; do
#                                      not iterate the caps.
set -eu
cd /home/archuser/repos/Crabomination
LADDER=${LADDER:-./target/release-fast/bot_ladder}
NET=nets/champion.safetensors
GAMES=${GAMES:-1000}

[ -x "$LADDER" ] || { echo "build it: cargo build --profile release-fast -p crabomination --bin bot_ladder" >&2; exit 1; }
[ -f "$NET" ] || { echo "missing $NET" >&2; exit 1; }

for arm in atk-chain-wide blk-chain; do
  f=".ladder/r56_census_${arm}.txt"
  if ! [ -s "$f" ]; then
    echo "=== step 0: incidence census, $arm vs dflt55 (50 games, seed 43) ==="
    CRAB_ATTACK_CENSUS=1 $LADDER --a "$arm" --b dflt55 --decks sealed --games 50 --seed 43 > "$f" 2>&1 || true
  fi
  grep -E "attack_census|block_census|decided" "$f" || echo "  (no census line in $f)"
done

run () { # tag A B seed
  local tag="$1" a="$2" b="$3" seed="$4"
  if [ -s ".ladder/r56_${tag}.txt" ] && grep -q "A win%" ".ladder/r56_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r56_${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  CRAB_NET="$NET" $LADDER --a "$a" --b "$b" \
      --decks sealed --games "$GAMES" --seed "$seed" \
      > ".ladder/r56_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r56_${tag}.txt" | tail -1 | tr -s ' ')  $(grep -E 'decided' ".ladder/r56_${tag}.txt" | tail -1)"
}

echo "=== atk-chain-wide vs dflt55 ==="
for lseed in 43 97 151 199; do run "wide_l${lseed}" atk-chain-wide dflt55 "$lseed"; done
echo "=== blk-chain vs dflt55 ==="
for lseed in 43 97 151 199; do run "blk_l${lseed}" blk-chain dflt55 "$lseed"; done
echo "=== net-chain-wide vs net-chain ==="
for lseed in 43 97; do run "netwide_l${lseed}" net-chain-wide net-chain "$lseed"; done
echo "=== net-bchain vs net-chain ==="
for lseed in 43 97; do run "netblk_l${lseed}" net-bchain net-chain "$lseed"; done

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
def cells(prefix, seeds):
    v = []
    for l in seeds:
        p = pathlib.Path(f".ladder/r56_{prefix}_l{l}.txt")
        if not p.exists():
            continue
        m = re.findall(r"A win%\s+([0-9.]+)%\s+\[([0-9.]+)%, ([0-9.]+)%\]", p.read_text())
        if m:
            v.append(tuple(float(x) for x in m[-1]))
    return v
for leg, seeds in (("wide", (43, 97, 151, 199)), ("blk", (43, 97, 151, 199)), ("netwide", (43, 97)), ("netblk", (43, 97))):
    v = cells(leg, seeds)
    if not v:
        continue
    pooled = st.mean(x[0] for x in v)
    if all(abs(x[0] - 50.0) < 0.05 for x in v):
        verdict = "never fired — read the census"
    elif pooled < 49.5:
        verdict = "NEGATIVE — park"
    elif pooled > 50.5:
        verdict = "clear win"
    elif all(x[1] > 50.0 for x in v):
        verdict = "small and replicated — every cell's interval clears 50 (r50 rule)"
    else:
        verdict = "small/null — intervals straddle 50"
    print(f"  {leg:8s} pooled {pooled:5.2f}%   cells {[x[0] for x in v]}   lows {[x[1] for x in v]}")
    print(f"           verdict: {verdict}")
PY
echo "round-56 complete"
