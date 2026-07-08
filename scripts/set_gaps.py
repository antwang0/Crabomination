#!/usr/bin/env python3
"""List cards in a Scryfall set not present in the catalog. Usage: set_gaps.py <setcode>"""
import sys, json, time, urllib.request, urllib.parse, subprocess, re
def fetch_set(code):
    cards=[]; url=f"https://api.scryfall.com/cards/search?"+urllib.parse.urlencode({"q":f"set:{code} -is:reprint","unique":"cards","order":"name"})
    while url:
        req=urllib.request.Request(url,headers={"User-Agent":"crab/1.0","Accept":"application/json"})
        with urllib.request.urlopen(req,timeout=15) as r: d=json.load(r)
        cards+=d.get("data",[]); url=d.get("next_page"); time.sleep(0.1)
    return cards
def in_catalog(name):
    return bool(subprocess.run(["grep","-rqF",f'"{name}"',"crabomination_catalog/src"]).returncode==0)
code=sys.argv[1]
for c in fetch_set(code):
    nm=c["name"]
    if nm.startswith("A-"): continue  # Alchemy rebalances
    if not in_catalog(nm) and not in_catalog(nm.split(" // ")[0]):
        tl=c.get("type_line","");cost=c.get("mana_cost","")
        print(f"{c['name']} | {cost} | {tl} | {c.get('rarity','')}")
