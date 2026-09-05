#!/usr/bin/env bash
# Round 63: the two combat-window pickers that were still rule tables.
#
# WHY. Rounds 55–56 turned two rule-shaped pickers (the greedy attack
# accretion, the greedy block trade table) into sim-priced searches and
# read +2.3 and +5.8. Four instant-speed pickers were still pure rules
# with no simulation behind them; the two that live in the combat window
# are done here in the same shape, the two stack-window ones (counters)
# are counted but not changed (see the census).
#  * `trick_sim` (profile `trick-sim`): `pick_combat_trick` casts the first
#    pump that flips a fight one of our creatures is losing, reading one
#    (blocker, attacker) pair at a time — so a double block reads "already
#    winning" twice and gets nothing, an unblocked attacker's extra damage
#    is never priced, and removal at a blocker is not its shape at all.
#    The scored picker offers every affordable pump at every own creature
#    in combat and every affordable removal at every opposing creature in
#    combat, runs each through the rest of the combat, and takes a strict
#    improvement over the rule's own line (candidate 0); a tie returns the
#    rule's pick. Pinned by
#    `trick_sim_pumps_the_double_blocked_attacker_the_rule_skips`.
#  * `removal_sim` (profile `removal-sim`): `pick_defensive_removal` casts
#    the first removal that answers an attacker worth six units or more.
#    The scored picker offers every affordable removal at every attacker
#    it answers, runs each through the combat (greedy blocks inside the
#    sim) and casts only when it beats holding; a tie holds. Pinned by
#    `removal_sim_answers_the_lethal_chaff_attacker_the_rule_ignores`.
#
# FIRST READING (2026-09-05, cells in .ladder/r63_narrow/): both flags
# built on the rule pickers' own shape filters (`is_combat_trick` = pure
# temporary pumps, `removal_leaf` = first-leaf destroy/damage) and read
# exactly 50.0 — removal never fired (0 candidates in 600 games), the
# trick sim ran 5 sims. The window census showed WHY, and it was not
# mana: at the post-block window the seat had an untapped source 80 % of
# the time and an instant in hand 29 %; at the pre-block window 70 % /
# 29 %. A pool probe found the sealed pools' instants are exile, modal
# charms, +1/+1-counter pumps, pumps with riders, tap effects and
# conditional damage — 1 pure pump and 2 first-leaf removal in 17
# instants on seed 43, 0 and 3 in 22 on seed 97. The shape filters, not
# the pricing, were the hole. SECOND READING: the generator is now every
# affordable instant in hand at every creature in the combat (plus the
# main-phase enumerator's modal / X / untargeted shapes), and the sim
# decides (`combat_instant_candidates`). Pinned by
# `removal_sim_exiles_the_dragon_the_shape_filter_never_offered`.
#
# BASE. `dflt56` = the frozen round-56 default (both chains); one flag
# per cell. No net leg: round 62 read the scored net at +0.05 over the
# heuristic, so it would only add noise.
#
# MEASUREMENT CLASS. As r55/r56: the sims draw no jitter, so a game where
# the scored pick equals the rule's plays identically; incidence is per
# combat window where a candidate exists (a trick or a removal in hand
# with the mana up), which the census reads first.
#
# BIAS, recorded. The removal sim's "hold" line declares GREEDY blocks
# inside the sim while the real bot then runs the block chain, so holding
# is under-priced and the sim leans toward casting; the trick sim's
# candidate 0 includes the rule's own cast through the sim's spell layer,
# so it leans toward the rule. Both lean toward the status quo of their
# decision, which is the safe direction for a first reading.
#
# PRE-REGISTERED READINGS (per leg, pooled over four seeds).
#   * clearly above 50.5            -> adopt.
#   * 50.0-50.5, intervals clear 50 -> adopt on replication (r50 rule).
#   * exactly 50.0 ±0.00            -> never fired; read the census.
#   * clearly below 50              -> the sim's combat-only horizon
#                                      mis-prices the card: a trick or a
#                                      removal spent now is a card not held
#                                      for a bigger fight, which a walk
#                                      that ends at end of combat cannot
#                                      see. Park; the fix is horizon, not
#                                      candidates.
set -eu
cd /home/archuser/repos/Crabomination
LADDER=${LADDER:-./target/release-fast/bot_ladder}
GAMES=${GAMES:-1000}
[ -x "$LADDER" ] || { echo "build it: cargo build --profile release-fast -p crabomination --bin bot_ladder" >&2; exit 1; }

for arm in trick-sim removal-sim; do
  f=".ladder/r63_census_${arm}.txt"
  if ! [ -s "$f" ]; then
    echo "=== step 0: incidence census, $arm vs dflt56 (50 games, seed 43) ==="
    CRAB_ATTACK_CENSUS=1 $LADDER --a "$arm" --b dflt56 --decks sealed --games 50 --seed 43 > "$f" 2>&1 || true
  fi
  grep -E "response_census|decided" "$f" || echo "  (no census line in $f)"
done

run () { # tag A B seed
  local tag="$1" a="$2" b="$3" seed="$4"
  if [ -s ".ladder/r63_${tag}.txt" ] && grep -q "A win%" ".ladder/r63_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r63_${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  $LADDER --a "$a" --b "$b" --decks sealed --games "$GAMES" --seed "$seed" > ".ladder/r63_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r63_${tag}.txt" | tail -1 | tr -s ' ')  $(grep -E 'decided' ".ladder/r63_${tag}.txt" | tail -1)"
}

echo "=== trick-sim vs dflt56 ==="
for lseed in 43 97 151 199; do run "trick_l${lseed}" trick-sim dflt56 "$lseed"; done
echo "=== removal-sim vs dflt56 ==="
for lseed in 43 97 151 199; do run "removal_l${lseed}" removal-sim dflt56 "$lseed"; done

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
def cells(prefix):
    v = []
    for l in (43, 97, 151, 199):
        p = pathlib.Path(f".ladder/r63_{prefix}_l{l}.txt")
        if not p.exists(): continue
        m = re.findall(r"A win%\s+([0-9.]+)%\s+\[([0-9.]+)%, ([0-9.]+)%\]", p.read_text())
        if m: v.append(tuple(float(x) for x in m[-1]))
    return v
for leg in ("trick", "removal"):
    v = cells(leg)
    if not v: continue
    pooled = st.mean(x[0] for x in v)
    if all(abs(x[0] - 50.0) < 0.05 for x in v): verdict = "never fired — read the census"
    elif pooled < 49.5: verdict = "NEGATIVE — horizon mis-prices the card; park"
    elif pooled > 50.5: verdict = "clear win"
    elif all(x[1] > 50.0 for x in v): verdict = "small and replicated — every cell's interval clears 50 (r50 rule)"
    else: verdict = "small/null — intervals straddle 50"
    print(f"  {leg:8s} pooled {pooled:5.2f}%   cells {[x[0] for x in v]}   lows {[x[1] for x in v]}")
    print(f"           verdict: {verdict}")
PY
echo "round-63 complete"
