#!/usr/bin/env python3
"""A creature token's name is its creature types (CR 111.4): every word of a
token named from `CreatureType` words must be among its `creature_types`.

"Eldrazi Scion" tokens shipped as plain Eldrazi (no Scion to sacrifice to a
Scion payoff), Sporemound's Saprolings as Plants, the Wall / Glimmer /
Shapeshifter tokens with no type at all. Reads the real definitions:

    target/debug/dump_cards --grep "" > /tmp/all.tsv
    python3 scripts/audit_token_types.py /tmp/all.tsv [--gate]
"""

import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
# The printed subtype is not a `CreatureType` yet (adding one touches the
# observation vocabulary): Nesting Dragon's Dragon Egg, Clown Extruder's Clown Robot.
ALLOWLIST = {("Nesting Dragon", "Dragon Egg"), ("Clown Extruder", "Clown Robot")}
TOKEN = re.compile(
    r'TokenDefinition \{ name: "([^"]*)", power: -?\d+, toughness: -?\d+, keywords: \[[^\]]*\], '
    r'card_types: \[([^\]]*)\], colors: \[[^\]]*\], supertypes: \[[^\]]*\], '
    r'subtypes: Subtypes \{ creature_types: \[([^\]]*)\]')


def creature_types():
    src = open(os.path.join(ROOT, "crabomination_base", "src", "card.rs"), encoding="utf-8").read()
    i = src.index("pub enum CreatureType {")
    body = re.sub(r"//[^\n]*", "", src[i:src.index("\n}", i)])
    return set(re.findall(r"\b([A-Z][A-Za-z]+)\b", body)) - {"CreatureType"}


def main():
    ct = creature_types()
    out = set()
    for line in open(sys.argv[1], encoding="utf-8"):
        if "\t" not in line:
            continue
        name, dbg = line.rstrip("\n").split("\t", 1)
        for m in TOKEN.finditer(dbg):
            tname, types, have = m.groups()
            words = [w.replace("-", "") for w in tname.split()]
            if "Creature" not in types or not words or (name, tname) in ALLOWLIST:
                continue
            have = {x.strip() for x in have.split(",") if x.strip()}
            missing = [w for w in words if w in ct and w not in have]
            if all(w in ct for w in words) and missing:
                out.add(f"{name}\ttoken \"{tname}\" lacks {','.join(missing)}")
    print("\n".join(sorted(out)))
    print(f"# {len(out)} rows", file=sys.stderr)
    if "--gate" in sys.argv and out:
        sys.exit(1)


if __name__ == "__main__":
    main()
