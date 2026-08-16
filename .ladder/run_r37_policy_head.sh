#!/usr/bin/env bash
# Round 37: the separate policy head, and the Gumbel root search that
# consumes it.
#
# Round 36 closed with the coupling: distillation into the win head makes
# the search *stronger* rather than redundant, because the search eats
# the same scalar as its leaf reward — and named the two ways out, "a
# separate policy head the search does not consume, or a search whose
# leaf value is frozen". This round builds the first: `head_policy.*`
# over the shared trunk, trained by the same soft-target distillation as
# round 35, exported, and consumed by the search only as its *prior* —
# the win head stays the leaf reward, untouched by policy gradients
# (`policy_steps_do_not_touch_the_win_head`).
#
# The consumer is not P-UCT (round 29: priors as a visit bonus at 64
# iterations were -3.7, plausibly starving arms the heuristic underrates)
# but Sequential Halving over Gumbel-perturbed prior logits, arms scored
# g + logit + sigma(q_hat) — Danihelka et al. (ICLR 2022), the allocator
# with a policy-improvement guarantee at exactly this budget shape. Its
# captures store improved-policy logits (logit + sigma(q_hat)), the
# principled successor to round 35's softmax(means / 0.1).
#
# Part 1 — gen-0 training, two seeds. The r35 recipe verbatim plus
# --policy-head --best-metric policy, so the existing r35 cells
# (nets_r35_{control,distil}_s{43,97}: same pilot, same seeds, same
# budget) are the controls. Pre-registered readouts:
#   val_policy   vs r35 distil (0.726/0.814): the head should match or
#                beat the shared-head result...
#   val_auc      vs r35 control (0.7815/0.7826): ...while NOT paying the
#                round-33 value tax (-0.010/-0.020), since the win head
#                no longer shares its scalar with the ranking objective.
#                Seed spread on AUC is 0.023 (round 32) — read the pair.
# Caveat for the r35 comparison: this round also fixed the decision
# holdout to split by the rows' trajectory hash (it hashed the game
# index before, leaking a game's positions into training while its
# decisions validated), so val_policy membership differs from r35's.
# Same distribution, different draw.
#
# Part 2 — the allocator gate: mcts-net-gumbel vs mcts-net-deep,
# IDENTICAL weights both sides (the round-36 design), 100 games per
# sealed archetype per ladder seed. The head nets measure SH + learned
# prior; the headless champion cell is the fallback path — SH + gumbel
# over heuristic-score priors — which isolates the allocator alone.
# At ~±2 per cell, one cell of a pair is not a reading (round 36).
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train
LADDER=./target/release/bot_ladder
PILOT=nets_r33_control_s43/best.safetensors

[ -f "$PILOT" ] || { echo "missing $PILOT" >&2; exit 1; }
[ -x "$TRAIN" ] || { echo "missing $TRAIN (cargo build --release --features cuda -p crabomination_ml --bin selfplay_train)" >&2; exit 1; }
[ -x "$LADDER" ] || { echo "missing $LADDER (cargo build --release -p crabomination --bin bot_ladder)" >&2; exit 1; }

for seed in 43 97; do
  out="nets_r37_head_s${seed}"
  rm -rf "$out"; mkdir -p "$out"
  $TRAIN --attn --lambda 0.7 --seed "$seed" \
      --use-best "$PILOT" --mcts-actors 64 --gpu-eval \
      --record-decisions --policy-every 4 --policy-temp 0.1 \
      --policy-head --best-metric policy \
      --games 20000 --window 500000 --lr 1e-4 --lr-cosine 60000 \
      --relabel-mode new --out "$out" \
      > "$out/log.txt" 2>&1
  grep -q "learner device: cuda" "$out/log.txt" || {
      echo "ABORT: $out is not on cuda" >&2; head -3 "$out/log.txt" >&2; exit 1; }
  grep -q "pilot baseline scored" "$out/log.txt" || {
      echo "ABORT: $out has no pilot reference; the result would be uninterpretable" >&2
      exit 1; }
  grep -q "into the policy head" "$out/log.txt" || {
      echo "ABORT: $out trained the win head, not the policy head" >&2; exit 1; }
  echo "$out done"
done

# The gate binary checks its own prior source: "gumbel priors: policy
# head" for the r37 nets, "...heuristic candidate scores" for the
# champion control. Both lines land in the cell logs below.
for net in nets_r37_head_s43/best.safetensors nets_r37_head_s97/best.safetensors nets/champion.safetensors; do
  for lseed in 43 97; do
    tag="$(basename "$(dirname "$net")")_l${lseed}"
    CRAB_NET="$net" $LADDER --a mcts-net-gumbel --b mcts-net-deep \
        --decks sealed --games 100 --seed "$lseed" \
        > ".ladder/r37_gumbel_${tag}.txt" 2>&1
    echo "gate ${tag} done: $(grep -E 'A win%|verdict' ".ladder/r37_gumbel_${tag}.txt" | tail -2 | tr '\n' ' ')"
  done
done
echo "round-37 complete"
