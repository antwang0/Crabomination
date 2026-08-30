#!/usr/bin/env bash
# Round 52: does value-ordered combat damage (CR 510.1c) pay?
#
# WHAT CHANGED. `Decision::CombatDamageOrder` had no policy arm at all —
# not in `decide_pending_policy`, not in the search. Every multi-blocked
# attacker in this program's history dealt lethal in declaration (CardId)
# order, and the 510.1e mirror (a creature blocking two attackers) divided
# its damage the same way. "Kill the better blocker" was not expressible
# at any valuation: the same missing-candidate shape as chump blocks
# (r43, +0.9), one step later in the same combat. The `damage_order` flag
# (profile `dmgorder`) answers the decision by simulating the engine's
# own default split per candidate order (exhaustive to five victims) and
# keeping the best signed `permanent_value` outcome for the deciding
# seat — signed, because banding and Defensive Formation hand the choice
# to the victims' controller, whose best order is the reverse.
#
# The `AssignCombatDamage` sibling deliberately stays on the engine
# default: lethal-to-each-then-trample is already the assigner's optimum
# outside the banding deny-trample corner, and the pool has no banding.
#
# MEASUREMENT CLASS. Heuristic-level, not search-level: the flag consumes
# no extra randomness, and an order that only ties the default answers
# empty, so a game where the choice cannot matter plays identically and
# the antithetic mirror survives (the r50/r51 free-precision class,
# expected ~±0.1 at 6000 pairs, NOT the ±0.6 search class). Gang blocks
# are the trigger, so incidence rides the `gang` profile's own block
# search — see the incidence note below before reading a null as
# "doesn't fire".
#
# INCIDENCE (measured first, r50 discipline): see .ladder/r52_probe.txt —
# `bot_probe --deck sos` counts CombatDamageOrder asks per game. If the
# ladder reads exactly 50.0 ±0.00, the flag never fired: check the probe
# count before believing the decision is dead (a wants_ui or
# decide_pending_policy wiring break is likelier).
#
# PRE-REGISTERED READINGS.
#   * clearly above 50.5            -> adopt; the menu-hole class pays again.
#   * 50.0-50.5, intervals clear 50 -> real and small; adopt on replication
#                                      across both seeds (r50 rule).
#   * exactly 50.0 ±0.00            -> never fired; check wiring vs probe.
#   * clearly below 50              -> the one-step value read is worse than
#                                      declaration order — most plausible
#                                      via the signed value misjudging a
#                                      trade the sims would have priced;
#                                      park, do not iterate weights.
set -eu
cd /home/archuser/repos/Crabomination
LADDER=./target/release/bot_ladder
GAMES=${GAMES:-1000}

[ -x "$LADDER" ] || { echo "build it: cargo build --release -p crabomination --bin bot_ladder" >&2; exit 1; }

run () { # tag A B seed
  local tag="$1" a="$2" b="$3"
  if [ -s ".ladder/r52_${tag}.txt" ] && grep -q "A win%" ".ladder/r52_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r52_${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  $LADDER --a "$a" --b "$b" \
      --decks sealed --games "$GAMES" --seed "$4" \
      > ".ladder/r52_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r52_${tag}.txt" | tail -1 | tr -s ' ')"
}

echo "=== dmgorder vs gang (the shipped default) ==="
for lseed in 43 97; do
  run "dmgorder_l${lseed}" dmgorder gang "$lseed"
done

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
v = []
for l in (43, 97):
    p = pathlib.Path(f".ladder/r52_dmgorder_l{l}.txt")
    if not p.exists():
        continue
    m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text())
    if m:
        v.append(float(m[-1]))
if v:
    pooled = st.mean(v)
    if all(abs(x - 50.0) < 0.05 for x in v):
        verdict = "flag never fired — check the probe count and the wants_ui path"
    elif pooled < 49.5:
        verdict = "NEGATIVE — declaration order beats the one-step value read"
    elif pooled > 50.5:
        verdict = "clear win"
    else:
        verdict = "small; adopt only if BOTH cells' intervals clear 50 (r50 rule)"
    print(f"  dmgorder pooled {pooled:5.2f}%   cells {v}")
    print(f"    verdict: {verdict}")
PY
echo "round-52 complete"
