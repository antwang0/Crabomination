#!/usr/bin/env python3
"""Does every "on the bottom … in a random order" card actually randomize?

`Effect::LookPickToHand`'s `LookPick` carries `rest_bottom_random`. With it
clear the unpicked cards go to the bottom **in the order they were revealed**,
which is not what the card says and is worse than cosmetic here: the player
just saw those cards, so a deterministic bottom is known information, and a
self-play net can learn it. With it set the batch is shuffled first.

The flag is `false` by default and 38 of the 135 `LookPick` cards in the
catalog printed the clause without it (2026-08-29). This is the check that
keeps that from happening again: it reads each factory's `name:` out of the
source, looks the oracle text up in `scripts/.scryfall_cache.json`, and
compares the clause against the flag in both directions.

    python3 scripts/audit_bottom_random.py            # findings only
    python3 scripts/audit_bottom_random.py --check    # exit 1 on any finding

**Cards the cache has no top-level oracle text for are skipped, not flagged.**
A split or MDFC card carries its text on the faces, so an empty string here
means "unknown", and treating it as "does not say random order" reported three
correct cards as findings the first time this was run by hand.
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CATALOG = ROOT / "crabomination_catalog" / "src"
CACHE = ROOT / "scripts" / ".scryfall_cache.json"


def oracle_by_name() -> dict[str, str]:
    raw = json.loads(CACHE.read_text())
    return {
        k.lower(): (v.get("oracle_text") or "")
        for k, v in raw.items()
        if isinstance(v, dict)
    }


def _balanced(src: str, open_at: int) -> int:
    """Index of the `}` closing the `{` at `open_at`."""
    depth = 0
    for j in range(open_at, len(src)):
        if src[j] == "{":
            depth += 1
        elif src[j] == "}":
            depth -= 1
            if depth == 0:
                return j
    raise ValueError("unbalanced braces")


def wants_random(oracle: str) -> bool:
    """True when some sentence puts the rest on the *bottom* in a random order.

    Both halves are needed: a card can say "random" about something else
    entirely, and one of the six planeswalkers here has three abilities.
    """
    return any(
        "random order" in s and "bottom" in s
        for s in re.split(r"(?<=[.])\s+", oracle)
    )


def main() -> int:
    check = "--check" in sys.argv
    oracle = oracle_by_name()
    missing, spurious, seen, skipped = [], [], 0, 0
    for f in sorted(CATALOG.rglob("*.rs")):
        src = f.read_text()
        for m in re.finditer(r"pub fn (\w+)\(\) -> CardDefinition \{", src):
            start = m.end() - 1
            body = src[start : _balanced(src, start) + 1]
            if "LookPick" not in body:
                continue
            nm = re.search(r'name:\s*"([^"]+)"', body)
            if not nm:
                continue
            text = oracle.get(nm.group(1).lower())
            if not text:
                skipped += 1
                continue
            seen += 1
            has = "rest_bottom_random: true" in body
            if wants_random(text) and not has:
                missing.append((nm.group(1), f.name))
            elif has and not wants_random(text):
                spurious.append((nm.group(1), f.name))

    print(f"{seen} `LookPick` cards cross-referenced ({skipped} skipped: no cached text)")
    for label, rows in (
        ("prints the clause, flag NOT set", missing),
        ("flag set, clause NOT printed", spurious),
    ):
        print(f"\n{label}: {len(rows)}")
        for name, where in rows:
            print(f"    {name:<34} {where}")
    if check and (missing or spurious):
        print(f"\n{len(missing) + len(spurious)} findings", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
