#!/usr/bin/env bash
# Round 64: three cells the round-62/63 findings point at, one build.
#
#  A. SEARCH DEPTH ON THE MATERIAL LEAF. Round 62: the net's leaf is worth
#     +0.25 inside the 64-iteration search and the search itself +2.5 over
#     the heuristic. Round 42 read 256 iterations vs 64 as +2.4, but on the
#     net leaf; a rollout with the material leaf has no net forward pass,
#     so depth is cheaper to buy now. `mcts-dflt-128` and `mcts-dflt-256`
#     vs `dflt`, seeds 43/97, 500 games (±0.95 — the question is a point
#     or more, not a tenth). `mcts-dflt` (64) vs `dflt` read 53.6 / 51.1
#     in round 62 and is the reference. Each cell's wall-clock is the
#     client's latency budget: record it.
#       readings: 256 clearly above 64's 52.35 -> the lobby runs 256
#                 material-leaf; within a point -> 64 stays and depth is
#                 closed like the r29 knobs; below -> a horizon-3 rollout
#                 budget saturates, park.
#  B. THE WIDE CHAIN'S COST. Its pair move fires at EVERY chain's first
#     step (`C(n, 2)` full-turn-cycle sims) and was built for the
#     empty-greedy overload. `attack_chain_lean` (profile `atk-lean`)
#     restricts the pair move to chains whose greedy declared nobody.
#     Gate for NO LOSS vs `dflt63` on four seeds, and time the mirrors
#     (`dflt63` vs `dflt63`, `atk-lean` vs `atk-lean`) uncontended.
#       readings: within ±0.3 and faster -> adopt; clearly below 50 -> the
#                 pairs on non-empty chains were doing work, keep wide.
#     READ at 51a29c3f8 (pre-rebase): 50.0 / 50.1 / 50.0 / 50.2, mirror
#     9.2 s -> 7.7 s. The same restriction had landed concurrently as
#     round 58's `attack_pairs_empty_only` (`pairs-empty`); the flag and
#     the profile were folded into it at the rebase, so stage B below is
#     the record and no longer runs.
#  C. THE COUNTERSPELL WINDOW. The last rule-shaped instant-speed picker
#     (`pick_stack_response`: a threat bar and the cheapest counter).
#     `counter_sim` (profile `counter-sim`) prices "let it resolve" against
#     each affordable counter by the main-phase outcome walk
#     (`evaluate_action_sequence`, combat-aware) and casts on a strict
#     improvement; a tie holds. Incidence ~0.1 casts a game (r63 census),
#     the rare class. Four seeds vs `dflt63`.
#       readings: as r63's removal leg (adopt on every cell clearing 50).
set -eu
cd /home/archuser/repos/Crabomination
LADDER=${LADDER:-./target/release-fast/bot_ladder}
GAMES=${GAMES:-1000}
GAMES_MCTS=${GAMES_MCTS:-500}
[ -x "$LADDER" ] || { echo "build it" >&2; exit 1; }

run () { # tag A B seed games
  local tag="$1" a="$2" b="$3" seed="$4" games="$5"
  if [ -s ".ladder/r64_${tag}.txt" ] && grep -q "A win%" ".ladder/r64_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r64_${tag}.txt" | tail -1 | tr -s ' ')  $(grep -E 'decided' ".ladder/r64_${tag}.txt" | tail -1)"
    return
  fi
  $LADDER --a "$a" --b "$b" --decks sealed --games "$games" --seed "$seed" > ".ladder/r64_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r64_${tag}.txt" | tail -1 | tr -s ' ')  $(grep -E 'decided' ".ladder/r64_${tag}.txt" | tail -1)"
}

STAGE=${STAGE:-all}
if [ "$STAGE" = all ] || [ "$STAGE" = quick ]; then
  # B ran at 51a29c3f8 as `atk-lean` (folded into round 58's `pairs-empty`
  # at the rebase; the profile is gone):
  #   for p in dflt63 atk-lean; do echo "  $p mirror: $($LADDER --a $p --b $p --decks sealed --games 1000 --seed 43 2>&1 | grep decided)"; done
  #   for s in 43 97 151 199; do run "lean_l$s" atk-lean dflt63 "$s" "$GAMES"; done
  echo "=== C. census: counter-sim vs dflt63 (50 games) ==="
  CRAB_ATTACK_CENSUS=1 $LADDER --a counter-sim --b dflt63 --decks sealed --games 50 --seed 43 2>&1 | grep -E "response_census" | tr ';' '\n' | grep -E "stack|counter"
  echo "=== C. counter-sim vs dflt63 ==="
  for s in 43 97 151 199; do run "counter_l$s" counter-sim dflt63 "$s" "$GAMES"; done
fi
if [ "$STAGE" = all ] || [ "$STAGE" = depth ]; then
  echo "=== A. search depth on the material leaf vs dflt ==="
  for it in 128 256; do for s in 43 97; do run "depth${it}_l$s" "mcts-dflt-$it" dflt "$s" "$GAMES_MCTS"; done; done
fi

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
def cells(prefix, seeds):
    v = []
    for l in seeds:
        p = pathlib.Path(f".ladder/r64_{prefix}_l{l}.txt")
        if not p.exists(): continue
        t = p.read_text()
        m = re.findall(r"A win%\s+([0-9.]+)%\s+\[([0-9.]+)%, ([0-9.]+)%\]", t)
        s = re.findall(r"in ([0-9.]+)s", t)
        if m: v.append((tuple(float(x) for x in m[-1]), float(s[-1]) if s else None))
    return v
for name, prefix, seeds in (("B atk-lean vs dflt63 (pre-rebase)", "lean", (43,97,151,199)),
                            ("C counter-sim vs dflt63", "counter", (43,97,151,199)),
                            ("A mcts-dflt-128 vs dflt", "depth128", (43,97)),
                            ("A mcts-dflt-256 vs dflt", "depth256", (43,97))):
    v = cells(prefix, seeds)
    if not v: continue
    pooled = st.mean(x[0][0] for x in v)
    print(f"  {name:26s} pooled {pooled:5.2f}   cells {[x[0][0] for x in v]}   lows {[x[0][1] for x in v]}   secs {[x[1] for x in v]}")
print("  reference: mcts-dflt (64) vs dflt read 53.6 / 51.1 = 52.35 (round 62)")
PY
echo "round-64 complete"
