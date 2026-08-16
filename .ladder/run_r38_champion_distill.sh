#!/usr/bin/env bash
# Round 38: champion-scale distillation with the separate policy head —
# the run round 36 named as the live next step, on the footing round 37
# built.
#
# The claim under test: distillation at champion scale (250 k games)
# produces a stronger net than the champion — as a pilot, and as the
# leaf evaluator inside mcts-net-deep. Round 36 measured the 20 k-game
# version at +0.40/+0.48 as a pilot and said "do not treat this as
# having refuted it at 250 k"; round 37 removed the shared-head value
# tax (head carries 0.80 ranking, win head untouched) and built the
# mixed fleet that makes the run affordable.
#
# Fleet sizing (from the round-38 probe, 64 threads / 8 mcts): value
# actors ~0.35 games/s/thread, mcts actors ~0.015 games/s/thread under
# contention. 128 + 128 aims both budgets at the same wall clock:
# ~25-40 games/s from the value fleet (250 k in ~2-3 h) and ~2 games/s
# from the mcts fleet (~15-25 k searched games, r35-scale decisions or
# better). The emergent mix is logged per checkpoint (games_mcts).
#
# Recipe deviations from the r20 champion regime, pre-registered:
#   --stop-after-stale 0  — generation is the point (the decision
#     budget); with the fed learner ticking checkpoints ~4x faster per
#     game, stale-12 would truncate generation long before 250 k.
#     best.safetensors still protects the artifact.
#   --best-metric auc     — the adoption gate is the *pilot* (win
#     head), so the artifact is selected the way every champion was.
#     val_policy rose monotonically in r37 (no peak to miss).
#
# Pilot: nets_r28f_full_s43 (v6-native champion-class, r30-verified in
# the champion band; the committed champion is v5 and cannot ride the
# candle --gpu-eval load path, ML_NOTES r34).
#
# Gates, pre-registered:
#   B (adoption): net vs atk-sim / gang, 1000 games/archetype x sealed
#     x 2 ladder seeds, both seed nets. Reference: champion pooled
#     52.7 atk-sim / 51.2 gang (r36 Part 2). Adoption only if a
#     candidate's pooled numbers clear the champion on BOTH opponents
#     (r30 rule: no clear -> champion stays).
#   C (search question): mcts-net-deep vs net, same weights both
#     sides, 100 games/archetype x 2 ladder seeds, both nets — does
#     64-iteration search still add on top of a 250 k distilled net
#     (r36 Part 1 at scale)?
#   D (exploratory, s43 net only): mcts-net-gumbel vs mcts-net-deep,
#     2 ladder seeds — does the r37 null move with a champion-scale
#     head? Exploratory: two cells decide nothing alone.
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train
LADDER=./target/release/bot_ladder
PILOT=nets_r28f_full_s43/best.safetensors

[ -f "$PILOT" ] || { echo "missing $PILOT" >&2; exit 1; }

for seed in 43 97; do
  out="nets_r38_head_s${seed}"
  rm -rf "$out"; mkdir -p "$out"
  $TRAIN --attn --lambda 0.7 --seed "$seed" \
      --use-best "$PILOT" \
      --mcts-actors 64 --mcts-fleet 128 --actors 256 --gpu-eval \
      --record-decisions --policy-every 4 --policy-temp 0.1 --policy-head \
      --games 250000 --window 500000 --lr 1e-4 --lr-cosine 60000 \
      --relabel-mode new --out "$out" \
      > "$out/log.txt" 2>&1
  grep -q "learner device: cuda" "$out/log.txt" || {
      echo "ABORT: $out is not on cuda" >&2; head -3 "$out/log.txt" >&2; exit 1; }
  grep -q "mixed fleet: 128 mcts actors" "$out/log.txt" || {
      echo "ABORT: $out fleet split missing" >&2; exit 1; }
  grep -q "into the policy head" "$out/log.txt" || {
      echo "ABORT: $out trained the win head, not the policy head" >&2; exit 1; }
  echo "$out done: $(tail -1 $out/stats.jsonl | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print("games",d["games"],"mcts",d["games_mcts"],"val_policy",d["val_policy"],"auc",d["val_auc"])')"
done

for seed in 43 97; do
  net="nets_r38_head_s${seed}/best.safetensors"
  for opp in atk-sim gang; do
    for lseed in 43 97; do
      tag="s${seed}_vs_${opp}_l${lseed}"
      CRAB_NET="$net" $LADDER --a net --b "$opp" \
          --decks sealed --games 1000 --seed "$lseed" \
          > ".ladder/r38_pilot_${tag}.txt" 2>&1
      echo "B ${tag}: $(grep -E 'A win%' ".ladder/r38_pilot_${tag}.txt" | tail -1 | tr -s ' ')"
    done
  done
  for lseed in 43 97; do
    tag="s${seed}_deep_vs_net_l${lseed}"
    CRAB_NET="$net" $LADDER --a mcts-net-deep --b net \
        --decks sealed --games 100 --seed "$lseed" \
        > ".ladder/r38_search_${tag}.txt" 2>&1
    echo "C ${tag}: $(grep -E 'A win%' ".ladder/r38_search_${tag}.txt" | tail -1 | tr -s ' ')"
  done
done
for lseed in 43 97; do
  tag="s43_gumbel_l${lseed}"
  CRAB_NET=nets_r38_head_s43/best.safetensors $LADDER --a mcts-net-gumbel --b mcts-net-deep \
      --decks sealed --games 100 --seed "$lseed" \
      > ".ladder/r38_gumbel_${tag}.txt" 2>&1
  echo "D ${tag}: $(grep -E 'A win%' ".ladder/r38_gumbel_${tag}.txt" | tail -1 | tr -s ' ')"
done
echo "round-38 complete"
