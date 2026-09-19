#!/usr/bin/env python3
"""Cards that print "enters tapped" and model it as an ETB TRIGGER.

The sibling census to `audit_as_enters.py`, over the clause that does **not**
open with "As". CR 614.1c — an effect that modifies how a permanent enters
the battlefield is a replacement; "This land enters tapped", and "This land
enters tapped unless [a fact about your board]", are both that. The permanent
is never on the battlefield untapped.

An `EventKind::EntersBattlefield` trigger is not that. It puts the land on the
battlefield **untapped**, puts the trigger on the stack, and hands its
controller priority — who can tap it for mana it should never have made, and
whose opponents see an untapped land when they decide whether to respond.

The engine has had the right shape the whole time and one half of the tree
uses it: `StaticEffect::EntersTapped` and `StaticEffect::EntersTappedUnless`
are applied by `GameState::apply_enters_tapped_replacement` inside the
battlefield hop. So every row below is a **catalog** edit — no primitive is
missing, which is what separates this class from `audit_as_enters`'s
pay-life-or-tapped bucket.

⚠ The comparison that makes the point is inside one file family:
`bfz::lands::battle_land` ("enters tapped unless you control two or more
basic lands") has always been the replacement, while `decks::lands::slow_land`
("unless you control two or more other lands") shipped the trigger until
2026-09-19. Same sentence, same cycle shape, two different answers.

Columns:

* **conditional** — "enters tapped unless …". The narrower one, and the one
  whose window is exploitable in one turn rather than academic.
* **unconditional** — "This land enters tapped." Mostly reached through the
  shared `etb_tap()` helper, so this column is one helper and its call sites
  rather than N hand-written bodies.
* **invented** — the mirror: a card that SHIPS the replacement and prints no
  "enters tapped" clause at all. A one-directional ratchet is how a
  conversion pass quietly hands a card a downside it does not have, so
  `--gate` fails on this column too.

**The class is CLOSED: this census read 83 when it was written and reads 0.**
`--gate` therefore fails on ANY row, not only on a regression of the two
cycles it names — a new card that prints "enters tapped" and ships a trigger
is a defect, not a backlog item.

⚠ Reader limitation, shared with every name-keyed script here: a card whose
body delegates to a private helper in another module is read through the
helper's *call*, so the helper names below are the ones the walk knows about.

Run: `python3 scripts/audit_enters_tapped.py [--gate]`
"""

import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CATALOG = ROOT / "crabomination_catalog/src"
CACHE = ROOT / "scripts/.scryfall_cache.json"

FN_RE = re.compile(r"pub fn (\w+)\(\) -> CardDefinition \{")
COMMENT = re.compile(r"^\s*//.*$", re.M)
NAME_RE = re.compile(r'name:\s*"((?:[^"\\]|\\.)*)"')
HELPER_NAME = re.compile(r'\(\s*"((?:[^"\\]|\\.)*)"')

# "This land enters tapped." / "This land enters tapped unless …" /
# "Kabira Crossroads enters tapped." Not "enters the battlefield tapped under
# an opponent's control" and not a clause about some *other* permanent.
CLAUSE = re.compile(
    r"^(This|[A-Z][^,\n]{1,40}) (land|permanent|creature|artifact|enchantment)"
    r" enters tapped(\.| unless| and)",
    re.M,
)

# An ETB trigger, written out or reached through a helper that builds one.
TRIGGER = re.compile(
    r"EventKind::EntersBattlefield"
    r"|\betb\(|\betb_tap\(|etb_tap_then_"
    r"|shockland_pay_two_or_tap\(|fastland_etb_conditional_tap\("
    r"|pay_three_or_tapped\("
)
# The printed clause in ANY inflection, for the mirror column: "this land
# enters tapped", "artifacts your opponents control enter tapped", "put it
# onto the battlefield tapped".
PRINTS_TAPPED = re.compile(
    r"enters? (?:the battlefield )?tapped|battlefield tapped|\btapped\b[^.]*\btoken\b",
    re.I,
)

# A replacement, written out or reached through one of its helpers.
REPLACEMENT = re.compile(
    r"StaticEffect::(EntersTapped|EntersTappedUnless)"
    r"|enters_tapped_unless\(|reveal_or_tapped_land\(|land_type_reveal_land\("
    r"|battle_land\(|triome\(|tri_land\(|tapland_typed\(|tapland_untyped\("
    r"|cycling_dual\(|enters_tapped\(|modern_etb_tap\(|fastland_enters_tapped\("
    # Helpers whose own body carries the replacement.
    r"|znr_mdfc_land\(|restless_land\(|slow_land\(|afr_land\("
)

# The two cycles put on the replacement 2026-09-19. `--gate` fails on a
# regression.
SLOW_LANDS = [
    "Deathcap Glade", "Deserted Beach", "Dreamroot Cascade", "Haunted Ridge",
    "Overgrown Farmland", "Rockfall Vale", "Shattered Sanctum",
    "Shipwreck Marsh", "Stormcarved Coast", "Sundown Pass",
]
BATTLE_LANDS = [
    "Canopy Vista", "Cinder Glade", "Eclipsed Steppe", "Prairie Stream",
    "Radiant Summit", "Scorched Geyser", "Smoldering Marsh",
    "Sodden Verdure", "Sunken Hollow", "Vernal Fen",
]



def _factory_body(src, start):
    """The factory's source, brace-matched from its opening `{`."""
    i = src.index("{", start)
    depth = 0
    while i < len(src):
        if src[i] == "{":
            depth += 1
        elif src[i] == "}":
            depth -= 1
            if depth == 0:
                return src[start : i + 1]
        i += 1
    return src[start:]


def card_bodies(cache):
    """{printed name: (factory, file, source body)} for every card factory.

    ⚠ The name is the first string literal in the body that the Scryfall cache
    KNOWS, not simply the first string literal. The sibling scripts take the
    first one and mis-key every card whose body opens with a helper call that
    takes a description — Shipwreck Marsh gets filed under "This land enters
    tapped unless you control two or more other lands", which then reads as
    "not in the catalog". One `in cache` test removes that whole class of
    reader miss.
    """
    out = {}
    for path in sorted(CATALOG.rglob("*.rs")):
        src = path.read_text()
        for m in FN_RE.finditer(src):
            start, ident = m.start(), m.group(1)
            # ⚠ Brace-matched from the factory's own `{`, NOT "up to the next
            # `pub fn`". A private helper between two factories is otherwise
            # read as part of the preceding card, and the card *after* such a
            # helper has its own body hidden behind it — Shifting Woodland was
            # hidden that way. Brace matching also drops the trailing doc
            # comment that belongs to the next card, which had read as a
            # shipped ability (Hardened Academic). Comments inside the body
            # still go, because a comment can name an effect the card does not
            # carry.
            body = COMMENT.sub("", _factory_body(src, start))
            candidates = [m.group(1) for m in NAME_RE.finditer(body)]
            candidates += [m.group(1) for m in HELPER_NAME.finditer(body)]
            name = next(
                (c.replace('\\"', '"') for c in candidates if c.replace('\\"', '"') in cache),
                None,
            )
            if name:
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
    bodies = card_bodies(cache)

    columns = {"conditional": [], "unconditional": []}
    regressed = []
    for name, (ident, fname, body) in bodies.items():
        entry = cache.get(name)
        if not isinstance(entry, dict):
            continue
        m = CLAUSE.search(oracle(entry))
        if not m or REPLACEMENT.search(body) or not TRIGGER.search(body):
            continue
        key = "conditional" if "unless" in m.group(0) else "unconditional"
        rank = entry.get("edhrec_rank")
        columns[key].append(
            (rank if rank is not None else 10**9, name, fname, ident, m.group(0))
        )
        if name in SLOW_LANDS or name in BATTLE_LANDS:
            regressed.append(name)

    for key in ("conditional", "unconditional"):
        rows = sorted(columns[key])
        print(f"\n== {key}: {len(rows)}")
        for rank, name, fname, ident, clause in rows:
            r = rank if rank < 10**9 else "-"
            print(f"  {r:>7}  {name}  [{fname}::{ident}]  {clause.strip()}")

    # The mirror column: a card that SHIPS the replacement and prints no
    # "enters tapped" clause at all. An invented replacement is the same kind
    # of defect as a missing one, and a one-directional ratchet is how a
    # conversion pass quietly gives a card a downside it does not have.
    invented = []
    for name, (ident, fname, body) in bodies.items():
        if not REPLACEMENT.search(body):
            continue
        entry = cache.get(name)
        if isinstance(entry, dict) and not PRINTS_TAPPED.search(oracle(entry)):
            invented.append((name, fname, ident))
    print(f"\n== invented (ships the replacement, prints no tapped clause): {len(invented)}")
    for name, fname, ident in sorted(invented):
        print(f"  {name}  [{fname}::{ident}]")

    total = sum(len(v) for v in columns.values())
    cycles = SLOW_LANDS + BATTLE_LANDS
    missing = [n for n in cycles if n not in bodies]
    print(
        f"\n{total} cards print an 'enters tapped' replacement (CR 614.1c) and "
        f"ship it as an EntersBattlefield trigger; every one is a catalog edit, "
        f"no primitive is missing. The slow and battle cycles are "
        f"{len(cycles) - len(regressed) - len(missing)}/{len(cycles)} on the "
        f"replacement ({len(regressed)} regressed, {len(missing)} not in the "
        f"catalog)."
    )
    if not gate:
        return 0
    for name in regressed:
        print(f"REGRESSED: {name} is back on a trigger", file=sys.stderr)
    for name in missing:
        print(f"MISSING: {name} left the catalog", file=sys.stderr)
    # The class closed 2026-09-19, so the gate is the whole census: any new
    # card that prints "enters tapped" and ships a trigger fails here.
    if total:
        print(f"{total} cards are back on the trigger", file=sys.stderr)
    if invented:
        print(f"{len(invented)} cards ship a replacement they do not print", file=sys.stderr)
    return 1 if regressed or missing or total or invented else 0


if __name__ == "__main__":
    sys.exit(main())
