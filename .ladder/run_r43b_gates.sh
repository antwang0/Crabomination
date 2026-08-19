#!/usr/bin/env bash
# Round-43b remainder: the three flag gates the stash reverted away
# before they ran (convlands/chumpblocks already measured — see
# .ladder/r43_convlands_* and r43_chumpblocks_*).
set -eu
cd /home/archuser/repos/Crabomination
cargo build --release -p crabomination --bin bot_ladder
LADDER=./target/release/bot_ladder
NET=nets_r41_v7_s151/best.safetensors

run_gate() {
  local a="$1" b="$2" games="$3"
  echo "=== ${a} vs ${b} ==="
  for lseed in 43 97; do
    CRAB_NET="$NET" $LADDER --a "$a" --b "$b" --decks sealed \
        --games "$games" --seed "$lseed" \
        > ".ladder/r43b_${a}_l${lseed}.txt" 2>&1
    echo "  l${lseed}: $(grep -E 'A win%' ".ladder/r43b_${a}_l${lseed}.txt" | tail -1 | tr -s ' ')"
  done
}

run_gate abilarms gang 6000
run_gate walkerchip atk-sim 6000
run_gate mcts-net-prep mcts-net-deep 1500
echo "round-43b remainder complete"
