#!/usr/bin/env python3
"""Cards whose oracle text says "you may" and whose definition has no optional
primitive — the printed choice dropped, so the effect fires unconditionally.

A dropped "may" is not cosmetic. `Aura Shards` reads "whenever a creature
enters under your control, you **may** destroy target artifact or
enchantment"; without the may, a board where the only legal target is your
own permanent forces you to blow it up, because a trigger's targets are
mandatory once it triggers and the "may" was the only out.

    python3 scripts/audit_dropped_may.py            # ranked list
    python3 scripts/audit_dropped_may.py --count    # just the totals

Reads `scripts/.scryfall_cache.json` (offline; `scripts/fetch_oracle.py`
fills it). Cards whose name is not in the cache are **skipped and counted
separately** — the catalog carries thousands of synthesized `(b###)` names
with no oracle to be wrong against, and flagging those would bury the real
findings.

The optional-primitive list below is what the engine spells a printed choice
with. It is deliberately generous: a false negative here costs nothing, a
false positive costs a reader's time. Same for `ORACLE_SKIP`, the "you may"
phrasings that are keyword reminder text or a cast-time permission rather
than a resolution choice.
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CATALOG = os.path.join(ROOT, "crabomination_catalog", "src")
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")

# Every way a definition can carry a printed choice.
OPTIONAL = (
    "MayDo", "MayDoBy", "MayDoElse", "MayPay", "MayTap", "MayDiscard",
    "MayCast", "MayPayOrElse", "MayReveal", "MaySacrifice", "Optional",
    "ChooseMode", "ChooseN", "Escalate", "TapOrUntap", "may_", "_may",
    # Optionality the engine spells as a *shape* rather than a `May…` name.
    # Every one of these was a false positive on a card that was already
    # correct when the list was audited by hand (2026-08-30): Devouring Greed
    # and Plumb the Forbidden are `SacrificeAnyNumber`, Voltage Surge's
    # "you may sacrifice an artifact" is its `kicker_action_cost`, and a
    # "you may put … from among them" is a `LookPick` with `up_to: true` or a
    # `Selector::one_of` the picker may decline.
    "AnyNumber", "up_to", "kicker_action_cost", "one_of", "OneOf",
    "SacrificeOrPayLife", "min_targets: 0",
)

# "you may" phrasings that are not a resolution choice the effect tree owns.
ORACLE_SKIP = (
    "you may cast",            # alternate-cost / from-exile permissions
    "you may play",
    "you may pay",             # MayPay, and also ward/kicker reminder text
    "you may put this card",   # suspend / foretell reminder text
    "you may look at",         # reminder text on scry-likes
    "you may exile it from",   # flashback-ish reminder text
    "you may have",            # copy-target reminder ("you may choose new targets")
    "you may choose new targets",
    "you may reveal",          # most reveals are engine-implicit
    "as you may",
)

# A "you may … rather than pay this spell's mana cost" is an alternative cost
# (`AlternativeCost` / `AdditionalCastCost`), announced at cast time, not a
# resolution choice the effect tree owns. Flare of Malice and Fireblast are
# implemented, not flattened.
ORACLE_SKIP_CONTAINS = ("rather than pay",)


def slug(name):
    """"Pestilent Cauldron" -> "pestilent_cauldron", the factory-name shape."""
    return re.sub(r"[^a-z0-9]+", "_", name.lower()).strip("_")


def defs_in(path):
    """(fn_name, card_name, body, path) for every `pub fn … -> CardDefinition`.

    A factory body often holds **several** `name:` literals — a transform back
    face, a token it mints, a `let` for the other half of an MDFC — and the
    first one is not reliably the card's. Taking it blind is how a name-keyed
    audit ends up scoring `pestilent_cauldron` against Restorative Burst's
    oracle. Prefer the literal whose slug matches the function name, which is
    this catalog's naming convention; fall back to the first only when none
    does, and say so by yielding it anyway (the caller's cache lookup is the
    second filter).
    """
    src = open(path, encoding="utf-8").read()
    for m in re.finditer(r"pub fn (\w+)\(\) -> CardDefinition \{", src):
        start = m.end() - 1
        depth, i = 0, start
        while i < len(src):
            if src[i] == "{":
                depth += 1
            elif src[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        body = src[start : i + 1]
        names = re.findall(r'name:\s*"([^"]+)"', body)
        if not names:
            continue
        fn = m.group(1)
        own = next((n for n in names if slug(n) == fn), None)
        yield fn, own or names[0], body, path


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    # A few entries in the cache are bare strings (a negative-lookup marker
    # from an older fetcher); they carry no oracle text and are not findings.
    lower = {k.lower(): v for k, v in cache.items() if isinstance(v, dict)}
    for k, v in list(cache.items()):
        if isinstance(v, dict):
            lower.setdefault(k.split(" // ")[0].lower(), v)

    hits, uncached, checked = [], 0, 0
    for dirpath, _, files in os.walk(CATALOG):
        for f in files:
            if not f.endswith(".rs"):
                continue
            for fn, name, body, path in defs_in(os.path.join(dirpath, f)):
                card = lower.get(name.lower())
                if card is None:
                    uncached += 1
                    continue
                checked += 1
                # Reminder text is where most "you may" phrasings live —
                # Modular's "you may put its +1/+1 counters on…", Exert,
                # Casualty, Collect evidence. The keyword itself is what the
                # definition implements, so a parenthesised span is never the
                # finding. Scryfall puts reminder text in parentheses.
                oracle = re.sub(r"\([^)]*\)", " ", card.get("oracle_text") or "").lower()
                if "you may" not in oracle:
                    continue
                spans = [
                    s for s in re.findall(r"you may[^.;\n]*", oracle)
                    if not any(s.startswith(p) for p in ORACLE_SKIP)
                    and not any(p in s for p in ORACLE_SKIP_CONTAINS)
                ]
                if not spans:
                    continue
                if any(tok in body for tok in OPTIONAL):
                    continue
                hits.append((name, fn, os.path.relpath(path, ROOT), spans[0][:80]))

    hits.sort()
    if "--count" not in sys.argv:
        for name, fn, path, span in hits:
            print(f"{path}::{fn}\n    {name}: “{span}”")
    print(
        f"# {len(hits)} definitions with a dropped 'you may', "
        f"{checked} checked against the oracle cache, {uncached} skipped as uncached "
        f"(synthesized names have no oracle)"
    )


if __name__ == "__main__":
    main()
