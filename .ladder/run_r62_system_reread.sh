#!/usr/bin/env bash
# Round 62: what does the net system add on top of the round-56 heuristic?
#
# WHY. Rounds 55–56 raised the heuristic default by ~8 points (attack
# chain +2.3, block chain +5.8, wide chain +0.5). Every recorded net-vs-
# heuristic level — the system reference 55.2 vs atk-sim / 53.65 vs gang
# (r48), the scored-pilot band — was read against controls that predate
# the chains, and round 57's side finding is that a freshly trained net
# piloting `net-bchain` (det1 + both chains) reads EXACTLY 50 against
# `dflt` on eight cells. So the standing premise of the ML program — search
# with the net's leaf beats the heuristic — has not been checked on the
# heuristic it now has to beat. A reference measurement, not a gate:
# nothing is adopted or rejected on it; it decides what the training
# program does next (head_leaf vs a new premise).
#
# ARMS (all vs `dflt`, the round-56 default, 1 000 games x 12 sealed
# decks, paired, seeds 43/97; `--games` lower on the slow cells):
#   A. `mcts-client`  — the lobby's exact pilot: 64-iteration search,
#                       horizon 3, on det1 + tail guard + both chains.
#                       THE question. Pre-registered readings:
#                         >= 52.5  the system still adds ~what it did
#                                  (r48: +3.65 over gang) — head_leaf and
#                                  iterations stay the ML levers;
#                         50.5–52.5 it adds less than before; the chains
#                                  ate part of what the search found;
#                         <= 50.5  the net system adds NOTHING on top of
#                                  the chained heuristic — the training
#                                  premise is void until the leaf head or
#                                  a new representation shows otherwise.
#   B. `mcts-net-deep` — the recorded system, unchanged (no chains):
#                       how stale is 55.2 / 53.65? Expected well below 50.
#   C. `net-bchain`     — the champion as a scored pilot with both chains:
#                       the r57 nets' shape, with the champion's weights.
#   D. `mcts-client` vs `gang` and vs `atk-sim`, one seed each: the new
#                       system numbers on the old controls.
#   F. `mcts-dflt` vs `dflt` — the same 64-iteration search on the
#                       default's weights with the MATERIAL leaf (no net).
#                       Added after A read +2.6: whatever A and F differ
#                       by is the net's leaf; if F reads what A reads, the
#                       net contributes nothing and the search alone is
#                       the margin.
#   E. `net-bchain` vs `gang` and vs `atk-sim`, one seed each: the r48
#                       part-B reference (the champion as the SCORED `net`
#                       pilot: 55.9 / 54.5 vs atk-sim, pooled 55.2, and
#                       53.65 vs gang) re-read with the chains under it,
#                       so that line can be replaced like for like.
#
# COST. A search-vs-heuristic cell is minutes to tens of minutes (the
# r48 mcts-vs-mcts cells were 40–50 min at 12 000 games); GAMES_MCTS
# defaults to 500 for the search arms (±0.85 a cell — enough to tell 50
# from 52.5) and 1 000 for the scored arm.
set -eu
cd /home/archuser/repos/Crabomination
LADDER=${LADDER:-./target/release-fast/bot_ladder}
NET=nets/champion.safetensors
GAMES_MCTS=${GAMES_MCTS:-500}
GAMES=${GAMES:-1000}
[ -x "$LADDER" ] || { echo "build it: cargo build --profile release-fast -p crabomination --bin bot_ladder" >&2; exit 1; }
[ -f "$NET" ] || { echo "missing $NET" >&2; exit 1; }

run () { # tag A B seed games
  local tag="$1" a="$2" b="$3" seed="$4" games="$5"
  if [ -s ".ladder/r62_${tag}.txt" ] && grep -q "A win%" ".ladder/r62_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r62_${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  CRAB_NET="$NET" $LADDER --a "$a" --b "$b" --decks sealed --games "$games" --seed "$seed" \
      > ".ladder/r62_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r62_${tag}.txt" | tail -1 | tr -s ' ')  $(grep -E 'decided' ".ladder/r62_${tag}.txt" | tail -1)"
}

echo "=== C. net-bchain (champion) vs dflt ==="
for s in 43 97; do run "scored_l$s" net-bchain dflt "$s" "$GAMES"; done
echo "=== A. mcts-client vs dflt ==="
for s in 43 97; do run "client_l$s" mcts-client dflt "$s" "$GAMES_MCTS"; done
echo "=== B. mcts-net-deep (no chains) vs dflt ==="
for s in 43 97; do run "stale_l$s" mcts-net-deep dflt "$s" "$GAMES_MCTS"; done
echo "=== D. mcts-client vs the old controls ==="
run "client_gang_l43" mcts-client gang 43 "$GAMES_MCTS"
run "client_atksim_l43" mcts-client atk-sim 43 "$GAMES_MCTS"
echo "=== E. net-bchain (champion, scored) vs the old controls ==="
run "scored_gang_l43" net-bchain gang 43 "$GAMES"
run "scored_atksim_l43" net-bchain atk-sim 43 "$GAMES"
echo "=== F. mcts-dflt (search, material leaf) vs dflt ==="
for s in 43 97; do run "matleaf_l$s" mcts-dflt dflt "$s" "$GAMES_MCTS"; done

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
def cell(tag):
    p = pathlib.Path(f".ladder/r62_{tag}.txt")
    if not p.exists(): return None
    m = re.findall(r"A win%\s+([0-9.]+)%\s+\[([0-9.]+)%, ([0-9.]+)%\]", p.read_text())
    return tuple(float(x) for x in m[-1]) if m else None
for name, tags in (("A mcts-client vs dflt", ["client_l43","client_l97"]),
                   ("B mcts-net-deep vs dflt", ["stale_l43","stale_l97"]),
                   ("C net-bchain vs dflt", ["scored_l43","scored_l97"]),
                   ("D mcts-client vs gang", ["client_gang_l43"]),
                   ("D mcts-client vs atk-sim", ["client_atksim_l43"]),
                   ("E net-bchain vs gang", ["scored_gang_l43"]),
                   ("E net-bchain vs atk-sim", ["scored_atksim_l43"]),
                   ("F mcts-dflt vs dflt", ["matleaf_l43","matleaf_l97"])):
    v = [c for c in (cell(t) for t in tags) if c]
    if not v: continue
    pooled = st.mean(x[0] for x in v)
    line = f"  {name:28s} pooled {pooled:5.2f}   cells {[x[0] for x in v]}   intervals {[(x[1], x[2]) for x in v]}"
    if name.startswith("A"):
        verdict = "system still adds ~r48's margin" if pooled >= 52.5 else ("adds less than before" if pooled > 50.5 else "adds NOTHING on top of the chained heuristic — premise void")
        line += f"\n       verdict: {verdict}"
    print(line)
PY
echo "round-62 complete"
