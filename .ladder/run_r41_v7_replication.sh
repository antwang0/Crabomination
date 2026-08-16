#!/usr/bin/env bash
# Round 41: does encoder v7's +0.3 survive a design that can resolve it?
#
# Round 40 measured full v7 at 53.0 / 51.5 against the r38 control band
# (52.5-52.85 / 51.1-51.35) — sign-consistent across all eight ladder
# cells and both training seeds. It is not adoptable as measured: the
# effect is the same size as r38's own between-seed spread (0.35 / 0.25),
# so two seeds cannot separate "+0.3" from "a lucky pair of seeds".
#
# Two changes to fix that, both of which the mixed fleet paid for. Runs
# cost ~95 minutes now instead of most of a day, and the experiment
# design never re-tuned around that:
#
#   1. FOUR training seeds per arm, not two.
#   2. PAIRED. The seed drives weight init *and* the generated games, so
#      it is the dominant nuisance variable. Each seed runs both arms;
#      the estimate is the mean of the four within-pair differences, and
#      the seed's contribution differences out. Same logic as the
#      antithetic seat pairs on the ladder, applied one level up.
#
# Control is `--ablate hist,exp,ctr`, which zeroes exactly the round-40
# blocks and leaves the encoder byte-identical to v6. Its extra weight
# columns are exactly dead, so this is a true parity arm and not an
# approximation of one.
#
# CRITICAL, and the reason the gate loop below sets CRAB_ABLATE: a net
# trained with a block ablated has never-trained, random-init weight
# columns for those features. Gating it under the full encoder feeds
# live features into random weights and measures garbage. The control's
# ladder cells must encode exactly as its training run did.
#
# Prior, stated before the numbers: raised since round 40 wrote this up.
# The leaf-side census gave the effect a mechanism it did not have —
# the `hist` block is 1.8-3.6x denser in the positions the search
# evaluates than in the rows the trainer fits, so it is not merely
# occupied, it is occupied where the net is asked to rank. That is a
# reason to expect a real effect, not a reason to believe one.
#
# Gate, pre-registered: net vs atk-sim / gang, 1000 games/archetype x
# sealed x 2 ladder seeds per net. The decision rule is the mean paired
# difference across the four seeds, with the per-pair differences
# printed so a single divergent seed is visible rather than averaged
# away. No search gate: round 40 measured v7 at 54.75 on
# mcts-net-deep vs net against the r38 reference of 54.85, so the
# search path is not the open question here.
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train
LADDER=./target/release/bot_ladder
PILOT=nets_r28f_full_s43/best.safetensors
SEEDS="43 97 151 199"

[ -f "$PILOT" ] || { echo "missing $PILOT" >&2; exit 1; }

for seed in $SEEDS; do
  for arm in ctrl v7; do
    out="nets_r41_${arm}_s${seed}"
    ablate=""
    [ "$arm" = "ctrl" ] && ablate="--ablate hist,exp,ctr"
    # Resume: a run that reached its full game count is kept, anything
    # short is redone from scratch. Eight 95-minute runs is long enough
    # that losing the finished ones to an interrupted sweep is the
    # difference between re-running an hour and re-running a day. The
    # guard reads the completion line rather than the directory, so a
    # half-written output is never mistaken for a result.
    if grep -q "^done: $((250000)) games" "$out/log.txt" 2>/dev/null; then
      echo "$out already complete, skipping"
      continue
    fi
    rm -rf "$out"; mkdir -p "$out"
    # shellcheck disable=SC2086
    $TRAIN --attn --lambda 0.7 --seed "$seed" \
        --use-best "$PILOT" \
        --mcts-actors 64 --mcts-fleet 128 --actors 256 --gpu-eval \
        --record-decisions --policy-every 4 --policy-temp 0.1 --policy-head \
        $ablate \
        --games 250000 --window 500000 --lr 1e-4 --lr-cosine 60000 \
        --relabel-mode new --out "$out" \
        > "$out/log.txt" 2>&1
    grep -q "learner device: cuda" "$out/log.txt" || {
        echo "ABORT: $out is not on cuda" >&2; head -3 "$out/log.txt" >&2; exit 1; }
    grep -q "mixed fleet: 128 mcts actors" "$out/log.txt" || {
        echo "ABORT: $out fleet split missing" >&2; exit 1; }
    # The arm IS the ablation. A control that silently trained the full
    # encoder is a second copy of the treatment, and the pair would then
    # agree for entirely the wrong reason.
    if [ "$arm" = "ctrl" ]; then
      grep -q "encoder ablation: hist, exp, ctr switched off" "$out/log.txt" || {
          echo "ABORT: $out did not ablate the v7 blocks" >&2; exit 1; }
    elif grep -q "encoder ablation" "$out/log.txt"; then
      echo "ABORT: $out ablated something; it is not the v7 arm" >&2; exit 1
    fi
    echo "$out done: $(tail -1 $out/stats.jsonl | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print("games",d["games"],"auc",d["val_auc"],"val_policy",d["val_policy"])')"
  done
done

echo "--- gates ---"
for seed in $SEEDS; do
  for arm in ctrl v7; do
    net="nets_r41_${arm}_s${seed}/best.safetensors"
    # See the note above: the control must encode as it trained.
    ab=""
    [ "$arm" = "ctrl" ] && ab="hist,exp,ctr"
    for opp in atk-sim gang; do
      for lseed in 43 97; do
        tag="${arm}_s${seed}_vs_${opp}_l${lseed}"
        CRAB_ABLATE="$ab" CRAB_NET="$net" $LADDER --a net --b "$opp" \
            --decks sealed --games 1000 --seed "$lseed" \
            > ".ladder/r41_${tag}.txt" 2>&1
        echo "gate ${tag}: $(grep -E 'A win%' ".ladder/r41_${tag}.txt" | tail -1 | tr -s ' ')"
      done
    done
  done
done

# Per-pair differences and their mean — the estimate this round exists
# to produce. Printed per seed so one divergent pair is visible.
python3 - "$SEEDS" <<'PY'
import re, sys, pathlib
seeds = sys.argv[1].split()
def pooled(arm, seed, opp):
    vals = []
    for lseed in (43, 97):
        p = pathlib.Path(f".ladder/r41_{arm}_s{seed}_vs_{opp}_l{lseed}.txt")
        if not p.exists():
            return None
        m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text())
        if m:
            vals.append(float(m[-1]))
    return sum(vals) / len(vals) if vals else None
for opp in ("atk-sim", "gang"):
    print(f"\n{opp}: per-seed paired difference (v7 - control)")
    diffs = []
    for s in seeds:
        c, v = pooled("ctrl", s, opp), pooled("v7", s, opp)
        if c is None or v is None:
            print(f"  seed {s}: incomplete")
            continue
        diffs.append(v - c)
        print(f"  seed {s}: control {c:.2f}  v7 {v:.2f}  diff {v - c:+.2f}")
    if len(diffs) > 1:
        mean = sum(diffs) / len(diffs)
        sd = (sum((d - mean) ** 2 for d in diffs) / (len(diffs) - 1)) ** 0.5
        se = sd / len(diffs) ** 0.5
        print(f"  mean {mean:+.2f}  sd {sd:.2f}  se {se:.2f}  "
              f"95% approx [{mean - 2 * se:+.2f}, {mean + 2 * se:+.2f}]")
PY
echo "round-41 complete"
