#!/usr/bin/env python3
"""Two one-word oracle filters vs the definition (exploratory).

- `other`: "other creatures you control get / have …" (an anthem that
  excludes its source) whose definition carries no `OtherThanSource` /
  `Other…` requirement — the source pumps itself.
- `side`: "… you don't control" / "an opponent controls" on a mass effect
  whose definition carries no opponent-side requirement — the effect hits
  your own board too.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_other_and_side.py /tmp/all.tsv [--pod NAMES] [--gate]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

OTHER = re.compile(r"\bother [^.]*?\byou control (get|have|gain)\b", re.I)
OTHER_OK = re.compile(r"OtherThan|Other\w*|Another|OtherCreatures|NotSource|ExceptSource")
SIDE = re.compile(r"\b(all|each) [^.]*?(you don't control|your opponents control)\b", re.I)
SIDE_OK = re.compile(r"ControlledByOpponent|Not\(ControlledByYou\)|NotControlledByYou|EachOpponent|"
                     r"Opponent|YouDontControl|NotYours")


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
        text = oracle_of(card)
        m = OTHER.search(text)
        if m and not OTHER_OK.search(dbg):
            out.append(f"{name}\tother\t{m.group(0)[:90]}")
        m = SIDE.search(text)
        if m and not SIDE_OK.search(dbg):
            out.append(f"{name}\tside\t{m.group(0)[:90]}")
    print("\n".join(sorted(out)))
    print(f"# {len(out)} rows", file=sys.stderr)
    if "--gate" in sys.argv and out:
        sys.exit(1)


if __name__ == "__main__":
    main()
