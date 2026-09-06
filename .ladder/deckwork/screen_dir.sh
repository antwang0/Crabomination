#!/usr/bin/env bash
# screen_dir.sh DIR INCUMBENT_NAME [THREADS] — screen every DIR/*.txt at 500 pairs x 24
# opponents, seeds 43 and 97 (skipping cells already in results.txt), then rank vs INCUMBENT.
set -u
cd /home/archuser/repos/Crabomination
W=.ladder/deckwork
G=./target/release-fast/deck_gauntlet
R=$W/results.txt
DIR=$1; INC=$2; T=${3:-8}
echo "=== screen_dir $DIR vs $INC start $(date)"
for f in "$DIR"/*.txt; do
  name=$(basename "$f" .txt)
  for s in 43 97; do
    if grep -q "^RESULT deck=$name pilot=dflt opp_pilot=dflt field=9,12,16,20x6/0xdecc0000 pairs=500 seed=$s " $R; then continue; fi
    echo "--- $name seed $s ($(date +%H:%M:%S))"
    $G --deck "$f" --pairs 500 --seed $s --threads $T --out $R 2>/dev/null | grep -E "^RESULT"
  done
done
python3 scripts/gauntlet_summary.py $R --vs "$INC" --pilot dflt --manifest "$DIR/manifest.tsv" | tee "$DIR/rank.txt"
echo "=== screen_dir $DIR done $(date)"
