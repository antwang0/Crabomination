#!/usr/bin/env python3
"""Printed target count ("up to two target", "three target") vs the spell's
declared target slots.

Non-modal instants and sorceries only: the printed count is the sum over each
"target" word of the number word before it ("up to two target creatures" is
2, a bare "target creature" is 1). The definition's count is the spell
effect's `target_slot_count`; "up to" also wants the slots optional
(`min_targets_in_mode` 0).

    cargo build -p crabomination --bin dump_cards
    target/debug/dump_cards --targets > /tmp/tg.tsv
    python3 scripts/audit_target_counts.py /tmp/tg.tsv [--gate]
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")
WORDS = {"one": 1, "two": 2, "three": 3, "four": 4, "five": 5, "six": 6, "seven": 7, "eight": 8,
         "nine": 9, "ten": 10}


def printed_count(oracle):
    """(slots, any_up_to) or None when the text is out of this scan's reach."""
    if re.search(r"any number of|\bX target|choose one|choose two|choose any|•|where X|for each|"
                 r"\bdivided\b|each of (?:them|those)|that target|same target|new target", oracle, re.I):
        return None
    # Keep the spell's own text: a cycling / flashback / kicker line is
    # another ability with its own targets.
    lines = [l for l in oracle.split("\n") if not re.match(
        r"(?:When you cycle|Cycling|\w+cycling|Flashback|Buyback|Kicker|Multikicker|Overload|Entwine|Splice|"
        r"Replicate|Escape|Madness|Morph|Channel|Forecast|Retrace|Jump-start|Aftermath|Suspend|Rebound)", l)]
    if any(re.search(r"kicked|was cast from|overloaded|instead", l) for l in lines):
        return None
    oracle = "\n".join(lines)
    total = 0
    players = 0
    up_to = True
    for m in re.finditer(r"(?:(up to )?(\w+) )?(?:other |another |additional )?\btarget\b(?!s|ed)", oracle, re.I):
        word = (m.group(2) or "").lower()
        if re.match(r"\s*(?:player|opponent)", oracle[m.end():]):
            players += 1
        if word in WORDS:
            total += WORDS[word]
            up_to &= bool(m.group(1))
        else:
            total += 1
            up_to = False
    return total, players, up_to and total > 0


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not args:
        sys.exit(__doc__)
    cache = {k: v for k, v in json.load(open(CACHE)).items() if isinstance(v, dict)}
    by_name = {}
    for k, v in cache.items():
        by_name.setdefault(k.lower(), v)
    rows = []
    for line in open(args[0]):
        name, slots, mn, modal = line.rstrip("\n").split("\t")
        card = by_name.get(name.lower())
        if not card or modal == "1" or card.get("card_faces"):
            continue
        oracle = re.sub(r"\([^)]*\)", "", card.get("oracle_text") or "")
        got = printed_count(oracle)
        if not got:
            continue
        want, players, up_to = got
        slots = int(slots)
        if slots and want != slots and want - players != slots:
            rows.append(f"{name:40} slots {slots} vs printed {want}")
        elif up_to and mn not in ("0",):
            rows.append(f"{name:40} printed up to {want}; min {mn}")
    for r in rows:
        print(r)
    print(f"{len(rows)} rows")
    if "--gate" in sys.argv and rows:
        sys.exit(1)


main()
