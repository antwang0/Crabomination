#!/usr/bin/env python3
"""Counter kinds the printed text names vs the kinds the definition uses.

"+1/+1 counter" shipped as `MinusOneMinusOne`, an "oil counter" shipped as
`Charge`, a "-1/-1 counter" card that only ever adds +1/+1 — each is a card
that plays the opposite game. Reads the real definitions (token and granted
bodies included), one field class at a time (TODO NEXT, Commander item 13):

    cargo build -p crabomination --bin dump_cards
    target/debug/dump_cards --grep Counter > /tmp/counters.tsv
    python3 scripts/audit_counter_kinds.py /tmp/counters.tsv           # all cards
    python3 scripts/audit_counter_kinds.py /tmp/counters.tsv --pod     # pod decks only

A row lists the kinds the oracle names but the definition never mentions
("missing") and the P/T kinds the definition uses but the oracle never names
("extra"). Named kinds are matched by CamelCase ("oil counter" -> `Oil`).
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")
DECKS = os.path.join(ROOT, "crabomination", "src", "pod", "decks.rs")

PT = {
    "+1/+1": "PlusOnePlusOne", "-1/-1": "MinusOneMinusOne", "-0/-1": "MinusZeroMinusOne",
    "-1/-0": "MinusOneMinusZero", "+1/+0": "PlusOnePlusZero", "+0/+1": "PlusZeroPlusOne",
    "+2/+0": "PlusTwoPlusZero", "+0/+2": "PlusZeroPlusTwo", "+2/+2": "PlusTwoPlusTwo",
    "-0/-2": "MinusZeroMinusTwo",
}
# Keyword counters (CR 122.1b) are keyword grants, not `CounterType`s.
KEYWORDS = {"flying", "first strike", "double strike", "deathtouch", "hexproof", "indestructible",
            "lifelink", "menace", "reach", "trample", "vigilance", "haste", "shield", "stun",
            "decayed"}
NAMED = re.compile(r"\b([a-z][a-z\-]*) counters?\b")
SKIP_WORDS = {"a", "the", "of", "those", "that", "each", "any", "one", "two", "three", "x", "more",
              "additional", "many", "those", "same", "kind", "other", "all", "no", "remove",
              "number", "twice", "ten", "five", "four", "six", "seven", "eight", "nine", "type",
              "with", "and", "or", "put", "loyalty", "keyword", "its", "their", "these",
              "fewer", "total", "instead", "several", "different", "double", "doubles",
              "on", "from", "has", "have", "had", "per", "an", "as", "ability", "counter"}


def snake(name):
    s = name.split(" // ")[0].lower().replace("'", "").replace(",", "")
    return re.sub(r"[^a-z0-9]+", "_", s).strip("_")


def camel(word):
    return "".join(p.capitalize() for p in word.split("-"))


def pod_idents():
    src = open(DECKS, encoding="utf-8").read()
    return set(re.findall(r"\b([a-z][a-z0-9_]+)\b", src))


def oracle_of(card):
    text = card.get("oracle_text") or ""
    for f in card.get("card_faces") or []:
        text += "\n" + (f.get("oracle_text") or "")
    return re.sub(r"\([^)]*\)", "", text)  # reminder text


def main():
    rows = [l.rstrip("\n").split("\t", 1) for l in open(sys.argv[1], encoding="utf-8") if "\t" in l]
    pod = pod_idents() if "--pod" in sys.argv else None
    cache = json.load(open(CACHE, encoding="utf-8"))
    lower = {k.lower(): v for k, v in cache.items()}
    out = []
    for name, dbg in rows:
        if pod is not None and snake(name) not in pod:
            continue
        card = cache.get(name) or lower.get(name.lower())
        if not isinstance(card, dict):
            continue
        oracle = oracle_of(card)
        said = {v for k, v in PT.items() if k + " counter" in oracle}
        for m in NAMED.finditer(oracle.lower()):
            w = m.group(1)
            if w in SKIP_WORDS or w in KEYWORDS or w.startswith(("+", "-")):
                continue
            said.add(camel(w))
        used = set(re.findall(r"\b(?:CounterType::)?(" + "|".join(PT.values()) + r")\b", dbg))
        missing = sorted(k for k in said if not re.search(r"\b" + k + r"\b", dbg))
        extra = sorted(used - said)
        if missing or extra:
            out.append(f"{name}\tmissing={','.join(missing)}\textra={','.join(extra)}")
    print("\n".join(sorted(out)))
    print(f"# {len(out)} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
