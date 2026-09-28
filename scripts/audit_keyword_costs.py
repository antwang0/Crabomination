#!/usr/bin/env python3
"""Printed "<Keyword> {cost}" mana costs vs the definition (exploratory).

Flashback, cycling, kicker, embalm, eternalize, unearth, dash, ninjutsu,
madness, escape, evoke, encore, … — a row names a keyword whose printed mana
cost (as a multiset of symbols) appears nowhere in the definition as a
`ManaCost { symbols: [...] }`. Dreamstealer's eternalize read {5}{B}{B}.

    target/debug/dump_cards --grep "" > /tmp/all.tsv   # --pod NAMES: dump_cards --pod
    python3 scripts/audit_keyword_costs.py /tmp/all.tsv [--pod NAMES]
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of  # noqa: E402

KWS = ("Flashback|Cycling|Kicker|Embalm|Eternalize|Unearth|Dash|Ninjutsu|Madness|Escape|Evoke|Encore|"
       "Blitz|Disturb|Morph|Megamorph|Disguise|Plot|Foretell|Harmonize|Mayhem|Retrace|Jump-start|"
       "Buyback|Overload|Spectacle|Surge|Emerge|Prowl|Bestow|Reconfigure|Crew|Warp|Craft with|Offspring|"
       "Squad|Transmute|Channel|Scavenge|Outlast|Level up|Fortify|Reinforce|Transfigure|Miracle")
PRINTED = re.compile(r"^(" + KWS + r")[ —]+((?:\{[^}]+\})+)", re.M)
COL = {"W": "White", "U": "Blue", "B": "Black", "R": "Red", "G": "Green"}


def symbols(cost):
    out = []
    for s in re.findall(r"\{([^}]+)\}", cost):
        if s.isdigit():
            if s != "0":
                out.append(f"Generic({s})")
        elif s in COL:
            out.append(f"Colored({COL[s]})")
        elif s == "C":
            out.append("Colorless(1)")
        elif s == "X":
            out.append("X")
        elif "/P" in s:
            out.append(f"Phyrexian({COL[s[0]]})")
        elif "/" in s:
            a, b = s.split("/")
            if a.isdigit():
                out.append(f"MonoHybrid({a}, {COL[b]})")
            else:
                out.append(f"Hybrid({COL[a]}, {COL[b]})")
        else:
            out.append(s)
    return sorted(out)


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
        have = []
        for body in re.findall(r"ManaCost \{ symbols: \[([^\]]*)\] \}", dbg):
            syms = []
            for s in re.findall(r"\w+\([^()]*\)|\bX\b", body):
                m = re.fullmatch(r"Colorless\((\d+)\)", s)
                syms += ["Colorless(1)"] * int(m.group(1)) if m else [s]
            have.append(sorted(syms))
        for m in PRINTED.finditer(oracle_of(card)):
            want = symbols(m.group(2))
            if want and want not in have:
                out.append(f"{name}\t{m.group(1)} {m.group(2)}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)


if __name__ == "__main__":
    main()
