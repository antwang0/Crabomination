#!/usr/bin/env bash
# Round 36: is MCTS still necessary at inference once the net has been
# distilled on what MCTS concluded?
#
# Round 35 answered the precondition: policy distillation takes top-1
# agreement with the search's picks from below chance (pilot_policy
# 0.359-0.413 vs chance 0.39-0.45) to 0.726 / 0.814, replicated on both
# training seeds. Value metrics were a wash (mean AUC 0.781 distil vs
# 0.782 control, inside the 0.023 seed spread). What that CANNOT say is
# whether ranking-like-the-search converts to winning-like-the-search:
# 0.81 top-1 still misses one pick in five, and the misses are not
# random draws — they are the positions where evaluation is hardest.
#
# PART 1 — the decisive comparison. `mcts-net-deep` (64 iters, h3) vs
# `net` (score each successor, pick the best) with the SAME weights on
# both sides, since the ladder has one net slot. So each cell asks:
# does 64 iterations of search still add anything on top of this net?
#   champion   -> the era anchor. Round 26 measured 53.4/52.5 for A;
#                 reproducing that on today's binary is what makes the
#                 other four rows readable (cross-era gate numbers are
#                 apples-to-oranges — the round-28 lesson).
#   r35 control -> v6-era, MCTS-actor data, no policy gradient. Without
#                 this arm a narrowed gap could just be "v6-era nets
#                 differ from the champion", not "distillation worked".
#   r35 distil  -> the treatment.
# Read: A% at 50 means search adds nothing and MCTS can go.
#
# PART 2 — absolute strength, and the confound Part 1 cannot see. Both
# sides of Part 1 share the net, and MCTS consumes it as the UCB1 leaf
# reward. A distilled net whose win head got *worse* as a value function
# would handicap the MCTS side too, narrowing the gap for the wrong
# reason. atk-sim and gang are pure-heuristic pilots that never touch the
# net, so these cells measure each net's strength on a fixed yardstick.
# Champion's published replacement gate (51.7 gang / 54.5 atk-sim) is the
# reference; it is re-run here at the same cell size as everything else.
#
# Cell sizes differ by cost, not by care: MCTS cells run ~480 s at
# --games 100 (12 archetypes x 100), while a heuristic-vs-net cell runs
# ~1 s per 120 games, so Part 2 buys an order of magnitude more
# resolution for a fraction of Part 1's wall clock. Paired (antithetic
# seat) estimates are the ones to read, as always.
set -eu
cd /home/archuser/repos/Crabomination
LADDER=./target/release-fast/bot_ladder

[ -x "$LADDER" ] || { echo "ABORT: build with cargo build --profile release-fast --bin bot_ladder" >&2; exit 1; }

# name -> weights. Champion is a v5 checkpoint; PlayNet::load zero-pads
# it for the engine's hand-rolled forward (this is NOT the candle
# VarMap::load path that round 34 found broken for --gpu-eval).
NETS="champion:nets/champion.safetensors
ctrl43:nets_r35_control_s43/best.safetensors
ctrl97:nets_r35_control_s97/best.safetensors
dist43:nets_r35_distil_s43/best.safetensors
dist97:nets_r35_distil_s97/best.safetensors"

echo "$NETS" | while IFS=: read -r name path; do
  [ -f "$path" ] || { echo "ABORT: missing $path" >&2; exit 1; }
done

# Part 2 first: it is ~40 min for the whole grid and answers the
# viability question that makes Part 1 interpretable. If a distilled net
# cannot pilot at all, Part 1's gap is moot.
echo "=== part 2: replacement gates (1000 games/archetype) ==="
echo "$NETS" | while IFS=: read -r name path; do
  for opp in atk-sim gang; do
    for gseed in 43 97; do
      out=".ladder/r36_${name}_vs_${opp}_s${gseed}.txt"
      CRAB_NET="$path" $LADDER --a net --b "$opp" --decks sealed \
          --games 1000 --seed "$gseed" > "$out" 2>&1
      printf '%-10s vs %-8s s%-3s %s\n' "$name" "$opp" "$gseed" \
          "$(grep -A1 '^        A win%' "$out" | head -1 || true)"
    done
  done
done

echo "=== part 1: does search still add anything? (100 games/archetype) ==="
echo "$NETS" | while IFS=: read -r name path; do
  for gseed in 43 97; do
    out=".ladder/r36_mctsdeep_vs_net_${name}_s${gseed}.txt"
    CRAB_NET="$path" $LADDER --a mcts-net-deep --b net --decks sealed \
        --games 100 --seed "$gseed" > "$out" 2>&1
    printf '%-10s mcts-deep vs net s%-3s %s\n' "$name" "$gseed" \
        "$(grep '^        A win%' "$out" | head -1 || true)"
  done
done

echo "round-36 gates complete"
