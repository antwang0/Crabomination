#!/bin/bash
# pod_census.sh BIN SEATS SEED0 GAMES [NDECKS] — rotate every target deck
# through N-seat Commander pods: the decks 1..NDECKS (default 183) are
# shuffled by SEED0 into groups of SEATS (the last topped up at random), and
# each group plays GAMES games at seed SEED0 + group index.
#
# One line per group: "seed S decks I,J,.. rc R undecided U". rc 134 / 101 is
# a panic (optimized / debug build); rc 124 is a HANG — one action that never
# returned, which the action cap cannot see (it counts actions): attach
# `gdb -p <pid> -batch -ex "thread apply all bt"` while it spins, or bisect
# with `--first i --games 1` under `timeout 20`. An undecided game prints its
# replay line; `CRAB_CAP_DIAG=10000000` on that replay names each seat's loss.
#
#   cargo build --profile release-fast -p crabomination --bin bot_ladder
#   scripts/pod_census.sh target/release-fast/bot_ladder 4 91000 500 > census.log
#   grep '^seed' census.log | grep -v 'rc 0 undecided 0'
BIN=$1; N=$2; SEED0=$3; GAMES=$4; ND=${5:-183}
GROUP_LIST=$(python3 - "$N" "$SEED0" "$ND" <<'PY'
import sys, random
n, s, nd = int(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3])
r = random.Random(s); ix = list(range(1, nd + 1)); r.shuffle(ix)
while len(ix) % n: ix.append(r.randint(1, nd))
for i in range(0, len(ix), n): print(",".join(map(str, ix[i:i + n])))
PY
)
seed=$SEED0
for g in $GROUP_LIST; do
  out=$(timeout "${POD_CENSUS_TIMEOUT:-1800}" "$BIN" --commander --pod-decks "$g" --games "$GAMES" --seed "$seed" 2>&1); rc=$?
  und=$(echo "$out" | grep -o "undecided [0-9]*" | head -1)
  echo "seed $seed decks $g rc $rc $und"
  if [ $rc -ne 0 ] || ! echo "$out" | grep -q "undecided 0 "; then
    echo "$out" | grep -E "panicked|undecided|error|overflow" | head -8
  fi
  seed=$((seed + 1))
done
