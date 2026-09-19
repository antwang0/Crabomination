#!/usr/bin/env python3
"""Implemented cards a pod deck of a given colour identity may legally run.

Joins the catalog's `pub fn … -> CardDefinition` factories to the offline
Scryfall cache and keeps the ones whose `color_identity` is a subset of the
commander's and whose `legalities.commander` is legal — i.e. the CR 903.4 and
CR 903.5b gates `format::validate_commander_deck` will apply, answered before
a deck list is written rather than after the suite rejects it.

    python3 scripts/pod_deck_candidates.py G            # mono-green
    python3 scripts/pod_deck_candidates.py GU --type creature
    python3 scripts/pod_deck_candidates.py G --have crabomination/src/pod/decks.rs

`--have <file>` marks the candidates already named in that file, which is how
a new list reuses the four existing ones without re-checking each card by
hand. Names are printed as the factory name, since that is what a deck list
holds.

⚠ **It cannot tell a complete implementation from an approximated one.** A
factory that ships half the card still passes every gate here; read
`INCOMPLETE_CARDS.md` for the card before leaning on it.
"""

import json
import os
import re
import sys
import unicodedata

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CATALOG = os.path.join(ROOT, "crabomination_catalog", "src")
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")


def slug(name):
    folded = unicodedata.normalize("NFKD", name)
    folded = "".join(c for c in folded if not unicodedata.combining(c))
    return re.sub(r"[^a-z0-9]+", "_", folded.lower().replace("'", "")).strip("_")


def factories():
    """{card name: fn} for every factory whose own name resolves."""
    out = {}
    for dp, _, fs in os.walk(CATALOG):
        for f in fs:
            if not f.endswith(".rs"):
                continue
            src = open(os.path.join(dp, f), encoding="utf-8").read()
            for m in re.finditer(r"pub fn (\w+)\(\) -> CardDefinition \{", src):
                start, depth, i = m.end() - 1, 0, m.end() - 1
                while i < len(src):
                    if src[i] == "{":
                        depth += 1
                    elif src[i] == "}":
                        depth -= 1
                        if depth == 0:
                            break
                    i += 1
                body = src[start : i + 1]
                fn = m.group(1)
                own = next(
                    (n for n in re.findall(r'"((?:[^"\\]|\\.)*)"', body) if slug(n) == fn),
                    None,
                )
                if own:
                    out.setdefault(own, fn)
    return out


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    identity = set(sys.argv[1].upper()) - {"C"}
    want_type = None
    if "--type" in sys.argv:
        want_type = sys.argv[sys.argv.index("--type") + 1].lower()
    have = set()
    if "--have" in sys.argv:
        have = set(
            re.findall(r"[a-z][a-z0-9_]{3,}", open(sys.argv[sys.argv.index("--have") + 1]).read())
        )

    cache = json.load(open(CACHE, encoding="utf-8"))
    rows = []
    for name, fn in factories().items():
        card = cache.get(name)
        if not isinstance(card, dict):
            continue
        ci = set(card.get("color_identity") or [])
        if not ci <= identity:
            continue
        legal = (card.get("legalities") or {}).get("commander")
        if legal is not None and legal != "legal":
            continue
        tl = (card.get("type_line") or "").lower()
        if want_type and want_type not in tl:
            continue
        rows.append((card.get("cmc") or 0, tl.split("—")[0].strip(), name, fn, fn in have))
    rows.sort()
    for cmc, tl, name, fn, seen in rows:
        print(f"{'*' if seen else ' '} {cmc:>4.0f}  {tl:<28} {name:<34} {fn}")
    print(f"# {len(rows)} candidates, {sum(1 for r in rows if r[4])} already in --have")
    return 0


if __name__ == "__main__":
    sys.exit(main())
