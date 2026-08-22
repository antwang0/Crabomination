#!/usr/bin/env bash
# Round 46: the targeting round — does letting the search pick targets
# pay, and does fixing auto-aim rehabilitate the ability class?
#
# WHY THIS ROUND. Four recorded games showed the bot bouncing its own
# creature with Proctor's Gaze; a fifth showed it stunning its own board
# with Homesickness. Both were the same defect in two places: the
# filtered auto-target walk had no side preference at all, and the
# friendly/hostile classifier answered per *effect* for a spell whose
# slots disagree. Both are fixed and pinned by regression tests
# (26e210f7c and its two parents). Neither fix is gated here, and that
# is a stated limitation rather than an oversight: the ladder A/Bs two
# profiles inside one process, so measuring a correctness fix in the
# default path would mean re-introducing the bug behind a flag. The
# justification for those two is the replay evidence and the tests. What
# IS gated here is everything that stayed behind a flag.
#
#   Part A — mcts-net-targetarms vs mcts-net-deep. THE cell of the
#     round. The candidate generators bake one auto-targeted assignment
#     into each cast arm, so a mis-aimed spell is the only arm of its
#     kind and the search cannot prefer the right target at any
#     valuation. This is the same structural shape as the chump-block
#     menu (r43, +0.9 adopted): a missing candidate, not a bad
#     valuation. Pre-registered expectation: SMALL POSITIVE, +0.5 to
#     +1.5. The class is real but rare per game, and the two extra arms
#     cost rollout budget on every *other* decision — round 42 says
#     iterations are the only lever that reliably pays, so this has to
#     clear the budget it spends. A null here means the class is too
#     rare to pay for its arms, NOT that the targeting was fine.
#   Part B — impulse vs gang. Ark of Hunger sat unactivated for five
#     turns while the bot topdecked with an empty hand, because the
#     ability generators are a whitelist of effect shapes and impulse
#     draw was on none of them. Expectation: small positive or null.
#     Sealed mirrors have to actually contain such a card for the flag
#     to do anything, so a zero-incidence 50.0 is a live outcome and
#     would mean the same thing it meant in r43 — measured nothing.
#   Part C — abilarms vs gang, RE-RUN. Round 43 read 48.0 / 50.5 and
#     was written up as "auto-aimed activations can harm". That cell
#     ran while auto-aiming pointed at the bot's own permanents, so it
#     was measuring ability enumeration PLUS a targeting bug and cannot
#     mean what it was taken to mean. This is the honest re-measurement
#     on a fixed targeter. Expectation: genuinely open — if the r43
#     negative was the targeting bug, this should come back to parity
#     or better; if it repeats, the class really is harmful and the
#     write-up stands on better evidence.
#
# MEASUREMENT. House rules: antithetic seat pairs, two ladder seeds per
# gated claim, paired intervals read from the harness. 1000 games per
# archetype x 12 sealed = 12 000 games per cell (+-0.57 paired), which
# resolves the effects above except a sub-half-point Part A.
set -eu
cd /home/archuser/repos/Crabomination
LADDER=./target/release/bot_ladder
NET=nets_r41_v7_s151/best.safetensors   # same net as r42-r45: the control
GAMES=${GAMES:-1000}

[ -f "$NET" ] || { echo "missing $NET" >&2; exit 1; }
[ -x "$LADDER" ] || { echo "build it: cargo build --release -p crabomination --bin bot_ladder" >&2; exit 1; }

run () { # name A B extra_env
  local tag="$1" a="$2" b="$3"
  if [ -s ".ladder/r46_${tag}.txt" ] && grep -q "A win%" ".ladder/r46_${tag}.txt"; then
    echo "  ${tag}: already done — $(grep -E 'A win%' ".ladder/r46_${tag}.txt" | tail -1 | tr -s ' ')"
    return
  fi
  CRAB_NET="$NET" $LADDER --a "$a" --b "$b" \
      --decks sealed --games "$GAMES" --seed "$4" \
      > ".ladder/r46_${tag}.txt" 2>&1
  echo "  ${tag}: $(grep -E 'A win%' ".ladder/r46_${tag}.txt" | tail -1 | tr -s ' ')"
}

echo "=== Part A: target arms on the search menu (the cell of the round) ==="
for lseed in 43 97; do
  run "a_targetarms_l${lseed}" mcts-net-targetarms mcts-net-deep "$lseed"
done

echo "=== Part B: impulse-draw activations (Ark of Hunger) ==="
for lseed in 43 97; do
  run "b_impulse_l${lseed}" impulse gang "$lseed"
done

echo "=== Part C: abilarms re-run on a fixed auto-targeter ==="
for lseed in 43 97; do
  run "c_abilarms_l${lseed}" abilarms gang "$lseed"
done

echo "=== summary ==="
python3 - <<'PY'
import re, pathlib, statistics as st
for label, tag in (
    ("A target arms", "a_targetarms"),
    ("B impulse", "b_impulse"),
    ("C abilarms", "c_abilarms"),
):
    v = []
    for l in (43, 97):
        p = pathlib.Path(f".ladder/r46_{tag}_l{l}.txt")
        if not p.exists():
            continue
        m = re.findall(r"A win%\s+([0-9.]+)%", p.read_text())
        if m:
            v.append(float(m[-1]))
    if v:
        flag = "  <-- ZERO INCIDENCE" if all(abs(x - 50.0) < 0.05 for x in v) else ""
        print(f"  {label:<15} pooled {st.mean(v):5.2f}%   cells {v}{flag}")
PY
echo "round-46 complete"
