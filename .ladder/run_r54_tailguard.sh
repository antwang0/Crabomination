#!/usr/bin/env bash
# Round 54: does the saturation fallback pay on the ladder?
#
# WHAT CHANGED. The scored combat pickers (`pick_attacks_scored`,
# `pick_blocks_scored`) score their candidates' settled leaves with the
# champion net under the net profiles — and a sigmoid's sensitivity is
# p·(1−p), so outside the calibrate histogram's own rankable band
# [0.05, 0.95] ("the search cannot rank lines inside a saturated band")
# every candidate collapses to the same integer and first-wins-ties
# hands the decision to the eval-free greedy menu default. Observed in
# the wild (2026-08-30 client replays, game 5 turns 18/20): a flying
# 5/5 the defender could not block, held for two turns of a lost race
# at 3-8 life, everything untapped, tap-verified. The `net_tail_guard`
# flag (profile `net-guard`) evaluates the PRE-DECISION state once and,
# on a saturated read, silences the net for that one decision so every
# candidate scores on the material eval — linear, still discriminative.
# Mid-band decisions are untouched; the decided-game clamp already
# implements this principle at p ∈ {0, 1}.
#
# MEASUREMENT CLASS AND THE PRE-REGISTERED CAVEAT. Heuristic-level (no
# search RNG perturbed; fires only at real scored combat decisions), so
# mirrors stay near-free precision. BUT: both seats of a mirror
# saturate together, and saturated positions are mostly already decided
# — the flag's wins should concentrate in the thin band of decided-
# looking-but-racy games (game 5 was one). A small or null ladder
# number therefore does NOT refute the client-quality case, which
# stands on the replay evidence — the determinized-search shape, where
# the ladder was structurally unable to price honesty. Decide adoption
# on: positive gate -> adopt; null gate -> adopt-or-not as a CLIENT
# default question with game 5 as the regression scenario, recorded
# either way.
#
# INCIDENCE PROXY (r50 discipline; a direct probe needs a net-loaded
# run): the round-11 calibrate histogram put 3.3-12.8 % of recorded
# positions outside [0.05, 0.95] depending on encoder; the champion's
# own figure is unmeasured. If the ladder reads exactly 50.0 ±0.00 the
# flag never fired — check wiring against `tail_guard_tests` before
# believing it.
#
# READINGS.
#   * clearly above 50.5            -> adopt.
#   * 50.0-50.5, intervals clear 50 -> real and small; adopt on
#                                      replication (r50 rule).
#   * null                          -> decide as the client-default
#                                      question above; do NOT iterate
#                                      the threshold looking for a
#                                      number (it is anchored to the
#                                      calibrate band on purpose).
#   * clearly below 50              -> the material eval is worse than a
#                                      flat net in the tails — surprising
#                                      and worth a decision-log look
#                                      before anything else.
set -eu
cd /home/archuser/repos/Crabomination
LADDER=./target/release/bot_ladder
NET=nets/champion.safetensors
GAMES=${GAMES:-1000}

[ -f "$NET" ] || { echo "missing $NET" >&2; exit 1; }
[ -x "$LADDER" ] || { echo "build it: cargo build --release -p crabomination --bin bot_ladder" >&2; exit 1; }

run () { # tag A B seed
  local tag="$1" a="$2" b="$3"
  if [ -s ".ladder/r54_${tag}.txt" ] && grep -q "A win%" ".ladder/r54_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r54_${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  CRAB_NET="$NET" $LADDER --a "$a" --b "$b" \
      --decks sealed --games "$GAMES" --seed "$4" \
      > ".ladder/r54_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r54_${tag}.txt" | tail -1 | tr -s ' ')"
}

echo "=== net-guard vs net-det1 (the client pilot, minus the flag) ==="
for lseed in 43 97; do
  run "tailguard_l${lseed}" net-guard net-det1 "$lseed"
done

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
v = []
for l in (43, 97):
    p = pathlib.Path(f".ladder/r54_tailguard_l{l}.txt")
    if not p.exists():
        continue
    m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text())
    if m:
        v.append(float(m[-1]))
if v:
    pooled = st.mean(v)
    if all(abs(x - 50.0) < 0.05 for x in v):
        verdict = "flag never fired — check wiring vs tail_guard_tests"
    elif pooled < 49.5:
        verdict = "NEGATIVE — material worse than a flat net in the tails; investigate"
    elif pooled > 50.5:
        verdict = "clear win"
    else:
        verdict = "small/null; the client-default question decides (pre-registered)"
    print(f"  tailguard pooled {pooled:5.2f}%   cells {v}")
    print(f"    verdict: {verdict}")
PY
echo "round-54 complete"
