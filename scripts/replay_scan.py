#!/usr/bin/env python3
"""Per-game and per-card report over a directory of match replays.

    replay_scan.py REPLAY_DIR [--deck-prefix NAME] [--top N] [--games-out FILE]

Reads the `replay-*.jsonl` files `CRAB_REPLAY_DIR` records (v2: a header
with the players, `{"e": [...], "n": {id: name}}` event lines, an end
footer). The "deck" seat is the player whose name starts with
`--deck-prefix`; without it, the seat whose name does NOT start with
"Sealed #" (the generated opponents' label).

Reports: win rate by seat order, mana development (lands after own turns
3/4/5, screw/flood shares), spells cast per turn, a per-card table for the
deck's seat (drawn-in-game win rate, casts, cast rate, mean cast turn),
converge X per payoff cast against the colours paid, Slumbering Trudge X,
and the round-65 attack-outcome classifier (unblocked / trade / won /
suicide / bounced) for both seats with the worst suicides listed.
"""
import collections
import glob
import json
import os
import re
import sys

CONVERGE = {"Rancorous Archaic", "Sundering Archaic", "Together as One", "Snarl Song", "Arcane Omens"}
MANA_EVENTS = {"ManaAdded", "ColorlessManaAdded", "TappedForMana", "PermanentTapped", "AbilityActivated"}


def ev_kind(ev):
    if isinstance(ev, str):
        return ev, None
    (k, v), = ev.items()
    return k, v


def scan(path, deck_prefix):
    names = {}
    players = None
    deck_seat = None
    g = {
        "path": path, "winner": None, "turns": 0, "life": [20, 20],
        "lands_by_own_turn": [collections.OrderedDict(), collections.OrderedDict()],
        "own_turns": [0, 0],
        "spells": [[], []],          # (name, own_turn, colors_paid)
        "drawn": [collections.Counter(), collections.Counter()],
        "played_lands": [collections.Counter(), collections.Counter()],
        "converge": [],              # (name, x, colors_paid, own_turn)
        "trudge": [],                # (x_estimate, own_turn)
        "attacks": [],               # (seat, turn, attacker, [blockers], attacker_died, blockers_died)
    }
    active = None
    turn = 0
    lands = [0, 0]
    mana_colors = [set(), set()]
    pending = []  # (seat, name, card_id, own_turn, colors) — recent casts awaiting their ETB/effect
    attackers, blocks, deaths, in_combat = [], collections.defaultdict(list), set(), False
    with open(path) as f:
        for line in f:
            try:
                d = json.loads(line)
            except Exception:
                continue
            if "replay" in d:
                players = d.get("players", [])
                if deck_prefix:
                    deck_seat = next((i for i, p in enumerate(players) if p.startswith(deck_prefix)), None)
                else:
                    deck_seat = next((i for i, p in enumerate(players) if not p.startswith("Sealed #")), None)
                continue
            if "end" in d:
                continue
            if "n" in d:
                names.update({int(k): v for k, v in d["n"].items()})
            for ev in d.get("e", []):
                k, v = ev_kind(ev)
                if k == "TurnStarted":
                    active, turn = v["player"], v["turn"]
                    g["turns"] = turn
                    g["own_turns"][active] += 1
                    mana_colors = [set(), set()]
                elif k == "StepChanged":
                    if v == "DeclareAttackers":
                        attackers, blocks, deaths, in_combat = [], collections.defaultdict(list), set(), True
                    elif v in ("EndCombat", "PostCombatMain", "End") and in_combat:
                        in_combat = False
                        for a in attackers:
                            bl = blocks.get(a, [])
                            g["attacks"].append((active, turn, names.get(a, f"#{a}"),
                                                 [names.get(b, f"#{b}") for b in bl],
                                                 a in deaths, sum(1 for b in bl if b in deaths)))
                    if v in ("PostCombatMain", "End", "PrecombatMain"):
                        # A main phase ends the "mana since the last cast" window.
                        pass
                elif k == "LandPlayed":
                    p = v["player"]
                    lands[p] += 1
                    g["played_lands"][p][names.get(v["card_id"], "?")] += 1
                    g["lands_by_own_turn"][p][g["own_turns"][p]] = lands[p]
                elif k == "CardDrawn":
                    g["drawn"][v["player"]][names.get(v["card_id"], "?")] += 1
                elif k == "ManaAdded":
                    mana_colors[v["player"]].add(v.get("color"))
                elif k == "SpellCast":
                    p = v["player"]
                    name = names.get(v["card_id"], "?")
                    colors = len({c for c in mana_colors[p] if c})
                    ot = g["own_turns"][p] if p == active else g["own_turns"][p] + 0.5
                    g["spells"][p].append((name, ot, colors))
                    pending.append((p, name, v["card_id"], ot, colors))
                    pending = pending[-6:]
                    mana_colors[p] = set()
                elif k == "PermanentEntered":
                    cid = v["card_id"]
                    for i, item in enumerate(pending):
                        if item[2] == cid and len(item) == 5:
                            pending[i] = item + ("entered",)
                elif k == "CounterAdded":
                    # The engine reports enters-with counters BEFORE PermanentEntered.
                    cid = v["card_id"]
                    for i, item in enumerate(pending):
                        if item[2] == cid:
                            p, name, pcid, ot, colors = item[:5]
                            if name == "Rancorous Archaic" and v.get("counter_type") == "PlusOnePlusOne":
                                g["converge"].append((name, v.get("count", 0), colors, ot, p))
                                pending.pop(i)
                            elif name == "Slumbering Trudge" and v.get("counter_type") == "Stun":
                                g["trudge"].append((3 - v.get("count", 0), ot, p))
                                pending.pop(i)
                            break
                elif k == "LifeGained":
                    p = v["player"]
                    g["life"][p] += v["amount"]
                    for i, item in enumerate(pending):
                        if item[0] == p and item[1] in ("Together as One", "Snarl Song"):
                            g["converge"].append((item[1], v["amount"], item[4], item[3], p))
                            pending.pop(i)
                            break
                elif k == "PermanentExiled":
                    for i, item in enumerate(pending):
                        if item[1] == "Sundering Archaic":
                            g["converge"].append((item[1], names.get(v["card_id"], "?"), item[4], item[3], item[0]))
                            pending.pop(i)
                            break
                elif k == "CardDiscarded":
                    for i, item in enumerate(pending):
                        if item[1] == "Arcane Omens" and item[0] != v["player"]:
                            g["converge"].append((item[1], 1, item[4], item[3], item[0]))
                            break
                elif k == "LifeLost" or k == "PaidLife":
                    g["life"][v["player"]] -= v["amount"]
                elif k == "DamageDealt" and v.get("to_player") is not None:
                    g["life"][v["to_player"]] -= v["amount"]
                elif k == "AttackerDeclared" and in_combat:
                    attackers.append(v)
                elif k == "BlockerDeclared" and in_combat:
                    blocks[v["attacker"]].append(v["blocker"])
                elif k in ("CreatureDied", "PermanentDied") and in_combat:
                    deaths.add(v["card_id"])
                elif k == "GameOver":
                    g["winner"] = v.get("winner")
    g["players"] = players or []
    g["deck_seat"] = deck_seat
    return g


def classify(rec):
    _, _, _, bl, died, bd = rec
    if not bl:
        return "unblocked"
    if died and bd == 0:
        return "suicide"
    if died and bd > 0:
        return "trade"
    if bd > 0:
        return "won"
    return "bounced"


def pct(a, b):
    return 100.0 * a / b if b else float("nan")


def main():
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        sys.exit(2)
    d = args[0]
    prefix = None
    top = 60
    games_out = None
    i = 1
    while i < len(args):
        if args[i] == "--deck-prefix":
            prefix = args[i + 1]; i += 2
        elif args[i] == "--top":
            top = int(args[i + 1]); i += 2
        elif args[i] == "--games-out":
            games_out = args[i + 1]; i += 2
        else:
            i += 1
    files = sorted(glob.glob(os.path.join(d, "replay-*.jsonl"))) if os.path.isdir(d) else sorted(glob.glob(d))
    games = []
    for p in files:
        try:
            g = scan(p, prefix)
        except Exception as e:  # a truncated file from an aborted run
            print(f"skip {p}: {e}", file=sys.stderr)
            continue
        if g["deck_seat"] is None or g["winner"] is None:
            continue
        games.append(g)
    n = len(games)
    print(f"{n} games from {len(files)} replay files under {d}")
    if not n:
        return
    ds = lambda g: g["deck_seat"]
    wins = sum(1 for g in games if g["winner"] == ds(g))
    on_play = [g for g in games if ds(g) == 0]
    on_draw = [g for g in games if ds(g) == 1]
    print(f"deck wins {wins}/{n} = {pct(wins, n):.1f}%  | on the play {pct(sum(1 for g in on_play if g['winner']==0), len(on_play)):.1f}% (n={len(on_play)})"
          f"  on the draw {pct(sum(1 for g in on_draw if g['winner']==1), len(on_draw)):.1f}% (n={len(on_draw)})")
    tw = [g["turns"] for g in games if g["winner"] == ds(g)]
    tl = [g["turns"] for g in games if g["winner"] != ds(g)]
    print(f"mean game length: deck wins {sum(tw)/max(1,len(tw)):.1f} turns, deck losses {sum(tl)/max(1,len(tl)):.1f} turns")

    # ── mana ──
    def lands_after(g, seat, t):
        m = g["lands_by_own_turn"][seat]
        best = 0
        for ot, l in m.items():
            if ot <= t:
                best = l
        return best
    print("\nmana (lands on battlefield after own turn N; games that reached it):")
    for label, seat_of in (("deck", ds), ("opp ", lambda g: 1 - ds(g))):
        row = []
        for t in (3, 4, 5, 6):
            gs = [g for g in games if g["own_turns"][seat_of(g)] >= t]
            if not gs:
                row.append(f"T{t}: n/a")
                continue
            vals = [lands_after(g, seat_of(g), t) for g in gs]
            short = sum(1 for v in vals if v < t)
            row.append(f"T{t}: mean {sum(vals)/len(vals):.2f}, <{t} lands {pct(short, len(gs)):.0f}%")
        print(f"  {label}: " + " | ".join(row))
    gs8 = [g for g in games if g["own_turns"][ds(g)] >= 8]
    flood = sum(1 for g in gs8 if lands_after(g, ds(g), 8) >= 8)
    print(f"  deck flood proxy: >=8 lands after own T8 in {pct(flood, len(gs8)):.0f}% of {len(gs8)} games reaching T8")
    screw_losses = [g for g in games if g["winner"] != ds(g) and g["own_turns"][ds(g)] >= 4 and lands_after(g, ds(g), 4) < 3]
    print(f"  deck losses with <3 lands after own T4: {len(screw_losses)} of {n - wins} losses")

    # ── tempo ──
    print("\nspells cast per own turn (mean; deck / opp):")
    for t in range(1, 8):
        a = [sum(1 for (_, ot, _) in g["spells"][ds(g)] if int(ot) == t) for g in games if g["own_turns"][ds(g)] >= t]
        b = [sum(1 for (_, ot, _) in g["spells"][1 - ds(g)] if int(ot) == t) for g in games if g["own_turns"][1 - ds(g)] >= t]
        print(f"  T{t}: {sum(a)/max(1,len(a)):.2f} / {sum(b)/max(1,len(b)):.2f}")

    # ── per-card ──
    print(f"\nper-card (deck seat): seen = drawn or played; WR when seen vs when not; casts / seen; mean cast turn")
    seen_games = collections.defaultdict(set)
    seen_count = collections.Counter()
    casts = collections.Counter()
    cast_turns = collections.defaultdict(list)
    for gi, g in enumerate(games):
        s = ds(g)
        for name, c in g["drawn"][s].items():
            seen_games[name].add(gi); seen_count[name] += c
        for name, c in g["played_lands"][s].items():
            if gi not in seen_games[name]:
                seen_games[name].add(gi); seen_count[name] += c
        for name, ot, _ in g["spells"][s]:
            casts[name] += 1; cast_turns[name].append(ot)
            if gi not in seen_games[name]:
                seen_games[name].add(gi); seen_count[name] += 1
    rows = []
    for name, gset in seen_games.items():
        w_in = sum(1 for gi in gset if games[gi]["winner"] == ds(games[gi]))
        out = [gi for gi in range(n) if gi not in gset]
        w_out = sum(1 for gi in out if games[gi]["winner"] == ds(games[gi]))
        rows.append((pct(w_in, len(gset)), name, len(gset), pct(w_out, len(out)), casts[name], seen_count[name],
                     sum(cast_turns[name]) / len(cast_turns[name]) if cast_turns[name] else float("nan")))
    rows.sort(reverse=True)
    print(f"  {'WR seen':>7} {'n':>4} {'WR not':>7} {'casts':>5} {'cast%':>6} {'castT':>6}  card")
    for wr, name, cnt, wro, c, sc, ct in rows[:top]:
        print(f"  {wr:6.1f}% {cnt:4d} {wro:6.1f}% {c:5d} {pct(c, sc):5.0f}% {ct:6.1f}  {name}")

    # ── converge ──
    conv = [c for g in games for c in g["converge"] if c[4] == ds(g)]
    if conv:
        print("\nconverge payoffs cast by the deck seat (X = counters / life / discards; Sundering = exiled target):")
        by = collections.defaultdict(list)
        for name, x, colors, ot, _ in conv:
            by[name].append((x, colors, ot))
        for name, lst in by.items():
            xs = [x for x, _, _ in lst if isinstance(x, int)]
            cols = [c for _, c, _ in lst]
            ots = [ot for _, _, ot in lst]
            xd = collections.Counter(xs)
            extra = f"X dist {dict(sorted(xd.items()))}" if xs else "targets " + str(collections.Counter(x for x, _, _ in lst).most_common(6))
            print(f"  {name}: {len(lst)} casts, mean colours paid {sum(cols)/len(cols):.2f}, mean cast turn {sum(ots)/len(ots):.1f}; {extra}")
        under = [(name, x, colors) for name, x, colors, _, _ in conv if isinstance(x, int) and x < colors]
        if under:
            print(f"  ! {len(under)} casts where X < colours paid (bookkeeping mismatch or a payment bug): {under[:8]}")
    tr = [t for g in games for t in g["trudge"] if t[2] == ds(g)]
    if tr:
        print(f"\nSlumbering Trudge: {len(tr)} casts; X distribution {dict(sorted(collections.Counter(x for x, _, _ in tr).items()))}; mean cast turn {sum(ot for _, ot, _ in tr)/len(tr):.1f}")

    # ── combat ──
    print("\nattack outcomes (deck seat / opp seat):")
    for label, seat_of in (("deck", ds), ("opp ", lambda g: 1 - ds(g))):
        cnt = collections.Counter()
        for g in games:
            for rec in g["attacks"]:
                if rec[0] == seat_of(g):
                    cnt[classify(rec)] += 1
        tot = sum(cnt.values())
        print(f"  {label}: {tot} attacks — " + ", ".join(f"{k} {cnt[k]} ({pct(cnt[k], tot):.0f}%)" for k in ("unblocked", "won", "trade", "bounced", "suicide")))
    worst = []
    for g in games:
        for rec in g["attacks"]:
            if rec[0] == ds(g) and classify(rec) == "suicide":
                worst.append((g["path"].rsplit("/", 1)[-1], rec[1], rec[2], rec[3]))
    if worst:
        print(f"  deck-seat suicides ({len(worst)}), first {min(15, len(worst))}:")
        for f, t, a, bl in worst[:15]:
            print(f"    {f} T{t}: {a} into {', '.join(bl)}")

    if games_out:
        with open(games_out, "w") as f:
            for g in games:
                f.write(json.dumps({"path": g["path"], "deck_seat": ds(g), "winner": g["winner"], "turns": g["turns"],
                                    "life": g["life"], "players": g["players"]}) + "\n")


if __name__ == "__main__":
    main()
