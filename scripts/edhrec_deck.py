#!/usr/bin/env python3
"""An EDHREC average deck against the catalog: which cards have no factory,
and — when none are missing — the pod seat's Rust lists.

    python3 scripts/edhrec_deck.py SLUG [SLUG ...]          # fetch + report
    python3 scripts/edhrec_deck.py --file DECK.json         # a saved page

SLUG is the json.edhrec.com average-decks slug (`kotis-the-fangkeeper`).
A card counts as present when a `pub fn <slug>() -> CardDefinition` exists
under crabomination_catalog/src (accents folded, apostrophes dropped, a
split card by both halves or its front face, an MDFC by its front face). The list moves daily; build from a
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


def slug(name, whole=False):
    name = name.replace(" // ", " ") if whole else name.split(" // ")[0]
    folded = "".join(c for c in unicodedata.normalize("NFKD", name) if not unicodedata.combining(c))
    return re.sub(r"[^a-z0-9]+", "_", folded.lower().replace("'", "")).strip("_")


def factories():
    out = set()
    for p in glob.glob(os.path.join(ROOT, "crabomination_catalog", "src", "**", "*.rs"), recursive=True):
        out.update(re.findall(r"pub fn (\w+)\(\) -> CardDefinition", open(p, encoding="utf-8").read()))
    return out


def by_printed_name():
    """Card name -> factory, from the first string literal in each factory's
    body: a factory named apart from its card (`danitha_capashen` for
    "Danitha Capashen, Paragon")."""
    out = {}
    for p in glob.glob(os.path.join(ROOT, "crabomination_catalog", "src", "**", "*.rs"), recursive=True):
        text = open(p, encoding="utf-8").read()
        for m in re.finditer(r"pub fn (\w+)\(\) -> CardDefinition \{(.*?)\n\}", text, re.S):
            lit = re.search(r'"((?:[^"\\]|\\.)*)"', m.group(2))
            if lit:
                out.setdefault(lit.group(1), m.group(1))
    return out


def report(page, fns):
    deck = page["deck"]
    rows = [tuple(x) for x in deck["commander_v2"]] + [tuple(x) for v in deck["cards"].values() for x in v]
    # A split card's factory is named for both halves (`spring_mind`), an
    # MDFC's for its front face.
    printed = by_printed_name()
    name_of = {
        n: slug(n, True) if slug(n, True) in fns else slug(n) if slug(n) in fns else printed.get(n, slug(n))
        for n, _ in rows
    }
    missing = [n for n, _ in rows if n not in BASICS and name_of[n] not in fns]
    print(f"{sum(k for _, k in rows)} cards; {len(missing)} missing: {missing}")
    if missing:
        return
    main, basics = [], {}
    for n, k in rows[len(deck["commander_v2"]):]:
        if n in BASICS:
            basics[n] = basics.get(n, 0) + k
        else:
            main += [name_of[n]] * k
    print("commanders:", ", ".join(name_of[n] for n, _ in deck["commander_v2"]))
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
        # EDHREC answers urllib's default User-Agent with 403.
        req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0 (crabomination)"})
        with urllib.request.urlopen(req, timeout=30) as r:
            print(f"== {s}")
            report(json.load(r), fns)


if __name__ == "__main__":
    main()
