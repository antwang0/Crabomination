#!/usr/bin/env python3
"""Printed "create … token with <keyword>" vs the token definitions a card
mints (exploratory). A row names a keyword the oracle gives a created token
that appears nowhere in the card's definition.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_token_keywords.py /tmp/all.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

KW = {"flying": "Flying", "haste": "Haste", "vigilance": "Vigilance", "trample": "Trample",
      "lifelink": "Lifelink", "deathtouch": "Deathtouch", "reach": "Reach", "menace": "Menace",
      "first strike": "FirstStrike", "double strike": "DoubleStrike", "defender": "Defender",
      "indestructible": "Indestructible", "hexproof": "Hexproof", "prowess": "Prowess",
      "ward": "Ward", "flash": "Flash", "infect": "Infect", "shroud": "Shroud"}
TOKKW = re.compile(r"TokenDefinition \{ name: \"[^\"]*\", power: -?\d+, toughness: -?\d+, keywords: \[([^\]]*)\]")
PRINTED = re.compile(r"\bcreates? [^.]*?\btokens? with ([^.\"“]*)", re.I)


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
        tok = " ".join(TOKKW.findall(dbg))
        if "TokenDefinition" not in dbg:
            continue
        for m in PRINTED.finditer(oracle_of(card)):
            words = m.group(1).lower()
            words = re.split(r"\b(?:where|that|for each|equal)\b", words)[0]
            for k, v in KW.items():
                if re.search(r"\b" + k + r"\b", words) and not re.search(r"\b" + v + r"\b", tok):
                    out.append(f"{name}\t{k}\t{m.group(0)[:90]}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
