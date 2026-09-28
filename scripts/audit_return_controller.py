#!/usr/bin/env python3
"""Printed "onto the battlefield under your / its owner's control" vs the
definition's `ZoneDest::Battlefield { controller }` (CR 110.2, 400.7).

A reanimation spell that reaches any graveyard ("under your control") written
with `OwnerOf` hands an opponent's creature back to them; a flicker of any
creature ("under its owner's control") written with `You` steals it. A row
names a card that prints ONE of the two and whose every battlefield move uses
the other controller. Exploratory: a card that prints both is skipped.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_return_controller.py /tmp/all.tsv [--pod NAMES] [--gate]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

YOURS = re.compile(r"battlefield under your control", re.I)
OWNERS = re.compile(r"battlefield under (its|their) owners?'s? control", re.I)
DEST = re.compile(r"Battlefield \{ controller: (\w+)")
# The printed object can only be yours already ("target creature you own",
# "from your graveyard", "this card"), so either controller reads right. NOT
# "a creature you control" or "return it": a creature you control may be
# stolen (Ephemerate, Felidar Guardian, Luminous Broodmoth returned it to the
# thief), and the owner-direction regex never matched "owner's" before
# 2026-09-28, so this list had never been exercised.
ONLY_YOURS = re.compile(
    r"from your graveyard|you own|your hand|your library|return (this|~)|"
    r"exile (this|~)|put (this)",
    re.I)


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
        yours, owners = bool(YOURS.search(text)), bool(OWNERS.search(text))
        if yours == owners:
            continue
        dests = set(DEST.findall(dbg))
        if not dests:
            continue
        if yours and dests <= {"OwnerOf", "OwnerOfMoved"} and "You" not in dests:
            out.append(f"{name}\tyours->owner\t{sorted(dests)}")
        elif owners and dests == {"You"} and not ONLY_YOURS.search(text):
            out.append(f"{name}\towner->you\t{sorted(dests)}")
    print("\n".join(sorted(out)))
    print(f"# {len(out)} rows", file=sys.stderr)
    if "--gate" in sys.argv and out:
        sys.exit(1)


if __name__ == "__main__":
    main()
