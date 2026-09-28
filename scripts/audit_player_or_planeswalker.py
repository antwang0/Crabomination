#!/usr/bin/env python3
""""Combat damage to a player or planeswalker" cards that fire on players only
(CR 510.2).

Every card whose oracle text says "deals combat damage to a player or
planeswalker" must carry an `EventKind::DealsCombatDamageToPlaneswalker`
trigger beside its player one (Bladewing, Dreadhorde Butcher, Psychic Frog
fired on players only). The factory body is found by the card's quoted name
in `crabomination_catalog/src`. `--gate` exits 1 on any row.

    target/debug/dump_cards --effects > /tmp/ef.tsv
    python3 scripts/audit_player_or_planeswalker.py /tmp/ef.tsv [--gate]
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")
TEXT = re.compile(r"combat damage to a player or (?:a )?planeswalker|combat damage to an opponent or (?:a )?planeswalker", re.I)


def bodies(names):
    found = {}
    for root, _, files in os.walk(os.path.join(ROOT, "crabomination_catalog", "src")):
        for f in files:
            if not f.endswith(".rs"):
                continue
            s = open(os.path.join(root, f), encoding="utf-8").read()
            for n in names:
                if n in found:
                    continue
                i = s.find('"' + n.replace('"', '\\"') + '"')
                if i < 0:
                    continue
                a = s.rfind("pub fn", 0, i)
                b = s.find("\npub fn", i)
                found[n] = s[a:b if b > 0 else len(s)]
    return found


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not args:
        sys.exit(__doc__)
    cache = {k.lower(): v for k, v in json.load(open(CACHE)).items() if isinstance(v, dict)}
    names = {line.split("\t", 1)[0] for line in open(args[0], encoding="utf-8")}
    hits = [n for n in names if TEXT.search((cache.get(n.lower()) or {}).get("oracle_text") or "")]
    src = bodies(hits)
    rows = sorted(n for n in hits if n in src and "DealsCombatDamageToPlaneswalker" not in src[n])
    print("\n".join(rows))
    print(f"# {len(rows)} rows ({len(hits)} cards print the clause)", file=sys.stderr)
    if "--gate" in sys.argv and rows:
        sys.exit(1)


if __name__ == "__main__":
    main()
