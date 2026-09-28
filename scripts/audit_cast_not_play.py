#!/usr/bin/env python3
"""Printed "you may CAST" vs an impulse grant that lets a land be PLAYED.

`ExileTopAndGrantMayPlay` stamps a may-play permission, so a land it exiles
can be played as a land drop — right for "you may play", wrong for "you may
cast" (CR 305.1: a land is played, never cast). The cast-only wording needs
`RestrictMayPlayToCasting` over the exiled cards after the grant. A row is a
card whose oracle (reminder text stripped) says "cast" and never "play" and
whose definition grants that way without the restriction. `--gate` exits 1
on any row.

    target/debug/dump_cards --grep "" > /tmp/all.tsv
    python3 scripts/audit_cast_not_play.py /tmp/all.tsv [--gate]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

# The cube pool is frozen (INCOMPLETE_CARDS' frozen row).
ALLOWLIST = {"Chandra, Torch of Defiance", "Ragavan, Nimble Pilferer", "Robber of the Rich"}


def main():
    path = sys.argv[1]
    gate = "--gate" in sys.argv
    cache = json.load(open(CACHE))
    rows, seen = [], set()
    for line in open(path):
        name, _, d = line.rstrip("\n").partition("\t")
        if name in seen or name in ALLOWLIST:
            continue
        seen.add(name)
        if "ExileTopAndGrantMayPlay" not in d or "RestrictMayPlayToCasting" in d:
            continue
        entry = cache.get(name)
        raw = oracle_of(entry) if isinstance(entry, dict) else ""
        # "play" anywhere (a Junk token's reminder text included) is a play grant.
        if re.search(r"\bplay\b", raw) or not re.search(r"\bcast\b", re.sub(r"\([^)]*\)", "", raw)):
            continue
        rows.append(name)
    for r in sorted(rows):
        print(r)
    print(f"# {len(rows)} rows")
    if gate and rows:
        sys.exit(1)


if __name__ == "__main__":
    main()
