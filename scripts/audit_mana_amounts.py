#!/usr/bin/env python3
"""Printed "{T}: Add {C}{C}" / "Add {G}{G}" / "Add two mana of any one color"
vs the definition's `AddMana` payloads (exploratory). A row names a printed
fixed amount the definition never produces: the multiset of printed symbols
for `Colors([...])` / `Colorless(Const(n))`, or the count for "N mana of any
one color".

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_mana_amounts.py /tmp/all.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

COL = {"W": "White", "U": "Blue", "B": "Black", "R": "Red", "G": "Green"}
NUM = {"one": 1, "two": 2, "three": 3, "four": 4, "five": 5, "six": 6, "seven": 7}
SYMS = re.compile(r"\bAdd ((?:\{[WUBRGC]\})+)(?=[.,]| or | for | to )")
ANY = re.compile(r"\bAdd (one|two|three|four|five) mana of any one color\b")


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
        colors = [sorted(re.findall(r"\w+", x)) for x in re.findall(r"pool: Colors\(\[([^\]]*)\]\)", dbg)]
        colorless = [int(n) for n in re.findall(r"pool: Colorless\(Const\((\d+)\)\)", dbg)]
        anyone = [int(n) for n in re.findall(r"pool: AnyOneColor\(Const\((\d+)\)\)", dbg)]
        for m in SYMS.finditer(text):
            syms = re.findall(r"\{(\w)\}", m.group(1))
            if len(set(syms)) == 1 and syms[0] == "C":
                if len(syms) not in colorless and not re.search(r"Colorless\(", dbg):
                    out.append(f"{name}\t{m.group(1)}\tno Colorless")
                elif colorless and len(syms) not in colorless:
                    out.append(f"{name}\t{m.group(1)}\thave Colorless {colorless}")
            elif "C" not in syms:
                want = sorted(COL[s] for s in syms)
                if colors and want not in colors and len(syms) > 1:
                    out.append(f"{name}\t{m.group(1)}\thave {colors}")
        for m in ANY.finditer(text):
            n = NUM[m.group(1)]
            if anyone and n not in anyone:
                out.append(f"{name}\t{m.group(0)}\thave AnyOneColor {anyone}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
