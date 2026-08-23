#!/usr/bin/env bash
# Round 49: decide the mulligan by playing it out.
#
# WHY THIS ROUND, AND WHY IT IS NOT A RETUNE OF A KNOWN NULL.
# `bot_probe --deck sos` over 300 games: Mulligan is 358 of 1424
# decisions — 25.1 %, more than double the next kind (ChooseTarget 13 %)
# — and it is the decision that sets up the whole game. It is also the
# only high-volume decision still answered by a predicate that never
# looks past the opening hand. Modes, optional triggers and
# sacrifice-for-value are all judged by playing the state forward and
# scoring the settled result; mulligan counts lands and asks whether one
# spell is castable.
#
# A quality-aware PREDICATE was already tried: `mull_quality`, 50.2 %
# [49.6, 50.8] over 28 800 paired games, a well-powered null whose tests
# still document two hands it reads backwards. `mull_sim` is a different
# mechanism, not a retune of that one: both branches are played forward
# four turns over six determinised samples each and scored with the
# bot's own evaluator, so going down a card is *measured* rather than
# priced by a hand-tuned threshold. If this is also a null, the pair of
# results says something worth writing down — that the opening-hand
# decision is not where this bot loses games, at either level of
# sophistication.
#
# PRE-REGISTERED EXPECTATION: +0.5 to +1.5. Mulligan is high-volume and
# high-leverage, but the shipped predicate is not obviously bad (2-5
# lands plus a castable early play is most of what matters), so the
# headroom is the marginal hands rather than the obvious ones. A null
# replicates the `mull_quality` result on a different mechanism and
# closes the mulligan question for now.
#
# COST IS PART OF THE VERDICT. This runs 12 forward simulations per
# mulligan decision (2 branches x 6 samples, 4 turns each). At ~1.2
# mulligan decisions per game that is ~14 short sims per game on top of
# everything else. Part C measures it; a flag that buys +1 for 2x the
# wall clock is a different proposition from one that buys it for free,
# and the client's latency budget is already the binding constraint on
# search iterations (round 48).
set -eu
cd /home/archuser/repos/Crabomination
LADDER=./target/release/bot_ladder
GAMES=${GAMES:-1000}

[ -x "$LADDER" ] || { echo "build the release ladder first" >&2; exit 1; }

echo "=== Part C: cost per game (1 thread, wall/4, r42 convention) ==="
for prof in gang mullsim; do
  start=$(date +%s.%N)
  $LADDER --a "$prof" --b gang --decks sealed --games 4 --seed 43 --threads 1 \
      > ".ladder/r49_cost_${prof}.txt" 2>&1 || true
  end=$(date +%s.%N)
  echo "  ${prof}: $(echo "$end $start" | awk '{printf "%.1f", ($1-$2)/4}')s per game"
done

echo "=== Part A: mullsim vs gang ==="
for lseed in 43 97; do
  tag="a_mullsim_l${lseed}"
  if [ -s ".ladder/r49_${tag}.txt" ] && grep -q "A win%" ".ladder/r49_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r49_${tag}.txt" | tail -1 | tr -s ' ')"
    continue
  fi
  $LADDER --a mullsim --b gang --decks sealed --games "$GAMES" --seed "$lseed" \
      > ".ladder/r49_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r49_${tag}.txt" | tail -1 | tr -s ' ')"
done

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
v = []
for l in (43, 97):
    p = pathlib.Path(f".ladder/r49_a_mullsim_l{l}.txt")
    if not p.exists():
        continue
    m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text())
    if m:
        v.append(float(m[-1]))
if v:
    flag = "  <-- ZERO INCIDENCE" if all(abs(x - 50.0) < 0.05 for x in v) else ""
    print(f"  mullsim vs gang   pooled {st.mean(v):5.2f}%   cells {v}{flag}")
    print("  reference: the mull_quality PREDICATE was 50.2 % [49.6, 50.8] over 28 800 games")
PY
echo "round-49 complete"
