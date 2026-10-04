#!/usr/bin/env python3
"""Printed target count ("up to two target", "three target") vs the spell's
declared target slots.

Non-modal instants and sorceries only: the printed count is the sum over each
"target" word of the number word before it ("up to two target creatures" is
2, a bare "target creature" is 1). The definition's count is the spell
effect's `target_slot_count`; "up to" also wants the slots optional
(`min_targets_in_mode` 0).

    cargo build -p crabomination --bin dump_cards
    target/debug/dump_cards --targets > /tmp/tg.tsv
    python3 scripts/audit_target_counts.py /tmp/tg.tsv [--gate]
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")
WORDS = {**{str(n): n for n in range(1, 11)}, "one": 1, "two": 2, "three": 3, "four": 4, "five": 5, "six": 6, "seven": 7, "eight": 8,
         "nine": 9, "ten": 10}


# Reviewed rows: a non-target choice the printed text makes, or a documented
# approximation (the card's own doc names it).
REVIEWED = {
    "Volcanic Offering": "two of its four targets are an opponent's choice",
    "Cytoshape": "'choose a nonlegendary creature' is modelled as a second slot",
    "Enchantment Alteration": "'another permanent of that type' is modelled as a second slot",
    "Backdraft": "'choose a player who cast …' is the one player slot",
    "Clear the Stage": "the conditional 'may return up to one target' is a resolution pick",
    "Rapid Decay": "'up to three target cards from a single graveyard': one-slot approximation",
    "Serene Remembrance": "'up to three target cards from a single graveyard': one-slot approximation",
    "Rite of Renewal": "two- and four-card graveyard targets collapsed: documented approximation",
    "Cathartic Parting": "the 'up to four target cards' shuffle is a resolution pick",
}


def printed_count(oracle):
    """(slots, any_up_to) or None when the text is out of this scan's reach."""
    if re.search(r"any number of|\bX target|choose one|choose two|choose any|•|where X|for each|"
                 r"\bdivided\b|each of (?:them|those)|that target|same target|new target|"
                 r"single target|change the target|one or more target", oracle, re.I):
        return None
    # A granted ability's own targets ("gains '{T}: Return target …'") and a
    # token's printed text are another object's, not this spell's.
    oracle = re.sub(r"\u201c[^\u201d]*\u201d|\"[^\"]*\"", "", oracle)
    # Keep the spell's own text: a cycling / flashback / kicker line is
    # another ability with its own targets.
    lines = [l for l in oracle.split("\n") if not re.match(
        r"(?:When you cycle|Cycling|\w+cycling|Flashback|Buyback|Kicker|Multikicker|Overload|Entwine|Splice|"
        r"Replicate|Escape|Madness|Morph|Channel\b|Forecast|Retrace|Jump-start|Aftermath|Suspend|Rebound|"
        r"Haunt\b|When the creature this card haunts|When you discard|\{[^}]*\}(?:\{[^}]*\})*, Exile this card)", l)]
    # A reflexive "When that permanent enters, …" sentence is a trigger's.
    lines = [re.sub(r"\. When (?:that|this|it)\b[^.]*", "", l) for l in lines]
    if any(re.search(r"kicked|was cast from|overloaded|instead", l) for l in lines):
        return None
    oracle = "\n".join(lines)
    # Support N (CR 702.104a) prints its targets only in the stripped reminder.
    oracle = re.sub(r"\bSupport (\w+)", lambda m: f"up to {m.group(1)} target creatures", oracle)
    total = 0
    players = 0
    up_to = True
    # "each of two targets" / "one or two targets" (Pinnacle of Rage).
    for m in re.finditer(r"\b(?:each of )?(?:(one or )?(\w+)) targets\b", oracle, re.I):
        if m.group(2).lower() in WORDS:
            total += WORDS[m.group(2).lower()]
            up_to &= bool(m.group(1))
    for m in re.finditer(r"(?:(up to )?(\w+) )?(?:other |another |additional )?\btarget\b(?!s|ed)", oracle, re.I):
        word = (m.group(2) or "").lower()
        if re.match(r"\s*(?:player|opponent)", oracle[m.end():]):
            players += 1
        if word in WORDS:
            total += WORDS[word]
            up_to &= bool(m.group(1))
        else:
            total += 1
            up_to = False
    return total, players, up_to and total > 0


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not args:
        sys.exit(__doc__)
    cache = {k: v for k, v in json.load(open(CACHE)).items() if isinstance(v, dict)}
    by_name = {}
    for k, v in cache.items():
        by_name.setdefault(k.lower(), v)
    rows = []
    for line in open(args[0]):
        name, slots, mn, modal = line.rstrip("\n").split("\t")
        card = by_name.get(name.lower())
        if not card or modal == "1" or card.get("card_faces"):
            continue
        oracle = re.sub(r"\([^)]*\)", "", card.get("oracle_text") or "")
        got = printed_count(oracle)
        if not got:
            continue
        want, players, up_to = got
        slots = int(slots)
        if slots and want != slots and want - players != slots:
            rows.append(f"{name:40} slots {slots} vs printed {want}")
        elif up_to and slots and mn not in ("0",):
            rows.append(f"{name:40} printed up to {want}; min {mn}")
    flagged = {r.split("  ")[0].strip() for r in rows}
    stale = sorted(set(REVIEWED) - flagged)
    rows = [r for r in rows if r.split("  ")[0].strip() not in REVIEWED]
    for r in rows:
        print(r)
    for name in stale:
        print(f"# REVIEWED is STALE: {name}", file=sys.stderr)
    print(f"{len(rows)} rows ({len(stale)} stale reviewed)")
    if "--gate" in sys.argv and (rows or stale):
        sys.exit(1)


main()
