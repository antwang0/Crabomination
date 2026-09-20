#!/usr/bin/env python3
"""`PlayerStaticTarget` against the printed wording — who the static hits.

A player-scoped static ("can't gain life", "gains twice that much life", "hand
size is N") names its victims with `PlayerStaticTarget::{Controller,
EachOpponent, EachPlayer, EnchantedPlayer}`, and the printed card names them in
words: "you", "your opponents", "each player", "enchanted player". Nothing
checks that the two agree.

**The fan-out family's fourth ratchet, and the one the other three cannot
see.** `audit_each_opponent`, `audit_each_player` and `audit_target_opponent`
all read `PlayerRef` needles inside an `Effect` body. A `PlayerStaticTarget`
is neither a `PlayerRef` nor inside an effect — it is a field on a
`StaticEffect` — so every one of them walks straight past it.

⚠ **`Controller` and `EachOpponent` are opposites, and the first find was
exactly that swap.** Erebos, God of the Dead prints "**Your opponents** can't
gain life" and shipped as `PlayerCannotGainLife { target: Controller }`, with a
`description` that said "You can't gain life" — so the card stopped *its own
controller* gaining life, which is the reverse of the card, and the description
agreed with the bug rather than with the print. A wrong seat set here is not a
degree of wrong; it is the other player.

The needle→wording map is deliberately narrow. A card whose text names no seat
set at all is skipped rather than guessed at.

Run: `python3 scripts/audit_player_static_target.py`
"""

import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SRC = ROOT / "crabomination_catalog/src"

FN_RE = re.compile(r"^pub fn (\w+)\(\) -> CardDefinition", re.M)
NAME_RE = re.compile(r'name:\s*"((?:[^"\\]|\\.)*)"')
# Cards built through a per-file helper pass the name as the first argument.
HELPER_NAME_RE = re.compile(r'\(\s*"((?:[^"\\]|\\.)*)"')
PAREN = re.compile(r"\([^)]*\)")
TARGET_RE = re.compile(r"PlayerStaticTarget::(\w+)")

# What each variant claims the printed text says. A card must match at least
# one of its variant's patterns, or it is a row.
WORDING = {
    "Controller": re.compile(
        r"\byou (?:can't|cannot|don't|gain|lose|draw|have|skip|may not)"
        r"|\byour (?:life total|hand size|maximum hand size|upkeep)"
        r"|if you would",
        re.I,
    ),
    "EachOpponent": re.compile(
        r"your opponents|each opponent|an opponent|opponents can't|opponents cannot"
        r"|if an opponent would|each of your opponents",
        re.I,
    ),
    "EachPlayer": re.compile(
        r"each player|players can't|players cannot|no player|all players"
        r"|if a player would|each player's",
        re.I,
    ),
    "EnchantedPlayer": re.compile(r"enchanted player", re.I),
}

# card name -> why the mismatch is not a defect. Reasons a gate cannot express.
ALLOW: dict[str, str] = {
    "Rain of Gore": (
        "the print names its seat set as \"its controller\" — the controller of "
        "the spell or ability doing the gaining, which is any player — and "
        "`EachPlayer` is exactly that. The gate reads seat NOUNS (\"you\", "
        "\"your opponents\", \"each player\"), and this card names the seat by "
        "relation instead. The card's own approximation is elsewhere and is a "
        "different question: it fires on any life gain, not only on a spell's"
    ),
}


def _factory_body(src, start):
    """The factory's source, brace-matched from its opening `{`.

    ⚠ NOT "up to the next `pub fn`". Two things go wrong with that slice and
    both produce a **phantom row on the wrong card**: a private helper between
    two factories is read as part of the preceding card, and the trailing doc
    comment that belongs to the *next* card is read as this card's body. This
    audit's first run reported Cloud of Faeries — a Faerie with no static
    ability at all — for `PlayerStaticTarget::EachOpponent` in a sentence
    written about Tainted Remedy two functions later.

    Shared shape with `audit_target_opponent.py` / `audit_each_player.py`,
    which hit the same thing independently.
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
    """Every card factory's source, keyed by the printed card name."""
    out = {}
    for path in sorted(SRC.rglob("*.rs")):
        src = path.read_text()
        for mm in FN_RE.finditer(src):
            pos, ident = mm.start(), mm.group(1)
            body = _factory_body(src, pos)
            m = NAME_RE.search(body) or HELPER_NAME_RE.search(body)
            if m:
                name = m.group(1).replace('\\"', '"').replace("\\'", "'")
                out.setdefault(name, (ident, path.relative_to(ROOT), body))
    return out


def main():
    cache = json.loads((ROOT / "scripts/.scryfall_cache.json").read_text())
    rows, allowed, checked = [], [], 0
    for name, (ident, path, body) in sorted(card_bodies().items()):
        variants = set(TARGET_RE.findall(body))
        if not variants:
            continue
        entry = cache.get(name)
        if not isinstance(entry, dict):
            continue
        # Reminder text in parentheses is not a clause.
        text = PAREN.sub(" ", entry.get("oracle_text") or "")
        checked += 1
        for v in sorted(variants):
            pat = WORDING.get(v)
            if pat is None or pat.search(text):
                continue
            # Which set DOES the print name? That is the useful half.
            says = [k for k, p in WORDING.items() if k != v and p.search(text)]
            if name in ALLOW:
                allowed.append(name)
                continue
            rows.append((name, ident, path, v, says, text.strip()))

    for name, ident, path, v, says, text in rows:
        instead = f", the print names {' / '.join(says)}" if says else ""
        print(f"- {name}  [{path.name}::{ident}]  uses {v}{instead}")
        print(f"    {text[:160]}")
    stale = sorted(set(ALLOW) - set(allowed))
    for name in stale:
        print(f"STALE allowlist entry, the card no longer mismatches: {name}")
    print(
        f"\n{len(rows) + len(allowed)} cards aim a player static at a seat set the "
        f"print does not name, over {checked} cards with a `PlayerStaticTarget` "
        f"and a cached oracle — {len(allowed)} allowlisted, {len(rows)} "
        f"unexplained, {len(stale)} stale"
    )
    return 1 if rows or stale else 0


if __name__ == "__main__":
    sys.exit(main())
