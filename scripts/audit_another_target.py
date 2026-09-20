#!/usr/bin/env python3
"""Printed "**another** target X" whose filter does not say "another".

"Another" is a restriction on the target, and the engine spells the common
half of it with `SelectionRequirement::OtherThanSource`. A filter without it
lets Fiend Hunter exile itself, Flickerwisp blink itself and Heliod give
itself lifelink — and the auto-targeter has no preference against the source,
so it does.

The word means one of two things and they need different fixes, so the census
splits them by where the *other* object comes from:

* **source-relative** — nothing else in the ability is targeted, so "another"
  can only mean "not this permanent". `OtherThanSource` is the whole fix, and
  these are catalog edits.
* **slot-relative** — the ability already targeted something earlier in the
  same paragraph ("target creature you control fights **another** target
  creature"), so "another" means "not the object chosen for that other slot".
  No `SelectionRequirement` can say that today: the requirement walker is
  handed one candidate and never the picks already made. `already_picked` in
  `auto_targets_for_effect_all_slots_kicked` is a *preference*, not a
  constraint, so the duplicate is merely unlikely rather than illegal.

    python3 scripts/audit_another_target.py           # both columns
    python3 scripts/audit_another_target.py --count   # just the totals
    python3 scripts/audit_another_target.py --gate    # exit 1 on a source-relative row

⚠ The split is by **paragraph**, not by sentence: Drooling Groodion's
"Another target creature gets -2/-2" is its own sentence and the slot it
refers to is in the one before it, on the same printed line.

⚠ Two rows are neither, and both are listed under source-relative with the
approximation named: Jackdaw Savior's "another" is other than the creature
that *died*, and Blade of Shared Souls' is other than the creature the
Equipment is attached to. `OtherThanSource` is right for those whenever the
source is that object and harmlessly wrong otherwise.

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

# Every way a definition can say "not this one".
OTHER_TOKENS = ("OtherThanSource", "other_than_source")


def split_class(oracle):
    """`(source_rows, slot_rows)` — the paragraphs naming "another target"."""
    source, slot = [], []
    for para in oracle.split("\n"):
        i = para.find("another target")
        if i < 0:
            continue
        # A `target` before it on the same printed line is the other slot.
        (slot if "target" in para[:i] else source).append(para.strip())
    return source, slot


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    lower = {k.lower(): v for k, v in cache.items() if isinstance(v, dict)}
    for k, v in list(cache.items()):
        if isinstance(v, dict):
            lower.setdefault(k.split(" // ")[0].lower(), v)

    src_hits, slot_hits, checked = [], [], 0
    for dirpath, _, files in os.walk(CATALOG):
        for f in files:
            if not f.endswith(".rs"):
                continue
            for fn, name, body, path in _adm.defs_in(os.path.join(dirpath, f)):
                card = lower.get(name.lower())
                if card is None:
                    continue
                oracle = re.sub(r"\([^)]*\)", " ", card.get("oracle_text") or "").lower()
                if "another target" not in oracle:
                    continue
                checked += 1
                source, slot = split_class(oracle)
                rel = os.path.relpath(path, ROOT)
                if source and not any(t in body for t in OTHER_TOKENS):
                    src_hits.append((name, fn, rel, source[0][:90]))
                # A slot-relative row is a finding whatever the body says:
                # nothing in the filter language can express it yet.
                for p in slot:
                    slot_hits.append((name, fn, rel, p[:90]))

    src_hits.sort()
    slot_hits.sort()
    if "--count" not in sys.argv:
        print("# source-relative — `OtherThanSource` is the fix:")
        for name, fn, path, para in src_hits:
            print(f"{path}::{fn}\n    {name}: “{para}”")
        print("\n# slot-relative — needs a cross-slot distinctness primitive:")
        for name, fn, path, para in slot_hits:
            print(f"{path}::{fn}\n    {name}: “{para}”")
    print(
        f"# {len(src_hits)} source-relative without `OtherThanSource`, "
        f"{len(slot_hits)} slot-relative (no primitive), {checked} cards printing "
        f"\"another target\""
    )
    if "--gate" in sys.argv and src_hits:
        sys.exit(1)


if __name__ == "__main__":
    main()
