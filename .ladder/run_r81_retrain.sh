#!/usr/bin/env bash
# Round 81: the value net retrained on today's engine (2026-10-20).
#
# Round 69's recipe, verbatim (`run_r69_retrain.sh train`), on the actors as
# they stand at the round-81 tip — rounds 70-80's adopted defaults and the
# engine fixes since 2026-09-07. Training only; the gate is round 69's `gate`
# half pointed at these nets, run when the training is read.
#
# RECIPE. --attn --lambda 0.7 --games 250000 --steps 200000 --window 500000
#   --lr 1e-4 --lr-cosine 60000 --relabel-mode new --stop-after-stale 12
#   --actors 3; four training seeds as two concurrent drivers:
#   SEEDS="43 151" run_r81_retrain.sh train   and   SEEDS="97 199" ... train
# (`queue` starts both once the GPU is free.)
#
# ⚠ The learner-device line is a GATE (ML_NOTES, round 28's lesson): a run
# whose log doesn't say "learner device: cuda" aborts — the binary must be
#   cargo build --release -p crabomination_ml --features cuda --bin selfplay_train
set -eu
cd /home/archuser/repos/Crabomination
MODE=${1:-train}
TRAIN=${TRAIN:-./target/release/selfplay_train}
SEEDS=${SEEDS:-"43 97 151 199"}
ACTORS=${ACTORS:-3}
R=.ladder/r81
mkdir -p $R
[ -x "$TRAIN" ] || { echo "build it: cargo build --release -p crabomination_ml --features cuda --bin selfplay_train" >&2; exit 1; }

if [ "$MODE" = queue ]; then
  # Wait until no other training job holds the GPU: under 8 GiB in use on
  # two checks a minute apart. Memory, not utilization — the desktop and a
  # running game (dnd-rs, ~2 GiB) render continuously; the job this waits on
  # holds ~15 GiB. Gives up after six hours.
  free=0
  for _ in $(seq 1 360); do
    mem=$(nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits | head -1 | tr -d ' ')
    if [ "$mem" -lt 8192 ]; then free=$((free + 1)); else free=0; fi
    [ "$free" -ge 2 ] && break
    if [ "$free" -eq 1 ]; then
      echo "$(date '+%F %T') GPU free (${mem} MiB in use); confirming in a minute"
    else
      echo "$(date '+%F %T') GPU busy (${mem} MiB in use); waiting"
    fi
    sleep 60
  done
  [ "$free" -ge 2 ] || { echo "$(date '+%F %T') ABORT: the GPU never freed up" >&2; exit 1; }
  echo "=== queue: GPU free, starting both drivers $(date '+%F %T')"
  SEEDS="43 151" "$0" train > $R/driver_a.txt 2>&1 &
  SEEDS="97 199" "$0" train > $R/driver_b.txt 2>&1 &
  wait
  echo "=== queue done $(date '+%F %T')"
  cat $R/driver_a.txt $R/driver_b.txt
  exit 0
fi

if [ "$MODE" = train ]; then
  for tseed in $SEEDS; do
    out="nets_r81_s${tseed}"
    if grep -q "^done: 250000 games" "$out/log.txt" 2>/dev/null; then echo "$out already complete"; continue; fi
    rm -rf "$out"; mkdir -p "$out"
    echo "=== train s$tseed start $(date)"
    $TRAIN --attn --lambda 0.7 --seed "$tseed" --games 250000 --steps 200000 \
        --actors "$ACTORS" --window 500000 --lr 1e-4 --lr-cosine 60000 \
        --relabel-mode new --stop-after-stale 12 --out "$out" > "$out/log.txt" 2>&1
    grep -q "learner device: cuda" "$out/log.txt" || { echo "ABORT: $out is not on cuda" >&2; head -3 "$out/log.txt" >&2; exit 1; }
    echo "$out done: $(tail -1 "$out/stats.jsonl" | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print("games",d["games"],"auc",d["val_auc"],"elapsed_s",d["elapsed_s"])')"
  done
  echo "=== train done $(date)"
  exit 0
fi

echo "usage: $0 queue|train" >&2
exit 2
