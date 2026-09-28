#!/usr/bin/env python3
"""Printed "Enchanted / Equipped creature gets +N/+M" vs the definition's
bonus (exploratory). The bonus rides `EquipBonus { power, toughness }` for
Auras and Equipment alike (or a static `PumpPT` on the host); a row names a
printed +N/+M that appears in neither shape.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_aura_bonus.py /tmp/all.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

PRINTED = re.compile(r"^(?:Enchanted|Equipped) creature gets ([+-]\d+)/([+-]\d+)([^.\n]*)", re.M)
SCALED = re.compile(r"for each|where X|as long as|\bif\b|unless|equal to", re.I)


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
        for m in PRINTED.finditer(oracle_of(card)):
            if SCALED.search(m.group(3)):
                continue
            p, t = int(m.group(1)), int(m.group(2))
            bonus = re.search(r"EquipBonus \{ power: (-?\d+), toughness: (-?\d+)", dbg)
            ok = bonus and (int(bonus.group(1)), int(bonus.group(2))) == (p, t)
            ok = ok or re.search(rf"power: Const\({p}\), toughness: Const\({t}\)", dbg) or \
                re.search(rf"power: {p}, toughness: {t}\b", dbg)
            if not ok:
                have = bonus.group(0)[12:] if bonus else "-"
                out.append(f"{name}\t{p:+}/{t:+}\thave {have}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
