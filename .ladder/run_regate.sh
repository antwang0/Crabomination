#!/usr/bin/env bash
# Re-gate the play net as a pilot on best.safetensors — and run the
# deck_duel the pool recommendation left open.
#
# Why the gate is being re-run: the "play net as evaluator" dead end was
# closed on ten gates that all piloted `latest.safetensors`, and round 11
# showed `latest` is a memorised checkpoint (holdout AUC 0.707 vs best's
# 0.780; saturation 12.8% vs 3.3%). Saturation was a mechanistically
# plausible cause of gate losses — a flat landscape the search cannot
# rank — and it just dropped 4x. This batch is the same measurement as
# the closed gates (same B profile, same sizing: 100 games x 12 sealed
# mirrors = 1200 games, paired), differing only in the checkpoint, so
# the result lands on the same scale as the 42-45% / ~49% numbers.
#
# B is atk-sim, not the adopted default: net_eval() is attack_search_sim
# with the evaluator swapped, so atk-sim isolates the evaluator inside an
# identical search shape — and it is what every prior gate ran against.
set -eu
LADDER=./target/release/bot_ladder
DUEL=./target/release/deck_duel
export CRAB_NET=nets_ab_full/best.safetensors

# The user's actual question first: which W/B build to sleeve. 2000
# antithetic pairs = 4000 games per seed, two seeds — the same
# replication discipline as the deck gate, since a single-seed answer on
# a 3-card difference is exactly where this repo has been burned before.
$DUEL decks/sealed_wb_sim.txt decks/sealed_wb_net.txt 2000 11 \
    > .ladder/duel_wb_s11.txt 2>&1
echo "duel seed 11 done"
$DUEL decks/sealed_wb_sim.txt decks/sealed_wb_net.txt 2000 12 \
    > .ladder/duel_wb_s12.txt 2>&1
echo "duel seed 12 done"

# The re-gate: replacement and blend, two seeds each.
for seed in 43 97; do
  $LADDER --a net --b atk-sim --decks sealed --games 100 --seed "$seed" \
      > ".ladder/regate_net_s$seed.txt" 2>&1
  echo "regate net seed $seed done"
  $LADDER --a net-blend --b atk-sim --decks sealed --games 100 --seed "$seed" \
      > ".ladder/regate_blend_s$seed.txt" 2>&1
  echo "regate blend seed $seed done"
done
echo "regate batch complete"
