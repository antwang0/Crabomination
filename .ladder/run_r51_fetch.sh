#!/usr/bin/env bash
# Round 51: does the library-search decision pay when the search makes it?
#
# WHAT CHANGED, in two separable pieces.
#
# (A) `fetch_arms` — a MENU hole, the class that has paid twice.
#     `MctsBot::next_action` falls through to the heuristic bot whenever a
#     decision is pending, so `Decision::SearchLibrary` has NEVER reached
#     the search menu: every tutor and every fetchland in this program's
#     history was answered by `decide_library_search`'s fixed ranking. The
#     flag enumerates the legal hits as `SubmitDecision` arms (up to four,
#     scored a hair apart in the heuristic's order so a tie leaves its pick
#     in front) and searches them like any other declaration. This is the
#     same shape as chump blocks (r43, +0.9) and target arms (r46, +0.95):
#     the right fetch was not a low-scoring arm the search rejected, it was
#     absent.
#
# (B) demand-aware ranking — a VALUATION fix, which the same rounds say is
#     the weaker class, and it is included because it is the fallback (A)
#     leans on and because both readings are cheap.
#       * Basics were ranked by "fewest existing sources among the colours
#         it makes" — supply with no look at the hand. With a hand of red
#         spells and one stray green one it fetched the Forest. Its sibling
#         `ChooseColor` has counted pips in hand since it was written; the
#         ranking now uses the same signal, supply as tiebreaker.
#       * Non-basic hits were ranked by raw mana value: the biggest card in
#         the library whether or not we can ever cast it. Castability now
#         leads, mana value sorts within.
#
# A = the change, B = the control. A above 50 means the change wins.
#
# INCIDENCE, MEASURED FIRST (the round-50 discipline).
# `bot_probe --deck sos --games 1000` counts SearchLibrary at **254 of 4942
# decisions (5.1 %)**, i.e. ~0.25 per game. That is DENSER than the walker
# activations of round 50 (~0.18/game), which resolved at +-0.21 and +-0.12
# because near-mirror antithetic pairs contribute no variance. So this is
# expected to be measurable, and a hard 50.0 +-0.00 would be a surprise
# worth investigating rather than a shrug.
#
# PRE-REGISTERED READINGS.
#   * A clearly above 50.5   -> adopt; the menu-hole class pays a third time.
#   * A in 50.0-50.5 with intervals CLEARING 50 -> real and small; adopt on
#     replication across both seeds, exactly as round 50 did.
#   * exactly 50.0 +-0.00    -> the flag never fired. Given 0.25 fetches per
#     game that would contradict the probe and means the wiring is wrong
#     (most likely `fetch_search` never sees the decision), NOT that the
#     idea is dead. Check before believing.
#   * clearly BELOW 50       -> the search is worse than the heuristic at
#     this decision. That is a real possibility and the reason to gate: a
#     fetch's payoff is often many turns out, past the 3-turn horizon, so
#     the sims may be judging it on noise. This is the same failure mode
#     that killed the simulated mulligan in round 49 (47.45), and it is the
#     outcome most worth spending the cells on.
#
# Part B is the cheaper half and gates independently: if (B) wins and (A)
# does not, the demand-aware read is adopted alone and the search keeps its
# hands off the decision.
set -eu
cd /home/archuser/repos/Crabomination
LADDER=./target/release/bot_ladder
NET=nets_r41_v7_s151/best.safetensors   # same net as r42-r46: the control
GAMES=${GAMES:-1000}

[ -f "$NET" ] || { echo "missing $NET" >&2; exit 1; }
[ -x "$LADDER" ] || { echo "build it: cargo build --release -p crabomination --bin bot_ladder" >&2; exit 1; }

run () { # tag A B seed
  local tag="$1" a="$2" b="$3"
  if [ -s ".ladder/r51_${tag}.txt" ] && grep -q "A win%" ".ladder/r51_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r51_${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  CRAB_NET="$NET" $LADDER --a "$a" --b "$b" \
      --decks sealed --games "$GAMES" --seed "$4" \
      > ".ladder/r51_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r51_${tag}.txt" | tail -1 | tr -s ' ')"
}

echo "=== Part A: the fetch as searched arms (the cell of the round) ==="
for lseed in 43 97; do
  run "a_fetcharms_l${lseed}" mcts-net-fetcharms mcts-net-deep "$lseed"
done

echo "=== Part B: demand-aware ranking vs the supply-only read ==="
for lseed in 43 97; do
  run "b_demand_l${lseed}" gang legacyfetch "$lseed"
done

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
for label, tag in (("fetch arms      ", "a_fetcharms"), ("demand ranking  ", "b_demand")):
    v = []
    for l in (43, 97):
        p = pathlib.Path(f".ladder/r51_{tag}_l{l}.txt")
        if not p.exists():
            continue
        m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text())
        if m:
            v.append(float(m[-1]))
    if not v:
        continue
    pooled = st.mean(v)
    if all(abs(x - 50.0) < 0.05 for x in v):
        verdict = "flag never fired — contradicts the probe, check the wiring"
    elif pooled < 49.5:
        verdict = "NEGATIVE — the change is worse than what it replaces"
    elif pooled > 50.5:
        verdict = "clear win"
    else:
        verdict = "small; adopt only if BOTH cells' intervals clear 50 (r50 rule)"
    print(f"  {label} pooled {pooled:5.2f}%   cells {v}")
    print(f"    verdict: {verdict}")
PY
echo "round-51 complete"
