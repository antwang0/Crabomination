#!/usr/bin/env python3
"""Printed "target opponent / target player <verb>" vs a definition that hits
EVERY opponent (CR 115.1 — one chosen player, not each).

At two seats the two readings agree, so nothing but a pod shows the bug:
the Thunder Junction deserts pinged every opponent for each land drop,
Highway Robber drained the table, Clackbridge Troll handed three Goats to
each opponent. A row is a card whose oracle says "target opponent|player"
followed by one of the verbs below, never says "each opponent|player"
with that verb, and whose definition routes that verb through
`EachOpponent` without any player target. `--gate` exits 1 on any row.

    target/debug/dump_cards --grep "" > /tmp/all.tsv
    python3 scripts/audit_target_vs_each.py /tmp/all.tsv [--gate]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

# Printed verb -> the definition shape that reads "each opponent".
VERBS = {
    "loses": r"(?:Drain|LoseLife) \{ (?:from|who): Player\(EachOpponent\)",
    "discards": r"Discard[A-Za-z]* \{ who: (?:Player\()?EachOpponent",
    "mills": r"Mill[A-Za-z]* \{ who: (?:Player\()?EachOpponent",
    "sacrifices": r"Sacrifice[A-Za-z]* \{ (?:who|player): (?:Player\()?EachOpponent",
    "creates": r"CreateToken \{ who: EachOpponent",
}
DAMAGE = re.compile(r"damage to target (?:player|opponent)(?: or planeswalker)?\b")
DAMAGE_DEF = re.compile(r"DealDamage \{ to: Player\(EachOpponent\)")
# Documented residuals (INCOMPLETE_CARDS).
ALLOWLIST = {"Jace, Vryn's Prodigy"}


def main():
    path = sys.argv[1]
    gate = "--gate" in sys.argv
    cache = json.load(open(CACHE))
    rows = []
    for line in open(path):
        name, _, d = line.rstrip("\n").partition("\t")
        if name in ALLOWLIST:
            continue
        entry = cache.get(name)
        text = oracle_of(entry) if isinstance(entry, dict) else ""
        if not text:
            continue
        for verb, shape in VERBS.items():
            if (
                re.search(rf"target (?:opponent|player) {verb}\b", text)
                and not re.search(rf"each (?:opponent|player) {verb}\b", text)
                and re.search(shape, d)
            ):
                rows.append(f"{name}\ttarget … {verb} reads every opponent")
        if (
            DAMAGE.search(text)
            and not re.search(r"damage to each (?:opponent|player)", text)
            and DAMAGE_DEF.search(d)
            and "DealDamage { to: Target" not in d
        ):
            rows.append(f"{name}\tdamage to target player/opponent reads every opponent")
    for r in sorted(set(rows)):
        print(r)
    print(f"# {len(set(rows))} rows")
    if gate and rows:
        sys.exit(1)


if __name__ == "__main__":
    main()
