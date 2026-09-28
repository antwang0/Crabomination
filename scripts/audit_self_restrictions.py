#!/usr/bin/env python3
"""Printed self-restrictions / evasion sentences vs the definition
(exploratory). "~ can't block." needs `CantBlock`, "~ can't be blocked."
`Unblockable`, "~ attacks each combat if able." `MustAttack`, and so on. A
row names a sentence the oracle prints about the card itself whose keyword
(or an equivalent) appears nowhere in the definition.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_self_restrictions.py /tmp/all.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

SELF = r"^(?:~|this creature|this vehicle)"
RULES = [
    (SELF + r" can't block\.", r"CantBlock\b|CantAttackOrBlock"),
    (SELF + r" can't attack\.", r"CantAttack\b|Defender|CantAttackOrBlock"),
    (SELF + r" can't attack or block\.", r"CantAttackOrBlock|CantAttack\b"),
    (SELF + r" can't be blocked\.", r"Unblockable|CantBeBlocked\b"),
    (SELF + r" attacks each combat if able\.", r"MustAttack"),
    (SELF + r" blocks each combat if able\.", r"MustBlock"),
    (SELF + r" attacks or blocks each combat if able\.", r"MustAttackOrBlock|MustAttack"),
    (SELF + r" can block an additional creature each combat\.", r"ExtraBlock|CanBlockAdditional|BlockAdditional"),
    (SELF + r" can block any number of creatures\.", r"BlockAnyNumber|CanBlockAnyNumber"),
    (SELF + r" doesn't untap during your untap step\.", r"DoesntUntap|PreventUntap"),
    (SELF + r" can't be blocked except by two or more creatures\.", r"Menace|CantBeBlockedExceptByTwo"),
    (SELF + r" can block only creatures with flying\.", r"CanBlockOnlyFlying"),
    (SELF + r" can't be countered\.", r"CantBeCountered|cant_be_countered: true"),
    (SELF + r" enters tapped\.", r"EntersTapped|enters_tapped: true|Tapped"),
]


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
        short = name.split(",")[0]
        text = oracle_of(card).replace(name, "~").replace(short, "~")
        for para in text.split("\n"):
            para = para.strip()
            for phrase, need in RULES:
                if re.search(phrase, para, re.I) and not re.search(need, dbg):
                    out.append(f"{name}\t{need.split('|')[0]}\t{para[:80]}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
