#!/usr/bin/env python3
"""Printed "target … card from/in your (a) graveyard" whose definition pulls
the card at resolution (`Selector::Take` over `CardsInZone { zone: Graveyard }`)
with no graveyard target slot — the caster never chooses and the card is not a
target (CR 115.1). Exploratory: a row may be a documented approximation.

    target/debug/dump_cards --grep "" > /tmp/all.tsv
    python3 scripts/audit_graveyard_take.py /tmp/all.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

PRINTED = re.compile(r"\btarget [^.]{0,60}cards? (?:from|in) (?:your|a|an opponent's|target player's|that player's) graveyard")


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
        text = re.sub(r"[\"“][^\"”]*[\"”]", "", oracle_of(card))
        if not PRINTED.search(text):
            continue
        d = re.sub(r"TokenDefinition \{.*?\}\}", "", dbg)
        targeted = re.search(r"TargetFiltered \{ slot: \d+, filter: [^}]*Graveyard", d) or (
            "ApplyToTargets" in d and "Graveyard" in d)
        if re.search(r"Take \{ inner: CardsInZone \{[^}]*zone: Graveyard", d) and not targeted:
            out.append(name)
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
