#!/usr/bin/env bash
# Follow-up to the deck-lever gate (2026-11-08): polish passed on SOS
# (polish3 51.84/51.47, polish1 51.23/50.99) and was flat on FRA. Confirm
# polish3 SOS at fresh seeds 11/29 (same bar: pooled >= 50.5, both > 50),
# and read whether more swaps keep paying (polish6, SOS 43/97 + FRA 43).
set -u
cd "$(dirname "$0")/../.."
while ! grep -q 'after done' .ladder/crn/deck_gate.log; do sleep 60; done
E=.ladder/crn/fra_card_eval_polish; out=.ladder/crn/deck; mkdir -p $out
cell() { # variant set seed
  local v=$1 set=$2 s=$3 pilot=dflt val=sos extra="--set sos"
  [ "$set" = fra ] && { pilot=fra-fixes; val=fra; extra=""; }
  local t0=$(date +%s)
  CRAB_DECK_POLICY=nets/deck-policy.json CRAB_DECK_VALUE=nets/deck-value-$val.json $E $extra --ab --best 16 --trial policy-value+$v \
    --pools 2000 --pairs 8 --pilot $pilot --seed $s > $out/${v}_${set}_s$s.txt 2> $out/${v}_${set}_s$s.log
  echo "$v $set s$s: $(tail -1 $out/${v}_${set}_s$s.txt) [$(( $(date +%s) - t0 ))s]"
}
cell polish3 sos 11; cell polish3 sos 29
cell polish6 sos 43; cell polish6 sos 97
cell polish6 fra 43
echo "polish done"
