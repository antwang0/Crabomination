#!/usr/bin/env python3
"""Printed "create … N/M … creature token" vs the token definitions a card
mints. A row names a printed N/M that no `TokenDefinition { … power: N,
toughness: M … }` in the definition carries (exploratory; a shared token
helper or a copy/X-sized token reads as a row).

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_token_pt.py /tmp/all.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

PRINTED = re.compile(r"\bcreates? [^.]*?\b(\d+)/(\d+)\b[^.]*?\btokens?\b", re.I)
TOKEN = re.compile(r"TokenDefinition \{ name: \"[^\"]*\", power: (-?\d+), toughness: (-?\d+)")


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
        have = set(TOKEN.findall(dbg))
        if not have:
            continue
        for m in PRINTED.finditer(oracle_of(card)):
            if "copy" in m.group(0).lower():
                continue
            if (m.group(1), m.group(2)) not in have:
                out.append(f"{name}\t{m.group(1)}/{m.group(2)}\thave {sorted(have)}\t{m.group(0)[:80]}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(out)} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
