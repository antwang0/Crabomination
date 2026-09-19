#!/usr/bin/env python3
"""Cards that print "As [this] enters …" and model it as an ETB TRIGGER.

CR 614.12 — "Some replacement effects modify how a permanent enters the
battlefield." An "As this creature enters, choose a color" or "As this land
enters, you may pay 2 life. If you don't, it enters tapped" is one of those:
the choice is made *before* the permanent is on the battlefield (CR 614.12a),
and the permanent is never on the battlefield in the un-chosen state.

An `EventKind::EntersBattlefield` trigger is not that. It puts the permanent
onto the battlefield first, then goes on the stack, and its controller gets
priority before it resolves. Three consequences, all reachable:

* a conditional tapper leaves the land untapped with the trigger on the
  stack, so its controller can tap it for mana it should never have made;
* a static keyed on the choice ("creatures of the chosen type get -1/-1")
  reads an unchosen value across at least one state-based-action check;
* CR 614.12's own example — a token copy of a card with an as-enters choice
  makes that choice as the token is created — has no trigger to fire.

What the class needs is a replacement that can ASK. The engine applies
`StaticEffect::EntersTapped` / `EntersTappedUnless` inside
`GameState::apply_enters_tapped_replacement`, which reads a `Predicate` — it
can test the game state but cannot put a question to a player. That is the
one missing primitive; every bucket below is downstream of it.

Columns, by what the printed clause asks for:

* **pay-life-or-tapped** — the ten shocklands and their descendants. The
  narrowest bucket and the one with the highest EDHREC ranks.
* **choose-a-type** / **choose-a-colour** / **choose-a-name** — a value
  chosen as it enters that the permanent's own static abilities then read.
* **other** — everything else, including the two that put counters or a
  copy on as they enter.

⚠ The **closed** sub-class is not a column here because it no longer has
rows: "As this land enters, you may reveal a [X] card from your hand. If you
don't, this land enters tapped" (fifteen cards) is a
`StaticEffect::EntersTappedUnless` over a hand predicate, which is a
replacement and reads as one. `--gate` fails if any of them regresses to a
trigger, which is what this script is for until the primitive lands.

⚠ Reader limitation, the same one every name-keyed script in this directory
has: a card whose body delegates to a private helper in another module is
read through the helper's *call*, not its body. The helper names below are
the ones the walk knows about.

Run: `python3 scripts/audit_as_enters.py [--gate]`
"""

import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CATALOG = ROOT / "crabomination_catalog/src"
CACHE = ROOT / "scripts/.scryfall_cache.json"

FN_RE = re.compile(r"pub fn (\w+)\(\) -> CardDefinition \{")
NAME_RE = re.compile(r'name:\s*"((?:[^"\\]|\\.)*)"')
HELPER_NAME = re.compile(r'\(\s*"((?:[^"\\]|\\.)*)"')

# The printed clause, at the start of a line: "As this land enters, …",
# "As Iona enters, …". Not "enters with", not "enters the battlefield under".
AS_ENTERS = re.compile(r"^As (this|[^,\n]{1,40}) enters", re.M)

# An ETB trigger, written out or reached through one of the helpers that
# builds one.
TRIGGER = re.compile(
    r"EventKind::EntersBattlefield"
    r"|\betb\(|\betb_tap\(|etb_tap_then_"
    r"|shockland_pay_two_or_tap\(|fastland_etb_conditional_tap\("
    r"|pay_three_or_tapped\("
)
# A replacement, written out or reached through one of its helpers.
REPLACEMENT = re.compile(
    r"StaticEffect::(EntersTapped|EntersTappedUnless|EntersWith\w*|AsEnters\w*)"
    r"|enters_tapped_unless\(|reveal_or_tapped_land\(|land_type_reveal_land\("
)

# The reveal cycle, closed 2026-09-19. `--gate` fails if any of these comes
# back as a trigger.
REVEAL_CYCLE = [
    "Port Town", "Choked Estuary", "Foreboding Ruins", "Game Trail",
    "Fortified Village", "Frostboil Snarl", "Furycalm Snarl",
    "Necroblossom Snarl", "Shineshadow Snarl", "Vineglimmer Snarl",
    "Ancient Amphitheater", "Auntie's Hovel", "Secluded Glen",
    "Wanderwine Hub", "Gilt-Leaf Palace",
]


def bucket(clause):
    if re.search(r"you may pay .*\blife\b", clause, re.I):
        return "pay-life-or-tapped"
    if re.search(r"choose a color", clause, re.I):
        return "choose-a-colour"
    if re.search(r"choose a \w*\s?type\b", clause, re.I):
        return "choose-a-type"
    if re.search(r"choose .*\bname\b", clause, re.I):
        return "choose-a-name"
    return "other"


def card_bodies():
    """{printed name: (factory, file, source body)} for every card factory."""
    out = {}
    for path in sorted(CATALOG.rglob("*.rs")):
        src = path.read_text()
        fns = [(m.start(), m.group(1)) for m in FN_RE.finditer(src)]
        for i, (start, ident) in enumerate(fns):
            stop = fns[i + 1][0] if i + 1 < len(fns) else len(src)
            body = src[start:stop]
            m = NAME_RE.search(body) or HELPER_NAME.search(body)
            if m:
                name = m.group(1).replace('\\"', '"')
                out.setdefault(name, (ident, path.name, body))
    return out


def oracle(entry):
    text = entry.get("oracle_text") or ""
    for face in entry.get("card_faces") or []:
        if isinstance(face, dict):
            text += "\n" + (face.get("oracle_text") or "")
    return text


def main():
    gate = "--gate" in sys.argv[1:]
    cache = json.loads(CACHE.read_text())
    bodies = card_bodies()

    buckets, regressed = {}, []
    for name, (ident, fname, body) in bodies.items():
        entry = cache.get(name)
        if not isinstance(entry, dict):
            continue
        text = oracle(entry)
        if not AS_ENTERS.search(text):
            continue
        if REPLACEMENT.search(body) or not TRIGGER.search(body):
            continue
        clause = next(
            (line for line in text.split("\n") if AS_ENTERS.match(line)), text
        )
        rank = entry.get("edhrec_rank")
        buckets.setdefault(bucket(clause), []).append(
            (rank if rank is not None else 10**9, name, fname, ident, clause)
        )
        if name in REVEAL_CYCLE:
            regressed.append(name)

    total = sum(len(v) for v in buckets.values())
    for key in sorted(buckets, key=lambda k: -len(buckets[k])):
        rows = sorted(buckets[key])
        print(f"\n== {key}: {len(rows)}")
        for rank, name, fname, ident, clause in rows:
            r = rank if rank < 10**9 else "-"
            print(f"  {r:>7}  {name}  [{fname}::{ident}]")
            print(f"           {clause[:120]}")

    missing = [n for n in REVEAL_CYCLE if n not in bodies]
    print(
        f"\n{total} cards print an 'As … enters' replacement and ship it as an "
        f"EntersBattlefield trigger. The reveal cycle is "
        f"{len(REVEAL_CYCLE) - len(regressed) - len(missing)}/{len(REVEAL_CYCLE)} "
        f"on the replacement ({len(regressed)} regressed, {len(missing)} not in "
        f"the catalog)."
    )
    if not gate:
        return 0
    for name in regressed:
        print(f"REGRESSED: {name} is back on a trigger", file=sys.stderr)
    for name in missing:
        print(f"MISSING: {name} left the catalog", file=sys.stderr)
    return 1 if regressed or missing else 0


if __name__ == "__main__":
    sys.exit(main())
