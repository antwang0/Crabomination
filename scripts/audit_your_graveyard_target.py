#!/usr/bin/env python3
"""Printed "target … card from/in your graveyard" whose definition never says
whose graveyard (`InYourGraveyard`, `OwnedByYou`, a `You` zone owner): the
target walk then offers every player's graveyard, and the card goes to the
wrong player's library or hand (CR 400.3). Found by a strict debug pod:
Mystic Sanctuary put an opponent's Valorous Stance into its controller's
library.

    target/debug/dump_cards --grep "" > /tmp/all.tsv
    python3 scripts/audit_your_graveyard_target.py /tmp/all.tsv [--gate]
"""
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

# "target … card(s) from/in your graveyard", the words between naming the card
# (not "target creature, where X is the number of cards in your graveyard").
PRINTED = re.compile(
    r"\btarget (?:(?!where|equal|gets|spell|player|unless|number)[^.;]){0,80}?cards? (?:from|in) your graveyard", re.I
)
# Anything in the definition that pins the card to its controller's graveyard.
# The effects named here act on their controller's own graveyard by definition.
PINNED = re.compile(
    r"InYourGraveyard|OwnedByYou|FromYourGraveyard|ReturnGraveyardCardsToHand|ShuffleGraveyardCardsIntoLibrary"
    r"|PutAnyNumberFromGraveyardOnTop|ExileRandomFromGraveyardWithSource|FinaleOfPromise|who: You, zone: Graveyard"
)
# Cards whose "your graveyard" target is pinned some other way, checked by hand.
ALLOW = set()


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    lower = {k.lower(): v for k, v in cache.items()}
    out = []
    for line in open(sys.argv[1], encoding="utf-8"):
        if "\t" not in line:
            continue
        name, dbg = line.rstrip("\n").split("\t", 1)
        card = lower.get(name.lower())
        if not isinstance(card, dict) or name in ALLOW:
            continue
        if not PRINTED.search(oracle_of(card)):
            continue
        if PINNED.search(dbg):
            continue
        out.append(name)
    for n in sorted(set(out)):
        print(n)
    print(f"# {len(set(out))} cards print a 'from your graveyard' target with nothing pinning the graveyard")
    if "--gate" in sys.argv and out:
        sys.exit(1)


if __name__ == "__main__":
    main()
