#!/usr/bin/env python3
"""An EDHREC average deck against the catalog: which cards have no factory,
and — when none are missing — the pod seat's Rust lists.

    python3 scripts/edhrec_deck.py SLUG [SLUG ...]          # fetch + report
    python3 scripts/edhrec_deck.py --file DECK.json         # a saved page

SLUG is the json.edhrec.com average-decks slug (`kotis-the-fangkeeper`).
A card counts as present when a `pub fn <slug>() -> CardDefinition` exists
under crabomination_catalog/src (accents folded, apostrophes dropped, a
split / MDFC name by its front face). The list moves daily; build from a
fresh fetch and record the date in the seat's doc comment.
"""
import glob
import json
import os
import re
import sys
import unicodedata
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BASICS = ("Plains", "Island", "Swamp", "Mountain", "Forest", "Wastes")


def slug(name):
    name = name.split(" // ")[0]
    folded = "".join(c for c in unicodedata.normalize("NFKD", name) if not unicodedata.combining(c))
    return re.sub(r"[^a-z0-9]+", "_", folded.lower().replace("'", "")).strip("_")


def factories():
    out = set()
    for p in glob.glob(os.path.join(ROOT, "crabomination_catalog", "src", "**", "*.rs"), recursive=True):
        out.update(re.findall(r"pub fn (\w+)\(\) -> CardDefinition", open(p, encoding="utf-8").read()))
    return out


def report(page, fns):
    deck = page["deck"]
    rows = [tuple(x) for x in deck["commander_v2"]] + [tuple(x) for v in deck["cards"].values() for x in v]
    missing = [n for n, _ in rows if n not in BASICS and slug(n) not in fns]
    print(f"{sum(k for _, k in rows)} cards; {len(missing)} missing: {missing}")
    if missing:
        return
    main, basics = [], {}
    for n, k in rows[len(deck["commander_v2"]):]:
        if n in BASICS:
            basics[n] = basics.get(n, 0) + k
        else:
            main += [slug(n)] * k
    print("commanders:", ", ".join(slug(n) for n, _ in deck["commander_v2"]))
    print(f"main ({len(main)} nonbasic + {sum(basics.values())} basics):")
    print(", ".join(main + [slug(b) for b, k in basics.items() for _ in range(k)]))


def main():
    fns = factories()
    args = sys.argv[1:]
    if args[:1] == ["--file"]:
        report(json.load(open(args[1], encoding="utf-8")), fns)
        return
    for s in args:
        url = f"https://json.edhrec.com/pages/average-decks/{s}.json"
        with urllib.request.urlopen(url, timeout=30) as r:
            print(f"== {s}")
            report(json.load(r), fns)


if __name__ == "__main__":
    main()
