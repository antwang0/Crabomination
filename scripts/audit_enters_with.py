#!/usr/bin/env python3
"""Printed "this enters with N <kind> counters" vs `enters_with_counters`.

Reads a card's OWN `enters_with_counters` (not a nested token's) against the
Scryfall oracle: a wrong counter kind, or a constant amount that differs from
the printed number. Non-constant values (X, "for each") are checked only for
not being a bare constant where the oracle prints X.

    cargo build -p crabomination --bin dump_cards
    target/debug/dump_cards --enters-with > /tmp/ew.tsv
    python3 scripts/audit_enters_with.py /tmp/ew.tsv [--gate]
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")
WORDS = {"a": 1, "an": 1, "one": 1, "two": 2, "three": 3, "four": 4, "five": 5, "six": 6,
         "seven": 7, "eight": 8, "nine": 9, "ten": 10, "eleven": 11, "twelve": 12}
KINDS = {"+1/+1": "PlusOnePlusOne", "-1/-1": "MinusOneMinusOne", "+1/+0": "PlusOnePlusZero",
         "+0/+1": "PlusZeroPlusOne", "+2/+2": "PlusTwoPlusTwo"}


def kind_name(k):
    return KINDS.get(k) or "".join(w.capitalize() for w in re.split(r"[\s-]+", k))


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
        name, dbg = line.rstrip("\n").split("\t", 1)
        card = by_name.get(name.lower())
        if not card:
            continue
        faces = card.get("card_faces") or []
        oracle = card.get("oracle_text") or "\n".join(f.get("oracle_text", "") for f in faces)
        oracle = re.sub(r"\([^)]*\)", "", oracle)
        short = name.split(",")[0]
        subj = r"(?:this [a-z]+|" + re.escape(name) + "|" + re.escape(short) + ")"
        m = re.search(subj + r" enters(?: the battlefield)?(?: tapped)?(?: and)? with (\w+)(?: additional)? "
                      r"([+\-0-9/]+|[a-z]+(?: [a-z]+)?) counters? on it", oracle, re.I)
        if not m:
            rows.append(f"{name:40} def {dbg[:70]}; oracle has no self enters-with")
            continue
        word, kind = m.group(1).lower(), m.group(2).lower()
        mk = re.match(r"\((\w+), (.*)\)$", dbg)
        if not mk:
            continue
        dkind, dval = mk.group(1), mk.group(2)
        want_kind = kind_name(kind)
        if want_kind != dkind and not dkind.startswith(want_kind):
            rows.append(f"{name:40} kind {dkind} vs printed {kind!r}")
        if word in WORDS:
            if dval != f"Const({WORDS[word]})":
                rows.append(f"{name:40} amount {dval[:60]} vs printed {word!r}")
        elif word == "x" and dval.startswith("Const("):
            rows.append(f"{name:40} amount {dval} vs printed X")
    for r in rows:
        print(r)
    print(f"{len(rows)} rows")
    if "--gate" in sys.argv and rows:
        sys.exit(1)


main()
