#!/usr/bin/env python3
"""Printed target qualifiers vs the definition's filters (exploratory):
"target attacking / blocking / attacking or blocking / tapped / untapped
creature", "target creature with flying / power N or …", "target legendary
…". A row names a qualifier with no matching requirement anywhere in the
definition — the target is wider than the card.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_target_qualifiers.py /tmp/all.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

RULES = [
    (r"\btarget attacking or blocking creature", r"IsAttackingOrBlocking|AttackingOrBlocking|Or\(IsAttacking, IsBlocking\)|Or\(IsBlocking, IsAttacking\)"),
    (r"\btarget attacking creature", r"IsAttacking|Attacking"),
    (r"\btarget blocking creature", r"IsBlocking|Blocking"),
    (r"\btarget tapped (?:creature|artifact|land|permanent)", r"\bTapped\b|IsTapped"),
    (r"\btarget untapped (?:creature|artifact|land|permanent)", r"Untapped|Not\(Tapped\)"),
    (r"\btarget creature with flying\b", r"HasKeyword\(Flying\)|Flying"),
    (r"\btarget legendary\b", r"HasSupertype\(Legendary\)|Legendary"),
    (r"\btarget multicolored\b", r"Multicolored"),
    (r"\btarget monocolored\b", r"Monocolored"),
    (r"\btarget colorless\b", r"Colorless"),
    (r"\btarget (?:face-down|facedown)\b", r"FaceDown"),
    (r"\btarget enchanted creature\b", r"IsEnchanted|Enchanted"),
    (r"\btarget equipped creature\b", r"IsEquipped|Equipped"),
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
        text = re.sub(r"[\"“][^\"”]*[\"”]", "", oracle_of(card))
        seen = set()
        for phrase, need in RULES:
            m = re.search(phrase, text, re.I)
            if m and not any(m.start() >= s and m.end() <= e for s, e in seen):
                seen.add((m.start(), m.end()))
                if not re.search(need, dbg):
                    out.append(f"{name}\t{m.group(0)}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
