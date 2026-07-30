#!/usr/bin/env python3
"""Like set_gaps.py but also prints oracle text/P-T, for implementation work."""
import sys, json, time, urllib.request, urllib.parse, subprocess
def fetch_set(code):
    cards=[]; url="https://api.scryfall.com/cards/search?"+urllib.parse.urlencode({"q":f"set:{code} -is:reprint","unique":"cards","order":"name"})
    while url:
        req=urllib.request.Request(url,headers={"User-Agent":"crab/1.0","Accept":"application/json"})
        with urllib.request.urlopen(req,timeout=20) as r: d=json.load(r)
        cards+=d.get("data",[]); url=d.get("next_page"); time.sleep(0.1)
    return cards
def in_catalog(name):
    return subprocess.run(["grep","-rqF",f'"{name}"',"crabomination_catalog/src"]).returncode==0
code=sys.argv[1]
lo,hi=(int(sys.argv[2]),int(sys.argv[3])) if len(sys.argv)>3 else (0,10**6)
n=0
for c in fetch_set(code):
    nm=c["name"]
    if nm.startswith("A-"): continue
    if in_catalog(nm) or in_catalog(nm.split(" // ")[0]): continue
    n+=1
    if not (lo < n <= hi): continue
    pt=f" [{c.get('power')}/{c.get('toughness')}]" if c.get('power') is not None else ""
    loy=f" <{c.get('loyalty')}>" if c.get('loyalty') else ""
    print(f"=== {nm} === {c.get('mana_cost','')} | {c.get('type_line','')}{pt}{loy} | {c.get('rarity','')}")
    print((c.get('oracle_text','') or '').strip())
    print()
