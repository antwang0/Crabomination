#!/bin/bash
# Actor scaling: does `selfplay_train` throughput track the actor count, or is
# there contention in the shared window / recorder / allocator?
#
# **The answer is in PERF at `(-52)`, and it is "linear to the core count, flat
# past it".** This script exists because three sessions have now re-derived it
# from a seed-list line — see `(-118)` — so run it and read the entry rather
# than re-deriving it a fourth time.
#
# It needs no new instrument: `selfplay_train` already prints the actor window
# on its own (`actors: N games/s over Ts`), which is the number to read.
# `games / elapsed` divides actor work by a denominator the *learner* owns, so
# a run bounded by `--steps` reads whatever the learner was doing.
#
# Release build, because a debug engine measures the compiler:
#   cargo build --release -p crabomination_ml --bin selfplay_train
#   scripts/actor_scaling.sh                 # 1 2 4 8 actors, 3 repeats
#   ACTORS="1 2 3 4" REPEATS=5 GAMES=6000 scripts/actor_scaling.sh
#
# Read the *per-actor* column, not the speedup: the one-actor row shares its
# core with the learner thread, so it is a depressed baseline and every ratio
# taken against it reads superlinear. Past the core count the total goes flat —
# that is saturation, not contention, and the two look the same in a speedup
# column and different in this one.
set -u
cd "$(dirname "$0")/.."

BIN=${BIN:-target/release/selfplay_train}
ACTORS=${ACTORS:-"1 2 4 8"}
REPEATS=${REPEATS:-3}
GAMES=${GAMES:-3000}
SEED=${SEED:-7}
OUT=${OUT:-/tmp/actor_scaling}

[ -x "$BIN" ] || { echo "no $BIN — cargo build --release -p crabomination_ml --bin selfplay_train"; exit 1; }
mkdir -p "$OUT"

printf '%-8s %-10s %-28s %s\n' actors median per-actor "runs"
for a in $ACTORS; do
  runs=""
  for _ in $(seq 1 "$REPEATS"); do
    g=$(RUST_MIN_STACK=33554432 "$BIN" --actors "$a" --games "$GAMES" --steps 1 \
          --seed "$SEED" --out "$OUT/a$a" 2>&1 | grep -oP '^actors: \K[0-9.]+')
    runs="$runs $g"
  done
  med=$(printf '%s\n' $runs | sort -n | awk '{v[NR]=$1} END{print v[int((NR+1)/2)]}')
  printf '%-8s %-10s %-28s %s\n' "$a" "$med" "$(awk -v m="$med" -v a="$a" 'BEGIN{printf "%.1f", m/a}')" "$runs"
done

# Peak RSS is a separate question and the answer is that it is **not** about
# actors: it is the replay window. `--window 250000` peaks at ~3.1 GiB and
# `--window 25000` at ~0.6 GiB, both flat from 1 to 8 actors (~10.4 KiB a row
# over a ~370 MiB floor). Size a training box off `--window`.
#
# **`GAMES` has to be large enough to FILL the window or the RSS reading is
# meaningless** — `(-52)`'s 805 MiB figure was taken at 600 games, which push
# ~58 k rows into a 250 k window, and it under-reads the filled cost by ~4x.
# The default 3000 games push ~288 k rows, which fills it.
echo
echo "peak RSS is set by --window, not --actors — see the comment at the foot of this script"
