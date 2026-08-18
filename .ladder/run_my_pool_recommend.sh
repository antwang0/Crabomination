#!/usr/bin/env bash
# Recommend the best build from decks/my_sealed_pool.txt (full pipeline:
# race -> refine -> attribution-guided local search), then race the
# recommendation head-to-head against decks/real_build_converge.txt with
# deck_duel (antithetic pairs).
#
# NOTE, established before launch: 14 of the current deck's 27 non-basic
# cards (both Forum of Amity included) are NOT in my_sealed_pool.txt, so
# the deck cannot have been built from this pool file. The recommendation
# is therefore "the best deck this pool supports", not a swap list for
# the current deck; the duel tells whether the pool's best beats the
# current deck anyway.
#
# Waits for the other session's round-42 ladder cells first: two
# consecutive clear checks three minutes apart, so the between-cell gap
# in the r42 runner does not read as "done".
set -eu
cd /home/archuser/repos/Crabomination

while :; do
  if ! pgrep -f "bot_ladder|run_r42" >/dev/null 2>&1; then
    sleep 180
    pgrep -f "bot_ladder|run_r42" >/dev/null 2>&1 || break
  fi
  sleep 120
done

cargo build --release -p crabomination --bin recommend_pool --bin deck_duel

# pool seed games cap pins refine_top variants racing search_gens gauntlet
./target/release/recommend_pool decks/my_sealed_pool.txt 43 20 8 '' 3 8 3 4 20 \
    > .ladder/my_pool_recommend.txt 2>&1

# Extract the "recommended build" tail into a decklist. Basics print as
# "N White basic" etc. and must be renamed to real basics for
# parse_decklist / CRAB_SEALED_DECK to read the file back.
python3 - <<'PY'
import re
lines = open(".ladder/my_pool_recommend.txt").read().splitlines()
start = next(i for i, l in enumerate(lines) if l.startswith("recommended build"))
basic = {"White": "Plains", "Blue": "Island", "Black": "Swamp", "Red": "Mountain", "Green": "Forest"}
out = []
for l in lines[start + 1:]:
    m = re.match(r"\s+(\d+) (.+)$", l)
    if not m:
        break
    n, name = m.group(1), m.group(2)
    b = re.match(r"(White|Blue|Black|Red|Green) basic$", name)
    out.append(f"{n} {basic[b.group(1)] if b else name}")
open("decks/recommended_from_pool.txt", "w").write("\n".join(out) + "\n")
print("wrote decks/recommended_from_pool.txt:", sum(int(x.split()[0]) for x in out), "cards")
PY

./target/release/deck_duel decks/recommended_from_pool.txt decks/real_build_converge.txt 2000 7 \
    > .ladder/my_pool_duel.txt 2>&1
echo "=== duel: recommended (A) vs current deck (B) ==="
tail -25 .ladder/my_pool_duel.txt
echo "pool recommendation complete"
