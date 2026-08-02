#!/usr/bin/env python3
"""Print oracle text for a newline-separated list of card names (offline cache).

Usage: oracle_file.py <names-file>   — avoids the shell-quoting pain of oracle.py
when the list is long or contains apostrophes.
"""
import sys, json
d = json.load(open('scripts/.scryfall_cache.json'))
lower = {k.lower(): k for k in d}
for line in open(sys.argv[1]):
    name = line.strip()
    if not name:
        continue
    c = d.get(name) or d.get(lower.get(name.lower(), ''))
    if not isinstance(c, dict):
        print(f"=== {name} ===\nNOT CACHED\n")
        continue
    for f in (c.get('card_faces') or [c]):
        pt = f" [{f.get('power')}/{f.get('toughness')}]" if f.get('power') is not None else ""
        loy = f" <{f.get('loyalty')}>" if f.get('loyalty') else ""
        print(f"=== {f.get('name')} === {f.get('mana_cost', '')} | {f.get('type_line', '')}{pt}{loy}")
        print(f.get('oracle_text', '') or '')
    print()
