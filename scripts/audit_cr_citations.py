#!/usr/bin/env python3
"""CR citations in Rust sources that name no rule in the shipped CR text.

A comment or test that cites "CR 701.15g" for regeneration was right under
an older numbering (701.15 is goad now); nothing flags the drift. This walks
every `CR NNN.N[x]` in the Rust tree and reports those absent from
`crabomination/MagicCompRules_*.txt` (the newest file), most-cited first.

    python3 scripts/audit_cr_citations.py            # the rows
    python3 scripts/audit_cr_citations.py --count    # the total only
    python3 scripts/audit_cr_citations.py --max N    # exit 1 above N rows, or on any misnamed keyword

A number that exists can still name the wrong rule. The second report
catches the commonest form: "Keyword (CR 70x.N)" where the name is exactly a
CR 701/702 header filed under another number ("Myriad (CR 702.115)" named
Ingest).
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
    rules, header, by_name = set(), {}, {}
    for line in open(rules_file, encoding="utf-8", errors="replace"):
        m = re.match(r"^([1-9][0-9]{2}\.[0-9]+[a-z]?)\.? ", line)
        if m:
            rules.add(m.group(1))
        h = re.match(r"^(70[12]\.[0-9]+)\. (.+)$", line.strip())
        if h:
            header[h.group(1)] = h.group(2).strip().lower()
            by_name.setdefault(h.group(2).strip().lower(), h.group(1))
    named = re.compile(r"\b([A-Z][a-z]+(?: [a-z]+)?) \(CR (70[12]\.[0-9]+)[a-z]?\)")
    # "CR 701.N — <verb>": the keyword-action verb right after the citation is
    # another 701 header's, and the cited one's appears nowhere on the line
    # ("CR 701.17a — a player can sacrifice": 701.17 is Mill, sacrifice 701.21).
    def forms(h):
        root = h[:-1] if h.endswith("e") else h
        return {h, h + "s", h + "d", h + "ed", h + "es", root + "ing", root + "ed"}
    # Verbs too common in plain prose to name a rule ("an activated ability",
    # "double-faced", "create a token") never count as the *other* keyword.
    common = {"activate", "cast", "create", "double", "play", "reveal"}
    roots = {n: forms(h) for n, h in header.items() if n.startswith("701.") and " " not in h}
    stem = {n: h[:5] for n, h in header.items() if n.startswith("701.")}
    verb_after = re.compile(r"CR (701\.[0-9]+)[a-z]? (?:—|-|:) ([^.;)]{0,50})")
    misnamed = []
    cited, where = Counter(), {}
    for d in DIRS:
        for dp, _, files in os.walk(os.path.join(ROOT, d)):
            for f in files:
                if not f.endswith(".rs"):
                    continue
                path = os.path.join(dp, f)
                for i, line in enumerate(open(path, encoding="utf-8", errors="replace"), 1):
                    for name, num in named.findall(line):
                        right = by_name.get(name.lower())
                        if right and right != num and header.get(num) != name.lower():
                            misnamed.append(f"{os.path.relpath(path, ROOT)}:{i}  {name} cites {num}, the rule is {right}")
                    for num, after in verb_after.findall(line):
                        words = re.findall(r"[a-z]+", after.lower())
                        own = roots.get(num)
                        hits = {n for n, fs in roots.items() if n != num and header[n] not in common and any(w in fs for w in words)}
                        mine = stem.get(num, "~")
                        if own and hits and not any(w.startswith(mine) for w in re.findall(r"[a-z]+", line.lower())):
                            right = ", ".join(f"{n} ({header[n]})" for n in sorted(hits))
                            misnamed.append(f"{os.path.relpath(path, ROOT)}:{i}  CR {num} ({header[num]}) reads as {right}")
                    for r in CITE.findall(line):
                        if r not in rules:
                            cited[r] += 1
                            where.setdefault(r, f"{os.path.relpath(path, ROOT)}:{i}")
    rows = cited.most_common()
    if "--count" not in sys.argv:
        for r, n in rows:
            print(f"{n:>4}  CR {r:<9} e.g. {where[r]}")
    print(f"# {len(rows)} cited rule numbers absent from {os.path.basename(rules_file)} ({sum(cited.values())} citations)")
    if "--count" not in sys.argv:
        for m in misnamed:
            print(f"  misnamed  {m}")
    print(f"# {len(misnamed)} keyword citations filed under another keyword's number")
    if "--max" in sys.argv:
        cap = int(sys.argv[sys.argv.index("--max") + 1])
        if len(rows) > cap or misnamed:
            sys.exit(1)


if __name__ == "__main__":
    main()
