#!/usr/bin/env python3
"""
verify_cards.py — compare our card definitions against Scryfall.

Usage:
    python scripts/verify_cards.py [--card "Card Name"] [--json cards.json]

By default it builds the dump_cards binary, runs it to get our definitions,
then queries Scryfall for each card and reports any mismatches.  Scryfall
responses are cached in scripts/.scryfall_cache.json so subsequent runs skip
the network entirely for cards already fetched.

Flags:
  --card NAME      Only verify a single named card.
  --json FILE      Read card definitions from a JSON file instead of building.
  --dump FILE      Write the dumped card JSON to FILE (for caching).
  --cache FILE     Scryfall response cache (default: scripts/.scryfall_cache.json).
  --no-cache       Ignore the cache and always hit Scryfall.
  --quiet          Only print cards with discrepancies.
"""

import argparse
import json
import subprocess
import sys
import time
import urllib.request
import urllib.error
import urllib.parse
from pathlib import Path
from typing import Optional

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_CACHE = Path(__file__).resolve().parent / ".scryfall_cache.json"
SCRYFALL_DELAY = 0.5   # seconds between requests; Scryfall asks ≤10 req/s but bursts cause 429
SCRYFALL_RETRY_WAIT = 5.0  # seconds to wait after a 429

# Sentinel stored in the cache for 404 responses.
_NOT_FOUND = "__not_found__"


# ── Cache ─────────────────────────────────────────────────────────────────────

class ScryfallCache:
    """Persist Scryfall responses keyed by exact card name.

    Hits (card data) and misses (404) are both cached so re-runs never
    re-query cards we've already looked up.  The file is written after
    each new fetch so progress survives interruption.
    """

    def __init__(self, path: Path):
        self._path = path
        self._data: dict = {}
        if path.exists():
            try:
                self._data = json.loads(path.read_text(encoding="utf-8"))
            except (json.JSONDecodeError, OSError):
                pass  # corrupt cache → start fresh

    def get(self, name: str):
        """Return cached value, or raise KeyError if not cached."""
        raw = self._data[name]  # raises KeyError on miss
        return None if raw == _NOT_FOUND else raw

    def set(self, name: str, value: Optional[dict]) -> None:
        self._data[name] = _NOT_FOUND if value is None else value
        try:
            self._path.write_text(
                json.dumps(self._data, indent=2, ensure_ascii=False),
                encoding="utf-8",
            )
        except OSError as exc:
            print(f"  [warning: could not write cache: {exc}]", file=sys.stderr)

    def __contains__(self, name: str) -> bool:
        return name in self._data


# ── Scryfall helpers ──────────────────────────────────────────────────────────

def scryfall_fetch(name: str) -> Optional[dict]:
    """Fetch a card by exact name from Scryfall (no cache). Returns None on 404."""
    url = "https://api.scryfall.com/cards/named?" + urllib.parse.urlencode({"exact": name})
    req = urllib.request.Request(url, headers={
        "User-Agent": "crabomination-verify/1.0 (contact: dev)",
        "Accept": "application/json",
    })
    for attempt in range(4):
        try:
            with urllib.request.urlopen(req, timeout=10) as resp:
                return json.loads(resp.read())
        except urllib.error.HTTPError as e:
            if e.code == 404:
                return None
            if e.code == 429:
                wait = SCRYFALL_RETRY_WAIT * (2 ** attempt)
                print(f"  [rate limited, waiting {wait:.0f}s]", flush=True)
                time.sleep(wait)
                continue
            raise
    raise RuntimeError(f"Still rate-limited after retries: {name}")


def scryfall_get(name: str, cache: Optional[ScryfallCache]) -> Optional[dict]:
    """Return Scryfall card data, using cache when available."""
    if cache is not None and name in cache:
        return cache.get(name)
    time.sleep(SCRYFALL_DELAY)
    result = scryfall_fetch(name)
    if cache is not None:
        cache.set(name, result)
    return result


def scryfall_cmc(card: dict) -> float:
    return card.get("cmc", 0.0)


def scryfall_types(card: dict) -> tuple[list[str], list[str], list[str]]:
    """Parse type_line into (supertypes, card_types, subtypes)."""
    type_line: str = card.get("type_line", "")
    # Handle double-faced cards: take the front face type line.
    type_line = type_line.split("//")[0].strip()

    if "—" in type_line:
        left, right = type_line.split("—", 1)
        sub_parts = [s.strip() for s in right.split()]
    else:
        left, sub_parts = type_line, []

    words = left.split()
    known_supers = {"Basic", "Legendary", "Snow", "World", "Elite", "Ongoing"}
    known_types = {"Land", "Creature", "Artifact", "Enchantment", "Planeswalker",
                   "Battle", "Instant", "Sorcery", "Tribal", "Kindred"}
    supers, types = [], []
    for w in words:
        if w in known_supers:
            supers.append(w)
        elif w in known_types:
            types.append(w)

    return supers, types, sub_parts


def scryfall_keywords(card: dict) -> list[str]:
    return [kw.lower() for kw in card.get("keywords", [])]


def scryfall_pt(card: dict):
    return card.get("power"), card.get("toughness")


def scryfall_loyalty(card: dict) -> Optional[int]:
    raw = card.get("loyalty")
    if raw is None:
        return None
    try:
        return int(raw)
    except ValueError:
        return None  # loyalty can be "X"


# ── Comparison ────────────────────────────────────────────────────────────────

def compare(local: dict, sf: dict) -> list[str]:
    """Return a list of human-readable discrepancy strings."""
    issues = []

    # CMC
    sf_cmc = scryfall_cmc(sf)
    local_cmc = local["cmc"]
    if local_cmc != sf_cmc:
        issues.append(f"CMC: ours={local_cmc}, scryfall={sf_cmc}")

    # Card types
    sf_supers, sf_types, sf_subs = scryfall_types(sf)
    # Supertypes
    our_supers = set(local["supertypes"])
    sf_supers_set = set(sf_supers)
    # Normalize: Scryfall "Legendary" = ours "Legendary"
    missing_supers = our_supers - sf_supers_set
    extra_supers = sf_supers_set - our_supers
    if missing_supers:
        issues.append(f"Supertypes we have but Scryfall doesn't: {sorted(missing_supers)}")
    if extra_supers:
        issues.append(f"Supertypes Scryfall has but we don't: {sorted(extra_supers)}")

    # Card types (normalize Tribal → Kindred)
    our_types = set(local["card_types"])
    sf_types_norm = {("Kindred" if t == "Tribal" else t) for t in sf_types}
    missing_types = our_types - sf_types_norm
    extra_types = sf_types_norm - our_types
    if missing_types:
        issues.append(f"Card types we have but Scryfall doesn't: {sorted(missing_types)}")
    if extra_types:
        issues.append(f"Card types Scryfall has but we don't: {sorted(extra_types)}")

    # Power / toughness (only for creatures)
    if "Creature" in our_types:
        sf_power, sf_tough = scryfall_pt(sf)
        our_power = local.get("power")
        our_tough = local.get("toughness")
        if sf_power is not None:
            try:
                sf_power_int = int(sf_power)
                if our_power != sf_power_int:
                    issues.append(f"Power: ours={our_power}, scryfall={sf_power}")
            except ValueError:
                pass  # skip "*" power
        if sf_tough is not None:
            try:
                sf_tough_int = int(sf_tough)
                if our_tough != sf_tough_int:
                    issues.append(f"Toughness: ours={our_tough}, scryfall={sf_tough}")
            except ValueError:
                pass

    # Base loyalty (planeswalkers)
    if "Planeswalker" in our_types:
        sf_loyalty = scryfall_loyalty(sf)
        our_loyalty = local.get("base_loyalty")
        if sf_loyalty is not None and our_loyalty != sf_loyalty:
            issues.append(f"Base loyalty: ours={our_loyalty}, scryfall={sf_loyalty}")

    # Keywords (best-effort: we compare what Scryfall tracks)
    our_kws = {k.lower() for k in local["keywords"]}
    sf_kws = set(scryfall_keywords(sf))

    # Scryfall doesn't track all keyword abilities the same way; skip ones
    # that differ in naming convention (e.g. "protection" needs color qualifier).
    skip_kws = {"protection", "ward", "cycling", "kicker", "flashback", "equip",
                "echo", "cumulative upkeep", "morph", "megamorph", "fortify",
                "dredge", "annihilator", "regenerate"}
    our_kws_check = our_kws - skip_kws
    sf_kws_check = sf_kws - skip_kws

    # Normalize naming differences
    normalize = {
        "first strike": "first strike",
        "double strike": "double strike",
    }
    our_kws_check = {normalize.get(k, k) for k in our_kws_check}
    sf_kws_check = {normalize.get(k, k) for k in sf_kws_check}

    missing_kws = our_kws_check - sf_kws_check
    extra_kws = sf_kws_check - our_kws_check
    if missing_kws:
        issues.append(f"Keywords we have but Scryfall doesn't: {sorted(missing_kws)}")
    if extra_kws:
        issues.append(f"Keywords Scryfall has but we don't: {sorted(extra_kws)}")

    return issues


# ── Build / load card data ────────────────────────────────────────────────────

def build_and_dump() -> list[dict]:
    """Build the dump_cards binary and return its JSON output."""
    print("Building dump_cards...", flush=True)
    build = subprocess.run(
        ["cargo", "build", "--bin", "dump_cards", "--quiet"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
    )
    if build.returncode != 0:
        print("Build failed:\n" + build.stderr, file=sys.stderr)
        sys.exit(1)

    run = subprocess.run(
        ["cargo", "run", "--bin", "dump_cards", "--quiet"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
    )
    if run.returncode != 0:
        print("Run failed:\n" + run.stderr, file=sys.stderr)
        sys.exit(1)

    return json.loads(run.stdout)


# ── Main ──────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Verify card definitions against Scryfall")
    parser.add_argument("--card", metavar="NAME", help="Only verify this card")
    parser.add_argument("--json", metavar="FILE", help="Read definitions from a JSON file")
    parser.add_argument("--dump", metavar="FILE", help="Write dumped definitions to FILE")
    parser.add_argument("--cache", metavar="FILE", default=str(DEFAULT_CACHE),
                        help="Scryfall response cache (default: scripts/.scryfall_cache.json)")
    parser.add_argument("--no-cache", action="store_true", help="Ignore the cache")
    parser.add_argument("--quiet", action="store_true", help="Only print cards with issues")
    parser.add_argument("--query", metavar="NAME", action="append", default=[],
                        help="Print Scryfall data for a card by name (no comparison). "
                             "May be repeated. Skips the cargo build / dump_cards run.")
    args = parser.parse_args()

    if args.query:
        cache = None if args.no_cache else ScryfallCache(Path(args.cache))
        if cache is not None:
            print(f"Cache: {args.cache} ({len(cache._data)} entries)")
        for name in args.query:
            try:
                sf = scryfall_get(name, cache)
            except Exception as exc:
                print(f"[ERROR] {name}: {exc}")
                continue
            if sf is None:
                print(f"[NOT FOUND] {name}")
                continue
            print(f"=== {sf.get('name')} ===")
            print(f"  set: {sf.get('set')!r}  released: {sf.get('released_at')!r}")
            print(f"  mana_cost: {sf.get('mana_cost')!r}  cmc: {sf.get('cmc')!r}")
            print(f"  type_line: {sf.get('type_line')!r}")
            pt = scryfall_pt(sf) if "Creature" in sf.get('type_line', '') else (None, None)
            if pt[0] is not None:
                print(f"  P/T: {pt[0]}/{pt[1]}")
            if sf.get('loyalty'):
                print(f"  loyalty: {sf.get('loyalty')!r}")
            print(f"  oracle_text:\n{sf.get('oracle_text', '').strip()}")
            print()
        return

    cache: Optional[ScryfallCache] = None
    if not args.no_cache:
        cache = ScryfallCache(Path(args.cache))
        print(f"Cache: {args.cache} ({len(cache._data)} entries)")

    if args.json:
        cards = json.loads(Path(args.json).read_text())
    else:
        cards = build_and_dump()

    if args.dump:
        Path(args.dump).write_text(json.dumps(cards, indent=2))
        print(f"Wrote {len(cards)} definitions to {args.dump}")

    if args.card:
        cards = [c for c in cards if c["name"].lower() == args.card.lower()]
        if not cards:
            print(f"Card not found in catalog: {args.card!r}", file=sys.stderr)
            sys.exit(1)

    # Skip purely engine-internal tokens that don't exist on Scryfall.
    skip_names = {"Clue", "Treasure", "Food", "Blood"}

    total = 0
    ok = 0
    not_found = 0
    errors = 0

    print(f"Verifying {len(cards)} card(s) against Scryfall...\n")

    for card in cards:
        name = card["name"]
        if name in skip_names:
            continue

        total += 1

        try:
            sf = scryfall_get(name, cache)
        except Exception as exc:
            print(f"[ERROR] {name}: {exc}")
            errors += 1
            continue

        if sf is None:
            print(f"[NOT FOUND] {name}")
            not_found += 1
            continue

        issues = compare(card, sf)

        if issues:
            errors += 1
            print(f"[MISMATCH] {name}")
            for issue in issues:
                print(f"  - {issue}")
        else:
            ok += 1
            if not args.quiet:
                print(f"[OK] {name}")

    print(f"\nResults: {ok} ok, {not_found} not found on Scryfall, {errors} mismatches/errors "
          f"(out of {total} checked)")


if __name__ == "__main__":
    main()
