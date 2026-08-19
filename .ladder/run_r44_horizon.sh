#!/usr/bin/env bash
# Round 44: the horizon in the other direction — can the net stand in
# for most of the rollout?
#
# WHY THIS ROUND. The forty-first perf pass vectorized the net's
# forward pass; the measured search split (CRAB_MCTS_TIMING) now reads
# rollout sim 88 %, leaf eval 12 %. The rollout plays THREE turns of
# engine actions (~63 per rollout at ~16 µs) before the net is asked a
# question it was trained to answer from a single position. MuZero's
# value-equivalence lesson, inverted for an engine that hands us real
# afterstates for free: if a 1-turn rollout plus the net matches a
# 3-turn rollout plus the net, every iteration gets ~2-3x cheaper, and
# rounds 27/29/42 say iterations are the only search lever that pays
# (256 vs 64 = +2.1 ±0.4 head-to-head). This round buys either a much
# cheaper iteration or the cleanest measurement yet of what the
# rollout actually contributes. h4 (round 42) already showed LONGER is
# not better; shorter has never been measured with the net leaf.
#
# THE ARMS.
#   h0 — evaluate the root candidate's immediate successor. The spell
#     is often still on the stack, and determinization cannot reach an
#     observable-info encoder, so this approximates the 1-ply `net`
#     pilot with extra steps. Sanity anchor: r36/r38 put the search's
#     edge over 1-ply at +4 to +5, so h0 should lose to h3 by roughly
#     that. If it doesn't, the rollout was never doing anything and
#     everything we think about this search is wrong. Small cell.
#   h1 — one turn: resolves the stack, one exchange, then the net.
#     This is the hypothesis cell.
#
# PRE-REGISTERED EXPECTATIONS.
#   Part A (h1 vs h3, both 64 iters): h1 loses a little or draws. A
#     loss of 1-3 points is compatible with the hypothesis — the
#     question is whether B's extra iterations buy it back cheaper
#     than the rollout bought it.
#   Part B (h1@192 vs h3@64 — rollout-cost-matched by Part C): the
#     adoption question. On the r27 curve (+~1.4/+~0.7 per doubling),
#     64→192 is worth ~+2 if the h1 iteration is worth what an h3
#     iteration is. B > 50 % ⇒ the horizon trade is profitable and the
#     client can search deeper at the same latency. B ≈ 50 % with A
#     negative ⇒ the rollout's turns carry signal iterations can't
#     replace; the value-equivalence direction closes here.
#   Part D (h0 vs h3, one seed, small): expect h3 by ~+4-5.
#
# MEASUREMENT. House rules: antithetic seat pairs, two ladder seeds
# per gated claim, paired intervals read from the harness, serial
# costs measured before the gates so "cost-matched" is a number and
# not a guess. 192 was picked from the 41st-pass split (h1 rollout
# ≈ 1/3 the engine actions of h3 + the same leaf); Part C reports the
# achieved ratio and the write-up owns any mismatch.
set -eu
cd /home/archuser/repos/Crabomination
LADDER=./target/release/bot_ladder
NET=nets_r41_v7_s151/best.safetensors   # same net as r42/r43 — measurement control
A_GAMES=${A_GAMES:-2000}
D_GAMES=${D_GAMES:-500}

[ -f "$NET" ] || { echo "missing $NET" >&2; exit 1; }

echo "=== Part C: serial cost per game (1 thread, wall/4, r42 convention) ==="
for prof in mcts-net-h0 mcts-net-h1 mcts-net-h1-192 mcts-net-deep; do
  start=$(date +%s.%N)
  CRAB_NET="$NET" $LADDER --a "$prof" --b net \
      --decks sealed --games 4 --seed 43 --threads 1 \
      > ".ladder/r44_cost_${prof}.txt" 2>&1 || true
  end=$(date +%s.%N)
  echo "  ${prof}: $(echo "$end $start" | awk '{printf "%.1f", ($1-$2)/4}')s per game"
done

echo "=== Part A: h1 vs h3 at 64 iterations (what do the extra turns buy) ==="
for lseed in 43 97; do
  tag="a_h1_vs_h3_l${lseed}"
  CRAB_NET="$NET" $LADDER --a mcts-net-h1 --b mcts-net-deep \
      --decks sealed --games "$A_GAMES" --seed "$lseed" \
      > ".ladder/r44_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r44_${tag}.txt" | tail -1 | tr -s ' ')"
done

echo "=== Part B: h1@192 vs h3@64, cost-matched (the adoption question) ==="
for lseed in 43 97; do
  tag="b_h1x192_vs_h3_l${lseed}"
  CRAB_NET="$NET" $LADDER --a mcts-net-h1-192 --b mcts-net-deep \
      --decks sealed --games "$A_GAMES" --seed "$lseed" \
      > ".ladder/r44_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r44_${tag}.txt" | tail -1 | tr -s ' ')"
done

echo "=== Part D: h0 anchor (expect h3 by ~+4-5) ==="
tag="d_h0_vs_h3_l43"
CRAB_NET="$NET" $LADDER --a mcts-net-h0 --b mcts-net-deep \
    --decks sealed --games "$D_GAMES" --seed 43 \
    > ".ladder/r44_${tag}.txt" 2>&1
echo "  ${tag}: $(grep -E 'A win%' ".ladder/r44_${tag}.txt" | tail -1 | tr -s ' ')"

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
for label, tag, seeds in (
    ("A: h1 vs h3", "a_h1_vs_h3", (43, 97)),
    ("B: h1@192 vs h3", "b_h1x192_vs_h3", (43, 97)),
    ("D: h0 vs h3", "d_h0_vs_h3", (43,)),
):
    v = []
    for l in seeds:
        p = pathlib.Path(f".ladder/r44_{tag}_l{l}.txt")
        if not p.exists():
            continue
        m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text())
        if m:
            v.append(float(m[-1]))
    if v:
        print(f"  {label:<16} pooled {st.mean(v):5.2f}%   cells {v}")
PY
echo "round-44 complete"
