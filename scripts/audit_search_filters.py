#!/usr/bin/env python3
"""Printed "search your library for a[n] X card" vs the definition's search filters.

Reads every `Search*` effect's `filter:` (the card's whole `Debug`, token
bodies included) against each oracle search phrase: a printed card type /
supertype / subtype / mana-value cap the filters never mention, or a filter
restriction (basic, creature, ...) no printed phrase carries.

    target/debug/dump_cards --grep Search > /tmp/search.tsv
    python3 scripts/audit_search_filters.py /tmp/search.tsv [--pod NAMES] [--gate]
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")
TYPES = {
    "creature": ("Creature",),
    "artifact": ("Artifact",),
    "enchantment": ("Enchantment",),
    "planeswalker": ("Planeswalker",),
    "instant": ("Instant",),
    "sorcery": ("Sorcery",),
    "land": ("Land", "HasLandType", "IsBasicLand", "IsNonbasicLand"),
    "battle": ("Battle",),
}
SEARCH = re.compile(r"\bSearch(?:UpToN|AnyNumber|Zones|PickedBy)? \{")
# Read by hand: a "basic land type" / self-name / count clause the textual
# match misreads, or a documented approximation (Tallowisp: any Aura, not
# "with enchant creature" — INCOMPLETE_CARDS).
ALLOW = {
    "Boseiju, Who Endures", "Perilous Forays", "Sprouting Goblin", "Celebrate the Harvest",
    "Harvest Season", "Rampant Rejuvenator", "Traverse the Outlands", "Kyoshi Island Plaza",
    "Enigmatic Incarnation", "Kasmina, Enigma Sage", "Pack Hunt", "Sisay, Weatherlight Captain",
    "Smuggler Captain", "Yisan, the Wanderer Bard", "Brightglass Gearhulk", "Tallowisp",
}


def filters(dbg):
    """Each `filter: …` value inside a `Search*` body, balanced to its comma."""
    out = []
    for m in SEARCH.finditer(dbg):
        i = dbg.find("filter: ", m.end())
        if i < 0:
            continue
        i += len("filter: ")
        depth, j = 0, i
        while j < len(dbg):
            c = dbg[j]
            if c in "([{":
                depth += 1
            elif c in ")]}":
                if depth == 0:
                    break
                depth -= 1
            elif c == "," and depth == 0:
                break
            j += 1
        out.append(dbg[i:j])
    return out


def phrases(oracle):
    """The noun phrase of each "search your library for …" clause."""
    oracle = re.sub(r"\([^)]*\)", "", oracle)
    out = []
    for m in re.finditer(r"[Ss]earch(?:es)? (?:your|their|its owner's|that player's|target \w+'s) "
                         r"library(?: and(?:/or)? graveyard)?(?: and(?:/or)? [a-z ]+?)? for ([^.]*?)"
                         r"(?:, (?:put|reveal|exile|then|and)| and put| and reveal| and exile| then|\.|$)", oracle):
        out.append(m.group(1).strip())
    return out


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not args:
        sys.exit(__doc__)
    pod = None
    if "--pod" in sys.argv:
        pod = {l.strip() for l in open(sys.argv[sys.argv.index("--pod") + 1]) if l.strip()}
        args = [a for a in args if a != sys.argv[sys.argv.index("--pod") + 1]]
    cache = {k: v for k, v in json.load(open(CACHE)).items() if isinstance(v, dict)}
    by_name = {}
    for k, v in cache.items():
        by_name.setdefault(k.lower(), v)
        by_name.setdefault(k.split(" // ")[0].lower(), v)
    rows = []
    for line in open(args[0], encoding="utf-8"):
        if "\t" not in line:
            continue
        name, dbg = line.rstrip("\n").split("\t", 1)
        if (pod is not None and name not in pod) or name in ALLOW:
            continue
        card = by_name.get(name.lower())
        if not card:
            continue
        faces = card.get("card_faces") or []
        oracle = card.get("oracle_text") or "\n".join(f.get("oracle_text", "") for f in faces)
        ph = phrases(oracle)
        fs = filters(dbg)
        if not ph or not fs:
            continue
        allf = " ".join(fs)
        text = " ".join(ph).lower()
        probs = []
        for word, needles in TYPES.items():
            printed = re.search(r"\b" + word + r"s?\b", text)
            # "a card with the same name", "nonland", "noncreature": skip negations.
            neg = re.search(r"\bnon" + word, text)
            has = any(n in allf for n in needles) or f"HasCardType({needles[0]})" in allf
            if printed and not neg and not has:
                probs.append(f"printed {word}, no filter")
        if "basic" in text and "IsBasicLand" not in allf and "Basic" not in allf:
            probs.append("printed basic, no Basic filter")
        if "IsBasicLand" in allf and "basic" not in text:
            probs.append("filter basic, not printed")
        if "legendary" in text and "Legendary" not in allf:
            probs.append("printed legendary, no filter")
        for n in re.findall(r"mana value (\d+) or less", text):
            if f"ManaValueAtMost({n})" not in allf:
                probs.append(f"printed mv<={n}, no ManaValueAtMost({n})")
        for n in re.findall(r"mana value (\d+)(?! or)", text):
            if f"ManaValueExactly({n})" not in allf:
                probs.append(f"printed mv=={n}, no ManaValueExactly({n})")
        for n in re.findall(r"mana value (\d+) or greater", text):
            if f"ManaValueAtLeast({n})" not in allf:
                probs.append(f"printed mv>={n}, no ManaValueAtLeast({n})")
        # Subtypes: capitalized words in the phrase (Forest, Equipment, Dragon, ...).
        for sub in set(re.findall(r"\b([A-Z][a-z]+)\b", " ".join(ph))):
            if sub in {"Search", "Put", "Reveal", "Then", "You", "It", "That"}:
                continue
            if sub not in allf:
                probs.append(f"printed {sub}, no filter")
        # Reverse: a filter card type no phrase prints (and no "permanent").
        for word, needles in TYPES.items():
            n0 = needles[0]
            if word == "land":
                continue
            inf = re.search(r"\b" + n0 + r"\b", allf)
            if inf and word not in text and "permanent" not in text and "historic" not in text \
                    and not re.search(r"\bnon" + word, text):
                probs.append(f"filter {n0}, not printed")
        if probs:
            rows.append(f"{name:36} {'; '.join(sorted(set(probs)))}  | {' / '.join(ph)[:90]}")
    rows.sort()
    print("\n".join(rows))
    print(f"{len(rows)} rows", file=sys.stderr)
    if "--gate" in sys.argv and rows:
        sys.exit(1)


if __name__ == "__main__":
    main()
