#!/usr/bin/env bash
# Round 48: re-gate target arms on a fixed ranking, and re-establish the
# champion's level now that the blocking deficit is gone.
#
# PART A — target arms, re-measured. Round 46 adopted them at +0.95
# (50.7 / 51.2). That gate ran with a flaw in the alternate ranking:
# alternates were ordered "opposite side from whatever the auto-targeter
# chose", which is right when the baked-in pick is our own permanent and
# exactly wrong when it is already correct — the common case — because
# then "opposite side" is our own board. So one of the two arms was
# routinely spent offering the search a self-target. Alternates are now
# filtered to the side the slot wants and ranked by body size.
#
#   The adoption is not in question: +0.95 was earned WITH the flaw, and
#   removing wasted arms cannot make the feature worse in expectation.
#   What this cell measures is how much of the feature was being thrown
#   away. Pre-registered expectation: +1.0 to +2.0 against
#   `mcts-net-noarms`, i.e. at or above the r46 figure. A result at or
#   below +0.95 would say the wasted arm was not costing anything, which
#   would be surprising and worth understanding rather than filing.
#
# PART B — what is the champion actually worth? Round 47 found the net
# profiles had been missing both adopted blocking layers, worth +3.2
# pilot-side, so every net-vs-heuristic level from rounds 26-46 is
# understated and none of them should be quoted as the net's standing
# strength. The band everyone cites (~52.7 vs atk-sim, ~51.2 vs gang) is
# stale. This re-measures it once, at the current tip, so the program has
# a number that means what it says. Not a gate — nothing is adopted or
# rejected on it — a reference measurement.
#
# WHAT IS DELIBERATELY NOT GATED, again. The hostile-value ranking
# ("prefer the biggest threat") lives in the engine's auto-targeter,
# which every profile shares, so it cannot be A/B'd inside one process
# the way a flag can. Measuring it would mean threading a weights flag
# through the ~10 bot call sites of `auto_targets_for_effect_all_slots`,
# or a thread-local that flips engine behaviour mid-game and risks
# leaking into the opponent's decisions and the sims. Neither is worth
# it for a change whose direction is not in doubt; it is justified by
# the replay and by `hostile_auto_target_prefers_the_biggest_threat`.
# Recording the cost of measuring it here so the choice is visible
# rather than silent: if it is ever suspect, that is the price.
set -eu
cd /home/archuser/repos/Crabomination
LADDER=./target/release/bot_ladder
NET=nets_r41_v7_s151/best.safetensors
GAMES=${GAMES:-1000}

[ -f "$NET" ] || { echo "missing $NET" >&2; exit 1; }
[ -x "$LADDER" ] || { echo "build the release ladder first" >&2; exit 1; }

run () {
  local tag="$1" a="$2" b="$3" seed="$4"
  if [ -s ".ladder/r48_${tag}.txt" ] && grep -q "A win%" ".ladder/r48_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r48_${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  CRAB_NET="$NET" $LADDER --a "$a" --b "$b" \
      --decks sealed --games "$GAMES" --seed "$seed" \
      > ".ladder/r48_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r48_${tag}.txt" | tail -1 | tr -s ' ')"
}

echo "=== Part B: the champion's level, post-rebase (reference, runs first — it is cheap) ==="
for lseed in 43 97; do
  run "b_net_vs_atksim_l${lseed}" net atk-sim "$lseed"
  run "b_net_vs_gang_l${lseed}"   net gang    "$lseed"
done

echo "=== Part A: target arms re-gated on the fixed ranking ==="
for lseed in 43 97; do
  run "a_rearm_l${lseed}" mcts-net-deep mcts-net-noarms "$lseed"
done

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
def cells(tag, seeds=(43, 97)):
    v = []
    for l in seeds:
        p = pathlib.Path(f".ladder/r48_{tag}_l{l}.txt")
        if not p.exists():
            continue
        m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text())
        if m:
            v.append(float(m[-1]))
    return v
for label, tag in (
    ("A target arms (vs noarms)", "a_rearm"),
    ("B net vs atk-sim", "b_net_vs_atksim"),
    ("B net vs gang", "b_net_vs_gang"),
):
    v = cells(tag)
    if v:
        print(f"  {label:<28} pooled {st.mean(v):5.2f}%   cells {v}")
print("\n  Reference for Part B — the STALE pre-rebase band was ~52.7 (atk-sim)")
print("  and ~51.2 (gang); those were measured with the net missing both")
print("  adopted blocking layers (round 47). Quote the new numbers.")
PY
echo "round-48 complete"
