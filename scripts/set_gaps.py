#!/usr/bin/env python3
"""List cards in a Scryfall set not present in the catalog.

Usage: set_gaps.py <setcode> [<setcode> ...]
"""
import os, pathlib, ssl, sys, json, time, urllib.request, urllib.parse

# The sandbox routes HTTPS through an agent proxy whose CA bundle isn't in
# Python's default store; point at it when it exists.
_CA = "/root/.ccr/ca-bundle.crt"
_CTX = ssl.create_default_context(cafile=_CA) if os.path.exists(_CA) else None


def fetch_set(code):
    cards = []
    url = "https://api.scryfall.com/cards/search?" + urllib.parse.urlencode(
        {"q": f"set:{code} -is:reprint", "unique": "cards", "order": "name"}
    )
    while url:
        req = urllib.request.Request(
            url, headers={"User-Agent": "crab/1.0", "Accept": "application/json"}
        )
        with urllib.request.urlopen(req, timeout=30, context=_CTX) as r:
            d = json.load(r)
        cards += d.get("data", [])
        url = d.get("next_page")
        time.sleep(0.1)
    return cards


def catalog_text():
    """Every catalog source concatenated once — a per-card `grep -r` over the
    tree costs minutes on a full set."""
    root = pathlib.Path(__file__).resolve().parent.parent / "crabomination_catalog" / "src"
    return "\n".join(p.read_text(encoding="utf-8", errors="replace") for p in root.rglob("*.rs"))


def main(codes):
    blob = catalog_text()
    for code in codes:
        if len(codes) > 1:
            print(f"### {code}")
        for c in fetch_set(code):
            nm = c["name"]
            if nm.startswith("A-"):  # Alchemy rebalances
                continue
            if f'"{nm}"' in blob or f'"{nm.split(" // ")[0]}"' in blob:
                continue
            print(f"{nm} | {c.get('mana_cost', '')} | {c.get('type_line', '')} | {c.get('rarity', '')}")


if __name__ == "__main__":
    main(sys.argv[1:])
