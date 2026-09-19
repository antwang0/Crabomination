#!/usr/bin/env python3
"""Cards whose BODY carries a resolution choice the printed card does not have.

The mirror of `audit_dropped_may.py`, and the stronger of the two directions.
A dropped "may" makes the engine's card *worse* than the print (the effect
fires when the player would have declined); an **invented** "may" makes it
strictly *better*, and better is what a bot exploits: a mandatory downside the
pilot can simply refuse. Boompile's "flip a coin … if it's tails, destroy all
artifacts" is not declinable; neither is a cumulative-upkeep payment's
consequence, nor "sacrifice a creature at the beginning of your end step".

Three gates, the same shape as `audit_target_opponent.py`'s:

1. the body names one of the **unconditional** optional wrappers — `MayDo`,
   `MayDoBy`, `MayRepeat`. These spell a bare "you may X" with no cost and no
   else-branch, so there is exactly one printed wording they can be modelling;
2. the printed text — **every face, reminder text included** — carries no
   optional wording at all: no "may", no "unless", no "up to", no "any
   number", no "if you choose", no "rather than". Reminder text is kept here
   (unlike the dropped-may audit, which strips it) because a keyword's
   reminder is exactly where a legitimate "you may" hides — Modular, Exert,
   Casualty — and this direction wants the *absence* proved, not the presence;
3. the needle is in a **resolution** position, not a token the card mints. A
   factory body holds the `TokenDefinition`s it creates, and a Role or Clue
   whose own ability prints "you may" is a different card's text. Bodies that
   mint a token are reported in their own bucket rather than dropped, since
   the wrapper may still be the card's.

    python3 scripts/audit_invented_may.py            # findings, grouped
    python3 scripts/audit_invented_may.py --count    # totals only
    python3 scripts/audit_invented_may.py --check    # exit 1 on any finding

⚠ `MayDoElse`, `MayPay*`, `MaySacrifice`, `MayTap` and `MayDiscard` are NOT
needles. Each models an "unless" or a cost — "unless that player sacrifices a
creature", "you may pay {2}" — and those print as a choice with no "may" in
sight, so they would report the whole class as invented. The gate that keeps
this audit at one reading is the wrapper set, not the oracle regex.
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CATALOG = os.path.join(ROOT, "crabomination_catalog", "src")
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")

# Gate 1 — every wrapper that spells a resolution-time choice, cost-bearing
# ones included. The narrow set (`MayDo` / `MayDoBy` / `MayRepeat`, the three
# that can only be a bare printed "you may") was measured first and read 2;
# adding the twelve cost-bearing and else-bearing wrappers — the ones that
# usually model an "unless" or a "you may pay", neither of which prints the
# word "may" — added exactly **one** true finding and one false positive.
# **It is gate 2 that keeps this audit precise, not the needle set**, so the
# wide set ships.
NEEDLES = (
    "Effect::MayDo {",
    "Effect::MayDoBy {",
    "Effect::MayRepeat {",
    "Effect::MayDoElse {",
    "Effect::MayPay {",
    "Effect::MayPayX {",
    "Effect::MayPayLife {",
    "Effect::MayPayRepeatedly {",
    "Effect::MayPayGenericUpTo {",
    "Effect::MayTap {",
    "Effect::MayDiscard {",
    "Effect::MayDiscardMatching {",
    "Effect::MaySacrifice {",
    "Effect::MaySacrificeSource {",
    "Effect::MayExileSelfThen {",
    "Effect::MayExileFromYourGraveyard {",
    "Effect::MayCopyThisSpell {",
    "Effect::OptionalTargets {",
)

# Gate 2 — any of these in the printed text and the card has *some* printed
# choice, so the wrapper has something to model and the card is out of scope.
OPTIONAL_WORDING = (
    "may",          # "you may", "that player may", "may be", …
    "unless",
    "up to",
    "any number",
    "rather than",
    "if you choose",
    "choose one",
    "choose two",
    "choose three",
    "choose up",
    "you don't",     # "you don't have to", "if you don't"
    "instead",       # a replacement the player elects
    "could",
    # "tap one or two target untapped creatures" is a declinable second
    # target, which is exactly what `OptionalTargets { min: 1 }` spells
    # (Coordinated Clobbering). A printed count range IS the printed choice.
    "one or two",
    "two or three",
)

# Gate 3 — a body that mints a token carries that token's rules text too.
TOKEN_MARKERS = ("TokenDefinition", "token_def", "TokenSpec")


def slug(name):
    return re.sub(r"[^a-z0-9]+", "_", name.lower()).strip("_")


def defs_in(path):
    """(fn, card name, body) per `pub fn … -> CardDefinition`.

    Same body walk as `audit_dropped_may.py`: prefer the `name:` literal whose
    slug matches the function, since a factory holds several (a back face, a
    token, the other half of an MDFC).
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
        fn = m.group(1)
        # ⚠ **`name:` alone is not where this catalog keeps the card's name.**
        # Most creature factories spread a constructor — `..creature("Lullmage
        # Mentor", …)` — so the only `name:` in the body belongs to the *token*
        # the card mints, and a `name:`-only walk scores the card against
        # "Merfolk". Take every string literal, prefer the one whose slug is
        # the function's, and fall back to the `name:` list.
        keyed = re.findall(r'name:\s*"((?:[^"\\]|\\.)*)"', body)
        own = next(
            (n for n in re.findall(r'"((?:[^"\\]|\\.)*)"', body) if slug(n) == fn), None
        )
        if own is None and not keyed:
            continue
        yield fn, own or keyed[0], body


def printed_text(card):
    """Every face's oracle text, reminder text kept (see gate 2)."""
    parts = [card.get("oracle_text") or ""]
    for face in card.get("card_faces") or []:
        parts.append(face.get("oracle_text") or "")
    return " \n ".join(parts).lower()


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    lower = {k.lower(): v for k, v in cache.items() if isinstance(v, dict)}
    for k, v in list(cache.items()):
        if isinstance(v, dict):
            lower.setdefault(k.split(" // ")[0].lower(), v)

    hits, token_hits = [], []
    checked = uncached = notext = 0
    for dirpath, _, files in os.walk(CATALOG):
        for f in files:
            if not f.endswith(".rs"):
                continue
            path = os.path.join(dirpath, f)
            for fn, name, body in defs_in(path):
                needle = next((n for n in NEEDLES if n in body), None)
                if needle is None:
                    continue
                card = lower.get(name.lower())
                if card is None:
                    uncached += 1
                    continue
                text = printed_text(card)
                if not text.strip():
                    # A card the cache has no text for is unknown, not silent.
                    notext += 1
                    continue
                checked += 1
                if any(w in text for w in OPTIONAL_WORDING):
                    continue
                row = (
                    name,
                    fn,
                    os.path.relpath(path, ROOT),
                    needle.split("::")[1].rstrip(" {"),
                    " ".join(text.split())[:110],
                )
                (token_hits if any(t in body for t in TOKEN_MARKERS) else hits).append(row)

    hits.sort()
    token_hits.sort()
    if "--count" not in sys.argv:
        for label, rows in (("", hits), ("# minting a token — check the token's text", token_hits)):
            if label and rows:
                print(label)
            for name, fn, path, needle, text in rows:
                print(f"{path}::{fn}  [{needle}]\n    {name}: “{text}”")
    print(
        f"# {len(hits)} invented resolution choices "
        f"({len(token_hits)} more on bodies that mint a token), "
        f"{checked} bodies with a needle checked against the oracle cache, "
        f"{uncached} skipped as uncached, {notext} skipped with no printed text"
    )
    if "--check" in sys.argv and hits:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
