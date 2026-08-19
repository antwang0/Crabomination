#!/usr/bin/env bash
# Round 43: two piloting candidates, one config-level and one heuristic.
#
# PART A — mcts-net-256es: r42 measured 256 iterations at +2.4 over the
# adopted 64 and left client adoption blocked on serial latency (~2-3 s
# per searched decision at 256 vs ~0.5-0.8 at 64). Early stop ALONE is
# the untested half of round 29's adaptive arm: that arm bundled a 4x
# close-call extension which spent ~2x and returned exactly its spend,
# so the stop's saving was never measured on its own. The stop only
# quits when the leader's confidence bound clears every rival's, so
# strength should hold while forced moves stop paying the full budget.
#   A1 cost: serial s/game at deep / 256 / 256es — the saving IS the
#      result; if 256es doesn't come in well under 121.9 s/game there
#      is nothing to adopt regardless of strength.
#   A2 parity: 256es vs 256 head-to-head. Expect ~50; a deficit means
#      the confidence bounds quit too early.
#   A3 margin: 256es vs deep head-to-head. Expect ≈ r42's +2.4.
#
# PART B — buff2for1: the stack 2-for-1 (kill the creature under the
# opponent's own aura/pump so the buff fizzles), the "removal responses,
# not just counters" line from TODO's instant-speed item. Heuristic
# pilots on mirror sealed decks, so piloting alone is measured. A null
# here may be dosage (how often do SOS bots buff into open mana?) —
# report the rate alongside the result if flat.
#
# Head-to-head, antithetic seat pairs, ladder seeds 43/97 per cell.
set -eu
cd /home/archuser/repos/Crabomination
cargo build --release -p crabomination --bin bot_ladder
LADDER=./target/release/bot_ladder
NET=nets_r41_v7_s151/best.safetensors
[ -f "$NET" ] || { echo "missing $NET" >&2; exit 1; }

echo "=== Part B: buff2for1 vs gang (heuristic, fast) ==="
for lseed in 43 97; do
  $LADDER --a buff2for1 --b gang --decks sealed --games 6000 --seed "$lseed" \
      > ".ladder/r43_buff2for1_l${lseed}.txt" 2>&1
  echo "  l${lseed}: $(grep -E 'A win%' ".ladder/r43_buff2for1_l${lseed}.txt" | tail -1 | tr -s ' ')"
done

echo "=== Part A1: serial cost per game (1 thread, 8 games) ==="
for prof in mcts-net-deep mcts-net-256 mcts-net-256es; do
  start=$(date +%s.%N)
  CRAB_NET="$NET" $LADDER --a "$prof" --b net \
      --decks sealed --games 8 --seed 43 --threads 1 \
      > ".ladder/r43_cost_${prof}.txt" 2>&1 || true
  end=$(date +%s.%N)
  echo "  ${prof}: $(echo "$end $start" | awk '{printf "%.1f", ($1-$2)/8}')s per game"
done

echo "=== Part A2: 256es vs 256 (parity check) ==="
for lseed in 43 97; do
  CRAB_NET="$NET" $LADDER --a mcts-net-256es --b mcts-net-256 \
      --decks sealed --games 2000 --seed "$lseed" \
      > ".ladder/r43_a2_es_vs_256_l${lseed}.txt" 2>&1
  echo "  l${lseed}: $(grep -E 'A win%' ".ladder/r43_a2_es_vs_256_l${lseed}.txt" | tail -1 | tr -s ' ')"
done

echo "=== Part A3: 256es vs deep-64 (the margin) ==="
for lseed in 43 97; do
  CRAB_NET="$NET" $LADDER --a mcts-net-256es --b mcts-net-deep \
      --decks sealed --games 2000 --seed "$lseed" \
      > ".ladder/r43_a3_es_vs_deep_l${lseed}.txt" 2>&1
  echo "  l${lseed}: $(grep -E 'A win%' ".ladder/r43_a3_es_vs_deep_l${lseed}.txt" | tail -1 | tr -s ' ')"
done
echo "round-43 complete"
