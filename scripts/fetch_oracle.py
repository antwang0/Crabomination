#!/usr/bin/env python3
"""Fetch oracle text for named cards from Scryfall; print full detail.

`--rulings` also prints each card's Gatherer rulings, one request per card.
The offline cache (`scripts/.scryfall_cache.json`) carries oracle text only,
so a question the printed line cannot answer has to come through here.
"""
import os, ssl, sys, json, urllib.request

# The sandbox routes HTTPS through an agent proxy whose CA bundle isn't in
# Python's default store; point at it when it exists (same as set_gaps.py).
_CA = "/root/.ccr/ca-bundle.crt"
_CTX = ssl.create_default_context(cafile=_CA) if os.path.exists(_CA) else None

_HDRS = {"User-Agent": "crab/1.0", "Accept": "application/json"}
args = sys.argv[1:]
rulings = "--rulings" in args
names = [a for a in args if not a.startswith("--")]
ids = [{"name": n} for n in names]
body = json.dumps({"identifiers": ids}).encode()
req = urllib.request.Request("https://api.scryfall.com/cards/collection",
    data=body, headers={**_HDRS, "Content-Type": "application/json"})
with urllib.request.urlopen(req, timeout=30, context=_CTX) as r:
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
    if rulings:
        rq = urllib.request.Request(c["rulings_uri"], headers=_HDRS)
        with urllib.request.urlopen(rq, timeout=30, context=_CTX) as r:
            for x in json.load(r).get("data", []):
                print(f"  RULING {x['published_at']}  {x['comment']}")
    print()
for nf in d.get("not_found", []):
    print("NOT FOUND:", nf)
