#!/usr/bin/env bash
# Round 81: the value net retrained on today's engine (2026-10-20).
#
# Round 69's recipe, verbatim (`run_r69_retrain.sh train`), on the actors as
# they stand at the round-81 tip — rounds 70-80's adopted defaults and the
# engine fixes since 2026-09-07. Training only; the gate is round 69's `gate`
# half pointed at these nets, run when the training is read.
#
# RECIPE. --attn --lambda 0.7 --games 250000 --steps 200000 --window 500000
#   --lr 1e-4 --lr-cosine 60000 --relabel-mode new --stop-after-stale 12
#   --actors 3; four training seeds as two concurrent drivers:
#   SEEDS="43 151" run_r81_retrain.sh train   and   SEEDS="97 199" ... train
# (`queue` starts both once the GPU is free.)
#
# ⚠ The learner-device line is a GATE (ML_NOTES, round 28's lesson): a run
# whose log doesn't say "learner device: cuda" aborts — the binary must be
#   cargo build --release -p crabomination_ml --features cuda --bin selfplay_train
set -eu
cd /home/archuser/repos/Crabomination
MODE=${1:-train}
TRAIN=${TRAIN:-./target/release/selfplay_train}
SEEDS=${SEEDS:-"43 97 151 199"}
ACTORS=${ACTORS:-3}
R=.ladder/r81
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
    out="nets_r81_s${tseed}"
    if grep -q "^done: 250000 games" "$out/log.txt" 2>/dev/null; then echo "$out already complete"; continue; fi
    rm -rf "$out"; mkdir -p "$out"
    echo "=== train s$tseed start $(date)"
    $TRAIN --attn --lambda 0.7 --seed "$tseed" --games 250000 --steps 200000 \
        --actors "$ACTORS" --window 500000 --lr 1e-4 --lr-cosine 60000 \
        --relabel-mode new --stop-after-stale 12 --out "$out" > "$out/log.txt" 2>&1
    grep -q "learner device: cuda" "$out/log.txt" || { echo "ABORT: $out is not on cuda" >&2; head -3 "$out/log.txt" >&2; exit 1; }
    echo "$out done: $(tail -1 "$out/stats.jsonl" | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print("games",d["games"],"auc",d["val_auc"],"elapsed_s",d["elapsed_s"])')"
  done
  echo "=== train done $(date)"
  exit 0
fi

# ---- gate (pre-registered 2026-10-20, before any cell ran) ----
# The reference is the committed champion (`nets/champion.safetensors`); the
# round-57 control nets round 69 compared against are not on this box.
#   scored — each r81 net pilots `net67` (= `dflt` + the net leaf) vs `dflt`,
#            sealed, 1 000 games x 12 decks, ladder seeds 43 / 97; the
#            champion pilots the same two cells.
#   search — the best r81 net (by its two scored cells) and the champion as
#            the leaf of `mcts-net67-256` vs `mcts-dflt-256`, 500 games x 12
#            decks, ladder seeds 43 / 97.
# READINGS (t interval over the four training seeds of fresh − champion,
# each seed's two cells averaged):
#   scored fresh − champion > +0.5   -> the r81 nets are the better scored
#                                       pilot; record, propose a champion swap.
#                within ±0.5         -> no change on the scored pilot.
#                < −0.5              -> the fresh nets are worse; keep champion.
#   search best fresh − champion > +0.5 -> a lobby candidate (record, don't flip).
if [ "$MODE" = gate ]; then
  LADDER=${LADDER:-./target/release-fast/bot_ladder}
  GAMES=${GAMES:-1000}
  [ -x "$LADDER" ] || { echo "build it: cargo build --profile release-fast -p crabomination --bin bot_ladder" >&2; exit 1; }
  run_cell() { # arm control net games threads seed out
    local f=$7
    if [ -s "$f" ] && grep -q "A win%" "$f"; then return; fi
    CRAB_NET="$3" $LADDER --a "$1" --b "$2" --decks sealed --games "$4" --seed "$6" --threads "$5" > "$f" 2>&1
    echo "  $(basename "$f"): $(grep -E 'A win%' "$f" | tail -1 | tr -s ' ')"
  }
  echo "=== gate start $(date)"
  for g in 43 97; do
    run_cell net67 dflt nets/champion.safetensors "$GAMES" 22 "$g" "$R/champ_g${g}.txt"
    for tseed in 43 97 151 199; do
      run_cell net67 dflt "nets_r81_s${tseed}/best.safetensors" "$GAMES" 22 "$g" "$R/fresh_s${tseed}_g${g}.txt"
    done
  done
  python3 - <<'PY' | tee $R/summary_scored.txt
import re, pathlib, statistics as st
def read(p):
    p = pathlib.Path(p)
    if not p.exists(): return None
    m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text()); return float(m[-1]) if m else None
champ = [read(f".ladder/r81/champ_g{g}.txt") for g in (43, 97)]
cm = st.mean(champ)
print(f"  champion: {champ} (mean {cm:.2f})")
d = []; best = None
for t in (43, 97, 151, 199):
    f = [read(f".ladder/r81/fresh_s{t}_g{g}.txt") for g in (43, 97)]
    fm = st.mean(f); d.append(st.mean(fi - ci for fi, ci in zip(f, champ)))
    print(f"  seed {t}: {f} (mean {fm:.2f})  fresh-champion {d[-1]:+.2f}")
    if best is None or fm > best[1]: best = (t, fm)
m = st.mean(d); half = 3.182 * st.stdev(d) / len(d) ** 0.5   # t(0.975, df=3)
print(f"  pooled fresh-champion {m:+.2f} [{m-half:+.2f}, {m+half:+.2f}] (t, 4 seeds); best fresh seed {best}")
open(".ladder/r81/best_seed.txt", "w").write(f"{best[0]}\n")
PY
  BF=$(cat $R/best_seed.txt)
  echo "--- search cells: fresh s$BF vs champion $(date)"
  for g in 43 97; do
    run_cell mcts-net67-256 mcts-dflt-256 "nets_r81_s${BF}/best.safetensors" 500 22 "$g" "$R/search_fresh_s${BF}_g${g}.txt"
    run_cell mcts-net67-256 mcts-dflt-256 nets/champion.safetensors 500 22 "$g" "$R/search_champ_g${g}.txt"
  done
  echo "=== gate done $(date)"
  exit 0
fi

echo "usage: $0 queue|train|gate" >&2
exit 2
