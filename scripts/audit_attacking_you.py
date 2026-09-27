#!/usr/bin/env python3
"""Printed "attacking you" modelled as plain "attacking".

CR 506.3 — a creature attacking *you* is one of the attackers whose defender
is you (or, where printed, a planeswalker you control). In a duel every
attacker is, so `SelectionRequirement::IsAttacking` passed each card's
two-player test; at three seats it reaches creatures attacking someone else
(Soul Snare exiled them, Watchdog shrank them, Hunting Kavu took them).

The atoms are `IsAttackingYou` / `IsAttackingYouOrYourPlaneswalker`; a card
whose seat is owned by the combat engine or a per-player count is listed in
`ALLOWLIST` with the reason.

    python3 scripts/audit_attacking_you.py           # the rows
    python3 scripts/audit_attacking_you.py --gate    # exit 1 on any row

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

CLAUSE = re.compile(r"attacking you\b", re.I)
TOKENS = ("IsAttackingYou", "CreaturesAttackingPlayer", "AttackedDefenderWithCountAtLeast")

TAX = "an attack tax, charged per creature attacking the controller by the combat engine (`StaticEffect::AttackTaxToController`)"
ALLOWLIST = {
    "ghostly_prison": TAX,
    "propaganda": TAX,
    "windborn_muse": TAX,
    "elephant_grass": TAX,
    "koskun_falls": TAX,
    "collective_restraint": TAX,
    "mirror_match": "`Effect::CopyAttackersAsBlockers` filters the attacks on the caster and their planeswalkers itself",
    "defensive_formation": "`StaticEffect::ControllerAssignsAttackersCombatDamage` is read per attack by the damage step",
}


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    lower = {k.lower(): v for k, v in cache.items() if isinstance(v, dict)}
    for k, v in list(cache.items()):
        if isinstance(v, dict):
            lower.setdefault(k.split(" // ")[0].lower(), v)
    hits, seen = [], set()
    for dirpath, _, files in os.walk(CATALOG):
        for f in files:
            if not f.endswith(".rs"):
                continue
            for fn, name, body, path in _adm.defs_in(os.path.join(dirpath, f)):
                card = lower.get(name.lower())
                oracle = (card or {}).get("oracle_text") or ""
                if not CLAUSE.search(oracle):
                    continue
                seen.add(fn)
                if fn in ALLOWLIST or any(t in body for t in TOKENS):
                    continue
                clause = re.search(r"[^.]*attacking you[^.]*", oracle, re.I).group(0).strip()
                hits.append((name, fn, os.path.relpath(path, ROOT), clause))
    hits.sort()
    for name, fn, path, clause in hits:
        print(f"{path}::{fn}\n    {name}: “{clause}”")
    print(f"# {len(hits)} open, {len(seen)} cards print “attacking you”, {len(ALLOWLIST)} allowlisted")
    stale = [k for k in ALLOWLIST if k not in seen]
    for k in stale:
        print(f"# allowlist entry `{k}` names no card here any more", file=sys.stderr)
    if "--gate" in sys.argv and (hits or stale):
        sys.exit(1)


if __name__ == "__main__":
    main()
