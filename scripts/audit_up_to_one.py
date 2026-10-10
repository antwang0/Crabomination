#!/usr/bin/env python3
"""Printed "up to one target <X>" vs a definition with no optional-slot marker
(exploratory). Modelled as a required slot, a hostile "up to one" falls back
to its controller's own permanent or graveyard card when no opponent has one
(Loran destroyed its own Sol Ring) — `declines_own_side_pick` only spares an
optional slot. A row is a card none of whose abilities carries a marker;
friendly filters ("you control") and opponent-only filters are tagged — an
"either" row is the one to read. A *trailing* spell slot is already omittable
at cast; an activated ability's is not (Path to the World Tree).

    target/debug/dump_cards --effects > /tmp/eff.tsv
    python3 scripts/audit_up_to_one.py /tmp/eff.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

PRINTED = re.compile(r"\bup to one (?:other )?target ([^.;:]*)", re.I)
MARKERS = re.compile(r"OptionalTargets|CapTargetsAt|MayDo|TargetsExactlyX|optional: true|UpTo|min: 0|min_targets: 0")


# Reviewed and left required, with the reason. A row here is not printed.
REVIEWED = {
    "Mistmeadow Vanisher": "a flicker: your own ETB creature is a fine pick (Kasla's seat)",
    "Phelia, Exuberant Shepherd": "a flicker that pays off on your own permanent",
    "Talon Gates of Madara": "phasing out your own creature is protection",
    "Dreamdew Entrancer": "draws two when it stuns your own creature",
    "Diregraf Scavenger": "your own creature card drains too",
    "Ardyn, the Usurper": "copies the exiled creature card: yours is a fine pick",
    "Persistent Constrictor": "filter is the upkeep player's creatures",
    "Relic Crush": "min 1 of 2 already (\"target ... and up to one other\")",
    "Return to Dust": "min 1 of 2 already",
    "Fiery Annihilation": "the Equipment is attached to the first target",
    "Combat Tutorial": "a trailing spell slot is already omittable at cast",
    "Vibrant Outburst": "a trailing spell slot is already omittable at cast",
    "Cost of Brilliance": "a trailing spell slot is already omittable at cast",
    "Render Speechless": "a trailing spell slot is already omittable at cast",
    "Twisted Fealty": "a trailing spell slot is already omittable at cast",
}


def side(phrase):
    p = phrase.lower()
    if re.search(r"\byou (control|own)\b|\byour graveyard\b", p):
        return "friendly"
    if re.search(r"opponent|you don't control|an opponent's", p):
        return "hostile-only"
    return "either"


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    pod = None
    if "--pod" in sys.argv:
        pod = {ln.strip() for ln in open(sys.argv[sys.argv.index("--pod") + 1], encoding="utf-8")}
    effects = {}
    for line in open(sys.argv[1], encoding="utf-8"):
        parts = line.rstrip("\n").split("\t", 2)
        if len(parts) == 3:
            effects.setdefault(parts[0], []).append(parts[2])
    out = []
    for name, dbgs in effects.items():
        card = cache.get(name)
        if not isinstance(card, dict) or (pod is not None and name not in pod) or name in REVIEWED:
            continue
        hits = [m.group(1)[:70] for m in PRINTED.finditer(oracle_of(card))]
        if not hits or any(MARKERS.search(d) for d in dbgs):
            continue
        for h in hits:
            out.append(f"{side(h)}\t{name}\t{h}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
