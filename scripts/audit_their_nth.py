#!/usr/bin/env python3
""""Whenever an opponent / a player casts or draws THEIR Nth … each turn" vs
a trigger that counts once for the table (CR 121.2, 603.4).

The count is the acting player's own: at N > 2 every opponent has a first
spell and a second card. A row names such a card whose opponent- or
any-player-scoped cast/draw trigger is `once_per_turn` (one fire for all
opponents together — Faerie Mastermind, Erayo's Essence) or reads the
ACTIVE player's tally (The Unagi of Kyoshi Island). `--gate` exits 1 on
any row.

    target/debug/dump_cards --grep "" > /tmp/all.tsv
    python3 scripts/audit_their_nth.py /tmp/all.tsv [--pod NAMES] [--gate]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

PHRASE = re.compile(r"\b(an opponent|a player|each opponent)\b[^.]*\btheir (first|second|third)\b[^.]*\beach turn", re.I)
KINDS = ("SpellCast", "CardDrawn", "NthCardDrawnThisTurn", "FirstCardDrawnThisTurn")


def triggers(dbg):
    out, j = [], 0
    while (k := dbg.find("TriggeredAbility { event: EventSpec { kind: ", j)) >= 0:
        depth, m = 0, k
        while m < len(dbg):
            if dbg[m] in "({[":
                depth += 1
            elif dbg[m] in ")}]":
                depth -= 1
                if depth == 0:
                    break
            m += 1
        out.append(dbg[k:m + 1])
        j = m + 1
    return out


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
        m = PHRASE.search(oracle_of(card))
        if not m:
            continue
        for t in triggers(dbg):
            kind = re.match(r"TriggeredAbility \{ event: EventSpec \{ kind: (\w+)", t).group(1)
            scope = re.search(r"scope: (\w+)", t).group(1)
            if kind not in KINDS or scope not in ("OpponentControl", "AnyPlayer"):
                continue
            if "once_per_turn: true" in t or "(ActivePlayer)" in t:
                out.append(f"{name}\t{kind} {scope}\t{m.group(0)[:90]}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)
    if "--gate" in sys.argv and out:
        sys.exit(1)


if __name__ == "__main__":
    main()
