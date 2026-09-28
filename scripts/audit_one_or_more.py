#!/usr/bin/env python3
"""Printed "whenever one or more …" (CR 603.2c) vs a trigger with no batch key.

A per-event trigger on a batch wording fires once per object: Ketramose drew a
card per exiled card, Deeproot Pilgrimage made a Merfolk per tapped Merfolk,
Quartzwood Crasher a Dinosaur per trampler. Accepted: an `EventSpec` with
`once_per_batch` / `once_per_turn` / `batch_across_players`, or a row below
whose event is already one per batch (a counter placement, the attack
declaration, a tally event) or whose repeat is harmless (idempotent body).
`--gate` exits 1 on any row.

    target/debug/dump_cards --grep "" > /tmp/all.tsv
    python3 scripts/audit_one_or_more.py /tmp/all.tsv [--gate]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

PHRASE = re.compile(
    r"[Ww]henever (?:one or more ([^,]*)"
    r"|(?:you|a player|an opponent) (?:discard|sacrifice|attacks?|attack with|roll|mill|get)"
    r" (?:[^,.]{0,30} )?one or more ([^,.]*))"
)
# `CounterAdded` / `CounterRemoved` is one event per placement.
COUNTERS = re.compile(r"counters? (are|is) (put|removed)")
BATCH = re.compile(
    r"once_per_batch: true|batch_across_players: true|once_per_turn: true"
    # Event kinds that are one per batch by construction.
    r"|kind: (?:YouAttack|DiscardedOneOrMore|RolledDice|EnergyGained)\b"
)
# One event per batch already, or a repeat that changes nothing.
ALLOWLIST = {
    # `YouAttack` / `YouAttackedPlayer` fire once per declaration / defender.
    "Demonic Covenant", "Duelist's Heritage", "Firkraag, Cunning Instigator",
    "Glass-Cast Heart", "Mavren Fein, Dusk Apostle", "Roar of Resistance",
    "Ancestor Dragon", "Clandestine Meddler", "Teo, Spirited Glider",
    "The Earth King", "Dollmaker's Shop // Porcelain Gallery",
    # Per attacker is the batch: "1 life for each attacking creature",
    # "those creatures get -1/-0", "twice that many rad counters".
    "Orim's Prayer", "Sabotage Strategist", "Struggle for Project Purity",
    # "Tap those creatures and put a stun counter on each of them".
    "Tamiyo, Upriser Crowned",
    # Per discarded card, "that many" sums to the batch.
    "Cryptcaller Chariot", "Marauding Mako", "Scrounging Skyray",
    # `BlocksNOrMore` is one event per declaration.
    "Tide of War",
    # `CardsExiledFromHandOrBy` is the dispatcher's per-batch tally.
    "Hero of Bretagard", "Ranar the Ever-Watchful",
    # "That many" per event sums to the batch; "becomes prepared" and a
    # self-transform / a return of this card from the graveyard repeat as
    # no-ops.
    "Woodland Champion", "Kirol, History Buff", "Ajani, Nacatl Pariah",
    "Killian's Confidence", "Pyrewild Shaman",
    # "Put them onto the battlefield" — per card is the same move.
    "Hedge Shredder",
    # Once-a-turn via `Not(SourceDoneThisTurn)` in the filter.
    "Deep Gnome Terramancer",
    # Static / delayed wordings with no EventSpec of their own.
    "Frontier Warmonger", "Forth Eorlingas!", "City in a Bottle",
    "Magmatic Galleon",
    # 🟡 "choose a creature card from among them": per card, the last copy
    # wins (INCOMPLETE_CARDS).
    "Kaya, Spirits' Justice",
}


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    out = []
    for line in open(sys.argv[1], encoding="utf-8"):
        if "\t" not in line:
            continue
        name, dbg = line.rstrip("\n").split("\t", 1)
        card = cache.get(name)
        if not isinstance(card, dict) or name in ALLOWLIST:
            continue
        hits = [a or b for a, b in PHRASE.findall(oracle_of(card))]
        hits = [h for h in hits if not COUNTERS.search(h)]
        if hits and len(BATCH.findall(dbg)) < len(hits):
            out.append(f"{name}\t{hits[0][:100]}")
    for row in sorted(out):
        print(row)
    print(f"{len(out)} rows", file=sys.stderr)
    if "--gate" in sys.argv and out:
        sys.exit(1)


if __name__ == "__main__":
    main()
