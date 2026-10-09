#!/usr/bin/env python3
"""Effects a card's body runs that its Oracle text never names (exploratory).

For each catalog factory whose own `name:` slugs to the factory name, take
the body up to its closing brace (not the private helpers after it) and flag
an `Effect::<Verb>` whose Oracle text carries none of the verb's words: a
Scry on a card that never scries, a Draw on one that only "puts that card
into your hand", a Destroy where Oracle gives -X/-X. 2026-10-09 it found
Strangle's and Fell's invented surveils, Brilliant Plan's scry, Ulcerate's
destroy, Lotus Cobra's Treasure, Slickshot Show-Off's draw and Dark
Confidant's draw. A row is a lead: read the card (and check the Scryfall API
for errata) before believing it.

    python3 scripts/audit_effect_oracle.py            # every row
    python3 scripts/audit_effect_oracle.py --verb Draw
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VERBS = {
    "Draw": r"\bdraws?\b",
    "Scry": r"\bscry|scries",
    "Surveil": r"surveil",
    "Mill": r"\bmill",
    "Discard": r"discard",
    "Destroy": r"destroy",
    "GainLife": r"\bgains?\b",
    "Proliferate": r"proliferate",
    "Investigate": r"investigate",
    "Explore": r"explore",
    "Fight": r"fight",
    "Regenerate": r"regenerat",
    "CreateToken": r"token|create|populate|amass|investigate|fabricate|incubate|myriad|"
    r"embalm|eternalize|encore|offspring|squad|for mirrodin|living weapon|afterlife|role|copy|copies",
}


def oracle_index():
    d = json.load(open(ROOT / "scripts" / ".scryfall_cache.json"))
    low = {}
    for k, v in d.items():
        if not isinstance(v, dict):
            continue
        faces = v.get("card_faces") or []
        low[k.lower()] = v.get("oracle_text") or " ".join(f.get("oracle_text", "") for f in faces)
        for f in faces:
            low.setdefault(f["name"].lower(), f.get("oracle_text", ""))
    return low


def slug(n):
    return re.sub(r"[^a-z0-9]+", "_", re.sub(r"[’'\",.!]", "", n.lower())).strip("_")


def own_body(text, start):
    """From `pub fn` to the brace that closes it."""
    i = text.index("{", start)
    depth = 0
    for j in range(i, len(text)):
        if text[j] == "{":
            depth += 1
        elif text[j] == "}":
            depth -= 1
            if depth == 0:
                return text[start : j + 1]
    return text[start:]


def main():
    only = sys.argv[sys.argv.index("--verb") + 1] if "--verb" in sys.argv else None
    low = oracle_index()
    rows = []
    for f in sorted((ROOT / "crabomination_catalog" / "src").rglob("*.rs")):
        t = f.read_text(encoding="utf-8")
        for m in re.finditer(r"\npub fn ([a-z0-9_]+)\(\) -> CardDefinition", t):
            fn = m.group(1)
            code = re.sub(r"//[^\n]*", "", own_body(t, m.start()))
            names = [n for n in re.findall(r'name: "([^"]+)"', code) if slug(n) == fn]
            if not names:
                continue
            oracle = low.get(names[0].lower())
            if not oracle:
                continue
            for verb, need in VERBS.items():
                if only and verb != only:
                    continue
                pat = rf"Effect::{verb}\s*\{{[^}}]*?" + (r"amount: Value::Const\((?!0\))" if verb == "Draw" else "")
                if re.search(rf"Effect::{verb}\b", code) and (verb != "Draw" or re.search(pat, code) or "Value::ONE" in code) \
                        and not re.search(need, oracle, re.I):
                    rows.append((verb, fn, f.relative_to(ROOT), oracle[:110].replace("\n", " ")))
    for r in rows:
        print(" | ".join(map(str, r)))
    print(f"# {len(rows)} rows")


if __name__ == "__main__":
    main()
