#!/usr/bin/env python3
"""Printed self-triggers vs the definition's `EventKind`s (exploratory).

"When ~ enters", "When ~ dies", "Whenever ~ attacks", "When ~ is turned face
up", "When you cast this spell", … each need a trigger of that kind (any
scope). A row names a printed self-trigger whose kind appears nowhere in the
definition — the whole ability is missing.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_self_triggers.py /tmp/all.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

RULES = [
    (r"^(?:[^—\n]*— )?When(?:ever)? ~ enters\b", r"EntersBattlefield|ETB|Enters"),
    (r"^(?:[^—\n]*— )?When(?:ever)? ~ dies\b", r"Died|Dies"),
    (r"^(?:[^—\n]*— )?Whenever ~ attacks\b", r"Attacks|Attack"),
    (r"^(?:[^—\n]*— )?Whenever ~ blocks\b", r"Blocks"),
    (r"^(?:[^—\n]*— )?Whenever ~ becomes blocked\b", r"BecomesBlocked"),
    (r"^(?:[^—\n]*— )?When(?:ever)? ~ is turned face up\b", r"TurnedFaceUp"),
    (r"^(?:[^—\n]*— )?When you cast this spell\b", r"SpellCast|CastSelf|OnCast"),
    (r"^(?:[^—\n]*— )?When(?:ever)? ~ leaves the battlefield\b", r"Leaves|LeftBattlefield"),
    (r"^(?:[^—\n]*— )?Whenever ~ deals (?:combat )?damage\b", r"Damage"),
    (r"^(?:[^—\n]*— )?Whenever ~ becomes tapped\b", r"Tapped"),
    (r"^(?:[^—\n]*— )?When(?:ever)? ~ is put into (?:a|your) graveyard\b", r"Graveyard|Died"),
    (r"^(?:[^—\n]*— )?When(?:ever)? you cycle ~\b", r"Cycled|Cycle"),
    (r"^(?:[^—\n]*— )?When(?:ever)? ~ becomes the target\b", r"BecameTarget|Targeted"),
    (r"^(?:[^—\n]*— )?When(?:ever)? ~ transforms into\b", r"Transformed"),
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
        text = re.sub(r"\bthis (creature|artifact|enchantment|permanent|land|Vehicle|Aura|Equipment|card|Saga|planeswalker)\b",
                      "~", text, flags=re.I)
        kinds = " ".join(re.findall(r"kind: (\w+)", dbg))
        for para in text.split("\n"):
            for phrase, need in RULES:
                if re.search(phrase, para.strip(), re.I) and not re.search(need, kinds + " " + dbg[:0]):
                    out.append(f"{name}\t{need.split('|')[0]}\t{para.strip()[:80]}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
