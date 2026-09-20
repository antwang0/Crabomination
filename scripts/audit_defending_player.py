#!/usr/bin/env python3
"""Printed "defending player controls" modelled as "an opponent controls".

CR 506.2 — the defending player is the **one** player being attacked (or the
controller of the attacked planeswalker or battle). In a duel that is the
only opponent, so `ControlledByOpponent` is exact and every such card passed
its two-player test. At three seats or more it is one seat of several, and
the clause quietly reaches the whole table: Cyclops Gladiator fought a
bystander's creature, provoke untapped a blocker on a seat it was not
attacking, and Blaze of Glory — which carried no controller filter at all —
could hand the block to a creature its own caster controlled.

`SelectionRequirement::ControlledByDefendingPlayer` is the atom. It shares
the source walk with `OwnedByDefendingPlayer` (falling through an Aura or
Equipment source to the host that is attacking) and then falls back to the
combat's single defender, because a clause can print "defending player" on
something that is not the attacker: an **instant** cast during combat (Yare,
Blaze of Glory) or a trigger that fires off *another* creature's attack
(Nazahn). Two different defenders in one combat leaves it ambiguous and the
filter takes nothing — a narrowing cannot make an illegal play.

    python3 scripts/audit_defending_player.py           # the rows
    python3 scripts/audit_defending_player.py --count   # just the totals
    python3 scripts/audit_defending_player.py --gate    # exit 1 on any row

⚠ Landwalk is cut before the match: "can't be blocked as long as defending
player controls an Island" is `Keyword::Islandwalk`, decided by the combat
engine against the attack's own defender, and there are 149 of them.

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

# A clause that *selects* a permanent that player controls, which is the half
# a filter has to carry.
SELECTS = re.compile(
    r"(target|each|all|a|an|up to \w+)\s+[\w\s/+\-']*?"
    r"\b(creature|permanent|artifact|land|blocking creature)s?\s+"
    r"defending player controls",
    re.I,
)
# Decided by the combat engine, not by a filter.
LANDWALK = re.compile(r"can'?t be blocked as long as defending player controls", re.I)

# The atom, or the mass-selection form that names the seat directly
# (`Selector::ControlledBy { who: PlayerRef::DefendingPlayer, .. }`),
# which is exact for an "each creature …" clause and needs no filter.
TOKENS = ("ControlledByDefendingPlayer", "PlayerRef::DefendingPlayer")
# Shared helpers that carry the atom for every card built out of them.
HELPERS = ("provoke()", "on_attack_goad()")

# Rows the atom is the wrong question for. `--gate` fails on a stale entry
# the same way it fails on a new finding.
ALLOWLIST = {
    "kusari_gama": (
        "its `DealsCombatDamageToCreature` body resolves **after** the combat "
        "teardown, so `attack_for` finds nothing and neither the atom nor "
        "`PlayerRef::DefendingPlayer` can answer. The trigger binds the "
        "damaged **blocker** as slot 0 and that creature's controller IS the "
        "defending player, so `ControllerOf(Target(0))` is exact at any seat "
        "count — a different reference to the same seat, not a looser one"
    ),
    "tromokratis": (
        "'can't be blocked unless all creatures defending player controls "
        "block it' is `Keyword::CantBeBlockedUnlessAllBlock`, and CR 509.1b's "
        "loop in `combat.rs` already scopes it with `defender_for(atk.target)` "
        "— per attack, so it is exact at any seat count"
    ),
}


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    lower = {k.lower(): v for k, v in cache.items() if isinstance(v, dict)}
    for k, v in list(cache.items()):
        if isinstance(v, dict):
            lower.setdefault(k.split(" // ")[0].lower(), v)

    hits, checked, exempt, walked = [], 0, 0, 0
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
                if "defending player" not in oracle.lower():
                    continue
                walked += 1
                clause = SELECTS.search(LANDWALK.sub("", oracle))
                if not clause:
                    continue
                checked += 1
                seen_fns[fn] = True
                if fn in ALLOWLIST:
                    exempt += 1
                    continue
                if any(t in body for t in TOKENS) or any(h in body for h in HELPERS):
                    continue
                hits.append((name, fn, os.path.relpath(path, ROOT), clause.group(0)))

    hits.sort()
    if "--count" not in sys.argv:
        print("# printed “defending player controls”, filter does not say it:")
        for name, fn, path, clause in hits:
            print(f"{path}::{fn}\n    {name}: “{clause.strip()}”")
    print(
        f"# {len(hits)} open, {checked} cards whose clause selects a permanent "
        f"the defending player controls ({walked} print the words at all), "
        f"{exempt} exempt because the COMBAT ENGINE owns the seat"
    )
    stale = [k for k in ALLOWLIST if not seen_fns.get(k)]
    for k in stale:
        print(f"# allowlist entry `{k}` names no card here any more", file=sys.stderr)
    if "--gate" in sys.argv and (hits or stale):
        sys.exit(1)


if __name__ == "__main__":
    main()
