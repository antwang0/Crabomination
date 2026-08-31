#!/usr/bin/env python3
"""Does a keyword the card *does* print carry the number the card prints?

`audit_keyword_drift.py` asks whether a mechanic keyword is there at all, and
it is deliberately blind to the payload: `Keyword::Bushido(1)` and
`Keyword::Bushido(2)` are the same row to it. But most mechanic keywords are
"keyword N" or "keyword {cost}", and the parameter is the card — Battle-Mad
Ronin printing Bushido 2 and shipping Bushido 1 is a different creature in
combat, and an Equip that charges the wrong mana is a different card every
turn it is moved.

So this reads the *payload*: every `Keyword::` variant whose argument is a
number or a `cost(&[..])`, against the number or cost the oracle prints after
that word.

    python3 scripts/audit_keyword_value.py           # the table
    python3 scripts/audit_keyword_value.py --rows 0  # every row

⚠ Reminder text is stripped first, or every "Cycling {2} ({2}, Discard this
card: …)" reads twice and a keyword whose reminder happens to name a
different cost reads as a mismatch.
⚠ Only the **first** printing of the word is compared. A card with two
costs for one keyword (Landcycling's per-type costs, a split card's two
halves) is a shape this cannot adjudicate, and its keyword is skipped.
⚠ Multi-face cards are skipped for the same reason `audit_keyword_drift`
skips them: the body here is the front face alone.
"""
import argparse
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CATALOG = ROOT / "crabomination_catalog" / "src"
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from audit_doc_drift import body_cost_symbols  # noqa: E402
from audit_keyword_drift import (  # noqa: E402
    BODY_NAME,
    FN,
    card_literal,
    factory_body,
    oracle_words,
)

# `Keyword::Variant(N)` -> the word printed before the number. Only the
# variants a card actually *prints* as "word N" are here: `Regenerate`,
# `CyclingLife`, `Firebending` and the twenty `CantAttackOrBlockUnless…`
# variants are engine primitives that stand in for spelled-out text, and
# there is no printed word to compare them against.
NUMERIC = {
    "Bushido": "bushido",
    "Absorb": "absorb",
    "Rampage": "rampage",
    "Frenzy": "frenzy",
    "Toxic": "toxic",
    "Poisonous": "poisonous",
    "Modular": "modular",
    "Fading": "fading",
    "Vanishing": "vanishing",
    "Impending": "impending",
    "Dredge": "dredge",
    "Annihilator": "annihilator",
    "Bloodthirst": "bloodthirst",
    "Crew": "crew",
    "Saddle": "saddle",
    "Casualty": "casualty",
}

# `Keyword::Variant(cost)` -> the word printed before the mana cost.
COSTED = {
    "Flashback": "flashback",
    "Kicker": "kicker",
    "Multikicker": "multikicker",
    "Buyback": "buyback",
    "Entwine": "entwine",
    "Cycling": "cycling",
    "Echo": "echo",
    "Morph": "morph",
    "Megamorph": "megamorph",
    "Disguise": "disguise",
    "Disturb": "disturb",
    "Squad": "squad",
    "Replicate": "replicate",
    "Ninjutsu": "ninjutsu",
    "Madness": "madness",
    "Equip": "equip",
    "Reconfigure": "reconfigure",
    "Fortify": "fortify",
    "Offspring": "offspring",
    "Harmonize": "harmonize",
    "Mayhem": "mayhem",
}

# Deliberate modellings, checked once. A card here prints a payload the
# engine's keyword cannot express, and the engine picked the closest one.
ALLOWED = {
    # "Kicker {B} and/or {R}" is two independent kicker costs; the engine's
    # `Multikicker` carries one. It takes the red half, so the black-only
    # kick is unreachable and the both-halves kick costs {R} twice.
    ("archangel_of_wrath", "multikicker"),
}

COST_RUN = r"((?:\{[^}]{1,7}\})+)"
# `Keyword::Equip(cost(&[generic(3)]))` — the payload is the whole `cost(..)`
# call, and `body_cost_symbols` wants what is inside the slice. ⚠ Handing it
# the wrapper makes it return None on *every* costed keyword, which read as
# 640 skipped rows and looked like the auditor simply having a small
# population.
COST_CALL = re.compile(r"^\s*cost\(&\[(.*)\]\)\s*$", re.S)


def payloads(body: str):
    """`[(variant, arg_text)]` for every `Keyword::V(arg)` at the top level."""
    m = re.search(r"^ {8}keywords: vec!\[", body, re.M)
    if not m:
        return []
    i = body.index("[", m.start())
    depth, j = 0, i
    while j < len(body):
        if body[j] in "[({":
            depth += 1
        elif body[j] in "])}":
            depth -= 1
            if depth == 0:
                break
        j += 1
    text = body[i : j + 1]
    out = []
    for mm in re.finditer(r"Keyword::([A-Za-z]+)\(", text):
        k = mm.end()
        depth, start = 1, k
        while k < len(text) and depth:
            if text[k] == "(":
                depth += 1
            elif text[k] == ")":
                depth -= 1
            k += 1
        out.append((mm.group(1), text[start : k - 1]))
    return out


def printed_once(words: str, pattern: str):
    """The single match of `pattern`, or None when absent or ambiguous."""
    hits = re.findall(pattern, words)
    if len(hits) != 1:
        return None
    return hits[0]


def spell(symbols) -> str:
    return "".join("{" + s + "}" for s in symbols)


def scan():
    rows, checked = [], 0
    for path in sorted(CATALOG.rglob("*.rs")):
        src = path.read_text(encoding="utf-8")
        for m in FN.finditer(src):
            body = factory_body(src, m.end())
            bn = BODY_NAME.search(body)
            if not bn:
                continue
            words = oracle_words(bn.group(1))
            if words is None:
                continue
            body = card_literal(body, bn.start())
            for variant, arg in payloads(body):
                if variant in NUMERIC:
                    word = NUMERIC[variant]
                    if not arg.strip().isdigit():
                        continue
                    got = printed_once(words, rf"\b{word} (\d+)\b")
                    if got is None:
                        continue
                    checked += 1
                    if int(got) != int(arg) and (m.group(1), word) not in ALLOWED:
                        rows.append((m.group(1), path.relative_to(ROOT),
                                     f"{word} {arg} vs printed {word} {got}"))
                elif variant in COSTED:
                    word = COSTED[variant]
                    # `Multikicker` prints as "kicker … any number of times".
                    probe = "kicker" if word == "multikicker" else word
                    call = COST_CALL.match(arg)
                    if not call:
                        continue
                    syms = body_cost_symbols(call.group(1))
                    if syms is None:
                        continue
                    got = printed_once(words, rf"\b{probe}\b[ —–-]*" + COST_RUN)
                    if got is None:
                        continue
                    checked += 1
                    # ⚠ `oracle_words` lower-cases the whole text, so the
                    # printed `{W}` comes back `{w}`. Comparing without this
                    # made every single costed keyword a finding.
                    printed = [s.upper() for s in re.findall(r"\{([^}]{1,7})\}", got)]
                    if sorted(syms) != sorted(printed) and (m.group(1), word) not in ALLOWED:
                        rows.append((m.group(1), path.relative_to(ROOT),
                                     f"{word} {spell(syms)} vs printed {word} {got}"))
    return checked, rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rows", type=int, default=40)
    args = ap.parse_args()
    checked, rows = scan()
    print(f"=== keyword payloads: {len(rows)} wrong of {checked} comparable")
    shown = rows if args.rows == 0 else rows[: args.rows]
    for name, path, why in shown:
        print(f"  {name:<42} {why}")
        print(f"  {'':<42} {path}")
    if len(shown) < len(rows):
        print(f"  … {len(rows) - len(shown)} more (--rows 0)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
