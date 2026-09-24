#!/usr/bin/env python3
"""Rank official Commander precons by how many of their cards the catalog lacks.

Downloads MTGJSON's `DeckList.json` and each "Commander Deck" file (cached in
`scripts/.mtgjson_decks/`, gitignored by name), then checks every card
against the catalog: its slug is a `pub fn … -> CardDefinition` factory, or
its name is a `name: "…"` literal (helpers such as `land("…")` build many
cards, so a name-field match alone undercounts). The offline
Scryfall cache cannot say how a precon splits into decks; MTGJSON can, which
is how `pod::decks::TEVAL_MAIN` (Sultai Arisen) was taken card for card.

    python3 scripts/precon_scan.py            # top 30, fewest missing first
    python3 scripts/precon_scan.py --top 60   # more rows (the 0-missing ones crowd the top)
    python3 scripts/precon_scan.py --deck SultaiArisen_TDC   # one list, as factories

⚠ "In the catalog" is a name match, not a verdict: a present card can still be
an approximation. Read INCOMPLETE_CARDS.md / the card's doc before relying on it.
MTGJSON rate-limits urllib's default user agent (403); this uses curl.
"""

import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CATALOG = os.path.join(ROOT, "crabomination_catalog", "src")
CACHE = os.path.join(ROOT, "scripts", ".mtgjson_decks")
BASE = "https://mtgjson.com/api/v5"

sys.path.insert(0, os.path.join(ROOT, "scripts"))
from pod_deck_candidates import slug  # noqa: E402


def fetch(name, sub=""):
    """`sub` is MTGJSON's directory: deck files live under `decks/`, the list at the root."""
    os.makedirs(CACHE, exist_ok=True)
    path = os.path.join(CACHE, name)
    if not os.path.exists(path):
        subprocess.run(["curl", "-sSf", "-o", path, f"{BASE}/{sub}{name}"], check=True)
    return json.load(open(path, encoding="utf-8"))["data"]


def catalog_names():
    names = set()
    name_pat = re.compile(r'name:\s*"((?:[^"\\]|\\.)*)"')
    fn_pat = re.compile(r"pub fn (\w+)\(\) -> (?:crate::card::)?CardDefinition")
    # A factory whose body is one helper call naming the card: `karoo("Karoo", …)`.
    helper_pat = re.compile(r'-> (?:crate::card::)?CardDefinition \{\s*[\w:]+\(\s*"((?:[^"\\]|\\.)*)"')
    for dp, _, fs in os.walk(CATALOG):
        for f in fs:
            if f.endswith(".rs"):
                src = open(os.path.join(dp, f), encoding="utf-8").read()
                names.update(m.group(1).replace("\\'", "'") for m in name_pat.finditer(src))
                names.update(fn_pat.findall(src))
                names.update(helper_pat.findall(src))
    return names


def cards(deck):
    out = []
    for c in deck.get("commander", []) + deck.get("mainBoard", []):
        out += [c["name"]] * c.get("count", 1)
    return out


def main():
    have = catalog_names()
    present = lambda n: any(k in have for k in (n, n.split(" // ")[0], slug(n.split(" // ")[0])))
    if "--deck" in sys.argv:
        deck = fetch(sys.argv[sys.argv.index("--deck") + 1] + ".json", "decks/")
        print("commanders:", ", ".join(slug(c["name"].split(" // ")[0]) for c in deck["commander"]))
        main = [c["name"] for c in deck["mainBoard"] for _ in range(c.get("count", 1))]
        print(f"main ({len(main)}):", ", ".join(slug(n.split(" // ")[0]) for n in main))
        # The commanders too: the first draft checked only the 99 and called a
        # list complete whose partner pair wasn't in the catalog at all.
        print("missing:", sorted({n for n in cards(deck) if not present(n)}))
        return 0
    rows = []
    for d in fetch("DeckList.json"):
        if d["type"] != "Commander Deck" or "Collector" in d["name"]:
            continue
        try:
            names = cards(fetch(d["fileName"] + ".json", "decks/"))
        except subprocess.CalledProcessError:
            print("fetch failed:", d["fileName"], file=sys.stderr)
            continue
        miss = sorted({n for n in names if not present(n)})
        rows.append((len(miss), d["releaseDate"], d["name"], d["fileName"], miss))
    rows.sort()
    top = int(sys.argv[sys.argv.index("--top") + 1]) if "--top" in sys.argv else 30
    for n, date, name, fname, miss in rows[:top]:
        print(f"{n:3d}  {date}  {name:<34} {fname:<36} {', '.join(miss[:8])}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
