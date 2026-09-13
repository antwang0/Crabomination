#!/usr/bin/env bash
# Round 72: an off-policy, advantage-weighted policy gradient on the decision
# stream, head-only (2026-09-13). PRE-REGISTERED before any number exists.
#
# QUESTION. No round has ever optimised a policy for RETURN: every pilot-weight
# round trained the value head (label-source nulls r14/r18/r27, capacity closed
# r45), and the one policy-target round (r35/36, distillation of MCTS root
# means) read a small real pilot gain. The net's policy head scores a
# successor state, so the policy over a decision is the softmax over its
# candidates' head logits — the only parameterised policy this pipeline can
# express. Does training that head on the game result, corrected for the
# behaviour that generated the decisions, make a better SCORED pilot than the
# win head's own ranking?
#
# DESIGN (head-only, so the control is the same net). The learner loads a
# headless round-57 control net, initialises `head_policy := head_win` (the
# policy starts as exactly the pilot's ranking) and runs `--pg-only`: no value
# step, trunk and win head frozen, only the 257-parameter head moves. The gate
# pilot `net67-pol` = `net67` (dflt + the net leaf) with the three live pickers
# — main-phase finalists, attack and block declarations — ranking by the policy
# head on each candidate's one-action successor (the state the recorder trained
# it on). Its control `net67` on the SAME FILE differs by that head swap and
# nothing else, so `pol − ctrl` within training seed isolates the head.
#
# ACTORS. Self-play mirror, `--pilot net67` (the gate's own profile; the old
# `--use-best` default `net_eval_det1` chains four adoptions behind dflt),
# every decision sampled: `--sample-temp 300 --sample-turns 99`. On the net's
# p·10 000 scale, T = 300 puts a candidate 3 win-points behind the argmax at
# weight e^-1 — the pilot still plays its game (round 57's caveat on result
# labels) while near-ties get enough mass that a ratio ≤ 2 is reachable. The
# learner's one-shot sanity aborts the run if mean π_b(chosen) over the first
# batch leaves [0.5, 0.9]. Behaviour record (scores + temperature) per
# decision; ratio truncated at 2; baseline = Σ_j π_θ(j)·V(succ_j) from the
# frozen win head; G = the game result (λ = 1); entropy bonus 0.01.
#
# RECIPE. 100 000 games a seed, four training seeds (43 / 97 / 151 / 199),
# `--steps 40000 --lr 3e-4 --lr-cosine 40000 --pg-batch 256`; the gated
# artifact is `latest.safetensors` after the fixed step count — NOT `best`:
# `val_policy` is imitation of the behaviour (the opposite objective), AUC is
# the frozen head, and the max of a holdout PG statistic over ~100 checkpoints
# is winner's-curse optimistic. Size ACTORS from `MODE=probe` first.
#
# PROBE (2026-09-13, before the budget): sanity 0.638 (in band), 417 games/s
# with capture on (22 actors), 0 dropped, 0.15 % wide rows skipped — and the
# 24-game smoke read `net67-pol` at 24.0 % vs `net67` after 200 steps, with the
# UNTRAINED warm-start file (head_policy := head_win, made by
# .ladder/r72/make_init_head.py) at 23.6 %. That is the CONSUMPTION, not the
# gradient: the policy-rank pilot scores the one-action successor, a state the
# frozen trunk never trained on, and round 44 already recorded that the net
# cannot score unsettled states. A two-arm gate against `net67` would read
# that ~−26 hole and nothing else. So the gate gains a third arm.
#
# GATES (sealed, paired, ladder seeds 43 / 97, 1 000 games × 12 decks, ±0.6):
#   pol  — `net67-pol` with nets_r72_s{seed}/latest.safetensors vs `dflt`
#   init — `net67-pol` with .ladder/r72/init_s{seed}.safetensors vs `dflt`
#          (the same file untrained: head_policy = head_win, bit for bit)
#   ctrl — `net67`     with nets_r57_ctrl_s{seed}/best.safetensors vs `dflt`
# The control is RE-RUN rather than read from .ladder/r68/old_*: those cells
# ran on the round-67 default, and rounds 68 and 70 have moved `dflt` since.
#
# PRE-REGISTERED READINGS (pooled over four training seeds, t intervals,
# paired within seed):
#   PRIMARY  pol − init  > +0.5  -> the policy gradient learned something the
#                                   win head's ranking of the same states did
#                                   not carry; the head-only PG signal is real.
#                                   Whether it can ever close the consumption
#                                   hole is the next question (`--pg-trunk`,
#                                   or a settled-successor consumption).
#                       within ±0.5 -> a linear re-ranker over frozen features
#                                   of unsettled states cannot learn from this
#                                   signal at this budget; the head-only arm is
#                                   closed. Trunk arm needs a value-step
#                                   partner to keep the win head calibrated.
#                       < −0.5    -> the PG signal hurt; read pg_clip_frac /
#                                   pg_det_frac / the entropy trajectory.
#   SECONDARY init − ctrl          -> the consumption hole, measured (expected
#                                   ≈ −26 from the smoke). Recorded, not gated.
#   Also recorded: val_policy (agreement with the SAMPLED behaviour pick; the
#   probe read 0.47 at init because the head ranks one-action successors while
#   the behaviour ranked sim-settled outcomes), decisions_dropped.
#
# PACING. The decision deque holds 200 k decisions (~8 k games). With 22
# actors generation is over in 4 minutes and the learner's tail would train
# ~40 k steps on 8 % of the data; with ACTORS=3 (~57 games/s, round 69's
# regime) 100 k games last ~30 minutes, the learner (~22 steps/s) runs
# alongside, and the reuse cap keeps it ≤ 6× on any decision.
set -eu
cd /home/archuser/repos/Crabomination
MODE=${1:-train}
TRAIN=${TRAIN:-./target/release/selfplay_train}
LADDER=${LADDER:-./target/release-fast/bot_ladder}
SEEDS=${SEEDS:-"43 97 151 199"}
GAMES=${GAMES:-1000}
GAMES_TRAIN=${GAMES_TRAIN:-100000}
STEPS=${STEPS:-10000}
ACTORS=${ACTORS:-3}
# Throughput recipe (post-gate A/B, 2026-09-13): the learner step is
# launch-bound — 16.6 ms whatever the batch — so a 4x batch at 1/4 the steps
# sees the same 10 M samples in far fewer launches; lr scaled by sqrt(4).
# Paired with the prefetch worker (packs the next batch on the CPU while the
# GPU runs the current one). `MODE=throughput` runs seed 43 under it into
# nets_r72b_s43 and prints its timings against nets_r72_s43's.
# Defaults are the MEASURED recipe (ABC run, 2026-09-13 21:52, quiet box, ABCABC):
# batch 1024 + prefetch = 13.5 k decisions/s vs 10.8 k (prefetch, 256) vs
# 9.0 k (inline, 256); strength-neutral on seed 43 (28.4/32.6 vs 28.4/32.3).
# The round-72 gate itself ran at PG_BATCH=256 LR=3e-4 STEPS=40000.
PG_BATCH=${PG_BATCH:-1024}
LR=${LR:-6e-4}
TEMP=${TEMP:-300}
GPU_EVAL=${GPU_EVAL:-0}
R=.ladder/r72
mkdir -p $R
[ -x "$TRAIN" ] || { echo "build it: cargo build --release -p crabomination_ml --features cuda --bin selfplay_train" >&2; exit 1; }
[ -x "$LADDER" ] || { echo "build it: cargo build --profile release-fast -p crabomination --bin bot_ladder" >&2; exit 1; }
for tseed in $SEEDS; do
  [ -s "nets_r57_ctrl_s${tseed}/best.safetensors" ] || { echo "missing pilot nets_r57_ctrl_s${tseed}/best.safetensors" >&2; exit 1; }
  [ -s "$R/init_s${tseed}.safetensors" ] || python3 .ladder/r72/make_init_head.py "nets_r57_ctrl_s${tseed}/best.safetensors" "$R/init_s${tseed}.safetensors"
done
gpu=""; [ "$GPU_EVAL" = 1 ] && gpu="--gpu-eval"

train_one() { # tseed games steps out
  local tseed=$1 games=$2 steps=$3 out=$4
  rm -rf "$out"; mkdir -p "$out"
  $TRAIN --pg-only --use-best "nets_r57_ctrl_s${tseed}/best.safetensors" --pilot net67 $gpu \
      --sample-temp "$TEMP" --sample-turns 99 --seed "$tseed" --games "$games" --steps "$steps" \
      --lr "$LR" --lr-cosine "$steps" --pg-batch "$PG_BATCH" --pg-entropy 0.01 --pg-rho-max 2 --pg-lambda 1 \
      --actors "$ACTORS" --holdout 0.05 --checkpoint-every 1000 --out "$out" > "$out/log.txt" 2>&1
}

run_cell() { # arm control net games threads seed out
  local f=$7
  if [ -s "$f" ] && grep -q "A win%" "$f"; then return; fi
  CRAB_NET="$3" $LADDER --a "$1" --b "$2" --decks sealed --games "$4" --seed "$6" --threads "$5" > "$f" 2>&1
  echo "  $(basename "$f"): $(grep -E 'A win%' "$f" | tail -1 | tr -s ' ')"
}

if [ "$MODE" = probe ]; then
  # 2 000 games, 200 steps: the sanity line, the capture cost, the drop count,
  # and a 24-game ladder smoke of the pol profile (and the headless refusal).
  train_one 43 2000 200 "$R/probe_s43"
  grep -E "pg:|pg sanity|ABORT|learner device" "$R/probe_s43/log.txt" || true
  tail -1 "$R/probe_s43/stats.jsonl" | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print({k:d[k] for k in ("games","actor_games_per_s","pg_loss","pg_adv_mean","pg_adv_std","pg_entropy","pg_rho_mean","pg_clip_frac","pg_det_frac","pg_wide_skipped","decisions_dropped","val_policy","val_pg_adv")})'
  CRAB_NET="$R/probe_s43/latest.safetensors" $LADDER --a net67-pol --b net67 --decks sealed --games 24 --seed 43 --threads 22 2>&1 | grep -E "loaded|A win%|error" || true
  if CRAB_NET="nets_r57_ctrl_s43/best.safetensors" $LADDER --a net67-pol --b net67 --decks sealed --games 2 --seed 43 --threads 2 > /dev/null 2>&1; then
    echo "BUG: net67-pol ran on a headless net" >&2; exit 1
  else
    echo "headless refusal OK"
  fi
  exit 0
fi

if [ "$MODE" = throughput ]; then
  # Same data budget as a gate seed (10.24 M samples), 4x batch, 1/4 steps,
  # sqrt-scaled lr; then the timing table and a 2-cell strength read of the
  # artifact (polb − init on the same ladder seeds).
  PG_BATCH=1024 LR=6e-4 STEPS=10000
  out="nets_r72b_s43"
  if ! { [ -s "$out/latest.safetensors" ] && grep -q "\"step\":$STEPS," "$out/stats.jsonl" 2>/dev/null; }; then
    echo "=== throughput s43 start $(date)"
    train_one 43 "$GAMES_TRAIN" "$STEPS" "$out"
    grep -q "learner device: cuda" "$out/log.txt" || { echo "ABORT: $out is not on cuda" >&2; exit 1; }
  fi
  python3 - <<'PY'
import json
def last(p):
    with open(p) as f: return json.loads(f.readlines()[-1])
a, b = last("nets_r72_s43/stats.jsonl"), last("nets_r72b_s43/stats.jsonl")
for name, d, bs in (("batch 256 (gate recipe)", a, 256), ("batch 1024 + prefetch", b, 1024)):
    print(f"  {name:28s} steps {d['step']:6d}  samples {d['step']*bs/1e6:5.2f} M  games {d['games']:6d}  elapsed {d['elapsed_s']:5.0f} s  steps/s {d['steps_per_s']:5.1f}  samples/s {d['steps_per_s']*bs:7.0f}  pg_entropy {d['pg_entropy']:.3f}  pg_adv_mean {d['pg_adv_mean']:+.4f}  val_policy {d['val_policy']:.3f}")
print(f"  wall clock ratio (b/a): {b['elapsed_s']/a['elapsed_s']:.2f}")
PY
  for g in 43 97; do
    run_cell net67-pol dflt "$out/latest.safetensors" "$GAMES" 22 "$g" "$R/polb_s43_g${g}.txt"
  done
  exit 0
fi

if [ "$MODE" = train ]; then
  for tseed in $SEEDS; do
    out="nets_r72_s${tseed}"
    if [ -s "$out/latest.safetensors" ] && grep -q "\"step\":$STEPS," "$out/stats.jsonl" 2>/dev/null; then echo "$out already complete"; continue; fi
    echo "=== train s$tseed start $(date)"
    train_one "$tseed" "$GAMES_TRAIN" "$STEPS" "$out"
    grep -q "learner device: cuda" "$out/log.txt" || { echo "ABORT: $out is not on cuda" >&2; head -3 "$out/log.txt" >&2; exit 1; }
    grep -q "pg sanity" "$out/log.txt" || { echo "ABORT: $out never reached a PG step" >&2; exit 1; }
    echo "$out done: $(tail -1 "$out/stats.jsonl" | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print("step",d["step"],"games",d["games"],"pg_adv_mean",d["pg_adv_mean"],"pg_entropy",d["pg_entropy"],"pg_clip_frac",d["pg_clip_frac"],"val_policy",d["val_policy"],"dropped",d["decisions_dropped"],"elapsed_s",d["elapsed_s"])')"
  done
  echo "=== train done $(date)"
  exit 0
fi

# ---- gate ----
echo "=== gate start $(date)"
for tseed in $SEEDS; do
  for g in 43 97; do
    run_cell net67-pol dflt "nets_r72_s${tseed}/latest.safetensors" "$GAMES" 22 "$g" "$R/pol_s${tseed}_g${g}.txt"
    run_cell net67-pol dflt "$R/init_s${tseed}.safetensors" "$GAMES" 22 "$g" "$R/init_s${tseed}_g${g}.txt"
    run_cell net67 dflt "nets_r57_ctrl_s${tseed}/best.safetensors" "$GAMES" 22 "$g" "$R/ctrl_s${tseed}_g${g}.txt"
  done
done
python3 - <<'PY' | tee $R/summary.txt
import re, pathlib, statistics as st
def read(p):
    p = pathlib.Path(p)
    if not p.exists(): return None
    m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text()); return float(m[-1]) if m else None
prim=[]; sec=[]
for t in (43,97,151,199):
    p=[read(f".ladder/r72/pol_s{t}_g{g}.txt") for g in (43,97)]
    i=[read(f".ladder/r72/init_s{t}_g{g}.txt") for g in (43,97)]
    c=[read(f".ladder/r72/ctrl_s{t}_g{g}.txt") for g in (43,97)]
    if None in p or None in i or None in c: continue
    pm, im, cm = st.mean(p), st.mean(i), st.mean(c); prim.append(pm-im); sec.append(im-cm)
    print(f"  seed {t}: pol {p} ({pm:.2f})  init {i} ({im:.2f})  ctrl {c} ({cm:.2f})  pol-init {pm-im:+.2f}  init-ctrl {im-cm:+.2f}")
def pooled(name, rows):
    if not rows: return
    m=st.mean(rows); sd=st.stdev(rows) if len(rows)>1 else float('nan')
    tcrit={1:float('nan'),2:12.706,3:4.303,4:2.776}[min(len(rows),4)]
    half=tcrit*sd/len(rows)**0.5 if len(rows)>1 else float('nan')
    print(f"  pooled {name} {m:+.2f} ±{half:.2f} (t, {len(rows)} seeds)")
pooled("pol-init (PRIMARY)", prim); pooled("init-ctrl (consumption hole)", sec)
PY
echo "=== gate done $(date)"
