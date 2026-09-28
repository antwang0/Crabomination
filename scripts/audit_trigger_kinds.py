#!/usr/bin/env python3
"""Printed compound trigger events vs the definition's `EventKind`s
(exploratory). "Whenever ~ attacks or blocks" needs both `Attacks` and
`Blocks`; "enters or attacks" both `EntersBattlefield` and `Attacks`; "deals
combat damage to a player" a combat-damage kind; "dies" a death kind. A row
names a printed half the definition never listens for.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_trigger_kinds.py /tmp/all.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

# (printed phrase, [alternatives of EventKind regexes that must each be present])
RULES = [
    (r"\battacks or blocks\b", [r"kind: Attacks\b|AttacksOrBlocks", r"kind: Blocks\b|AttacksOrBlocks"]),
    (r"\bblocks or becomes blocked\b", [r"kind: Blocks\b", r"kind: BecomesBlocked\b"]),
    (r"\benters or attacks\b", [r"kind: EntersBattlefield\b", r"kind: Attacks\b"]),
    (r"\benters or dies\b", [r"kind: EntersBattlefield\b", r"Died\b"]),
    (r"\benters and whenever\b", [r"kind: EntersBattlefield\b"]),
    (r"\bwhenever [^.]*? deals combat damage to a player\b", [r"CombatDamage|DealsDamage"]),
    (r"\bwhenever [^.]*? becomes tapped\b", [r"Tapped|BecomesTapped"]),
    (r"\bwhenever [^.]*? is dealt damage\b", [r"DealtDamage|IsDealtDamage|Enrage"]),
    (r"\bwhenever you cycle\b|\bwhenever you cycle or discard\b", [r"Cycled|Discarded"]),
    (r"\bwhenever you gain life\b", [r"LifeGained"]),
    (r"\bwhenever you sacrifice\b", [r"Sacrificed"]),
    (r"\bwhenever [^.]*? becomes the target\b", [r"BecameTarget|Targeted"]),
    (r"\bwhenever [^.]*? leaves the battlefield\b", [r"LeavesBattlefield|LeftBattlefield|Leaves"]),
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
        text = oracle_of(card).replace(name, "~")
        # Granted abilities in quotes belong to another object; skip them.
        text = re.sub(r"[\"“][^\"”]*[\"”]", "", text)
        for phrase, needs in RULES:
            m = re.search(phrase, text, re.I)
            if not m:
                continue
            for need in needs:
                if not re.search(need, dbg):
                    out.append(f"{name}\t{need.split('|')[0]}\t{m.group(0)[:80]}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
