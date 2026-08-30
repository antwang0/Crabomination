#!/usr/bin/env bash
# Round 53: do outcome-judged mid-resolution targets pay?
#
# WHAT CHANGED. The suspending `Decision::ChooseTarget` path — trigger
# target picks (`drain_trigger_queue`) and the cast-slot / off-board
# picks in `actions.rs` — was answered by `decide_choose_target`, a
# polarity guess: hit the opponent's biggest permanent, else give up our
# cheapest, else the lowest-life opponent. Right for removal, backwards
# for every beneficial resolution effect whose legal set spans both
# sides (a "put two +1/+1 counters on target creature" trigger BUFFED
# THE OPPONENT'S BIGGEST CREATURE), blind to undersized removal (marks
# the 3/3 rather than killing the 2/2), and structurally unable to
# decline an optional "up to one" target. This is the round-46
# classifier gap (`target_arms`, +0.95) at the raise sites that flag
# never touched — target_arms varies cast slot 0 only.
#
# The `target_eval` flag (profile `targeteval`) settles the corner
# candidates — biggest/smallest permanent per side, each legal player,
# the decline when optional — through `settle_answer` (the machinery
# `decide_mode_by_outcome` already uses) and replaces the guess only on
# STRICT improvement. Real decisions only; sims keep the cheap guess
# (`eval_modes` gate, the same split as every other outcome policy).
#
# NOT COVERED, recorded so nobody reads the result as covering it: the
# seven inline `self.decider.decide` ChooseTarget sites in
# `effects/mod.rs` (council votes, copy retargeting, "change the
# target") never reach any policy — AutoDecider's first-legal for every
# seat. Fixing those needs per-site suspend plumbing; separate round.
#
# MEASUREMENT CLASS. Heuristic-level: no extra randomness consumed, and
# the strict-improvement rule keeps every position where the guess is
# already right (or the difference doesn't settle) bit-identical — the
# r50/r51 free-precision class, expected ~±0.1-0.2, not the ±0.6 search
# class.
#
# INCIDENCE (measured first, r50 discipline): see .ladder/r53_probe.txt.
#
# PRE-REGISTERED READINGS.
#   * clearly above 50.5            -> adopt; the outcome-eval class pays at
#                                      a second decision family.
#   * 50.0-50.5, intervals clear 50 -> real and small; adopt on replication
#                                      across both seeds (r50 rule).
#   * exactly 50.0 ±0.00            -> never fired; check the probe count
#                                      and the wants_ui path before
#                                      believing it.
#   * clearly below 50              -> the one-ply settled eval misprices
#                                      these effects (the r49 mull_sim
#                                      failure mode — payoffs past the
#                                      horizon). Park; do not iterate
#                                      weights.
set -eu
cd /home/archuser/repos/Crabomination
LADDER=./target/release/bot_ladder
GAMES=${GAMES:-1000}

[ -x "$LADDER" ] || { echo "build it: cargo build --release -p crabomination --bin bot_ladder" >&2; exit 1; }

run () { # tag A B seed
  local tag="$1" a="$2" b="$3"
  if [ -s ".ladder/r53_${tag}.txt" ] && grep -q "A win%" ".ladder/r53_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r53_${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  $LADDER --a "$a" --b "$b" \
      --decks sealed --games "$GAMES" --seed "$4" \
      > ".ladder/r53_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r53_${tag}.txt" | tail -1 | tr -s ' ')"
}

echo "=== targeteval vs gang (the shipped default) ==="
for lseed in 43 97; do
  run "targeteval_l${lseed}" targeteval gang "$lseed"
done

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
v = []
for l in (43, 97):
    p = pathlib.Path(f".ladder/r53_targeteval_l{l}.txt")
    if not p.exists():
        continue
    m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text())
    if m:
        v.append(float(m[-1]))
if v:
    pooled = st.mean(v)
    if all(abs(x - 50.0) < 0.05 for x in v):
        verdict = "flag never fired — check the probe count and the wants_ui path"
    elif pooled < 49.5:
        verdict = "NEGATIVE — the settled one-ply eval misprices these effects"
    elif pooled > 50.5:
        verdict = "clear win"
    else:
        verdict = "small; adopt only if BOTH cells' intervals clear 50 (r50 rule)"
    print(f"  targeteval pooled {pooled:5.2f}%   cells {v}")
    print(f"    verdict: {verdict}")
PY
echo "round-53 complete"
