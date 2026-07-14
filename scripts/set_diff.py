#!/usr/bin/env python3
"""Fetch a Scryfall set's card list and report which cards are NOT yet present
in the Rust catalog (by exact `name: "..."` match). Usage:
    python3 scripts/set_diff.py tdm            # set code
    python3 scripts/set_diff.py tdm --oracle   # also print oracle text
Reads unique English cards, skips basic lands. Respects the local cache dir
only for /named; set search hits the live API (paginated)."""
import json, sys, time, urllib.request, urllib.parse
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CATALOG = REPO / "crabomination_catalog" / "src"


def catalog_blob() -> str:
    # One big text blob of every catalog source file. Presence is tested by
    # substring `"<name>"`, which is robust to cards defined via helper
    # factories that take the display name as a positional arg
    # (`dual_land_with("Wind-Scarred Crag", ...)`).
    parts = []
    for p in CATALOG.rglob("*.rs"):
        parts.append(p.read_text(errors="ignore"))
    return "\n".join(parts)


def is_present(name: str, blob: str) -> bool:
    return f'"{name}"' in blob


def fetch_set(code: str) -> list[dict]:
    cards = []
    url = "https://api.scryfall.com/cards/search?" + urllib.parse.urlencode(
        {"q": f"set:{code} -is:extra unique:cards", "order": "set"}
    )
    while url:
        req = urllib.request.Request(url, headers={"User-Agent": "crabomination/1.0", "Accept": "application/json"})
        with urllib.request.urlopen(req, timeout=20) as resp:
            data = json.load(resp)
        cards.extend(data.get("data", []))
        url = data.get("next_page")
        time.sleep(0.1)
    return cards


def main():
    code = sys.argv[1]
    show_oracle = "--oracle" in sys.argv
    blob = catalog_blob()
    cards = fetch_set(code)
    missing = []
    for c in cards:
        if c.get("type_line", "").startswith("Basic Land"):
            continue
        # Present if the full name OR (for DFC/split) either face name appears.
        names = [c["name"]] + [f.get("name", "") for f in c.get("card_faces", []) or []]
        if any(is_present(n, blob) for n in names if n):
            continue
        missing.append(c)
    print(f"{code}: {len(cards)} cards, {len(missing)} missing from catalog\n")
    for c in missing:
        pt = f" {c.get('power')}/{c.get('toughness')}" if c.get("power") is not None else ""
        print(f"MISS {c['name']}: {c.get('mana_cost','')} {c.get('type_line','')}{pt}")
        if show_oracle:
            ot = (c.get("oracle_text") or "").replace("\n", " | ")
            print(f"     {ot}")
            for f in c.get("card_faces", []) or []:
                print(f"     [{f.get('name')}] {(f.get('oracle_text') or '').replace(chr(10),' | ')}")


if __name__ == "__main__":
    main()
