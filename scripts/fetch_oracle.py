#!/usr/bin/env python3
"""Fetch oracle text for named cards from Scryfall; print full detail."""
import sys, json, urllib.request, time
names = sys.argv[1:]
ids = [{"name": n} for n in names]
body = json.dumps({"identifiers": ids}).encode()
req = urllib.request.Request("https://api.scryfall.com/cards/collection",
    data=body, headers={"User-Agent":"crab/1.0","Accept":"application/json","Content-Type":"application/json"})
with urllib.request.urlopen(req, timeout=30) as r:
    d = json.load(r)
for c in d.get("data", []):
    faces = c.get("card_faces")
    def show(o):
        pt = f" [{o.get('power')}/{o.get('toughness')}]" if o.get('power') is not None else ""
        loy = f" <{o.get('loyalty')}>" if o.get('loyalty') else ""
        print(f"--- {o.get('name')} {o.get('mana_cost','')} | {o.get('type_line','')}{pt}{loy}")
        print(o.get('oracle_text','') or '')
    print(f"=== {c['name']} ===")
    if faces:
        for f in faces: show(f)
    else:
        show(c)
    print()
for nf in d.get("not_found", []):
    print("NOT FOUND:", nf)
