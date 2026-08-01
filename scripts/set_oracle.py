#!/usr/bin/env python3
"""Dump oracle text for a whole Scryfall set (GET-only; POST is blocked by the
agent proxy). Usage: set_oracle.py <setcode> [name-filter-file]"""
import sys, json, time, urllib.request, urllib.parse
def fetch(code):
    cards=[]; url="https://api.scryfall.com/cards/search?"+urllib.parse.urlencode(
        {"q":f"set:{code} -is:reprint","unique":"cards","order":"name"})
    while url:
        req=urllib.request.Request(url,headers={"User-Agent":"crab/1.0","Accept":"application/json"})
        with urllib.request.urlopen(req,timeout=20) as r: d=json.load(r)
        cards+=d.get("data",[]); url=d.get("next_page"); time.sleep(0.1)
    return cards
code=sys.argv[1]
want=None
if len(sys.argv)>2:
    want={l.strip() for l in open(sys.argv[2]) if l.strip()}
for c in fetch(code):
    if want is not None and c["name"] not in want: continue
    def show(o):
        pt=f" [{o.get('power')}/{o.get('toughness')}]" if o.get('power') is not None else ""
        loy=f" <{o.get('loyalty')}>" if o.get('loyalty') else ""
        print(f"--- {o.get('name')} {o.get('mana_cost','')} | {o.get('type_line','')}{pt}{loy}")
        print(o.get('oracle_text','') or '')
    print(f"=== {c['name']} ===")
    for f in (c.get("card_faces") or [c]): show(f)
    print()
