#!/usr/bin/env python3
"""Fetch a list of card definitions from Scryfall and merge into the
local cache (`scripts/.scryfall_cache.json`). Reuses verify_cards.py's
ScryfallCache so subsequent runs of the verifier benefit too.

Usage:
    python3 scripts/fetch_cards.py "Card 1" "Card 2" ...

Prints a one-line summary per card with cost / type / oracle preview.
"""
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "scripts"))
from verify_cards import ScryfallCache, scryfall_get, DEFAULT_CACHE  # noqa: E402


def main(names):
    cache = ScryfallCache(DEFAULT_CACHE)
    for name in names:
        data = scryfall_get(name, cache)
        if data is None:
            print(f"NOT FOUND: {name}")
            continue
        cost = data.get("mana_cost", "")
        tline = data.get("type_line", "")
        otext = (data.get("oracle_text", "") or "").replace("\n", " | ")[:200]
        pt = ""
        if data.get("power") is not None:
            pt = f" {data.get('power')}/{data.get('toughness')}"
        print(f"{name}: {cost} {tline}{pt}")
        print(f"  {otext}")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(__doc__, file=sys.stderr)
        sys.exit(1)
    main(sys.argv[1:])
