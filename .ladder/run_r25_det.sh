#!/usr/bin/env bash
# Task #25: what is the search's information cheat worth? Asymmetric
# gates (mirrors can't see it — both seats cheat identically):
#
#   det1 vs gang         the peek's value to the heuristic searcher
#   net-det1 vs net      the peek's value to the champion net pilot
#   net-det1 vs gang     absolute strength of the honest champion
#   net-det1 vs atk-sim
#
# Uses the committed champion via the CRAB_NET fallback. If honest-search
# strength holds (deltas ~0), determinize gets adopted for client-facing
# defaults; if it collapses, the searches were leaning on the peek and
# every past number needs the asterisk.
set -eu
LADDER=./target/release-fast/bot_ladder

for gseed in 43 97; do
  $LADDER --a det1 --b gang --decks sealed --games 100 --seed "$gseed" \
      > ".ladder/r25_det1_gang_s$gseed.txt" 2>&1
  $LADDER --a net-det1 --b net --decks sealed --games 100 --seed "$gseed" \
      > ".ladder/r25_netdet1_net_s$gseed.txt" 2>&1
  $LADDER --a net-det1 --b gang --decks sealed --games 100 --seed "$gseed" \
      > ".ladder/r25_netdet1_gang_s$gseed.txt" 2>&1
  $LADDER --a net-det1 --b atk-sim --decks sealed --games 100 --seed "$gseed" \
      > ".ladder/r25_netdet1_atksim_s$gseed.txt" 2>&1
  echo "seed $gseed done"
done
echo "determinize gates complete"
