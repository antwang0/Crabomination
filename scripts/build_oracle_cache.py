#!/usr/bin/env python3
"""Build an offline, name-keyed Scryfall cache covering *every* oracle card.

The card-implementation routine runs in a sandbox where `api.scryfall.com`
is firewalled, so it can only consult the committed cache at
`scripts/.scryfall_cache.json`. That cache historically held only the few
hundred cards already in the catalog, which meant the routine could not look
up — and therefore could not faithfully implement — any *new* card.

This script downloads Scryfall's `oracle_cards` bulk export (one entry per
unique card, no duplicate printings), trims each card down to the fields the
routine and `verify_cards.py` actually read, and merges the result into
`.scryfall_cache.json`. Existing, richer (full-object) entries are preserved;
only missing cards are added. Re-run it whenever you want to refresh against a
newer Scryfall bulk.

Usage:
    python scripts/build_oracle_cache.py                  # download + merge
    python scripts/build_oracle_cache.py --bulk FILE.json # use a local bulk
"""
import argparse
import json
import time
import unicodedata
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CACHE = Path(__file__).resolve().parent / ".scryfall_cache.json"
BULK_INDEX = "https://api.scryfall.com/bulk-data"
HEADERS = {"User-Agent": "crabomination-fetch/1.0", "Accept": "application/json"}

# Fields we keep per card. Enough to implement a card (cost, types, oracle
# text, P/T, loyalty, keywords, colors) and to verify it (cmc, type_line, …).
FIELDS = ["name", "mana_cost", "cmc", "type_line", "oracle_text", "power",
          "toughness", "loyalty", "keywords", "colors", "color_identity",
          "produced_mana", "layout", "card_faces"]
FACE_FIELDS = ["name", "mana_cost", "type_line", "oracle_text", "power",
               "toughness", "loyalty", "colors", "loyalty"]
# Layouts that aren't real, castable cards (art, tokens, emblems, etc.).
SKIP_LAYOUTS = {"art_series", "double_faced_token", "emblem", "token",
                "scheme", "planar", "vanguard"}
_NOT_FOUND = "__not_found__"


def _get(url: str) -> bytes:
    req = urllib.request.Request(url, headers=HEADERS)
    return urllib.request.urlopen(req, timeout=180).read()


def download_bulk() -> list:
    print("Resolving oracle_cards bulk URI…", flush=True)
    index = json.loads(_get(BULK_INDEX))
    entry = next(x for x in index["data"] if x["type"] == "oracle_cards")
    print(f"Downloading {entry['download_uri']} "
          f"({entry['size'] / 1e6:.0f} MB)…", flush=True)
    t = time.time()
    data = _get(entry["download_uri"])
    print(f"  …{len(data) / 1e6:.0f} MB in {time.time() - t:.0f}s", flush=True)
    return json.loads(data)


def _canonical_name(name: str) -> str:
    """Best-effort repair of legacy cache keys so they line up with Scryfall's
    NFC names. Fixes both decomposed (NFD) accents and double-encoded mojibake
    (UTF-8 bytes stored as Latin-1, e.g. "DandÃ¢n" → "Dandân")."""
    try:
        repaired = name.encode("latin-1").decode("utf-8")
        # Only accept the repair if it actually removed mojibake artifacts.
        if "Ã" in name or "Â" in name:
            name = repaired
    except (UnicodeEncodeError, UnicodeDecodeError):
        pass
    return unicodedata.normalize("NFC", name)


def trim(card: dict) -> dict:
    out = {k: card[k] for k in FIELDS if k in card}
    if "card_faces" in out:
        out["card_faces"] = [
            {k: f[k] for k in FACE_FIELDS if k in f} for f in out["card_faces"]
        ]
    return out


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--bulk", metavar="FILE",
                    help="use a local oracle-cards bulk JSON instead of downloading")
    args = ap.parse_args()

    cards = json.loads(Path(args.bulk).read_text(encoding="utf-8")) \
        if args.bulk else download_bulk()
    print(f"Bulk objects: {len(cards)}")

    existing = {}
    if CACHE.exists():
        existing = json.loads(CACHE.read_text(encoding="utf-8"))
    print(f"Existing cache entries: {len(existing)}")

    merged = {}
    real = []
    for c in cards:
        if c.get("layout") in SKIP_LAYOUTS:
            continue
        merged[c["name"]] = trim(c)
        real.append(c)
    # Multi-face cards (adventure / transform / MDFC / split / flip) are keyed
    # under the combined "Front // Back" name in the bulk, but the catalog —
    # and therefore the routine — looks them up by a single face name. Register
    # each face name as an alias, without clobbering a standalone card that
    # already owns that name.
    for c in real:
        for face in c.get("card_faces", []):
            fname = face.get("name")
            if fname and fname not in merged:
                merged[fname] = trim(c)

    for name, value in existing.items():
        name = _canonical_name(name)
        # Keep full, real entries we already curated; drop stale not-found
        # sentinels that the oracle bulk now resolves.
        if isinstance(value, dict):
            merged[name] = value
        elif name not in merged:
            merged[name] = value  # genuine miss not in the bulk

    added = len(merged) - len(existing)
    CACHE.write_text(json.dumps(merged, ensure_ascii=False, indent=2),
                     encoding="utf-8")
    size_mb = CACHE.stat().st_size / 1e6
    print(f"Wrote {len(merged)} entries (+{added} new) to {CACHE.name} "
          f"({size_mb:.1f} MB)")


if __name__ == "__main__":
    main()
