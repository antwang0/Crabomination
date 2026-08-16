#!/usr/bin/env bash
# Round 39: the two levers round 38 pointed at — inside the search's
# inputs, not the pilot's weights.
#
# Arm SV — search-value targets (Reanalyze-lite). The λ-objective is
# ~91 % self-smoothness and round 27's null was label *provenance*; this
# regresses the win head toward the search's per-arm mean rewards on the
# captured successor states (visit-weighted MSE, --search-value-weight
# 0.25, the auxiliary convention as the first dose), a far
# lower-variance value target on exactly the states a search ranks.
#
# Arm OPP — the opponent-hand belief head. Trains head_opp.* (per-name
# hold probabilities, multi-label BCE vs recorded truth) for
# belief-weighted determinization: the redeal is currently uniform over
# the information set, and held-back mana / declined blocks / uncast
# spells are evidence it ignores.
#
# Both arms are the r38 recipe (mixed fleet 128/128, 250 k games,
# policy head, best-on-AUC) plus exactly one flag, so the r38 cells are
# the controls. Two seeds each.
#
# Gates, pre-registered:
#   Pilot (both arms): net vs atk-sim / gang, 1000 games/archetype x 2
#     ladder seeds. References: champion pooled 52.7 / 51.2 and the r38
#     band (52.5-52.85 / 51.1-51.35). The SV arm is the one with a
#     mechanism to move this; OPP moves it only via aux-learning.
#   Search (SV arm): mcts-net-deep vs net, same weights — a value head
#     trained on search values could narrow the search's ~+4.5 edge
#     over its own 1-ply pilot.
#   Belief (OPP arm, the payoff gate): mcts-net-bdeep vs mcts-net-deep
#     AND net-bdet1 vs net-det1, same weights both sides — isolates the
#     redeal distribution in the rollouts and in the sims respectively.
#     The bot_ladder startup line must say "belief redeal:
#     opponent-hand head"; INERT means the cell measures nothing.
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train
LADDER=./target/release/bot_ladder
PILOT=nets_r28f_full_s43/best.safetensors

[ -f "$PILOT" ] || { echo "missing $PILOT" >&2; exit 1; }

for seed in 43 97; do
  for arm in sv opp; do
    out="nets_r39_${arm}_s${seed}"
    extra="--search-value-weight 0.25"
    [ "$arm" = "opp" ] && extra="--opp-head"
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
    echo "$out done: $(tail -1 $out/stats.jsonl | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print("games",d["games"],"mcts",d["games_mcts"],"auc",d["val_auc"],"val_policy",d["val_policy"],"sv",d.get("policy_sv"),"opp",d.get("loss_opp"))')"
  done
done

for seed in 43 97; do
  for arm in sv opp; do
    net="nets_r39_${arm}_s${seed}/best.safetensors"
    for opp in atk-sim gang; do
      for lseed in 43 97; do
        tag="${arm}_s${seed}_vs_${opp}_l${lseed}"
        CRAB_NET="$net" $LADDER --a net --b "$opp" \
            --decks sealed --games 1000 --seed "$lseed" \
            > ".ladder/r39_pilot_${tag}.txt" 2>&1
        echo "pilot ${tag}: $(grep -E 'A win%' ".ladder/r39_pilot_${tag}.txt" | tail -1 | tr -s ' ')"
      done
    done
  done
  # SV: the search question on the sv net.
  for lseed in 43 97; do
    tag="sv_s${seed}_deep_vs_net_l${lseed}"
    CRAB_NET="nets_r39_sv_s${seed}/best.safetensors" $LADDER --a mcts-net-deep --b net \
        --decks sealed --games 100 --seed "$lseed" \
        > ".ladder/r39_search_${tag}.txt" 2>&1
    echo "search ${tag}: $(grep -E 'A win%' ".ladder/r39_search_${tag}.txt" | tail -1 | tr -s ' ')"
  done
  # OPP: the belief payoff gates, same weights both sides.
  for lseed in 43 97; do
    net="nets_r39_opp_s${seed}/best.safetensors"
    tag="opp_s${seed}_bdeep_l${lseed}"
    CRAB_NET="$net" $LADDER --a mcts-net-bdeep --b mcts-net-deep \
        --decks sealed --games 100 --seed "$lseed" \
        > ".ladder/r39_belief_${tag}.txt" 2>&1
    grep -q "belief redeal: opponent-hand head" ".ladder/r39_belief_${tag}.txt" || {
        echo "ABORT: ${tag} belief head inert" >&2; exit 1; }
    echo "belief ${tag}: $(grep -E 'A win%' ".ladder/r39_belief_${tag}.txt" | tail -1 | tr -s ' ')"
    tag="opp_s${seed}_bdet1_l${lseed}"
    CRAB_NET="$net" $LADDER --a net-bdet1 --b net-det1 \
        --decks sealed --games 1000 --seed "$lseed" \
        > ".ladder/r39_belief_${tag}.txt" 2>&1
    echo "belief ${tag}: $(grep -E 'A win%' ".ladder/r39_belief_${tag}.txt" | tail -1 | tr -s ' ')"
  done
done
echo "round-39 complete"
