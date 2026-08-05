#!/usr/bin/env python3
"""Print Unfinity Attraction cards with their lit-up numbers and oracle text."""
import os, ssl, json, urllib.request, urllib.parse, time, sys

_CA = "/root/.ccr/ca-bundle.crt"
_CTX = ssl.create_default_context(cafile=_CA) if os.path.exists(_CA) else None

url = "https://api.scryfall.com/cards/search?" + urllib.parse.urlencode(
    {"q": "t:attraction", "unique": "cards", "order": "name"}
)
seen = {}
while url:
    req = urllib.request.Request(url, headers={"User-Agent": "crab/1.0", "Accept": "application/json"})
    with urllib.request.urlopen(req, timeout=30, context=_CTX) as r:
        d = json.load(r)
    for c in d.get("data", []):
        seen.setdefault(c["name"], c)
    url = d.get("next_page")
    time.sleep(0.1)
for name, c in sorted(seen.items()):
    print(f"=== {name} | {c.get('type_line','')} | lights={c.get('attraction_lights')} | {c.get('rarity')}")
    print((c.get('oracle_text') or '').strip())
    print()
print(f"total {len(seen)}")
