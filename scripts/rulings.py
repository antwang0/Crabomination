#!/usr/bin/env python3
"""Print Scryfall's rulings for cards (live API; the cache has none).

  scripts/rulings.py "Arcane Lighthouse" "Mathas, Fiend Seeker"
"""
import json, sys, time, urllib.parse, urllib.request

def get(url):
    req = urllib.request.Request(url, headers={"User-Agent": "crabomination/1.0", "Accept": "application/json"})
    with urllib.request.urlopen(req) as r:
        return json.load(r)

for name in sys.argv[1:]:
    card = get("https://api.scryfall.com/cards/named?exact=" + urllib.parse.quote(name))
    print(f"=== {card['name']} ===")
    for r in get(card["rulings_uri"])["data"]:
        print(f"  [{r['published_at']}] {r['comment']}")
    time.sleep(0.1)
