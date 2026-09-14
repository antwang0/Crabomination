#!/usr/bin/env bash
# Round 73: the round-72 policy gradient on the SETTLED successor (2026-09-13).
# PRE-REGISTERED before any number exists.
#
# QUESTION. Round 72's head-only policy gradient was real (pol − init +6.8 ±3.1,
# four seeds) but its pilot sat at 31 % because the policy head ranked each
# candidate's ONE-ACTION successor — a state the frozen trunk never trained on
# (init − ctrl −26.4). This round trains and consumes the head on the state the
# pickers' scores were read from: a main-phase action resolved to quiescence
# and walked through combat (`settled_successor`), a declaration at its sim's
# leaf (`simulate_attack_leaf` / `simulate_block_leaf`, redeal k = 0). The
# untrained head is then the win head over the states the sims already score,
# so `init` should read ≈ `net67` — and the gate becomes the question round 72
# could not ask: does the policy gradient beat the pilot's own ranking?
#
# DESIGN. As round 72 with two switches: the actors record settled successors
# (`--capture-settled`) and the gate pilot is `net67-pols` (`policy_settled`).
# Head-only, `head_policy := head_win` warm start on the four round-57 control
# nets, `--pilot net67`, `--sample-temp 300 --sample-turns 99`. The measured
# throughput recipe (r72 ABC), halved after an OOM: batch 512, lr 4.2e-4,
# 20 000 steps — the same 10.24 M samples as round 72's gate — with the
# prefetch worker; ACTORS=4
# because settled capture runs one extra sim per candidate on the actor path.
#
# GATES (sealed, paired, ladder seeds 43 / 97, 1 000 games × 12 decks, ±0.65):
#   pol  — `net67-pols` with nets_r73_s{seed}/latest.safetensors vs `dflt`
#   init — `net67-pols` with .ladder/r72/init_s{seed}.safetensors vs `dflt`
#   ctrl — `net67`      with nets_r57_ctrl_s{seed}/best.safetensors vs `dflt`
#          (re-used from .ladder/r72/ctrl_* — same binary date, same default)
#
# PRE-REGISTERED READINGS (pooled over four training seeds, t intervals,
# paired within seed):
#   CHECK    init − ctrl  within ±1.0 -> the settled consumption is the sims'
#                                       own state; the hole is closed. Outside
#                                       it, read the residual first (lookahead
#                                       follow-ups, decided-game clamp, the
#                                       chain's re-score) before the gradient.
#   PRIMARY  pol − init  > +0.5      -> the policy gradient improves on the win
#                                       head's ranking of the states the sims
#                                       score. HEADLINE pol − ctrl then says by
#                                       how much a scored pilot gains over
#                                       `net67`'s +1; > +0.5 there is the first
#                                       training-side adoption candidate since
#                                       the deck net, and the on-policy loop
#                                       (PPO) is next.
#                        within ±0.5 -> the r72 signal was the head learning
#                                       what the sims already knew (a
#                                       representation gap, not a policy gap);
#                                       queue `--pg-trunk` with value steps.
#                        < −0.5      -> the gradient hurts on settled states;
#                                       read clip_frac / det_frac / entropy.
set -eu
cd /home/archuser/repos/Crabomination
MODE=${1:-train}
TRAIN=${TRAIN:-./target/release/selfplay_train}
LADDER=${LADDER:-./target/release-fast/bot_ladder}
SEEDS=${SEEDS:-"43 97 151 199"}
GAMES=${GAMES:-1000}
GAMES_TRAIN=${GAMES_TRAIN:-100000}
# Batch 512 / 20 k steps / lr 3e-4·√2 (2026-09-14): the first attempt at 1024
# / 10 k died at seed 97 step 8 000 with CUDA_ERROR_OUT_OF_MEMORY in the PG
# step (seed 43 had survived the same recipe; nets_r73_b1024_s43 keeps it).
# Same 10.24 M samples at half the peak; all four seeds on one recipe.
STEPS=${STEPS:-20000}
ACTORS=${ACTORS:-4}
TEMP=${TEMP:-300}
PG_BATCH=${PG_BATCH:-512}
LR=${LR:-4.2e-4}
R=.ladder/r73
mkdir -p $R
[ -x "$TRAIN" ] || { echo "build it: cargo build --release -p crabomination_ml --features cuda --bin selfplay_train" >&2; exit 1; }
[ -x "$LADDER" ] || { echo "build it: cargo build --profile release-fast -p crabomination --bin bot_ladder" >&2; exit 1; }
for tseed in $SEEDS; do
  [ -s "nets_r57_ctrl_s${tseed}/best.safetensors" ] || { echo "missing pilot nets_r57_ctrl_s${tseed}/best.safetensors" >&2; exit 1; }
  [ -s ".ladder/r72/init_s${tseed}.safetensors" ] || python3 .ladder/r72/make_init_head.py "nets_r57_ctrl_s${tseed}/best.safetensors" ".ladder/r72/init_s${tseed}.safetensors"
done

train_one() { # tseed games steps out
  local tseed=$1 games=$2 steps=$3 out=$4
  rm -rf "$out"; mkdir -p "$out"
  $TRAIN --pg-only --capture-settled --use-best "nets_r57_ctrl_s${tseed}/best.safetensors" --pilot net67 \
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
  # The consumption check first: the UNTRAINED head on the settled successor
  # against net67 on the same file should read ≈ 50 (24 games × 12 decks).
  CRAB_NET=".ladder/r72/init_s43.safetensors" $LADDER --a net67-pols --b net67 --decks sealed --games 24 --seed 43 --threads 22 2>&1 | grep -E "loaded|A win%|error" || true
  train_one 43 2000 100 "$R/probe_s43"
  grep -E "pg:|pg sanity|SETTLED|ABORT|learner device" "$R/probe_s43/log.txt" || true
  tail -1 "$R/probe_s43/stats.jsonl" | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print({k:d[k] for k in ("games","actor_games_per_s","pg_loss","pg_adv_mean","pg_adv_std","pg_entropy","pg_rho_mean","pg_clip_frac","pg_wide_skipped","decisions_dropped","val_policy")})'
  exit 0
fi

if [ "$MODE" = train ]; then
  for tseed in $SEEDS; do
    out="nets_r73_s${tseed}"
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
    run_cell net67-pols dflt "nets_r73_s${tseed}/latest.safetensors" "$GAMES" 22 "$g" "$R/pol_s${tseed}_g${g}.txt"
    run_cell net67-pols dflt ".ladder/r72/init_s${tseed}.safetensors" "$GAMES" 22 "$g" "$R/init_s${tseed}_g${g}.txt"
    run_cell net67 dflt "nets_r57_ctrl_s${tseed}/best.safetensors" "$GAMES" 22 "$g" ".ladder/r72/ctrl_s${tseed}_g${g}.txt"
  done
done
python3 - <<'PY' | tee $R/summary.txt
import re, pathlib, statistics as st
def read(p):
    p = pathlib.Path(p)
    if not p.exists(): return None
    m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text()); return float(m[-1]) if m else None
prim=[]; chk=[]; head=[]
for t in (43,97,151,199):
    p=[read(f".ladder/r73/pol_s{t}_g{g}.txt") for g in (43,97)]
    i=[read(f".ladder/r73/init_s{t}_g{g}.txt") for g in (43,97)]
    c=[read(f".ladder/r72/ctrl_s{t}_g{g}.txt") for g in (43,97)]
    if None in p or None in i or None in c: continue
    pm, im, cm = st.mean(p), st.mean(i), st.mean(c); prim.append(pm-im); chk.append(im-cm); head.append(pm-cm)
    print(f"  seed {t}: pol {p} ({pm:.2f})  init {i} ({im:.2f})  ctrl {c} ({cm:.2f})  pol-init {pm-im:+.2f}  init-ctrl {im-cm:+.2f}  pol-ctrl {pm-cm:+.2f}")
def pooled(name, rows):
    if not rows: return
    m=st.mean(rows); sd=st.stdev(rows) if len(rows)>1 else float('nan')
    tcrit={1:float('nan'),2:12.706,3:4.303,4:2.776}[min(len(rows),4)]
    half=tcrit*sd/len(rows)**0.5 if len(rows)>1 else float('nan')
    print(f"  pooled {name} {m:+.2f} ±{half:.2f} (t, {len(rows)} seeds)")
pooled("init-ctrl (CHECK)", chk); pooled("pol-init (PRIMARY)", prim); pooled("pol-ctrl (HEADLINE)", head)
PY
echo "=== gate done $(date)"
