#!/usr/bin/env python3
"""Printed "target ... card from your graveyard" read as any graveyard.

`SelectionRequirement::InGraveyard` is ANY graveyard; "your graveyard" is
`InYourGraveyard` (or an `OwnedByYou` alongside). ~35 cards shipped with the
former — Buried Ruin could return an opponent's artifact to your hand. The
reverse row ("from a graveyard" read as yours) is reported too.

    cargo build --profile release-fast -p crabomination --bin dump_cards
    target/release-fast/dump_cards --grep TargetFiltered > /tmp/tf.tsv
    python3 scripts/audit_your_graveyard.py /tmp/tf.tsv [--gate]
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")


def target_filters(dbg):
    out = []
    for m in re.finditer(r"TargetFiltered \{ slot: \d+, filter: ", dbg):
        i = j = m.end()
        depth = 0
        while j < len(dbg):
            ch = dbg[j]
            if ch in "({[":
                depth += 1
            elif ch in ")}]":
                if depth == 0:
                    break
                depth -= 1
            j += 1
        out.append(dbg[i:j])
    return out


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not args:
        sys.exit(__doc__)
    cache = {k: v for k, v in json.load(open(CACHE)).items() if isinstance(v, dict)}
    by_name = {}
    for k, v in cache.items():
        by_name.setdefault(k.lower(), v)
        by_name.setdefault(k.split(" // ")[0].lower(), v)
    rows = []
    for line in open(args[0]):
        name, dbg = line.rstrip("\n").split("\t", 1)
        card = by_name.get(name.lower())
        if not card:
            continue
        oracle = card.get("oracle_text") or "\n".join(f.get("oracle_text", "") for f in card.get("card_faces", []))
        oracle = re.sub(r"\([^)]*\)", "", oracle).lower()
        fl = " ".join(f for f in target_filters(dbg) if "Graveyard" in f)
        if not fl:
            continue
        yours = re.search(r"target [^.]*card[^.]* from your graveyard", oracle)
        other = re.search(
            r"a graveyard|any graveyard|graveyards|opponent.s graveyard|that player.s graveyard|"
            r"target player.s graveyard|its owner.s graveyard",
            oracle,
        )
        if yours and not other and not re.search(r"InYourGraveyard|OwnedByYou|InOpponent", fl):
            rows.append(f"{name:40} printed your graveyard; filter {fl[:90]}")
        anyg = re.search(r"target [^.]*card[^.]* from a graveyard", oracle)
        if anyg and not yours and "InYourGraveyard" in fl and not re.search(r"(?<![A-Za-z])InGraveyard\b", fl):
            rows.append(f"{name:40} printed a graveyard; filter {fl[:90]}")
    for r in rows:
        print(r)
    print(f"{len(rows)} rows")
    if "--gate" in sys.argv and rows:
        sys.exit(1)


main()
