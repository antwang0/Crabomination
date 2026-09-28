#!/usr/bin/env python3
"""Printed "for each <permanent> <side>" counts vs the definition's count side.

A row names a card whose oracle counts permanents on one side and whose
definition holds no count on that side:

- `you`: "for each creature you control" with no `ControlledByYou` count.
- `opp`: "for each creature an opponent controls / you don't control" with no
  opponent-side count (the count includes your own board).
- `all`: "for each creature on the battlefield" / "for each other creature"
  whose every permanent count is `ControlledByYou` (the count drops opponents').

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_for_each.py /tmp/all.tsv [--pod NAMES] [--gate]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

PHRASE = re.compile(r"\bfor each ([^.,;:\n]*)", re.I)
PERM = re.compile(r"\b(creature|artifact|land|enchantment|planeswalker|permanent|token|"
                  r"forest|island|swamp|mountain|plains|gate|equipment|aura|vehicle)s?\b", re.I)
SKIP = re.compile(r"\b(card|cards|graveyard|hand|library|exile|counter|counters|mana|time|times|"
                  r"color|colors|life|damage|spell|spells|type|types|name|opponent|player|"
                  r"players|opponents who|way|kind|basic land type|among)\b|\{", re.I)
YOU = re.compile(r"\byou control\b", re.I)
OPP = re.compile(r"\b(an opponent controls|your opponents control|opponents control|"
                 r"you don't control|that player controls|defending player controls)\b", re.I)
ALL = re.compile(r"\bon the battlefield\b|^(other|attacking|blocking|tapped|untapped) ", re.I)
OPENERS = ("CountOf(", "CountMatching {", "TotalManaValueOf(", "ForEach { selector:",
           "PowerOf(", "ToughnessOf(")
F_YOU = re.compile(r"ControlledByYou\b|who: You\b|ControlledBy \{ who: You")
F_OPP = re.compile(r"Opponent|Not\(ControlledByYou\)|who: Target|TargetPlayer|TriggerPlayer|"
                   r"ControlledByTarget|NotControlledByYou|DefendingPlayer")


def fragments(dbg):
    out = []
    for op in OPENERS:
        start = 0
        while (i := dbg.find(op, start)) >= 0:
            depth, j = 0, i
            while j < len(dbg):
                c = dbg[j]
                if c in "({[":
                    depth += 1
                elif c in ")}]":
                    depth -= 1
                    if depth == 0:
                        break
                j += 1
            out.append(dbg[i:j + 1])
            start = i + 1
    return [f for f in out if "EachPermanent" in f or "ControlledBy" in f]


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
        frags = fragments(dbg)
        for m in PHRASE.finditer(oracle_of(card)):
            ph = m.group(1)
            if not PERM.search(ph) or SKIP.search(ph):
                continue
            if YOU.search(ph):
                if not F_YOU.search(dbg):
                    out.append(f"{name}\tyou\t{ph[:80]}")
            elif OPP.search(ph):
                if not any(F_OPP.search(f) for f in frags) and not F_OPP.search(dbg):
                    out.append(f"{name}\topp\t{ph[:80]}")
            elif ALL.search(ph):
                if frags and all(F_YOU.search(f) for f in frags):
                    out.append(f"{name}\tall\t{ph[:80]}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)
    if "--gate" in sys.argv and out:
        sys.exit(1)


if __name__ == "__main__":
    main()
