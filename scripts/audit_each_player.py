#!/usr/bin/env python3
"""Cards whose printed fan-out names the WRONG set — "each player" vs "each opponent".

The third member of the fan-out family, after `audit_each_opponent.py` (the
print says each opponent, the body names nobody) and
`audit_target_opponent.py` (the print says *target*, the body fans out). This
one is about the fan-out being *present and wrong by one seat*: the
controller.

Two directions, and only one of them is duel-invisible:

* **print "each player", body `EachOpponent`** — the controller is spared.
  Wrong at two seats as well, so these are rarer and usually deliberate.
* **print "each opponent", body `EachPlayer`** — the controller is hit too.
  Also wrong at two seats.

Either way, "and it is symmetric in a pod" is the reason to check: a drain
that should cost the caster nothing and does, three times over, is a bigger
swing at four seats than at two.

A card is out of scope when the printed text names **both** sets (a mode that
says "each opponent" and another that says "each player"), because then either
needle can be the right one for its own clause.

**Reading at the first run: 5 hits, 1 real.** The other four are recorded here
so nobody re-triages them:

* **Conciliator's Duelist** and **Realm Seekers** are *decompositions* — "each
  player loses 1 life" written as `EachOpponent` plus `You`, and "all players'
  hands" as `Sum[HandSizeOf(You), HandSizeOf(EachOpponent)]`. Same set. The
  `DECOMPOSED` gate below drops them.
* **Roiling Vortex**'s "each player" clause is a *trigger scope*
  (`StepBegins(Upkeep)` for any player), not a fan-out; the `EachOpponent` in
  its body is the printed "your opponents can't gain life".
* **Pir's Whim**'s friend/foe split is a documented approximation — every
  opponent is a foe and the controller is the friend.

The one real find was **Parallax Nexus**, and it is a *target-clause* defect
(`audit_target_opponent.py`'s class) that that audit could not see: its
printed text also carries an "each player" clause, which its `MULTI` gate uses
to stay precise. **Two ratchets over one family catch what one cannot** — the
gate that keeps one honest is the hole in the other.

Run: `python3 scripts/audit_each_player.py`
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

OPPONENTS = re.compile(r"each opponent|each of your opponents", re.I)
PLAYERS = re.compile(r"each player|all players|each of the players", re.I)
# "each other player" is neither: it is everyone but the controller, which in a
# duel is the one opponent and in a pod is not the same set as "each player".
OTHERS = re.compile(r"each other player", re.I)

OPP_NEEDLE = re.compile(r"PlayerRef::EachOpponent\b|each_opponent\(")
PLR_NEEDLE = re.compile(r"PlayerRef::EachPlayer\b|each_player\(")
# A condition or a gift promisee is not the effect's recipient set.
CONTEXT = re.compile(r"Predicate::|condition:|Gift \{|gifted_effect")

# card name -> why the mismatch is not a gap. Keep this to cases a gate cannot
# express; a reason that is really a code property belongs in the code.
ALLOW = {
    "Roiling Vortex": (
        "the printed 'each player' is a TRIGGER SCOPE (at the beginning of "
        "each player's upkeep), which the event system already fires once a "
        "seat; the `EachOpponent` in the body is the other printed clause, "
        "'your opponents can't gain life this turn'"
    ),
}


def card_bodies():
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


# A hit next to a sibling clause naming the controller is a **decomposition**,
# not a gap: "each player loses 1 life" written as `LoseLife{EachOpponent}` +
# `LoseLife{You}` is the same set, and so is `Sum[HandSizeOf(You),
# HandSizeOf(EachOpponent)]`. Both shapes were in the first run's five.
DECOMPOSED = re.compile(r"PlayerRef::You\b|Selector::You\b")


def recipient_hits(body, needle):
    """Needle occurrences that are not a condition, a gift or a decomposition."""
    out = []
    for m in needle.finditer(body):
        before, after = body[: m.start()][-300:], body[m.end() :][:300]
        if CONTEXT.search(before):
            continue
        if DECOMPOSED.search(before) or DECOMPOSED.search(after):
            continue
        out.append(m)
    return out


def main():
    cache = json.loads((ROOT / "scripts/.scryfall_cache.json").read_text())
    rows, allowed = [], []
    for name, (ident, path, body) in card_bodies().items():
        entry = cache.get(name)
        if not isinstance(entry, dict):
            continue
        text = PAREN.sub(" ", entry.get("oracle_text") or "")
        says_opp = bool(OPPONENTS.search(text))
        says_plr = bool(PLAYERS.search(text)) and not OTHERS.search(text)
        if says_opp == says_plr:
            continue  # both or neither — the needles cannot be told apart
        wrong = PLR_NEEDLE if says_opp else OPP_NEEDLE
        hits = recipient_hits(body, wrong)
        if not hits:
            continue
        want = "each opponent" if says_opp else "each player"
        clause = next(
            (
                s.strip()
                for s in re.split(r"(?<=[.•\n])", text)
                if (OPPONENTS if says_opp else PLAYERS).search(s)
            ),
            text.strip(),
        )
        if name in ALLOW:
            allowed.append(name)
            continue
        rows.append((name, ident, path, want, hits[0].group(0), clause))

    for name, ident, path, want, needle, clause in sorted(rows):
        print(f"- {name}  [{path.name}::{ident}]  prints {want!r}, body has {needle}")
        print(f"    {clause[:150]}")
    stale = sorted(set(ALLOW) - set(allowed))
    for name in stale:
        print(f"STALE allowlist entry, the card no longer hits: {name}")
    print(
        f"\n{len(rows) + len(allowed)} cards fan out over a set the print does not "
        f"name, {len(allowed)} allowlisted, {len(rows)} unexplained, {len(stale)} stale"
    )
    return 1 if rows or stale else 0


if __name__ == "__main__":
    sys.exit(main())
