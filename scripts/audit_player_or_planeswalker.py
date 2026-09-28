#!/usr/bin/env python3
""""Combat damage to a player or planeswalker" cards that fire on players only
(CR 510.2), and "attacks a player" cards that fire on a planeswalker attack
(CR 506.2).

Every card whose oracle text says "deals combat damage to a player or
planeswalker" must carry an `EventKind::DealsCombatDamageToPlaneswalker`
trigger beside its player one (Bladewing, Dreadhorde Butcher, Psychic Frog
fired on players only). The factory body is found by the card's quoted name
in `crabomination_catalog/src`. A "whenever this creature attacks a player" /
"one or more … attack a player" body must name the player side
(`IsAttackingOpponentPlayer`, `on_attack_player`, `YouAttackedPlayer`).
`--gate` exits 1 on any row.

    target/debug/dump_cards --effects > /tmp/ef.tsv
    python3 scripts/audit_player_or_planeswalker.py /tmp/ef.tsv [--gate]
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")
ATTACKS = re.compile(r"whenever (?:~|this creature|one or more [^,.]{0,40}?) attacks? (?:a player|an opponent)\b"
                     r"|whenever you attack (?:a player|an opponent) with", re.I)
PLAYER_SIDE = re.compile(r"IsAttackingOpponentPlayer|on_attack_player|YouAttackedPlayer|AttackersOfPlayerMatching")
TEXT = re.compile(r"combat damage to a player or (?:a )?planeswalker|combat damage to an opponent or (?:a )?planeswalker", re.I)


def bodies(names):
    found = {}
    for root, _, files in os.walk(os.path.join(ROOT, "crabomination_catalog", "src")):
        for f in files:
            if not f.endswith(".rs"):
                continue
            s = open(os.path.join(root, f), encoding="utf-8").read()
            for n in names:
                if n in found:
                    continue
                i = s.find('"' + n.replace('"', '\\"') + '"')
                if i < 0:
                    continue
                a = s.rfind("pub fn", 0, i)
                b = s.find("\npub fn", i)
                found[n] = s[a:b if b > 0 else len(s)]
    return found


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not args:
        sys.exit(__doc__)
    cache = {k.lower(): v for k, v in json.load(open(CACHE)).items() if isinstance(v, dict)}
    names = {line.split("\t", 1)[0] for line in open(args[0], encoding="utf-8")}
    hits = [n for n in names if TEXT.search((cache.get(n.lower()) or {}).get("oracle_text") or "")]
    src = bodies(hits)
    rows = sorted(n for n in hits if n in src and "DealsCombatDamageToPlaneswalker" not in src[n])
    def oracle(n):
        c = cache.get(n.lower()) or {}
        return (c.get("oracle_text") or "").replace(n, "~").replace(n.split(",")[0], "~")
    attackers = [n for n in names if ATTACKS.search(oracle(n))]
    src.update(bodies([n for n in attackers if n not in src]))
    rows += sorted(f"{n}\t(attacks a player)" for n in attackers if n in src and not PLAYER_SIDE.search(src[n]))
    print("\n".join(rows))
    print(f"# {len(rows)} rows ({len(hits)} damage, {len(attackers)} attack cards)", file=sys.stderr)
    if "--gate" in sys.argv and rows:
        sys.exit(1)


if __name__ == "__main__":
    main()
