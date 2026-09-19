#!/usr/bin/env python3
"""Cards whose oracle says "TARGET opponent/player" but whose body fans OUT.

The mirror of `audit_each_opponent.py`, and the half that only bites at three
seats or more: a card printed "target opponent discards two cards" modelled as
`Selector::Player(PlayerRef::EachOpponent)` is correct in a duel and hits the
whole table in a pod. The duel is why it survives review — a one-opponent
fan-out *is* the target.

A hit needs three things, and the third is what keeps the precision up:

1. the printed text names a **target** player or opponent (reminder text in
   parentheses stripped — "(you may cast…)" is not a clause);
2. the printed text has **no** per-opponent/per-player clause at all, so there
   is nothing in the card a fan-out could legitimately be modelling;
3. the body names a fan-out `PlayerRef` in a recipient position.

`SelectionRequirement::OpponentPlayer` is *not* a fan-out — it is the target
filter that spells "target opponent" — so it is not one of the needles.

Run: `python3 scripts/audit_target_opponent.py`
"""

import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SRC = ROOT / "crabomination_catalog/src"

FN_RE = re.compile(r"^pub fn (\w+)\(\) -> CardDefinition", re.M)
NAME_RE = re.compile(r'name:\s*"((?:[^"\\]|\\.)*)"')
PAREN = re.compile(r"\([^)]*\)")

# The printed clause that names one player as a target.
TARGET = re.compile(r"target (opponent|player)\b", re.I)
# Any clause that legitimately reaches more than one seat. One of these in the
# text and the card is out of scope: the fan-out in the body may well be it.
MULTI = re.compile(
    r"each opponent|each player|each other player|each of your opponents"
    r"|any number of target|opponents control|players control"
    r"|each opponent's|each player's|all players|your opponents",
    re.I,
)
# A fan-out reference in a *recipient* position. `OpponentPlayer` is a target
# filter, not a fan-out, and is deliberately absent.
FANOUT = re.compile(
    r"PlayerRef::(EachOpponent|EachPlayer|EachOtherPlayer|EachTeammate)"
    r"|Effect::ForEachOpponent\b|Effect::EachPlayerDoes\b"
)


# A fan-out inside a `Predicate::` or an `AlternativeCost` condition is not a
# recipient: Archive Trap's "if an opponent searched their library this turn"
# and Ravenous Trap's "if three or more cards were put into an opponent's
# graveyard" are printed per-opponent and modelled correctly.
CONDITION = re.compile(r"Predicate::|condition:")
# Two more places a fan-out is the printed text rather than a recipient:
# `PlayersMayAccept`'s offer is literally "any player may …" (Browbeat), and a
# `Gift`'s token goes to the opponent the caster promises, which the engine
# has no single-recipient shape for (Mind Spiral, Sazacap's Brew — filed in
# CARD_BACKLOG, not a target-clause defect).
NOT_A_RECIPIENT = re.compile(r"PlayersMayAccept|Gift \{|gifted_effect|label:")


def in_condition(body, at):
    """The needle at `at` sits somewhere that is not a target recipient."""
    head = body[:at]
    return bool(CONDITION.search(head[-220:])) or bool(
        NOT_A_RECIPIENT.search(head[-400:])
    )


def card_bodies():
    """Every card factory's source, keyed by the printed card name."""
    out = {}
    for path in sorted(SRC.rglob("*.rs")):
        src = path.read_text()
        fns = [(m.start(), m.group(1)) for m in FN_RE.finditer(src)]
        for i, (pos, ident) in enumerate(fns):
            end = fns[i + 1][0] if i + 1 < len(fns) else len(src)
            body = src[pos:end]
            m = NAME_RE.search(body)
            if m:
                name = m.group(1).replace('\\"', '"').replace("\\'", "'")
                out.setdefault(name, (ident, path.relative_to(ROOT), body))
    return out


def main():
    cache = json.loads((ROOT / "scripts/.scryfall_cache.json").read_text())
    rows = []
    for name, (ident, path, body) in card_bodies().items():
        entry = cache.get(name)
        if not isinstance(entry, dict):
            continue
        text = entry.get("oracle_text") or ""
        stripped = PAREN.sub(" ", text)
        if not TARGET.search(stripped) or MULTI.search(stripped):
            continue
        hit = next(
            (m for m in FANOUT.finditer(body) if not in_condition(body, m.start())), None
        )
        if not hit:
            continue
        clause = next(
            (s.strip() for s in re.split(r"(?<=[.•\n])", stripped) if TARGET.search(s)),
            stripped.strip(),
        )
        rows.append((name, ident, path, hit.group(0), clause))

    for name, ident, path, needle, clause in sorted(rows):
        print(f"- {name}  [{path.name}::{ident}]  {needle}")
        print(f"    {clause[:150]}")
    print(
        f"\n{len(rows)} implemented cards print a TARGET player clause and fan out instead"
    )
    return 1 if rows else 0


if __name__ == "__main__":
    sys.exit(main())
