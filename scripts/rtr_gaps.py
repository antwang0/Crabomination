#!/usr/bin/env python3
"""List RTR (Return to Ravnica) cards not yet implemented anywhere in the catalog.

Diffs `set:rtr` from Scryfall against every card-name string literal in
`crabomination_catalog/src`. Writes misses to `scripts/.rtr_missing.json`.
Pass a set code as argv[1] to target another set.
"""
import json, sys, time, urllib.request, urllib.parse
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SET = sys.argv[1] if len(sys.argv) > 1 else "rtr"


def get(url):
    req = urllib.request.Request(
        url, headers={"User-Agent": "crabomination-verify/1.0", "Accept": "application/json"}
    )
    with urllib.request.urlopen(req, timeout=15) as r:
        return json.load(r)


def main():
    catalog = "\n".join(p.read_text() for p in (REPO / "crabomination_catalog/src").rglob("*.rs"))
    cards = []
    q = urllib.parse.quote(f"set:{SET} -is:digital")
    url = f"https://api.scryfall.com/cards/search?q={q}&unique=cards&order=name"
    while url:
        d = get(url)
        cards.extend(d["data"])
        url = d.get("next_page")
        time.sleep(0.15)
    missing = [c for c in cards if f'"{c["name"]}"' not in catalog and not c.get("card_faces")]
    print(f"set:{SET} cards={len(cards)}  single-faced missing={len(missing)}")
    json.dump(missing, open(REPO / f"scripts/.{SET}_missing.json", "w"))
    for c in sorted(missing, key=lambda c: len(c.get("oracle_text", "") or "")):
        ot = (c.get("oracle_text", "") or "").replace("\n", " | ")
        pt = f" {c['power']}/{c['toughness']}" if c.get("power") is not None else ""
        print(f"{c['name']} | {c.get('mana_cost','')} | {c.get('type_line','')}{pt}\n   {ot}")


if __name__ == "__main__":
    main()
