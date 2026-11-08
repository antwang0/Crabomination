#!/usr/bin/env bash
# AIVAT read 1 (2026-11-08): the luck correction's variance ratio on a
# scored-pilot cell, net67 vs dflt, SOS sealed, paired 400 x 12, seeds 43 / 97.
# The corrected and raw means must agree (same estimand).
set -u
cd "$(dirname "$0")/../.."
for s in 43 97; do
  CRAB_AIVAT=1 .ladder/crn/bot_ladder_aivat --a net67 --b dflt --games 400 --decks sealed --threads 4 --seed $s > .ladder/crn/aivat_heur_s$s.txt 2>&1
  grep -E 'A win%|aivat' .ladder/crn/aivat_heur_s$s.txt
done
