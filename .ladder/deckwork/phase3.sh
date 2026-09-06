#!/usr/bin/env bash
# Phase 3: hand-made variants (PLAN.md), 500 pairs x 24 opponents at seeds 43 and 97,
# 8 threads (a search cell runs alongside). Same binary as the baseline cells.
set -u
cd /home/archuser/repos/Crabomination
W=.ladder/deckwork
G=./target/release-fast/deck_gauntlet
R=$W/results.txt
echo "=== phase3 start $(date)"
for v in v1_trim40 v2a_17lands v2b_5colour v3_bombs v4_removal v5_curve v6_46at17 v7_notrudge; do
  for s in 43 97; do
    echo "--- screen $v seed $s ($(date +%H:%M:%S))"
    $G --deck $W/variants/$v.txt --pairs 500 --seed $s --threads 8 --out $R 2>/dev/null | grep -E "^RESULT"
  done
done
python3 scripts/gauntlet_summary.py $R --vs real_build_converge3 --pilot dflt | tee $W/phase3_rank.txt
echo "=== phase3 done $(date)"
