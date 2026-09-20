#!/usr/bin/env python3
"""The word "another" in a TRIGGER clause — does the source exclude itself?

`audit_another_target.py` reads "another **target**" and asks where the other
object comes from. This is the trigger half, which that audit does not see:
"Whenever **another** creature you control enters" is not a targeting clause at
all, it is an `EventSpec` scope, and getting it wrong makes the permanent fire
off **itself** as it arrives.

That failure mode is quiet in the direction that matters. A Soul Warden printed
"whenever another creature enters" and modelled as `YourControl` gains one
extra life the turn it lands, once, and every test written from the model
agrees with it — the card simply does slightly more than it prints, on exactly
one event per permanent, forever.

Two encodings are correct and the audit accepts both:

* `EventScope::AnotherOfYours` — the scope says it; and
* `EventScope::YourControl` (or `AnyPlayer`) plus an `OtherThanSource` in the
  trigger's own filter — the filter says it.

⚠ **A third shape is correct and is NOT accepted, deliberately**: a trigger
whose event cannot fire off its own source at all (a `Dies` on a permanent that
is not a creature, say). Those go in `ALLOW` with the reason, because "this
event happens not to reach the source" is a fact about the rest of the card and
a gate cannot read it.

Both directions are checked. The print says "another" and the body does not
exclude the source; or the body excludes the source and the print never says
"another" / "other".

Run: `python3 scripts/audit_another_trigger.py`
"""

import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SRC = ROOT / "crabomination_catalog/src"

FN_RE = re.compile(r"^pub fn (\w+)\(\) -> CardDefinition", re.M)
NAME_RE = re.compile(r'name:\s*"((?:[^"\\]|\\.)*)"')
HELPER_NAME_RE = re.compile(r'\.\.\s*\w+\(\s*"((?:[^"\\]|\\.)*)"')
PAREN = re.compile(r"\([^)]*\)")

# A printed trigger clause naming "another"/"other" object. Anchored on the
# trigger word so "exile another target creature" (a targeting clause, and
# `audit_another_target.py`'s subject) does not come through here.
# ⚠ **The needle must be in the EVENT clause, not in the effect.** A printed
# trigger is "Whenever <event>, <effect>.", and "another" is at least as common
# on the right of that comma as on the left: Desolation Giant's "When this
# creature enters, destroy all **other** creatures you control" is an effect,
# and Atzocan Archer's "fight **another** target creature" is
# `audit_another_target.py`'s subject. Anchoring on the trigger word and
# allowing 80 characters swallowed both — 99 rows, most of them this.
PRINT_ANOTHER = re.compile(
    r"\b(?:whenever|when|at the beginning of)\b([^,.•\n]*)",
    re.I,
)
ANOTHER_WORD = re.compile(r"\b(?:another|other)\b", re.I)


# ⚠⚠ **"another" does not always EXCLUDE the source, and the commonest printed
# shape that contains it INCLUDES the source.** Blood Artist is "Whenever Blood
# Artist **or** another creature dies"; Boros Elite is "Whenever Boros Elite
# **and** at least two **other** creatures attack". Both name the source
# explicitly and then widen, so a body that fires off the source is right.
#
# This gate is why the first run was 221 rows and not a class: the needle
# "another" is shared by two opposite clauses, and only the one WITHOUT a
# self-reference beside it means "not me".
SELF_INCLUSIVE = re.compile(
    r"\b(?:or|and)\s+(?:at least \w+\s+|one or more\s+|two or more\s+)?(?:another|other)\b"
    r"|\b(?:another|other)\s+\w[\w\s]{0,30}?\bor\s+this\b",
    re.I,
)

def event_clause_says_another(text):
    """True when some trigger's EVENT clause (up to its first comma) says
    "another"/"other" and does not also name the source beside it."""
    for m in PRINT_ANOTHER.finditer(text):
        clause = m.group(1)
        if ANOTHER_WORD.search(clause) and not SELF_INCLUSIVE.search(clause):
            return clause.strip()
    return None


# ⚠ **Four encodings say "not me", and a gate that knows fewer reports correct
# cards.** Each of these was found by reading a row the audit produced and
# discovering the card was already right:
#
# * `EventScope::AnotherOfYours` — the scope says it (Pitiless Plunderer, via
#   the `on_other_dies` shortcut);
# * an `OtherThanSource` / `OtherThanTargetSlot` in the trigger's filter;
# * `Predicate::Not(TriggerSourceIsSelf)` — the explicit form (Last Laugh);
# * a scope whose own name carries "Other" —
#   `YourOtherSourceDamagedOpponent` (Talon of Pain).
EXCLUDES_SELF = re.compile(
    r"EventScope::AnotherOfYours"
    r"|OtherThanSource|OtherThanTargetSlot"
    r"|TriggerSourceIsSelf"
    r"|EventScope::\w*Other\w*"
)

# card name -> why the row is not a defect. ⚠ Every one of these was read by
# hand; the categories are the gate's limits, not the catalog's.
#
# **(a) "other" is not about the source.** The word qualifies a zone, a player
# or an ordinal, and the audit's event-clause reader cannot tell which noun it
# attaches to.
_ZONE = (
    "\"from anywhere **other than your hand**\" qualifies the ZONE, not the "
    "source — nothing here can fire off the permanent itself"
)
_PLAYER = "\"another player\" is an opponent; the word qualifies a PLAYER, not the source"
#
# **(b) the source cannot be the event's subject.** True of the rest of the
# card rather than of this clause, which is why it cannot be a gate.
_NOT_ON_BOARD = (
    "the clause is about a card CYCLED or DISCARDED from hand, and the source "
    "is a battlefield permanent when the trigger is live — it can never be the "
    "card in question, so \"another\" is already satisfied by the zones"
)
ALLOW: dict[str, str] = {
    "Graham O'Brien": _ZONE,
    "Keeper of Secrets": _ZONE,
    "Kellan, the Kid": _ZONE,
    "Unstable Amulet": _ZONE,
    "Vega, the Watcher": _ZONE,
    "Night Dealings": _PLAYER,
    "Risky Move": _PLAYER,
    "Archfiend of Ifnir": _NOT_ON_BOARD,
    "Curator of Mysteries": _NOT_ON_BOARD,
    "Drannith Healer": _NOT_ON_BOARD,
    "Drannith Stinger": _NOT_ON_BOARD,
    "Flourishing Fox": _NOT_ON_BOARD,
    "Horror of the Broken Lands": _NOT_ON_BOARD,
    "Valiant Rescuer": _NOT_ON_BOARD,
    "Slitherwisp": (
        "\"another spell that has flash\" — a spell on the stack, and the "
        "Slitherwisp is a battlefield permanent while the trigger is live"
    ),
    "Ichneumon Druid": (
        "\"other than **the first** instant spell that player casts each turn\" "
        "is an ORDINAL, not the source"
    ),
    "Eye of Singularity": (
        "\"a permanent other than **a basic land**\" excludes a card TYPE, not "
        "the source"
    ),
    "Boggart Shenanigans": (
        "\"another **Goblin** you control\" — the Shenanigans is an "
        "Enchantment and never a Goblin, so the event cannot reach its own "
        "source whatever the scope says"
    ),
    "City in a Bottle": (
        "\"other nontoken permanents with a name originally printed in Arabian "
        "Nights\" — the Bottle is one such card, but the clause is a "
        "state-trigger over a printed-set list the engine models as an explicit "
        "name set that does not contain the Bottle"
    ),
    "Measure of Wickedness": (
        "the \"another card\" clause is a GRAVEYARD-fill counter, and the "
        "Measure's own trip to the graveyard ends the permanent — the modelled "
        "ability is its end-step half, which prints no \"another\" at all"
    ),
}


def _factory_body(src, start):
    """The factory's source, brace-matched from its opening `{`.

    ⚠ NOT "up to the next `pub fn`": a private helper between two factories is
    read as part of the preceding card, and the trailing doc comment that
    belongs to the *next* card is read as this card's body. Shared shape with
    the other `audit_*.py` readers, which hit both independently.
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


HELPER_FN = re.compile(r"^(?:pub )?fn (\w+)", re.M)
CALL = re.compile(r"\b(\w+)\s*\(")

# Cross-file helpers every set file reaches for. ⚠ `audit_dropped_may` learned
# the same-file half of this lesson; the trigger scopes live one file further
# out, in the shared shortcut module, so a card built through `on_other_dies`
# carries **none** of its own scope.
SHARED = ROOT / "crabomination_base/src/effect/shortcut.rs"


def _helpers_in(src):
    """name -> body for every `fn` in `src` (helpers and factories alike)."""
    out = {}
    for m in HELPER_FN.finditer(src):
        out[m.group(1)] = _factory_body(src, m.start())
    return out


def _with_helpers(body, local, shared):
    """`body` plus the bodies of the helpers it names, resolved twice.

    ⚠⚠ **A factory's helper IS part of its body, and for trigger scopes the
    helper is usually in another file.** Pitiless Plunderer's whole ability is
    `on_other_dies(mint_treasures(1))`, and `on_other_dies` is where the
    `EventScope::AnotherOfYours` lives — so a reader that looks only at the
    factory sees a card printed "another creature" with no exclusion anywhere,
    and reports a correct card. That was 43 rows of this audit's first honest
    run, and nearly all of them were this.
    """
    seen, text = set(), body
    for _ in range(2):
        for m in CALL.finditer(text):
            n = m.group(1)
            if n in seen:
                continue
            h = local.get(n) or shared.get(n)
            if h:
                seen.add(n)
                text += "\n" + h
    return text


def card_bodies():
    out = {}
    shared = _helpers_in(SHARED.read_text())
    for path in sorted(SRC.rglob("*.rs")):
        src = path.read_text()
        local = _helpers_in(src)
        for mm in FN_RE.finditer(src):
            pos, ident = mm.start(), mm.group(1)
            body = _factory_body(src, pos)
            m = NAME_RE.search(body) or HELPER_NAME_RE.search(body)
            if m:
                name = m.group(1).replace('\\"', '"').replace("\\'", "'")
                out.setdefault(
                    name, (ident, path.relative_to(ROOT), _with_helpers(body, local, shared))
                )
    return out


def main():
    cache = json.loads((ROOT / "scripts/.scryfall_cache.json").read_text())
    missing, checked = [], 0
    spurious, allowed = [], []
    for name, (ident, path, body) in sorted(card_bodies().items()):
        if "triggered_abilities" not in body and "TriggeredAbility" not in body:
            continue
        entry = cache.get(name)
        if not isinstance(entry, dict):
            continue
        text = PAREN.sub(" ", entry.get("oracle_text") or "")
        checked += 1
        event_clause = event_clause_says_another(text)
        says_another = event_clause is not None
        excludes = bool(EXCLUDES_SELF.search(body))
        hit = (says_another and not excludes) or (
            excludes and not says_another and not re.search(r"\bother\b", text, re.I)
        )
        if name in ALLOW:
            # ⚠ Recorded, not skipped. `continue`-ing here left the stale check
            # comparing ALLOW against a list its own entries could never reach,
            # so every allowlisted card read as stale — a ratchet that cries
            # "stale" on all 21 of its own reasons teaches the next run to
            # ignore the word.
            if hit:
                allowed.append(name)
            continue
        if says_another and not excludes:
            missing.append((name, ident, path, event_clause))
        elif excludes and not says_another and not re.search(r"\bother\b", text, re.I):
            spurious.append((name, ident, path, text.strip()))

    for name, ident, path, clause in missing:
        print(f"- {name}  [{path.name}::{ident}]  prints \"another\", body does not exclude the source")
        print(f"    {clause[:150]}")
    # ⚠ **The reverse direction is counted and NOT printed, on purpose.**
    # "Body excludes the source, print never says another" reads 276 rows,
    # because the needle is body-wide: an `OtherThanSource` anywhere in a card
    # — in an anthem, an activated ability, a targeting filter — scores against
    # a printed text that had no reason to say "another" in its *trigger*. A
    # ratchet that prints 276 rows nobody will read teaches the next run to
    # skip its output, which costs more than the direction is worth. The count
    # stays so a sudden jump is visible; making it precise needs the filter
    # attributed to its own trigger, which is a parse this reader does not do.
    stale = sorted(set(ALLOW) - set(allowed))
    for n in stale:
        print(f"STALE allowlist entry: {n}")
    print(
        f"\n{len(missing)} triggers print \"another\" and do not exclude their own source, "
        f"over {checked} cards with a trigger and a cached oracle — {len(allowed)} "
        f"allowlisted, {len(stale)} stale. ({len(spurious)} in the reverse direction, "
        f"counted but not printed — see the note in `main`.)"
    )
    return 1 if stale else 0


if __name__ == "__main__":
    sys.exit(main())
