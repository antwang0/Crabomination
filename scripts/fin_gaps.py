#!/usr/bin/env python3
import json, re, time, urllib.request, urllib.parse, urllib.error
from pathlib import Path
REPO = Path(__file__).resolve().parent.parent
def slug(name):
    s = name.lower().split(" //")[0]
    return re.sub(r"[^a-z0-9]+", "_", s).strip("_")
def get(url):
    req = urllib.request.Request(url, headers={"User-Agent":"crabomination-verify/1.0","Accept":"application/json"})
    with urllib.request.urlopen(req, timeout=15) as r:
        return json.load(r)
finrs = (REPO/"crabomination_catalog/src/sets/fin.rs").read_text()
impl_fns = set(re.findall(r"^pub fn ([a-z0-9_]+)", finrs, re.M))
cards=[]
q = urllib.parse.quote("set:fin -is:digital")
url = f"https://api.scryfall.com/cards/search?q={q}&unique=cards&order=name"
while url:
    d = get(url); cards.extend(d["data"]); url = d.get("next_page"); time.sleep(0.15)
missing=[c for c in cards if slug(c["name"]) not in impl_fns and f'"{c["name"]}"' not in finrs]
print(f"total={len(cards)} impl_fns={len(impl_fns)} missing={len(missing)}")
json.dump(missing, open(REPO/"scripts/.fin_missing.json","w"))
