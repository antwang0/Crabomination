#!/usr/bin/env python3
"""List cards from a Scryfall set search not yet present in the catalog."""
import sys, json, urllib.parse, urllib.request, time, re
from pathlib import Path
REPO = Path(__file__).resolve().parent.parent

def search(q):
    out=[]
    url="https://api.scryfall.com/cards/search?"+urllib.parse.urlencode({"q":q})
    while url:
        req=urllib.request.Request(url, headers={"User-Agent":"crabo/1.0","Accept":"application/json"})
        data=json.load(urllib.request.urlopen(req, timeout=30))
        out.extend(data.get("data",[]))
        url=data.get("next_page")
        time.sleep(0.1)
    return out

def defined_names():
    names=set()
    for p in (REPO/"crabomination_catalog/src").rglob("*.rs"):
        for m in re.finditer(r'name:\s*"([^"]+)"', p.read_text()):
            names.add(m.group(1).split(" (")[0])
    return names

if __name__=="__main__":
    q=sys.argv[1]
    defs=defined_names()
    cards=search(q)
    missing=[]
    for c in cards:
        nm=c.get("name","").split(" //")[0]
        if nm not in defs:
            missing.append((nm, c.get("mana_cost",""), c.get("type_line",""), (c.get("oracle_text","") or "").replace("\n"," | ")[:120]))
    print(f"{len(cards)} cards in query, {len(missing)} missing from catalog\n")
    for nm,cost,tl,ot in missing:
        print(f"{nm} :: {cost} :: {tl}")
        print(f"    {ot}")
