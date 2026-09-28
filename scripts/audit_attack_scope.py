#!/usr/bin/env python3
"""Printed "attacks you" wording vs the defender-side attack scope (CR 506.3).

Three scopes ride the defender-side walk in `combat.rs`:

* `ControllerAttackedByOpponent` — "attacks you OR a planeswalker you
  control" (it fires for both);
* `ControllerAttackedDirectlyByOpponent` — a bare "attacks you" (an attack
  on your planeswalker is not an attack on you);
* `ControllerPlaneswalkerAttackedByOpponent` — "attacks a planeswalker you
  control" only.

A row is a card whose wording and scopes disagree: a bare "attacks you" on
the you-or-planeswalker scope (Hissing Miasma fired on a planeswalker
attack), a planeswalker-only wording on either player scope (Oath of Kaya
fired on attacks on you), or both the you-or-planeswalker and the
planeswalker-only scope on one wording (Revenge of Ravens drained twice).
`--gate` exits 1 on any row.

    target/debug/dump_cards --grep "" > /tmp/all.tsv
    python3 scripts/audit_attack_scope.py /tmp/all.tsv [--gate]
"""

import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

BOTH = re.compile(r"attacks? you,? (?:or|and/or) (?:a|one or more) planeswalkers?|attacking you and/or planeswalkers", re.I)
BARE = re.compile(r"attacks? you\b|attacking you\b", re.I)
PW_ONLY = re.compile(r"attacks? (?:a|one or more) planeswalkers? you control", re.I)
# Cards whose wording is neither shape ("if you're the defending player",
# "if they attacked you and/or a planeswalker you control") or that belong to
# a frozen pool.
ALLOWLIST = {
    "Kazuul, Tyrant of the Cliffs",  # defending player: you or your planeswalker
    "Ever-Watching Threshold",  # "attacked you and/or a planeswalker"
    "Everett K. Ross, Hapless Attaché",
    "Coveted Jewel",  # cube pool (frozen); bare "attack you" on the wide scope
}


def main():
    import json

    path = sys.argv[1]
    gate = "--gate" in sys.argv
    cache = json.load(open(CACHE))
    rows = []
    for line in open(path):
        name, _, d = line.rstrip("\n").partition("\t")
        if name in ALLOWLIST:
            continue
        wide = "ControllerAttackedByOpponent" in d
        direct = "ControllerAttackedDirectlyByOpponent" in d
        pw = "ControllerPlaneswalkerAttackedByOpponent" in d
        if not (wide or direct or pw):
            continue
        text = oracle_of(cache.get(name) or {})
        both = bool(BOTH.search(text))
        bare = bool(BARE.search(BOTH.sub("", text)))
        pw_only = bool(PW_ONLY.search(text)) and not both
        why = None
        if wide and pw and both:
            why = "you-or-planeswalker wording on two scopes (fires twice)"
        elif wide and not both and bare:
            why = "bare \"attacks you\" on the you-or-planeswalker scope"
        elif (wide or direct) and pw_only and not bare and not both:
            why = "planeswalker-only wording on a player scope"
        elif direct and both and not bare:
            why = "you-or-planeswalker wording on the direct scope"
        if why:
            rows.append(f"{name}\t{why}")
    for r in sorted(rows):
        print(r)
    print(f"# {len(rows)} rows")
    if gate and rows:
        sys.exit(1)


if __name__ == "__main__":
    main()
