#!/usr/bin/env bash
# Round 50: does fixing the planeswalker cash-out read pay?
#
# WHAT CHANGED. `pick_loyalty_ability`'s guard restricted the bot to
# loyalty-SPENDING abilities whenever total enemy creature power met
# current loyalty. That fired essentially always past the opening turns,
# so the bot dumped loyalty the turn a walker landed and let it die:
# **zero plus activations against eight minuses** across every recorded
# human game (2026-08-23), including Ral Zarek spending its last point
# to strip one card and die with a +1 available. The read now counts
# only creatures that could actually attack, subtracts our untapped
# blockers, and compares against the loyalty the walker would have AFTER
# its best plus.
#
# A = `gang` (the fix, since it is unflagged and on by default)
# B = `walkerlegacy` (the pre-fix read)
# A above 50 means the fix wins.
#
# POWER WARNING, STATED FIRST BECAUSE IT IS THE LIKELY OUTCOME.
# This flag can only move a game that contains a planeswalker, and they
# are rare here: `bot_probe --deck sos` counted **54 loyalty activations
# over 300 games**, i.e. ~0.18 per game, and that is a constructed SOS
# field. Sealed pools will not be denser. Five flags in the last three
# rounds (buff2for1, convlands, walkerchip, impulse, and abilarms'
# quieter cousin) returned exactly 50.0 because the mirror never reached
# the situation at all.
#
# So the pre-registered readings are:
#   * exactly 50.0 on both seeds -> ZERO INCIDENCE. The ladder never put
#     a walker on the board with a live choice. Measures nothing; the fix
#     stands on the replay evidence and the two regression tests.
#   * 50.0-50.5 -> low incidence. Consistent with a real per-walker
#     improvement that the deck field is too thin to resolve. NOT
#     evidence the fix is worthless.
#   * clearly above 50.5 -> the fix pays even at this density, which
#     would be a pleasant surprise.
#   * clearly BELOW 50 -> the new read is wrong and the old
#     always-cash-out was accidentally right. That is the outcome worth
#     spending the cells on, and the reason to run this rather than
#     assume.
#
# The asymmetry is the point: this gate is cheap (heuristic profiles,
# ~0.2 s/game) and its job is to catch a regression, not to certify a
# win it is underpowered to see.
set -eu
cd /home/archuser/repos/Crabomination
LADDER=./target/release/bot_ladder
GAMES=${GAMES:-1000}

[ -x "$LADDER" ] || { echo "build the release ladder first" >&2; exit 1; }

echo "=== walker cash-out: gang (fixed) vs walkerlegacy (pre-fix) ==="
for lseed in 43 97; do
  tag="a_walker_l${lseed}"
  if [ -s ".ladder/r50_${tag}.txt" ] && grep -q "A win%" ".ladder/r50_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r50_${tag}.txt" | tail -1 | tr -s ' ')"
    continue
  fi
  $LADDER --a gang --b walkerlegacy --decks sealed --games "$GAMES" --seed "$lseed" \
      > ".ladder/r50_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r50_${tag}.txt" | tail -1 | tr -s ' ')"
done

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
v = []
for l in (43, 97):
    p = pathlib.Path(f".ladder/r50_a_walker_l{l}.txt")
    if not p.exists():
        continue
    m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text())
    if m:
        v.append(float(m[-1]))
if v:
    pooled = st.mean(v)
    if all(abs(x - 50.0) < 0.05 for x in v):
        verdict = "ZERO INCIDENCE — the field never produced a live walker choice"
    elif pooled < 49.5:
        verdict = "REGRESSION — the new read is worse than always cashing out"
    elif pooled > 50.5:
        verdict = "pays even at this walker density"
    else:
        verdict = "low incidence; underpowered by the deck field, not evidence against"
    print(f"  gang vs walkerlegacy   pooled {pooled:5.2f}%   cells {v}")
    print(f"  verdict: {verdict}")
PY
echo "round-50 complete"
