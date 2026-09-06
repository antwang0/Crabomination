#!/usr/bin/env python3
"""Generate single-change variants of a decklist from a sealed pool.

    deck_variants.py --deck LIST --pool POOL --cards cards.json --out DIR MODE ...

MODEs:
  cuts  --neutral CARD          each distinct non-land card cut, CARD added
  adds  --neutral-cut CARD      each legal pool card added, CARD cut
  swaps --cuts A;B --adds X;Y   every (cut, add) pair
  lands                         +1/-1 of each basic the deck plays (spell count fixed
                                by cutting/adding --neutral)

A card is "legal" when the deck's mana base has at least --min-sources (3)
sources of every coloured pip it needs (hybrid: either colour), the deck
holds fewer copies than the pool, and it is not already the neutral card.
`cards.json` is `dump_cards` output (name, mana_cost). Variants are written
as `<out>/<slug>.txt` with a comment line describing the change; a
manifest `<out>/manifest.tsv` maps slug -> change.
"""
import argparse
import json
import os
import re
from collections import Counter

BASICS = {"Plains": "W", "Island": "U", "Swamp": "B", "Mountain": "R", "Forest": "G"}
DUALS = {
    "Forum of Amity": "WB", "Fields of Strife": "RW", "Paradox Gardens": "GU", "Titan's Grave": "BG",
    "Spectacle Summit": "UR", "Dreamroot Cascade": "GU", "Shattered Sanctum": "WB", "Terramorphic Expanse": "WUBRG",
}


def load_list(path):
    d = Counter()
    for line in open(path):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        n, name = line.split(" ", 1)
        d[name.strip()] += int(n)
    return d


def canon(name, table):
    key = name.lower().split(" // ")[0]
    return table.get(key, name)


def pips(cost):
    """Coloured pip requirements as a list of colour-option sets."""
    out = []
    for sym in re.findall(r"\{([^}]*)\}", cost or ""):
        if "/" in sym:
            opts = {c for c in sym.split("/") if c in "WUBRG"}
            if opts:
                out.append(opts)
        elif sym in "WUBRG" and sym:
            out.append({sym})
    return out


def sources(deck):
    s = Counter()
    for name, n in deck.items():
        if name in BASICS:
            s[BASICS[name]] += n
        elif name in DUALS:
            for c in DUALS[name]:
                s[c] += n
    return s


def is_land(name):
    return name in BASICS or name in DUALS


def legal(name, deck, pool, cost_of, min_sources):
    if deck.get(name, 0) >= pool.get(name, 0):
        return False
    src = sources(deck)
    for opts in pips(cost_of.get(name, "")):
        if max(src[c] for c in opts) < min_sources:
            return False
    return True


def slug(s):
    return re.sub(r"[^a-z0-9]+", "_", s.lower()).strip("_")[:40]


def write(out, name, deck, note, manifest):
    path = os.path.join(out, f"{name}.txt")
    n = sum(deck.values())
    lands = sum(v for k, v in deck.items() if is_land(k))
    with open(path, "w") as f:
        f.write(f"# {name}: {note}\n# {n} cards, {lands} lands\n")
        for k, v in sorted(deck.items(), key=lambda kv: (is_land(kv[0]), kv[0])):
            f.write(f"{v} {k}\n")
    manifest.write(f"{name}\t{note}\n")


def changed(deck, cuts=(), adds=()):
    d = Counter(deck)
    for c in cuts:
        if d[c] <= 0:
            raise SystemExit(f"cannot cut {c}: not in deck")
        d[c] -= 1
        if d[c] == 0:
            del d[c]
    for a in adds:
        d[a] += 1
    return d


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--deck", required=True)
    ap.add_argument("--pool", required=True)
    ap.add_argument("--cards", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--min-sources", type=int, default=3)
    ap.add_argument("--exclude", default="", help="';'-separated pool cards never to add")
    ap.add_argument("--prefix", default="", help="file-name prefix (a generation tag: the same swap on a different base needs a distinct name)")
    ap.add_argument("mode", choices=["cuts", "adds", "swaps", "lands"])
    ap.add_argument("--neutral", default="Oracle's Restoration")
    ap.add_argument("--neutral-cut", default=None)
    ap.add_argument("--cuts", default="")
    ap.add_argument("--adds", default="")
    a = ap.parse_args()

    cards = json.load(open(a.cards))
    table = {c["name"].lower(): c["name"] for c in cards}
    cost_of = {c["name"]: c["mana_cost"] for c in cards}
    deck = Counter({canon(k, table): v for k, v in load_list(a.deck).items()})
    pool = Counter({canon(k, table): v for k, v in load_list(a.pool).items()})
    # Basics are unlimited in sealed.
    for b in BASICS:
        pool[b] = 99
    exclude = {canon(x.strip(), table) for x in a.exclude.split(";") if x.strip()}
    os.makedirs(a.out, exist_ok=True)
    manifest = open(os.path.join(a.out, "manifest.tsv"), "a")
    neutral = canon(a.neutral, table)
    made = 0
    if a.mode == "cuts":
        for name in sorted(deck):
            if is_land(name) or name == neutral:
                continue
            d = changed(deck, cuts=[name], adds=[neutral])
            write(a.out, f"{a.prefix}cut_{slug(name)}", d, f"-{name} +{neutral}", manifest)
            made += 1
    elif a.mode == "adds":
        ncut = canon(a.neutral_cut or a.neutral, table)
        for name in sorted(pool):
            if is_land(name) or name == ncut or name in exclude:
                continue
            if not legal(name, deck, pool, cost_of, a.min_sources):
                continue
            d = changed(deck, cuts=[ncut], adds=[name])
            write(a.out, f"{a.prefix}add_{slug(name)}", d, f"+{name} -{ncut}", manifest)
            made += 1
    elif a.mode == "swaps":
        cuts = [canon(x.strip(), table) for x in a.cuts.split(";") if x.strip()]
        adds = [canon(x.strip(), table) for x in a.adds.split(";") if x.strip()]
        for c in cuts:
            for x in adds:
                if c == x:
                    continue
                d = changed(deck, cuts=[c], adds=[x])
                write(a.out, f"{a.prefix}swap_{slug(c)}__{slug(x)}", d, f"-{c} +{x}", manifest)
                made += 1
    elif a.mode == "lands":
        for b in BASICS:
            if deck.get(b, 0) > 0:
                d = changed(deck, cuts=[b], adds=[neutral])
                write(a.out, f"{a.prefix}land_minus_{slug(b)}", d, f"-{b} +{neutral}", manifest)
                made += 1
            d = changed(deck, cuts=[neutral] if deck.get(neutral, 0) else [], adds=[b])
            note = f"+{b} -{neutral}" if deck.get(neutral, 0) else f"+{b} (41 cards)"
            write(a.out, f"{a.prefix}land_plus_{slug(b)}", d, note, manifest)
            made += 1
    print(f"{made} variants written to {a.out}")


if __name__ == "__main__":
    main()
