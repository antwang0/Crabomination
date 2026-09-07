#!/usr/bin/env bash
# Round 69 (numbered 68 while it ran; the perf session landed its 68 first): the value net retrained on post-round-67 self-play (2026-09-07).
#
# QUESTION. Round 62 read the champion (a round-20 net) as worth +0.05 as a
# scored pilot and +0.25 as the search leaf on the chained heuristic, and
# round 57's four fresh nets piloted the new default to exactly 50. Every
# row those nets ever trained on came from actors whose auto-target picker
# aimed hostile player slots at the caster (round 67), and the actors never
# carried the engine seat flags at all (fixed in `play_recorded_game_mcts`
# this round). Is the null the data, or the idea?
#
# RECIPE. Round 23 / 57's, verbatim, on the round-67 default actors:
#   --attn --lambda 0.7 --games 250000 --steps 200000 --window 500000
#   --lr 1e-4 --lr-cosine 60000 --relabel-mode new --stop-after-stale 12
# `--actors 3` for round 23's generation rate (the regime match, r57);
# four training seeds as two concurrent drivers:
#   SEEDS="43 151" run_r68_retrain.sh train   and   SEEDS="97 199" ... train
# then `run_r68_retrain.sh gate`.
#
# GATES (sealed, paired, ladder seeds 43 / 97):
#   scored  — each fresh net pilots `net67` (= `dflt` + the net leaf; the
#             old `net-bchain` chains from a base four adoptions behind)
#             vs `dflt`, 1 000 games x 12 decks; the four ROUND-57 CONTROL
#             nets (old data) pilot the same `net67` for the reference, so
#             "fresh − old" is read within training seed on the same pilot.
#   search  — the best fresh net and the best old net as the leaf of
#             `mcts-net67-256` vs `mcts-dflt-256`, 500 games x 12 decks.
#
# PRE-REGISTERED READINGS (pooled over four training seeds, t intervals):
#   scored fresh − old  > +0.5   -> the data was the problem; the fresh net
#                                   becomes the `net-*` family's champion.
#                        within ±0.5 -> the null is the idea: a value leaf adds
#                                   nothing the sims don't, on clean data too.
#   search fresh − 50   > +0.5   -> the leaf pays inside the search on clean
#                                   data; a lobby decision (record, don't flip).
#   Holdout AUC recorded, not compared across regimes (r23 caveat).
set -eu
cd /home/archuser/repos/Crabomination
MODE=${1:-train}
TRAIN=${TRAIN:-./target/release/selfplay_train}
LADDER=${LADDER:-./target/release-fast/bot_ladder}
SEEDS=${SEEDS:-"43 97 151 199"}
GAMES=${GAMES:-1000}
ACTORS=${ACTORS:-3}
R=.ladder/r68
mkdir -p $R
[ -x "$TRAIN" ] || { echo "build it: cargo build --release -p crabomination_ml --features cuda --bin selfplay_train" >&2; exit 1; }
[ -x "$LADDER" ] || { echo "build it: cargo build --profile release-fast -p crabomination --bin bot_ladder" >&2; exit 1; }

if [ "$MODE" = train ]; then
  for tseed in $SEEDS; do
    out="nets_r68_s${tseed}"
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

# ---- gate ----
run_cell() { # arm control net out games threads seed
  local f=$7
  if [ -s "$f" ] && grep -q "A win%" "$f"; then return; fi
  CRAB_NET="$3" $LADDER --a "$1" --b "$2" --decks sealed --games "$4" --seed "$6" --threads "$5" > "$f" 2>&1
  echo "  $(basename "$f"): $(grep -E 'A win%' "$f" | tail -1 | tr -s ' ')"
}
echo "=== gate start $(date)"
for tseed in 43 97 151 199; do
  for g in 43 97; do
    run_cell net67 dflt "nets_r68_s${tseed}/best.safetensors" "$GAMES" 22 "$g" "$R/fresh_s${tseed}_g${g}.txt"
    run_cell net67 dflt "nets_r57_ctrl_s${tseed}/best.safetensors" "$GAMES" 22 "$g" "$R/old_s${tseed}_g${g}.txt"
  done
done
python3 - <<'PY' | tee $R/summary_scored.txt
import re, pathlib, statistics as st
def read(p):
    p = pathlib.Path(p)
    if not p.exists(): return None
    m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text()); return float(m[-1]) if m else None
rows=[]; best_fresh=None; best_old=None
for t in (43,97,151,199):
    f=[read(f".ladder/r68/fresh_s{t}_g{g}.txt") for g in (43,97)]
    o=[read(f".ladder/r68/old_s{t}_g{g}.txt") for g in (43,97)]
    if None in f or None in o: continue
    fm, om = st.mean(f), st.mean(o); rows.append((t,f,o,fm-om))
    print(f"  seed {t}: fresh {f} (mean {fm:.2f})  old {o} (mean {om:.2f})  fresh-old {fm-om:+.2f}")
    if best_fresh is None or fm>best_fresh[1]: best_fresh=(t,fm)
    if best_old is None or om>best_old[1]: best_old=(t,om)
if rows:
    d=[r[3] for r in rows]; m=st.mean(d); sd=st.stdev(d) if len(d)>1 else float('nan')
    half=2.776*sd/len(d)**0.5 if len(d)>1 else float('nan')   # t(0.975, df=3)
    fm=st.mean(r[1][0]+r[1][1] for r in rows)/2 if False else st.mean([x for r in rows for x in r[1]])
    print(f"  pooled fresh-old {m:+.2f} ±{half:.2f} (t, {len(d)} seeds); pooled fresh {fm:.2f}")
    print(f"  best fresh seed {best_fresh}; best old seed {best_old}")
    open(".ladder/r68/best_seeds.txt","w").write(f"{best_fresh[0]} {best_old[0]}\n")
PY
read BF BO < $R/best_seeds.txt
echo "--- search cells: fresh s$BF, old s$BO $(date)"
for g in 43 97; do
  run_cell mcts-net67-256 mcts-dflt-256 "nets_r68_s${BF}/best.safetensors" 500 22 "$g" "$R/search_fresh_s${BF}_g${g}.txt"
done
for g in 43 97; do
  run_cell mcts-net67-256 mcts-dflt-256 "nets_r57_ctrl_s${BO}/best.safetensors" 500 22 "$g" "$R/search_old_s${BO}_g${g}.txt"
done
echo "=== gate done $(date)"
