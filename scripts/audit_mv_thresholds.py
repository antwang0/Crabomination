#!/usr/bin/env python3
"""Printed "mana value N or less / or greater / N" vs the definition's
`ManaValueAtMost(N)` / `ManaValueAtLeast(N)` / `ManaValueExactly(N)`.
A row names a printed threshold whose requirement, with that number,
appears nowhere in the definition (a sibling of `audit_pt_thresholds.py`).

    target/debug/dump_cards --grep "" > /tmp/all.tsv
    python3 scripts/audit_mv_thresholds.py /tmp/all.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

PRINTED = re.compile(r"\b(?:mana value|converted mana cost) (\d+)( or (?:less|greater))?\b", re.I)


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
        text = re.sub(r"[\"“][^\"”]*[\"”]", "", oracle_of(card))
        for m in PRINTED.finditer(text):
            n = int(m.group(1))
            tail = (m.group(2) or "").strip().lower()
            if tail == "or less":
                want, alts = f"ManaValueAtMost({n})", [f"Not(ManaValueAtLeast({n + 1}))"]
            elif tail == "or greater":
                want, alts = f"ManaValueAtLeast({n})", [f"Not(ManaValueAtMost({n - 1}))"]
            else:
                want, alts = f"ManaValueExactly({n})", []
            if want not in dbg and not any(a in dbg for a in alts):
                out.append(f"{name}\t{want}\t{m.group(0)}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
