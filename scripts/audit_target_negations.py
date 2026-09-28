#!/usr/bin/env python3
"""Printed "target non<X> …" filters vs the definition (exploratory). A row
names a negated qualifier ("nonland", "noncreature", "nonblack",
"non-Human", "nontoken", "nonbasic", "nonlegendary") printed on a target
whose negation appears nowhere in the definition — the target is wider than
the card.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_target_negations.py /tmp/all.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

PRINTED = re.compile(r"\btarget ((?:non-?[A-Za-z]+,? )+)", re.I)
NEED = {
    "nonland": r"Nonland|Not\(Land\)|Not\(HasCardType\(Land\)\)",
    "noncreature": r"Noncreature|NonCreature|Not\(Creature\)|Not\(HasCardType\(Creature\)\)",
    "nonartifact": r"Nonartifact|Not\(Artifact\)|Not\(HasCardType\(Artifact\)\)",
    "nontoken": r"NotToken|Not\(IsToken\)",
    "nonbasic": r"Nonbasic|Not\(IsBasicLand\)|Not\(Basic|Not\(HasSupertype\(Basic\)\)",
    "nonlegendary": r"Not\(HasSupertype\(Legendary\)\)|Nonlegendary|NonLegendary",
    "nonwhite": r"Not\(HasColor\(White\)\)|NonWhite", "nonblue": r"Not\(HasColor\(Blue\)\)|NonBlue",
    "nonblack": r"Not\(HasColor\(Black\)\)|NonBlack", "nonred": r"Not\(HasColor\(Red\)\)|NonRed",
    "nongreen": r"Not\(HasColor\(Green\)\)|NonGreen",
    "nonenchantment": r"Not\(Enchantment\)|Not\(HasCardType\(Enchantment\)\)",
    "nonplaneswalker": r"Not\(Planeswalker\)|Not\(HasCardType\(Planeswalker\)\)",
    "nonattacking": r"Not\(IsAttacking\)|NotAttacking",
    "noncommander": r"Not\(IsCommander\)|NotCommander|Noncommander",
}


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
            for word in re.findall(r"non-?([A-Za-z]+)", m.group(1)):
                key = "non" + word.lower()
                need = NEED.get(key)
                if need is None:
                    need = r"Not\(HasCreatureType\(" + word.capitalize() + r"\)\)|Not" + word.capitalize()
                if not re.search(need, dbg):
                    out.append(f"{name}\t{key}\t{m.group(0)[:70]}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
