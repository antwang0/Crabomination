#!/usr/bin/env python3
"""Printed "power / toughness N or less / or greater" vs the definition's
`PowerAtMost(N)` / `PowerAtLeast(N)` / `ToughnessAtMost(N)` /
`ToughnessAtLeast(N)` (exploratory). A row names a printed threshold whose
requirement, with that number, appears nowhere in the definition.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_pt_thresholds.py /tmp/all.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

PRINTED = re.compile(r"\b(power|toughness) (\d+) or (less|greater)\b", re.I)


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
            stat = m.group(1).capitalize()
            n = int(m.group(2))
            kind = "AtMost" if m.group(3).lower() == "less" else "AtLeast"
            want = f"{stat}{kind}({n})"
            alt = f"{stat}{'AtLeast' if kind == 'AtMost' else 'AtMost'}({n + 1 if kind == 'AtMost' else n - 1})"
            if want not in dbg and f"Not({alt})" not in dbg:
                out.append(f"{name}\t{want}\t{m.group(0)}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
