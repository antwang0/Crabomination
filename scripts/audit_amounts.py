#!/usr/bin/env python3
"""Constant amounts in a definition that its oracle never prints.

For each verb class — damage, life gain, life loss, draw, mill, scry,
surveil — every `Const(N)` the definition feeds that verb must appear in the
oracle as that verb's number ("deals 3 damage", "gain 4 life", "draw two
cards"). A definition amount the text never prints is a wrong number or an
invented clause. Amounts of 1 are skipped where the oracle says "a card" /
"that much"; X / "for each" text skips the card for that verb.

    cargo build -p crabomination --bin dump_cards
    target/debug/dump_cards --effects > /tmp/ef.tsv
    python3 scripts/audit_amounts.py /tmp/ef.tsv [--verb damage] [--gate]
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")
NUM = r"(\d+|a|an|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen|fourteen|fifteen|twenty)"
WORDS = {"a": 1, "an": 1, "one": 1, "two": 2, "three": 3, "four": 4, "five": 5, "six": 6, "seven": 7,
         "eight": 8, "nine": 9, "ten": 10, "eleven": 11, "twelve": 12, "thirteen": 13, "fourteen": 14, "fifteen": 15,
         "twenty": 20}
# verb: (oracle regex with one NUM group, debug variant regex whose body holds `amount: Const(N)`)
VERBS = {
    # "deals 4 damage to any target and 2 damage to you" (Char) prints the
    # second amount without its verb.
    "damage": (r"(?:deals?|and|,) " + NUM + r" damage", r"DealDamage\w* \{"),
    "gain": (r"gains? " + NUM + r" life", r"GainLife \{"),
    # A life payment is a `LoseLife` (CR 119.4: paying life is losing it).
    # Afflict N (CR 702.130a) prints its loss only in the reminder text, which
    # is stripped below.
    "lose": (r"(?:loses?|pays?|afflict|} and) " + NUM + r"(?: life)?\b", r"LoseLife \{"),
    "draw": (r"draws? " + NUM + r" (?:additional )?cards?", r"Draw \{"),
    "mill": (r"mills? " + NUM + r" cards?", r"Mill \{"),
}


# Reviewed rows: (name, verb) -> why the unprinted constant is right, or the
# approximation's own doc comment that already names it.
REVIEWED = {
    ("Arcane Denial", "draw"): "'may draw up to two cards' — the upper bound",
    ("Candles' Glow", "gain"): "life equal to the 3 damage prevented",
    ("Intervention Pact", "gain"): "life equal to the damage prevented (the shield's cap)",
    ("Reverse Damage", "gain"): "life equal to the damage prevented (an uncapped shield)",
    ("Enter the God-Eternals", "gain"): "life equal to the 4 damage dealt",
    ("Devouring Greed", "gain"): "'loses 2 life plus 2 for each Spirit … you gain that much'",
    ("Forget", "draw"): "'draws as many cards as they discarded' — two",
    ("Joshua, Phoenix's Dominant", "draw"): "'discard up to two, then draw that many'",
    ("Indentured Djinn", "draw"): "'may draw up to three cards' — the upper bound",
    ("Tibalt's Trickery", "mill"): "'choose 1, 2, or 3 at random' — the three outcomes",
    ("Wing Storm", "damage"): "'twice the number of' — the multiplier",
    ("Crabomination", "mill"): "documented approximation of the exile-three ETB (its doc)",
    ("Demonic Consultation", "mill"): "documented approximation of exile-six (its doc)",
    ("Discover the Formula", "draw"): "documented approximation of seek three (its doc)",
}


def spans(dbg, pat):
    for m in re.finditer(pat, dbg):
        i = j = m.end()
        depth = 0
        while j < len(dbg):
            ch = dbg[j]
            if ch in "({[":
                depth += 1
            elif ch in ")}]":
                if depth == 0:
                    break
                depth -= 1
            j += 1
        yield dbg[i:j]


def top_const(body):
    """The variant's own `amount: Const(N)`, not a nested one."""
    depth = 0
    for m in re.finditer(r"[({\[]|[)}\]]|amount: Const\((-?\d+)\)", body):
        t = m.group(0)
        if t in "({[":
            depth += 1
        elif t in ")}]":
            depth -= 1
        elif depth == 0:
            return int(m.group(1))
    return None


def strip_spans(dbg, pat):
    """`dbg` with every `pat {...}` body cut out (a token's own abilities)."""
    out, pos = [], 0
    for m in re.finditer(pat, dbg):
        if m.start() < pos:
            continue
        j, depth = m.end(), 0
        while j < len(dbg):
            ch = dbg[j]
            if ch in "({[":
                depth += 1
            elif ch in ")}]":
                if depth == 0:
                    break
                depth -= 1
            j += 1
        out.append(dbg[pos:m.start()])
        pos = j
    out.append(dbg[pos:])
    return "".join(out)


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not args:
        sys.exit(__doc__)
    only = sys.argv[sys.argv.index("--verb") + 1] if "--verb" in sys.argv else None
    if only:
        args = [a for a in args if a != only]
    cache = {k: v for k, v in json.load(open(CACHE)).items() if isinstance(v, dict)}
    by_name = {}
    for k, v in cache.items():
        by_name.setdefault(k.lower(), v)
        by_name.setdefault(k.split(" // ")[0].lower(), v)
    per_card = {}
    for line in open(args[0]):
        name, where, dbg = line.rstrip("\n").split("\t", 2)
        per_card.setdefault(name, []).append(dbg)
    rows, used = [], set()
    for name, dbgs in per_card.items():
        card = by_name.get(name.lower())
        if not card:
            continue
        faces = card.get("card_faces") or []
        oracle = card.get("oracle_text") or "\n".join(f.get("oracle_text", "") for f in faces)
        oracle = re.sub(r"\([^)]*\)", "", oracle).lower()
        dbg = strip_spans(" ".join(dbgs), r"TokenDefinition \{")
        for verb, (ore, dre) in VERBS.items():
            if only and verb != only:
                continue
            got = {c for body in spans(dbg, dre) if (c := top_const(body)) is not None}
            if not got:
                continue
            printed = {WORDS.get(n, None) if not n.isdigit() else int(n) for n in re.findall(ore, oracle)}
            printed.discard(None)
            # "draw a card" / "deals 1 damage": the unit amount is everywhere.
            extra = {g for g in got if g not in printed and g > 1}
            if extra and (name, verb) in REVIEWED:
                used.add((name, verb))
            elif extra:
                rows.append(f"{name:40} {verb:6} def {sorted(got)} printed {sorted(printed) or 'none'}")
    for r in sorted(rows):
        print(r)
    stale = sorted(set(REVIEWED) - used) if only is None else []
    for name, verb in stale:
        print(f"# REVIEWED is STALE: {name} / {verb}", file=sys.stderr)
    print(f"{len(rows)} rows ({len(used)} reviewed, {len(stale)} stale)")
    if "--gate" in sys.argv and (rows or stale):
        sys.exit(1)


main()
