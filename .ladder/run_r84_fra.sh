#!/usr/bin/env bash
# Round 84: the FRA value net retrained on RATED-built decks (2026-10-26).
#
# Why. Round 82's actors built their decks with heuristic_sealed_build (the
# noisy v2 sample); the client's FRA opponent now builds with the rated
# builder (item 1, .ladder/r84/opp_fra_s*.txt), and the user's own decks are
# hand-tuned. The net reads positions from decks weaker than the ones it
# plays. Round 83 (fra-fixes actors) lost on BOTH pilots (fixes -2.21,
# default -3.01, .ladder/r84/diag), so this round drops --actor-fra-fixes and
# changes one thing against round 82: --actor-rated-decks.
#
# RECIPE. Round 82's verbatim plus --actor-rated-decks: FRA sealed
# (CRAB_SEALED_SET=fra), --attn --lambda 0.7 --games 250000 --steps 200000
# --window 500000 --lr 1e-4 --lr-cosine 60000 --relabel-mode new
# --stop-after-stale 12 --actors 3; four training seeds, two concurrent drivers.
#
# ⚠ The learner-device line is a GATE: a log without "learner device: cuda"
#   aborts. Binary frozen at .ladder/r84/selfplay_train (built 2026-10-26).
set -eu
cd /home/archuser/repos/Crabomination
export CRAB_SEALED_SET=fra
MODE=${1:-train}
TRAIN=${TRAIN:-.ladder/r84/selfplay_train}
SEEDS=${SEEDS:-"43 97 151 199"}
ACTORS=${ACTORS:-3}
R=.ladder/r84/train
mkdir -p $R
[ -x "$TRAIN" ] || { echo "build it: cargo build --release -p crabomination_ml --features cuda --bin selfplay_train" >&2; exit 1; }

if [ "$MODE" = queue ]; then
  # Wait until no other training job holds the GPU: under 8 GiB in use on
  # two checks a minute apart. Memory, not utilization — the desktop and a
  # running game (dnd-rs, ~2 GiB) render continuously; the job this waits on
  # holds ~15 GiB. Gives up after six hours.
  free=0
  for _ in $(seq 1 360); do
    mem=$(nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits | head -1 | tr -d ' ')
    if [ "$mem" -lt 8192 ]; then free=$((free + 1)); else free=0; fi
    [ "$free" -ge 2 ] && break
    if [ "$free" -eq 1 ]; then
      echo "$(date '+%F %T') GPU free (${mem} MiB in use); confirming in a minute"
    else
      echo "$(date '+%F %T') GPU busy (${mem} MiB in use); waiting"
    fi
    sleep 60
  done
  [ "$free" -ge 2 ] || { echo "$(date '+%F %T') ABORT: the GPU never freed up" >&2; exit 1; }
  echo "=== queue: GPU free, starting both drivers $(date '+%F %T')"
  SEEDS="43 151" "$0" train > $R/driver_a.txt 2>&1 &
  SEEDS="97 199" "$0" train > $R/driver_b.txt 2>&1 &
  wait
  echo "=== queue done $(date '+%F %T')"
  cat $R/driver_a.txt $R/driver_b.txt
  exit 0
fi

if [ "$MODE" = train ]; then
  for tseed in $SEEDS; do
    out="nets_r84_fra_s${tseed}"
    if grep -q "^done: 250000 games" "$out/log.txt" 2>/dev/null; then echo "$out already complete"; continue; fi
    rm -rf "$out"; mkdir -p "$out"
    echo "=== train s$tseed start $(date)"
    $TRAIN --attn --lambda 0.7 --seed "$tseed" --games 250000 --steps 200000 \
        --actors "$ACTORS" --window 500000 --lr 1e-4 --lr-cosine 60000 \
        --relabel-mode new --stop-after-stale 12 --actor-rated-decks --out "$out" > "$out/log.txt" 2>&1
    grep -q "learner device: cuda" "$out/log.txt" || { echo "ABORT: $out is not on cuda" >&2; head -3 "$out/log.txt" >&2; exit 1; }
    echo "$out done: $(tail -1 "$out/stats.jsonl" | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print("games",d["games"],"auc",d["val_auc"],"elapsed_s",d["elapsed_s"])')"
  done
  echo "=== train done $(date)"
  exit 0
fi

# ---- gate (pre-registered 2026-10-26, before any training ran) ----
# FRA sealed. Reference: round 82's FRA net (nets/fra-champion.safetensors).
# Two fields: RATED (CRAB_SEALED_BUILDER=rated, the decks the client's
# opponent builds after item 1 — the distribution this round targets) and
# DEFAULT (the r82/r83 gates' field).
#   scored — each r84 net and the r82 net pilot `net67-fixes` vs `fra-fixes`,
#            1 000 games x 12 decks, ladder seeds 43 / 97, both fields.
#   search — the best r84 net (by its rated-field scored cells) and the r82
#            net as the leaf of `mcts-net67-fixes` vs `mcts-fixes-256`,
#            300 x 12, ladder seeds 43 / 97, rated field.
# READINGS (t interval over the four training seeds of fresh − r82):
#   rated scored > +0.5, default scored > −0.5, and search best − r82 > +0.5
#        -> propose the best r84 net as nets/fra-champion.safetensors
#           (the user's call; no swap without it).
#   otherwise -> record; the r82 net stays.
if [ "$MODE" = gate ]; then
  LADDER=${LADDER:-.ladder/r84/bot_ladder}
  GAMES=${GAMES:-1000}
  run_cell() { # field arm control net games seed out
    local f=$7
    if [ -s "$f" ] && grep -q "A win%" "$f"; then return; fi
    local b=""; [ "$1" = rated ] && b=rated
    CRAB_SEALED_BUILDER=$b CRAB_NET="$4" $LADDER --a "$2" --b "$3" --decks sealed --games "$5" --seed "$6" --threads 22 > "$f" 2>&1
    echo "  $(basename "$f"): $(grep -E 'A win%' "$f" | tail -1 | tr -s ' ')"
  }
  echo "=== gate start $(date)"
  for field in rated default; do for g in 43 97; do
    run_cell $field net67-fixes fra-fixes nets/fra-champion.safetensors "$GAMES" "$g" "$R/${field}_r82_g${g}.txt"
    for tseed in 43 97 151 199; do
      run_cell $field net67-fixes fra-fixes "nets_r84_fra_s${tseed}/best.safetensors" "$GAMES" "$g" "$R/${field}_fresh_s${tseed}_g${g}.txt"
    done
  done; done
  python3 - <<'PY' | tee $R/summary_scored.txt
import re, pathlib, statistics as st
R = ".ladder/r84/train"
def read(p):
    m = re.findall(r"A win%\s+([0-9.]+)%", pathlib.Path(p).read_text()); return float(m[-1])
best = None
for field in ("rated", "default"):
    ref = [read(f"{R}/{field}_r82_g{g}.txt") for g in (43, 97)]
    print(f"{field} field: r82 net {ref} (mean {st.mean(ref):.2f})"); d = []
    for t in (43, 97, 151, 199):
        f = [read(f"{R}/{field}_fresh_s{t}_g{g}.txt") for g in (43, 97)]
        d.append(st.mean(a - b for a, b in zip(f, ref)))
        print(f"  seed {t}: {f}  fresh-r82 {d[-1]:+.2f}")
        if field == "rated" and (best is None or st.mean(f) > best[1]): best = (t, st.mean(f))
    m = st.mean(d); h = 3.182 * st.stdev(d) / 2
    print(f"  pooled fresh-r82 {m:+.2f} [{m-h:+.2f}, {m+h:+.2f}] (t, 4 seeds)")
    open(f"{R}/pooled_{field}.txt", "w").write(f"{m}\n")
print(f"best fresh seed (rated field) {best}")
open(f"{R}/best_seed.txt", "w").write(f"{best[0]}\n")
PY
  rated=$(cat $R/pooled_rated.txt); dflt=$(cat $R/pooled_default.txt)
  if python3 -c "import sys; sys.exit(0 if $rated > 0.5 and $dflt > -0.5 else 1)"; then
    bs=$(cat $R/best_seed.txt)
    echo "--- search cells: fresh s$bs vs r82 $(date)"
    for g in 43 97; do
      run_cell rated mcts-net67-fixes mcts-fixes-256 nets/fra-champion.safetensors 300 "$g" "$R/search_r82_g${g}.txt"
      run_cell rated mcts-net67-fixes mcts-fixes-256 "nets_r84_fra_s${bs}/best.safetensors" 300 "$g" "$R/search_fresh_s${bs}_g${g}.txt"
    done
  else
    echo "scored bar not met (rated $rated, default $dflt): search cells not run"; touch $R/gate.stopped
  fi
  touch $R/gate.done
  echo "=== gate done $(date)"
fi
