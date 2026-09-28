#!/usr/bin/env python3
"""A permanent whose oracle prints a static-ability paragraph but whose
definition holds no static ability and no field that could carry one
(exploratory; CR 604.1). Found Cayth, Bellowing Tanglewurm, Spider-Ham and
The Seriema, each shipping without its grant.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_missing_static.py /tmp/all.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

PERMANENT = re.compile(r"\b(Creature|Artifact|Enchantment|Planeswalker|Land|Battle)\b")
TRIGGER = re.compile(r"^(?:[^—]*— )?(When|Whenever|At |At the)", re.I)
ACTIVATED = re.compile(r"^(?:[^—]*— )?[^\"“]*?:\s")
NOT_STATIC = re.compile(
    r"^(Enchant |Equip|Station|Level up|LEVEL|\d+\+ \||Crew|Reconfigure|Saddle|Prototype|"
    r"Flashback|Kicker|Cycling|Ward|Protection|Chapter|\(|I+V?\b|Partner|Companion|Mutate|Bestow|"
    r"Adventure|Choose one|•|As (this|~) enters|This (creature|land|artifact|enchantment) enters (tapped|with)|"
    r"~ enters (tapped|with)|If (this|~) would|You may cast|You may have this|Craft|Suspend|Morph|Disguise|"
    r"Megamorph|Evoke|Dash|Emerge|Escape|Encore|Unearth|Embalm|Eternalize|Ninjutsu|Transmute|Channel|"
    r"Plot|Foretell|Spectacle|Surge|Prowl|Madness|Miracle|Offspring|Gift|Blitz|Casualty|Squad|Toxic|"
    r"Backup|Bargain|Harmonize|Warp|Exhaust|Max speed|Start your engines|Specialize|Visit|Prize)", re.I)
CARRIERS = re.compile(
    r"static_abilities: \[[^\]]|equipped_bonus: Some|enchant\w*_bonus: Some|aura\w*: Some|"
    r"level_bands: \[[^\]]|station: \[[^\]]")


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
        if card.get("card_faces") or not PERMANENT.search(card.get("type_line", "")):
            continue
        kws = {k.lower() for k in card.get("keywords") or []}
        text = oracle_of(card).replace(name, "~")
        for para in text.split("\n"):
            para = para.strip()
            if not para or TRIGGER.search(para) or ACTIVATED.search(para) or NOT_STATIC.search(para):
                continue
            first = para.split(",")[0].split(" ")[0].lower()
            if any(para.lower().startswith(k) for k in kws) or first in kws:
                continue
            if not CARRIERS.search(dbg):
                out.append(f"{name}\t{para[:110]}")
            break
    print("\n".join(sorted(out)))
    print(f"# {len(out)} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
