#!/usr/bin/env python3
"""Print oracle text for cards from the local scryfall cache (offline)."""
import sys, json
d=json.load(open('scripts/.scryfall_cache.json'))
for name in sys.argv[1:]:
    c=d.get(name)
    if not c:
        # try case-insensitive / front face
        for k in d:
            if k.lower()==name.lower() or k.split(' // ')[0].lower()==name.lower():
                c=d[k]; break
    if not c:
        print(f"=== {name} ===\nNOT CACHED\n"); continue
    pt=f" [{c.get('power')}/{c.get('toughness')}]" if c.get('power') is not None else ""
    loy=f" <{c.get('loyalty')}>" if c.get('loyalty') else ""
    print(f"=== {c['name']} === {c.get('mana_cost','')} | {c.get('type_line','')}{pt}{loy}")
    print(c.get('oracle_text','') or '')
    print()
