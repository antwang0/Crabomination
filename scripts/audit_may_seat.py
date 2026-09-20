#!/usr/bin/env python3
"""Cards whose printed "may" belongs to someone else and that ask the CONTROLLER.

The third column of the `may` family. `audit_dropped_may` asks whether a
printed "you may" is in the body at all; `audit_invented_may` asks whether a
body's "may" is printed. Neither asks **whose** it is.

`Effect::MayDo` / `Effect::MayPay` put the question to `ctx.controller` — the
player resolving the effect. When the card says "**target opponent** may …",
"**that player** may …" or "**each player** may …", that is the wrong seat.
The two ways it goes unnoticed are both worth stating:

* **The body can already be right.** Divine Gambit named the target's
  controller in both halves of its gift-back (`ControllerOf(Target(0))`) and
  asked the caster whether to run it. The permanent went to the right player;
  only the choice was taken from them.
* **`AutoDecider` declines everything**, so in self-play both seats "agree"
  and no aggregate moves. It bites a `wants_ui` seat (a human is never
  offered their own choice), a `ScriptedDecider`, a net policy, and any pod
  where the seats do not answer alike. Divine Gambit's own comment made that
  argument for shipping it: "the auto outcomes are equivalent since both
  perspectives align on declining".

The fix is the seat-routed sibling — `Effect::MayDoBy { who, … }` /
`MayPayBy`, or `EachPlayerDoes`, which re-seats `ctx.controller` per player
and so makes a nested plain `MayDo` correct.

⚠ **A "may" after the other player's name is often still yours.** "At the
beginning of each opponent's upkeep, **you** may …" and "For each opponent,
**you** may …" are the controller's choice, and the needle below excludes an
intervening "you" for exactly that reason — four of the first eight rows were
that shape.

Run: `python3 scripts/audit_may_seat.py [--gate]`
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
COMMENT = re.compile(r"^\s*//.*$", re.M)

# "target opponent may", "that player may", "each player may" — with no "you"
# in between, which is what separates "each opponent may draw" from "at the
# beginning of each opponent's upkeep, you may draw".
OTHER_MAY = re.compile(
    r"(target (?:opponent|player)|that player|each (?:opponent|player)|any player)"
    r"((?:(?!\byou\b)[^.]){0,40})\bmay\b",
    re.I,
)
# The seat-routed forms, and the wrappers that re-seat `ctx.controller`.
ROUTED = re.compile(
    r"MayDoBy|MayPayBy|MayDiscardBy|MaySacrificeBy|EachPlayerDoes|EachPlayerMay"
    r"|UnlessPlayerPays|TemptingOffer|JoinForces|VoteTally|ask_seat"
)
# The controller-asked forms.
CONTROLLER_ASKED = re.compile(r"Effect::May(Do|Pay)\b")


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
    """{printed name: (factory, file, body)} — comments stripped, brace-matched."""
    out = {}
    for path in sorted(CATALOG.rglob("*.rs")):
        src = path.read_text()
        for m in FN_RE.finditer(src):
            body = COMMENT.sub("", _factory_body(src, m.start()))
            names = [x.group(1) for x in NAME_RE.finditer(body)]
            names += [x.group(1) for x in HELPER_NAME.finditer(body)]
            name = next(
                (n.replace('\\"', '"') for n in names if n.replace('\\"', '"') in cache),
                None,
            )
            if name:
                out.setdefault(name, (m.group(1), path.name, body))
    return out


def oracle(entry):
    text = entry.get("oracle_text") or ""
    for face in entry.get("card_faces") or []:
        if isinstance(face, dict):
            text += "\n" + (face.get("oracle_text") or "")
    return text


def main():
    gate = "--gate" in sys.argv[1:]
    cache = {
        k: v
        for k, v in json.loads(CACHE.read_text()).items()
        if isinstance(v, dict)
    }
    rows, checked = [], 0
    for name, (ident, fname, body) in card_bodies(cache).items():
        clause = OTHER_MAY.search(oracle(cache[name]))
        if not clause:
            continue
        checked += 1
        if ROUTED.search(body) or not CONTROLLER_ASKED.search(body):
            continue
        rank = cache[name].get("edhrec_rank")
        rows.append(
            (rank if rank is not None else 10**9, name, fname, ident, clause.group(0))
        )

    for rank, name, fname, ident, clause in sorted(rows):
        r = rank if rank < 10**9 else "-"
        print(f"  {r:>7}  {name}  [{fname}::{ident}]")
        print(f"           …{clause.strip()}…")
    print(
        f"\n{len(rows)} cards print a 'may' that belongs to another seat and ask "
        f"the controller; {checked} cards with such a clause checked."
    )
    if not gate:
        return 0
    if rows:
        print(f"{len(rows)} misrouted 'may' asks", file=sys.stderr)
    return 1 if rows else 0


if __name__ == "__main__":
    sys.exit(main())
