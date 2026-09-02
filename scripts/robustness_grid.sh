#!/bin/bash
# The `-C debug-assertions=yes` robustness gate, as a grid.
#
# Every `debug_assert!`-backed invariant in the engine — the memo families on
# `CardMemo`, the `*_scan` presence-gate audits, the `computed_kw` subset
# guard — is compiled out of every release profile, so the suite is not their
# audit on real boards. PERF's (-58) is the case in point: 18,795 tests missed
# a board that sixty games of `cube` found in four seconds.
#
# Three legs. The **ladder** leg is `bot_ladder` over five pools x six seeds.
# The **actor** leg is `selfplay_train`, and it is not redundant: `bot_ladder`
# runs the encoder on no pool, so the four `debug_assert!`s in
# `server/encode.rs` — the vocab-index slot among them — have no other audit.
# The **pilots** leg (`--pilots`, on by default under `--wide`) runs the other
# ~45 decision policies against `gang`: the grid ran one of them for its whole
# history, and a `debug_assert!` on a path only `mcts` or `abilarms` reaches had
# no audit at all.
# Costs ~9 min of build into its own target dir (the flag re-fingerprints
# every crate), ~4 min for the 30-cell ladder grid, ~10 min for the actor
# leg's own build plus ~1 min for its three cells, and ~12 min for the pilots.
#
#   scripts/robustness_grid.sh              # build + verify + both legs
#   scripts/robustness_grid.sh --no-build   # both legs, reusing target-audit/
#   scripts/robustness_grid.sh --no-actor   # ladder leg only
#   scripts/robustness_grid.sh --actor-only # actor leg only (still builds it)
#   scripts/robustness_grid.sh --wide       # the sizes that actually find things
#   scripts/robustness_grid.sh --pilots     # the third leg: 40+ decision policies
#
# ⚠ **THE DEFAULT SIZES ARE A SMOKE TEST, NOT THE AUDIT, AND THAT IS MEASURED.**
# The hundred-and-nineteenth pass found **five** shipped bugs with this script
# and **the committed defaults are green on the tree that had all five** — 30
# cells, 0 failures, run against a `debug-assertions` binary built at the
# pre-fix commit. What each one needed:
#
#   1  a graveyard-only ability functioning on the battlefield   seed 2
#   2  `(life + clock - 1) / clock` overflow (the race check)    seeds 53, 73
#   3  `life_value`'s three multiplies overflow                  seeds 53, 73
#   4  a counter cost that kills its source, corpse left in play  --pilots
#   5  the CR 732.3 loop guard counting the stack it watches      --pilots
#
# 1-3 sit on seeds the default list does not carry (`1 3 7 11 23 97`) and 2-3
# need the Beacon of Immortality board, which takes hundreds of turns to build.
# **4 and 5 need a pilot other than `gang`**, which this grid had never run:
# `abilarms` on `--decks cube` was piling ~50,000 copies of one ability onto one
# stack, and fixing it took that cell from **846,610 ms to 304 ms**.
#
# **Run `--wide` before calling robustness green**, and read a default run as
# "nothing obvious moved".
#
# ⚠ `undecided_by draw` is a **rules outcome, not a stall** — two players
# losing at once is a legal game end. Only `cap` (turn limit) and `stuck` (no
# legal progress) are defects, so the totals below keep the three apart.
set -u
cd "$(dirname "$0")/.."

POOLS="${POOLS:-fixed cube sos sealed all}"
SEEDS="${SEEDS:-1 3 7 11 23 97}"
GAMES="${GAMES:-120}"
ACTOR_SEEDS="${ACTOR_SEEDS:-1 7 23}"
ACTOR_GAMES="${ACTOR_GAMES:-600}"
# The third leg. `bot_ladder`'s pilots are ~45 decision policies over one
# engine, and the grid ran exactly one of them (`gang`) for its whole history —
# so every `debug_assert!` on a path only another policy reaches had no audit.
# Each cell is `--a <pilot> --b gang`, which is also how the ladder gates them.
PILOTS="${PILOTS:-mcts mcts-heur det1 det3 baseline landseq mull chumpblocks
  dmgorder targeteval walkerchip abilarms impulse holdsick holdinst atk
  atk-cheap atk-hold atk-sim atk-race atk-life dflt-life life power base kw25
  buff2for1 convlands mullsim walkerlegacy legacyfetch landseq2 mull2 race2
  look1 look2 blk combat lookahead planner scaled keywords v2 pretap smarttap}"
# Twelve, not forty. `abilarms` puts every activated ability into the
# outcome-judged candidate list, so a board with an activation combo (Spike
# Feeder + Greater Good + Devoted Druid, seed 23's `cube` and `sos` pairs) costs
# it hundreds of probes a turn: the cell reads **1.1 s at 12 games, 1.5 s at 16,
# and then does not finish 20 in fifteen minutes or 40 in thirty** on the audit
# build — one game past 3,000 actions is the whole of the jump. That is the
# pilot's price, not a loop — the two loops that *were* on that board are fixed
# (see ENGINE_BACKLOG) and `--bench` is unmoved. Raise `PILOT_GAMES` only with
# the timeout below raised to match.
PILOT_GAMES="${PILOT_GAMES:-12}"
PILOT_SEED="${PILOT_SEED:-23}"
BIN=target-audit/overflow/bot_ladder
ACTOR_BIN=target-audit/overflow/selfplay_train

build=1 ladder=1 actor=1 pilots=0
for arg in "$@"; do
  case "$arg" in
    --no-build) build=0 ;;
    --no-actor) actor=0 ;;
    --actor-only) ladder=0 ;;
    --pilots) pilots=1 ;;
    # The sizes the three findings needed. 26 seeds x 400 games on the widest
    # pool is ~18 min of ladder; the actor leg goes to a filled `--window`.
    --wide)
      SEEDS="1 2 3 5 7 11 13 17 19 23 29 31 37 41 47 53 59 61 67 71 73 79 83 89 97 101"
      GAMES=400
      POOLS="all sealed"
      ACTOR_SEEDS="7 20260901"
      ACTOR_GAMES=30000
      pilots=1
      ;;
    *) echo "unknown flag $arg"; exit 2 ;;
  esac
done

# **Check the binary, not the flags.** `RUSTFLAGS` reaching the crate that
# carries the assertion is the thing that can silently not happen, and the
# proof is two seconds: the audit binary carries the messages and a
# `release-fast` one carries none.
check_asserts() {
  [ -x "$1" ] || { echo "no $1 — run without --no-build"; exit 1; }
  local n
  n=$(strings "$1" 2>/dev/null | grep -c "memo is stale")
  echo "assertion strings in $1: $n  (0 means the flag did not reach it — stop)"
  [ "$n" -gt 0 ] || exit 1
}

fail=0

if [ "$ladder" = 1 ]; then
  if [ "$build" = 1 ]; then
    RUSTFLAGS="-C debug-assertions=yes" CARGO_TARGET_DIR=target-audit \
      cargo build --profile overflow -p crabomination --bin bot_ladder || exit 1
  fi
  check_asserts "$BIN"

  cells=0 games=0 cap=0 stuck=0 draw=0
  for pool in $POOLS; do
    for seed in $SEEDS; do
      out=$(RUST_MIN_STACK=33554432 timeout 1800 "$BIN" \
        --a gang --b gang --games "$GAMES" --threads 3 --seed "$seed" --decks "$pool" 2>&1)
      rc=$?
      line=$(echo "$out" | grep -E "^[0-9]+ decided" | tail -1)
      # `undecided_by cap N / stuck N / draw N` — printed whenever the run has
      # any undecided game. Split here so a moving rate names its own cause
      # without a second run.
      by=$(echo "$out" | grep -E "^  undecided_by" | tail -1)
      if [ $rc -ne 0 ] || [ -z "$line" ]; then
        echo "FAIL pool=$pool seed=$seed rc=$rc"; echo "$out" | tail -20; fail=$((fail + 1))
      else
        echo "ok pool=$pool seed=$seed  $line ${by:+ |$by}"
        games=$((games + $(echo "$line" | awk '{print $1 + $3}')))
        if [ -n "$by" ]; then
          set -- $(echo "$by" | awk '{print $3, $6, $9}')
          cap=$((cap + $1)) stuck=$((stuck + $2)) draw=$((draw + $3))
        fi
      fi
      cells=$((cells + 1))
    done
  done
  # A capped or stuck game is a defect cell in its own right, counted HERE
  # so the ladder line carries it — a shared counter bumped after the echo
  # read as "failures=1" on the actor and pilots lines with every cell ok.
  [ $((cap + stuck)) -eq 0 ] || fail=$((fail + 1))
  echo "GRID DONE cells=$cells games=$games failures=$fail"
  echo "  undecided      cap $cap / stuck $stuck / draw $draw   (only cap+stuck are defects)"
fi

if [ "$actor" = 1 ]; then
  if [ "$build" = 1 ]; then
    RUSTFLAGS="-C debug-assertions=yes" CARGO_TARGET_DIR=target-audit \
      cargo build --profile overflow -p crabomination_ml --bin selfplay_train || exit 1
  fi
  check_asserts "$ACTOR_BIN"
  acells=0
  afail=0
  out_dir=$(mktemp -d)
  trap 'rm -rf "$out_dir"' EXIT
  for seed in $ACTOR_SEEDS; do
    out=$(RUST_MIN_STACK=33554432 timeout 3600 "$ACTOR_BIN" \
      --actors 3 --games "$ACTOR_GAMES" --steps 2 --seed "$seed" \
      --out "$out_dir/s$seed" 2>&1)
    rc=$?
    line=$(echo "$out" | grep -E "^actors: " | tail -1)
    if [ $rc -ne 0 ]; then
      echo "FAIL actor seed=$seed rc=$rc"; echo "$out" | tail -25; afail=$((afail + 1))
    else
      echo "ok actor seed=$seed  ${line:-ran}"
    fi
    acells=$((acells + 1))
  done
  echo "ACTOR DONE cells=$acells failures=$afail"
  fail=$((fail + afail))
fi

if [ "$pilots" = 1 ]; then
  check_asserts "$BIN"
  pcells=0
  pfail=0
  for pilot in $PILOTS; do
    out=$(RUST_MIN_STACK=33554432 timeout 1800 "$BIN" \
      --a "$pilot" --b gang --games "$PILOT_GAMES" --threads 3 \
      --seed "$PILOT_SEED" --decks all 2>&1)
    rc=$?
    line=$(echo "$out" | grep -E "^[0-9]+ decided" | tail -1)
    if [ $rc -ne 0 ] || [ -z "$line" ]; then
      echo "FAIL pilot=$pilot rc=$rc"; echo "$out" | tail -20; pfail=$((pfail + 1))
    else
      echo "ok pilot=$pilot  $line"
    fi
    pcells=$((pcells + 1))
  done
  echo "PILOTS DONE cells=$pcells failures=$pfail"
  fail=$((fail + pfail))
fi

[ "$fail" -eq 0 ]
