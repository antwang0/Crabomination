#!/usr/bin/env bash
# Round 42: is the search worth shipping at 256, and does it still climb
# past it?
#
# WHY THIS ROUND. Round 41 measured the best net change in the program
# at +0.4 paired. The search is worth +4 to +5 over the same net used as
# a 1-ply pilot (round 26). Round 29 tested every MCTS lever and found
# raw iterations to be the only one that pays. And the strongest
# measured configuration is not the one anything runs: the client's
# `local_bot` and every adopted profile use 64 iterations, while round
# 27 measured 256 at +2.0 over it.
#
# A PRECISION PROBLEM, STATED FIRST. Round 27's curve (24→64→128→256 =
# 49.4→53.0→54.35→55.0 % vs the champion) used 1200 games per cell,
# which is ±2.8 points. Its 128→256 step of +0.65 was well inside its
# own noise, so "still climbing at 256" is a much weaker claim than it
# reads as. Resolving a +0.5 step at the precision this program now uses
# (±0.57, from 12 000 games) would cost ~35 hours per cell at 512
# iterations. That experiment is not affordable and this script does not
# pretend to run it.
#
# What IS affordable is head-to-head. Comparing two rungs directly
# measures the difference instead of subtracting two noisy numbers, and
# the ladder's antithetic seat pairing (~4x precision, rho ~ -0.48)
# applies to it. So:
#
#   Part A — 256 vs 64, head to head. This is the client-adoption
#     question, and its expected effect (+2 to +4 head-to-head) is
#     comfortably resolvable at 2000 paired games.
#   Part B — 512 vs 256, head to head. This is the curve question.
#     Pre-registered expectation: UNDERPOWERED. If the true step is
#     +0.5-1.0 and the cell is ±2, a null here means "not resolved at
#     this width", NOT "the curve is flat". Reported as a point estimate
#     with that caveat attached, because the alternative is spending a
#     day and a half to maybe resolve it.
#
# Cost, measured single-threaded on this box before writing this:
# 64 iters 29.6 s/game, 256 iters 115.8 s/game, i.e. linear in
# iterations. 512 is ~232 s/game and 1024 ~464, which is why 1024 is not
# in the sweep — it would be a five-hour cell that cannot resolve its
# own effect.
#
# LATENCY GATES ADOPTION INDEPENDENTLY OF STRENGTH. A rung that wins and
# takes twelve seconds per decision does not ship into a client a human
# is sitting in front of. Part C measures per-game serial cost at each
# rung; the per-decision figure is derived in the write-up from the
# searched decisions in a game (tens), and is reported as a range rather
# than asserted, since the ladder counts games and not decisions.
set -eu
cd /home/archuser/repos/Crabomination
LADDER=./target/release/bot_ladder
NET=nets_r41_v7_s151/best.safetensors   # a round-41 v7 net: the current encoder
A_GAMES=${A_GAMES:-2000}
B_GAMES=${B_GAMES:-1000}

[ -f "$NET" ] || { echo "missing $NET" >&2; exit 1; }

echo "=== Part C: serial cost per game (1 thread) ==="
for prof in mcts-net-deep mcts-net-128 mcts-net-256 mcts-net-512; do
  start=$(date +%s.%N)
  CRAB_NET="$NET" $LADDER --a "$prof" --b net \
      --decks sealed --games 4 --seed 43 --threads 1 \
      > ".ladder/r42_cost_${prof}.txt" 2>&1 || true
  end=$(date +%s.%N)
  echo "  ${prof}: $(echo "$end $start" | awk '{printf "%.1f", ($1-$2)/4}')s per game"
done

echo "=== Part A: 256 vs 64 (the adoption question) ==="
for lseed in 43 97; do
  tag="a_256_vs_64_l${lseed}"
  CRAB_NET="$NET" $LADDER --a mcts-net-256 --b mcts-net-deep \
      --decks sealed --games "$A_GAMES" --seed "$lseed" \
      > ".ladder/r42_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r42_${tag}.txt" | tail -1 | tr -s ' ')"
done

echo "=== Part B: 512 vs 256 (the curve question; expect underpowered) ==="
for lseed in 43 97; do
  tag="b_512_vs_256_l${lseed}"
  CRAB_NET="$NET" $LADDER --a mcts-net-512 --b mcts-net-256 \
      --decks sealed --games "$B_GAMES" --seed "$lseed" \
      > ".ladder/r42_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r42_${tag}.txt" | tail -1 | tr -s ' ')"
done

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
for label, tag in (("256 vs 64", "a_256_vs_64"), ("512 vs 256", "b_512_vs_256")):
    v = []
    for l in (43, 97):
        p = pathlib.Path(f".ladder/r42_{tag}_l{l}.txt")
        if not p.exists():
            continue
        m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text())
        if m:
            v.append(float(m[-1]))
    if v:
        print(f"  {label:<12} pooled {st.mean(v):5.2f}%   cells {v}")
PY
echo "round-42 complete"
