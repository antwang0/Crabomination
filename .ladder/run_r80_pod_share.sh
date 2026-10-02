#!/usr/bin/env bash
# Round 80 (2026-10-02): the pod search's leaf as a SHARE of the table.
#
# The search's leaf (`MctsBot::reward`) squashes `eval_material` — the
# searching seat's material less the SUM of every hostile seat's — as
# `sigmoid(m/30)`. A duel starts at m = 0; a four-seat pod at m = -136
# (40 life + 7 cards a seat), reward 0.011, the squash's slope 1/24 of a
# duel's (six seats: 0.0001, ~1/2000). UCB1's exploration term (1.0) then
# dwarfs every gap between arms, and the mean is carried by the rare
# rollout that knocks out ANY seat (a whole seat leaves the sum: ~9x the
# reward, whoever it was). Code reading, 2026-10-02 — the first thing this
# round measures (CRAB_MCTS_TIMING's leaf mean / arm spread lines).
# The pod search read 0.87x its due share in a `dflt` field (92 games,
# 2026-09-26) and the lobby gives pods the heuristic.
#
# The fix, exact in a duel: `EvalWeights::pod_share_leaf` scores the leaf
# as the seat's share exp(m_me/T) / sum exp(m_s/T) over itself and the
# living hostile seats, each m its own material (T = 30: at two seats
# this IS the old squash, and the old path runs). `pod_share_eval` puts
# the same share into the heuristic's own evaluation (T·logit(share): its
# material less a soft maximum of the opponents', not their sum).
# `MctsConfig::pod_lap_horizon` makes the rollout horizon at least one lap
# (three turns stop short of the third opponent at four seats).
#
# FIELDS (fixed here, before any read): four seats, 12 fields, field j =
# precons 1+4j, 47+4j, 93+4j, 139+4j, seed 11000+j; six seats, 12 fields,
# field j = 1+j, 31+j, 61+j, 91+j, 121+j, 151+j, seed 12000+j. Every cell
# of a stage runs the same fields and seeds, so arms pair by deal group.
#
# STAGE scored (minutes): `podshare15` / `podshare30` / `podshare60` as the
#   hero in a `dflt` field, 2,000 games a four-seat field, 1,800 a six-seat
#   one. PRE-REGISTERED: a temperature is a CANDIDATE if it reads >= 1.02x
#   at both seat counts with the 95 % interval above 1.00x at both; no
#   adoption this round either way — a candidate gets the every-seat census
#   (pod length; `leader0` read 1.17x as one seat and was left off for
#   40-57 % longer pods) first.
#
# STAGE search (hours): `mcts-dflt-256` (control), `mcts-share-256`,
#   `mcts-sharelap-256` as the hero in a `dflt` field, four seats, the same
#   12 fields, GAMES4 = 384 games a field (96 deal groups, four a worker
#   at 24 threads; 4,608 games an arm). Cost, from a 48-game probe on a
#   field outside this list (decks 3/49/95/141, seed 99): 14.3 CPU-s a game
#   for the control, 16.9 for the share leaf — ~50 min an arm. The same
#   probe's diagnostics, read before this pre-registration: best - worst
#   arm mean 0.021 (control) vs 0.051 (share) a decision; leaf mean 0.091
#   vs 0.251; no rollout ended on fuel. PRE-REGISTERED, on the group-paired
#   difference in hero share (SE over the deal groups):
#     share - control >= +2.0 points, 95 % CI above 0
#                                  -> the saturation was binding; ADOPT
#                                     `pod_share_leaf: 30` on the default
#                                     (duel-exact, so no duel cell).
#     CI includes 0                -> the leaf's scale is not what limits
#                                     the pod search; record, close.
#     sharelap - share >= +1.5 points, 95 % CI above 0
#                                  -> ADOPT the lap horizon on top (its
#                                     cost per game recorded beside it).
#   The pod search as the lobby's pod pilot (the client's choice, not this
#   round's) needs >= 1.05x due with the 95 % interval above 1.00x AND a
#   latency read; this round reports the first.
#
# STAGE scored READ (2026-10-02, 19:57): every temperature is a CANDIDATE —
#   podshare15 1.057x [1.040, 1.073] / 1.028x [1.004, 1.052], podshare30
#   1.063x / 1.031x, podshare60 1.070x [1.054, 1.087] / 1.035x [1.011,
#   1.059] (four / six seats); the temperatures do not separate (paired
#   differences within ±0.4 points).
#
# STAGE census (written after that read, before any census cell): every
#   seat on `dflt`, `podshare15`, `podshare30`, `podshare60` (no --b), the
#   same fields and seeds, 2,000 / 1,800 games a four- / six-seat field.
#   PRE-REGISTERED: ADOPT `pod_share_eval` at the first of T = 60, 30, 15
#   (the scored read's order) whose every-seat pods run <= 15 % more turns
#   than `dflt`'s at both seat counts AND whose undecided share (draw +
#   action cap + board cap + no legal move) is no more than `dflt`'s + 0.5
#   points at both; none passing -> left off, recorded. (`pod_horizon` was
#   adopted at ~10 % longer pods; `leader0` / `leader25` left off at
#   40-57 % / 20-22 %.)
#
# STAGE census READ (2026-10-02, 21:48): every seat on podshare60 vs every
#   seat on dflt — turns -1.4 % (four seats) / +0.0 % (six), undecided
#   +0.00 / +0.01 points -> passes; `pod_share_eval: 60` ADOPTED on the
#   default. (podshare30 +0.6 % / +2.1 %; the podshare15 cells were not
#   needed by the rule.)
set -u
cd "$(dirname "$0")/.."
LADDER=${LADDER:-.ladder/r80/bot_ladder_r80}
R=.ladder/r80
THREADS=${THREADS:-24}
GAMES4=${GAMES4:-384}
[ -x "$LADDER" ] || { echo "no $LADDER"; exit 2; }

fields4() { for j in $(seq 0 11); do echo "$j $((1+4*j)),$((47+4*j)),$((93+4*j)),$((139+4*j)) $((11000+j))"; done; }
fields6() { for j in $(seq 0 11); do echo "$j $((1+j)),$((31+j)),$((61+j)),$((91+j)),$((121+j)),$((151+j)) $((12000+j))"; done; }

# census PILOT SEATS GAMES_PER_FIELD: every seat on PILOT.
census() {
  local a=$1 seats=$2 games=$3 d="$R/census_$1_s$2"
  mkdir -p "$d"
  "fields$seats" | while read -r j decks seed; do
    local f="$d/f$j.txt"
    if [ -s "$f" ] && grep -q "^exit: 0" "$f"; then continue; fi
    echo "== $f $(date -Is)"
    "$LADDER" --commander --a "$a" --pod-decks "$decks" --games "$games" --seed "$seed" \
      --threads "$THREADS" > "$f" 2>&1
    echo "exit: $?" >> "$f"
  done
}

# cell HERO FIELD_PILOT SEATS GAMES_PER_FIELD
cell() {
  local a=$1 b=$2 seats=$3 games=$4 d="$R/$1_vs_$2_s$3"
  mkdir -p "$d"
  "fields$seats" | while read -r j decks seed; do
    local f="$d/f$j.txt"
    if [ -s "$f" ] && grep -q "^exit: 0" "$f"; then continue; fi
    echo "== $f $(date -Is)"
    CRAB_POD_GROUPS=1 CRAB_MCTS_TIMING=1 "$LADDER" --commander --a "$a" --b "$b" \
      --pod-decks "$decks" --games "$games" --seed "$seed" --threads "$THREADS" > "$f" 2>&1
    echo "exit: $?" >> "$f"
  done
}

case ${STAGE:-scored} in
  scored)
    for t in 15 30 60; do
      cell "podshare$t" dflt 4 2000
      cell "podshare$t" dflt 6 1800
    done
    ;;
  search)
    for a in mcts-dflt-256 mcts-share-256 mcts-sharelap-256; do
      cell "$a" dflt 4 "$GAMES4"
    done
    ;;
  census)
    for a in dflt podshare60 podshare30 podshare15; do
      census "$a" 4 2000
      census "$a" 6 1800
    done
    python3 - "$R" <<'EOF2'
import glob, os, re, sys
root = sys.argv[1]
rows = {}
for d in sorted(glob.glob(os.path.join(root, "census_*_s*"))):
    name = os.path.basename(d)[len("census_"):]
    games = turns = undecided = 0
    for f in glob.glob(os.path.join(d, "f*.txt")):
        txt = open(f).read()
        if "exit: 0" not in txt:
            continue
        g = re.search(r"games (\d+) in .* undecided (\d+)", txt)
        t = re.search(r"turns/game ([\d.]+)", txt)
        games += int(g.group(1)); undecided += int(g.group(2)); turns += float(t.group(1)) * int(g.group(1))
    if games:
        rows[name] = (games, turns / games, 100 * undecided / games)
for name, (games, t, u) in rows.items():
    seats = name.rsplit("_s", 1)[1]
    base = rows.get(f"dflt_s{seats}")
    rel = f"{100 * (t / base[1] - 1):+5.1f} % turns, undecided {u - base[2]:+.2f} pts vs dflt" if base else ""
    print(f"census {name:<16} {games:>6} games  turns/game {t:6.2f}  undecided {u:5.2f} %  {rel}")
EOF2
    exit 0
    ;;
  *) echo "STAGE is scored, search or census"; exit 2 ;;
esac

# Summary: per cell, the hero's share pooled over every (field, group) and
# its x-due; per pair of cells on the same seats, the group-paired
# difference.
python3 - "$R" <<'EOF'
import glob, math, os, re, sys
root = sys.argv[1]
cells = {}
for d in sorted(glob.glob(os.path.join(root, "*_vs_*_s*"))):
    seats = int(d.rsplit("_s", 1)[1])
    groups, secs, games = {}, 0.0, 0
    for f in sorted(glob.glob(os.path.join(d, "f*.txt"))):
        j = int(re.search(r"f(\d+)\.txt$", f).group(1))
        txt = open(f).read()
        if "exit: 0" not in txt:
            continue
        for g, w, n in re.findall(r"^  group (\d+) (\d+)/(\d+)$", txt, re.M):
            groups[(j, int(g))] = (int(w), int(n))
        m = re.search(r"games (\d+) in ([\d.]+)s", txt)
        if m:
            games += int(m.group(1)); secs += float(m.group(2))
    if groups:
        cells[os.path.basename(d)] = (seats, groups, secs, games)

def stats(xs):
    k = len(xs); mean = sum(xs) / k
    var = sum((x - mean) ** 2 for x in xs) / (k - 1) if k > 1 else 0.0
    return mean, math.sqrt(var / k)

for name, (seats, groups, secs, games) in cells.items():
    share, se = stats([w / n for w, n in groups.values()])
    due = 1 / seats
    print(f"{name:<40} {len(groups):>4} groups {games:>6} games {secs:>7.0f}s wall  "
          f"share {100*share:5.2f} % ± {100*se:4.2f}  {share/due:.3f}x [{(share-1.96*se)/due:.3f}, {(share+1.96*se)/due:.3f}]")
names = list(cells)
for i, a in enumerate(names):
    for b in names[i + 1:]:
        if cells[a][0] != cells[b][0]:
            continue
        ga, gb = cells[a][1], cells[b][1]
        common = sorted(set(ga) & set(gb))
        if len(common) < 2:
            continue
        diff, se = stats([ga[k][0] / ga[k][1] - gb[k][0] / gb[k][1] for k in common])
        print(f"  {a} - {b}: {100*diff:+.2f} points ± {100*se:.2f} paired over {len(common)} groups, "
              f"95 % [{100*(diff-1.96*se):+.2f}, {100*(diff+1.96*se):+.2f}]")
EOF
