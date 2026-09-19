#!/usr/bin/env python3
"""Cards that print a RETARGET clause and whose body does something else.

CR 115.7 — "change the target(s) of" (115.7a), "change a target" (115.7b),
"change any targets" (115.7c) and "choose new targets for" (115.7d) are four
permissions to *move* a spell or ability that is already on the stack. None of
them counters it, none of them copies it, and none of them is a fizzle.

The class this found: **Deflecting Swat** — "You may choose new targets for
target spell or ability", a top-100 EDHREC Commander staple and the red member
of the CR 118.9 free-spell cycle this branch deliberately completed — shipped
as `Effect::CounterSpell`. Its whole printed effect is a retarget; the card
cannot counter anything. The primitive it needed
(`Effect::ChooseNewTargetsForSpell`) was already in the tree and already used
by Redirect, Bolt Bend, Divert, Goblin Flectomancer and Redirect Lightning, so
the card was the only thing wrong.

Two columns, because a retarget clause reaches the print two different ways:

* **pure** — the card prints a retarget clause and does **not** print "copy".
  The body has to name a retarget effect. This is the precise column and the
  one Deflecting Swat was in.
* **copy** — "copy target …, you may choose new targets for the copy". The
  body has to name a copy effect; the engine repoints a copy's slots in
  `repoint_copy_slot` rather than in a separate effect, so a bare `Copy*` is
  the right answer here and this column is a count, not a queue. ⚠ Its nine
  unread rows are **reader misses, not defects** — Swarm Intelligence, Mirari,
  Thousand-Year Storm, Izzet Guildmage and Nivix Guildmage spell their copy
  through a private helper (`copy_cast_spell`, `copy_low_mv_is_ability`) whose
  body is not inside the card's own factory, which is the same limitation
  every name-keyed reader in this directory has.

Run: `python3 scripts/audit_retarget.py`   (exit 1 on any `pure` finding)
"""

import json
import os
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CATALOG = ROOT / "crabomination_catalog/src"
CACHE = ROOT / "scripts/.scryfall_cache.json"

PAREN = re.compile(r"\([^)]*\)")
# The four CR 115.7 permissions, as printed.
RETARGET = re.compile(
    r"choose new targets|change the target|change the targets|change a target"
    r"|change any targets|changes the target",
    re.I,
)
COPY_PRINTED = re.compile(r"\bcopy\b|\bcopies\b", re.I)

# Effects that move an object already on the stack.
MOVES = re.compile(
    r"Effect::(ChooseNewTargetsForSpell|ChangeSpellTarget|ChangeTargetOfAbility"
    r"|RedirectSpellTargetToSelf|RevealTopGreatestMayChangeTargets)"
)
# Effects that put a copy on the stack; the copy's slots are repointed by
# `repoint_copy_slot`, not by an effect of their own.
COPIES = re.compile(r"Effect::(\w*Copy\w*)")

FN_RE = re.compile(r"pub fn (\w+)\(\) -> CardDefinition \{")
NAME_RE = re.compile(r'name:\s*"((?:[^"\\]|\\.)*)"')
HELPER_NAME = re.compile(r'\(\s*"((?:[^"\\]|\\.)*)"')



def _factory_body(src, start):
    """The factory's source, brace-matched from its opening `{`.

    ⚠ NOT "up to the next `pub fn`". A private helper between two factories is
    otherwise read as part of the preceding card, and the card *after* such a
    helper has its own body hidden behind it — `audit_enters_tapped` had a row
    hidden that way. Brace matching also drops the trailing doc comment that
    belongs to the next card, which had read as a shipped ability.
    """
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

def card_bodies():
    """{printed name: (factory, path, source body)} for every card factory."""
    out = {}
    for path in sorted(CATALOG.rglob("*.rs")):
        src = path.read_text()
        for m in FN_RE.finditer(src):
            start, ident = m.start(), m.group(1)
            body = _factory_body(src, start)
            m = NAME_RE.search(body) or HELPER_NAME.search(body)
            if m:
                name = m.group(1).replace('\\"', '"')
                out.setdefault(name, (ident, path.relative_to(ROOT), body))
    return out


def main():
    cache = json.loads(CACHE.read_text())
    pure, copy_col, unread = [], 0, 0
    for name, (ident, path, body) in card_bodies().items():
        entry = cache.get(name)
        if not isinstance(entry, dict):
            continue
        text = PAREN.sub(" ", entry.get("oracle_text") or "")
        for face in entry.get("card_faces") or []:
            if isinstance(face, dict):
                text += " " + PAREN.sub(" ", face.get("oracle_text") or "")
        if not RETARGET.search(text):
            continue
        if COPY_PRINTED.search(text):
            copy_col += 1
            if not COPIES.search(body) and not MOVES.search(body):
                unread += 1
            continue
        if MOVES.search(body):
            continue
        clause = next(
            (s.strip() for s in re.split(r"(?<=[.•\n])", text) if RETARGET.search(s)),
            text.strip(),
        )
        shipped = re.search(r"Effect::(\w+)", body)
        pure.append((name, ident, path.name, shipped.group(0) if shipped else "?", clause))

    for name, ident, fname, shipped, clause in sorted(pure):
        print(f"- {name}  [{fname}::{ident}]  ships {shipped}")
        print(f"    {clause[:150]}")
    print(
        f"\n{len(pure)} cards print a CR 115.7 retarget and no copy, and their body "
        f"names no retarget effect; {copy_col} copy-and-retarget cards checked "
        f"({unread} of those name neither a copy nor a retarget effect)"
    )
    return 1 if pure else 0


if __name__ == "__main__":
    sys.exit(main())
