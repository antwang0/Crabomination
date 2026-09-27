#!/usr/bin/env python3
"""Printed "create a tapped … token" vs a definition that mints it untapped.

An untapped Treasure from Blood Money / Goldvein Hydra pays for the rest of
the turn. Accepted shapes: `TokenDefinition { tapped: true }`, a
`Tap { what: LastCreatedToken(s) }` after the mint, an `EntersTapped` or
`…Tapped…` effect. `--gate` exits 1 on any row.

    target/debug/dump_cards --grep "" > /tmp/all.tsv
    python3 scripts/audit_tapped_tokens.py /tmp/all.tsv [--gate]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

PHRASE = re.compile(r"\bcreate[^.]*?\btapped\b[^.]*", re.I)
ACCEPT = re.compile(r"tapped: true|Tap \{ what: LastCreated|EntersTapped|[A-Z]\w*Tapped\w*|CreateTokenAttacking")
# "…with 'whenever this token becomes tapped'" — not a tapped mint.
ALLOWLIST = {"Wildfire Awakener"}


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    out = []
    for line in open(sys.argv[1], encoding="utf-8"):
        if "\t" not in line:
            continue
        name, dbg = line.rstrip("\n").split("\t", 1)
        card = cache.get(name)
        if not isinstance(card, dict) or name in ALLOWLIST:
            continue
        hits = [m.group(0) for m in PHRASE.finditer(oracle_of(card))]
        hits = [h for h in hits if "tapped and attacking" not in h and "token" in h.lower()]
        if hits and not ACCEPT.search(dbg):
            out.append(f"{name}\t{hits[0][:100]}")
    print("\n".join(sorted(out)))
    print(f"# {len(out)} rows", file=sys.stderr)
    if "--gate" in sys.argv and out:
        sys.exit(1)


if __name__ == "__main__":
    main()
