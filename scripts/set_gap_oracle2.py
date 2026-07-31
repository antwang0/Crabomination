#!/usr/bin/env python3
"""Dump oracle text for every card of a set missing from the catalog.

Usage: set_gap_oracle2.py <setcode>
Same gap detection as `set_gaps.py`, but prints full oracle text so a card can
be implemented without a second lookup.
"""
import sys, json, time, urllib.request, urllib.parse, subprocess

def fetch_set(code):
    cards, url = [], "https://api.scryfall.com/cards/search?" + urllib.parse.urlencode(
        {"q": f"set:{code} -is:reprint", "unique": "cards", "order": "name"})
    while url:
        req = urllib.request.Request(url, headers={"User-Agent": "crab/1.0", "Accept": "application/json"})
        with urllib.request.urlopen(req, timeout=20) as r:
            d = json.load(r)
        cards += d.get("data", []); url = d.get("next_page"); time.sleep(0.1)
    return cards

def in_catalog(name):
    return subprocess.run(["grep", "-rqF", f'"{name}"', "crabomination_catalog/src"]).returncode == 0

for c in fetch_set(sys.argv[1]):
    nm = c["name"]
    if nm.startswith("A-") or in_catalog(nm) or in_catalog(nm.split(" // ")[0]):
        continue
    pt = f" [{c.get('power')}/{c.get('toughness')}]" if c.get("power") is not None else ""
    loy = f" <{c.get('loyalty')}>" if c.get("loyalty") else ""
    print(f"=== {nm} === {c.get('mana_cost','')} | {c.get('type_line','')}{pt}{loy}")
    text = c.get("oracle_text")
    if text is None:
        text = "\n// ".join(f["name"] + " | " + f.get("mana_cost", "") + " | "
                            + f.get("type_line", "") + "\n" + (f.get("oracle_text") or "")
                            for f in c.get("card_faces", []))
    print(text or "")
    print()
