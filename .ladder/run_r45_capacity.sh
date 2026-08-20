#!/usr/bin/env bash
# Round 45: capacity, finally fed — 2x widths under the r41 recipe on
# a learner that is measurably not starving.
#
# WHY THIS ROUND. Round 4 ran "5x capacity" and recorded the verdict
# as UNTESTED: the CPU learner managed 0.4 visits/row before the tail
# cap, so the wide net never saw the data budget its size implied.
# Every round since (5-43) trained at champion width. The learner has
# been on CUDA since r41 and the forty-first perf pass roughly doubled
# actor generation on top (smoke 2026-08-20: 20 value actors, 170
# games/s, learner 95 % train duty, ~21 k rows/s consumed at default
# width). The capacity question is the last round-4 lever never run
# under conditions that could answer it. The scaling literature's
# consistent shape (MiniZero across four games): capacity effects
# appear only when the learner is fed — a null here, with feeding
# demonstrated, is a real null and closes the lever; round 4's was
# neither.
#
# THE ARMS. ctrl = the r41 champion recipe at standard widths
# (emb 32, obj 64, trunk 512x256, --attn). cap = the same recipe at
# double widths (emb 64, obj 128, trunk 1024x512, ~4x params). One
# variable moves. lr and window stay pinned — if the wide arm needed
# its own lr, that is a *further* round, not a silent second variable
# in this one.
#
# DESIGN. r41's measurement floor verbatim: FOUR training seeds per
# arm, PAIRED within seed (seed drives init and the generated games;
# the estimate is the mean of four within-pair differences). Gates:
# net vs atk-sim / gang, 1000 games/archetype x sealed x 2 ladder
# seeds per net, pooled per (arm, seed, opp); plus one search cell
# per arm at s43 (mcts-net-deep vs net, same weights both sides) —
# capacity's consumer is the search leaf, so the search gate is part
# of the question this time, unlike r41.
#
# PRE-REGISTERED EXPECTATIONS. The pilot bar is the r38/r41 band
# (52.5-52.85 / 51.1-51.35): cap must beat ctrl's own cells paired,
# not just land in the band. The search cell's reference is ~54-55
# (r38/r39/r40). Prior: genuinely uncertain — r41's +0.4 says the
# representation lever is live, r4's bands say nothing (untested).
# A paired null at four seeds closes capacity at 2x under this
# recipe; a win sets up the 4x cell and lr tuning.
#
# FEEDING ASSERTIONS, the round-4 lesson: every run must report
# "learner device: cuda" and the fleet split, and the write-up reports
# rows-consumed / rows-generated per arm so a starved wide learner is
# visible instead of assumed away.
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=./target/release/selfplay_train
LADDER=./target/release/bot_ladder
PILOT=nets_r28f_full_s43/best.safetensors
SEEDS="43 97 151 199"

[ -f "$PILOT" ] || { echo "missing $PILOT" >&2; exit 1; }

for seed in $SEEDS; do
  for arm in ctrl cap; do
    out="nets_r45_${arm}_s${seed}"
    wide=""
    [ "$arm" = "cap" ] && wide="--emb-dim 64 --obj-hidden 128 --h1 1024 --h2 512"
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
        $wide \
        --games 250000 --window 500000 --lr 1e-4 --lr-cosine 60000 \
        --relabel-mode new --out "$out" \
        > "$out/log.txt" 2>&1
    grep -q "learner device: cuda" "$out/log.txt" || {
        echo "ABORT: $out is not on cuda" >&2; head -3 "$out/log.txt" >&2; exit 1; }
    grep -q "mixed fleet: 128 mcts actors" "$out/log.txt" || {
        echo "ABORT: $out fleet split missing" >&2; exit 1; }
    echo "$out done: $(tail -1 $out/stats.jsonl | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print("games",d["games"],"auc",d["val_auc"],"val_policy",d["val_policy"])')"
  done
done

echo "--- gates ---"
for seed in $SEEDS; do
  for arm in ctrl cap; do
    net="nets_r45_${arm}_s${seed}/best.safetensors"
    for opp in atk-sim gang; do
      for lseed in 43 97; do
        tag="${arm}_s${seed}_vs_${opp}_l${lseed}"
        if [ -s ".ladder/r45_${tag}.txt" ] && grep -q "A win%" ".ladder/r45_${tag}.txt"; then
          echo "gate ${tag}: already done"; continue
        fi
        CRAB_NET="$net" $LADDER --a net --b "$opp" \
            --decks sealed --games 1000 --seed "$lseed" \
            > ".ladder/r45_${tag}.txt" 2>&1
        echo "gate ${tag}: $(grep -E 'A win%' ".ladder/r45_${tag}.txt" | tail -1 | tr -s ' ')"
      done
    done
  done
done

echo "--- search cells (s43, one per arm) ---"
for arm in ctrl cap; do
  net="nets_r45_${arm}_s43/best.safetensors"
  tag="search_${arm}_s43"
  if ! { [ -s ".ladder/r45_${tag}.txt" ] && grep -q "A win%" ".ladder/r45_${tag}.txt"; }; then
    CRAB_NET="$net" $LADDER --a mcts-net-deep --b net \
        --decks sealed --games 1000 --seed 43 \
        > ".ladder/r45_${tag}.txt" 2>&1
  fi
  echo "search ${tag}: $(grep -E 'A win%' ".ladder/r45_${tag}.txt" | tail -1 | tr -s ' ')"
done

echo "--- paired summary ---"
python3 - "$SEEDS" <<'PY'
import re, sys, pathlib, statistics as st
seeds = sys.argv[1].split()
def pooled(arm, seed, opp):
    vals = []
    for lseed in (43, 97):
        p = pathlib.Path(f".ladder/r45_{arm}_s{seed}_vs_{opp}_l{lseed}.txt")
        if not p.exists():
            return None
        m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text())
        if m:
            vals.append(float(m[-1]))
    return sum(vals) / len(vals) if vals else None
T95 = {1: 12.706, 2: 4.303, 3: 3.182, 4: 2.776}
for opp in ("atk-sim", "gang"):
    diffs = []
    for s in seeds:
        c, v = pooled("ctrl", s, opp), pooled("cap", s, opp)
        if c is None or v is None:
            continue
        diffs.append(v - c)
        print(f"  seed {s} vs {opp}: ctrl {c:.2f}  cap {v:.2f}  diff {v-c:+.2f}")
    if len(diffs) >= 2:
        m = st.mean(diffs)
        se = st.stdev(diffs) / len(diffs) ** 0.5
        t = T95[len(diffs) - 1]
        print(f"  {opp}: mean paired diff {m:+.3f} ± {t*se:.3f} (t95, n={len(diffs)})")
PY
echo "round-45 complete"
