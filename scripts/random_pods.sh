#!/bin/bash
# Commander pods over RANDOM target-deck groups, printing only the groups that
# end a game undecided, panic or fail to run.
#
#     scripts/random_pods.sh SEATS GROUPS GAMES SEED
#
# `bot_ladder --commander` alone seats the FIRST N target decks, whatever the
# seed, so a plain smoke never sees most of the pool; this samples `SEATS`
# distinct decks per group (`--pod-decks`, 1-based) and plays `GAMES` games of
# each. 2026-10-21: 1,000 four-seat groups x 10 games ran in ~10 minutes and
# found the two board caps and the Descent into Avernus draws in TODO's NEXT.
#
# BIN (default target/release/bot_ladder) is the binary; TOTAL (default: the
# binary's own seat count) bounds the ids. Copy the binary out of target/ first
# if you will build while it runs.
seats=$1; groups=$2; games=$3; seed=$4
bin=${BIN:-target/release/bot_ladder}
if [ -z "$TOTAL" ]; then
  TOTAL=$("$bin" --commander --seats 2 --games 1 --pod-decks 999999,1 2>&1 | grep -oE "in 1\.\.=[0-9]+" | grep -oE "[0-9]+$")
fi
[ -z "$TOTAL" ] && { echo "set TOTAL to the binary's seat count"; exit 2; }
RANDOM=$seed
for g in $(seq 1 "$groups"); do
  ids=$(for _ in $(seq 1 "$seats"); do echo $(( (RANDOM * 32768 + RANDOM) % TOTAL + 1 )); done | sort -un | paste -sd,)
  [ "$(echo "$ids" | tr ',' '\n' | wc -l)" -lt "$seats" ] && continue
  out=$(timeout 600 "$bin" --commander --seats "$seats" --games "$games" --seed $((seed + g)) --threads 3 --pod-decks "$ids" 2>&1)
  rc=$?
  if [ $rc -ne 0 ] || ! echo "$out" | grep -q "undecided 0 " || echo "$out" | grep -qi "panick"; then
    echo "HIT seats=$seats ids=$ids seed=$((seed + g)) rc=$rc $(echo "$out" | grep -E "undecided_by|panick" | head -3 | tr '\n' ' ')"
  fi
done
echo "done $seats x $groups"
