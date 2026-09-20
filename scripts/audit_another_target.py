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
  `SelectionRequirement::OtherThanTargetSlot(n)` says that, the sibling of
  `SameControllerAsTargetSlot`: both read `GameState::target_slots_scratch`,
  which the cast and activation validators stamp with the whole chosen
  vector. `already_picked` in `auto_targets_for_effect_all_slots_kicked` was
  only ever a *preference*, so before this the duplicate was merely unlikely
  rather than illegal; that walk now runs `cross_slot_targets_ok` over the
  slots it has filled.

    python3 scripts/audit_another_target.py           # both columns
    python3 scripts/audit_another_target.py --count   # just the totals
    python3 scripts/audit_another_target.py --gate    # exit 1 on any open row

⚠ The split is by **paragraph**, not by sentence: Drooling Groodion's
"Another target creature gets -2/-2" is its own sentence and the slot it
refers to is in the one before it, on the same printed line.

⚠ Two allowlists carry the rows that are neither, each with its reason, and
`--gate` fails on a **stale** entry as well as on a new finding. Jackdaw
Savior's "another" is other than the creature that *died* and Blade of Shared
Souls' is other than the creature the Equipment is attached to — neither is
the source; Fiendish Panda's non-Bear filter already excludes it; Etched
Slith's clause is unmodelled and belongs to `audit_incomplete`. On the
slot-relative side, a pair of slots that cannot name one object ("you
control" against "an opponent controls") costs nothing.

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

# Rows where "another" is satisfied without the atom, or where the atom is the
# wrong question. Keyed on the factory ident, which no body can shadow; a
# stale entry fails `--gate` the same way a new finding does.
SOURCE_ALLOWLIST = {
    "fiendish_panda": (
        "the filter already excludes it: the Panda is a Bear Demon and the "
        "clause is 'another target **non-Bear** creature card'"
    ),
    "jackdaw_savior": (
        "'another' is other than the creature that **died**, not other than "
        "the Savior — the two coincide only when the Savior itself dies, and "
        "no requirement names the trigger source"
    ),
    "blade_of_shared_souls": (
        "'another' is other than the creature the Equipment is **attached "
        "to**, which is not the source either"
    ),
    "atzocan_archer": (
        "narrowed to 'a creature you don't control' on purpose — a mandatory "
        "fight slot would otherwise hand the bot its own creature; the "
        "narrowing never allows an illegal play"
    ),
    "nessian_wilds_ravager": "same narrowing, same reason",
    "etched_slith": (
        "the whole 'when you do, remove a counter from another target "
        "permanent or opponent' clause is unmodelled — `audit_incomplete`'s "
        "row, not this one"
    ),
}

# Slot-relative rows whose two slots cannot name one object anyway, so the
# printed "another" costs nothing. Same keying and the same staleness rule.
SLOT_ALLOWLIST = {
    "comet_storm": (
        "one `ApplyToTargets` instance, so CR 115.3's within-one-instance "
        "distinctness already forbids the repeat (`distinct_target_count`)"
    ),
    "biomantic_mastery": (
        "both slots are `PlayerRef::Target(n)` inside a `Value`, so no slot "
        "filter exists to hang a requirement on"
    ),
    "pit_fight": "slot 0 is 'you control' and slot 1 'an opponent controls'",
    "domri_rade": "the -2 is modelled as you-control vs opponent-controls",
    "ulvenwald_tracker": "same disjoint pair",
    "stiltzkin_moogle_merchant": (
        "slot 0 is a **player** and slot 1 a permanent, so they are never the "
        "same object; the printed 'another' is source-relative and the filter "
        "carries it"
    ),
}


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
    seen_fns = {}
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
                seen_fns[fn] = True
                if (
                    source
                    and fn not in SOURCE_ALLOWLIST
                    and not any(t in body for t in OTHER_TOKENS)
                ):
                    src_hits.append((name, fn, rel, source[0][:90]))
                if fn in SLOT_ALLOWLIST or "OtherThanTargetSlot" in body:
                    continue
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
        f"{len(slot_hits)} slot-relative still open, {checked} cards printing "
        f"\"another target\""
    )
    stale = [
        k
        for k in list(SOURCE_ALLOWLIST) + list(SLOT_ALLOWLIST)
        if not seen_fns.get(k)
    ]
    for k in stale:
        print(f"# allowlist entry `{k}` names no card here any more", file=sys.stderr)
    if "--gate" in sys.argv and (src_hits or slot_hits or stale):
        sys.exit(1)


if __name__ == "__main__":
    main()
