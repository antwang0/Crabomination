#!/usr/bin/env python3
"""CR citations in Rust sources that name no rule in the shipped CR text.

A comment or test that cites "CR 701.15g" for regeneration was right under
an older numbering (701.15 is goad now); nothing flags the drift. This walks
every `CR NNN.N[x]` in the Rust tree and reports those absent from
`crabomination/MagicCompRules_*.txt` (the newest file), most-cited first.

    python3 scripts/audit_cr_citations.py            # the rows
    python3 scripts/audit_cr_citations.py --count    # the total only
    python3 scripts/audit_cr_citations.py --max N    # exit 1 above N rows (a ratchet)

A number that exists can still name the wrong rule; this only catches the
ones that exist nowhere.
"""

import glob
import os
import re
import sys
from collections import Counter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DIRS = ["crabomination/src", "crabomination_base/src", "crabomination_catalog/src", "crabomination_tests", "crabomination_client/src"]
CITE = re.compile(r"CR ([1-9][0-9]{2}\.[0-9]+[a-z]?)")


def main():
    rules_file = sorted(glob.glob(os.path.join(ROOT, "crabomination", "MagicCompRules_*.txt")))[-1]
    rules = set()
    for line in open(rules_file, encoding="utf-8", errors="replace"):
        m = re.match(r"^([1-9][0-9]{2}\.[0-9]+[a-z]?)\.? ", line)
        if m:
            rules.add(m.group(1))
    cited, where = Counter(), {}
    for d in DIRS:
        for dp, _, files in os.walk(os.path.join(ROOT, d)):
            for f in files:
                if not f.endswith(".rs"):
                    continue
                path = os.path.join(dp, f)
                for i, line in enumerate(open(path, encoding="utf-8", errors="replace"), 1):
                    for r in CITE.findall(line):
                        if r not in rules:
                            cited[r] += 1
                            where.setdefault(r, f"{os.path.relpath(path, ROOT)}:{i}")
    rows = cited.most_common()
    if "--count" not in sys.argv:
        for r, n in rows:
            print(f"{n:>4}  CR {r:<9} e.g. {where[r]}")
    print(f"# {len(rows)} cited rule numbers absent from {os.path.basename(rules_file)} ({sum(cited.values())} citations)")
    if "--max" in sys.argv:
        cap = int(sys.argv[sys.argv.index("--max") + 1])
        if len(rows) > cap:
            sys.exit(1)


if __name__ == "__main__":
    main()
