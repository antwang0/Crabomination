#!/usr/bin/env python3
"""A clause that must see cards the SAME resolution just moved, modelled as a
cast-time target.

"Mill three cards, **then** you may return a creature or land card from your
graveyard to your hand" is one effect resolving in printed order (CR 608.2).
The return is not targeted — the word "target" does not appear — and it looks
at the graveyard *after* the mill has filled it. Modelled as
`target_filtered(… .from_your_graveyard())` it is a **cast-time** choice (CR
601.2c) made against the graveyard as it was *before* the spell resolved, so
over an empty graveyard Grapple with the Past was a pure three-card self-mill:
the graveyard went 0 → 4 and the hand never moved.

The idiom that works is Corpse Churn's — `Effect::MayDo` around an
`Effect::Move` whose `what` is a `Selector::one_of(CardsInZone { Graveyard })`,
which is resolved when the arm runs and therefore sees what the mill put
there.

    python3 scripts/audit_resolution_order.py           # the rows
    python3 scripts/audit_resolution_order.py --count   # just the totals
    python3 scripts/audit_resolution_order.py --gate    # exit 1 on any row

⚠ This is the **sharp** half of a wider class. 69 non-Aura cards print no
"target" anywhere and still declare a target slot, most of them the karoo
shape ("when this land enters, return a land you control to its owner's
hand" — Azorius Chancery and its nine siblings, Kor Skyfisher, the Planeshift
gainlands). Those diverge only when the chosen permanent leaves in response,
or when it has shroud and so cannot be targeted at all; they are a census in
ENGINE_BACKLOG rather than rows here, because nothing about them is provably
wrong on a still board. What this file gates is the subset where the target
provably cannot see what the clause is about.

Reads `scripts/.scryfall_cache.json` (offline) and `audit_dropped_may`'s body
reader.
"""

import importlib.util
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CATALOG = os.path.join(ROOT, "crabomination_catalog", "src")
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")

_spec = importlib.util.spec_from_file_location(
    "audit_dropped_may", os.path.join(ROOT, "scripts", "audit_dropped_may.py")
)
_adm = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_adm)

# The clause fills a zone during its own resolution…
FILLS = re.compile(r"\bmills? \w+ cards?\b|\bsurveil \d|\bexplores?\b", re.I)
# …and then reaches into that zone for something to move.
REACHES = re.compile(
    r"then\b[^.]*\b(return|put)\b[^.]*from your graveyard", re.I
)
# A cast-time target slot pointed at the same zone.
TARGETS_GRAVEYARD = (
    "from_your_graveyard",
    "InYourGraveyard",
    "InGraveyard",
)

ALLOWLIST = {}


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    lower = {k.lower(): v for k, v in cache.items() if isinstance(v, dict)}
    for k, v in list(cache.items()):
        if isinstance(v, dict):
            lower.setdefault(k.split(" // ")[0].lower(), v)

    hits, checked = [], 0
    seen_fns = {}
    for dirpath, _, files in os.walk(CATALOG):
        for f in files:
            if not f.endswith(".rs"):
                continue
            for fn, name, body, path in _adm.defs_in(os.path.join(dirpath, f)):
                card = lower.get(name.lower())
                if card is None:
                    continue
                oracle = card.get("oracle_text") or ""
                if not FILLS.search(oracle):
                    continue
                m = REACHES.search(oracle)
                if not m:
                    continue
                # A printed "target" in the reaching half is a real target.
                if "target" in re.sub(r"\([^)]*\)", " ", m.group(0)).lower():
                    continue
                checked += 1
                seen_fns[fn] = True
                if fn in ALLOWLIST:
                    continue
                if not any(t in body for t in TARGETS_GRAVEYARD):
                    continue
                if "target_filtered" not in body and "TargetFiltered" not in body:
                    continue
                hits.append(
                    (name, fn, os.path.relpath(path, ROOT), m.group(0).strip()[:88])
                )

    hits.sort()
    if "--count" not in sys.argv:
        print("# a post-fill clause whose choice is a cast-time target:")
        for name, fn, path, clause in hits:
            print(f"{path}::{fn}\n    {name}: “{clause}”")
    print(
        f"# {len(hits)} open, {checked} cards whose clause fills a zone and then "
        f"reaches into it without printing \"target\""
    )
    stale = [k for k in ALLOWLIST if not seen_fns.get(k)]
    for k in stale:
        print(f"# allowlist entry `{k}` names no card here any more", file=sys.stderr)
    if "--gate" in sys.argv and (hits or stale):
        sys.exit(1)


if __name__ == "__main__":
    main()
