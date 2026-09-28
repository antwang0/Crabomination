#!/usr/bin/env python3
"""Printed "… you control have / get +N/+N and have <keyword>" static grants
vs the definition (exploratory). A row names a granted keyword that appears
nowhere in the definition as `keyword: <KW>` or inside a `keywords: [...]`
list.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_keyword_grants.py /tmp/all.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

KW = {"flying": "Flying", "haste": "Haste", "vigilance": "Vigilance", "trample": "Trample",
      "lifelink": "Lifelink", "deathtouch": "Deathtouch", "reach": "Reach", "menace": "Menace",
      "first strike": "FirstStrike", "double strike": "DoubleStrike", "indestructible": "Indestructible",
      "hexproof": "Hexproof", "shroud": "Shroud", "intimidate": "Intimidate", "fear": "Fear",
      "infect": "Infect", "wither": "Wither", "prowess": "Prowess", "flash": "Flash"}
PRINTED = re.compile(r"\byou control (?:get [+-]\d+/[+-]\d+ and )?(?:have|gain) ([a-z ,]+?)(?:\.|$| until| as long| for)",
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
        text = re.sub(r"[\"“][^\"”]*[\"”]", "", oracle_of(card))
        for m in PRINTED.finditer(text):
            for word in re.split(r",\s*|\s+and\s+", m.group(1).strip()):
                word = word.strip().lower()
                kw = KW.get(word)
                if kw and not re.search(r"keyword: " + kw + r"\b|keywords: \[[^\]]*\b" + kw + r"\b", dbg):
                    out.append(f"{name}\t{kw}\t{m.group(0)[:90]}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
