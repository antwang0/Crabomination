#!/usr/bin/env python3
"""Printed token-copy "except …" riders (CR 707.9) vs the definition.

"Create a token that's a copy of X, except it isn't legendary / has haste /
it's a 1/1 / it's an artifact …" — each rider is a field on the copy effect
(`non_legendary`, `extra_keywords`, `override_pt`, `extra_card_types`,
`extra_creature_types`, `override_colors`, `legendary`) or a named variant.
A row names a card whose oracle prints the rider and whose definition carries
no field that could hold it. `--gate` exits 1 on any row.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_copy_except.py /tmp/all.tsv [--gate] [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

EXCEPT = re.compile(r"cop(?:y|ies) of[^.]*?\bexcept\b([^.]*)", re.I)
COLORS = ("white", "blue", "black", "red", "green", "colorless")
# rider -> (oracle regex, definition regex that satisfies it)
RIDERS = [
    ("not legendary", r"(isn't|aren't|is not|not) legendary",
     r"non_legendary: true|NonLegendary|StripLegendary|nonlegendary"),
    ("legendary", r"(it's|they're|is) legendary",
     r"legendary: true|AddSupertype|Legendary\]"),
    ("haste", r"\bhaste\b", r"Haste"),
    ("flying", r"\bflying\b", r"Flying"),
    ("p/t", r"\b\d+/\d+\b|base power and toughness",
     r"override_pt: Some|pt: Some|SetBasePT|SetBasePowerToughness|CopyOnePerOpponentWithTotalStats"),
    ("artifact", r"(an?|is an?|are) artifacts?\b",
     r"extra_card_types: \[[^\]]*Artifact|card_types: \[Artifact\]|AddCardType"),
    ("color", r"\b(" + "|".join(COLORS) + r")\b",
     r"override_colors: Some|SetColor|Colou?rs?\(|CopyOnePerOpponentWithTotalStats"),
]


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    pod = None
    if "--pod" in sys.argv:
        path = sys.argv[sys.argv.index("--pod") + 1]
        pod = {ln.strip() for ln in open(path, encoding="utf-8")}
    out = []
    for line in open(sys.argv[1], encoding="utf-8"):
        if "\t" not in line:
            continue
        name, dbg = line.rstrip("\n").split("\t", 1)
        card = cache.get(name)
        if not isinstance(card, dict) or (pod is not None and name not in pod):
            continue
        for m in EXCEPT.finditer(oracle_of(card)):
            clause = m.group(1)
            for tag, orx, drx in RIDERS:
                if re.search(orx, clause, re.I) and not re.search(drx, dbg):
                    out.append(f"{name}\t{tag}\t{clause.strip()[:90]}")
    out = sorted(set(out))
    print("\n".join(out))
    print(f"# {len(out)} rows", file=sys.stderr)
    if "--gate" in sys.argv and out:
        sys.exit(1)


if __name__ == "__main__":
    main()
