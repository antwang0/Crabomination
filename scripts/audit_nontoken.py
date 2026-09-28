#!/usr/bin/env python3
"""Printed "nontoken" / "token" filters vs a definition that has neither.

"Whenever a nontoken creature you control dies" written without the filter
fires on every token too (a doubling loop at a token-heavy seat). A row names
a card whose oracle prints "nontoken" and whose definition holds no token
test at all. `--gate` exits 1 on any row.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_nontoken.py /tmp/all.tsv [--pod NAMES] [--gate]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

ACCEPT = re.compile(r"NotToken|IsToken|Nontoken|NonToken|is_token|nontoken")


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
        m = re.search(r"[^.]*\bnontoken\b[^.]*", oracle_of(card), re.I)
        if m and not ACCEPT.search(dbg):
            out.append(f"{name}\t{m.group(0).strip()[:100]}")
    print("\n".join(sorted(out)))
    print(f"# {len(out)} rows", file=sys.stderr)
    if "--gate" in sys.argv and out:
        sys.exit(1)


if __name__ == "__main__":
    main()
