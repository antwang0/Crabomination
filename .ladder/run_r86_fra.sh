#!/usr/bin/env bash
# Round 86: the FRA value net with the learner's steps per game PINNED, and a
# gate without the champion's winner's curse (2026-10-27).
#
# Why. Round 84 found the ratio of learner steps to generated games was set
# by CPU contention, not the recipe: the learner is the bottleneck (it never
# slept), so a box shared with another job slowed the actors and handed each
# game more steps. r82's seeds read 0.274 / 0.274 / 0.176 / 0.173 and r84's
# 0.193 / 0.191 / 0.159 / 0.159, and strength tracked the ratio in both. The
# champion (r82 s97, 0.274) was also chosen as the best of four seeds on the
# same ladder seeds it was judged on, about +1 over its own seed mean.
#
# RECIPE. selfplay_train --steps-per-game 0.274 (new: actors and learner
# wait on each other, so the ratio is the recipe's), the champion's own
# ratio. Two arms on the same four training seeds, one thing different:
#   A (heur)  — round 82's recipe verbatim (heuristic_sealed_build decks)
#   B (rated) — round 84's: + --actor-rated-decks
# Both: CRAB_SEALED_SET=fra --attn --lambda 0.7 --games 250000 --steps
# 200000 --window 500000 --lr 1e-4 --lr-cosine 60000 --relabel-mode new
# --stop-after-stale 12 --actors 3; two concurrent drivers.
#
# ⚠ The learner-device line is a GATE: a log without "learner device: cuda"
#   aborts. Binary frozen at .ladder/r86/selfplay_train.
set -eu
cd /home/archuser/repos/Crabomination
export CRAB_SEALED_SET=fra
MODE=${1:-train}
TRAIN=${TRAIN:-.ladder/r86/selfplay_train}
SEEDS=${SEEDS:-"43 97 151 199"}
ARMS=${ARMS:-"heur rated"}
ACTORS=${ACTORS:-3}
SPG=0.274
R=.ladder/r86/train
mkdir -p $R
[ -x "$TRAIN" ] || { echo "missing $TRAIN" >&2; exit 1; }

if [ "$MODE" = queue ]; then
  # Wait until no other training job holds the GPU: under 8 GiB in use on
  # two checks a minute apart (memory, not utilization — the desktop renders
  # continuously). Gives up after twelve hours.
  free=0
  for _ in $(seq 1 720); do
    mem=$(nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits | head -1 | tr -d ' ')
    if [ "$mem" -lt 8192 ]; then free=$((free + 1)); else free=0; fi
    [ "$free" -ge 2 ] && break
    echo "$(date '+%F %T') GPU ${mem} MiB in use; free checks $free"
    sleep 60
  done
  [ "$free" -ge 2 ] || { echo "$(date '+%F %T') ABORT: the GPU never freed up" >&2; exit 1; }
  echo "=== queue: GPU free, starting both drivers $(date '+%F %T')"
  SEEDS="43 151" "$0" train > $R/driver_a.txt 2>&1 &
  SEEDS="97 199" "$0" train > $R/driver_b.txt 2>&1 &
  wait
  echo "=== queue done $(date '+%F %T')"
  cat $R/driver_a.txt $R/driver_b.txt
  touch $R/train.done
  exit 0
fi

if [ "$MODE" = train ]; then
  for tseed in $SEEDS; do for arm in $ARMS; do
    out="nets_r86_${arm}_s${tseed}"
    if grep -q "^done: 250000 games\|early stop" "$out/log.txt" 2>/dev/null && [ -f "$out/best.safetensors" ]; then
      echo "$out already complete"; continue
    fi
    extra=""; [ "$arm" = rated ] && extra="--actor-rated-decks"
    rm -rf "$out"; mkdir -p "$out"
    echo "=== train $out start $(date)"
    $TRAIN --attn --lambda 0.7 --seed "$tseed" --games 250000 --steps 200000 \
        --actors "$ACTORS" --window 500000 --lr 1e-4 --lr-cosine 60000 \
        --relabel-mode new --stop-after-stale 12 --steps-per-game $SPG $extra \
        --out "$out" > "$out/log.txt" 2>&1
    grep -q "learner device: cuda" "$out/log.txt" || { echo "ABORT: $out is not on cuda" >&2; head -3 "$out/log.txt" >&2; exit 1; }
    echo "$out done: $(tail -1 "$out/stats.jsonl" | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print("games",d["games"],"steps/game %.3f" % (d["step"]/d["games"]),"auc",d["val_auc"],"elapsed_s",d["elapsed_s"])')"
  done; done
  echo "=== train done $(date)"
  exit 0
fi

# ---- gate (pre-registered 2026-10-27, before any training ran) ----
# FRA sealed, pilot `net67-fixes` vs `fra-fixes` (the scored cell), 1 000
# games x 12 decks. Ladder seeds 211 / 307 — none of round 82's seeds, so the
# champion was not selected on them. Two fields: RATED (CRAB_SEALED_BUILDER=
# rated, the client opponent's decks) and DEFAULT.
#
# INSTRUMENT CHECK first: every run's final steps/game within 0.274 +- 3 %
# (the tail after generation adds a fixed ~6k steps, so measure the
# generation phase: steps at the last checkpoint before the actors finished
# — reported by the summary). Fails -> the pin is broken; nothing below is
# read.
#
# READINGS (t interval over the four training seeds, paired by seed):
#   Q1 rated decks: B − A on the rated field > +0.5 and default > −0.5
#        -> rated-deck training helps once the ratio is equal.
#   Q2 the pin: arm A's seed mean vs round 82's four-seed mean (same recipe,
#        unpinned; the r82 nets re-read on 211 / 307) — reported, no bar.
#   Q3 champion: the best of the 8 nets by its rated-field mean on 211 / 307
#        is CONFIRMED on fresh ladder seed 401 (rated field, scored 1 000 x
#        12) and by the search cell (`mcts-net67-fixes` vs `mcts-fixes-256`,
#        300 x 12, rated field, seeds 401 / 503), each against the champion
#        on the same cells. Both > +0.5 -> PROPOSE it as
#        nets/fra-champion.safetensors (the user's call; no swap without it).
if [ "$MODE" = gate ]; then
  LADDER=${LADDER:-.ladder/r86/bot_ladder}
  run_cell() { # field arm control net games seed out
    local f=$7
    if [ -s "$f" ] && grep -q "A win%" "$f"; then return; fi
    local b=""; [ "$1" = rated ] && b=rated
    CRAB_SEALED_BUILDER=$b CRAB_NET="$4" $LADDER --a "$2" --b "$3" --decks sealed --games "$5" --seed "$6" --threads 22 > "$f" 2>&1
    echo "  $(basename "$f"): $(grep -E 'A win%' "$f" | tail -1 | tr -s ' ')"
  }
  echo "=== gate start $(date)"
  for field in rated default; do for g in 211 307; do
    run_cell $field net67-fixes fra-fixes nets/fra-champion.safetensors 1000 "$g" "$R/${field}_champ_g${g}.txt"
    for t in 43 97 151 199; do
      run_cell $field net67-fixes fra-fixes "nets_r82_fra_s${t}/best.safetensors" 1000 "$g" "$R/${field}_r82s${t}_g${g}.txt"
      for arm in heur rated; do
        run_cell $field net67-fixes fra-fixes "nets_r86_${arm}_s${t}/best.safetensors" 1000 "$g" "$R/${field}_${arm}_s${t}_g${g}.txt"
      done
    done
  done; done
  python3 - <<'PY' | tee $R/summary_scored.txt
import re, json, pathlib, statistics as st
R = ".ladder/r86/train"
def read(p):
    m = re.findall(r"A win%\s+([0-9.]+)%", pathlib.Path(p).read_text()); return float(m[-1])
def t_int(d):
    m = st.mean(d); h = 3.182 * st.stdev(d) / 2; return m, m - h, m + h
ok = True
print("instrument check (generation-phase steps/game, target 0.274 +- 3 %):")
for arm in ("heur", "rated"):
    for t in (43, 97, 151, 199):
        rows = [json.loads(l) for l in open(f"nets_r86_{arm}_s{t}/stats.jsonl")]
        final_games = rows[-1]["games"]
        gen = [r for r in rows if r["games"] < final_games] or rows
        r = gen[-1]; spg = r["step"] / r["games"]
        good = abs(spg / 0.274 - 1) <= 0.03
        ok &= good
        print(f"  {arm} s{t}: {spg:.3f} at {r['games']} games {'ok' if good else 'OFF'}")
print("PIN", "OK" if ok else "BROKEN")
seeds = (43, 97, 151, 199); G = (211, 307)
best = None
for field in ("rated", "default"):
    champ = st.mean(read(f"{R}/{field}_champ_g{g}.txt") for g in G)
    m = {k: [st.mean(read(f"{R}/{field}_{k}_s{t}_g{g}.txt") for g in G) for t in seeds] for k in ("heur", "rated")}
    m["r82"] = [st.mean(read(f"{R}/{field}_r82s{t}_g{g}.txt") for g in G) for t in seeds]
    print(f"{field} field (mean of ladder seeds 211/307): champion {champ:.2f}")
    for k, v in m.items():
        print(f"  {k:5}: " + " ".join(f"s{t} {x:.2f}" for t, x in zip(seeds, v)) + f"  mean {st.mean(v):.2f}")
    d = [b - a for a, b in zip(m["heur"], m["rated"])]
    print("  Q1 rated - heur %+.2f [%+.2f, %+.2f]" % t_int(d))
    d = [a - b for a, b in zip(m["heur"], m["r82"])]
    print("  Q2 heur(pinned) - r82(unpinned) %+.2f [%+.2f, %+.2f]" % t_int(d))
    open(f"{R}/q1_{field}.txt", "w").write(f"{st.mean(b - a for a, b in zip(m['heur'], m['rated']))}\n")
    if field == "rated":
        for k in ("heur", "rated"):
            for t, x in zip(seeds, m[k]):
                if best is None or x > best[2]: best = (k, t, x)
print(f"best net (rated field): {best}")
open(f"{R}/best.txt", "w").write(f"{best[0]} {best[1]}\n")
PY
  read arm bs < $R/best.txt
  net="nets_r86_${arm}_s${bs}/best.safetensors"
  echo "--- Q3 confirmation: $net vs champion $(date)"
  run_cell rated net67-fixes fra-fixes nets/fra-champion.safetensors 1000 401 "$R/confirm_champ_g401.txt"
  run_cell rated net67-fixes fra-fixes "$net" 1000 401 "$R/confirm_best_g401.txt"
  for g in 401 503; do
    run_cell rated mcts-net67-fixes mcts-fixes-256 nets/fra-champion.safetensors 300 "$g" "$R/search_champ_g${g}.txt"
    run_cell rated mcts-net67-fixes mcts-fixes-256 "$net" 300 "$g" "$R/search_best_g${g}.txt"
  done
  python3 - <<'PY' | tee -a $R/summary_scored.txt
import re, pathlib, statistics as st
R = ".ladder/r86/train"
def read(p):
    m = re.findall(r"A win%\s+([0-9.]+)%", pathlib.Path(p).read_text()); return float(m[-1])
sc = read(f"{R}/confirm_best_g401.txt") - read(f"{R}/confirm_champ_g401.txt")
se = st.mean(read(f"{R}/search_best_g{g}.txt") - read(f"{R}/search_champ_g{g}.txt") for g in (401, 503))
print(f"Q3 scored best - champion (seed 401) {sc:+.2f}; search best - champion (401/503) {se:+.2f}")
print("Q3", "PROPOSE" if sc > 0.5 and se > 0.5 else "KEEP the champion")
PY
  touch $R/gate.done
  echo "=== gate done $(date)"
fi
