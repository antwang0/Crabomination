#!/usr/bin/env python3
"""Printed "Whenever ~ or another <X> enters / dies / attacks" vs a definition
that listens only with `AnotherOfYours` (or another other-than-self filter)
and never for the source itself (exploratory). A row names a card whose
self half is missing.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_self_or_another.py /tmp/all.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

PRINTED = re.compile(r"\bwhenever ~ or another [^.,]*?\b(enters|dies|attacks|leaves|becomes tapped|is put into)\b",
                     re.I)
KIND = {"enters": "EntersBattlefield", "dies": r"CreatureDied|PermanentDied|Died", "attacks": "Attacks",
        "leaves": "LeavesBattlefield", "becomes tapped": "Tapped", "is put into": "Graveyard|Died"}


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
        short = name.split(",")[0]
        text = oracle_of(card).replace(name, "~").replace(short, "~")
        text = re.sub(r"\bthis (creature|artifact|enchantment|permanent|land)\b", "~", text, flags=re.I)
        for m in PRINTED.finditer(text):
            kind = KIND[m.group(1).lower()]
            scopes = re.findall(r"kind: (\w+)(?:\([^)]*\))?, scope: (\w+)", dbg)
            hit = [s for k, s in scopes if re.fullmatch(kind, k) or re.search(kind, k)]
            if hit and all(s in ("AnotherOfYours", "OtherCreature", "AnotherCreature") for s in hit):
                out.append(f"{name}\t{hit}\t{m.group(0)[:90]}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
