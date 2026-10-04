#!/usr/bin/env python3
"""Printed "this enters with N <kind> counters" vs `enters_with_counters`.

Reads a card's OWN `enters_with_counters` (not a nested token's) against the
Scryfall oracle: a wrong counter kind, or a constant amount that differs from
the printed number. Non-constant values (X, "for each") are checked only for
not being a bare constant where the oracle prints X.

    cargo build -p crabomination --bin dump_cards
    target/debug/dump_cards --enters-with > /tmp/ew.tsv
    python3 scripts/audit_enters_with.py /tmp/ew.tsv [--gate]
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")
WORDS = {"a": 1, "an": 1, "one": 1, "two": 2, "three": 3, "four": 4, "five": 5, "six": 6,
         "seven": 7, "eight": 8, "nine": 9, "ten": 10, "eleven": 11, "twelve": 12}
KINDS = {"+1/+1": "PlusOnePlusOne", "-1/-1": "MinusOneMinusOne", "+1/+0": "PlusOnePlusZero",
         "+0/+1": "PlusZeroPlusOne", "+2/+2": "PlusTwoPlusTwo"}


# Reviewed rows. The named counter kinds have no `CounterType` of their own
# and ride `Charge` (nothing in the catalog reads a javelin / wish / crystal /
# isolation / arrowhead counter by name, so only proliferate-style "a kind of
# counter" reads could tell).
REVIEWED = {
    "Skyclave Shade": "'if kicked, two': one kicker, so 2 x TimesKicked is two or none",
    "Icatian Javelineers": "javelin counter rides Charge",
    "Wishclaw Talisman": "wish counter rides Charge",
    "Prism Array": "crystal counter rides Charge",
    "Quarantine Field": "isolation counter rides Charge",
    "Serrated Arrows": "arrowhead counter rides Charge",
    "Intrepid Adversary": "'pay any number of times, that many valor counters' ETB as an enters-with",
}


def kind_name(k):
    return KINDS.get(k) or "".join(w.capitalize() for w in re.split(r"[\s-]+", k))


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not args:
        sys.exit(__doc__)
    cache = {k: v for k, v in json.load(open(CACHE)).items() if isinstance(v, dict)}
    by_name = {}
    for k, v in cache.items():
        by_name.setdefault(k.lower(), v)
        by_name.setdefault(k.split(" // ")[0].lower(), v)
    rows = []
    for line in open(args[0]):
        name, dbg = line.rstrip("\n").split("\t", 1)
        card = by_name.get(name.lower())
        if not card:
            continue
        faces = card.get("card_faces") or []
        oracle = card.get("oracle_text") or "\n".join(f.get("oracle_text", "") for f in faces)
        oracle = re.sub(r"\([^)]*\)", "", oracle)
        short = name.split(",")[0]
        # The card names itself by full name, the part before the comma, its
        # first word ("Rasputin enters with …"), "this creature", or a
        # pronoun after its own condition ("If … kicked, it enters with").
        first = name.split()[0]
        subj = (r"(?:this [a-z]+|\bit|\bhe|\bshe|\bthey|" + re.escape(name) + "|" + re.escape(short)
                + "|" + re.escape(first) + ")")
        m = re.search(subj + r" enters(?: the battlefield)?(?: tapped)?(?: and)? with (\w+)(?: additional)? "
                      r"([+\-0-9/]+|[a-z]+(?: [a-z]+)?) counters? on it", oracle, re.I)
        if not m:
            # A keyword that puts the counters on (CR 702.43a modular, STX's
            # prepared, a planeswalker's loyalty, vanishing / fading, the
            # stun "enters tapped with N stun counters" rewrites) prints the
            # count only as the keyword or in stripped reminder text.
            kw = re.search(r"\b(?:modular|prepared|vanishing|fading|loyalty|graft|bloodthirst|"
                           r"evolve|outlast|reinforce|fabricate|riot|unleash|devour|amplify|"
                           r"sunburst|tribute|backup|ravenous|training|mentor|adapt|"
                           r"enters prepared|escapes with)", oracle + " " + (card.get("type_line") or ""), re.I)
            # The clause is there in a shape the count parser can't read ("a
            # number of … equal to", "twice X", "an additional … for each"):
            # the kind is still checked against the def, the amount is not.
            loose = re.search(subj + r" (?:enters|escapes)[^.]*? with [^.]*?([+\-0-9/]+|\b[a-z]+) counters?\b",
                              oracle, re.I)
            if kw or "Loyalty" in dbg or (loose and kind_name(loose.group(1).lower()) in dbg):
                continue
            rows.append(f"{name:40} def {dbg[:70]}; oracle has no self enters-with")
            continue
        word, kind = m.group(1).lower(), m.group(2).lower()
        mk = re.match(r"\((\w+), (.*)\)$", dbg)
        if not mk:
            continue
        dkind, dval = mk.group(1), mk.group(2)
        want_kind = kind_name(kind)
        if want_kind != dkind and not dkind.startswith(want_kind):
            rows.append(f"{name:40} kind {dkind} vs printed {kind!r}")
        # "with a +1/+1 counter on it for each …" / "… if it was kicked" /
        # "for each time it was kicked": the article is not the amount.
        rest = oracle[m.end():m.end() + 120].lower()
        scaled = re.match(r"\s*(?:for each|equal to|if |unless |that many|where x|plus)", rest) or \
            re.search(r"for each|equal to|where x|kicked|converge|that many", rest.split(".")[0])
        if word in WORDS and not (scaled and not dval.startswith("Const(")):
            # An `IfPred` whose either branch is the printed number: "If you
            # do, it enters with …" (Undead Sprinter) / "It escapes with
            # twelve … instead" (Polukranos).
            if dval != f"Const({WORDS[word]})" and not (
                    dval.startswith("IfPred") and f"Const({WORDS[word]})" in dval):
                rows.append(f"{name:40} amount {dval[:60]} vs printed {word!r}")
        elif word == "x" and dval.startswith("Const("):
            rows.append(f"{name:40} amount {dval} vs printed X")
    flagged = {r[:40].strip() for r in rows}
    stale = sorted(set(REVIEWED) - flagged)
    rows = [r for r in rows if r[:40].strip() not in REVIEWED]
    for r in rows:
        print(r)
    for name in stale:
        print(f"# REVIEWED is STALE: {name}", file=sys.stderr)
    print(f"{len(rows)} rows ({len(stale)} stale reviewed)")
    if "--gate" in sys.argv and (rows or stale):
        sys.exit(1)


main()
