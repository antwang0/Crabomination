#!/usr/bin/env python3
"""Flag `Effect::ExileTopAndGrantMayPlay` sites that grant a FREE cast
(`pay_own_cost: false`, `pay_any_color: false`) on a card whose Oracle text
never says "without paying" — the impulse-draw class that cast for nothing.

Offline: reads scripts/.scryfall_cache.json. Exit status 1 when any site is
flagged; `ALLOW` lists the fns whose free grant is conditional or correct but
whose text the name heuristic can't see.
"""
import glob, json, re, sys

ALLOW = {"caves_of_chaos_adventurer"}  # free only once a dungeon is done
cache = {k.lower(): v for k, v in json.load(open("scripts/.scryfall_cache.json")).items()}


def oracle(name):
    c = cache.get(name.lower())
    if not c:
        return ""
    faces = c.get("card_faces") or []
    return (c.get("oracle_text") or "") + " ".join(f.get("oracle_text", "") for f in faces)


bad = []
for f in glob.glob("crabomination_catalog/src/**/*.rs", recursive=True):
    s = open(f).read()
    for m in re.finditer(r"ExileTopAndGrantMayPlay\s*\{", s):
        i, depth = m.end(), 1
        while depth and i < len(s):
            depth += {"{": 1, "}": -1}.get(s[i], 0)
            i += 1
        blk = s[m.start():i]
        if "pay_own_cost: false" not in blk or "pay_any_color: true" in blk:
            continue
        fn_at = s.rfind("pub fn ", 0, m.start())
        fn = re.match(r"pub fn (\w+)", s[fn_at:]).group(1)
        names = re.findall(r'"([^"]+)"', s[fn_at:i + 2000])
        if fn in ALLOW or any("without paying" in oracle(n) for n in names):
            continue
        bad.append(f"{f}:{s.count(chr(10), 0, m.start()) + 1} {fn}")
print("\n".join(bad) or "ok: every free impulse grant says 'without paying'")
sys.exit(1 if bad else 0)
