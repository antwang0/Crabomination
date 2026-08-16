#!/usr/bin/env bash
# Round 40: the encoder's last untried category — and the thing the
# census found while checking it.
#
# The plan was three feature blocks from TODO.md's "Encoder gaps".
# `selfplay_train --feature-census` (new, no GPU) measured how often
# each feature is actually non-zero in recorded self-play positions
# before any of them earned a run, and the answer reshaped the round.
#
# Census, 300 games / 29 696 positions / 1.04 M objects, v6 encoder and
# the round-39 recorder:
#
#   hist  globals 43..=54  6.9-51.7 % of positions      <- a real block
#   exp   object feats 45..=47  0.13-0.16 % of objects  <- nearly empty
#   ctr   object feats 48..=52  0-1.6 % of objects      <- thin
#
#   g11 (coarse combat phase)        0.00 %
#   g19 (attackers)                  0.00 %
#   g36..=40 (round-28 combat block) 0.00 %
#   f10 / f28 / f29 (attacking, blocking, blocked)  0.00 %
#   f37..=39 (round-28 counterpart sums)            0.00 %
#
# Thirteen features are identically zero in every training row ever
# recorded. The recorder snapshots at each new turn, at post-combat
# main, and at end step: combat is over by the first and hasn't started
# by the second, so no training row has ever been a combat row. Round
# 28f gated the combat block against a training set in which it cannot
# vary — the null was arithmetic, not evidence about the block. And
# those columns are *live at inference*: the attack and block sims
# evaluate mid-combat positions, feeding real features into weights
# that never received a gradient.
#
# So the round runs two arms, not three, and the recorder is the one
# with a mechanism:
#
#   Arm E (encoder) — full v7: hist + exp + ctr, recorder unchanged.
#     Tests whether turn-scoped history moves anything. exp and ctr ride
#     along; the census says not to expect much of them alone.
#   Arm C (combat) — full v7 + --record-combat: snapshots at
#     declare-attackers, declare-blockers, first-strike/combat damage
#     and end-of-combat as well. Makes the thirteen dead features live
#     for the first time (post-change census: g36 19.0 %, g37 7.3 %,
#     g38 18.7 %, g39/40 5.5 %, g11 45.0 %) and gives the exp block
#     real occupancy, since marked damage exists during combat.
#
# Control: the round-38 cells. Arm E minus its three blocks is
# byte-identical to the v6 encoder (the new columns are exactly dead),
# and the recipe below is r38's with flags added, so r38 is the control
# the same way it was for round 39.
#
# Known confound, stated up front: arm C records ~82 % more rows per
# game, so at a fixed 500 k-row window it covers ~55 % as many games as
# the control. That is part of the treatment, not a bug — but a null
# for arm C is a null for "combat rows at equal window", not for
# "combat rows".
#
# Gates, pre-registered:
#   Pilot (both arms): net vs atk-sim / gang, 1000 games/archetype x 2
#     ladder seeds, sealed, 4 cells pooled. References: champion 52.7 /
#     51.2, r38 band 52.5-52.85 / 51.1-51.35.
#   Search (arm C, the one with the mechanism): mcts-net-deep vs net,
#     same weights both sides. If training on the positions the search
#     ranks is worth anything, this is where it shows: the search's edge
#     over its own 1-ply pilot was 54.85 (r38) / 53.95 (r39).
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train
LADDER=./target/release/bot_ladder
PILOT=nets_r28f_full_s43/best.safetensors

[ -f "$PILOT" ] || { echo "missing $PILOT" >&2; exit 1; }

for seed in 43 97; do
  for arm in enc combat; do
    out="nets_r40_${arm}_s${seed}"
    extra=""
    [ "$arm" = "combat" ] && extra="--record-combat"
    rm -rf "$out"; mkdir -p "$out"
    # shellcheck disable=SC2086
    $TRAIN --attn --lambda 0.7 --seed "$seed" \
        --use-best "$PILOT" \
        --mcts-actors 64 --mcts-fleet 128 --actors 256 --gpu-eval \
        --record-decisions --policy-every 4 --policy-temp 0.1 --policy-head \
        $extra \
        --games 250000 --window 500000 --lr 1e-4 --lr-cosine 60000 \
        --relabel-mode new --out "$out" \
        > "$out/log.txt" 2>&1
    grep -q "learner device: cuda" "$out/log.txt" || {
        echo "ABORT: $out is not on cuda" >&2; head -3 "$out/log.txt" >&2; exit 1; }
    grep -q "mixed fleet: 128 mcts actors" "$out/log.txt" || {
        echo "ABORT: $out fleet split missing" >&2; exit 1; }
    # The arm is the cadence: a combat arm that recorded the default
    # cadence measures the encoder change twice and nothing else.
    if [ "$arm" = "combat" ]; then
      grep -q "declare-blockers/end-combat" "$out/log.txt" || {
          echo "ABORT: $out did not record combat rows" >&2; exit 1; }
    else
      grep -q "no combat rows" "$out/log.txt" || {
          echo "ABORT: $out recorded combat rows; it is not the encoder arm" >&2; exit 1; }
    fi
    echo "$out done: $(tail -1 $out/stats.jsonl | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print("games",d["games"],"mcts",d["games_mcts"],"auc",d["val_auc"],"val_policy",d["val_policy"])')"
  done
done

for seed in 43 97; do
  for arm in enc combat; do
    net="nets_r40_${arm}_s${seed}/best.safetensors"
    for opp in atk-sim gang; do
      for lseed in 43 97; do
        tag="${arm}_s${seed}_vs_${opp}_l${lseed}"
        CRAB_NET="$net" $LADDER --a net --b "$opp" \
            --decks sealed --games 1000 --seed "$lseed" \
            > ".ladder/r40_pilot_${tag}.txt" 2>&1
        echo "pilot ${tag}: $(grep -E 'A win%' ".ladder/r40_pilot_${tag}.txt" | tail -1 | tr -s ' ')"
      done
    done
  done
  # The search question, on the combat arm: does a net trained on
  # mid-combat positions narrow the search's edge over its own pilot?
  for lseed in 43 97; do
    tag="combat_s${seed}_deep_vs_net_l${lseed}"
    CRAB_NET="nets_r40_combat_s${seed}/best.safetensors" $LADDER --a mcts-net-deep --b net \
        --decks sealed --games 100 --seed "$lseed" \
        > ".ladder/r40_search_${tag}.txt" 2>&1
    echo "search ${tag}: $(grep -E 'A win%' ".ladder/r40_search_${tag}.txt" | tail -1 | tr -s ' ')"
  done
done
echo "round-40 complete"
