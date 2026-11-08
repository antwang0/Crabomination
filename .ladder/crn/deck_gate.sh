#!/usr/bin/env bash
# Deck-builder levers (pre-registered 2026-11-08, ML_NOTES "Research sweep"):
# each lever of DeckPolicy::build_valued_with against the ADOPTED learned
# builder (build_valued, 16 samples, nets/deck-policy.json +
# nets/deck-value-{fra,sos}.json), 2,000 pools x 8 pairs, pool-level,
# seeds 43 / 97, FRA (fra-fixes pilot) and SOS (dflt pilot).
#   rated   — the rated best-of-16 build as an extra candidate
#   lands   — more-land variants (to 18) of the value's top 3
#   polish1 / polish3 — at most 1 / 3 value-guided swaps after the pick
# Moved (per pool) = pooled >= 50.5 with both seeds above 50. Adoption is
# the user's call.
set -u
cd "$(dirname "$0")/../.."
E=.ladder/crn/fra_card_eval_sweep; out=.ladder/crn/deck; mkdir -p $out
for v in rated lands polish1 polish3; do
  for s in 43 97; do
    CRAB_DECK_POLICY=nets/deck-policy.json CRAB_DECK_VALUE=nets/deck-value-fra.json $E --ab --best 16 --trial policy-value+$v \
      --pools 2000 --pairs 8 --pilot fra-fixes --seed $s > $out/${v}_fra_s$s.txt 2> $out/${v}_fra_s$s.log
    echo "$v fra s$s: $(tail -1 $out/${v}_fra_s$s.txt)"
    CRAB_DECK_POLICY=nets/deck-policy.json CRAB_DECK_VALUE=nets/deck-value-sos.json $E --set sos --ab --best 16 --trial policy-value+$v \
      --pools 2000 --pairs 8 --pilot dflt --seed $s > $out/${v}_sos_s$s.txt 2> $out/${v}_sos_s$s.log
    echo "$v sos s$s: $(tail -1 $out/${v}_sos_s$s.txt)"
  done
done
