#!/usr/bin/env python3
"""Invented triggers, read off the DUMPED definition (exploratory).

`audit_invented_trigger.py` reads a factory's own literal and skips helpers on
purpose — so a helper-built trigger the card never prints (Eccentric
Apprentice shipped `magecraft_self_pump`) is invisible to it. This reads the
`Debug` dump instead: every `EventKind` in the definition must have its
(lenient) phrase somewhere in the printed text, every face, reminder text
kept. A row names an event kind the card never mentions.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_invented_trigger_dump.py /tmp/all.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE  # noqa: E402

PHRASE = {
    "SpellCast": r"\bcast|\bcopy|magecraft|storm|prowess",
    "EntersBattlefield": r"\benter|\betb|landfall|constellation|alliance|evolve|fabricate|exploit|backup|tribute|bolster|devour|amass|explore|training|squad|celebration|eerie",
    "CreatureDied": r"\bdie|dies|died|graveyard|afterlife|persist|undying|morbid",
    "PermanentDied": r"\bdie|dies|graveyard",
    "Attacks": r"\battack|exert|myriad|melee|annihilator|battle cry|mentor|dethrone|raid|bushido|flanking|rampage|afflict|training|enlist|saddle|provoke",
    "Blocks": r"\bblock|bushido|rampage",
    "BecomesBlocked": r"\bblock|rampage|bushido|flanking",
    "DealsCombatDamageToPlayer": r"combat damage|damage to (a|an|that|each) (player|opponent)|ninjutsu|poisonous|ingest|renown|toxic",
    "LandPlayed": r"\bland|landfall",
    "CardDrawn": r"\bdraw",
    "LifeGained": r"\blife",
    "DealtDamage": r"\bdamage|enrage",
    "Tapped": r"\btap",
    "CardDiscarded": r"\bdiscard|madness",
    "CardCycled": r"cycl",
    "PermanentSacrificed": r"sacrific",
    "BecameTarget": r"\btarget|ward|heroic|valiant|inspired",
    "TurnedFaceUp": r"face up|morph|disguise|manifest|cloak",
    "Transformed": r"transform|daybound|nightbound|disturb",
    "CounterAdded": r"counter",
    "PermanentLeavesBattlefield": r"leaves|leave the battlefield|exile|until",
}


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    pod = None
    if "--pod" in sys.argv:
        path = sys.argv[sys.argv.index("--pod") + 1]
        pod = {ln.strip() for ln in open(path, encoding="utf-8")}
    out = []
    for line in open(sys.argv[1], encoding="utf-8"):
        if "\t" not in line:
            continue
        name, dbg = line.rstrip("\n").split("\t", 1)
        card = cache.get(name)
        if not isinstance(card, dict) or (pod is not None and name not in pod):
            continue
        text = (card.get("oracle_text") or "") + "".join(
            "\n" + (f.get("oracle_text") or "") for f in card.get("card_faces") or [])
        text += " " + " ".join(card.get("keywords") or [])
        for kind in sorted(set(re.findall(r"event: EventSpec \{ kind: (\w+)", dbg))):
            ph = PHRASE.get(kind)
            if ph and not re.search(ph, text, re.I):
                out.append(f"{name}\t{kind}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
