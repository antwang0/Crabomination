#!/usr/bin/env python3
"""Like set_gaps.py but also prints oracle text/P-T, for implementation work.

Usage: set_gap_oracle.py <setcode> [<lo> <hi>]
The optional bounds slice the gap list (1-based, half-open `lo < n <= hi`) so a
long set can be worked in batches.
"""
import os, pathlib, ssl, sys, json, time, urllib.request, urllib.parse

# The sandbox routes HTTPS through an agent proxy whose CA bundle isn't in
# Python's default store; point at it when it exists.
_CA = "/root/.ccr/ca-bundle.crt"
_CTX = ssl.create_default_context(cafile=_CA) if os.path.exists(_CA) else None


def fetch_set(code):
    cards, url = [], "https://api.scryfall.com/cards/search?" + urllib.parse.urlencode(
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


def main():
    code = sys.argv[1]
    lo, hi = (int(sys.argv[2]), int(sys.argv[3])) if len(sys.argv) > 3 else (0, 10**6)
    blob, n = catalog_text(), 0
    for c in fetch_set(code):
        nm = c["name"]
        if nm.startswith("A-"):
            continue
        if f'"{nm}"' in blob or f'"{nm.split(" // ")[0]}"' in blob:
            continue
        n += 1
        if not (lo < n <= hi):
            continue
        pt = f" [{c.get('power')}/{c.get('toughness')}]" if c.get("power") is not None else ""
        loy = f" <{c.get('loyalty')}>" if c.get("loyalty") else ""
        print(f"=== {nm} === {c.get('mana_cost', '')} | {c.get('type_line', '')}{pt}{loy} | {c.get('rarity', '')}")
        print((c.get("oracle_text", "") or "").strip())
        print()


if __name__ == "__main__":
    main()
