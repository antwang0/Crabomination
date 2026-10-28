#!/usr/bin/env bash
# Round 87: the FRA value net's two open recipe questions, at a PINNED ratio
# (2026-10-28, pre-registered before any run).
#
# Q1 — round 83 trained on `fra-fixes` actors and lost (−2.2 / −3.0), but its
#      learner took 0.31–0.34 steps a game against r82's 0.27: the ratio and
#      the actors were confounded. Arm F = round 86's winning recipe (rated
#      decks, 0.274) + --actor-fra-fixes (the bundle the client plays, now
#      with clock_eval 4).
# Q2 — strength tracked the ratio in rounds 82 / 84. Arms R18 and R40 = round
#      86's rated recipe at 0.18 and 0.40 steps a game.
# Reference R274 = round 86's rated arm (nets_r86_rated_s*, 0.274, trained by
#      .ladder/r86/selfplay_train; its default actors are unchanged since).
# All: CRAB_SEALED_SET=fra --attn --lambda 0.7 --games 250000 --steps 200000
# --window 500000 --lr 1e-4 --lr-cosine 60000 --relabel-mode new
# --stop-after-stale 12 --actors 3 --actor-rated-decks; four drivers, one per
# training seed. Binary frozen at .ladder/r87/selfplay_train.
set -eu
cd /home/archuser/repos/Crabomination
export CRAB_SEALED_SET=fra
MODE=${1:-train}
TRAIN=${TRAIN:-.ladder/r87/selfplay_train}
SEEDS=${SEEDS:-"43 97 151 199"}
ARMS=${ARMS:-"f274 r40 r18"}
R=.ladder/r87/train
mkdir -p $R
[ -x "$TRAIN" ] || { echo "missing $TRAIN" >&2; exit 1; }

if [ "$MODE" = queue ]; then
  free=0
  for _ in $(seq 1 720); do
    mem=$(nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits | head -1 | tr -d ' ')
    if [ "$mem" -lt 8192 ]; then free=$((free + 1)); else free=0; fi
    [ "$free" -ge 2 ] && break
    echo "$(date '+%F %T') GPU ${mem} MiB in use; free checks $free"
    sleep 60
  done
  [ "$free" -ge 2 ] || { echo "ABORT: the GPU never freed up" >&2; exit 1; }
  echo "=== queue: GPU free, four drivers $(date '+%F %T')"
  for s in $SEEDS; do SEEDS=$s "$0" train > $R/driver_s$s.txt 2>&1 & done
  wait
  echo "=== queue done $(date '+%F %T')"
  touch $R/train.done
  exit 0
fi

if [ "$MODE" = train ]; then
  for tseed in $SEEDS; do for arm in $ARMS; do
    out="nets_r87_${arm}_s${tseed}"
    if grep -q "^done: " "$out/log.txt" 2>/dev/null && [ -f "$out/best.safetensors" ]; then echo "$out already complete"; continue; fi
    case $arm in
      f274) spg=0.274; extra="--actor-fra-fixes" ;;
      r40) spg=0.40; extra="" ;;
      r18) spg=0.18; extra="" ;;
    esac
    rm -rf "$out"; mkdir -p "$out"
    echo "=== train $out start $(date)"
    $TRAIN --attn --lambda 0.7 --seed "$tseed" --games 250000 --steps 200000 \
        --actors 3 --window 500000 --lr 1e-4 --lr-cosine 60000 \
        --relabel-mode new --stop-after-stale 12 --steps-per-game $spg \
        --actor-rated-decks $extra --out "$out" > "$out/log.txt" 2>&1
    grep -q "learner device: cuda" "$out/log.txt" || { echo "ABORT: $out is not on cuda" >&2; exit 1; }
    echo "$out done: $(tail -1 "$out/stats.jsonl" | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print("games",d["games"],"steps/game %.3f" % (d["step"]/d["games"]),"auc",d["val_auc"],"elapsed_s",d["elapsed_s"])')"
  done; done
  exit 0
fi

# ---- gate (pre-registered 2026-10-28, before any training ran) ----
# FRA sealed, `net67-fixes` vs `fra-fixes`, 1 000 x 12, ladder seeds 601 / 701
# (fresh: no net was chosen on them), RATED (CRAB_SEALED_BUILDER=rated) and
# DEFAULT fields. Ladder binary .ladder/r87/bot_ladder for every cell.
# INSTRUMENT CHECK: each run's generation-phase steps/game within 3 % of its
# arm's target; otherwise that arm is not read.
# READINGS (t interval over the four training seeds, paired by seed):
#   Q1  F − R274: rated > +0.5 and default > −0.5 -> fixes actors help at an
#       equal ratio (round 83 was the ratio); else they do not.
#   Q2  R18 − R274 and R40 − R274, both fields, reported; a ratio whose
#       rated-field interval is wholly above 0 becomes the recipe's.
#   Q3  champion: the best of the 12 new nets by its rated-field mean on 601 /
#       701, CONFIRMED against nets/fra-champion.safetensors (round 86's
#       rated s43) on fresh seed 809 (scored, rated field, 1 000 x 12) and
#       under the client's search (`mcts-net67-fixes` vs `mcts-fixes-256`, 300
#       x 12, rated field, seeds 809 / 907). Both > +0.5 -> PROPOSE (the
#       user's call).
if [ "$MODE" = gate ]; then
  LADDER=${LADDER:-.ladder/r87/bot_ladder}
  run_cell() { # field arm control net games seed out
    local f=$7
    if [ -s "$f" ] && grep -q "A win%" "$f"; then return; fi
    local b=""; [ "$1" = rated ] && b=rated
    CRAB_SEALED_BUILDER=$b CRAB_NET="$4" $LADDER --a "$2" --b "$3" --decks sealed --games "$5" --seed "$6" --threads 22 > "$f" 2>&1
    echo "  $(basename "$f"): $(grep -E 'A win%' "$f" | tail -1 | tr -s ' ')"
  }
  net_of() { case $1 in r274) echo "nets_r86_rated_s$2/best.safetensors" ;; *) echo "nets_r87_$1_s$2/best.safetensors" ;; esac; }
  echo "=== gate start $(date)"
  for field in rated default; do for g in 601 701; do
    run_cell $field net67-fixes fra-fixes nets/fra-champion.safetensors 1000 "$g" "$R/${field}_champ_g${g}.txt"
    for t in 43 97 151 199; do for arm in r274 f274 r40 r18; do
      run_cell $field net67-fixes fra-fixes "$(net_of $arm $t)" 1000 "$g" "$R/${field}_${arm}_s${t}_g${g}.txt"
    done; done
  done; done
  python3 - <<'PY' | tee $R/summary_scored.txt
import re, json, statistics as st
R = ".ladder/r87/train"
def read(p): return float(re.findall(r"A win%\s+([0-9.]+)%", open(p).read())[-1])
def t_int(d):
    m = st.mean(d); h = 3.182 * st.stdev(d) / 2; return m, m - h, m + h
target = {"r274": 0.274, "f274": 0.274, "r40": 0.40, "r18": 0.18}
def net(arm, t): return f"nets_r86_rated_s{t}" if arm == "r274" else f"nets_r87_{arm}_s{t}"
seeds = (43, 97, 151, 199); G = (601, 701); ok = {}
print("instrument check (generation-phase steps/game):")
for arm in target:
    ok[arm] = True
    for t in seeds:
        rows = [json.loads(l) for l in open(f"{net(arm, t)}/stats.jsonl")]
        gen = [r for r in rows if r["games"] < rows[-1]["games"]] or rows
        spg = gen[-1]["step"] / gen[-1]["games"]
        good = abs(spg / target[arm] - 1) <= 0.03; ok[arm] &= good
        print(f"  {arm} s{t}: {spg:.3f} {'ok' if good else 'OFF'}")
best = None
for field in ("rated", "default"):
    champ = st.mean(read(f"{R}/{field}_champ_g{g}.txt") for g in G)
    m = {a: [st.mean(read(f"{R}/{field}_{a}_s{t}_g{g}.txt") for g in G) for t in seeds] for a in target}
    print(f"{field} field (mean of 601/701): champion {champ:.2f}")
    for a, v in m.items():
        print(f"  {a:5}: " + " ".join(f"s{t} {x:.2f}" for t, x in zip(seeds, v)) + f"  mean {st.mean(v):.2f}" + ("" if ok[a] else "  (PIN OFF)"))
    for a in ("f274", "r40", "r18"):
        d = [x - y for x, y in zip(m[a], m["r274"])]
        print(f"  {a} - r274 %+.2f [%+.2f, %+.2f]" % t_int(d))
    if field == "rated":
        for a in ("f274", "r40", "r18"):
            if not ok[a]: continue
            for t, x in zip(seeds, m[a]):
                if best is None or x > best[2]: best = (a, t, x)
print(f"best new net (rated field): {best}")
open(f"{R}/best.txt", "w").write(f"{best[0]} {best[1]}\n")
PY
  read arm bs < $R/best.txt
  net="nets_r87_${arm}_s${bs}/best.safetensors"
  echo "--- Q3: $net vs champion $(date)"
  run_cell rated net67-fixes fra-fixes nets/fra-champion.safetensors 1000 809 "$R/confirm_champ_g809.txt"
  run_cell rated net67-fixes fra-fixes "$net" 1000 809 "$R/confirm_best_g809.txt"
  for g in 809 907; do
    run_cell rated mcts-net67-fixes mcts-fixes-256 nets/fra-champion.safetensors 300 "$g" "$R/search_champ_g${g}.txt"
    run_cell rated mcts-net67-fixes mcts-fixes-256 "$net" 300 "$g" "$R/search_best_g${g}.txt"
  done
  python3 - <<'PY' | tee -a $R/summary_scored.txt
import re, statistics as st
R = ".ladder/r87/train"
def read(p): return float(re.findall(r"A win%\s+([0-9.]+)%", open(p).read())[-1])
sc = read(f"{R}/confirm_best_g809.txt") - read(f"{R}/confirm_champ_g809.txt")
se = st.mean(read(f"{R}/search_best_g{g}.txt") - read(f"{R}/search_champ_g{g}.txt") for g in (809, 907))
print(f"Q3 scored best - champion (809) {sc:+.2f}; search (809/907) {se:+.2f}")
print("Q3", "PROPOSE" if sc > 0.5 and se > 0.5 else "KEEP the champion")
PY
  touch $R/gate.done
  echo "=== gate done $(date)"
fi
