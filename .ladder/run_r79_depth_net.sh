#!/usr/bin/env bash
# Round 79 (2026-09-18): the search's depth curve on the NET leaf — the
# lobby's and the client's search (`mcts-net67-256`, adopted r74) against
# the same search at 512 iterations. Round 64 read the curve on the
# MATERIAL leaf: 64 → 128 = +2.4, 128 → 256 = +0.5 (flattening at horizon
# 3). The net leaf replaced the leaf entirely (+4.25 at 256) and its curve
# has never been read. Rounds 76b–78 settled that scored-pick fixes reach
# the search only through the root menu, whose six-arm cap the r77 census
# priced as not a hole; the budget is the one untested lever for what the
# human faces.
#
# CELLS: `mcts-net67-512` (A) vs `mcts-net67-256` (B), sealed, paired, 500
# games × 12 decks, seeds 43 / 97 (±0.9 a cell; the 512 side doubles the
# search, so ~100 min a cell at 22 threads). Runs from a copied binary.
#
# PRE-REGISTERED:
#   pooled >= +1.0, both cells > 50  -> the search is budget-limited on the
#                                       net leaf; expose the iteration count
#                                       as a client setting and read 1024
#                                       next (latency ~0.5 s → 1 s → 2 s a
#                                       searched decision, single-threaded).
#   pooled in (+0.3, +1.0)           -> flattening as on the material leaf;
#                                       record, no adoption (the lobby's
#                                       latency budget is not free).
#   pooled <= +0.3                   -> judgement-limited: the leaf, not the
#                                       budget, is the next lever; close the
#                                       depth line at horizon 3.
set -u
cd /home/archuser/repos/Crabomination
LADDER=${LADDER:-.ladder/r79/bot_ladder_r79}
R=.ladder/r79
[ -x "$LADDER" ] || { echo "no $LADDER"; exit 2; }
for g in ${SEEDS:-43 97}; do
  f="$R/net512_vs_256_g${g}.txt"
  if [ -s "$f" ] && grep -q "A win%" "$f"; then echo "have $f"; continue; fi
  echo "== $f $(date -Is)"
  $LADDER --a mcts-net67-512 --b mcts-net67-256 --decks sealed --games 500 --seed "$g" --threads 22 > "$f" 2>&1
  echo "exit: $?" >> "$f"
done
echo "== done $(date -Is)" > "$R/depth.done"
grep -h "A win%" "$R"/net512_vs_256_g*.txt
