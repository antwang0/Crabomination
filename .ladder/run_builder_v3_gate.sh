#!/usr/bin/env bash
# Builder v3 gate: does quality-and-curve-aware shape ranking build
# stronger sealed decks, and how much does the client's new best-of-16
# opponent recipe add on top?
#
# WHY. The v2 repair fixed the per-card picks but left the *shape
# ranker* (static_build_score) body-blind: candidate color pairs are
# still ranked by the legacy pip/type/curve-bucket scorer, so a pair of
# individually-worse colors can outrank the pair holding the bombs. v3
# ranks shapes with score_card_quality plus a curve penalty (5 early
# spells wanted, 6 five-plus tolerated). Separately, the client's sealed
# opponent was a single noisy sample from the builder distribution; it
# is now best-of-16 under the v3 static judge (best_build_v3).
#
# DESIGN. Same harness as the builder_v2 gate: 12 paired pools, same
# pilots, same game seeds, only the build differs. Two races per seed:
#   race 1 — v3 single sample vs v2 single sample (same build seed):
#            the scoring change alone.
#   race 2 — best-of-16 v3 vs v2 single sample: the combined client
#            delta (scoring + selection).
# POOL COUNT IS THE PRECISION AXIS, LEARNED FROM THE FIRST RUN. At
# 800 games x 12 pools the two harness seeds contradicted each other in
# BOTH races (47.4 vs 51.8; 55.4 vs 48.6), each side of each
# contradiction "significant" by its own Wilson interval. Games are not
# independent units here: each pool is one deck-pair matchup and one
# draw from each builder distribution, and the pool-level sd is ~15
# points — so 12 pools carry a ±6.5-point t-interval no matter how many
# games each gets. The rerun spends the same games wide and flat:
# 50 games x 192 pools per race, ~±2.3 per seed, pools as the unit and
# a t-interval in the binary's own output. First-run artifacts kept as
# builder_v3_gate_s{43,97}.narrow.txt, the cautionary shape.
#
# 50 games x 192 pools = 9600 games per race, two harness seeds.
# builder_v3 stays
# default-OFF regardless of outcome here: flipping it changes every
# generated field including the ladder's sealed gate decks, so adoption
# into training/ladder is a separate, re-baselined decision. This gate
# decides (a) whether the client recipe is justified, (b) whether a
# training-field flip is worth proposing at all.
set -eu
cd /home/archuser/repos/Crabomination
cargo build --release -p crabomination_ml --bin selfplay_train
BIN=./target/release/selfplay_train
GAMES=${GAMES:-50}
export CRAB_GATE_POOLS=${CRAB_GATE_POOLS:-192}

for seed in 43 97; do
  $BIN --gate-builder-v3 "$GAMES" --seed "$seed" \
      > ".ladder/builder_v3_gate_s${seed}.txt" 2>&1
  echo "=== seed $seed ==="
  grep -E "builder gate:|TOTAL|POOLS|verdict" ".ladder/builder_v3_gate_s${seed}.txt"
done
echo "builder-v3 gate complete"
