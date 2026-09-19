#!/usr/bin/env python3
"""Riders a body carries that the printed card never says.

The third INVENTED column, after `audit_invented_may.py` (a choice the print
does not offer) and `audit_invented_trigger.py` (an event the print does not
name). This one is about a *rider on an effect the card does print* — the
verb is right and a clause has been bolted onto it.

It is a **table**, deliberately: each row is `(needle, printed phrases,
note)`, and adding a rider is one line rather than one script. The needle is
read from the factory's OWN literal, with no helper expansion and no global
table — ENGINE_BACKLOG's "CLOSED WITH A REASON 2026-09-12" explains why an
invented column cannot afford an over-approximated `have` set.

    python3 scripts/audit_invented_rider.py            # findings
    python3 scripts/audit_invented_rider.py --count    # totals only
    python3 scripts/audit_invented_rider.py --check    # exit 1 on any finding

⚠ Both directions of a rider are defects, and this column only asks one of
them. A rider the card prints and the body drops is the MISSING direction and
belongs to `core_rules/catalog_registration.rs`'s clause-ratchet family (see
CARD_BACKLOG, "The printed-clause ratchet family").
"""

import json
import os
import re
import sys
import unicodedata
from collections import Counter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CATALOG = os.path.join(ROOT, "crabomination_catalog", "src")
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")

TOKEN_MARKERS = ("TokenDefinition", "token_def", "TokenSpec", "_token()")

# (needle in the body, phrases any of which licenses it, one-line note)
RIDERS = [
    (
        "Effect::DestroyNoRegen",
        ("regenerat",),
        "CR 701.15g — a destroy that also blanks regeneration shields",
    ),
    (
        "Effect::CantBeRegeneratedThisTurn",
        ("regenerat",),
        "the standalone 'creatures can't be regenerated this turn'",
    ),
    (
        "once_per_turn: true",
        # ⚠ The flag has two jobs and both are printed differently. On an
        # activated ability it is "activate only once each turn"; on a trigger
        # it is CR 603.2c batching ("whenever ONE OR MORE …") or a
        # first-of-the-turn clause ("attacks for the first time each turn",
        # "your first Human creature spell each turn", "your second card each
        # turn"). A phrase set that knows only the first reports the whole
        # trigger population.
        (
            "once each turn",
            "once during each turn",
            "only once",
            "first time each turn",
            "one or more",
            "your first",
            "second card each turn",
        ),
        "an activation limit the printed ability does not carry",
    ),
    (
        "sorcery_speed: true",
        (
            # "as a sorcery" subsumes "activate only as a sorcery", "only any
            # time you could cast a sorcery" and CR 716.2c's Class reminder
            # ("gain the next level as a sorcery"). Any printed occurrence of
            # the phrase IS the restriction.
            "as a sorcery",
            # A turn restriction is narrower than sorcery speed, so the flag
            # over-restricts rather than inventing (Wand of Ith).
            "only during your turn",
        ),
        "a sorcery-speed restriction the printed ability does not carry",
    ),
]

# Rule-implied riders: the printed card does not spell the clause because the
# **rules** supply it. Each row is `(fn, needle): reason`; a row whose site is
# gone fails, the way `audit_invented_trigger.py`'s ALLOW does.
ALLOW = {
    ("arcanum_wings", "sorcery_speed: true"):
        "CR 702.64a — aura swap is 'Activate only as a sorcery' by rule",
    ("squee_the_immortal", "sorcery_speed: true"):
        "CR 307.1/302.1 — casting a creature is sorcery-speed; the grant is not a second permission",
    ("eternal_scourge", "sorcery_speed: true"):
        "CR 307.1/302.1 — 'you may cast this card from exile' on a creature is still sorcery-speed",
    ("phyrexian_battleflies", "once_per_turn: true"):
        "'no more than twice each turn' has no primitive — `once_per_turn` is a bool; documented in INCOMPLETE_CARDS",
}


def slug(name):
    folded = unicodedata.normalize("NFKD", name)
    folded = "".join(c for c in folded if not unicodedata.combining(c))
    return re.sub(r"[^a-z0-9]+", "_", folded.lower().replace("'", "")).strip("_")


def defs_in(path):
    src = open(path, encoding="utf-8").read()
    for m in re.finditer(r"pub fn (\w+)\(\) -> CardDefinition \{", src):
        start, depth, i = m.end() - 1, 0, m.end() - 1
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
        own = next(
            (n for n in re.findall(r'"((?:[^"\\]|\\.)*)"', body) if slug(n) == fn), None
        )
        if own:
            yield fn, own, body


def printed_text(card):
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

    hits, token_hits, per_row = [], [], Counter()
    checked = uncached = allowed = 0
    used = set()
    for dirpath, _, files in os.walk(CATALOG):
        for f in files:
            if not f.endswith(".rs"):
                continue
            path = os.path.join(dirpath, f)
            for fn, name, body in defs_in(path):
                present = [r for r in RIDERS if r[0] in body]
                if not present:
                    continue
                card = lower.get(name.lower())
                if card is None:
                    uncached += 1
                    continue
                text = printed_text(card)
                if not text.strip():
                    continue
                checked += 1
                mints_token = any(t in body for t in TOKEN_MARKERS)
                for needle, phrases, note in present:
                    per_row[needle] += 1
                    if any(p in text for p in phrases):
                        continue
                    if (fn, needle) in ALLOW:
                        allowed += 1
                        used.add((fn, needle))
                        continue
                    row = (name, fn, os.path.relpath(path, ROOT), needle, note,
                           " ".join(text.split())[:100])
                    # A factory holds the rules text of every token it mints,
                    # and the card's oracle does not. Roadkill Rodney's
                    # sorcery-speed flag is on the **Mutagen token**, whose own
                    # printed text says "Activate only as a sorcery".
                    (token_hits if mints_token else hits).append(row)

    hits.sort()
    token_hits.sort()
    if "--count" not in sys.argv:
        for label, rows in (("", hits),
                            ("# minting a token — check the TOKEN's text", token_hits)):
            if label and rows:
                print(label)
            for name, fn, path, needle, note, text in rows:
                print(f"{path}::{fn}  [{needle}]  {note}\n    {name}: “{text}”")
    stale = sorted(set(ALLOW) - used)
    for fn, needle in stale:
        print(f"# ALLOW is STALE: {fn} no longer carries {needle}", file=sys.stderr)
    print(
        f"# {len(hits)} invented riders ({len(token_hits)} more on bodies that mint a "
        f"token) over {checked} bodies with a needle, {allowed} rule-implied "
        f"({len(stale)} stale), {uncached} skipped as uncached; per rider: "
        + ", ".join(f"{k.split('::')[-1]} {v}" for k, v in per_row.most_common())
    )
    return 1 if ("--check" in sys.argv and (hits or stale)) else 0


if __name__ == "__main__":
    sys.exit(main())
