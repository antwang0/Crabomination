#!/usr/bin/env python3
"""One ability reading target slot 0 through two DIFFERENT filters.

`target_filtered(F)` is always slot 0, so a body written as
`Seq[Destroy(target_filtered(Creature)), Destroy(target_filtered(Land))]`
asks for ONE target that is both — Plague Spores ("target nonblack creature
and target land") was uncastable, and Bond of Passion's "2 damage to any
other target" hit the creature it had just stolen. A second printed target
belongs on slot 1 (`Selector::TargetFiltered { slot: 1, .. }`).

Rows: abilities with >= 2 distinct slot-0 filters and no slot >= 1 (modal,
kicker-swapped, reflexive and granted bodies skipped), on cards
whose oracle prints two or more "target"s. Narrowing reads of the same
target ("if it's a creature") are the expected false positives; triage.

    cargo build -p crabomination --bin dump_cards
    target/debug/dump_cards --effects > /tmp/ef.tsv
    python3 scripts/audit_slot_reuse.py /tmp/ef.tsv [--gate]
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")


def slot_filters(dbg, slot):
    out = []
    for m in re.finditer(r"TargetFiltered \{ slot: %d, filter: " % slot, dbg):
        i = j = m.end()
        depth = 0
        while j < len(dbg):
            ch = dbg[j]
            if ch in "({[":
                depth += 1
            elif ch in ")}]":
                if depth == 0:
                    break
                depth -= 1
            j += 1
        out.append(dbg[i:j].rstrip(" "))
    return out


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not args:
        sys.exit(__doc__)
    cache = {k: v for k, v in json.load(open(CACHE)).items() if isinstance(v, dict)}
    by_name = {}
    for k, v in cache.items():
        by_name.setdefault(k.lower(), v)
        by_name.setdefault(k.split(" // ")[0].lower(), v)
    rows = []
    for line in open(args[0]):
        name, where, dbg = line.rstrip("\n").split("\t", 2)
        # A nested token / granted ability has slots of its own.
        dbg = re.sub(r"definition: [^\t]*", "", dbg) if "CreateToken" in dbg else dbg
        f0 = set(slot_filters(dbg, 0))
        if len(f0) < 2 or re.search(r"slot: [1-9]|Target\([1-9]\)|ApplyToTargets|additional|ChooseMode|ChooseN|Spree|Escalate|Entwine|Kicked|ReflexiveTrigger|GrantActivated|GrantTriggered|GainActivatedAbility|GainTriggeredAbility|CreateToken", dbg):
            continue
        card = by_name.get(name.lower())
        if not card:
            continue
        oracle = card.get("oracle_text") or "\n".join(f.get("oracle_text", "") for f in card.get("card_faces") or [])
        oracle = re.sub(r"\([^)]*\)", "", oracle)
        if len(re.findall(r"\btarget\b(?!s|ed)", oracle, re.I)) < 2:
            continue
        rows.append(f"{name:40} {where:11} {' | '.join(sorted(f[:40] for f in f0))}")
    for r in rows:
        print(r)
    print(f"{len(rows)} rows")
    if "--gate" in sys.argv and rows:
        sys.exit(1)


main()
