#!/usr/bin/env bash
# Round 47: the net profiles never received the adopted blocking.
#
# WHAT WAS FOUND. `net_eval` branched off `attack_search_sim`, which
# predates both blocking adoptions, so every net profile — the ladder's
# `net`, the champion `mcts-net-deep`, and the client's `local_bot` —
# has been piloting with block_gang=false, block_search=0,
# chump_blocks=false. Value gang-blocks was adopted at 51.3 % over
# 28 800 games and desperation chump blocks at 51.0 / 50.8 over 24 000,
# and neither ever reached the code the net actually plays with.
#
# TWO CONSEQUENCES, and the second is the bigger one.
#
#   1. The client blocks worse than the ladder's `gang` control does.
#      That is a straight bug and the rebase fixes it.
#   2. Every `net` vs `gang` / `net` vs `atk-sim` gate from round 26
#      through round 46 differed in TWO ways, not one: evaluator AND
#      blocking. The net side was handicapped in all of them. So the
#      champion's ~52.7 / ~51.2 band, the replacement-vs-blend results,
#      the v7 +0.4, the r45 capacity null — all of them measured a net
#      profile carrying a blocking deficit. The *rankings* within a
#      round are unaffected (both arms of every net-vs-net cell shared
#      the deficit), which is why nothing looked wrong; the
#      net-vs-heuristic *levels* are understated.
#
# This round measures how much, so the write-up can say it with a
# number instead of an argument.
#
#   Part A — `net` vs `net-preblocks`. What the two blocking layers are
#     worth on the net evaluator, pilot-side. Pre-registered
#     expectation: +1 to +2. Gang-blocks alone measured +1.3 on the
#     heuristic and chumps +0.9, but they are not independent (both are
#     blocking) and the net evaluator judges the resulting boards
#     differently, so additivity is not assumed.
#   Part B — `mcts-net-deep` vs `mcts-net-preblocks`. The same question
#     inside the search, which is what the client runs. Expectation:
#     same sign, smaller — rollouts already play some of these blocks
#     out, so part of the value should already be priced in.
#
# A NEGATIVE IS POSSIBLE AND WOULD BE INTERESTING. If the net evaluates
# post-gang-block boards worse than the heuristic did, the layers could
# fail to transfer. That would not argue for keeping the deficit — it
# would argue that the blocking layers were fitted to the heuristic
# evaluator and need re-measuring on the net.
set -eu
cd /home/archuser/repos/Crabomination
LADDER=./target/release/bot_ladder
NET=nets_r41_v7_s151/best.safetensors
GAMES=${GAMES:-1000}

[ -f "$NET" ] || { echo "missing $NET" >&2; exit 1; }
[ -x "$LADDER" ] || { echo "build the release ladder first" >&2; exit 1; }

run () {
  local tag="$1" a="$2" b="$3" seed="$4"
  if [ -s ".ladder/r47_${tag}.txt" ] && grep -q "A win%" ".ladder/r47_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r47_${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  CRAB_NET="$NET" $LADDER --a "$a" --b "$b" \
      --decks sealed --games "$GAMES" --seed "$seed" \
      > ".ladder/r47_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r47_${tag}.txt" | tail -1 | tr -s ' ')"
}

echo "=== Part A: pilot-side — net vs net-preblocks ==="
for lseed in 43 97; do
  run "a_pilot_l${lseed}" net net-preblocks "$lseed"
done

echo "=== Part B: search-side — mcts-net-deep vs mcts-net-preblocks ==="
for lseed in 43 97; do
  run "b_search_l${lseed}" mcts-net-deep mcts-net-preblocks "$lseed"
done

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
for label, tag in (("A pilot", "a_pilot"), ("B search", "b_search")):
    v = []
    for l in (43, 97):
        p = pathlib.Path(f".ladder/r47_{tag}_l{l}.txt")
        if not p.exists():
            continue
        m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text())
        if m:
            v.append(float(m[-1]))
    if v:
        print(f"  {label:<10} pooled {st.mean(v):5.2f}%   cells {v}")
PY
echo "round-47 complete"
