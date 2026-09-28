#!/usr/bin/env python3
"""Printed "to its owner's hand" / "its owner's library" vs a definition that
moves the card to YOUR hand / library.

"Return a creature you control to its owner's hand" moved with
`Hand(You)` hands a stolen creature to the thief (CR 110.2 — a card goes to
its owner's hand). A row names a card whose oracle prints the owner's hand,
never "your hand", and whose every `Hand(..)` destination is `You`. `--gate`
exits 1 on any row (61 fixed 2026-09-28, the Invasion `bounce_own` riders
among them).

    target/debug/dump_cards --grep "" > /tmp/all.tsv
    python3 scripts/audit_owner_hand.py /tmp/all.tsv [--pod NAMES] [--gate]
"""

import json
import os
import re
import sys

CACHE = os.path.join(os.path.dirname(os.path.abspath(__file__)), ".scryfall_cache.json")
OWNERS = re.compile(r"to (its|their) owners?'s? hands?", re.I)
YOURS = re.compile(r"(into|to) your hand", re.I)
LIB_OWNERS = re.compile(r"(its|their) owners?'s? librar", re.I)
LIB_YOURS = re.compile(r"your library", re.I)


def oracle(c):
    t = c.get("oracle_text") or ""
    if c.get("card_faces"):
        t = "\n".join(f.get("oracle_text", "") for f in c["card_faces"])
    return re.sub(r'"[^"]*"', "", re.sub(r"\([^)]*\)", "", t))


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    pod = None
    if "--pod" in sys.argv:
        pod = {ln.strip() for ln in open(sys.argv[sys.argv.index("--pod") + 1], encoding="utf-8")}
    out = []
    for line in open(sys.argv[1], encoding="utf-8"):
        if "\t" not in line:
            continue
        name, dbg = line.rstrip("\n").split("\t", 1)
        card = cache.get(name)
        if not isinstance(card, dict) or (pod is not None and name not in pod):
            continue
        text = oracle(card)
        if OWNERS.search(text) and not YOURS.search(text):
            if set(re.findall(r"to: Hand\((\w+)", dbg)) == {"You"}:
                out.append(f"{name}\thand")
        if LIB_OWNERS.search(text) and not LIB_YOURS.search(text):
            if set(re.findall(r"Library \{ who: (\w+)", dbg)) == {"You"}:
                out.append(f"{name}\tlibrary")
    print("\n".join(sorted(out)))
    print(f"# {len(out)} rows", file=sys.stderr)
    if "--gate" in sys.argv and out:
        sys.exit(1)


if __name__ == "__main__":
    main()
