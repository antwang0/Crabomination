#!/usr/bin/env python3
"""Printed "gain control of …" durations (CR 611.2, 611.2c) vs the definition.

Three printed shapes: "until end of turn" (threaten -> `duration: EndOfTurn`),
"for as long as …" (one of the `GainControlWhile*` variants), and no duration
(a permanent steal -> `duration: Permanent`). A row names a card whose
oracle prints one shape and whose definition holds no control change of that
shape. Exploratory; `--gate` exits 1 on any row.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_control_duration.py /tmp/all.tsv [--pod NAMES] [--gate]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

CLAUSE = re.compile(r"gains? control of ([^.]*)", re.I)
SHAPES = {
    "eot": r"GainControl \{[^}]*duration: EndOfTurn|UntilEndOfTurn|Threaten|EndOfTurn",
    "while": r"GainControlWhile|WhileSource|WhileYouControl|WhileCounter",
    "perm": r"GainControl \{[^}]*duration: Permanent|ExchangeControl|GainControlAndReattach|Permanent",
}


def shape(clause):
    if re.search(r"until end of turn", clause, re.I):
        return "eot"
    if re.search(r"for as long as|until .* leaves", clause, re.I):
        return "while"
    if re.search(r"until (your|the|its|their) next|until the end of", clause, re.I):
        return None  # other bounded durations: read by hand
    return "perm"


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
        if "Control" not in dbg:
            continue
        for m in CLAUSE.finditer(oracle_of(card)):
            s = shape(m.group(1))
            if s and not re.search(SHAPES[s], dbg):
                out.append(f"{name}\t{s}\t{m.group(1)[:90]}")
    out = sorted(set(out))
    print("\n".join(out))
    print(f"# {len(out)} rows", file=sys.stderr)
    if "--gate" in sys.argv and out:
        sys.exit(1)


if __name__ == "__main__":
    main()
