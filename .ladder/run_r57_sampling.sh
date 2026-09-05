#!/usr/bin/env bash
# Round 57: actor-side softmax sampling, re-run with the attack chain as the
# exploration mechanism (the training-side half of the round-55 proposal).
#
# WHAT CHANGED SINCE ROUND 23. Round 23 measured `--sample-temp 120
# --sample-turns 6` on the round-20 champion config and read a null (51.7 /
# 53.7 vs the control's 51.8 / 54.4): sampling only ever perturbed the
# FINAL argmax of a decision. Since round 55 the actor pilot
# (`EvalWeights::default()`) grows its attack declaration through the
# chain, and the chain's finalize-or-add step goes through the same
# `choose_scored`, so the same flag now samples at every micro-step of the
# declaration — the coverage argument round 23 lacked: partial declarations
# the argmax pilot never visits get rows, and the outcome label can teach
# a pair that only pays together. Both arms run on the NEW default — the
# round-56 one, with the block chain and the wide attack chain as well —
# so the pilot changed and round 23's control is not reusable. The block
# chain's moves go through the same chooser, so blocks are sampled too.
#
# RECIPE. Round 23's, verbatim (heuristic actors, no net pilot, so the
# argmax fixed point is the heuristic's): champion config
# `--attn --lambda 0.7 --games 250000 --steps 200000 --window 500000
#  --lr 1e-4 --lr-cosine 60000 --relabel-mode new --stop-after-stale 12`,
# arm adds `--sample-temp 120 --sample-turns 6`. Four training seeds
# paired within seed (the r41+ house rule); ~35 min a run at the r55
# default's throughput, eight runs.
#
# GATE. Each trained net pilots `net-bchain` (det1 + attack chain + block
# chain, the scored pilot the client's shape reduces to) against `dflt`
# (the round-56 default, both chains) on ladder
# seeds 43/97, 1 000 games x 12 sealed decks, paired; the reading is the
# ARM-minus-CONTROL difference within each training seed, then pooled.
# Holdout AUC from stats.jsonl is recorded but is NOT comparable across
# arms (the sampled arm's validation games contain exploration moves —
# round 23's caveat).
#
# PRE-REGISTERED READINGS (pooled arm − control, four seeds).
#   * clearly above +0.5           -> adopt sampling as the actor default;
#                                     the micro-step mechanism paid.
#   * within ±0.5                  -> null again; sampling at the chain's
#                                     steps is not the missing exploration.
#                                     Record with round 23; the remaining
#                                     untried arm is sampling under a
#                                     NET-piloted generator.
#   * clearly below −0.5           -> the sampled rows' label quality cost
#                                     more than the coverage bought (the
#                                     "exploration confined to the opening"
#                                     argument); try --sample-turns 3
#                                     before parking.
set -eu
cd /home/archuser/repos/Crabomination
TRAIN=${TRAIN:-./target/release/selfplay_train}
LADDER=${LADDER:-./target/release-fast/bot_ladder}
SEEDS=${SEEDS:-"43 97 151 199"}
GAMES=${GAMES:-1000}

[ -x "$TRAIN" ] || { echo "build it: cargo build --release -p crabomination_ml --features cuda --bin selfplay_train" >&2; exit 1; }
[ -x "$LADDER" ] || { echo "build it: cargo build --profile release-fast -p crabomination --bin bot_ladder" >&2; exit 1; }

for tseed in $SEEDS; do
  for arm in ctrl samp; do
    out="nets_r57_${arm}_s${tseed}"
    if grep -q "^done: 250000 games" "$out/log.txt" 2>/dev/null; then
      echo "$out already complete, skipping"
      continue
    fi
    rm -rf "$out"; mkdir -p "$out"
    extra=""
    [ "$arm" = "samp" ] && extra="--sample-temp 120 --sample-turns 6"
    # shellcheck disable=SC2086
    $TRAIN --attn --lambda 0.7 --seed "$tseed" --games 250000 --steps 200000 \
        --window 500000 --lr 1e-4 --lr-cosine 60000 \
        --relabel-mode new --stop-after-stale 12 \
        $extra \
        --out "$out" > "$out/log.txt" 2>&1
    grep -q "learner device: cuda" "$out/log.txt" || {
        echo "ABORT: $out is not on cuda" >&2; head -3 "$out/log.txt" >&2; exit 1; }
    echo "$out done: $(tail -1 "$out/stats.jsonl" | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print("games",d["games"],"auc",d["val_auc"],"elapsed_s",d["elapsed_s"])')"
  done
done

for tseed in $SEEDS; do
  for arm in ctrl samp; do
    net="nets_r57_${arm}_s${tseed}/best.safetensors"
    [ -f "$net" ] || { echo "missing $net" >&2; continue; }
    for gseed in 43 97; do
      f=".ladder/r57_${arm}_s${tseed}_g${gseed}.txt"
      if [ -s "$f" ] && grep -q "A win%" "$f"; then continue; fi
      CRAB_NET="$net" $LADDER --a net-bchain --b dflt --decks sealed --games "$GAMES" --seed "$gseed" > "$f" 2>&1
      echo "  ${arm} s${tseed} g${gseed}: $(grep -E 'A win%' "$f" | tail -1 | tr -s ' ')"
    done
  done
done

echo "=== summary (arm − control, within training seed) ==="
python3 - <<'PY'
import re, pathlib, statistics as st, os
seeds = os.environ.get("SEEDS", "43 97 151 199").split()
def read(arm, t, g):
    p = pathlib.Path(f".ladder/r57_{arm}_s{t}_g{g}.txt")
    if not p.exists():
        return None
    m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text())
    return float(m[-1]) if m else None
diffs = []
for t in seeds:
    c = [read("ctrl", t, g) for g in (43, 97)]
    s_ = [read("samp", t, g) for g in (43, 97)]
    if None in c or None in s_:
        continue
    d = st.mean(s_) - st.mean(c)
    diffs.append(d)
    print(f"  seed {t}: ctrl {c} samp {s_}  diff {d:+.2f}")
if diffs:
    mean = st.mean(diffs)
    sd = st.pstdev(diffs) if len(diffs) > 1 else float('nan')
    verdict = "adopt" if mean > 0.5 else ("NEGATIVE — try --sample-turns 3" if mean < -0.5 else "null again (record with r23)")
    print(f"  pooled diff {mean:+.2f} over {len(diffs)} seeds (sd {sd:.2f})  verdict: {verdict}")
PY
echo "round-57 complete"
