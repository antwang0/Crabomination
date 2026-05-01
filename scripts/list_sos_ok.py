#!/usr/bin/env python3
"""List all SOS cards with ✅ status from STRIXHAVEN2.md, grouped by section."""

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
DOC = REPO / "STRIXHAVEN2.md"

SECTION_RE = re.compile(r"^##\s+(.+?)\s*$")
ROW_RE = re.compile(
    r"^\|\s*(?P<name>.+?)\s*"
    r"\|\s*(?P<mana>.*?)\s*"
    r"\|\s*(?P<typ>.*?)\s*"
    r"\|\s*(?P<pt>.*?)\s*"
    r"\|\s*(?P<oracle>.*?)\s*"
    r"\|\s*(?P<status>.+?)\s*"
    r"\|\s*(?P<notes>.+?)\s*"
    r"\|\s*$"
)

SOS_SECTIONS = [
    "White", "Blue", "Black", "Red", "Green",
    "Prismari (Blue-Red)", "Witherbloom (Black-Green)",
    "Silverquill (White-Black)", "Quandrix (Green-Blue)",
    "Lorehold (Red-White)", "Colorless",
]

ok_cards = {sec: [] for sec in SOS_SECTIONS}
current = None
for line in DOC.read_text(encoding="utf-8").splitlines():
    m = SECTION_RE.match(line)
    if m:
        current = m.group(1).strip()
        continue
    if current not in ok_cards:
        continue
    if not line.startswith("|") or line.startswith("|---"):
        continue
    rm = ROW_RE.match(line)
    if not rm:
        continue
    name = rm.group("name").strip()
    if name.lower() == "card":
        continue
    status = rm.group("status").strip()
    typ = rm.group("typ").strip()
    mana = rm.group("mana").strip()
    if "✅" in status:
        ok_cards[current].append((name, mana, typ))

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
total = 0
for sec in SOS_SECTIONS:
    cards = ok_cards[sec]
    total += len(cards)
    print(f"\n{sec} ({len(cards)}):")
    for name, mana, typ in cards:
        print(f"  {name}  {mana}  --  {typ}")
print(f"\nTotal OK: {total}")
